//! World shots of the lagometer's modes ([`crate::hud::lagometer`]) over a recorded
//! game: the demo `SJK_TEST_DEMO` names is played without a server and frozen on its
//! first snapshot (`cl_freezeDemo`), the HUD drawing as in a match, and the network
//! samples are a steady made-up connection (a demo has no network: once a short demo
//! has run out, its frames read seconds ahead of its last snapshot). The speed is the
//! recorder's at that snapshot: standing still, the speed graph stays empty and mode 3
//! hides its number. Each mode, 1 to 3, at `cg_lagometerScale` 1 and 2, saved as a
//! full frame and as a sheet of the bottom right corners. Ignored like the other world
//! shots; skipped when `SJK_TEST_DEMO` is not set.

use super::card_outline_shots::open_demo;
use super::*;

/// A steady 60 ms connection drawn 20 to 45 ms behind its newest snapshot, one
/// snapshot held back for rate and one dropped.
fn connection() -> sjk_client::LagometerSamples {
    let mut samples = sjk_client::LagometerSamples::new();
    for sample in 0..64 {
        samples.add_frame(-20 - (sample * 7) % 26);
        match sample {
            40 => samples.add_dropped(),
            50 => samples.add_snapshot(140, 1),
            _ => samples.add_snapshot(55 + (sample * 3) % 12, 0),
        }
    }
    samples
}

#[test]
#[ignore = "renders with the GPU, the installed game data and the demo SJK_TEST_DEMO names"]
fn lagometer_modes_on_a_demo() {
    on_big_stack(|| {
        let Some(demo) = std::env::var_os("SJK_TEST_DEMO").map(PathBuf::from) else {
            println!("SJK_TEST_DEMO is not set: skipped");
            return;
        };
        let Some(mut gpu) = open_demo(&demo, demo_playback::Camera::FirstPerson, [1920, 1080])
        else {
            return;
        };
        if let Some(console) = gpu.console.as_mut() {
            console.set_cvar("cl_freezeDemo", "1");
        }
        // Let the content load and the speed history fill its columns.
        let _ = frame(&mut gpu, 70);
        let mut corners = Vec::new();
        for scale in ["1", "2"] {
            for mode in ["1", "2", "3"] {
                if let Some(console) = gpu.console.as_mut() {
                    console.set_cvar(crate::hud::lagometer::CVAR, mode);
                    console.set_cvar("cg_lagometerScale", scale);
                }
                gpu.lagometer = connection();
                let path = shoot(&mut gpu, 1, &format!("lagometer_mode{mode}_scale{scale}"));
                println!("{}", path.display());
                let image = image::open(&path).expect("the shot").to_rgba8();
                // The graph sits 48 by 144 virtual units from the bottom right; scale 2
                // and mode 3's tall speed graph reach up to about 900 px from the bottom.
                let corner = image::imageops::crop_imm(&image, 1_520, 180, 400, 900).to_image();
                corners.push(corner);
            }
        }
        println!(
            "modes 1 to 3 across, scales 1 and 2 down: {}",
            sheet(&corners, 3, 400, "lagometer_modes").display()
        );
    });
}
