//! Fetching a GIF from GIPHY, on the GIFs' worker: only the addresses
//! [`super::link::media_url`] rebuilds from an id (checked by
//! [`super::link::fetchable`] before each request), over HTTPS, with no redirect
//! followed, a [`TIMEOUT`] and at most [`MAX_BYTES`] read. The small rendition is
//! asked first (the GIF picker's tiles ask the preview before it); the original only
//! when GIPHY has no small one (404), within the same cap. GIPHY serves a "not found" picture with a 200 for the original of an id it does
//! not know; its marking header makes that a 404 too.

use super::link::{self, GifId, Rendition};
use std::time::Duration;

/// The most bytes read of one file: 8 MiB since 18/10/2026 (Sol's decision; 4 MiB
/// before), as the GIF a Tenor page names is often 4 to 5 MB. Memory is bounded by
/// decoding's own limits, not by this.
pub(crate) const MAX_BYTES: u64 = 8 << 20;
/// How long one request may take, from connecting to the last byte.
pub(crate) const TIMEOUT: Duration = Duration::from_secs(10);

/// The header GIPHY's media host puts on the picture it serves in place of a GIF it
/// does not have.
const NOT_FOUND_HEADER: &str = "x-retry-not-found-metric";

/// Why a GIF could not be fetched.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Failure {
    /// GIPHY answered with this status (a redirect included: none is followed).
    Status(u16),
    /// Larger than [`MAX_BYTES`].
    TooLarge,
    /// Not reached, or the answer broke off.
    Network(String),
    /// The address is not one SJK fetches (never, as it is rebuilt).
    Refused,
    /// A redirect or a page led to this host, which the player has not trusted.
    Untrusted(String),
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Status(status) => write!(f, "Media source answered {status}"),
            Self::TooLarge => write!(f, "larger than {} MiB", MAX_BYTES >> 20),
            Self::Network(error) => write!(f, "{error}"),
            Self::Refused => write!(f, "Media source blocked or unsupported; check GIF Sources"),
            Self::Untrusted(host) => {
                write!(f, "its media is on {host}, not trusted; check GIF Sources")
            }
        }
    }
}

/// What a request gets: the status and, for a 200, the body.
pub(crate) trait Get {
    fn get(&mut self, url: &str) -> Result<(u16, Vec<u8>), Failure>;
}

/// The real requests, through one agent kept by the worker.
pub(crate) struct Http {
    agent: ureq::Agent,
    media: ureq::Agent,
}

impl Http {
    pub(crate) fn new() -> Self {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(TIMEOUT))
            .user_agent(format!("SJK/{}", crate::build_info::VERSION))
            .https_only(true)
            .max_redirects(0)
            .http_status_as_error(false)
            .build()
            .into();
        let media = ureq::Agent::with_parts(
            ureq::Agent::config_builder()
                .timeout_global(Some(TIMEOUT))
                .user_agent(format!("SJK/{}", crate::build_info::VERSION))
                .https_only(true)
                .max_redirects(0)
                .http_status_as_error(false)
                // A proxy can resolve names elsewhere, defeating the public-address check.
                .proxy(None)
                .build(),
            ureq::unversioned::transport::DefaultConnector::default(),
            PublicResolver,
        );
        Self { agent, media }
    }
    /// One generic request; policy is checked again immediately before I/O.
    pub(crate) fn media_request(
        &mut self,
        url: &str,
        page: bool,
    ) -> Result<super::discover::Response, Failure> {
        let checked = super::sources::checked_url(url).ok_or(Failure::Refused)?;
        if !super::enabled() || !super::sources::permits(&checked) {
            return Err(Failure::Refused);
        }
        let video = super::video::available();
        let mut response = self
            .media
            .get(url)
            .header("Accept", accept(video, page))
            .call()
            .map_err(|error| Failure::Network(error.to_string()))?;
        let status = response.status().as_u16();
        let location = response
            .headers()
            .get("location")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if refused_before_body(video, &content_type) {
            return Err(Failure::Refused);
        }
        let limit = if content_type.starts_with("text/html") {
            super::discover::HTML_BYTES as u64
        } else {
            MAX_BYTES
        };
        let bytes = if status == 200 {
            response
                .body_mut()
                .with_config()
                .limit(limit)
                .read_to_vec()
                .map_err(|error| match error {
                    ureq::Error::BodyExceedsLimit(_) => Failure::TooLarge,
                    error => Failure::Network(error.to_string()),
                })?
        } else {
            Vec::new()
        };
        Ok(super::discover::Response {
            status,
            location,
            content_type,
            bytes,
        })
    }
}

