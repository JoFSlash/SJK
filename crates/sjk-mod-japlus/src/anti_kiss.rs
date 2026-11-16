//! Anti-kiss (`japlus.antiAmKiss`): JoF EternalJK's `cg_antiAmKiss` (`cg_view.c:2635-2640`, in
//! `CG_DrawActiveFrame`). When another player's JA+ `amkiss` puts the local player in
//! the kiss-victim animation, the client uses Force Push once, which breaks the pair.
//!
//! Off by default and archived. The client asks this module each frame while JA+
//! tools are loaded and the setting is on ([`kiss_is_active`], [`Latch`]); the
//! response is the generic command [`RESPONSE`], which the client's input puts in the
//! next user command as a bound `force_throw` would. One push per interaction, so
//! the initiator's animation is not disturbed over and over. Spectators and a player
//! following another never push.

use sjk_mod::ServerKind;
use sjk_protocol::PlayerState;

/// The setting.
pub const CVAR: &str = "japlus.antiAmKiss";

/// JoF EternalJK's name for it, kept as an alias so a config made for it still works.
pub const EJK_CVAR: &str = "cg_antiAmKiss";

/// What the client sends in response: the bind command whose `genCmds_t` value
/// the next user command carries (`GENCMD_FORCE_THROW`).
pub const RESPONSE: &str = "force_throw";

/// `BOTH_KISSEE`, the victim's half of the paired kiss animation, as the wire
/// numbers animations (`anims.h`; the client's tests pin it to its animation table).
pub const KISS_VICTIM_ANIMATION: u16 = 1493;

/// `ANIM_TOGGLEBIT`: flipped when the same animation restarts.
pub const ANIM_TOGGLEBIT: u16 = 0x800;

/// `PMF_FOLLOW`: the player state is another player's, followed.
pub const PMF_FOLLOW: u16 = 0x1000;

/// `PM_SPECTATOR`.
pub const PM_SPECTATOR: u8 = 4;

/// A server family anti-kiss answers on, with the evidence.
#[derive(Debug)]
pub struct ServerCase {
    pub kind: ServerKind,
    pub name: &'static str,
    pub evidence: &'static str,
}

/// Where the kiss-victim animation means a JA+ `amkiss` and Force Push answers it.
/// Supporting another family is one more entry.
pub const SERVER_CASES: &[ServerCase] = &[ServerCase {
    kind: ServerKind::JaPlus,
    name: "JA+ (JoF's included)",
    evidence: "JoF EJK cg_view.c:2635-2640; Lukas's live JoF JA+ check, 19/10/2026 (SJK #190)",
}];

/// Whether anti-kiss answers on a server of `kind` ([`SERVER_CASES`]).
pub fn applies_on(kind: ServerKind) -> bool {
    SERVER_CASES.iter().any(|case| case.kind == kind)
}

fn is_kiss_victim(animation: u16) -> bool {
    animation & !ANIM_TOGGLEBIT == KISS_VICTIM_ANIMATION
}

/// Whether the local player is a kiss victim now: the server's player state (not a
/// followed player's, not a spectator's) or the predicted legs and torso animations
/// show `BOTH_KISSEE`.
pub fn kiss_is_active(authoritative: &PlayerState, predicted: Option<[u16; 2]>) -> bool {
    if authoritative.movement_flags() & PMF_FOLLOW != 0
        || authoritative.movement_type() == PM_SPECTATOR
    {
        return false;
    }
    predicted.is_some_and(|animations| animations.into_iter().any(is_kiss_victim))
        || is_kiss_victim(authoritative.leg_animation())
        || is_kiss_victim(authoritative.torso_animation())
}

/// Pushes once per interaction: on the frame the victim animation starts.
#[derive(Debug, Default)]
pub struct Latch {
    kissed: bool,
}

impl Latch {
    /// Whether to push now, given whether the kiss is active this frame.
    pub fn observe(&mut self, kissed: bool) -> bool {
        let push = kissed && !self.kissed;
        self.kissed = kissed;
        push
    }

    /// Forget the interaction (the setting off, the mod unloaded, no game).
    pub fn clear(&mut self) {
        self.kissed = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn authoritative(animation: u16) -> PlayerState {
        let mut state = PlayerState::default();
        assert!(state.set_raw_field(13, u32::from(animation)));
        assert!(state.set_raw_field(15, u32::from(animation)));
        state
    }

    #[test]
    fn the_victim_animation_pushes_on_ja_plus_only() {
        assert!(applies_on(ServerKind::JaPlus));
        for kind in [ServerKind::BaseJka, ServerKind::JaPro, ServerKind::Other] {
            assert!(!applies_on(kind), "{kind:?}");
        }
        let kissed = [KISS_VICTIM_ANIMATION; 2];
        assert!(kiss_is_active(&PlayerState::default(), Some(kissed)));
        assert!(kiss_is_active(&authoritative(KISS_VICTIM_ANIMATION), None));
        assert!(!kiss_is_active(&PlayerState::default(), Some([0, 0])));
        assert!(!kiss_is_active(&PlayerState::default(), None));
    }

    #[test]
    fn following_and_spectating_never_push() {
        let kissed = [KISS_VICTIM_ANIMATION; 2];
        let mut following = authoritative(KISS_VICTIM_ANIMATION);
        assert!(following.set_raw_field(38, u32::from(PMF_FOLLOW)));
        assert!(!kiss_is_active(&following, Some(kissed)));
        let mut spectator = authoritative(KISS_VICTIM_ANIMATION);
        assert!(spectator.set_raw_field(63, u32::from(PM_SPECTATOR)));
        assert!(!kiss_is_active(&spectator, Some(kissed)));
    }

    #[test]
    fn animation_toggle_bit_does_not_hide_an_interaction() {
        let toggled = KISS_VICTIM_ANIMATION | ANIM_TOGGLEBIT;
        assert!(kiss_is_active(
            &PlayerState::default(),
            Some([toggled, toggled])
        ));
    }

    #[test]
    fn kiss_pushes_once_until_the_animation_ends() {
        let mut latch = Latch::default();
        assert!(latch.observe(true));
        assert!(!latch.observe(true));
        assert!(!latch.observe(false));
        assert!(latch.observe(true));
        latch.clear();
        assert!(latch.observe(true));
    }
}
