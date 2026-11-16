//! Shape relief: the large shapes and bevels of a painted texture, which the band-pass
//! height of [`crate::generate`] loses.
//!
//! The band-pass height high-passes the luminance at an eighth of the texture to strip
//! lighting baked into the paint, and with it every plate, inset panel and crate face
//! larger than that: a raised plate became a thin ridge round its edge with a dish
//! inside, and a whole-wall panel frame was lost (Sol's world notes, 07/10 to 13/10/2026:
//! FFA5's `taspir` walls and trims, FFA3's crates, FFA1's `byss/bay_trim_bumper`, "make
//! it 3d"). Baked lighting and shapes differ in how they change, not in how large they
//! are: lighting changes slowly, a shape's edge changes sharply. So this relief is built
//! in the gradient domain, at most [`WORK_SIZE`] texels across:
//!
//! 1. **Edges** (after Land's retinex, which splits paint from light the same way). The
//!    luminance's gradient, slightly smoothed, is kept where it is steep and dropped
//!    where it is gentle: [`smoothstep`] from the texture's 80th to its 97th percentile
//!    of steepness (no lower than [`EDGE_FLOOR`] and [`EDGE_SPAN`] times it), so baked
//!    gradients and most texel grain go, plate and seam edges stay. The kept field is
//!    integrated ([`crate::spectral::integrate`]): plates become plateaus at their own
//!    brightness. Each gradient is measured against its surroundings
//!    ([`EDGE_SURROUNDINGS`]), and the integrated height's slow part
//!    ([`EDGE_FLATTEN`]) is taken out, so baked light leaves no slope across a plate.
//! 2. **Painted shading.** Retail textures are painted lit from above, so a bevel facing
//!    up is painted light and one facing down dark. The fine luminance (a band of
//!    [`SHADING_BAND`] texels at 512) is taken as the height's slope down the image and
//!    integrated: a light line over a dark one becomes a step, as a bevel is, instead of
//!    a ridge and a groove. Weighted by [`SHADING_WEIGHT`]; only the vertical light is
//!    used (the horizontal one cannot be told from the paint).
//!
//! Both are in luminance units, so [`crate::generate`] mixes them with its band-pass
//! height by the class's `shape` weight and normalises the result as before.
//! Alpha-tested textures (grates, cut-outs) get none: their holes are not plates.

use crate::filters::{Plane, halve, percentile, resample};
use crate::spectral::{gaussian, gradient, integrate, transformable};

/// Largest side of the plane the shape relief is computed on: the normals of larger
/// textures are measured per 1/512 of the texture anyway
/// ([`crate::generate::SLOPE_REFERENCE`]).
pub const WORK_SIZE: usize = 512;

/// Smallest side (in texels) a texture needs for a shape relief.
pub const MIN_SIZE: usize = 16;

/// Smoothing (Gaussian sigma in texels at 512) of the luminance before its edges are
/// measured, so texel grain does not count as edges.
pub const EDGE_SMOOTHING: f32 = 1.5;

/// Steepness (luminance per 1/512 of the texture) below which a gradient is never an
/// edge, and how many times the lower threshold the upper one is at least: a texture
/// with only gentle shading keeps no edges.
pub const EDGE_FLOOR: f32 = 0.006;
pub const EDGE_SPAN: f32 = 2.5;

/// Percentiles of a texture's steepness between which gradients turn into edges.
pub const EDGE_PERCENTILES: (f32, f32) = (0.80, 0.97);

/// How far (in textures) the edge height carries a step before fading
/// ([`crate::spectral::integrate`]'s reach): beyond the texture, so plates stay plateaus.
pub const EDGE_REACH: f32 = 2.0;

/// Gaussian sigma (texels at 512) of the local mean gradient taken out before edges are
/// kept: an edge is steep against its own surroundings, and a baked light's slope under
/// it is not kept with it.
pub const EDGE_SURROUNDINGS: f32 = 16.0;

/// Gaussian sigma (as a fraction of the texture) of the edge height's low part, taken
/// out: where the baked light changes an edge's contrast along it (a frame brighter at
/// the top), the integrated plates slope across the texture, and that slope would take
/// the height's range from the shapes.
pub const EDGE_FLATTEN: f32 = 0.125;

/// The painted-shading band: Gaussian sigmas (texels at 512) of the fine and the coarse
/// luminance whose difference is taken as slope.
pub const SHADING_BAND: (f32, f32) = (0.7, 24.0);

/// How far (in textures) painted shading carries a step before fading: shorter than the
/// edges', so paint that is not shading (a stain's rim) cannot tilt a whole plate.
pub const SHADING_REACH: f32 = 0.25;

