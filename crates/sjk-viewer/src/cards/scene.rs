//! Card scenes (`docs/cards.md`, "Scenes"; 30/10/2026, Sol's idea): a card whose file
//! names a `scene` has its picture drawn by the client from the player's own game data,
//! a character's model posed or a weapon's model, instead of the pack's picture. The
//! pack's picture stays: older clients draw it, and this one shows it until the drawing is
//! ready or when the game data lacks the model. Nothing of the game travels: each player's
//! client draws its own copy, once, and keeps it beside its packs ([`cache_path`]).
//!
//! A scene is one line: up to three figures joined by `+`, each a kind and a name and its
//! options, and the camera's options anywhere:
//!
//! ```text
//! player kyle
//! player jedi_tf/default + player jedi_zf
//! player desann anim=BOTH_FORCELIGHTNING_HOLD at=0.4 yaw=40
//! player luke anim=BOTH_STAND2 saber=luke blade=green
//! player jedi_zf anim=BOTH_SABERDUAL_STANCE saber=single_1 saber2=single_1 blade=purple
//! model models/weapons2/blaster_r/blaster.md3 yaw=90 pitch=15 zoom=1.2
//! ```
//!
//! `player <model>[/<skin>]` is a player model of `models/players/` posed in `anim` (an
//! `animation.cfg` name; its idle when none) at `at` of the way through it (0 to 1),
//! holding the hilt `saber` (a `.sab` name: `single_1`, `dual_1`, `luke`) in its right
//! hand and `saber2` in its left, lit in `blade` and `blade2` (red, orange, yellow, green,
//! blue or purple; blue by default, the second as the first);
//! `model <path>` is any `.md3` or `.glm` under `models/`. `yaw` turns the camera round
//! the figures from straight in front (-180 to 180 degrees, 25 by default), `pitch` raises
//! it (-60 to 60, 8 by default) and `zoom` brings it nearer (0.5 to 3). This module reads
//! scenes, names their pictures, queues them for the renderer (`crate::card_scenes`),
//! frames the camera and lays the drawing over the card's backdrop; the drawing is the
//! renderer's.

use glam::{Mat4, Vec3};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

/// The drawing's version: a new one draws every cached scene again.
pub(crate) const VERSION: u32 = 1;
/// A scene's longest text, figures at most, and the picture's size (the art window's).
pub(crate) const TEXT_MAX: usize = 200;
pub(crate) const FIGURES_MAX: usize = 3;
pub(crate) const SIZE: [u32; 2] = [1020, 850];
/// The camera's vertical field of view, degrees, and the share of the frame the figures
/// fill at zoom 1.
pub(crate) const FIELD_OF_VIEW: f32 = 30.0;
const FILL: f32 = 0.86;
/// The camera by default: a little to the figures' left, a little above.
const YAW: f32 = 25.0;
const PITCH: f32 = 8.0;
/// The folder the drawings are kept in, beside the packs' (`assets/`).
const FOLDER: &str = "card_scenes";
/// The quality the drawings are kept at, as the hub keeps card pictures.
const QUALITY: u8 = 92;

/// The blades' colours by name, in the retail order (`crate::saber::Color::ALL`).
pub(crate) const BLADES: [&str; 6] = ["red", "orange", "yellow", "green", "blue", "purple"];
/// Blue, the blades' colour when none is named.
const BLUE: usize = 4;

/// What a scene shows.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Figure {
    /// A player model (`kyle`, `jedi_tf/default`), posed in `anim` at `at` of it, the
    /// hilts `sabers` in its right and left hands lit in `blades` (in [`BLADES`]).
    Player {
        model: String,
        anim: Option<String>,
        at: f32,
        sabers: [Option<String>; 2],
        blades: [usize; 2],
    },
    /// A rigid model by its game path.
    Model { path: String },
}

/// A scene: its figures and the camera.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Scene {
    pub(crate) figures: Vec<Figure>,
    pub(crate) yaw: f32,
    pub(crate) pitch: f32,
    pub(crate) zoom: f32,
}

/// Whether `name` is a model or skin folder's name: lowercase letters, digits, `_`, `-`.
fn folder_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_-".contains(&byte))
}

/// `value` as a number from `low` to `high`, for option `key`.
fn number(key: &str, value: &str, low: f32, high: f32) -> Result<f32, String> {
    value
        .parse::<f32>()
        .ok()
        .filter(|number| number.is_finite() && (low..=high).contains(number))
        .ok_or_else(|| format!("{key} must be a number from {low} to {high}"))
}

