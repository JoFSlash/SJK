//! Block compression of map and model textures (`r_textureCompression`): BC1 for
//! opaque textures (an eighth of their RGBA8 video memory), BC3 for those with alpha (a
//! quarter). For small and integrated graphics cards: a JoF map such as `amjh3te` held
//! 1.3 GB of RGBA8 textures, about 180 MB as blocks (`scripts/vram-estimate.ps1`).
//!
//! The loading workers compress a texture's mip chain after building it, every level
//! [`super::upload_chains`] uploads, and keep the blocks on disk
//! ([`store`], `cache/textures` beside the configuration) named by what the texture is
//! made from: the MD4 of each source image file
//! ([`crate::decoded_image_cache::content_digest`]), the array's size and the
//! `r_picmip` levels left off. A map or pack updated with other images gives other
//! names, so its textures are compressed again and the old files age out of the
//! store's budget; nothing is ever looked up by file name. Textures without a source
//! file (generated pictures) are compressed again on each load.
//!
//! Left uncompressed: textures whose uploaded size is not whole 4x4 blocks or is smaller
//! than [`SMALLEST`], the non-mipmapped filtering path, devices without
//! `TEXTURE_COMPRESSION_BC`, and every other texture (fonts, UI, effects, lightmaps).
//! The change applies to textures uploaded after it, so from the next map.
use super::RgbaImage;
use block_compression::{CompressionVariant, encode::compress_rgba8};
use md5::{Digest, Md5};
use std::{
    error::Error,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

#[path = "texture_compression_store.rs"]
mod store;

/// The setting: compress map and model textures, on by default.
pub(crate) const CVAR: &str = "r_textureCompression";
/// Textures whose longest uploaded side is shorter stay RGBA8: little to save.
pub(crate) const SMALLEST: u32 = 64;
/// Part of every stored name: raise it when the encoder, the mip chain or the file
/// layout change, so blocks made the old way are never read.
const VERSION: u32 = 1;

static ENABLED: AtomicBool = AtomicBool::new(false);
static SUPPORTED: AtomicBool = AtomicBool::new(false);
static DIRECTORY: Mutex<Option<PathBuf>> = Mutex::new(None);

/// What the loads since the last [`take_stats`] compressed.
static ARRAYS: AtomicU64 = AtomicU64::new(0);
static FROM_DISK: AtomicU64 = AtomicU64::new(0);
static RGBA_BYTES: AtomicU64 = AtomicU64::new(0);
static BLOCK_BYTES: AtomicU64 = AtomicU64::new(0);

/// The block format of a compressed texture; both are sRGB like the RGBA8 path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Format {
    /// Opaque: 8 bytes a 4x4 block.
    Bc1,
    /// With alpha: 16 bytes a 4x4 block.
    Bc3,
}

impl Format {
    pub(crate) fn texture_format(self) -> wgpu::TextureFormat {
        match self {
            Self::Bc1 => wgpu::TextureFormat::Bc1RgbaUnormSrgb,
            Self::Bc3 => wgpu::TextureFormat::Bc3RgbaUnormSrgb,
        }
    }

    fn variant(self) -> CompressionVariant {
        match self {
            Self::Bc1 => CompressionVariant::BC1,
            Self::Bc3 => CompressionVariant::BC3,
        }
    }

    fn block_bytes(self) -> u32 {
        match self {
            Self::Bc1 => 8,
            Self::Bc3 => 16,
        }
    }

    /// The bytes of a `width` x `height` level, rounded up to whole blocks.
    fn level_bytes(self, width: u32, height: u32) -> usize {
        self.variant().blocks_byte_size(width, height)
    }

    fn tag(self) -> u8 {
        match self {
            Self::Bc1 => 1,
            Self::Bc3 => 3,
        }
    }

    fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            1 => Some(Self::Bc1),
            3 => Some(Self::Bc3),
            _ => None,
        }
    }
}

/// A texture array's levels as blocks, from the first level uploaded.
pub(crate) struct Compressed {
    pub(crate) format: Format,
    /// The `r_picmip` levels left off before the first level here.
    pub(crate) skip: u32,
    /// The first level's size, in texels.
    pub(crate) width: u32,
    pub(crate) height: u32,
    /// Per layer, per level: that level's blocks.
    pub(crate) layers: Vec<Vec<Vec<u8>>>,
}

