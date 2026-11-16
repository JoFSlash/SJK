//! Load-time multitexture collapse.
use super::*;
use sjk_shader::{AlphaGen, RgbGen};

#[cfg(test)]
#[path = "world_stage_collapse_tests.rs"]
mod tests;

/// Apply rd-vanilla `CollapseMultitexture` at map load.
///
/// The decision mirrors `codemp/rd-vanilla/tr_shader.cpp:2442-2522`: `FinishShader`
/// calls it once, for the first two stages only (`tr_shader.cpp:3160`). The
/// stages need identical state apart from blend and depth-write, one of
/// the eight blend pairs of `tr_shader.cpp:2404-2432`, and identical colour
/// generators after `ParseStage` defaults ([`sjk_shader::StageColour`]), with
/// identical waveforms for wave generators. An add collapse also needs identity
/// colour. Constant colours, texture-coordinate generators and tcMods are not
/// compared. The collapsed pass keeps the first stage's colour, depth-write and
/// depth function, and carries a leading lightmap in its second bundle.
///
/// rd-vanilla also requires both stages to be active, which a stage without an
/// image is not. SJK draws such a stage with the surface's own texture instead,
/// as its implicit default stages rely on, so that check has no counterpart here.
pub(crate) fn collapse_multitexture(stages: &[ShaderStage]) -> Vec<CompiledStage> {
    let mut result = Vec::with_capacity(stages.len());
    let mut rest = stages;
    if let Some((passes, tail)) = glossy_base(stages) {
        result.extend(passes);
        rest = tail;
    } else if let [first, second, tail @ ..] = stages
        && let Some((operator, output_blend)) = collapse_pair(first, second)
    {
        let (mut primary, secondary) = if first.texture_generator == TextureGenerator::Lightmap {
            (second.clone(), first.clone())
        } else {
            (first.clone(), second.clone())
        };
        // rd-vanilla swaps only the texture bundles: the colour stays stage 0's.
        primary.rgb_generator = first.rgb_generator.clone();
        primary.alpha_generator = first.alpha_generator.clone();
        primary.resolved_colour = first.resolved_colour;
        primary.rgb_wave = first.rgb_wave.clone();
        primary.alpha_wave = first.alpha_wave.clone();
        primary.rgb_constant = first.rgb_constant;
        primary.alpha_constant = first.alpha_constant;
        result.push(CompiledStage {
            primary,
            secondary: Some(secondary),
            combine: operator,
            output_blend,
            output_depth_write: first.depth_write,
            output_depth_function: first.depth_function,
            glow: first.glow,
        });
        rest = tail;
    }
    result.extend(rest.iter().map(|stage| CompiledStage {
        primary: stage.clone(),
        secondary: None,
        combine: CollapseOperator::None,
        output_blend: stage.blend.clone(),
        output_depth_write: stage.depth_write,
        output_depth_function: stage.depth_function,
        glow: stage.glow,
    }));
    result
}

/// A glossy base: an environment stage, paint alpha-blended over it (its alpha masks the
/// shine) and the lightmap filtered over both, as Korriban's walls draw
/// (`textures/korriban/door`, `entrance_top`, `os_outsidebased`, `os_basic_pillarb`:
/// `map textures/common/etest4 tcGen environment`, the paint with
/// `blendFunc GL_SRC_ALPHA GL_ONE_MINUS_SRC_ALPHA`, then `$lightmap` with
/// `GL_DST_COLOR GL_ZERO`). rd-vanilla collapses none of it and draws three passes,
/// `lightmap × (paint × a + shine × (1 − a))`.
///
/// SJK draws the same colour in two passes, each a lightmap collapse: the shine times
/// the lightmap (opaque, writing depth), then the paint times the lightmap, blended
/// over it with the paint's alpha: `lightmap × paint × a + lightmap × shine × (1 − a)`.
/// The paint pass is then an ordinary lightmapped diffuse pair, so material maps apply
/// to it (`material_maps::diffuse_bundle`), where the three passes kept the lightmap
/// out of reach of the paint. The paint's alpha stays a mask over the shine: the first
/// pass is opaque, so nothing behind the wall shows.
///
/// Any colour other than identity, a wave, an alpha test or a different depth
/// function keeps the authored passes.
fn glossy_base(stages: &[ShaderStage]) -> Option<([CompiledStage; 2], &[ShaderStage])> {
    let [shine, paint, lightmap, tail @ ..] = stages else {
        return None;
    };
    let plain = |stage: &ShaderStage| {
        matches!(
            stage.resolved_colour.rgb,
            RgbGen::Identity | RgbGen::IdentityLighting
        ) && matches!(
            stage.resolved_colour.alpha,
            AlphaGen::Identity | AlphaGen::Skip
        ) && stage.rgb_wave.is_none()
            && stage.alpha_wave.is_none()
            && alpha_test(stage.alpha_function.as_deref()) == 0
            && stage.depth_function == shine.depth_function
            && !stage.glow
    };
    let filter = matches!(
        blend_bits(&lightmap.blend),
        BlendBits::ModulateSource | BlendBits::ModulateDestination
    );
    if shine.texture_generator != TextureGenerator::Environment
        || shine.blend != StageBlend::Replace
        || paint.texture_generator != TextureGenerator::Base
        || paint.blend != StageBlend::Alpha
        || lightmap.texture_generator != TextureGenerator::Lightmap
        || !filter
        || ![shine, paint, lightmap].into_iter().all(plain)
    {
        return None;
    }
    let pass = |diffuse: &ShaderStage, output_blend: StageBlend| CompiledStage {
        primary: diffuse.clone(),
        secondary: Some(lightmap.clone()),
        combine: CollapseOperator::Modulate,
        output_blend,
        output_depth_write: diffuse.depth_write,
        output_depth_function: diffuse.depth_function,
        glow: false,
    };
    Some((
        [
            pass(shine, StageBlend::Replace),
            pass(paint, StageBlend::Alpha),
        ],
        tail,
    ))
}

