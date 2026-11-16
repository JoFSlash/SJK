//! The Shaders tab's saber on its own ([`SaberTurn`]): it lies across the clear right of
//! the screen, rolling slowly about its own length, and the pointer turns it. A drag
//! across turns it about the window's vertical (the tip towards the viewer or away), a
//! drag up or down rolls it about its length; let go, it keeps the drag's speed and slows
//! to its own roll, and a little later its tip swings back to where it rests, so the
//! blade never stays end on.

use std::f32::consts::{PI, TAU};

/// How the lone saber is turned, radians: `yaw` about the window's vertical, from the
/// saber lying across with its tip to the right; `roll` about the window's horizontal,
/// its own length while it lies across.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct SaberTurn {
    pub(crate) yaw: f32,
    pub(crate) roll: f32,
}

/// The yaw the saber rests at: its tip a little towards the viewer, so the blade reads
/// as a thing in the room rather than a flat bar.
pub(super) const REST_YAW: f32 = -0.2;
/// Its roll about its length while nothing holds it, radians a second.
const IDLE_ROLL: f32 = 0.42;
/// Radians a frame pixel of drag turns it.
const PER_PIXEL: f32 = 0.009;
/// How fast a flung saber slows, a share of its speed a second.
const FRICTION: f32 = 2.6;
/// Seconds after letting go before the tip swings back, and how fast it swings (a share
/// of what is left a second).
const SETTLE_AFTER: f32 = 1.4;
const SETTLE: f32 = 2.2;
/// The fastest a fling may leave it, radians a second.
const FLING_MAX: f32 = 14.0;

/// The turn and what drives it: the pointer holding it, or the speed it was let go with.
#[derive(Clone, Copy, Debug)]
pub(super) struct Spin {
    turn: SaberTurn,
    /// Radians a second, yaw and roll, from the last drag.
    speed: [f32; 2],
    /// Where the pointer was (window pixels) while it holds the saber.
    held: Option<[f32; 2]>,
    /// The page's seconds at the last step, and at the last let-go.
    last: Option<f32>,
    released: f32,
    /// What the last drag moved, waiting for the next step to turn it into a speed.
    dragged: [f32; 2],
}

impl Default for Spin {
    fn default() -> Self {
        Self {
            turn: SaberTurn {
                yaw: REST_YAW,
                roll: 0.0,
            },
            speed: [0.0; 2],
            held: None,
            last: None,
            released: f32::NEG_INFINITY,
            dragged: [0.0; 2],
        }
    }
}

impl Spin {
    /// The turn now.
    pub(super) fn turn(&self) -> SaberTurn {
        self.turn
    }

    /// Whether the pointer holds the saber.
    pub(super) fn held(&self) -> bool {
        self.held.is_some()
    }

    /// Held still at `turn`, as a drag leaves it (a world shot's).
    #[cfg(test)]
    pub(super) fn hold(&mut self, turn: SaberTurn) {
        self.grab([0.0; 2]);
        self.turn = turn;
    }

    /// The pointer took hold at `at` (window pixels).
    pub(super) fn grab(&mut self, at: [f32; 2]) {
        self.held = Some(at);
        self.speed = [0.0; 2];
        self.dragged = [0.0; 2];
    }

    /// The pointer moved to `at` (window pixels) in a window whose frame pixel is `scale`
    /// window pixels: a held saber turns with it.
    pub(super) fn drag(&mut self, at: [f32; 2], scale: f32) {
        let Some(from) = self.held else {
            return;
        };
        let per = PER_PIXEL / scale.max(0.1);
        let yaw = (at[0] - from[0]) * per;
        let roll = (at[1] - from[1]) * per;
        self.turn.yaw += yaw;
        self.turn.roll += roll;
        self.dragged[0] += yaw;
        self.dragged[1] += roll;
        self.held = Some(at);
    }

    /// The pointer let go at the page's `seconds`.
    pub(super) fn release(&mut self, seconds: f32) {
        if self.held.take().is_some() {
            self.released = seconds;
        }
    }