impl Get for Http {
    fn get(&mut self, url: &str) -> Result<(u16, Vec<u8>), Failure> {
        if !super::enabled() || !super::sources::trusted_host("giphy.com") {
            return Err(Failure::Refused);
        }
        let mut response = self
            .agent
            .get(url)
            .header("Accept", "image/gif")
            .call()
            .map_err(|error| Failure::Network(error.to_string()))?;
        let status = response.status().as_u16();
        if status != 200 {
            return Ok((status, Vec::new()));
        }
        // For an id it does not know, GIPHY answers `giphy.gif` with 200 and a picture
        // of its own saying so, marked by this header (seen 10/10/2026): that is a 404.
        if response.headers().contains_key(NOT_FOUND_HEADER) {
            return Ok((404, Vec::new()));
        }
        let body = response
            .body_mut()
            .with_config()
            .limit(MAX_BYTES)
            .read_to_vec()
            .map_err(|error| match error {
                ureq::Error::BodyExceedsLimit(_) => Failure::TooLarge,
                error => Failure::Network(error.to_string()),
            })?;
        Ok((status, body))
    }
}

/// The file of `id`'s GIF: the small rendition, or the original when GIPHY has no
/// small one; for a picker's tile (`preview`), the preview before them.
pub(crate) fn fetch(get: &mut impl Get, id: &GifId, preview: bool) -> Result<Vec<u8>, Failure> {
    let renditions: &[Rendition] = if preview {
        &[Rendition::Preview, Rendition::Small, Rendition::Original]
    } else {
        &[Rendition::Small, Rendition::Original]
    };
    let mut answer = Err(Failure::Status(404));
    for rendition in renditions {
        answer = one(get, &link::media_url(id, *rendition));
        if answer != Err(Failure::Status(404)) {
            break;
        }
    }
    answer
}

