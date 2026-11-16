use super::*;
use crate::pmove::{MovementCollision, MovementConfig, MovementTrace, Predictor};
use sjk_protocol::UserCommand;
include!("pmove_japro_fixtures.rs");

#[test]
#[ignore = "release-only command cost measurement"]
fn measure_japro_prediction_commands() {
    assert!(!cfg!(debug_assertions));
    const COMMANDS: i32 = 200_000;
    for enabled in [false, true] {
        let mut seed = player(true, 1);
        seed.set_gravity(0);
        let mut predictor = Predictor::from_player_state(
            &seed,
            MovementConfig {
                ja_pro: JaProRules {
                    enabled,
                    high_fps_fix: false,
                    head_slide: false,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        let start = std::time::Instant::now();
        for number in 1..=COMMANDS {
            predictor.predict_command(
                std::hint::black_box(UserCommand {
                    server_time: number * 8,
                    ..Default::default()
                }),
                &Clear,
            );
        }
        std::hint::black_box(predictor.state());
        println!(
            "jaPRO policy {enabled}: {:.2} ns per empty-world command",
            start.elapsed().as_nanos() as f64 / f64::from(COMMANDS)
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
fn japro_policy_matches_compiled_taystjk() {
    for &(input, expected) in SLICE {
        let [enabled, race, style, rolling, fixed, remaining] = input;
        let rules = JaProRules {
            enabled: enabled != 0,
            high_fps_fix: false,
            head_slide: false,
            ..Default::default()
        };
        let state = MovementState {
            ja_pro: JaProState {
                race: race != 0,
                style: style as u32,
                ..Default::default()
            },
            legs_anim: 1167,
            legs_timer: rolling,
            ..Default::default()
        };
        assert_eq!(
            rules.slice_millis(&state, remaining, (fixed != 0).then_some(16)),
            expected,
            "{input:?}"
        );
    }
    for &(input, expected) in SNAP {
        let [race, style, team, high, floating, millis] = input;
        let rules = JaProRules {
            enabled: true,
            high_fps_fix: high != 0,
            head_slide: false,
            ..Default::default()
        };
        let state = MovementState {
            ja_pro: JaProState {
                race: race != 0,
                style: style as u32,
                ..Default::default()
            },
            team: team as u8,
            ..Default::default()
        };
        assert_eq!(
            rules.snap_velocity(&state, millis, floating == 0),
            expected,
            "{input:?}"
        );
    }
}

fn player(race: bool, style: u32) -> PlayerState {
    let mut player = PlayerState::default();
    player.stats[0] = 100;
    player.stats[11] = u32::from(race);
    player.stats[13] = style;
    player.set_speed(250.0);
    player.set_base_speed(250);
    player.set_gravity(800);
    player.set_ground_entity_num(sjk_protocol::ENTITY_NUMBER_NONE);
    player.set_origin([20.0, 30.0, 200.0]);
    player.set_velocity([100.375, -30.25, 0.0]);
    player
}

#[test]
fn japro_fractional_velocity_survives_prediction_and_reseed() {
    for millis in [8, 7, 4, 3] {
        for (enabled, race, style, high) in [
            (true, true, 1, false),
            (true, true, 16, false),
            (true, false, 1, true),
            (false, true, 1, true),
        ] {
            let config = MovementConfig {
                ja_pro: JaProRules {
                    enabled,
                    high_fps_fix: high,
                    head_slide: false,
                    ..Default::default()
                },
                ..Default::default()
            };
            let seed = player(race, style);
            let mut predictor = Predictor::from_player_state(&seed, config);
            predictor.predict_command(
                UserCommand {
                    server_time: millis,
                    ..Default::default()
                },
                &Clear,
            );
            let fractional = enabled && (race && style != 16 || high && millis < 4);
            assert_eq!(
                predictor.state().velocity[0],
                if fractional { 100.375 } else { 100.0 }
            );
            assert_eq!(
                predictor.state().velocity[1],
                if fractional { -30.25 } else { -30.0 }
            );
            let next = player(false, 16);
            predictor = predictor.reseeded(&next);
            assert_eq!(
                predictor.state().ja_pro,
                JaProState {
                    race: false,
                    style: 16,
                    predicts_jetpack_flag: enabled,
                    ..Default::default()
                }
            );
        }
    }
}

#[test]
fn japro_long_commands_clamp_simulated_time_without_chopping_race_moves() {
    for (race, style, expected_timer) in [(true, 1, 800), (true, 16, 650), (false, 1, 650)] {
        let mut seed = player(race, style);
        seed.set_movement_type(3); // noclip: timers expose actual simulated duration.
        let mut predictor = Predictor::from_player_state(
            &seed,
            MovementConfig {
                ja_pro: JaProRules {
                    enabled: true,
                    high_fps_fix: false,
                    head_slide: false,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        predictor.state_mut().legs_timer = 1000;
        predictor.state_mut().delta_angles[2] = 4096;
        predictor.predict_command(
            UserCommand {
                server_time: 350,
                angles: [0, 0, 8192],
                ..Default::default()
            },
            &Clear,
        );
        assert_eq!(predictor.state().legs_timer, expected_timer);
        assert_eq!(predictor.state().command_time, 350);
        assert_eq!(predictor.state().view_angles[2], 0.0);
        assert_eq!(predictor.state().delta_angles[2], 0);
    }
}

#[test]
fn japro_wallbug_cleanup_is_race_only() {
    for (enabled, race, team, cleared) in [
        (true, true, 0, true),
        (true, true, 3, false),
        (true, false, 0, false),
        (false, true, 0, false),
    ] {
        let rules = JaProRules {
            enabled,
            high_fps_fix: false,
            head_slide: false,
            ..Default::default()
        };
        let mut state = MovementState {
            ja_pro: JaProState {
                race,
                style: 1,
                ..Default::default()
            },
            team,
            velocity: [10.0; 3],
            ..Default::default()
        };
        rules.finish_velocity(&mut state, [0.0; 3], 8, true);
        assert_eq!(state.velocity, if cleared { [0.0; 3] } else { [10.0; 3] });
    }
}

#[test]
fn japro_metadata_is_interpreted_only_for_advertised_dialect() {
    let mut game = GameState::empty_local(0);
    for (name, enabled) in [
        ("japro", true),
        ("TaystJK", true),
        ("basejka", false),
        ("JA+ Mod", false),
        ("lugormod", false),
    ] {
        game.replace_config_string(
            0,
            format!("\\gamename\\{name}\\jcinfo\\2097152").into_bytes(),
        )
        .unwrap();
        let config = MovementConfig::from_game_state(&game);
        assert_eq!(config.ja_pro.enabled, enabled);
        assert_eq!(config.ja_pro.high_fps_fix, enabled);
    }
}
