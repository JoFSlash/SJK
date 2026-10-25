//! The GPU cost of every signature skin's own pipelines next to the stock blade's, measured
//! with timestamp queries offscreen at 1920x1080: 16 players with two blades each, seen
//! third-person at 100 to 560 units, and one blade close to the camera filling much of the
//! picture, drawn through the game's blade pipelines (`blade_pipeline`: depth-tested, the
//! effect layer's format) as `Runtime::draw` does (the main pass, then the solid one for a
//! signature with solid parts). The glow pass draws every blade with the shared pipeline, so
//! it is left out. Prints each signature's median milliseconds per frame, its ratio to stock
//! and its pipelines' build time. `SJK_SIGNATURES` (ids, comma-separated) measures only those.
//! `SJK_COST_SCENE` picks the scene: `game` (the default, as above), `close` (only the near
//! blade, across the view) or `along` (only a near blade seen along its length, its tip
//! pointing away, as Sol asked for the round 23 review: the views where an effect's ray
//! march is longest).
use super::*;
use wgpu::util::DeviceExt;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
/// Frames drawn inside one timed pass.
const REPEATS: u32 = 8;
/// Timed passes per blade; the median is kept.
const SAMPLES: u32 = 15;

#[test]
#[ignore = "requires an available GPU with timestamp queries; uses no installed game assets"]
fn signature_gpu_cost_against_stock() {
    let gpu = pollster::block_on(crate::gpu_context::Context::new(None, None)).unwrap();
    let device = &gpu.device;
    let queue = &gpu.queue;
    assert!(
        device.features().contains(wgpu::Features::TIMESTAMP_QUERY),
        "the adapter has no timestamp queries"
    );
    println!("GPU {} ({})", gpu.adapter_name, gpu.adapter_backend);
    let visible = wgpu::ShaderStages::VERTEX_FRAGMENT;
    let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: visible,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let sampler_entry = |binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    };
    let material_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            texture_layout_entry(0),
            texture_layout_entry(1),
            texture_layout_entry(3),
            sampler_entry(2),
            sampler_entry(4),
        ],
    });
    let skins_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: visible,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[
            Some(&camera_layout),
            Some(&material_layout),
            Some(&skins_layout),
        ],
        immediate_size: 0,
    });
    let format = crate::frame_target::aa::effects::FORMAT;
    let pipeline = |module: &wgpu::ShaderModule, vertex: &str, fragment: &str, blend| {
        blade_pipeline(
            device,
            &layout,
            format,
            "cost",
            module,
            vertex,
            fragment,
            Some(blend),
            None,
        )
    };
    // Third person: the eye a little above the players, cg_fov 80 at 4:3 (about 65
    // degrees vertically).
    let eye = glam::Vec3::new(0.0, -80.0, 70.0);
    let matrix = glam::camera::rh::proj::directx::perspective(
        1.13,
        WIDTH as f32 / HEIGHT as f32,
        4.0,
        8192.0,
    ) * glam::camera::rh::view::look_at_mat4(
        eye,
        glam::Vec3::new(0.0, 400.0, 40.0),
        glam::Vec3::Z,
    );
    let mut camera = matrix.to_cols_array().to_vec();
    camera.extend_from_slice(&[eye.x, eye.y, eye.z, 0.0]);
    let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&camera),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let camera_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &camera_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: camera_buffer.as_entire_binding(),
        }],
    });
    let samplers = saber::Samplers::new(device);
    let only: Option<Vec<String>> = std::env::var("SJK_SIGNATURES").ok().map(|ids| {
        ids.split(',')
            .map(|id| format!("saber_{}", id.trim()))
            .collect()
    });
    let skins = LoadedSkins::of(
        crate::blade_signatures::SIGNATURES
            .iter()
            .map(|signature| {
                let def =
                    crate::blade_skin_file::parse(signature.id, signature.default_file()).unwrap();
                crate::saber_skins::LoadedSkin::new(
                    signature.id,
                    def,
                    &sjk_vfs::VirtualFileSystem::new(),
                )
                .unwrap()
            })
            .collect(),
        1,
    );
    let skins_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&skins.uniforms()),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let skins_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &skins_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: skins_buffer.as_entire_binding(),
        }],
    });

    // The scene's blades: 16 players two blades each, then the close one.
    let scene = std::env::var("SJK_COST_SCENE").unwrap_or_else(|_| "game".to_owned());
    println!("scene {scene}");
    let blades: Vec<(saber::Blade, [f32; 4])> = if scene == "along" {
        // The eye just behind and above the hilt, looking down the blade.
        let forward = (glam::Vec3::new(0.0, 400.0, 40.0) - eye).normalize();
        vec![(
            saber::Blade {
                base: (eye + forward * 18.0 - glam::Vec3::Z * 4.0).to_array(),
                direction: (forward + glam::Vec3::Z * 0.08).normalize().to_array(),
                length: 40.0,
                radius: 3.0,
            },
            crate::saber_motion::STILL,
        )]
    } else if scene == "close" {
        vec![(
            saber::Blade {
                base: [-14.0, -40.0, 50.0],
                direction: glam::Vec3::new(0.7, 0.15, 0.7).normalize().to_array(),
                length: 40.0,
                radius: 3.0,
            },
            crate::saber_motion::STILL,
        )]
    } else {
        let swing = [0.9, 1_000.0, 1_000.0, 0.25];
        let clash = [0.0, 0.1, 1_000.0, 1_000.0];
        let mut blades = Vec::new();
        for player in 0..16 {
            let distance = 100.0 + 30.0 * player as f32;
            let side = ((player * 7) % 11) as f32 / 10.0 - 0.5;
            let at = glam::Vec3::new(side * distance * 1.1, distance, 30.0);
            for hand in 0..2 {
                let lean = 0.3 + 0.5 * hand as f32 + 0.1 * player as f32;
                let direction =
                    glam::Vec3::new(lean.cos() * 0.6, -0.3, lean.sin().abs() + 0.4).normalize();
                let motion = [crate::saber_motion::STILL, swing, clash][(player + hand) % 3];
                blades.push((
                    saber::Blade {
                        base: (at + glam::Vec3::new(10.0 * hand as f32 - 5.0, 0.0, 0.0)).to_array(),
                        direction: direction.to_array(),
                        length: 40.0,
                        radius: 3.0,
                    },
                    motion,
                ));
            }
        }
        // The player's own blade across the near view.
        blades.push((
            saber::Blade {
                base: [-14.0, -40.0, 50.0],
                direction: glam::Vec3::new(0.7, 0.15, 0.7).normalize().to_array(),
                length: 40.0,
                radius: 3.0,
            },
            crate::saber_motion::STILL,
        ));
        blades
    };
    let instances = |color: crate::saber_rgb::BladeColor| -> Vec<Instance> {
        blades
            .iter()
            .enumerate()
            .flat_map(|(index, (blade, motion))| {
                let across = glam::Vec3::from_array(blade.direction)
                    .cross(glam::Vec3::Z)
                    .normalize_or(glam::Vec3::X);
                Instance::pair(*blade, color).map(|i| {
                    i.with_animation(5.4, index as u32)
                        .with_motion(*motion)
                        .with_frame(across.to_array(), 40.0)
                })
            })
            .collect()
    };

    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = target.create_view(&Default::default());
    let depth = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: target.size(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DepthTarget::FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let depth_view = depth.create_view(&Default::default());
    let queries = device.create_query_set(&wgpu::QuerySetDescriptor {
        label: None,
        ty: wgpu::QueryType::Timestamp,
        count: 2 * SAMPLES,
    });
    let resolve = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: u64::from(2 * SAMPLES) * 8,
        usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let read = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: resolve.size(),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let tick_ms = f64::from(queue.get_timestamp_period()) * 1e-6;

    // Median GPU milliseconds of one frame of the scene drawn with `passes`.
    let measure = |material: &wgpu::BindGroup,
                   vertices: &wgpu::Buffer,
                   count: u32,
                   passes: &[&wgpu::RenderPipeline]|
     -> f64 {
        let frame = |encoder: &mut wgpu::CommandEncoder, sample: Option<u32>| {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: sample.map(|sample| wgpu::RenderPassTimestampWrites {
                    query_set: &queries,
                    beginning_of_pass_write_index: Some(2 * sample),
                    end_of_pass_write_index: Some(2 * sample + 1),
                }),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_bind_group(0, &camera_bind, &[]);
            pass.set_bind_group(1, material, &[]);
            pass.set_bind_group(2, &skins_bind, &[]);
            pass.set_vertex_buffer(0, vertices.slice(..));
            for _ in 0..REPEATS {
                for pipeline in passes {
                    pass.set_pipeline(pipeline);
                    pass.draw(0..6, 0..count);
                }
            }
        };
        // Warm up, then time each sample in its own submission.
        let mut encoder = device.create_command_encoder(&Default::default());
        for _ in 0..3 {
            frame(&mut encoder, None);
        }
        queue.submit([encoder.finish()]);
        for sample in 0..SAMPLES {
            let mut encoder = device.create_command_encoder(&Default::default());
            frame(&mut encoder, Some(sample));
            queue.submit([encoder.finish()]);
        }
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.resolve_query_set(&queries, 0..2 * SAMPLES, &resolve, 0);
        encoder.copy_buffer_to_buffer(&resolve, 0, &read, 0, resolve.size());
        queue.submit([encoder.finish()]);
        read.slice(..)
            .map_async(wgpu::MapMode::Read, |r| r.unwrap());
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        let stamps: Vec<u64> =
            bytemuck::cast_slice(&read.slice(..).get_mapped_range().unwrap()).to_vec();
        read.unmap();
        let mut times: Vec<f64> = stamps
            .chunks(2)
            .map(|pair| pair[1].saturating_sub(pair[0]) as f64 * tick_ms / f64::from(REPEATS))
            .collect();
        times.sort_by(f64::total_cmp);
        times[times.len() / 2]
    };

    // Share of the picture the last measured frame lit, as a check that the blades are drawn.
    let coverage = || {
        let image = crate::world_shot::read_back(device, queue, &target);
        let lit = image
            .pixels()
            .filter(|p| p.0[..3].iter().any(|&c| c > 0))
            .count();
        100.0 * lit as f64 / f64::from(WIDTH * HEIGHT)
    };
    let stock_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("stock"),
        source: wgpu::ShaderSource::Wgsl(include_str!("saber.wgsl").into()),
    });
    let stock = pipeline(
        &stock_module,
        "vertex_main",
        "fragment_main",
        wgpu::BlendState::ADDITIVE,
    );
    let neutral = crate::saber_rgb::create_material(device, queue, &material_layout, &samplers);
    let upload = |instances: &[Instance]| {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(instances),
            usage: wgpu::BufferUsages::VERTEX,
        })
    };
    let stock_instances = instances(crate::saber_rgb::BladeColor::Rgb([40, 140, 255]));
    let count = stock_instances.len() as u32;
    let stock_vertices = upload(&stock_instances);
    // Twice: the first runs while the GPU may still be clocking up.
    measure(&neutral, &stock_vertices, count, &[&stock]);
    let stock_ms = measure(&neutral, &stock_vertices, count, &[&stock]);
    println!(
        "stock blade (RGB, neutral pair): {stock_ms:.4} ms per frame of {} blades ({count} instances), {:.1}% of the picture lit",
        blades.len(),
        coverage()
    );

    let mut rows = Vec::new();
    let mut stand_in = None;
    for (_, skin) in skins.iter() {
        let signature = skin.signature.unwrap();
        if only
            .as_ref()
            .is_some_and(|only| !only.iter().any(|id| id == signature.id))
        {
            continue;
        }
        let started = std::time::Instant::now();
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(signature.id),
            source: wgpu::ShaderSource::Wgsl(signature.module_source().into()),
        });
        let main = pipeline(
            &module,
            "vertex_main",
            "fragment_main",
            wgpu::BlendState::ADDITIVE,
        );
        let solid = signature.solid().then(|| {
            pipeline(
                &module,
                "vertex_solid",
                "fragment_solid",
                wgpu::BlendState::ALPHA_BLENDING,
            )
        });
        let built_ms = started.elapsed().as_secs_f64() * 1e3;
        let material = saber::material(
            device,
            queue,
            &material_layout,
            &samplers,
            &skin.glow,
            &skin.core,
        );
        let color = skins.color_of(signature.id).unwrap();
        let vertices = upload(&instances(crate::saber_rgb::BladeColor::Skin(color)));
        let mut passes = vec![&main];
        passes.extend(solid.as_ref());
        let ms = measure(&material, &vertices, count, &passes);
        let lit = coverage();
        if stand_in.is_none() {
            // A skin with no signature of its own: the stock module's skin path.
            let ms = measure(&material, &vertices, count, &[&stock]);
            stand_in = Some((ms, coverage()));
        }
        rows.push((signature.id, ms, built_ms, solid.is_some(), lit));
    }
    if let Some((ms, lit)) = stand_in {
        println!(
            "skin through the stock module (stand-in): {ms:.4} ms ({:.2}x stock), {lit:.1}% lit",
            ms / stock_ms
        );
    }
    rows.sort_by(|a, b| b.1.total_cmp(&a.1));
    println!("rank signature                      ms/frame  x stock  build ms  lit %  solid");
    for (rank, (id, ms, built, solid, lit)) in rows.iter().enumerate() {
        println!(
            "{:>4} {:<30} {:>8.4} {:>8.2} {:>9.1} {:>6.1}  {}",
            rank + 1,
            id,
            ms,
            ms / stock_ms,
            built,
            lit,
            if *solid { "yes" } else { "" }
        );
    }
    if !rows.is_empty() {
        let mut ms: Vec<f64> = rows.iter().map(|row| row.1).collect();
        ms.sort_by(f64::total_cmp);
        let mean = ms.iter().sum::<f64>() / ms.len() as f64;
        let builds: Vec<f64> = rows.iter().map(|row| row.2).collect();
        println!(
            "signatures {}: median {:.4} ms ({:.2}x), mean {mean:.4} ms, p90 {:.4} ms, max {:.4} ms; build mean {:.1} ms, max {:.1} ms",
            ms.len(),
            ms[ms.len() / 2],
            ms[ms.len() / 2] / stock_ms,
            ms[ms.len() * 9 / 10],
            ms[ms.len() - 1],
            builds.iter().sum::<f64>() / builds.len() as f64,
            builds.iter().cloned().fold(0.0, f64::max),
        );
    }
}
