//! Exact HTTPS host trust, owned by this player. No requests are made by this module.
//! GIPHY is the sole default provider; its existing adapter rewrites to its media host.
//! Other hosts, including a page's separate media CDN, require separate approval.

use super::GifId;
use arc_swap::ArcSwap;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::{
        Arc, LazyLock, OnceLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};
use url::Url;

const FILE: &str = "gif-sources.txt";
pub(crate) const MAX: usize = 64;
pub(crate) const MAX_URL_BYTES: usize = 2048;
const MAX_FILE: u64 = 32 << 10;
static PATH: OnceLock<PathBuf> = OnceLock::new();
static INITIALIZED: AtomicBool = AtomicBool::new(false);
static REVISION: AtomicU64 = AtomicU64::new(0);
static POLICY: LazyLock<ArcSwap<BTreeSet<String>>> =
    LazyLock::new(|| ArcSwap::from_pointee(defaults()));
fn defaults() -> BTreeSet<String> {
    BTreeSet::from(["giphy.com".to_owned()])
}

/// Parse a bounded, credential-free, standard-port HTTPS URL with a DNS hostname.
/// Local names, IP literals and trailing-dot aliases are never media sources.
pub(crate) fn checked_url(value: &str) -> Option<Url> {
    if value.len() > MAX_URL_BYTES || value.bytes().any(|b| b.is_ascii_control() || b == b'\\') {
        return None;
    }
    let mut url = Url::parse(value).ok()?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
    {
        return None;
    }
    let host = url.host_str()?;
    if !valid_host(host) {
        return None;
    }
    url.set_fragment(None);
    Some(url)
}
fn valid_host(host: &str) -> bool {
    host.len() <= 253
        && host.contains('.')
        && !host.ends_with('.')
        && host.parse::<std::net::IpAddr>().is_err()
        && ![".localhost", ".local", ".internal", ".home", ".lan"]
            .iter()
            .any(|suffix| host.ends_with(suffix))
        && host.split('.').all(|part| {
            !part.is_empty()
                && part.len() <= 63
                && !part.starts_with('-')
                && !part.ends_with('-')
                && part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
}
/// A typed host or pasted HTTPS link, normalised without fetching anything.
pub(crate) fn host(value: &str) -> Option<String> {
    let value = value.trim();
    let url = if value.contains("://") {
        value.to_owned()
    } else {
        format!("https://{value}/")
    };
    let url = checked_url(&url)?;
    Some(url.host_str()?.to_owned())
}
pub(crate) fn revision() -> u64 {
    REVISION.load(Ordering::Acquire)
}
/// Start a new policy revision and drop all decoded media and queued work, as
/// removing a source does (`cl_sjkChatVideo` switched).
pub(crate) fn invalidate() {
    REVISION.fetch_add(1, Ordering::AcqRel);
    super::sources_changed();
}
pub(crate) fn trusted(id: &GifId) -> bool {
    trusted_host(id.host())
}
pub(crate) fn trusted_host(host: &str) -> bool {
    (cfg!(test) || INITIALIZED.load(Ordering::Acquire)) && POLICY.load().contains(host)
}
pub(crate) fn permits(url: &Url) -> bool {
    url.host_str().is_some_and(trusted_host)
}
pub(crate) fn list() -> Vec<String> {
    POLICY.load().iter().cloned().collect()
}
/// Read once beside config.cfg. A missing file keeps GIPHY; an empty file trusts none.
pub(crate) fn load(directory: &Path) {
    if PATH.get().is_some() {
        return;
    }
    let path = directory.join(FILE);
    if PATH.set(path.clone()).is_err() {
        return;
    }
    use std::io::Read;
    match std::fs::File::open(path) {
        Ok(file) => {
            let mut text = String::new();
            let read = file.take(MAX_FILE + 1).read_to_string(&mut text);
            let list = match read {
                Ok(_) if text.len() as u64 <= MAX_FILE => {
                    text.lines().filter_map(host).take(MAX).collect()
                }
                _ => {
                    crate::log::progress(format_args!(
                        "GIF sources could not be read; media sources are disabled."
                    ));
                    BTreeSet::new()
                }
            };
            POLICY.store(Arc::new(list));
            REVISION.fetch_add(1, Ordering::AcqRel);
            super::sources_changed();
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            POLICY.store(Arc::new(BTreeSet::new()));
            REVISION.fetch_add(1, Ordering::AcqRel);
            super::sources_changed();
            crate::log::progress(format_args!(
                "GIF sources could not be read; media sources are disabled: {error}"
            ));
        }
    }
    INITIALIZED.store(true, Ordering::Release);
    REVISION.fetch_add(1, Ordering::AcqRel);
}
/// Explicit approval/revocation, persisted before publishing the policy. Revocation
/// clears all decoded frames and invalidates queued/in-flight results by revision.
pub(crate) fn set(value: &str, trust: bool) -> Result<(), &'static str> {
    let host = host(value)
        .ok_or("Enter an HTTPS site or DNS hostname, without a login or custom port.")?;
    let mut list = (**POLICY.load()).clone();
    if trust {
        if list.len() >= MAX && !list.contains(&host) {
            return Err("The source list is full; remove a source first.");
        }
        list.insert(host);
    } else {
        list.remove(&host);
    }
    let path = PATH.get().ok_or("The profile folder is not ready yet.")?;
    let pending = path.with_extension("txt.pending");
    std::fs::create_dir_all(path.parent().ok_or("No profile folder.")?)
        .map_err(|_| "Could not create the profile folder.")?;
    let text = list
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&pending, text)
        .and_then(|()| std::fs::rename(&pending, path))
        .map_err(|_| "Could not save sources; the list has not changed.")?;
    POLICY.store(Arc::new(list));
    REVISION.fetch_add(1, Ordering::AcqRel);
    super::sources_changed();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hosts_are_exact_and_urls_have_no_credentials_or_local_targets() {
        for bad in [
            "https://user:password@example.org/a.gif",
            "http://example.org/a.gif",
            "https://example.org:444/a.gif",
            "https://127.0.0.1/a.gif",
            "https://[::1]/a",
            "https://example.org./a",
            "https://host.local/a",
            "https://localhost/a",
            "https://example.org\\@other.org/a",
        ] {
            assert!(checked_url(bad).is_none(), "{bad}");
        }
        assert_eq!(
            host("https://EXAMPLE.ORG/a.gif#fragment").as_deref(),
            Some("example.org")
        );
        let defaults = defaults();
        assert!(defaults.contains("giphy.com"));
        assert!(!defaults.contains("media.example.org"));
        assert!(!defaults.contains("giphy.com.example.org"));
    }
}

