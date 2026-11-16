//! A body fading away 30 s after it fell (`EV_BODYFADE`), drawn as cgame draws it
//! ([`sjk_client::legacy_body_fade`]; OpenJK `cg_ents.c`, the `ET_BODY` block of
//! `CG_General`, which EternalJK keeps): the body thinning out while a glow over it
//! brightens, the glow then burning away from the body's origin, with the side's sounds
//! and, on the dark side, the body crackling red.
//!
//! [`GpuState::track_body_fades`] starts a body's fade when the event arrives
//! ([`sjk_client::LegacyBodyFadeTracker`]) and turns off its `head_eyes_mouth`
//! surface, as `EV_BODYFADE` does, so the mouth's inside does not show through the
//! thinning face; [`submit`] draws its layers in place of the body.

use crate::{ActorInstance, ActorMesh, GpuState, entity_materials};
use sjk_client::body_fade_assets as assets;
use sjk_runtime::{EntityId, EntityKind};
use std::cell::Cell;

/// The surface `EV_BODYFADE` turns off (`G2SURFACEFLAG_OFF`).
const MOUTH_SURFACE: &str = "head_eyes_mouth";
/// Instance `view_flags` bit read by `stage_runtime.wgsl`: the stage's colour is meant for
/// rd-vanilla's display-space `GL_DST_COLOR GL_ONE` (the scene times one plus the colour,
/// on display values), so the program rewrites it for SJK's linear scene. The light side's
/// glow, `gfx/misc/personalshield`, blends so in both its stages (retail `effects.shader`);
/// blended in linear light it brightened the scene far less and only on the texture's
/// brightest texels, a faint blue speckle (hub report #190).
pub(crate) const DISPLAY_FILTER_ADD: u32 = 1 << 27;
/// `CHAN_AUTO`.
const CHAN_AUTO: sjk_audio::ChannelId = sjk_audio::ChannelId(0);

/// One body's fade (`centity_t::bodyFadeTime`, `dustTrailTime`).
#[derive(Debug)]
pub(crate) struct State {
    /// When `EV_BODYFADE` arrived, in presentation time.
    pub(crate) started: i64,
    /// `teamPowerType`: the player was on the light side (`ircg`).
    pub(crate) light_side: bool,
    /// The sounds were registered for this fade.
    registered: Cell<bool>,
    /// The burn's start sound was played.
    burn_heard: Cell<bool>,
    /// The last 100 ms step the light side's hum played in.
    hum_step: Cell<i64>,
    /// The last step the dark side's crackle was rolled in.
    crackle_step: Cell<i64>,
}

impl State {
    pub(crate) fn new(started: i64, light_side: bool) -> Self {
        Self {
            started,
            light_side,
            registered: Cell::new(false),
            burn_heard: Cell::new(false),
            hum_step: Cell::new(i64::MIN),
            crackle_step: Cell::new(i64::MIN),
        }
    }

    /// The frame cgame draws `now`; `None` before the event or once `bodyFadeTime`
    /// has passed (the body is then drawn as usual, though the server has unlinked
    /// it long before).
    pub(crate) fn frame(&self, now: i64) -> Option<sjk_client::BodyFadeFrame> {
        let elapsed = i32::try_from(now - self.started).ok()?;
        sjk_client::legacy_body_fade(elapsed)
    }

    /// The burn radius of the glow (`RB_CalcDisintegrateColors`), from the frame its
    /// burn started (`dustTrailTime`).
    fn threshold(&self, now: i64) -> f32 {
        (now - self.started - i64::from(assets::BURN_START_MILLIS)).max(0) as f32 * 0.045
    }
}

