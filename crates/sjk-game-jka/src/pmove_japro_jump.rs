//! jaPRO jump policy from TaystJK `2d60c0fa5` multiplayer PM_CheckJump,
//! PM_ForceJumpingUp and PM_DropTimers. Style ordinals follow bg_public.h.
use super::*;

#[derive(Clone, Copy)]
pub(super) struct Rules {
    velocity: f32,
    ramp: f32,
    pub(super) held_jump: bool,
    pub(super) force_rise: bool,
    pub(super) force_special: bool,
    pub(super) ramp_curve: bool,
    timer: bool,
    last_speed: bool,
    pjk_delay: bool,
}

const REGULAR: Rules = Rules {
    velocity: 225.0,
    ramp: 0.0,
    held_jump: false,
    force_rise: true,
    force_special: true,
    ramp_curve: false,
    timer: false,
    last_speed: false,
    pjk_delay: false,
};
const QW: Rules = Rules {
    ramp: 0.75,
    held_jump: true,
    ramp_curve: true,
    timer: true,
    last_speed: true,
    ..REGULAR
};
const ARENA: Rules = Rules {
    velocity: 270.0,
    ramp: 1.25,
    held_jump: true,
    force_rise: false,
    force_special: false,
    timer: true,
    last_speed: true,
    ..REGULAR
};

// One row per advertised movementStyle_e. OCPM deliberately needs jump release;
// Jetpack permits held jumps but not a Force rise, as in the source's two lists.
const JUMPS: [Rules; 19] = [
    REGULAR, // Siege
    REGULAR, // JKA
    QW,
    ARENA, // CPM
    ARENA, // Q3
    Rules {
        pjk_delay: true,
        ..QW
    },
    Rules {
        velocity: 280.0,
        ramp: 0.75,
        ..ARENA
    }, // Warsow
    ARENA,   // RJQ3
    ARENA,   // RJCPM
    REGULAR, // Swoop
    Rules {
        held_jump: true,
        force_rise: false,
        ..REGULAR
    }, // Jetpack
    REGULAR, // Speed
    REGULAR, // SP
    ARENA,   // Slick
    ARENA,   // BotCPM
    REGULAR, // CoopJKA
    Rules {
        held_jump: false,
        ..ARENA
    }, // OCPM
    Rules {
        velocity: 275.0,
        force_rise: false,
        timer: true,
        ..REGULAR
    }, // Tribes
    Rules {
        velocity: 270.0,
        force_rise: false,
        ..REGULAR
    }, // Surf
];

impl Predictor {
    pub(super) fn japro_jump_rules(&self) -> Option<Rules> {
        self.japro_style().map(|rules| JUMPS[rules.style as usize])
    }

    /// Input, PJK delay and held-jump gates, in reference order.
    pub(super) fn japro_jump_blocked(&self, command: &mut UserCommand) -> bool {
        let Some(rules) = self.japro_jump_rules() else {
            return false;
        };
        if command.up_move < 10 && self.state.ground_entity_number != ENTITY_NUMBER_NONE {
            return true;
        }
        if rules.pjk_delay && self.state.ja_pro.jump_time > 375 {
            return true;
        }
        if !rules.held_jump && self.state.movement_flags & PMF_JUMP_HELD != 0 {
            command.up_move = 0;
            return true;
        }
        false
    }

    /// Replace the ordinary takeoff impulse, without changing the common jump
    /// event, ground state or animation path. Metadata is updated before superjump.
    pub(super) fn japro_jump_takeoff(&mut self, ground: &GroundState) -> bool {
        let Some(rules) = self.japro_jump_rules() else {
            return false;
        };
        let mut velocity = rules.velocity;
        if rules.ramp != 0.0 {
            let horizontal = Vec3::new(self.state.velocity[0], self.state.velocity[1], 0.0);
            let speed = horizontal.length();
            let added =
                (-horizontal.dot(Vec3::from_array(ground.trace.plane_normal))).min(speed * 0.5);
            if added > 0.0 {
                velocity += added * rules.ramp;
            } else if self.state.ja_pro.jump_time > 0 {
                velocity += 100.0;
                self.state.movement_flags &= !PMF_JUMP_HELD;
            }
        }
        if rules.timer {
            self.state.ja_pro.jump_time = 401;
        }
        if rules.last_speed {
            self.state.ja_pro.last_jump_speed = velocity as i32;
        }
        if self.state.ja_pro.restrictions & crate::pmove_japro::RESTRICT_SUPERJUMP != 0 {
            velocity *= 3.0;
        }
        self.state.velocity[2] = velocity;
        true
    }
}

#[cfg(test)]
#[path = "pmove_japro_jump_tests.rs"]
mod tests;
