//! Command-driven `PM_Animate`, OpenJK codemp/game/bg_pmove.c:7707-7731.

use crate::pmove::MovementState;
use crate::predicted_events::PredictedEvents;
use sjk_protocol::UserCommand;

const BUTTON_GESTURE: u16 = 8;
const HANDEXTEND_NONE: u8 = 0;
const HANDEXTEND_TAUNT: u8 = 10;
const BOTH_ENGAGETAUNT: u16 = 1_187;
const EV_TAUNT: u16 = 115;

pub(super) fn animate(
    state: &mut MovementState,
    command: &UserCommand,
    events: &mut PredictedEvents,
) {
    if command.buttons & BUTTON_GESTURE == 0 {
        return;
    }
    if state.vehicle_entity_num != 0 && state.force_hand_extend_time < command.server_time {
        state.force_hand_extend = HANDEXTEND_NONE;
    }
    if state.torso_timer < 1
        && state.force_hand_extend == HANDEXTEND_NONE
        && state.legs_timer < 1
        && state.weapon_time < 1
        && state.saber_lock_time < command.server_time
    {
        state.force_hand_extend = HANDEXTEND_TAUNT;
        state.force_dodge_anim = BOTH_ENGAGETAUNT;
        state.force_hand_extend_time = command.server_time + 1_000;
        crate::pmove_weapon_charge::event(state, events, EV_TAUNT, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pmove::{MovementCollision, MovementConfig, MovementTrace, Predictor};
    struct Void;
    impl MovementCollision for Void {
        fn trace(
            &self,
            _: [f32; 3],
            _: [f32; 3],
            _: [f32; 3],
            end: [f32; 3],
            _: u32,
        ) -> MovementTrace {
            MovementTrace::miss(end)
        }
    }

    #[test]
    fn gesture_predicts_the_taunt_event_before_weapon_animation() {
        for step in [8, 7, 4, 3] {
            let mut predictor = Predictor::from_state(
                MovementState {
                    health: 100,
                    gravity: 800.0,
                    weapon: 5,
                    weapons: 1 << 5,
                    standing_height: 40.0,
                    ..MovementState::default()
                },
                MovementConfig::default(),
            );
            predictor.predict_command(
                UserCommand {
                    server_time: step,
                    weapon: 5,
                    buttons: BUTTON_GESTURE,
                    ..UserCommand::default()
                },
                &Void,
            );
            assert_eq!(predictor.state().force_hand_extend, HANDEXTEND_TAUNT);
            assert_eq!(predictor.state().force_dodge_anim, BOTH_ENGAGETAUNT);
            assert_eq!(predictor.state().force_hand_extend_time, step + 1_000);
            let mut taunts = 0;
            predictor.emit_events(|event| {
                if event.event == EV_TAUNT {
                    taunts += 1;
                }
            });
            assert_eq!(taunts, 1);
            predictor.predict_command(
                UserCommand {
                    server_time: step * 2,
                    weapon: 5,
                    buttons: BUTTON_GESTURE,
                    ..UserCommand::default()
                },
                &Void,
            );
            predictor.emit_events(|event| {
                if event.event == EV_TAUNT {
                    taunts += 1;
                }
            });
            assert_eq!(taunts, 1);
        }
    }

    #[test]
    fn busy_tracks_and_saber_lock_block_gestures() {
        for mut state in [
            MovementState {
                torso_timer: 1,
                ..MovementState::default()
            },
            MovementState {
                legs_timer: 1,
                ..MovementState::default()
            },
            MovementState {
                weapon_time: 1,
                ..MovementState::default()
            },
            MovementState {
                force_hand_extend: 1,
                ..MovementState::default()
            },
            MovementState {
                saber_lock_time: 100,
                ..MovementState::default()
            },
        ] {
            animate(
                &mut state,
                &UserCommand {
                    server_time: 100,
                    buttons: BUTTON_GESTURE,
                    ..UserCommand::default()
                },
                &mut PredictedEvents::default(),
            );
            assert_eq!(state.event_sequence, 0);
        }
    }
}
