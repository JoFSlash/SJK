//! A view the game server holds: a melee grapple or throw, a saber lock and, on a JA+
//! server, its kisses, kicks and ledges, and its knockdowns and get-ups except on the
//! servers known to leave those free ([`FREE_KNOCKDOWN_VIEW`], [`KNOCKDOWN_CVAR`])
//! ([`sjk_game_jka::prediction_policy::server_holds_view`], the table of cases with their
//! evidence). The server rewrites `delta_angles` every frame then, so anything else that
//! turns the view fights it:
//!
//! - the mouse: the view shook as the player looked around while kicked down (Rollembones'
//!   report #146). With [`LOOK_CVAR`] off the mouse does not turn a held view (on by default);
//! - a toy whose commands carry their own angles (the Ragdoll's walk, the Hax's steering):
//!   each server rewrite turned the camera on by the gap between the two views, so it spun
//!   (#139, #140). The toys stop while the view is held ([`crate::ragdoll`],
//!   [`crate::hax`]).

use crate::GpuState;

/// `1` lets the mouse turn a view the server holds, as before (it shakes).
pub(crate) const LOOK_CVAR: &str = "cg_heldViewLook";
/// [`LOOK_CVAR`]'s default: the mouse turns a held view, as in the base game (Sol: a
/// locked camera while down is worse than the shake).
pub(crate) const DEFAULT_LOOK: bool = true;

/// `-1` (the default) leaves the view through JA+ knockdowns and get-ups to
/// [`FREE_KNOCKDOWN_VIEW`]; `0` has the server hold it, as EternalJK's cgame does; `1`
/// leaves it free, on any JA+ server.
pub(crate) const KNOCKDOWN_CVAR: &str = "cg_japlusFreeKnockdownView";

/// A server family whose JA+ server leaves the view free through knockdowns and their
/// get-ups (AGENTS.md, no hardcoded special cases; the families are
/// `sjk_mod::known_servers`'). Other JA+ servers keep EternalJK's
/// `CG_JAPlusViewLockedState`, which holds the view there, as some do.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FreeKnockdownFamily {
    /// The family's id in `sjk_mod::known_servers`.
    pub(crate) family: &'static str,
    /// Its name, as the family's.
    pub(crate) name: &'static str,
}

/// The families known to leave the view free through knockdowns and get-ups.
pub(crate) const FREE_KNOCKDOWN_VIEW: &[FreeKnockdownFamily] = &[
    // Two demos of JoF's knockdowns (27/10/2026: `BOTH_KNOCKDOWN1`, `BOTH_GETUP1`, the
    // get-up rolls, `BOTH_FORCE_GETUP_B1`): `delta_angles` never changed and the view
    // followed the mouse. Held as EternalJK does, the pmove pulled the camera back to the
    // server's lagging view every snapshot (Rollembones' hub report #164), and
    // `cg_heldViewLook 0` locked it (Kushina's #169).
    FreeKnockdownFamily {
        family: "jof",
        name: "JoF",
    },
];

/// Whether the server leaves the view free through knockdowns and get-ups: by
/// [`KNOCKDOWN_CVAR`] when set, else by the family of the server joined.
pub(crate) fn free_knockdown_view(console: Option<&crate::console::ViewerConsole>) -> bool {
    let Some(console) = console else {
        return false; // evidence runs keep EternalJK's rule
    };
    knockdown_view_free(
        console.integer_cvar(KNOCKDOWN_CVAR),
        console.server_family(),
    )
}

/// [`free_knockdown_view`] from [`KNOCKDOWN_CVAR`]'s value and the server's family.
fn knockdown_view_free(setting: Option<i64>, family: Option<&sjk_mod::KnownServer>) -> bool {
    match setting {
        Some(0) => false,
        Some(1) => true,
        _ => free_knockdown_family(family).is_some(),
    }
}

/// The family of [`FREE_KNOCKDOWN_VIEW`] `family` is, if any.
fn free_knockdown_family(
    family: Option<&sjk_mod::KnownServer>,
) -> Option<&'static FreeKnockdownFamily> {
    let family = family?;
    FREE_KNOCKDOWN_VIEW
        .iter()
        .find(|free| free.family == family.id)
}

impl GpuState {
    /// Whether the game server holds the player's own view in the latest snapshot.
    pub(crate) fn server_holds_view(&self) -> bool {
        let Some(session) = self.live_session.as_ref() else {
            return false;
        };
        let player = &session.latest_snapshot().player;
        if player.is_spectator()
            || !crate::local_prediction::predicts_local_view(player.movement_flags())
        {
            return false;
        }
        sjk_game_jka::prediction_policy::server_holds_view(player, || {
            sjk_game_jka::pmove_japlus::JaPlusRules {
                free_knockdown_view: free_knockdown_view(self.console.as_ref()),
                ..sjk_game_jka::pmove_japlus::JaPlusRules::from_game_state(session.game_state())
            }
        })
    }

    /// Whether the mouse leaves the view alone this frame: the server holds it, the
    /// player has not asked to look anyway, and no free or fake-noclip camera flies.
    pub(crate) fn mouse_look_held(&self) -> bool {
        if self.local_prediction.fake_noclip() || self.local_prediction.free_camera() {
            return false;
        }
        let look = self
            .console
            .as_ref()
            .and_then(|console| console.bool_cvar(LOOK_CVAR))
            .unwrap_or(DEFAULT_LOOK);
        !look && self.server_holds_view()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jof_servers_leave_knockdowns_free_unless_the_player_says_otherwise() {
        let jof = sjk_mod::Server {
            kind: sjk_mod::ServerKind::JaPlus,
            address: "127.0.0.1:29070".into(),
            info: r"\V\2.5B0".into(),
        };
        let other = sjk_mod::Server {
            info: r"\V\2.4B7".into(),
            ..jof.clone()
        };
        assert!(knockdown_view_free(Some(-1), jof.family()));
        assert!(knockdown_view_free(None, jof.family()));
        assert!(!knockdown_view_free(Some(-1), other.family()));
        assert!(!knockdown_view_free(Some(0), jof.family()));
        assert!(knockdown_view_free(Some(1), other.family()));
        assert!(!free_knockdown_view(None));
    }

    #[test]
    fn every_free_knockdown_family_is_a_known_family() {
        for free in FREE_KNOCKDOWN_VIEW {
            let family = sjk_mod::KNOWN_SERVERS
                .iter()
                .find(|family| family.id == free.family)
                .unwrap_or_else(|| panic!("{} is no known family", free.family));
            assert_eq!(family.name, free.name);
        }
    }
}
