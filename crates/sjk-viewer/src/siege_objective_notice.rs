//! A completed siege objective's notice (`CG_SiegeObjectiveCompleted`, `cg_saga.c:885-985`):
//! the map's `.siege` file names, per objective, the line each team is shown in the
//! middle of the screen (`message_team1`/`message_team2`) and the announcer line each
//! team hears (`sound_team1`/`sound_team2`). The objective that ends the round is not
//! announced as one: `CG_SiegeRoundOver` (`cg_saga.c:575-671`) takes the viewer's team
//! group's `wonround`/`lostround` line and `roundover_sound_wewon`/`_welost` announcer
//! instead. The viewer's own team decides which.
//!
//! SJK addition: the retail maps name round-over announcers (`40MOM011` to `40MOM014`)
//! that the retail archives never shipped, so the reference ends every retail round
//! in silence. A named announcer no installed file answers to gives way to the
//! announcer's own winner or loser line ([`ROUND_OVER_FALLBACK`]); one that exists,
//! as a community map's may, still plays as written.

use crate::ingame_menu::siege_data::{group, is_siege, read, value};
use sjk_protocol::{GameState, InfoString};
use sjk_vfs::VirtualFileSystem;
use std::time::Instant;

/// `CHAN_ANNOUNCER`.
const ANNOUNCER_CHANNEL: sjk_audio::ChannelId = sjk_audio::ChannelId(7);

/// The round-over announcer for a round won and one lost when the `.siege` file's
/// is missing: `cgs.media.winnerSound` and `loserSound` (`cg_main.c:992-993`), which
/// the retail cgame registers but never plays.
const ROUND_OVER_FALLBACK: [&str; 2] = [
    "sound/chars/protocol/misc/40MOM006",
    "sound/chars/protocol/misc/40MOM010",
];

/// The `.siege` objectives of the map last asked about, read once per map.
#[derive(Default)]
pub(crate) struct Notices {
    /// The map they were read for (empty before any).
    map: String,
    /// Each team's group of the file (team1, team2): its `ObjectiveN` groups.
    teams: [Vec<String>; 2],
}

impl Notices {
    /// Centre print and announce the objectives completed and the round won in the
    /// snapshot `events` last observed, for client `client` of `my_team` (`PERS_TEAM`);
    /// a spectator is told nothing.
    pub(crate) fn announce(
        &mut self,
        events: &sjk_client::ObituaryTracker,
        client: u16,
        my_team: u8,
        game: &GameState,
        vfs: &VirtualFileSystem,
        localization: &crate::localization::Localization,
        chat: &mut crate::chat::ChatOverlay,
        mut audio: Option<&mut crate::audio::GameAudio>,
    ) {
        let (done, round_won) = (events.siege_objectives(), events.siege_round_won());
        if (done.is_empty() && round_won.is_none())
            || !matches!(my_team, 1 | 2)
            || !is_siege(Some(game))
        {
            return;
        }
        self.load(game, vfs);
        let mine = usize::from(my_team);
        let mut say = |message: Option<&str>, sound: Option<&str>, fallback: Option<&str>| {
            Self::say(
                message,
                sound,
                fallback,
                client,
                vfs,
                localization,
                chat,
                audio.as_deref_mut(),
            );
        };
        // `CG_SiegeObjectiveCompleted`: the completing team's `ObjectiveN` names what
        // each team is shown and hears.
        for objective in done {
            let team = if objective.team == 1 { 0 } else { 1 };
            let Some(found) = group(
                &self.teams[team],
                &format!("Objective{}", objective.objective),
            ) else {
                continue;
            };
            say(
                value(found, &format!("message_team{mine}")),
                value(found, &format!("sound_team{mine}")),
                None,
            );
        }
        // `CG_SiegeRoundOver`: the last objective ends the round instead of being
        // announced; the viewer's own team group names the line and the announcer for
        // a round won (`wonround`, `roundover_sound_wewon`) or lost.
        if let Some(winner) = round_won {
            let team = &self.teams[mine - 1];
            let (message, sound, fallback) = if winner == my_team {
                ("wonround", "roundover_sound_wewon", ROUND_OVER_FALLBACK[0])
            } else {
                (
                    "lostround",
                    "roundover_sound_welost",
                    ROUND_OVER_FALLBACK[1],
                )
            };
            // A team group that names no announcer stays silent, as in the reference.
            say(value(team, message), value(team, sound), Some(fallback));
        }
    }

    /// Centre print `message` (an `@` names a string-table entry,
    /// `CG_DrawSiegeMessageNonMenu`) and play `sound` on the announcer channel, or
    /// `fallback` when no installed file answers to `sound`.
    #[allow(clippy::too_many_arguments)]
    fn say(
        message: Option<&str>,
        sound: Option<&str>,
        fallback: Option<&str>,
        client: u16,
        vfs: &VirtualFileSystem,
        localization: &crate::localization::Localization,
        chat: &mut crate::chat::ChatOverlay,
        audio: Option<&mut crate::audio::GameAudio>,
    ) {
        if let Some(message) = message {
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
        if let (Some(path), Some(audio)) = (sound, audio) {
            let path = match fallback {
                Some(fallback) if !audio.register_effect(vfs, path) => {
                    audio.register_effect(vfs, fallback);
                    fallback
                }
                _ => {
                    audio.register_effect(vfs, path);
                    path
                }
            };
            audio.play_local(
                path,
                1.0,
                sjk_audio::SourceId(u32::from(client)),
                ANNOUNCER_CHANNEL,
            );
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
