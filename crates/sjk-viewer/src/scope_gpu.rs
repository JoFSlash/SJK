//! Fixed-capacity stock scope quads; textures are resolved once at world construction.
use super::binoculars;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

const MASK_IMAGE: &str = "gfx/2d/cropcircle2";

const PATHS: [&str; 5] = [
    "gfx/misc/scanline",
    // The mask stage's own image, looked up as an image (not through the
    // `gfx/2d/cropCircle2` shader, whose other stage drew a full-screen white
    // picture) in the engine's `.jpg`, `.png`, `.tga` order, so JoF's HD scope
    // (`JoF_HDWeaponScopeTrue.pk3`, `cropcircle2.png`) wins as in EternalJK.
    MASK_IMAGE,
    "gfx/2d/cropCircle",
    "gfx/2d/insertTick",
    "gfx/2d/crop_charge",
];

/// The binoculars' stage images (`shaders/zoom.shader` in `assets1.pk3`), by stage
/// image rather than shader, as the disruptor's mask is: the window's static, the
/// number strip's two layers and its scanlines, the circle, the frame, the arrow and
/// the triangle. They follow [`PATHS`] in [`Mask::textures`], then a white texel for
/// the black box and the markers (`cgs.media.whiteShader`). The binoculars draw only
/// when every one of them loaded.
const BINOCULAR_PATHS: [&str; 8] = [
    "gfx/2d/zoomWindow",
    "gfx/2d/zoomNumOverlay2",
    "gfx/2d/zoomNumOverlay",
    "gfx/misc/scanline2",
    "gfx/2d/binCircle",
    "gfx/2d/binMask",
    "gfx/2d/binSideArrow",
    "gfx/2d/binTopTri",
];

// Material indices: the disruptor's, then the binoculars'.
const SCANLINE: usize = 0;
const TICK: usize = 3;
const WINDOW: usize = 5;
const NUMBER_SHADE: usize = 6;
const NUMBER_GLOW: usize = 7;
const NUMBER_SCANLINE: usize = 8;
const CIRCLE: usize = 9;
const FRAME: usize = 10;
const ARROW: usize = 11;
const TRIANGLE: usize = 12;
const WHITE: usize = 13;

/// Pipelines by blend: `GL_SRC_ALPHA GL_ONE_MINUS_SRC_ALPHA`, `GL_ONE GL_ONE`, and the
/// filter `GL_DST_COLOR GL_SRC_COLOR`.
const ALPHA: usize = 0;
const ADD: usize = 1;
const FILTER: usize = 2;

/// Most quads a frame holds: the binoculars' fifteen-odd, the disruptor's twenty-odd.
const QUADS: usize = 48;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}

const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
    wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4];

/// A stage's texture-coordinate change: `tcMod scale`, then `tcMod scroll`'s offset.
#[derive(Clone, Copy)]
struct Uv {
    scale: [f32; 2],
    offset: [f32; 2],
}

impl Uv {
    const PLAIN: Self = Self::scaled(1.0, 1.0);

    const fn scaled(u: f32, v: f32) -> Self {
        Self {
            scale: [u, v],
            offset: [0.0, 0.0],
        }
    }

    /// `tcMod scroll`: the fraction of `speed` times the shader time in seconds.
    fn scrolled(self, speed: [f32; 2], seconds: f32) -> Self {
        Self {
            offset: speed.map(|speed| (speed * seconds).fract()),
            ..self
        }
    }
}

/// Map-owned stock textures and a bounded batch of scope quads.
pub(crate) struct Mask {
    pipeline: [wgpu::RenderPipeline; 3],
    textures: Vec<wgpu::BindGroup>,
    binoculars: bool,
    /// The binoculars' top triangle, which flips now and then (stock's `static flip`).
    triangle_flipped: bool,
    /// Stand-in for `Q_flrand` in the flip and the flicker, advanced each frame.
    random: u32,
    buffer: wgpu::Buffer,
    vertices: [Vertex; QUADS * 6],
    materials: [usize; QUADS],
    count: usize,
}

