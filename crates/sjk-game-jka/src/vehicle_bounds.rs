//! Geometry of codemp `BG_VehicleAdjustBBoxForOrientation` (OpenJK 4d0dfaf1).
//!
//! This is the shape calculation only. Authoritative movement checks clearance;
//! cgame's remote-vehicle collision passes no clearance callback.

use crate::pmove::flight;
use crate::vehicle_fields::{VehicleInfo, kind};

/// Definition-owned dimensions used by local and remote vehicle collision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleBounds {
    kind: i32,
    length: f32,
    width: f32,
    height: f32,
}

impl VehicleBounds {
    /// Read a vehicle definition. Any zero dimension leaves its linked box alone.
    pub fn from_info(info: &VehicleInfo) -> Option<Self> {
        (info.length != 0.0 && info.width != 0.0 && info.height != 0.0).then_some(Self {
            kind: info.kind,
            length: info.length,
            width: info.width,
            height: info.height,
        })
    }

    /// Whether the reference checks the new orientation-dependent box for clearance.
    pub fn dynamic(self) -> bool {
        matches!(self.kind, kind::FIGHTER | kind::FLIER)
    }

    /// Expand around nose, tail and wingtips in codemp's operation order.
    /// World-space addition/subtraction is retained for the reference's rounding.
    pub fn at(self, origin: [f32; 3], orientation: [f32; 3]) -> ([f32; 3], [f32; 3]) {
        if !self.dynamic() {
            return (
                [-self.width / 2.0, -self.width / 2.0, -24.0],
                [self.width / 2.0, self.width / 2.0, self.height - 24.0],
            );
        }
        let axis = flight::angles_to_axis(orientation);
        let along = |from: [f32; 3], scale: f32, direction: [f32; 3]| {
            std::array::from_fn(|at| from[at] + direction[at] * scale)
        };
        let nose = along(origin, self.length / 2.0, axis[0]);
        let tail = along(origin, -self.length / 2.0, axis[0]);
        let left = along(origin, self.width / 2.0, axis[1]);
        let right = along(origin, -self.width / 2.0, axis[1]);
        let mut points = [[0.0; 3]; 8];
        for (index, side) in [nose, tail, left, right].into_iter().enumerate() {
            let top = along(side, self.height / 2.0, axis[2]);
            let (first, second) = if index < 2 {
                (index, index + 2)
            } else {
                (index + 2, index + 4)
            };
            points[first] = top;
            points[second] = along(top, -self.height, axis[2]);
        }
        let (mut low, mut high) = (origin, origin);
        for at in 0..3 {
            for point in &points {
                if point[at] > high[at] {
                    high[at] = point[at];
                } else if point[at] < low[at] {
                    low[at] = point[at];
                }
            }
        }
        (
            std::array::from_fn(|at| low[at] - origin[at]),
            std::array::from_fn(|at| high[at] - origin[at]),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    include!("vehicle_bounds_fixtures.rs");

    #[test]
    fn vehicle_bounds_match_openjk_emitted_fixtures() {
        let origins = [
            [0.0; 3],
            [8192.5, -4096.25, 123.75],
            [1_048_576.0, -524_288.0, 262_144.0],
        ];
        for &(kind, step, origin, minimums, maximums) in FIXTURES {
            let minimums = minimums.map(f32::from_bits);
            let maximums = maximums.map(f32::from_bits);
            let info = VehicleInfo {
                kind,
                length: 120.0,
                width: 40.0,
                height: 20.0,
                ..VehicleInfo::default()
            };
            let (low, high) = VehicleBounds::from_info(&info)
                .unwrap()
                .at(origins[origin], [25.0, (57 + step) as f32, -31.0]);
            for i in 0..3 {
                assert!(
                    (low[i] - minimums[i]).abs() < 0.0001,
                    "kind {kind}, {step} ms, origin {origin}, minimum {i}: {} != {}",
                    low[i],
                    minimums[i]
                );
                assert!(
                    (high[i] - maximums[i]).abs() < 0.0001,
                    "kind {kind}, {step} ms, origin {origin}, maximum {i}: {} != {}",
                    high[i],
                    maximums[i]
                );
            }
        }
    }

    #[test]
    fn missing_dimensions_leave_the_linked_box_alone() {
        for [length, width, height] in [[0.0, 40.0, 20.0], [120.0, 0.0, 20.0], [120.0, 40.0, 0.0]] {
            assert!(
                VehicleBounds::from_info(&VehicleInfo {
                    length,
                    width,
                    height,
                    ..VehicleInfo::default()
                })
                .is_none()
            );
        }
    }
}
