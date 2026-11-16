use super::*;
use crate::player_profile::SaberColor;
use sjk_protocol::LEGACY_ENTITY_FIELDS;

/// An `ET_NPC` holding a saber, its saber entity `saber_entity` and `boltToPlayer`.
fn npc(saber_entity: u16, bolt_to_player: u8) -> EntityState {
    let mut state = EntityState::zero(120, &LEGACY_ENTITY_FIELDS);
    state.set_raw_field(8, u32::from(ET_NPC));
    state.set_raw_field(14, 3);
    state.set_raw_field(37, u32::from(saber_entity));
    state.set_raw_field(101, u32::from(bolt_to_player));
    state
}

#[test]
fn without_an_override_both_blades_take_the_definition_colour() {
    assert_eq!(legacy_npc_saber_color_overrides(&npc(300, 0)), [None, None]);
    // Until the hilt's colour replaces it, the stand-in is `WP_SaberSetDefaults`' red.
    let held = legacy_npc_equipment(&npc(300, 0), true).unwrap();
    assert_eq!(held.color, legacy_blade_rgb(0));
}

#[test]
fn a_packed_override_names_each_saber_and_a_missing_one_is_blue() {
    // Saber 0 green (3 + 1), saber 1 unset while boltToPlayer is non-zero: stock's -1.
    let overrides = legacy_npc_saber_color_overrides(&npc(300, 4));
    assert_eq!(overrides[0], Some(legacy_blade_rgb(3)));
    assert_eq!(overrides[1], Some(legacy_blade_rgb(SABER_BLUE)));
    let held = legacy_npc_equipment(&npc(300, 4), true).unwrap();
    assert_eq!(held.color, legacy_blade_rgb(3));
    assert_eq!(held.secondary_color, legacy_blade_rgb(SABER_BLUE));
}

#[test]
fn definition_colours_are_fixed_or_steady_random() {
    assert_eq!(
        legacy_npc_definition_rgb(LegacySaberColor::Fixed(SaberColor::Purple), 120),
        legacy_blade_rgb(5)
    );
    let random = legacy_npc_definition_rgb(LegacySaberColor::Random, 120);
    assert_eq!(
        random,
        legacy_npc_definition_rgb(LegacySaberColor::Random, 120)
    );
    // Orange to purple, never red.
    assert!((0..1_024).all(|number| {
        legacy_npc_definition_rgb(LegacySaberColor::Random, number) != legacy_blade_rgb(0)
    }));
}

#[test]
fn an_npc_without_a_saber_entity_draws_no_primary() {
    assert!(legacy_npc_primary_saber_missing(&npc(0, 0)));
    assert!(!legacy_npc_primary_saber_missing(&npc(300, 0)));
    // Players are left as they are.
    let mut player = npc(0, 0);
    player.set_raw_field(8, 1);
    assert!(!legacy_npc_primary_saber_missing(&player));
}
