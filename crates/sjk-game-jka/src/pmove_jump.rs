//! `PM_CheckJump` (`codemp/game/bg_pmove.c:1757-2753`) whole, as `PM_WalkMove` and
//! `PM_AirMove` both call it: the gates, the Force jump's upkeep and its held rise
//! ([`super::force_jump`]), the special jumps off walls
//! ([`super::wall_moves`]) and the ordinary jump.
//!
//! Jumping observes the respawn latch before Force-jump upkeep. The
//! zero-gravity branch preserves codemp's `fraction <= 1` test, including
//! its impulse on a clear backward trace.

use super::wall_moves::JumpRules;
#[cfg(test)]
mod respawn_tests {
    use super::*;
    use crate::pmove::{MovementConfig, MovementState, MovementTrace};
    struct Floor;
    impl MovementCollision for Floor {
        fn trace(
            &self,
            start: [f32; 3],
            _: [f32; 3],
            _: [f32; 3],
            end: [f32; 3],
            _: u32,
        ) -> MovementTrace {
            if end[2] < start[2] && start[2] <= 24.0 {
                MovementTrace {
                    fraction: 0.0,
                    end_position: start,
                    plane_normal: [0.0, 0.0, 1.0],
                    entity_number: super::super::ENTITY_NUMBER_WORLD,
                    ..MovementTrace::miss(end)
                }
            } else {
                MovementTrace::miss(end)
            }
        }
    }
    #[test]
    fn respawn_jump_waits_for_attack_and_use_release() {
        for step in [8, 7, 4, 3] {
            for button in [1, 4] {
                let mut predictor = Predictor::from_state(
                    MovementState {
                        health: 100,
                        gravity: 800.0,
                        origin: [0.0, 0.0, 24.0],
                        standing_height: 40.0,
                        crouching_height: 16.0,
                        ground_entity_number: super::super::ENTITY_NUMBER_WORLD,
                        movement_flags: PMF_RESPAWNED,
                        ..MovementState::default()
                    },
                    MovementConfig::default(),
                );
                predictor.predict_command(
                    UserCommand {
                        server_time: step,
                        up_move: 127,
                        buttons: button,
                        ..UserCommand::default()
                    },
                    &Floor,
                );
                assert_eq!(
                    predictor.state().movement_flags & PMF_RESPAWNED,
                    PMF_RESPAWNED
                );
                assert_eq!(predictor.state().velocity[2], 0.0);
                predictor.predict_command(
                    UserCommand {
                        server_time: step * 2,
                        up_move: 127,
                        ..UserCommand::default()
                    },
                    &Floor,
                );
                assert_eq!(predictor.state().movement_flags & PMF_RESPAWNED, 0);
                assert!(predictor.state().velocity[2] > 0.0);
            }
        }
    }
}
/// `PMF_RESPAWNED` (`codemp/game/bg_public.h`).
const PMF_RESPAWNED: u16 = 512;
use super::{
    Bounds, GroundState, JUMP_VELOCITY, MAX_CLIENTS, MoveContext, MovementCollision, PMF_JUMP_HELD,
    Predictor,
};
use sjk_protocol::{ENTITY_NUMBER_NONE, UserCommand};

/// `HANDEXTEND_KNOCKDOWN`, `HANDEXTEND_PRETHROWN`, `HANDEXTEND_POSTTHROWN`
/// (`bg_public.h:200-216`).
const HANDEXTEND_KNOCKDOWN: u8 = 8;
const HANDEXTEND_PRETHROWN: u8 = 13;
const HANDEXTEND_POSTTHROWN: u8 = 14;
/// `PM_JETPACK`, `CLASS_VEHICLE`, `EF2_FLYING`, `WP_SABER`, `WP_MELEE`.
const PM_JETPACK: u8 = 1;
const CLASS_VEHICLE: i32 = 53;
const EF2_FLYING: u32 = 1 << 4;
const WP_SABER: u8 = 3;
const WP_MELEE: u8 = 2;
/// `EV_JUMP`.
const EV_JUMP: u16 = 16;

impl Predictor {
    /// `PM_SetForceJumpZStart` (`bg_pmove.c:1728-1735`): where a jump began, nudged off
    /// zero so that a jump from height zero still counts as one.
    pub(super) fn set_force_jump_start(&mut self, height: f32) {
        self.state.force_jump_start_height = height;
        if self.state.force_jump_start_height == 0.0 {
            self.state.force_jump_start_height -= 0.1;
        }
    }

