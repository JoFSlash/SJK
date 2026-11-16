//! Drawing card scenes (`cards::scene`): one at a time, the figures a card's scene names
//! are built from the game data as the menu stage builds its actor and its props
//! (`menu_stage`), set side by side, framed by their own camera and drawn into a target
//! like the model preview's (`menu_stage::preview`, whose encode makes the 8-bit picture
//! with the background clear), then copied out and read back. A thread lays the picture
//! over the card's backdrop and keeps it as a JPEG (`cards::scene::keep`), and the card's
//! face takes it at its next upload (`ui_renderer::card_art`).
//!
//! Each drawing is recorded and submitted on its own inside validation and out-of-memory
//! scopes, as the preview's probe frame is, so a model that breaks a pipeline fails that
//! drawing alone: the card keeps its pack's picture and the scene is not tried again this
//! session. The figures are lit from the camera's upper left, as a portrait is, whatever
//! map the client stands on: a picture of the model, not of the place. A player model is
//! the game's own, whatever rig model a pack lays over it. Its hilts stand at its hand
//! bolts as on the menu stage, and their blades go through the blade renderer's preview
//! slot, which is the model preview's again once the drawing is recorded.

use super::*;
use crate::actor_instance::EntityLight;
use crate::cards::scene::{self, Figure, Job};
use crate::menu_stage::preview;
use crate::player_assets::{GlaCache, load_stock_player_appearance, split_model_cvar};
use crate::saber_rgb::BladeColor;
use crate::world_materials::{DetachedDraw, DetachedMaterials};
use std::sync::mpsc::{Receiver, TryRecvError, sync_channel};

/// How long the target is kept once nothing is left to draw.
const IDLE: Duration = Duration::from_secs(10);
/// The space between figures side by side, in game units.
const GAP: f32 = 4.0;
/// The light: how much comes from all round and from the key light.
const AMBIENT: f32 = 0.85;
const DIRECTED: f32 = 0.9;

/// One part of a figure built for drawing: the body or a hilt.
struct Built {
    draws: Vec<DetachedDraw>,
    materials: DetachedMaterials,
    vertex_buffer: wgpu::Buffer,
    geometry_binding: wgpu::BindGroup,
    index_buffer: wgpu::Buffer,
    /// Its points as it stands in the figure, turned, before the figure's place in the row.
    points: Vec<Vec3>,
    rotation: Quat,
    /// Where it stands in the figure (a hilt in a hand; the body at 0).
    local: Vec3,
    /// The way it faces.
    front: Vec3,
    /// A hilt's lit blades, in the figure.
    blades: Vec<(crate::saber::Blade, BladeColor)>,
}

/// A drawing being read back.
struct Readback {
    job: Job,
    buffer: wgpu::Buffer,
    padded_row_bytes: u32,
    receiver: Receiver<Result<(), wgpu::BufferAsyncError>>,
}

/// The card scenes' drawing state on [`GpuState`].
#[derive(Default)]
pub(crate) struct CardScenes {
    gla_cache: GlaCache,
    /// The `.sab` definitions, read once the first hilt is needed.
    saber_definitions: Option<BTreeMap<String, crate::saber_defs::Definition>>,
    encode: Option<preview::Encode>,
    target: Option<preview::Target>,
    reading: Option<Readback>,
    /// When the last drawing was made, so the target goes once idle.
    last: Option<Instant>,
}

