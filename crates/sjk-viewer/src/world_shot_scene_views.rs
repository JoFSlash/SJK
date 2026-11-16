//! The secondary views' targets ([`crate::scene_views`]): which ones a world holds, and
//! frames through each view, to compare a change to how they are made with the frames
//! before it. Polished floors at `r_floorReflections 1`, then off and on again (the four
//! largest planes of the first retail map of [`FLOOR_MAPS`] with any), duel6's sky
//! portal (the `CS_SKYBOXORG` its `misc_skyportal` makes the server send) and the
//! mirrors of the shader test map `sjk_shaders_e` (its `misc_portal_surface`s as the
//! server spawns them; skipped without the map, see rendering.md's "Shader test maps").
//! `SJK_SCENE_VIEWS_BEFORE=<directory>` compares each contact sheet with the same-named
//! PNG there. Ignored like the other world shots.

use super::*;

const SIZE: [u32; 2] = [1280, 720];
/// Retail maps tried in turn for a polished floor.
const FLOOR_MAPS: [&str; 4] = [
    "maps/mp/ffa3.bsp",
    "maps/mp/ctf2.bsp",
    "maps/mp/duel2.bsp",
    "maps/mp/siege_hoth.bsp",
];
const PORTAL_MAP: &str = "maps/sjk_shaders_e.bsp";
/// `ES_ENTITY_TYPE`, `ES_POS_BASE` and `ES_ORIGIN2` (`sjk-dedicated`'s bridge).
const ES_ENTITY_TYPE: usize = 8;
const ES_POS_BASE: [usize; 3] = [2, 1, 4];
const ES_ORIGIN2: [usize; 3] = [56, 60, 53];

/// Print which targets `gpu` holds and which views its last frame drew.
fn state(gpu: &GpuState, name: &str) {
    let [floors, sky, portal] = gpu.scene_views.targets();
    let [mirrors, sky_view, portal_view] = gpu.scene_views.drawn();
    println!(
        "{name}: targets: floor mirrors {floors}, sky portal {sky}, map portal {portal}; \
         drawn: floor mirrors {mirrors}, sky portal {sky_view}, map portal {portal_view}"
    );
}

/// The pixels of `a` and `b` that differ, those by more than 8/255, and the largest
/// difference.
fn difference(a: &image::RgbaImage, b: &image::RgbaImage) -> (usize, usize, u8) {
    let (mut differing, mut above, mut worst) = (0usize, 0usize, 0u8);
    for (old, new) in a.pixels().zip(b.pixels()) {
        let delta = old
            .0
            .iter()
            .zip(new.0)
            .map(|(a, b)| a.abs_diff(b))
            .max()
            .unwrap_or(0);
        differing += usize::from(delta > 0);
        above += usize::from(delta > 8);
        worst = worst.max(delta);
    }
    (differing, above, worst)
}

/// Save `images` as the contact sheet `name` and compare it with the one in
/// `SJK_SCENE_VIEWS_BEFORE`.
fn report(name: &str, images: &[image::RgbaImage]) {
    let path = sheet(images, 2, SIZE[0], name);
    println!("{}", path.display());
    let Some(before) = std::env::var_os("SJK_SCENE_VIEWS_BEFORE") else {
        return;
    };
    let Ok(before) = image::open(PathBuf::from(before).join(format!("{name}.png"))) else {
        println!("{name}: nothing to compare with");
        return;
    };
    let now = image::open(&path).expect("the sheet").to_rgba8();
    let (differing, above, worst) = difference(&before.to_rgba8(), &now);
    println!(
        "{name}: {differing} of {} pixels differ from before ({above} by more than 8/255), \
         at most {worst}/255",
        now.width() * now.height()
    );
}

/// A camera `distance` units off a face of `center` and `normal`, raised along it and
/// looking at its centre.
fn face_on(gpu: &mut GpuState, center: Vec3, normal: Vec3, distance: f32, raise: f32) {
    let side = if normal.z.abs() > 0.7 {
        Vec3::X
    } else {
        normal
    };
    let eye = center + side * distance + Vec3::Z * raise;
    let (yaw, pitch) = look(eye.to_array(), center.to_array());
    aim(gpu, eye.to_array(), yaw, pitch);
}

