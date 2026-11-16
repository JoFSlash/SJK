//! Live JoF Force animation FX, using the already evaluated actor pose.
use super::*;
use sjk_client::LegacyForceAnimFx;

pub(super) fn submit(
    sinks: &mut Sinks<'_>,
    mesh: usize,
    number: u16,
    transform: sjk_runtime::Transform,
    snapshot: &Snapshot,
    time: i32,
    now: Instant,
) {
    let effect = LegacyForceAnimFx::from_snapshot(snapshot, number, sinks.predicted_local_state);
    if !sinks.effect_aux.force_anim.due(number, effect, time) {
        return;
    }
    let Some(effect) = effect else {
        return;
    };
    if !sinks.force_fx.animation {
        return;
    }
    let actor = &sinks.actor_meshes[mesh];
    let origin = Vec3::from_array(transform.translation);
    let rotation = weapon_view::actor_world_rotation(transform.rotation);
    let scale = Vec3::from_array(transform.scale);
    if effect == LegacyForceAnimFx::QuickHeal {
        // EJK #58 leaves this uninitialized on a missing hand bolt. Use the actor
        // origin instead, never the world origin or a previous actor's hand.
        let hand = actor.weapon_attachments[1].map_or(origin, |bolt| {
            saber::world_attachment(origin, rotation, bolt.scaled(transform.scale)).0
        });
        let left = Vec3::new(-sinks.camera_yaw.sin(), sinks.camera_yaw.cos(), 0.0);
        sprites::pair(sinks.particles, sinks.effects, hand, left, false, time, now);
        return;
    }
    let (position, direction) = chest_frame(actor.force_bones.chest, origin, rotation, scale);
    let name = match effect {
        LegacyForceAnimFx::Rage => "force/rage2",
        LegacyForceAnimFx::Heal => "force/heal2",
        LegacyForceAnimFx::QuickHeal => unreachable!(),
    };
    if sinks.effects.contains_definition(name) {
        effect_runtime::spawn_effect(
            sinks.particles,
            sinks.effect_aux,
            sinks.effects,
            sinks.vfs,
            name,
            position,
            now,
            u32::from(number) ^ time as u32,
            0,
            sinks.game_audio,
            Quat::from_rotation_arc(Vec3::X, direction.normalize_or(Vec3::X)),
        );
    }
}

fn chest_frame(
    raw: Option<crate::bolt::BoltMatrix>,
    origin: Vec3,
    rotation: Quat,
    scale: Vec3,
) -> (Vec3, Vec3) {
    raw.map_or((origin, rotation * Vec3::X), |raw| {
        let bolt = crate::bolt::game_facing(raw);
        (
            origin + rotation * (Vec3::from_array(crate::bolt::column(&bolt, 3)) * scale),
            rotation * -Vec3::from_array(crate::bolt::column(&bolt, 1)),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chest_follows_scaled_pose_and_ghoul2_negative_y_axis() {
        let raw = [
            [1.0, 0.0, 0.0, 2.0],
            [0.0, 1.0, 0.0, 3.0],
            [0.0, 0.0, 1.0, 4.0],
        ];
        let origin = Vec3::new(10.0, 20.0, 30.0);
        let rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
        let (position, direction) = chest_frame(Some(raw), origin, rotation, Vec3::splat(2.0));
        assert!(position.abs_diff_eq(Vec3::new(4.0, 24.0, 38.0), 0.0001));
        assert!(direction.abs_diff_eq(-Vec3::Y, 0.0001));
        let (position, direction) = chest_frame(None, origin, rotation, Vec3::splat(2.0));
        assert_eq!(position, origin);
        assert!(direction.abs_diff_eq(Vec3::Y, 0.0001));
    }
}
