//! Optional load-time mip chains. Level zero uses the existing resize policy unchanged.
//!
//! `r_picmip` (as in Jedi Academy) leaves a texture's largest levels off the card:
//! each step halves its sides and takes a quarter of its video memory, for cards
//! that run out (a 3 GB card ran out on most JoF maps, 21/10/2026). Textures of
//! [`PICMIP_FLOOR`] pixels or less keep their size. The chains are still built
//! whole on the processor; only the upload skips levels. A change applies to the
//! textures uploaded after it, so from the next map.
use image::{RgbaImage, imageops};
use std::{
    error::Error,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};

#[path = "texture_mip_cache.rs"]
pub(crate) mod cache;
#[path = "texture_compression.rs"]
pub(crate) mod compression;

/// The texture detail setting: the levels left off.
pub(crate) const PICMIP_CVAR: &str = "r_picmip";
/// The largest `r_picmip` taken.
pub(crate) const PICMIP_MAX: u32 = 3;
/// Textures whose longest side is this or less are never shrunk.
pub(crate) const PICMIP_FLOOR: u32 = 128;
/// `r_picmip` as last set ([`set_picmip`]).
static PICMIP: AtomicU32 = AtomicU32::new(0);

/// Read `r_picmip` when a graphics context is made, with the filtering
/// ([`super::Policy::sample`]).
pub(crate) fn sample_picmip(console: Option<&crate::console::ViewerConsole>) {
    set_picmip(
        console
            .and_then(|console| console.integer_cvar(PICMIP_CVAR))
            .unwrap_or(0),
    );
}

/// Take `levels` of `r_picmip` for the textures uploaded from now on: a world being
/// built when it changes may mix the two sizes, the next map has the new one.
pub(crate) fn set_picmip(levels: i64) {
    let levels = levels.clamp(0, i64::from(PICMIP_MAX)) as u32;
    if PICMIP.swap(levels, Ordering::Relaxed) != levels {
        crate::log::progress(format_args!(
            "texture detail: r_picmip {levels}, from the next map"
        ));
    }
}

/// `r_picmip` as last set.
pub(crate) fn picmip() -> u32 {
    PICMIP.load(Ordering::Relaxed)
}

/// `image` with the levels `r_picmip` leaves off ([`skipped`]) taken away, for a
/// texture uploaded whole rather than as a chain; `None` when it keeps its size.
pub(crate) fn shrunk(image: &RgbaImage) -> Option<RgbaImage> {
    let (width, height) = image.dimensions();
    let skip = skipped(width, height, picmip());
    (skip > 0).then(|| {
        imageops::resize(
            image,
            (width >> skip).max(1),
            (height >> skip).max(1),
            imageops::FilterType::Triangle,
        )
    })
}

/// The levels of a `width` x `height` texture left off the card at `picmip`.
pub(crate) fn skipped(width: u32, height: u32, picmip: u32) -> u32 {
    let longest = width.max(height);
    (0..picmip.min(PICMIP_MAX))
        .take_while(|skip| longest >> (skip + 1) >= PICMIP_FLOOR)
        .count() as u32
}

/// Number of complete mip levels through 1x1, including rectangular/NPOT images.
pub(crate) fn levels(width: u32, height: u32) -> u32 {
    32 - width.max(height).max(1).leading_zeros()
}

/// One layer's mip chain, level zero first: `image` fitted to `width` x `height`, then
/// halved down to one texel. Pure processor work, so a loader may build many at once.
pub(crate) fn chain(image: &RgbaImage, width: u32, height: u32) -> Vec<RgbaImage> {
    let mut chain = Vec::with_capacity(levels(width, height) as usize);
    chain.push(if image.width() == width && image.height() == height {
        image.clone()
    } else {
        imageops::resize(image, width, height, imageops::FilterType::Lanczos3)
    });
    for _ in 1..levels(width, height) {
        let above = chain.last().expect("level zero is there");
        chain.push(imageops::resize(
            above,
            (above.width() / 2).max(1),
            (above.height() / 2).max(1),
            imageops::FilterType::Triangle,
        ));
    }
    chain
}

/// The extent of the array `images` become layers of.
pub(crate) fn extent(images: &[Arc<RgbaImage>]) -> (u32, u32) {
    (
        images.iter().map(|v| v.width()).max().unwrap_or(1),
        images.iter().map(|v| v.height()).max().unwrap_or(1),
    )
}

/// What the worlds of one context share a map texture as ([`crate::shared_textures`]):
/// the view, and the images it was made from, held so their addresses (the key) cannot
/// name other pixels while it lives.
type Shared = (wgpu::TextureView, Arc<[Arc<RgbaImage>]>);
const SHARED: &str = "map texture";

/// The key of the `width` x `height` array `images` become. Decoded map images are
/// one `Arc` per mounted file and content for the process
/// ([`crate::decoded_image_cache`]), so two maps using the same file give one key.
fn shared_key(images: &[Arc<RgbaImage>], width: u32, height: u32) -> u64 {
    let mut parts = vec![width.to_le_bytes().to_vec(), height.to_le_bytes().to_vec()];
    parts.extend(
        images
            .iter()
            .map(|image| (Arc::as_ptr(image) as usize as u64).to_le_bytes().to_vec()),
    );
    crate::shared_textures::key(&parts.iter().map(Vec::as_slice).collect::<Vec<_>>())
}

