//! SJK's strafe aura ([`crate::DrawCommand::StrafeAura`]): the speed gain of every
//! heading, shaded per pixel. [`shade`] is the reference for `ui_shapes.wgsl`, which
//! evaluates the same expressions per fragment, and for CPU previews.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

/// What one strafe aura draws: an arc whose top is the view and each point a turn of
/// that angle (left on the left), glowing with what one move would gain there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StrafeAura {
    /// Radius of the glow's centre line, pixels.
    pub radius: f32,
    /// Thickness of the glow, pixels.
    pub width: f32,
    /// Turn shown each side of the top, radians.
    pub span: f32,
    /// Horizontal speed the move starts from.
    pub speed: f32,
    /// Most speed one move adds (acceleration times wish speed times frame time).
    pub acceleration: f32,
    /// Wish speed of the keys held.
    pub wishspeed: f32,
    /// Angle between the wish direction and the velocity at the top, radians.
    pub base: f32,
    /// Seconds, for the flow of the glow.
    pub time: f32,
    /// Opacity, `0..=1`.
    pub alpha: f32,
}

/// Speed gained in one move by wishing `angle` radians off the velocity
/// (`PM_Accelerate`, codemp `bg_pmove.c`): the velocity grows along the wish direction
/// by `wishspeed` less the speed already along it, at most `acceleration`. Negative
/// where the push slows the player down. Written as a difference over a sum so the
/// small gain keeps its precision against the large speed.
pub fn strafe_gain(speed: f32, acceleration: f32, wishspeed: f32, angle: f32) -> f32 {
    let along = speed * angle.cos();
    let add = (wishspeed - along).clamp(0.0, acceleration);
    let grow = 2.0 * along * add + add * add;
    let next = (speed * speed + grow).max(0.0).sqrt();
    if next + speed > 0.0 {
        grow / (next + speed)
    } else {
        0.0
    }
}

/// The wish angle off the velocity that gains the most, radians: where the push first
/// reaches `acceleration`.
pub fn strafe_peak(speed: f32, acceleration: f32, wishspeed: f32) -> f32 {
    if speed <= 0.0 {
        return 0.0;
    }
    ((wishspeed - acceleration) / speed).clamp(-1.0, 1.0).acos()
}

