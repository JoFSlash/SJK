//! A completed siege objective's notice (`CG_SiegeObjectiveCompleted`, `cg_saga.c:885-985`):
//! the map's `.siege` file names, per objective, the line each team is shown in the
//! middle of the screen (`message_team1`/`message_team2`) and the announcer line each
//! team hears (`sound_team1`/`sound_team2`). The viewer's own team decides which.

use crate::ingame_menu::siege_data::{group, is_siege, read, value};
use sjk_protocol::{GameState, InfoString};
use sjk_vfs::VirtualFileSystem;
use std::time::Instant;

/// `CHAN_ANNOUNCER`.
const ANNOUNCER_CHANNEL: sjk_audio::ChannelId = sjk_audio::ChannelId(7);

/// The `.siege` objectives of the map last asked about, read once per map.
#[derive(Default)]
pub(crate) struct Notices {
    /// The map they were read for (empty before any).
    map: String,
    /// Each team's group of the file (team1, team2): its `ObjectiveN` groups.
    teams: [Vec<String>; 2],
}

impl Notices {
    /// Centre print and announce every objective in `done` for client `client` of
    /// `my_team` (`PERS_TEAM`); a spectator is told nothing.
    pub(crate) fn announce(
        &mut self,
        done: &[sjk_client::SiegeObjectiveComplete],
        client: u16,
        my_team: u8,
        game: &GameState,
        vfs: &VirtualFileSystem,
        localization: &crate::localization::Localization,
        chat: &mut crate::chat::ChatOverlay,
        audio: Option<&mut crate::audio::GameAudio>,
    ) {
        if done.is_empty() || !matches!(my_team, 1 | 2) || !is_siege(Some(game)) {
            return;
        }
        self.load(game, vfs);
        let mine = usize::from(my_team);
        let mut audio = audio;
        for objective in done {
            let team = if objective.team == 1 { 0 } else { 1 };
            let Some(found) = group(
                &self.teams[team],
                &format!("Objective{}", objective.objective),
            ) else {
                continue;
            };
            // `CG_DrawSiegeMessageNonMenu`: a `@` names a string-table entry.
            if let Some(message) = value(found, &format!("message_team{mine}")) {
                let text = match message.strip_prefix('@') {
                    Some(key) => localization
                        .strings
                        .get(&key.to_ascii_uppercase())
                        .cloned()
                        .unwrap_or_else(|| key.to_owned()),
                    None => message.to_owned(),
                };
                chat.receive(
                    sjk_client::ServerEventKind::CenterPrint,
                    text,
                    None,
                    Instant::now(),
                );
            }
            if let (Some(path), Some(audio)) = (
                value(found, &format!("sound_team{mine}")),
                audio.as_deref_mut(),
            ) {
                audio.register_effect_paths(vfs, [path]);
                audio.play_local(
                    path,
                    1.0,
                    sjk_audio::SourceId(u32::from(client)),
                    ANNOUNCER_CHANNEL,
                );
            }
        }
    }

    /// `CG_InitSiegeMode`'s read of `maps/<map>.siege`: the `Teams` group names each
    /// team's group, unless the server overrides it (`g_siegeTeam1`, `g_siegeTeam2`).
    fn load(&mut self, game: &GameState, vfs: &VirtualFileSystem) {
        let Some(info) = game
            .config_string(0)
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
            .and_then(|text| InfoString::parse(text).ok())
        else {
            return;
        };
        let Some(map) = info.get("mapname") else {
            return;
        };
        if map.eq_ignore_ascii_case(&self.map) {
            return;
        }
        self.map = map.to_owned();
        self.teams = Default::default();
        let tokens = match read(vfs, &format!("maps/{map}.siege")) {
            Ok(tokens) => tokens,
            Err(error) => {
                crate::log::progress(format_args!("siege objectives: {error}"));
                return;
            }
        };
        let names = group(&tokens, "Teams");
        for (index, slot) in self.teams.iter_mut().enumerate() {
            let name = info
                .get(&format!("g_siegeTeam{}", index + 1))
                .filter(|name| !name.is_empty() && !name.eq_ignore_ascii_case("none"))
                .or_else(|| names.and_then(|names| value(names, &format!("team{}", index + 1))));
            if let Some(team) = name.and_then(|name| group(&tokens, name)) {
                *slot = team.to_vec();
            }
        }
    }
}
