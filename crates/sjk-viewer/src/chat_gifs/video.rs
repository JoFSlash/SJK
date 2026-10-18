//! Silent, bounded AVC/H.264 MP4 clips, decoded on the media worker, never the frame
//! thread. Audio tracks are ignored. Fragmented MP4, HEVC and WebM are unsupported.
//!
//! SJK carries no video codec of its own. On Windows the clip goes to the system's
//! Media Foundation: its MPEG-4 source reads the container, its H.264 decoder and
//! video processor turn the pictures into RGB32. It is reached through the `windows`
//! crate, with `mfplat.dll` and `mfreadwrite.dll` loaded at run time ([`mf`]), so a
//! Windows N edition without the Media Feature Pack still starts. Where Media
//! Foundation is missing, and on every other system, MP4 media is unavailable
//! ([`available`]): the link stays text, said once in the log.
//!
//! Clips are a player opt-in, off by default (`cl_sjkChatVideo`, Sol's condition,
//! 18/10/2026): they come from the internet and are parsed by Windows code outside
//! SJK's memory-safe Rust. While it is off, MP4 is unavailable the same way and Media
//! Foundation is never loaded ([`set_allowed`]).
//!
//! Before any native code sees the file, [`boxes`] walks its boxes within fixed bounds
//! and refuses what this subset does not take (fragments, external data references,
//! oversized tables, extended box sizes, no AVC sample entry). [`Clip`] keeps the
//! frames within the GIF limits: each fits [`super::decode::MAX_SIDE`] pixels, at most
//! [`MAX_FRAMES`] frames and [`MAX_DECODED`] bytes of pixels (a longer clip loops its
//! retained beginning), and the decode gives up after [`BUDGET`].

use super::decode::{Frame, Gif, MAX_CANVAS, MAX_DECODED, MAX_FRAMES, fitted};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// How long one clip's decode may work, checked between samples.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) const BUDGET: Duration = Duration::from_secs(5);

/// Whether `bytes` look like an MP4 (ISO base media) file: its first box is `ftyp`.
pub(crate) fn is_mp4(bytes: &[u8]) -> bool {
    bytes.get(4..8) == Some(b"ftyp")
}

/// Whether MP4 clips can be decoded here: the player turned them on ([`set_allowed`])
/// and Media Foundation is present. The first time it is missing, the log says why,
/// once for the whole run. While clips are off nothing is loaded.
pub(crate) fn available() -> bool {
    if !ALLOWED.load(Ordering::Relaxed) || UNAVAILABLE.load(Ordering::Relaxed) {
        return false;
    }
    match mf::missing() {
        None => true,
        Some(reason) => {
            unavailable(reason);
            false
        }
    }
}

static UNAVAILABLE: AtomicBool = AtomicBool::new(false);
static ALLOWED: AtomicBool = AtomicBool::new(false);

/// Follow `cl_sjkChatVideo`; whether that changed the switch.
pub(crate) fn set_allowed(on: bool) -> bool {
    ALLOWED.swap(on, Ordering::Relaxed) != on
}

/// Held by the tests that flip [`set_allowed`] or rely on it being off.
#[cfg(test)]
pub(crate) static SWITCH: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Mark MP4 decoding unavailable for the rest of the run, logging `reason` once.
fn unavailable(reason: &str) {
    if !UNAVAILABLE.swap(true, Ordering::Relaxed) {
        crate::log::progress(format_args!(
            "Chat MP4 clips unavailable: {reason}; their links stay text"
        ));
    }
}

/// `bytes` (an MP4 file) decoded, or `None` when it is not an AVC MP4 this subset
/// takes, is broken, exceeds a limit, or MP4 decoding is unavailable here.
pub(crate) fn decode(bytes: &[u8]) -> Option<Gif> {
    if bytes.len() as u64 > super::fetch::MAX_BYTES || !is_mp4(bytes) {
        return None;
    }
    let mut walk = Walk::default();
    boxes(bytes, 0, &mut walk)?;
    if !walk.avc1 || !available() {
        return None;
    }
    mf::decode(bytes)
}

fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(
        bytes.get(at..at.checked_add(4)?)?.try_into().ok()?,
    ))
}

/// What [`boxes`] counted and found.
#[derive(Default)]
struct Walk {
    count: usize,
    avc1: bool,
}

/// Walk the boxes of `bytes` at nesting `depth`, refusing (`None`) what the subset
/// does not take. Read only: unknown boxes are skipped, never entered.
fn boxes(bytes: &[u8], depth: usize, walk: &mut Walk) -> Option<()> {
    if depth > 8 {
        return None;
    }
    let mut offset = 0;
    while offset < bytes.len() {
        walk.count += 1;
        if walk.count > 4096 {
            return None;
        }
        let size = u32_at(bytes, offset)? as usize;
        if size < 8 {
            return None;
        } // no extended/zero-sized boxes in this subset
        let atom = bytes.get(offset..offset.checked_add(size)?)?;
        let kind: [u8; 4] = atom[4..8].try_into().ok()?;
        let body = &atom[8..];
        match &kind {
            b"moov" | b"trak" | b"mdia" | b"minf" | b"stbl" | b"dinf" => {
                boxes(body, depth + 1, walk)?
            }
            b"moof" | b"mvex" => return None,
            b"stsd" => {
                if u32_at(body, 4)? > 16 {
                    return None;
                }
                boxes(body.get(8..)?, depth + 1, walk)?;
            }
            b"dref" => {
                if u32_at(body, 4)? != 1 {
                    return None;
                }
                boxes(body.get(8..)?, depth + 1, walk)?;
            }
            // Only self-contained media data: an external reference would have the
            // reader open another file or address.
            b"url " => {
                if body != [0, 0, 0, 1] {
                    return None;
                }
            }
            b"urn " => return None,
            b"avc1" => {
                walk.avc1 = true;
                boxes(body.get(78..)?, depth + 1, walk)?
            }
            b"stts" | b"ctts" | b"stsc" | b"stco" | b"co64" | b"stss" => {
                let entries = u32_at(body, 4)? as usize;
                let stride = match &kind {
                    b"stsc" => 12,
                    b"stts" | b"ctts" | b"co64" => 8,
                    _ => 4,
                };
                if entries > 4096 || entries.checked_mul(stride)?.checked_add(8)? > body.len() {
                    return None;
                }
                if matches!(&kind, b"stts" | b"ctts") {
                    let mut samples = 0u32;
                    for row in body[8..8 + entries * stride].chunks_exact(stride) {
                        samples = samples.checked_add(u32_at(row, 0)?)?;
                    }
                    if samples > 4096 {
                        return None;
                    }
                }
            }
            b"stsz" => {
                let size = u32_at(body, 4)?;
                let samples = u32_at(body, 8)? as usize;
                if samples > 4096 || size as u64 > super::fetch::MAX_BYTES {
                    return None;
                }
                if size == 0 {
                    if samples.checked_mul(4)?.checked_add(12)? > body.len() {
                        return None;
                    }
                    for row in body[12..12 + samples * 4].chunks_exact(4) {
                        if u32_at(row, 0)? as u64 > super::fetch::MAX_BYTES {
                            return None;
                        }
                    }
                }
            }
            b"ftyp" => {
                if body.len() < 8 || body.len() % 4 != 0 {
                    return None;
                }
            }
            b"hdlr" => {
                if body.len() < 24 {
                    return None;
                }
            }
            b"mvhd" | b"tkhd" | b"mdhd" => {
                if body.is_empty() || body[0] > 1 {
                    return None;
                }
                let long = usize::from(body[0] == 1);
                let minimum = match &kind {
                    b"mvhd" => [100, 112][long],
                    b"tkhd" => [84, 96][long],
                    _ => [24, 36][long],
                };
                if body.len() < minimum {
                    return None;
                }
            }
            b"avcC" if body.len() > 65536 => return None,
            _ => {}
        }
        offset += size;
    }
    Some(())
}

