//! The scene's raster size against the window's, one number: the scene's size in percent
//! of the window per axis. Supersampling (`r_superSample` 2 or 3, 200 or 300 %) draws the
//! scene larger and box-resolves it; the render scale (`r_renderScale`, 50 to 99 %) draws
//! it smaller and stretches it with a bilinear filter and a light sharpen, for weak GPUs.
//! Either way the resolve comes before the scene's postprocessing, and the HUD, menus and
//! text are drawn after it at the window's size. Supersampling wins: at `r_superSample`
//! 2 or 3 the render scale is ignored. At 100 % no scene target is made: the frame is the
//! one without either setting. Target ownership stays in the viewer.
use sjk_shell::{CvarDefinition, CvarError, CvarFlags, CvarRegistry};

/// The render scale: the world's size in percent of the window per axis, 50 to 100.
pub(crate) const CVAR: &str = "r_renderScale";
/// The smallest render scale: a quarter of the window's pixels.
pub(crate) const MIN_PERCENT: u32 = 50;
/// The window's own size: no scene target of its own.
pub(crate) const NATIVE: u32 = 100;
/// The sharpen after the render scale's stretch, 0 (none) to 1 (the most RCAS allows).
const SHARPNESS: f64 = 0.5;

/// Register startup policy; changing attachment dimensions takes a graphics reload.
pub(crate) fn register(cvars: &mut CvarRegistry) -> Result<(), CvarError> {
    cvars.register(CvarDefinition::new(
        "r_superSample",
        1_i64,
        CvarFlags::ARCHIVE,
        "Scene supersampling: 1 off, 2 or 3 per axis; applies after a graphics reload (vid_restart)",
    ))?;
    cvars.on_change("r_superSample", |_| {
        crate::log::progress(format_args!(
            "r_superSample changed: {}",
            crate::graphics_reload::APPLY
        ));
        crate::graphics_reload::notice();
    })?;
    cvars.register(CvarDefinition::new(
        CVAR,
        i64::from(NATIVE),
        CvarFlags::ARCHIVE,
        "Render scale: the world drawn at 50-100 % of the window per axis and stretched to it, \
         the HUD and menus at full size; ignored while r_superSample is above 1; applies after \
         a graphics reload (vid_restart)",
    ))?;
    cvars.on_change(CVAR, |_| {
        crate::log::progress(format_args!(
            "{CVAR} changed: {}",
            crate::graphics_reload::APPLY
        ));
        crate::graphics_reload::notice();
    })?;
    // The window takes a new size at once (`runtime_settings.rs`); the world's resolution
    // with the graphics reload.
    cvars.register(CvarDefinition::new(
        crate::settings::HIGH_DPI_CVAR,
        crate::settings::HIGH_DPI_DEFAULT,
        CvarFlags::ARCHIVE,
        "1 the window's size (r_resolution) and the world count the screen's pixels, 0 its \
         points as EternalJK on a Mac (the world at half a Retina screen's pixels each way, \
         the HUD and menus sharp); default 0 on a Mac, 1 elsewhere; the world's resolution \
         applies after a graphics reload (vid_restart)",
    ))?;
    cvars.on_change(crate::settings::HIGH_DPI_CVAR, |_| {
        crate::log::progress(format_args!(
            "{} changed: {}",
            crate::settings::HIGH_DPI_CVAR,
            crate::graphics_reload::APPLY
        ));
        crate::graphics_reload::notice();
    })
}

/// The scene's size in percent of the window per axis the loaded config asks for:
/// `r_superSample` 2 or 3 as 200 or 300, else `r_renderScale`, 50 to 100. Sample the
/// actual loaded config, not a parallel default-only settings object.
pub(crate) fn requested(console: Option<&crate::console::ViewerConsole>) -> u32 {
    // `SJK_RENDER_SCALE` overrides the archived supersampling for one run (a launcher's
    // choice).
    let supersample = std::env::var("SJK_RENDER_SCALE")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .or_else(|| console.and_then(|c| c.integer_cvar("r_superSample")))
        .unwrap_or(1)
        .clamp(1, 3) as u32;
    if supersample > 1 {
        return supersample * NATIVE;
    }
    console
        .and_then(|c| c.integer_cvar(CVAR))
        .unwrap_or(i64::from(NATIVE))
        .clamp(i64::from(MIN_PERCENT), i64::from(NATIVE)) as u32
}

/// The scene's percent of the window's pixels for `percent` of its points: divided by the
/// window's `scale_factor` (pixels per point) without high DPI (`r_highDpi` 0), so 100 %
/// on a Retina screen draws half its pixels each way, as EternalJK does there. Unchanged
/// with high DPI or on an unscaled screen. Supersampling that becomes a fraction above
/// the pixels (3 on a Retina screen, 150 %) is drawn at the native size.
pub(crate) fn in_points(percent: u32, high_dpi: bool, scale_factor: f64) -> u32 {
    if high_dpi || scale_factor <= 1.0 {
        return percent;
    }
    ((f64::from(percent) / scale_factor).round() as u32).max(1)
}

