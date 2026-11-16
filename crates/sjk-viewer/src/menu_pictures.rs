//! Menu pictures (`ui_menuPictures`): the menus stand on pictures of their map
//! instead of the live map, to save video memory.
//!
//! The live menu world holds a whole map on the graphics card (its textures,
//! lightmaps, light caches), and joining a server builds the server's map while
//! it is still on show, so a card short of memory runs out on the join. With the
//! setting on, the menu world is built from an empty map ([`empty_world`]) and the
//! backdrop shows pictures taken beforehand from each view its camera would fly
//! to ([`crate::menu_backdrop::pictures`]): a 3840x2160 JPEG a view, bundled with
//! the client (`assets/menu_pictures/<map>/`, `pictures.txt` naming each view's
//! camera). The Collection's saber shader swatches show their drawings instead of
//! the game's live blade pictures ([`crate::blade_pictures::set_live`]). The player on
//! the stage, the thrown saber and the holocron are still drawn in 3D, standing where the picture shows the stage. [`Painter`] draws the
//! picture first in the world pass, so the menus, bloom and gamma go on top as on
//! the live map. `world_shot_menu_pictures.rs` takes the pictures again.
//!
//! A map that fails to build because the card ran out of memory
//! ([`is_out_of_memory`]) turns the setting on ([`after_out_of_memory`]), and the
//! failure screen says so; once it is on, each such failure lowers the texture
//! detail (`r_picmip`) a step instead, down to 2. The pictures take effect the next
//! time the menu world is built: at once in the menus, after a match when the menus
//! come back.
//!
//! A picture menu world is named by its map path with [`PREFIX`]
//! (`pictures:maps/mp/duel6.bsp`), so the menu world's map, kept to build it again
//! after a match ([`crate::menu_world`]), says which kind it is.

#[path = "menu_pictures_paint.rs"]
mod paint;

use crate::menu_backdrop::{Backdrop, Vantage, pictures::View};
pub(crate) use paint::Painter;

/// The setting: 1 stands the menus on pictures.
pub(crate) const CVAR: &str = "ui_menuPictures";
/// The size the pictures are taken at.
#[cfg(test)]
pub(crate) const SIZE: [u32; 2] = [3840, 2160];
/// What a picture menu world's map path starts with.
const PREFIX: &str = "pictures:";

/// Register `ui_menuPictures` (default 0, archived), which the blade pictures follow,
/// and the small-card check's [`crate::video_memory::CVAR`].
pub(crate) fn register(cvars: &mut sjk_shell::CvarRegistry) -> Result<(), sjk_shell::CvarError> {
    crate::video_memory::register(cvars)?;
    cvars.register(sjk_shell::CvarDefinition::new(
        CVAR,
        0_i64,
        sjk_shell::CvarFlags::ARCHIVE,
        "Stand the menus on pictures of their map instead of the live map, which saves video \
         memory (0 live map, 1 pictures); turned on by itself when a map runs out of video memory",
    ))?;
    cvars.on_change(CVAR, |change| {
        if let sjk_shell::CvarValue::Integer(value) = change.current {
            crate::blade_pictures::set_live(value == 0);
        }
    })
}

/// Whether the menus stand on pictures.
pub(crate) fn enabled(console: Option<&crate::console::ViewerConsole>) -> bool {
    console
        .and_then(|console| console.integer_cvar(CVAR))
        .is_some_and(|value| value != 0)
}

/// The menu world's map path for `map` (a picture one or not) with the pictures on or
/// off; a map without a picture set stays live.
pub(crate) fn menu_map(map: &str, pictures: bool) -> String {
    let live = pictured(map).unwrap_or(map);
    if pictures && set_for(live).is_some() {
        format!("{PREFIX}{live}")
    } else {
        live.to_owned()
    }
}

/// The map a picture menu world's path shows, `None` for a live map.
pub(crate) fn pictured(map: &str) -> Option<&str> {
    map.strip_prefix(PREFIX)
}

/// The map a picture menu world is built from: nothing in it.
pub(crate) fn empty_world() -> sjk_bsp::Bsp {
    sjk_bsp::Bsp::empty([-4_096.0; 3], [4_096.0; 3])
}

/// The world error a graphics card short of memory gives: wgpu's
/// `Out of Memory` (an allocation it refused), as the install worker reports it.
pub(crate) fn is_out_of_memory(error: &str) -> bool {
    let error = error.to_ascii_lowercase();
    error.contains("out of memory") || error.contains("outofmemory")
}

