//! Fetching the manifest and the files, checked before they are used.
//!
//! A file is written to `<name>.part` and renamed into place only once its size
//! and MD5 match the manifest, so the game never mounts half a file. A `.part`
//! left by a stopped sync is continued with a `Range` request. Each host is
//! tried in turn: a file the mirror has not copied yet (missing, or an older
//! one that fails its MD5) comes from the origin.

use crate::manifest::{Asset, Manifest};
use crate::source::{Directory, Source, listed};
use md5::{Digest, Md5};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// The manifest is about 250 KB; anything past this is not one.
const MANIFEST_LIMIT: u64 = 32 * 1024 * 1024;
/// The slowest pace a file is waited for, so a stalled connection ends.
const SLOWEST_BYTES_PER_SECOND: u64 = 32 * 1024;

/// Why a fetch stopped.
#[derive(Debug, PartialEq, Eq)]
pub enum FetchError {
    Cancelled,
    /// Every host failed; the last one's reason.
    Failed(String),
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cancelled => formatter.write_str("stopped"),
            Self::Failed(reason) => formatter.write_str(reason),
        }
    }
}

/// The HTTP client, named to the hosts as `SJK/<version> (JoF content)`.
pub fn agent(version: &str) -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_secs(20)))
        .timeout_recv_response(Some(Duration::from_secs(60)))
        .user_agent(format!("SJK/{version} (JoF content)"))
        .build()
        .into()
}

/// The largest answer of the hub's content sources.
const DIRECTORY_LIMIT: u64 = 64 * 1024;

/// The hosts to use: the hub's for the collection when it names them (the player's
/// choice among them when it is still listed), else `fallback`. The hub's empty list
/// means the collection is off; no hub and no fallback means there is nowhere to ask.
pub fn sources(
    agent: &ureq::Agent,
    directory: Option<&Directory>,
    fallback: &[Source],
) -> Result<Vec<Source>, FetchError> {
    let Some(directory) = directory else {
        return Ok(fallback.to_vec());
    };
    let hub = directory.hub.trim().trim_end_matches('/');
    let answer = if hub.is_empty() {
        Err("no SJK hub is set (cl_hubUrl)".to_owned())
    } else {
        agent
            .get(format!("{hub}/v1/content-sources"))
            .config()
            .timeout_global(Some(Duration::from_secs(15)))
            .build()
            .call()
            .map_err(|error| error.to_string())
            .and_then(|mut response| {
                response
                    .body_mut()
                    .with_config()
                    .limit(DIRECTORY_LIMIT)
                    .read_to_vec()
                    .map_err(|error| error.to_string())
            })
            .and_then(|bytes| {
                listed(&bytes, directory.collection)
                    .ok_or_else(|| "the SJK hub's list of hosts is unreadable".to_owned())
            })
    };
    match answer {
        Ok(listed) if listed.is_empty() => Err(FetchError::Failed(
            "switched off by SJK for now; try again later".into(),
        )),
        Ok(listed) => {
            let chosen: Vec<Source> = listed
                .iter()
                .filter(|source| {
                    directory
                        .only
                        .as_deref()
                        .is_none_or(|only| source.name == only)
                })
                .copied()
                .collect();
            Ok(if chosen.is_empty() { listed } else { chosen })
        }
        Err(_) if !fallback.is_empty() => Ok(fallback.to_vec()),
        Err(error) => Err(FetchError::Failed(format!(
            "cannot ask the SJK hub where it is: {error}"
        ))),
    }
}

/// The manifest from the first host that gives a readable one, and that host.
pub fn manifest(agent: &ureq::Agent, sources: &[Source]) -> Result<(Manifest, Source), FetchError> {
    let mut last = String::from("no host to ask");
    for source in sources {
        let result = agent
            .get(source.manifest)
            .config()
            .timeout_global(Some(Duration::from_secs(60)))
            .build()
            .call()
            .map_err(|error| error.to_string())
            .and_then(|mut response| {
                response
                    .body_mut()
                    .with_config()
                    .limit(MANIFEST_LIMIT)
                    .read_to_vec()
                    .map_err(|error| error.to_string())
            })
            .and_then(|bytes| Manifest::parse(&bytes).map_err(|error| error.to_string()));
        match result {
            Ok(manifest) => return Ok((manifest, *source)),
            Err(error) => last = format!("{}: {error}", source.name),
        }
    }
    Err(FetchError::Failed(last))
}