/// Width (texels at 512) of a typical painted bevel: the integrated shading is divided
/// by it, so it comes out in luminance units like the edges.
pub const SHADING_WIDTH: f32 = 4.0;

/// Weight of the painted shading against the edges.
pub const SHADING_WEIGHT: f32 = 0.35;

/// The two parts of a texture's shape relief, at the texture's size, in luminance units.
#[derive(Clone, Debug)]
pub struct ShapeRelief {
    /// Plates and seams from the paint's edges: a plate as much higher than its frame as
    /// it is brighter. Like the band-pass height, bright is high, so it is turned upside
    /// down with it.
    pub edges: Plane,
    /// Steps from the painted shading, [`SHADING_WEIGHT`] included. Lit from above
    /// whichever way the relief is turned, so never turned.
    pub shading: Plane,
}

impl ShapeRelief {
    /// The two parts summed, the right way up.
    pub fn total(&self) -> Plane {
        self.edges.zip_map(&self.shading, |edge, step| edge + step)
    }
}

/// The shape relief of `luminance` (0–1), or `None` for textures smaller than
/// [`MIN_SIZE`].
pub fn shape_relief(luminance: &Plane) -> Option<ShapeRelief> {
    let (width, height) = (luminance.width, luminance.height);
    if width.min(height) < MIN_SIZE {
        return None;
    }
    let mut work = luminance.clone();
    while work.width.max(work.height) > WORK_SIZE
        && work.width.is_multiple_of(2)
        && work.height.is_multiple_of(2)
    {
        work = halve(&work);
    }
    let side = |length: usize| {
        if length.is_power_of_two() {
            length
        } else {
            // The nearest power of two, at most WORK_SIZE.
            let up = length.next_power_of_two();
            let down = up / 2;
            if length - down < up - length {
                down
            } else {
                up
            }
            .clamp(MIN_SIZE, WORK_SIZE)
        }
    };
    let (work_width, work_height) = (side(work.width), side(work.height));
    let work = resample(&work, work_width, work_height);
    debug_assert!(transformable(work.width, work.height));
    // Texels per 1/512 of the texture, at the working size.
    let scale = work.width.max(work.height) as f32 / 512.0;

    // 1. Edges.
    let smooth = gaussian(&work, EDGE_SMOOTHING * scale);
    let (dx, dy) = gradient(&smooth);
    let steepness = dx.zip_map(&dy, |x, y| x.hypot(y) * scale);
    let low = percentile(&steepness, None, EDGE_PERCENTILES.0).max(EDGE_FLOOR);
    let high = percentile(&steepness, None, EDGE_PERCENTILES.1).max(low * EDGE_SPAN);
    let keep = steepness.map(|value| smoothstep(low, high, value));
    let size = work.width.max(work.height) as f32;
    let kept = |gradient: &Plane| {
        let surroundings = gaussian(gradient, EDGE_SURROUNDINGS * scale);
        Plane::from_fn(gradient.width, gradient.height, |x, y| {
            (gradient.at(x, y) - surroundings.at(x, y)) * keep.at(x, y)
        })
    };
    let edges = integrate(&kept(&dx), &kept(&dy), EDGE_REACH * size);
    let edges = edges.zip_map(&gaussian(&edges, EDGE_FLATTEN * size), |edge, low| {
        edge - low
    });

    // 2. Painted shading: light from above, so the fine luminance is the slope down the
    // image (+t).
    let shading = gaussian(&work, SHADING_BAND.0 * scale)
        .zip_map(&gaussian(&work, SHADING_BAND.1 * scale), |fine, coarse| {
            fine - coarse
        });
    let flat = Plane::filled(work.width, work.height, 0.0);
    let steps = integrate(&flat, &shading, SHADING_REACH * size);
    let gain = SHADING_WEIGHT / (SHADING_WIDTH * scale).max(1.0);
    Some(ShapeRelief {
        edges: resample(&edges, width, height),
        shading: resample(&steps.map(|step| gain * step), width, height),
    })
}

