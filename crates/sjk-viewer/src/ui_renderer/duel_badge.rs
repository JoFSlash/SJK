//! The duel badge: two crossed lightsabers, white blades in a coloured glow, on a
//! disc filled and ringed in the same colour, one badge per duel colour ([`COLOURS`]), drawn once at start into an icon cell
//! ([`super::duel_texture`]). The nameplates put it before the name of a player
//! who is in a private duel.
//!
//! It is computed rather than loaded, like the verified badge
//! ([`super::verified_badge`]): signed distances to the disc, its ring and the
//! sabers' strokes, each blended over one pixel, so the edges stay smooth at the
//! cell's size and the sampler's scaling keeps them so.

/// The disc's radius, as a share of the cell.
const RADIUS: f32 = 0.46;
/// The ring's width, as a share of the radius.
const RING: f32 = 0.13;
/// The disc's fill, top to bottom, and its opacity.
const DISC_TOP: [f32; 3] = [0.07, 0.11, 0.24];
const DISC_BOTTOM: [f32; 3] = [0.03, 0.05, 0.12];
const DISC_ALPHA: f32 = 0.95;
/// How much of the duel colour tints the disc, so the whole badge reads as it even
/// small, at the top and the bottom of the disc.
const TINT: f32 = 0.9;
const TINT_TOP: f32 = 0.85;
const TINT_BOTTOM: f32 = 0.55;
/// The duels' colours, for the ring and the blades' glow: the two duellists of one
/// duel wear the same one ([`crate::hud::duel_pairs`]). The duel blue first; the last,
/// grey, is a duellist's whose opponent is unknown. Saturated, so they tell apart at a
/// nameplate's size.
pub(crate) const COLOURS: [[f32; 3]; 8] = [
    [0.25, 0.5, 1.0],
    [1.0, 0.55, 0.12],
    [0.3, 0.88, 0.32],
    [0.95, 0.35, 0.9],
    [1.0, 0.88, 0.2],
    [0.25, 0.9, 0.95],
    [1.0, 0.25, 0.25],
    [0.6, 0.62, 0.66],
];
/// A saber's hilt end and blade tip, in radii from the centre (y down); the other
/// saber is its mirror image.
const HILT: [f32; 2] = [-0.56, 0.56];
const TIP: [f32; 2] = [0.56, -0.56];
/// Where the hilt ends and the blade starts, as a share of the way to the tip.
const EMITTER: f32 = 0.3;
/// Half thicknesses, in radii: the hilt, the blade's white core and its glow.
const HILT_HALF: f32 = 0.085;
const CORE_HALF: f32 = 0.045;
const GLOW_HALF: f32 = 0.19;
/// The hilt's metal.
const METAL: [f32; 3] = [0.5, 0.53, 0.58];

/// Distance from `p` to the segment `a`..`b`.
fn segment(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let ap = [p[0] - a[0], p[1] - a[1]];
    let t = ((ap[0] * ab[0] + ap[1] * ab[1]) / (ab[0] * ab[0] + ab[1] * ab[1])).clamp(0.0, 1.0);
    let d = [ap[0] - ab[0] * t, ap[1] - ab[1] * t];
    (d[0] * d[0] + d[1] * d[1]).sqrt()
}

/// Coverage of an edge at signed distance `d` pixels (negative inside).
fn coverage(d: f32) -> f32 {
    (0.5 - d).clamp(0.0, 1.0)
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t)
}

/// One saber from `hilt` to `tip`, at `p` (radii): how much hilt, blade core and
/// glow cover the pixel, `radius` pixels to a radius.
fn saber(p: [f32; 2], hilt: [f32; 2], tip: [f32; 2], radius: f32) -> [f32; 3] {
    let emitter = [
        hilt[0] + (tip[0] - hilt[0]) * EMITTER,
        hilt[1] + (tip[1] - hilt[1]) * EMITTER,
    ];
    let handle = coverage((segment(p, hilt, emitter) - HILT_HALF) * radius);
    let blade = segment(p, emitter, tip);
    let core = coverage((blade - CORE_HALF) * radius);
    // The glow falls off smoothly from the core to its edge.
    let glow = (1.0 - (blade - CORE_HALF) / (GLOW_HALF - CORE_HALF)).clamp(0.0, 1.0);
    [handle, core, glow * glow]
}