/// The mipmapped array of `images` another world of this context holds, if any.
pub(crate) fn find_shared(images: &[Arc<RgbaImage>]) -> Option<wgpu::TextureView> {
    let (width, height) = extent(images);
    crate::shared_textures::find::<Shared>(SHARED, shared_key(images, width, height))
        .map(|(view, _)| view)
}

/// Mip chains and, when compressed ([`compression`]), their blocks: what a worker
/// prepares for [`upload_chains`].
pub(crate) type Prepared = (
    Arc<Vec<Vec<RgbaImage>>>,
    Option<Arc<compression::Compressed>>,
);

/// The chains of `images` as a `width` x `height` array ([`cache::prepare`]) and their
/// blocks: loading work, for a worker where there is one.
pub(crate) fn prepare(images: &[Arc<RgbaImage>], width: u32, height: u32) -> Prepared {
    let chains = cache::prepare(images, width, height);
    let compressed = compression::compress(&chains, upload_skip(width, height, &chains), images);
    (chains, compressed)
}

/// [`upload_chains`] of `images`, or another world's copy of the same array.
pub(crate) fn upload_shared(
    device: &wgpu::Device,
    queue: &crate::frame_queue::FrameQueue,
    images: &[Arc<RgbaImage>],
    width: u32,
    height: u32,
    prepared: impl FnOnce() -> Prepared,
) -> Result<wgpu::TextureView, Box<dyn Error>> {
    let key = shared_key(images, width, height);
    if let Some((view, _)) = crate::shared_textures::find::<Shared>(SHARED, key) {
        return Ok(view);
    }
    let (chains, compressed) = prepared();
    let view = upload_chains(device, queue, width, height, &chains, compressed.as_deref())?;
    let shared: Shared = (view.clone(), images.into());
    crate::shared_textures::share(SHARED, key, || shared);
    Ok(view)
}

/// Upload mipmapped stage frames once; no frame-path image generation or readback.
pub(crate) fn upload(
    device: &wgpu::Device,
    queue: &crate::frame_queue::FrameQueue,
    images: &[Arc<RgbaImage>],
) -> Result<wgpu::TextureView, Box<dyn Error>> {
    let (width, height) = extent(images);
    upload_shared(device, queue, images, width, height, || {
        prepare(images, width, height)
    })
}

/// The levels of `chains`, a `width` x `height` array, that `r_picmip` leaves off now
/// ([`skipped`]), keeping at least one.
pub(crate) fn upload_skip(width: u32, height: u32, chains: &[Vec<RgbaImage>]) -> u32 {
    let shortest = chains.iter().map(Vec::len).min().unwrap_or(0);
    skipped(width, height, PICMIP.load(Ordering::Relaxed))
        .min(u32::try_from(shortest.saturating_sub(1)).unwrap_or(u32::MAX))
}

/// Upload one [`chain`] per layer of a `width` x `height` array, less the levels
/// `r_picmip` leaves off ([`skipped`]): as `compressed`'s blocks when they were made for
/// those levels, else as RGBA8.
pub(crate) fn upload_chains(
    device: &wgpu::Device,
    queue: &crate::frame_queue::FrameQueue,
    width: u32,
    height: u32,
    chains: &[Vec<RgbaImage>],
    compressed: Option<&compression::Compressed>,
) -> Result<wgpu::TextureView, Box<dyn Error>> {
    let skip = upload_skip(width, height, chains);
    // `r_picmip` changed since the worker compressed them: those blocks are other levels.
    if let Some(compressed) = compressed
        .filter(|compressed| compressed.skip == skip && compressed.layers.len() == chains.len())
    {
        return compression::upload(device, queue, compressed);
    }
    let (width, height) = ((width >> skip).max(1), (height >> skip).max(1));
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("SJK optional mipmapped material"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: chains.len().max(1).try_into()?,
        },
        mip_level_count: levels(width, height),
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    for (layer, chain) in chains.iter().enumerate() {
        for (level, pixels) in chain.iter().skip(skip as usize).enumerate() {
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
                pixels.as_raw(),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(pixels.width() * 4),
                    rows_per_image: Some(pixels.height()),
                },
                wgpu::Extent3d {
                    width: pixels.width(),
                    height: pixels.height(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picmip_halves_large_textures_down_to_the_floor_only() {
        assert_eq!(skipped(1024, 512, 0), 0);
        assert_eq!(skipped(1024, 512, 1), 1);
        assert_eq!(skipped(1024, 512, 3), 3);
        // 256 halves once to the floor, 128 and less keep their size.
        assert_eq!(skipped(256, 256, 3), 1);
        assert_eq!(skipped(128, 64, 3), 0);
        assert_eq!(skipped(4096, 4096, 9), PICMIP_MAX);
    }

    #[test]
    fn a_shrunk_chain_keeps_its_smaller_levels_whole() {
        // A 512 x 256 texture at r_picmip 2 is its 128 x 64 level and the levels under.
        let (width, height) = (512_u32, 256_u32);
        let skip = skipped(width, height, 2);
        assert_eq!(skip, 2);
        let image = RgbaImage::new(width, height);
        let chain = chain(&image, width, height);
        let kept = &chain[skip as usize..];
        assert_eq!((kept[0].width(), kept[0].height()), (128, 64));
        assert_eq!(kept.len() as u32, levels(width >> skip, height >> skip));
    }
}
