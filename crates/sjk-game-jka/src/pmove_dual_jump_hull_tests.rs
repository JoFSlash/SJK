//! Regression for the fixed standing hull during dual forward jump attacks.
use super::*;
use crate::pmove_anim::{AnimationLengthTable, AnimationTiming};
use std::sync::Arc;

struct Floor;
impl MovementCollision for Floor {
    fn trace(
        &self,
        start: [f32; 3],
        mins: [f32; 3],
        _: [f32; 3],
        end: [f32; 3],
        _: u32,
    ) -> MovementTrace {
        let a = start[2] + mins[2] - 0.125;
        let b = end[2] + mins[2] - 0.125;
        if b < a && b < 0.0 {
            let fraction = (a / (a - b)).max(0.0);
            MovementTrace {
                fraction,
                end_position: std::array::from_fn(|i| start[i] + (end[i] - start[i]) * fraction),
                plane_normal: [0.0, 0.0, 1.0],
                entity_number: ENTITY_NUMBER_WORLD,
                ..MovementTrace::miss(end)
            }
        } else {
            MovementTrace::miss(end)
        }
    }
}

#[test]
fn dual_forward_jump_landing_keeps_feet_on_floor() {
    for step in [8, 7, 4, 3] {
        for timer in [500, 600, 1500] {
            let mut predictor = Predictor::from_state(
                MovementState {
                    health: 100,
                    weapon: 3,
                    legs_anim: 861,
                    legs_timer: timer,
                    torso_timer: timer,
                    origin: [0.0, 0.0, 24.125],
                    velocity: [0.0, 0.0, -254.0],
                    gravity: 800.0,
                    standing_height: 40.0,
                    crouching_height: 16.0,
                    ground_entity_number: ENTITY_NUMBER_NONE,
                    ..MovementState::default()
                },
                MovementConfig::default(),
            );
            // Synthetic lengths, not retail animation data.
            predictor.set_animation_lengths(Arc::new(AnimationLengthTable::new(vec![
                AnimationTiming { frame_count: 200, frame_lerp_ms: 10 }; 862
            ])));
            predictor.predict_command(
                UserCommand {
                    server_time: step,
                    weapon: 3,
                    ..UserCommand::default()
                },
                &Floor,
            );
            assert_eq!(
                predictor.state().origin[2],
                24.125,
                "{step} ms, timer {timer}"
            );
            assert_eq!(
                predictor.state().velocity[2],
                0.0,
                "{step} ms, timer {timer}"
            );
            assert_eq!(predictor.state().ground_entity_number, ENTITY_NUMBER_WORLD);
        }
    }
}
