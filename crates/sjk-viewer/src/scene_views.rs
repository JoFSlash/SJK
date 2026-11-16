//! Map-authored mirrors, camera portals and sky portals. Offscreen targets
//! are reused; no recursive views or per-frame geometry/resource creation.
use crate::{
    ActorInstance, Bsp, CameraUniform, GpuState, GpuVertex, scene_flatten::FlattenedScene,
};
use glam::{Mat4, Quat, Vec3};
use sjk_protocol::{EntityState, GameState, Snapshot};
use sjk_shader::ShaderCatalog;
use std::ops::Range;

#[path = "floor_reflections.rs"]
mod floor_reflections;
#[path = "scene_view_gpu.rs"]
mod gpu;
#[path = "scene_portal_math.rs"]
mod math;
#[path = "reflection_capture.rs"]
mod reflection_capture;
#[path = "scene_view_render.rs"]
mod render;
#[path = "scene_sky_visibility.rs"]
mod sky_visibility;

/// codemp `ET_PORTAL` (bg_public.h `entityType_t`): what `SP_misc_portal_surface` sends.
/// Until 19/10/2026 this was 7 (`ET_BEAM`), so no camera portal ever found its
/// marker and portal faces showed only their own texture.
const ET_PORTAL: u8 = 8;

/// Whether `entity` is a `misc_portal_surface` marker a portal face can match.
fn is_portal_marker(entity: &EntityState) -> bool {
    entity.entity_type() == ET_PORTAL
}

struct Face {
    indices: Range<u32>,
    model: Option<usize>,
    clusters: Vec<usize>,
    normal: Vec3,
    distance: f32,
    center: Vec3,
    radius: f32,
    range: f32,
    candidates: Vec<usize>,
}

struct Selection {
    face: usize,
    instance: ActorInstance,
    view: math::View,
}

/// Map-owned secondary views, retained GPU resources and conservative scheduling policy.
pub(crate) struct Runtime {
    floors: floor_reflections::Floors,
    faces: Vec<Face>,
    portals: Vec<math::Portal>,
    portal_enabled: bool,
    portal_areas: crate::world_materials::areas::Areas,
    portal_cluster: Option<usize>,
    model_owners: Vec<Option<u16>>,
    signature: u64,
    selection: Option<Selection>,
    sky_view: Option<math::View>,
    sky_visibility: sky_visibility::SkyVisibility,

    sky_orientation: Option<(Vec3, f32)>,
    offline_sky: Option<Vec3>,
    pub(crate) sky_only_fog: bool,
    needs_environment: bool,
    /// Made the first frame a portal face selects a view ([`Runtime::provide_targets`]).
    portal_target: gpu::Lazy,
    /// Made the first frame the sky portal is drawn: a map's sky shaders alone, without the
    /// server's `CS_SKYBOXORG` (a `misc_skyportal`), never draw it.
    sky_target: gpu::Lazy,
    pipeline: wgpu::RenderPipeline,
    /// The sky portal's composite on the sky faces, pushed behind them like the sky.
    sky_pipeline: wgpu::RenderPipeline,
    camera_layout: wgpu::BindGroupLayout,
    sample_layout: wgpu::BindGroupLayout,
    format: wgpu::TextureFormat,
}

/// Register the floor mirrors' `r_floorReflections`.
pub(crate) fn register_floor_reflections(
    cvars: &mut sjk_shell::CvarRegistry,
) -> Result<(), sjk_shell::CvarError> {
    floor_reflections::register(cvars)
}

impl Runtime {
    /// Bind retained mirror suppression resources once the map renderer exists.
    pub(crate) fn configure_floor_commands(
        &mut self,
        device: &wgpu::Device,
        arguments: Option<&wgpu::Buffer>,
    ) {
        self.floors.configure_commands(device, arguments);
    }

