//! Grip victim puffs from EJK #33, with #90's viewed-player camera guard.
//!
//! Reference: multiplayer `codemp/cgame/cg_players.c`, revisions
//! f255db278192dc0440f0673cc3f68422c101c2f6 and
//! 1acbe8937f12a0b057c80eba068e9fe43887c239. This is presentation only:
//! the choking animation is a visual marker, not proof of server Grip authority.
use super::*;

/// Archived, default-on player preference, shared by both settings interfaces.
pub(crate) const CVAR: &str = "cg_gripVictimFX";
/// The canonical multiplayer animation ordinal (`animtable.h`).
const BOTH_CHOKE3: u16 = 1322;

/// The later reference uses the leg animation so torso transitions retain the puffs.
fn visible(enabled: bool, legs: u16, local: bool, third_person: bool, zoom: u8) -> bool {
    enabled && legs == BOTH_CHOKE3 && (!local || (third_person && zoom == 0))
}

/// Use local prediction for the viewed player, otherwise the decoded snapshot marker.
fn active(
    snapshot: &Snapshot,
    number: u16,
    predicted: Option<&sjk_client::pmove::MovementState>,
    enabled: bool,
    third_person: bool,
    detached_camera: bool,
) -> bool {
    let local = number == snapshot.player.client_num();
    let zoom = predicted.map_or(snapshot.player.zoom_mode(), |p| p.zoom_mode);
    let (legs, tricked) = if local {
        (
            predicted.map_or(snapshot.player.leg_animation(), |p| p.legs_anim),
            false,
        )
    } else {
        let Ok(index) = snapshot
            .entities
            .binary_search_by_key(&number, |e| e.number())
        else {
            return false;
        };
        let entity = &snapshot.entities[index];
        let powers = predicted.map_or(snapshot.player.force_powers_active(), |p| {
            p.force_powers_active
        });
        (
            entity.leg_animation(),
            sjk_client::legacy_mind_tricked(
                sjk_client::legacy_entity_trick_targets(entity),
                snapshot.player.client_num(),
                powers,
            ),
        )
    };
    !tricked && visible(enabled, legs, local && !detached_camera, third_person, zoom)
}

/// Prefer the posed neck, then the head minus eight world units; never guess a joint.
fn point(
    neck: Option<Vec3>,
    head: Option<Vec3>,
    transform: sjk_runtime::Transform,
) -> Option<Vec3> {
    let rotation = weapon_view::actor_world_rotation(transform.rotation);
    let world = |point: Vec3| {
        Vec3::from_array(transform.translation)
            + rotation * (point * Vec3::from_array(transform.scale))
    };
    let origin = neck
        .map(world)
        .or_else(|| head.map(|head| world(head) - Vec3::Z * 8.0))?;
    Some(origin + rotation * Vec3::X * 2.0)
}

