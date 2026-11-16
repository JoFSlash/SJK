//! Card art for the SJK Blades set: one picture per blade skin a holocron may drop, the
//! lit blade drawn by the game's own saber renderer (its effects, extras and light) on
//! the dark of the void beyond duel6, laid diagonally across a 1020 x 850 frame (a card's
//! 6:5 art window). Each picture is rendered at twice the size and scaled down.
//!
//! With `SJK_BLADE_CARDS` set to a folder, writes there `cards/blade<NNN>.png` and the
//! card's text file `cards/blade<NNN>.txt` in the hub's `import_cards.py` format (its
//! number, tier, name, smith and description from [`crate::unlockables`]);
//! `SJK_BLADE_CARD_SKINS` (ids without `saber_`, comma-separated) picks which. The blade
//! skins come from the packs `SJK_TEST_PACKS` names. Ignored like the other world shots.

use super::*;
use crate::saber::Blade;
use crate::saber_skins::ShotColor;

/// The card picture's size, and the size it is rendered at before scaling down.
const CARD: [u32; 2] = [1020, 850];
const RENDER: [u32; 2] = [CARD[0] * 2, CARD[1] * 2];
/// The blade's tilt from the horizontal, degrees, and how far from the camera it lies.
const TILT: f32 = 52.0;
const DISTANCE: f32 = 27.0;
/// A fixed exposure and no fog, so every card is lit alike; no Discord status.
const CVARS: [(&str, &str); 5] = [
    ("r_autoExposure", "0"),
    ("r_hdrExposure", "0.6"),
    ("r_volumetrics", "0"),
    ("r_drawfog", "0"),
    ("cl_discordPresence", "0"),
];
/// The animation time the blades are drawn at, seconds.
const SECONDS: f64 = 3.4;

#[test]
#[ignore = "renders with the GPU and the installed game data named by JKA_GAME_DATA"]
fn blade_card_art() {
    let Some(folder) = std::env::var_os("SJK_BLADE_CARDS").map(PathBuf::from) else {
        println!("SJK_BLADE_CARDS is not set: no card art written");
        return;
    };
    on_big_stack(move || {
        saber_skins::mount_test_packs();
        let picked: Option<Vec<String>> = std::env::var("SJK_BLADE_CARD_SKINS").ok().map(|list| {
            list.split(',')
                .map(|id| format!("saber_{}", id.trim()))
                .collect()
        });
        let skins: Vec<_> = crate::unlockables::blade_skins()
            .filter(|skin| skin.droppable)
            .filter(|skin| {
                picked
                    .as_ref()
                    .is_none_or(|ids| ids.iter().any(|id| id == skin.id))
            })
            .collect();
        let cards = folder.join("cards");
        std::fs::create_dir_all(&cards).expect("the cards folder");
        let Some((mut gpu, _profile)) = open("maps/mp/duel6.bsp", RENDER, None, &CVARS) else {
            return;
        };
        // Beyond the map's walls nothing of the world is drawn: the night tint behind.
        let maximums = Vec3::from_array(gpu.bsp.render().models()[0].maximums);
        let eye = maximums + Vec3::new(4_000.0, 4_000.0, 2_000.0);
        aim(&mut gpu, eye.to_array(), 0.0, 0.0);
        // Looking along +X: the screen's right is -Y, its up +Z.
        let (right, up) = (Vec3::NEG_Y, Vec3::Z);
        let direction =
            (right * TILT.to_radians().cos() + up * TILT.to_radians().sin()).normalize();
        let length = 40.0;
        let middle = eye + Vec3::X * DISTANCE;
        let blade = Blade {
            base: (middle - direction * (length * 0.5)).to_array(),
            direction: direction.to_array(),
            length,
            radius: 3.0,
        };
        gpu.saber_skins.shot_seconds = Some(SECONDS);
        let _ = frame(&mut gpu, 30);
        gpu.saber_gpu.build_every_signature();
        // Signature skins are drawn from their files until their own pipelines are built
        // off the render thread: wait for every one.
        let started = std::time::Instant::now();
        loop {
            let (built, loaded) = gpu.saber_gpu.signatures_built();
            if loaded > 0 && built == loaded {
                println!("{built} signature shaders built");
                break;
            }
            assert!(
                started.elapsed().as_secs() < 3_000,
                "signature shaders still building: {built} of {loaded}"
            );
            let _ = frame(&mut gpu, 10);
        }
        for skin in skins {
            gpu.saber_skins.shot_blades = vec![(blade, ShotColor::Skin(skin.id), 2)];
            let image = frame(&mut gpu, 6);
            let image = image::imageops::resize(
                &on_backdrop(image),
                CARD[0],
                CARD[1],
                image::imageops::FilterType::Lanczos3,
            );
            let name = format!("blade{}", skin.number_text());
            image::DynamicImage::ImageRgba8(image)
                .to_rgb8()
                .save(cards.join(format!("{name}.png")))
                .expect("write the card art");
            std::fs::write(cards.join(format!("{name}.txt")), card_text(skin, &name))
                .expect("write the card text");
            println!("{name} {} {}", skin.id, skin.tier.label());
        }
    });
}

/// The card's text file for `import_cards.py`.
fn card_text(skin: &crate::unlockables::Unlockable, name: &str) -> String {
    format!(
        "number: {}\ncollection: SJK Blades\nkind: blade\nrarity: {}\ntitle: {}\nart: {name}.png\nartist: {}\nblade: {}\n\ndescription: {}\n",
        skin.number,
        skin.tier.label().to_lowercase(),
        skin.name,
        skin.smith,
        skin.id,
        skin.description,
    )
}

/// `image` with the void's flat tint (its border's median) taken out and a dark backdrop
/// put under the blade: a cool slate lifting a little toward the middle. The blades add
/// their light, so taking the tint out leaves them as the game draws them.
fn on_backdrop(mut image: image::RgbaImage) -> image::RgbaImage {
    let (width, height) = image.dimensions();
    let mut border: [Vec<u8>; 3] = Default::default();
    for x in (0..width).step_by(7) {
        for y in [0, 1, height - 2, height - 1] {
            let pixel = image.get_pixel(x, y);
            for channel in 0..3 {
                border[channel].push(pixel[channel]);
            }
        }
    }
    let tint = border.map(|mut values| {
        values.sort_unstable();
        f32::from(values[values.len() / 2])
    });
    let (middle_x, middle_y) = (width as f32 * 0.5, height as f32 * 0.5);
    let reach = (middle_x * middle_x + middle_y * middle_y).sqrt();
    let centre = [20.0, 22.0, 32.0];
    let edge = [5.0, 5.0, 8.0];
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let (dx, dy) = (x as f32 - middle_x, y as f32 - middle_y);
        let t = ((dx * dx + dy * dy).sqrt() / reach).min(1.0);
        let t = t * t * (3.0 - 2.0 * t);
        for channel in 0..3 {
            let backdrop = centre[channel] + (edge[channel] - centre[channel]) * t;
            let lit = (f32::from(pixel[channel]) - tint[channel]).max(0.0);
            pixel[channel] = (lit + backdrop).round().min(255.0) as u8;
        }
    }
    image
}
