//! The ticks beside the gold one (`PROTOCOL.md` of the hub, "Ticks"), drawn once at start
//! into icon cells ([`super::tick_texture`]): the verified seal's scalloped shape in a
//! group's colours (JoF cyan, Sith crimson, Jawa leather), rusted for JKR's creator, a
//! silver starburst edged in the team's teal for SJK staff, and a sun for SJK's creator
//! whose rays turn and over which a glint passes, in [`CREATOR_FRAMES`] cells the
//! renderer steps through ([`super::animated`]), so it costs no more to draw than any
//! icon.
//!
//! Computed like the seal ([`super::verified_badge`]): signed distances blended over
//! one pixel.

use super::verified_badge::{TICK, TICK_HALF, coverage, mix, segment};
use sjk_identity::Tick;

/// The seal's lobes, their depth and mean radius, as the gold seal's.
const LOBES: f32 = 8.0;
const LOBE_DEPTH: f32 = 0.075;
const RADIUS: f32 = 0.43;

/// The staff's starburst: its points, how far their tips and the valleys between them
/// reach, and the radius the face's shading and the tick are drawn to, as shares of the
/// cell.
const STAR_POINTS: f32 = 16.0;
const STAR_REACH: f32 = 0.49;
const STAR_VALLEY: f32 = 0.36;
const STAR_FACE: f32 = 0.35;

/// The creator's sun: cells in its loop, and how many the renderer shows a second.
pub(crate) const CREATOR_FRAMES: u32 = 40;
pub(crate) const CREATOR_FPS: f64 = 10.0;
/// Its rays, the disc's radius and the rays' reach, as shares of the cell.
const RAYS: f32 = 12.0;
const SUN_RADIUS: f32 = 0.33;
const RAY_REACH: f32 = 0.49;
/// The share of the loop the glint takes to cross; the rest it waits.
const GLINT_SHARE: f32 = 0.35;

/// A seal's colours: top and bottom of its face, its rim and its tick.
struct Palette {
    top: [f32; 3],
    bottom: [f32; 3],
    rim: [f32; 3],
    tick: [f32; 3],
}

const JOF: Palette = Palette {
    top: [0.5, 0.95, 1.0],
    bottom: [0.0, 0.6, 0.78],
    rim: [0.0, 0.4, 0.55],
    tick: [1.0; 3],
};
/// Deep crimson with a dark rim, so it never reads as the chat ban's bright red disc.
const SITH: Palette = Palette {
    top: [0.8, 0.1, 0.13],
    bottom: [0.45, 0.02, 0.05],
    rim: [0.2, 0.0, 0.02],
    tick: [1.0; 3],
};
const JAWA: Palette = Palette {
    top: [0.74, 0.55, 0.35],
    bottom: [0.47, 0.3, 0.16],
    rim: [0.28, 0.17, 0.08],
    tick: [1.0, 0.96, 0.88],
};
/// Rust, the JKR CREATOR pill's colour (`hud/player_card.rs`), mottled by [`patina`].
const JKR: Palette = Palette {
    top: [0.9, 0.46, 0.22],
    bottom: [0.6, 0.24, 0.1],
    rim: [0.33, 0.12, 0.04],
    tick: [1.0, 0.93, 0.84],
};
/// Silver with the SJK TEAM pill's teal (`hud/player_card.rs`), darkened, as edge and
/// tick: the tick coloured on a light face, where every seal has a white tick.
const STAFF: Palette = Palette {
    top: [0.97, 0.98, 0.99],
    bottom: [0.62, 0.67, 0.73],
    rim: [0.06, 0.42, 0.38],
    tick: [0.08, 0.55, 0.5],
};
/// The shaded side of the starburst's points.
const STAFF_SHADE: [f32; 3] = [0.5, 0.56, 0.63];

/// The colour a word about tick `tick` is written in beside it (the player card's
/// VERIFIED): its face's, the gold seal's for gold; `None` for no tick.
pub(crate) fn accent(tick: Tick) -> Option<[f32; 3]> {
    Some(match tick {
        Tick::None => return None,
        Tick::Gold | Tick::Other => [1.0, 0.82, 0.25],
        Tick::Jof => [0.35, 0.88, 1.0],
        Tick::Sith => [0.95, 0.3, 0.3],
        Tick::Jawa => JAWA.top,
        Tick::Jkr => JKR.top,
        Tick::Staff => [0.22, 0.86, 0.78],
        Tick::Creator => [1.0, 0.6, 0.22],
    })
}

