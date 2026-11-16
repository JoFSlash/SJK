//! Wrap-around frequency-domain filters on power-of-two planes: an exact Gaussian blur
//! and the least-squares integration of a gradient field (Frankot and Chellappa's
//! method), which [`crate::shape`] uses to turn paint edges and painted shading into
//! height.
//!
//! The transform is a plain iterative radix-2 FFT in `f64`, rows then columns. Every
//! plane here tiles like the texture it comes from, so the periodic transform has no
//! edge to handle. Planes whose sides are not powers of two are resampled by the caller
//! ([`crate::filters::resample`]).

use crate::filters::Plane;
use std::f64::consts::TAU;

/// A plane's discrete Fourier transform, row-major like [`Plane`].
#[derive(Clone, Debug)]
pub struct Spectrum {
    pub width: usize,
    pub height: usize,
    re: Vec<f64>,
    im: Vec<f64>,
}

/// Whether both sides of `plane` are powers of two (the transform's requirement).
pub fn transformable(width: usize, height: usize) -> bool {
    width.is_power_of_two() && height.is_power_of_two()
}

impl Spectrum {
    /// The transform of `plane`, whose sides must be powers of two.
    pub fn forward(plane: &Plane) -> Self {
        assert!(
            transformable(plane.width, plane.height),
            "spectral planes have power-of-two sides, not {}x{}",
            plane.width,
            plane.height
        );
        let mut spectrum = Self {
            width: plane.width,
            height: plane.height,
            re: plane.data.iter().map(|value| f64::from(*value)).collect(),
            im: vec![0.0; plane.data.len()],
        };
        spectrum.transform(false);
        spectrum
    }

    /// The real part of the inverse transform.
    pub fn inverse(mut self) -> Plane {
        self.transform(true);
        let scale = 1.0 / (self.width * self.height) as f64;
        Plane {
            width: self.width,
            height: self.height,
            data: self.re.iter().map(|value| (value * scale) as f32).collect(),
        }
    }

    /// Angular frequencies (radians per texel, -π to π) of column `x` and row `y`.
    fn frequency(&self, x: usize, y: usize) -> (f64, f64) {
        let angular = |index: usize, length: usize| {
            let signed = if index < length.div_ceil(2) {
                index as f64
            } else {
                index as f64 - length as f64
            };
            TAU * signed / length as f64
        };
        (angular(x, self.width), angular(y, self.height))
    }

    fn transform(&mut self, inverse: bool) {
        let (width, height) = (self.width, self.height);
        let mut re = vec![0.0; width.max(height)];
        let mut im = vec![0.0; width.max(height)];
        for row in 0..height {
            let span = row * width..(row + 1) * width;
            fft(&mut self.re[span.clone()], &mut self.im[span], inverse);
        }
        for column in 0..width {
            for y in 0..height {
                re[y] = self.re[y * width + column];
                im[y] = self.im[y * width + column];
            }
            fft(&mut re[..height], &mut im[..height], inverse);
            for y in 0..height {
                self.re[y * width + column] = re[y];
                self.im[y * width + column] = im[y];
            }
        }
    }
}

