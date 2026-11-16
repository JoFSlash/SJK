//! World shots of the Monitor ([`crate::monitor`]) on a recorded game: the demo
//! `SJK_TEST_DEMO` names is played without a server, the camera orbits another player, a
//! monitor flies at their head, and they are sent flying as the blow takes them (four
//! moments); a second sheet strikes them at the head, the chest and the legs, and shows a
//! monitor at rest. Ignored like the other world shots; skipped when `SJK_TEST_DEMO` is
//! not set.

use super::*;

/// The moments of the flight shot, in steps of the simulation (16 ms).
const STEPS: [u32; 4] = [6, 18, 40, 2_500];
/// The blow of a monitor thrown at 1,100 units a second at a player standing still,
/// along +X (`monitor::launch`).
const PUSH: [i32; 3] = [1_650, 0, 260];

#[test]
#[ignore = "renders with the GPU, the installed game data and the demo SJK_TEST_DEMO names"]
fn monitor_on_a_demo() {
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
            .expect("the player")
            .map(|c| c.round() as i32);
        let camera = demo_playback::Camera::Free {
            origin: [
                (origin[0] + 40) as f32,
                (origin[1] - 260) as f32,
                (origin[2] + 60) as f32,
            ],
            yaw: 90.0,
            pitch: 8.0,
        };
        let Some(mut gpu) = card_outline_shots::open_demo(&demo, camera, [1600, 900]) else {
            return;
        };
        let _ = frame(&mut gpu, 20);
        // The monitor on its way, then the player flying (0 their head).
        let mut shots = Vec::with_capacity(STEPS.len() + 1);
        let throw_from = [origin[0] - 140, origin[1], origin[2] + 38];
        gpu.shot_monitor(target as u8, 0, throw_from, [1_100, 0, 40], 4);
        shots.push(frame(&mut gpu, 2));
        for steps in STEPS {
            gpu.shot_ragdoll_struck(target, [0; 3], 90, Some((2, PUSH)), steps);
            let back = [origin[0] - 12, origin[1], origin[2] + 40];
            gpu.shot_monitor(target as u8, 0, back, [-500, 0, 150], steps);
            shots.push(frame(&mut gpu, 2));
        }
        println!(
            "player {target}: {}",
            sheet(&shots, 2, 800, "monitor_flight").display()
        );
        // Where the blow lands: the head (2), the chest (1), a knee (10); a monitor at rest.
        let mut shots = Vec::with_capacity(4);
        for joint in [2, 1, 10] {
            gpu.shot_ragdoll_struck(target, [0; 3], 90, Some((joint, PUSH)), 30);
            shots.push(frame(&mut gpu, 2));
        }
        gpu.shot_ragdoll(target, [0; 3], 90, 0);
        gpu.shot_monitor(
            target as u8,
            0,
            [origin[0] + 30, origin[1] - 40, origin[2] + 30],
            [200, 50, 100],
            2_000,
        );
        shots.push(frame(&mut gpu, 2));
        println!("{}", sheet(&shots, 2, 800, "monitor_blows").display());
    });
}
