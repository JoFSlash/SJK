//! Block-compressed map and model textures (`r_textureCompression`), for graphics cards
//! short of video memory: an array whose texels are all opaque becomes BC1 (an eighth of
//! its RGBA8 size), one with any translucent texel BC3 (a quarter), each in the sRGB form
//! that matches `Rgba8UnormSrgb`. One format per array: an animated stage with one
//! translucent frame takes BC3 for all of them.
//!
//! The blocks are encoded on the processor where the mip chains are built (the world
//! installer's workers, the model warm-up worker, the installer itself), from the chains'
//! levels `r_picmip` keeps; the render thread only uploads them, and a texture it would
//! have to prepare itself stays RGBA8. The CPU mip cache keeps the blocks
//! ([`super::cache::prepare_levels`]). Off by default; a change applies to the textures
//! uploaded after it, so from the next map, as `r_picmip`. Adapters without BC formats
//! (`TEXTURE_COMPRESSION_BC`, missing on GLES) keep RGBA8, and so do textures whose kept
//! level zero is not a whole number of 4x4 blocks or is under [`SMALLEST`] pixels.
//!
//! Not compressed: video stages, sky boxes and the non-mipmapped path (`upload_array`; a
//! sky's gradients band in BC1), material maps (normal maps would need BC5 and the shader
//! rebuilding their third component), and everything outside the world materials (UI,
//! menus, fonts, atlases a world rewrites).
use block_compression::{CompressionVariant, encode::compress_rgba8};
use image::RgbaImage;
use std::{
    error::Error,
    sync::atomic::{AtomicBool, Ordering},
};

/// The setting: 0 off (the default), 1 on.
pub(crate) const CVAR: &str = "r_textureCompression";
/// Arrays whose kept level zero has no side this long (as small as `r_picmip` shrinks)
/// stay RGBA8: they hold almost nothing, and small pictures (environment maps, glows,
/// effects) showed block artefacts most (`gfx/effects/chrome.jpg`, 64 pixels: 28 dB).
pub(crate) const SMALLEST: u32 = 128;
/// The allocation label of compressed arrays, beside the RGBA8 ones'
/// "SJK optional mipmapped material", for the video-memory audits.
const LABEL: &str = "SJK compressed mipmapped material";

/// `r_textureCompression` as last set ([`set_enabled`]).
static ENABLED: AtomicBool = AtomicBool::new(false);
/// The device takes BC formats ([`set_device`]).
static SUPPORTED: AtomicBool = AtomicBool::new(false);

/// Read the setting when a graphics context is made, with `r_picmip`.
pub(crate) fn sample(console: Option<&crate::console::ViewerConsole>) {
    set_enabled(
        console
            .and_then(|console| console.integer_cvar(CVAR))
            .unwrap_or(0)
            != 0,
    );
}

/// Compress the textures uploaded from now on, or stop: a world being built when it
/// changes may mix the two, the next map has the new setting throughout.
pub(crate) fn set_enabled(on: bool) {
    if ENABLED.swap(on, Ordering::Relaxed) != on {
        crate::log::progress(format_args!(
            "texture compression: {}, from the next map{}",
            if on { "on" } else { "off" },
            if on && !SUPPORTED.load(Ordering::Relaxed) {
                " (this graphics card takes no BC formats: textures stay uncompressed)"
            } else {
                ""
            }
        ));
    }
}

/// Note whether the device made for the graphics context takes BC formats.
pub(crate) fn set_device(features: wgpu::Features) {
    SUPPORTED.store(
        features.contains(wgpu::Features::TEXTURE_COMPRESSION_BC),
        Ordering::Relaxed,
    );
}

/// Textures are compressed: the setting is on and the device takes BC formats.
pub(crate) fn active() -> bool {
    ENABLED.load(Ordering::Relaxed) && SUPPORTED.load(Ordering::Relaxed)
}

/// An array whose kept level zero is `width` x `height` is compressed now.
pub(crate) fn applies(width: u32, height: u32) -> bool {
    active() && fits(width, height)
}

