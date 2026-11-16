//! Drawing the menu pictures ([`super`]): each picture is decoded on a worker the
//! first time it is wanted, scaled to what the screen shows of it, and uploaded; two
//! are kept, the one on show and the one before it, for the crossfade. The crossfade
//! starts once the new picture is uploaded, so the last one stays on show while it
//! decodes. The pipeline is built the first time a picture is drawn, so a live menu
//! costs nothing.
//!
//! The pictures are taken at 16:9. The camera keeps their vertical field of view on a
//! screen as wide or narrower (its sides crop away) and narrows it on a wider screen
//! ([`fit`]), so the screen never sees past the picture and the 3D stage, drawn with
//! that field of view from the picture's camera, lands where the picture shows it.

use super::Set;
use crate::menu_backdrop::pictures::View;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, Instant};

/// How long one picture takes to give way to the next.
const FADE: Duration = Duration::from_millis(900);

/// What [`Painter::prepare`] needs of the frame.
pub(super) struct Frame<'a> {
    pub(super) device: &'a wgpu::Device,
    pub(super) queue: &'a crate::frame_queue::FrameQueue,
    pub(super) scene_format: wgpu::TextureFormat,
    /// The scene exposure the final resolve reads; `None` without a resolve.
    pub(super) exposure: Option<&'a wgpu::Buffer>,
    /// The picture set of the menu world on show, `None` on a live map.
    pub(super) set: Option<&'static Set>,
    /// Index of the picture the backdrop wants, in the set's order.
    pub(super) shown: Option<usize>,
    pub(super) screen: [u32; 2],
}

/// The vertical field of view (degrees) for a screen of `aspect` showing pictures
/// taken at `picture_fov` and `picture_aspect`, and the share of the picture it then
/// shows each way.
pub(crate) fn fit(picture_fov: f32, picture_aspect: f32, aspect: f32) -> (f32, [f32; 2]) {
    let tan = (picture_fov.to_radians() * 0.5).tan();
    let screen_tan = if aspect > picture_aspect {
        tan * picture_aspect / aspect
    } else {
        tan
    };
    let fov = (2.0 * screen_tan.atan()).to_degrees();
    (
        fov,
        [
            aspect * screen_tan / (picture_aspect * tan),
            screen_tan / tan,
        ],
    )
}

/// One uploaded picture.
struct Slot {
    view: View,
    texture: wgpu::TextureView,
}

/// A picture being decoded.
struct Decoding {
    view: View,
    receiver: Receiver<Option<image::RgbaImage>>,
}

/// The pipeline and what it binds, built on first use.
struct Gpu {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    controls: wgpu::Buffer,
    /// Bound when the frame has no resolve to read the exposure from.
    neutral_exposure: wgpu::Buffer,
    format: wgpu::TextureFormat,
}

/// The bind group of this frame's pair of pictures, kept while the pair and the
/// exposure buffer stay the same.
struct Bound {
    current: View,
    previous: View,
    exposure: wgpu::Buffer,
    group: wgpu::BindGroup,
}

/// The menu pictures' GPU side, one per world.
#[derive(Default)]
pub(crate) struct Painter {
    gpu: Option<Gpu>,
    slots: Vec<Slot>,
    decoding: Option<Decoding>,
    bound: Option<Bound>,
    /// The field of view this frame's camera takes, while a picture shows.
    fov: Option<f32>,
    /// Whether a picture draws this frame.
    draws: bool,
    /// A picture failed to decode or upload: the world clear shows instead.
    failed: bool,
    /// The picture on show (coming in, while `fading_from` lasts).
    on_show: Option<View>,
    /// The picture going out and when it started to.
    fading_from: Option<(View, Instant)>,
}

impl Painter {
    /// The field of view that lines the 3D stage up with the picture on show.
    pub(crate) fn field_of_view(&self) -> Option<f32> {
        self.fov
    }

