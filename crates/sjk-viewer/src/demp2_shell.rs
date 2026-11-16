//! The DEMP2 alternate fire's shock bubble (OpenJK `codemp/cgame/fx_demp2.c:263-282`,
//! `FX_DEMP2_AltDetonate`, called for `EFFECT_EXPLOSION_DEMP2ALT` by `cg_event.c`):
//! besides `demp2/altdetonate`, the event leaves a local entity, `LE_FADE_SCALE_MODEL`,
//! which [`submit`] draws as `CG_AddFadeScaleModel` (`cg_localents.c:407-432`) does.
//!
//! The bubble is `models/items/sphere.md3` with `gfx/effects/demp2shell`, unrotated at
//! the event's origin, for 800 ms. Its radius is the event's `weapon`, the charge's units
//! doubled; the axes are scaled by that radius times the cube of the elapsed fraction,
//! the vertical by half of it, so it grows slowly and then bursts. `shaderRGBA` falls
//! from white to black on the same curve, its alpha zero throughout. `RF_VOLUMETRIC`
//! (`tr_shade.cpp:1164-1189`, the `RF_VOLUMETRIC` instance bit read by
//! `stage_runtime.wgsl`) then colours each vertex by that red, dimmed where the model's
//! normal runs along the view, so the edge glows and the middle stays clear.

use crate::{ActorInstance, entity_materials};
use glam::Vec3;
use std::time::{Duration, Instant};

/// `cgs.media.demp2Shell` and `cgs.media.demp2ShellShader` (`cg_weaponinit.c:431-432`).
pub(crate) const MODEL: &str = "models/items/sphere.md3";
pub(crate) const SHADER: &str = "gfx/effects/demp2shell";
/// Instance `view_flags` bit for `RF_VOLUMETRIC`, read by `stage_runtime.wgsl`.
pub(crate) const RF_VOLUMETRIC: u32 = 1 << 26;
/// `EV_PLAY_EFFECT` and `EFFECT_EXPLOSION_DEMP2ALT` (`bg_public.h`).
const EV_PLAY_EFFECT: u16 = 68;
const EFFECT_EXPLOSION_DEMP2ALT: u8 = 9;
/// `endTime - startTime`.
const LIFETIME: Duration = Duration::from_millis(800);
/// Bubbles alive at once; a full pool drops the oldest. Each lasts 800 ms and a DEMP2
/// fires its alternate at most every 500 ms, so 32 covers every player firing together.
const CAPACITY: usize = 32;

/// One bubble: `refEntity.origin`, `radius`, `startTime`.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Shell {
    origin: Vec3,
    radius: f32,
    started: Instant,
}

/// The bubbles in flight.
#[derive(Debug, Default)]
pub(crate) struct Shells {
    live: Vec<Shell>,
}

impl Shells {
    /// Start a bubble when `event` is `EV_PLAY_EFFECT` with `EFFECT_EXPLOSION_DEMP2ALT`,
    /// at `origin` with the event entity's `weapon` as its radius.
    pub(crate) fn observe(
        &mut self,
        event: u16,
        effect: u8,
        weapon: u8,
        origin: Vec3,
        now: Instant,
    ) {
        if event != EV_PLAY_EFFECT || effect != EFFECT_EXPLOSION_DEMP2ALT {
            return;
        }
        if self.live.len() == CAPACITY {
            self.live.remove(0);
        }
        self.live.push(Shell {
            origin,
            radius: f32::from(weapon),
            started: now,
        });
    }

    /// Drop the bubbles whose 800 ms are up (`CG_AddLocalEntities` frees an entity once
    /// `cg.time >= endTime`).
    fn expire(&mut self, now: Instant) {
        self.live
            .retain(|shell| now.saturating_duration_since(shell.started) < LIFETIME);
    }
}

/// `CG_AddFadeScaleModel` at `elapsed` into a bubble of `radius`: the axes' scale and
/// the entity colour's red, green and blue (alpha stays zero).
fn look(radius: f32, elapsed: Duration) -> ([f32; 3], f32) {
    let fraction = elapsed.as_secs_f32() / LIFETIME.as_secs_f32();
    let fraction = fraction * fraction * fraction;
    let size = radius * fraction;
    // `shaderRGBA` is a byte: `255 * (1 - frac)` truncated.
    let shade = (255.0 * (1.0 - fraction)).clamp(0.0, 255.0).trunc() / 255.0;
    ([size, size, size * 0.5], shade)
}

