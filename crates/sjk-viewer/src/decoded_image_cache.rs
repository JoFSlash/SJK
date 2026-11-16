//! Process-lifetime decoded image cache shared by map and shell textures.
//!
//! One decoded image per file name and content, so two maps using the same file get the
//! same `Arc` (the shared map textures and the CPU mip cache key on its address and hold
//! it while they do). Until 25/10/2026 nothing ever left the cache: every texture of
//! every map, model and HUD seen in a session stayed decoded at full size. Now, once the
//! images held grow [`SPARE_BYTES`] past what the last sweep kept, the next image decoded
//! sweeps out every image only the cache still holds ([`Images::sweep`]): an image a
//! world's textures, a key or a loader still holds stays, so an address in use as a key
//! is never freed and reused.

use image::RgbaImage;
use md4::{Digest, Md4};
use sjk_vfs::{AssetCacheIdentity, VirtualFileSystem, VirtualPath};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

type ImageKey = (String, [u8; 16]);
type ResolvedKey = (AssetCacheIdentity, String);

/// Decoded bytes the cache keeps beyond what the last sweep found in use.
const SPARE_BYTES: usize = 128 << 20;

/// The cache's two indexes of the same images.
#[derive(Default)]
struct Images {
    /// One image per file name and content.
    decoded: HashMap<ImageKey, Arc<RgbaImage>>,
    /// The image a file system generation's path resolved to.
    resolved: HashMap<ResolvedKey, Arc<RgbaImage>>,
    /// Bytes of the images in `decoded`.
    bytes: usize,
    /// Held bytes past which the next new image sweeps.
    sweep_at: usize,
}

impl Images {
    fn new() -> Self {
        Self {
            decoded: HashMap::with_capacity(1_024),
            resolved: HashMap::with_capacity(1_024),
            bytes: 0,
            sweep_at: SPARE_BYTES,
        }
    }

    /// Keep `image`, decoded from the file under `key`; another thread may have kept the
    /// same one first, which is then returned instead.
    fn keep(&mut self, key: ImageKey, image: Arc<RgbaImage>) -> Arc<RgbaImage> {
        if let Some(kept) = self.decoded.get(&key) {
            return Arc::clone(kept);
        }
        self.bytes += image.as_raw().len();
        self.decoded.insert(key, Arc::clone(&image));
        if self.bytes > self.sweep_at {
            self.sweep();
        }
        image
    }

    /// Drop every image nothing but these indexes holds, and sweep again once
    /// [`SPARE_BYTES`] more are held.
    fn sweep(&mut self) {
        let mut held = HashMap::<*const RgbaImage, usize>::new();
        for image in self.decoded.values().chain(self.resolved.values()) {
            *held.entry(Arc::as_ptr(image)).or_default() += 1;
        }
        let used = |image: &Arc<RgbaImage>| {
            Arc::strong_count(image) > held.get(&Arc::as_ptr(image)).copied().unwrap_or(0)
        };
        self.resolved.retain(|_, image| used(image));
        self.decoded.retain(|_, image| used(image));
        self.bytes = self
            .decoded
            .values()
            .map(|image| image.as_raw().len())
            .sum();
        self.sweep_at = self.bytes + SPARE_BYTES;
    }
}

static IMAGES: OnceLock<Mutex<Images>> = OnceLock::new();

fn images() -> MutexGuard<'static, Images> {
    IMAGES
        .get_or_init(|| Mutex::new(Images::new()))
        .lock()
        .expect("decoded image cache lock")
}

/// Decode one image through the cache used by all viewer material paths.
pub(crate) fn cached_decoded_image(
    vfs: &VirtualFileSystem,
    path: &str,
) -> Result<Option<Arc<RgbaImage>>, Box<dyn Error>> {
    // A leading slash is skipped as the file system's reads do (`VirtualPath::for_read`).
    let normalized = VirtualPath::for_read(path)?.as_str().to_owned();
    let resolved = vfs
        .asset_cache_identity(&normalized)?
        .map(|id| (id, normalized.clone()));

    if let Some(key) = &resolved
        && let Some(image) = images().resolved.get(key).cloned()
    {
        return Ok(Some(image));
    }
    let image = read_decoded_image(vfs, &normalized)?;
    if let (Some(key), Some(image)) = (resolved, &image) {
        images().resolved.insert(key, Arc::clone(image));
    }
    Ok(image)
}

/// Bytes of decoded images the cache holds now, for measurements.
#[cfg(test)]
pub(crate) fn held_bytes() -> usize {
    images().bytes
}

fn read_decoded_image(
    vfs: &VirtualFileSystem,
    path: &str,
) -> Result<Option<Arc<RgbaImage>>, Box<dyn Error>> {
    let Some(asset) = vfs.read(path)? else {
        return Ok(None);
    };

    // A virtual path alone aliases different server packs. Hash only while loading assets;
    // decoded images and GPU resources are still retained normally during rendering.
    let key = (path.to_ascii_lowercase(), Md4::digest(&asset.bytes).into());
    if let Some(image) = images().decoded.get(&key).cloned() {
        return Ok(Some(image));
    }
    let Ok(decoded) = crate::decode_image(&asset.bytes, path) else {
        return Ok(None);
    };
    let decoded = Arc::new(decoded.into_rgba8());
    Ok(Some(images().keep(key, decoded)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(name: &str) -> ImageKey {
        (name.to_owned(), [0; 16])
    }

    #[test]
    fn a_sweep_drops_only_the_images_nothing_else_holds() {
        let mut images = Images::new();
        let held = images.keep(key("held"), Arc::new(RgbaImage::new(64, 64)));
        drop(images.keep(key("dropped"), Arc::new(RgbaImage::new(64, 64))));
        let resolved = images.keep(key("resolved"), Arc::new(RgbaImage::new(32, 32)));
        let mut vfs = VirtualFileSystem::new();
        vfs.mount_memory("synthetic", [("resolved.tga", vec![0_u8; 4])])
            .expect("mounted");
        let identity = vfs
            .asset_cache_identity("resolved.tga")
            .expect("an identity")
            .expect("the file");
        let alias = (identity, "resolved".to_owned());
        images.resolved.insert(alias.clone(), Arc::clone(&resolved));
        drop(resolved);
        assert_eq!(images.bytes, 2 * 64 * 64 * 4 + 32 * 32 * 4);
        images.sweep();
        // Only the image a caller still holds is left, in both indexes.
        assert!(images.decoded.contains_key(&key("held")));
        assert!(!images.decoded.contains_key(&key("dropped")));
        assert!(!images.decoded.contains_key(&key("resolved")));
        assert!(!images.resolved.contains_key(&alias));
        assert_eq!(images.bytes, 64 * 64 * 4);
        assert_eq!(images.sweep_at, images.bytes + SPARE_BYTES);
        // The same file again is the same image while it is held.
        let again = images.keep(key("held"), Arc::new(RgbaImage::new(64, 64)));
        assert!(Arc::ptr_eq(&held, &again));
    }

    #[test]
    fn images_past_the_spare_bytes_sweep_themselves() {
        let mut images = Images::new();
        images.sweep_at = 64 * 64 * 4;
        drop(images.keep(key("first"), Arc::new(RgbaImage::new(64, 64))));
        assert_eq!(images.decoded.len(), 1, "at the mark, nothing sweeps yet");
        let second = images.keep(key("second"), Arc::new(RgbaImage::new(64, 64)));
        // The second image passed the mark: the first, held by nothing, went.
        assert_eq!(images.decoded.len(), 1);
        assert!(images.decoded.contains_key(&key("second")));
        drop(second);
    }
}
