//! External lightmap pages of maps compiled with q3map2 `-external`.
//!
//! Such a BSP stores no lightmap tiles; its surfaces index `maps/<name>/lm_NNNN.tga`
//! pages (often 512x512) shipped beside it, which EJK's `R_FindLightmap` loads by
//! name. A page that is missing or does not decode leaves its slot empty, and the
//! surfaces drawn with it take the fallback lightmap.
use sjk_bsp::{Bsp, Lightmap};
use sjk_vfs::VirtualFileSystem;

/// Fill every empty lightmap slot of `bsp` (read from `map_path`) from its pages.
pub(crate) fn load(vfs: &VirtualFileSystem, map_path: &str, bsp: &mut Bsp) {
    let missing: Vec<usize> = bsp.render().missing_lightmaps().collect();
    if missing.is_empty() {
        return;
    }
    let folder = map_path.strip_suffix(".bsp").unwrap_or(map_path);
    let mut loaded = 0;
    for index in &missing {
        let path = page_path(folder, *index);
        match page(vfs, &path) {
            Ok(Some(lightmap)) => {
                loaded += usize::from(bsp.set_external_lightmap(*index, lightmap))
            }
            Ok(None) => {}
            Err(error) => crate::log::progress(format_args!(
                "external lightmaps: {path} did not decode: {error}"
            )),
        }
    }
    crate::log::progress(format_args!(
        "external lightmaps: loaded {loaded} of {} pages from {folder}/",
        missing.len()
    ));
}

fn page_path(folder: &str, index: usize) -> String {
    format!("{folder}/lm_{index:04}.tga")
}

fn page(
    vfs: &VirtualFileSystem,
    path: &str,
) -> Result<Option<Lightmap>, Box<dyn std::error::Error>> {
    let Some(asset) = vfs.read(path)? else {
        return Ok(None);
    };
    let image = crate::decode_image(&asset.bytes, path)?.into_rgb8();
    Ok(Some(Lightmap {
        width: image.width(),
        height: image.height(),
        rgb: image.into_raw().into_boxed_slice(),
    }))
}

#[cfg(test)]
mod tests {
    #[test]
    fn pages_are_named_like_q3map2_writes_them() {
        assert_eq!(
            super::page_path("maps/baronshed_share", 7),
            "maps/baronshed_share/lm_0007.tga"
        );
    }

    /// A community test map built with `-external`: 57 pages of 512x512, no tiles.
    #[test]
    #[ignore = "reads baronshed_share.pk3 from the game data named by JKA_GAME_DATA"]
    fn an_external_lightmap_map_loads_its_pages() {
        let game = std::path::PathBuf::from(
            std::env::var_os("JKA_GAME_DATA").expect("set JKA_GAME_DATA to the GameData directory"),
        );
        let vfs = crate::assets::mount_game_data(&game).expect("mount the game data");
        let bsp = crate::assets::load_bsp_from_vfs(&vfs, "maps/baronshed_share.bsp")
            .expect("the map parses");
        assert_eq!(bsp.render().lightmap_count(), 57);
        assert_eq!(bsp.render().missing_lightmaps().count(), 0);
        let page = bsp.render().lightmap(50).expect("page 50");
        assert_eq!((page.width, page.height), (512, 512));
    }
}
