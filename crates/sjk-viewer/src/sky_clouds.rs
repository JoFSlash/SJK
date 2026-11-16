//! A sky shader's own stages: rd-vanilla's cloud layer.
//!
//! `RB_StageIteratorSky` draws the outer box, then the shader's stages over it as
//! clouds (`R_BuildCloudData`, `rd-vanilla/tr_sky.cpp`): the box's faces, the bottom
//! one left out, with texture coordinates projected onto a dome `cloudheight` above
//! a world of radius 4096 (`R_InitSkyTexCoords`), and each stage drawn by the generic
//! iterator with its tcMods, colour and blend. The sky program does it per pixel from
//! the view direction; this packs the stages it needs.
use bytemuck::{Pod, Zeroable};
use image::RgbaImage;
use sjk_shader::{ShaderDefinition, ShaderStage};
use sjk_vfs::VirtualFileSystem;
use std::error::Error;
use std::sync::Arc;

use crate::world_stage;

/// rd-vanilla's `MAX_SHADER_STAGES`.
const MAX_STAGES: usize = 8;
const MAX_TEXMODS: usize = 4;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct GpuCloudStage {
    tcmod_a: [[f32; 4]; MAX_TEXMODS],
    tcmod_b: [[f32; 4]; MAX_TEXMODS],
    /// Image layer, tcMod count, source and destination blend factor codes.
    control: [f32; 4],
    /// Constant colour (`rgbGen const`/`alphaGen const`), white otherwise.
    colour: [f32; 4],
    rgb_wave: [f32; 4],
    alpha_wave: [f32; 4],
    /// rgb from a wave, alpha from a wave, their wave function codes.
    generators: [f32; 4],
    /// Alpha function code, `clampmap`.
    test: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct GpuClouds {
    /// Stage count, cloud height.
    header: [f32; 4],
    stages: [GpuCloudStage; MAX_STAGES],
}

/// The packed stages and one image layer per stage.
pub(super) type Compiled = (GpuClouds, Vec<Arc<RgbaImage>>);

/// The cloud stages of `definition` and their images, one layer each; no stage, no clouds.
pub(super) fn compile(
    definition: &ShaderDefinition,
    vfs: &VirtualFileSystem,
    shaders: &sjk_shader::ShaderCatalog,
) -> Result<Compiled, Box<dyn Error>> {
    let mut clouds = GpuClouds::zeroed();
    let mut images = Vec::new();
    let Some(sky) = &definition.sky else {
        return Ok((clouds, images));
    };
    for stage in &definition.stages {
        if images.len() == MAX_STAGES {
            break;
        }
        // An animMap shows its first frame; a stage whose image is missing is left out.
        let Some(image) = stage_image(stage, vfs, shaders)? else {
            continue;
        };
        clouds.stages[images.len()] = compile_stage(stage, images.len());
        images.push(image);
    }
    clouds.header = [images.len() as f32, sky.cloud_height, 0.0, 0.0];
    Ok((clouds, images))
}

fn stage_image(
    stage: &ShaderStage,
    vfs: &VirtualFileSystem,
    shaders: &sjk_shader::ShaderCatalog,
) -> Result<Option<Arc<RgbaImage>>, Box<dyn Error>> {
    let Some(name) = stage.images.first() else {
        return Ok(None);
    };
    if name.eq_ignore_ascii_case("$whiteimage") || name.eq_ignore_ascii_case("*white") {
        return Ok(Some(Arc::new(RgbaImage::from_pixel(
            1,
            1,
            image::Rgba([255; 4]),
        ))));
    }
    let Some(path) = shaders.resolve_stage_image(vfs, name)? else {
        return Ok(None);
    };
    crate::decoded_image_cache::cached_decoded_image(vfs, path.as_str())
}

fn compile_stage(stage: &ShaderStage, layer: usize) -> GpuCloudStage {
    let mut result = GpuCloudStage::zeroed();
    for (index, modification) in stage
        .texture_modifications
        .iter()
        .take(MAX_TEXMODS)
        .enumerate()
    {
        (result.tcmod_a[index], result.tcmod_b[index]) = world_stage::compile_tcmod(modification);
    }
    // The first stage without a blendFunc is opaque and covers the box, as in retail.
    let (source, destination) = world_stage::blend_factors(&stage.blend);
    result.control = [
        layer as f32,
        stage.texture_modifications.len().min(MAX_TEXMODS) as f32,
        factor_code(source),
        factor_code(destination),
    ];
    let rgb = stage.rgb_constant.unwrap_or([1.0; 3]);
    result.colour = [rgb[0], rgb[1], rgb[2], stage.alpha_constant.unwrap_or(1.0)];
    let wave = |wave: Option<&sjk_shader::WaveForm>| {
        wave.map_or([0.0; 4], |wave| {
            [wave.base, wave.amplitude, wave.phase, wave.frequency]
        })
    };
    result.rgb_wave = wave(stage.rgb_wave.as_ref());
    result.alpha_wave = wave(stage.alpha_wave.as_ref());
    let from_wave = |generator: Option<&str>, wave: Option<&sjk_shader::WaveForm>| {
        f32::from(wave.is_some() && generator.is_some_and(|g| g.eq_ignore_ascii_case("wave")))
    };
    result.generators = [
        from_wave(stage.rgb_generator.as_deref(), stage.rgb_wave.as_ref()),
        from_wave(stage.alpha_generator.as_deref(), stage.alpha_wave.as_ref()),
        stage
            .rgb_wave
            .as_ref()
            .map_or(0.0, |wave| world_stage::wave_code(&wave.function)),
        stage
            .alpha_wave
            .as_ref()
            .map_or(0.0, |wave| world_stage::wave_code(&wave.function)),
    ];
    result.test = [
        world_stage::alpha_function_code(stage.alpha_function.as_deref()),
        f32::from(stage.clamp),
        0.0,
        0.0,
    ];
    result
}

/// The sky program's code for a blend factor (`sky_stage.wgsl` `blend_factor`).
fn factor_code(factor: wgpu::BlendFactor) -> f32 {
    use wgpu::BlendFactor as F;
    match factor {
        F::Zero => 0.0,
        F::One => 1.0,
        F::Src => 2.0,
        F::OneMinusSrc => 3.0,
        F::SrcAlpha => 4.0,
        F::OneMinusSrcAlpha => 5.0,
        F::Dst => 6.0,
        F::OneMinusDst => 7.0,
        F::DstAlpha => 8.0,
        F::OneMinusDstAlpha => 9.0,
        F::SrcAlphaSaturated => 10.0,
        _ => 1.0,
    }
}

/// Whether a sky shader has clouds to draw.
pub(super) fn any(clouds: &GpuClouds) -> bool {
    clouds.header[0] > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stage(text: &str) -> GpuCloudStage {
        let script = format!(
            "textures/test/sky\n{{\n skyparms - 2048 -\n {{\n map textures/a\n {text}\n }}\n}}\n"
        );
        let definitions =
            sjk_shader::parse_shader_script(script.as_bytes(), "scripts/test.shader").unwrap();
        compile_stage(&definitions[0].stages[0], 0)
    }

    /// `baronshed_share`'s `beach_sky` scrolls its clouds: the sky up from the beach,
    /// four ways, then again four seconds later, on one sheet (`target/world-shots`).
    #[test]
    #[ignore = "renders with the GPU and baronshed_share.pk3 from JKA_GAME_DATA"]
    fn baronshed_clouds_move() {
        use crate::world_shot::*;
        on_big_stack(|| {
            let map = "maps/baronshed_share.bsp";
            let game = std::path::PathBuf::from(std::env::var_os("JKA_GAME_DATA").unwrap());
            let (bsp, _) = crate::assets::load_bsp(&game, map).expect("the map");
            let origin = under_sky(&bsp, "beach_sky").expect("open ground under beach_sky");
            println!("camera at {origin:?}");
            let Some((mut gpu, _profile)) = open(map, [960, 540], None, &[]) else {
                return;
            };
            let take = |gpu: &mut crate::GpuState| -> Vec<image::RgbaImage> {
                [0.0, 90.0, 180.0, 270.0]
                    .iter()
                    .map(|yaw| {
                        aim(gpu, origin, *yaw, 50.0);
                        frame(gpu, 30)
                    })
                    .collect()
            };
            let before = take(&mut gpu);
            std::thread::sleep(std::time::Duration::from_secs(4));
            let after = take(&mut gpu);
            let changed: usize = before
                .iter()
                .zip(&after)
                .map(|(a, b)| {
                    a.pixels()
                        .zip(b.pixels())
                        .filter(|(p, q)| (0..3).any(|c| p.0[c].abs_diff(q.0[c]) > 6))
                        .count()
                })
                .sum();
            let all: Vec<_> = before.into_iter().chain(after).collect();
            let path = sheet(&all, 4, 480, "baronshed-clouds");
            println!("{} ; {changed} pixels changed in 4 s", path.display());
        });
    }

    #[test]
    fn a_scrolling_cloud_stage_keeps_its_tcmods_in_order() {
        let packed = stage("tcMod scroll 0.0025 -0.0075\n tcMod scale 2 2");
        assert_eq!(packed.control[1], 2.0);
        assert_eq!(packed.tcmod_a[0], [1.0, 0.0025, -0.0075, 0.0]);
        assert_eq!(packed.tcmod_a[1], [2.0, 2.0, 2.0, 0.0]);
        // No blendFunc: ONE, ZERO, the layer covers the box.
        assert_eq!((packed.control[2], packed.control[3]), (1.0, 0.0));
    }

    /// Open ground under a sky shader's ceiling: below the middle of one of its faces,
    /// standing height over the floor, with that sky straight up and far away.
    fn under_sky(bsp: &sjk_bsp::Bsp, sky: &str) -> Option<[f32; 3]> {
        use crate::weather::cover::SURF_SKY;
        let point = sjk_bsp::Aabb::POINT;
        let render = bsp.render();
        render.surfaces().iter().find_map(|surface| {
            let name = bsp.shaders()[surface.shader].name_lossy();
            if !name.to_ascii_lowercase().contains(sky) || surface.vertices.is_empty() {
                return None;
            }
            let corners = &render.vertices()[surface.vertices.clone()];
            let centre = corners.iter().fold(glam::Vec3::ZERO, |sum, corner| {
                sum + glam::Vec3::from_array(corner.position)
            }) / corners.len() as f32;
            let start = [centre[0], centre[1], centre[2] - 24.0];
            let down = bsp.trace_box(start, [start[0], start[1], start[2] - 8192.0], point, 1);
            if down.start_solid || down.fraction >= 1.0 {
                return None;
            }
            let eye = down.end_position;
            let eye = [eye[0], eye[1], eye[2] + 64.0];
            let up = bsp.trace_box(eye, [eye[0], eye[1], eye[2] + 16384.0], point, 1);
            let open = up.surface_flags & SURF_SKY != 0 && up.end_position[2] - eye[2] > 512.0;
            open.then_some(eye)
        })
    }

    #[test]
    fn a_mask_stage_blends_with_its_factors() {
        let packed = stage("blendFunc GL_ONE_MINUS_SRC_ALPHA GL_SRC_ALPHA");
        assert_eq!((packed.control[2], packed.control[3]), (5.0, 4.0));
    }
}