/// The scene's size for a window of `output` at `percent`, rounded, at least a pixel;
/// exactly 2 or 3 times the window when supersampled.
pub(crate) fn scaled(output: [u32; 2], percent: u32) -> [u32; 2] {
    output.map(|n| ((u64::from(n) * u64::from(percent) + 50) / 100).max(1) as u32)
}

/// The percent to install for `request`: a render scale as asked, as a smaller scene
/// always fits; supersampling at the largest factor up to the request whose scene fits the
/// device's texture size and four colour/depth pairs in 512 MiB (HDR RGBA16F uses 48
/// rather than 32 bytes); else native.
pub(crate) fn supported_bytes(request: u32, output: [u32; 2], maximum: u32, bytes: u64) -> u32 {
    if request < NATIVE {
        return request.max(MIN_PERCENT);
    }
    (2..=(request / NATIVE).clamp(1, 3))
        .rev()
        .find(|&scale| {
            let [w, h] = output.map(|n| u64::from(n) * u64::from(scale));
            w > 0 && h > 0 && w <= u64::from(maximum) && h <= u64::from(maximum)
            // Reserve four colour/depth pairs, including secondary/preview headroom.
            && w.saturating_mul(h).saturating_mul(bytes) <= 512 * 1024 * 1024
        })
        .map_or(NATIVE, |scale| scale * NATIVE)
}

/// Allocate transactionally at installation/resize, never during steady frame drawing.
pub(crate) fn checked<T>(device: &wgpu::Device, build: impl FnOnce() -> T) -> Option<T> {
    crate::gpu_guard::capture(device, build)
        .inspect_err(|error| {
            crate::log::progress(format_args!(
                "scene scale allocation failed: {error}; reducing scale"
            ));
        })
        .ok()
}

/// Retained scene colour and its resolve into the window's size: an integer box resolve
/// when supersampled, a bilinear stretch and light sharpen below native. HUD depth remains
/// output-sized.
pub(crate) struct Runtime {
    /// Scene colour at the same dimensions as main depth and secondary views.
    pub(crate) scene: wgpu::TextureView,
    /// Native attachment for pipelines that depth-test neither HUD nor text.
    pub(crate) hud_depth: crate::DepthTarget,
    /// Effective scene size, after dimension and memory clamps.
    pub(crate) size: [u32; 2],
    /// The scene's size in percent of the window per axis; never [`NATIVE`].
    pub(crate) percent: u32,
    bind: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl Runtime {
    /// Create inside an allocation error scope, before publishing any resized attachment.
    pub(crate) fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        output: [u32; 2],
        percent: u32,
    ) -> Self {
        let size = scaled(output, percent);
        let supersampled = percent > NATIVE;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(if supersampled {
                "SJK supersampled scene"
            } else {
                "SJK render-scale scene"
            }),
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
        let scene = texture.create_view(&Default::default());
        let (pipeline, bind) = if supersampled {
            let pipeline = resolve_pipeline(
                device,
                format,
                "SJK integer box resolve",
                include_str!("render_scale.wgsl"),
                &[("SCALE", f64::from(percent / NATIVE))],
            );
            let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &pipeline.get_bind_group_layout(0),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&scene),
                }],
            });
            (pipeline, bind)
        } else {
            let pipeline = resolve_pipeline(
                device,
                format,
                "SJK render-scale stretch",
                include_str!("render_scale_up.wgsl"),
                &[("SHARPNESS", SHARPNESS)],
            );
            let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("SJK render-scale stretch"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            });
            let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("SJK render-scale stretch"),
                layout: &pipeline.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&scene),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });
            (pipeline, bind)
        };
        Self {
            scene,
            size,
            percent,
            bind,
            pipeline,
            hud_depth: crate::DepthTarget::labelled(device, output[0], output[1], "SJK HUD depth"),
        }
    }

    /// The whole supersampling factor the scene holds (1 below native), which the light
    /// buffer divides out to stay at half the window's size.
    pub(crate) fn supersampling(&self) -> u32 {
        (self.percent / NATIVE).max(1)
    }

    /// Average every covered linear-light texel (supersampled) or stretch the scene over
    /// the window (render scale), without filtering the later HUD.
    pub(crate) fn draw(&self, encoder: &mut wgpu::CommandEncoder, output: &wgpu::TextureView) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(if self.percent > NATIVE {
                "SJK supersample box resolve"
            } else {
                "SJK render-scale stretch"
            }),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind, &[]);
        pass.draw(0..3, 0..1);
    }
}

/// A full-screen pass from the scene into a `format` target: `source`'s `vertex` and
/// `fragment` entry points, with its override `constants`.
fn resolve_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    label: &str,
    source: &str,
    constants: &[(&str, f64)],
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("SJK scene resolve before scene postprocessing"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vertex"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fragment"),
            compilation_options: wgpu::PipelineCompilationOptions {
                constants,
                ..Default::default()
            },
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    })
}

