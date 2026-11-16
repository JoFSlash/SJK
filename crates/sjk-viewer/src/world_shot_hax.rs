//! A world shot of the Hax's digital rain ([`crate::hax`]) on a recorded game: the demo
//! `SJK_TEST_DEMO` names is played without a server, the camera orbits another player and
//! the rain is emitted round them as it is round a hacked player, every 70 ms for a
//! second. Ignored like the other world shots; skipped when `SJK_TEST_DEMO` is not set.

use super::*;

#[test]
#[ignore = "renders with the GPU, the installed game data and the demo SJK_TEST_DEMO names"]
fn hax_rain_on_a_demo() {
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
        for _ in 0..15 {
            gpu.shot_hax_glyphs(target);
            std::thread::sleep(std::time::Duration::from_millis(70));
            let _ = frame(&mut gpu, 1);
        }
        println!("player {target}: {} particles", gpu.particles.len());
        assert!(!gpu.particles.is_empty(), "the rain is drawn");
        println!("{}", shoot(&mut gpu, 1, "hax_rain").display());
    });
}
