//! BaseJKA obituary decoding and fixed-capacity kill-feed state.

use sjk_protocol::{EntityState, GameState, Snapshot};

const ET_EVENTS: u8 = 18;
const EVENT_MASK: u16 = 0xff;
const MAX_ENTITIES: usize = 1_024;
const MAX_CLIENTS: u16 = 64;
const CS_PLAYERS: usize = 1_131;

/// `EV_OBITUARY` in `codemp/game/bg_public.h:927`.
pub const EV_OBITUARY: u16 = 93;
/// `EV_SIEGE_ROUNDOVER`, eight past `EV_OBITUARY` in `entity_event_t`: a siege round is
/// won, by the team in its `eventParm` (`SiegeRoundComplete`, `g_saga.c`).
pub const EV_SIEGE_ROUNDOVER: u16 = sjk_game_jka::siege::EV_SIEGE_ROUNDOVER as u16;

/// The last siege round won that a snapshot carried.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SiegeRoundOver {
    /// `SIEGETEAM_TEAM1` (1, red) or `SIEGETEAM_TEAM2` (2, blue).
    pub winner: u8,
    /// The snapshot's server time.
    pub server_time: i32,
}
/// `EV_SIEGE_OBJECTIVECOMPLETE`: a siege objective done (`SiegeBroadcast_OBJECTIVECOMPLETE`,
/// `g_saga.c`).
const EV_SIEGE_OBJECTIVECOMPLETE: u16 = sjk_game_jka::siege::EV_SIEGE_OBJECTIVECOMPLETE as u16;

/// A siege objective a snapshot said was completed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SiegeObjectiveComplete {
    /// The team whose objective it is (`eventParm`): 1 or 2.
    pub team: u8,
    /// Its number in that team's `.siege` group (`trickedentindex`), from 1.
    pub objective: u16,
}
/// `EV_SIEGESPEC`: a siege player now waits for its respawn wave, which comes at the
/// event's `time` (`g_siegeRespawnCheck`).
const EV_SIEGESPEC: u16 = 191;
/// `PM_SPECTATOR`: how a siege player waiting for its wave moves.
use sjk_game_jka::PM_SPECTATOR;

/// Gender used by the gendered self-kill strings in `CG_Obituary`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Gender {
    Female,
    Neuter,
    #[default]
    Male,
}

/// One decoded obituary, preserving the localization keys selected by codemp.
///
/// `message` is the victim-only/self-kill key and `attacker_message` is the
/// three-part `victim message attacker` key. See
/// `codemp/cgame/cg_event.c:124-454`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObituaryEvent {
    /// Client number of the victim.
    pub target: u16,
    /// Client number of the attacker, or the legacy world entity number.
    pub attacker: u16,
    /// `meansOfDeath_t` ordinal carried by `eventParm`.
    pub means_of_death: u8,
    /// Localization key for world and self kills.
    pub message: &'static str,
    /// Localization key placed between victim and attacker for player kills.
    pub attacker_message: Option<&'static str>,
    /// Whether the current client killed another player.
    pub local_fragged: bool,
    /// Whether another player killed the current client.
    pub local_was_killed: bool,
    /// Snapshot server time at which the event was accepted.
    pub server_time: i32,
    /// The attacker's `forcePowersActive` bits in that snapshot (0 when the
    /// snapshot does not hold the attacker), which tell Force Grip from Force
    /// Lightning, as `MOD_FORCE_DARK` covers both.
    pub attacker_force: u32,
}

/// Eight-entry retained kill-feed ring. Oldest entries are overwritten.
pub struct KillFeed {
    entries: [Option<ObituaryEvent>; 8],
    next: usize,
    len: usize,
}

impl KillFeed {
    /// Construct an empty fixed-capacity feed.
    pub const fn new() -> Self {
        Self {
            entries: [None; 8],
            next: 0,
            len: 0,
        }
    }

    /// Append an event, overwriting the oldest entry once full.
    pub fn push(&mut self, event: ObituaryEvent) {
        self.entries[self.next] = Some(event);
        self.next = (self.next + 1) % self.entries.len();
        self.len = (self.len + 1).min(self.entries.len());
    }

