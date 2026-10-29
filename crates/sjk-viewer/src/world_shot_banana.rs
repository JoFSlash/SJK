//! World shots of the Banana peel ([`crate::banana`]): peels on the floor ahead of a map's
//! first spawn point, one still on its way from the hand, without a session (the shots
//! place them); the toy's card and hover card in Collection > Toys; and, on a recorded
//! game (`SJK_TEST_DEMO`), another player slipping on a peel. Ignored like the other
//! world shots; the profile turns the Discord status off.
//!
//! `SJK_BANANA_MAPS` names the maps of the first (`duel6,ffa3`; default duel6).

use super::*;
use crate::player_menu::ReturnTarget;
use crate::profile_hub::Tab;

/// The camera from the first peel in its yaw frame, and where it looks at.
const VIEWS: [(Vec3, Vec3); 4] = [
    (Vec3::new(-45.0, -15.0, 28.0), Vec3::new(0.0, 0.0, 2.0)),
    (Vec3::new(-24.0, -10.0, 12.0), Vec3::new(0.0, 0.0, 2.0)),
    (Vec3::new(-8.0, 0.0, 60.0), Vec3::new(0.0, 0.0, 0.0)),
    (Vec3::new(-120.0, 30.0, 48.0), Vec3::new(10.0, 0.0, 0.0)),
];

/// The shots' cvars: SJK's menus, and never the machine's own Discord.
const CVARS: [(&str, &str); 3] = [
    ("ui_menuStyle", "sjk"),
    (crate::settings::quick::HIDE_CVAR, "1"),
    (crate::discord_presence::CVAR, "0"),
];

#[test]
#[ignore = "renders with the GPU and the installed game data named by JKA_GAME_DATA"]
fn banana_peels_ahead_of_the_first_spawn() {
    on_big_stack(|| {
        let maps = std::env::var("SJK_BANANA_MAPS").unwrap_or_else(|_| "duel6".to_owned());
        for map in maps.split(',').map(str::trim).filter(|map| !map.is_empty()) {
            shoot_map(map);
        }
    });
}

/// The floor under `point`.
fn floor(gpu: &mut GpuState, point: Vec3) -> Vec3 {
    let trace = gpu.bsp.trace_box_with(
        &mut gpu.trace_scratch,
        point.to_array(),
        (point - Vec3::Z * 512.0).to_array(),
        sjk_bsp::Aabb::new([0.0; 3], [0.0; 3]).expect("point bounds are valid"),
        1,
    );
    Vec3::from_array(trace.end_position)
}

fn shoot_map(map: &str) {
    let Some((mut gpu, _profile)) = open(&format!("maps/mp/{map}.bsp"), [1280, 720], None, &CVARS)
    else {
        return;
    };
    let (spawn, yaw) = assets::initial_camera(&gpu.bsp).expect("a spawn point");
    let turn = glam::Quat::from_rotation_z(yaw);
    let eye = Vec3::from_array(spawn);
    let first = floor(&mut gpu, eye + turn * Vec3::new(150.0, 0.0, 0.0));
    let second = floor(&mut gpu, eye + turn * Vec3::new(175.0, 40.0, 0.0));
    let third = floor(&mut gpu, eye + turn * Vec3::new(140.0, -45.0, 0.0));
    let flying = floor(&mut gpu, eye + turn * Vec3::new(200.0, -10.0, 0.0));
    let yaw_degrees = yaw.to_degrees();
    gpu.shot_banana_time(10_000);
    gpu.shot_banana(1, 0, first, yaw_degrees + 20.0, 5_000);
    gpu.shot_banana(1, 1, second, yaw_degrees + 140.0, 5_000);
    gpu.shot_banana(2, 0, third, yaw_degrees - 100.0, 5_000);
    // Half way from the hand.
    gpu.shot_banana(3, 0, flying, yaw_degrees, crate::banana::FLIGHT_MS / 2);
    let mut images = Vec::new();
    for (index, (from, at)) in VIEWS.iter().enumerate() {
        let camera = first + turn * *from;
        let target = first + turn * *at;
        let (look_yaw, look_pitch) = look(camera.to_array(), target.to_array());
        aim(&mut gpu, camera.to_array(), look_yaw, look_pitch);
        images.push(frame(&mut gpu, if index == 0 { 40 } else { 12 }));
    }
    println!("{map}: peels at {first:?}, {second:?}, {third:?}, flying to {flying:?}");
    println!(
        "{}",
        sheet(&images, 2, 960, &format!("banana-peels-{map}")).display()
    );
}

/// The Banana peel's row chosen in Collection > Toys, its card beside the model, then its
/// hover card.
#[test]
#[ignore = "renders with the GPU and the installed game data named by JKA_GAME_DATA"]
fn banana_peel_toys_card() {
    on_big_stack(|| {
        let Some((mut gpu, _profile)) = open(
            "maps/mp/duel6.bsp",
            [1920, 1080],
            Some(menu::ClientMenu::new(true, String::new())),
            &CVARS,
        ) else {
            return;
        };
        gpu.looks.shot_owns_unlocks = true;
        let _ = frame(&mut gpu, 10);
        if let Some(console) = gpu.console.as_mut() {
            console.preview_profile(super::tests::profile_preview());
            console.open_profile_hub_page(Tab::Toys, ReturnTarget::MainMenu);
        }
        gpu.profile_hub_show_for_shot(Tab::Toys);
        assert_eq!(gpu.profile_hub_tab(), Some(Tab::Toys));
        let banana = 9;
        if let Some(console) = gpu.console.as_mut() {
            console.collection_toy_for_shot(banana);
        }
        gpu.ui_epoch -= std::time::Duration::from_millis(3_000);
        let card = shoot(&mut gpu, 60, "duel6-collection-toys-banana");
        println!("{}", card.display());
        if let Some(console) = gpu.console.as_mut() {
            assert!(!console.profile_pages_overflowed(), "the Toys tab");
            console.collection_hover_for_shot(Some(banana));
        }
        let hover = shoot(&mut gpu, 30, "duel6-hover-toys-banana");
        println!("{}", hover.display());
    });
}

