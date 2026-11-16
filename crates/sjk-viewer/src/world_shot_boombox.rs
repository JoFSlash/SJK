//! World shots of the Boombox ([`crate::boombox`]) on a recorded game: the demo
//! `SJK_TEST_DEMO` names is played without a server, every player in it carrying the
//! stereo, seen by a camera orbiting one of them. Ignored like the other world shots;
//! skipped when `SJK_TEST_DEMO` is not set.

use super::*;

#[test]
#[ignore = "renders with the GPU, the installed game data and the demo SJK_TEST_DEMO names"]
fn boombox_on_a_demo() {
    on_big_stack(|| {
        let Some(demo) = std::env::var_os("SJK_TEST_DEMO").map(PathBuf::from) else {
            println!("SJK_TEST_DEMO is not set: skipped");
            return;
        };
        let probe = demo_playback::Session::open(&demo, demo_playback::Camera::FirstPerson, 60)
            .expect("the demo");
        let own = probe.latest_snapshot().player.client_num();
        let targets: Vec<u16> = probe
            .latest_snapshot()
            .entities
            .iter()
            .map(|entity| entity.number())
            .filter(|number| *number < 32 && *number != own)
            .take(3)
            .collect();
        if targets.is_empty() {
            println!("no other player in the demo's first snapshot: skipped");
            return;
        }
        let mut shots = Vec::with_capacity(targets.len());
        for target in targets {
            let camera = demo_playback::Camera::Orbit(target);
            let Some(mut gpu) = card_outline_shots::open_demo(&demo, camera, [1200, 900]) else {
                return;
            };
            gpu.boombox.shot_carriers = u64::MAX;
            shots.push(frame(&mut gpu, 30));
        }
        println!("{}", sheet(&shots, 2, 800, "boombox").display());
    });
}
