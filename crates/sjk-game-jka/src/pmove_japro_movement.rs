//! jaPRO on-foot style kernels: TaystJK `2d60c0fa5` multiplayer
//! `bg_pmove.c` PM_Accelerate/AirMove/WalkMove/Friction/CmdScale.
use super::*;

pub(super) const BUTTON_DASH: u16 = 8192;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Style {
    Siege,
    Jka,
    Qw,
    Cpm,
    Q3,
    Pjk,
    Wsw,
    RjQ3,
    RjCpm,
    Swoop,
    Jetpack,
    Speed,
    Sp,
    Slick,
    BotCpm,
    CoopJka,
    Ocpm,
    Tribes,
    Surf,
}

#[derive(Clone, Copy)]
enum Air {
    Standard,
    Qw,
    Cpm,
    Slick,
    Sp,
    Tribes,
    Surf,
}

#[derive(Clone, Copy)]
pub(super) struct Rules {
    pub(super) style: Style,
    pub(super) ground_acceleration: f32,
    pub(super) friction: f32,
    pub(super) duck: f32,
    pub(super) scale_upmove: bool,
    air: Air,
}

// One row per bg_public.h movementStyle_e ordinal. In particular OCPM uses
// stock friction (6), while CPM uses 8: preserve the reference's explicit list.
const STYLES: [Rules; 19] = [
    Rules {
        style: Style::Siege,
        ground_acceleration: 10.0,
        friction: 6.0,
        duck: 0.5,
        scale_upmove: false,
        air: Air::Standard,
    },
    Rules {
        style: Style::Jka,
        ground_acceleration: 10.0,
        friction: 6.0,
        duck: 0.5,
        scale_upmove: false,
        air: Air::Standard,
    },
    Rules {
        style: Style::Qw,
        ground_acceleration: 10.0,
        friction: 6.0,
        duck: 0.5,
        scale_upmove: false,
        air: Air::Qw,
    },
    Rules {
        style: Style::Cpm,
        ground_acceleration: 15.0,
        friction: 8.0,
        duck: 0.5,
        scale_upmove: false,
        air: Air::Cpm,
    },
    Rules {
        style: Style::Q3,
        ground_acceleration: 10.0,
        friction: 6.0,
        duck: 0.25,
        scale_upmove: false,
        air: Air::Standard,
    },
    Rules {
        style: Style::Pjk,
        ground_acceleration: 10.0,
        friction: 6.0,
        duck: 0.5,
        scale_upmove: false,
        air: Air::Cpm,
    },
    Rules {
        style: Style::Wsw,
        ground_acceleration: 12.0,
        friction: 8.0,
        duck: 0.3125,
        scale_upmove: false,
        air: Air::Cpm,
    },
    Rules {
        style: Style::RjQ3,
        ground_acceleration: 10.0,
        friction: 6.0,
        duck: 0.25,
        scale_upmove: false,
        air: Air::Standard,
    },
    Rules {
        style: Style::RjCpm,
        ground_acceleration: 15.0,
        friction: 8.0,
        duck: 0.5,
        scale_upmove: false,
        air: Air::Cpm,
    },
    Rules {
        style: Style::Swoop,
        ground_acceleration: 10.0,
        friction: 6.0,
        duck: 0.5,
        scale_upmove: false,
        air: Air::Standard,
    },
    Rules {
        style: Style::Jetpack,
        ground_acceleration: 10.0,
        friction: 6.0,
        duck: 0.5,
        scale_upmove: true,
        air: Air::Standard,
    },
    Rules {
        style: Style::Speed,
        ground_acceleration: 10.0,
        friction: 6.0,
        duck: 0.5,
        scale_upmove: false,
        air: Air::Standard,
    },
    Rules {
        style: Style::Sp,
        ground_acceleration: 12.0,
        friction: 6.0,
        duck: 0.5,
        scale_upmove: true,
        air: Air::Sp,
    },
    Rules {
        style: Style::Slick,
        ground_acceleration: 30.0,
        friction: 8.0,
        duck: 0.5,
        scale_upmove: false,
        air: Air::Slick,
    },
    Rules {
        style: Style::BotCpm,
        ground_acceleration: 15.0,
        friction: 8.0,
        duck: 0.5,
        scale_upmove: false,
        air: Air::Cpm,
    },
    Rules {
        style: Style::CoopJka,
        ground_acceleration: 10.0,
        friction: 6.0,
        duck: 0.5,
        scale_upmove: false,
        air: Air::Standard,
    },
    Rules {
        style: Style::Ocpm,
        ground_acceleration: 15.0,
        friction: 6.0,
        duck: 0.5,
        scale_upmove: true,
        air: Air::Cpm,
    },
    Rules {
        style: Style::Tribes,
        ground_acceleration: 10.0,
        friction: 6.0,
        duck: 0.5,
        scale_upmove: true,
        air: Air::Tribes,
    },
    Rules {
        style: Style::Surf,
        ground_acceleration: 12.0,
        friction: 6.0,
        duck: 0.5,
        scale_upmove: false,
        air: Air::Surf,
    },
];

