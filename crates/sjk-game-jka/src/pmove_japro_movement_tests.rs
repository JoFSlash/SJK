use super::*;
use crate::pmove_japro::JaProRules;
include!("pmove_japro_movement_fixtures.rs");

fn mover(style: u32) -> Predictor {
    let mut seed = PlayerState::default();
    seed.stats[0] = 100;
    seed.stats[11] = 1;
    seed.stats[13] = style;
    seed.set_speed(320.0);
    seed.set_base_speed(320);
    // Air-kernel fixtures use ordinary gravity; zero-gravity push-off is a separate path.
    seed.set_gravity(800);
    seed.set_origin([0.0, 0.0, 200.0]);
    seed.set_ground_entity_num(ENTITY_NUMBER_NONE);
    Predictor::from_player_state(
        &seed,
        MovementConfig {
            ja_pro: JaProRules {
                enabled: true,
                ..Default::default()
            },
            ..Default::default()
        },
    )
}

fn compare(actual: [f32; 3], expected: [u32; 3], input: [i32; 3]) {
    for axis in 0..3 {
        let expected = f32::from_bits(expected[axis]);
        assert!(
            (actual[axis] - expected).abs() <= 0.0001,
            "{input:?}, axis {axis}: {} != {expected}",
            actual[axis]
        );
    }
}

#[test]
fn japro_all_style_air_kernels_match_compiled_taystjk() {
    for &(input, expected) in AIR {
        let [style, millis, sample] = input;
        let mut predictor = mover(style as u32);
        predictor.state.velocity = if sample == 0 {
            [200.0, 40.0, 20.0]
        } else {
            [-200.0, -40.0, -20.0]
        };
        predictor.state.movement_direction = if sample == 0 { 0 } else { 2 };
        let direction = if sample == 0 {
            Vec3::new(0.6, 0.8, 0.0)
        } else {
            Vec3::Y
        };
        let rules = predictor.japro_style().unwrap();
        if rules.style == Style::Siege {
            predictor.accelerate(direction, 320.0, 1.0, frame_seconds(millis));
        } else {
            rules.air_accelerate(
                &mut predictor.state,
                direction,
                320.0,
                frame_seconds(millis),
            );
        }
        compare(predictor.state.velocity, expected, input);
    }
}

#[test]
fn japro_style_ground_parameters_and_command_scale_match_taystjk() {
    for &(style, expected) in PARAMS {
        let rules = mover(style).japro_style().unwrap();
        assert_eq!(
            [rules.ground_acceleration.to_bits(), rules.duck.to_bits()],
            expected,
            "style {style}"
        );
    }
    for &(input, expected) in SCALE {
        let [style, sample] = input;
        let command = UserCommand {
            forward_move: 127,
            right_move: if sample == 0 { 0 } else { 64 },
            up_move: 127,
            ..Default::default()
        };
        assert_eq!(
            mover(style as u32).command_scale(&command).to_bits(),
            expected,
            "{input:?}"
        );
    }
}

#[test]
fn japro_friction_walk_ski_water_and_headslide_match_taystjk() {
    for &(input, expected) in FRICTION {
        let [style, millis, sample] = input;
        let mut predictor = mover(style as u32);
        predictor.state.velocity = [500.0, 40.0, 20.0];
        predictor.state.ground_entity_number = if sample >= 4 { 1 } else { ENTITY_NUMBER_WORLD };
        predictor.state.water_level = u8::from(sample == 3);
        predictor.config.ja_pro.head_slide = sample == 5;
        let mut ground = GroundState::air(MovementTrace::miss([0.0; 3]));
        ground.walking = true;
        let command = UserCommand {
            buttons: match sample {
                1 => 16,
                2 => BUTTON_DASH,
                _ => 0,
            },
            ..Default::default()
        };
        predictor.friction_for_command(frame_seconds(millis), &ground, &command);
        compare(predictor.state.velocity, expected, input);
    }
}

struct Clear;
impl MovementCollision for Clear {
    fn trace(&self, _: [f32; 3], _: [f32; 3], _: [f32; 3], end: [f32; 3], _: u32) -> MovementTrace {
        MovementTrace::miss(end)
    }
}

