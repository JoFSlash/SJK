//! World shots of card scenes ([`crate::cards::scene`]): each scene of [`SCENES`] drawn as
//! a client draws a card's picture from the game's models, kept as the JPEG a card would
//! show in `target/world-shots/card-scenes/`, and a contact sheet of them all
//! (`card-scenes.png`). Ignored like the other world shots.

use super::*;
use crate::cards::scene;

/// (file name, scene) for the shot.
const SCENES: &[(&str, &str)] = &[
    ("kyle", "player kyle"),
    ("luke", "player luke"),
    ("desann", "player desann"),
    ("students", "player jedi_tf + player jedi_zf"),
    ("rancor", "player rancor"),
    ("r2d2", "player r2d2"),
    ("push", "player jedi_hm anim=BOTH_FORCEPUSH at=0.5"),
    ("e11", "model models/weapons2/blaster_r/blaster.md3"),
    (
        "disruptor",
        "model models/weapons2/disruptor/disruptor_w.glm",
    ),
    (
        "bowcaster",
        "model models/weapons2/bowcaster/bowcaster.md3 yaw=90 pitch=50",
    ),
    ("bacta", "model models/map_objects/mp/bacta.md3"),
    (
        "luke-saber",
        "player luke anim=BOTH_STAND2 saber=luke blade=green",
    ),
];

#[test]
#[ignore = "renders with the GPU and the installed game data named by JKA_GAME_DATA"]
fn card_scenes_drawn_from_the_game_models() {
    on_big_stack(|| {
        let Some((mut gpu, _profile)) = open("maps/mp/duel6.bsp", [640, 360], None, &[]) else {
            return;
        };
        crate::cards::faces::decode_inline_for_shots();
        let folder = directory().join("card-scenes");
        std::fs::create_dir_all(&folder).unwrap();
        // One scene (`SJK_SCENE`), a file of them (`SJK_SCENES`: a name, a tab and a
        // scene a line), else [`SCENES`].
        let wanted: Vec<(String, String)> = if let Ok(text) = std::env::var("SJK_SCENE") {
            vec![("custom".to_owned(), text)]
        } else if let Ok(file) = std::env::var("SJK_SCENES") {
            std::fs::read_to_string(file)
                .unwrap()
                .lines()
                .filter_map(|line| line.split_once('\t'))
                .map(|(name, text)| (name.trim().to_owned(), text.trim().to_owned()))
                .collect()
        } else {
            SCENES
                .iter()
                .map(|(name, text)| ((*name).to_owned(), (*text).to_owned()))
                .collect()
        };
        for (name, text) in &wanted {
            let path = folder.join(format!("{name}.jpg"));
            let _ = std::fs::remove_file(&path);
            scene::want(scene::Job {
                id: name.clone(),
                scene: scene::parse(text).unwrap(),
                path,
                tint: [166, 77, 255],
            });
        }
        let mut frames = 0;
        while scene::pending() && frames < 6_000 {
            let _ = gpu.render(&mut None);
            frames += 1;
        }
        // Sheets of twelve, in the order asked; a scene not drawn leaves its cell black.
        for (page, chunk) in wanted.chunks(12).enumerate() {
            let mut images = Vec::new();
            for (name, text) in chunk {
                match image::open(folder.join(format!("{name}.jpg"))) {
                    Ok(picture) => images.push(picture.into_rgba8()),
                    Err(error) => {
                        println!("{name} ({text}): not drawn: {error}");
                        images.push(image::RgbaImage::new(scene::SIZE[0], scene::SIZE[1]));
                    }
                }
            }
            let name = format!("card-scenes-{}", page + 1);
            println!("{}", sheet(&images, 4, 510, &name).display());
        }
    });
}
