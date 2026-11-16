//! A vehicle's gunfire: `EV_VEH_FIRE` (`cg_event.c:2594-2602`) and
//! `CG_VehMuzzleFireFX` (`cg_event.c:1391-1432`).
//!
//! The game names every muzzle that fired in `trickedentindex` of the volley's last
//! projectile (or of a temp entity), and the vehicle in `owner`. The client plays, at each
//! of those muzzles, the `muzzleFX` of the weapon on it (`.vwp`): the flash, and through the
//! effect's `Sound` primitive the gun's firing sound, at full volume where the muzzle is
//! (`S_StartSound` on `ENTITYNUM_NONE`). Vehicle guns have no other firing sound: the
//! ordinary weapon events of a vehicle play nothing.
//!
//! Stock bolts the effect to the muzzle (`FX_PlayBoltedEffectID`); here it starts at the
//! muzzle where the vehicle is posed in the frame that observes the event. A vehicle that
//! is not posed (not drawn) plays it where the event entity is, along the projectile.

use crate::effect_runtime::{EffectLibrary, Particle};
use glam::Vec3;
use sjk_game_jka::vehicle_fields::VEHICLE_MUZZLES;
use sjk_protocol::EntityState;
use std::time::{Duration, Instant};

/// `EV_VEH_FIRE`.
pub(crate) const EV_VEH_FIRE: u16 = 24;

/// Wire field `s.trickedentindex`: the muzzles that fired, one bit each.
const ES_TRICKED_ENTITY: usize = 58;

/// Volleys kept for the next drawn frame; vehicle maps fire far fewer per frame.
const CAPACITY: usize = 32;

/// `EVENT_VALID_MSEC`: a volley not drawn by then is dropped rather than played late.
const VALID: Duration = Duration::from_millis(300);

/// `*muzzleN`, else `*flashN` (`cg_players.c:9423-9433`).
const MUZZLE_BOLTS: [&str; VEHICLE_MUZZLES] = [
    "*muzzle1",
    "*muzzle2",
    "*muzzle3",
    "*muzzle4",
    "*muzzle5",
    "*muzzle6",
    "*muzzle7",
    "*muzzle8",
    "*muzzle9",
    "*muzzle10",
    "*muzzle11",
    "*muzzle12",
];
const FLASH_BOLTS: [&str; VEHICLE_MUZZLES] = [
    "*flash1", "*flash2", "*flash3", "*flash4", "*flash5", "*flash6", "*flash7", "*flash8",
    "*flash9", "*flash10", "*flash11", "*flash12",
];

/// A muzzle in the vehicle's model space this frame: its origin and the effect's forward
/// (`GetOriginAxisFromBolt`'s `axis[0]`, the raw bolt matrix's first column).
pub(crate) type Muzzle = [[f32; 3]; 2];

/// One observed volley.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Fire {
    /// The vehicle's entity number (`es->owner`).
    pub(crate) vehicle: u16,
    /// The vehicle's `modelindex`, a `CS_MODELS` slot naming it (`$name`).
    pub(crate) model_index: usize,
    /// The muzzles that fired, one bit each.
    pub(crate) muzzles: u32,
    /// Where the event entity was, and the way its projectile flies: the fallback.
    pub(crate) origin: [f32; 3],
    pub(crate) direction: [f32; 3],
    pub(crate) at: Instant,
}

/// Volleys observed in snapshots and not yet played.
pub(crate) struct Pending {
    fires: Vec<Fire>,
}

impl Default for Pending {
    fn default() -> Self {
        Self {
            fires: Vec::with_capacity(CAPACITY),
        }
    }
}