static BLOCKED_REVISION: AtomicU64 = AtomicU64::new(0);
pub(crate) fn blocked_revision() -> u64 {
    BLOCKED_REVISION.load(Ordering::Relaxed)
}
static BLOCKED: LazyLock<ArcSwap<BTreeSet<String>>> =
    LazyLock::new(|| ArcSwap::from_pointee(BTreeSet::new()));
/// A discovered media host, shown for explicit approval without contacting it.
pub(crate) fn blocked(host: &str) {
    if !valid_host(host) {
        return;
    }
    BLOCKED.rcu(|old| {
        let mut next = (**old).clone();
        if next.len() < MAX {
            next.insert(host.to_owned());
        }
        Arc::new(next)
    });
    BLOCKED_REVISION.fetch_add(1, Ordering::Relaxed);
}
pub(crate) fn blocked_list() -> Vec<String> {
    BLOCKED
        .load()
        .iter()
        .filter(|host| !POLICY.load().contains(*host))
        .cloned()
        .collect()
}

#[cfg(test)]
mod release_measurement {
    #[test]
    #[ignore = "release CPU measurement; no network or game assets"]
    fn trust_policy_cost() {
        use std::{hint::black_box, time::Instant};
        let messages = (0..24)
            .map(|index| {
                if index % 2 == 0 {
                    "look https://giphy.com/gifs/MeasureGif01"
                } else {
                    "look https://media.example.org/clip.mp4"
                }
            })
            .collect::<Vec<_>>();
        for text in &messages {
            black_box(super::super::for_message(text, false));
        }
        let started = Instant::now();
        for _ in 0..100_000 {
            for text in &messages {
                black_box(super::super::for_message(black_box(text), false));
            }
        }
        println!(
            "24 mixed media messages, warm parse cache: {:.3} us/iteration (100000 iterations)",
            started.elapsed().as_secs_f64() * 1_000_000.0 / 100_000.0
        );
    }
}

#[cfg(test)]
mod profile_check {
    /// Runs alone because it deliberately changes the process-wide profile policy.
    #[test]
    #[ignore = "isolated profile/persistence check; no network or game assets"]
    fn profile_trust_roundtrip() {
        use super::*;
        let directory = tempfile::tempdir().unwrap();
        load(directory.path());
        assert!(trusted_host("giphy.com"));
        let id = GifId::from_url("https://media.example.org/clip.mp4").unwrap();
        assert!(!trusted(&id));
        assert!(super::super::for_message(id.as_str(), false).1.is_none());
        set("https://MEDIA.EXAMPLE.ORG/clip.mp4", true).unwrap();
        assert!(trusted(&id));
        assert!(super::super::for_message(id.as_str(), false).1.is_some());
        assert!(!trusted_host("other.example.org"));
        let file = directory.path().join(FILE);
        assert!(
            std::fs::read_to_string(&file)
                .unwrap()
                .contains("media.example.org")
        );
        super::super::insert_for_shot(
            &id,
            &super::super::decode::test_animation([8, 8], 2, 100),
            0,
        );
        assert!(super::super::asked_for(&id));
        let before = revision();
        set("media.example.org", false).unwrap();
        assert!(revision() > before);
        assert!(!trusted(&id));
        assert!(super::super::for_message(id.as_str(), false).1.is_none());
        assert!(!super::super::asked_for(&id));
        // Switching chat video clips (`cl_sjkChatVideo`) drops decoded media the same way.
        super::super::insert_for_shot(
            &id,
            &super::super::decode::test_animation([8, 8], 2, 100),
            0,
        );
        let before = revision();
        invalidate();
        assert!(revision() > before);
        assert!(!super::super::asked_for(&id));
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "giphy.com");
        set("giphy.com", false).unwrap();
        assert!(!trusted_host("giphy.com"));
        assert!(std::fs::read_to_string(&file).unwrap().is_empty());
        let before = revision();
        std::fs::create_dir(file.with_extension("txt.pending")).unwrap();
        assert!(set("media.example.org", true).is_err());
        assert_eq!(revision(), before);
        assert!(!trusted(&id));
    }
}