impl GpuState {
    /// Start or end each body's fade from `EV_BODYFADE` (`cg_event.c`).
    pub(crate) fn track_body_fades(&mut self) {
        for mesh in &mut self.actor_meshes {
            let Some(id) = mesh.entity_id.filter(|_| mesh.corpse_pool) else {
                stop(mesh);
                continue;
            };
            let started = u16::try_from(id.get().saturating_sub(1))
                .ok()
                .and_then(|number| self.body_fades.started(number));
            match (started, mesh.body_fade.as_ref()) {
                (Some(started), None) => start(mesh, started),
                (Some(started), Some(state)) if state.started != i64::from(started) => {
                    start(mesh, started);
                }
                (None, Some(_)) => stop(mesh),
                _ => {}
            }
        }
    }

    /// A body copied into slot `entity` starts unfaded (`CG_BodyQueueCopy`).
    pub(crate) fn reset_body_fade(&mut self, entity: u16) {
        self.body_fades.reset(entity);
    }
}

fn start(mesh: &mut ActorMesh, started: i32) {
    // `teamPowerType` is 0, the dark side, on a body no `ircg` named.
    let light_side = mesh
        .body_identity
        .as_ref()
        .is_some_and(|body| body.light_side);
    mesh.body_fade = Some(State::new(i64::from(started), light_side));
    mesh.surfaces
        .turn_off(&mesh.preview.mesh.hierarchy, MOUTH_SURFACE);
}

/// End a fade: a pooled mesh given to another body, which no `ircg` may copy surfaces
/// into, gets its mouth back.
fn stop(mesh: &mut ActorMesh) {
    if mesh.body_fade.take().is_some() {
        mesh.surfaces
            .restore(&mesh.preview.mesh.hierarchy, MOUTH_SURFACE);
    }
}

/// Everything [`submit`] draws into and plays on.
pub(crate) struct Frame<'a> {
    /// Filled up to its capacity, never past it.
    pub(crate) overrides: &'a mut Vec<entity_materials::OverrideInstance>,
    pub(crate) materials: crate::model_materials::Overrides,
    pub(crate) audio: Option<&'a mut crate::GameAudio>,
    pub(crate) vfs: &'a sjk_vfs::VirtualFileSystem,
    pub(crate) now: i64,
}

impl Frame<'_> {
    fn push(&mut self, instance: entity_materials::OverrideInstance) {
        if self.overrides.len() < self.overrides.capacity() {
            self.overrides.push(instance);
        }
    }
}

/// Draw body `mesh` fading as `state` says, in place of the body; false when it is not
/// fading now, and is drawn as usual.
pub(crate) fn submit(
    frame: &mut Frame<'_>,
    mesh: usize,
    state: &State,
    entity: &sjk_runtime::SceneEntity,
    body: ActorInstance,
) -> bool {
    debug_assert_eq!(entity.kind, EntityKind::Corpse);
    let Some(fade) = state.frame(frame.now) else {
        return false;
    };
    let mesh_ref = entity_materials::OverrideMesh::Actor(mesh);
    // 1. The body itself, thinning (`RF_FORCE_ENT_ALPHA`).
    if let Some(alpha) = fade.body_alpha {
        frame.push(entity_materials::OverrideInstance {
            mesh: mesh_ref,
            material: None,
            instance: body.with_forced_alpha(alpha),
            no_depth: false,
            forced_alpha: true,
        });
    }
    // 2. The glow: the side's shader, `RF_RGB_TINT` at `tMult`, then burning away
    // from the body's origin (`hitLoc` is `lerpOrigin`).
    let glow_material = if state.light_side {
        frame.materials.force(assets::LIGHT_SHADER)
    } else {
        Some(frame.materials.body_fade_dark)
    };
    if let Some(material) = glow_material {
        let mut glow = if fade.burning {
            let mut burning = body;
            burning.view_flags |= crate::disintegration::RF_DISINTEGRATE2;
            burning.light_direction = body.position;
            burning.light_directed = [state.threshold(frame.now), 0.0, 0.0];
            burning
        } else {
            let grey = f32::from(fade.glow) / 255.0;
            body.with_rgb_tint([grey; 4])
        };
        if state.light_side {
            glow.view_flags |= DISPLAY_FILTER_ADD;
        }
        frame.push(entity_materials::OverrideInstance {
            mesh: mesh_ref,
            material: Some(material),
            instance: glow,
            no_depth: false,
            forced_alpha: false,
        });
    }
    // 3. The dark side's red, crackling body for the first 3.4 s.
    let random = frame_random(entity.id, frame.now);
    if fade.extra
        && !state.light_side
        && let Some(material) = frame
            .materials
            .force(assets::ELECTRIC_SHADERS[usize::from(random & 1 == 1)])
    {
        frame.push(entity_materials::OverrideInstance {
            mesh: mesh_ref,
            material: Some(material),
            instance: body.with_rgb_tint([1.0, 0.0, 0.0, 1.0]),
            no_depth: false,
            forced_alpha: false,
        });
    }
    play_sounds(frame, state, &fade, entity.id, body.position);
    true
}

