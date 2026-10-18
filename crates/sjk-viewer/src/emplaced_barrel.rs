//! An emplaced gun's barrel turned where its gunner aims (`CG_General`,
//! `codemp/cgame/cg_ents.c:1314-1338`). Stock turns the gun's `Bone02` with
//! `G2API_SetBoneAngles`; rigid models here have no skeleton, so the part of
//! `turret_chair.glm` that `Bone02` (and the bones under it) carries is loaded as a model of
//! its own, [`BARREL`], and drawn turned round that bone.
use super::*;
use sjk_protocol::EntityState;

/// The emplaced gun's model (`SP_emplaced_gun`).
pub(super) const MODEL: &str = "models/map_objects/mp/turret_chair.glm";
/// Its turning part, as a model of its own: the gun's file with the bone it turns on.
pub(super) const BARREL: &str = "models/map_objects/mp/turret_chair.glm#Bone02";
/// The bone stock turns.
const BONE: &str = "Bone02";
/// `MAX_CLIENTS`: an emplaced gun is a game entity, numbered above the clients.
const MAX_CLIENTS: u16 = 32;
/// `NUM_FORCE_POWERS + 1`: `s.activeForcePass` while the gun is worked.
const WORKED: u32 = 19;
/// `entityState_t::activeForcePass` (netfield 68).
const ES_ACTIVE_FORCE_PASS: usize = 68;
/// `CLASS_VEHICLE`, whose entities this is not for.
const CLASS_VEHICLE: u8 = 53;

/// Which part of the gun's file a model is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Part {
    /// Any other model: all of it.
    Whole,
    /// The gun without its barrel.
    Base,
    /// The barrel alone.
    Barrel,
}

/// The file `model` is read from and which part of it is drawn.
pub(super) fn part(model: &str) -> (&str, Part) {
    if model.eq_ignore_ascii_case(BARREL) {
        (MODEL, Part::Barrel)
    } else if model.eq_ignore_ascii_case(MODEL) {
        (MODEL, Part::Base)
    } else {
        (model, Part::Whole)
    }
}

/// The triangles `Bone02` carries, per LOD-0 surface, and where the bone stands in the
/// model: `None` where the model's skeleton has no such bone.
pub(super) struct Split {
    barrel: Vec<Vec<bool>>,
    pub(super) pivot: [f32; 3],
}

impl Split {
    /// Read the model's skeleton (`<animation name>.gla`) and mark each vertex the bone or
    /// one under it carries most.
    pub(super) fn read(vfs: &VirtualFileSystem, model: &Glm) -> Option<Self> {
        let path = format!("{}.gla", model.animation_name.trim_end_matches(".gla"));
        let asset = vfs.read(&path).ok().flatten()?;
        let gla = Gla::parse(&asset.bytes).ok()?;
        let root = gla
            .bones
            .iter()
            .position(|bone| bone.name.eq_ignore_ascii_case(BONE))?;
        let mut carried = vec![false; gla.bones.len()];
        let mut open = vec![root];
        while let Some(bone) = open.pop() {
            if let Some(slot) = carried.get_mut(bone).filter(|slot| !**slot) {
                *slot = true;
                open.extend(gla.bones[bone].children.iter().copied());
            }
        }
        let lod = model.lods.first()?;
        let barrel = lod
            .surfaces
            .iter()
            .map(|surface| {
                surface
                    .vertices
                    .iter()
                    .map(|vertex| {
                        vertex
                            .weights
                            .iter()
                            .max_by(|a, b| a.weight.total_cmp(&b.weight))
                            .and_then(|weight| surface.bone_references.get(weight.bone_reference))
                            .is_some_and(|bone| carried.get(*bone).copied().unwrap_or(false))
                    })
                    .collect()
            })
            .collect();
        let pose = gla.bones[root].base_pose;
        Some(Self {
            barrel,
            pivot: [pose[0][3], pose[1][3], pose[2][3]],
        })
    }

    /// Whether triangle `triangle` of surface `surface` belongs to the barrel: most of
    /// its corners do.
    pub(super) fn in_barrel(&self, surface: usize, triangle: &[u32; 3]) -> bool {
        let Some(marks) = self.barrel.get(surface) else {
            return false;
        };
        triangle
            .iter()
            .filter(|corner| marks.get(**corner as usize).copied().unwrap_or(false))
            .count()
            >= 2
    }
}

/// The angles `CG_General` gives `Bone02` of gun `state`, or `None` when nobody works it:
/// the gunner's view (`view` for the local gunner, its entity's angles for anyone else),
/// the pitch held to 40 degrees down and the yaw made relative to the gun's own.
pub(super) fn angles(
    state: &EntityState,
    snapshot: &Snapshot,
    view: Option<[f32; 3]>,
) -> Option<[f32; 3]> {
    if state.number() < MAX_CLIENTS
        || state.raw_field(ES_ACTIVE_FORCE_PASS).unwrap_or(0) != WORKED
        || state.npc_class() == CLASS_VEHICLE
    {
        return None;
    }
    let owner = u16::try_from(state.emplaced_owner()).ok()?;
    let mut angles = if owner == snapshot.player.client_num() {
        view.unwrap_or_else(|| snapshot.player.view_angles())
    } else {
        snapshot
            .entities
            .iter()
            .find(|entity| entity.number() == owner)
            .map_or([0.0; 3], EntityState::angular_trajectory_base)
    };
    angles[0] = angles[0].min(40.0);
    angles[1] -= state.angles()[1];
    Some(angles)
}

/// Where the barrel of a gun placed at `gun` is drawn, turned by `angles` (pitch and yaw
/// in degrees, in the gun's own frame) round `pivot`, the bone's place in the model.
pub(super) fn placement(
    gun: sjk_runtime::Transform,
    pivot: [f32; 3],
    angles: [f32; 3],
) -> ([f32; 3], [f32; 4]) {
    let turn = Quat::from_rotation_z(angles[1].to_radians())
        * Quat::from_rotation_y(angles[0].to_radians());
    let rotation = Quat::from_array(gun.rotation);
    let pivot = Vec3::from_array(pivot);
    let scale = Vec3::from_array(gun.scale);
    let offset = rotation * (scale * (pivot - turn * pivot));
    (
        (Vec3::from_array(gun.translation) + offset).to_array(),
        (rotation * turn).to_array(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unturned_barrel_stays_with_the_gun() {
        let gun = sjk_runtime::Transform {
            translation: [10.0, 20.0, 30.0],
            rotation: Quat::from_rotation_z(1.0).to_array(),
            scale: [1.0; 3],
        };
        let (translation, rotation) = placement(gun, [4.0, 0.0, 40.0], [0.0; 3]);
        for axis in 0..3 {
            assert!((translation[axis] - gun.translation[axis]).abs() < 1e-4);
        }
        for axis in 0..4 {
            assert!((rotation[axis] - gun.rotation[axis]).abs() < 1e-6);
        }
    }

    #[test]
    fn a_turned_barrel_keeps_its_pivot_in_place() {
        let gun = sjk_runtime::Transform::IDENTITY;
        let pivot = [4.0, 2.0, 40.0];
        let (translation, rotation) = placement(gun, pivot, [30.0, 90.0, 0.0]);
        let moved =
            Vec3::from_array(translation) + Quat::from_array(rotation) * Vec3::from_array(pivot);
        assert!((moved - Vec3::from_array(pivot)).length() < 1e-4);
    }
}