/// A kept level zero of `width` x `height` can be compressed: whole 4x4 blocks (the BC
/// formats ask it of level zero; smaller levels are padded) and not tiny.
pub(crate) fn fits(width: u32, height: u32) -> bool {
    width.is_multiple_of(4) && height.is_multiple_of(4) && width.max(height) >= SMALLEST
}

/// A block-compressed format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Format {
    /// Opaque colour, 8 bytes a block.
    Bc1,
    /// Colour and interpolated alpha, 16 bytes a block.
    Bc3,
    /// Colour and alpha in a better-fitting code, 16 bytes a block: measured beside the
    /// others (`world_shot_texture_compression.rs`), not used.
    #[cfg(test)]
    Bc7,
}

/// The format of arrays with a translucent texel. BC7, the same size, measured 5 to 7 dB
/// better on mp/ffa3's and mp/siege_desert's, but its encoder took 18 times as long
/// (4.4 s of one core for siege_desert's 23 against BC3's 0.24 s; their BC1 5.8 s), and
/// BC7 for the opaque ones would double their size (`docs/rendering.md`, "GPU memory").
pub(crate) const ALPHA: Format = Format::Bc3;

impl Format {
    fn variant(self) -> CompressionVariant {
        match self {
            Self::Bc1 => CompressionVariant::BC1,
            Self::Bc3 => CompressionVariant::BC3,
            #[cfg(test)]
            Self::Bc7 => CompressionVariant::BC7(block_compression::BC7Settings::alpha_ultrafast()),
        }
    }

    /// The sRGB texture format, as `Rgba8UnormSrgb` decodes the RGBA8 arrays.
    pub(crate) fn texture_format(self) -> wgpu::TextureFormat {
        match self {
            Self::Bc1 => wgpu::TextureFormat::Bc1RgbaUnormSrgb,
            Self::Bc3 => wgpu::TextureFormat::Bc3RgbaUnormSrgb,
            #[cfg(test)]
            Self::Bc7 => wgpu::TextureFormat::Bc7RgbaUnormSrgb,
        }
    }

    fn block_bytes(self) -> u32 {
        match self {
            Self::Bc1 => 8,
            Self::Bc3 => 16,
            #[cfg(test)]
            Self::Bc7 => 16,
        }
    }
}

/// An array's levels as the card takes them: the levels `r_picmip` left off are gone.
pub(crate) struct Blocks {
    pub(crate) format: Format,
    /// Level zero's size.
    pub(crate) width: u32,
    pub(crate) height: u32,
    /// Each layer's levels, level zero first, each as rows of 4x4 blocks.
    pub(crate) layers: Vec<Vec<Vec<u8>>>,
}

impl Blocks {
    /// The bytes held, as the card holds them.
    pub(crate) fn bytes(&self) -> u64 {
        self.layers
            .iter()
            .flatten()
            .map(|level| level.len() as u64)
            .sum()
    }
}

/// The columns and rows of 4x4 blocks of a `width` x `height` level (a level under four
/// texels takes one whole block).
fn grid(width: u32, height: u32) -> (u32, u32) {
    (width.max(1).div_ceil(4), height.max(1).div_ceil(4))
}

/// The levels of `chains` (one whole chain per layer, all the same size) from `skip` on,
/// as BC1 when every texel of each layer's first kept level is opaque, else [`ALPHA`].
pub(crate) fn encode(chains: &[Vec<RgbaImage>], skip: usize) -> Blocks {
    let translucent = chains
        .iter()
        .filter_map(|chain| chain.get(skip))
        .any(|level| level.pixels().any(|pixel| pixel[3] != u8::MAX));
    encode_as(chains, skip, if translucent { ALPHA } else { Format::Bc1 })
}

