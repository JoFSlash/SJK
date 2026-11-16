//! Players outside the local player's private duel hidden, as JoF EJK hides them.
//!
//! EternalJK `CG_Player` (`cgame/cg_players.c:11983-11990`) returns before drawing
//! anyone but the viewer and the duel partner while `cg.snap->ps.duelInProgress`, so
//! a followed duellist's view hides the same way. It does so on a JA+ server unless
//! the player's `cp_pluginDisable` sets bit 1 ("duel see others"), on a jaPRO server
//! always, and elsewhere only with `cg_stylePlayer`'s `JAPRO_STYLE_HIDENONDUELERS`.
//! A JA+ server sends bystanders to a plugin client because it expects the plugin to
//! hide them (`duel_isolation.rs`); a client without the plugin is not sent them at
//! all. The thrown saber, the blade, its glow and trail and the Force effects are
//! drawn by the owner's `CG_Player`, so they go with the body.
//!
//! The server families are a table, [`DUEL_HIDE_SERVERS`], read by
//! [`DuelHideSettings::policy`]; the player's `cg_duelHideOthers` ([`DuelHideMode`])
//! turns hiding off, follows the table (the default) or hides on every server.
//!
//! SJK hides by leaving the entities out of what is presented, as if the server had
//! not sent them ([`DuelHide::hides`]): the bodies, held and thrown sabers, the NPCs,
//! where the server says who fired them the missiles, and the one-shot events EJK
//! skips for a non-duellist (`cgame/cg_event.c`, [`EVENT_ACTORS`]). Their sounds,
//! effects and trails go with them. Movement prediction, the HUD and demos keep the
//! snapshot as received.

use crate::CompatProfile;
use sjk_protocol::{EntityState, GameState, PlayerState, Snapshot};

/// `cp_pluginDisable` bit 1, `JAPRO_PLUGIN_DUELSEEOTHERS` (EternalJK
/// `game/bg_public.h:516`): set, a JA+ server's bystanders stay visible in a duel.
pub const DUEL_SEE_OTHERS: u32 = 1 << 1;

const ET_GENERAL: u8 = 0;
const ET_PLAYER: u8 = 1;
const ET_MISSILE: u8 = 3;
const ET_NPC: u8 = 13;
/// `ET_EVENTS` (`bg_public.h:1265`): a temporary event entity is this plus its event.
const ET_EVENTS: u8 = 18;
/// `CLASS_VEHICLE` (`teams.h`).
const CLASS_VEHICLE: u8 = 53;

/// `cg_duelHideOthers`: whether players outside the duel are hidden.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DuelHideMode {
    /// 0: never; they are drawn very dark ([`crate::legacy_duel_tint`]).
    Off,
    /// 1, the default: on the servers [`DUEL_HIDE_SERVERS`] lists, as JoF EJK.
    #[default]
    Auto,
    /// 2: on every server.
    Always,
}

impl DuelHideMode {
    /// The mode a cvar value names; values out of range are Auto.
    pub fn from_cvar(value: i64) -> Self {
        match value {
            0 => Self::Off,
            2 => Self::Always,
            _ => Self::Auto,
        }
    }
}

/// When a server family's bystanders are hidden on Auto.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DuelHideRule {
    /// In every private duel.
    Always,
    /// Unless the player's `cp_pluginDisable` sets this bit.
    UnlessPluginDisables(u32),
}

/// A server family that hides bystanders during a private duel.
#[derive(Debug, Eq, PartialEq)]
pub struct DuelHideServer {
    /// The family's [`CompatProfile::id`].
    pub profile: &'static str,
    /// Its name, as the docs and the setting's help name it.
    pub name: &'static str,
    /// When its bystanders are hidden on Auto.
    pub rule: DuelHideRule,
    /// Whether its missiles carry their owner (`entityState_t::owner`), so a
    /// bystander's shots can be hidden too.
    pub missile_owners: bool,
}