/// The burn's start sound, the light side's hum (EternalJK: at most once per 100 ms)
/// and the dark side's crackle, on the body (`CHAN_AUTO`).
fn play_sounds(
    frame: &mut Frame<'_>,
    state: &State,
    fade: &sjk_client::BodyFadeFrame,
    id: EntityId,
    origin: [f32; 3],
) {
    let Some(audio) = frame.audio.as_deref_mut() else {
        return;
    };
    if !state.registered.replace(true) {
        audio.register_effect_paths(
            frame.vfs,
            [
                assets::LIGHT_START_SOUND,
                assets::DARK_START_SOUND,
                assets::HUM_SOUND,
                assets::CRACKLE_SOUND,
            ],
        );
    }
    let source = sjk_audio::SourceId(id.get().saturating_sub(1) as u32);
    let now = frame.now;
    if fade.burning && !state.burn_heard.replace(true) {
        let path = if state.light_side {
            assets::LIGHT_START_SOUND
        } else {
            assets::DARK_START_SOUND
        };
        audio.play_on_channel(path, 1.0, origin, source, CHAN_AUTO);
    }
    if !fade.extra {
        return;
    }
    if state.light_side {
        let step = now.div_euclid(i64::from(assets::HUM_PERIOD_MILLIS));
        if fade.hum && state.hum_step.replace(step) != step {
            audio.play_on_channel(assets::HUM_SOUND, 1.0, origin, source, CHAN_AUTO);
        }
    } else {
        // EternalJK rolls `Q_flrand > 0.9` once per 100 ms (`cg.time % 100 <=
        // cg.frametime`, its fix for the loud body sound), as for the hum.
        let step = now.div_euclid(i64::from(assets::HUM_PERIOD_MILLIS));
        if state.crackle_step.replace(step) != step && frame_random(id, step).is_multiple_of(10) {
            audio.play_on_channel(assets::CRACKLE_SOUND, 1.0, origin, source, CHAN_AUTO);
        }
    }
}

/// A presentation random number for entity `id` at `salt`: the same within a frame,
/// different between bodies and frames.
fn frame_random(id: EntityId, salt: i64) -> u32 {
    let mixed = (id.get() as u32)
        .wrapping_mul(0x9e37_79b9)
        .wrapping_add(salt as u32)
        .wrapping_mul(0x85eb_ca6b)
        .rotate_left(13);
    mixed >> 8
}

#[cfg(test)]
mod tests {
    use super::State;

    #[test]
    fn the_glow_burns_from_its_own_start() {
        let state = State::new(10_000, true);
        assert_eq!(state.threshold(12_000), 0.0);
        assert!((state.threshold(12_513 + 1_000) - 45.0).abs() < 1e-4);
    }

    #[test]
    fn the_stage_program_reads_the_display_filter_bit() {
        assert_eq!(super::DISPLAY_FILTER_ADD.trailing_zeros(), 27);
        assert!(include_str!("stage_runtime.wgsl").contains("instance.view_flags & (1u << 27u)"));
    }

    #[test]
    fn a_fade_ends_after_a_minute() {
        let state = State::new(10_000, false);
        assert!(state.frame(9_999).is_none());
        assert!(state.frame(10_000).is_some());
        assert!(state.frame(70_000).is_none());
    }
}
