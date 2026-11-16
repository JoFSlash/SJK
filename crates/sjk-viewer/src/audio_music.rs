//! Map-music ownership and level-clock starts, after JoF EJK #103/#146/#161.
use super::*;
pub(crate) const CVAR: &str = "cg_musicSync";

/// A server's empty CS_MUSIC is authoritative silence, not a worldspawn fallback.
pub(super) fn select_music(
    game: Option<&GameState>,
    fallback: impl FnOnce() -> Option<assets::MusicSpec>,
) -> Option<assets::MusicSpec> {
    match game {
        Some(game) => assets::game_music(game),
        None => fallback(),
    }
}

/// `CS_LEVEL_START_TIME` (21, codemp/game/bg_public.h); already decoded, no wire changes.
pub(super) fn level_start_time(game: &GameState) -> i32 {
    game.config_string(21)
        .and_then(|bytes| std::str::from_utf8(bytes).ok())
        .and_then(|value| value.trim().parse::<i32>().ok())
        .unwrap_or(0)
}

/// `CG_MusicStartTime` (JoF EJK #159): a server may state the track's start as a
/// third numeric `CS_MUSIC` token, which `CG_StartMusic` never reads.
fn advertised_start_time(game: &GameState) -> Option<i32> {
    let value = std::str::from_utf8(game.config_string(2)?).ok()?;
    let token = value.split_ascii_whitespace().nth(2)?;
    let digits = token.bytes().take_while(u8::is_ascii_digit).count();
    // `atoi` on a token that starts with a digit; larger values saturate.
    (digits > 0).then(|| token[..digits].parse::<i32>().unwrap_or(i32::MAX))
}

/// Time since the current track began: the advertised start, else the last
/// `CS_MUSIC` change seen (the level start for a track set at map load).
fn elapsed_ms(snapshot: &Snapshot, game: &GameState, music_start: i32) -> u64 {
    let start = advertised_start_time(game).unwrap_or(music_start);
    (i64::from(snapshot.server_time) - i64::from(start)).max(0) as u64
}

impl GameAudio {
    pub(super) fn restore_map_music(&mut self, snapshot: &Snapshot, game: &GameState) {
        if let Some((intro, repeating)) = self.map_music {
            if self.music_sync && self.dynamic.is_none() {
                self.output.start_music_at(
                    intro,
                    repeating,
                    elapsed_ms(snapshot, game, self.music_start_time),
                );
            } else {
                self.output.start_music(intro, repeating);
            }
        } else {
            self.output.stop_music();
        }
    }

