//! World shots of Order 66's blast ([`crate::order66`]) on a recorded game: the demo
//! `SJK_TEST_DEMO` names is played without a server, the camera orbits another player
//! and they are blown apart as an order would. One shot shows the explosion; a sheet
//! shows the pieces (the explosion left out) moved on to four moments; a last shot the
//! confetti that replaces them without dismemberment. Ignored like the other world
//! shots; skipped when `SJK_TEST_DEMO` is not set.

use super::*;

/// The moments shot after the blast, in seconds.
const MOMENTS: [f32; 4] = [0.15, 0.4, 0.9, 4.0];

#[test]
#[ignore = "renders with the GPU, the installed game data and the demo SJK_TEST_DEMO names"]
fn order66_blast_on_a_demo() {
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
        let open = || {
            let mut gpu = card_outline_shots::open_demo(&demo, camera, [1600, 900])?;
            let _ = frame(&mut gpu, 20);
            Some(gpu)
        };

        let Some(mut gpu) = open() else {
            return;
        };
        let pieces = gpu.shot_order66_blast(target, false, true);
        println!("player {target}: {pieces} pieces");
        assert_eq!(pieces, 6, "a stock model flies apart in six");
        gpu.shot_order66_advance(0.06);
        gpu.order66.shot_frozen = true;
        println!("{}", shoot(&mut gpu, 3, "order66_explosion").display());

        let Some(mut gpu) = open() else {
            return;
        };
        gpu.shot_order66_blast(target, false, false);
        gpu.order66.shot_frozen = true;
        let mut shots = Vec::with_capacity(MOMENTS.len());
        let mut at = 0.0;
        for moment in MOMENTS {
            gpu.shot_order66_advance(moment - at);
            at = moment;
            shots.push(frame(&mut gpu, 1));
        }
        println!("{}", sheet(&shots, 2, 800, "order66_pieces").display());

        let Some(mut gpu) = open() else {
            return;
        };
        assert_eq!(
            gpu.shot_order66_blast(target, true, true),
            0,
            "confetti, no pieces"
        );
        println!("{}", shoot(&mut gpu, 12, "order66_confetti").display());
    });
}
