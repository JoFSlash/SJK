//! What the lamps that only glow light (`r_lampLights`, `world_lamp_gain.rs`): Sol's
//! world notes of 12-13/10/2026 about `byss/lights`, `byss/biglight`,
//! `bounty/light` (`mp/ffa1`), `taspir/light_ceiling`, `taspir/t_wall5` and
//! `flares/solid_red` (`mp/ffa5`), `rooftop/groundlight` (`t2_rogue`) and
//! `flares/solid_blue` (`t2_rancor`), each seen from beside the note's eye toward
//! the floor under the lamp, so what the lamp lights is in view rather than the
//! lamp's own face. One sheet per map, a row per view and a column per multiplier
//! (`SJK_LAMP_GAINS`, `0,1,4,8` by default; each loads the map again, since the
//! multiplier applies when a map loads). `SJK_LAMP_CVARS` (`name=value,...`) sets
//! renderer settings (real-time lighting needs `r_dayNight=1`); `SJK_LAMP_MAPS`
//! picks maps by part of their name.
//!
//! `SJK_LAMP_TIMING=1` (with `SJK_GPU_PHASES=1`, in a release build) also holds each
//! view for 640 frames (`SJK_LAMP_FRAMES`) after a `timing-view <map> <view> gain <gain>` line on stderr,
//! so the `gpu-phases` lines that follow (every 32nd frame) can be grouped by view and
//! multiplier: the light pass is the sum of the `light-...` phases.

use super::*;
use glam::Vec3;

/// One view: the lamp's noted point, where the note's eye stood and the floor height
/// under it (the note's `setviewpos` height less the player's 24 units).
struct View {
    name: &'static str,
    lamp: [f32; 3],
    eye: [f32; 3],
    floor: f32,
}

const MAPS: &[(&str, &[View])] = &[
    (
        "maps/mp/ffa1.bsp",
        &[
            View {
                name: "byss-lights-16",
                lamp: [4144.1, 87.3, 1911.0],
                eye: [4118.3, 61.3, 1732.1],
                floor: 1672.0,
            },
            View {
                name: "byss-lights-17",
                lamp: [2996.8, 836.6, 2824.0],
                eye: [3570.2, 1021.5, 2329.4],
                floor: 2269.0,
            },
            View {
                name: "byss-lights-38-blue",
                lamp: [3052.8, 612.9, 1208.0],
                eye: [3079.7, 626.9, 1120.6],
                floor: 1061.0,
            },
            View {
                name: "byss-lights-39",
                lamp: [3522.2, 334.1, 2824.0],
                eye: [3595.5, 649.8, 2312.6],
                floor: 2253.0,
            },
            View {
                name: "byss-biglight-40",
                lamp: [2627.8, 777.2, 2011.0],
                eye: [2412.7, 708.0, 1815.8],
                floor: 1756.0,
            },
            View {
                name: "bounty-light-42",
                lamp: [3837.3, 833.6, 2214.9],
                eye: [3514.1, 797.6, 2187.9],
                floor: 2128.0,
            },
        ],
    ),
    (
        "maps/mp/ffa5.bsp",
        &[
            View {
                name: "taspir-t_wall5-24",
                lamp: [-1530.6, -928.0, 754.2],
                eye: [-1636.9, -1116.0, 530.6],
                floor: 471.0,
            },
            View {
                name: "taspir-light_ceiling-26",
                lamp: [-1022.2, -1471.2, 696.0],
                eye: [-1024.2, -1544.8, 516.1],
                floor: 456.0,
            },
            View {
                name: "flares-solid_red-34",
                lamp: [-823.1, -4.0, 131.0],
                eye: [-819.2, -64.6, 124.1],
                floor: 64.0,
            },
        ],
    ),
    (
        "maps/t2_rogue.bsp",
        &[View {
            name: "rooftop-groundlight-3",
            lamp: [2411.6, 1466.9, -371.6],
            eye: [2319.0, 1481.7, -587.9],
            floor: -648.0,
        }],
    ),
    (
        "maps/t2_rancor.bsp",
        &[View {
            name: "flares-solid_blue-14",
            lamp: [-2834.7, 2228.4, 1173.4],
            eye: [-2840.4, 2258.7, 1163.2],
            floor: 1103.0,
        }],
    ),
];

