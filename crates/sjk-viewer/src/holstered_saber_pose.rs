//! Cached hip bones/back tag, evaluated only while a holster was requested.
use crate::{PlayerPreview, bolt::BoltMatrix, saber::Attachment};
use glam::{EulerRot, Mat3, Quat, Vec3};
use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_IDENTITY: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy)]
enum Bolt {
    Bone(usize),
    Surface { index: usize, bone: Option<usize> },
}

impl Bolt {
    fn resolve(preview: &PlayerPreview, name: &str) -> Option<Self> {
        if let Some(index) = preview
            .animation
            .bones
            .iter()
            .position(|b| b.name.eq_ignore_ascii_case(name))
        {
            return Some(Self::Bone(index));
        }
        Some(Self::Surface {
            index: preview
                .mesh
                .hierarchy
                .iter()
                .position(|s| s.name.eq_ignore_ascii_case(name))?,
            bone: preview.mesh.surface_bolt_bone(name, 0),
        })
    }
    fn matrix(self, preview: &PlayerPreview, matrices: &[[[f32; 4]; 3]]) -> Option<BoltMatrix> {
        let (bolt, bone) = match self {
            Self::Surface { index, bone } => (
                preview
                    .mesh
                    .surface_bolt_matrix_at(index, 0, matrices)
                    .ok()??,
                bone,
            ),
            Self::Bone(index) => {
                let matrix = matrices.get(index)?;
                let bind = preview.animation.bones.get(index)?.base_pose;
                (
                    std::array::from_fn(|row| {
                        std::array::from_fn(|col| {
                            (0..3)
                                .map(|axis| matrix[row][axis] * bind[axis][col])
                                .sum::<f32>()
                                + if col == 3 { matrix[row][3] } else { 0.0 }
                        })
                    }),
                    Some(index),
                )
            }
        };
        Some(match (&preview.rig, bone) {
            (Some(rig), Some(bone)) => {
                rig.retarget_bolt(bone, matrices, Some(&preview.rig_pose), bolt)
            }
            _ => bolt,
        })
    }
}

/// Actor-owned attachments, dropped together with the model. No second hilt models.
pub(crate) struct Worn {
    bolts: [Option<Bolt>; 3],
    cached: Cell<[Option<BoltMatrix>; 3]>,
    wanted: Cell<bool>,
    identity: Cell<u64>,
    pub(crate) swap: super::staff::Swap,
}

impl Worn {
    pub(crate) fn new(preview: &PlayerPreview) -> Self {
        Self {
            bolts: [
                Bolt::resolve(preview, "rfemurX").or_else(|| Bolt::resolve(preview, "*hip_r")),
                Bolt::resolve(preview, "lfemurX").or_else(|| Bolt::resolve(preview, "*hip_l")),
                Bolt::resolve(preview, "*back"),
            ],
            cached: Cell::new([None; 3]),
            wanted: Cell::new(false),
            identity: Cell::new(NEXT_IDENTITY.fetch_add(1, Ordering::Relaxed)),
            swap: Default::default(),
        }
    }
    pub(crate) fn update(&mut self, preview: &PlayerPreview, matrices: &[[[f32; 4]; 3]]) {
        if self.wanted.get() {
            self.cached.set(
                self.bolts
                    .map(|b| b.and_then(|b| b.matrix(preview, matrices))),
            );
        }
    }
    /// On the first visible frame use the already evaluated pose, never a bind pose.
    pub(crate) fn attachments(
        &self,
        mesh: &crate::ActorMesh,
        back: bool,
        geometry: Geometry,
    ) -> [Option<Attachment>; 2] {
        let cached = if self.wanted.replace(true) {
            self.cached.get()
        } else {
            let current = self
                .bolts
                .map(|b| b.and_then(|b| b.matrix(&mesh.preview, mesh.animator.matrices())));
            self.cached.set(current);
            current
        };
        placements(cached, back, geometry)
    }
    /// Whether the model has the `*back` tag a staff is stowed on.
    pub(crate) fn has_back_tag(&self) -> bool {
        self.bolts[2].is_some()
    }
    pub(crate) fn hide(&self) {
        self.wanted.set(false);
    }
    pub(crate) fn identity(&self) -> u64 {
        self.identity.get()
    }
    pub(crate) fn invalidate(&self) {
        self.identity
            .set(NEXT_IDENTITY.fetch_add(1, Ordering::Relaxed));
        self.swap.reset();
        self.hide();
    }
}

fn placements(
    cached: [Option<BoltMatrix>; 3],
    back: bool,
    geometry: Geometry,
) -> [Option<Attachment>; 2] {
    let on_back = back && cached[2].is_some();
    let bolts = [
        if on_back { cached[2] } else { cached[0] },
        if on_back { None } else { cached[1] },
    ];
    std::array::from_fn(|hand| {
        placement(
            crate::bolt::game_facing(bolts[hand]?),
            hand,
            on_back,
            geometry,
        )
    })
}

