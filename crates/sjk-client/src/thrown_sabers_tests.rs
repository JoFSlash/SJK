use super::*;
use sjk_protocol::{LEGACY_ENTITY_FIELDS, PlayerState};

const CS_MODELS: usize = 298;
const CS_PLAYERS: usize = 1_131;

fn snapshot(entities: Vec<EntityState>) -> Snapshot {
    let mut player = PlayerState::zero();
    player.set_client_num(0);
    Snapshot {
        message_sequence: 1,
        reliable_acknowledge: 0,
        server_commands: Vec::new(),
        server_time: 1_000,
        delta_from: None,
        flags: 0,
        area_mask: Vec::new(),
        player,
        vehicle_player: None,
        entities,
        consumed_bits: 0,
    }
}

/// `w_saber.c`'s flying saber: `ET_GENERAL`, `WP_SABER`, owner + 1024 in netfield 18.
fn thrown_saber(number: u16, owner: u16, in_flight: bool) -> EntityState {
    let mut state = EntityState::zero(number, &LEGACY_ENTITY_FIELDS);
    state.set_raw_field(14, 3);
    state.set_raw_field(18, 1_024 + u32::from(owner));
    state.set_raw_field(81, u32::from(in_flight));
    state
}

/// An `ET_NPC` throwing its saber, `npcSaber1` naming `CS_MODELS + 7`.
fn npc(number: u16, bolt_to_player: u8) -> EntityState {
    let mut state = EntityState::zero(number, &LEGACY_ENTITY_FIELDS);
    state.set_raw_field(8, u32::from(ET_NPC));
    state.set_raw_field(14, 3);
    state.set_raw_field(72, 7);
    state.set_raw_field(81, 1);
    state.set_raw_field(101, u32::from(bolt_to_player));
    state
}

fn game_state() -> GameState {
    let mut game = GameState::empty_local(0);
    game.replace_config_string(CS_MODELS + 7, b"@Desann".to_vec())
        .unwrap();
    game.replace_config_string(CS_PLAYERS + 3, b"n\\Player\\st\\single_2\\c1\\4".to_vec())
        .unwrap();
    game
}

#[test]
fn an_npc_owns_its_thrown_saber() {
    assert_eq!(
        legacy_thrown_saber_owner(&thrown_saber(300, 120, true)),
        Some(120)
    );
    assert_eq!(
        legacy_thrown_saber_owner(&thrown_saber(300, 1_023, true)),
        Some(1_023)
    );
    let mut beyond = thrown_saber(300, 0, true);
    beyond.set_raw_field(18, 1_024 + 1_024);
    assert_eq!(legacy_thrown_saber_owner(&beyond), None);
}

#[test]
fn an_npc_thrown_saber_takes_its_npc_saber_and_packed_colour() {
    let game = game_state();
    // Saber 0 packed as SABER_BLUE (4) + 1.
    let snapshot = snapshot(vec![npc(120, 5), thrown_saber(300, 120, true)]);
    let sabers = LegacyThrownSabers::new(&snapshot, &game);
    let thrown = sabers.get(300).expect("the NPC's saber is drawn in flight");
    assert_eq!(thrown.owner, 120);
    assert_eq!(thrown.name, "Desann");
    assert_eq!(
        thrown.color,
        crate::presentation_equipment::legacy_blade_rgb(4)
    );
    assert!(thrown.in_flight);
    assert!(!thrown.returning);
    assert!(!thrown.definition_color);
    // Without an override the hilt's `.sab` colour applies.
    let snapshot = self::snapshot(vec![npc(120, 0), thrown_saber(300, 120, true)]);
    let sabers = LegacyThrownSabers::new(&snapshot, &game);
    assert!(sabers.get(300).unwrap().definition_color);
}

#[test]
fn a_thrown_saber_without_its_npc_draws_nothing() {
    let game = game_state();
    let snapshot = snapshot(vec![thrown_saber(300, 120, true)]);
    assert!(LegacyThrownSabers::new(&snapshot, &game).get(300).is_none());
    // An owner number that is not an NPC (a mover, say) has no saber either.
    let mut mover = npc(120, 0);
    mover.set_raw_field(8, 4);
    let snapshot = self::snapshot(vec![mover, thrown_saber(300, 120, true)]);
    assert!(LegacyThrownSabers::new(&snapshot, &game).get(300).is_none());
}

#[test]
fn a_player_thrown_saber_still_reads_clientinfo() {
    let game = game_state();
    let mut owner = EntityState::zero(3, &LEGACY_ENTITY_FIELDS);
    owner.set_raw_field(8, 1);
    owner.set_raw_field(81, 1);
    let snapshot = snapshot(vec![owner, thrown_saber(300, 3, true)]);
    let sabers = LegacyThrownSabers::new(&snapshot, &game);
    let thrown = sabers.get(300).expect("player saber");
    assert_eq!(thrown.owner, 3);
    assert_eq!(thrown.name, "single_2");
}
