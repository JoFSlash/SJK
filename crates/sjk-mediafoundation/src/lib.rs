//! Silent H.264 MP4 decoding through Windows Media Foundation, for SJK chat clips
//! (`docs/hub-chat.md`, "Trusted media sources"), behind a safe API.
//!
//! SJK carries no video codec of its own. The system's MPEG-4 source reads the
//! container from memory, its H.264 decoder and the source reader's video processor
//! turn the pictures into RGB32 ([`decode_h264_mp4`]); audio is never selected. The
//! `windows` crate supplies the COM interfaces, while `mfplat.dll` and
//! `mfreadwrite.dll` are loaded at run time from System32 only ([`missing`]), so the
//! client imports nothing from them and still starts on a Windows N edition without
//! the Media Feature Pack, where clips are simply unavailable.
//!
//! COM and Media Foundation are started once on each thread that decodes (SJK's media
//! workers) and shut down when that thread ends.
//!
//! This is one of the two workspace crates allowed `unsafe` (with sjk-gpu-memory): every call below is a COM or
//! Media Foundation call. The caller checks the file's boxes and keeps its own limits
//! before and while frames arrive.
#![cfg(windows)]

use std::ffi::c_void;
use std::mem::transmute;
use std::ops::ControlFlow;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use windows::Win32::Foundation::{E_POINTER, HGLOBAL};
use windows::Win32::Media::MediaFoundation::{
    IMFAttributes, IMFByteStream, IMFMediaType, IMFSample, IMFSourceReader,
    MF_BYTESTREAM_CONTENT_TYPE, MF_E_TOPO_CODEC_NOT_FOUND, MF_MT_DEFAULT_STRIDE, MF_MT_FRAME_SIZE,
    MF_MT_MAJOR_TYPE, MF_MT_MINIMUM_DISPLAY_APERTURE, MF_MT_SUBTYPE, MF_SOURCE_READER_ALL_STREAMS,
    MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING, MF_SOURCE_READER_FIRST_VIDEO_STREAM,
    MF_SOURCE_READERF_CURRENTMEDIATYPECHANGED, MF_SOURCE_READERF_ENDOFSTREAM,
    MF_SOURCE_READERF_ERROR, MF_VERSION, MFMediaType_Video, MFSTARTUP_LITE, MFVideoArea,
    MFVideoFormat_H264, MFVideoFormat_RGB32,
};
use windows::Win32::System::Com::StructuredStorage::CreateStreamOnHGlobal;
use windows::Win32::System::Com::{
    COINIT_DISABLE_OLE1DDE, COINIT_MULTITHREADED, CoInitializeEx, IStream, STREAM_SEEK_SET,
};
use windows::Win32::System::LibraryLoader::{
    GetProcAddress, LOAD_LIBRARY_SEARCH_SYSTEM32, LoadLibraryExW,
};
use windows::core::{HRESULT, Interface, PCSTR, PCWSTR, Result, s, w};

/// Why a clip was not decoded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// Media Foundation cannot decode clips on this system at all (missing, did not
    /// start, no H.264 decoder): not worth trying again this run.
    Unavailable(&'static str),
    /// This file is not an H.264 MP4 the reader takes, is broken, has a picture over
    /// the size asked for, or took longer than its budget.
    Failed,
}

/// One decoded picture: RGB32 (bytes B, G, R, unused), `size` pixels, its top row
/// starting at byte `top` of `pixels` and each next row `stride` bytes further (a
/// negative stride walks back up the buffer, as a bottom-up picture's rows do).
pub struct Picture<'a> {
    pub pixels: &'a [u8],
    pub top: usize,
    pub stride: isize,
    /// The visible picture (the display aperture), width and height.
    pub size: [u32; 2],
    /// Presentation time and stated duration, in 100 ns units (0 when unstated).
    pub time: i64,
    pub duration: i64,
}

/// Why clips cannot be decoded here, or `None` when Media Foundation's DLLs are
/// present.
pub fn missing() -> Option<&'static str> {
    api().is_none().then_some(
        "Windows Media Foundation is not installed (a Windows N edition needs the Media Feature Pack)",
    )
}