    /// Step to the page's `seconds`.
    pub(super) fn advance(&mut self, seconds: f32) {
        let dt = self
            .last
            .map_or(0.0, |last| (seconds - last).clamp(0.0, 0.1));
        self.last = Some(seconds);
        if dt <= 0.0 {
            return;
        }
        if self.held.is_some() {
            // The drag's speed, eased so one jerky frame does not fling it.
            for (speed, dragged) in self.speed.iter_mut().zip(self.dragged) {
                let now = (dragged / dt).clamp(-FLING_MAX, FLING_MAX);
                *speed += (now - *speed) * (dt * 18.0).min(1.0);
            }
            self.dragged = [0.0; 2];
            return;
        }
        let slow = (-FRICTION * dt).exp();
        self.speed = self.speed.map(|speed| speed * slow);
        self.turn.yaw += self.speed[0] * dt;
        // The roll slows to its own, never to a stop.
        self.turn.roll = (self.turn.roll + (IDLE_ROLL + self.speed[1]) * dt).rem_euclid(TAU);
        if seconds - self.released > SETTLE_AFTER {
            // The tip swings back the short way round to where it rests.
            let off = wrap(self.turn.yaw - REST_YAW);
            self.turn.yaw = REST_YAW + off * (-SETTLE * dt).exp();
        }
    }
}

/// `angle` brought within half a turn of nought.
fn wrap(angle: f32) -> f32 {
    (angle + PI).rem_euclid(TAU) - PI
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Step 60 frames a second from `from` for `seconds`.
    fn run(spin: &mut Spin, from: f32, seconds: f32) {
        let mut t = from;
        while t <= from + seconds + 1e-4 {
            spin.advance(t);
            t += 1.0 / 60.0;
        }
    }

    #[test]
    fn on_its_own_it_rolls_and_rests_its_tip() {
        let mut spin = Spin::default();
        run(&mut spin, 0.0, 2.0);
        let turn = spin.turn();
        assert!((turn.yaw - REST_YAW).abs() < 1e-4, "{turn:?}");
        assert!(
            (turn.roll - IDLE_ROLL * 2.0).abs() < 0.05,
            "rolls at its own pace: {turn:?}"
        );
    }

    #[test]
    fn a_drag_turns_it_with_the_pointer_and_holds_it_still() {
        let mut spin = Spin::default();
        spin.advance(0.0);
        spin.grab([100.0, 100.0]);
        spin.drag([200.0, 100.0], 1.0);
        let turn = spin.turn();
        assert!((turn.yaw - (REST_YAW + 100.0 * PER_PIXEL)).abs() < 1e-5);
        assert_eq!(turn.roll, 0.0);
        // A 4K window's drag of the same frame pixels turns it as far.
        let mut big = Spin::default();
        big.grab([0.0, 0.0]);
        big.drag([0.0, 200.0], 2.0);
        assert!((big.turn().roll - 100.0 * PER_PIXEL).abs() < 1e-5);
        // Held still, it does not roll on its own.
        let held = spin.turn();
        run(&mut spin, 1.0 / 60.0, 0.5);
        assert!(spin.held());
        assert!((spin.turn().roll - held.roll).abs() < 1e-5);
        // Without a hold, moving the pointer turns nothing.
        let mut free = Spin::default();
        free.drag([500.0, 500.0], 1.0);
        assert_eq!(free.turn(), Spin::default().turn());
    }

    #[test]
    fn let_go_it_carries_on_slows_and_its_tip_swings_back() {
        let mut spin = Spin::default();
        spin.advance(0.0);
        spin.grab([0.0, 0.0]);
        let mut t = 0.0;
        for step in 1..=6 {
            t = step as f32 / 60.0;
            spin.drag([step as f32 * 20.0, 0.0], 1.0);
            spin.advance(t);
        }
        spin.release(t);
        let at_release = spin.turn().yaw;
        run(&mut spin, t + 1.0 / 60.0, 0.3);
        assert!(
            spin.turn().yaw > at_release + 0.1,
            "flung on: {:?}",
            spin.turn()
        );
        // Long after, its tip is back where it rests (the short way round) and it rolls.
        run(&mut spin, t + 0.4, 6.0);
        let off = wrap(spin.turn().yaw - REST_YAW);
        assert!(off.abs() < 0.05, "{:?}", spin.turn());
        let roll = spin.turn().roll;
        run(&mut spin, t + 6.5, 0.5);
        assert!(wrap(spin.turn().roll - roll) > 0.1);
    }

    #[test]
    fn angles_wrap_the_short_way() {
        assert!((wrap(TAU + 0.5) - 0.5).abs() < 1e-5);
        assert!((wrap(PI + 0.25) - (0.25 - PI)).abs() < 1e-5);
        assert!((wrap(-0.3) + 0.3).abs() < 1e-6);
    }
}
