//! Other players' toys, seen or not (Sol's request, 29/10/2026): each toy other players
//! are seen with has a switch on its card in the Collection's Toys tab, "See ...",
//! that hides theirs on this PC alone, for a player who does not like them (the Monitor's
//! CRTs first). Each is an archived cvar, on by default: the Illuminate holocron's,
//! the cardboard box's and the boombox's go through [`crate::looks::Looks::set_unseen`]
//! (as if those players had them off here), the drone's and the ragdoll's where their
//! relayed events are drawn, and the Monitor's is its own [`crate::monitor::SHOW_CVAR`].
//! Nothing changes for the toys' rules: a hidden toy is still the others'.

use crate::console::ViewerConsole;

/// Other players' Illuminate holocrons are seen.
pub(crate) const ILLUMINATE: &str = "cg_toyIlluminateShow";
/// Other players' cardboard boxes are seen (and lifted).
pub(crate) const CARDBOARD_BOX: &str = "cg_toyBoxShow";
/// Other players' falls as ragdolls are seen.
pub(crate) const RAGDOLL: &str = "cg_toyRagdollShow";
/// Other players' drones are seen and heard.
pub(crate) const DRONE: &str = "cg_toyDroneShow";
/// Other players' boomboxes are seen and heard.
pub(crate) const BOOMBOX: &str = "cg_toyBoomboxShow";

/// The switches this module adds, with their help, for the cvar registry.
pub(crate) const CVARS: [(&str, &str); 5] = [
    (
        ILLUMINATE,
        "Other players' Illuminate holocrons are seen (0: hidden on this PC; SJK)",
    ),
    (
        CARDBOARD_BOX,
        "Other players' cardboard boxes are seen (0: hidden on this PC, and not lifted; SJK)",
    ),
    (
        RAGDOLL,
        "Other players' Ragdoll falls are seen (0: their bodies as the game shows them; SJK)",
    ),
    (
        DRONE,
        "Other players' drones are seen and heard (0: hidden on this PC; SJK)",
    ),
    (
        BOOMBOX,
        "Other players' boomboxes are seen and heard (0: hidden on this PC; SJK)",
    ),
];

/// Whether `cvar` lets other players' toy be seen: on unless set to 0.
pub(crate) fn seen(console: Option<&ViewerConsole>, cvar: &str) -> bool {
    console.and_then(|console| console.bool_cvar(cvar)) != Some(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every switch is on by default and off once set to 0.
    #[test]
    fn other_players_toys_are_seen_until_switched_off() {
        let directory = tempfile::tempdir().unwrap();
        let mut console = ViewerConsole::new(directory.path().join("config.cfg")).unwrap();
        for (cvar, _) in CVARS {
            assert!(seen(Some(&console), cvar), "{cvar}");
            console.set_cvar(cvar, "0");
            assert!(!seen(Some(&console), cvar), "{cvar}");
        }
        assert!(seen(None, ILLUMINATE));
    }
}