impl GpuState {
    /// Read back the drawing under way, or draw the next scene asked for. Called once a
    /// frame; costs nothing while no card asks for a drawing.
    pub(crate) fn draw_card_scenes(&mut self) {
        self.poll_card_scene();
        if self.card_scenes.reading.is_some() {
            return;
        }
        let Some(job) = scene::next() else {
            if self
                .card_scenes
                .last
                .is_some_and(|last| last.elapsed() > IDLE)
            {
                self.card_scenes.target = None;
                self.card_scenes.encode = None;
                self.card_scenes.last = None;
            }
            return;
        };
        self.card_scenes.last = Some(Instant::now());
        let device = self.device.clone();
        let failure = match crate::gpu_guard::capture(&device, || self.record_card_scene(&job)) {
            Ok(Ok((buffer, padded_row_bytes))) => {
                let (sender, receiver) = sync_channel(1);
                buffer
                    .slice(..)
                    .map_async(wgpu::MapMode::Read, move |result| {
                        let _ = sender.try_send(result);
                    });
                self.card_scenes.reading = Some(Readback {
                    job,
                    buffer,
                    padded_row_bytes,
                    receiver,
                });
                return;
            }
            Ok(Err(error)) => error.to_string(),
            Err(error) => error.to_string(),
        };
        crate::log::progress(format_args!(
            "cards: {}: the scene was not drawn, the pack's picture stays: {failure}",
            job.id
        ));
        scene::finished(&job, false);
    }