/// Decode the H.264 video of the MP4 file `bytes`, handing each picture to `each`
/// in presentation order until the clip ends or `each` breaks (its value returned).
/// A picture side over `max_side` fails the clip, and so does decoding past `budget`
/// (checked between samples).
pub fn decode_h264_mp4<B>(
    bytes: &[u8],
    max_side: u32,
    budget: Duration,
    each: impl FnMut(Picture<'_>) -> ControlFlow<B>,
) -> std::result::Result<Option<B>, Error> {
    let api = api().ok_or(Error::Unavailable(
        "Windows Media Foundation is not installed",
    ))?;
    if !started() {
        return Err(Error::Unavailable("Windows Media Foundation did not start"));
    }
    // SAFETY: COM and Media Foundation run on this thread (`started`); every pointer
    // used inside comes from the calls themselves or from `bytes`.
    match unsafe { decode_with(api, bytes, max_side, budget, each) } {
        Ok(Some(flow)) => Ok(flow),
        Ok(None) => Err(Error::Failed),
        Err(error) if error.code() == MF_E_TOPO_CODEC_NOT_FOUND => {
            Err(Error::Unavailable("this system has no H.264 decoder"))
        }
        Err(_) => Err(Error::Failed),
    }
}

/// The visible part of a `size` picture, `[x, y, width, height]`: the display
/// aperture `area` when it lies within the picture (an H.264 1080p picture is coded
/// 1088 rows high), else the whole picture.
pub fn aperture(size: [u32; 2], area: Option<[i64; 4]>) -> [u32; 4] {
    let whole = [0, 0, size[0], size[1]];
    let Some([x, y, w, h]) = area else {
        return whole;
    };
    let inside = x >= 0
        && y >= 0
        && w > 0
        && h > 0
        && x + w <= i64::from(size[0])
        && y + h <= i64::from(size[1]);
    if inside {
        [x as u32, y as u32, w as u32, h as u32]
    } else {
        whole
    }
}

/// Where a create function writes the new object.
type Out = *mut *mut c_void;

/// An export as `GetProcAddress` gives it, before it is given its signature.
type Proc = unsafe extern "system" fn() -> isize;
type Startup = unsafe extern "system" fn(u32, u32) -> HRESULT;
type CreateAttributes = unsafe extern "system" fn(Out, u32) -> HRESULT;
type CreateMediaType = unsafe extern "system" fn(Out) -> HRESULT;
type ByteStreamOnStream = unsafe extern "system" fn(*mut c_void, Out) -> HRESULT;
type SourceReader = unsafe extern "system" fn(*mut c_void, *mut c_void, Out) -> HRESULT;

/// The Media Foundation functions called, found at run time.
struct Api {
    startup: Startup,
    create_attributes: CreateAttributes,
    create_media_type: CreateMediaType,
    byte_stream_on_stream: ByteStreamOnStream,
    source_reader: SourceReader,
}

/// Export `name` of the system's `dll`, loaded from System32 only (never a copy
/// planted beside the game) and kept loaded for the run.
fn proc(dll: PCWSTR, name: PCSTR) -> Option<Proc> {
    // SAFETY: plain library loading by name from System32; the module is never
    // freed, so the address returned stays valid.
    unsafe {
        let module = LoadLibraryExW(dll, None, LOAD_LIBRARY_SEARCH_SYSTEM32).ok()?;
        GetProcAddress(module, name)
    }
}

fn api() -> Option<&'static Api> {
    static API: OnceLock<Option<Api>> = OnceLock::new();
    API.get_or_init(|| {
        let plat = w!("mfplat.dll");
        // SAFETY: each address is the named export, given its documented signature
        // (mfapi.h, mfreadwrite.h).
        unsafe {
            Some(Api {
                startup: transmute::<Proc, Startup>(proc(plat, s!("MFStartup"))?),
                create_attributes: transmute::<Proc, CreateAttributes>(proc(
                    plat,
                    s!("MFCreateAttributes"),
                )?),
                create_media_type: transmute::<Proc, CreateMediaType>(proc(
                    plat,
                    s!("MFCreateMediaType"),
                )?),
                byte_stream_on_stream: transmute::<Proc, ByteStreamOnStream>(proc(
                    plat,
                    s!("MFCreateMFByteStreamOnStream"),
                )?),
                source_reader: transmute::<Proc, SourceReader>(proc(
                    w!("mfreadwrite.dll"),
                    s!("MFCreateSourceReaderFromByteStream"),
                )?),
            })
        }
    })
    .as_ref()
}