impl Mask {
    /// Load retail shader images without requiring scope assets in custom-only installations.
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &crate::frame_queue::FrameQueue,
        format: wgpu::TextureFormat,
        vfs: &sjk_vfs::VirtualFileSystem,
        shaders: &sjk_shader::ShaderCatalog,
    ) -> Option<Self> {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("scope textures"),
            entries: &[
                crate::render_helpers::texture_layout_entry(0),
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let upload = |name: &str, image: &image::RgbaImage| {
            let texture = device.create_texture_with_data(
                queue.raw(),
                &wgpu::TextureDescriptor {
                    label: Some(name),
                    size: wgpu::Extent3d {
                        width: image.width(),
                        height: image.height(),
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: crate::ui_target::TEXTURE_FORMAT,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                },
                wgpu::util::TextureDataOrder::LayerMajor,
                image.as_raw(),
            );
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(name),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &texture.create_view(&Default::default()),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            })
        };
        let mut textures = Vec::with_capacity(PATHS.len() + BINOCULAR_PATHS.len() + 1);
        for name in PATHS {
            let loaded = if name == MASK_IMAGE {
                stage_image(vfs, shaders, name)
            } else if name.ends_with(".tga") {
                vfs.read(name)
                    .map_err(|error| Box::new(error) as Box<dyn std::error::Error>)
                    .and_then(|asset| asset.ok_or_else(|| format!("missing {name}").into()))
                    .and_then(|asset| {
                        crate::decode_image(&asset.bytes, name)
                            .map(|image| image.into_rgba8())
                            .map_err(Into::into)
                    })
            } else {
                crate::shader_image::load_shader_image(vfs, shaders, name)
            };
            let image = match loaded {
                Ok(image) => image,
                Err(error) => {
                    eprintln!("scope unavailable: {error}");
                    return None;
                }
            };
            textures.push(upload(name, &image));
        }
        let binocular_images: Result<Vec<_>, _> = BINOCULAR_PATHS
            .iter()
            .map(|name| stage_image(vfs, shaders, name).map(|image| (*name, image)))
            .collect();
        let binoculars = match binocular_images {
            Ok(images) => {
                for (name, image) in &images {
                    textures.push(upload(name, image));
                }
                textures.push(upload(
                    "white",
                    &image::RgbaImage::from_pixel(1, 1, image::Rgba([255; 4])),
                ));
                true
            }
            Err(error) => {
                eprintln!("binocular view unavailable: {error}");
                false
            }
        };
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("scope"),
            source: wgpu::ShaderSource::Wgsl(include_str!("scope.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("scope"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let add = wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        };
        let filter = wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::Dst,
            dst_factor: wgpu::BlendFactor::Src,
            operation: wgpu::BlendOperation::Add,
        };
        let pipeline = [
            wgpu::BlendState::ALPHA_BLENDING,
            wgpu::BlendState {
                color: add,
                alpha: wgpu::BlendComponent::OVER,
            },
            wgpu::BlendState {
                color: filter,
                alpha: wgpu::BlendComponent::OVER,
            },
        ]
        .map(|blend| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("scope"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &ATTRIBUTES,
                    })],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(blend),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: Default::default(),
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
            })
        });
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scope quads"),
            size: (QUADS * 6 * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Some(Self {
            pipeline,
            textures,
            binoculars,
            triangle_flipped: true,
            random: 0x9E37_79B9,
            buffer,
            vertices: [Vertex::zeroed(); QUADS * 6],
            materials: [0; QUADS],
            count: 0,
        })
    }

    /// Rebuild the original 640x480 mask, rotating FOV insert, ammo ticks and charge readout.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare(
        &mut self,
        queue: &crate::frame_queue::FrameQueue,
        visible: bool,
        style: i64,
        fov: f32,
        time: i32,
        ammo: f32,
        charge: Option<f32>,
    ) {
        self.count = 0;
        if !visible {
            return;
        }
        let [mask, insert] = crate::cgame_options::scope_layers(style);
        if mask {
            self.quad(
                SCANLINE,
                [320.0, 240.0, 640.0, 480.0],
                0.0,
                [1.0; 4],
                Uv::scaled(1.0, 10.5),
            );
            self.quad(1, [320.0, 240.0, 640.0, 480.0], 0.0, [1.0; 4], Uv::PLAIN);
        }
        let level = ((50.0 - fov) / 50.0).clamp(0.0, 1.0) * 103.0;
        let alpha = if level >= 99.0 {
            0.7 + (time as f32 * 0.01).sin() * 0.3
        } else {
            1.0
        };
        if insert {
            self.quad(
                2,
                [320.0, 240.0, 640.0, 480.0],
                -level,
                [1.0, 1.0, 1.0, alpha],
                Uv::PLAIN,
            );
        }
        let ammo = ammo.clamp(0.0, 1.0);
        let color = if ammo < 0.15 && time & 512 != 0 {
            [0.0, 0.0, 0.0, 1.0]
        } else {
            [
                ((1.0 - ammo) * 2.0).min(1.0),
                (ammo * 1.5).min(1.0),
                0.0,
                1.0,
            ]
        };
        let mut angle = 18.5;
        while angle <= 18.5 + ammo * 58.0 {
            let radians: f32 = (angle + 90.0) / 57.296;
            self.quad(
                TICK,
                [
                    320.0 + radians.sin() * 190.0,
                    240.0 + radians.cos() * 190.0,
                    12.0,
                    24.0,
                ],
                90.0 - angle,
                color,
                Uv::PLAIN,
            );
            angle += 3.0;
        }
        if let Some(charge) = charge {
            let charge = charge.clamp(0.0, 1.0);
            self.quad(
                4,
                [257.0 + 67.0 * charge, 452.0, 134.0 * charge, 34.0],
                0.0,
                [1.0; 4],
                Uv::scaled(charge, 1.0),
            );
        }
        self.upload(queue);
    }

    /// Whether the binoculars' pictures all loaded.
    pub(crate) fn has_binoculars(&self) -> bool {
        self.binoculars
    }

    /// The binoculars' view for this frame (`CG_DrawZoomMask`, `zoomMode == 2`), in
    /// stock's order: window, black box, markers, number strip, circle, frame, arrow,
    /// triangle. `zoom_fov` is the player state's, `yaw` the view's in degrees and
    /// `time` the presentation time in milliseconds (the shaders' clock too).
    pub(crate) fn prepare_binoculars(
        &mut self,
        queue: &crate::frame_queue::FrameQueue,
        zoom_fov: f32,
        yaw: f32,
        time: i32,
    ) {
        self.count = 0;
        if !self.binoculars {
            return;
        }
        let seconds = time as f32 * 0.001;
        let pic = |x: f32, y: f32, w: f32, h: f32| [x + w * 0.5, y + h * 0.5, w, h];
        // `gfx/2d/binocularWindow`: the scrolling static, then scanlines (`tcMod scale 1 5`).
        let window = pic(34.0, 48.0, 570.0, 362.0);
        self.quad(
            WINDOW,
            window,
            0.0,
            [1.0; 4],
            Uv::PLAIN.scrolled([60.0, 41.5], seconds),
        );
        self.quad(SCANLINE, window, 0.0, [1.0; 4], Uv::scaled(1.0, 5.0));
        let [x, y, w, h] = binoculars::NUMBER_BOX;
        self.quad(WHITE, pic(x, y, w, h), 0.0, [0.0, 0.0, 0.0, 1.0], Uv::PLAIN);
        let [marker_y, marker_size] = binoculars::MARKER;
        for marker_x in binoculars::markers(yaw) {
            self.quad(
                WHITE,
                pic(marker_x, marker_y, marker_size, marker_size),
                0.0,
                binoculars::NUMBER_COLOR,
                Uv::PLAIN,
            );
        }
        // `gfx/2d/binocularNumOverlay`'s three stages over the numbers.
        let strip = pic(212.0, 367.0, 200.0, 28.0);
        self.quad(NUMBER_SHADE, strip, 0.0, [1.0; 4], Uv::PLAIN);
        self.quad(NUMBER_GLOW, strip, 0.0, [1.0; 4], Uv::PLAIN);
        self.quad(
            NUMBER_SCANLINE,
            strip,
            0.0,
            [1.0; 4],
            Uv::scaled(1.0, 0.7).scrolled([0.0, 6.2], seconds),
        );
        let grey = binoculars::circle_grey(time);
        self.quad(
            CIRCLE,
            pic(82.0, 94.0, 16.0, 16.0),
            0.0,
            [grey, grey, grey, 1.0],
            Uv::PLAIN,
        );
        // The frame's shader ignores the colour (`rgbGen identity`); the arrow and the
        // triangle take the flicker.
        self.quad(FRAME, pic(0.0, 0.0, 640.0, 480.0), 0.0, [1.0; 4], Uv::PLAIN);
        let flicker = [
            0.7 + self.next_random(-0.1, 0.1),
            0.8 + self.next_random(-0.1, 0.1),
            0.7 + self.next_random(-0.1, 0.1),
            1.0,
        ];
        let level = binoculars::arrow_level(zoom_fov);
        self.quad(
            ARROW,
            pic(4.0, 282.0 - level, 16.0, 16.0),
            0.0,
            flicker,
            Uv::PLAIN,
        );
        // A negative size draws the picture mirrored, as `CG_DrawPic` does.
        let triangle = if self.triangle_flipped {
            pic(330.0, 60.0, -26.0, -30.0)
        } else {
            pic(307.0, 40.0, 26.0, 30.0)
        };
        self.quad(TRIANGLE, triangle, 0.0, flicker, Uv::PLAIN);
        let chance = self.next_random(0.0, 1.0);
        if binoculars::flips(time, chance) {
            self.triangle_flipped = !self.triangle_flipped;
        }
        self.upload(queue);
    }

    /// A number in `low..high` from a small xorshift, standing in for `Q_flrand`.
    fn next_random(&mut self, low: f32, high: f32) -> f32 {
        self.random ^= self.random << 13;
        self.random ^= self.random >> 17;
        self.random ^= self.random << 5;
        low + (self.random >> 8) as f32 / (1u32 << 24) as f32 * (high - low)
    }

    fn upload(&self, queue: &crate::frame_queue::FrameQueue) {
        queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(&self.vertices[..self.count * 6]),
        );
    }

    fn quad(&mut self, material: usize, rect: [f32; 4], degrees: f32, color: [f32; 4], uv: Uv) {
        if self.count == QUADS {
            return;
        }
        let (sin, cos) = degrees.to_radians().sin_cos();
        for (index, [u, v]) in [
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
        ]
        .into_iter()
        .enumerate()
        {
            let x = (u - 0.5) * rect[2];
            let y = (v - 0.5) * rect[3];
            self.vertices[self.count * 6 + index] = Vertex {
                position: [
                    (rect[0] + x * cos - y * sin) / 320.0 - 1.0,
                    1.0 - (rect[1] + x * sin + y * cos) / 240.0,
                ],
                uv: [
                    u * uv.scale[0] + uv.offset[0],
                    v * uv.scale[1] + uv.offset[1],
                ],
                color,
            };
        }
        self.materials[self.count] = material;
        self.count += 1;
    }

    /// Draw after scene geometry and before text with no per-frame allocation.
    pub(crate) fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        if self.count == 0 {
            return;
        }
        pass.set_vertex_buffer(0, self.buffer.slice(..));
        for index in 0..self.count {
            let material = self.materials[index];
            pass.set_pipeline(&self.pipeline[blend(material)]);
            pass.set_bind_group(0, &self.textures[material], &[]);
            pass.draw(index as u32 * 6..index as u32 * 6 + 6, 0..1);
        }
    }
}

