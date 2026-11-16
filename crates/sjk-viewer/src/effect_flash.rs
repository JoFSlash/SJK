//! An effect's `Flash` primitive (`CFlash`, OpenJK `codemp/client/FxPrimitives.cpp`):
//! a flash of the screen whose colour `CFlash::Init` scales by where the effect is from
//! the viewer when it starts. Full within a few units, fading with the square of the
//! distance to nothing at 900 units, and nothing beyond 60 degrees off the view axis
//! unless within 100 units. `demp2/altdetonate` has one (2048 units of
//! `gfx/effects/whiteFlash`): drawn at full colour wherever a DEMP2 shock went off, it
//! lit the whole screen from across the map (Slash, hub report #125).
//!
//! `CFlash::Draw` then draws it on the viewer, not where it went off: an `RT_SPRITE`
//! `FLASH_DISTANCE_FROM_VIEWER` (12) units along the view axis, of radius
//! `12 * tan(fov_x / 2)`, which spans the view's width and, on a screen wider than tall,
//! more than its height. Its size curve is ignored. `whiteFlash` is `$whiteimage`, added
//! (`blendFunc GL_ONE GL_ONE`) with `depthFunc disable` and `sort nearest`, so nothing
//! hides it. SJK draws that sprite as the clip-space quad [`screen_half_extents`] gives
//! (`entity.wgsl` kind 6), with the primitive's colour and alpha curves as stock's
//! `UpdateRGB`/`UpdateAlpha` give them. Drawn as a 2048-unit billboard at the shock, the
//! flash was hidden by everything nearer than it and tinted all behind it, the sky
//! included, up to a hard edge. `localizedFlash` (`FX_LOCALIZED_FLASH`, a 2D picture at
//! the effect's screen position) is not modelled: no retail effect uses it.

use glam::Vec3;
use std::sync::atomic::{AtomicBool, Ordering};

/// `cg_reduceFlashes`: the accessibility switch for photosensitive players (Piza, hub
/// report #138). On, no bright flash is drawn: no saber clash flare
/// ([`crate::saber_clash_flare`]), no effect `Flash` (the screen flash of a DEMP2
/// shock, a rocket or a concussion blast), and no effect particle drawn with the clash
/// flare's own picture ([`is_clash_flare_picture`]). Off by default: retail draws them.
pub(crate) const REDUCE_CVAR: &str = "cg_reduceFlashes";

/// [`REDUCE_CVAR`] as the frame last read it, for the effect spawner and submission,
/// which have no console.
static REDUCED: AtomicBool = AtomicBool::new(false);

/// Note [`REDUCE_CVAR`]'s value (once a frame).
pub(crate) fn set_reduced(reduced: bool) {
    REDUCED.store(reduced, Ordering::Relaxed);
}

/// Whether bright flashes are reduced ([`REDUCE_CVAR`]).
pub(crate) fn reduced() -> bool {
    REDUCED.load(Ordering::Relaxed)
}

/// The clash flare's picture, `gfx/effects/saberFlare`, which the saber clash flare
/// draws over the screen. Retail effects never use it, but packs that remake the saber
/// block effect as a flash do (JoF's `JoF_HDMovieSaberClash.pk3` draws a 55-unit
/// flare with it at every block), so reduced flashes leave out any effect particle
/// that draws it, whatever pack supplies the effect.
pub(crate) fn is_clash_flare_picture(shader: &str) -> bool {
    shader.eq_ignore_ascii_case(crate::saber_clash_flare::PICTURE)
}

/// `CFlash::Draw`'s `FLASH_DISTANCE_FROM_VIEWER`: how far in front of the eye the
/// flash's sprite stands.
const FLASH_DISTANCE_FROM_VIEWER: f32 = 12.0;

/// `CFlash::Init`'s `maxRange`.
const MAX_RANGE: f32 = 900.0;

/// `CFlash::Init`'s colour scale for a flash at `origin`, seen from `viewer`'s eye
/// looking along its unit `forward` axis; 0 draws nothing. No viewer yet: no flash.
pub(crate) fn visibility(viewer: Option<(Vec3, Vec3)>, origin: Vec3) -> f32 {
    let Some((eye, forward)) = viewer else {
        return 0.0;
    };
    let offset = origin - eye;
    let distance = offset.length();
    // `VectorNormalize` leaves a zero vector at zero length.
    let direction = if distance > 0.0 {
        offset / distance
    } else {
        Vec3::ZERO
    };
    let mut scale = direction.dot(forward);
    if distance > MAX_RANGE || (scale < 0.5 && distance > 100.0) {
        scale = 0.0;
    } else if scale < 0.5 && distance <= 100.0 {
        scale += 1.1;
    }
    scale * (1.0 - (distance * distance) / (MAX_RANGE * MAX_RANGE))
}

/// Half the width and height, in normalized device coordinates, of `CFlash::Draw`'s
/// sprite for a view of `aspect` (width over height): a square of radius
/// `12 * tan(fov_x / 2)` 12 units ahead, so exactly the view's width and `aspect` times
/// its height (`tan(fov_x / 2) / tan(fov_y / 2)`).
pub(crate) fn screen_half_extents(aspect: f32) -> [f32; 2] {
    let tan_half_y = 1.0_f32;
    let tan_half_x = aspect * tan_half_y;
    let radius = FLASH_DISTANCE_FROM_VIEWER * tan_half_x;
    [
        radius / (FLASH_DISTANCE_FROM_VIEWER * tan_half_x),
        radius / (FLASH_DISTANCE_FROM_VIEWER * tan_half_y),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const EYE: Vec3 = Vec3::new(10.0, 20.0, 64.0);

    fn at(forward: f32, side: f32) -> f32 {
        visibility(Some((EYE, Vec3::X)), EYE + Vec3::new(forward, side, 0.0))
    }

    #[test]
    fn only_the_clash_flare_picture_is_a_flash_picture() {
        assert!(is_clash_flare_picture("gfx/effects/saberFlare"));
        assert!(is_clash_flare_picture("GFX/EFFECTS/SABERFLARE"));
        assert!(!is_clash_flare_picture("gfx/effects/whiteFlare"));
        assert!(!is_clash_flare_picture("gfx/misc/spark"));
    }

    #[test]
    fn a_flash_fades_with_distance_and_is_gone_far_away_or_off_the_view() {
        // Straight ahead: full close by, 1 - d^2 / 900^2 further.
        assert!((at(10.0, 0.0) - (1.0 - 100.0 / 810_000.0)).abs() < 1e-5);
        assert!((at(450.0, 0.0) - 0.75).abs() < 1e-5);
        // Across the map (Slash #125): nothing.
        assert_eq!(at(901.0, 0.0), 0.0);
        assert_eq!(at(3_000.0, 0.0), 0.0);
        // Off to the side past 60 degrees, or behind, beyond 100 units: nothing.
        assert_eq!(at(100.0, 200.0), 0.0);
        assert_eq!(at(-300.0, 0.0), 0.0);
        // Within 100 units off the view, stock brightens it instead.
        let near = at(0.0, 50.0);
        assert!((near - 1.1 * (1.0 - 2_500.0 / 810_000.0)).abs() < 1e-5);
        // No viewer, no flash.
        assert_eq!(visibility(None, EYE), 0.0);
    }

    #[test]
    fn the_flash_spans_the_view_width_and_overfills_its_height() {
        assert_eq!(screen_half_extents(1.0), [1.0, 1.0]);
        let [width, height] = screen_half_extents(16.0 / 9.0);
        assert!((width - 1.0).abs() < 1e-6);
        assert!((height - 16.0 / 9.0).abs() < 1e-6);
    }
}
