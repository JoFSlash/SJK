//! JA+/jaPRO black blades: JoF EJK #50 (`4ebc7e7e`, `ClampSaberColor`).
//! On a plugin server a set plugin bit (or `cg_blackSabers 0`) turns black into
//! orange; elsewhere colour 11 keeps the client's ordinary fallback.
//! Raw colour numbers stay in this adapter rather than the runtime world.

use super::BladeColor;
use sjk_client::{CompatProfile, LegacyClientInfo};
use sjk_protocol::{GameState, LEGACY_MAX_CLIENTS};

// Lowercase for allocation-free registry reads in the frame path.
pub(crate) const CVAR: &str = "cg_blacksabers";
const DISABLE_BIT: i64 = 1 << 3;

/// Server families (`sjk_mod::known_servers` ids) that carry black blades besides
/// the JA+ and jaPRO game modules, each with why. The player's `cg_blackSabers` and
/// `cp_pluginDisable` bit 3 still decide, as on JA+.
pub(crate) const FAMILIES: &[(&str, &str)] = &[(
    "sjk",
    "SJK's own server (devmap, Create game) passes colour 11 on as it is; JoF EJK \
     shows black on its own devmap, where JoF's JA+-based game module runs (Dave, \
     27/10/2026), and SJK's devmap runs SJK's server",
)];

/// Servers with the black-blade contract: by the session's [`CompatProfile`] (its
/// documented serverinfo table), as EJK #50 permits JA+ and its jaPRO lineage and
/// TaystJK inherits jaPRO's colour enumeration, or by a family in [`FAMILIES`].
fn supports_black(game: &GameState) -> bool {
    let profile = CompatProfile::from_game_state(game);
    if matches!(
        profile,
        CompatProfile::JaPlus { .. } | CompatProfile::TaystJk
    ) {
        return true;
    }
    let info = game
        .config_string(0)
        .and_then(|bytes| std::str::from_utf8(bytes).ok())
        .unwrap_or_default();
    let server = sjk_mod::Server {
        kind: crate::mods::server_kind(&profile),
        address: String::new(),
        info: info.to_owned(),
    };
    sjk_mod::known_servers::identify(&server)
        .is_some_and(|family| FAMILIES.iter().any(|(id, _)| *id == family.id))
}

/// The serverinfo and clientinfo part of the policy, rebuilt only after
/// `CS_SERVERINFO` or a `CS_PLAYERS` slot changed ([`Self::invalidate`]).
#[derive(Clone, Copy, Default)]
pub(crate) struct Cache {
    valid: bool,
    supported: bool,
    black: [u32; 2],
}

impl Cache {
    /// Called by the configstring dispatch for slots 0 and `CS_PLAYERS`.
    pub(crate) fn invalidate(&mut self) {
        self.valid = false;
    }

    /// The frame's policy; the cvars and mutes are read each frame, the
    /// configstrings only when they changed.
    pub(crate) fn policy(
        &mut self,
        game: Option<&GameState>,
        enabled: bool,
        disabled: i64,
        muted: u32,
    ) -> Policy {
        let Some(game) = game else {
            return Policy::default();
        };
        if !self.valid {
            *self = Self::scan(game);
        }
        Policy {
            black: self.black,
            supported: self.supported,
            allowed: self.supported && enabled && disabled & DISABLE_BIT == 0,
            muted,
        }
    }

    fn scan(game: &GameState) -> Self {
        let mut cache = Self {
            valid: true,
            supported: supports_black(game),
            black: [0; 2],
        };
        for client in 0..LEGACY_MAX_CLIENTS {
            let Some(raw) = game.config_string(1131 + client) else {
                continue;
            };
            let info = LegacyClientInfo::new(raw);
            for (hand, key) in ["c1", "c2"].into_iter().enumerate() {
                let color = info
                    .text(key)
                    .map(sjk_client::split_color_value)
                    .map(|value| value.0);
                if color.is_some_and(|color| color >= 0 && color % 12 == 11) {
                    cache.black[hand] |= 1 << client;
                }
            }
        }
        cache
    }
}

/// One frame's black-blade decisions, reused by held and flying blades.
#[derive(Clone, Copy, Default)]
pub(crate) struct Policy {
    black: [u32; 2],
    supported: bool,
    allowed: bool,
    muted: u32,
}

