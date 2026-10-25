//! The compressed textures kept on disk (`cache/textures` beside the configuration), so
//! a map loaded again skips the encoder. One file per texture array, named by what it is
//! made from ([`super::stored_name`]); a file that does not match what is asked for, in
//! size or layout, is ignored and written again. At most [`BUDGET`] bytes: each start,
//! and each eighth of it written since, deletes the files used longest ago (a load
//! touches the file it reads).
use super::{Compressed, Format};
use std::{
    error::Error,
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::SystemTime,
};

/// The store's size after each start's trim.
const BUDGET: u64 = 2 << 30;
const MAGIC: &[u8; 8] = b"SJKBLKS1";
const EXTENSION: &str = "blocks";
/// Magic, format, skip, layers, levels, width, height.
const HEADER: usize = 8 + 1 + 1 + 2 + 2 + 4 + 4;
/// Bytes written since the last trim: a long session of new maps trims again.
static WRITTEN: AtomicU64 = AtomicU64::new(0);

fn path(directory: &Path, name: &str) -> PathBuf {
    directory.join(format!("{name}.{EXTENSION}"))
}

/// The stored blocks of `name`, when they are `layers` arrays of `levels` levels whose
/// first level is `width` x `height`.
pub(super) fn load(
    directory: &Path,
    name: &str,
    width: u32,
    height: u32,
    layers: usize,
    levels: usize,
) -> Option<Compressed> {
    let path = path(directory, name);
    let bytes = fs::read(&path).ok()?;
    let compressed = parse(&bytes, width, height, layers, levels)?;
    // Used now: the trim keeps it over files not read for longer.
    if let Ok(file) = fs::File::options().write(true).open(&path) {
        let _ = file.set_modified(SystemTime::now());
    }
    Some(compressed)
}

fn parse(
    bytes: &[u8],
    width: u32,
    height: u32,
    layers: usize,
    levels: usize,
) -> Option<Compressed> {
    let header = bytes.get(..HEADER)?;
    if &header[..8] != MAGIC {
        return None;
    }
    let format = Format::from_tag(header[8])?;
    let skip = u32::from(header[9]);
    let stored_layers = usize::from(u16::from_le_bytes([header[10], header[11]]));
    let stored_levels = usize::from(u16::from_le_bytes([header[12], header[13]]));
    let stored_width = u32::from_le_bytes(header[14..18].try_into().ok()?);
    let stored_height = u32::from_le_bytes(header[18..22].try_into().ok()?);
    if (stored_width, stored_height, stored_layers, stored_levels)
        != (width, height, layers, levels)
    {
        return None;
    }
    let sizes: Vec<usize> = (0..levels)
        .map(|level| format.level_bytes((width >> level).max(1), (height >> level).max(1)))
        .collect();
    let mut body = &bytes[HEADER..];
    if body.len() != sizes.iter().sum::<usize>() * layers {
        return None;
    }
    let layers = (0..layers)
        .map(|_| {
            sizes
                .iter()
                .map(|&size| {
                    let (level, rest) = body.split_at(size);
                    body = rest;
                    level.to_vec()
                })
                .collect()
        })
        .collect();
    Some(Compressed {
        format,
        skip,
        width,
        height,
        layers,
    })
}

fn serialize(compressed: &Compressed) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = Vec::with_capacity(HEADER + compressed.bytes() as usize);
    bytes.extend_from_slice(MAGIC);
    bytes.push(compressed.format.tag());
    bytes.push(u8::try_from(compressed.skip)?);
    bytes.extend_from_slice(&u16::try_from(compressed.layers.len())?.to_le_bytes());
    bytes.extend_from_slice(&u16::try_from(compressed.levels())?.to_le_bytes());
    bytes.extend_from_slice(&compressed.width.to_le_bytes());
    bytes.extend_from_slice(&compressed.height.to_le_bytes());
    for level in compressed.layers.iter().flatten() {
        bytes.extend_from_slice(level);
    }
    Ok(bytes)
}