/// One request to `url`, which must be one SJK fetches.
fn one(get: &mut impl Get, url: &str) -> Result<Vec<u8>, Failure> {
    if !link::fetchable(url) {
        return Err(Failure::Refused);
    }
    let (status, body) = get.get(url)?;
    if status != 200 {
        return Err(Failure::Status(status));
    }
    if body.len() as u64 > MAX_BYTES {
        return Err(Failure::TooLarge);
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What a scripted address answers.
    type Answer = Result<(u16, Vec<u8>), Failure>;

    /// Answers by address, keeping what was asked.
    struct Script {
        answers: Vec<(String, Answer)>,
        asked: Vec<String>,
    }

    impl Get for Script {
        fn get(&mut self, url: &str) -> Answer {
            self.asked.push(url.to_owned());
            self.answers
                .iter()
                .find(|(at, _)| at == url)
                .map_or(Ok((404, Vec::new())), |(_, answer)| answer.clone())
        }
    }

    fn script(answers: Vec<(String, Answer)>) -> Script {
        Script {
            answers,
            asked: Vec::new(),
        }
    }

    fn id() -> GifId {
        GifId::new("3o7TKSjRrfIPjeiVyM").unwrap()
    }

    #[test]
    fn the_small_rendition_is_asked_first() {
        let small = link::media_url(&id(), Rendition::Small);
        let mut get = script(vec![(small.clone(), Ok((200, b"GIF89a".to_vec())))]);
        assert_eq!(fetch(&mut get, &id(), false), Ok(b"GIF89a".to_vec()));
        assert_eq!(get.asked, [small]);
    }

    #[test]
    fn the_original_only_when_the_small_one_is_missing() {
        let small = link::media_url(&id(), Rendition::Small);
        let original = link::media_url(&id(), Rendition::Original);
        let mut get = script(vec![(original.clone(), Ok((200, b"GIF87a".to_vec())))]);
        assert_eq!(fetch(&mut get, &id(), false), Ok(b"GIF87a".to_vec()));
        assert_eq!(get.asked, [small.clone(), original.clone()]);
        // Any other refusal, a redirect included, ends there.
        for status in [301, 302, 403, 500] {
            let mut get = script(vec![(small.clone(), Ok((status, Vec::new())))]);
            assert_eq!(fetch(&mut get, &id(), false), Err(Failure::Status(status)));
            assert_eq!(get.asked, std::slice::from_ref(&small));
        }
        let mut get = script(vec![(small.clone(), Err(Failure::TooLarge))]);
        assert_eq!(fetch(&mut get, &id(), false), Err(Failure::TooLarge));
        assert_eq!(get.asked.len(), 1);
        // Neither there.
        let mut get = script(Vec::new());
        assert_eq!(fetch(&mut get, &id(), false), Err(Failure::Status(404)));
        assert_eq!(get.asked, [small, original]);
    }

    #[test]
    fn a_tile_asks_the_preview_first() {
        let preview = link::media_url(&id(), Rendition::Preview);
        let small = link::media_url(&id(), Rendition::Small);
        let mut get = script(vec![(preview.clone(), Ok((200, b"GIF89a".to_vec())))]);
        assert_eq!(fetch(&mut get, &id(), true), Ok(b"GIF89a".to_vec()));
        assert_eq!(get.asked, std::slice::from_ref(&preview));
        // No preview: the small one, as a message's GIF.
        let mut get = script(vec![(small.clone(), Ok((200, b"GIF87a".to_vec())))]);
        assert_eq!(fetch(&mut get, &id(), true), Ok(b"GIF87a".to_vec()));
        assert_eq!(get.asked, [preview, small]);
    }

    #[test]
    fn a_body_over_the_cap_is_refused() {
        let small = link::media_url(&id(), Rendition::Small);
        let big = vec![0_u8; MAX_BYTES as usize + 1];
        let mut get = script(vec![(small, Ok((200, big)))]);
        assert_eq!(fetch(&mut get, &id(), false), Err(Failure::TooLarge));
    }

    /// One real GIF from GIPHY, to see that the addresses SJK makes are served:
    /// `cargo test -p sjk-viewer chat_gifs::fetch::tests::a_real_gif -- --ignored`.
    #[test]
    #[ignore = "fetches one GIF from GIPHY over the network"]
    fn a_real_gif_from_giphy() {
        let mut http = Http::new();
        let started = std::time::Instant::now();
        let bytes = fetch(&mut http, &id(), false).expect("fetched");
        let gif = super::super::decode::decode(&bytes).expect("decoded");
        println!(
            "{} bytes in {:?}: {}x{}, {} frames, {} ms",
            bytes.len(),
            started.elapsed(),
            gif.size[0],
            gif.size[1],
            gif.frames.len(),
            gif.length_ms
        );
        assert!(gif.frames.len() > 1);
        let missing = GifId::new("notarealid123XYZ").unwrap();
        assert_eq!(fetch(&mut http, &missing, false), Err(Failure::Status(404)));
    }
}

/// The generic media requests' Accept header: MP4 only while clips can be played
/// (`video`, [`super::video::available`]).
fn accept(video: bool, page: bool) -> &'static str {
    match (video, page) {
        (true, true) => "image/gif, image/webp, video/mp4, text/html;q=0.5",
        (true, false) => "image/gif, image/webp, video/mp4",
        (false, true) => "image/gif, image/webp, text/html;q=0.5",
        (false, false) => "image/gif, image/webp",
    }
}