    /// Number of retained entries.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Read an entry, where offset zero is newest.
    pub fn newest(&self, offset: usize) -> Option<ObituaryEvent> {
        if offset >= self.len {
            return None;
        }
        let index = (self.next + self.entries.len() - 1 - offset) % self.entries.len();
        self.entries[index]
    }
}

impl Default for KillFeed {
    fn default() -> Self {
        Self::new()
    }
}

/// Allocation-free entity-event observer for obituary events, which also keeps the last
/// siege round won (`EV_SIEGE_ROUNDOVER`) from the same scan.
pub struct ObituaryTracker {
    signatures: [u16; MAX_ENTITIES],
    present: [bool; MAX_ENTITIES],
    feed: KillFeed,
    decoded: u64,
    siege_round_over: Option<SiegeRoundOver>,
    siege_objectives: Vec<SiegeObjectiveComplete>,
    siege_respawn_at: i32,
    hack: (i32, i32),
}

impl ObituaryTracker {
    /// Construct an empty event-signature tracker and feed.
    pub fn new() -> Self {
        Self {
            signatures: [0; MAX_ENTITIES],
            present: [false; MAX_ENTITIES],
            feed: KillFeed::new(),
            decoded: 0,
            siege_round_over: None,
            siege_objectives: Vec::with_capacity(4),
            siege_respawn_at: 0,
            hack: (0, 0),
        }
    }

    /// Observe every entity event in one accepted snapshot exactly once.
    pub fn observe(&mut self, snapshot: &Snapshot, game: &GameState) {
        self.present.fill(false);
        self.siege_objectives.clear();
        // A wave the viewed player no longer waits for — it is back in play, or the level,
        // round or server changed under it — is forgotten.
        if snapshot.player.movement_type() != PM_SPECTATOR {
            self.siege_respawn_at = 0;
        }
        // `ps.hackingTime` and `ps.hackingBaseTime` (netfields 91 and 124).
        self.hack = (
            snapshot.player.raw_field(91).unwrap_or(0) as i32,
            snapshot.player.raw_field(124).unwrap_or(0) as i32,
        );
        for entity in &snapshot.entities {
            let index = usize::from(entity.number());
            if index >= MAX_ENTITIES {
                continue;
            }
            self.present[index] = true;
            let signature = raw_event(entity);
            if signature == 0 || signature == self.signatures[index] {
                continue;
            }
            self.signatures[index] = signature;
            let event = event_number(entity, signature);
            if event == EV_SIEGE_ROUNDOVER {
                self.siege_round_over = Some(SiegeRoundOver {
                    winner: entity.event_parameter(),
                    server_time: snapshot.server_time,
                });
                continue;
            }
            if event == EV_SIEGE_OBJECTIVECOMPLETE {
                self.siege_objectives.push(SiegeObjectiveComplete {
                    team: entity.event_parameter(),
                    objective: entity.tracked_entity_num(),
                });
                continue;
            }
            // `CG_EntityEvent`'s `EV_SIEGESPEC`: only the viewed player's own wait is kept
            // (`cg_siegeDeathTime`).
            if event == EV_SIEGESPEC && entity.owner() == snapshot.player.client_num() {
                self.siege_respawn_at = entity.time();
            }
            if event != EV_OBITUARY {
                continue;
            }
            let gender = client_gender(game, entity.other_entity_num());
            let mut event = legacy_obituary(
                entity,
                snapshot.player.client_num(),
                gender,
                snapshot.server_time,
            );
            event.attacker_force = attacker_force(snapshot, event.attacker);
            self.feed.push(event);
            self.decoded += 1;
        }
        for (index, present) in self.present.iter().copied().enumerate() {
            if !present {
                self.signatures[index] = 0;
            }
        }
    }

    /// Access the retained feed without copying it.
    pub const fn feed(&self) -> &KillFeed {
        &self.feed
    }

    /// When the viewed player's siege respawn wave comes (`cg_siegeDeathTime`), or 0
    /// before any `EV_SIEGESPEC` named it.
    pub const fn siege_respawn_at(&self) -> i32 {
        self.siege_respawn_at
    }

    /// The viewed player's hack (`ps.hackingTime`, `ps.hackingBaseTime`): when it is done
    /// and how long it takes, both 0 while it hacks nothing.
    pub const fn hack(&self) -> (i32, i32) {
        self.hack
    }

