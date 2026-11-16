//! The deployable force field (`HI_SHIELD`, the holdable a Merc places), as codemp draws
//! it (`CG_Special` → `FX_DrawPortableShield`, `codemp/cgame/cg_ents.c:458-557`).
//!
//! The game sends the field as an `ET_SPECIAL` entity with `modelindex == HI_SHIELD`; it
//! has no model, so nothing else draws it. `time2` packs its shape: bit 24 says it runs
//! along the x axis (else y), bits 16-23 its height, bits 8-15 its width on the positive
//! side of its origin and bits 0-7 on the negative side. `otherEntityNum2` is the owner's
//! team (red draws `gfx/misc/red_portashield`, everything else the blue one) and a
//! non-zero `trickedentindex` the "being damaged" look (`*_dmgshield`). `EF_NODRAW` hides
//! it while it is down.
//!
//! Every cgame frame stock adds an `LE_OLINE` local entity (`FX_AddOrientedLine`) that
//! lives 50 ms: an upright quad from one end of the field to the other, `height` tall,
//! its texture's s across the height and t along the length (`RB_SurfaceOrientedLine`,
//! `rd-vanilla/tr_surface.cpp`). The copies overlap, so the field, drawn additively,
//! looks as bright as at the reference 125 FPS when they are added on the fixed cgame
//! cadence ([`crate::effect_cadence`]), as other continuously played effects are.

use crate::effect_runtime::EffectLibrary;
use crate::particle_types::PrimitiveShape;
use crate::{Particle, particle_types};
use glam::Vec3;
use sjk_protocol::Snapshot;
use std::time::{Duration, Instant};

/// `ET_SPECIAL` (`bg_public.h`).
const ET_SPECIAL: u8 = 4;
/// `HI_SHIELD` (`bg_public.h`), the field's `modelindex`.
const HI_SHIELD: i16 = 2;
/// `EF_NODRAW` (`bg_public.h`).
const EF_NODRAW: u32 = 1 << 7;
/// `TEAM_RED` (`bg_public.h`).
const TEAM_RED: u16 = 1;
/// `entityState_t::time2` (protocol-26 netfield 61).
const TIME2: usize = 61;
/// The `killTime` `FX_DrawPortableShield` gives `FX_AddOrientedLine`.
const LIFETIME: Duration = Duration::from_millis(50);

/// The four looks, registered by `CG_RegisterGraphics` (`cg_main.c`); kept in the
/// effect atlas through [`crate::effect_assets::CODE_SHADERS`].
pub(crate) const SHADERS: [&str; 4] = [
    "gfx/misc/red_portashield",
    "gfx/misc/blue_portashield",
    "gfx/misc/red_dmgshield",
    "gfx/misc/blue_dmgshield",
];

/// One field this frame: the ends of its base line, its height and its look.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Field {
    /// The middle of the field's height at its negative end.
    pub(crate) start: Vec3,
    /// The same at its positive end.
    pub(crate) end: Vec3,
    pub(crate) height: f32,
    pub(crate) shader: &'static str,
}

/// The field `state` draws, when it is a standing portable shield.
pub(crate) fn field(state: &sjk_protocol::EntityState) -> Option<Field> {
    if state.entity_type() != ET_SPECIAL
        || state.model_index() != HI_SHIELD
        || state.e_flags() & EF_NODRAW != 0
    {
        return None;
    }
    let packed = state.raw_field(TIME2).unwrap_or(0);
    let along_x = (packed >> 24) & 1 != 0;
    let height = ((packed >> 16) & 255) as f32;
    let positive = ((packed >> 8) & 255) as f32;
    let negative = (packed & 255) as f32;
    let axis = if along_x { Vec3::X } else { Vec3::Y };
    // A placed field is stationary: `lerpOrigin` is its base.
    let origin = Vec3::from_array(state.trajectory_base()) + Vec3::Z * (height / 2.0);
    let red = state.other_entity_num2() == TEAM_RED;
    let damaged = state.tracked_entity_num() != 0;
    let shader = SHADERS[usize::from(!red) + 2 * usize::from(damaged)];
    Some(Field {
        start: origin - axis * negative,
        end: origin + axis * positive,
        height,
        shader,
    })
}

/// Add this cgame frame's copy of every standing field in `snapshot`.
pub(crate) fn spawn(
    snapshot: &Snapshot,
    particles: &mut Vec<Particle>,
    auxiliary: &mut crate::effect_aux::Runtime,
    effects: &mut EffectLibrary,
    now: Instant,
) {
    if !auxiliary.continuous.due(now) {
        return;
    }
    for field in snapshot.entities.iter().filter_map(field) {
        if particles.len() >= particle_types::MAX_PARTICLES {
            return;
        }
        particles.push(particle(field, effects, now));
    }
}

/// The particle a world shot places for `field` ([`crate::world_shot`]).
#[cfg(test)]
pub(crate) fn shot_particle(field: Field, effects: &mut EffectLibrary, now: Instant) -> Particle {
    particle(field, effects, now)
}