/// Whether a response of `content_type` is refused before its body is read: a clip
/// that cannot be played here (`video` false) is not downloaded at all.
fn refused_before_body(video: bool, content_type: &str) -> bool {
    !video && content_type.starts_with("video/")
}

/// Filter the actual addresses handed to the connector, avoiding a separate DNS
/// preflight/rebinding race. IPv6 is restricted to native global unicast (no mapped
/// IPv4, NAT64 or other transition addresses); no local/network services are fetched.
#[derive(Debug)]
struct PublicResolver;
impl ureq::unversioned::resolver::Resolver for PublicResolver {
    fn resolve(
        &self,
        uri: &ureq::http::Uri,
        config: &ureq::config::Config,
        timeout: ureq::unversioned::transport::NextTimeout,
    ) -> Result<ureq::unversioned::resolver::ResolvedSocketAddrs, ureq::Error> {
        use ureq::unversioned::resolver::DefaultResolver;
        let found = DefaultResolver::default().resolve(uri, config, timeout)?;
        let mut allowed = self.empty();
        for address in found.iter().filter(|address| public_ip(address.ip())) {
            allowed.push(*address);
        }
        if allowed.is_empty() {
            return Err(ureq::Error::HostNotFound);
        }
        Ok(allowed)
    }
}
fn public_ip(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(ip) => {
            let [a, b, _, _] = ip.octets();
            !ip.is_private()
                && !ip.is_loopback()
                && !ip.is_link_local()
                && !ip.is_broadcast()
                && !ip.is_documentation()
                && !ip.is_unspecified()
                && a != 0
                && a < 224
                && !(a == 100 && (64..=127).contains(&b))
                && !(a == 198 && matches!(b, 18 | 19))
                && !(a == 192 && b == 0)
        }
        std::net::IpAddr::V6(ip) => {
            let first = ip.segments()[0];
            (0x2000..=0x3fff).contains(&first)
                && first != 0x2001
                && first != 0x2002
                && !ip.is_multicast()
        }
    }
}
#[cfg(test)]
mod public_address_tests {
    #[test]
    fn local_and_transition_addresses_are_refused() {
        for ip in [
            "127.0.0.1",
            "10.0.0.1",
            "169.254.169.254",
            "100.64.0.1",
            "192.168.1.2",
            "::1",
            "::ffff:127.0.0.1",
            "64:ff9b::7f00:1",
            "2002:7f00:1::",
            "2001:db8::1",
        ] {
            assert!(!super::public_ip(ip.parse().unwrap()), "{ip}");
        }
        for ip in ["8.8.8.8", "2606:4700:4700::1111"] {
            assert!(super::public_ip(ip.parse().unwrap()), "{ip}");
        }
    }
}

#[cfg(test)]
mod video_policy_tests {
    #[test]
    fn without_clips_video_is_neither_asked_for_nor_downloaded() {
        assert!(!super::accept(false, true).contains("video"));
        assert!(super::accept(true, true).contains("video/mp4"));
        assert!(super::accept(false, true).contains("image/webp"));
        // After a page, its media is asked for without accepting another page.
        assert!(!super::accept(true, false).contains("text/html"));
        assert!(!super::accept(false, false).contains("text/html"));
        assert!(super::refused_before_body(false, "video/mp4"));
        assert!(!super::refused_before_body(true, "video/mp4"));
        assert!(!super::refused_before_body(false, "image/webp"));
        assert!(!super::refused_before_body(
            false,
            "text/html; charset=utf-8"
        ));
        // Off by default: the real requests run without video.
        let _switch = crate::chat_gifs::video::SWITCH
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(!crate::chat_gifs::video::available());
    }
}
