use super::*;
use crate::pmove_japro::{JaProRules, JaProState};
include!("pmove_japro_jetpack_fixtures.rs");

struct Distance(f32);

#[test]
#[ignore = "release-only jetpack command cost measurement"]
fn measure_japro_pack_commands() {
    assert!(!cfg!(debug_assertions));
    for style in [10, 17] {
        let mut p = mover(style);
        p.state.origin[2] = 200.0;
        let start = std::time::Instant::now();
        for number in 1..=100_000 {
            p.predict_command(
                std::hint::black_box(UserCommand {
                    server_time: number * 8,
                    up_move: 127,
                    forward_move: 127,
                    ..Default::default()
                }),
                &Clear,
            );
        }
        std::hint::black_box(p.state());
        println!(
            "jaPRO pack style {style}: {:.2} ns per empty-world command",
            start.elapsed().as_nanos() as f64 / 100_000.0
        );
    }
}
struct Clear;
impl MovementCollision for Clear {
    fn trace(&self, _: [f32; 3], _: [f32; 3], _: [f32; 3], end: [f32; 3], _: u32) -> MovementTrace {
        MovementTrace::miss(end)
    }
}

#[test]
fn japro_pack_commands_follow_current_server_and_reseed() {
    for millis in [8, 7, 4, 3] {
        let mut p = mover(10);
        let mut game = sjk_protocol::GameState::empty_local(0);
        for name in [
            "japro", "basejka", "JA+ Mod", "lugormod", "TaystJK", "basejka",
        ] {
            game.replace_config_string(0, format!("\\gamename\\{name}\\jcinfo\\4096").into_bytes())
                .unwrap();
            p.set_config(MovementConfig::from_game_state(&game));
            let mut seed = PlayerState::default();
            seed.stats[0] = 100;
            seed.stats[2] = OWNED_JETPACK;
            seed.stats[11] = 1;
            seed.stats[13] = 10;
            seed.set_raw_field(39, 100);
            seed.set_speed(320.0);
            seed.set_base_speed(320);
            seed.set_gravity(800);
            seed.set_origin([0.0, 0.0, 200.0]);
            seed.set_velocity([0.0, 0.0, 100.0]);
            seed.set_ground_entity_num(ENTITY_NUMBER_NONE);
            p = p.reseeded(&seed);
            p.predict_command(
                UserCommand {
                    server_time: millis,
                    up_move: 127,
                    ..Default::default()
                },
                &Clear,
            );
            let enabled = matches!(name, "japro" | "TaystJK");
            assert_eq!(p.new_jetpack_active(), enabled, "{name}, {millis}");
            let expected = if enabled {
                100.0 + (425.0 * (320.0 / 127.0) * 1.4 - 800.0) * frame_seconds(millis)
            } else {
                (100.0 - 800.0 * frame_seconds(millis)).round_ties_even()
            };
            assert!(
                (p.state.velocity[2] - expected).abs() < 0.0005,
                "{name}, {millis}: {} vs {expected}",
                p.state.velocity[2]
            );
            assert_eq!(p.state.ja_pro.jetpack_fuel, 100);
            assert_eq!(
                p.state.ja_pro.jump_time,
                if enabled { 500 - millis } else { 0 }
            );
        }
    }
}