/// Fetch `asset` into `directory`, trying each host. `on_bytes` hears how many
/// of the file's bytes are on disk. Returns the checked file's path.
pub fn file(
    agent: &ureq::Agent,
    sources: &[Source],
    asset: &Asset,
    directory: &Path,
    cancel: &AtomicBool,
    on_bytes: &mut dyn FnMut(u64),
) -> Result<PathBuf, FetchError> {
    std::fs::create_dir_all(directory).map_err(|error| {
        FetchError::Failed(format!("cannot make {}: {error}", directory.display()))
    })?;
    let target = directory.join(&asset.name);
    let part = directory.join(format!("{}.part", asset.name));
    let mut last = String::from("no host to ask");
    for source in sources {
        match fetch_from(
            agent,
            &source.file_url(asset),
            asset,
            &part,
            cancel,
            on_bytes,
        ) {
            Ok(()) => {
                std::fs::rename(&part, &target).map_err(|error| {
                    FetchError::Failed(format!("cannot put {} in place: {error}", asset.name))
                })?;
                return Ok(target);
            }
            Err(FetchError::Cancelled) => return Err(FetchError::Cancelled),
            Err(FetchError::Failed(error)) => last = format!("{}: {error}", source.name),
        }
    }
    Err(FetchError::Failed(last))
}

fn fetch_from(
    agent: &ureq::Agent,
    url: &str,
    asset: &Asset,
    part: &Path,
    cancel: &AtomicBool,
    on_bytes: &mut dyn FnMut(u64),
) -> Result<(), FetchError> {
    let failed = |error: &dyn std::fmt::Display| FetchError::Failed(error.to_string());
    let mut have = std::fs::metadata(part).map_or(0, |metadata| metadata.len());
    if have >= asset.size {
        // A finished or too long `.part`: start again.
        let _ = std::fs::remove_file(part);
        have = 0;
    }
    let wait =
        Duration::from_secs(120 + asset.size.saturating_sub(have) / SLOWEST_BYTES_PER_SECOND);
    let mut request = agent
        .get(url)
        .config()
        .timeout_recv_body(Some(wait))
        .build();
    if have > 0 {
        request = request.header("Range", format!("bytes={have}-"));
    }
    let mut response = request.call().map_err(|error| failed(&error))?;
    let mut hasher = Md5::new();
    let mut file = if have > 0 && response.status().as_u16() == 206 {
        // Continue: hash what is there, then append.
        let mut existing = File::open(part).map_err(|error| failed(&error))?;
        copy_into(&mut existing, &mut hasher, None, cancel, &mut |_| {})?;
        OpenOptions::new()
            .append(true)
            .open(part)
            .map_err(|error| failed(&error))?
    } else {
        have = 0;
        File::create(part).map_err(|error| failed(&error))?
    };
    on_bytes(have);
    let mut reader = response.body_mut().as_reader();
    let written = copy_into(
        &mut reader,
        &mut hasher,
        Some(&mut file),
        cancel,
        &mut |done| on_bytes(have + done),
    )?;
    file.flush().map_err(|error| failed(&error))?;
    drop(file);
    let size = have + written;
    let digest: [u8; 16] = hasher.finalize().into();
    if size != asset.size || digest != asset.md5 {
        let _ = std::fs::remove_file(part);
        return Err(FetchError::Failed(format!(
            "{} did not match the manifest ({} of {} bytes, MD5 {})",
            asset.name,
            size,
            asset.size,
            crate::md5_hex(&digest)
        )));
    }
    Ok(())
}