    pub(super) fn prepare(&mut self, frame: &Frame<'_>) {
        self.draws = false;
        self.fov = None;
        let (Some(set), Some(shown)) = (frame.set, frame.shown) else {
            return;
        };
        let Ok(manifest) = set.manifest() else {
            return;
        };
        let picture_aspect = manifest.size[0] as f32 / manifest.size[1].max(1) as f32;
        let aspect = frame.screen[0] as f32 / frame.screen[1].max(1) as f32;
        let (fov, scale) = fit(manifest.fov, picture_aspect, aspect);
        self.fov = Some(fov);
        if self.failed {
            return;
        }
        let view_at = |index: usize| manifest.views.get(index).map(|(view, _)| *view);
        let Some(current) = view_at(shown) else {
            return;
        };
        self.receive(frame);
        let have = |slots: &[Slot], view: View| slots.iter().any(|slot| slot.view == view);
        // The picture coming in first, then the one going out.
        if self.decoding.is_none() && !have(&self.slots, current) {
            self.decode(set, current, manifest.size, scale, frame.screen);
        }
        // The wanted picture takes over once it is in; the last one fades out.
        let now = Instant::now();
        if have(&self.slots, current) && self.on_show != Some(current) {
            self.fading_from = self.on_show.map(|view| (view, now));
            self.on_show = Some(current);
        }
        let Some(shown_view) = self.on_show else {
            return;
        };
        let (from_view, mix) = match self.fading_from {
            Some((view, began)) if now.duration_since(began) < FADE => (
                view,
                now.duration_since(began).as_secs_f32() / FADE.as_secs_f32(),
            ),
            _ => {
                self.fading_from = None;
                (shown_view, 1.0)
            }
        };
        self.ensure_gpu(frame);
        let Some(gpu) = &self.gpu else {
            return;
        };
        let controls = [scale[0], scale[1], mix, 0.0];
        frame
            .queue
            .write_buffer(&gpu.controls, 0, bytemuck::cast_slice(&controls));
        let exposure = frame.exposure.unwrap_or(&gpu.neutral_exposure).clone();
        let fresh = self.bound.as_ref().is_some_and(|bound| {
            bound.current == shown_view && bound.previous == from_view && bound.exposure == exposure
        });
        if !fresh {
            let texture = |view: View| {
                &self
                    .slots
                    .iter()
                    .find(|slot| slot.view == view)
                    .expect("a decoded picture")
                    .texture
            };
            let group = frame.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("SJK menu picture"),
                layout: &gpu.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(texture(shown_view)),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(texture(from_view)),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&gpu.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: gpu.controls.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: exposure.as_entire_binding(),
                    },
                ],
            });
            self.bound = Some(Bound {
                current: shown_view,
                previous: from_view,
                exposure,
                group,
            });
        }
        // Only the pictures on show stay on the card.
        self.slots.retain(|slot| {
            slot.view == shown_view || slot.view == from_view || slot.view == current
        });
        self.draws = true;
    }

    /// Whether the wanted picture is on show, decoded and faded in (world shots wait
    /// for it).
    #[cfg(test)]
    pub(crate) fn settled(&self) -> bool {
        self.decoding.is_none() && self.fading_from.is_none() && self.on_show.is_some()
    }

    /// Draw this frame's picture over the whole view; first in the world pass.
    pub(crate) fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        let (true, Some(gpu), Some(bound)) = (self.draws, &self.gpu, &self.bound) else {
            return;
        };
        pass.set_pipeline(&gpu.pipeline);
        pass.set_bind_group(0, &bound.group, &[]);
        pass.draw(0..3, 0..1);
    }

    /// Start decoding `view`'s picture, scaled to what a `screen` showing `scale` of
    /// it needs (never up).
    fn decode(
        &mut self,
        set: &'static Set,
        view: View,
        size: [u32; 2],
        scale: [f32; 2],
        screen: [u32; 2],
    ) {
        let Some(jpeg) = set.jpeg(view) else {
            self.failed = true;
            return;
        };
        let shrink = (screen[1] as f32 / (size[1] as f32 * scale[1]))
            .max(screen[0] as f32 / (size[0] as f32 * scale[0]))
            .min(1.0);
        let target = [
            ((size[0] as f32 * shrink).round() as u32).max(1),
            ((size[1] as f32 * shrink).round() as u32).max(1),
        ];
        let (sender, receiver) = mpsc::channel();
        let spawned = std::thread::Builder::new()
            .name("sjk-menu-picture".into())
            .spawn(move || {
                let image = image::load_from_memory_with_format(jpeg, image::ImageFormat::Jpeg)
                    .ok()
                    .map(|image| {
                        let image = image.to_rgba8();
                        if [image.width(), image.height()] == target {
                            image
                        } else {
                            image::imageops::resize(
                                &image,
                                target[0],
                                target[1],
                                image::imageops::FilterType::Triangle,
                            )
                        }
                    });
                let _ = sender.send(image);
            });
        if spawned.is_ok() {
            self.decoding = Some(Decoding { view, receiver });
        } else {
            self.failed = true;
        }
    }

    /// Upload a picture the worker has decoded.
    fn receive(&mut self, frame: &Frame<'_>) {
        let Some(decoding) = &self.decoding else {
            return;
        };
        let image = match decoding.receiver.try_recv() {
            Err(TryRecvError::Empty) => return,
            Ok(Some(image)) => image,
            Ok(None) | Err(TryRecvError::Disconnected) => {
                crate::log::progress(format_args!(
                    "menu pictures: {} did not decode",
                    decoding.view.name()
                ));
                self.decoding = None;
                self.failed = true;
                return;
            }
        };
        let view = decoding.view;
        self.decoding = None;
        let size = wgpu::Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        };
        let texture = frame.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("SJK menu picture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // Display values, read as they are: the shader decodes them.
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        frame.queue.write_texture(
            texture.as_image_copy(),
            image.as_raw(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.width),
                rows_per_image: Some(size.height),
            },
            size,
        );
        self.slots.push(Slot {
            view,
            texture: texture.create_view(&wgpu::TextureViewDescriptor::default()),
        });
    }

    fn ensure_gpu(&mut self, frame: &Frame<'_>) {
        if self
            .gpu
            .as_ref()
            .is_none_or(|gpu| gpu.format != frame.scene_format)
        {
            self.gpu = Some(build(frame.device, frame.scene_format));
            self.bound = None;
        }
    }
}