#[test]
fn japro_style_rules_follow_current_server_across_switches() {
    // All servers send the same occupied race/style slots and jcinfo bits.
    // Only the current jaPRO dialect may interpret them as jaPRO movement.
    for millis in [8, 7, 4, 3] {
        let mut game = sjk_protocol::GameState::empty_local(0);
        let mut predictor = mover(3);
        for name in [
            "japro", "basejka", "JA+ Mod", "lugormod", "TaystJK", "basejka",
        ] {
            game.replace_config_string(
                0,
                format!("\\gamename\\{name}\\jcinfo\\2097184").into_bytes(),
            )
            .unwrap();
            predictor.set_config(MovementConfig::from_game_state(&game));
            let mut seed = PlayerState::default();
            seed.stats[0] = 100;
            seed.stats[11] = 1;
            seed.stats[13] = 3;
            seed.set_speed(320.0);
            seed.set_base_speed(320);
            seed.set_origin([0.0, 0.0, 200.0]);
            seed.set_ground_entity_num(ENTITY_NUMBER_NONE);
            predictor = predictor.reseeded(&seed);
            let enabled = matches!(name, "japro" | "TaystJK");
            assert_eq!(predictor.japro_style().is_some(), enabled, "{name}");
            assert_eq!(predictor.config.ja_pro.head_slide, enabled, "{name}");
            predictor.predict_command(
                UserCommand {
                    server_time: millis,
                    right_move: 127,
                    ..Default::default()
                },
                &Clear,
            );
            let expected = if enabled {
                -(70.0 * frame_seconds(millis) * 30.0)
            } else {
                -(frame_seconds(millis) * 320.0).round_ties_even()
            };
            assert!(
                (predictor.state.velocity[1] - expected).abs() < 0.0001,
                "{name}, {millis}: {:?}",
                predictor.state.velocity
            );
        }
    }
}

#[test]
fn japro_surf_input_and_siege_vector_push_reach_live_command_path() {
    for millis in [8, 7, 4, 3] {
        let mut surf = mover(18);
        surf.predict_command(
            UserCommand {
                server_time: millis,
                forward_move: 127,
                right_move: 127,
                ..Default::default()
            },
            &Clear,
        );
        assert_eq!(surf.state.velocity[0], 30.0);
        assert_eq!(surf.state.velocity[1], 0.0);
        let mut siege = mover(0);
        siege.state.velocity = [200.0, 100.0, 0.0];
        siege.accelerate(Vec3::X, 320.0, 1.0, frame_seconds(millis));
        assert!(siege.state.velocity[0] > 200.0 && siege.state.velocity[1] < 100.0);
    }
}

#[test]
fn japro_steep_plane_rule_is_confined_to_slick_and_tribes() {
    let velocity = Vec3::new(100.0, 0.0, 10.0);
    let normal = Vec3::new(0.8, 0.0, 0.6);
    for style in 0..19 {
        let mut predictor = mover(style);
        predictor.state.ground_entity_number = ENTITY_NUMBER_WORLD;
        let clipped = predictor.clip_velocity(velocity, normal, OVERCLIP);
        if matches!(style, 13 | 17) {
            assert!(clipped.z < 0.0);
        } else {
            assert_eq!(clipped.z, 10.0);
        }
        predictor.config.ja_pro.enabled = false;
        assert_eq!(predictor.clip_velocity(velocity, normal, OVERCLIP).z, 10.0);
    }
}

#[test]
#[ignore = "release-only style command measurement"]
fn measure_japro_style_commands() {
    assert!(!cfg!(debug_assertions));
    const COMMANDS: i32 = 100_000;
    for style in [1, 2, 3, 17, 18] {
        let mut predictor = mover(style);
        let start = std::time::Instant::now();
        for number in 1..=COMMANDS {
            predictor.predict_command(
                std::hint::black_box(UserCommand {
                    server_time: number * 8,
                    forward_move: if number % 2 == 0 { 127 } else { 0 },
                    right_move: 127,
                    ..Default::default()
                }),
                &Clear,
            );
        }
        std::hint::black_box(predictor.state());
        println!(
            "jaPRO style {style}: {:.2} ns per empty-air command",
            start.elapsed().as_nanos() as f64 / f64::from(COMMANDS)
        );
    }
}