impl Compressed {
    fn levels(&self) -> usize {
        self.layers.first().map_or(0, Vec::len)
    }

    fn bytes(&self) -> u64 {
        self.layers
            .iter()
            .flatten()
            .map(|level| level.len() as u64)
            .sum()
    }
}

/// Read the setting, the store's folder and whether `device` takes BC textures, when a
/// graphics context is made ([`super::super::Policy::sample`]'s callers).
pub(crate) fn sample(console: Option<&crate::console::ViewerConsole>, device: &wgpu::Device) {
    SUPPORTED.store(
        device
            .features()
            .contains(wgpu::Features::TEXTURE_COMPRESSION_BC),
        Ordering::Relaxed,
    );
    set_enabled(
        console
            .and_then(|console| console.integer_cvar(CVAR))
            .unwrap_or(1)
            != 0,
    );
    if let Some(console) = console {
        let directory = console.config_directory().join("cache").join("textures");
        let mut current = DIRECTORY.lock().expect("texture store folder lock");
        if current.as_ref() != Some(&directory) {
            store::trim_in_background(directory.clone());
            *current = Some(directory);
        }
    }
}

/// Compress the textures uploaded from now on (`r_textureCompression`), from the next map.
pub(crate) fn set_enabled(on: bool) {
    if ENABLED.swap(on, Ordering::Relaxed) != on {
        crate::log::progress(format_args!(
            "texture compression: {}, from the next map",
            if on { "on" } else { "off" }
        ));
    }
}

/// Whether textures uploaded now are compressed.
pub(crate) fn active() -> bool {
    ENABLED.load(Ordering::Relaxed) && SUPPORTED.load(Ordering::Relaxed)
}

/// The blocks of `chains` (one mip chain per layer, from [`super::chain`]) from level
/// `skip` on, or `None` when they stay RGBA8. `sources` are the decoded images the
/// chains were made from, which name the stored copy. Loading work only: a large texture
/// takes a few hundred milliseconds the first time.
pub(crate) fn compress(
    chains: &[Vec<RgbaImage>],
    skip: u32,
    sources: &[Arc<RgbaImage>],
) -> Option<Arc<Compressed>> {
    if !active() {
        return None;
    }
    let (width, height) = compressible(chains, skip)?;
    let levels = chains[0].len() - skip as usize;
    let name = stored_name(sources, chains, skip);
    let directory = DIRECTORY.lock().expect("texture store folder lock").clone();
    let stored = name
        .as_ref()
        .zip(directory.as_ref())
        .and_then(|(name, directory)| {
            store::load(directory, name, width, height, chains.len(), levels)
        });
    let from_disk = stored.is_some();
    let compressed = match stored {
        Some(compressed) => compressed,
        None => {
            let compressed = encode(chains, skip);
            if let Some((name, directory)) = name.as_ref().zip(directory.as_ref())
                && let Err(error) = store::save(directory, name, &compressed)
            {
                crate::log::progress(format_args!("texture compression: not stored ({error})"));
            }
            compressed
        }
    };
    ARRAYS.fetch_add(1, Ordering::Relaxed);
    FROM_DISK.fetch_add(u64::from(from_disk), Ordering::Relaxed);
    RGBA_BYTES.fetch_add(
        chains
            .iter()
            .flat_map(|chain| &chain[skip as usize..])
            .map(|level| level.as_raw().len() as u64)
            .sum(),
        Ordering::Relaxed,
    );
    BLOCK_BYTES.fetch_add(compressed.bytes(), Ordering::Relaxed);
    Some(Arc::new(compressed))
}

/// The first uploaded level's size when `chains` can become blocks from level `skip`:
/// every layer the same size and as many levels, in whole blocks, not too small.
fn compressible(chains: &[Vec<RgbaImage>], skip: u32) -> Option<(u32, u32)> {
    let first = chains.first()?;
    let (width, height) = first.get(skip as usize)?.dimensions();
    let same = chains.iter().all(|chain| {
        chain.len() == first.len() && chain[skip as usize].dimensions() == (width, height)
    });
    (same && width.is_multiple_of(4) && height.is_multiple_of(4) && width.max(height) >= SMALLEST)
        .then_some((width, height))
}

/// BC1 when every texel of the first uploaded level of every layer is opaque, else BC3.
fn format_of(chains: &[Vec<RgbaImage>], skip: u32) -> Format {
    if chains
        .iter()
        .all(|chain| chain[skip as usize].pixels().all(|texel| texel[3] == 255))
    {
        Format::Bc1
    } else {
        Format::Bc3
    }
}