/// 0 below `edge0`, 1 above `edge1`, smooth between.
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A light plate (0.7) a quarter of the texture across over a darker frame (0.35),
    /// with a baked light falling off across the whole texture and some texel grain.
    fn plate(size: usize, bake: f32) -> Plane {
        let inside = 3 * size / 8..5 * size / 8;
        Plane::from_fn(size, size, |x, y| {
            let hash = ((x * 7919 + y * 104_729) as u32)
                .wrapping_mul(2_654_435_761)
                .rotate_left(13);
            let grain = ((hash >> 24) as f32 / 255.0 - 0.5) * 0.04;
            let paint = if inside.contains(&x) && inside.contains(&y) {
                0.7
            } else {
                0.35
            };
            let light = bake * (std::f32::consts::TAU * x as f32 / size as f32).sin();
            paint + light + grain
        })
    }

    fn mean(plane: &Plane, xs: std::ops::Range<usize>, ys: std::ops::Range<usize>) -> f32 {
        let mut sum = 0.0;
        for y in ys.clone() {
            for x in xs.clone() {
                sum += plane.at(x, y);
            }
        }
        sum / (xs.len() * ys.len()) as f32
    }

    #[test]
    fn plates_step_up_and_baked_light_goes() {
        // The plate spans 96..160 of 256; the baked light is 0.2 brighter at its left
        // edge than at its right one.
        let shape = shape_relief(&plate(256, 0.15)).expect("a shape").total();
        let rows = 120..136;
        // Both edges step up by most of the brightness step (0.35).
        let left_step = mean(&shape, 100..104, rows.clone()) - mean(&shape, 88..92, rows.clone());
        let right_step =
            mean(&shape, 152..156, rows.clone()) - mean(&shape, 164..168, rows.clone());
        assert!(left_step > 0.2, "left step {left_step}");
        assert!(right_step > 0.2, "right step {right_step}");
        // The baked light left no slope: the plate is as high at both edges.
        let tilt = mean(&shape, 100..104, rows.clone()) - mean(&shape, 152..156, rows.clone());
        assert!(tilt.abs() < 0.05, "tilt {tilt}");
        // Without the bake, the same plate.
        let unbaked = shape_relief(&plate(256, 0.0)).expect("a shape").total();
        let difference = shape
            .data
            .iter()
            .zip(&unbaked.data)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f32::max);
        assert!(difference < 0.08, "difference {difference}");
    }

    #[test]
    fn gentle_paint_has_no_shape() {
        // Only baked light and grain: nothing is steep enough to be an edge.
        let size = 64;
        let paint = Plane::from_fn(size, size, |x, _| {
            0.5 + 0.2 * (std::f32::consts::TAU * x as f32 / size as f32).sin()
        });
        let shape = shape_relief(&paint).expect("a shape").total();
        assert!(
            shape.data.iter().all(|v| v.abs() < 0.02),
            "{:?}",
            &shape.data[..8]
        );
        assert!(shape_relief(&Plane::filled(8, 8, 0.5)).is_none());
    }

    #[test]
    fn painted_bevels_become_steps() {
        // A plate the same colour as its frame, its top rim painted light and its bottom
        // rim dark: lit from above, it is raised, and only the shading says so.
        let paint = Plane::from_fn(256, 256, |x, y| {
            let across = (64..192).contains(&x);
            match y {
                64..68 if across => 0.75,
                188..192 if across => 0.15,
                _ => 0.45,
            }
        });
        let shape = shape_relief(&paint).expect("a shape").total();
        let columns = 112..144;
        let inside_top = mean(&shape, columns.clone(), 70..76);
        let above = mean(&shape, columns.clone(), 56..62);
        let inside_bottom = mean(&shape, columns.clone(), 180..186);
        let below = mean(&shape, columns.clone(), 194..200);
        assert!(
            inside_top > above + 0.02,
            "inside {inside_top} above {above}"
        );
        assert!(
            inside_bottom > below + 0.02,
            "inside {inside_bottom} below {below}"
        );
        // The same rims upside down (dark top, light bottom) sink the plate.
        let sunk = shape_relief(&paint.map(|v| 0.9 - v))
            .expect("a shape")
            .total();
        assert!(mean(&sunk, columns.clone(), 70..76) < mean(&sunk, columns, 56..62));
    }

    #[test]
    fn odd_sizes_and_large_textures_keep_their_size_and_tiling() {
        let paint = Plane::from_fn(100, 60, |x, y| f32::from((x / 25 + y / 20) % 2 == 0) * 0.5);
        let shape = shape_relief(&paint).expect("a shape").total();
        assert_eq!((shape.width, shape.height), (100, 60));
        let large = plate(1024, 0.0);
        let shape = shape_relief(&large).expect("a shape").total();
        assert_eq!((shape.width, shape.height), (1024, 1024));
        // The seam is no steeper than the inside of the frame.
        let seam = (0..1024)
            .map(|y| (shape.at(1023, y) - shape.at(0, y)).abs())
            .fold(0.0, f32::max);
        assert!(seam < 0.02, "seam {seam}");
    }
}
