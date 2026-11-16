//! The camera of a fall to death (a `trigger_hurt` with damage -1, the pits that
//! fade the screen to black): it stays where the fall began and looks down at the
//! player instead of following him into the pit.
//!
//! EternalJK keeps the player state's origin of the first frame with
//! `ps.fallingToDeath` set as `gCGFallVector` (`cg_draw.c:11400-11443`; cleared when
//! the flag clears). The third-person camera then takes that point, not the player,
//! as its focus (`CG_CalcIdealThirdPersonViewTarget`, `cg_view.c:286-292`), and once
//! the view is computed `CG_DrawActiveFrame` moves the eye to the point itself,
//! turned along the line from the computed eye to the snapshot's origin
//! (`cg_view.c:3224-3233`). The flag stays set after the fall kills the player, so
//! the camera stays there until the respawn clears it (`g_client.c:4283-4285`).

use glam::Vec3;

/// `gCGFallVector`: where the fall to death began, while it lasts.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct FallCamera {
    origin: Option<Vec3>,
}

impl FallCamera {
    /// Follow this frame's presented player state: `falling` is its
    /// `fallingToDeath != 0`, `origin` its origin. Returns the fall point while the
    /// fall lasts.
    pub(crate) fn observe(&mut self, falling: bool, origin: Vec3) -> Option<Vec3> {
        if !falling {
            self.origin = None;
        } else if self.origin.is_none() {
            self.origin = Some(origin);
        }
        self.origin
    }

    /// The fall point, while a fall to death lasts.
    pub(crate) fn origin(&self) -> Option<Vec3> {
        self.origin
    }

    /// Forget the fall (no session, a new map or a different view).
    pub(crate) fn clear(&mut self) {
        self.origin = None;
    }
}

impl crate::GpuState {
    /// Follow the presented snapshot's fall to death (EternalJK reads `cg.snap`),
    /// returning the fall point and the snapshot's origin while it lasts.
    /// Without a live or demo session the fall is forgotten.
    pub(crate) fn update_fall_camera(&mut self, presentation_time: i32) -> Option<(Vec3, Vec3)> {
        let Some(snapshot) = crate::first_person_view::presented_snapshot(
            self.live_session.as_ref(),
            self.demo_session.as_ref(),
            presentation_time,
        ) else {
            self.fall_camera.clear();
            return None;
        };
        let origin = Vec3::from_array(snapshot.player.origin());
        let falling = snapshot.player.falling_to_death() != 0;
        self.fall_camera
            .observe(falling, origin)
            .map(|fall| (fall, origin))
    }
}

/// The view placed at the fall point `fall` and turned along the line from the
/// computed eye `view_position` to the player's `origin` (`cg_view.c:3224-3233`):
/// position, target and up. If the two points coincide, the computed direction is
/// kept. The view is unrolled (`vectoangles` gives no roll), and a straight vertical
/// line gets `AnglesToAxis`'s up for yaw 0 (+X looking down, -X looking up).
pub(crate) fn look(
    fall: Vec3,
    origin: Vec3,
    view_position: Vec3,
    view_target: Vec3,
) -> (Vec3, Vec3, Vec3) {
    let direction = (origin - view_position)
        .try_normalize()
        .or_else(|| (view_target - view_position).try_normalize())
        .unwrap_or(Vec3::NEG_Z);
    let up = if direction.truncate().length() > 1e-4 {
        Vec3::Z
    } else if direction.z < 0.0 {
        Vec3::X
    } else {
        Vec3::NEG_X
    };
    (fall, fall + direction * 256.0, up)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fall_point_is_where_the_fall_began_until_it_ends() {
        let mut fall = FallCamera::default();
        assert_eq!(fall.observe(false, Vec3::new(1.0, 2.0, 3.0)), None);
        let start = Vec3::new(100.0, 0.0, 64.0);
        assert_eq!(fall.observe(true, start), Some(start));
        // The player drops on; the point stays where the fall began.
        assert_eq!(fall.observe(true, start - Vec3::Z * 500.0), Some(start));
        assert_eq!(fall.origin(), Some(start));
        // The respawn clears the flag and the camera follows again.
        assert_eq!(fall.observe(false, Vec3::ZERO), None);
        let next = Vec3::new(-5.0, 0.0, 0.0);
        assert_eq!(fall.observe(true, next), Some(next));
        fall.clear();
        assert_eq!(fall.origin(), None);
    }

    #[test]
    fn the_view_sits_at_the_fall_point_and_looks_from_the_computed_eye_to_the_player() {
        let fall = Vec3::new(0.0, 0.0, 100.0);
        // Third-person eye behind and above the fall point, the player far below.
        let eye = Vec3::new(-80.0, 0.0, 130.0);
        let player = Vec3::new(0.0, 0.0, -400.0);
        let (position, target, up) = look(fall, player, eye, eye + Vec3::X);
        assert_eq!(up, Vec3::Z);
        assert_eq!(position, fall);
        let expected = (player - eye).normalize();
        assert!(((target - position).normalize() - expected).length() < 1e-5);
        assert!(expected.z < -0.9, "it looks down into the pit");
    }

    #[test]
    fn an_eye_on_the_player_keeps_the_computed_direction() {
        let fall = Vec3::new(0.0, 0.0, 100.0);
        let eye = Vec3::new(5.0, 5.0, 5.0);
        let (_, target, _) = look(fall, eye, eye, eye + Vec3::Y * 10.0);
        assert!(((target - fall).normalize() - Vec3::Y).length() < 1e-6);
    }

    #[test]
    fn a_straight_drop_below_the_eye_gets_a_usable_up() {
        let fall = Vec3::ZERO;
        let eye = Vec3::new(0.0, 0.0, 36.0);
        let (_, target, up) = look(fall, Vec3::ZERO, eye, eye + Vec3::X);
        assert_eq!((target - fall).normalize(), Vec3::NEG_Z);
        assert_eq!(up, Vec3::X);
    }
}