/// Every map's views at every multiplier, one sheet per map in `target/world-shots`.
#[test]
#[ignore = "renders with the GPU and the installed game data named by JKA_GAME_DATA"]
fn lamp_light() {
    on_big_stack(|| {
        let gains: Vec<String> = std::env::var("SJK_LAMP_GAINS")
            .unwrap_or_else(|_| "0,1,4,8".to_owned())
            .split(',')
            .map(|gain| gain.trim().to_owned())
            .collect();
        let settings = std::env::var("SJK_LAMP_CVARS").unwrap_or_default();
        let only = std::env::var("SJK_LAMP_MAPS").unwrap_or_default();
        let timing = std::env::var_os("SJK_LAMP_TIMING").is_some();
        let frames: u32 = std::env::var("SJK_LAMP_FRAMES")
            .ok()
            .and_then(|frames| frames.parse().ok())
            .unwrap_or(640);
        // The player's generated packs, as the game mounts them.
        if let Some(game_data) = std::env::var_os("JKA_GAME_DATA") {
            crate::assets::generated::locate_at_start(&PathBuf::from(game_data).join("SJK"));
        }
        for (map, views) in MAPS {
            if !only.is_empty() && !only.split(',').any(|part| map.contains(part.trim())) {
                continue;
            }
            // Row-major: one row per view, one column per multiplier.
            let mut cells: Vec<Vec<image::RgbaImage>> = views.iter().map(|_| Vec::new()).collect();
            for gain in &gains {
                let mut cvars: Vec<(&str, &str)> = settings
                    .split(',')
                    .filter_map(|pair| pair.split_once('='))
                    .collect();
                cvars.push((crate::world_materials::emission::lamp_gain::CVAR, gain));
                let Some((mut gpu, _profile)) = open(map, [1280, 720], None, &cvars) else {
                    return;
                };
                let _ = frame(&mut gpu, 30);
                for (index, view) in views.iter().enumerate() {
                    let lamp = Vec3::from_array(view.lamp);
                    let eye = Vec3::from_array(view.eye);
                    // Back from the lamp and a little up, toward the floor under it.
                    let away = Vec3::new(eye.x - lamp.x, eye.y - lamp.y, 0.).normalize_or(Vec3::X);
                    let from = eye + away * 120. + Vec3::Z * 40.;
                    let at = [lamp.x, lamp.y, view.floor];
                    let (yaw, pitch) = look(from.to_array(), at);
                    aim(&mut gpu, from.to_array(), yaw, pitch);
                    if timing {
                        // The profiler prints a `gpu-phases` line every 32nd frame
                        // (`SJK_GPU_PHASES`); this line tells which view they belong to.
                        eprintln!("timing-view {map} {} gain {gain}", view.name);
                        for _ in 0..frames {
                            let _ = gpu.render(&mut None);
                            // Headless, nothing else waits for the GPU: without this the
                            // frames queue up (out of memory after a thousand) and the
                            // timestamp readback never lands.
                            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
                        }
                    }
                    cells[index].push(frame(&mut gpu, 12));
                }
            }
            let images: Vec<image::RgbaImage> = cells.into_iter().flatten().collect();
            let name = format!(
                "lamp-light-{}",
                map.trim_start_matches("maps/")
                    .trim_end_matches(".bsp")
                    .replace('/', "-")
            );
            println!(
                "{map} (gains {}): {}",
                gains.join(","),
                sheet(&images, gains.len() as u32, 640, &name).display()
            );
        }
    });
}