#[test]
fn japro_pack_pose_priorities_and_downward_floor_cutoff() {
    let mut p = mover(10);
    p.state.entity_flags = ACTIVE;
    p.set_animation_lengths(std::sync::Arc::new(
        crate::pmove_anim::AnimationLengthTable::new(vec![
            crate::pmove_anim::AnimationTiming {
                frame_count: 10,
                frame_lerp_ms: 50
            };
            1500
        ]),
    ));
    for (forward_move, right_move, expected) in [
        (0, 0, 1152),
        (0, 127, 1160),
        (0, -127, 1157),
        (127, 127, 1151),
        (-127, 127, 1154),
    ] {
        p.new_jetpack_pose(&UserCommand {
            forward_move,
            right_move,
            ..Default::default()
        });
        assert_eq!(p.state.legs_anim, expected);
    }
    for distance in [15.0, 16.0] {
        p.state.entity_flags = ACTIVE;
        p.state.velocity = [0.0; 3];
        p.new_jetpack_thrust(
            &UserCommand {
                up_move: -127,
                ..Default::default()
            },
            bounds(),
            &Distance(distance),
            0.008,
            100.0,
        );
        assert_eq!(p.new_jetpack_active(), distance >= 16.0);
        assert_eq!(p.state.velocity[2] < 0.0, distance >= 16.0);
    }
}
impl MovementCollision for Distance {
    fn trace(
        &self,
        start: [f32; 3],
        _: [f32; 3],
        _: [f32; 3],
        mut end: [f32; 3],
        _: u32,
    ) -> MovementTrace {
        end[2] = start[2] - self.0;
        MovementTrace::miss(end)
    }
}
fn bounds() -> Bounds {
    Bounds {
        minimums: [-15.0, -15.0, -24.0],
        maximums: [15.0, 15.0, 40.0],
    }
}
fn mover(style: u32) -> Predictor {
    Predictor::from_state(
        MovementState {
            health: 100,
            speed: 320.0,
            base_speed: 320.0,
            gravity: 800.0,
            holdable_items: OWNED_JETPACK,
            force_power: 10,
            ground_entity_number: ENTITY_NUMBER_NONE,
            ja_pro: JaProState {
                race: true,
                style,
                jetpack_fuel: 10,
                jump_time: 300,
                ..Default::default()
            },
            ..Default::default()
        },
        MovementConfig {
            ja_pro: JaProRules {
                enabled: true,
                jetpack: true,
                ..Default::default()
            },
            ..Default::default()
        },
    )
}

#[test]
fn japro_pack_thrust_matches_compiled_taystjk() {
    let velocities = [
        -4000.0, -3000.0, -600.0, -1.0, 0.0, 100.0, 249.0, 250.0, 800.0, 1600.0, 1999.0, 2000.0,
    ];
    for &(input, expected, flags) in THRUST {
        let [style, millis, sample, up, _] = input;
        let mut p = mover(style as u32);
        p.state.entity_flags = ACTIVE;
        p.state.velocity = [
            if style == 17 { 200.0 } else { 0.0 },
            0.0,
            velocities[sample as usize],
        ];
        let cmd = UserCommand {
            up_move: (up * 127) as i8,
            forward_move: if style == 17 { -127 } else { 0 },
            ..Default::default()
        };
        p.new_jetpack_thrust(
            &cmd,
            bounds(),
            &Distance(100.0),
            millis as f32 * 0.001,
            100.0,
        );
        for (actual, bits) in p.state.velocity.into_iter().zip(expected) {
            assert!(
                (actual - f32::from_bits(bits)).abs() < 0.0005,
                "{input:?}: {actual} vs {}",
                f32::from_bits(bits)
            );
        }
        assert_eq!(p.state.entity_flags, flags, "{input:?}");
        assert_eq!(p.state.ja_pro.jetpack_fuel, 10);
    }
    for &(input, velocity, flags) in NONRACE {
        let [millis, sample] = input;
        let mut p = mover(10);
        p.state.entity_flags = ACTIVE;
        p.state.ja_pro.race = false;
        p.state.gravity = 400.0;
        p.state.velocity[2] = [0.0, 100.0, -600.0, 1600.0][sample as usize];
        p.new_jetpack_thrust(
            &UserCommand {
                up_move: 127,
                ..Default::default()
            },
            bounds(),
            &Distance(100.0),
            millis as f32 * 0.001,
            100.0,
        );
        assert!(
            (p.state.velocity[2] - f32::from_bits(velocity)).abs() < 0.0005,
            "{input:?}"
        );
        assert_eq!(p.state.entity_flags, flags);
    }
}

