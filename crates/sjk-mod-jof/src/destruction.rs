//! JoF EJK #338/#389 Destruction client contract, loaded by the JoF mod only.
use sjk_client::force_wheel::Ability;
use sjk_protocol::GameState;
/// Archived JoF override: -1 server flag, 0 off, 1 accept its grant without flag.
pub const CVAR: &str = "jof.destructionSupport";
/// Local-only wheel slot; never sent as protocol Force selection.
pub const SLOT: u8 = 21;
/// Original server command retained for existing binds.
pub const COMMAND: &str = "force_destruction";
/// Display name supplied to the host UI.
pub const LABEL: &str = "Destruction";
/// Optional installed icon; the host may fall back to stock Lightning.
pub const ICON: &str = "gfx/forcedestruction/force_destruction";
/// Evidence: EJK #338 CG_HasDestruction / CG_BuildForceWheel / CL_CmdButtons.
pub const ENTRIES: &[Ability] = &[Ability {
    slot: SLOT,
    anchor: 7,
    before: true,
    label: LABEL,
    icon: Some(ICON),
    fallback: 7,
    command: Some(COMMAND),
    server_flag: Some("g_forceDestruction"),
    live_only: true,
}];
/// Apply only after the JoF mod's server/switch gate. The override cannot grant
/// a power, and the engine additionally filters dead/spectating/following players.
pub fn entries(game: Option<&GameState>, override_value: i64) -> &'static [Ability] {
    let info = sjk_client::LegacyClientInfo::new(
        game.and_then(|game| game.config_string(0))
            .unwrap_or_default(),
    );
    let supported = ENTRIES.iter().all(|entry| {
        entry
            .server_flag
            .is_none_or(|key| info.integer(key).is_some_and(|value| value != 0))
    });
    if override_value == 0 || (override_value != 1 && !supported) {
        &[]
    } else {
        ENTRIES
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use sjk_client::{force_wheel, selection::Selection};
    #[test]
    fn capability_grant_and_live_player_are_all_required_and_slot_stays_local() {
        let mut game = GameState::empty_local(0);
        game.replace_config_string(0, br"\g_forceDestruction\1".to_vec())
            .unwrap();
        assert!(entries(None, -1).is_empty());
        assert!(!entries(Some(&game), -1).is_empty());
        assert!(entries(Some(&game), 0).is_empty());
        assert!(!entries(None, 1).is_empty());
        for step in [8, 7, 4, 3] {
            let mut player = sjk_protocol::PlayerState::default();
            player.stats[0] = 100;
            player.set_raw_field(51, (1 << 3) | (1 << SLOT));
            let mut selection = Selection::default();
            assert!(!selection.select(&player, 0, SLOT));
            selection.set_extensions(ENTRIES);
            assert!(selection.select(&player, step, 3));
            assert!(selection.select(&player, step * 2, SLOT));
            assert_eq!(selection.force, Some(3));
            let mut remap = force_wheel::UseRemap::default();
            assert_eq!(
                remap.apply_with(1 << 9, Some(SLOT), selection.known(&player), false, ENTRIES),
                (0, Some(COMMAND))
            );
            assert_eq!(
                remap.apply_with(1 << 9, Some(SLOT), selection.known(&player), false, ENTRIES),
                (0, None)
            );
            for (field, value) in [(63, 3), (63, 4), (38, 4096)] {
                let mut blocked = player.clone();
                blocked.set_raw_field(field, value);
                assert_eq!(force_wheel::known(&blocked, ENTRIES) & (1 << SLOT), 0);
            }
            player.stats[0] = 0;
            assert_eq!(selection.known(&player) & (1 << SLOT), 0);
            selection.set_extensions(&[]);
            selection.sync(&player, step * 3);
            assert_eq!(selection.wheel_pseudo(), None);
        }
    }
    #[test]
    fn all_anchor_and_grant_combinations_match_ejk_with_no_base_activation() {
        let reference = [
            5, 0, 10, 9, 11, 20, 2, 3, 4, 14, 18, 19, 21, 7, 13, 8, 6, 12,
        ];
        assert!(!force_wheel::valid(u32::MAX, SLOT));
        assert_eq!(force_wheel::build(u32::MAX).1, 17);
        for grants in 0u32..16 {
            for anchors in 0u32..8 {
                let known = (grants << 18)
                    | ((anchors & 1) << 2)
                    | (((anchors >> 1) & 1) << 14)
                    | (((anchors >> 2) & 1) << 7);
                let (slots, count) = force_wheel::build_with(known, ENTRIES);
                let expected: Vec<_> = reference
                    .into_iter()
                    .filter(|slot| known & (1 << slot) != 0)
                    .collect();
                assert_eq!(&slots[..count], expected);
            }
        }
    }
}
