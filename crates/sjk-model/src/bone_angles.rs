//! Engine-generic skeletal angle overrides.
//!
//! An adapter supplies Euler angles, signed axis mappings, and composition
//! mode. The model layer precomputes the base-pose-conjugated override once per
//! update and applies it while walking the hierarchy; it contains no game bone
//! names or policy constants.

use super::{GlaBone, multiply_3x4, quake_angles_matrix};

/// Signed skeleton axis selected for one Euler rotation component.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoneAxis {
    NegativeX,
    PositiveX,
    NegativeY,
    PositiveY,
    NegativeZ,
    PositiveZ,
}

/// Point at which an angle override composes with the animated local matrix.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoneAngleMode {
    /// Compose the base-pose-conjugated command after the animated local.
    PostMultiply,
}

/// One adapter-authored bone-angle command.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoneAngleCommand {
    pub angles_degrees: [f32; 3],
    pub mode: BoneAngleMode,
    pub up: BoneAxis,
    pub left: BoneAxis,
    pub forward: BoneAxis,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct BoneAngleOverride {
    pub(super) matrix: [[f32; 4]; 3],
}

pub(super) fn compile(bone: &GlaBone, command: BoneAngleCommand) -> BoneAngleOverride {
    let mapped = map_angles(
        command.angles_degrees,
        command.up,
        command.left,
        command.forward,
    );
    let rotation = quake_angles_matrix(mapped);
    let matrix = multiply_3x4(
        bone.base_pose,
        multiply_3x4(rotation, bone.inverse_base_pose),
    );
    let BoneAngleMode::PostMultiply = command.mode;
    BoneAngleOverride { matrix }
}

/// The model-space matrix of `bone` turned by `G2API_SetBoneAngles` with
/// `BONE_ANGLES_REPLACE` on a skeleton otherwise at its bind pose: the bone keeps its
/// base-pose origin and faces, in model space, the axes the angles give, mapped through
/// `up`, `left` and `forward` (`G2_Generate_Matrix`'s permutation, `G2_bones.cpp:333-422`,
/// and `G2_TransformBone`'s replace step, `tr_ghoul2.cpp:1702-1880`). Vertices bound to
/// the bone or its unanimated descendants are carried by this matrix.
pub fn replace_bone_matrix(
    bone: &GlaBone,
    angles_degrees: [f32; 3],
    up: BoneAxis,
    left: BoneAxis,
    forward: BoneAxis,
) -> [[f32; 4]; 3] {
    let mut angles = angles_degrees;
    // "why I should need do this ... alone knows. But I do." (`G2_bones.cpp:337`)
    if left == BoneAxis::PositiveY {
        angles[0] += 180.0;
    }
    let axes = quake_angles_matrix(angles);
    // Column 0 takes `forward`, column 1 `left`, column 2 `up` of the permutation.
    let mut permutation = [[0.0; 4]; 3];
    for (column, axis) in [forward, left, up].into_iter().enumerate() {
        let (row, sign) = match axis {
            BoneAxis::NegativeX => (0, -1.0),
            BoneAxis::PositiveX => (0, 1.0),
            BoneAxis::NegativeY => (1, -1.0),
            BoneAxis::PositiveY => (1, 1.0),
            BoneAxis::NegativeZ => (2, -1.0),
            BoneAxis::PositiveZ => (2, 1.0),
        };
        permutation[row][column] = sign;
    }
    let mut replaced = multiply_3x4(axes, permutation);
    for (row, base) in replaced.iter_mut().zip(bone.base_pose) {
        row[3] = base[3];
    }
    multiply_3x4(replaced, bone.inverse_base_pose)
}

