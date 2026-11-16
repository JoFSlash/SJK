//! Incremental legacy table updates around the asynchronous initial load.

use super::*;
use sjk_protocol::ConfigStringDirty;

pub(super) type DeferredSoundChanges = Option<(ConfigStringDirty, GameState)>;

pub(super) struct SoundTableRefresh {
    applied: crate::config_string_refresh::ConfigStringRefresh,
    pending: ConfigStringDirty,
    latest: Option<GameState>,
}

impl SoundTableRefresh {
    /// Retain each changed slot while the initial worker is still constructing tables.
    fn record(&mut self, index: usize, game: &GameState, loading: bool) -> bool {
        if !self
            .applied
            .accept(index, game.config_string(index).unwrap_or_default())
        {
            return false;
        }
        if loading {
            self.pending.mark(index);
            self.latest = Some(game.clone());
        }
        !loading
    }

    pub(super) fn pending_snapshot(&self) -> DeferredSoundChanges {
        self.latest
            .as_ref()
            .map(|game| (self.pending.clone(), game.clone()))
    }

    pub(super) fn new(game: &GameState) -> Self {
        Self {
            applied: crate::config_string_refresh::ConfigStringRefresh::new(Some(game)),
            pending: ConfigStringDirty::default(),
            latest: None,
        }
    }
}

impl GameAudio {
    /// Apply sound notifications before observing their snapshot's first event.
    /// The session retains its bitset for the one viewer drain at frame end.
    pub(crate) fn prepare_config_strings(
        &mut self,
        changes: &ConfigStringDirty,
        game: &GameState,
        vfs: &VirtualFileSystem,
    ) {
        changes.visit(|index| {
            // CS_SERVERINFO: the flame contract follows the same detection the
            // visuals re-run each frame (a mid-map gamename change included).
            if index == 0 {
                self.flame.detected =
                    crate::actor_world_submission::force_powers::flame::detect(Some(game));
            }
            if matches!(index, 32 | 37..=292 | 811..=1066) {
                self.refresh_sound_table(index, game, vfs);
            }
        });
    }

    /// Re-intern a changed slot; repeat notifications perform no asset work.
    pub(crate) fn refresh_sound_table(
        &mut self,
        index: usize,
        game: &GameState,
        vfs: &VirtualFileSystem,
    ) {
        let Some(state) = &mut self.sound_table_refresh else {
            return;
        };
        if state.record(index, game, self.legacy.is_none()) {
            self.apply_sound_slot(index, game, vfs);
        }
    }

    pub(super) fn apply_sound_slot(
        &mut self,
        index: usize,
        game: &GameState,
        vfs: &VirtualFileSystem,
    ) {
        let Some(adapter) = &mut self.legacy else {
            return;
        };
        let handles = &mut self.handles;
        let next_handle = &mut self.next_handle;
        let output = &mut self.output;
        adapter.refresh_sound_table(index, game, vfs, |path, bytes| {
            let key = path.to_ascii_lowercase();
            if let Some(handle) = handles.get(&key) {
                return Some(*handle);
            }
            let handle = SoundHandle(*next_handle);
            *next_handle = next_handle.wrapping_add(1);
            output.decode(handle, bytes, path.rsplit('.').next().unwrap_or("wav"));
            handles.insert(key, handle);
            Some(handle)
        });
    }

    pub(super) fn apply_pending_sound_tables(&mut self) {
        let Some(state) = &mut self.sound_table_refresh else {
            return;
        };
        let Some(game) = state.latest.take() else {
            return;
        };
        let mut changes = std::mem::take(&mut state.pending);
        let Some(vfs) = self.legacy_vfs.clone() else {
            return;
        };
        changes.drain(|index| self.apply_sound_slot(index, &game, &vfs));
    }

    /// `CG_StartMusic`: an empty `CS_MUSIC` stops the map track.
    pub(crate) fn refresh_music(&mut self, game: &GameState, vfs: &VirtualFileSystem) {
        let value = game.config_string(2).unwrap_or_default();
        if self.music_value.as_deref() == Some(value) {
            return;
        }
        self.music_value = Some(value.to_vec());
        // `cgs.musicStartTime = cg.time`: a mid-level `target_play_music` starts
        // its track now, so a later resync is timed from here, not the level start.
        self.music_start_time = self
            .last_server_time
            .unwrap_or_else(|| super::music::level_start_time(game));
        if let Some(music) = assets::game_music(game) {
            self.start_music(vfs, &music);
        } else {
            self.clear_music();
        }
    }
}
