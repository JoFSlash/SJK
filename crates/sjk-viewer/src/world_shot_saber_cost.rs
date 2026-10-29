//! What skinned blades cost to draw (hub reports 63 and 64, 17/10/2026): on duel6, a
//! Spectral blade held up close as in first person, the same swinging past with its
//! afterimages, and the Sun up close, each saved as a shot (`duel6-skin-cost-<scene>`)
//! and timed over [`FRAMES`] frames in each of [`ROUNDS`] rounds, every frame waited for on the GPU; the same view
//! with no blade gives the frame's cost without them. The Sun's swing (26/10/2026, Sol: the
//! frames drop as it swings) is timed too: a swing sweeping across the view up close, worn as
//! the stock blue blade (the blur), the Sun (its plasma) and the Sun with its extras (the
//! Flare trail, the Heat wake; Swing embers come from live swings only, not from these
//! laid trails). `SJK_COST_4K` draws at 3840x2160. Wall-clock times of whole frames,
//! so only the difference to the empty view says something, and only on this machine.
//! The skins come from `SJK_TEST_PACKS` ([`super::saber_skins::mount_test_packs`]).

use super::*;
use crate::saber::Blade;
use crate::saber_skins::ShotColor;

/// The shots' size: a 1440p screen, where a blade up close covers a million pixels.
const SIZE: [u32; 2] = [2560, 1440];
/// The same at 4K (`SJK_COST_4K`).
const SIZE_4K: [u32; 2] = [3840, 2160];
/// Frames timed a scene.
const FRAMES: usize = 100;
/// Rounds over the scenes.
const ROUNDS: usize = 5;

