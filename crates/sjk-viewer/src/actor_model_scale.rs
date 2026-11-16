//! Network actor scale is presentation policy, not movement collision policy.

/// The scale percentage and NPC class to apply to an actor: its entity state's, or
/// for the local player, who is never sent as an entity, the playerState's
/// `iModelScale` (`BG_PlayerStateToEntityState`, `bg_misc.c`): without it a
/// Ysalamiri size boost showed to everyone but the player (hub #105, SanjiJ).
pub(crate) fn source(
    state: Option<&sjk_protocol::EntityState>,
    local_player: Option<&sjk_protocol::PlayerState>,
) -> Option<(i32, u8)> {
    match (state, local_player) {
        (Some(state), _) => Some((state.model_scale_percent(), state.npc_class())),
        (None, Some(player)) => Some((player.model_scale_percent(), 0)),
        (None, None) => None,
    }
}

/// Apply CG_Player's iModelScale and non-vehicle origin correction to a fresh frame root.
///
/// OpenJK codemp/cgame/cg_players.c:8479-8493; CLASS_VEHICLE is teams.h:93.
/// Zero is the network's unit-scale sentinel, not a singular draw transform.
pub(crate) fn apply(root: &mut sjk_runtime::Transform, percent: i32, npc_class: u8) {
    if percent == 0 {
        return;
    }
    let scale = percent as f32 / 100.0;
    root.scale = [scale; 3];
    if npc_class != 53 && scale != 0.0 && scale != 1.0 {
        root.translation[2] += 24.0 * (scale - 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_local_player_takes_its_scale_from_the_player_state() {
        let mut player = sjk_protocol::PlayerState::default();
        assert!(player.set_raw_field(123, 150));
        assert_eq!(player.model_scale_percent(), 150);
        assert_eq!(source(None, Some(&player)), Some((150, 0)));
        assert_eq!(source(None, None), None);
        let mut root = sjk_runtime::Transform::IDENTITY;
        apply(&mut root, 150, 0);
        assert_eq!(root.scale, [1.5; 3]);
        assert_eq!(root.translation[2], 12.0);
    }
}