/// Moments of the slip, in steps of the simulation (16 ms).
const STEPS: [u32; 5] = [4, 12, 22, 36, 2_500];

/// Another player of the demo `SJK_TEST_DEMO` names walking onto a peel and slipping:
/// the peel at their feet, then the fall at five moments (their feet thrown up over
/// their head, then on their back).
#[test]
#[ignore = "renders with the GPU, the installed game data and the demo SJK_TEST_DEMO names"]
fn banana_slip_on_a_demo() {
    on_big_stack(|| {
        let Some(demo) = std::env::var_os("SJK_TEST_DEMO").map(PathBuf::from) else {
            println!("SJK_TEST_DEMO is not set: skipped");
            return;
        };
        let probe = demo_playback::Session::open(&demo, demo_playback::Camera::FirstPerson, 60)
            .expect("the demo");
        let own = probe.latest_snapshot().player.client_num();
        let Some(target) = probe
            .latest_snapshot()
            .entities
            .iter()
            .map(|entity| entity.number())
            .find(|number| *number < 32 && *number != own)
        else {
            println!("no other player in the demo's first snapshot: skipped");
            return;
        };
        let origin = probe
            .world()
            .entity(sjk_runtime::EntityId::new(u64::from(target) + 1))
            .map(|entity| {
                entity
                    .sample(i64::from(probe.latest_snapshot().server_time))
                    .translation
            })
            .expect("the player");
        let origin = Vec3::from_array(origin);
        // The way along the floor with the most room (of eight), from the map's brushes.
        let game_data = PathBuf::from(
            std::env::var_os("JKA_GAME_DATA").expect("JKA_GAME_DATA names the GameData directory"),
        );
        let map = crate::app_launch::map_path_from_gamestate(probe.game_state()).expect("a map");
        let selection = assets::session_content::Selection::from_game(probe.game_state())
            .expect("the demo's content");
        let vfs = selection.mount(&game_data).expect("the content");
        let bsp = selection.load_bsp(&vfs, &map).expect("the map");
        let mut scratch = sjk_bsp::TraceScratch::default();
        let bounds = sjk_bsp::Aabb::new([-15.0, -15.0, -8.0], [15.0, 15.0, 8.0]).expect("bounds");
        let room = |way: Vec3, scratch: &mut sjk_bsp::TraceScratch| {
            let trace = bsp.trace_box_with(
                scratch,
                origin.to_array(),
                (origin + way * 400.0).to_array(),
                bounds,
                1,
            );
            trace.fraction
        };
        let way = (0..8)
            .map(|eighth| {
                let angle = eighth as f32 * std::f32::consts::FRAC_PI_4;
                Vec3::new(angle.cos(), angle.sin(), 0.0)
            })
            .max_by(|a, b| room(*a, &mut scratch).total_cmp(&room(*b, &mut scratch)))
            .expect("a way");
        // From the side, a little ahead of them.
        let side = Vec3::new(-way.y, way.x, 0.0);
        let at = origin + way * 50.0 - Vec3::Z * 10.0;
        let eye = at - side * 170.0 + Vec3::Z * 45.0;
        let (yaw, pitch) = look(eye.to_array(), at.to_array());
        let camera = demo_playback::Camera::Free {
            origin: eye.to_array(),
            yaw,
            pitch: -pitch,
        };
        let Some(mut gpu) = card_outline_shots::open_demo(&demo, camera, [1600, 900]) else {
            return;
        };
        if let Some(console) = gpu.console.as_mut() {
            console.set_cvar(crate::discord_presence::CVAR, "0");
        }
        let _ = frame(&mut gpu, 20);
        // Walking that way onto a peel at their feet.
        let velocity = way * 250.0;
        let walk_yaw = way.y.atan2(way.x).to_degrees().rem_euclid(360.0);
        gpu.shot_banana_time(10_000);
        gpu.shot_banana(
            1,
            0,
            origin + way * 14.0 - Vec3::Z * 24.0,
            walk_yaw + 90.0,
            5_000,
        );
        let mut shots = vec![frame(&mut gpu, 4)];
        gpu.banana.peels.list.clear();
        let push = crate::banana::slip_push(velocity);
        for steps in STEPS {
            gpu.shot_ragdoll_struck(
                target,
                velocity.to_array().map(|c| c.round() as i32),
                walk_yaw.round() as u16 % 360,
                Some((crate::ragdoll::L_FOOT as u8, push)),
                steps,
            );
            shots.push(frame(&mut gpu, 2));
        }
        println!(
            "player {target}: {}",
            sheet(&shots, 3, 640, "banana_slip").display()
        );
    });
}
