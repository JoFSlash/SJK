//! World shots of the cardboard box toy ([`crate::cardboard_box`]): two cartons on the
//! floor ahead of a map's first spawn point, without a session (the shots place them).
//! Ignored like the other world shots.
//!
//! `SJK_BOX_MAPS` names the maps (`duel6,ffa3`; default duel6). Each map gives one
//! sheet of four views: from the spawn, from a step aside, close up, and from above,
//! with a third box tipping over and a fourth lying on its side.

use super::*;

const VIEWS: [(Vec3, Vec3); 4] = [
    // The camera from the first carton in its yaw frame, and where it looks at.
    (Vec3::new(-150.0, 0.0, 30.0), Vec3::new(0.0, 0.0, 20.0)),
    (Vec3::new(-90.0, -80.0, 40.0), Vec3::new(0.0, 0.0, 20.0)),
    (Vec3::new(-45.0, -30.0, 40.0), Vec3::new(0.0, 0.0, 22.0)),
    (Vec3::new(-60.0, 0.0, 140.0), Vec3::new(0.0, 0.0, 0.0)),
];

#[test]
#[ignore = "renders with the GPU and the installed game data named by JKA_GAME_DATA"]
fn cartons_at_the_first_spawn() {
    on_big_stack(|| {
        let maps = std::env::var("SJK_BOX_MAPS").unwrap_or_else(|_| "duel6".to_owned());
        for map in maps.split(',').map(str::trim).filter(|map| !map.is_empty()) {
            shoot_map(map);
        }
    });
}

/// The origin a player standing on the floor under `point` has (24 over the floor).
fn standing(gpu: &mut GpuState, point: Vec3) -> Vec3 {
    let trace = gpu.bsp.trace_box_with(
        &mut gpu.trace_scratch,
        point.to_array(),
        (point - Vec3::Z * 512.0).to_array(),
        sjk_bsp::Aabb::new([0.0; 3], [0.0; 3]).expect("point bounds are valid"),
        1,
    );
    Vec3::from_array(trace.end_position) + Vec3::Z * 24.0
}

fn shoot_map(map: &str) {
    let Some((mut gpu, _profile)) = open(&format!("maps/mp/{map}.bsp"), [1280, 720], None, &[])
    else {
        return;
    };
    let (spawn, yaw) = assets::initial_camera(&gpu.bsp).expect("a spawn point");
    let turn = glam::Quat::from_rotation_z(yaw);
    let eye = Vec3::from_array(spawn);
    let first = standing(&mut gpu, eye + turn * Vec3::new(160.0, 0.0, 0.0));
    let second = standing(&mut gpu, eye + turn * Vec3::new(200.0, 70.0, 0.0));
    let tipping = standing(&mut gpu, eye + turn * Vec3::new(170.0, -80.0, 0.0));
    let lying = standing(&mut gpu, eye + turn * Vec3::new(230.0, -20.0, 0.0));
    gpu.third_person_choice = true;
    gpu.cardboard_box.shot_cartons = vec![
        (first.to_array(), yaw + 0.4),
        (second.to_array(), yaw + std::f32::consts::PI - 0.3),
    ];
    gpu.cardboard_box.shot_fallen = vec![
        (
            tipping.to_array(),
            yaw,
            yaw - 1.2,
            std::time::Duration::from_millis(330),
        ),
        (
            lying.to_array(),
            yaw + 0.5,
            yaw + 0.9,
            std::time::Duration::from_secs(2),
        ),
    ];
    let mut images = Vec::new();
    for (index, (from, at)) in VIEWS.iter().enumerate() {
        let camera = first + turn * *from;
        let target = first + turn * *at;
        let (look_yaw, look_pitch) = look(camera.to_array(), target.to_array());
        aim(&mut gpu, camera.to_array(), look_yaw, look_pitch);
        images.push(frame(&mut gpu, if index == 0 { 40 } else { 12 }));
    }
    println!("{map}: cartons at {first:?} and {second:?}");
    println!(
        "{}",
        sheet(&images, 2, 960, &format!("cardboard-box-{map}")).display()
    );
}