impl Policy {
    /// Uncached, for tests and one-off callers.
    #[cfg(test)]
    pub(crate) fn new(game: Option<&GameState>, enabled: bool, disabled: i64, muted: u32) -> Self {
        Cache::default().policy(game, enabled, disabled, muted)
    }

    /// This policy with entity `entity`'s black blade dropped from each hand where
    /// `replaced` holds (a team colour forced over it, `team_sabers.rs`).
    pub(crate) fn without(mut self, entity: u64, replaced: [bool; 2]) -> Self {
        if let Some(client) = entity
            .checked_sub(1)
            .filter(|client| *client < LEGACY_MAX_CLIENTS as u64)
        {
            for (bits, replaced) in self.black.iter_mut().zip(replaced) {
                if replaced {
                    *bits &= !(1 << client);
                }
            }
        }
        self
    }

    pub(crate) fn color(self, entity: u64, hand: usize, fallback: BladeColor) -> BladeColor {
        let Some(client) = entity
            .checked_sub(1)
            .filter(|client| *client < LEGACY_MAX_CLIENTS as u64)
        else {
            return fallback;
        };
        if self.muted & (1 << client) != 0 || !self.supported {
            return fallback;
        }
        if self
            .black
            .get(hand)
            .is_none_or(|bits| bits & (1 << client) == 0)
        {
            return fallback;
        }
        if self.allowed {
            BladeColor::Black
        } else {
            BladeColor::Retail(crate::saber::Color::Orange)
        }
    }
}

#[cfg(test)]
#[path = "black_saber_tests.rs"]
mod tests;

/// Prefer installed community textures (EternalJK's own come from its folder,
/// `asset_search_paths.rs` `eternaljk_black_sabers`); no community asset is bundled or
/// downloaded. Fallback coverage uses the engine's neutral profiles for the glow and
/// core and [`generated_trail`] for the trail, with black RGB and alpha.
pub(crate) fn image(
    vfs: &sjk_vfs::VirtualFileSystem,
    shaders: &sjk_shader::ShaderCatalog,
    kind: &str,
) -> image::RgbaImage {
    let name = format!("gfx/effects/sabers/black{kind}");
    if let Ok(image) = crate::shader_image::load_shader_image(vfs, shaders, &name) {
        return image;
    }
    if kind == "trail" {
        return generated_trail();
    }
    let (size, mut pixels) = if kind == "glow" {
        (super::glow_size(), super::neutral_glow())
    } else {
        (super::core_size(), super::neutral_core())
    };
    for pixel in pixels.chunks_exact_mut(4) {
        let alpha = pixel[0];
        pixel.copy_from_slice(&[0, 0, 0, alpha]);
    }
    image::RgbaImage::from_raw(size[0], size[1], pixels).expect("generated RGBA profile")
}

/// The generated trail's size, as jaPRO's `blacktrail` (128 by 256).
const TRAIL_SIZE: [u32; 2] = [128, 256];

/// A black trail picture shaped as jaPRO's `gfx/effects/sabers/blacktrail` (JoF EJK's
/// `japro-assets.pk3`), for players without it: solid over the slices' first two thirds
/// of U (U is a slice's age as it scrolls, `saber_trail.wgsl`), fading out by 0.8, and
/// solid along V but for its two ends (the hilt and the tip). The core profile it used
/// to fall back to is solid only in a narrow middle band of U, so a swing showed a
/// translucent fan, clear by the blade and banded behind it, where JoF EJK draws a solid
/// black wedge (Dave's test, 27/10/2026).
pub(crate) fn generated_trail() -> image::RgbaImage {
    fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
        let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
    let [width, height] = TRAIL_SIZE;
    image::RgbaImage::from_fn(width, height, |x, y| {
        let u = (x as f32 + 0.5) / width as f32;
        let v = (y as f32 + 0.5) / height as f32;
        let across = 1.0 - smoothstep(0.66, 0.8, u);
        let along = smoothstep(0.03, 0.12, v) * (1.0 - smoothstep(0.8, 0.97, v));
        image::Rgba([0, 0, 0, (across * along * 255.0).round() as u8])
    })
}
