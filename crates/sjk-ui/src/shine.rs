//! A finish shaded per pixel over any rounded tile ([`crate::DrawCommand::CardShine`]): a
//! card, or any other tile that wants one (a saber shader's, a medal's). Holo's smooth
//! rainbow foil, Solar's living corona round the tile's edge, after the Sun blade, and the
//! soft glare under the pointer.
//! [`shade`] is the reference for `ui_shapes.wgsl`, which evaluates the same expressions
//! per fragment, and for CPU previews.

use std::f32::consts::{PI, TAU};

/// Which finish a [`CardShine`] shades.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShineKind {
    /// A rainbow foil slanting across the card, a white glint at its middle.
    Holo,
    /// A solar corona burning along the card's edge, flares running round it, sparks,
    /// and a faint band of sunlight over the face.
    Solar,
    /// A soft white light under the pointer, whatever the finish.
    Glare,
}

/// What one tile's finish draws over the tile's rectangle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardShine {
    pub kind: ShineKind,
    /// The tile's size in pixels, as laid out (before any transform).
    pub size: [f32; 2],
    /// Its corner radius, pixels: nothing is drawn outside the rounded tile.
    pub radius: f32,
    /// Seconds, for the motion.
    pub time: f32,
    /// Where the light crosses the tile, a share of its width and height from its top
    /// left (across may lie off the tile, while a sweep waits): Holo's glint and Solar's
    /// band of sunlight pass through it, and the glare is centred on it.
    pub light: [f32; 2],
    /// How firmly the pointer holds the light there, 0 to 1 (eased in and out): the foil
    /// shines a little brighter and the glare shows as much.
    pub hold: f32,
    /// Opacity, `0..=1`.
    pub alpha: f32,
}

/// The Sun blade's colours (the hub's `saber_sun.bladeskin`): its white-gold core, the
/// gold inside its corona and the rim's hot and cool reds.
pub const SUN_CORE: [f32; 3] = [1.0, 0.97, 0.86];
pub const SUN_INNER: [f32; 3] = [1.0, 0.72, 0.24];
pub const SUN_RIM_HOT: [f32; 3] = [1.0, 0.42, 0.05];
pub const SUN_RIM_COOL: [f32; 3] = [1.0, 0.2, 0.02];
/// The Sun's rising sparks.
const SUN_MOTE: [f32; 3] = [1.0, 0.82, 0.42];

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mix3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// PCG hash of a lattice point, in `[0, 1]` (the saber shader's `skin_hash`).
pub fn hash(cell: [i32; 2]) -> f32 {
    let mut v = (cell[0] as u32)
        .wrapping_mul(747_796_405)
        .wrapping_add(2_891_336_453);
    v ^= (cell[1] as u32).wrapping_mul(2_654_435_761);
    v = v.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    let word = ((v >> ((v >> 28) + 4)) ^ v).wrapping_mul(277_803_737);
    ((word >> 22) ^ word) as f32 / 4_294_967_295.0
}