/// COM joined on this thread (the multithreaded apartment), once per thread.
///
/// Nothing is undone when the thread ends. A thread-local's destructor runs under the
/// Windows loader lock, and `MFShutdown` there waits for Media Foundation's own worker
/// threads, which need that lock to end: the thread hung, and every thread starting or
/// ending after it with it (a decode worker's exit in the game, a test's in `cargo
/// test`). The process's end releases COM and Media Foundation.
struct Com;

impl Com {
    fn join() -> Self {
        // SAFETY: a documented call; the apartment is left joined until the thread
        // ends (see [`Com`]).
        let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED | COINIT_DISABLE_OLE1DDE) };
        Self
    }
}

thread_local! {
    static COM: Com = Com::join();
}

/// Whether Media Foundation runs: started once for the process (`MFStartup`, never
/// shut down, see [`Com`]), COM joined on this thread.
fn started() -> bool {
    static MF: OnceLock<bool> = OnceLock::new();
    COM.with(|_| ());
    *MF.get_or_init(|| {
        // MFSTARTUP_LITE: no socket support; clips are decoded from memory.
        // SAFETY: a documented call, made once; see [`Com`] for why it is not undone.
        api().is_some_and(|api| unsafe { (api.startup)(MF_VERSION, MFSTARTUP_LITE) }.is_ok())
    })
}

/// The object a create function wrote to `raw`, when it succeeded (`hr`).
///
/// # Safety
/// `raw` is what that function wrote: one reference to a `T`, or null.
unsafe fn created<T: Interface>(hr: HRESULT, raw: *mut c_void) -> Result<T> {
    hr.ok()?;
    if raw.is_null() {
        return Err(E_POINTER.into());
    }
    // SAFETY: per the contract, the one reference is handed over.
    Ok(unsafe { T::from_raw(raw) })
}

fn frame_size(media_type: &IMFMediaType) -> Result<[u32; 2]> {
    // SAFETY: attribute read on a live media type.
    let packed = unsafe { media_type.GetUINT64(&MF_MT_FRAME_SIZE)? };
    Ok([(packed >> 32) as u32, packed as u32])
}

fn display_area(media_type: &IMFMediaType) -> Option<[i64; 4]> {
    let mut blob = [0u8; size_of::<MFVideoArea>()];
    let mut got = 0u32;
    // SAFETY: GetBlob writes at most blob.len() bytes and the length to `got`.
    unsafe {
        media_type
            .GetBlob(&MF_MT_MINIMUM_DISPLAY_APERTURE, &mut blob, Some(&mut got))
            .ok()?;
    }
    if got as usize != blob.len() {
        return None;
    }
    // SAFETY: MFVideoArea is plain data of exactly blob's size.
    let area: MFVideoArea = unsafe { std::ptr::read_unaligned(blob.as_ptr().cast()) };
    Some([
        i64::from(area.OffsetX.value),
        i64::from(area.OffsetY.value),
        i64::from(area.Area.cx),
        i64::from(area.Area.cy),
    ])
}

/// The RGB32 output's coded size, visible area and row stride.
struct Output {
    size: [u32; 2],
    area: [u32; 4],
    stride: isize,
}

fn within(size: [u32; 2], max_side: u32) -> bool {
    size[0] > 0 && size[1] > 0 && size[0].max(size[1]) <= max_side
}

fn output(reader: &IMFSourceReader, stream: u32, max_side: u32) -> Result<Option<Output>> {
    // SAFETY: a call on a live source reader.
    let current = unsafe { reader.GetCurrentMediaType(stream)? };
    let size = frame_size(&current)?;
    if !within(size, max_side) {
        return Ok(None);
    }
    // SAFETY: attribute read on a live media type.
    let stride = unsafe { current.GetUINT32(&MF_MT_DEFAULT_STRIDE) }
        .map(|stride| stride as i32 as isize)
        .unwrap_or(size[0] as isize * 4);
    if stride.unsigned_abs() < size[0] as usize * 4 {
        return Ok(None);
    }
    Ok(Some(Output {
        size,
        area: aperture(size, display_area(&current)),
        stride,
    }))
}

/// One locked RGB32 buffer as a [`Picture`] of its visible area, top row first;
/// `None` when the buffer is too short for it.
fn picture<'a>(pixels: &'a [u8], out: &Output, time: i64, duration: i64) -> Option<Picture<'a>> {
    let [x, y, width, height] = out.area;
    let pitch = out.stride.unsigned_abs();
    // A negative stride is a bottom-up picture: its top row is stored last.
    let (first, last) = if out.stride < 0 {
        (out.size[1] - 1 - y, out.size[1] - y - height)
    } else {
        (y, y + height - 1)
    };
    let top = (first as usize)
        .checked_mul(pitch)?
        .checked_add(x as usize * 4)?;
    // The row stored last ends the bytes read.
    let end = (first.max(last) as usize)
        .checked_mul(pitch)?
        .checked_add((x + width) as usize * 4)?;
    if end > pixels.len() {
        return None;
    }
    Some(Picture {
        pixels,
        top,
        stride: out.stride,
        size: [width, height],
        time,
        duration,
    })
}

