//! A scripted shader naming a stage image that is not there, as retail treats it.
//!
//! `ParseStage` fails on a `map`, `clampmap` or `animMap` frame whose image
//! `R_FindImageFile` cannot find (rd-vanilla `tr_shader.cpp:1218-1243`, `:1275-1279`),
//! printing "R_FindImageFile could not find"; `$whiteimage` and `$lightmap` are never
//! looked up (`:1196-1205`). `ParseShader` then stops at that stage (`:2136`), which
//! never becomes active (`:2140`), so neither it nor anything after it in the script
//! is kept, and `R_FindShader` flags the shader as a default one (`:3460-3463`). Who
//! asked for it decides what draws:
//! - world surfaces take `tr.defaultShader`, the grey box (`tr_bsp.cpp:339`), and so do
//!   shaders named inside a model, whose flagged handle is 0 (`tr_model.cpp:248`,
//!   `tr_ghoul2.cpp:4307`);
//! - a skin keeps the flagged shader itself (`tr_skin.cpp:168`, `tr_ghoul2.cpp:2447`),
//!   drawn with the stages that parsed before the missing one: none at all leaves the
//!   surface undrawn.
//!
//! SJK's materials do not say whether a model's shader came from its skin, so model
//! surfaces (`LIGHTMAP_NONE`, players and other entities) take the skin's way, the case
//! players meet, and lit world surfaces the default shader. Before this, the missing
//! stage was drawn with a magenta checker (Nib's report, 15/10/2026).

use sjk_shader::{ShaderCatalog, ShaderDefinition, ShaderStage, TextureGenerator};
use sjk_vfs::VirtualFileSystem;
use std::collections::HashSet;
use std::error::Error;
use std::sync::Mutex;

/// What a material draws instead of its script.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Fallback {
    /// Every stage image is there: the script as written.
    Keep,
    /// A skin's flagged shader: only the stages before this index.
    Truncate(usize),
    /// `tr.defaultShader`.
    Default,
}

/// Decide once, when the material is built, for `definition` drawn with `lightmap`.
pub(crate) fn fallback(
    vfs: &VirtualFileSystem,
    shaders: &ShaderCatalog,
    definition: Option<&ShaderDefinition>,
    lightmap: i32,
) -> Result<Fallback, Box<dyn Error>> {
    let Some(definition) = definition else {
        return Ok(Fallback::Keep);
    };
    let mut missing = None;
    for (index, stage) in definition.stages.iter().enumerate() {
        if let Some(name) = missing_image(stage, |name| {
            Ok::<_, Box<dyn Error>>(shaders.resolve_stage_image(vfs, name)?.is_some())
        })? {
            missing = Some((index, name));
            break;
        }
    }
    let Some((index, name)) = missing else {
        return Ok(Fallback::Keep);
    };
    warn_once(name, &definition.name);
    Ok(decide(index, lightmap))
}

/// The fallback for a script whose stage `index` names a missing image.
fn decide(index: usize, lightmap: i32) -> Fallback {
    if lightmap == crate::world_stage::LIGHTMAP_NONE {
        Fallback::Truncate(index)
    } else {
        Fallback::Default
    }
}

/// The first image `stage` names that `exists` does not find, as `ParseStage` looks
/// them up: its `map`/`clampmap` image or every `animMap` frame, never `$lightmap`,
/// `$whiteimage` or `*white`. A stage without images (a `videoMap`, or none named) is
/// not a missing image.
fn missing_image<E>(
    stage: &ShaderStage,
    mut exists: impl FnMut(&str) -> Result<bool, E>,
) -> Result<Option<&str>, E> {
    if stage.texture_generator == TextureGenerator::Lightmap || stage.video.is_some() {
        return Ok(None);
    }
    for name in &stage.images {
        let special = ["$lightmap", "$whiteimage", "*white"]
            .iter()
            .any(|special| name.eq_ignore_ascii_case(special));
        if !special && !exists(name)? {
            return Ok(Some(name));
        }
    }
    Ok(None)
}

/// Retail's warning, once per image and shader.
fn warn_once(image: &str, shader: &str) {
    static SEEN: Mutex<Option<HashSet<(String, String)>>> = Mutex::new(None);
    let mut seen = SEEN.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if seen
        .get_or_insert_with(HashSet::new)
        .insert((image.to_owned(), shader.to_owned()))
    {
        crate::log::progress(format_args!(
            "WARNING: R_FindImageFile could not find '{image}' in shader '{shader}'"
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stage(images: &[&str]) -> ShaderStage {
        let mut stage = crate::world_stage::default_stage();
        stage.images = images.iter().map(|name| (*name).to_owned()).collect();
        stage
    }

    fn found(name: &str) -> Result<bool, ()> {
        Ok(!name.contains("gone"))
    }

    #[test]
    fn special_images_are_never_missing() {
        for name in ["$lightmap", "$whiteimage", "*white", "*WHITE"] {
            assert_eq!(
                missing_image(&stage(&[name]), |_| Ok::<_, ()>(false)),
                Ok(None)
            );
        }
        assert_eq!(missing_image(&stage(&["textures/a"]), found), Ok(None));
        assert_eq!(
            missing_image(&stage(&["textures/gone"]), found),
            Ok(Some("textures/gone"))
        );
    }

    #[test]
    fn one_missing_anim_map_frame_fails_the_stage() {
        let frames = stage(&["fx/frame1", "fx/gone2", "fx/frame3"]);
        assert_eq!(missing_image(&frames, found), Ok(Some("fx/gone2")));
    }

    #[test]
    fn models_keep_the_stages_before_and_the_world_takes_the_default_shader() {
        // Nib's purple bits: a skin's glow stage naming an image that is not there.
        assert_eq!(
            decide(1, crate::world_stage::LIGHTMAP_NONE),
            Fallback::Truncate(1)
        );
        assert_eq!(decide(1, 0), Fallback::Default);
        assert_eq!(
            decide(0, crate::world_stage::LIGHTMAP_BY_VERTEX),
            Fallback::Default
        );
    }
}