impl crate::GpuState {
    /// Window resize retains native output dimensions, then rebuilds scene attachments.
    pub(crate) fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.size = size;
        self.configuration.width = size.width;
        self.configuration.height = size.height;
        // The frame in flight presents before its swapchain is reconfigured.
        self.frame_pacer.split.wait_previous();
        if let Some(surface) = &self.context.surface {
            surface.configure(&self.device, &self.configuration);
        }
        self.resize_scene_depth(size);
    }

    /// Output aspect is unchanged; only raster dimensions change.
    pub(crate) fn scene_size(&self) -> [u32; 2] {
        self.render_scale
            .as_ref()
            .map_or([self.size.width, self.size.height], |s| s.size)
    }

    /// The whole supersampling factor the scene holds: 1 at native and below.
    pub(crate) fn scene_supersampling(&self) -> u32 {
        self.render_scale.as_ref().map_or(1, Runtime::supersampling)
    }

    /// The offscreen texture the scene is drawn into, when the frame has one; without it
    /// the scene is drawn straight into the swapchain image.
    pub(crate) fn offscreen_scene_target(&self) -> Option<&wgpu::TextureView> {
        self.render_scale
            .as_ref()
            .map(|scale| &scale.scene)
            .or_else(|| self.post_aa.as_ref().map(|aa| &aa.scene))
    }

    /// Publish scaled attachments only after the entire candidate succeeded.
    pub(crate) fn with_render_scale(mut self) -> Self {
        self.install_render_scale();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_high_dpi_the_scene_counts_points() {
        // A Retina screen: 100 % of its points is half its pixels each way.
        assert_eq!(in_points(100, false, 2.0), 50);
        assert_eq!(in_points(50, false, 2.0), 25);
        assert_eq!(in_points(200, false, 2.0), NATIVE);
        // Windows at 150 %.
        assert_eq!(in_points(100, false, 1.5), 67);
        // High DPI, or a screen without scaling, keeps the pixels.
        assert_eq!(in_points(100, true, 2.0), 100);
        assert_eq!(in_points(75, false, 1.0), 75);
    }

    fn profile(cvars: &[(&str, &str)]) -> (tempfile::TempDir, crate::console::ViewerConsole) {
        let directory = tempfile::tempdir().expect("a profile directory");
        let mut console = crate::console::ViewerConsole::new(directory.path().join("config.cfg"))
            .expect("a console");
        for (name, value) in cvars {
            console.set_cvar(name, value);
        }
        (directory, console)
    }

    #[test]
    fn the_window_size_is_the_default() {
        // 100 %: `install_render_scale_for` makes no scene target, so the frame is drawn
        // exactly as before the setting existed.
        let (_profile, console) = profile(&[]);
        assert_eq!(console.integer_cvar(CVAR), Some(i64::from(NATIVE)));
        assert_eq!(requested(Some(&console)), NATIVE);
        assert_eq!(requested(None), NATIVE);
        assert_eq!(supported_bytes(NATIVE, [1920, 1080], 16384, 48), NATIVE);
    }

    #[test]
    fn the_render_scale_keeps_to_its_range() {
        for (value, percent) in [("75", 75), ("50", 50), ("30", 50), ("0", 50), ("150", 100)] {
            let (_profile, console) = profile(&[(CVAR, value)]);
            assert_eq!(requested(Some(&console)), percent, "{CVAR} {value}");
        }
    }

    #[test]
    fn supersampling_wins_over_the_render_scale() {
        let (_profile, console) = profile(&[("r_superSample", "2"), (CVAR, "50")]);
        assert_eq!(requested(Some(&console)), 200);
        let (_profile, console) = profile(&[("r_superSample", "1"), (CVAR, "50")]);
        assert_eq!(requested(Some(&console)), 50);
    }

    #[test]
    fn scene_sizes_round_and_supersampling_stays_exact() {
        assert_eq!(scaled([1920, 1080], 50), [960, 540]);
        assert_eq!(scaled([1920, 1080], 75), [1440, 810]);
        assert_eq!(scaled([1366, 768], 67), [915, 515]);
        assert_eq!(scaled([1, 1], 50), [1, 1]);
        assert_eq!(scaled([1921, 1081], 200), [3842, 2162]);
        assert_eq!(scaled([1921, 1081], 300), [5763, 3243]);
    }

    #[test]
    fn a_render_scale_always_fits_and_supersampling_falls_back() {
        assert_eq!(supported_bytes(50, [3840, 2160], 2048, 48), 50);
        assert_eq!(supported_bytes(75, [1920, 1080], 16384, 48), 75);
        // 1920x1080 at 3x in HDR is 854 MiB for four pairs, past 512 MiB; 2x fits.
        assert_eq!(supported_bytes(300, [1920, 1080], 16384, 48), 200);
        // Past the device's texture size, native.
        assert_eq!(supported_bytes(200, [3840, 2160], 4096, 32), NATIVE);
    }
}