/// [`encode`] in `format`.
pub(crate) fn encode_as(chains: &[Vec<RgbaImage>], skip: usize, format: Format) -> Blocks {
    let (width, height) = chains
        .first()
        .and_then(|chain| chain.get(skip))
        .map_or((4, 4), RgbaImage::dimensions);
    let count = super::levels(width, height) as usize;
    Blocks {
        format,
        width,
        height,
        layers: chains
            .iter()
            .map(|chain| {
                chain
                    .iter()
                    .skip(skip)
                    .take(count)
                    .map(|level| encode_level(format, level))
                    .collect()
            })
            .collect(),
    }
}

/// One level as blocks; a level that is not whole blocks repeats its last column and row
/// to fill them (the texels past its edge are never sampled).
pub(crate) fn encode_level(format: Format, level: &RgbaImage) -> Vec<u8> {
    let (columns, rows) = grid(level.width(), level.height());
    let (width, height) = (columns * 4, rows * 4);
    let padded;
    let pixels = if level.dimensions() == (width, height) {
        level.as_raw().as_slice()
    } else {
        padded = pad(level, width, height);
        padded.as_slice()
    };
    let variant = format.variant();
    let mut blocks = vec![0; variant.blocks_byte_size(width, height)];
    compress_rgba8(variant, pixels, &mut blocks, width, height, width * 4);
    blocks
}

/// `level` grown to `width` x `height` by repeating its edge texels.
fn pad(level: &RgbaImage, width: u32, height: u32) -> Vec<u8> {
    let mut pixels = Vec::with_capacity(width as usize * height as usize * 4);
    let (last_x, last_y) = (level.width().max(1) - 1, level.height().max(1) - 1);
    for y in 0..height {
        for x in 0..width {
            pixels.extend_from_slice(&level.get_pixel(x.min(last_x), y.min(last_y)).0);
        }
    }
    pixels
}