/// The scene `text` describes, checked.
pub(crate) fn parse(text: &str) -> Result<Scene, String> {
    let text = text.trim();
    if text.is_empty() || text.len() > TEXT_MAX || !text.is_ascii() {
        return Err(format!("a scene is 1 to {TEXT_MAX} ASCII characters"));
    }
    let mut scene = Scene {
        figures: Vec::new(),
        yaw: YAW,
        pitch: PITCH,
        zoom: 1.0,
    };
    let mut camera = HashSet::new();
    for part in text.split('+') {
        let mut words = part.split_whitespace();
        let (Some(kind), Some(name)) = (words.next(), words.next()) else {
            return Err("each figure is a kind (player or model) and a name".to_owned());
        };
        let name = name.to_ascii_lowercase();
        let mut figure = match kind {
            "player" => {
                let (model, skin) = name.split_once('/').unwrap_or((&name, "default"));
                if !folder_name(model) || !folder_name(skin) {
                    return Err(format!(
                        "{name} is not a player model (model or model/skin)"
                    ));
                }
                Figure::Player {
                    model: format!("{model}/{skin}"),
                    anim: None,
                    at: 0.0,
                    sabers: [None, None],
                    blades: [BLUE; 2],
                }
            }
            "model" => {
                let good = name.starts_with("models/")
                    && (name.ends_with(".md3") || name.ends_with(".glm"))
                    && !name.contains("..")
                    && name.split('/').all(|part| {
                        !part.is_empty()
                            && part.bytes().all(|byte| {
                                byte.is_ascii_lowercase()
                                    || byte.is_ascii_digit()
                                    || b"_-.".contains(&byte)
                            })
                    });
                if !good {
                    return Err(format!("{name} is not a .md3 or .glm path under models/"));
                }
                Figure::Model { path: name }
            }
            other => return Err(format!("{other} is not a kind of figure (player or model)")),
        };
        let mut second_blade = false;
        for word in words {
            let (key, value) = word
                .split_once('=')
                .ok_or_else(|| format!("{word} is not an option (key=value)"))?;
            match (key, &mut figure) {
                ("anim", Figure::Player { anim, .. }) => {
                    let good = !value.is_empty()
                        && value.bytes().all(|byte| {
                            byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_'
                        });
                    if !good {
                        return Err(format!("{value} is not an animation name (BOTH_STAND1)"));
                    }
                    *anim = Some(value.to_owned());
                }
                ("at", Figure::Player { at, .. }) => *at = number("at", value, 0.0, 1.0)?,
                ("saber" | "saber2", Figure::Player { sabers, .. }) => {
                    let good = (1..=32).contains(&value.len())
                        && value.bytes().all(|byte| {
                            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
                        });
                    if !good {
                        return Err(format!("{value} is not a saber's name (single_1)"));
                    }
                    sabers[usize::from(key == "saber2")] = Some(value.to_owned());
                }
                ("blade" | "blade2", Figure::Player { blades, .. }) => {
                    let colour = BLADES
                        .iter()
                        .position(|name| *name == value)
                        .ok_or_else(|| format!("{key} must be one of {}", BLADES.join(", ")))?;
                    if key == "blade2" {
                        second_blade = true;
                        blades[1] = colour;
                    } else {
                        blades[0] = colour;
                    }
                }
                ("yaw" | "pitch" | "zoom", _) => {
                    if !camera.insert(key) {
                        return Err(format!("{key} is given twice"));
                    }
                    match key {
                        "yaw" => scene.yaw = number("yaw", value, -180.0, 180.0)?,
                        "pitch" => scene.pitch = number("pitch", value, -60.0, 60.0)?,
                        _ => scene.zoom = number("zoom", value, 0.5, 3.0)?,
                    }
                }
                ("anim" | "at" | "saber" | "saber2" | "blade" | "blade2", Figure::Model { .. }) => {
                    return Err(format!("{key} is for a player, not a model"));
                }
                _ => {
                    return Err(format!(
                        "{key} is not an option (anim, at, saber, saber2, blade, blade2, yaw, pitch, zoom)"
                    ));
                }
            }
        }
        if let Figure::Player { blades, .. } = &mut figure
            && !second_blade
        {
            blades[1] = blades[0];
        }
        scene.figures.push(figure);
    }
    if scene.figures.len() > FIGURES_MAX {
        return Err(format!("a scene has at most {FIGURES_MAX} figures"));
    }
    Ok(scene)
}