/// In-place iterative radix-2 FFT of one line (`e^{-iωn}` forward, unscaled).
fn fft(re: &mut [f64], im: &mut [f64], inverse: bool) {
    let length = re.len();
    if length < 2 {
        return;
    }
    let mut j = 0;
    for i in 1..length {
        let mut bit = length >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
    let sign = if inverse { 1.0 } else { -1.0 };
    let mut size = 2;
    while size <= length {
        let step = sign * TAU / size as f64;
        for start in (0..length).step_by(size) {
            for k in 0..size / 2 {
                let (sin, cos) = (step * k as f64).sin_cos();
                let (a, b) = (start + k, start + k + size / 2);
                let (tr, ti) = (re[b] * cos - im[b] * sin, re[b] * sin + im[b] * cos);
                re[b] = re[a] - tr;
                im[b] = im[a] - ti;
                re[a] += tr;
                im[a] += ti;
            }
        }
        size *= 2;
    }
}

/// Exact wrap-around Gaussian blur of standard deviation `sigma` texels.
pub fn gaussian(plane: &Plane, sigma: f32) -> Plane {
    if sigma <= 0.0 {
        return plane.clone();
    }
    let mut spectrum = Spectrum::forward(plane);
    let variance = f64::from(sigma).powi(2);
    for y in 0..spectrum.height {
        for x in 0..spectrum.width {
            let (u, v) = spectrum.frequency(x, y);
            let gain = (-0.5 * variance * (u * u + v * v)).exp();
            let index = y * spectrum.width + x;
            spectrum.re[index] *= gain;
            spectrum.im[index] *= gain;
        }
    }
    spectrum.inverse()
}

/// Central-difference gradient of `plane` (units per texel), wrapping.
pub fn gradient(plane: &Plane) -> (Plane, Plane) {
    let along = |dx: isize, dy: isize| {
        Plane::from_fn(plane.width, plane.height, |x, y| {
            let (x, y) = (x as isize, y as isize);
            0.5 * (plane.wrapped(x + dx, y + dy) - plane.wrapped(x - dx, y - dy))
        })
    };
    (along(1, 0), along(0, 1))
}

/// The height whose central-difference gradient best matches (`dx`, `dy`) in the least
/// squares sense (Frankot and Chellappa), with mean 0. Gradients that no height has
/// (the field's curl) are dropped. Shapes much larger than `reach` texels are flattened:
/// a field that does not integrate exactly would otherwise leave ramps across the
/// whole texture.
pub fn integrate(dx: &Plane, dy: &Plane, reach: f32) -> Plane {
    let gx = Spectrum::forward(dx);
    let gy = Spectrum::forward(dy);
    let mut height = gx.clone();
    let floor = (TAU / f64::from(reach.max(1.0))).powi(2);
    for y in 0..gx.height {
        for x in 0..gx.width {
            let index = y * gx.width + x;
            let (u, v) = gx.frequency(x, y);
            // The central difference's own response, i·sin ω, keeps the result exact for
            // the gradients `gradient` measures.
            let (su, sv) = (u.sin(), v.sin());
            let denominator = su * su + sv * sv + floor;
            // H = -i (su Gx + sv Gy) / denominator.
            let (re, im) = (
                su * gx.re[index] + sv * gy.re[index],
                su * gx.im[index] + sv * gy.im[index],
            );
            height.re[index] = im / denominator;
            height.im[index] = -re / denominator;
        }
    }
    height.re[0] = 0.0;
    height.im[0] = 0.0;
    height.inverse()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: &Plane, b: &Plane, tolerance: f32) -> bool {
        a.data
            .iter()
            .zip(&b.data)
            .all(|(a, b)| (a - b).abs() <= tolerance)
    }

    #[test]
    fn transform_round_trips() {
        let plane = Plane::from_fn(16, 8, |x, y| ((x * 7 + y * 3) % 5) as f32 - 2.0);
        assert!(close(&Spectrum::forward(&plane).inverse(), &plane, 1e-5));
        // One-texel and one-row planes too.
        let line = Plane::from_fn(8, 1, |x, _| x as f32);
        assert!(close(&Spectrum::forward(&line).inverse(), &line, 1e-5));
    }

    #[test]
    fn gaussian_keeps_the_mean_and_wraps() {
        let spike = Plane::from_fn(32, 32, |x, y| f32::from(x == 0 && y == 0));
        let blurred = gaussian(&spike, 2.0);
        let sum: f32 = blurred.data.iter().sum();
        assert!((sum - 1.0).abs() < 1e-4);
        // Symmetric across the seam.
        assert!((blurred.at(1, 0) - blurred.at(31, 0)).abs() < 1e-6);
        assert!((blurred.at(0, 2) - blurred.at(0, 30)).abs() < 1e-6);
        assert!(blurred.at(0, 0) < 0.05);
        // A constant stays constant.
        let flat = Plane::filled(16, 16, 0.3);
        assert!(close(&gaussian(&flat, 3.0), &flat, 1e-5));
    }

    #[test]
    fn integration_recovers_a_height_from_its_gradient() {
        let height = Plane::from_fn(64, 32, |x, y| {
            let (s, t) = (TAU * x as f64 / 64.0, TAU * y as f64 / 32.0);
            (s.sin() * 0.5 + (2.0 * t).cos() * 0.25 + (s + t).sin() * 0.1) as f32
        });
        let (dx, dy) = gradient(&height);
        // A reach of many textures leaves every shape of the texture in place.
        let recovered = integrate(&dx, &dy, 1.0e6);
        assert!(close(&recovered, &height, 1e-3));
    }

    #[test]
    fn a_slope_along_one_axis_integrates_to_a_step() {
        // A one-row band of slope +1 down the image: a step up of one unit, wrapping
        // back down evenly over the rest of the texture.
        let dy = Plane::from_fn(32, 64, |_, y| f32::from(y == 20));
        let dx = Plane::filled(32, 64, 0.0);
        let height = integrate(&dx, &dy, 1.0e6);
        assert!(height.at(5, 21) - height.at(5, 19) > 0.9);
        // The same everywhere along the row: no streaks across the image.
        assert!((height.at(0, 25) - height.at(17, 25)).abs() < 1e-4);
        // The field's curl is dropped: a slope along x on one row has no height.
        let curl = integrate(&dy, &dx, 1.0e6);
        assert!(curl.data.iter().all(|v| v.abs() < 0.1));
    }

    #[test]
    fn reach_flattens_large_shapes() {
        let height = Plane::from_fn(64, 64, |x, _| (TAU * x as f64 / 64.0).sin() as f32);
        let (dx, dy) = gradient(&height);
        let kept = integrate(&dx, &dy, 1.0e6);
        let flattened = integrate(&dx, &dy, 4.0);
        let range = |plane: &Plane| {
            plane.data.iter().copied().fold(f32::MIN, f32::max)
                - plane.data.iter().copied().fold(f32::MAX, f32::min)
        };
        assert!(range(&flattened) < 0.1 * range(&kept));
    }
}