    /// Build `job`'s figures, draw them and copy the picture out, all submitted at once:
    /// the buffer the picture lands in, and its rows' padded length.
    fn record_card_scene(&mut self, job: &Job) -> Result<(wgpu::Buffer, u32), Box<dyn Error>> {
        let vfs = self.vfs.clone().ok_or("no game data")?;
        let format = self.context.scene_format();
        if self
            .card_scenes
            .encode
            .as_ref()
            .is_none_or(|encode| encode.format != format)
        {
            self.card_scenes.encode = Some(preview::Encode::new(&self.device, format));
            self.card_scenes.target = None;
        }
        let mut figures = Vec::with_capacity(job.scene.figures.len());
        for figure in &job.scene.figures {
            figures.push(self.build_card_figure(&vfs, figure)?);
        }
        let front = figures
            .first()
            .and_then(|parts| parts.first())
            .ok_or("no figure")?
            .front;
        // Side by side across the first figure's front, the row centred.
        let across = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2) * front;
        let spans: Vec<(f32, f32)> = figures
            .iter()
            .map(|parts| {
                parts
                    .iter()
                    .flat_map(|part| &part.points)
                    .map(|point| point.dot(across))
                    .fold((f32::MAX, f32::MIN), |(low, high), at| {
                        (low.min(at), high.max(at))
                    })
            })
            .collect();
        let mut offsets = Vec::with_capacity(figures.len());
        let mut along = 0.0;
        for (low, high) in &spans {
            offsets.push(along - low);
            along += high - low + GAP;
        }
        let row = along - GAP;
        let offsets: Vec<Vec3> = offsets
            .into_iter()
            .map(|offset| across * (offset - row * 0.5))
            .collect();
        let points: Vec<Vec3> = figures
            .iter()
            .zip(&offsets)
            .flat_map(|(parts, offset)| {
                parts
                    .iter()
                    .flat_map(|part| &part.points)
                    .map(move |point| *point + *offset)
            })
            .collect();
        // Each part where its figure stands, and the blades it lights.
        let built: Vec<(&Built, Vec3)> = figures
            .iter()
            .zip(&offsets)
            .flat_map(|(parts, offset)| parts.iter().map(move |part| (part, *offset + part.local)))
            .collect();
        let seconds = crate::menu::art::motion::seconds();
        let mut blades: Vec<crate::saber::Instance> = Vec::new();
        for (index, (blade, colour)) in figures
            .iter()
            .zip(&offsets)
            .flat_map(|(parts, offset)| {
                parts
                    .iter()
                    .flat_map(move |part| part.blades.iter().map(move |blade| (blade, *offset)))
            })
            .map(|((blade, colour), offset)| {
                let mut blade = *blade;
                blade.base = (Vec3::from_array(blade.base) + offset).to_array();
                (blade, *colour)
            })
            .enumerate()
        {
            blades.extend(
                crate::saber::Instance::pair(blade, colour)
                    .map(|instance| instance.with_animation(seconds, index as u32)),
            );
        }
        let aspect = scene::SIZE[0] as f32 / scene::SIZE[1] as f32;
        let (view_projection, eye, forward) = scene::frame(
            &points,
            front,
            job.scene.yaw,
            job.scene.pitch,
            job.scene.zoom,
            aspect,
        );
        let right = forward.cross(Vec3::Z).normalize_or(Vec3::Y);
        let light = EntityLight {
            ambient: [AMBIENT; 3],
            directed: [DIRECTED; 3],
            direction: (-forward * 0.55 - right * 0.55 + Vec3::Z * 0.65)
                .normalize()
                .to_array(),
        };
        let instances: Vec<wgpu::Buffer> = built
            .iter()
            .map(|(part, at)| {
                let mut instance =
                    ActorInstance::new(at.to_array(), part.rotation.to_array(), [1.0; 3]);
                instance.set_light(light);
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("SJK card scene instance"),
                        contents: bytemuck::bytes_of(&instance),
                        usage: wgpu::BufferUsages::VERTEX,
                    })
            })
            .collect();
        if self.card_scenes.target.is_none() {
            let encode = self.card_scenes.encode.as_ref().ok_or("no encode")?;
            self.card_scenes.target = Some(preview::Target::new(
                &self.device,
                encode,
                &self.camera_layout,
                scene::SIZE,
            ));
        }
        let (Some(encode), Some(target)) = (
            self.card_scenes.encode.as_ref(),
            self.card_scenes.target.as_ref(),
        ) else {
            return Err("no target".into());
        };
        let uniform = CameraUniform {
            view_projection: view_projection.to_cols_array_2d(),
            camera_position: eye.to_array(),
            shader_time: crate::menu::art::motion::seconds() as f32,
            view_forward: forward.to_array(),
            _padding: preview::VIEW_FLAGS,
        };
        self.queue
            .write_buffer(&target.camera, 0, bytemuck::bytes_of(&uniform));
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("SJK card scene"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("SJK card scene"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.color,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &target.depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            for blended in [false, true] {
                for ((part, _), instance) in built.iter().zip(&instances) {
                    self.world_materials.draw_detached(
                        &mut pass,
                        &target.camera_bind,
                        &part.geometry_binding,
                        &part.vertex_buffer,
                        &part.index_buffer,
                        instance,
                        &part.materials,
                        &part.draws,
                        blended,
                    );
                }
            }
        }
        // The blades, hidden behind the bodies as their depth says; the layer is cleared
        // either way, since the encode adds what it holds.
        if !blades.is_empty() {
            self.saber_gpu.prepare_preview(&self.queue, &mut blades);
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("SJK card scene blades"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.blades,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &target.depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if !blades.is_empty() {
                self.saber_gpu.draw_preview(&mut pass, &target.camera_bind);
            }
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("SJK card scene encode"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.display,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&encode.pipeline);
            pass.set_bind_group(0, &target.encode_bind, &[]);
            pass.draw(0..3, 0..1);
        }
        let [width, height] = scene::SIZE;
        let padded_row_bytes = (width * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("SJK card scene readback"),
            size: u64::from(padded_row_bytes) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_texture_to_buffer(
            target.display.texture().as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row_bytes),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([encoder.finish()]);
        if !blades.is_empty() {
            self.restore_preview_blades();
        }
        Ok((buffer, padded_row_bytes))
    }

    /// Build one figure as the menu stage builds its actor (a player model, posed, with
    /// its hilts) or its prop (a rigid model): its parts, the body first.
    fn build_card_figure(
        &mut self,
        vfs: &VirtualFileSystem,
        figure: &Figure,
    ) -> Result<Vec<Built>, Box<dyn Error>> {
        let mut flattened = FlattenedScene::default();
        match figure {
            Figure::Player {
                model,
                anim,
                at,
                sabers,
                blades,
            } => {
                let (directory, variant) = split_model_cvar(model);
                let mut appearance = load_stock_player_appearance(
                    vfs,
                    &directory,
                    variant,
                    &mut self.card_scenes.gla_cache,
                )?;
                appearance.origin = [0.0; 3];
                appearance.yaw = 0.0;
                if let Some(name) = anim {
                    appearance.sequence = appearance
                        .config
                        .get(name)
                        .cloned()
                        .ok_or_else(|| format!("{model} has no animation {name}"))?;
                }
                let sequence = &appearance.sequence;
                let step = (sequence.frame_count.saturating_sub(1) as f32 * at).round() as usize;
                let frame = sequence.first_frame + step;
                let (draws, _) = append_actor_mesh(&mut flattened, &appearance, frame)?;
                // The model faces its yaw once turned as the game turns a Ghoul2 model.
                let rotation = weapon_view::actor_world_rotation(Quat::IDENTITY.to_array());
                let front =
                    rotation * Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2) * Vec3::X;
                let mut parts = vec![self.card_part(
                    vfs,
                    &flattened,
                    draws,
                    rotation,
                    Vec3::ZERO,
                    front,
                    Vec::new(),
                )?];
                if sabers.iter().any(Option::is_some) {
                    let matrices = appearance.animation.frame_matrices(frame)?;
                    for (hand, bolt) in [crate::saber::HAND_BOLT, crate::saber::LEFT_HAND_BOLT]
                        .into_iter()
                        .enumerate()
                    {
                        let Some(name) = &sabers[hand] else {
                            continue;
                        };
                        let attachment =
                            crate::saber::attachment_from_matrices(&appearance, &matrices, bolt)
                                .ok_or_else(|| format!("{model} has no {bolt}"))?;
                        let (grip, held) =
                            crate::saber::world_attachment(Vec3::ZERO, rotation, attachment);
                        let colour = crate::saber::Color::ALL
                            .get(blades[hand])
                            .copied()
                            .unwrap_or(crate::saber::Color::Blue);
                        parts.push(self.card_hilt(
                            vfs,
                            name,
                            grip,
                            held,
                            front,
                            BladeColor::Retail(colour),
                        )?);
                    }
                }
                Ok(parts)
            }
            Figure::Model { path } => {
                let asset = vfs
                    .read(path)?
                    .ok_or_else(|| format!("{path} is missing"))?;
                let draws = if path.ends_with(".glm") {
                    append_static_glm_mesh(&mut flattened, &Glm::parse(&asset.bytes)?)?
                } else {
                    append_md3_mesh(&mut flattened, &Md3::parse(&asset.bytes)?)?
                };
                Ok(vec![self.card_part(
                    vfs,
                    &flattened,
                    draws,
                    Quat::IDENTITY,
                    Vec3::ZERO,
                    Vec3::X,
                    Vec::new(),
                )?])
            }
        }
    }

    /// The hilt `name` (a `.sab` definition) held at `grip`, turned `held`, its blades lit
    /// in `colour`, as the menu stage builds a hilt.
    fn card_hilt(
        &mut self,
        vfs: &VirtualFileSystem,
        name: &str,
        grip: Vec3,
        held: Quat,
        front: Vec3,
        colour: BladeColor,
    ) -> Result<Built, Box<dyn Error>> {
        if self.card_scenes.saber_definitions.is_none() {
            self.card_scenes.saber_definitions = Some(crate::saber_defs::load(vfs)?);
        }
        let definition = self
            .card_scenes
            .saber_definitions
            .as_ref()
            .and_then(|definitions| definitions.get(name))
            .ok_or_else(|| format!("no saber named {name}"))?
            .clone();
        let asset = vfs
            .read(&definition.model)?
            .ok_or_else(|| format!("hilt model {} is missing", definition.model))?;
        let model = Glm::parse(&asset.bytes)?;
        let sockets = crate::saber_hilts::hilt_sockets(&model)?;
        let blades = crate::saber_hilts::hilt_blades(&definition, &sockets)
            .into_iter()
            .take(usize::from(definition.num_blades))
            .flatten()
            .map(|blade| {
                let lit =
                    crate::saber::world_blade(grip, held, blade.socket, blade.length, blade.radius);
                (lit, colour)
            })
            .collect();
        let mut flattened = FlattenedScene::default();
        let draws = append_static_glm_mesh(&mut flattened, &model)?;
        self.card_part(vfs, &flattened, draws, held, grip, front, blades)
    }

    /// Upload one part's mesh and compile its materials.
    #[allow(clippy::too_many_arguments)]
    fn card_part(
        &mut self,
        vfs: &VirtualFileSystem,
        flattened: &FlattenedScene,
        draws: Vec<ActorDraw>,
        rotation: Quat,
        local: Vec3,
        front: Vec3,
        blades: Vec<(crate::saber::Blade, BladeColor)>,
    ) -> Result<Built, Box<dyn Error>> {
        if draws.is_empty() || flattened.vertices.is_empty() {
            return Err("nothing to draw".into());
        }
        let materials = self.world_materials.compile_detached(
            &self.device,
            &self.queue,
            vfs,
            &self.shaders,
            &flattened.materials,
        )?;
        let draws = draws
            .into_iter()
            .map(|draw| DetachedDraw {
                indices: draw.indices,
                material: draw.material,
            })
            .collect();
        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("SJK card scene vertices"),
                contents: bytemuck::cast_slice(&flattened.vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
            });
        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("SJK card scene indices"),
                contents: bytemuck::cast_slice(&flattened.indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        let quads = crate::shared_geometry::quads::upload(
            &self.device,
            flattened.vertices.len(),
            &flattened.indices,
            0,
        );
        let geometry_binding =
            crate::shared_geometry::quads::bind(&self.device, &vertex_buffer, &quads);
        // The camera frames the blades' tips too.
        let tips = blades
            .iter()
            .flat_map(|(blade, _): &(crate::saber::Blade, BladeColor)| {
                let base = Vec3::from_array(blade.base);
                [
                    base,
                    base + Vec3::from_array(blade.direction) * blade.length,
                ]
            });
        let points = flattened
            .vertices
            .iter()
            .map(|vertex| local + rotation * Vec3::from_array(vertex.position))
            .chain(tips)
            .collect();
        Ok(Built {
            draws,
            materials,
            vertex_buffer,
            geometry_binding,
            index_buffer,
            points,
            rotation,
            local,
            front,
            blades,
        })
    }

    /// Take the drawing under way once read back: a thread lays it over the backdrop and
    /// keeps it, then tells the faces.
    fn poll_card_scene(&mut self) {
        let Some(reading) = &self.card_scenes.reading else {
            return;
        };
        let _ = self.device.poll(wgpu::PollType::Poll);
        let mapped = match reading.receiver.try_recv() {
            Err(TryRecvError::Empty) => return,
            Ok(Ok(())) => true,
            Ok(Err(_)) | Err(TryRecvError::Disconnected) => false,
        };
        let Some(reading) = self.card_scenes.reading.take() else {
            return;
        };
        let pixels = mapped.then(|| {
            let [width, height] = scene::SIZE;
            let row_bytes = (width * 4) as usize;
            let slice = reading.buffer.slice(..);
            let view = slice.get_mapped_range().ok()?;
            let mut pixels = vec![0; row_bytes * height as usize];
            for (source, destination) in view
                .chunks(reading.padded_row_bytes as usize)
                .zip(pixels.chunks_mut(row_bytes))
            {
                destination.copy_from_slice(&source[..row_bytes]);
            }
            drop(view);
            reading.buffer.unmap();
            Some(pixels)
        });
        let job = reading.job;
        let failed = job.clone();
        let Some(pixels) = pixels.flatten() else {
            crate::log::progress(format_args!(
                "cards: {}: the scene's picture could not be read back",
                job.id
            ));
            scene::finished(&job, false);
            return;
        };
        let keep = move || {
            let mut pixels = pixels;
            scene::over_backdrop(&mut pixels, scene::SIZE, job.tint);
            let kept = scene::keep(&job.path, scene::SIZE, &pixels);
            if let Err(error) = &kept {
                crate::log::progress(format_args!(
                    "cards: {}: the scene's picture was not kept: {error}",
                    job.id
                ));
            }
            scene::finished(&job, kept.is_ok());
        };
        #[cfg(test)]
        if crate::cards::faces::inline_for_shots() {
            keep();
            return;
        }
        if let Err(error) = std::thread::Builder::new()
            .name("sjk-card-scene".into())
            .spawn(keep)
        {
            crate::log::progress(format_args!(
                "cards: {}: the scene's picture was not kept: {error}",
                failed.id
            ));
            scene::finished(&failed, false);
        }
    }
}