/// `Ok(None)`: this file is refused.
///
/// # Safety
/// COM and Media Foundation run on this thread.
unsafe fn decode_with<B>(
    api: &Api,
    bytes: &[u8],
    max_side: u32,
    budget: Duration,
    mut each: impl FnMut(Picture<'_>) -> ControlFlow<B>,
) -> Result<Option<Option<B>>> {
    let began = Instant::now();
    let Ok(length) = u32::try_from(bytes.len()) else {
        return Ok(None);
    };
    // SAFETY: documented COM/Media Foundation calls on this thread, with pointers
    // from `bytes`, locals or the calls' own results; the locked buffer is only read
    // between Lock and Unlock, within the length Lock reports.
    unsafe {
        // The clip in memory, as a stream the MPEG-4 source can seek.
        let stream: IStream = CreateStreamOnHGlobal(HGLOBAL::default(), true)?;
        let mut written = 0u32;
        stream
            .Write(bytes.as_ptr().cast(), length, Some(&mut written))
            .ok()?;
        if written != length {
            return Ok(None);
        }
        stream.Seek(0, STREAM_SEEK_SET, None)?;
        let mut raw = std::ptr::null_mut();
        let hr = (api.byte_stream_on_stream)(stream.as_raw(), &mut raw);
        let byte_stream: IMFByteStream = created(hr, raw)?;
        // The source resolver picks the MPEG-4 source by this type.
        byte_stream
            .cast::<IMFAttributes>()?
            .SetString(&MF_BYTESTREAM_CONTENT_TYPE, w!("video/mp4"))?;
        let mut raw = std::ptr::null_mut();
        let hr = (api.create_attributes)(&mut raw, 1);
        let attributes: IMFAttributes = created(hr, raw)?;
        // YUV to RGB32 in the reader's own video processor.
        attributes.SetUINT32(&MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING, 1)?;
        let mut raw = std::ptr::null_mut();
        let hr = (api.source_reader)(byte_stream.as_raw(), attributes.as_raw(), &mut raw);
        let reader: IMFSourceReader = created(hr, raw)?;

        // The first video stream only: audio is never decoded.
        let video = MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32;
        reader.SetStreamSelection(MF_SOURCE_READER_ALL_STREAMS.0 as u32, false)?;
        reader.SetStreamSelection(video, true)?;
        let native = reader.GetNativeMediaType(video, 0)?;
        if native.GetGUID(&MF_MT_SUBTYPE)? != MFVideoFormat_H264
            || !within(frame_size(&native)?, max_side)
        {
            return Ok(None);
        }
        let mut raw = std::ptr::null_mut();
        let hr = (api.create_media_type)(&mut raw);
        let rgb: IMFMediaType = created(hr, raw)?;
        rgb.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
        rgb.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_RGB32)?;
        reader.SetCurrentMediaType(video, None, &rgb)?;
        let Some(mut out) = output(&reader, video, max_side)? else {
            return Ok(None);
        };

        loop {
            if began.elapsed() >= budget {
                return Ok(None);
            }
            let mut flags = 0u32;
            let mut time = 0i64;
            let mut sample: Option<IMFSample> = None;
            reader.ReadSample(
                video,
                0,
                None,
                Some(&mut flags),
                Some(&mut time),
                Some(&mut sample),
            )?;
            if flags & MF_SOURCE_READERF_ERROR.0 as u32 != 0 {
                return Ok(None);
            }
            if flags & MF_SOURCE_READERF_CURRENTMEDIATYPECHANGED.0 as u32 != 0 {
                let Some(changed) = output(&reader, video, max_side)? else {
                    return Ok(None);
                };
                out = changed;
            }
            if let Some(sample) = sample {
                let duration = sample.GetSampleDuration().unwrap_or(0);
                let buffer = sample.ConvertToContiguousBuffer()?;
                let mut data = std::ptr::null_mut();
                let mut length = 0u32;
                buffer.Lock(&mut data, None, Some(&mut length))?;
                let flow = if data.is_null() {
                    None
                } else {
                    let pixels = std::slice::from_raw_parts(data, length as usize);
                    picture(pixels, &out, time, duration).map(&mut each)
                };
                buffer.Unlock()?;
                match flow {
                    None => return Ok(None),
                    Some(ControlFlow::Break(value)) => return Ok(Some(Some(value))),
                    Some(ControlFlow::Continue(())) => {}
                }
            }
            if flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
                return Ok(Some(None));
            }
        }
    }
}