fn particle(field: Field, effects: &mut EffectLibrary, now: Instant) -> Particle {
    let constant = |value| {
        crate::effect_envelope::Envelope::from_values(
            value,
            value,
            0.0,
            sjk_effect::CurveFlags::default(),
        )
    };
    Particle {
        motion: crate::particle_motion::Motion::new(
            field.start,
            Vec3::ZERO,
            Vec3::ZERO,
            0.0,
            0.0,
            now,
        ),
        spawned_at: now,
        delay: Duration::ZERO,
        lifetime: LIFETIME,
        size: constant(field.height),
        length: None,
        streak: None,
        trace_streak: false,
        normal: None,
        alpha: constant(1.0),
        use_alpha: true,
        set_shader_time: false,
        rgb: [constant(1.0); 3],
        seed: 0,
        shader: effects.shader(field.shader),
        physics: crate::particle_physics::State::new(
            effects.code_primitive_definition(),
            0,
            0,
            0.0,
            sjk_effect::PrimitiveFlags::default(),
        ),
        shape: PrimitiveShape::OrientedLine {
            end: field.end,
            up: Vec3::Z,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjk_protocol::{EntityState, LEGACY_ENTITY_FIELDS};

    fn shield(time2: u32, team: u16, damaged: bool, flags: u32) -> EntityState {
        let mut state = EntityState::zero(9, &LEGACY_ENTITY_FIELDS);
        state.set_raw_field(8, ET_SPECIAL.into());
        state.set_raw_field(2, 100.0f32.to_bits());
        state.set_raw_field(1, 200.0f32.to_bits());
        state.set_raw_field(4, 10.0f32.to_bits());
        state.set_raw_field(46, HI_SHIELD as u32);
        state.set_raw_field(TIME2, time2);
        state.set_raw_field(39, team.into());
        state.set_raw_field(58, u32::from(damaged));
        state.set_raw_field(19, flags);
        state
    }

    #[test]
    fn time2_gives_the_fields_shape() {
        // Along y, 64 tall, 32 to the positive side and 48 to the negative one.
        let along_y = field(&shield(64 << 16 | 32 << 8 | 48, 2, false, 0)).unwrap();
        let origin = Vec3::from_array(shield(0, 0, false, 0).trajectory_base());
        assert_eq!(origin, Vec3::new(100.0, 200.0, 10.0));
        assert_eq!(along_y.start, Vec3::new(100.0, 152.0, 42.0));
        assert_eq!(along_y.end, Vec3::new(100.0, 232.0, 42.0));
        assert_eq!(along_y.height, 64.0);
        // Bit 24: along x.
        let along_x = field(&shield(1 << 24 | 64 << 16 | 32 << 8 | 48, 2, false, 0)).unwrap();
        assert_eq!(along_x.start, Vec3::new(52.0, 200.0, 42.0));
        assert_eq!(along_x.end, Vec3::new(132.0, 200.0, 42.0));
    }

    #[test]
    fn team_and_damage_choose_the_look() {
        let look = |team, damaged| field(&shield(1 << 16, team, damaged, 0)).unwrap().shader;
        assert_eq!(look(1, false), "gfx/misc/red_portashield");
        assert_eq!(look(2, false), "gfx/misc/blue_portashield");
        // Free for all: no team, the blue field.
        assert_eq!(look(0, false), "gfx/misc/blue_portashield");
        assert_eq!(look(1, true), "gfx/misc/red_dmgshield");
        assert_eq!(look(2, true), "gfx/misc/blue_dmgshield");
    }

    #[test]
    fn the_field_is_an_upright_quad_as_tall_as_its_height() {
        // `DoLine_Oriented`: half the width either side of the line along `up`, s across
        // it from the top, t along it from the start.
        let mut mesh = crate::effect_geometry::Mesh::default();
        mesh.oriented_line(
            Vec3::new(0.0, 0.0, 32.0),
            Vec3::new(0.0, 80.0, 32.0),
            Vec3::Z,
            32.0,
            [0.0, 0.0, 1.0, 1.0],
            [1.0, 1.0, 0.0, 0.0],
            [1.0; 4],
        );
        let corners: Vec<_> = mesh
            .vertices()
            .iter()
            .map(|vertex| (vertex.position, vertex.local_uv))
            .collect();
        assert_eq!(
            corners,
            [
                ([0.0, 0.0, 64.0], [0.0, 0.0]),
                ([0.0, 0.0, 0.0], [1.0, 0.0]),
                ([0.0, 80.0, 0.0], [1.0, 1.0]),
                ([0.0, 80.0, 64.0], [0.0, 1.0]),
            ]
        );
        assert_eq!(mesh.indices(), [0, 1, 2, 2, 3, 0]);
    }

    #[test]
    fn only_standing_shields_are_drawn() {
        assert!(field(&shield(1 << 16, 2, false, EF_NODRAW)).is_none());
        let mut other = shield(1 << 16, 2, false, 0);
        other.set_raw_field(46, 3);
        assert!(field(&other).is_none());
        let mut general = shield(1 << 16, 2, false, 0);
        general.set_raw_field(8, 0);
        assert!(field(&general).is_none());
    }
}