/// The tick `tick` as `size` x `size` straight RGBA bytes, `frame` of the creator's
/// loop; empty for a tick without a cell of its own (gold and none).
pub(crate) fn pixels(size: u32, tick: Tick, frame: u32) -> Vec<u8> {
    match tick {
        Tick::Jof => seal(size, &JOF, false),
        Tick::Sith => seal(size, &SITH, false),
        Tick::Jawa => seal(size, &JAWA, false),
        Tick::Jkr => seal(size, &JKR, true),
        Tick::Staff => starburst(size, &STAFF),
        Tick::Creator => sun(size, frame),
        Tick::None | Tick::Gold | Tick::Other => Vec::new(),
    }
}

/// Distance in radii from `unit` to the tick's strokes.
fn tick_distance(unit: [f32; 2]) -> f32 {
    TICK.windows(2)
        .map(|pair| segment(unit, pair[0], pair[1]))
        .fold(f32::INFINITY, f32::min)
}

/// The scalloped seal in `palette`, `rusted` with patches of patina.
fn seal(size: u32, palette: &Palette, rusted: bool) -> Vec<u8> {
    let side = size as f32;
    let radius = RADIUS * side;
    let centre = side * 0.5;
    let mut out = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let p = [x as f32 + 0.5 - centre, y as f32 + 0.5 - centre];
            let length = (p[0] * p[0] + p[1] * p[1]).sqrt();
            let angle = p[1].atan2(p[0]);
            let edge = radius * (1.0 + LOBE_DEPTH * (LOBES * angle).cos());
            let outside = length - edge;
            let unit = [p[0] / radius, p[1] / radius];
            let mut face = mix(
                palette.top,
                palette.bottom,
                ((unit[1] + 1.0) * 0.5).clamp(0.0, 1.0),
            );
            if rusted {
                let stain = patina(x as f32 / side, y as f32 / side);
                face = mix(
                    face,
                    [0.36, 0.13, 0.05],
                    (stain - 0.45).clamp(0.0, 0.4) * 1.6,
                );
                face = mix(face, [1.0, 0.66, 0.38], (0.3 - stain).clamp(0.0, 0.3));
            }
            let rim = (1.0 - (-outside / (side * 0.03)).clamp(0.0, 1.0)) * 0.8;
            let colour = mix(face, palette.rim, rim);
            let tick = coverage((tick_distance(unit) - TICK_HALF) * radius);
            let colour = mix(colour, palette.tick, tick);
            push(&mut out, colour, coverage(outside));
        }
    }
    out
}

/// The staff's starburst in `palette`: [`STAR_POINTS`] straight points, one up, each
/// with its clockwise side in shade, a bright band slanting across the silver, the
/// edge in the rim's colour and a tick a little bolder than the seals'.
fn starburst(size: u32, palette: &Palette) -> Vec<u8> {
    let side = size as f32;
    let centre = side * 0.5;
    let face = STAR_FACE * side;
    let spoke = std::f32::consts::TAU / STAR_POINTS;
    // The edge of a point lying along +x, from its tip to the valley on its +y side.
    let tip = [STAR_REACH * side, 0.0];
    let valley = [
        STAR_VALLEY * side * (spoke * 0.5).cos(),
        STAR_VALLEY * side * (spoke * 0.5).sin(),
    ];
    let along = [valley[0] - tip[0], valley[1] - tip[1]];
    let mut out = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let p = [x as f32 + 0.5 - centre, y as f32 + 0.5 - centre];
            let length = (p[0] * p[0] + p[1] * p[1]).sqrt();
            // The angle off the nearest point's middle line, the first point straight up.
            let angle = p[1].atan2(p[0]) + std::f32::consts::FRAC_PI_2;
            let off = angle - (angle / spoke).round() * spoke;
            // Folded onto that edge: the distance to it in pixels, negative inside.
            let folded = [length * off.cos(), length * off.abs().sin()];
            let inside = along[0] * (folded[1] - tip[1]) - along[1] * (folded[0] - tip[0]);
            let outside = segment(folded, tip, valley).copysign(-inside);
            let unit = [p[0] / face, p[1] / face];
            let mut colour = mix(
                palette.top,
                palette.bottom,
                ((unit[1] + 1.0) * 0.5).clamp(0.0, 1.0),
            );
            let band = (1.0 - (unit[0] * 0.4 + unit[1] + 0.35).abs() / 0.18).clamp(0.0, 1.0);
            colour = mix(colour, [1.0; 3], band * 0.5);
            // From where the points leave the face, each one's far side in shade.
            let points = ((length - 0.3 * side) / (0.08 * side)).clamp(0.0, 1.0);
            let shaded = (off * length + 0.5).clamp(0.0, 1.0);
            colour = mix(colour, STAFF_SHADE, shaded * points * 0.5);
            let rim = (1.0 - (-outside / (side * 0.05)).clamp(0.0, 1.0)) * 0.9;
            colour = mix(colour, palette.rim, rim);
            let tick = coverage((tick_distance(unit) - TICK_HALF * 1.1) * face);
            colour = mix(colour, palette.tick, tick);
            push(&mut out, colour, coverage(outside));
        }
    }
    out
}