    /// `PM_CheckJump`: whether the player jumped off the ground. It may change the
    /// command, as the reference changes `pm->cmd`: a jump spent or refused clears its
    /// `upmove`, a run up a wall its stick.
    pub(super) fn check_jump(
        &mut self,
        command: &mut UserCommand,
        bounds: Bounds,
        ground: &mut GroundState,
        collision: &impl MovementCollision,
        context: &MoveContext,
    ) -> bool {
        let state = &self.state;
        // A flying NPC (`FLY_NORMAL`) moves by `PM_FlyMove` and never reaches
        // `PM_CheckJump`; a vehicle does not jump either.
        if state.client_num >= MAX_CLIENTS
            && self
                .npc
                .is_some_and(|npc| npc.class == CLASS_VEHICLE || npc.flags2 & EF2_FLYING != 0)
            || matches!(
                state.force_hand_extend,
                HANDEXTEND_KNOCKDOWN | HANDEXTEND_PRETHROWN | HANDEXTEND_POSTTHROWN
            )
            || state.movement_type == PM_JETPACK
            || self.new_jetpack_active()
        {
            return false;
        }
        // `PM_CheckJump`, before Force-jump upkeep: attack/use must first release
        // PmoveSingle's respawn latch, even if jump itself has already been released.
        if state.movement_flags & PMF_RESPAWNED != 0 {
            return false;
        }
        // Knocked down, getting up or rolling, there is no jumping (`bg_pmove.c:1790-1793`).
        if crate::pmove_hand_extend::in_knockdown(state.legs_anim, state.legs_timer_at_entry)
            || crate::pmove_roll_anim::in_roll(state.legs_anim) && state.legs_timer_at_entry > 0
        {
            return false;
        }
        let mut rules = JumpRules::of(state, context);
        if self.config.ja_pro.enabled
            && state.ja_pro.restrictions & crate::pmove_japro::RESTRICT_BHOP != 0
        {
            rules = rules.without_wall_moves();
        }
        let ja_pro_jump = self.japro_jump_rules();
        self.force_jump_upkeep(command.server_time);
        if self.state.force_jump_flip {
            self.force_jump_flip(command, rules.flips);
            return true;
        }
        if self.check_force_jump(command, rules.flips)
            && !ja_pro_jump.is_some_and(|rules| rules.held_jump)
        {
            command.up_move = 0;
            return false;
        }
        let grounded =
            |predictor: &Self| predictor.state.ground_entity_number != ENTITY_NUMBER_NONE;
        if ja_pro_jump.is_none() && command.up_move < 10 && grounded(self) {
            return false;
        }
        // Must wait for the jump to be released.
        if ja_pro_jump.is_some() {
            if self.japro_jump_blocked(command) {
                return false;
            }
        } else if self.state.movement_flags & PMF_JUMP_HELD != 0 {
            command.up_move = 0;
            return false;
        }
        // PM_CheckJump, codemp/bg_pmove.c:2070-2085. Preserve the reference's
        // <= 1 test: even a clear trace adds the impulse, once per eligible slice.
        if self.state.gravity <= 0.0 {
            let (forward, _) = super::flight::flight_axes(self.state.view_angles);
            let back = (glam::Vec3::from_array(self.state.origin) - forward * 8.0).to_array();
            let trace = collision.trace(
                self.state.origin,
                bounds.minimums,
                bounds.maximums,
                back,
                super::PLAYER_CONTENT_MASK,
            );
            if trace.fraction <= 1.0 {
                self.state.velocity = (glam::Vec3::from_array(self.state.velocity)
                    + forward * (JUMP_VELOCITY * 2.0))
                    .to_array();
                if let Some(lengths) = self.animation_lengths.as_deref() {
                    crate::pmove_anim::set_animation(
                        &mut self.state,
                        crate::pmove_anim::SETANIM_LEGS,
                        super::wall_moves::BOTH_FORCEJUMP1,
                        crate::pmove_anim::SETANIM_FLAG_OVERRIDE
                            | crate::pmove_anim::SETANIM_FLAG_HOLD
                            | crate::pmove_anim::SETANIM_FLAG_RESTART,
                        lengths,
                    );
                }
            }
            command.up_move = 0;
        } else if command.up_move > 0
            && ja_pro_jump.is_none_or(|rules| {
                rules.force_special && self.state.movement_flags & PMF_JUMP_HELD == 0
            })
            && self.state.water_level < 2
            && self.state.levitation_level > 0
            && matches!(self.state.weapon, WP_SABER | WP_MELEE)
            && crate::pmove_saber_attack::can_levitate_now(
                &self.state,
                command.server_time,
                self.gametype,
            )
        {
            if grounded(self) {
                self.ground_special_jump(command, bounds, collision, rules);
            } else {
                self.air_special_jump(command, bounds, collision, context, rules);
            }
        }
        if !grounded(self) {
            return false;
        }
        if command.up_move > 0 {
            // No special jump.
            if !self.japro_jump_takeoff(ground) {
                self.state.velocity[2] = JUMP_VELOCITY;
            }
            self.set_force_jump_start(self.state.origin[2]);
            self.state.movement_flags |= PMF_JUMP_HELD;
        }
        ground.ground_plane = false;
        ground.walking = false;
        self.state.movement_flags |= PMF_JUMP_HELD;
        self.state.ground_entity_number = ENTITY_NUMBER_NONE;
        self.set_force_jump_start(self.state.origin[2]);
        self.add_event(EV_JUMP, 0); // bg_pmove.c:2739.
        if self.state.gravity > 0.0
            && !crate::pmove_roll_anim::special_jump(self.state.legs_anim)
            && let Some(lengths) = self.animation_lengths.as_deref()
        {
            crate::pmove_locomotion::jump_for_direction(&mut self.state, command, lengths);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pmove::{MovementState, MovementTrace};
    use crate::pmove_anim::{AnimationLengths, AnimationTiming};
    use std::cell::Cell;

    struct BackTrace {
        fraction: f32,
        calls: Cell<usize>,
    }
    impl MovementCollision for BackTrace {
        fn trace(
            &self,
            start: [f32; 3],
            mins: [f32; 3],
            maxs: [f32; 3],
            end: [f32; 3],
            _: u32,
        ) -> MovementTrace {
            self.calls.set(self.calls.get() + 1);
            assert_eq!(mins, [-15.0, -15.0, -24.0]);
            assert_eq!(maxs, [15.0, 15.0, 40.0]);
            assert!((glam::Vec3::from_array(end).distance(start.into()) - 8.0).abs() < 0.0001);
            MovementTrace {
                fraction: self.fraction,
                ..MovementTrace::miss(end)
            }
        }
    }
    #[derive(Debug)]
    struct Timings;
    impl AnimationLengths for Timings {
        fn length_ms(&self, _: u16) -> Option<i32> {
            Some(500)
        }

        fn timing(&self, _: u16) -> Option<AnimationTiming> {
            Some(AnimationTiming {
                frame_count: 10,
                frame_lerp_ms: 50,
            })
        }
    }

    #[test]
    fn zero_gravity_push_preserves_the_clear_trace_quirk_and_jump_gates() {
        // OpenJK bg_pmove.c:2070-2085: +450 along the aim, even fraction == 1.
        for step in [8, 7, 4, 3] {
            for gravity in [0.0, -100.0] {
                for fraction in [0.25, 1.0] {
                    for (grounded, up, held, pushes) in [
                        (false, 0, false, true),
                        (false, 127, false, true),
                        (false, 127, true, false),
                        (true, 0, false, false),
                        (true, 127, false, true),
                    ] {
                        let mut predictor = Predictor::from_state(
                            MovementState {
                                health: 100,
                                gravity,
                                command_time: step,
                                view_angles: [-30.0, 90.0, 0.0],
                                velocity: [10.0, 20.0, 30.0],
                                ground_entity_number: if grounded {
                                    1022
                                } else {
                                    ENTITY_NUMBER_NONE
                                },
                                movement_flags: if held { PMF_JUMP_HELD } else { 0 },
                                ..Default::default()
                            },
                            Default::default(),
                        );
                        predictor.set_animation_lengths(std::sync::Arc::new(Timings));
                        let world = BackTrace {
                            fraction,
                            calls: Cell::new(0),
                        };
                        let mut command = UserCommand {
                            server_time: step,
                            up_move: up,
                            ..Default::default()
                        };
                        let mut ground = GroundState::air(MovementTrace::miss([0.0; 3]));
                        predictor.check_jump(
                            &mut command,
                            Bounds {
                                minimums: [-15.0, -15.0, -24.0],
                                maximums: [15.0, 15.0, 40.0],
                            },
                            &mut ground,
                            &world,
                            &MoveContext::CLIENT,
                        );
                        assert_eq!(world.calls.get(), usize::from(pushes));
                        let expected = if pushes {
                            let (forward, _) =
                                super::super::flight::flight_axes([-30.0, 90.0, 0.0]);
                            (glam::Vec3::new(10.0, 20.0, 30.0) + forward * 450.0).to_array()
                        } else {
                            [10.0, 20.0, 30.0]
                        };
                        assert_eq!(predictor.state().velocity, expected);
                        if pushes {
                            assert_eq!(command.up_move, 0);
                            assert_eq!(
                                predictor.state().legs_anim,
                                super::super::wall_moves::BOTH_FORCEJUMP1
                            );
                            assert!(predictor.state().legs_timer > 0);
                        }
                    }
                }
            }
        }
    }
    struct Empty;
    impl MovementCollision for Empty {
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
    fn airborne_zero_gravity_push_runs_on_each_unlatched_command() {
        for step in [8, 7, 4, 3] {
            let mut predictor = Predictor::from_state(
                MovementState {
                    health: 100,
                    gravity: 0.0,
                    speed: 250.0,
                    weapon: 2,
                    origin: [0.0, 0.0, 100.0],
                    standing_height: 40.0,
                    crouching_height: 16.0,
                    ground_entity_number: ENTITY_NUMBER_NONE,
                    ..Default::default()
                },
                Default::default(),
            );
            for n in 1..=2 {
                predictor.predict_command(
                    UserCommand {
                        server_time: n * step,
                        ..Default::default()
                    },
                    &Empty,
                );
                assert_eq!(predictor.state().velocity, [450.0 * n as f32, 0.0, 0.0]);
            }
            assert!(
                (predictor.state().origin[0] - 1350.0 * super::super::frame_seconds(step)).abs()
                    < 0.0001
            );
        }
    }
}
