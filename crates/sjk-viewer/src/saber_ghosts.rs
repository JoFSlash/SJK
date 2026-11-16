//! Afterimages of a blade whose skin leaves them (`ghosts` in its blade-skin file,
//! `docs/unlockables.md`): every `spacing` milliseconds a blade's pose is kept, and the
//! last few are drawn again as the glow alone, each dimmer than the one before. A pose the
//! blade has not moved away from is not drawn (it would only brighten the blade). The
//! poses live in the blade's fixed state ([`crate::saber_trail::BladeState`]); nothing
//! is allocated.

use crate::blade_skin_file::effects::MAX_GHOSTS;
use crate::saber::{Blade, Instance};
use crate::saber_rgb::BladeColor;

/// A skin's afterimages, as its colour carries them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GhostSpec {
    /// Afterimages, 1 to [`MAX_GHOSTS`].
    pub(crate) count: u8,
    pub(crate) spacing_millis: f32,
    /// The first one's brightness; each next is this much of the one before.
    pub(crate) fade: f32,
    /// Kept in the wearer's own frame (Creyon's signature blades, round 12): turning the
    /// view or running moves the whole body, which leaves no afterimage; only the blade's
    /// own swing does.
    pub(crate) wearer: bool,
}

/// How far (units) a kept pose's tip must be from the blade's for it to be drawn.
const MOVED: f32 = 1.5;

/// The faintest afterimage drawn: one dimmer barely shows over the others and the scene
/// (the Spectral's fourth, 0.45⁴ ≈ 0.04, hub report 64).
const FAINTEST: f32 = 0.06;

/// One blade's kept poses, newest first.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct GhostTrail {
    poses: [Option<Blade>; MAX_GHOSTS as usize],
    last_millis: i64,
}

impl GhostTrail {
    /// Keep `blade` when `spacing` has passed since the last pose kept; a jump back in
    /// time (a new map, a demo seek) or a long pause forgets them all.
    pub(crate) fn record(&mut self, blade: Blade, now: i64, spec: GhostSpec) {
        let since = now - self.last_millis;
        if !(0..=1_000).contains(&since) {
            *self = Self::default();
        }
        if self.poses[0].is_some() && (since as f32) < spec.spacing_millis {
            return;
        }
        self.poses.rotate_right(1);
        self.poses[0] = Some(blade);
        self.last_millis = now;
    }

    /// The afterimages of `blade` (the one drawn this frame), as glow instances of
    /// `color`, each `fade` times the one before; poses it has not moved from are skipped.
    pub(crate) fn instances(
        &self,
        blade: Blade,
        color: BladeColor,
        spec: GhostSpec,
    ) -> impl Iterator<Item = Instance> + '_ {
        self.instances_placed(blade, color, spec, |pose| pose)
    }

    /// [`Self::instances`] for poses kept in another frame (the wearer's): `blade` in that
    /// frame too, and `place` taking a pose back to the world before it is drawn.
    pub(crate) fn instances_placed<'s>(
        &'s self,
        blade: Blade,
        color: BladeColor,
        spec: GhostSpec,
        place: impl Fn(Blade) -> Blade + 's,
    ) -> impl Iterator<Item = Instance> + 's {
        let tip = |b: &Blade| {
            glam::Vec3::from_array(b.base) + glam::Vec3::from_array(b.direction) * b.length
        };
        let now_tip = tip(&blade);
        self.poses
            .iter()
            .take(usize::from(spec.count))
            .enumerate()
            .filter_map(move |(index, pose)| {
                let pose = (*pose)?;
                let fade = spec.fade.powi(index as i32 + 1);
                (fade >= FAINTEST && tip(&pose).distance(now_tip) >= MOVED)
                    .then(|| Instance::ghost(place(pose), color, fade))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blade(x: f32) -> Blade {
        Blade {
            base: [x, 0.0, 0.0],
            direction: [0.0, 0.0, 1.0],
            length: 40.0,
            radius: 3.0,
        }
    }

    const SPEC: GhostSpec = GhostSpec {
        wearer: false,
        count: 3,
        spacing_millis: 40.0,
        fade: 0.5,
    };

    #[test]
    fn poses_are_kept_every_spacing_and_drawn_fading() {
        let mut trail = GhostTrail::default();
        for (step, now) in [1_000, 1_020, 1_040, 1_080, 1_120].into_iter().enumerate() {
            trail.record(blade(step as f32 * 10.0), now, SPEC);
        }
        // Kept at 1000, 1040, 1080 and 1120 (1020 was too soon): newest first.
        let kept: Vec<f32> = trail.poses.iter().flatten().map(|b| b.base[0]).collect();
        assert_eq!(kept, [40.0, 30.0, 20.0, 0.0]);
        let color = BladeColor::from_rgb([0, 0, 255]);
        // Drawn from the blade at x = 50: all three moved, fading by halves.
        let fades: Vec<f32> = trail
            .instances(blade(50.0), color, SPEC)
            .map(|i| i.fade())
            .collect();
        // To a byte.
        assert_eq!(fades.len(), 3);
        for (fade, expected) in fades.into_iter().zip([0.5, 0.25, 0.125]) {
            assert!((fade - expected).abs() < 1.0 / 255.0, "{fade}");
        }
        // From the newest pose itself, that one is skipped.
        assert_eq!(trail.instances(blade(40.0), color, SPEC).count(), 2);
        // Each is marked as an afterimage, drawn as the plain capsule.
        assert!(
            trail
                .instances(blade(50.0), color, SPEC)
                .all(|i| i.is_afterimage())
        );
        // One fainter than `FAINTEST` is left out: the Spectral's fourth.
        let spectral = GhostSpec {
            wearer: false,
            count: 4,
            fade: 0.45,
            ..SPEC
        };
        let fades: Vec<f32> = trail
            .instances(blade(50.0), color, spectral)
            .map(|i| i.fade())
            .collect();
        assert_eq!(fades.len(), 3, "{fades:?}");
    }

    #[test]
    fn a_jump_in_time_forgets_the_poses() {
        let mut trail = GhostTrail::default();
        trail.record(blade(0.0), 5_000, SPEC);
        trail.record(blade(5.0), 9_000, SPEC);
        assert_eq!(trail.poses.iter().flatten().count(), 1);
        trail.record(blade(9.0), 100, SPEC);
        assert_eq!(trail.poses.iter().flatten().count(), 1);
    }
}