/// The server families whose bystanders are hidden during a private duel on Auto.
pub const DUEL_HIDE_SERVERS: &[DuelHideServer] = &[
    // A JA+ server sends bystanders to a plugin client and leaves them to the plugin
    // to hide, unless `cp_pluginDisable` asks to see them (EternalJK
    // `cg_players.c:11987`). Stock `CreateMissile` leaves `s.owner` unset.
    DuelHideServer {
        profile: "japlus",
        name: "JA+",
        rule: DuelHideRule::UnlessPluginDisables(DUEL_SEE_OTHERS),
        missile_owners: false,
    },
    // jaPRO (TaystJK) always hides them (`cg_players.c:11987`), and sets a missile's
    // `s.owner` so that clients can hide a bystander's shots (EternalJK
    // `game/g_missile.c:319-320`, `CG_DuelCull`, `cg_ents.c:2982`).
    DuelHideServer {
        profile: "taystjk",
        name: "jaPRO",
        rule: DuelHideRule::Always,
        missile_owners: true,
    },
];

/// The player's duel-hiding choice and the plugin bits it reads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DuelHideSettings {
    /// `cg_duelHideOthers`.
    pub mode: DuelHideMode,
    /// The `cp_pluginDisable` the server is sent ([`crate::PLUGIN_DISABLE_DEFAULT`]
    /// when unset).
    pub plugin_disable: u32,
}

impl Default for DuelHideSettings {
    fn default() -> Self {
        Self {
            mode: DuelHideMode::Auto,
            plugin_disable: crate::PLUGIN_DISABLE_DEFAULT,
        }
    }
}

/// What is hidden on one server.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DuelHidePolicy {
    /// Missiles are hidden by their owner.
    pub missiles: bool,
}

impl DuelHideSettings {
    /// Whether bystanders are hidden on a server of `profile`, and how.
    pub fn policy(&self, profile: &CompatProfile) -> Option<DuelHidePolicy> {
        let family = DUEL_HIDE_SERVERS
            .iter()
            .find(|family| family.profile == profile.id());
        let missiles = family.is_some_and(|family| family.missile_owners);
        let hides = match self.mode {
            DuelHideMode::Off => false,
            DuelHideMode::Always => true,
            DuelHideMode::Auto => family.is_some_and(|family| match family.rule {
                DuelHideRule::Always => true,
                DuelHideRule::UnlessPluginDisables(bit) => self.plugin_disable & bit == 0,
            }),
        };
        hides.then_some(DuelHidePolicy { missiles })
    }
}

/// Who a temporary event entity is from, when EJK skips it for a non-duellist.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventActor {
    /// `es->clientNum`.
    Client,
    /// `es->otherEntityNum2` (the saber's wielder).
    OtherEntity2,
}

/// The events EternalJK `CG_EntityEvent` skips while the viewer duels when they come
/// from someone outside the duel (`cgame/cg_event.c`), with who they come from.
/// Events EJK tells by `es->number` (weapon fire, no ammo, weapon change, charge,
/// gib) ride the player's own entity, which is hidden whole.
pub const EVENT_ACTORS: &[(u16, EventActor)] = &[
    (2, EventActor::Client),        // EV_FOOTSTEP, cg_event.c:1915
    (11, EventActor::Client),       // EV_FALL, :2000
    (16, EventActor::Client),       // EV_JUMP, :2110
    (17, EventActor::Client),       // EV_ROLL, :2165
    (29, EventActor::Client),       // EV_SABER_ATTACK, :2803
    (30, EventActor::OtherEntity2), // EV_SABER_HIT, :2842
    (31, EventActor::OtherEntity2), // EV_SABER_BLOCK, :2988
    (33, EventActor::Client),       // EV_SABER_UNHOLSTER, :3090
    (39, EventActor::Client),       // EV_DISRUPTOR_ZOOMSOUND, :3253
    (64, EventActor::Client),       // EV_PLAYER_TELEPORT_IN, :3467
    (65, EventActor::Client),       // EV_PLAYER_TELEPORT_OUT, :3501
    (89, EventActor::Client),       // EV_PAIN, :4204
    (90, EventActor::Client),       // EV_DEATH1, :4218
    (91, EventActor::Client),       // EV_DEATH2
    (92, EventActor::Client),       // EV_DEATH3
    (115, EventActor::Client),      // EV_TAUNT, :2202
];

