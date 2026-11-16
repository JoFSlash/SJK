//! The trust marks (`PROTOCOL.md` of the hub, "Trust"), drawn once at start into icon
//! cells ([`super::trust_texture`]) next to the gold verified seal
//! ([`super::verified_badge`]): an empty ring with a grey tick for a new player, a green
//! disc with a white tick for a trusted one, a red disc with a white cross while a chat
//! ban holds them. Round where the seal is scalloped, so green never reads as verified.
//!
//! Computed like the seal: signed distances blended over one pixel.

use super::verified_badge::{TICK, TICK_HALF, coverage, mix, segment};
use sjk_identity::Trust;

/// The disc's radius, as a share of the cell: the seal's mean radius.
const RADIUS: f32 = 0.43;
/// The new mark's ring and tick: a light grey that reads on dark and mid grounds.
const RING: [f32; 3] = [0.8, 0.82, 0.85];
/// The ring's half thickness, in radii.
const RING_HALF: f32 = 0.07;
/// Green at the top and bottom of the trusted disc, and its rim.
const GREEN_TOP: [f32; 3] = [0.42, 0.86, 0.45];
const GREEN_BOTTOM: [f32; 3] = [0.13, 0.62, 0.27];
const GREEN_RIM: [f32; 3] = [0.07, 0.42, 0.17];
/// Red at the top and bottom of the banned disc, and its rim.
const RED_TOP: [f32; 3] = [0.97, 0.38, 0.35];
const RED_BOTTOM: [f32; 3] = [0.78, 0.13, 0.13];
const RED_RIM: [f32; 3] = [0.52, 0.06, 0.06];
/// The cross's arm reach, in radii from the centre.
const CROSS: f32 = 0.36;

/// The mark for `trust` as `size` x `size` straight RGBA bytes; `Unknown` is clear.
pub(crate) fn pixels(size: u32, trust: Trust) -> Vec<u8> {
    let side = size as f32;
    let radius = RADIUS * side;
    let centre = side * 0.5;
    let mut out = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let p = [x as f32 + 0.5 - centre, y as f32 + 0.5 - centre];
            let length = (p[0] * p[0] + p[1] * p[1]).sqrt();
            let unit = [p[0] / radius, p[1] / radius];
            let tick = TICK
                .windows(2)
                .map(|pair| segment(unit, pair[0], pair[1]))
                .fold(f32::INFINITY, f32::min);
            let tick = coverage((tick - TICK_HALF) * radius);
            let outside = length - radius;
            let disc = coverage(outside);
            let fill = |top, bottom, rim| {
                let shade = mix(top, bottom, ((unit[1] + 1.0) * 0.5).clamp(0.0, 1.0));
                let edge = (1.0 - (-outside / (side * 0.03)).clamp(0.0, 1.0)) * 0.8;
                mix(shade, rim, edge)
            };
            let (colour, alpha) = match trust {
                Trust::New => {
                    let ring = coverage(outside.abs() - RING_HALF * radius);
                    (RING, ring.max(tick))
                }
                Trust::Trusted => {
                    let green = fill(GREEN_TOP, GREEN_BOTTOM, GREEN_RIM);
                    (mix(green, [1.0; 3], tick), disc)
                }
                Trust::Banned => {
                    let cross = segment(unit, [-CROSS, -CROSS], [CROSS, CROSS]).min(segment(
                        unit,
                        [-CROSS, CROSS],
                        [CROSS, -CROSS],
                    ));
                    let cross = coverage((cross - TICK_HALF) * radius);
                    let red = fill(RED_TOP, RED_BOTTOM, RED_RIM);
                    (mix(red, [1.0; 3], cross), disc)
                }
                Trust::Unknown => ([0.0; 3], 0.0),
            };
            out.extend(
                colour
                    .iter()
                    .map(|channel| (channel * 255.0).round() as u8)
                    .chain([(alpha * 255.0).round() as u8]),
            );
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(pixels: &[u8], size: u32, x: u32, y: u32) -> [u8; 4] {
        let index = ((y * size + x) * 4) as usize;
        std::array::from_fn(|channel| pixels[index + channel])
    }

    #[test]
    fn new_is_an_empty_ring_trusted_green_and_banned_red() {
        let size = 128;
        let new = pixels(size, Trust::New);
        // The middle of the ring, away from the tick, is clear; the ring is not.
        assert_eq!(at(&new, size, 64, 30)[3], 0);
        assert_eq!(at(&new, size, 64, 64 - (RADIUS * 128.0) as u32)[3], 255);
        let green = at(&pixels(size, Trust::Trusted), size, 64, 30);
        assert!(
            green[1] > 150 && green[0] < 150 && green[3] == 255,
            "{green:?}"
        );
        let red = at(&pixels(size, Trust::Banned), size, 64, 30);
        assert!(red[0] > 180 && red[1] < 120 && red[3] == 255, "{red:?}");
        // The cross's middle is white.
        let middle = at(&pixels(size, Trust::Banned), size, 64, 64);
        assert!(middle.iter().all(|channel| *channel > 230), "{middle:?}");
        assert!(pixels(size, Trust::Unknown).iter().all(|byte| *byte == 0));
    }

    /// Writes the three marks side by side to `SJK_TRUST_PREVIEW` (a PNG path).
    #[test]
    fn a_preview_can_be_written() {
        let Ok(path) = std::env::var("SJK_TRUST_PREVIEW") else {
            return;
        };
        let size = 128;
        let marks = [Trust::New, Trust::Trusted, Trust::Banned].map(|trust| pixels(size, trust));
        let mut sheet = image::RgbaImage::new(size * 3, size);
        for (index, mark) in marks.iter().enumerate() {
            for y in 0..size {
                for x in 0..size {
                    let pixel = at(mark, size, x, y);
                    sheet.put_pixel(index as u32 * size + x, y, image::Rgba(pixel));
                }
            }
        }
        sheet.save(path).unwrap();
    }
}
