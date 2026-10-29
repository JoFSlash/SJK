//! jaPRO's new jetpack, separate from PM_JETPACK: TaystJK `2d60c0fa5`
//! codemp BG_CanJetpack, PmoveSingle, PM_AirMove and PM_GroundTraceMissed.
use super::japro_movement::{BUTTON_DASH, Style};
use super::*;

pub(super) const ACTIVE: u32 = 1 << 11;
const FLAMING: u32 = 1 << 30;
const OWNED_JETPACK: u32 = 1 << 7; // HI_JETPACK, bg_public.h
const BOTH_FORCEJUMP1: u16 = 1151;
const BOTH_FORCEINAIR1: u16 = 1152;
const BOTH_FORCEJUMPBACK1: u16 = 1154;
const BOTH_FORCEJUMPLEFT1: u16 = 1157;
const BOTH_FORCEJUMPRIGHT1: u16 = 1160;

impl Predictor {
    pub(super) fn new_jetpack_active(&self) -> bool {
        self.config.ja_pro.enabled
            && self.vehicle.is_none()
            && self.state.entity_flags & ACTIVE != 0
    }

    pub(super) fn can_start_new_jetpack(&self) -> bool {
        if !self.config.ja_pro.enabled || self.vehicle.is_some() {
            return false;
        }
        let state = &self.state;
        let tribes = state.ja_pro.style == Style::Tribes as u32;
        state.holdable_items & OWNED_JETPACK != 0
            && if tribes {
                state.force_power >= 10
            } else {
                state.ja_pro.jetpack_fuel >= 10
            }
            && !crate::saber_rules::in_special(state.saber_move)
            && !crate::pmove_roll::in_roll(state)
            && !crate::saber_rules::special_jump(state.legs_anim)
            && !crate::pmove_hand_extend::knockdown_animation(state.legs_anim)
            && state.water_level <= 1
    }

    /// Return the reference's gDist scratch value for the thrust stage. Fuel is
    /// authoritative snapshot state; cgame predicts the switch, not resource drain.
    pub(super) fn activate_new_jetpack(
        &mut self,
        command: &UserCommand,
        bounds: Bounds,
        collision: &impl MovementCollision,
    ) -> f32 {
        if !self.config.ja_pro.enabled
            || self.vehicle.is_some()
            || !(self.config.ja_pro.jetpack || self.state.ja_pro.race)
        {
            return 0.0;
        }
        let tribes = self.state.ja_pro.style == Style::Tribes as u32;
        let empty = if tribes {
            self.state.force_power == 0
        } else {
            self.state.ja_pro.jetpack_fuel == 0
        };
        let mut distance = 0.0;
        if command.up_move == 0 || empty {
            self.state.entity_flags &= !ACTIVE;
        } else if self.state.movement_type == 0
            && self.state.ground_entity_number == ENTITY_NUMBER_NONE
            && (tribes || self.state.movement_flags & PMF_JUMP_HELD == 0)
            && self.can_start_new_jetpack()
        {
            distance = self.ground_distance(bounds, collision);
            let allowed = if tribes {
                self.state.ja_pro.jump_time < 300 || distance > 64.0
            } else {
                self.state.velocity[2] > 0.0 || distance > 64.0
            };
            if allowed {
                self.start_new_jetpack();
            }
        } else if tribes
            && command.up_move > 0
            && command.buttons & BUTTON_DASH != 0
            && self.can_start_new_jetpack()
        {
            self.start_new_jetpack();
        }
        distance
    }

    fn start_new_jetpack(&mut self) {
        if self.state.entity_flags & ACTIVE == 0 {
            self.state.ja_pro.jump_time = 500;
            self.state.entity_flags |= ACTIVE;
        }
    }

    pub(super) fn new_jetpack_thrust(
        &mut self,
        command: &UserCommand,
        bounds: Bounds,
        collision: &impl MovementCollision,
        seconds: f32,
        mut distance: f32,
    ) {
        if !self.new_jetpack_active() {
            return;
        }
        let scale = self.command_scale(command) / self.state.speed * 320.0;
        let velocity = &mut self.state.velocity[2];
        if command.up_move > 0 && *velocity < 2000.0 {
            let mut up = scale;
            if !self.state.ja_pro.race {
                up *= self.state.gravity / 800.0;
            }
            let boost = if *velocity > 0.0 && *velocity < 250.0 {
                (250.0 / *velocity).clamp(1.0, 1.4)
            } else if *velocity < 0.0 {
                (*velocity / -600.0).clamp(1.4, 4.0)
            } else if *velocity > 800.0 {
                (800.0 / *velocity).max(0.5)
            } else {
                1.0
            };
            *velocity += 425.0 * seconds * up * boost;
            self.state.entity_flags |= FLAMING;
        } else if command.up_move < 0 && *velocity > -4000.0 {
            distance = self.ground_distance(bounds, collision);
            if distance >= 16.0 {
                self.state.velocity[2] -= 150.0 * seconds * scale;
                self.state.entity_flags |= FLAMING;
            }
        }
        if self.state.ja_pro.style == Style::Tribes as u32
            && (command.forward_move != 0 || command.right_move != 0)
        {
            let (forward, right) = flight::flight_axes(self.move_view);
            let input =
                forward * f32::from(command.forward_move) + right * f32::from(command.right_move);
            let direction = vector_normalize(Vec3::new(input.x, input.y, 0.0));
            let velocity = Vec3::from_array(self.state.velocity);
            let strength = if velocity.dot(direction) < 0.0 {
                4.0
            } else {
                800.0 / velocity.length()
            };
            self.state.velocity =
                (velocity + direction * (strength.clamp(1.2, 4.0) * seconds * 100.0)).to_array();
        }
        if distance < 16.0 && command.up_move < 0 {
            self.state.entity_flags &= !ACTIVE;
        }
    }

    /// The forward/backward selector overrides strafe, unlike the old pack's pose.
    pub(super) fn new_jetpack_pose(&mut self, command: &UserCommand) {
        if !self.new_jetpack_active() {
            return;
        }
        let animation = if command.forward_move < 0 {
            BOTH_FORCEJUMPBACK1
        } else if command.forward_move > 0 {
            BOTH_FORCEJUMP1
        } else if command.right_move > 0 {
            BOTH_FORCEJUMPRIGHT1
        } else if command.right_move < 0 {
            BOTH_FORCEJUMPLEFT1
        } else {
            BOTH_FORCEINAIR1
        };
        if let Some(lengths) = self.animation_lengths.as_deref() {
            crate::pmove_anim::set_animation(
                &mut self.state,
                crate::pmove_anim::SETANIM_LEGS,
                animation,
                crate::pmove_anim::SETANIM_FLAG_OVERRIDE,
                lengths,
            );
        }
    }
}

#[cfg(test)]
#[path = "pmove_japro_jetpack_tests.rs"]
mod tests;