/// Read everything from `reader` into the hash (and `file`), in 256 KiB pieces.
fn copy_into(
    reader: &mut dyn Read,
    hasher: &mut Md5,
    mut file: Option<&mut File>,
    cancel: &AtomicBool,
    on_bytes: &mut dyn FnMut(u64),
) -> Result<u64, FetchError> {
    let mut buffer = vec![0_u8; 256 * 1024];
    let mut done = 0;
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err(FetchError::Cancelled);
        }
        let read = match reader.read(&mut buffer) {
            Ok(0) => return Ok(done),
            Ok(read) => read,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => {
                return Err(FetchError::Failed(format!(
                    "the download broke off: {error}"
                )));
            }
        };
        hasher.update(&buffer[..read]);
        if let Some(file) = file.as_deref_mut() {
            file.write_all(&buffer[..read])
                .map_err(|error| FetchError::Failed(format!("cannot write: {error}")))?;
        }
        done += read as u64;
        on_bytes(done);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::io::BufRead;
    use std::net::TcpListener;

    fn directory(hub: &str, collection: &'static str, only: Option<&str>) -> Directory {
        Directory {
            hub: hub.to_owned(),
            collection,
            only: only.map(str::to_owned),
        }
    }

    const LISTED: &[u8] = br#"{"jof":[{"name":"mirror","manifest":"https://m.test/list","files":"https://m.test/files/"},{"name":"origin","manifest":"https://o.test/list","files":"https://o.test/files"}],"sith":[]}"#;

    #[test]
    fn the_hubs_hosts_come_first_and_the_players_choice_among_them() {
        let agent = agent("test");
        let (hub, server) = serve(LISTED, 2, false);
        let all = sources(&agent, Some(&directory(&hub, "jof", None)), &[]).unwrap();
        assert_eq!(
            all.iter().map(|source| source.name).collect::<Vec<_>>(),
            ["mirror", "origin"]
        );
        assert_eq!(all[0].files, "https://m.test/files");
        let only = sources(&agent, Some(&directory(&hub, "jof", Some("origin"))), &[]).unwrap();
        assert_eq!(only.len(), 1);
        assert_eq!(only[0].manifest, "https://o.test/list");
        server.join().unwrap();
    }

    #[test]
    fn an_empty_list_switches_the_collection_off_even_with_a_fallback() {
        let agent = agent("test");
        let (hub, server) = serve(LISTED, 1, false);
        let fallback = crate::source::SOURCES;
        let error = sources(&agent, Some(&directory(&hub, "sith", None)), fallback).unwrap_err();
        assert!(matches!(error, FetchError::Failed(reason) if reason.contains("switched off")));
        server.join().unwrap();
    }

    #[test]
    fn without_the_hub_the_fallback_is_used_and_without_one_it_says_so() {
        let agent = agent("test");
        let fallback = crate::source::SOURCES;
        assert_eq!(
            sources(&agent, Some(&directory("", "jof", None)), fallback).unwrap(),
            fallback
        );
        let error = sources(&agent, Some(&directory("", "sith", None)), &[]).unwrap_err();
        assert!(matches!(error, FetchError::Failed(reason) if reason.contains("cl_hubUrl")));
        assert_eq!(sources(&agent, None, fallback).unwrap(), fallback);
    }

    /// A one-file HTTP server on 127.0.0.1 for `requests` requests, answering
    /// `Range` with 206 when `ranges`; returns its address and the requests'
    /// `Range` headers seen.
    pub(crate) fn serve(
        body: &'static [u8],
        requests: usize,
        ranges: bool,
    ) -> (String, std::thread::JoinHandle<Vec<Option<String>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = format!("http://{}", listener.local_addr().unwrap());
        let thread = std::thread::spawn(move || {
            let mut seen = Vec::new();
            for _ in 0..requests {
                let (stream, _) = listener.accept().unwrap();
                let mut reader = std::io::BufReader::new(stream.try_clone().unwrap());
                let mut range = None;
                let mut path = String::new();
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if path.is_empty() {
                        path = line.clone();
                    }
                    if let Some(value) = line.to_ascii_lowercase().strip_prefix("range: bytes=") {
                        range = Some(value.trim().trim_end_matches('-').to_owned());
                    }
                    if line == "\r\n" || line.is_empty() {
                        break;
                    }
                }
                let mut stream = stream;
                if path.contains("missing") {
                    write!(
                        stream,
                        "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    )
                    .unwrap();
                } else if let (true, Some(from)) = (ranges, range.as_deref()) {
                    let from: usize = from.parse().unwrap();
                    let rest = &body[from..];
                    write!(
                        stream,
                        "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nContent-Range: bytes {from}-{}/{}\r\nConnection: close\r\n\r\n",
                        rest.len(),
                        body.len() - 1,
                        body.len()
                    )
                    .unwrap();
                    stream.write_all(rest).unwrap();
                } else {
                    write!(
                        stream,
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    )
                    .unwrap();
                    stream.write_all(body).unwrap();
                }
                seen.push(range);
            }
            seen
        });
        (address, thread)
    }

    pub(crate) fn asset(name: &str, body: &[u8]) -> Asset {
        Asset {
            id: 1,
            name: name.into(),
            size: body.len() as u64,
            md5: Md5::digest(body).into(),
            description: String::new(),
            target: crate::Target::Base,
            repository_path: vec!["assets".into()],
            updated: String::new(),
        }
    }

    pub(crate) fn source(address: &str, files: &str) -> Source {
        Source {
            name: "test",
            manifest: "",
            files: Box::leak(format!("{address}/{files}").into_boxed_str()),
        }
    }

    const BODY: &[u8] = b"a JoF map, not really";

    #[test]
    fn a_file_is_checked_then_put_in_place() {
        let (address, server) = serve(BODY, 1, false);
        let directory = tempfile::tempdir().unwrap();
        let mut last = 0;
        let path = file(
            &agent("test"),
            &[source(&address, "repo")],
            &asset("m.pk3", BODY),
            directory.path(),
            &AtomicBool::new(false),
            &mut |bytes| last = bytes,
        )
        .unwrap();
        assert_eq!(std::fs::read(path).unwrap(), BODY);
        assert_eq!(last, BODY.len() as u64);
        assert!(!directory.path().join("m.pk3.part").exists());
        server.join().unwrap();
    }

    #[test]
    fn a_stopped_download_continues_where_it_was() {
        let (address, server) = serve(BODY, 1, true);
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("m.pk3.part"), &BODY[..6]).unwrap();
        file(
            &agent("test"),
            &[source(&address, "repo")],
            &asset("m.pk3", BODY),
            directory.path(),
            &AtomicBool::new(false),
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(std::fs::read(directory.path().join("m.pk3")).unwrap(), BODY);
        assert_eq!(server.join().unwrap(), [Some("6".to_owned())]);
    }

    #[test]
    fn a_wrong_file_is_refused_and_the_next_host_tried() {
        let (address, server) = serve(b"something else entirely", 2, false);
        let directory = tempfile::tempdir().unwrap();
        let error = file(
            &agent("test"),
            &[source(&address, "missing"), source(&address, "repo")],
            &asset("m.pk3", BODY),
            directory.path(),
            &AtomicBool::new(false),
            &mut |_| {},
        )
        .unwrap_err();
        let FetchError::Failed(reason) = error else {
            panic!("not a failure")
        };
        assert!(reason.contains("did not match the manifest"), "{reason}");
        assert!(!directory.path().join("m.pk3").exists());
        assert!(!directory.path().join("m.pk3.part").exists());
        server.join().unwrap();
    }

    #[test]
    fn a_raised_cancel_stops_before_anything_is_kept() {
        let (address, server) = serve(BODY, 1, false);
        let directory = tempfile::tempdir().unwrap();
        let error = file(
            &agent("test"),
            &[source(&address, "repo")],
            &asset("m.pk3", BODY),
            directory.path(),
            &AtomicBool::new(true),
            &mut |_| {},
        )
        .unwrap_err();
        assert_eq!(error, FetchError::Cancelled);
        assert!(!directory.path().join("m.pk3").exists());
        server.join().unwrap();
    }
}