fn wrap(angle: f32) -> f32 {
    angle - TAU * ((angle + PI) / TAU).floor()
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Straight-alpha colour of the aura at `point`, pixels from its centre (y down).
///
/// Force blue where turning there gains speed, thicker and whiter the more it gains;
/// a white-hot seam at the best heading; nothing short of it; faint red where it slows
/// you down. Wisps flow towards the best heading, and the whole aura pulses while the
/// view itself gains at least 90% of the best.
pub fn shade(point: [f32; 2], aura: &StrafeAura) -> [f32; 4] {
    let none = [0.0; 4];
    let half = aura.width * 0.5;
    if half <= 0.0 || aura.alpha <= 0.0 {
        return none;
    }
    let distance = point[0].hypot(point[1]);
    let across = (distance - aura.radius) / half;
    if across.abs() > 1.6 {
        return none;
    }
    // The top is the view; turning left (positive) goes anticlockwise on screen.
    let turn = wrap(-FRAC_PI_2 - point[1].atan2(point[0]));
    let fade = 1.0 - smoothstep(aura.span - 0.15, aura.span, turn.abs());
    if fade <= 0.0 {
        return none;
    }
    let peak = strafe_peak(aura.speed, aura.acceleration, aura.wishspeed);
    let top = strafe_gain(aura.speed, aura.acceleration, aura.wishspeed, peak);
    if top <= 0.0 {
        return none;
    }
    let gain = |angle: f32| strafe_gain(aura.speed, aura.acceleration, aura.wishspeed, angle);
    let angle = aura.base + turn;
    let n = gain(angle) / top;
    let to_best = wrap(angle - peak).abs().min(wrap(angle + peak).abs());
    let t = aura.time;
    // Wisps: waves running towards the best heading, bent by a slower sway.
    let flow = 0.5 + 0.5 * (to_best * 40.0 - t * 5.0 + (across * 3.0 + t * 1.7).sin() * 1.2).sin();
    let flicker =
        0.5 + 0.5 * ((turn * 23.0 + t * 2.3 + across * 4.0).sin() * (turn * 11.0 - t * 1.3).sin());
    let pulse = if gain(aura.base) / top > 0.9 {
        1.0 + 0.35 * (t * 9.0).sin()
    } else {
        1.0
    };
    let seam = (-(to_best / 0.02).powi(2)).exp() * (-(across / 0.55).powi(2)).exp();
    let (mut rgb, mut a) = if n > 0.0 {
        let hot = n * n;
        let thickness = 0.35 + 0.65 * n;
        let flame = across / thickness + (flow - 0.5) * 0.35;
        let body = (-flame * flame * 3.0).exp();
        let core = (-(across / 0.18).powi(2)).exp();
        let rgb = [0.22 + 0.63 * hot, 0.5 + 0.45 * hot, 1.0];
        let a = (body * (0.3 + 0.4 * flow + 0.2 * flicker) * n.sqrt() + core * 0.6 * hot) * pulse;
        (rgb, a)
    } else if n < 0.0 {
        let a = (-n).min(1.0) * 0.55 * (-(across / 0.3).powi(2)).exp() * (0.7 + 0.3 * flow);
        ([1.0, 0.28, 0.2], a)
    } else {
        ([1.0; 3], 0.0)
    };
    if seam > 0.0 {
        rgb = rgb.map(|c| c + (1.0 - c) * seam);
        a = a.max(seam * 0.95 * pulse.min(1.0));
    }
    [
        rgb[0],
        rgb[1],
        rgb[2],
        (a * fade * aura.alpha).clamp(0.0, 1.0),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aura(base: f32) -> StrafeAura {
        StrafeAura {
            radius: 100.0,
            width: 40.0,
            span: 60f32.to_radians(),
            speed: 400.0,
            acceleration: 2.0,
            wishspeed: 250.0,
            base,
            time: 0.0,
            alpha: 1.0,
        }
    }

    /// A point on the glow's centre line, `turn` radians left of the top.
    fn at(turn: f32) -> [f32; 2] {
        let angle = -FRAC_PI_2 - turn;
        [100.0 * angle.cos(), 100.0 * angle.sin()]
    }

    #[test]
    fn the_gain_peaks_at_the_best_angle_and_turns_negative_far_past_it() {
        let (v, a, w) = (400.0, 2.0, 250.0);
        let best = strafe_peak(v, a, w);
        let top = strafe_gain(v, a, w, best);
        assert!(top > 0.0);
        let step = 1f32.to_radians();
        assert!(strafe_gain(v, a, w, best - step) < top);
        assert!(strafe_gain(v, a, w, best + step) < top);
        assert_eq!(strafe_gain(v, a, w, best - 2.0 * step), 0.0);
        assert!(strafe_gain(v, a, w, 120f32.to_radians()) < 0.0);
    }

    #[test]
    fn the_best_heading_is_white_hot_and_short_of_it_is_dark() {
        // W+A along the velocity: the wish is 45 degrees off, the best about 51.7.
        let aura = aura(45f32.to_radians());
        let best = strafe_peak(400.0, 2.0, 250.0) - aura.base;
        let seam = shade(at(best), &aura);
        assert!(seam[3] > 0.9 && seam[0] > 0.95, "{seam:?}");
        assert!(shade(at(0.0), &aura)[3] < 0.01);
        // Past the best the glow is blue and fainter; beyond the span, nothing.
        let past = shade(at(best + 10f32.to_radians()), &aura);
        assert!(
            past[3] > 0.05 && past[3] < seam[3] && past[2] > past[0],
            "{past:?}"
        );
        assert_eq!(shade(at(70f32.to_radians()), &aura)[3], 0.0);
        // Off the band, nothing.
        assert_eq!(shade([0.0, -200.0], &aura)[3], 0.0);
    }

    #[test]
    fn headings_that_slow_you_down_are_red() {
        // Wishing 100 degrees off the velocity at the top: about 110 slows you down.
        let aura = aura(100f32.to_radians());
        let red = shade(at(10f32.to_radians()), &aura);
        assert!(red[3] > 0.0 && red[0] > red[2], "{red:?}");
    }
}