/// Submit through the existing fixed cadence and bounded pool, without another pose.
pub(in super::super) fn submit(
    sinks: &mut Sinks<'_>,
    mesh: usize,
    entity: &sjk_runtime::SceneEntity,
    transform: sjk_runtime::Transform,
    snapshot: &Snapshot,
    time: i32,
    now: Instant,
) {
    if !active(
        snapshot,
        entity.id.get().saturating_sub(1) as u16,
        sinks.predicted_local_state,
        sinks.grip_victim,
        sinks.third_person,
        sinks.detached_camera,
    ) {
        return;
    }
    let bones = &sinks.actor_meshes[mesh].force_bones;
    let Some(origin) = point(bones.neck, bones.head, transform) else {
        return;
    };
    if !sinks.effect_aux.continuous.due(now) {
        return;
    }
    sprites::emit(
        sinks.particles,
        sinks.effects,
        origin,
        Vec3::new(-sinks.camera_yaw.sin(), sinks.camera_yaw.cos(), 0.0),
        sprites::Kind::Victim,
        time,
        now,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grip_victim_respects_camera_and_setting_without_hiding_remote_players() {
        assert_eq!(
            sjk_game_jka::legacy_animation_name(usize::from(BOTH_CHOKE3)),
            Some("BOTH_CHOKE3")
        );
        for zoom in [0, 1, 2] {
            for third in [false, true] {
                assert!(visible(true, BOTH_CHOKE3, false, third, zoom));
                assert_eq!(
                    visible(true, BOTH_CHOKE3, true, third, zoom),
                    third && zoom == 0
                );
                assert!(!visible(false, BOTH_CHOKE3, false, third, zoom));
                assert!(!visible(true, BOTH_CHOKE3 - 1, false, third, zoom));
            }
        }
    }

    #[test]
    fn grip_victim_reads_local_and_remote_markers_and_prediction() {
        use sjk_protocol::{EntityState, LEGACY_ENTITY_FIELDS, PlayerState};
        let mut player = PlayerState::zero();
        player.set_client_num(3);
        player.set_raw_field(13, u32::from(BOTH_CHOKE3));
        let mut entity = EntityState::zero(7, &LEGACY_ENTITY_FIELDS);
        entity.set_raw_field(16, u32::from(BOTH_CHOKE3));
        let mut snapshot = Snapshot {
            message_sequence: 1,
            reliable_acknowledge: 0,
            server_commands: Vec::new(),
            server_time: 1000,
            delta_from: None,
            flags: 0,
            area_mask: Vec::new(),
            player,
            vehicle_player: None,
            entities: vec![entity],
            consumed_bits: 0,
        };
        assert!(active(&snapshot, 3, None, true, true, false));
        assert!(!active(&snapshot, 3, None, true, false, false));
        assert!(active(&snapshot, 7, None, true, false, false));
        assert!(!active(&snapshot, 8, None, true, true, false));
        let mut predicted = sjk_client::pmove::MovementState::default();
        assert!(!active(&snapshot, 3, Some(&predicted), true, true, false));
        predicted.legs_anim = BOTH_CHOKE3;
        assert!(active(&snapshot, 3, Some(&predicted), true, true, false));
        predicted.zoom_mode = 1;
        assert!(!active(&snapshot, 3, Some(&predicted), true, true, false));
        assert!(active(&snapshot, 7, Some(&predicted), true, false, false));
        assert!(active(&snapshot, 3, Some(&predicted), true, false, true));
        snapshot.entities[0].set_raw_field(58, 1 << 3);
        assert!(!active(&snapshot, 7, None, true, false, false));
        snapshot.player.set_raw_field(82, 1 << 14);
        assert!(active(&snapshot, 7, None, true, false, false));
    }

    #[test]
    fn grip_victim_puffs_keep_reference_colour_life_and_density() {
        let start = Instant::now();
        for step in [8, 7, 4, 3] {
            let mut cadence = crate::effect_aux::cadence::Cadence::default();
            let mut particles = Vec::with_capacity(256);
            let mut effects = EffectLibrary::default();
            for time in (0..1000).step_by(step) {
                let now = start + Duration::from_millis(time as u64);
                if cadence.due(now) {
                    sprites::emit(
                        &mut particles,
                        &mut effects,
                        Vec3::ZERO,
                        Vec3::Y,
                        sprites::Kind::Victim,
                        time,
                        now,
                    );
                }
            }
            assert_eq!(
                particles.len(),
                250,
                "8 ms emission cadence at {step} ms steps"
            );
            for (side, particle) in particles.iter_mut().take(2).enumerate() {
                assert_eq!(particle.lifetime, Duration::from_millis(120));
                assert_eq!(&*particle.shader, "gfx/effects/forcepush");
                assert_eq!(particle.size.sample(0.0, 120.0, 0), 8.0);
                assert_eq!(particle.size.sample(120.0, 120.0, 0), 10.0);
                for (channel, expected) in [60.0, 8.0, 8.0].into_iter().enumerate() {
                    assert_eq!(
                        particle.rgb[channel].sample(0.0, 120.0, 0),
                        expected / 255.0
                    );
                    assert_eq!(particle.rgb[channel].sample(120.0, 120.0, 0), 0.0);
                }
                let sample = particle.motion.advance(start + Duration::from_millis(120));
                assert!((sample.origin.y - if side == 0 { 6.6 } else { -6.6 }).abs() < 0.00001);
            }
        }
    }

    #[test]
    fn grip_victim_origin_prefers_neck_and_handles_scaled_head_fallback() {
        let transform = sjk_runtime::Transform {
            translation: [10.0, 20.0, 30.0],
            scale: [2.0; 3],
            rotation: [0.0, 0.0, 0.0, 1.0],
        };
        let neck = Vec3::new(0.0, 0.0, 4.0);
        let head = Vec3::new(0.0, 0.0, 12.0);
        assert!(
            point(Some(neck), Some(head), transform)
                .unwrap()
                .distance(Vec3::new(10.0, 22.0, 38.0))
                < 0.00001
        );
        assert!(
            point(None, Some(head), transform)
                .unwrap()
                .distance(Vec3::new(10.0, 22.0, 46.0))
                < 0.00001
        );
        let turned = sjk_runtime::Transform {
            rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2).to_array(),
            ..transform
        };
        assert!(
            point(Some(neck), None, turned)
                .unwrap()
                .distance(Vec3::new(8.0, 20.0, 38.0))
                < 0.00001
        );
        assert!(point(None, None, transform).is_none());
    }
}
