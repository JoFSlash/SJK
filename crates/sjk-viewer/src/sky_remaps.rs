//! Retarget authored sky images without replacing day/night resources.
use super::*;
impl Runtime {
    pub(crate) fn refresh_remaps(
        &mut self,
        device: &wgpu::Device,
        queue: &crate::frame_queue::FrameQueue,
        vfs: &VirtualFileSystem,
        shaders: &ShaderCatalog,
        remaps: Option<&sjk_client::ShaderRemapTable>,
        map: &crate::world_materials::map_remaps::MapRemaps,
    ) -> Result<(), Box<dyn Error>> {
        for material in &mut self.materials {
            let Some(name) = sjk_client::shader_name(&material.name) else {
                continue;
            };
            if !map.affects(&name, remaps) && material.remapped.is_none() {
                continue;
            }
            let (target, _) = crate::world_materials::remap_target(map, remaps, &name);
            if material.remapped.as_deref() == Some(target) {
                continue;
            }
            let Some(definition) = shaders.get(target).filter(|d| d.sky.is_some()) else {
                continue;
            };
            let (bind_group, vertex_buffer, vertex_count, missing_faces) = load_sky(
                device,
                queue,
                &gpu::texture_layout(device),
                vfs,
                shaders,
                definition,
            )?;
            material.bind_group = bind_group;
            material.vertex_buffer = vertex_buffer;
            material.vertex_count = vertex_count;
            material.missing_faces = missing_faces;
            material.remapped = Some(target.to_owned());
        }
        Ok(())
    }
}
