//! World shots of the capture-the-flag HUD ([`crate::hud::flag_status`]) on a recorded
//! game: the demo `SJK_TEST_DEMO` names is played without a server, with a made-up CTF
//! state staged over it (the viewer carrying the blue flag on red; then on blue, the red
//! flag dropped and its own flag taken; following a carrier; team FFA with long scores),
//! in each HUD style. Ignored like the other world shots; skipped when `SJK_TEST_DEMO`
//! is not set.

use super::card_outline_shots::open_demo;
use super::*;
use crate::hud::flag_status::{Flag, Staged};

#[test]
#[ignore = "renders with the GPU, the installed game data and the demo SJK_TEST_DEMO names"]
fn ctf_flags_on_a_demo() {
    on_big_stack(|| {
        let Some(demo) = std::env::var_os("SJK_TEST_DEMO").map(PathBuf::from) else {
            println!("SJK_TEST_DEMO is not set: skipped");
            return;
        };
        let states = [
            (
                "carry",
                Staged {
                    team: 1,
                    flags: [Flag::Taken, Flag::Taken],
                    carried: true,
                    scores: [3, 5],
                    ysalamiri: false,
                    ctf: true,
                    following: false,
                },
            ),
            (
                "dropped",
                Staged {
                    team: 2,
                    flags: [Flag::Dropped, Flag::Taken],
                    carried: false,
                    scores: [12, 9],
                    ysalamiri: false,
                    ctf: true,
                    following: false,
                },
            ),
            (
                "following",
                Staged {
                    team: 1,
                    flags: [Flag::AtBase, Flag::Taken],
                    carried: true,
                    scores: [104, 99],
                    ysalamiri: false,
                    ctf: true,
                    following: true,
                },
            ),
            (
                "tffa",
                Staged {
                    team: 1,
                    flags: [Flag::AtBase; 2],
                    carried: false,
                    scores: [300, -12],
                    ysalamiri: false,
                    ctf: false,
                    following: false,
                },
            ),
        ];
        for style in ["game", "classic", "radial"] {
            let Some(mut gpu) = open_demo(&demo, demo_playback::Camera::FirstPerson, [1920, 1080])
            else {
                return;
            };
            if let Some(console) = gpu.console.as_mut() {
                console.set_cvar("cg_hudStyle", style);
            }
            for (name, staged) in states {
                gpu.hud.flag_status.stage_for_shot(staged);
                let _ = frame(&mut gpu, 20);
                println!(
                    "{}",
                    shoot(&mut gpu, 20, &format!("ctf_flags_{style}_{name}")).display()
                );
            }
        }
    });
}