#[test]
fn japro_pack_activation_matches_compiled_taystjk() {
    for &(input, flags, timer, distance) in ACTIVATE {
        let [style, sample, active] = input;
        let mut p = mover(style as u32);
        p.state.entity_flags = if active != 0 { ACTIVE } else { 0 };
        p.state.ja_pro.jump_time = if sample == 10 { 299 } else { 300 };
        p.state.ja_pro.jetpack_fuel = match sample {
            1 => 0,
            2 => 9,
            _ => 10,
        };
        p.state.force_power = match sample {
            3 => 0,
            4 => 9,
            _ => 10,
        };
        if sample == 0 {
            p.state.holdable_items = 0;
        }
        if sample == 5 {
            p.state.movement_flags = PMF_JUMP_HELD;
        }
        if sample == 6 {
            p.state.velocity[2] = 1.0;
        }
        if sample == 12 {
            p.state.water_level = 2;
        }
        if sample == 13 {
            p.vehicle = Some(Box::new(crate::vehicle::Vehicle::new(
                std::sync::Arc::new(crate::vehicle_fields::VehicleInfo::default()),
                0,
            )));
        }
        if sample == 14 {
            p.state.ground_entity_number = ENTITY_NUMBER_WORLD;
        }
        if sample == 15 {
            p.state.saber_move = (0..200)
                .find(|&m| crate::saber_rules::in_special(m))
                .unwrap();
        }
        if sample == 16 {
            p.state.legs_anim = crate::pmove_roll::BOTH_ROLL_F;
            p.state.legs_timer = 100;
        }
        if sample == 17 {
            p.state.legs_anim = (0..1500)
                .find(|&a| {
                    crate::saber_rules::special_jump(a)
                        && !crate::pmove_hand_extend::knockdown_animation(a)
                })
                .unwrap();
        }
        let cmd = UserCommand {
            up_move: match sample {
                7 => 0,
                8 => -127,
                _ => 127,
            },
            buttons: if sample == 14 { BUTTON_DASH } else { 0 },
            ..Default::default()
        };
        let ground = Distance(match sample {
            9 => 64.0,
            11 => 63.0,
            _ => 100.0,
        });
        let actual = p.activate_new_jetpack(&cmd, bounds(), &ground);
        assert_eq!(
            (
                p.state.entity_flags,
                p.state.ja_pro.jump_time,
                actual.to_bits()
            ),
            (flags, timer, distance),
            "{input:?}"
        );
    }
}

#[test]
fn japro_pack_knockdown_gate_is_timeless() {
    assert_eq!(KNOCKDOWN.len(), 25);
    for animation in 0..1500 {
        assert_eq!(
            crate::pmove_hand_extend::knockdown_animation(animation),
            KNOCKDOWN.contains(&animation),
            "{animation}"
        );
    }
}

#[test]
fn japro_pack_switch_disables_every_entry_and_flag_ownership() {
    for millis in [8, 7, 4, 3] {
        let mut p = mover(17);
        p.state.entity_flags = ACTIVE;
        p.state.velocity = [200.0, 0.0, 100.0];
        let command = UserCommand {
            up_move: 127,
            forward_move: -127,
            ..Default::default()
        };
        for enabled in [true, false, true, false] {
            let mut config = p.config;
            config.ja_pro.enabled = enabled;
            p.set_config(config);
            assert_eq!(p.new_jetpack_active(), enabled);
            if !enabled {
                let before = p.state.clone();
                assert!(!p.can_start_new_jetpack());
                assert_eq!(
                    p.activate_new_jetpack(&command, bounds(), &Distance(100.0)),
                    0.0
                );
                p.new_jetpack_thrust(
                    &command,
                    bounds(),
                    &Distance(100.0),
                    millis as f32 * 0.001,
                    100.0,
                );
                p.new_jetpack_pose(&command);
                assert_eq!(p.state.velocity, before.velocity);
                assert_eq!(p.state.legs_anim, before.legs_anim);
                assert_eq!(p.state.entity_flags, before.entity_flags);
            }
            for server_flag in [0, ACTIVE] {
                let mut player = PlayerState::default();
                player.set_raw_field(17, server_flag);
                p.write_player_state(&mut player);
                assert_eq!(
                    player.entity_flags() & ACTIVE,
                    if enabled { ACTIVE } else { server_flag }
                );
            }
        }
    }
}
