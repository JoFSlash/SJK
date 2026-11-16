//! Local camera and sun control (the Camera control panel, `demo_camera` and
//! `demo_sun`). Camera offsets never enter input, prediction or usercmds.
use glam::Vec3;
use std::time::Instant;
#[path = "demo_director_commands.rs"]
mod commands;
#[path = "demo_motion.rs"]
mod motion;
use motion::Motion;
pub(crate) use motion::Request;

/// The sun's current presentation owner, independent of whether the panel is visible.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum SunMode {
    #[default]
    Automatic,
    Manual,
    Returning,
}

/// Transient shot state owned by the console, separate from simulation and saved cvars.
pub(crate) struct Director {
    epoch: Instant,
    camera: Motion<4>,
    /// The framing cvars the third-person view last read ([`Self::follow_cvars`]).
    framing: Option<[f32; 4]>,
    sun: Motion<2>,
    sky_weight: Motion<1>,
}

impl Default for Director {
    fn default() -> Self {
        Self {
            epoch: Instant::now(),
            camera: Motion::default(),
            framing: None,
            sun: Motion::default(),
            sky_weight: Motion::default(),
        }
    }
}

impl Director {
    /// Queue a camera edit from either the console or the shot panel.
    pub(crate) fn set_camera(&mut self, request: Request<4>) {
        self.camera.pending = Some(request);
    }

    /// Queue sunlight and its matching atmosphere transition through one path.
    pub(crate) fn set_sun(&mut self, request: Request<2>) {
        self.sky_weight.pending = Some(match request {
            Request::Target(_, seconds) => Request::Target([1.], seconds),
            Request::Auto(seconds) => Request::Auto(seconds),
            _ => Request::Target([1.], 2.),
        });
        self.sun.pending = Some(request);
    }

    /// Remove all manual sunlight and sky state, including requests not yet sampled.
    /// The next render upload uses the existing day clock without an override.
    pub(crate) fn reset_sun(&mut self) {
        self.sun = Motion::default();
        self.sky_weight = Motion::default();
    }

    /// Read the actual controller mode, not a parallel UI toggle.
    pub(crate) fn sun_mode(&self) -> SunMode {
        if self.sun.returning() {
            SunMode::Returning
        } else if self.sun.active() {
            SunMode::Manual
        } else {
            SunMode::Automatic
        }
    }

    /// The third-person view read `framing` from its cvars (`cg_thirdPersonAngle`,
    /// `cg_thirdPersonPitchOffset`, `cg_thirdPersonRange`, `cg_thirdPersonVertOffset`): a
    /// cvar changed since the last frame (from the console, a bind or Settings) takes its
    /// place in the camera Camera control holds, as EternalJK applies it at once.
    /// Without this a pose set in Camera control kept overriding the cvars until its Reset
    /// (Rollembones' report, 22/10/2026: `cg_thirdPersonRange` did nothing). Called by the
    /// third-person view only, so the panel's own reads never count as a change.
    pub(crate) fn follow_cvars(&mut self, framing: [f32; 4]) {
        let Some(last) = self.framing.replace(framing) else {
            return;
        };
        for (index, (was, now)) in last.into_iter().zip(framing).enumerate() {
            if was.to_bits() != now.to_bits() {
                self.camera.adopt(index, now);
            }
        }
    }

    /// Camera yaw/pitch offsets, range and target height; None leaves the stock path intact.
    pub(crate) fn camera(&mut self, fallback: [f32; 4]) -> Option<[f32; 4]> {
        if !self.camera.active() {
            return None;
        }
        self.camera
            .sample(fallback, self.epoch.elapsed().as_secs_f64())
    }

    /// True only while a manual sun control needs sampling.
    pub(crate) fn sun_active(&self) -> bool {
        self.sun.active()
    }

    /// World-space sun direction, including a smooth handoff back to the running day clock.
    pub(crate) fn sun(&mut self, natural: Vec3) -> Option<(Vec3, f32)> {
        let natural = natural.normalize_or_zero();
        let angles = [
            natural.y.atan2(natural.x).to_degrees(),
            natural.z.clamp(-1., 1.).asin().to_degrees(),
        ];
        let now = self.epoch.elapsed().as_secs_f64();
        let weight = self.sky_weight.sample([0.], now).map_or(0., |v| v[0]);
        self.sun.sample(angles, now).map(|[yaw, elevation]| {
            let (sy, cy) = yaw.to_radians().sin_cos();
            let (se, ce) = elevation.to_radians().sin_cos();
            (Vec3::new(cy * ce, sy * ce, se), weight)
        })
    }
}

pub(super) use commands::register;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_framing_cvar_changed_replaces_its_part_of_the_held_camera() {
        let cvars = [0., 0., 80., 16.];
        let mut director = Director::default();
        director.follow_cvars(cvars);
        assert_eq!(
            director.camera(cvars),
            None,
            "no manual camera: the cvars rule"
        );
        director.set_camera(Request::Target([90., 10., 200., 30.], 0.));
        director.follow_cvars(cvars);
        assert_eq!(director.camera(cvars), Some([90., 10., 200., 30.]));
        // `cg_thirdPersonRange 150` from the console: the range follows, the rest stays.
        let ranged = [0., 0., 150., 16.];
        director.follow_cvars(ranged);
        assert_eq!(director.camera(ranged), Some([90., 10., 150., 30.]));
        // An unchanged cvar leaves the held pose alone.
        director.follow_cvars(ranged);
        assert_eq!(director.camera(ranged), Some([90., 10., 150., 30.]));
        // A change asked for before the pose is first sampled is kept too.
        let mut director = Director::default();
        director.follow_cvars(cvars);
        director.set_camera(Request::Target([90., 10., 200., 30.], 0.));
        director.follow_cvars(ranged);
        assert_eq!(director.camera(ranged), Some([90., 10., 150., 30.]));
    }
}
