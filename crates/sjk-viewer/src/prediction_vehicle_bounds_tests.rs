//! Remote-vehicle collision regression fixtures; no game assets or live server.
use super::*;
use sjk_client::pmove::MovementCollision;
use sjk_protocol::{EntityState, LEGACY_ENTITY_FIELDS, PlayerState};

fn vehicle(number: u16, yaw: f32) -> EntityState {
    let mut state = EntityState::zero(number, &LEGACY_ENTITY_FIELDS);
    state.set_raw_field(8, 13); // ET_NPC
    state.set_raw_field(88, 53); // CLASS_VEHICLE
    state.set_raw_field(46, 1); // CS_MODELS + 1
    state.set_raw_field(26, 10 | (10 << 8) | (42 << 16));
    state.set_raw_field(23, 2); // TR_LINEAR, also interpolated for remote vehicles
    state.set_raw_field(3, yaw.to_bits()); // apos.trBase[YAW]
    state
}

fn snapshot(entities: Vec<EntityState>, time: i32, sequence: i32) -> Snapshot {
    Snapshot {
        message_sequence: sequence,
        reliable_acknowledge: 0,
        server_commands: Vec::new(),
        server_time: time,
        delta_from: None,
        flags: 0,
        area_mask: Vec::new(),
        player: PlayerState::zero(),
        vehicle_player: None,
        entities,
        consumed_bits: 0,
    }
}

fn inputs() -> (VirtualFileSystem, GameState, Bsp) {
    let mut files = VirtualFileSystem::new();
    files.mount_memory("test", [("ext_data/vehicles/probe.veh", b"probe\n{\nname probe\ntype VH_FIGHTER\nlength 120\nwidth 40\nheight 20\n}\nwide\n{\nname wide\ntype VH_FIGHTER\nlength 120\nwidth 160\nheight 20\n}\n".to_vec())]).unwrap();
    let mut game = GameState::empty_local(0);
    game.replace_config_string(299, b"$probe".to_vec()).unwrap();
    let bsp = Bsp::parse(&sjk_bsp::write_collision_map(
        "{\n\"classname\" \"worldspawn\"\n}\n",
        &[sjk_bsp::CollisionShader {
            name: "textures/test/empty".into(),
            surface_flags: 0,
            content_flags: 1,
        }],
        &[],
    ))
    .unwrap();
    (files, game, bsp)
}

#[test]
fn remote_vehicle_traces_follow_the_presented_hull_at_each_command_step() {
    let (files, game, bsp) = inputs();
    for (step, smooth) in [8, 7, 4, 3]
        .into_iter()
        .flat_map(|step| [false, true].map(|smooth| (step, smooth)))
    {
        for (yaw, hit) in [(0.0, true), (90.0, false)] {
            let snap = snapshot(vec![vehicle(45, yaw)], 1_000, 1);
            let mut prediction = LocalPrediction::new(Some(&snap), None, Some(&game), &files);
            prediction.set_smooth_clients(smooth);
            let mut scratch = bsp.trace_scratch();
            prediction.physics_movers.set_angle_time(1_000 + step);
            let collision = BspMovementCollision::with_movers(
                &bsp,
                &mut scratch,
                &prediction.physics_movers.colliders,
            );
            let trace = collision.trace(
                [50.0, -100.0, 0.0],
                [0.0; 3],
                [0.0; 3],
                [50.0, 100.0, 0.0],
                0x100,
            );
            assert_eq!(trace.fraction < 1.0, hit, "{step} ms, yaw {yaw}");
            if hit {
                assert_eq!(trace.entity_number, 45);
            }
        }
    }
}

#[test]
fn remote_vehicle_hull_uses_interpolated_orientation_and_model_changes() {
    let (files, mut game, _) = inputs();
    for (step, smooth) in [8, 7, 4, 3]
        .into_iter()
        .flat_map(|step| [false, true].map(|smooth| (step, smooth)))
    {
        let before = snapshot(vec![vehicle(45, 0.0)], 1_000, 1);
        let after = snapshot(vec![vehicle(45, 90.0)], 1_000 + 2 * step, 2);
        let mut prediction = LocalPrediction::new(Some(&before), None, Some(&game), &files);
        prediction.set_smooth_clients(smooth);
        prediction.physics_movers.update(&after, 1_000 + step);
        prediction
            .physics_movers
            .present_boxes(&prediction.render_movers, 1_000 + step);
        let midpoint = prediction.physics_movers.colliders[0].bounds.unwrap();
        assert!((midpoint.maximums()[0] - 42.426407).abs() < 0.001);
        assert!((midpoint.maximums()[1] - 42.426407).abs() < 0.001);
        game.replace_config_string(299, b"$wide".to_vec()).unwrap();
        prediction.resolve_ride(&after, &game);
        prediction.physics_movers.update(&after, 1_000 + 2 * step);
        let changed = prediction.physics_movers.colliders[0].bounds.unwrap();
        assert!((changed.maximums()[0] - 80.0).abs() < 0.001);
        game.replace_config_string(299, b"$unknown".to_vec())
            .unwrap();
        prediction.resolve_ride(&after, &game);
        prediction.physics_movers.update(&after, 1_000 + 2 * step);
        assert_eq!(
            prediction.physics_movers.colliders[0]
                .bounds
                .unwrap()
                .maximums(),
            [10.0; 3]
        );
        game.replace_config_string(299, b"$probe".to_vec()).unwrap();
    }
}

#[cfg(debug_assertions)]
#[test]
fn warmed_remote_vehicle_prediction_allocates_nothing() {
    let (files, game, _) = inputs();
    let snap = snapshot(vec![vehicle(45, 30.0)], 1_000, 1);
    let mut prediction = LocalPrediction::new(Some(&snap), None, Some(&game), &files);
    let before = assert_no_alloc::violation_count();
    assert_no_alloc::assert_no_alloc(|| {
        for frame in 0..60 {
            prediction.resolve_ride(&snap, &game);
            prediction.physics_movers.set_angle_time(1_000 + frame * 3);
        }
    });
    assert_eq!(assert_no_alloc::violation_count(), before);
}

#[test]
#[ignore = "release-only remote vehicle hull update measurement"]
fn measure_remote_vehicle_hull_updates() {
    assert!(!cfg!(debug_assertions), "run with --release");
    let (files, game, _) = inputs();
    let entities = (32..160)
        .map(|number| {
            let mut entity = vehicle(number, 30.0);
            entity.set_raw_field(15, 2); // TR_LINEAR angular motion
            entity.set_raw_field(44, 90.0_f32.to_bits());
            entity.set_raw_field(34, 1_000);
            entity
        })
        .collect();
    let snap = snapshot(entities, 1_000, 1);
    let mut prediction = LocalPrediction::new(Some(&snap), None, Some(&game), &files);
    let iterations = 10_000;
    let start = std::time::Instant::now();
    for frame in 0..iterations {
        prediction
            .physics_movers
            .set_angle_time(std::hint::black_box(1_000 + frame * 3));
        prediction
            .physics_movers
            .present_boxes(&prediction.render_movers, 1_000 + frame * 3);
        std::hint::black_box(&prediction.physics_movers.colliders);
    }
    println!(
        "128 remote vehicles: {:.2} ns per vehicle frame update (placement and presentation)",
        start.elapsed().as_nanos() as f64 / (iterations as f64 * 128.0)
    );
}