fn map_angles(angles: [f32; 3], up: BoneAxis, left: BoneAxis, forward: BoneAxis) -> [f32; 3] {
    let yaw = match up {
        BoneAxis::NegativeX => angles[2] + 180.0,
        BoneAxis::PositiveX => angles[2],
        BoneAxis::NegativeY | BoneAxis::PositiveY => angles[0],
        BoneAxis::NegativeZ => angles[1] + 180.0,
        BoneAxis::PositiveZ => angles[1],
    };
    let pitch = match left {
        BoneAxis::NegativeX => angles[2],
        BoneAxis::PositiveX => angles[2] + 180.0,
        BoneAxis::NegativeY => angles[0],
        BoneAxis::PositiveY => angles[0] + 180.0,
        BoneAxis::NegativeZ | BoneAxis::PositiveZ => angles[1],
    };
    let roll = match forward {
        BoneAxis::NegativeX | BoneAxis::PositiveX => angles[2],
        BoneAxis::NegativeY => angles[0],
        BoneAxis::PositiveY => angles[0] + 180.0,
        BoneAxis::NegativeZ => angles[1],
        BoneAxis::PositiveZ => angles[1] + 180.0,
    };
    [pitch, yaw, roll]
}

#[cfg(test)]
mod replace_tests {
    use super::*;

    /// A bone like the emplaced gun's `Bone02`: its local x up, y along model +Y, z along
    /// model -X, its origin above the base.
    fn swivel_bone() -> GlaBone {
        let pivot = [0.0, 1.2, 46.6];
        let base_pose = [
            [0.0, 0.0, -1.0, pivot[0]],
            [0.0, 1.0, 0.0, pivot[1]],
            [1.0, 0.0, 0.0, pivot[2]],
        ];
        let mut inverse_base_pose = [[0.0; 4]; 3];
        for row in 0..3 {
            for column in 0..3 {
                inverse_base_pose[row][column] = base_pose[column][row];
            }
            inverse_base_pose[row][3] = -(0..3)
                .map(|inner| base_pose[inner][row] * pivot[inner])
                .sum::<f32>();
        }
        GlaBone {
            name: "Bone02".to_owned(),
            flags: 0,
            parent: None,
            base_pose,
            inverse_base_pose,
            children: Vec::new(),
        }
    }

    fn turn(matrix: [[f32; 4]; 3], direction: [f32; 3]) -> [f32; 3] {
        std::array::from_fn(|row| {
            (0..3)
                .map(|inner| matrix[row][inner] * direction[inner])
                .sum()
        })
    }

    fn close(left: [f32; 3], right: [f32; 3]) -> bool {
        left.iter()
            .zip(right)
            .all(|(left, right)| (left - right).abs() < 1e-4)
    }

    /// `cg_ents.c`'s emplaced gun call: up `NEGATIVE_Y`, left `NEGATIVE_X`, forward
    /// `POSITIVE_Z`. Its barrels, along model -Y at the bind pose, point where the
    /// angles look, and the bone keeps its origin.
    #[test]
    fn the_emplaced_gun_mapping_points_the_barrels_along_the_angles() {
        let bone = swivel_bone();
        let matrix = |angles| {
            replace_bone_matrix(
                &bone,
                angles,
                BoneAxis::NegativeY,
                BoneAxis::NegativeX,
                BoneAxis::PositiveZ,
            )
        };
        let barrel = [0.0, -1.0, 0.0];
        assert!(close(
            turn(matrix([0.0, 0.0, 0.0]), barrel),
            [1.0, 0.0, 0.0]
        ));
        assert!(close(
            turn(matrix([0.0, 90.0, 0.0]), barrel),
            [0.0, 1.0, 0.0]
        ));
        let (sine, cosine) = 30_f32.to_radians().sin_cos();
        assert!(close(
            turn(matrix([30.0, 0.0, 0.0]), barrel),
            [cosine, 0.0, -sine]
        ));
        // The top stays upright, and the pivot does not move.
        assert!(close(
            turn(matrix([0.0, 45.0, 0.0]), [0.0, 0.0, 1.0]),
            [0.0, 0.0, 1.0]
        ));
        let moved = matrix([20.0, 70.0, 0.0]);
        let pivot = [0.0, 1.2, 46.6];
        let placed: [f32; 3] = std::array::from_fn(|row| turn(moved, pivot)[row] + moved[row][3]);
        assert!(close(placed, pivot));
    }
}