/// A clip's frames as they are decoded, kept within the GIF limits (only Media
/// Foundation decodes them, so elsewhere only the tests use it).
#[cfg_attr(not(windows), allow(dead_code))]
#[derive(Default)]
struct Clip {
    size: Option<[u32; 2]>,
    frames: Vec<Frame>,
    /// Each frame's presentation time and stated duration, in 100 ns units.
    times: Vec<(i64, i64)>,
}

#[cfg_attr(not(windows), allow(dead_code))]
impl Clip {
    /// Whether another frame of the clip's size still fits [`MAX_FRAMES`] and
    /// [`MAX_DECODED`].
    fn room(&self) -> bool {
        self.frames.len() < MAX_FRAMES
            && self.size.is_none_or(|[w, h]| {
                (self.frames.len() + 1) * w as usize * h as usize * 4 <= MAX_DECODED
            })
    }

    /// Add a decoded picture of `width` by `height` pixels of 32-bit BGRX (Media
    /// Foundation's RGB32), its top row starting at byte `top` of `pixels` and each
    /// next row `stride` bytes further (a negative stride walks back up the buffer, as
    /// a bottom-up picture's rows do). `None` refuses the clip: a picture outside
    /// `pixels`, or one of another size.
    fn push_bgrx(
        &mut self,
        pixels: &[u8],
        top: usize,
        stride: isize,
        [width, height]: [u32; 2],
        time: (i64, i64),
    ) -> Option<()> {
        if width == 0 || height == 0 || width.max(height) > MAX_CANVAS {
            return None;
        }
        let fitted = fitted([width, height]);
        if self.size.is_some_and(|size| size != fitted) {
            return None;
        }
        let row_bytes = width as usize * 4;
        let mut rgba = Vec::with_capacity(row_bytes * height as usize);
        for y in 0..height as usize {
            let start = isize::try_from(top)
                .ok()?
                .checked_add(stride.checked_mul(y as isize)?)?;
            let start = usize::try_from(start).ok()?;
            let row = pixels.get(start..start.checked_add(row_bytes)?)?;
            for bgrx in row.chunks_exact(4) {
                rgba.extend_from_slice(&[bgrx[2], bgrx[1], bgrx[0], 255]);
            }
        }
        let image = image::RgbaImage::from_raw(width, height, rgba)?;
        let image = if [width, height] == fitted {
            image
        } else {
            image::imageops::resize(
                &image,
                fitted[0],
                fitted[1],
                image::imageops::FilterType::Triangle,
            )
        };
        self.size = Some(fitted);
        self.frames.push(Frame {
            rgba: image.into_raw().into_boxed_slice(),
            delay_ms: 0,
        });
        self.times.push(time);
        Some(())
    }

    /// The clip, each frame showing for its stated duration (else until the next
    /// frame, else as long as the one before), 10 ms to a second.
    fn finish(mut self) -> Option<Gif> {
        let size = self.size?;
        let mut last = 333_333;
        for index in 0..self.frames.len() {
            let (time, duration) = self.times[index];
            let until_next = self
                .times
                .get(index + 1)
                .map(|(next, _)| next - time)
                .filter(|gap| *gap > 0);
            let ticks = if duration > 0 {
                duration
            } else {
                until_next.unwrap_or(last)
            };
            last = ticks;
            self.frames[index].delay_ms = (ticks / 10_000).clamp(10, 1000) as u32;
        }
        let length_ms = self
            .frames
            .iter()
            .map(|frame| u64::from(frame.delay_ms))
            .sum();
        (!self.frames.is_empty()).then_some(Gif {
            size,
            frames: self.frames,
            length_ms,
        })
    }
}

#[cfg(not(windows))]
mod mf {
    use super::Gif;

    /// Media Foundation is Windows only.
    pub(super) fn missing() -> Option<&'static str> {
        Some("they need Windows Media Foundation")
    }

    pub(super) fn decode(_bytes: &[u8]) -> Option<Gif> {
        None
    }
}

#[cfg(windows)]
mod mf {
    //! Media Foundation through `sjk-mediafoundation`, the workspace's one crate
    //! allowed `unsafe`.