#[test]
#[ignore = "renders with the GPU and the installed game data named by JKA_GAME_DATA"]
fn duel6_saber_skin_cost() {
    on_big_stack(|| {
        super::saber_skins::mount_test_packs();
        // `SJK_COST_NO_GLOW` draws without the dynamic glow, for what that pass costs.
        let glow = if std::env::var_os("SJK_COST_NO_GLOW").is_some() {
            "0"
        } else {
            "1"
        };
        let cvars = [("r_DynamicGlow", glow)];
        let size = if std::env::var_os("SJK_COST_4K").is_some() {
            SIZE_4K
        } else {
            SIZE
        };
        let Some((mut gpu, _profile)) = open("maps/mp/duel6.bsp", size, None, &cvars) else {
            return;
        };
        let (spawn, yaw) = assets::initial_camera(&gpu.bsp).expect("a spawn point");
        let turn = glam::Quat::from_rotation_z(yaw);
        let eye = Vec3::from_array(spawn);
        let ahead = |x: f32, y: f32, z: f32| eye + turn * Vec3::new(x, y, z);
        let blade = |base: Vec3, toward: Vec3| Blade {
            base: base.to_array(),
            direction: (turn * toward).normalize().to_array(),
            length: 40.0,
            radius: 3.0,
        };
        let (look_yaw, look_pitch) = look(eye.to_array(), ahead(40.0, 0.0, -4.0).to_array());
        aim(&mut gpu, eye.to_array(), look_yaw, look_pitch);
        gpu.saber_skins.shot_persona = crate::saber_persona::Persona::of(b"Sol", 0);
        // Held as in first person: from below the view's right, up and across it.
        let held = blade(ahead(9.0, -7.0, -12.0), Vec3::new(1.0, 0.55, 0.9));
        // Swinging past a little further: the poses it left 45 ms apart.
        let swung = blade(ahead(30.0, -14.0, -16.0), Vec3::new(0.3, 0.6, 0.75));
        let ghosts: Vec<_> = [0.45_f32, 0.2025, 0.091, 0.041]
            .into_iter()
            .enumerate()
            .map(|(back, fade)| {
                let lean = 0.6 - 0.22 * (back as f32 + 1.0);
                (
                    blade(ahead(30.0, -14.0, -16.0), Vec3::new(0.3, lean, 0.75)),
                    "saber_spectral",
                    fade,
                )
            })
            .collect();
        // A swing across the view up close: 15 poses 8 ms apart, from the right over to the
        // left, ending as the frame is drawn.
        let sweep: Vec<Blade> = (0..15)
            .map(|step| {
                let angle = (160.0 - 10.0 * step as f32).to_radians();
                blade(
                    ahead(16.0, 0.0, -14.0),
                    Vec3::new(0.25, angle.cos(), angle.sin()),
                )
            })
            .collect();
        let swing = |color: ShotColor| {
            (
                vec![(*sweep.last().unwrap(), color, 2)],
                vec![(sweep.clone(), color)],
            )
        };
        let sun = |off| ShotColor::Parts("saber_sun", off);
        use crate::blade_skin_file::extras::option_bit;
        let extras =
            option_bit("flare_trail") | option_bit("heat_wake") | option_bit("swing_embers");
        let swings = [
            (
                "blue-swing",
                swing(ShotColor::Stock(crate::saber_rgb::BladeColor::Retail(
                    crate::saber::Color::Blue,
                ))),
            ),
            ("sun-swing", swing(sun(0))),
            ("sun-swing-flare", swing(sun(option_bit("flare_trail")))),
            ("sun-swing-wake", swing(sun(option_bit("heat_wake")))),
            ("sun-swing-extras", swing(sun(extras))),
        ];
        let mut scenes: Vec<(&str, Vec<_>, Vec<_>, Vec<_>)> = vec![
            ("empty", vec![], vec![], vec![]),
            (
                "spectral-close",
                vec![(held, ShotColor::Skin("saber_spectral"), 2)],
                vec![],
                vec![],
            ),
            (
                "spectral-ghosts",
                vec![(swung, ShotColor::Skin("saber_spectral"), 2)],
                ghosts,
                vec![],
            ),
            (
                "sun-close",
                vec![(held, ShotColor::Skin("saber_sun"), 2)],
                vec![],
                vec![],
            ),
        ];
        // The held Sun with one part off at a time, then all of them, for what each costs.
        let parts = [
            ("sun-close-no-corona", "sunspots_and_corona"),
            ("sun-close-no-haze", "haze"),
            ("sun-close-no-prominences", "prominences"),
            ("sun-close-no-sparks", "sparks"),
            ("sun-close-no-glint", "glint"),
        ];
        for (name, part) in parts {
            let off = crate::saber_skin_options::bits([part]);
            scenes.push((name, vec![(held, sun(off), 2)], vec![], vec![]));
        }
        let all = crate::saber_skin_options::bits([
            "sunspots_and_corona",
            "haze",
            "prominences",
            "sparks",
            "glint",
        ]);
        scenes.push((
            "sun-close-no-parts",
            vec![(held, sun(all), 2)],
            vec![],
            vec![],
        ));
        for (name, (blades, trails)) in swings {
            scenes.push((name, blades, vec![], trails));
        }
        // Each scene shot once, then timed in turn for [`ROUNDS`] rounds, so a change of
        // the GPU's clocks during the run touches every scene alike; the best round counts.
        let mut times: Vec<(&str, f64, f64)> = Vec::new();
        for (name, blades, ghosts, trails) in &scenes {
            gpu.saber_skins.shot_blades = blades.clone();
            gpu.saber_skins.shot_ghosts = ghosts.clone();
            gpu.saber_skins.shot_swings = trails.clone();
            gpu.saber_skins.shot_seconds = Some(3.0);
            let shot = frame(&mut gpu, 30);
            std::fs::create_dir_all(directory()).expect("the shot directory");
            let path = directory().join(format!("duel6-skin-cost-{name}.png"));
            shot.save(&path).expect("write the shot");
            println!("{}", path.display());
            times.push((*name, f64::MAX, f64::MAX));
        }
        for _ in 0..ROUNDS {
            for ((_, blades, ghosts, trails), time) in scenes.iter().zip(&mut times) {
                gpu.saber_skins.shot_blades = blades.clone();
                gpu.saber_skins.shot_ghosts = ghosts.clone();
                gpu.saber_skins.shot_swings = trails.clone();
                let mut frames: Vec<f64> = (0..FRAMES)
                    .map(|_| {
                        let start = std::time::Instant::now();
                        let _ = gpu.render(&mut None);
                        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
                        start.elapsed().as_secs_f64() * 1_000.0
                    })
                    .collect();
                frames.sort_by(f64::total_cmp);
                time.1 = time.1.min(frames[FRAMES / 2]);
                time.2 = time.2.min(frames[FRAMES / 10]);
            }
        }
        let empty = times[0].1;
        for (name, median, fast) in times {
            println!(
                "{name:16} median {median:6.3} ms (fastest tenth {fast:6.3} ms; best round), \
                 {:+6.3} ms over the empty view",
                median - empty
            );
        }
    });
}