    /// Total obituary events decoded by this tracker.
    pub const fn decoded(&self) -> u64 {
        self.decoded
    }

    /// The siege objectives the last snapshot observed said were completed.
    pub fn siege_objectives(&self) -> &[SiegeObjectiveComplete] {
        &self.siege_objectives
    }

    /// The last siege round won that this tracker saw (`EV_SIEGE_ROUNDOVER`), the same
    /// entity scan the obituaries take.
    pub const fn siege_round_over(&self) -> Option<SiegeRoundOver> {
        self.siege_round_over
    }
}

impl Default for ObituaryTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// A client's name as the kill messages show it: the `n` key of its `CS_PLAYERS`
/// string with its colour codes, read from the string's bytes
/// ([`crate::LegacyClientInfo::name`]), or `noname` for a slot with no name at all,
/// where `CG_Obituary` would print an empty one.
pub fn obituary_name(game: &GameState, client: u16) -> std::borrow::Cow<'_, str> {
    game.config_string(CS_PLAYERS + usize::from(client))
        .and_then(|info| crate::LegacyClientInfo::new(info).name())
        .unwrap_or(std::borrow::Cow::Borrowed("noname"))
}

/// Select the exact localization key used by `CG_Obituary`.
pub fn legacy_obituary(
    entity: &EntityState,
    local_client: u16,
    gender: Gender,
    server_time: i32,
) -> ObituaryEvent {
    let target = entity.other_entity_num();
    let attacker = entity.other_entity_num2();
    let means = entity.event_parameter();
    let self_kill = target == attacker;
    let (message, attacker_message) = if self_kill {
        (self_message(means, gender), None)
    } else if target < MAX_CLIENTS && attacker < MAX_CLIENTS {
        ("", Some(kill_message(means)))
    } else if let Some(message) = single_message(means) {
        (message, None)
    } else {
        ("DIED_GENERIC", None)
    };
    ObituaryEvent {
        target,
        attacker,
        means_of_death: means,
        message,
        attacker_message,
        local_fragged: attacker == local_client && target != attacker,
        local_was_killed: target == local_client && attacker != target,
        server_time,
        attacker_force: 0,
    }
}

/// The `forcePowersActive` bits of client `attacker` in `snapshot`, 0 when it
/// holds no such client.
fn attacker_force(snapshot: &Snapshot, attacker: u16) -> u32 {
    if attacker >= MAX_CLIENTS {
        return 0;
    }
    if attacker == snapshot.player.client_num() {
        return snapshot.player.force_powers_active();
    }
    snapshot
        .entities
        .iter()
        .find(|entity| entity.number() == attacker)
        .map_or(0, EntityState::force_powers_active)
}

/// `cg_event.c:160-175`: water, slime, lava, crush, falling, suicide and
/// trigger_hurt all share `DIED_GENERIC`; only the target laser differs.
fn single_message(means: u8) -> Option<&'static str> {
    match means {
        33..=36 | 38 | 39 | 41 => Some("DIED_GENERIC"),
        40 => Some("DIED_LASER"),
        _ => None,
    }
}

fn self_message(means: u8, gender: Gender) -> &'static str {
    let stem = match means {
        4..=13 | 17 => "SUICIDE_SHOT",
        14 | 18..=30 => "SUICIDE_EXPLOSIVES",
        15 => "SUICIDE_ELECTROCUTED",
        38 => "SUICIDE_FALLDEATH",
        _ => "SUICIDE_GENERICDEATH",
    };
    match (stem, gender) {
        ("SUICIDE_SHOT", Gender::Female) => "SUICIDE_SHOT_FEMALE",
        ("SUICIDE_SHOT", Gender::Neuter) => "SUICIDE_SHOT_GENDERLESS",
        ("SUICIDE_SHOT", Gender::Male) => "SUICIDE_SHOT_MALE",
        ("SUICIDE_EXPLOSIVES", Gender::Female) => "SUICIDE_EXPLOSIVES_FEMALE",
        ("SUICIDE_EXPLOSIVES", Gender::Neuter) => "SUICIDE_EXPLOSIVES_GENDERLESS",
        ("SUICIDE_EXPLOSIVES", Gender::Male) => "SUICIDE_EXPLOSIVES_MALE",
        ("SUICIDE_ELECTROCUTED", Gender::Female) => "SUICIDE_ELECTROCUTED_FEMALE",
        ("SUICIDE_ELECTROCUTED", Gender::Neuter) => "SUICIDE_ELECTROCUTED_GENDERLESS",
        ("SUICIDE_ELECTROCUTED", Gender::Male) => "SUICIDE_ELECTROCUTED_MALE",
        ("SUICIDE_FALLDEATH", Gender::Female) => "SUICIDE_FALLDEATH_FEMALE",
        ("SUICIDE_FALLDEATH", Gender::Neuter) => "SUICIDE_FALLDEATH_GENDERLESS",
        ("SUICIDE_FALLDEATH", Gender::Male) => "SUICIDE_FALLDEATH_MALE",
        (_, Gender::Female) => "SUICIDE_GENERICDEATH_FEMALE",
        (_, Gender::Neuter) => "SUICIDE_GENERICDEATH_GENDERLESS",
        (_, Gender::Male) => "SUICIDE_GENERICDEATH_MALE",
    }
}