/// Smooth value noise in `[0, 1]`.
pub fn noise([x, y]: [f32; 2]) -> f32 {
    let (cx, cy) = (x.floor(), y.floor());
    let (fx, fy) = (x - cx, y - cy);
    let (ux, uy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let cell = [cx as i32, cy as i32];
    let a = hash(cell);
    let b = hash([cell[0] + 1, cell[1]]);
    let c = hash([cell[0], cell[1] + 1]);
    let d = hash([cell[0] + 1, cell[1] + 1]);
    let top = a + (b - a) * ux;
    let bottom = c + (d - c) * ux;
    top + (bottom - top) * uy
}

fn wrap(angle: f32) -> f32 {
    angle - TAU * ((angle + PI) / TAU).floor()
}

/// Straight-alpha colour `top` laid over `under` (both straight alpha).
fn over(under: [f32; 4], top: [f32; 3], alpha: f32) -> [f32; 4] {
    let alpha = alpha.clamp(0.0, 1.0);
    let out = alpha + under[3] * (1.0 - alpha);
    if out <= 0.0 {
        return [0.0; 4];
    }
    let blend = |t: f32, u: f32| (t * alpha + u * under[3] * (1.0 - alpha)) / out;
    [
        blend(top[0], under[0]),
        blend(top[1], under[1]),
        blend(top[2], under[2]),
        out,
    ]
}

/// Straight-alpha colour of the finish at `point`, pixels from the card's top left.
pub fn shade(point: [f32; 2], shine: &CardShine) -> [f32; 4] {
    let [w, h] = shine.size;
    if w <= 0.0 || h <= 0.0 || shine.alpha <= 0.0 {
        return [0.0; 4];
    }
    // Signed distance to the rounded card, pixels (negative inside).
    let radius = shine.radius.clamp(0.0, w.min(h) * 0.5);
    let qx = (point[0] - w * 0.5).abs() - (w * 0.5 - radius);
    let qy = (point[1] - h * 0.5).abs() - (h * 0.5 - radius);
    let outside = qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - radius;
    let coverage = (0.5 - outside).clamp(0.0, 1.0);
    if coverage <= 0.0 {
        return [0.0; 4];
    }
    let mut out = match shine.kind {
        ShineKind::Holo => holo(point, shine),
        ShineKind::Solar => solar(point, -outside, shine),
        ShineKind::Glare => glare(point, shine),
    };
    out[3] = (out[3] * coverage * shine.alpha).clamp(0.0, 1.0);
    out
}

/// Holo: a pearl over the card, a rainbow foil whose hue runs smoothly across the slant
/// through the light, strongest about it and fading far from it, and a white glint.
fn holo([x, y]: [f32; 2], shine: &CardShine) -> [f32; 4] {
    let [w, h] = shine.size;
    let t = shine.time;
    let (u, v) = (x / w, y / h);
    // Across the slant through the light, pixels, then in shares of the foil's spread.
    let across = (x - shine.light[0] * w) + 0.84 * (y - shine.light[1] * h);
    let k = across / (0.68 * w);
    // The pearl: a faint cool-to-pink veil.
    let pearl = mix3([0.75, 0.85, 1.0], [1.0, 0.7, 0.95], u);
    let mut out = [pearl[0], pearl[1], pearl[2], 0.05 + 0.03 * u];
    // The foil's hue: red to violet across the slant, swaying gently as foil does.
    let hue = k * 0.9 + 0.5 + 0.07 * (u * 5.0 + v * 2.0 + t * 0.6).sin();
    let rainbow = [0.0_f32, 1.0 / 3.0, 2.0 / 3.0]
        .map(|offset| 0.5 + 0.5 * (TAU * (hue - offset)).cos())
        .map(|c| c + (1.0 - c) * 0.3);
    let envelope = (-k * k * 2.2).exp();
    let strength = 0.26 + 0.04 * shine.hold.clamp(0.0, 1.0);
    out = over(out, rainbow, 0.035 + strength * envelope);
    // The glint: a thin white line with a soft halo.
    let line = across / (0.03 * w);
    let halo = across / (0.12 * w);
    let glint = (-line * line).exp() * 0.3 + (-halo * halo).exp() * 0.07;
    over(out, [1.0, 1.0, 1.0], glint)
}

/// Solar: the Sun blade's corona burning along the card's edge (`inside` pixels in from
/// it): a white-gold core at the very edge, gold inside graded to a red rim as it
/// reaches in, its reach and brightness stirred by drifting granulation, flame tongues
/// and flares running round the card; rising sparks in it; a warm wash from the top and
/// a soft band of sunlight over the face.
fn solar([x, y]: [f32; 2], inside: f32, shine: &CardShine) -> [f32; 4] {
    let [w, h] = shine.size;
    let t = shine.time;
    // The corona's reach in pixels: a share of the card, never under a few pixels.
    let reach = (w * 0.06).max(5.0);
    let (u, v) = (x / w, y / h);
    // The wash from the top and the band of sunlight, faint over the face.
    let mut out = [SUN_INNER[0], SUN_INNER[1], SUN_INNER[2], 0.06 * (1.0 - v)];
    let across = (x - shine.light[0] * w) + 0.7 * (y - shine.light[1] * h);
    let wide = across / (0.16 * w);
    let thin = across / (0.025 * w);
    out = over(out, SUN_INNER, (-wide * wide).exp() * 0.07);
    out = over(out, SUN_CORE, (-thin * thin).exp() * 0.13);
    if inside > reach * 3.2 {
        return out;
    }
    // Flares: three bright swells running round the card, coming and going.
    let angle = (v - 0.5).atan2(u - 0.5);
    let mut flare = 0.0;
    for index in 0..3 {
        let i = index as f32;
        let direction = if index == 1 { -1.0 } else { 1.0 };
        let centre = direction * t * (0.32 + 0.11 * i) + i * 2.1;
        let off = wrap(angle - centre) / 0.3;
        let swell = 0.55 + 0.45 * (t * 1.3 + i * 1.7).sin();
        flare += (-off * off).exp() * swell;
    }
    // Granulation: two octaves drifting as the blade's do, in the corona's own scale.
    let p = [x / reach, y / reach];
    let grain = 0.62 * noise([p[0] * 0.55 + t * 0.35, p[1] * 0.55 - t * 0.9])
        + 0.38 * noise([p[0] * 1.3 - t * 0.6, p[1] * 1.3 - t * 1.6 + 17.0]);
    let grain = smoothstep(0.15, 0.85, grain);
    // Tongues: ridges of a third noise, sharp, licking further in.
    let ridge = 1.0 - (2.0 * noise([p[0] * 0.8 + 31.0, p[1] * 0.8 - t * 1.2]) - 1.0).abs();
    let tongue = ridge * ridge * ridge;
    let height = reach * (0.6 + 0.7 * grain + 0.9 * tongue + 0.8 * flare);
    let depth = inside.max(0.0) / height;
    // Shimmer: the blade's two quick waves.
    let shimmer = 1.0 + 0.045 * (t * 9.0 + u * 4.4).sin() + 0.03 * (t * 15.7 - v * 8.2).sin();
    if depth < 1.0 {
        let body = (1.0 - depth).powf(1.6);
        let bright = (0.45 + 0.4 * grain + 0.25 * tongue + 0.6 * flare).min(1.0);
        let rim = mix3(SUN_RIM_HOT, SUN_RIM_COOL, (1.0 - grain).clamp(0.0, 1.0));
        let mut colour = mix3(SUN_INNER, rim, smoothstep(0.15, 0.85, depth));
        let core = inside.max(0.0) / (reach * 0.16);
        let core = (-core * core).exp() * (0.6 + 0.4 * flare.min(1.0));
        colour = mix3(colour, SUN_CORE, core);
        out = over(out, colour, (body * bright * shimmer).min(0.95));
    }
    // Sparks: motes rising through the corona, each in its own cell, twinkling.
    let cell = reach * 0.9;
    let rise = [x / cell, (y + t * reach * 1.1) / cell];
    let at = [rise[0].floor() as i32, rise[1].floor() as i32];
    let seed = hash(at);
    if seed > 0.86 {
        let mx = rise[0].floor() + 0.2 + 0.6 * hash([at[0] + 7, at[1]]);
        let my = rise[1].floor() + 0.2 + 0.6 * hash([at[0], at[1] + 7]);
        let dx = (rise[0] - mx) * cell;
        let dy = (rise[1] - my) * cell * 0.6;
        let size = (reach * 0.09).max(0.9);
        let dot = (dx * dx + dy * dy) / (size * size);
        let twinkle = 0.5 + 0.5 * (t * 3.0 + seed * 40.0).sin();
        let fade = 1.0 - smoothstep(reach * 1.2, reach * 3.0, inside);
        out = over(out, SUN_MOTE, (-dot).exp() * twinkle * fade * 0.9);
    }
    out
}

/// The glare: a soft round white light about the light's point, its spread a share of
/// the tile's narrower side, as bright as the pointer's hold.
fn glare([x, y]: [f32; 2], shine: &CardShine) -> [f32; 4] {
    let [w, h] = shine.size;
    let spread = 0.32 * w.min(h);
    let dx = (x - shine.light[0] * w) / spread;
    let dy = (y - shine.light[1] * h) / spread;
    let alpha = 0.16 * (-(dx * dx + dy * dy)).exp() * shine.hold.clamp(0.0, 1.0);
    [1.0, 1.0, 1.0, alpha]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(kind: ShineKind) -> CardShine {
        CardShine {
            kind,
            size: [400.0, 560.0],
            radius: 28.0,
            time: 1.5,
            light: [0.5, 0.5],
            hold: 0.0,
            alpha: 1.0,
        }
    }

    #[test]
    fn nothing_shows_outside_the_rounded_card() {
        for kind in [ShineKind::Holo, ShineKind::Solar, ShineKind::Glare] {
            let mut shine = card(kind);
            shine.hold = 1.0;
            assert_eq!(shade([1.0, 1.0], &shine)[3], 0.0, "{kind:?} corner");
            assert_eq!(shade([-5.0, 200.0], &shine)[3], 0.0, "{kind:?} left");
            assert!(shade([200.0, 280.0], &shine)[3] > 0.0, "{kind:?} middle");
        }
    }

    /// Holo's hue runs smoothly: neighbouring pixels across the foil differ by little.
    #[test]
    fn the_holo_foil_has_no_steps() {
        let shine = card(ShineKind::Holo);
        let mut last = shade([20.0, 280.0], &shine);
        for x in 21..380 {
            let now = shade([x as f32, 280.0], &shine);
            for channel in 0..4 {
                assert!(
                    (now[channel] - last[channel]).abs() < 0.03,
                    "{x}: {last:?} -> {now:?}"
                );
            }
            last = now;
        }
    }

    /// Solar burns at the edge and keeps the face nearly clear.
    #[test]
    fn the_solar_corona_lies_along_the_edge() {
        let shine = card(ShineKind::Solar);
        // Averaged along each side, so a flare's place does not decide it.
        let average = |inside: f32| {
            let mut sum = 0.0;
            for step in 0..50 {
                let y = 60.0 + step as f32 * 9.0;
                sum += shade([inside, y], &shine)[3] + shade([400.0 - inside, y], &shine)[3];
            }
            sum / 100.0
        };
        let edge = average(2.0);
        let face = average(150.0);
        assert!(edge > 0.4, "{edge}");
        assert!(face < 0.15, "{face}");
        // Its edge is the Sun's gold, warm.
        let [r, g, b, _] = shade([2.0, 300.0], &shine);
        assert!(r > 0.95 && g > b, "{r} {g} {b}");
    }

    /// The corona moves: the same edge pixel changes over time.
    #[test]
    fn the_solar_corona_lives() {
        let mut shine = card(ShineKind::Solar);
        let first = shade([6.0, 300.0], &shine);
        shine.time += 0.7;
        let later = shade([6.0, 300.0], &shine);
        assert!((first[3] - later[3]).abs() > 0.01 || (first[1] - later[1]).abs() > 0.01);
    }

    /// The glare is round and smooth, brightest at the light, and shows only as much as
    /// the pointer holds it.
    #[test]
    fn the_glare_is_smooth_and_follows_the_hold() {
        let mut shine = card(ShineKind::Glare);
        assert_eq!(shade([200.0, 280.0], &shine)[3], 0.0, "not held");
        shine.hold = 1.0;
        let peak = shade([200.0, 280.0], &shine)[3];
        let mut last = peak;
        for x in 201..380 {
            let now = shade([x as f32, 280.0], &shine)[3];
            assert!(now <= last && last - now < 0.01, "{x}: {last} -> {now}");
            last = now;
        }
        shine.hold = 0.5;
        assert!((shade([200.0, 280.0], &shine)[3] - peak * 0.5).abs() < 1e-4);
    }
}
