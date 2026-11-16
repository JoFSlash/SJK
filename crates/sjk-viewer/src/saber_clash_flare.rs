//! View projection and BSP visibility for codemp's screen-space clash flare, with
//! EternalJK's switches for it (`cg_saberClash`, `cg_saberClashSize`, JoF EJK
//! `cg_draw.c` `CG_SaberClashFlare`) and SJK's `cg_reduceFlashes`
//! ([`crate::effect_flash::REDUCE_CVAR`]), which turns it off with every other bright
//! flash (`docs/client.md`, "Bright flashes").

use glam::{Mat4, Vec3};
use sjk_bsp::{Aabb, Bsp, TraceScratch};
use sjk_client::{LegacySaberClashFlare, LegacySaberClashVisibility};

const CONTENTS_SOLID: u32 = 1; // codemp/qcommon/surfaceflags.h:18

/// The flare's picture (`cgs.media.saberClashFlare`, `cg_main.c`).
pub(crate) const PICTURE: &str = "gfx/effects/saberFlare";

/// `cg_saberClash`: EternalJK's switch for the flare, on by default.
pub(crate) const CVAR: &str = "cg_saberClash";
/// `cg_saberClashSize`: EternalJK's scale of the flare, 0 to 2, 1 by default.
pub(crate) const SIZE_CVAR: &str = "cg_saberClashSize";

/// How the flare is drawn, read from the settings once a frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Options {
    /// `cg_saberClash` on and `cg_reduceFlashes` off.
    pub(crate) enabled: bool,
    /// `cg_saberClashSize`, clamped to 0..2 as EternalJK's `CG_SaberClashSizeChange`.
    pub(crate) size: f32,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            enabled: true,
            size: 1.0,
        }
    }
}

impl Options {
    /// The flare's settings; without a console, retail's flare.
    pub(crate) fn read(console: Option<&crate::console::ViewerConsole>) -> Self {
        let Some(console) = console else {
            return Self::default();
        };
        Self {
            enabled: console.bool_cvar(CVAR).unwrap_or(true)
                && !console
                    .bool_cvar(crate::effect_flash::REDUCE_CVAR)
                    .unwrap_or(false),
            size: console
                .float_cvar(SIZE_CVAR)
                .map_or(1.0, |size| (size as f32).clamp(0.0, 2.0)),
        }
    }

    /// EternalJK's `v * cg_saberClashSize`, kept above its 0.001 floor.
    fn scale(self, picture_scale: f32) -> f32 {
        (picture_scale * self.size).max(0.001)
    }
}

/// Screen-space quad consumed by the existing additive particle pipeline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ProjectedFlare {
    pub(crate) center_ndc: [f32; 2],
    pub(crate) half_extent_ndc: [f32; 2],
    pub(crate) picture_scale: f32,
}