fn kill_message(means: u8) -> &'static str {
    match means {
        1 => "KILLED_STUN",
        2 => "KILLED_MELEE",
        3 => "KILLED_SABER",
        4 | 5 => "KILLED_BRYAR",
        6 | 7 => "KILLED_BLASTER",
        8 | 9 => "KILLED_DISRUPTOR",
        10 => "KILLED_DISRUPTORSNIPE",
        11 => "KILLED_BOWCASTER",
        12 => "KILLED_REPEATER",
        13 | 14 => "KILLED_REPEATERALT",
        15 | 16 => "KILLED_DEMP2",
        17 => "KILLED_FLECHETTE",
        18 => "KILLED_FLECHETTE_MINE",
        19 | 20 => "KILLED_ROCKET",
        21 | 22 => "KILLED_ROCKET_HOMING",
        23 | 24 => "KILLED_THERMAL",
        25 => "KILLED_TRIPMINE",
        26 => "KILLED_TRIPMINE_TIMED",
        27 => "KILLED_DETPACK",
        31 => "KILLED_DARKFORCE",
        32 => "KILLED_SENTRY",
        37 => "KILLED_TELEFRAG",
        38 => "KILLED_FORCETOSS",
        _ => "KILLED_GENERIC",
    }
}

fn client_gender(game: &GameState, client: u16) -> Gender {
    let Some(info) = game.config_string(CS_PLAYERS + usize::from(client)) else {
        return Gender::Male;
    };
    // Read from the bytes, not a UTF-8 view of the whole string: a name with a
    // Latin-1 letter (`é` is byte 0xE9) must not cost the player their gender.
    let info = crate::LegacyClientInfo::new(info);
    let Some(sex) = info.bytes("ds").or_else(|| info.bytes("sex")) else {
        return Gender::Male;
    };
    match sex.first().copied().map(|value| value.to_ascii_lowercase()) {
        Some(b'f') => Gender::Female,
        Some(b'n') => Gender::Neuter,
        _ => Gender::Male,
    }
}

fn raw_event(entity: &EntityState) -> u16 {
    if entity.entity_type() >= ET_EVENTS {
        u16::from(entity.entity_type())
    } else {
        entity.event()
    }
}