/// The duel of one snapshot's player whose outsiders are hidden.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DuelHide {
    /// The viewer (`ps.clientNum`, the followed player when following).
    pub client: u16,
    /// The duel partner (`ps.duelIndex`).
    pub partner: u16,
    /// Missiles are hidden by their owner.
    pub missiles: bool,
}

impl DuelHide {
    /// The hiding for `player`, if it duels and `policy` hides.
    pub fn new(player: &PlayerState, policy: Option<DuelHidePolicy>) -> Option<Self> {
        let policy = policy?;
        player.duel_in_progress().then(|| Self {
            client: player.client_num(),
            partner: player.duel_index(),
            missiles: policy.missiles,
        })
    }

    /// The hiding for `snapshot` on the server of `game`.
    pub fn for_snapshot(
        snapshot: &Snapshot,
        game: &GameState,
        settings: DuelHideSettings,
    ) -> Option<Self> {
        if settings.mode == DuelHideMode::Off || !snapshot.player.duel_in_progress() {
            return None;
        }
        Self::new(
            &snapshot.player,
            settings.policy(&CompatProfile::from_game_state(game)),
        )
    }

    fn in_duel(&self, number: u16) -> bool {
        number == self.client || number == self.partner
    }

    /// Whether `state` is left out of what is presented.
    pub fn hides(&self, state: &EntityState) -> bool {
        match state.entity_type() {
            ET_PLAYER => !self.in_duel(state.number()),
            // Every NPC but the viewer's own vehicle (`cg_players.c:11983`).
            ET_NPC => !(state.npc_class() == CLASS_VEHICLE && state.owner() == self.client),
            ET_GENERAL => {
                crate::legacy_thrown_saber_owner(state).is_some_and(|owner| !self.in_duel(owner))
            }
            ET_MISSILE => self.missiles && !self.in_duel(state.owner()),
            kind if kind >= ET_EVENTS => {
                let event = u16::from(kind - ET_EVENTS);
                EVENT_ACTORS
                    .iter()
                    .find(|(listed, _)| *listed == event)
                    .is_some_and(|(_, actor)| {
                        !self.in_duel(match actor {
                            EventActor::Client => state.client_num(),
                            EventActor::OtherEntity2 => state.other_entity_num2(),
                        })
                    })
            }
            _ => false,
        }
    }
}