    /// Whether a visible map surface selected a nonrecursive remote or mirror camera.
    pub(crate) fn has_portal_view(&self) -> bool {
        self.selection.is_some()
    }
    /// Resolve map-authored view metadata; the views' targets are made when first drawn.
    pub(crate) fn new(
        device: &wgpu::Device,
        camera_layout: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
        scene: &FlattenedScene,
        bsp: &Bsp,
        shaders: &ShaderCatalog,
        floor_maps: &dyn Fn(usize) -> Option<crate::world_materials::material_maps::FloorMaps>,
    ) -> Self {
        let mut faces = Vec::new();
        for draw in &scene.draws {
            let Some(def) = shaders.get(&scene.materials[draw.material].shader) else {
                continue;
            };
            if def.resolved_sort() != 1.0 {
                continue;
            }
            let Some(surface) = draw.surface_index.map(|i| &bsp.render().surfaces()[i]) else {
                continue;
            };
            let points = &bsp.render().vertices()[surface.vertices.clone()];
            if points.is_empty() {
                continue;
            }
            let normal = Vec3::from_array(surface.lightmap_vectors[2]).normalize_or_zero();
            if normal.length_squared() < 0.5 {
                continue;
            }
            let center = points
                .iter()
                .fold(Vec3::ZERO, |sum, v| sum + Vec3::from_array(v.position))
                / points.len() as f32;
            let radius = points
                .iter()
                .map(|v| Vec3::from_array(v.position).distance(center))
                .fold(0.0, f32::max);
            let model = (!draw.world_surface)
                .then(|| {
                    bsp.render()
                        .models()
                        .iter()
                        .position(|m| m.surfaces.contains(&draw.surface_index.unwrap()))
                })
                .flatten();
            faces.push(Face {
                indices: draw.indices.clone(),
                model,
                clusters: draw.clusters.clone(),
                normal,
                distance: normal.dot(Vec3::from_array(points[0].position)),
                center,
                radius,
                range: def.portal_range.unwrap_or(256.0),
                candidates: Vec::with_capacity(sjk_protocol::MAX_LEGACY_ENTITIES),
            });
        }
        let entities = sjk_entity::parse_entity_lump(bsp.entities()).unwrap_or_default();
        let offline_sky = entities
            .iter()
            .rev()
            .find(|e| e.classname() == Some("misc_skyportal"))
            .and_then(|e| e.vector("origin").ok().flatten())
            .map(Vec3::from_array);
        let sky_orientation = entities
            .iter()
            .rev()
            .find(|e| e.classname() == Some("misc_skyportal_orient"))
            .and_then(|e| {
                Some((
                    Vec3::from_array(e.vector("origin").ok()??),
                    e.number("modelscale").ok()?.unwrap_or(0.0),
                ))
            });
        let sky_only_fog = entities
            .iter()
            .any(|e| e.classname() == Some("misc_skyportal") && e.get("onlyfoghere") == Some("1"));
        let sample_layout = gpu::sample_layout(device);
        let pipeline = gpu::pipeline(device, camera_layout, &sample_layout, format, "portal_main");
        let sky_pipeline = gpu::pipeline(
            device,
            camera_layout,
            &sample_layout,
            format,
            "sky_portal_main",
        );
        let sky_visibility = sky_visibility::SkyVisibility::new(scene, bsp, shaders);
        let portal_enabled = std::env::var_os("SJK_MAP_PORTALS").is_none_or(|v| v != "0");
        let portals = Vec::with_capacity(if faces.is_empty() {
            0
        } else {
            sjk_protocol::MAX_LEGACY_ENTITIES
        });
        Self {
            floors: floor_reflections::Floors::new(
                device,
                camera_layout,
                format,
                scene,
                shaders,
                floor_maps,
            ),
            faces,
            portals,
            portal_enabled,
            portal_areas: crate::world_materials::areas::Areas::new(bsp),
            portal_cluster: None,
            model_owners: vec![None; bsp.render().models().len()],
            signature: u64::MAX,
            selection: None,
            sky_view: None,
            sky_visibility,

            sky_orientation,
            offline_sky,
            sky_only_fog,
            portal_target: gpu::Lazy::new(gpu::MAP_PORTAL),
            sky_target: gpu::Lazy::new(gpu::SKY_PORTAL),
            pipeline,
            sky_pipeline,
            needs_environment: scene.materials.iter().any(|m| {
                m.shader.starts_with("@sjk-surface-sprites/") || m.shader.starts_with("@sjk-flare/")
            }),
            camera_layout: camera_layout.clone(),
            sample_layout,
            format,
        }
    }

