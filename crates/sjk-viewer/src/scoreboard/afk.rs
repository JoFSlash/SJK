//! Who is away from the keyboard, for the scoreboard (`cg_scoreboardAfk`).
//!
//! Only what the server sends anyway is read, so it works on every server. A
//! player in view whose view angles and weapon have not changed for
//! [`AWAY_AFTER`] is away: someone playing moves the mouse all the time, even
//! standing still to watch a duel. The time out of view counts too once the
//! player is seen again unchanged: their view could not come back to the same
//! angle by chance. A player never seen yet, or whose stillness is not proven,
//! counts as present: the board may miss someone away, never mark someone playing.
//!
//! Their chat, a weapon change or a score gain shows them back. A respawn or a
//! teleport (`EF_TELEPORT_BIT` toggled) sets the view somewhere new without the
//! player doing anything, so it starts a fresh watch keeping the time already
//! idle, as a new map does; while dead the view follows the killer and is not
//! read. Being pushed around moves a player but not their view, so position is
//! not read either. A name wearing one of [`NAME_TAGS`] is away at once.

use sjk_client::{EF_TELEPORT_BIT, ScoreEntry};
use sjk_protocol::{GameState, Snapshot};

/// Archived cvar: away players dimmed at the bottom of their list (on by default).
pub(crate) const CVAR: &str = "cg_scoreboardAfk";
/// How long a player in view stays still before they are away, in milliseconds.
pub(super) const AWAY_AFTER: i32 = 180_000;
/// The tags players put in their name to say they are away (case ignored,
/// colours stripped): SJK's own `afk` command adds `[AFK]` (`cl_afkPrefix`), and
/// players of every client type these by hand.
pub(super) const NAME_TAGS: &[&str] = &["[afk]", "(afk)", "<afk>", "{afk}", "|afk|"];

/// `EF_DEAD` (`bg_public.h`).
const EF_DEAD: u32 = 1 << 1;
/// `ET_PLAYER` (`bg_public.h`).
const ET_PLAYER: u8 = 1;
const CS_PLAYERS: usize = 1_131;
const CLIENTS: usize = 32;
/// The smallest view change that counts, in degrees: far below a mouse count.
const TURN: f32 = 0.01;

/// Whether `cg_scoreboardAfk` is on (unless switched off).
pub(super) fn enabled(console: Option<&crate::console::ViewerConsole>) -> bool {
    console
        .and_then(|console| console.bool_cvar(CVAR))
        .unwrap_or(true)
}

/// Whether `name` wears one of [`NAME_TAGS`].
pub(super) fn tagged(name: &str) -> bool {
    let plain = sjk_client::chat_plain_text(name).to_ascii_lowercase();
    NAME_TAGS.iter().any(|tag| plain.contains(tag))
}

/// What the player controls in their entity: where they look and what they hold.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Pose {
    pitch: f32,
    yaw: f32,
    weapon: u8,
}

impl Pose {
    fn moved(self, other: Self) -> bool {
        let yaw = (self.yaw - other.yaw).rem_euclid(360.0);
        (self.pitch - other.pitch).abs() > TURN
            || yaw.min(360.0 - yaw) > TURN
            || self.weapon != other.weapon
    }
}

/// One client slot's watch.
#[derive(Clone, Copy, Debug, Default)]
struct Watch {
    /// The pose last seen alive since the watch started.
    pose: Option<Pose>,
    teleport: bool,
    dead: bool,
    /// Server time the watch started (the last activity, or a fresh start).
    since: i32,
    /// Server time the pose was last seen unchanged.
    confirmed: i32,
    /// Time already idle when the watch last started fresh.
    carried: i32,
    score: Option<i32>,
    spoken: u32,
    /// The name's hash: a new name (or a new player in the slot) is activity.
    name: u64,
}

impl Watch {
    fn idle(&self) -> i32 {
        self.carried
            .saturating_add(self.confirmed.saturating_sub(self.since))
    }

    fn active(&mut self, now: i32) {
        self.carried = 0;
        self.since = now;
        self.confirmed = now;
    }

    /// Start a fresh watch at `now`, keeping the time already idle.
    fn restart(&mut self, now: i32) {
        self.carried = self.idle();
        self.since = now;
        self.confirmed = now;
        self.pose = None;
    }
}

/// Every client slot's watch.
pub(super) struct Afk {
    watches: [Watch; CLIENTS],
    time: i32,
}

impl Default for Afk {
    fn default() -> Self {
        Self {
            watches: [Watch::default(); CLIENTS],
            time: i32::MIN,
        }
    }
}