impl Pending {
    /// Queue the volley `entity`'s new `EV_VEH_FIRE` names, if its vehicle is among the
    /// snapshot's `entities` (stock reads the vehicle's `centity_t`, which needs its model).
    pub(crate) fn observe(
        &mut self,
        entity: &EntityState,
        origin: [f32; 3],
        entities: &[EntityState],
        now: Instant,
    ) {
        let muzzles = entity.raw_field(ES_TRICKED_ENTITY).unwrap_or(0);
        let vehicle = entity.owner();
        if muzzles == 0 {
            return;
        }
        let Some(owner) = entities.iter().find(|state| state.number() == vehicle) else {
            return;
        };
        let Ok(model_index) = usize::try_from(owner.model_index()) else {
            return;
        };
        if self.fires.len() == CAPACITY {
            self.fires.remove(0);
        }
        self.fires.push(Fire {
            vehicle,
            model_index,
            muzzles,
            origin,
            direction: entity.trajectory_delta(),
            at: now,
        });
    }

    /// Take the oldest queued volley of `vehicle`.
    pub(crate) fn take(&mut self, vehicle: u16) -> Option<Fire> {
        let index = self.fires.iter().position(|fire| fire.vehicle == vehicle)?;
        Some(self.fires.remove(index))
    }

    /// Take the oldest volley still queued, dropping those too old to play.
    pub(crate) fn take_any(&mut self, now: Instant) -> Option<Fire> {
        self.fires
            .retain(|fire| now.saturating_duration_since(fire.at) <= VALID);
        (!self.fires.is_empty()).then(|| self.fires.remove(0))
    }
}

/// The muzzles of a vehicle posed as `matrices` (`None` where it has no such bolt);
/// all `None` for anything that is not a vehicle.
pub(crate) fn muzzles(
    preview: &crate::PlayerPreview,
    matrices: &[[[f32; 4]; 3]],
    output: &mut [Option<Muzzle>; VEHICLE_MUZZLES],
) {
    if preview.vehicle.is_none() {
        *output = [None; VEHICLE_MUZZLES];
        return;
    }
    for (index, muzzle) in output.iter_mut().enumerate() {
        let bolt = |name| {
            preview
                .mesh
                .surface_bolt_matrix(name, 0, matrices)
                .ok()
                .flatten()
        };
        *muzzle = bolt(MUZZLE_BOLTS[index])
            .or_else(|| bolt(FLASH_BOLTS[index]))
            .map(|matrix| {
                [
                    crate::bolt::column(&matrix, 3),
                    crate::bolt::column(&matrix, 0),
                ]
            });
    }
}

/// Where to play one muzzle's flash and which way, in the world.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Socket {
    pub(crate) origin: Vec3,
    pub(crate) forward: Vec3,
}

/// `muzzle` of a vehicle drawn with `transform`.
pub(crate) fn world_socket(muzzle: Muzzle, transform: &sjk_runtime::Transform) -> Socket {
    let rotation = crate::weapon_view::actor_world_rotation(transform.rotation);
    let offset = Vec3::from_array(muzzle[0]) * Vec3::from_array(transform.scale);
    Socket {
        origin: Vec3::from_array(transform.translation) + rotation * offset,
        forward: (rotation * Vec3::from_array(muzzle[1])).normalize_or(Vec3::X),
    }
}

/// Everything playing an effect needs.
pub(crate) struct Sinks<'a> {
    pub(crate) particles: &'a mut Vec<Particle>,
    pub(crate) auxiliary: &'a mut crate::effect_aux::Runtime,
    pub(crate) effects: &'a mut EffectLibrary,
    pub(crate) vfs: &'a sjk_vfs::VirtualFileSystem,
    pub(crate) audio: &'a mut Option<crate::GameAudio>,
    pub(crate) missiles: &'a sjk_client::LegacyMissileEffects,
}

/// Play `fire`'s muzzle flashes: each fired muzzle's at `socket(muzzle)`, or at the
/// event's place without one.
pub(crate) fn play(
    sinks: &mut Sinks<'_>,
    fire: &Fire,
    socket: impl Fn(usize) -> Option<Socket>,
    now: Instant,
) {
    let fallback = Socket {
        origin: Vec3::from_array(fire.origin),
        forward: Vec3::from_array(fire.direction).normalize_or(Vec3::Z),
    };
    for muzzle in fired(fire.muzzles) {
        let Some(effect) = sinks
            .missiles
            .vehicle_muzzle_effect(fire.model_index, muzzle)
        else {
            continue;
        };
        let place = socket(muzzle).unwrap_or(fallback);
        crate::effect_runtime::spawn_effect(
            sinks.particles,
            sinks.auxiliary,
            sinks.effects,
            sinks.vfs,
            effect,
            place.origin,
            now,
            u32::from(fire.vehicle) | (muzzle as u32) << 12 | 0x5000_0000,
            0,
            sinks.audio,
            crate::combat_effects::rotation_from_direction(place.forward.to_array()),
        );
    }
}