    /// Refresh plane/entity matches only when portal entity state changes.
    fn cache_portals(&mut self, game: &GameState, snapshot: &Snapshot) {
        if self.faces.is_empty() {
            return;
        }
        use std::hash::{Hash, Hasher};
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        for e in sjk_client::legacy_scene_entities(game, snapshot).filter(|e| is_portal_marker(e)) {
            e.number().hash(&mut hash);
            for field in [e.trajectory_base(), e.origin2()] {
                field.map(f32::to_bits).hash(&mut hash);
            }
            e.event_parameter().hash(&mut hash);
            e.powerups().hash(&mut hash);
            e.client_num().hash(&mut hash);
            e.integer_field(83).hash(&mut hash);
        }
        let signature = hash.finish();
        if self.signature == signature {
            return;
        }
        self.signature = signature;
        self.portals.clear();
        self.portals.extend(
            sjk_client::legacy_scene_entities(game, snapshot)
                .filter(|e| is_portal_marker(e))
                .map(math::Portal::from_entity),
        );
        for face in &mut self.faces {
            face.candidates.clear();
            for (index, e) in self.portals.iter().enumerate() {
                // Inline surfaces are checked with their live translated plane.
                if face.model.is_some() || (face.normal.dot(e.origin) - face.distance).abs() <= 64.0
                {
                    face.candidates.push(index);
                }
            }
        }
    }

    fn select(
        &mut self,
        bsp: &Bsp,
        game: &GameState,
        snapshot: &Snapshot,
        view: Mat4,
        projection: Mat4,
        time: i32,
        area_mask: &[u8],
    ) {
        if self.portal_enabled {
            self.cache_portals(game, snapshot);
        }
        if !self.faces.is_empty() {
            self.portal_areas.update(area_mask);
        }
        self.selection = None;
        self.sky_view = None;
        let eye = view.inverse().w_axis.truncate();
        let leaf = bsp.leaf_at(eye.to_array());
        let cluster = usize::try_from(bsp.leaves()[leaf].cluster).ok();
        self.portal_cluster = cluster;
        let vp = projection * view;
        if !self.faces.is_empty() {
            self.model_owners.fill(None);
            for state in sjk_client::legacy_scene_entities(game, snapshot)
                .filter(|e| e.solid() == 0x00ff_ffff)
            {
                if let Some(slot) = self.model_owners.get_mut(state.model_index() as usize) {
                    slot.get_or_insert(state.number());
                }
            }
        }
        for (face_index, face) in self.faces.iter().enumerate() {
            if !self.portal_enabled
                || face.candidates.is_empty()
                || !self
                    .portal_areas
                    .visible(&face.clusters, cluster, bsp.render().visibility())
            {
                continue;
            }
            let state = face
                .model
                .and_then(|model| self.model_owners[model])
                .and_then(|number| {
                    snapshot
                        .entities
                        .binary_search_by_key(&number, |e| e.number())
                        .ok()
                        .map(|i| &snapshot.entities[i])
                        .or_else(|| game.baseline(usize::from(number)))
                });
            if face.model.is_some() && state.is_none() {
                continue;
            }
            let instance = state.map_or(
                ActorInstance::new([0.; 3], [0., 0., 0., 1.], [1.; 3]),
                |state| {
                    ActorInstance::new(
                        sjk_client::legacy_evaluate_trajectory(
                            state.trajectory_base(),
                            state.trajectory_delta(),
                            state.trajectory_type(),
                            state.trajectory_time(),
                            state.trajectory_duration(),
                            time,
                        ),
                        sjk_client::legacy_angles_to_quaternion(
                            sjk_client::legacy_evaluate_trajectory_angles(
                                state.angular_trajectory_base(),
                                state.angular_trajectory_delta(),
                                state.angular_trajectory_type(),
                                state.angular_trajectory_time(),
                                state.angular_trajectory_duration(),
                                time,
                            ),
                        ),
                        [1.; 3],
                    )
                },
            );
            let rotation = Quat::from_array(instance.rotation);
            let origin = Vec3::from_array(instance.position);
            let normal = rotation * face.normal;
            let distance = face.distance + normal.dot(origin);
            let center = origin + rotation * face.center;
            if normal.dot(eye) - distance < 0.0 || !sphere_visible(vp, center, face.radius) {
                continue;
            }
            for candidate in &face.candidates {
                let portal = &self.portals[*candidate];
                let Some(remote) = math::portal(normal, distance, portal, view, time) else {
                    continue;
                };
                if !remote.mirror && (eye.distance(center) - face.radius).max(0.0) > face.range {
                    continue;
                }
                self.selection = Some(Selection {
                    face: face_index,
                    instance,
                    view: remote,
                });
                break;
            }
            if self.selection.is_some() {
                break;
            }
        }
        let origin = game
            .config_string(810)
            .and_then(|s| std::str::from_utf8(s).ok())
            .and_then(|text| {
                let mut tokens = text.split_whitespace();
                Some(Vec3::new(
                    tokens.next()?.parse().ok()?,
                    tokens.next()?.parse().ok()?,
                    tokens.next()?.parse().ok()?,
                ))
            });
        self.select_sky(bsp, origin, view, projection, area_mask);
    }