/// Each material's blend, from its shader stage's `blendFunc`.
const fn blend(material: usize) -> usize {
    match material {
        SCANLINE | WINDOW | NUMBER_SHADE | NUMBER_SCANLINE => FILTER,
        TICK | NUMBER_GLOW | CIRCLE | ARROW | TRIANGLE => ADD,
        _ => ALPHA,
    }
}

/// One stage image by its exact name in the engine's `.jpg`, `.png`, `.tga` order.
fn stage_image(
    vfs: &sjk_vfs::VirtualFileSystem,
    shaders: &sjk_shader::ShaderCatalog,
    name: &str,
) -> Result<image::RgbaImage, Box<dyn std::error::Error>> {
    shaders
        .resolve_stage_image(vfs, name)
        .map_err(|error| Box::new(error) as Box<dyn std::error::Error>)
        .and_then(|path| path.ok_or_else(|| format!("missing {name}").into()))
        .and_then(|path| {
            vfs.read(path.as_str())
                .map_err(|error| Box::new(error) as Box<dyn std::error::Error>)
                .and_then(|asset| asset.ok_or_else(|| format!("missing {path}").into()))
                .and_then(|asset| {
                    crate::decode_image(&asset.bytes, path.as_str())
                        .map(|image| image.into_rgba8())
                        .map_err(Into::into)
                })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_material_blends_as_its_shader_stage() {
        // The disruptor's scanlines filter, its ticks add, the rest blend by alpha.
        assert_eq!(blend(SCANLINE), FILTER);
        assert_eq!(blend(1), ALPHA);
        assert_eq!(blend(TICK), ADD);
        // zoom.shader: binocularWindow and the number overlay's outer stages filter,
        // its middle stage, the circle, arrow and triangle add, the frame blends.
        for material in [WINDOW, NUMBER_SHADE, NUMBER_SCANLINE] {
            assert_eq!(blend(material), FILTER);
        }
        for material in [NUMBER_GLOW, CIRCLE, ARROW, TRIANGLE] {
            assert_eq!(blend(material), ADD);
        }
        assert_eq!(blend(FRAME), ALPHA);
        assert_eq!(blend(WHITE), ALPHA);
        assert_eq!(WHITE, PATHS.len() + BINOCULAR_PATHS.len());
    }
}
