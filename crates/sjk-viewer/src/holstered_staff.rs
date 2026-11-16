//! Frame-driven JA+ staff handoff; no changes to gameplay or animation timing.
use std::cell::Cell;

/// Animation names from EJK f21eee8 CG_StaffSwapPhaseReal. Each entry states
/// whether the reach draws the hilt or puts it away. These are content contracts.
const REACHES: &[(&str, bool)] = &[
    ("BOTH_STAND1TO2_NEW", true),
    ("BOTH_S1_S7_NEW", true),
    ("BOTH_STAND2TO1_NEW", false),
    ("BOTH_S7_S1_NEW", false),
];
const PRELUDES: &[(&str, &str)] = &[
    ("BOTH_STAND1TO2", "BOTH_STAND1TO2_NEW"),
    ("BOTH_S1_S7", "BOTH_S1_S7_NEW"),
];

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub(crate) enum Phase {
    #[default]
    None,
    OnBack,
    InHand,
}

#[derive(Clone, Copy, Default)]
struct Latch {
    key: Option<(usize, u64, i64)>,
    swapped: bool,
    time: i64,
}

/// Retained with the actor, so replacing a model/client never retains a swap.
#[derive(Default)]
pub(crate) struct Swap(Cell<Latch>);

impl Swap {
    pub(crate) fn reset(&self) {
        self.0.set(Latch::default());
    }

    pub(crate) fn phase(&self, mesh: &crate::ActorMesh, now: i64, threshold: f32) -> (Phase, bool) {
        let Some(track) = mesh.animator.requested.map(|state| state.upper) else {
            self.reset();
            return (Phase::None, false);
        };
        let Some(name) = sjk_client::legacy_animation_name(track.clip) else {
            self.reset();
            return (Phase::None, false);
        };
        if let Some((_, reach)) = PRELUDES.iter().find(|(plain, _)| *plain == name) {
            self.reset();
            return (
                if mesh.preview.config.get_exact(reach).is_some_and(|s| {
                    s.frame_count > 1
                        && s.first_frame + s.frame_count <= mesh.preview.animation.frames.len()
                }) {
                    Phase::OnBack
                } else {
                    Phase::None
                },
                false,
            );
        }
        let Some((_, drawing)) = REACHES.iter().find(|(reach, _)| *reach == name) else {
            self.reset();
            return (Phase::None, false);
        };
        let Some(sequence) = mesh.preview.config.get_exact(name).filter(|s| {
            s.frame_count > 1
                && s.first_frame
                    .checked_add(s.frame_count)
                    .is_some_and(|end| end <= mesh.preview.animation.frames.len())
                && track.forced_frame.is_none()
        }) else {
            self.reset();
            return (Phase::None, false);
        };
        // The installed track must be this reach, not a fallback idle clip.
        let Some(frame) = mesh
            .animator
            .torso_frame(&mesh.preview.animation, now)
            .filter(|_| mesh.animator.torso_clip() == Some(track.clip))
        else {
            self.reset();
            return (Phase::None, false);
        };
        let fraction = (frame - sequence.first_frame as f32) / sequence.frame_count as f32;
        (
            self.sample(
                (track.clip, track.revision, track.started_at_millis),
                now,
                fraction,
                threshold,
                *drawing,
            ),
            !drawing,
        )
    }

    fn sample(
        &self,
        key: (usize, u64, i64),
        now: i64,
        fraction: f32,
        threshold: f32,
        drawing: bool,
    ) -> Phase {
        let mut latch = self.0.get();
        if latch.key != Some(key) || now < latch.time {
            latch = Latch {
                key: Some(key),
                ..Latch::default()
            };
        }
        latch.time = now;
        let after = if drawing {
            Phase::InHand
        } else {
            Phase::OnBack
        };
        let phase = if fraction.clamp(0.0, 1.0) < threshold {
            Phase::OnBack
        } else {
            Phase::InHand
        };
        latch.swapped |= phase == after;
        self.0.set(latch);
        if latch.swapped { after } else { phase }
    }
}

pub(crate) fn installed(mesh: &crate::ActorMesh) -> bool {
    REACHES.iter().any(|(name, _)| {
        mesh.preview.config.get_exact(name).is_some_and(|s| {
            s.frame_count > 1
                && s.first_frame
                    .checked_add(s.frame_count)
                    .is_some_and(|end| end <= mesh.preview.animation.frames.len())
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stowing_retracts_immediately_but_next_ignition_grows() {
        let mut blade = crate::saber_trail::BladeState::default();
        assert_eq!(blade.extension.update(true, 40.0, 0), 40.0);
        blade.stow(100);
        assert_eq!(blade.extension.update(false, 40.0, 100), 0.0);
        let length = blade.extension.update(true, 40.0, 200);
        assert!(length > 0.0 && length < 40.0);
    }
    #[test]
    fn draw_and_reverse_putaway_swap_once_at_every_command_step() {
        for step in [8, 7, 4, 3] {
            for drawing in [true, false] {
                let swap = Swap::default();
                let mut crossed = false;
                for now in (0..=1000).step_by(step) {
                    let fraction = if drawing {
                        now as f32 / 1000.0
                    } else {
                        1.0 - now as f32 / 1000.0
                    };
                    let phase = swap.sample((1, 1, 0), now, fraction, 0.45, drawing);
                    let after = if drawing {
                        Phase::InHand
                    } else {
                        Phase::OnBack
                    };
                    crossed |= phase == after;
                    if crossed {
                        assert_eq!(phase, after);
                    }
                }
                assert!(crossed);
                assert_eq!(
                    swap.sample(
                        (1, 1, 0),
                        1100,
                        if drawing { 0.44 } else { 0.46 },
                        0.45,
                        drawing
                    ),
                    if drawing {
                        Phase::InHand
                    } else {
                        Phase::OnBack
                    }
                );
                assert_eq!(
                    swap.sample(
                        (1, 2, 1101),
                        1101,
                        if drawing { 0.0 } else { 1.0 },
                        0.45,
                        drawing
                    ),
                    if drawing {
                        Phase::OnBack
                    } else {
                        Phase::InHand
                    }
                );
            }
        }
    }
    #[test]
    fn rewind_and_reset_forget_the_old_playing() {
        let s = Swap::default();
        assert_eq!(s.sample((1, 1, 0), 500, 0.6, 0.45, true), Phase::InHand);
        assert_eq!(s.sample((1, 1, 0), 100, 0.1, 0.45, true), Phase::OnBack);
        s.reset();
        assert_eq!(s.sample((1, 1, 0), 600, 0.1, 0.45, true), Phase::OnBack);
    }
}