#[derive(Clone, Copy)]
pub(crate) struct Geometry {
    pub(crate) hip_pos: [f32; 3],
    pub(crate) hip_angles: [[f32; 3]; 2],
    pub(crate) back_pos: [f32; 3],
    pub(crate) back_angles: [f32; 3],
}
impl Default for Geometry {
    fn default() -> Self {
        Self {
            hip_pos: [0.0, 3.5, 0.0],
            hip_angles: [[0.0, 0.0, 270.0]; 2],
            back_pos: [0.0; 3],
            back_angles: [0.0, 0.0, 150.0],
        }
    }
}
impl Geometry {
    pub(crate) fn read(console: Option<&crate::console::ViewerConsole>) -> Self {
        let default = Self::default();
        let read = |name, fallback| {
            console
                .and_then(|c| c.text_cvar(name).ok())
                .and_then(vector)
                .unwrap_or(fallback)
        };
        Self {
            hip_pos: read("cg_holsteredsaberpos", default.hip_pos),
            hip_angles: [
                read("cg_holsteredsaberang1", default.hip_angles[0]),
                read("cg_holsteredsaberang2", default.hip_angles[1]),
            ],
            back_pos: read("cg_holsteredstaffpos", default.back_pos),
            back_angles: read("cg_holsteredstaffang", default.back_angles),
        }
    }
}
fn vector(value: &str) -> Option<[f32; 3]> {
    let mut words = value.split_whitespace();
    let result = [
        words.next()?.parse::<f32>().ok()?,
        words.next()?.parse().ok()?,
        words.next()?.parse().ok()?,
    ];
    (words.next().is_none() && result.iter().all(|x| x.is_finite() && x.abs() <= 3600.0))
        .then_some(result)
}

/// Scalar axis operations from EJK f21eee8 CG_DrawHolsteredSaber.
fn placement(bolt: BoltMatrix, hand: usize, back: bool, geometry: Geometry) -> Option<Attachment> {
    let [x, y, z, origin] = [0, 1, 2, 3].map(|c| Vec3::from_array(crate::bolt::column(&bolt, c)));
    let pos = if back {
        geometry.back_pos
    } else {
        geometry.hip_pos
    };
    let origin = if back {
        origin + x * pos[0] + y * pos[1] + z * pos[2]
    } else {
        let side = if hand == 0 { 1.0 } else { -1.0 };
        origin + side * (y * pos[0] + x * pos[1]) + z * pos[2]
    };
    let axes = if back {
        Mat3::from_cols(x, y, z)
    } else if hand == 0 {
        Mat3::from_cols(y.cross(-z), y, -z)
    } else {
        Mat3::from_cols(-z, y, x)
    };
    let angles = if back {
        geometry.back_angles
    } else {
        geometry.hip_angles[hand]
    }
    .map(f32::to_radians);
    let axes = axes
        * Mat3::from_quat(Quat::from_euler(
            EulerRot::ZYX,
            angles[1],
            angles[0],
            angles[2],
        ));
    crate::saber::attachment_from_bolt(std::array::from_fn(|row| {
        [
            axes.x_axis[row],
            axes.y_axis[row],
            axes.z_axis[row],
            origin[row],
        ]
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn positions_and_axes_match_reference_for_each_attachment() {
        let identity = [[1., 0., 0., 10.], [0., 1., 0., 20.], [0., 0., 1., 30.]];
        let g = Geometry::default();
        let primary = placement(identity, 0, false, g).unwrap();
        let second = placement(identity, 1, false, g).unwrap();
        let back = placement(identity, 0, true, g).unwrap();
        assert_eq!(primary.grip, [13.5, 20., 30.]);
        assert_eq!(second.grip, [6.5, 20., 30.]);
        assert_eq!(back.grip, [10., 20., 30.]);
        for (attachment, expected) in [
            (primary, [-Vec3::X, Vec3::Z, Vec3::Y]),
            (second, [-Vec3::Z, -Vec3::X, Vec3::Y]),
        ] {
            let rotation = Quat::from_array(attachment.rotation);
            for (axis, expected) in [Vec3::X, Vec3::Y, Vec3::Z].into_iter().zip(expected) {
                assert!((rotation * axis - expected).length() < 1e-5);
            }
        }
        for a in [primary, second, back] {
            assert!((Quat::from_array(a.rotation).length() - 1.).abs() < 1e-5);
        }
        let invalid = [[0.; 4]; 3];
        assert!(placement(invalid, 0, false, g).is_none());
    }
    #[test]
    fn geometry_values_are_finite_complete_and_bounded() {
        assert_eq!(vector("0 3.5 0"), Some([0., 3.5, 0.]));
        for bad in ["0 1", "0 1 2 3", "NaN 0 0", "inf 0 0", "99999 0 0"] {
            assert!(vector(bad).is_none());
        }
    }

    #[test]
    fn missing_bolts_are_independent_and_staff_back_falls_back_to_hip() {
        let bolt = Some([[1., 0., 0., 10.], [0., 1., 0., 20.], [0., 0., 1., 30.]]);
        let g = Geometry::default();
        assert!(placements([None, bolt, None], false, g)[1].is_some());
        assert!(placements([bolt, None, None], false, g)[0].is_some());
        assert!(placements([None; 3], true, g).iter().all(Option::is_none));
        let hip = placements([bolt, None, None], true, g)[0].unwrap();
        let ordinary = placements([bolt, None, None], false, g)[0].unwrap();
        assert_eq!(hip.grip, ordinary.grip);
        let back = placements([None, None, bolt], true, g);
        assert!(back[0].is_some() && back[1].is_none());
    }

    #[test]
    #[ignore = "release CPU measurement of 64 actors' attachment placement, no GPU or assets"]
    fn measure_holstered_attachment_cpu_cost() {
        use std::{hint::black_box, time::Instant};
        let bolt = Some([[1., 0., 0., 10.], [0., 1., 0., 20.], [0., 0., 1., 30.]]);
        let start = Instant::now();
        for frame in 0..20_000 {
            for actor in 0..64 {
                black_box(placements(
                    black_box([bolt; 3]),
                    black_box((frame + actor) % 3 == 0),
                    black_box(Geometry::default()),
                ));
            }
        }
        eprintln!(
            "holsters: 64 actors, 20000 frames, {:?}/frame (placement only)",
            start.elapsed() / 20_000
        );
    }
}