impl Afk {
    /// Read a frame: the roster, the chat counters ([`crate::chat::ChatOverlay::spoken`]),
    /// the scores and, once per snapshot, the players in view.
    pub(super) fn observe(
        &mut self,
        snapshot: &Snapshot,
        game: &GameState,
        scores: &[ScoreEntry],
        spoken: &[u32; CLIENTS],
    ) {
        let now = snapshot.server_time;
        if now < self.time {
            // A new map's clock, or a demo rewound.
            for watch in &mut self.watches {
                watch.restart(now);
            }
        }
        let fresh = now != self.time;
        self.time = now;
        for (client, watch) in self.watches.iter_mut().enumerate() {
            let name = game
                .config_string(CS_PLAYERS + client)
                .filter(|bytes| !bytes.is_empty())
                .map(|bytes| {
                    let info = sjk_client::LegacyClientInfo::new(bytes);
                    super::byte_signature(info.bytes("n").unwrap_or_default())
                });
            let Some(name) = name else {
                *watch = Watch {
                    since: now,
                    confirmed: now,
                    spoken: spoken[client],
                    ..Watch::default()
                };
                continue;
            };
            if watch.name != name || watch.spoken != spoken[client] {
                watch.active(now);
            }
            watch.name = name;
            watch.spoken = spoken[client];
        }
        for score in scores {
            let Some(watch) = self.watches.get_mut(usize::from(score.client_num)) else {
                continue;
            };
            if watch.score.is_some_and(|old| score.score > old) {
                watch.active(now);
            }
            watch.score = Some(score.score);
        }
        if !fresh {
            return;
        }
        let local = snapshot.player.client_num();
        for entity in &snapshot.entities {
            let client = entity.number();
            if entity.entity_type() != ET_PLAYER || client == local {
                continue;
            }
            let Some(watch) = self.watches.get_mut(usize::from(client)) else {
                continue;
            };
            let flags = entity.e_flags();
            let teleport = flags & EF_TELEPORT_BIT != 0;
            let dead = flags & EF_DEAD != 0;
            let [pitch, yaw, _] = entity.angular_trajectory_base();
            let pose = Pose {
                pitch,
                yaw,
                weapon: entity.weapon(),
            };
            if dead {
                watch.pose = None;
            } else {
                match watch.pose {
                    Some(seen) if teleport == watch.teleport && !watch.dead => {
                        if pose.moved(seen) {
                            watch.active(now);
                        } else {
                            watch.confirmed = now;
                        }
                    }
                    _ => watch.restart(now),
                }
                watch.pose = Some(pose);
            }
            watch.teleport = teleport;
            watch.dead = dead;
        }
    }