/// The badge as `size` x `size` straight (not premultiplied) RGBA bytes.
pub(crate) fn pixels(size: u32, colour: [f32; 3]) -> Vec<u8> {
    let side = size as f32;
    let radius = RADIUS * side;
    let centre = side * 0.5;
    let mut out = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let p = [x as f32 + 0.5 - centre, y as f32 + 0.5 - centre];
            let length = (p[0] * p[0] + p[1] * p[1]).sqrt();
            let disc = coverage(length - radius);
            let ring = coverage(radius * (1.0 - RING) - length) * disc;
            let unit = [p[0] / radius, p[1] / radius];
            let tinted = |dark: [f32; 3], share: f32| {
                std::array::from_fn(|i| dark[i] * (1.0 - TINT) + colour[i] * TINT * share)
            };
            let fill = mix(
                tinted(DISC_TOP, TINT_TOP),
                tinted(DISC_BOTTOM, TINT_BOTTOM),
                ((unit[1] + 1.0) * 0.5).clamp(0.0, 1.0),
            );
            let mut out_colour = mix(fill, colour, ring);
            let mut alpha = disc * (DISC_ALPHA + (1.0 - DISC_ALPHA) * ring);
            // The second saber is drawn over the first, so they read as crossed.
            for mirror in [1.0, -1.0] {
                let [handle, core, glow] = saber(
                    unit,
                    [HILT[0] * mirror, HILT[1]],
                    [TIP[0] * mirror, TIP[1]],
                    radius,
                );
                // Only inside the ring: the badge keeps its round edge.
                let inside = coverage(length - radius * (1.0 - RING));
                out_colour = mix(out_colour, colour, glow * inside);
                out_colour = mix(out_colour, [1.0; 3], core * inside);
                out_colour = mix(out_colour, METAL, handle * inside);
                alpha = alpha.max(disc * inside * handle.max(core));
            }
            out.extend(
                out_colour
                    .iter()
                    .map(|channel| (channel.clamp(0.0, 1.0) * 255.0).round() as u8)
                    .chain([(alpha.clamp(0.0, 1.0) * 255.0).round() as u8]),
            );
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(pixels: &[u8], size: u32, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * size + x) * 4) as usize;
        [pixels[i], pixels[i + 1], pixels[i + 2], pixels[i + 3]]
    }

    #[test]
    fn the_badge_is_a_round_disc_with_white_crossed_blades() {
        let size = 128;
        let pixels = pixels(size, COLOURS[0]);
        assert_eq!(pixels.len(), (size * size * 4) as usize);
        // Corners are clear: the badge is round.
        assert_eq!(at(&pixels, size, 0, 0)[3], 0);
        assert_eq!(at(&pixels, size, size - 1, size - 1)[3], 0);
        // The blades cross at the centre in white.
        let centre = at(&pixels, size, size / 2, size / 2);
        assert!(centre[..3].iter().all(|&c| c > 230), "{centre:?}");
        assert_eq!(centre[3], 255);
        // The ring at the top is the duel blue.
        let ring = at(
            &pixels,
            size,
            size / 2,
            (size as f32 * (0.5 - RADIUS * 0.95)) as u32,
        );
        assert!(ring[2] > 200 && ring[0] < 140, "{ring:?}");
    }

    #[test]
    fn each_duel_colour_rings_its_own_badge_with_white_blades() {
        let size = 64;
        let top = (size as f32 * (0.5 - RADIUS * 0.96)) as u32;
        for colour in COLOURS {
            let pixels = pixels(size, colour);
            let ring = at(&pixels, size, size / 2, top);
            let expected = colour.map(|c| (c * 255.0) as i32);
            for channel in 0..3 {
                assert!(
                    (i32::from(ring[channel]) - expected[channel]).abs() < 40,
                    "{ring:?} {colour:?}"
                );
            }
            let centre = at(&pixels, size, size / 2, size / 2);
            assert!(centre[..3].iter().all(|&c| c > 200), "{centre:?}");
        }
    }
}
