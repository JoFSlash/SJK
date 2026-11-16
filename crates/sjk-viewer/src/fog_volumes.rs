//! Map-lifetime rd-vanilla fog parameters and entity assignment.
//!
//! `R_LoadFogs` (codemp/rd-vanilla/tr_bsp.cpp:1658-1795) is evaluated once.
//! Entity overlap follows tr_mesh.cpp:256-285 / tr_ghoul2.cpp:864-890. The viewer's
//! retained actor/rigid mesh records have no bounding radius, so assignment uses
//! the instance origin (radius zero). The overlap helper also accepts a radius
//! for future retained model bounds. The strict boundary comparisons are preserved.
//! Global fog uses GL_EXP2 by default (tr_shade.cpp:1565,1574-1610,1879),
//! while brush volumes retain RB_FogPass. The global marker occupies color.w.

use bytemuck::{Pod, Zeroable};
use sjk_bsp::{Bsp, FogVolume};
use sjk_shader::ShaderCatalog;

/// Five fog-index bits in rd-vanilla's draw sort key (tr_main.cpp:1126).
/// Slot zero is reserved for no fog; this is a renderer compatibility table.
pub(crate) const FOG_SLOTS: usize = 32;

/// Retail `r_drawfog` mapping (tr_init.cpp:1645, tr_shade.cpp:1574-1575,1879).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Mode {
    /// Skip every fog draw.
    Off,
    /// Use the volume square-root curve for every fog, including global fog.
    Volume,
    /// Use EXP2 for global distance fog and the volume curve for brush fogs.
    #[default]
    GlobalExp2,
}

impl Mode {
    /// Only exactly 2 selects EXP2; other nonzero values use the volume pass.
    pub(crate) fn from_cvar(value: i64) -> Self {
        match value {
            0 => Self::Off,
            2 => Self::GlobalExp2,
            _ => Self::Volume,
        }
    }
}

/// Uniform entry; colour uses identityLight == 1 (see `entity_lighting`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct GpuFog {
    /// rgb = fog colour; w = global fog marker, not alpha.
    pub(crate) color: [f32; 4],
    pub(crate) surface: [f32; 4],
    /// xyz bounds; w = 1 / (max(depthForOpaque, 1) * 8).
    pub(crate) bounds_min: [f32; 4],
    /// xyz bounds; w = hasSurface, including global and visibleSide == -1 fog.
    pub(crate) bounds_max: [f32; 4],
}

/// Fixed CPU copy for allocation-free per-instance overlap tests.
pub(crate) struct Table {
    pub(crate) entries: [GpuFog; FOG_SLOTS],
    pub(crate) count: usize,
}

impl Table {
    /// Resolve every fog's brush and shader once; reject unrepresentable sort indices.
    pub(crate) fn build(bsp: &Bsp, shaders: &ShaderCatalog) -> Result<Self, &'static str> {
        if bsp.fogs().len() >= FOG_SLOTS {
            return Err("more than 31 BSP fog volumes");
        }
        let mut result = Self {
            entries: [GpuFog::zeroed(); FOG_SLOTS],
            count: bsp.fogs().len(),
        };
        for (index, fog) in bsp.fogs().iter().enumerate() {
            let volume = FogVolume::derive(fog, bsp);
            let parms = shaders
                .get(&fog.shader)
                .and_then(|shader| shader.fog)
                .unwrap_or(sjk_shader::FogParms {
                    color: [1.0, 0.0, 0.0],
                    depth_for_opaque: 250.0,
                });
            result.entries[index + 1] = GpuFog::new(volume, parms, fog.brush.is_none());
            if index >= bsp.root_fog_count() {
                result.entries[index + 1].bounds_max[3] = 2.0;
            }
        }
        Ok(result)
    }

    /// The map's global fog colour, unquantized: `R_LoadFogs` keeps the last
    /// `brushNum == -1` fog as `globalFog` (tr_bsp.cpp:1717-1722), and the frame and
    /// every world view clear to its `parms.color` (tr_backend.cpp:547-553,1812-1818),
    /// so a sky with no box or clouds (`skyParms - 512 -`, `taspir1`) shows the fog.
    pub(crate) fn global_color(&self) -> Option<[f32; 3]> {
        self.entries[1..=self.count]
            .iter()
            .rev()
            .find(|fog| fog.color[3] != 0.0 && fog.bounds_max[3] < 2.0)
            .map(|fog| [fog.color[0], fog.color[1], fog.color[2]])
    }

    /// First strictly overlapping volume, preserving BSP order, including global fog.
    pub(crate) fn at_sphere(&self, origin: [f32; 3], radius: f32) -> u32 {
        self.entries[1..=self.count]
            .iter()
            .position(|fog| {
                fog.bounds_max[3] < 2.0
                    && (0..3).all(|axis| {
                        origin[axis] - radius < fog.bounds_max[axis]
                            && origin[axis] + radius > fog.bounds_min[axis]
                    })
            })
            .map_or(0, |index| index as u32 + 1)
    }
}

