//! World shots of a body fading away ([`crate::body_fade`]) on a recorded game: the demo
//! `SJK_TEST_DEMO` names is scanned for `EV_BODYFADE` on an `ET_BODY`, then played to
//! four moments of that fade (the body thinning, the glow at its brightest, the burn,
//! the burn nearly done) with a fixed camera by the body. Ignored like the other world
//! shots; skipped when `SJK_TEST_DEMO` is not set or its game has no fading body.

use super::*;

/// The moments shot, in ms after the event.
const MOMENTS: [u64; 4] = [600, 1_800, 2_800, 3_600];
/// How many fades of the demo are shot.
const FADES: usize = 2;

#[test]
#[ignore = "renders with the GPU, the installed game data and the demo SJK_TEST_DEMO names"]
fn body_fade_on_a_demo() {
    on_big_stack(|| {
        let Some(demo) = std::env::var_os("SJK_TEST_DEMO").map(PathBuf::from) else {
            println!("SJK_TEST_DEMO is not set: skipped");
            return;
        };
        // Every fade of the demo: when (relative to its first snapshot), which body, where.
        let mut probe = demo_playback::Session::open(&demo, demo_playback::Camera::FirstPerson, 60)
            .expect("the demo");
        let first = probe.first_server_time();
        let mut fades = Vec::new();
        let mut step = 0;
        while !probe.ended() && fades.len() < FADES {
            step += 1_000;
            let mut seen = Vec::new();
            probe
                .advance_to(step, |snapshot, _| {
                    for entity in &snapshot.entities {
                        if entity.entity_type() == 15
                            && entity.event() & 0xff == sjk_client::body_fade_assets::EV_BODYFADE
                        {
                            seen.push((snapshot.server_time, entity.number()));
                        }
                    }
                })
                .expect("the demo plays");
            for (time, number) in seen {
                if fades.iter().any(|(_, seen, _)| *seen == number) {
                    continue;
                }
                let origin = probe
                    .world()
                    .entity(sjk_runtime::EntityId::new(u64::from(number) + 1))
                    .map(|entity| entity.sample(i64::from(time)).translation);
                if let Some(origin) = origin {
                    fades.push(((time - first) as u64, number, origin));
                }
            }
        }
        if fades.is_empty() {
            println!("no body fades in the demo: skipped");
            return;
        }
        for (relative, number, origin) in fades {
            let camera = demo_playback::Camera::Free {
                origin: [origin[0] + 55.0, origin[1] - 70.0, origin[2] + 30.0],
                yaw: 125.0,
                pitch: 35.0,
            };
            let Some(mut gpu) = card_outline_shots::open_demo(&demo, camera, [1600, 900]) else {
                return;
            };
            let mut shots = Vec::with_capacity(MOMENTS.len());
            for moment in MOMENTS {
                // The demo clock follows the frame clock from `ui_epoch`.
                gpu.ui_epoch =
                    std::time::Instant::now() - std::time::Duration::from_millis(relative + moment);
                shots.push(frame(&mut gpu, 2));
            }
            let name = format!("body_fade_{number}");
            println!("body {number}: {}", sheet(&shots, 2, 800, &name).display());
        }
    });
}
