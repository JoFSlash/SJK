//! `PM_CheckDualForwardJumpDuck` and `PM_CheckFixMins`, codemp bg_pmove.c:4290-4376.

use super::{MovementCollision, PLAYER_CONTENT_MASK, PMF_DUCKED, Predictor};
const PMF_FIX_MINS: u16 = 128;
const BOTH_JUMPATTACK6: u16 = 861;
const MINS_Z: f32 = -24.0;

impl Predictor {
    pub(super) fn dual_jump_minimum(&mut self, collision: &impl MovementCollision) -> f32 {
        let state = &mut self.state;
        if state.legs_anim == BOTH_JUMPATTACK6
            && let Some(length) = self
                .animation_lengths
                .as_deref()
                .and_then(|lengths| lengths.length_ms(BOTH_JUMPATTACK6))
        {
            let elapsed = length - state.legs_timer;
            if (state.legs_timer >= 1_450 && elapsed >= 400)
                || (state.legs_timer >= 400 && elapsed >= 1_100)
            {
                state.movement_flags |= PMF_FIX_MINS;
                return 0.0;
            }
        }
        if state.movement_flags & PMF_FIX_MINS == 0 {
            return MINS_Z;
        }
        let origin = state.origin;
        let minimums = [-15.0, -15.0, 0.0];
        let maximums = [15.0, 15.0, state.standing_height];
        let mut end = origin;
        end[2] += MINS_Z;
        let down = collision.trace(origin, minimums, maximums, end, PLAYER_CONTENT_MASK);
        if down.all_solid || down.start_solid {
            return MINS_Z;
        }
        if down.fraction >= 1.0 {
            state.movement_flags &= !PMF_FIX_MINS;
            return MINS_Z;
        }
        let up_distance = (1.0 - down.fraction) * -MINS_Z;
        end[2] = origin[2] + up_distance;
        let up = collision.trace(origin, minimums, maximums, end, PLAYER_CONTENT_MASK);
        if !up.all_solid && !up.start_solid {
            if up.fraction >= 1.0 {
                state.origin[2] += up_distance;
                state.movement_flags &= !PMF_FIX_MINS;
            } else if state.legs_anim != BOTH_JUMPATTACK6 || state.legs_timer <= 200 {
                state.origin[2] -= MINS_Z;
                if state.legs_anim == BOTH_JUMPATTACK6 {
                    state.legs_timer = 0;
                    state.torso_timer = 0;
                }
                state.movement_flags = (state.movement_flags | PMF_DUCKED) & !PMF_FIX_MINS;
            }
        }
        // PM_CheckDuck also restores a zero minimum after an unsuccessful fix.
        MINS_Z
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pmove::{MovementConfig, MovementState, MovementTrace};
    use crate::pmove_anim::{AnimationLengthTable, AnimationTiming};
    use std::sync::Arc;

    struct Space {
        down: f32,
        up: f32,
    }
    impl MovementCollision for Space {
        fn trace(
            &self,
            start: [f32; 3],
            _: [f32; 3],
            _: [f32; 3],
            end: [f32; 3],
            _: u32,
        ) -> MovementTrace {
            MovementTrace {
                fraction: if end[2] < start[2] {
                    self.down
                } else {
                    self.up
                },
                ..MovementTrace::miss(end)
            }
        }
    }
    fn predictor(timer: i32) -> Predictor {
        let mut predictor = Predictor::from_state(
            MovementState {
                legs_anim: BOTH_JUMPATTACK6,
                legs_timer: timer,
                torso_timer: timer,
                standing_height: 40.0,
                ..MovementState::default()
            },
            MovementConfig::default(),
        );
        predictor.set_animation_lengths(Arc::new(AnimationLengthTable::new(vec![
            AnimationTiming {
                frame_count: 200,
                frame_lerp_ms: 10
            };
            BOTH_JUMPATTACK6
                as usize
                + 1
        ])));
        predictor
    }
    #[test]
    fn dual_jump_hull_follows_both_authored_windows() {
        assert_eq!(
            crate::legacy_animation::NAMES[BOTH_JUMPATTACK6 as usize],
            "BOTH_JUMPATTACK6"
        );
        for step in [8, 7, 4, 3] {
            for timer in [
                1_600 + step,
                1_600,
                1_450,
                1_450 - step,
                900 + step,
                900,
                400,
                400 - step,
            ] {
                let mut predictor = predictor(timer);
                let reduced = (1_450..=1_600).contains(&timer) || (400..=900).contains(&timer);
                assert_eq!(
                    predictor.dual_jump_minimum(&Space { down: 1.0, up: 1.0 }),
                    if reduced { 0.0 } else { -24.0 },
                    "timer {timer}"
                );
                assert_eq!(
                    predictor.state().movement_flags & PMF_FIX_MINS != 0,
                    reduced
                );
            }
        }
    }
    #[test]
    fn hull_restoration_moves_up_or_crouches_under_a_ceiling() {
        for step in [8, 7, 4, 3] {
            for up in [1.0, 0.5] {
                let mut predictor = predictor(200 - step);
                predictor.state_mut().movement_flags |= PMF_FIX_MINS;
                assert_eq!(
                    predictor.dual_jump_minimum(&Space { down: 0.25, up }),
                    -24.0
                );
                assert_eq!(
                    predictor.state().origin[2],
                    if up == 1.0 { 18.0 } else { 24.0 }
                );
                assert_eq!(predictor.state().movement_flags & PMF_FIX_MINS, 0);
                assert_eq!(predictor.state().movement_flags & PMF_DUCKED != 0, up < 1.0);
                if up < 1.0 {
                    assert_eq!(
                        (predictor.state().legs_timer, predictor.state().torso_timer),
                        (0, 0)
                    );
                }
            }
        }
    }
}