impl GpuFog {
    fn new(volume: FogVolume, parms: sjk_shader::FogParms, global: bool) -> Self {
        let [min, max] = volume.bounds;
        // ColorBytes4 (tr_bsp.cpp:1772-1775) truncates to unsigned bytes.
        // GL_EXP2 uses parms.color directly (tr_shade.cpp:1618); mode 1 quantizes
        // global colour in the fragment. Brush colours retain their original bytes.
        let color = if global {
            parms.color
        } else {
            parms
                .color
                .map(|value| (value * 255.0) as u8 as f32 / 255.0)
        };
        Self {
            color: [color[0], color[1], color[2], if global { 1.0 } else { 0.0 }],
            surface: volume.surface.unwrap_or([0.0; 4]),
            bounds_min: [
                min[0],
                min[1],
                min[2],
                1.0 / (parms.depth_for_opaque.max(1.0) * 8.0),
            ],
            bounds_max: [max[0], max[1], max[2], 1.0],
        }
    }
}

/// Fog and colour shaders compile the identical projection and instance transforms.
pub(crate) const SHADER: &str = concat!(
    include_str!("vertex_transform.wgsl"),
    include_str!("gpu_skinning.wgsl"),
    include_str!("geometry_stage.wgsl"),
    include_str!("fog_pass.wgsl"),
    include_str!("surface_deform.wgsl"),
    include_str!("surface_tables.wgsl"),
    include_str!("surface_noise.wgsl"),
    include_str!("surface_sprites.wgsl"),
);

#[cfg(test)]
mod tests {
    use super::*;

    fn table(fogs: &[([f32; 3], bool, bool)]) -> Table {
        let mut table = Table {
            entries: [GpuFog::zeroed(); FOG_SLOTS],
            count: fogs.len(),
        };
        for (index, &(color, global, submodel)) in fogs.iter().enumerate() {
            let volume = FogVolume {
                bounds: [[-1.0; 3], [1.0; 3]],
                surface: None,
            };
            let parms = sjk_shader::FogParms {
                color,
                depth_for_opaque: 4096.0,
            };
            table.entries[index + 1] = GpuFog::new(volume, parms, global);
            if submodel {
                table.entries[index + 1].bounds_max[3] = 2.0;
            }
        }
        table
    }

    #[test]
    fn the_last_global_fog_of_the_world_is_the_clear_colour() {
        let lava = [1.0, 0.2, 0.0];
        let taspir1 = [0.8, 0.7, 0.4];
        assert_eq!(table(&[]).global_color(), None);
        assert_eq!(table(&[(lava, false, false)]).global_color(), None);
        // Unquantized, unlike the brush fogs' ColorBytes4 colours.
        assert_eq!(
            table(&[(lava, false, false), (taspir1, true, false)]).global_color(),
            Some(taspir1)
        );
        assert_eq!(
            table(&[(lava, true, false), (taspir1, true, false)]).global_color(),
            Some(taspir1)
        );
        // An appended asset's fogs stay in model space and never replace the world's.
        assert_eq!(
            table(&[(taspir1, true, false), (lava, true, true)]).global_color(),
            Some(taspir1)
        );
    }
}
