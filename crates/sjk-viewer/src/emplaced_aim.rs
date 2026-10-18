//! An emplaced gun's barrel turned where its gunner aims (`CG_General`,
//! `codemp/cgame/cg_ents.c:1314-1338`): while a player works the gun (`s.activeForcePass`
//! is `NUM_FORCE_POWERS + 1`), the gun's `Bone02` takes the gunner's view angles (the
//! local player's own view, any other gunner's angles as its entity carries them), the
//! pitch held to 40 degrees down and the yaw made relative to the gun's own.
use super::ActorMesh;
use sjk_protocol::{EntityState, Snapshot};

/// `MAX_CLIENTS`: an emplaced gun is a game entity, numbered above the clients.
const MAX_CLIENTS: u16 = 32;
/// `NUM_FORCE_POWERS + 1`: what `emplaced_gun_use` sets `s.activeForcePass` to while the
/// gun is worked.
const WORKED: u32 = 19;
/// `entityState_t::activeForcePass` (protocol 26 netfield 68).
const ES_ACTIVE_FORCE_PASS: usize = 68;
/// `CLASS_VEHICLE`, whose entities this is not for.
const CLASS_VEHICLE: u8 = 53;
/// `G2API_SetBoneAngles(..., NEGATIVE_Y, NEGATIVE_X, POSITIVE_Z, ...)` as
/// `set_server_bone_angles` reads an orientation: up, left and forward, three bits each
/// (`NEGATIVE_Y` 6, `NEGATIVE_X` 4, `POSITIVE_Z` 2).
const ORIENT: u32 = (6 << 6) | (4 << 3) | 2;

/// The angles `CG_General` gives `Bone02` of gun `state`, or `None` when nobody works it.
/// `view` is the local player's own view (`cg.refdef.viewangles`), used when the local
/// player is the gunner.
pub(crate) fn angles(
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
    let local = owner == snapshot.player.client_num();
    let mut angles = match view {
        Some(view) if local => view,
        _ if local => snapshot.player.view_angles(),
        _ => snapshot
            .entities
            .iter()
            .find(|entity| entity.number() == owner)
            .map_or([0.0; 3], EntityState::angular_trajectory_base),
    };
    angles[0] = angles[0].min(40.0);
    angles[1] -= state.angles()[1];
    Some(angles)
}

/// Turns `Bone02` of gun `state` on `mesh` as [`angles`] has it: nothing while nobody works
/// the gun, or on a skeleton without the bone.
pub(crate) fn apply(
    mesh: &mut ActorMesh,
    state: &EntityState,
    snapshot: &Snapshot,
    view: Option<[f32; 3]>,
) {
    let Some(angles) = angles(state, snapshot, view) else {
        return;
    };
    let Some(bone) = mesh
        .preview
        .animation
        .bones
        .iter()
        .position(|bone| bone.name.eq_ignore_ascii_case("Bone02"))
    else {
        return;
    };
    let _ = mesh
        .animator
        .set_server_bone_angles(&mesh.preview.animation, bone, angles, ORIENT);
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjk_protocol::{LEGACY_ENTITY_FIELDS, PlayerState};

    fn snapshot(entities: Vec<EntityState>) -> Snapshot {
        Snapshot {
            message_sequence: 0,
            reliable_acknowledge: 0,
            server_commands: Vec::new(),
            server_time: 1_000,
            delta_from: None,
            flags: 0,
            area_mask: Vec::new(),
            player: PlayerState::zero(),
            vehicle_player: None,
            entities,
            consumed_bits: 0,
        }
    }

    fn gun(owner: u32, yaw: f32) -> EntityState {
        let mut gun = EntityState::zero(118, &LEGACY_ENTITY_FIELDS);
        gun.set_raw_field(ES_ACTIVE_FORCE_PASS, WORKED);
        gun.set_raw_field(47, owner);
        gun.set_raw_field(9, yaw.to_bits());
        gun
    }

    #[test]
    fn the_local_gunner_aims_the_barrel_with_its_view() {
        let snapshot = snapshot(Vec::new());
        // Client 0 is the local player; its view is 60 down and 120 round, the gun faces 90.
        let aimed = angles(&gun(0, 90.0), &snapshot, Some([60.0, 120.0, 0.0]));
        assert_eq!(aimed, Some([40.0, 30.0, 0.0]));
    }

    #[test]
    fn an_unworked_gun_keeps_its_pose() {
        let mut idle = gun(0, 90.0);
        idle.set_raw_field(ES_ACTIVE_FORCE_PASS, 0);
        assert_eq!(angles(&idle, &snapshot(Vec::new()), Some([0.0; 3])), None);
    }
}