/// Upload `blocks` as a mipmapped array texture.
pub(crate) fn upload(
    device: &wgpu::Device,
    queue: &crate::frame_queue::FrameQueue,
    blocks: &Blocks,
) -> Result<wgpu::TextureView, Box<dyn Error>> {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(LABEL),
        size: wgpu::Extent3d {
            width: blocks.width,
            height: blocks.height,
            depth_or_array_layers: blocks.layers.len().max(1).try_into()?,
        },
        mip_level_count: super::levels(blocks.width, blocks.height),
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: blocks.format.texture_format(),
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    for (layer, levels) in blocks.layers.iter().enumerate() {
        for (level, data) in levels.iter().enumerate() {
            let level = u32::try_from(level)?;
            let (columns, rows) = grid(blocks.width >> level, blocks.height >> level);
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: level,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: layer.try_into()?,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(columns * blocks.format.block_bytes()),
                    rows_per_image: Some(rows),
                },
                // A level's physical size: whole blocks, as wgpu asks of compressed copies.
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

/// `blocks`' level `level` of layer `layer` decoded back to RGBA8 at its own size, for the
/// quality measurements.
#[cfg(test)]
pub(crate) fn decode(blocks: &Blocks, layer: usize, level: usize) -> RgbaImage {
    let (width, height) = (
        (blocks.width >> level).max(1),
        (blocks.height >> level).max(1),
    );
    let (columns, rows) = grid(width, height);
    let mut pixels = vec![0; (columns * rows * 64) as usize];
    block_compression::decode::decompress_blocks_as_rgba8(
        blocks.format.variant(),
        columns * 4,
        rows * 4,
        &blocks.layers[layer][level],
        &mut pixels,
    );
    let whole = RgbaImage::from_raw(columns * 4, rows * 4, pixels).expect("whole blocks");
    image::imageops::crop_imm(&whole, 0, 0, width, height).to_image()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gradient(width: u32, height: u32, alpha: impl Fn(u32, u32) -> u8) -> RgbaImage {
        RgbaImage::from_fn(width, height, |x, y| {
            image::Rgba([
                (x * 255 / width.max(2)) as u8,
                (y * 255 / height.max(2)) as u8,
                ((x + y) % 7 * 30) as u8,
                alpha(x, y),
            ])
        })
    }

    #[test]
    fn only_whole_blocks_and_not_tiny_textures_fit() {
        assert!(fits(256, 128));
        assert!(fits(128, 4));
        assert!(!fits(250, 128), "level zero must be whole blocks");
        assert!(!fits(256, 2));
        assert!(!fits(64, 64), "tiny");
        assert!(!fits(1, 1));
    }

    #[test]
    fn opaque_arrays_take_bc1_and_translucent_ones_bc3() {
        let opaque = super::super::chain(&gradient(128, 128, |_, _| 255), 128, 128);
        assert_eq!(encode(&[opaque.clone()], 0).format, Format::Bc1);
        // One translucent frame of an animation decides for the array.
        let translucent = super::super::chain(&gradient(128, 128, |x, _| (x * 2) as u8), 128, 128);
        assert_eq!(encode(&[opaque, translucent], 0).format, ALPHA);
    }

    #[test]
    fn small_levels_are_whole_blocks_of_the_right_size() {
        // A 64 x 16 texture at r_picmip 0: 64x16, 32x8, 16x4, 8x2, 4x1, 2x1, 1x1.
        let chain = super::super::chain(&gradient(64, 16, |_, _| 255), 64, 16);
        let blocks = encode(&[chain], 0);
        let sizes: Vec<usize> = blocks.layers[0].iter().map(Vec::len).collect();
        assert_eq!(sizes, [16 * 4 * 8, 8 * 2 * 8, 4 * 8, 2 * 8, 8, 8, 8]);
        assert_eq!(blocks.layers[0].len() as u32, super::super::levels(64, 16));
        assert_eq!(blocks.bytes(), sizes.iter().sum::<usize>() as u64);
    }

    #[test]
    fn the_kept_levels_start_at_the_skip() {
        let chain = super::super::chain(&gradient(512, 256, |_, _| 255), 512, 256);
        let blocks = encode(&[chain], 2);
        assert_eq!((blocks.width, blocks.height), (128, 64));
        assert_eq!(blocks.layers[0].len() as u32, super::super::levels(128, 64));
    }

    #[test]
    fn bc1_never_makes_an_opaque_texel_transparent() {
        // Flat blocks (equal end points) are where BC1's punch-through mode could show.
        let mut image = gradient(64, 64, |_, _| 255);
        for y in 0..16 {
            for x in 0..16 {
                image.put_pixel(x, y, image::Rgba([90, 60, 30, 255]));
            }
        }
        let chain = super::super::chain(&image, 64, 64);
        let blocks = encode(&[chain], 0);
        assert_eq!(blocks.format, Format::Bc1);
        for level in 0..blocks.layers[0].len() {
            let decoded = decode(&blocks, 0, level);
            assert!(
                decoded.pixels().all(|pixel| pixel[3] == 255),
                "level {level}"
            );
        }
        let flat = decode(&blocks, 0, 0);
        let pixel = flat.get_pixel(5, 5).0;
        assert!(pixel[0].abs_diff(90) <= 4 && pixel[1].abs_diff(60) <= 4);
    }

    #[test]
    fn a_padded_level_decodes_close_to_its_texels() {
        // A ramp along one colour line, which a BC1 block's four colours can follow.
        let level = RgbaImage::from_fn(6, 3, |x, _| {
            image::Rgba([(x * 40) as u8, (x * 20) as u8, 100, 255])
        });
        let blocks = Blocks {
            format: Format::Bc1,
            width: 6,
            height: 3,
            layers: vec![vec![encode_level(Format::Bc1, &level)]],
        };
        assert_eq!(blocks.layers[0][0].len(), 2 * 8);
        let decoded = decode(&blocks, 0, 0);
        assert_eq!(decoded.dimensions(), (6, 3));
        for (a, b) in level.pixels().zip(decoded.pixels()) {
            for channel in 0..3 {
                assert!(a[channel].abs_diff(b[channel]) <= 40, "{a:?} {b:?}");
            }
        }
    }
}