/// An MP4 file of `frames` (each `size` pixels of top-down RGB32 with its duration in
/// 100 ns units), encoded by Windows' own H.264 encoder (Baseline profile) through
/// Media Foundation's sink writer: clips for tests, with no fixture file and no codec
/// of SJK's. `Ok(None)` where this Windows has no H.264 encoder or no Media
/// Foundation.
#[cfg(feature = "test-encoder")]
pub fn encode_h264_mp4(
    size: [u32; 2],
    frames: &[(Vec<u8>, i64)],
) -> std::result::Result<Option<Vec<u8>>, String> {
    encoder::encode(size, frames).map_err(|error| error.to_string())
}

#[cfg(feature = "test-encoder")]
mod encoder {
    use super::{Out, Proc, api, created, proc, started};
    use std::ffi::c_void;
    use std::mem::transmute;
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Foundation::E_FAIL;
    use windows::Win32::Media::MediaFoundation::{
        IMFMediaBuffer, IMFMediaType, IMFSample, IMFSinkWriter, MF_E_TOPO_CODEC_NOT_FOUND,
        MF_MT_AVG_BITRATE, MF_MT_DEFAULT_STRIDE, MF_MT_FRAME_RATE, MF_MT_FRAME_SIZE,
        MF_MT_INTERLACE_MODE, MF_MT_MAJOR_TYPE, MF_MT_MPEG2_PROFILE, MF_MT_PIXEL_ASPECT_RATIO,
        MF_MT_SUBTYPE, MFMediaType_Video, MFVideoFormat_H264, MFVideoFormat_RGB32,
        MFVideoInterlace_Progressive, eAVEncH264VProfile_Base,
    };
    use windows::core::{GUID, HRESULT, PCWSTR, Result, s, w};

