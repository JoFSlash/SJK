//! CG_ForcePushBlur/CG_ForceGripEffect plus CG_AddPuff's RGB fade and growth.
use super::*;

/// The stock hand puffs and EJK #33's red victim puffs share motion and lifetime.
#[derive(Clone, Copy)]
pub(crate) enum Kind {
    Push,
    Grip,
    Victim,
}

/// Emit a stock opposing-velocity puff pair without growing the particle pool.
pub(crate) fn pair(
    particles: &mut Vec<Particle>,
    effects: &mut EffectLibrary,
    origin: Vec3,
    view_left: Vec3,
    grip: bool,
    time: i32,
    now: Instant,
) {
    emit(
        particles,
        effects,
        origin,
        view_left,
        if grip { Kind::Grip } else { Kind::Push },
        time,
        now,
    );
}

/// Emit a named puff style through the existing bounded particle pool.
pub(crate) fn emit(
    particles: &mut Vec<Particle>,
    effects: &mut EffectLibrary,
    origin: Vec3,
    view_left: Vec3,
    kind: Kind,
    time: i32,
    now: Instant,
) {
    let grip = matches!(kind, Kind::Grip);
    let red = (200.0 + ((time as f32 * 0.004).sin() * 0.08 + 0.1) * 255.0).min(255.0);
    for side in 0..2 {
        // Puffs are effect particles: they leave the billboard reserve alone.
        if !crate::particle_room::effect_fits(particles.len()) {
            break;
        }
        let color = if matches!(kind, Kind::Victim) {
            [60.0, 8.0, 8.0]
        } else if !grip {
            [24.0, 32.0, 40.0]
        } else if side == 0 {
            [red, 0.0, 0.0]
        } else {
            [255.0; 3]
        };
        let shader = if grip && side == 1 {
            "gfx/effects/sabers/red_glow"
        } else {
            "gfx/effects/forcePush"
        };
        let linear = sjk_effect::CurveFlags {
            linear: true,
            ..Default::default()
        };
        let envelope =
            |start, end| crate::effect_envelope::Envelope::from_values(start, end, 0.0, linear);
        particles.push(Particle {
            motion: crate::particle_motion::Motion::new(
                origin,
                view_left * if side == 0 { 55.0 } else { -55.0 },
                Vec3::ZERO,
                side as f32 * 180.0,
                0.0,
                now,
            ),

            spawned_at: now,
            delay: Duration::ZERO,
            lifetime: Duration::from_millis(120),
            size: envelope(8.0, 10.0),
            length: None,
            streak: None,
            trace_streak: false,
            normal: None,
            alpha: envelope(1.0, 1.0),
            use_alpha: false,
            set_shader_time: false,
            rgb: color.map(|value| envelope(value / 255.0, 0.0)),
            seed: time as u32,
            shader: effects.shader(shader),
            physics: crate::particle_physics::State::new(
                effects.code_primitive_definition(),
                0,
                0,
                0.0,
                sjk_effect::PrimitiveFlags::default(),
            ),
            shape: crate::particle_types::PrimitiveShape::Billboard,
        });
    }
}