/// `snapshot` without what [`DuelHide`] hides, or `None` when nothing is hidden.
pub fn duel_hidden_snapshot(
    snapshot: &Snapshot,
    game: &GameState,
    settings: DuelHideSettings,
) -> Option<Snapshot> {
    let hide = DuelHide::for_snapshot(snapshot, game, settings)?;
    if !snapshot.entities.iter().any(|state| hide.hides(state)) {
        return None;
    }
    let mut hidden = snapshot.clone();
    hidden.entities.retain(|state| !hide.hides(state));
    Some(hidden)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjk_protocol::LEGACY_ENTITY_FIELDS;

    fn entity(number: u16, kind: u8) -> EntityState {
        let mut state = EntityState::zero(number, &LEGACY_ENTITY_FIELDS);
        state.set_raw_field(8, u32::from(kind));
        state
    }

    fn with(mut state: EntityState, field: usize, value: u32) -> EntityState {
        state.set_raw_field(field, value);
        state
    }

    /// Client 0 duelling client 1, missiles told by owner when `missiles`.
    fn duel(missiles: bool) -> DuelHide {
        DuelHide {
            client: 0,
            partner: 1,
            missiles,
        }
    }

    fn settings(mode: DuelHideMode, plugin_disable: u32) -> DuelHideSettings {
        DuelHideSettings {
            mode,
            plugin_disable,
        }
    }

    #[test]
    fn the_mode_follows_the_cvar() {
        assert_eq!(DuelHideMode::from_cvar(0), DuelHideMode::Off);
        assert_eq!(DuelHideMode::from_cvar(1), DuelHideMode::Auto);
        assert_eq!(DuelHideMode::from_cvar(2), DuelHideMode::Always);
        assert_eq!(DuelHideMode::from_cvar(7), DuelHideMode::Auto);
        assert_eq!(DuelHideMode::from_cvar(-1), DuelHideMode::Auto);
        assert_eq!(DuelHideSettings::default().mode, DuelHideMode::Auto);
    }

    #[test]
    fn auto_hides_on_ja_plus_unless_the_plugin_bit_and_always_on_japro() {
        let ja_plus = CompatProfile::JaPlus { version: None };
        let auto = |bits| settings(DuelHideMode::Auto, bits);
        // SJK's default bits (1536) leave bit 1 clear: hidden, as in EJK.
        let hidden = Some(DuelHidePolicy { missiles: false });
        assert_eq!(DuelHideSettings::default().policy(&ja_plus), hidden);
        assert_eq!(auto(DUEL_SEE_OTHERS | 1_536).policy(&ja_plus), None);
        let japro = Some(DuelHidePolicy { missiles: true });
        assert_eq!(auto(0).policy(&CompatProfile::TaystJk), japro);
        assert_eq!(auto(DUEL_SEE_OTHERS).policy(&CompatProfile::TaystJk), japro);
        assert_eq!(auto(0).policy(&CompatProfile::BaseJka), None);
        assert_eq!(auto(0).policy(&CompatProfile::Unknown("x".into())), None);
    }

    #[test]
    fn off_never_hides_and_always_hides_everywhere() {
        let ja_plus = CompatProfile::JaPlus { version: None };
        for profile in [&ja_plus, &CompatProfile::TaystJk, &CompatProfile::BaseJka] {
            assert_eq!(settings(DuelHideMode::Off, 0).policy(profile), None);
        }
        let always = settings(DuelHideMode::Always, DUEL_SEE_OTHERS);
        let shown = Some(DuelHidePolicy { missiles: false });
        assert_eq!(always.policy(&ja_plus), shown);
        assert_eq!(always.policy(&CompatProfile::BaseJka), shown);
        let japro = Some(DuelHidePolicy { missiles: true });
        assert_eq!(always.policy(&CompatProfile::TaystJk), japro);
    }

    #[test]
    fn every_family_names_a_profile() {
        for family in DUEL_HIDE_SERVERS {
            assert!(
                ["japlus", "taystjk"].contains(&family.profile),
                "{}",
                family.name
            );
        }
        assert_eq!(CompatProfile::JaPlus { version: None }.id(), "japlus");
        assert_eq!(CompatProfile::TaystJk.id(), "taystjk");
    }

    #[test]
    fn only_a_duelling_player_hides() {
        let policy = Some(DuelHidePolicy { missiles: false });
        let mut player = PlayerState::default();
        player.set_client_num(3);
        assert_eq!(DuelHide::new(&player, policy), None);
        player.set_raw_field(119, 1);
        player.set_raw_field(44, 5);
        let hide = DuelHide::new(&player, policy).unwrap();
        assert_eq!((hide.client, hide.partner), (3, 5));
        assert_eq!(DuelHide::new(&player, None), None);
    }

    #[test]
    fn bystanders_bodies_and_npcs_hide_and_the_pair_stays() {
        let hide = duel(false);
        assert!(!hide.hides(&entity(0, ET_PLAYER)));
        assert!(!hide.hides(&entity(1, ET_PLAYER)));
        assert!(hide.hides(&entity(2, ET_PLAYER)));
        assert!(hide.hides(&entity(40, ET_NPC)));
        // The viewer's own vehicle stays, another's goes.
        let vehicle = with(entity(41, ET_NPC), 88, u32::from(CLASS_VEHICLE));
        assert!(!hide.hides(&with(vehicle.clone(), 40, 0)));
        assert!(hide.hides(&with(vehicle, 40, 2)));
        // Movers, items and the rest of the world are not touched.
        assert!(!hide.hides(&entity(60, 4)));
        assert!(!hide.hides(&entity(61, 2)));
    }

    #[test]
    fn a_bystanders_thrown_saber_hides_and_the_pairs_stays() {
        let saber =
            |number, owner: u32| with(with(entity(number, ET_GENERAL), 14, 3), 18, 1_024 + owner);
        let hide = duel(false);
        assert!(!hide.hides(&saber(200, 0)));
        assert!(!hide.hides(&saber(201, 1)));
        assert!(hide.hides(&saber(202, 2)));
        // An NPC's saber goes with the NPC.
        assert!(hide.hides(&saber(203, 40)));
        // A general entity that is no saber stays.
        assert!(!hide.hides(&entity(204, ET_GENERAL)));
    }

    #[test]
    fn missiles_hide_by_owner_only_where_the_server_says_whose() {
        let rocket = |owner| with(entity(300, ET_MISSILE), 40, owner);
        assert!(!duel(true).hides(&rocket(0)));
        assert!(!duel(true).hides(&rocket(1)));
        assert!(duel(true).hides(&rocket(2)));
        // Without owners every missile stays (stock leaves `owner` 0).
        assert!(!duel(false).hides(&rocket(2)));
    }

    #[test]
    fn a_bystanders_events_hide_as_ejk_skips_them() {
        let event = |kind: u16| entity(500, ET_EVENTS + kind as u8);
        let hide = duel(false);
        // EV_PLAYER_TELEPORT_OUT by clientNum.
        assert!(hide.hides(&with(event(65), 32, 2)));
        assert!(!hide.hides(&with(event(65), 32, 1)));
        // EV_SABER_HIT by its wielder.
        assert!(hide.hides(&with(event(30), 39, 2)));
        assert!(!hide.hides(&with(event(30), 39, 0)));
        // EV_SABER_BLOCK likewise.
        assert!(hide.hides(&with(event(31), 39, 7)));
        // Events EJK keeps: an obituary, a global sound, a clash flare.
        assert!(!hide.hides(&with(event(93), 32, 2)));
        assert!(!hide.hides(&with(event(77), 32, 2)));
        assert!(!hide.hides(&with(event(32), 39, 2)));
        for (listed, _) in EVENT_ACTORS {
            assert!(*listed < 256 - u16::from(ET_EVENTS));
        }
    }

    #[test]
    fn the_hidden_snapshot_drops_only_what_hides() {
        let mut snapshot = Snapshot {
            message_sequence: 1,
            reliable_acknowledge: 0,
            server_commands: Vec::new(),
            server_time: 1_000,
            delta_from: None,
            flags: 0,
            area_mask: Vec::new(),
            player: PlayerState::default(),
            vehicle_player: None,
            entities: vec![entity(1, ET_PLAYER), entity(2, ET_PLAYER), entity(60, 4)],
            consumed_bits: 0,
        };
        let game = GameState::empty_local(0);
        let always = settings(DuelHideMode::Always, 0);
        // Not duelling: nothing.
        assert!(duel_hidden_snapshot(&snapshot, &game, always).is_none());
        snapshot.player.set_raw_field(119, 1);
        snapshot.player.set_raw_field(44, 1);
        let hidden = duel_hidden_snapshot(&snapshot, &game, always).unwrap();
        let numbers: Vec<u16> = hidden.entities.iter().map(EntityState::number).collect();
        assert_eq!(numbers, [1, 60]);
        // Off, or nothing to hide: the snapshot as received.
        let off = settings(DuelHideMode::Off, 0);
        assert!(duel_hidden_snapshot(&snapshot, &game, off).is_none());
        snapshot.entities.remove(1);
        assert!(duel_hidden_snapshot(&snapshot, &game, always).is_none());
    }
}