    use super::{BUDGET, Clip, Gif, MAX_CANVAS, unavailable};
    use sjk_mediafoundation::Error;
    use std::ops::ControlFlow;

    pub(super) fn missing() -> Option<&'static str> {
        sjk_mediafoundation::missing()
    }

    pub(super) fn decode(bytes: &[u8]) -> Option<Gif> {
        let mut clip = Clip::default();
        // Break(true): the limits are reached, the beginning kept; Break(false): a
        // picture the clip refuses.
        let flow = sjk_mediafoundation::decode_h264_mp4(bytes, MAX_CANVAS, BUDGET, |picture| {
            let time = (picture.time, picture.duration);
            if clip
                .push_bgrx(
                    picture.pixels,
                    picture.top,
                    picture.stride,
                    picture.size,
                    time,
                )
                .is_none()
            {
                ControlFlow::Break(false)
            } else if clip.room() {
                ControlFlow::Continue(())
            } else {
                ControlFlow::Break(true)
            }
        });
        match flow {
            Ok(None | Some(true)) => clip.finish(),
            Ok(Some(false)) | Err(Error::Failed) => None,
            Err(Error::Unavailable(reason)) => {
                unavailable(reason);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One box: its size, `kind` and `body`.
    fn atom(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut bytes = ((body.len() + 8) as u32).to_be_bytes().to_vec();
        bytes.extend_from_slice(kind);
        bytes.extend_from_slice(body);
        bytes
    }

    /// A box with a version/flags word, an entry count and `entries`.
    fn table(kind: &[u8; 4], count: u32, entries: &[u8]) -> Vec<u8> {
        let mut body = vec![0; 4];
        body.extend_from_slice(&count.to_be_bytes());
        body.extend_from_slice(entries);
        atom(kind, &body)
    }

    /// The box tree of a one-track AVC MP4 (no real pictures), `data_ref` as its data
    /// reference.
    fn skeleton(data_ref: &[u8]) -> Vec<u8> {
        let mut avc1 = vec![0; 78];
        avc1.extend(atom(b"avcC", &[1, 66, 0, 30, 0xff, 0xe0, 0]));
        let stsd = table(b"stsd", 1, &atom(b"avc1", &avc1));
        let stbl = atom(
            b"stbl",
            &[
                stsd,
                table(b"stts", 1, &[0, 0, 0, 3, 0, 0, 0, 100]),
                table(b"stsc", 0, &[]),
                atom(b"stsz", &[0; 12]),
                table(b"stco", 0, &[]),
            ]
            .concat(),
        );
        let dinf = atom(b"dinf", &table(b"dref", 1, data_ref));
        let minf = atom(b"minf", &[dinf, stbl].concat());
        let mdia = atom(
            b"mdia",
            &[atom(b"mdhd", &[0; 24]), atom(b"hdlr", &[0; 24]), minf].concat(),
        );
        let trak = atom(b"trak", &[atom(b"tkhd", &[0; 84]), mdia].concat());
        let moov = atom(b"moov", &[atom(b"mvhd", &[0; 100]), trak].concat());
        [
            atom(b"ftyp", b"isom\0\0\0\0isom"),
            moov,
            atom(b"mdat", &[0; 16]),
        ]
        .concat()
    }

    #[test]
    fn malformed_mp4_boxes_and_huge_tables_are_refused_before_parsing() {
        for bytes in [
            b"\0\0\0\x01ftyp".as_slice(),
            b"\0\0\0\x04ftyp",
            b"\xff\xff\xff\xffftyp",
            b"\0\0\0\x10stco\0\0\0\0\xff\xff\xff\xff",
            b"\0\0\0\x08moof",
        ] {
            assert!(boxes(bytes, 0, &mut Walk::default()).is_none(), "{bytes:?}");
        }
        assert!(decode(b"not a video").is_none());
    }

    #[test]
    fn a_self_contained_avc_tree_passes_and_external_data_does_not() {
        let mut walk = Walk::default();
        assert!(boxes(&skeleton(&atom(b"url ", &[0, 0, 0, 1])), 0, &mut walk).is_some());
        assert!(walk.avc1);
        // Media data in another file or at an address is never opened.
        let external = atom(b"url ", b"\0\0\0\0https://example.org/a.mp4\0");
        assert!(boxes(&skeleton(&external), 0, &mut Walk::default()).is_none());
        assert!(
            boxes(
                &skeleton(&atom(b"urn ", b"\0\0\0\0x\0")),
                0,
                &mut Walk::default()
            )
            .is_none()
        );
        // Without an AVC sample entry there is nothing to decode.
        let mut walk = Walk::default();
        let audio = [atom(b"ftyp", b"isom\0\0\0\0isom"), atom(b"mdat", &[])].concat();
        assert!(boxes(&audio, 0, &mut walk).is_some());
        assert!(!walk.avc1);
        assert!(decode(&audio).is_none());
        // Too many samples stated in the timing table.
        let mut many = skeleton(&atom(b"url ", &[0, 0, 0, 1]));
        let at = many.windows(4).position(|w| w == b"stts").unwrap() + 12;
        many[at..at + 4].copy_from_slice(&5000u32.to_be_bytes());
        assert!(boxes(&many, 0, &mut Walk::default()).is_none());
    }

    #[test]
    fn rgb32_pictures_become_rgba_top_row_first() {
        // 2 x 2 BGRX, rows 12 bytes apart (padded): top blue then red, bottom green.
        let top = [255, 0, 0, 0, 0, 0, 255, 0, 9, 9, 9, 9];
        let bottom = [0, 255, 0, 0, 0, 255, 0, 0, 9, 9, 9, 9];
        let mut clip = Clip::default();
        let down = [top, bottom].concat();
        clip.push_bgrx(&down, 0, 12, [2, 2], (0, 0)).unwrap();
        // The same picture stored bottom-up: the top row last, a negative stride.
        let up = [bottom, top].concat();
        clip.push_bgrx(&up, 12, -12, [2, 2], (0, 0)).unwrap();
        let expected = [
            0, 0, 255, 255, 255, 0, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255,
        ];
        assert_eq!(&*clip.frames[0].rgba, &expected);
        assert_eq!(&*clip.frames[1].rgba, &expected);
        // A picture reaching past its buffer, or of another size, refuses the clip.
        assert!(clip.push_bgrx(&down, 8, 12, [2, 2], (0, 0)).is_none());
        assert!(clip.push_bgrx(&down, 0, 4, [1, 3], (0, 0)).is_none());
    }

    #[test]
    fn clips_keep_the_gif_limits() {
        let mut clip = Clip::default();
        let pixel = [0u8; 4];
        while clip.room() {
            clip.push_bgrx(&pixel, 0, 4, [1, 1], (0, 0)).unwrap();
        }
        assert_eq!(clip.frames.len(), MAX_FRAMES);
        // 480 x 480 pictures: 36 fit 32 MiB of pixels, the beginning loops.
        let big = vec![0u8; 480 * 480 * 4];
        let mut clip = Clip::default();
        while clip.room() {
            clip.push_bgrx(&big, 0, 480 * 4, [480, 480], (0, 0))
                .unwrap();
        }
        assert_eq!(clip.size, Some([480, 480]));
        assert_eq!(clip.frames.len(), 36);
        let over = vec![0u8; 4];
        let side = MAX_CANVAS + 1;
        assert!(
            Clip::default()
                .push_bgrx(&over, 0, 0, [side, 1], (0, 0))
                .is_none()
        );
    }

    #[test]
    fn frame_delays_come_from_durations_then_gaps() {
        let mut clip = Clip::default();
        let pixel = [0u8; 4];
        // Stated 100 ms; no duration but the next frame 40 ms later; a 3 s duration
        // clamps to a second; the last, with neither, as long as the one before.
        for time in [
            (0, 1_000_000),
            (1_000_000, 0),
            (1_400_000, 30_000_000),
            (1_800_000, 0),
        ] {
            clip.push_bgrx(&pixel, 0, 4, [1, 1], time).unwrap();
        }
        let gif = clip.finish().unwrap();
        let delays: Vec<u32> = gif.frames.iter().map(|frame| frame.delay_ms).collect();
        assert_eq!(delays, [100, 40, 1000, 1000]);
        assert_eq!(gif.length_ms, 2140);
        assert!(Clip::default().finish().is_none());
    }

    #[test]
    fn clips_are_off_by_default_and_then_never_reach_media_foundation() {
        let _switch = SWITCH
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(!available());
        assert!(
            !UNAVAILABLE.load(Ordering::Relaxed),
            "off is not logged as missing"
        );
        assert!(set_allowed(true), "a change");
        assert!(!set_allowed(true), "no change");
        assert_eq!(available(), cfg!(windows) && mf::missing().is_none());
        assert!(set_allowed(false));
        assert!(!available());
    }

    #[cfg(not(windows))]
    #[test]
    fn without_media_foundation_mp4_is_unavailable() {
        assert!(!available());
        let bytes = skeleton(&atom(b"url ", &[0, 0, 0, 1]));
        assert!(decode(&bytes).is_none());
    }
}

#[cfg(all(test, windows))]
mod media_foundation_tests {
    //! A real clip, encoded at test time by Windows' own H.264 encoder (no codec of
    //! SJK's, no fixture file), then decoded the way chat clips are.

    use super::*;

    const SIZE: [u32; 2] = [64, 48];
    /// Each frame's top and bottom halves (BGRX): red over blue, green over red, blue
    /// over green.
    const COLOURS: [([u8; 4], [u8; 4]); 3] = [
        ([0, 0, 230, 0], [230, 0, 0, 0]),
        ([0, 230, 0, 0], [0, 0, 230, 0]),
        ([230, 0, 0, 0], [0, 230, 0, 0]),
    ];

    #[test]
    fn an_h264_clip_decodes_with_its_pictures_and_timing() {
        let frames: Vec<(Vec<u8>, i64)> = COLOURS
            .iter()
            .map(|(top, bottom)| {
                let half = (SIZE[0] * SIZE[1] / 2) as usize;
                ([top.repeat(half), bottom.repeat(half)].concat(), 1_000_000)
            })
            .collect();
        let Some(bytes) =
            sjk_mediafoundation::encode_h264_mp4(SIZE, &frames).expect("encoding the clip")
        else {
            eprintln!("skipped: no Media Foundation H.264 encoder on this Windows");
            return;
        };
        let mut walk = Walk::default();
        assert!(
            boxes(&bytes, 0, &mut walk).is_some(),
            "bounded MP4 structure"
        );
        assert!(walk.avc1);
        // Media Foundation itself: the player's switch (off by default) is not needed.
        let gif = mf::decode(&bytes).expect("H.264 MP4 decodes");
        assert_eq!(gif.size, SIZE);
        assert_eq!(gif.frames.len(), 3);
        assert_eq!(gif.length_ms, 300);
        let channel = |bgrx: [u8; 4]| [2, 1, 0].into_iter().position(|c| bgrx[c] > 0).unwrap();
        let [width, height] = SIZE.map(|side| side as usize);
        for (frame, (top, bottom)) in gif.frames.iter().zip(COLOURS) {
            for (row, colour) in [(4, top), (height - 5, bottom)] {
                let at = (row * width + width / 2) * 4;
                let pixel = &frame.rgba[at..at + 4];
                assert!(pixel[channel(colour)] > 150, "row {row}: {pixel:?}");
                assert_eq!(pixel[3], 255);
            }
        }
        assert_eq!(gif.frame_at(100), 1);
        assert_eq!(gif.frame_at(300), 0);
        // A clip whose tables were cut is refused before Media Foundation sees it.
        assert!(boxes(&bytes[..bytes.len() / 2], 0, &mut Walk::default()).is_none());
        // While the player has clips off (the default), even a good clip is not decoded.
        let _switch = SWITCH
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(!available());
        assert!(decode(&bytes).is_none());
    }
}