/// The name a scene's drawing is kept under: 16 hex of the SHA-256 of the drawing's
/// version and the scene's words, so a changed scene or drawing is drawn anew.
pub(crate) fn key(text: &str) -> String {
    let words = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let digest = Sha256::digest(format!("{VERSION}\n{words}").as_bytes());
    digest[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Where card `id`'s drawing of `text` is kept, once the packs' folder is known.
pub(crate) fn cache_path(id: &str, text: &str) -> Option<PathBuf> {
    let packs = crate::sjk_packs::directory()?;
    Some(
        packs
            .parent()?
            .join(FOLDER)
            .join(format!("{id}-{}.jpg", key(text))),
    )
}

/// A drawing asked for.
#[derive(Clone, Debug)]
pub(crate) struct Job {
    pub(crate) id: String,
    pub(crate) scene: Scene,
    pub(crate) path: PathBuf,
    /// The card's rarity colour, which the backdrop glows in.
    pub(crate) tint: [u8; 3],
}

/// What the face worker and the renderer share.
#[derive(Default)]
struct Shared {
    queue: Vec<Job>,
    /// The drawing under way.
    busy: Option<PathBuf>,
    /// Drawings that failed this session (a model missing): not asked for again.
    failed: HashSet<PathBuf>,
    /// Cards drawn since the faces last looked: their pictures are uploaded again.
    drawn: Vec<String>,
}

static SHARED: Mutex<Option<Shared>> = Mutex::new(None);

fn shared() -> MutexGuard<'static, Option<Shared>> {
    SHARED
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Ask for `job`'s drawing, unless it is queued, under way or failed already.
pub(crate) fn want(job: Job) {
    let mut guard = shared();
    let shared = guard.get_or_insert_with(Shared::default);
    let known = shared.failed.contains(&job.path)
        || shared.busy.as_ref() == Some(&job.path)
        || shared.queue.iter().any(|queued| queued.path == job.path);
    if !known {
        shared.queue.push(job);
    }
}

/// The next drawing for the renderer, now under way.
pub(crate) fn next() -> Option<Job> {
    let mut guard = shared();
    let shared = guard.as_mut()?;
    if shared.busy.is_some() || shared.queue.is_empty() {
        return None;
    }
    let job = shared.queue.remove(0);
    shared.busy = Some(job.path.clone());
    Some(job)
}

/// Whether drawings are queued or under way.
#[cfg(test)]
pub(crate) fn pending() -> bool {
    shared()
        .as_ref()
        .is_some_and(|shared| shared.busy.is_some() || !shared.queue.is_empty())
}

/// The drawing under way ended: kept (`ok`), or failed and not asked for again.
pub(crate) fn finished(job: &Job, ok: bool) {
    let mut guard = shared();
    let shared = guard.get_or_insert_with(Shared::default);
    if shared.busy.as_ref() == Some(&job.path) {
        shared.busy = None;
    }
    if ok {
        shared.drawn.push(job.id.clone());
    } else {
        shared.failed.insert(job.path.clone());
    }
}

/// The cards drawn since the last call.
pub(crate) fn take_drawn() -> Vec<String> {
    shared()
        .as_mut()
        .map(|shared| std::mem::take(&mut shared.drawn))
        .unwrap_or_default()
}

/// The camera for figures whose points are `points`: looking at them from `yaw` and
/// `pitch` (degrees) off `front` (the way they face), far enough for all of them to fill
/// [`FILL`] of a frame of `aspect` (more at a `zoom` over 1), centred on what shows.
/// Answers the view projection, the eye and its forward.
pub(crate) fn frame(
    points: &[Vec3],
    front: Vec3,
    yaw: f32,
    pitch: f32,
    zoom: f32,
    aspect: f32,
) -> (Mat4, Vec3, Vec3) {
    let (low, high) = points.iter().fold(
        (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)),
        |(low, high), point| (low.min(*point), high.max(*point)),
    );
    let middle = if points.is_empty() {
        Vec3::ZERO
    } else {
        (low + high) * 0.5
    };
    let front = Vec3::new(front.x, front.y, 0.0).normalize_or(Vec3::X);
    let (yaw, pitch) = (yaw.to_radians(), pitch.to_radians());
    let flat = glam::Quat::from_rotation_z(yaw) * front;
    let back = (flat * pitch.cos() + Vec3::Z * pitch.sin()).normalize();
    let forward = -back;
    let right = forward.cross(Vec3::Z).normalize_or(Vec3::Y);
    let up = right.cross(forward);
    let tan_v = (FIELD_OF_VIEW * 0.5).to_radians().tan() * FILL * zoom.max(0.1);
    let tan_h = tan_v * aspect.max(0.1);
    // Centre what shows: the middle of the points across and up the view.
    let across = |point: &Vec3| (*point - middle).dot(right);
    let height = |point: &Vec3| (*point - middle).dot(up);
    let span = |measure: &dyn Fn(&Vec3) -> f32| {
        points
            .iter()
            .map(measure)
            .fold((f32::MAX, f32::MIN), |(low, high), value| {
                (low.min(value), high.max(value))
            })
    };
    let (left, right_edge) = span(&across);
    let (bottom, top) = span(&height);
    let centre = if points.is_empty() {
        middle
    } else {
        middle + right * (left + right_edge) * 0.5 + up * (bottom + top) * 0.5
    };
    let mut distance: f32 = 1.0;
    let mut depth: f32 = 0.0;
    for point in points {
        let offset = *point - centre;
        let toward = offset.dot(back);
        depth = depth.max(offset.length());
        distance = distance
            .max(toward + offset.dot(right).abs() / tan_h)
            .max(toward + offset.dot(up).abs() / tan_v);
    }
    let eye = centre + back * distance;
    let view = glam::camera::rh::view::look_at_mat4(eye, centre, Vec3::Z);
    let near = (distance - depth - 8.0).max(1.0);
    let projection = glam::camera::rh::proj::directx::perspective(
        FIELD_OF_VIEW.to_radians(),
        aspect,
        near,
        distance + depth + 64.0,
    );
    (projection * view, eye, forward)
}

/// How far the light round the figures' outline reaches, in pixels, and how strong it is.
const RIM_REACH: usize = 14;
const RIM: f32 = 0.75;
/// The figures' tones lifted: a gamma under 1 opens the shadows of dark costumes.
const LIFT: f32 = 0.82;

/// `values` (`width` a row) blurred by a box of `reach` pixels each way, twice, which is
/// near a Gaussian.
fn blurred(values: &[f32], width: usize, reach: usize) -> Vec<f32> {
    let height = values.len() / width.max(1);
    let mut out = values.to_vec();
    let mut line = Vec::new();
    for _ in 0..2 {
        for (stride, count, steps) in [(1, width, height), (width, height, width)] {
            for step in 0..steps {
                let start = if stride == 1 { step * width } else { step };
                line.clear();
                line.extend((0..count).map(|at| out[start + at * stride]));
                let mut sum: f32 = line.iter().take(reach + 1).sum();
                for at in 0..count {
                    let low = at.saturating_sub(reach);
                    let high = (at + reach).min(count - 1);
                    out[start + at * stride] = sum / (high - low + 1) as f32;
                    if at + reach + 1 < count {
                        sum += line[at + reach + 1];
                    }
                    if at >= reach {
                        sum -= line[at - reach];
                    }
                }
            }
        }
    }
    out
}

/// Lay the drawing `rgba` (straight alpha, `size`) over the card's backdrop: a deep
/// space gradient with a glow of `tint` behind the figures and a softer light of it round
/// their outline, so a dark costume stands out; the figures' shadows are opened a little.
/// The result is opaque.
pub(crate) fn over_backdrop(rgba: &mut [u8], size: [u32; 2], tint: [u8; 3]) {
    let [width, height] = size.map(|side| side.max(1) as f32);
    let alphas: Vec<f32> = rgba
        .chunks_exact(4)
        .map(|pixel| f32::from(pixel[3]) / 255.0)
        .collect();
    let spread = blurred(&alphas, size[0].max(1) as usize, RIM_REACH);
    for (index, pixel) in rgba.chunks_exact_mut(4).enumerate() {
        let x = (index as u32 % size[0]) as f32 / width;
        let y = (index as u32 / size[0]) as f32 / height;
        let dx = (x - 0.5) * 1.2;
        let dy = y - 0.42;
        let glow = (1.0 - (dx * dx + dy * dy).sqrt() / 0.62)
            .clamp(0.0, 1.0)
            .powi(2)
            * 0.40;
        let alpha = alphas[index];
        let rim = ((spread[index] - alpha) * 2.0).clamp(0.0, 1.0) * RIM;
        let back: [f32; 3] = std::array::from_fn(|channel| {
            let top = [26.0, 34.0, 60.0][channel];
            let bottom = [8.0, 12.0, 24.0][channel];
            top + (bottom - top) * y + f32::from(tint[channel]) * (glow + rim)
        });
        for channel in 0..3 {
            let lifted = (f32::from(pixel[channel]) / 255.0).powf(LIFT) * 255.0;
            let value = lifted * alpha + back[channel] * (1.0 - alpha);
            pixel[channel] = value.round().clamp(0.0, 255.0) as u8;
        }
        pixel[3] = 255;
    }
}

/// Keep the opaque drawing `rgba` (`size`) at `path` as a JPEG, through a temporary file
/// renamed into place, so a half-written drawing is never read.
pub(crate) fn keep(path: &Path, size: [u32; 2], rgba: &[u8]) -> Result<(), String> {
    let folder = path.parent().ok_or("no folder")?;
    std::fs::create_dir_all(folder).map_err(|error| error.to_string())?;
    let rgb: Vec<u8> = rgba
        .chunks_exact(4)
        .flat_map(|pixel| [pixel[0], pixel[1], pixel[2]])
        .collect();
    let mut bytes = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, QUALITY)
        .encode(&rgb, size[0], size[1], image::ExtendedColorType::Rgb8)
        .map_err(|error| error.to_string())?;
    let partial = path.with_extension("jpg.part");
    std::fs::write(&partial, &bytes).map_err(|error| error.to_string())?;
    std::fs::rename(&partial, path).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scenes_read_with_their_defaults_and_options() {
        let scene = parse("player Kyle").unwrap();
        assert_eq!(
            scene,
            Scene {
                figures: vec![Figure::Player {
                    model: "kyle/default".into(),
                    anim: None,
                    at: 0.0,
                    sabers: [None, None],
                    blades: [BLUE; 2],
                }],
                yaw: YAW,
                pitch: PITCH,
                zoom: 1.0
            }
        );
        let scene = parse(
            "player jedi_tf/siege anim=BOTH_FORCEPUSH at=0.5 + model models/weapons2/blaster_r/blaster.md3 yaw=-40 zoom=1.5",
        )
        .unwrap();
        assert_eq!(scene.figures.len(), 2);
        assert_eq!(
            scene.figures[0],
            Figure::Player {
                model: "jedi_tf/siege".into(),
                anim: Some("BOTH_FORCEPUSH".into()),
                at: 0.5,
                sabers: [None, None],
                blades: [BLUE; 2],
            }
        );
        assert_eq!(
            scene.figures[1],
            Figure::Model {
                path: "models/weapons2/blaster_r/blaster.md3".into()
            }
        );
        assert_eq!((scene.yaw, scene.pitch, scene.zoom), (-40.0, PITCH, 1.5));
        for (bad, says) in [
            ("", "1 to 200"),
            ("player", "a kind"),
            ("npc kyle", "not a kind"),
            ("player ../kyle", "not a player model"),
            ("model models/../x.md3", "not a .md3"),
            ("model sound/x.md3", "not a .md3"),
            ("model models/x.tga", "not a .md3"),
            ("player kyle anim=stand", "animation name"),
            ("player kyle at=2", "at must"),
            ("player kyle yaw=1 + player luke yaw=2", "twice"),
            ("model models/a.md3 at=0.5", "for a player"),
            ("player kyle colour=red", "not an option"),
            ("player kyle saber=Single", "saber's name"),
            ("player kyle blade=pink", "blade must be one of"),
            ("model models/a.md3 saber=single_1", "for a player"),
            ("player kyle loud", "key=value"),
            ("player a + player b + player c + player d", "at most 3"),
            ("player kylé", "ASCII"),
        ] {
            let error = parse(bad).unwrap_err();
            assert!(error.contains(says), "{bad}: {error}");
        }
        assert!(parse(&format!("player {}", "k".repeat(TEXT_MAX))).is_err());
        // Sabers in both hands; the second blade takes the first's colour unless named.
        let dual = parse("player jedi_zf saber=single_1 saber2=single_2 blade=green").unwrap();
        let Figure::Player { sabers, blades, .. } = &dual.figures[0] else {
            panic!("a player");
        };
        assert_eq!(
            sabers,
            &[Some("single_1".to_owned()), Some("single_2".to_owned())]
        );
        assert_eq!(blades, &[3, 3]);
        let mixed = parse("player alora saber=single_1 blade2=red").unwrap();
        let Figure::Player { blades, .. } = &mixed.figures[0] else {
            panic!("a player");
        };
        assert_eq!(blades, &[BLUE, 0]);
    }

    #[test]
    fn a_scene_is_kept_under_its_words_and_the_drawing_version() {
        assert_eq!(key("player kyle"), key("  player   kyle "));
        assert_ne!(key("player kyle"), key("player luke"));
        assert_eq!(key("player kyle").len(), 16);
    }

    #[test]
    fn the_queue_hands_out_one_drawing_at_a_time_and_forgets_failures() {
        let job = |id: &str| Job {
            id: id.into(),
            scene: parse("player kyle").unwrap(),
            path: PathBuf::from(format!("scene-test-{id}.jpg")),
            tint: [0; 3],
        };
        *shared() = None;
        want(job("a"));
        want(job("a"));
        want(job("b"));
        let first = next().unwrap();
        assert_eq!(first.id, "a");
        assert!(next().is_none(), "one at a time");
        want(job("a"));
        finished(&first, false);
        want(job("a"));
        let second = next().unwrap();
        assert_eq!(second.id, "b", "a failed drawing is not asked for again");
        finished(&second, true);
        assert!(!pending());
        assert_eq!(take_drawn(), ["b"]);
        assert!(take_drawn().is_empty());
        *shared() = None;
    }

    #[test]
    fn the_camera_frames_every_point_in_front_of_the_figures() {
        // A standing player: 56 high, 30 wide, facing +X.
        let mut points = Vec::new();
        for x in [-8.0, 8.0] {
            for y in [-15.0, 15.0] {
                for z in [-24.0, 32.0] {
                    points.push(Vec3::new(x, y, z));
                }
            }
        }
        let aspect = SIZE[0] as f32 / SIZE[1] as f32;
        let (view_projection, eye, forward) = frame(&points, Vec3::X, 0.0, 0.0, 1.0, aspect);
        assert!(eye.x > 50.0 && forward.x < -0.99, "in front: {eye}");
        let mut widest: f32 = 0.0;
        for point in &points {
            let clip = view_projection * point.extend(1.0);
            let ndc = clip.truncate() / clip.w;
            assert!(
                ndc.x.abs() <= FILL + 1e-3 && ndc.y.abs() <= FILL + 1e-3,
                "{point}: {ndc}"
            );
            widest = widest.max(ndc.x.abs().max(ndc.y.abs()));
        }
        assert!(widest > FILL - 0.05, "it fills the frame: {widest}");
        // Zoomed in, the figure overflows the frame; turned, the eye goes round.
        let (zoomed, ..) = frame(&points, Vec3::X, 0.0, 0.0, 2.0, aspect);
        let clip = zoomed * Vec3::new(0.0, 0.0, 32.0).extend(1.0);
        assert!((clip.y / clip.w) > 1.0);
        let (_, side, _) = frame(&points, Vec3::X, 90.0, 0.0, 1.0, aspect);
        assert!(side.y > 50.0, "{side}");
    }

    #[test]
    fn the_backdrop_shows_where_the_drawing_is_clear() {
        let mut rgba = vec![200, 100, 50, 255, 200, 100, 50, 0];
        over_backdrop(&mut rgba, [2, 1], [255, 200, 50]);
        // The figure keeps its colour, its tones lifted (never darkened, white stays).
        let lifted = |value: f32| ((value / 255.0).powf(LIFT) * 255.0).round() as u8;
        assert_eq!(
            &rgba[..4],
            &[lifted(200.0), lifted(100.0), lifted(50.0), 255]
        );
        assert!(rgba[0] >= 200 && rgba[1] >= 100 && rgba[2] >= 50);
        assert_eq!(lifted(255.0), 255);
        // Where the drawing is clear the backdrop shows, opaque.
        assert_eq!(rgba[7], 255);
        assert_ne!(&rgba[4..7], &[200, 100, 50]);
    }

    #[test]
    fn a_blur_spreads_a_figure_into_a_soft_rim() {
        let mut alpha = vec![0.0; 21 * 21];
        alpha[10 * 21 + 10] = 1.0;
        let spread = blurred(&alpha, 21, 3);
        let total: f32 = spread.iter().sum();
        assert!((total - 1.0).abs() < 0.05, "kept: {total}");
        assert!(spread[10 * 21 + 10] < 1.0 && spread[10 * 21 + 12] > 0.0);
        assert_eq!(spread[0], 0.0, "beyond the reach");
    }
}