/// Keep `compressed` as `name`: written beside, then moved into place, so a reader never
/// sees half a file.
pub(super) fn save(
    directory: &Path,
    name: &str,
    compressed: &Compressed,
) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(directory)?;
    let mut file = tempfile::NamedTempFile::new_in(directory)?;
    let bytes = serialize(compressed)?;
    file.write_all(&bytes)?;
    file.persist(path(directory, name))?;
    let written = WRITTEN.fetch_add(bytes.len() as u64, Ordering::Relaxed) + bytes.len() as u64;
    if written > BUDGET / 8 && WRITTEN.swap(0, Ordering::Relaxed) > 0 {
        trim_in_background(directory.to_path_buf());
    }
    Ok(())
}

/// Delete the files used longest ago until the store is within [`BUDGET`], on a thread
/// of its own.
pub(super) fn trim_in_background(directory: PathBuf) {
    WRITTEN.store(0, Ordering::Relaxed);
    let _ = std::thread::Builder::new()
        .name("sjk-texture-store".into())
        .spawn(move || {
            let removed = trim(&directory, BUDGET);
            if removed > 0 {
                crate::log::progress(format_args!(
                    "texture compression: {removed} stored textures used longest ago deleted"
                ));
            }
        });
}

/// [`trim_in_background`]'s work: the number of files deleted.
fn trim(directory: &Path, budget: u64) -> usize {
    let Ok(entries) = fs::read_dir(directory) else {
        return 0;
    };
    let mut files: Vec<(SystemTime, u64, PathBuf)> = entries
        .flatten()
        .filter(|entry| entry.path().extension().is_some_and(|e| e == EXTENSION))
        .filter_map(|entry| {
            let metadata = entry.metadata().ok()?;
            Some((metadata.modified().ok()?, metadata.len(), entry.path()))
        })
        .collect();
    let mut total: u64 = files.iter().map(|(_, size, _)| size).sum();
    files.sort_by_key(|(modified, _, _)| *modified);
    let mut removed = 0;
    for (_, size, path) in files {
        if total <= budget {
            break;
        }
        if fs::remove_file(&path).is_ok() {
            total -= size;
            removed += 1;
        }
    }
    removed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Compressed {
        Compressed {
            format: Format::Bc1,
            skip: 1,
            width: 8,
            height: 4,
            layers: vec![vec![vec![7; 16], vec![9; 8], vec![3; 8], vec![5; 8]]; 2],
        }
    }

    #[test]
    fn stored_blocks_read_back_and_a_different_size_is_refused() {
        let directory = tempfile::tempdir().unwrap();
        save(directory.path(), "abc", &sample()).unwrap();
        let read = load(directory.path(), "abc", 8, 4, 2, 4).unwrap();
        assert_eq!(read.format, Format::Bc1);
        assert_eq!(read.skip, 1);
        assert_eq!(read.layers, sample().layers);
        assert!(load(directory.path(), "abc", 8, 8, 2, 4).is_none());
        assert!(load(directory.path(), "abc", 8, 4, 1, 4).is_none());
        assert!(load(directory.path(), "missing", 8, 4, 2, 4).is_none());
    }

    #[test]
    fn a_cut_file_is_refused() {
        let directory = tempfile::tempdir().unwrap();
        save(directory.path(), "abc", &sample()).unwrap();
        let file = path(directory.path(), "abc");
        let bytes = fs::read(&file).unwrap();
        fs::write(&file, &bytes[..bytes.len() - 1]).unwrap();
        assert!(load(directory.path(), "abc", 8, 4, 2, 4).is_none());
    }

    #[test]
    fn trim_deletes_the_files_used_longest_ago() {
        let directory = tempfile::tempdir().unwrap();
        let now = SystemTime::now();
        for (index, name) in ["old", "middle", "new"].iter().enumerate() {
            let file = path(directory.path(), name);
            fs::write(&file, [0; 100]).unwrap();
            fs::File::options()
                .write(true)
                .open(&file)
                .unwrap()
                .set_modified(now - std::time::Duration::from_secs(100 - index as u64 * 10))
                .unwrap();
        }
        fs::write(directory.path().join("other.txt"), [0; 500]).unwrap();
        assert_eq!(trim(directory.path(), 200), 1);
        assert!(!path(directory.path(), "old").exists());
        assert!(path(directory.path(), "middle").exists());
        assert!(directory.path().join("other.txt").exists());
    }
}