impl Rules {
    pub(super) fn skiing(self, command: &UserCommand) -> bool {
        self.style == Style::Tribes && command.buttons & BUTTON_DASH != 0
    }

    pub(super) fn ground_friction(self, command: &UserCommand) -> f32 {
        if self.style == Style::Slick && command.buttons & 16 == 0 || self.skiing(command) {
            0.0
        } else {
            self.friction
        }
    }

    pub(super) fn air_accelerate(
        self,
        state: &mut MovementState,
        direction: Vec3,
        mut wish_speed: f32,
        seconds: f32,
    ) {
        let velocity = Vec3::from_array(state.velocity);
        let projection = velocity.dot(direction);
        match self.air {
            Air::Standard => add_velocity(state, direction, wish_speed, 1.0 * seconds * wish_speed),
            Air::Sp => {
                if projection < 0.0 {
                    wish_speed *= 1.35;
                }
                add_velocity(state, direction, wish_speed, 4.0 * seconds * wish_speed);
            }
            Air::Qw | Air::Tribes => {
                if state.movement_type == PM_DEAD || state.movement_flags & PMF_TIME_WATERJUMP != 0
                {
                    return;
                }
                let (cap, amount) = if matches!(self.air, Air::Qw) {
                    (wish_speed.min(30.0), 0.7 * wish_speed * seconds * 4.0)
                } else {
                    (wish_speed, 0.25 * wish_speed * seconds * 1.9)
                };
                add_velocity(state, direction, cap, amount);
            }
            Air::Surf => add_velocity(
                state,
                direction,
                wish_speed.min(30.0),
                100.0 * wish_speed * seconds * 6.0,
            ),
            Air::Cpm | Air::Slick => {
                let control_speed = wish_speed;
                let mut acceleration = if projection < 0.0 { 2.5 } else { 1.0 };
                if matches!(state.movement_direction, 2 | 6) {
                    wish_speed = wish_speed.min(30.0);
                    acceleration = if matches!(self.air, Air::Slick) {
                        100.0
                    } else {
                        70.0
                    };
                }
                add_velocity(
                    state,
                    direction,
                    wish_speed,
                    acceleration * seconds * wish_speed,
                );
                air_control(state, direction, control_speed, seconds);
            }
        }
    }
}

fn add_velocity(state: &mut MovementState, direction: Vec3, cap: f32, amount: f32) {
    let velocity = Vec3::from_array(state.velocity);
    let remaining = cap - velocity.dot(direction);
    if remaining > 0.0 {
        state.velocity = (velocity + direction * amount.min(remaining)).to_array();
    }
}

pub(super) fn ski_accelerate(
    state: &mut MovementState,
    direction: Vec3,
    wish_speed: f32,
    seconds: f32,
) {
    if state.movement_type != PM_DEAD && state.movement_flags & PMF_TIME_WATERJUMP == 0 {
        add_velocity(
            state,
            direction,
            wish_speed.min(30.0),
            5.0 * wish_speed * seconds * 0.5,
        );
    }
}

fn air_control(state: &mut MovementState, direction: Vec3, wish_speed: f32, seconds: f32) {
    if !matches!(state.movement_direction, 0 | 4) || wish_speed == 0.0 {
        return;
    }
    let z = state.velocity[2];
    let horizontal = Vec3::new(state.velocity[0], state.velocity[1], 0.0);
    let speed = horizontal.length();
    let mut normalized = vector_normalize(horizontal);
    let dot = normalized.dot(direction);
    let k = 32.0 * (150.0 * dot * dot * seconds);
    if dot > 0.0 {
        normalized.x = normalized.x * speed + direction.x * k;
        normalized.y = normalized.y * speed + direction.y * k;
        normalized = vector_normalize(normalized);
    }
    state.velocity = [normalized.x * speed, normalized.y * speed, z];
}

impl Predictor {
    pub(super) fn japro_style(&self) -> Option<Rules> {
        if !self.config.ja_pro.enabled || self.state.client_num >= MAX_CLIENTS {
            return None;
        }
        let ordinal = if self.state.vehicle_entity_num != 0 {
            9
        } else {
            self.state.ja_pro.style
        };
        Some(*STYLES.get(ordinal as usize).unwrap_or(&STYLES[1]))
    }

    pub(super) fn japro_siege_accelerate(
        &mut self,
        direction: Vec3,
        speed: f32,
        acceleration: f32,
        seconds: f32,
    ) -> bool {
        if !self
            .japro_style()
            .is_some_and(|rules| rules.style == Style::Siege)
            || self.state.movement_type != 0
            || self.state.vehicle_entity_num != 0
        {
            return false;
        }
        let velocity = Vec3::from_array(self.state.velocity);
        let push = direction * speed - velocity;
        let length = push.length();
        self.state.velocity = (velocity
            + vector_normalize(push) * (acceleration * seconds * speed).min(length))
        .to_array();
        true
    }
}

#[cfg(test)]
#[path = "pmove_japro_movement_tests.rs"]
mod tests;
