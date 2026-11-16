//! The menu pictures ([`crate::menu_pictures`]) and the shots that review them.
//!
//! `menu_pictures_generate` takes the pictures: for each set's map, every view its
//! backdrop has ([`crate::menu_backdrop::Backdrop::picture_views`]) rendered off
//! screen at [`crate::menu_pictures::SIZE`], written as a JPEG into
//! `crates/sjk-viewer/assets/menu_pictures/<map>/` with the manifest the client reads
//! (`pictures.txt`: each view's camera, the field of view). Run it again after a
//! route, the tour or the renderer's look changes:
//!
//! ```text
//! JKA_GAME_DATA=<GameData> cargo test -p sjk-viewer --release menu_pictures_generate -- --ignored
//! ```
//!
//! The scene bloom is off while they are taken: the client blooms the picture again
//! as it draws it, so the bloom is not doubled.

use crate::world_shot::*;
use crate::{GpuState, menu, menu_backdrop, menu_pictures};

/// Render frames until the picture the menu wants is decoded and faded in.
fn until_settled(gpu: &mut GpuState) {
    for _ in 0..2_000 {
        let _ = frame(gpu, 1);
        if gpu.menu_pictures.settled() {
            return;
        }
    }
    panic!("the menu picture never settled");
}

/// The JPEG quality the pictures are saved at.
const QUALITY: u8 = 85;

#[test]
#[ignore = "renders with the GPU and the installed game data named by JKA_GAME_DATA"]
fn menu_pictures_generate() {
    on_big_stack(|| {
        for set in menu_pictures::SETS {
            let cvars = [("r_sceneBloom", "0"), (crate::version_overlay::CVAR, "0")];
            let Some((mut gpu, _profile)) = open(set.map, menu_pictures::SIZE, None, &cvars) else {
                return;
            };
            let backdrop =
                menu_backdrop::Backdrop::from_bsp(&gpu.bsp).expect("the map's menu backdrop");
            let message = menu_backdrop::worldspawn_message(&gpu.bsp).expect("a message");
            let directory = menu_pictures::source_directory(set);
            std::fs::create_dir_all(&directory).expect("the picture directory");
            let _ = frame(&mut gpu, 30);
            let fov = gpu.field_of_view;
            let mut manifest = format!(
                "# The menu pictures of {}, written by world_shot_menu_pictures.rs: do not edit.\n\
                 message {message}\nfov {fov}\nsize {} {}\n",
                set.map,
                menu_pictures::SIZE[0],
                menu_pictures::SIZE[1],
            );
            for (view, vantage) in backdrop.picture_views() {
                gpu.camera_position = vantage.origin;
                gpu.camera_yaw = vantage.yaw;
                gpu.camera_pitch = vantage.pitch();
                let image = frame(&mut gpu, 12);
                let name = view.name();
                let path = directory.join(format!("{name}.jpg"));
                let rgb = image::DynamicImage::ImageRgba8(image).to_rgb8();
                let file = std::fs::File::create(&path).expect("the picture file");
                image::codecs::jpeg::JpegEncoder::new_with_quality(
                    std::io::BufWriter::new(file),
                    QUALITY,
                )
                .encode_image(&rgb)
                .expect("write the picture");
                let origin = vantage.origin;
                manifest.push_str(&format!(
                    "view {name} {} {} {} {} {}\n",
                    origin.x,
                    origin.y,
                    origin.z,
                    vantage.yaw,
                    vantage.pitch()
                ));
                println!(
                    "{} ({} KiB)",
                    path.display(),
                    std::fs::metadata(&path).map_or(0, |meta| meta.len() / 1024)
                );
            }
            std::fs::write(directory.join("pictures.txt"), manifest).expect("the manifest");
        }
    });
}

/// The picture menu as a player sees it, in the SJK UI on duel6's pictures: the
/// main page, the player stage, the Holocrons tab; and the classic style's
/// main menu on ffa3's.
#[test]
#[ignore = "renders with the GPU and the installed game data named by JKA_GAME_DATA"]
fn menu_pictures_shots() {
    on_big_stack(|| {
        for (style, map, name) in [
            ("sjk", "maps/mp/duel6.bsp", "menu-pictures-sjk"),
            ("classic", "maps/mp/ffa3.bsp", "menu-pictures-classic"),
        ] {
            let menu = menu::ClientMenu::new(true, String::new());
            let cvars = [
                ("ui_menuStyle", style),
                (crate::settings::quick::HIDE_CVAR, "1"),
                (menu_pictures::CVAR, "1"),
            ];
            let map = menu_pictures::menu_map(map, true);
            let Some((mut gpu, _profile)) = open(&map, [1920, 1080], Some(menu), &cvars) else {
                return;
            };
            gpu.graphics_reload.map = map;
            menu_pictures::fit_backdrop(&mut gpu);
            until_settled(&mut gpu);
            println!("{}", shoot(&mut gpu, 2, name).display());
            if let (Some(menu), Some(console)) = (gpu.client_menu.as_mut(), gpu.console.as_ref()) {
                menu.open_player(console, crate::player_menu::ReturnTarget::MainMenu);
            }
            let _ = frame(&mut gpu, 2);
            until_settled(&mut gpu);
            println!(
                "{}",
                shoot(&mut gpu, 2, &format!("{name}-player")).display()
            );
        }
    });
}

/// A wider and a narrower screen than the pictures: the stage still stands where
/// the picture shows it.
#[test]
#[ignore = "renders with the GPU and the installed game data named by JKA_GAME_DATA"]
fn menu_pictures_aspects() {
    on_big_stack(|| {
        for (size, name) in [
            ([2560, 1080], "menu-pictures-ultrawide"),
            ([1440, 1080], "menu-pictures-4-3"),
        ] {
            let menu = menu::ClientMenu::new(true, String::new());
            let cvars = [
                ("ui_menuStyle", "sjk"),
                (crate::settings::quick::HIDE_CVAR, "1"),
                (menu_pictures::CVAR, "1"),
            ];
            let map = menu_pictures::menu_map("maps/mp/duel6.bsp", true);
            let Some((mut gpu, _profile)) = open(&map, size, Some(menu), &cvars) else {
                return;
            };
            gpu.graphics_reload.map = map;
            menu_pictures::fit_backdrop(&mut gpu);
            if let (Some(menu), Some(console)) = (gpu.client_menu.as_mut(), gpu.console.as_ref()) {
                menu.open_player(console, crate::player_menu::ReturnTarget::MainMenu);
            }
            let _ = frame(&mut gpu, 2);
            until_settled(&mut gpu);
            println!("{}", shoot(&mut gpu, 2, name).display());
        }
    });
}