/// The texture detail (`r_picmip`) running out of video memory steps down to at
/// most: a quarter of the sides, a sixteenth of the memory.
const AUTO_PICMIP_MAX: i64 = 2;

/// After `map` ran out of video memory, save some: the pictures on first, then each
/// time one step less texture detail (`r_picmip`, [`AUTO_PICMIP_MAX`] at most), which
/// the next map takes. The failure screen's message.
pub(crate) fn after_out_of_memory(
    console: Option<&mut crate::console::ViewerConsole>,
    map: &str,
) -> String {
    let map = map.trim_start_matches("maps/").trim_end_matches(".bsp");
    let map = if map.is_empty() { "the map" } else { map };
    let saved = console.and_then(|console| {
        if !enabled(Some(console)) && console.set_cvar(CVAR, "1") {
            return Some(
                "The menus now use pictures to save memory (Settings, Menu pictures); a lower \
                 graphics preset may help too."
                    .to_owned(),
            );
        }
        let cvar = crate::world_materials::filtering::mips::PICMIP_CVAR;
        let picmip = console.integer_cvar(cvar).unwrap_or(0).max(0) + 1;
        (picmip <= AUTO_PICMIP_MAX && console.set_cvar(cvar, &picmip.to_string())).then(|| {
            format!(
                "Textures are now smaller to save memory (Settings, Texture detail {picmip}); \
                 join again to use them."
            )
        })
    });
    let saved = saved
        .unwrap_or_else(|| "A lower graphics preset or a lower resolution may help.".to_owned());
    format!("Ran out of video memory loading {map}. {saved}")
}

/// One map's pictures, bundled with the client.
pub(crate) struct Set {
    /// The map they show.
    pub(crate) map: &'static str,
    /// `pictures.txt`: the views' cameras and the field of view.
    manifest: &'static str,
    /// Each view's JPEG by its name ([`View::name`]).
    files: &'static [(&'static str, &'static [u8])],
}

macro_rules! picture {
    ($folder:literal, $name:literal) => {
        (
            $name,
            include_bytes!(concat!(
                "../assets/menu_pictures/",
                $folder,
                "/",
                $name,
                ".jpg"
            )) as &[u8],
        )
    };
}

/// The picture sets: one for each menu style's map (`menu::style::MenuStyle::boot_map`).
pub(crate) const SETS: &[Set] = &[
    Set {
        map: "maps/mp/duel6.bsp",
        manifest: include_str!("../assets/menu_pictures/duel6/pictures.txt"),
        files: &[
            picture!("duel6", "tour-0"),
            picture!("duel6", "tour-1"),
            picture!("duel6", "tour-2"),
            picture!("duel6", "tour-3"),
            picture!("duel6", "tour-4"),
            picture!("duel6", "tour-5"),
            picture!("duel6", "tour-6"),
            picture!("duel6", "tour-7"),
            picture!("duel6", "tour-8"),
            picture!("duel6", "tour-9"),
            picture!("duel6", "player"),
            picture!("duel6", "saber"),
            picture!("duel6", "holocrons"),
        ],
    },
    Set {
        map: "maps/mp/ffa3.bsp",
        manifest: include_str!("../assets/menu_pictures/ffa3/pictures.txt"),
        files: &[
            picture!("ffa3", "main"),
            picture!("ffa3", "browser"),
            picture!("ffa3", "settings"),
            picture!("ffa3", "player"),
            picture!("ffa3", "saber"),
        ],
    },
];

/// The set of `map`, compared without case.
pub(crate) fn set_for(map: &str) -> Option<&'static Set> {
    SETS.iter().find(|set| set.map.eq_ignore_ascii_case(map))
}

/// A set's manifest, read.
#[derive(Debug, PartialEq)]
pub(crate) struct Manifest {
    /// The map's worldspawn message, which its routes are filed under.
    pub(crate) message: String,
    /// The vertical field of view the pictures were taken with, degrees.
    pub(crate) fov: f32,
    /// The pictures' size.
    pub(crate) size: [u32; 2],
    /// Each picture's view and camera, in the set's order.
    pub(crate) views: Vec<(View, Vantage)>,
}

impl Set {
    /// Its manifest; the tests hold every bundled one to read.
    pub(crate) fn manifest(&self) -> Result<Manifest, String> {
        parse_manifest(self.manifest)
    }

