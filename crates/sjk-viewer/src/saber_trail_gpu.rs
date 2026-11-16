//! WGPU adapter for fixed-capacity saber trail slices.

use crate::saber_trail::{MAX_SEGMENTS, SegmentPool, Vertex};
use crate::{DepthTarget, load_shader_texture, texture_layout_entry};
use sjk_shader::{ShaderCatalog, ShaderDefinition, StageBlend};
use sjk_vfs::VirtualFileSystem;
use std::error::Error;

pub(crate) struct Runtime {
    pipeline: wgpu::RenderPipeline,
    black_pipeline: wgpu::RenderPipeline,
    any_black: bool,
    /// The heat wake (`saber_trail.wgsl` `fragment_wake`): drawn with the blades' heat
    /// haze, first in the effect pass, bending the scene behind the slices marked
    /// [`crate::saber_trail::WAKE`].
    wake_pipeline: wgpu::RenderPipeline,
    any_wake: bool,
    material: wgpu::BindGroup,
    buffer: wgpu::Buffer,
    vertices: Vec<Vertex>,
    vertex_count: u32,
    last_quads: usize,
}

impl Runtime {
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &crate::frame_queue::FrameQueue,
        vfs: &VirtualFileSystem,
        shaders: &ShaderCatalog,
        camera_layout: &wgpu::BindGroupLayout,
        scene_layout: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
    ) -> Result<Self, Box<dyn Error>> {
        let blur = additive_shader(shaders, "gfx/effects/sabers/saberBlur")?;
        let sword = additive_shader(shaders, "gfx/effects/sabers/swordTrail")?;
        let glow_image = stage_image(blur, 0)?;
        let core_image = stage_image(blur, 1)?;
        let sword_image = stage_image(sword, 0)?;
        let blend = shader_blend(blur)?;
        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("SJK saber trail texture layout"),
            entries: &[
                texture_layout_entry(0),
                texture_layout_entry(1),
                texture_layout_entry(2),
                texture_layout_entry(4),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("SJK saber trail sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let glow = load_shader_texture(device, queue, vfs, shaders, glow_image)?;
        let core = load_shader_texture(device, queue, vfs, shaders, core_image)?;
        let sword = load_shader_texture(device, queue, vfs, shaders, sword_image)?;
        let black_image = crate::saber_rgb::black::image(vfs, shaders, "trail");
        let black = crate::gpu_texture::upload_display_image(
            device,
            queue,
            "SJK black saber trail",
            &black_image,
        );
        let material = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("SJK stock saber trail material"),
            layout: &texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&black),
                },
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
                    resource: wgpu::BindingResource::TextureView(&sword),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("SJK saber trail pipeline layout"),
            bind_group_layouts: &[Some(camera_layout), Some(&texture_layout)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("SJK saber trail shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("saber_trail.wgsl").into()),
        });
        let wake_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("SJK saber trail wake pipeline layout"),
            bind_group_layouts: &[
                Some(camera_layout),
                Some(&texture_layout),
                Some(scene_layout),
            ],
            immediate_size: 0,
        });
        let create = |entry, blend: Option<wgpu::BlendState>, layout| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("SJK additive saber trail pipeline"),
                layout: Some(layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vertex_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[Some(Vertex::layout())],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(entry),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    cull_mode: None,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DepthTarget::FORMAT,
                    depth_write_enabled: Some(false),
                    depth_compare: Some(wgpu::CompareFunction::LessEqual),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            })
        };
        let pipeline = create("fragment_main", Some(blend), &layout);
        let black_pipeline = create(
            "fragment_black",
            Some(wgpu::BlendState::ALPHA_BLENDING),
            &layout,
        );
        // The wake replaces what it covers with the scene's pixels, bent (the layer still
        // holds the scene there: nothing else has drawn yet).
        let wake_pipeline = create("fragment_wake", None, &wake_layout);
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("SJK saber trail vertices"),
            size: (MAX_SEGMENTS * 6 * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Ok(Self {
            pipeline,
            black_pipeline,
            any_black: false,
            wake_pipeline,
            any_wake: false,
            material,
            buffer,
            vertices: Vec::with_capacity(MAX_SEGMENTS * 6),
            vertex_count: 0,
            last_quads: 0,
        })
    }

    pub(crate) fn prepare(
        &mut self,
        queue: &crate::frame_queue::FrameQueue,
        pool: &mut SegmentPool,
        now: i64,
    ) {
        self.vertices.clear();
        self.last_quads = pool.append_vertices(now, &mut self.vertices);
        self.any_black = self.vertices.iter().any(|vertex| vertex.style == 2);
        self.any_wake = self
            .vertices
            .iter()
            .any(|vertex| vertex.style & crate::saber_trail::WAKE != 0);
        self.vertex_count = u32::try_from(self.vertices.len()).unwrap_or(u32::MAX);
        if !self.vertices.is_empty() {
            queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.vertices));
        }
    }

    pub(crate) fn draw<'pass>(
        &'pass self,
        pass: &mut wgpu::RenderPass<'pass>,
        camera: &'pass wgpu::BindGroup,
        black_trails: bool,
    ) {
        if self.vertex_count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, camera, &[]);
        pass.set_bind_group(1, &self.material, &[]);
        pass.set_vertex_buffer(0, self.buffer.slice(..));
        pass.draw(0..self.vertex_count, 0..1);
        if black_trails && self.any_black {
            pass.set_pipeline(&self.black_pipeline);
            pass.draw(0..self.vertex_count, 0..1);
        }
    }

    /// Whether the last prepared frame has a slice that bends what is behind it.
    pub(crate) const fn has_wake(&self) -> bool {
        self.any_wake
    }

    /// Draw the heat wakes into the effect layer, the scene image bound in `scene`
    /// (`saber_gpu.rs` `draw_haze`, whose layout the pipeline was made with).
    pub(crate) fn draw_wake(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        camera: &wgpu::BindGroup,
        scene: &wgpu::BindGroup,
    ) {
        if !self.any_wake || self.vertex_count == 0 {
            return;
        }
        pass.set_pipeline(&self.wake_pipeline);
        pass.set_bind_group(0, camera, &[]);
        pass.set_bind_group(1, &self.material, &[]);
        pass.set_bind_group(2, scene, &[]);
        pass.set_vertex_buffer(0, self.buffer.slice(..));
        pass.draw(0..self.vertex_count, 0..1);
    }

    /// World positions of the last prepared frame's trail vertices.
    pub(crate) fn positions(&self) -> impl Iterator<Item = [f32; 3]> + '_ {
        self.vertices.iter().map(|vertex| vertex.position)
    }

    /// Whether the last prepared frame has any trail vertex.
    pub(crate) const fn has_draws(&self) -> bool {
        self.vertex_count != 0
    }
}

fn additive_shader<'a>(
    shaders: &'a ShaderCatalog,
    name: &str,
) -> Result<&'a ShaderDefinition, Box<dyn Error>> {
    let definition = shaders
        .get(name)
        .ok_or_else(|| format!("missing stock trail shader {name}"))?;
    if definition.stages.is_empty()
        || definition
            .stages
            .iter()
            .any(|stage| stage.blend != StageBlend::Add)
    {
        return Err(format!("{name} is no longer an all-additive shader").into());
    }
    Ok(definition)
}

fn stage_image(definition: &ShaderDefinition, index: usize) -> Result<&str, Box<dyn Error>> {
    definition
        .stages
        .get(index)
        .and_then(|stage| stage.images.first())
        .map(String::as_str)
        .ok_or_else(|| {
            format!(
                "stock trail shader {} has no stage {index} image",
                definition.name
            )
            .into()
        })
}

fn shader_blend(definition: &ShaderDefinition) -> Result<wgpu::BlendState, Box<dyn Error>> {
    match definition.stages.first().map(|stage| &stage.blend) {
        Some(StageBlend::Add) => Ok(wgpu::BlendState::ADDITIVE),
        blend => Err(format!("unsupported stock saber trail blend {blend:?}").into()),
    }
}