fn snapshot(entities: Vec<sjk_protocol::EntityState>) -> sjk_protocol::Snapshot {
    sjk_protocol::Snapshot {
        message_sequence: 0,
        reliable_acknowledge: 0,
        server_commands: Vec::new(),
        server_time: 0,
        delta_from: None,
        flags: 0,
        area_mask: Vec::new(),
        player: Default::default(),
        vehicle_player: None,
        entities,
        consumed_bits: 0,
    }
}

fn set_floor_reflections(gpu: &mut GpuState, value: &str) {
    gpu.console
        .as_mut()
        .expect("a console")
        .set_cvar("r_floorReflections", value);
}

/// A floor mirror made at `r_superSample 2`, then the window resized: the target is
/// made at the scene's supersampled size and made again at the new one.
#[test]
#[ignore = "renders with the GPU and the installed game data named by JKA_GAME_DATA"]
fn scene_view_targets_resize() {
    on_big_stack(|| {
        let cvars = [("r_floorReflections", "1"), ("r_superSample", "2")];
        let Some((mut gpu, _profile)) = open(FLOOR_MAPS[0], SIZE, None, &cvars) else {
            return;
        };
        gpu.is_menu_world = false;
        let Some(&(center, radius, normal)) = gpu.scene_views.polished_faces().first() else {
            println!("{}: no polished floor", FLOOR_MAPS[0]);
            return;
        };
        face_on(
            &mut gpu,
            center,
            normal,
            (radius * 1.2).clamp(96.0, 320.0),
            48.0,
        );
        let mut images = Vec::new();
        for size in [SIZE, [1600, 900]] {
            if size != SIZE {
                gpu.resize(winit::dpi::PhysicalSize::new(size[0], size[1]));
                attach_target(&mut gpu, size);
            }
            let image = frame(&mut gpu, 8);
            let [floors, _, _] = gpu.scene_views.target_sizes();
            let [mirrors, _, _] = gpu.scene_views.drawn();
            println!(
                "window {size:?}: scene {:?}, floor mirror target {floors:?}, drawn {mirrors}",
                gpu.scene_size()
            );
            assert!(mirrors, "the mirror is drawn");
            assert_eq!(floors, Some(gpu.scene_size()), "at the scene's size");
            images.push(image::imageops::resize(
                &image,
                SIZE[0],
                SIZE[1],
                image::imageops::FilterType::Triangle,
            ));
        }
        report("scene-views-floor-resize", &images);
    });
}