fn build(device: &wgpu::Device, format: wgpu::TextureFormat) -> Gpu {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("SJK menu picture"),
        source: wgpu::ShaderSource::Wgsl(
            concat!(
                include_str!("menu_pictures_paint.wgsl"),
                include_str!("post_hdr.wgsl")
            )
            .into(),
        ),
    });
    let texture = |binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        count: None,
        ty: wgpu::BindingType::Texture {
            multisampled: false,
            view_dimension: wgpu::TextureViewDimension::D2,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
        },
    };
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("SJK menu picture"),
        entries: &[
            texture(0),
            texture(1),
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                count: None,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                count: None,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                count: None,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            },
        ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("SJK menu picture"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let encoding = crate::frame_target::aa::effects::Encoding { scene: format }.mode();
    let constants = [("ENCODING", encoding)];
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("SJK menu picture"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: &constants,
                ..Default::default()
            },
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: Default::default(),
        // The world pass's depth, untouched: the picture is behind everything.
        depth_stencil: Some(wgpu::DepthStencilState {
            format: crate::DepthTarget::FORMAT,
            depth_write_enabled: Some(false),
            depth_compare: Some(wgpu::CompareFunction::Always),
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    });
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("SJK menu picture"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let controls = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("SJK menu picture controls"),
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    Gpu {
        pipeline,
        layout,
        sampler,
        controls,
        neutral_exposure: crate::frame_target::aa::exposure::neutral(device),
        format,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_screen_as_wide_as_the_pictures_shows_them_whole() {
        let (fov, scale) = fit(90.0, 16.0 / 9.0, 16.0 / 9.0);
        assert!((fov - 90.0).abs() < 1e-3);
        assert!((scale[0] - 1.0).abs() < 1e-5 && (scale[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn a_narrower_screen_crops_the_sides_and_a_wider_one_the_top_and_bottom() {
        let (fov, scale) = fit(90.0, 16.0 / 9.0, 4.0 / 3.0);
        assert!((fov - 90.0).abs() < 1e-3, "the same vertical view");
        assert!((scale[0] - 0.75).abs() < 1e-4 && (scale[1] - 1.0).abs() < 1e-5);
        let (fov, scale) = fit(90.0, 16.0 / 9.0, 64.0 / 27.0);
        assert!(
            fov < 90.0,
            "narrowed so the screen stays inside the picture"
        );
        assert!((scale[0] - 1.0).abs() < 1e-4, "{scale:?}");
        assert!((scale[1] - 0.75).abs() < 1e-4, "{scale:?}");
    }
}
