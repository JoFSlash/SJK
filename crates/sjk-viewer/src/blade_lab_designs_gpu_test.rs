//! An offscreen render of every Blade Lab design: its glow and its solid pass, through
//! the real skin table and `saber.wgsl`, on a dark ground. Each must light the blade and
//! leave the far corner dark (nothing growing out to the quad's edge). With
//! `SJK_LAB_PREVIEW` set to a folder, each picture is saved there as `<id>.png`.
use super::*;
use wgpu::util::DeviceExt;

const SIZE: u32 = 192;

#[test]
#[ignore = "requires an available GPU; uses no installed game assets"]
fn every_design_draws_its_blade_and_nothing_at_the_edge() {
    let gpu = pollster::block_on(crate::gpu_context::Context::new(None, None)).unwrap();
    let device = &gpu.device;
    let queue = &gpu.queue;
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("lab designs"),
        source: wgpu::ShaderSource::Wgsl(include_str!("saber.wgsl").into()),
    });
    let visible = wgpu::ShaderStages::VERTEX_FRAGMENT;
    let texture_entry = |binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: visible,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    let sampler_entry = |binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: visible,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    };
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
    let material_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            texture_entry(0),
            texture_entry(1),
            sampler_entry(2),
            texture_entry(3),
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
    let pipeline = |vertex: &str, fragment: &str, blend: wgpu::BlendState| {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some(vertex),
                compilation_options: Default::default(),
                buffers: &[Some(crate::saber::Instance::layout())],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some(fragment),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(blend),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        })
    };
    let additive = wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent::OVER,
    };
    let glow = pipeline("vertex_main", "fragment_main", additive);
    let solid = pipeline(
        "vertex_solid",
        "fragment_solid",
        wgpu::BlendState::ALPHA_BLENDING,
    );
    // The blade seen side on, its 40 units filling most of the picture's height.
    let matrix = glam::camera::rh::proj::directx::orthographic(-24., 24., -26., 26., 0.1, 200.)
        * glam::camera::rh::view::look_at_mat4(
            glam::Vec3::new(0., -100., 0.),
            glam::Vec3::ZERO,
            glam::Vec3::Z,
        );
    let mut camera = matrix.to_cols_array().to_vec();
    camera.extend_from_slice(&[0., -100., 0., 0.]);
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
    let white = device.create_texture_with_data(
        queue.raw(),
        &wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        &[255, 255, 255, 255],
    );
    let white = white.create_view(&Default::default());
    let sampler = device.create_sampler(&Default::default());
    let materials = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &material_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&white),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&white),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&white),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    let save = std::env::var_os("SJK_LAB_PREVIEW").map(std::path::PathBuf::from);
    // Every design's skin, loaded from the test sample under its id.
    let skins = crate::saber_skins::LoadedSkins::of(
        DESIGNS
            .iter()
            .map(|design| {
                let def =
                    crate::blade_skin_file::parse(design.id, crate::blade_skin_file::tests::SAMPLE)
                        .unwrap();
                crate::saber_skins::LoadedSkin::new(
                    design.id,
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
    for design in &DESIGNS {
        let color = skins.color_of(design.id).unwrap();
        // A chroma worn with a green saber.
        let green = color.worn_with(crate::saber_rgb::BladeColor::from_rgb([40, 255, 60]));
        // Four moments: still, mid-swing (the monarchs gone, a comic word), after a
        // clash (the reactor critical), and still again in green.
        let mut frames = Vec::new();
        for (seconds, motion, color) in [
            (3.0, crate::saber_motion::STILL, color),
            (5.4, [0.9, 1_000.0, 1_000.0, 0.25], color),
            (24.0, [0.0, 0.1, 1_000.0, 1_000.0], color),
            (3.0, crate::saber_motion::STILL, green),
        ] {
            let pair = crate::saber::Instance::pair(
                crate::saber::Blade {
                    base: [0.0, 0.0, -20.0],
                    direction: [0.0, 0.0, 1.0],
                    length: 40.0,
                    radius: 3.0,
                },
                crate::saber_rgb::BladeColor::Skin(color),
            )
            .map(|i| i.with_animation(seconds, 7).with_motion(motion));
            let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&pair),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let target = device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: SIZE,
                    height: SIZE,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let view = target.create_view(&Default::default());
            let mut encoder = device.create_command_encoder(&Default::default());
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.02,
                                g: 0.025,
                                b: 0.035,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                pass.set_bind_group(0, &camera_bind, &[]);
                pass.set_bind_group(1, &materials, &[]);
                pass.set_bind_group(2, &skins_bind, &[]);
                pass.set_vertex_buffer(0, vertices.slice(..));
                pass.set_pipeline(&glow);
                pass.draw(0..6, 0..2);
                pass.set_pipeline(&solid);
                pass.draw(0..6, 0..1);
            }
            queue.submit([encoder.finish()]);
            frames.push(crate::world_shot::read_back(device, queue, &target));
        }
        for frame in &frames {
            // Lit somewhere across the blade's middle (the obsidian blade is black glass
            // in a white edge).
            let brightest = (SIZE / 2 - 16..SIZE / 2 + 16)
                .map(|x| {
                    frame.get_pixel(x, SIZE / 2).0[..3]
                        .iter()
                        .map(|&c| u32::from(c))
                        .sum::<u32>()
                })
                .max()
                .unwrap_or(0);
            assert!(brightest > 120, "{}: the blade is not drawn", design.id);
            let corner = frame.get_pixel(2, 2).0;
            assert!(
                corner[..3].iter().all(|&c| c < 24),
                "{}: light at the picture's corner ({corner:?})",
                design.id
            );
        }
        if let Some(folder) = &save {
            let mut sheet = image::RgbaImage::new(SIZE * frames.len() as u32, SIZE);
            for (index, frame) in frames.iter().enumerate() {
                image::imageops::overlay(&mut sheet, frame, i64::from(SIZE) * index as i64, 0);
            }
            sheet
                .save(folder.join(format!("{}.png", design.id)))
                .unwrap();
        }
    }
}
