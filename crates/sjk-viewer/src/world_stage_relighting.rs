//! Recognize diffuse covers in legacy stacks that draw emission between two base layers.
use sjk_shader::{ShaderStage, StageBlend, TextureGenerator};

/// Stage metadata bit: sample live world light on an otherwise unlit diffuse cover.
/// This changes only day-mode composition; authored generators and alpha stay intact.
pub(crate) const DIFFUSE_COVER: u32 = 4;

/// A repeated base texture with alpha blending covers the preceding effect layers.
/// Its solid paint needs the same live light as the earlier diffuse layer. Additive,
/// glowing, animated-colour and unrelated textures must retain authored emission.
pub(crate) fn diffuse_cover(stage: &ShaderStage, stages: &[ShaderStage], lightmap: i32) -> bool {
    if stage.blend != StageBlend::Alpha || stage.glow || !identity_rgb(stage) {
        return false;
    }
    let lightmapped = lightmap >= 0
        && stages
            .iter()
            .any(|s| s.texture_generator == TextureGenerator::Lightmap);
    stages.iter().take_while(|s| *s != stage).any(|base| {
        let vertex_lit = lightmap == -3
            && base.rgb_generator.as_deref().is_some_and(|g| {
                g.eq_ignore_ascii_case("vertex") || g.eq_ignore_ascii_case("exactVertex")
            });
        (vertex_lit || (lightmapped && identity_rgb(base)))
            && matches!(base.blend, StageBlend::Replace | StageBlend::Filter)
            && !base.glow
            && !base.images.is_empty()
            && base.images.len() == stage.images.len()
            && base
                .images
                .iter()
                .zip(&stage.images)
                .all(|(a, b)| a.eq_ignore_ascii_case(b))
            && base.texture_generator == TextureGenerator::Base
            && stage.texture_generator == base.texture_generator
            && stage.texture_modifications == base.texture_modifications
            && stage.animation_frequency == base.animation_frequency
            && stage.one_shot == base.one_shot
            && stage.clamp == base.clamp
    })
}

fn identity_rgb(stage: &ShaderStage) -> bool {
    stage.rgb_constant.is_none()
        && stage.rgb_wave.is_none()
        && stage.rgb_generator.as_deref().is_none_or(|g| {
            g.eq_ignore_ascii_case("identity") || g.eq_ignore_ascii_case("identityLighting")
        })
}

/// Stage metadata bit: an unlit environment shine whose real-time light follows the
/// surface's live light relative to its bake (`world_lighting_mode.wgsl`).
pub(crate) const RELIT_SHINE: u32 = 8;

/// An unlit additive environment stage on a lightmapped or vertex-lit world surface:
/// the stock shine over polished paint (`tcGen environment` with `blendFunc add` or
/// `GL_SRC_ALPHA GL_ONE`, identity colour), such as `textures/yavin/obsidian_templebasic`.
///
/// Retail adds it to paint lit by the map's bake. In real-time lighting SJK relights the
/// paint (often far darker indoors than the bake) while the shine kept its authored
/// strength, and the scene adds it in linear light, so over the relit obsidian of
/// `mp/ffa1` the shine made 80% of the wall's brightness and hid its bricks (Sol's world
/// note, 12/10/2026: "way too reflective"). Flagged stages are scaled by the live light's
/// luminance over the baked light's (lightmap texel or vertex colour), at most 1: the
/// shine dims with the room and is never stronger than authored.
pub(crate) fn relit_shine(stage: &ShaderStage, lightmap: i32) -> bool {
    let additive = match &stage.blend {
        StageBlend::Add => true,
        StageBlend::Custom {
            source,
            destination,
        } => {
            destination.eq_ignore_ascii_case("gl_one")
                && (source.eq_ignore_ascii_case("gl_one")
                    || source.eq_ignore_ascii_case("gl_src_alpha"))
        }
        _ => false,
    };
    (lightmap >= 0 || lightmap == -3)
        && stage.texture_generator == TextureGenerator::Environment
        && additive
        && !stage.glow
        && identity_rgb(stage)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stage(text: &str) -> ShaderStage {
        let script = format!(
            "textures/t/s\n{{\n{{ map textures/t/base rgbGen vertex }}\n{{ {text} }}\n}}\n"
        );
        sjk_shader::parse_shader_script(script.as_bytes(), "shaders/t.shader")
            .expect("parses")
            .remove(0)
            .stages
            .remove(1)
    }

    #[test]
    fn unlit_environment_shines_follow_the_live_light() {
        // obsidian_templebasic's shine, and the plain additive one.
        let obsidian = stage(
            "map textures/byss/env_large_floor2 blendFunc GL_SRC_ALPHA GL_ONE \
             rgbGen identity alphaGen const 0.25 tcGen environment",
        );
        assert!(relit_shine(&obsidian, 6));
        assert!(relit_shine(&obsidian, -3));
        // Models and entities keep their own light.
        assert!(!relit_shine(&obsidian, -1));
        assert!(relit_shine(
            &stage("map textures/a/env blendFunc add tcGen environment"),
            0
        ));
        for other in [
            // Not environment-mapped, glowing, coloured, or not additive.
            "map textures/a/glow blendFunc add",
            "map textures/a/env blendFunc add tcGen environment glow",
            "map textures/a/env blendFunc add rgbGen wave sin 0 1 0 1 tcGen environment",
            "map textures/a/env blendFunc filter tcGen environment",
            "map textures/a/env blendFunc GL_DST_COLOR GL_ONE tcGen environment",
        ] {
            assert!(!relit_shine(&stage(other), 0), "{other}");
        }
    }
}