#[test]
#[ignore = "renders with the GPU and the installed game data named by JKA_GAME_DATA"]
fn scene_view_targets() {
    on_big_stack(|| {
        // Polished floors, mirrored, then plain, then mirrored again.
        for map in FLOOR_MAPS {
            let Some((mut gpu, _profile)) = open(map, SIZE, None, &[("r_floorReflections", "1")])
            else {
                return;
            };
            gpu.is_menu_world = false;
            state(&gpu, &format!("{map} built"));
            let faces: Vec<_> = gpu
                .scene_views
                .polished_faces()
                .into_iter()
                .take(4)
                .collect();
            if faces.is_empty() {
                println!("{map}: no polished floor");
                continue;
            }
            let mut sheets = [Vec::new(), Vec::new(), Vec::new()];
            for (index, &(center, radius, normal)) in faces.iter().enumerate() {
                println!("{map}: polished face {index} at {center} (radius {radius:.0})");
                face_on(
                    &mut gpu,
                    center,
                    normal,
                    (radius * 1.2).clamp(96.0, 320.0),
                    48.0,
                );
                for (step, value) in ["1", "0", "1"].into_iter().enumerate() {
                    set_floor_reflections(&mut gpu, value);
                    sheets[step].push(frame(&mut gpu, 8));
                    state(&gpu, &format!("face {index}, r_floorReflections {value}"));
                }
                let (differing, _, _) = difference(&sheets[0][index], &sheets[1][index]);
                println!("face {index}: the mirror changes {differing} pixels");
            }
            report("scene-views-floor-on", &sheets[0]);
            report("scene-views-floor-off", &sheets[1]);
            report("scene-views-floor-on-again", &sheets[2]);
            break;
        }

        // duel6's sky portal, once the server's CS_SKYBOXORG names it.
        let Some((mut gpu, _profile)) = open("maps/mp/duel6.bsp", SIZE, None, &[]) else {
            return;
        };
        gpu.is_menu_world = false;
        state(&gpu, "maps/mp/duel6.bsp built");
        let (origin, yaw) = (gpu.camera_position, gpu.camera_yaw.to_degrees());
        aim(&mut gpu, origin.to_array(), yaw, 25.0);
        let without = frame(&mut gpu, 6);
        state(&gpu, "duel6 without a session");
        let entities = sjk_entity::parse_entity_lump(gpu.bsp.entities()).unwrap_or_default();
        let sky = entities
            .iter()
            .find(|entity| entity.classname() == Some("misc_skyportal"))
            .expect("duel6's sky portal");
        let mut game = sjk_protocol::GameState::empty_local(0);
        game.replace_config_string(
            sjk_game_jka::map_scenery::CS_SKYBOXORG,
            sjk_game_jka::map_scenery::sky_portal(sky),
        )
        .expect("the sky portal's config string");
        gpu.resident.scenery = Some((game, snapshot(Vec::new())));
        let with = frame(&mut gpu, 12);
        state(&gpu, "duel6 with CS_SKYBOXORG");
        let (differing, above, _) = difference(&without, &with);
        println!("the sky portal changes {differing} pixels ({above} by more than 8/255)");
        report("scene-views-sky", &[without, with]);
        drop(gpu);

        // The shader test map's mirrors, their surfaces sent as the server spawns them.
        let game_data = PathBuf::from(std::env::var_os("JKA_GAME_DATA").expect("JKA_GAME_DATA"));
        if assets::load_bsp(&game_data, PORTAL_MAP).is_err() {
            println!("{PORTAL_MAP} is not installed: no map portal shot");
            return;
        }
        let Some((mut gpu, _profile)) = open(PORTAL_MAP, SIZE, None, &[]) else {
            return;
        };
        gpu.is_menu_world = false;
        state(&gpu, &format!("{PORTAL_MAP} built"));
        let origins: Vec<Vec3> = sjk_entity::parse_entity_lump(gpu.bsp.entities())
            .unwrap_or_default()
            .iter()
            .filter(|entity| entity.classname() == Some("misc_portal_surface"))
            .filter_map(|entity| entity.vector("origin").ok().flatten())
            .map(Vec3::from_array)
            .collect();
        let entities = origins
            .iter()
            .enumerate()
            .map(|(index, origin)| {
                let mut state = sjk_protocol::EntityState::zero(
                    100 + index as u16,
                    &sjk_protocol::LEGACY_ENTITY_FIELDS,
                );
                state.set_raw_field(ES_ENTITY_TYPE, sjk_game_jka::map_scenery::ET_PORTAL);
                for axis in 0..3 {
                    // No camera: the view is from where it stands, a mirror.
                    state.set_raw_field(ES_POS_BASE[axis], origin[axis].to_bits());
                    state.set_raw_field(ES_ORIGIN2[axis], origin[axis].to_bits());
                }
                state
            })
            .collect();
        // Each surface's nearest portal face within the 64 units a mirror matches at.
        let portal_faces = gpu.scene_views.portal_faces();
        let faces: Vec<_> = origins
            .iter()
            .filter_map(|origin| {
                portal_faces
                    .iter()
                    .filter(|(center, radius, normal)| {
                        (normal.dot(*origin) - normal.dot(*center)).abs() <= 64.0
                            && origin.distance(*center) <= radius + 128.0
                    })
                    .min_by(|a, b| a.0.distance(*origin).total_cmp(&b.0.distance(*origin)))
                    .copied()
            })
            .collect();
        println!(
            "{} of {} portal faces face a surface entity",
            faces.len(),
            portal_faces.len()
        );
        let (center, _, normal) = *faces
            .first()
            .expect("a portal face with its surface entity");
        face_on(&mut gpu, center, normal, 200.0, 24.0);
        let mut images = vec![frame(&mut gpu, 4)];
        state(&gpu, "the first mirror without a session");
        gpu.resident.scenery = Some((sjk_protocol::GameState::empty_local(0), snapshot(entities)));
        for (index, &(center, _, normal)) in faces.iter().enumerate() {
            face_on(&mut gpu, center, normal, 200.0, 24.0);
            images.push(frame(&mut gpu, 8));
            state(&gpu, &format!("mirror {index}"));
        }
        let (differing, above, _) = difference(&images[0], &images[1]);
        println!("the first mirror changes {differing} pixels ({above} by more than 8/255)");
        report("scene-views-portals", &images);
    });
}