/// Evaluate the legacy flare and project its virtual 640x480 picture rect.
///
/// The point trace reuses `Bsp::trace_box`, the same collision implementation
/// used by pmove and the third-person camera; no parallel trace exists here.
#[allow(clippy::too_many_arguments)]
pub(crate) fn project(
    options: Options,
    flare: &LegacySaberClashFlare,
    cg_time: i32,
    view_origin: Vec3,
    view_forward: Vec3,
    view_projection: Mat4,
    aspect: f32,
    bsp: &Bsp,
    scratch: &mut TraceScratch,
) -> (LegacySaberClashVisibility, Option<ProjectedFlare>) {
    if !options.enabled {
        return (LegacySaberClashVisibility::NotActive, None);
    }
    let bounds = Aabb::new([0.0; 3], [0.0; 3]).expect("zero point bounds are valid");
    let sample = flare.sample(
        cg_time,
        view_origin.to_array(),
        view_forward.to_array(),
        |start, end| {
            bsp.trace_box_with(scratch, start, end, bounds, CONTENTS_SOLID)
                .fraction
        },
    );
    if sample.visibility != LegacySaberClashVisibility::Visible {
        return (sample.visibility, None);
    }
    let clip = view_projection * Vec3::from_array(sample.position).extend(1.0);
    if clip.w <= 0.01 {
        return (LegacySaberClashVisibility::BehindView, None);
    }
    let center = clip.truncate() / clip.w;
    // CG_DrawPic uses virtual 640x480 coordinates. x/y are each scaled to the
    // current framebuffer by the UI transform, so these NDC extents are
    // resolution independent (cg_draw.c:5397-5399).
    // EternalJK scales the picture's width by `cgs.widthRatioCoef` (640x480 over the
    // window's shape), so the flare stays round on a wide screen instead of
    // stretching with the virtual 640-wide canvas.
    let width_ratio = if aspect > 0.0 {
        (4.0 / 3.0) / aspect
    } else {
        1.0
    };
    let picture_scale = options.scale(sample.picture_scale);
    let half_extent_ndc = [
        picture_scale * 300.0 / 320.0 * width_ratio,
        picture_scale * 300.0 / 240.0,
    ];
    (
        LegacySaberClashVisibility::Visible,
        Some(ProjectedFlare {
            center_ndc: [center.x, center.y],
            half_extent_ndc,
            picture_scale,
        }),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn append(
    options: Options,
    flare: &LegacySaberClashFlare,
    cg_time: i32,
    view_origin: Vec3,
    view_forward: Vec3,
    view_projection: Mat4,
    aspect: f32,
    bsp: &Bsp,
    scratch: &mut TraceScratch,
    atlas: &crate::ParticleAtlas,
    output: &mut Vec<crate::EntityInstance>,
) {
    let (_, projected) = project(
        options,
        flare,
        cg_time,
        view_origin,
        view_forward,
        view_projection,
        aspect,
        bsp,
        scratch,
    );
    let Some(projected) = projected else {
        return;
    };
    let Some(layer) = atlas.first_layer(PICTURE, 0.0) else {
        return;
    };
    output.push(crate::EntityInstance {
        position: [projected.center_ndc[0], projected.center_ndc[1], 0.0],
        kind: 6,
        size: 1.0,
        alpha: layer.alpha,
        uv_rect: layer.uv_rect,
        color: [0.8 * layer.rgb, 0.8 * layer.rgb, 0.8 * layer.rgb, 1.0],
        direction: [
            projected.half_extent_ndc[0],
            projected.half_extent_ndc[1],
            0.0,
        ],
        rotation: 0.0,
        uv_transform: layer.uv_transform,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::ViewerConsole;

    /// Hub report #138 (Piza): the clash flare follows EternalJK's switch and size, and
    /// `cg_reduceFlashes` turns it off whatever they say.
    #[test]
    fn the_flare_follows_its_settings_and_reduced_flashes() {
        assert_eq!(Options::read(None), Options::default());
        let directory = tempfile::tempdir().unwrap();
        let mut console = ViewerConsole::new(directory.path().join("config.cfg")).unwrap();
        let defaults = Options::read(Some(&console));
        assert_eq!(defaults, Options::default(), "on, retail's size");
        assert_eq!(defaults.scale(1.5), 1.5);
        assert!(console.set_cvar(SIZE_CVAR, "0.5"));
        let half = Options::read(Some(&console));
        assert!(half.enabled);
        assert_eq!(half.scale(1.5), 0.75);
        // EternalJK's floor of 0.001, and its 0 to 2 range.
        assert!(console.set_cvar(SIZE_CVAR, "0"));
        assert_eq!(Options::read(Some(&console)).scale(1.5), 0.001);
        assert!(console.set_cvar(SIZE_CVAR, "5"));
        assert_eq!(Options::read(Some(&console)).size, 2.0);
        assert!(console.set_cvar(CVAR, "0"));
        assert!(!Options::read(Some(&console)).enabled);
        assert!(console.set_cvar(CVAR, "1"));
        assert!(console.set_cvar(crate::effect_flash::REDUCE_CVAR, "1"));
        assert!(!Options::read(Some(&console)).enabled);
    }
}