    fn video_type(size: [u32; 2], subtype: &GUID) -> Result<Option<IMFMediaType>> {
        let Some(api) = api() else {
            return Ok(None);
        };
        let mut raw = std::ptr::null_mut();
        // SAFETY: a documented create call, then attribute writes on its result.
        unsafe {
            let media_type: IMFMediaType = created((api.create_media_type)(&mut raw), raw)?;
            media_type.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
            media_type.SetGUID(&MF_MT_SUBTYPE, subtype)?;
            media_type.SetUINT64(
                &MF_MT_FRAME_SIZE,
                (u64::from(size[0]) << 32) | u64::from(size[1]),
            )?;
            media_type.SetUINT64(&MF_MT_FRAME_RATE, (10 << 32) | 1)?;
            media_type.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, (1 << 32) | 1)?;
            media_type.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)?;
            Ok(Some(media_type))
        }
    }

    pub(super) fn encode(size: [u32; 2], frames: &[(Vec<u8>, i64)]) -> Result<Option<Vec<u8>>> {
        type SinkWriter =
            unsafe extern "system" fn(PCWSTR, *mut c_void, *mut c_void, Out) -> HRESULT;
        type NewSample = unsafe extern "system" fn(Out) -> HRESULT;
        type NewBuffer = unsafe extern "system" fn(u32, Out) -> HRESULT;
        if api().is_none() || !started() {
            return Ok(None);
        }
        let (Some(h264), Some(rgb)) = (
            video_type(size, &MFVideoFormat_H264)?,
            video_type(size, &MFVideoFormat_RGB32)?,
        ) else {
            return Ok(None);
        };
        let path = std::env::temp_dir().join(format!(
            "sjk-mediafoundation-test-{}-{:?}.mp4",
            std::process::id(),
            std::thread::current().id()
        ));
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
        let bytes = size[0] * size[1] * 4;
        // SAFETY: documented Media Foundation calls on this thread, where it started;
        // each export is given its documented signature; the locked buffer is written
        // within the length it was created with.
        unsafe {
            let plat = w!("mfplat.dll");
            let (Some(sink), Some(sample), Some(buffer)) = (
                proc(w!("mfreadwrite.dll"), s!("MFCreateSinkWriterFromURL")),
                proc(plat, s!("MFCreateSample")),
                proc(plat, s!("MFCreateMemoryBuffer")),
            ) else {
                return Ok(None);
            };
            let sink = transmute::<Proc, SinkWriter>(sink);
            let new_sample = transmute::<Proc, NewSample>(sample);
            let new_buffer = transmute::<Proc, NewBuffer>(buffer);
            let mut raw = std::ptr::null_mut();
            let hr = sink(
                PCWSTR(wide.as_ptr()),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut raw,
            );
            let writer: IMFSinkWriter = created(hr, raw)?;
            h264.SetUINT32(&MF_MT_AVG_BITRATE, 800_000)?;
            h264.SetUINT32(&MF_MT_MPEG2_PROFILE, eAVEncH264VProfile_Base.0 as u32)?;
            let stream = match writer.AddStream(&h264) {
                Err(error) if error.code() == MF_E_TOPO_CODEC_NOT_FOUND => return Ok(None),
                other => other?,
            };
            rgb.SetUINT32(&MF_MT_DEFAULT_STRIDE, size[0] * 4)?;
            match writer.SetInputMediaType(stream, &rgb, None) {
                Err(error) if error.code() == MF_E_TOPO_CODEC_NOT_FOUND => return Ok(None),
                other => other?,
            }
            writer.BeginWriting()?;
            let mut time = 0;
            for (pixels, duration) in frames {
                let mut raw = std::ptr::null_mut();
                let media_buffer: IMFMediaBuffer = created(new_buffer(bytes, &mut raw), raw)?;
                let mut data = std::ptr::null_mut();
                media_buffer.Lock(&mut data, None, None)?;
                let length = pixels.len().min(bytes as usize);
                std::ptr::copy_nonoverlapping(pixels.as_ptr(), data, length);
                media_buffer.Unlock()?;
                media_buffer.SetCurrentLength(length as u32)?;
                let mut raw = std::ptr::null_mut();
                let media_sample: IMFSample = created(new_sample(&mut raw), raw)?;
                media_sample.AddBuffer(&media_buffer)?;
                media_sample.SetSampleTime(time)?;
                media_sample.SetSampleDuration(*duration)?;
                writer.WriteSample(stream, &media_sample)?;
                time += duration;
            }
            writer.Finalize()?;
        }
        let encoded = std::fs::read(&path).map_err(|_| windows::core::Error::from(E_FAIL));
        let _ = std::fs::remove_file(&path);
        encoded.map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_display_aperture_crops_only_within_the_picture() {
        assert_eq!(aperture([1920, 1088], None), [0, 0, 1920, 1088]);
        assert_eq!(
            aperture([1920, 1088], Some([0, 0, 1920, 1080])),
            [0, 0, 1920, 1080]
        );
        assert_eq!(aperture([64, 48], Some([0, 0, 65, 48])), [0, 0, 64, 48]);
        assert_eq!(aperture([64, 48], Some([-1, 0, 8, 8])), [0, 0, 64, 48]);
    }

    #[test]
    fn pictures_never_reach_past_their_buffer() {
        let out = |stride| Output {
            size: [2, 2],
            area: [0, 0, 2, 2],
            stride,
        };
        let buffer = [0u8; 16];
        let down = picture(&buffer, &out(8), 0, 0).unwrap();
        assert_eq!((down.top, down.stride, down.size), (0, 8, [2, 2]));
        let up = picture(&buffer, &out(-8), 0, 0).unwrap();
        assert_eq!((up.top, up.stride), (8, -8));
        assert!(picture(&buffer[..15], &out(8), 0, 0).is_none());
        // The visible area of a taller coded picture, bottom-up: its top row is the
        // coded picture's last.
        let tall = Output {
            size: [2, 4],
            area: [0, 0, 2, 3],
            stride: -8,
        };
        assert_eq!(picture(&[0; 32], &tall, 0, 0).unwrap().top, 24);
    }

    #[test]
    fn garbage_is_refused_or_reported_unavailable() {
        let result = decode_h264_mp4(
            b"\0\0\0\x08ftypnot a clip",
            2048,
            Duration::from_secs(5),
            |_| ControlFlow::<()>::Continue(()),
        );
        match missing() {
            None => assert!(result.is_err()),
            Some(_) => assert!(matches!(result, Err(Error::Unavailable(_)))),
        }
    }
}