    /// The JPEG of the picture at `index` in the manifest's order.
    pub(crate) fn jpeg(&self, view: View) -> Option<&'static [u8]> {
        let name = view.name();
        self.files
            .iter()
            .find(|(file, _)| *file == name)
            .map(|(_, bytes)| *bytes)
    }

    /// The menu backdrop the set stands for.
    fn backdrop(&self) -> Option<Backdrop> {
        let manifest = self.manifest().ok()?;
        Backdrop::from_pictures(&manifest.message, manifest.views)
    }

    /// Where the generator writes the set.
    #[cfg(test)]
    pub(crate) fn source_directory(&self) -> std::path::PathBuf {
        // The folder is named after the map: `duel6` for maps/mp/duel6.bsp.
        let folder = std::path::Path::new(self.map)
            .file_stem()
            .expect("a map file name");
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/menu_pictures")
            .join(folder)
    }
}

/// Where the generator writes `set`.
#[cfg(test)]
pub(crate) fn source_directory(set: &Set) -> std::path::PathBuf {
    set.source_directory()
}

/// Read a `pictures.txt`: `message <text>`, `fov <degrees>`, `size <w> <h>` and a
/// `view <name> <x> <y> <z> <yaw> <pitch>` line a picture (radians); `#` comments.
fn parse_manifest(text: &str) -> Result<Manifest, String> {
    let mut message = None;
    let mut fov = None;
    let mut size = None;
    let mut views = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, rest) = line.split_once(' ').unwrap_or((line, ""));
        let numbers = || {
            rest.split_whitespace()
                .map(str::parse::<f32>)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| format!("{line:?}: {error}"))
        };
        match key {
            "message" => message = Some(rest.to_owned()),
            "fov" => fov = numbers()?.first().copied(),
            "size" => {
                let values = numbers()?;
                if let [width, height] = values[..] {
                    size = Some([width as u32, height as u32]);
                }
            }
            "view" => {
                let (name, rest) = rest.split_once(' ').unwrap_or((rest, ""));
                let view = View::parse(name).ok_or_else(|| format!("unknown view {name:?}"))?;
                let values = rest
                    .split_whitespace()
                    .map(str::parse::<f32>)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|error| format!("{line:?}: {error}"))?;
                let [x, y, z, yaw, pitch] = values[..] else {
                    return Err(format!("{line:?}: five numbers wanted"));
                };
                views.push((view, Vantage::new(glam::Vec3::new(x, y, z), yaw, pitch)));
            }
            _ => return Err(format!("unknown line {line:?}")),
        }
    }
    Ok(Manifest {
        message: message.ok_or("no message")?,
        fov: fov.ok_or("no fov")?,
        size: size.ok_or("no size")?,
        views,
    })
}

/// Give the menus the backdrop of the menu world on show: its pictures' for a
/// picture world, its map's own otherwise. Run when a menu world takes over.
pub(crate) fn fit_backdrop(gpu: &mut crate::GpuState) {
    if !gpu.is_menu_world {
        return;
    }
    let backdrop = match pictured(&gpu.graphics_reload.map).and_then(set_for) {
        Some(set) => set.backdrop(),
        None => Backdrop::from_bsp(&gpu.bsp),
    };
    if let Some(menu) = &mut gpu.client_menu {
        menu.set_backdrop(backdrop);
    }
}

