//! Reusable render targets and screen-space portal compositing.
use super::*;
use wgpu::util::DeviceExt;

/// The floor mirrors' target's labels (colour, depth).
pub(super) const FLOOR_MIRROR: [&str; 2] = ["SJK floor mirror view", "SJK floor mirror depth"];
/// The sky portal's target's labels.
pub(super) const SKY_PORTAL: [&str; 2] = ["SJK sky portal view", "SJK sky portal depth"];
/// The map portal's target's labels.
pub(super) const MAP_PORTAL: [&str; 2] = ["SJK map portal view", "SJK map portal depth"];

/// A secondary view's target, made the first frame the view is drawn rather than with the
/// world: a scene-format colour target and a depth target at the scene's size, 25 MiB at
/// 1920x1080 in HDR and four times that at 4K, which most maps never draw (a sky shader
/// without a `misc_skyportal`, polished floors with `r_floorReflections 0`, portal
/// shaders without a `misc_portal_surface`). Once made it is kept, and made again at a
/// new size, until the world goes or its owner drops it.
pub(super) struct Lazy {
    labels: [&'static str; 2],
    target: Option<Target>,
    /// Making it failed (out of video memory): the view is not drawn and making it is
    /// not tried again until the scene's size changes.
    refused: bool,
}

impl Lazy {
    pub fn new(labels: [&'static str; 2]) -> Self {
        Self {
            labels,
            target: None,
            refused: false,
        }
    }

    pub fn get(&self) -> Option<&Target> {
        self.target.as_ref()
    }

    pub fn is_some(&self) -> bool {
        self.target.is_some()
    }

    /// The target, made at `size` now if there is none yet; `None` when the card has no
    /// room for it, which is logged once and leaves the view undrawn rather than ending
    /// the game.
    pub fn make(
        &mut self,
        device: &wgpu::Device,
        camera_layout: &wgpu::BindGroupLayout,
        samples: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
        size: [u32; 2],
    ) -> Option<&Target> {
        if self.target.is_none() && !self.refused {
            let labels = self.labels;
            match crate::gpu_guard::capture(device, || {
                Target::new(device, camera_layout, samples, format, size, labels)
            }) {
                Ok(target) => self.target = Some(target),
                Err(error) => {
                    self.refused = true;
                    crate::log::progress(format_args!(
                        "warning: no {} at {}x{} ({error}); that view is not drawn",
                        labels[0], size[0], size[1]
                    ));
                }
            }
        }
        self.target.as_ref()
    }

    /// The target made again at `size` if there is one, for a resize; one refused may
    /// be tried again at the new size.
    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        camera_layout: &wgpu::BindGroupLayout,
        samples: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
        size: [u32; 2],
    ) {
        let rebuilt = self.rebuilt(device, camera_layout, samples, format, size);
        self.set(rebuilt);
    }

    /// A new target at `size` if there is one now, for a resize that installs it with
    /// [`Self::set`] once every attachment fits (`install_render_scale_for`).
    pub fn rebuilt(
        &self,
        device: &wgpu::Device,
        camera_layout: &wgpu::BindGroupLayout,
        samples: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
        size: [u32; 2],
    ) -> Option<Target> {
        self.target
            .is_some()
            .then(|| Target::new(device, camera_layout, samples, format, size, self.labels))
    }

    /// Replace the target (`None` drops it); a refused one may be tried again.
    pub fn set(&mut self, target: Option<Target>) {
        self.target = target;
        self.refused = false;
    }
}

pub(super) struct Target {
    pub size: [u32; 2],
    pub color: wgpu::TextureView,
    pub depth: crate::DepthTarget,
    pub camera: wgpu::Buffer,
    pub camera_bind: wgpu::BindGroup,
    pub instances: wgpu::Buffer,
    pub normal_sample: wgpu::BindGroup,
    pub mirror_sample: wgpu::BindGroup,
}

impl Target {
    pub fn new(
        device: &wgpu::Device,
        camera_layout: &wgpu::BindGroupLayout,
        samples: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
        size: [u32; 2],
        labels: [&str; 2],
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(labels[0]),
            size: wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let color = texture.create_view(&Default::default());
        let camera = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("SJK portal camera"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera.as_entire_binding(),
            }],
        });
        let instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("SJK portal surface transform"),
            size: std::mem::size_of::<ActorInstance>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let sample = |mirror: f32| {
            let control = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[mirror, 0., 0., 0.]),
                usage: wgpu::BufferUsages::UNIFORM,
            });
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("SJK portal composite sample"),
                layout: samples,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&color),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: control.as_entire_binding(),
                    },
                ],
            })
        };
        let normal_sample = sample(0.);
        let mirror_sample = sample(1.);
        Self {
            size,
            color,
            depth: crate::DepthTarget::labelled(device, size[0], size[1], labels[1]),
            camera,
            camera_bind,
            instances,
            normal_sample,
            mirror_sample,
        }
    }
}

pub(super) fn sample_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

/// The composite of a portal's view (`portal_main`) or of the sky portal's on the sky
/// faces (`sky_portal_main`, behind them as the sky stage's faces are).
pub(super) fn pipeline(
    device: &wgpu::Device,
    camera: &wgpu::BindGroupLayout,
    samples: &wgpu::BindGroupLayout,
    format: wgpu::TextureFormat,
    vertex_entry: &str,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("SJK portal composite"),
        source: wgpu::ShaderSource::Wgsl(
            concat!(
                include_str!("vertex_transform.wgsl"),
                include_str!("scene_view.wgsl")
            )
            .into(),
        ),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[Some(camera), Some(samples)],
        immediate_size: 0,
    });
    let buffers = [Some(GpuVertex::layout()), Some(ActorInstance::layout())];
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("SJK map view composite"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some(vertex_entry),
            compilation_options: Default::default(),
            buffers: &buffers,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("sample_view"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: crate::DepthTarget::FORMAT,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::LessEqual),
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    })
}
