//! An offscreen pixel regression using only engine-generated images.
use wgpu::util::DeviceExt;

#[test]
#[ignore = "requires an available GPU; uses no installed game assets"]
fn black_blade_alpha_pass_darkens_a_white_target() {
    let gpu = pollster::block_on(crate::gpu_context::Context::new(None, None)).unwrap();
    eprintln!(
        "black-blade pixel check: {} / {}",
        gpu.adapter_name, gpu.adapter_backend
    );
    let device = &gpu.device;
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("black-blade pixel test"),
        source: wgpu::ShaderSource::Wgsl(include_str!("saber.wgsl").into()),
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("black-blade alpha pass"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vertex_main"),
            compilation_options: Default::default(),
            buffers: &[Some(crate::saber::Instance::layout())],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fragment_black"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8Unorm,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
    });
    let matrix = glam::camera::rh::proj::directx::orthographic(-32., 32., -32., 32., 0.1, 200.)
        * glam::camera::rh::view::look_at_mat4(
            glam::Vec3::new(0., 0., 100.),
            glam::Vec3::ZERO,
            glam::Vec3::Y,
        );
    let mut camera = matrix.to_cols_array().to_vec();
    camera.extend_from_slice(&[0., 0., 100., 0.]);
    let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&camera),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let camera_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: camera_buffer.as_entire_binding(),
        }],
    });
    let skins_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: &vec![
            0;
            std::mem::size_of::<crate::saber_skins::SkinUniform>()
                * crate::saber_skins::MAX_SKINS
        ],
        usage: wgpu::BufferUsages::STORAGE,
    });
    let skins_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(2),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: skins_buffer.as_entire_binding(),
        }],
    });
    let vfs = sjk_vfs::VirtualFileSystem::new();
    let shaders = sjk_shader::ShaderCatalog::default();
    let glow = crate::gpu_texture::upload_display_image(
        device,
        &gpu.queue,
        "black test glow",
        &super::image(&vfs, &shaders, "glow"),
    );
    let core = crate::gpu_texture::upload_display_image(
        device,
        &gpu.queue,
        "black test core",
        &super::image(&vfs, &shaders, "core"),
    );
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let materials = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(1),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&glow),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&core),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    let pair = crate::saber::Instance::pair(
        crate::saber::Blade {
            // Align the thin core with a pixel centre rather than between pixels.
            base: [0.5, -20., 0.],
            direction: [0., 1., 0.],
            length: 40.,
            radius: 3.,
        },
        crate::saber_rgb::BladeColor::Black,
    );
    let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&pair),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &camera_bind, &[]);
        pass.set_bind_group(1, &materials, &[]);
        pass.set_bind_group(2, &skins_bind, &[]);
        pass.set_vertex_buffer(0, vertices.slice(..));
        pass.draw(0..6, 0..2);
    }
    gpu.queue.submit([encoder.finish()]);
    let pixels = crate::world_shot::read_back(device, &gpu.queue, &texture);
    eprintln!("black-blade centre pixel: {:?}", pixels.get_pixel(32, 32).0);
    assert!(
        pixels.get_pixel(32, 32)[0] < 100,
        "centre should be dark, not additive white"
    );
    assert_eq!(pixels.get_pixel(0, 0).0, [255; 4]);
}