/// Frame `frame` of the creator's sun: a gold disc going orange at its edge, a ring of
/// pointed rays turning one ray's width over the loop (so the loop joins), a glint
/// crossing the disc in the loop's first [`GLINT_SHARE`], and the white tick.
fn sun(size: u32, frame: u32) -> Vec<u8> {
    let side = size as f32;
    let centre = side * 0.5;
    let core = SUN_RADIUS * side;
    let reach = RAY_REACH * side;
    let phase = (frame % CREATOR_FRAMES) as f32 / CREATOR_FRAMES as f32;
    let spoke = std::f32::consts::TAU / RAYS;
    let turn = phase * spoke;
    let glint = phase / GLINT_SHARE;
    let mut out = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let p = [x as f32 + 0.5 - centre, y as f32 + 0.5 - centre];
            let length = (p[0] * p[0] + p[1] * p[1]).sqrt();
            let angle = p[1].atan2(p[0]) - turn;
            // Off the nearest ray's middle line, in pixels at this distance.
            let off = ((angle / spoke).round() * spoke - angle).abs() * length;
            // Each ray narrows from a third of a spoke at the disc to a point at `reach`.
            let along = ((length - core) / (reach - core)).clamp(0.0, 1.0);
            let half = (1.0 - along) * core * spoke * 0.36;
            let ray = coverage(off - half).min(coverage(length - reach));
            let disc = coverage(length - core);
            let unit = [p[0] / core, p[1] / core];
            // The disc: warm gold in the middle, deep orange at the rim, dark enough
            // under the white tick.
            let depth = (length / core).clamp(0.0, 1.0);
            let mut colour = mix([1.0, 0.72, 0.2], [0.96, 0.42, 0.06], depth * depth);
            let rim = (1.0 - ((core - length) / (side * 0.03)).clamp(0.0, 1.0)) * 0.7;
            colour = mix(colour, [0.85, 0.35, 0.05], rim);
            if glint < 1.0 {
                // A band across the disc at 45 degrees, moving from top left to bottom right.
                let across = (unit[0] + unit[1]) * 0.5;
                let at = glint * 3.0 - 1.5;
                let band = (1.0 - ((across - at).abs() / 0.16)).clamp(0.0, 1.0);
                colour = mix(colour, [1.0, 1.0, 0.92], band * band * 0.75 * disc);
            }
            let tick =
                coverage((tick_distance([unit[0] / 0.98, unit[1] / 0.98]) - TICK_HALF) * core);
            colour = mix(colour, [1.0; 3], tick * disc);
            // Outside the disc, the rays: orange going to amber at their points.
            let ray_colour = mix([1.0, 0.62, 0.15], [1.0, 0.82, 0.3], along);
            let colour = mix(ray_colour, colour, disc);
            let alpha = disc.max(ray * (1.0 - along * 0.35));
            push(&mut out, colour, alpha);
        }
    }
    out
}

/// Low patches of 0..1, smooth, for the rust's stains: two octaves of value noise on a
/// fixed lattice, the same at every start.
fn patina(u: f32, v: f32) -> f32 {
    fn hash(x: i32, y: i32) -> f32 {
        let mut h = (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841);
        h ^= h >> 13;
        h = h.wrapping_mul(0x9e37_79b9);
        (h >> 8) as f32 / (1u32 << 24) as f32
    }
    fn noise(u: f32, v: f32) -> f32 {
        let (x, y) = (u.floor(), v.floor());
        let (fx, fy) = (u - x, v - y);
        let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
        let (x, y) = (x as i32, y as i32);
        let top = hash(x, y) + (hash(x + 1, y) - hash(x, y)) * sx;
        let bottom = hash(x, y + 1) + (hash(x + 1, y + 1) - hash(x, y + 1)) * sx;
        top + (bottom - top) * sy
    }
    noise(u * 5.0, v * 5.0) * 0.65 + noise(u * 13.0 + 7.0, v * 13.0 + 3.0) * 0.35
}