    /// The whole minutes `client` has been seen idle, once that reaches [`AWAY_AFTER`].
    pub(super) fn away(&self, client: u8) -> Option<u32> {
        let idle = self.watches.get(usize::from(client))?.idle();
        (idle >= AWAY_AFTER).then_some((idle / 60_000) as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjk_protocol::{EntityState, LEGACY_ENTITY_FIELDS, PlayerState};

    const SECOND: i32 = 1_000;

    fn game(names: &[(usize, &str)]) -> GameState {
        let mut game = GameState::empty_local(0);
        for (client, name) in names {
            game.replace_config_string(
                CS_PLAYERS + client,
                format!("n\\{name}\\t\\0").into_bytes(),
            )
            .unwrap();
        }
        game
    }

    /// Client `client`'s entity looking at `yaw` (pitch 10), with `flags`.
    fn player(client: u16, yaw: f32, flags: u32) -> EntityState {
        let mut entity = EntityState::zero(client, &LEGACY_ENTITY_FIELDS);
        entity.set_raw_field(8, u32::from(ET_PLAYER));
        entity.set_raw_field(5, 10.0_f32.to_bits());
        entity.set_raw_field(3, yaw.to_bits());
        entity.set_raw_field(19, flags);
        entity
    }

    /// The local player is client 0.
    fn snapshot(time: i32, entities: Vec<EntityState>) -> Snapshot {
        Snapshot {
            message_sequence: 1,
            reliable_acknowledge: 0,
            server_commands: Vec::new(),
            server_time: time,
            delta_from: None,
            flags: 0,
            area_mask: Vec::new(),
            player: PlayerState::zero(),
            vehicle_player: None,
            entities,
            consumed_bits: 0,
        }
    }

    fn score(score: i32) -> ScoreEntry {
        ScoreEntry {
            client_num: 3,
            score,
            ping: 50,
            time_minutes: 1,
            flags: 0,
            powerups: 0,
            defends: 0,
            assists: 0,
            captures: 0,
            deaths: None,
        }
    }

    /// Feed `afk` the players in view every second from `from` to `to`.
    fn watch(
        afk: &mut Afk,
        game: &GameState,
        from: i32,
        to: i32,
        view: impl Fn(i32) -> Vec<EntityState>,
    ) {
        let mut time = from;
        while time <= to {
            afk.observe(&snapshot(time, view(time)), game, &[], &[0; CLIENTS]);
            time += SECOND;
        }
    }

    #[test]
    fn a_still_view_is_away_after_three_minutes_and_a_turn_brings_it_back() {
        let game = game(&[(3, "Sleeper")]);
        let mut afk = Afk::default();
        watch(&mut afk, &game, 1_000, 180_000, |_| {
            vec![player(3, 90.0, 0)]
        });
        assert_eq!(afk.away(3), None);
        watch(&mut afk, &game, 181_000, 241_000, |_| {
            vec![player(3, 90.0, 0)]
        });
        assert_eq!(afk.away(3), Some(4));
        watch(&mut afk, &game, 242_000, 242_000, |_| {
            vec![player(3, 91.0, 0)]
        });
        assert_eq!(afk.away(3), None);
    }

    #[test]
    fn a_moving_view_never_is_away() {
        let game = game(&[(3, "Player")]);
        let mut afk = Afk::default();
        watch(&mut afk, &game, 1_000, 600_000, |time| {
            vec![player(3, (time / 30_000) as f32, 0)]
        });
        assert_eq!(afk.away(3), None);
    }

    #[test]
    fn time_out_of_view_counts_once_seen_again_unchanged() {
        let game = game(&[(3, "Corner")]);
        let mut afk = Afk::default();
        watch(&mut afk, &game, 1_000, 2_000, |_| vec![player(3, 45.0, 0)]);
        watch(&mut afk, &game, 3_000, 300_000, |_| Vec::new());
        // Unseen, nothing is proven yet.
        assert_eq!(afk.away(3), None);
        watch(&mut afk, &game, 301_000, 301_000, |_| {
            vec![player(3, 45.0, 0)]
        });
        assert_eq!(afk.away(3), Some(5));
        // Back with another view: they were playing out of sight.
        let mut afk = Afk::default();
        watch(&mut afk, &game, 1_000, 2_000, |_| vec![player(3, 45.0, 0)]);
        watch(&mut afk, &game, 3_000, 300_000, |_| Vec::new());
        watch(&mut afk, &game, 301_000, 301_000, |_| {
            vec![player(3, 50.0, 0)]
        });
        assert_eq!(afk.away(3), None);
    }

    #[test]
    fn deaths_and_respawns_keep_the_idle_time() {
        let game = game(&[(3, "Victim")]);
        let mut afk = Afk::default();
        watch(&mut afk, &game, 1_000, 120_000, |_| vec![player(3, 0.0, 0)]);
        // Killed: the view turns to the killer; respawned elsewhere, teleport bit set.
        watch(&mut afk, &game, 121_000, 125_000, |_| {
            vec![player(3, 200.0, EF_DEAD)]
        });
        watch(&mut afk, &game, 126_000, 190_000, |_| {
            vec![player(3, 270.0, EF_TELEPORT_BIT)]
        });
        assert_eq!(afk.away(3), Some(3));
    }

    #[test]
    fn chat_score_and_a_new_name_are_activity() {
        let mut game = game(&[(3, "Quiet")]);
        let mut afk = Afk::default();
        watch(&mut afk, &game, 1_000, 200_000, |_| vec![player(3, 0.0, 0)]);
        assert!(afk.away(3).is_some());
        let mut spoken = [0; CLIENTS];
        spoken[3] = 1;
        afk.observe(
            &snapshot(201_000, vec![player(3, 0.0, 0)]),
            &game,
            &[],
            &spoken,
        );
        assert_eq!(afk.away(3), None);

        let mut afk = Afk::default();
        afk.observe(
            &snapshot(500, Vec::new()),
            &game,
            &[score(4)],
            &[0; CLIENTS],
        );
        watch(&mut afk, &game, 1_000, 200_000, |_| vec![player(3, 0.0, 0)]);
        // Losing points (being killed) is not activity; gaining them is.
        afk.observe(
            &snapshot(201_000, vec![player(3, 0.0, 0)]),
            &game,
            &[score(3)],
            &[0; CLIENTS],
        );
        assert!(afk.away(3).is_some());
        afk.observe(
            &snapshot(202_000, vec![player(3, 0.0, 0)]),
            &game,
            &[score(5)],
            &[0; CLIENTS],
        );
        assert_eq!(afk.away(3), None);

        let mut afk = Afk::default();
        watch(&mut afk, &game, 1_000, 200_000, |_| vec![player(3, 0.0, 0)]);
        game.replace_config_string(CS_PLAYERS + 3, b"n\\Back\\t\\0".to_vec())
            .unwrap();
        afk.observe(
            &snapshot(201_000, vec![player(3, 0.0, 0)]),
            &game,
            &[],
            &[0; CLIENTS],
        );
        assert_eq!(afk.away(3), None);
    }

    #[test]
    fn the_local_player_and_empty_slots_are_never_away() {
        let game = game(&[(0, "Me")]);
        let mut afk = Afk::default();
        // Slot 0 is the local player; slot 5 has no player.
        watch(&mut afk, &game, 1_000, 400_000, |_| {
            vec![player(0, 0.0, 0), player(5, 0.0, 0)]
        });
        assert_eq!(afk.away(0), None);
        assert_eq!(afk.away(5), None);
    }

    #[test]
    fn name_tags_are_read_without_colours_or_case() {
        assert!(tagged("^1[AFK]^7Sol"));
        assert!(tagged("Sol (afk)"));
        assert!(!tagged("Afkar"));
        assert!(!tagged("Sol"));
    }
}