fn event_number(entity: &EntityState, raw: u16) -> u16 {
    if entity.entity_type() >= ET_EVENTS {
        u16::from(entity.entity_type() - ET_EVENTS)
    } else {
        raw & EVENT_MASK
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjk_protocol::LEGACY_ENTITY_FIELDS;

    /// A self-kill by `client` with `means`, as the event entity carries it.
    fn self_kill(client: u16, means: u8) -> EntityState {
        let mut entity = EntityState::zero(200, &LEGACY_ENTITY_FIELDS);
        entity.set_raw_field(8, u32::from(ET_EVENTS) + u32::from(EV_OBITUARY));
        entity.set_raw_field(59, u32::from(client));
        entity.set_raw_field(39, u32::from(client));
        entity.set_raw_field(42, u32::from(means));
        entity
    }

    /// `EV_SIEGESPEC` for `owner`, its wave at `time`.
    fn siege_spec(number: u16, owner: u16, time: i32) -> EntityState {
        let mut entity = EntityState::zero(number, &LEGACY_ENTITY_FIELDS);
        entity.set_raw_field(8, u32::from(ET_EVENTS) + u32::from(EV_SIEGESPEC));
        entity.set_raw_field(40, u32::from(owner));
        entity.set_raw_field(65, time as u32);
        entity
    }

    fn snapshot(entities: Vec<EntityState>) -> Snapshot {
        Snapshot {
            message_sequence: 0,
            reliable_acknowledge: 0,
            server_commands: Vec::new(),
            server_time: 1_000,
            delta_from: None,
            flags: 0,
            area_mask: Vec::new(),
            player: sjk_protocol::PlayerState::zero(),
            vehicle_player: None,
            entities,
            consumed_bits: 0,
        }
    }

    #[test]
    fn the_viewed_players_siege_wave_is_kept() {
        let game = GameState::empty_local(0);
        let mut tracker = ObituaryTracker::new();
        // Another player's wait is not the viewed player's (client 0).
        tracker.observe(&snapshot(vec![siege_spec(300, 5, 20_000)]), &game);
        assert_eq!(tracker.siege_respawn_at(), 0);
        let mut waiting = snapshot(vec![siege_spec(301, 0, 40_000)]);
        waiting.player.set_movement_type(PM_SPECTATOR);
        tracker.observe(&waiting, &game);
        assert_eq!(tracker.siege_respawn_at(), 40_000);
        waiting.entities.clear();
        tracker.observe(&waiting, &game);
        assert_eq!(tracker.siege_respawn_at(), 40_000);
        // Back in play: the wave is forgotten.
        tracker.observe(&snapshot(Vec::new()), &game);
        assert_eq!(tracker.siege_respawn_at(), 0);
        // A hack in progress is read from the player state, and gone once it is.
        let mut hacking = snapshot(Vec::new());
        hacking.player.set_raw_field(91, 11_000);
        hacking.player.set_raw_field(124, 10_000);
        tracker.observe(&hacking, &game);
        assert_eq!(tracker.hack(), (11_000, 10_000));
        tracker.observe(&snapshot(Vec::new()), &game);
        assert_eq!(tracker.hack(), (0, 0));
        assert_eq!(tracker.decoded(), 0);
    }

    #[test]
    fn a_siege_round_won_is_kept_once_with_its_winner() {
        let mut won = EntityState::zero(300, &LEGACY_ENTITY_FIELDS);
        won.set_raw_field(8, u32::from(ET_EVENTS) + u32::from(EV_SIEGE_ROUNDOVER));
        won.set_raw_field(42, 2);
        let snapshot = |server_time| sjk_protocol::Snapshot {
            message_sequence: 0,
            reliable_acknowledge: 0,
            server_commands: Vec::new(),
            server_time,
            delta_from: None,
            flags: 0,
            area_mask: Vec::new(),
            player: sjk_protocol::PlayerState::zero(),
            vehicle_player: None,
            entities: vec![won.clone()],
            consumed_bits: 0,
        };
        let (mut tracker, game) = (ObituaryTracker::new(), GameState::empty_local(1));
        assert_eq!(tracker.siege_round_over(), None);
        tracker.observe(&snapshot(5_000), &game);
        // The same event entity in the next snapshot is not a second round.
        tracker.observe(&snapshot(5_050), &game);
        assert_eq!(
            tracker.siege_round_over(),
            Some(SiegeRoundOver {
                winner: 2,
                server_time: 5_000
            })
        );
        assert_eq!(tracker.decoded(), 0);
    }

    #[test]
    fn a_latin1_name_keeps_the_players_gender() {
        let mut game = GameState::empty_local(1);
        game.replace_config_string(
            CS_PLAYERS + 4,
            b"n\\Zo\xe9\\t\\0\\model\\jan\\ds\\f".to_vec(),
        )
        .unwrap();
        let gender = client_gender(&game, 4);
        assert_eq!(gender, Gender::Female);
        let event = legacy_obituary(&self_kill(4, 38), 0, gender, 0);
        assert_eq!(event.message, "SUICIDE_FALLDEATH_FEMALE");
        assert_eq!(event.attacker_message, None);
    }
}