    /// Make the targets of the views this frame draws, at the scene's `size`: each the
    /// first time its view is drawn, the floor mirrors' too. A view whose target the card
    /// has no room for is not drawn.
    fn provide_targets(
        &mut self,
        device: &wgpu::Device,
        queue: &crate::frame_queue::FrameQueue,
        size: [u32; 2],
    ) {
        let (camera, sample, format) = (&self.camera_layout, &self.sample_layout, self.format);
        self.floors
            .provide_target(device, queue, (camera, sample), format, size);
        if self.selection.is_some()
            && self
                .portal_target
                .make(device, camera, sample, format, size)
                .is_none()
        {
            self.selection = None;
        }
        if self.sky_view.is_some()
            && self
                .sky_target
                .make(device, camera, sample, format, size)
                .is_none()
        {
            self.sky_view = None;
        }
    }

    fn select_sky(
        &mut self,
        bsp: &Bsp,
        origin: Option<Vec3>,
        view: Mat4,
        projection: Mat4,
        area_mask: &[u8],
    ) {
        self.sky_view = None;
        let Some(mut origin) = origin else {
            return;
        };
        let parent = view.inverse();
        let eye = parent.w_axis.truncate();
        let cluster = usize::try_from(bsp.leaves()[bsp.leaf_at(eye.to_array())].cluster).ok();
        if !self
            .sky_visibility
            .visible(bsp, area_mask, cluster, projection * view)
        {
            return;
        }
        if let Some((anchor, scale)) = self.sky_orientation {
            origin += (eye - anchor) * scale;
        }
        if !origin.is_finite() {
            return;
        }
        let forward = -parent.z_axis.truncate();
        self.sky_view = Some(math::View {
            matrix: glam::camera::rh::view::look_at_mat4(
                origin,
                origin + forward,
                parent.y_axis.truncate(),
            ),
            eye: origin,
            forward,
            pvs: origin,
            clip_point: Vec3::ZERO,
            clip_normal: Vec3::ZERO,
            mirror: false,
        });
    }
}

fn sphere_visible(matrix: Mat4, center: Vec3, radius: f32) -> bool {
    let p = center.extend(1.0);
    let w = matrix.row(3);
    [
        w + matrix.row(0),
        w - matrix.row(0),
        w + matrix.row(1),
        w - matrix.row(1),
        matrix.row(2),
        w - matrix.row(2),
    ]
    .iter()
    .all(|plane| plane.dot(p) >= -radius * plane.truncate().length())
}

#[cfg(test)]
impl Runtime {
    /// Whether the floor mirrors, the sky portal and the map portal hold a target.
    pub(crate) fn targets(&self) -> [bool; 3] {
        self.target_sizes().map(|size| size.is_some())
    }

    /// The sizes of the floor mirrors', the sky portal's and the map portal's targets.
    pub(crate) fn target_sizes(&self) -> [Option<[u32; 2]>; 3] {
        [&self.floors.target, &self.sky_target, &self.portal_target]
            .map(|target| target.get().map(|target| target.size))
    }

    /// Whether this frame draws floor mirrors, the sky portal and a map portal.
    pub(crate) fn drawn(&self) -> [bool; 3] {
        [
            self.floors.active(),
            self.sky_view.is_some(),
            self.selection.is_some(),
        ]
    }

    /// Each polished plane's largest face, largest first: its centre, its radius and the
    /// plane's normal.
    pub(crate) fn polished_faces(&self) -> Vec<(Vec3, f32, Vec3)> {
        self.floors.largest_faces()
    }

    /// Each portal face's centre, radius and normal.
    pub(crate) fn portal_faces(&self) -> Vec<(Vec3, f32, Vec3)> {
        self.faces
            .iter()
            .map(|face| (face.center, face.radius, face.normal))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjk_protocol::LEGACY_ENTITY_FIELDS;

    fn entity(entity_type: u8) -> EntityState {
        let mut state = EntityState::zero(100, &LEGACY_ENTITY_FIELDS);
        state.set_raw_field(8, u32::from(entity_type));
        state
    }

    #[test]
    fn portal_surface_entity_is_a_portal_marker() {
        assert!(is_portal_marker(&entity(8)));
    }

    #[test]
    fn beam_entity_is_not_a_portal_marker() {
        // ET_BEAM (target_laser), which this file mistook for ET_PORTAL.
        assert!(!is_portal_marker(&entity(7)));
    }
}