fn collapse_pair(
    first: &ShaderStage,
    second: &ShaderStage,
) -> Option<(CollapseOperator, StageBlend)> {
    // The GPU stage carries `tcGen vector` axes for its primary bundle only.
    if first.texture_generator == TextureGenerator::Vector
        || second.texture_generator == TextureGenerator::Vector
    {
        return None;
    }
    // State bits other than blend and depth-write: alpha test and depth function.
    if first.depth_function != second.depth_function
        || alpha_test(first.alpha_function.as_deref())
            != alpha_test(second.alpha_function.as_deref())
    {
        return None;
    }
    let (operator, output) = collapse_rule(&first.blend, &second.blend)?;
    let colour = first.resolved_colour;
    if colour != second.resolved_colour {
        return None;
    }
    if operator == CollapseOperator::Add && colour.rgb != RgbGen::Identity {
        return None;
    }
    if colour.rgb == RgbGen::Waveform
        && !same_wave(first.rgb_wave.as_ref(), second.rgb_wave.as_ref())
    {
        return None;
    }
    if colour.alpha == AlphaGen::Waveform
        && !same_wave(first.alpha_wave.as_ref(), second.alpha_wave.as_ref())
    {
        return None;
    }
    Some((operator, output))
}

/// `NameToAFunc`: names other than these four leave the alpha test off.
fn alpha_test(function: Option<&str>) -> u8 {
    match function.map(str::to_ascii_lowercase).as_deref() {
        Some("gt0") => 1,
        Some("lt128") => 2,
        Some("ge128") => 3,
        Some("ge192") => 4,
        _ => 0,
    }
}

/// The `memcmp` of two `waveForm_t`, with `NameToGenFunc`'s sine fallback.
fn same_wave(a: Option<&WaveForm>, b: Option<&WaveForm>) -> bool {
    let key = |wave: Option<&WaveForm>| {
        wave.map(|wave| {
            let function = match wave.function.to_ascii_lowercase().as_str() {
                "square" => 1,
                "triangle" => 2,
                "sawtooth" => 3,
                "inversesawtooth" => 4,
                "noise" => 5,
                "random" => 6,
                _ => 0,
            };
            (
                function,
                wave.base.to_bits(),
                wave.amplitude.to_bits(),
                wave.phase.to_bits(),
                wave.frequency.to_bits(),
            )
        })
    };
    key(a) == key(b)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BlendBits {
    Replace,
    ModulateSource,
    ModulateDestination,
    Add,
    Other,
}

fn collapse_rule(
    first: &StageBlend,
    second: &StageBlend,
) -> Option<(CollapseOperator, StageBlend)> {
    use BlendBits::{Add, ModulateDestination, ModulateSource, Replace};
    use CollapseOperator::{Add as AddTextures, Modulate};
    let first = blend_bits(first);
    let second = blend_bits(second);
    match (first, second) {
        (Replace, ModulateSource) | (Replace, ModulateDestination) => {
            Some((Modulate, StageBlend::Replace))
        }
        (ModulateSource, ModulateSource)
        | (ModulateDestination, ModulateSource)
        | (ModulateSource, ModulateDestination)
        | (ModulateDestination, ModulateDestination) => Some((Modulate, StageBlend::Filter)),
        (Replace, Add) => Some((AddTextures, StageBlend::Replace)),
        (Add, Add) => Some((AddTextures, StageBlend::Add)),
        _ => None,
    }
}

fn blend_bits(blend: &StageBlend) -> BlendBits {
    match blend {
        StageBlend::Replace => BlendBits::Replace,
        StageBlend::Filter => BlendBits::ModulateSource,
        StageBlend::Add => BlendBits::Add,
        StageBlend::Custom {
            source,
            destination,
        } if source.eq_ignore_ascii_case("gl_zero")
            && destination.eq_ignore_ascii_case("gl_src_color") =>
        {
            BlendBits::ModulateDestination
        }
        _ => BlendBits::Other,
    }
}
