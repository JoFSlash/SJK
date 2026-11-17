//! The box hunt's grapple cooldown (hub report #320, adwe's idea, 14/11/2026; Sol: "Maybe
//! a CD"): a mark who grapples away every time they are picked cannot be caught, so while
//! the player is in a hunt's chase, the mark or a hunter still in it, the JA+ grapple hook
//! (`+grapple`, `+button12`) rests for a while after each release: the button is kept off
//! the user commands until the cooldown is over, and a plate under the hunt's card counts
//! it down. Out of a chase (no mark, the toy off, out of it) the hook is as ever.
//!
//! The hook is the game server's (JA+), which knows nothing of the hunt, so this client
//! holds the button back itself: a client changed to send it anyway is not stopped. The
//! length is the hub's (`grapple_cooldown` in the hunt's read), [`DEFAULT`] from a hub
//! that does not say, none for 0.

use crate::GpuState;
use std::time::{Duration, Instant};

/// The grapple hook's button (`+button12`, EternalJK's `+grapple`).
pub(super) const BUTTON: u16 = 1 << 12;
/// How long the hook rests after each release when the hub does not say (PROVISIONAL,
/// 17/11/2026: Sol's to tune, on the hub).
pub(super) const DEFAULT: Duration = Duration::from_secs(5);

/// The cooldown as the user commands go.
#[derive(Default)]
pub(crate) struct Cooldown {
    /// When the hook may be used again.
    until: Option<Instant>,
    /// The last command sent the hook's button.
    sent: bool,
    /// The last command had the hook's button held back.
    held_back: bool,
}

impl Cooldown {
    /// Shape one user command's `buttons` at `now`; `length` is the cooldown while the
    /// player is in a chase, `None` out of one.
    pub(crate) fn shape(&mut self, buttons: &mut u16, length: Option<Duration>, now: Instant) {
        let held = *buttons & BUTTON != 0;
        let Some(length) = length.filter(|length| !length.is_zero()) else {
            self.until = None;
            self.held_back = false;
            self.sent = held;
            return;
        };
        // A release starts the rest.
        if self.sent && !held {
            self.until = Some(now + length);
        }
        self.held_back = held && self.until.is_some_and(|until| now < until);
        if self.held_back {
            *buttons &= !BUTTON;
        }
        self.sent = *buttons & BUTTON != 0;
    }

    /// Whether the last command held the hook's button back: its release (`-grapple`)
    /// then lets go of nothing, so the `+use` tap JA+ takes for it is not sent.
    pub(crate) fn held_back(&self) -> bool {
        self.held_back
    }

    /// How long the hook still rests at `now`, while it does.
    pub(super) fn left(&self, now: Instant) -> Option<Duration> {
        self.until
            .map(|until| until.saturating_duration_since(now))
            .filter(|left| !left.is_zero())
    }
}

/// The plate's words while the hook rests `left` longer: a caption, then the line.
pub(super) fn words(left: Duration) -> (String, String) {
    let seconds = left.as_secs() + u64::from(left.subsec_nanos() > 0);
    (
        "BOX HUNT \u{b7} GRAPPLE COOLDOWN".to_owned(),
        format!("Grapple ready in {seconds}s"),
    )
}

impl GpuState {
    /// The grapple cooldown while the player is in a box hunt's chase (the mark, or a
    /// hunter not out), `None` out of one.
    pub(crate) fn box_hunt_grapple_cooldown(&self) -> Option<Duration> {
        self.mystery_boxes.grapple_cooldown(self.box_hunt.on())
    }

    /// The grapple cooldown's state, for the user commands.
    pub(crate) fn box_hunt_grapple(&mut self) -> &mut Cooldown {
        &mut self.mystery_boxes.grapple
    }
}

impl super::MysteryBoxes {
    /// [`GpuState::box_hunt_grapple_cooldown`], `on` the Box hunt toy's switch.
    pub(crate) fn grapple_cooldown(&self, on: bool) -> Option<Duration> {
        if !on {
            return None;
        }
        self.hunt.card().filter(|card| !card.out)?;
        let hub = crate::player_identity::with_snapshot(|snapshot| {
            snapshot
                .box_hunt
                .as_ref()
                .and_then(|(_, _, state)| state.grapple_cooldown)
        })
        .flatten();
        Some(hub.map_or(DEFAULT, Duration::from_secs))
    }

    /// The grapple cooldown's state, for the user commands.
    pub(crate) fn grapple_mut(&mut self) -> &mut Cooldown {
        &mut self.grapple
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hook_rests_after_each_release_in_a_chase() {
        let start = Instant::now();
        let at = |millis| start + Duration::from_millis(millis);
        let mut cooldown = Cooldown::default();
        let chase = Some(DEFAULT);
        // The first hook goes through, held as long as the player likes.
        for millis in [0, 8, 1000] {
            let mut buttons = BUTTON;
            cooldown.shape(&mut buttons, chase, at(millis));
            assert_eq!(buttons, BUTTON);
        }
        // Released: the rest begins, and a new press is held back until it ends.
        let mut buttons = 0;
        cooldown.shape(&mut buttons, chase, at(1008));
        assert_eq!(cooldown.left(at(1008)), Some(DEFAULT));
        let mut buttons = BUTTON | 1;
        cooldown.shape(&mut buttons, chase, at(3000));
        assert_eq!(buttons, 1, "only the hook is held back");
        assert!(cooldown.held_back());
        // Held through the end, the hook fires once it is over.
        let mut buttons = BUTTON;
        cooldown.shape(&mut buttons, chase, at(6008));
        assert_eq!(buttons, BUTTON);
        assert!(!cooldown.held_back() && cooldown.left(at(6008)).is_none());
    }

    #[test]
    fn out_of_a_chase_the_hook_is_free() {
        let now = Instant::now();
        let mut cooldown = Cooldown::default();
        let mut buttons = BUTTON;
        cooldown.shape(&mut buttons, Some(DEFAULT), now);
        let mut buttons = 0;
        cooldown.shape(&mut buttons, Some(DEFAULT), now);
        // The chase ends: the rest goes with it.
        let mut buttons = BUTTON;
        cooldown.shape(&mut buttons, None, now);
        assert_eq!(buttons, BUTTON);
        assert!(cooldown.left(now).is_none());
        // A hub that says 0: none at all.
        let mut buttons = 0;
        cooldown.shape(&mut buttons, Some(Duration::ZERO), now);
        let mut buttons = BUTTON;
        cooldown.shape(&mut buttons, Some(Duration::ZERO), now);
        assert_eq!(buttons, BUTTON);
    }

    #[test]
    fn the_plate_counts_whole_seconds_up() {
        let (caption, line) = words(Duration::from_millis(4200));
        assert_eq!(caption, "BOX HUNT \u{b7} GRAPPLE COOLDOWN");
        assert_eq!(line, "Grapple ready in 5s");
        assert_eq!(words(Duration::from_secs(1)).1, "Grapple ready in 1s");
    }
}
