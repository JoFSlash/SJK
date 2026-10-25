//! World shots of the Ragdoll ([`crate::ragdoll`]) on a recorded game: the demo
//! `SJK_TEST_DEMO` names is played without a server, the camera orbits another player and
//! they go limp where they stand, shown at four moments of the fall; a second sheet
//! throws them forward and up. Ignored like the other world shots; skipped when
//! `SJK_TEST_DEMO` is not set.

use super::*;

/// The moments shot, in steps of the simulation (16 ms).
const STEPS: [u32; 4] = [0, 20, 45, 2_500];

#[test]
#[ignore = "renders with the GPU, the installed game data and the demo SJK_TEST_DEMO names"]
fn ragdoll_on_a_demo() {
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
        let camera = demo_playback::Camera::Orbit(target);
        let Some(mut gpu) = card_outline_shots::open_demo(&demo, camera, [1600, 900]) else {
            return;
        };
        let _ = frame(&mut gpu, 20);
        for (name, velocity) in [
            ("ragdoll_limp", [0, 0, 0]),
            ("ragdoll_thrown", [260, 0, 240]),
        ] {
            let mut shots = Vec::with_capacity(STEPS.len());
            for steps in STEPS {
                gpu.shot_ragdoll(target, velocity, 0, steps);
                shots.push(frame(&mut gpu, 2));
            }
            println!("player {target}: {}", sheet(&shots, 2, 800, name).display());
        }
        // Close by, from above the player's feet and a little aside: the body as it lies.
        let origin = probe
            .world()
            .entity(sjk_runtime::EntityId::new(u64::from(target) + 1))
            .map(|entity| {
                entity
                    .sample(i64::from(probe.latest_snapshot().server_time))
                    .translation
            })
            .expect("the player");
        let camera = demo_playback::Camera::Free {
            origin: [origin[0] + 55.0, origin[1] - 70.0, origin[2] + 30.0],
            yaw: 125.0,
            pitch: 35.0,
        };
        let Some(mut gpu) = card_outline_shots::open_demo(&demo, camera, [1600, 900]) else {
            return;
        };
        let _ = frame(&mut gpu, 20);
        let mut shots = Vec::with_capacity(STEPS.len());
        for (velocity, yaw, steps) in [
            ([0, 0, 0], 0, 30),
            ([0, 0, 0], 0, 2_500),
            ([0, 0, 0], 200, 2_500),
            ([-150, 120, 200], 90, 2_500),
        ] {
            gpu.shot_ragdoll(target, velocity, yaw, steps);
            shots.push(frame(&mut gpu, 2));
        }
        println!("{}", sheet(&shots, 2, 800, "ragdoll_close").display());
    });
}