fn encode(chains: &[Vec<RgbaImage>], skip: u32) -> Compressed {
    let format = format_of(chains, skip);
    let first = &chains[0][skip as usize];
    Compressed {
        format,
        skip,
        width: first.width(),
        height: first.height(),
        layers: chains
            .iter()
            .map(|chain| {
                chain[skip as usize..]
                    .iter()
                    .map(|level| encode_level(format, level))
                    .collect()
            })
            .collect(),
    }
}

/// One level's blocks. Levels that are not whole blocks (2x2, 1x1, or the odd sizes
/// below a size that is) repeat their last row and column to fill the block, as the
/// card samples only the level's own texels.
pub(crate) fn encode_level(format: Format, level: &RgbaImage) -> Vec<u8> {
    let (width, height) = level.dimensions();
    let (padded_width, padded_height) = (width.div_ceil(4) * 4, height.div_ceil(4) * 4);
    let padded;
    let pixels = if (padded_width, padded_height) == (width, height) {
        level.as_raw().as_slice()
    } else {
        padded = RgbaImage::from_fn(padded_width, padded_height, |x, y| {
            *level.get_pixel(x.min(width - 1), y.min(height - 1))
        });
        padded.as_raw().as_slice()
    };
    let mut blocks = vec![0; format.level_bytes(padded_width, padded_height)];
    compress_rgba8(
        format.variant(),
        pixels,
        &mut blocks,
        padded_width,
        padded_height,
        padded_width * 4,
    );
    blocks
}

/// The stored copy's name: what the texture is made from, or `None` when a source has
/// no file behind it.
fn stored_name(sources: &[Arc<RgbaImage>], chains: &[Vec<RgbaImage>], skip: u32) -> Option<String> {
    let mut hash = Md5::new();
    hash.update(b"sjk texture blocks");
    hash.update(VERSION.to_le_bytes());
    let (width, height) = chains[0][0].dimensions();
    for value in [
        width,
        height,
        skip,
        chains.len() as u32,
        sources.len() as u32,
    ] {
        hash.update(value.to_le_bytes());
    }
    for source in sources {
        hash.update(crate::decoded_image_cache::content_digest(source)?);
        hash.update(source.width().to_le_bytes());
        hash.update(source.height().to_le_bytes());
    }
    Some(
        hash.finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    )
}

/// An array texture of `compressed`, all its levels and layers written.
pub(crate) fn upload(
    device: &wgpu::Device,
    queue: &crate::frame_queue::FrameQueue,
    compressed: &Compressed,
) -> Result<wgpu::TextureView, Box<dyn Error>> {
    let format = compressed.format;
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("SJK optional mipmapped material (compressed)"),
        size: wgpu::Extent3d {
            width: compressed.width,
            height: compressed.height,
            depth_or_array_layers: compressed.layers.len().max(1).try_into()?,
        },
        mip_level_count: compressed.levels().try_into()?,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: format.texture_format(),
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    for (layer, levels) in compressed.layers.iter().enumerate() {
        for (level, blocks) in levels.iter().enumerate() {
            let (width, height) = (
                (compressed.width >> level).max(1),
                (compressed.height >> level).max(1),
            );
            let (columns, rows) = (width.div_ceil(4), height.div_ceil(4));
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: level.try_into()?,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: layer.try_into()?,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                blocks,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(columns * format.block_bytes()),
                    rows_per_image: Some(rows),
                },
                // Whole blocks: the level's physical size.
                wgpu::Extent3d {
                    width: columns * 4,
                    height: rows * 4,
                    depth_or_array_layers: 1,
                },
            );
        }
    }
    Ok(texture.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        ..Default::default()
    }))
}

/// What was compressed since the last call, for the load's log line: arrays, of them
/// read from the store, and the RGBA8 and block bytes.
pub(crate) fn take_stats() -> (u64, u64, u64, u64) {
    (
        ARRAYS.swap(0, Ordering::Relaxed),
        FROM_DISK.swap(0, Ordering::Relaxed),
        RGBA_BYTES.swap(0, Ordering::Relaxed),
        BLOCK_BYTES.swap(0, Ordering::Relaxed),
    )
}

#[cfg(test)]
#[path = "texture_compression_tests.rs"]
mod tests;