    pub(super) fn apply_music_action(
        &mut self,
        action: Option<LegacyMusicAction>,
        snapshot: &Snapshot,
        game: &GameState,
    ) {
        match action {
            Some(LegacyMusicAction::StartPrivateDuel) if self.duel_enabled => {
                if let Some(handle) = self.duel_music {
                    self.output.start_music(handle, handle);
                }
            }
            Some(LegacyMusicAction::RestoreMap) => self.restore_map_music(snapshot, game),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjk_protocol::PlayerState;
    #[test]
    fn music_selection_honors_server_silence_and_offline_worldspawn() {
        let mut game = GameState::empty_local(0);
        assert!(
            select_music(Some(&game), || panic!(
                "server silence must not use worldspawn"
            ))
            .is_none()
        );
        game.replace_config_string(2, b"music/intro music/loop".to_vec())
            .unwrap();
        let selected = select_music(Some(&game), || panic!("server owns the track")).unwrap();
        assert_eq!(selected.intro, "music/intro");
        assert_eq!(selected.repeating, "music/loop");
        assert!(select_music(None, || Some(selected)).is_some());
    }
    #[test]
    fn level_music_clock_clamps_future_start_and_preserves_long_elapsed_time() {
        let mut game = GameState::empty_local(0);
        let mut snapshot = snapshot_at(10000);
        game.replace_config_string(21, b"2000".to_vec()).unwrap();
        let start = level_start_time(&game);
        assert_eq!(elapsed_ms(&snapshot, &game, start), 8000);
        snapshot.server_time = 1000;
        assert_eq!(elapsed_ms(&snapshot, &game, start), 0);
        snapshot.server_time = i32::MAX;
        game.replace_config_string(21, i32::MIN.to_string().into_bytes())
            .unwrap();
        assert_eq!(
            elapsed_ms(&snapshot, &game, level_start_time(&game)),
            u64::from(u32::MAX)
        );
    }
    #[test]
    fn an_advertised_third_music_token_is_the_track_start() {
        let mut game = GameState::empty_local(0);
        let snapshot = snapshot_at(10000);
        for (value, start) in [
            (&b"music/intro music/loop 7000"[..], Some(7000)),
            (b"music/intro music/loop 7000abc", Some(7000)),
            (b"music/intro music/loop 99999999999", Some(i32::MAX)),
            (b"music/intro music/loop -5", None),
            (b"music/intro music/loop x7", None),
            (b"music/intro music/loop", None),
            (b"music/intro", None),
        ] {
            game.replace_config_string(2, value.to_vec()).unwrap();
            assert_eq!(advertised_start_time(&game), start, "{value:?}");
        }
        game.replace_config_string(2, b"a b 7000".to_vec()).unwrap();
        assert_eq!(elapsed_ms(&snapshot, &game, 1000), 3000);
        game.replace_config_string(2, b"a b".to_vec()).unwrap();
        assert_eq!(elapsed_ms(&snapshot, &game, 1000), 9000);
        // The tracks themselves still parse from the first two tokens.
        game.replace_config_string(2, b"music/intro music/loop 7000".to_vec())
            .unwrap();
        let spec = assets::game_music(&game).unwrap();
        assert_eq!(spec.intro, "music/intro");
        assert_eq!(spec.repeating, "music/loop");
    }
    #[test]
    fn a_mid_level_music_change_resumes_from_its_own_start() {
        let mut audio = GameAudio::with_output(AudioOutput::test_output().0);
        let mut game = GameState::empty_local(0);
        game.replace_config_string(21, b"2000".to_vec()).unwrap();
        game.replace_config_string(2, b"music/a".to_vec()).unwrap();
        audio.music_start_time = level_start_time(&game);
        audio.music_value = Some(b"music/a".to_vec());
        let vfs = VirtualFileSystem::new();
        audio.last_server_time = Some(9000);
        // The same value again (a repeated notification) keeps the start.
        audio.refresh_music(&game, &vfs);
        assert_eq!(audio.music_start_time, 2000);
        // target_play_music at 9000: the new track is timed from then.
        game.replace_config_string(2, b"music/b".to_vec()).unwrap();
        audio.refresh_music(&game, &vfs);
        assert_eq!(audio.music_start_time, 9000);
        assert_eq!(
            elapsed_ms(&snapshot_at(15000), &game, audio.music_start_time),
            6000
        );
        // Silence also restarts the reference, as EJK sets it on every change.
        audio.last_server_time = Some(16000);
        game.replace_config_string(2, Vec::new()).unwrap();
        audio.refresh_music(&game, &vfs);
        assert_eq!(audio.music_start_time, 16000);
    }
    fn snapshot_at(server_time: i32) -> Snapshot {
        Snapshot {
            message_sequence: 1,
            reliable_acknowledge: 0,
            server_commands: vec![],
            server_time,
            delta_from: None,
            flags: 0,
            area_mask: vec![],
            player: PlayerState::zero(),
            vehicle_player: None,
            entities: vec![],
            consumed_bits: 0,
        }
    }
}

#[cfg(test)]
mod restoration_tests {
    use super::*;
    use sjk_protocol::PlayerState;
    #[test]
    fn ending_a_duel_on_a_musicless_map_stops_the_current_track() {
        let (output, mut source) = AudioOutput::test_output();
        let mut audio = GameAudio::with_output(output);
        // One second of constant synthetic PCM, so silence means stopped, not EOF.
        let samples = 44100_u32;
        let size = samples * 2;
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + size).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16_u32.to_le_bytes());
        wav.extend_from_slice(&1_u16.to_le_bytes());
        wav.extend_from_slice(&1_u16.to_le_bytes());
        wav.extend_from_slice(&44100_u32.to_le_bytes());
        wav.extend_from_slice(&88200_u32.to_le_bytes());
        wav.extend_from_slice(&2_u16.to_le_bytes());
        wav.extend_from_slice(&16_u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&size.to_le_bytes());
        for _ in 0..samples {
            wav.extend_from_slice(&16384_i16.to_le_bytes());
        }
        let mut vfs = VirtualFileSystem::new();
        vfs.mount_memory("synthetic", [("music/fixture.wav", wav)])
            .unwrap();
        audio.duel_music = audio.register_vfs_async(&vfs, "music/fixture.wav");
        assert!(audio.duel_music.is_some());
        assert!(audio.map_music.is_none());
        let snapshot = Snapshot {
            message_sequence: 1,
            reliable_acknowledge: 0,
            server_commands: vec![],
            server_time: 1000,
            delta_from: None,
            flags: 0,
            area_mask: vec![],
            player: PlayerState::zero(),
            vehicle_player: None,
            entities: vec![],
            consumed_bits: 0,
        };
        let game = GameState::empty_local(0);
        audio.apply_music_action(Some(LegacyMusicAction::StartPrivateDuel), &snapshot, &game);
        let deadline = Instant::now() + std::time::Duration::from_secs(10);
        loop {
            if source.next().unwrap().abs() > 0.001 {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "synthetic duel track did not start"
            );
            std::thread::yield_now();
        }
        audio.apply_music_action(Some(LegacyMusicAction::RestoreMap), &snapshot, &game);
        let deadline = Instant::now() + std::time::Duration::from_secs(10);
        loop {
            if source.next().unwrap().abs() < f32::EPSILON {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "duel track continued on a musicless map"
            );
            std::thread::yield_now();
        }
    }
}