/// Ready this frame's picture (decoding and uploading it when it changes) and the
/// field of view that lines the 3D stage up with it. Nothing on a live map.
pub(crate) fn prepare(gpu: &mut crate::GpuState) {
    let set = gpu
        .is_menu_world
        .then(|| pictured(&gpu.graphics_reload.map).and_then(set_for))
        .flatten();
    let shown = gpu
        .client_menu
        .as_ref()
        .and_then(crate::menu::ClientMenu::shown_picture);
    let exposure = gpu.post_aa.as_ref().map(|aa| aa.exposure_state());
    gpu.menu_pictures.prepare(&paint::Frame {
        device: &gpu.device,
        queue: &gpu.queue,
        scene_format: gpu.context.scene_format(),
        exposure,
        set,
        shown,
        screen: [gpu.configuration.width, gpu.configuration.height],
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_setting_is_off_by_default() {
        let directory = tempfile::tempdir().expect("a profile directory");
        let console = crate::console::ViewerConsole::new(directory.path().join("config.cfg"))
            .expect("a console");
        assert!(!enabled(Some(&console)));
        assert!(!enabled(None));
    }

    #[test]
    fn a_menu_map_turns_to_pictures_and_back() {
        let pictures = menu_map("maps/mp/duel6.bsp", true);
        assert_eq!(pictures, "pictures:maps/mp/duel6.bsp");
        assert_eq!(pictured(&pictures), Some("maps/mp/duel6.bsp"));
        assert_eq!(menu_map(&pictures, true), pictures);
        assert_eq!(menu_map(&pictures, false), "maps/mp/duel6.bsp");
        assert_eq!(pictured("maps/mp/duel6.bsp"), None);
        // A map without pictures stays live.
        assert_eq!(menu_map("maps/mp/ffa5.bsp", true), "maps/mp/ffa5.bsp");
    }

    #[test]
    fn out_of_memory_is_told_from_other_errors() {
        assert!(is_out_of_memory("wgpu error: Out of Memory"));
        assert!(is_out_of_memory(
            "Failed to get mapped range: OutOfMemory while creating buffer"
        ));
        assert!(!is_out_of_memory("wgpu error: Validation Error"));
        assert!(!is_out_of_memory("map maps/mp/x.bsp was not found"));
    }

    #[test]
    fn running_out_of_memory_turns_the_pictures_on_once() {
        let directory = tempfile::tempdir().expect("a profile directory");
        let mut console = crate::console::ViewerConsole::new(directory.path().join("config.cfg"))
            .expect("a console");
        let message = after_out_of_memory(Some(&mut console), "maps/mp/siege_desert.bsp");
        assert!(enabled(Some(&console)));
        assert!(message.contains("siege_desert"), "{message}");
        assert!(message.contains("now use pictures"), "{message}");
        // Then the textures, one step a time, down to the automatic floor.
        for step in 1..=AUTO_PICMIP_MAX {
            let again = after_out_of_memory(Some(&mut console), "maps/mp/ffa3.bsp");
            assert!(!again.contains("now use pictures"), "{again}");
            assert!(again.contains("Textures are now smaller"), "{again}");
            assert_eq!(
                console.integer_cvar(crate::world_materials::filtering::mips::PICMIP_CVAR),
                Some(step)
            );
        }
        let last = after_out_of_memory(Some(&mut console), "maps/mp/ffa3.bsp");
        assert!(last.contains("lower resolution"), "{last}");
        assert_eq!(
            console.integer_cvar(crate::world_materials::filtering::mips::PICMIP_CVAR),
            Some(AUTO_PICMIP_MAX)
        );
    }

    #[test]
    fn every_bundled_set_reads_and_has_each_views_picture() {
        for set in SETS {
            let manifest = set.manifest().expect("the manifest reads");
            assert_eq!(manifest.size, SIZE, "{}", set.map);
            assert!(!manifest.views.is_empty());
            for (view, _) in &manifest.views {
                let jpeg = set
                    .jpeg(*view)
                    .unwrap_or_else(|| panic!("{}: {} has no picture", set.map, view.name()));
                assert_eq!(&jpeg[..2], &[0xFF, 0xD8], "{} is a JPEG", view.name());
            }
            assert_eq!(set.files.len(), manifest.views.len(), "{}", set.map);
            assert!(set.backdrop().is_some(), "{}", set.map);
        }
    }

    #[test]
    fn each_menu_styles_map_has_pictures() {
        for name in crate::menu::style::MenuStyle::NAMES {
            let style = crate::menu::style::MenuStyle::from_cvar(Some(name));
            assert!(set_for(style.boot_map()).is_some(), "{name}");
        }
    }

    #[test]
    fn a_manifest_reads_its_views() {
        let manifest = parse_manifest(
            "# comment\nmessage Yavin Training Grounds\nfov 90\nsize 3840 2160\n\
             view main 1 2 3 0.5 -0.25\nview tour-2 4 5 6 0 0\n",
        )
        .expect("it reads");
        assert_eq!(manifest.message, "Yavin Training Grounds");
        assert_eq!(manifest.fov, 90.0);
        assert_eq!(manifest.views.len(), 2);
        assert_eq!(manifest.views[1].0, View::Tour(2));
        assert_eq!(manifest.views[0].1.pitch(), -0.25);
        assert!(parse_manifest("message x\nfov 90\nsize 1 1\nview elsewhere 1 2 3 4 5").is_err());
    }
}