fn push(out: &mut Vec<u8>, colour: [f32; 3], alpha: f32) {
    out.extend(
        colour
            .iter()
            .map(|channel| (channel.clamp(0.0, 1.0) * 255.0).round() as u8)
            .chain([(alpha.clamp(0.0, 1.0) * 255.0).round() as u8]),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(pixels: &[u8], size: u32, x: u32, y: u32) -> [u8; 4] {
        let index = ((y * size + x) * 4) as usize;
        std::array::from_fn(|i| pixels[index + i])
    }

    #[test]
    fn each_group_seal_has_its_colour_and_a_clear_corner() {
        let size = 128;
        for (tick, check) in [
            (
                Tick::Jof,
                (|c: [u8; 4]| c[2] > 180 && c[0] < 120) as fn([u8; 4]) -> bool,
            ),
            (Tick::Sith, |c| c[0] > 130 && c[1] < 40 && c[2] < 40),
            (Tick::Jawa, |c| c[0] > c[1] && c[1] > c[2] && c[0] < 230),
            (Tick::Jkr, |c| c[0] > 120 && c[2] < 90),
            (Tick::Staff, |c| c[0] > 180 && c[1] > 180 && c[2] > 180),
        ] {
            let badge = pixels(size, tick, 0);
            assert_eq!(badge.len(), (size * size * 4) as usize, "{tick:?}");
            assert_eq!(at(&badge, size, 0, 0)[3], 0, "{tick:?}");
            let face = at(&badge, size, 64, 30);
            assert_eq!(face[3], 255, "{tick:?}");
            assert!(check(face), "{tick:?}: {face:?}");
        }
        assert!(pixels(size, Tick::Gold, 0).is_empty());
    }

    #[test]
    fn the_staff_tick_is_a_starburst() {
        let size = 128;
        let star = pixels(size, Tick::Staff, 0);
        let spoke = std::f32::consts::TAU / STAR_POINTS;
        // Alpha at `turn` radians clockwise from straight up, past the valleys' reach.
        let alpha = |turn: f32| {
            let reach = 0.44 * size as f32;
            let x = 64.0 + reach * turn.sin();
            let y = 64.0 - reach * turn.cos();
            at(&star, size, x as u32, y as u32)[3]
        };
        for point in 0..STAR_POINTS as u32 {
            let turn = point as f32 * spoke;
            assert!(alpha(turn) > 200, "point {point}: {}", alpha(turn));
            assert_eq!(alpha(turn + spoke * 0.5), 0, "between points {point}");
        }
    }

    #[test]
    fn the_sun_turns_and_its_loop_joins() {
        let size = 128;
        let first = pixels(size, Tick::Creator, 0);
        assert_ne!(first, pixels(size, Tick::Creator, CREATOR_FRAMES / 2));
        // The frame after the last is the first again.
        assert_eq!(first, pixels(size, Tick::Creator, CREATOR_FRAMES));
        // The disc's middle is opaque gold, the cell's corner clear.
        let middle = at(&first, size, 64, 40);
        assert_eq!(middle[3], 255);
        assert!(middle[0] > 230 && middle[2] < 200, "{middle:?}");
        assert_eq!(at(&first, size, 0, 0)[3], 0);
    }

    /// Writes every tick, the gold one first and the sun's frames last, side by side to
    /// `SJK_TICKS_PREVIEW` (a PNG path) to look at them.
    #[test]
    fn a_preview_can_be_written() {
        let Ok(path) = std::env::var("SJK_TICKS_PREVIEW") else {
            return;
        };
        let size = 128;
        let mut cells = vec![super::super::verified_badge::pixels(size)];
        for tick in [Tick::Jof, Tick::Sith, Tick::Jawa, Tick::Jkr, Tick::Staff] {
            cells.push(pixels(size, tick, 0));
        }
        for frame in (0..CREATOR_FRAMES).step_by(5) {
            cells.push(pixels(size, Tick::Creator, frame));
        }
        let mut sheet = image::RgbaImage::new(size * cells.len() as u32, size);
        for (index, cell) in cells.iter().enumerate() {
            let cell = image::RgbaImage::from_raw(size, size, cell.clone()).unwrap();
            image::imageops::overlay(&mut sheet, &cell, i64::from(index as u32 * size), 0);
        }
        sheet.save(path).unwrap();
    }
}
