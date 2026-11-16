//! Bounded reuse of immutable CPU mip chains across map installs.
//!
//! Keys refer to decoded image identities, not filenames. The image cache checks
//! mounted content before returning those identities; replacements cannot reuse an
//! old chain. Entries retain their source Arcs so pointer addresses cannot be recycled.
//! With `r_textureCompression` an entry holds the encoded blocks of the levels a texture
//! detail keeps instead of the RGBA8 chains ([`prepare_levels`]).
use super::{Levels, compress};
use image::RgbaImage;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

const MAX_BYTES: u64 = 128 << 20;
const MAX_ENTRIES: usize = 1_024;
type Chains = Arc<Vec<Vec<RgbaImage>>>;
/// Target size and image identities, and for blocks the levels left off.
type Key = (u32, u32, Vec<usize>, Option<u32>);

struct Entry {
    _images: Vec<Arc<RgbaImage>>,
    levels: Levels,
    bytes: u64,
    used: u64,
}

#[derive(Default)]
struct Cache {
    entries: HashMap<Key, Entry>,
    bytes: u64,
    clock: u64,
}

static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();

fn cache() -> &'static Mutex<Cache> {
    CACHE.get_or_init(|| Mutex::new(Cache::default()))
}

fn key(images: &[Arc<RgbaImage>], width: u32, height: u32, blocks: Option<u32>) -> Key {
    (
        width,
        height,
        images
            .iter()
            .map(|image| Arc::as_ptr(image) as usize)
            .collect(),
        blocks,
    )
}

/// The entry under `key`, marked used.
fn find(key: &Key) -> Option<Levels> {
    let mut cache = cache().lock().expect("mip cache lock");
    cache.clock = cache.clock.wrapping_add(1);
    let used = cache.clock;
    let entry = cache.entries.get_mut(key)?;
    entry.used = used;
    Some(entry.levels.clone())
}

/// Keep `levels` under `key` within the budget, counting `bytes` and the source images;
/// returns what is kept there (another worker's copy when it was quicker).
fn keep(key: Key, images: &[Arc<RgbaImage>], levels: Levels, bytes: u64) -> Levels {
    // Include retained source storage in the budget, even when the decoded cache
    // already owns it. Active uploads may temporarily keep evicted chains alive.
    let bytes = bytes
        + images
            .iter()
            .map(|image| image.as_raw().len() as u64)
            .sum::<u64>();
    if bytes > MAX_BYTES {
        return levels;
    }
    let mut cache = cache().lock().expect("mip cache lock");
    // Another map worker may have prepared the same texture concurrently.
    if let Some(entry) = cache.entries.get(&key) {
        return entry.levels.clone();
    }
    while cache.bytes + bytes > MAX_BYTES || cache.entries.len() >= MAX_ENTRIES {
        let Some(oldest) = cache
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.used)
            .map(|(key, _)| key.clone())
        else {
            break;
        };
        let entry = cache
            .entries
            .remove(&oldest)
            .expect("selected existing entry");
        cache.bytes -= entry.bytes;
    }
    cache.clock = cache.clock.wrapping_add(1);
    let used = cache.clock;
    cache.bytes += bytes;
    cache.entries.insert(
        key,
        Entry {
            _images: images.to_vec(),
            levels: levels.clone(),
            bytes,
            used,
        },
    );
    levels
}

/// Return the unchanged resize algorithm's result, retaining at most 128 MiB.
/// Called only when loading textures; filtering and rendering never take this lock.
pub(crate) fn prepare(images: &[Arc<RgbaImage>], width: u32, height: u32) -> Chains {
    let key = key(images, width, height, None);
    if let Some(Levels::Rgba(chains)) = find(&key) {
        return chains;
    }
    let chains = build(images, width, height);
    let bytes = chains
        .iter()
        .flatten()
        .map(|image| image.as_raw().len() as u64)
        .sum::<u64>();
    match keep(key, images, Levels::Rgba(Arc::clone(&chains)), bytes) {
        Levels::Rgba(kept) => kept,
        Levels::Blocks(_) => chains,
    }
}

fn build(images: &[Arc<RgbaImage>], width: u32, height: u32) -> Chains {
    Arc::new(
        images
            .iter()
            .map(|image| super::chain(image, width, height))
            .collect::<Vec<_>>(),
    )
}

/// The levels the card takes for `images` at `picmip`: block-compressed when
/// `r_textureCompression` applies to the kept size ([`compress::applies`]), encoded once
/// and kept here in place of the chains (which are built for it but not kept), else
/// [`prepare`]'s chains. Run where the chains would be built, never on the render thread.
pub(crate) fn prepare_levels(
    images: &[Arc<RgbaImage>],
    width: u32,
    height: u32,
    picmip: u32,
) -> Levels {
    let skip = super::skipped(width, height, picmip);
    levels_at(
        images,
        width,
        height,
        skip,
        compress::applies(width >> skip, height >> skip),
    )
}

/// [`prepare_levels`] with `skip` levels left off, as blocks when `encode`.
fn levels_at(
    images: &[Arc<RgbaImage>],
    width: u32,
    height: u32,
    skip: u32,
    encode: bool,
) -> Levels {
    if !encode {
        return Levels::Rgba(prepare(images, width, height));
    }
    let key = key(images, width, height, Some(skip));
    if let Some(levels @ Levels::Blocks(_)) = find(&key) {
        return levels;
    }
    let chains = match find(&self::key(images, width, height, None)) {
        Some(Levels::Rgba(chains)) => chains,
        _ => build(images, width, height),
    };
    let blocks = Arc::new(compress::encode(&chains, skip as usize));
    let bytes = blocks.bytes();
    keep(key, images, Levels::Blocks(blocks), bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn images() -> Vec<Arc<RgbaImage>> {
        vec![Arc::new(RgbaImage::from_fn(256, 128, |x, y| {
            image::Rgba([x as u8, y as u8, 7, 255])
        }))]
    }

    #[test]
    fn blocks_are_encoded_once_per_skip_and_kept() {
        let images = images();
        let Levels::Blocks(first) = levels_at(&images, 256, 128, 1, true) else {
            panic!("blocks");
        };
        assert_eq!((first.width, first.height), (128, 64));
        let Levels::Blocks(again) = levels_at(&images, 256, 128, 1, true) else {
            panic!("blocks");
        };
        assert!(
            Arc::ptr_eq(&first, &again),
            "the kept blocks, not encoded again"
        );
        let Levels::Blocks(other) = levels_at(&images, 256, 128, 0, true) else {
            panic!("blocks");
        };
        assert_eq!(
            (other.width, other.height),
            (256, 128),
            "another skip, its own"
        );
    }

    #[test]
    fn without_compression_the_chains_are_the_plain_ones() {
        let images = images();
        let Levels::Rgba(chains) = levels_at(&images, 256, 128, 1, false) else {
            panic!("chains");
        };
        assert!(Arc::ptr_eq(&chains, &prepare(&images, 256, 128)));
        assert_eq!(chains[0].len() as u32, super::super::levels(256, 128));
    }
}