/// Draw the live bubbles with `mesh` (the sphere) and `material` (`demp2shell`) into
/// `overrides`. A local entity is in the main scene only, never the sky portal's.
pub(crate) fn submit(
    shells: &mut Shells,
    mesh: Option<usize>,
    material: usize,
    overrides: &mut Vec<entity_materials::OverrideInstance>,
    now: Instant,
) {
    shells.expire(now);
    let Some(mesh) = mesh else {
        return;
    };
    for shell in &shells.live {
        if overrides.len() == overrides.capacity() {
            break;
        }
        let (scale, shade) = look(shell.radius, now - shell.started);
        let mut instance = ActorInstance::new(shell.origin.to_array(), [0.0, 0.0, 0.0, 1.0], scale);
        instance.entity_color = [shade, shade, shade, 0.0];
        instance.view_flags = RF_VOLUMETRIC;
        overrides.push(entity_materials::OverrideInstance {
            mesh: entity_materials::OverrideMesh::Object(mesh),
            material: Some(material),
            instance,
            no_depth: false,
            forced_alpha: false,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grows_as_the_cube_and_fades() {
        let (scale, shade) = look(6.0, Duration::ZERO);
        assert_eq!(scale, [0.0; 3]);
        assert_eq!(shade, 1.0);
        let (scale, shade) = look(6.0, Duration::from_millis(400));
        assert!((scale[0] - 0.75).abs() < 1e-5);
        assert!((scale[2] - 0.375).abs() < 1e-5);
        // 255 * 0.875 = 223.125, truncated.
        assert_eq!(shade, 223.0 / 255.0);
        let (scale, shade) = look(6.0, Duration::from_millis(800));
        assert!((scale[1] - 6.0).abs() < 1e-5);
        assert_eq!(shade, 0.0);
    }

    #[test]
    fn only_the_demp2_alt_effect_starts_a_bubble() {
        let now = Instant::now();
        let mut shells = Shells::default();
        shells.observe(EV_PLAY_EFFECT, 7, 6, Vec3::ZERO, now);
        shells.observe(69, EFFECT_EXPLOSION_DEMP2ALT, 6, Vec3::ZERO, now);
        assert!(shells.live.is_empty());
        shells.observe(EV_PLAY_EFFECT, EFFECT_EXPLOSION_DEMP2ALT, 4, Vec3::X, now);
        assert_eq!(shells.live.len(), 1);
        assert_eq!(shells.live[0].radius, 4.0);
    }

    #[test]
    fn bubbles_are_drawn_for_800_ms() {
        let start = Instant::now();
        let mut shells = Shells::default();
        shells.observe(
            EV_PLAY_EFFECT,
            EFFECT_EXPLOSION_DEMP2ALT,
            2,
            Vec3::ONE,
            start,
        );
        let mut overrides = Vec::with_capacity(4);
        submit(
            &mut shells,
            Some(3),
            11,
            &mut overrides,
            start + Duration::from_millis(799),
        );
        assert_eq!(overrides.len(), 1);
        let drawn = &overrides[0];
        assert_eq!(drawn.mesh, entity_materials::OverrideMesh::Object(3));
        assert_eq!(drawn.material, Some(11));
        assert_eq!(drawn.instance.view_flags & RF_VOLUMETRIC, RF_VOLUMETRIC);
        assert_eq!(drawn.instance.entity_color[3], 0.0);
        overrides.clear();
        submit(&mut shells, Some(3), 11, &mut overrides, start + LIFETIME);
        assert!(overrides.is_empty());
        assert!(shells.live.is_empty());
    }

    #[test]
    fn a_full_pool_drops_the_oldest() {
        let now = Instant::now();
        let mut shells = Shells::default();
        for radius in 0..=CAPACITY as u8 {
            shells.observe(
                EV_PLAY_EFFECT,
                EFFECT_EXPLOSION_DEMP2ALT,
                radius,
                Vec3::ZERO,
                now,
            );
        }
        assert_eq!(shells.live.len(), CAPACITY);
        assert_eq!(shells.live[0].radius, 1.0);
    }
}