/// The muzzles set in `bits`, in stock's order.
fn fired(bits: u32) -> impl Iterator<Item = usize> {
    (0..VEHICLE_MUZZLES).filter(move |muzzle| bits & (1 << muzzle) != 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjk_protocol::LEGACY_ENTITY_FIELDS;

    fn event(muzzles: u32, owner: u16) -> EntityState {
        let mut state = EntityState::zero(300, &LEGACY_ENTITY_FIELDS);
        state.set_raw_field(ES_TRICKED_ENTITY, muzzles);
        state.set_raw_field(40, u32::from(owner));
        state
    }

    fn snapshot_with_vehicle(number: u16, model_index: u32) -> Vec<EntityState> {
        let mut vehicle = EntityState::zero(number, &LEGACY_ENTITY_FIELDS);
        vehicle.set_raw_field(MODEL_INDEX, model_index);
        vec![vehicle]
    }

    /// Wire field `s.modelindex`.
    const MODEL_INDEX: usize = 46;

    #[test]
    fn a_volley_names_its_vehicle_model_and_muzzles() {
        let now = Instant::now();
        let mut pending = Pending::default();
        let snapshot = snapshot_with_vehicle(70, 12);
        pending.observe(&event(0b101, 70), [1.0, 2.0, 3.0], &snapshot, now);
        // No muzzle, or a vehicle not in the snapshot: nothing to play.
        pending.observe(&event(0, 70), [0.0; 3], &snapshot, now);
        pending.observe(&event(1, 71), [0.0; 3], &snapshot, now);
        let fire = pending.take(70).expect("queued");
        assert_eq!((fire.vehicle, fire.model_index), (70, 12));
        assert_eq!(fired(fire.muzzles).collect::<Vec<_>>(), [0, 2]);
        assert_eq!(fire.origin, [1.0, 2.0, 3.0]);
        assert!(pending.take_any(now).is_none());
    }

    #[test]
    fn an_undrawn_volley_is_dropped_once_too_old() {
        let now = Instant::now();
        let mut pending = Pending::default();
        let snapshot = snapshot_with_vehicle(70, 12);
        pending.observe(&event(1, 70), [0.0; 3], &snapshot, now);
        pending.observe(&event(2, 70), [0.0; 3], &snapshot, now + VALID);
        let late = now + VALID + Duration::from_millis(1);
        assert_eq!(pending.take_any(late).map(|fire| fire.muzzles), Some(2));
        assert!(pending.take_any(late).is_none());
        for _ in 0..CAPACITY + 5 {
            pending.observe(&event(1, 70), [0.0; 3], &snapshot, now);
        }
        assert_eq!(pending.fires.len(), CAPACITY);
    }

    #[test]
    fn a_muzzle_turns_and_moves_with_its_vehicle() {
        let transform = sjk_runtime::Transform {
            translation: [100.0, 0.0, 0.0],
            rotation: sjk_client::legacy_angles_to_quaternion([0.0, 90.0, 0.0]),
            scale: [2.0; 3],
        };
        let socket = world_socket([[10.0, 0.0, 0.0], [1.0, 0.0, 0.0]], &transform);
        let model_x = crate::weapon_view::actor_world_rotation(transform.rotation) * Vec3::X;
        assert!((socket.origin - (Vec3::new(100.0, 0.0, 0.0) + model_x * 20.0)).length() < 1e-3);
        assert!((socket.forward - model_x).length() < 1e-5);
    }
}
