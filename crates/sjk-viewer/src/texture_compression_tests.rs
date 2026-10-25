use super::*;
use block_compression::decode::decompress_blocks_as_rgba8;
use image::Rgba;

fn gradient(width: u32, height: u32, alpha: u8) -> RgbaImage {
    RgbaImage::from_fn(width, height, |x, y| {
        Rgba([
            (x * 255 / width) as u8,
            (y * 255 / height) as u8,
            128,
            alpha,
        ])
    })
}

fn chain(width: u32, height: u32, alpha: u8) -> Vec<RgbaImage> {
    let image = gradient(width, height, alpha);
    super::super::chain(&image, width, height)
}

#[test]
fn only_whole_blocks_of_matching_layers_and_large_enough_textures_are_compressed() {
    assert_eq!(compressible(&[chain(256, 128, 255)], 0), Some((256, 128)));
    // r_picmip leaves the first level off: the second is uploaded first.
    assert_eq!(compressible(&[chain(256, 128, 255)], 1), Some((128, 64)));
    assert_eq!(compressible(&[chain(32, 32, 255)], 0), None);
    assert_eq!(compressible(&[chain(66, 64, 255)], 0), None);
    assert_eq!(
        compressible(&[chain(128, 128, 255), chain(64, 64, 255)], 0),
        None
    );
    assert_eq!(compressible(&[], 0), None);
}

#[test]
fn opaque_textures_take_bc1_and_any_alpha_bc3() {
    assert_eq!(format_of(&[chain(64, 64, 255)], 0), Format::Bc1);
    assert_eq!(
        format_of(&[chain(64, 64, 255), chain(64, 64, 254)], 0),
        Format::Bc3
    );
}

#[test]
fn every_level_down_to_one_texel_becomes_whole_blocks_close_to_the_texels() {
    let ramp = RgbaImage::from_fn(64, 32, |x, y| {
        let value = ((x + y) * 255 / 95) as u8;
        Rgba([value, value / 2, 64, 255])
    });
    let layers = [super::super::chain(&ramp, 64, 32)];
    let compressed = encode(&layers, 0);
    assert_eq!(compressed.format, Format::Bc1);
    assert_eq!((compressed.width, compressed.height), (64, 32));
    assert_eq!(compressed.levels(), layers[0].len());
    for (level, (blocks, source)) in compressed.layers[0].iter().zip(&layers[0]).enumerate() {
        let (width, height) = source.dimensions();
        let (padded_width, padded_height) = (width.div_ceil(4) * 4, height.div_ceil(4) * 4);
        assert_eq!(
            blocks.len(),
            (padded_width / 4 * padded_height / 4 * 8) as usize,
            "level {level}"
        );
        let mut decoded = vec![0; (padded_width * padded_height * 4) as usize];
        decompress_blocks_as_rgba8(
            CompressionVariant::BC1,
            padded_width,
            padded_height,
            blocks,
            &mut decoded,
        );
        // BC1 keeps two 5:6:5 colours a block and two between them, one line through
        // colour space: a ramp along such a line comes back a few steps off.
        let mut difference = 0_u64;
        for (x, y, texel) in source.enumerate_pixels() {
            let at = ((y * padded_width + x) * 4) as usize;
            for channel in 0..3 {
                difference += u64::from(decoded[at + channel].abs_diff(texel[channel]));
            }
            assert_eq!(decoded[at + 3], 255);
        }
        let mean = difference as f64 / f64::from(width * height * 3);
        assert!(mean <= 8.0, "level {level}: {mean}");
    }
}

#[test]
fn alpha_survives_bc3() {
    let image = RgbaImage::from_fn(64, 64, |x, _| {
        Rgba([200, 100, 50, if x < 32 { 0 } else { 255 }])
    });
    let blocks = encode_level(Format::Bc3, &image);
    let mut decoded = vec![0; 64 * 64 * 4];
    decompress_blocks_as_rgba8(CompressionVariant::BC3, 64, 64, &blocks, &mut decoded);
    assert_eq!(decoded[3], 0);
    assert_eq!(decoded[(40 * 4) + 3], 255);
}

#[test]
fn images_without_a_file_behind_them_are_not_stored() {
    let source = Arc::new(gradient(64, 64, 255));
    assert_eq!(stored_name(&[source], 64, 64, 0), None);
}
