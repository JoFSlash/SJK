//! Bounded generic media discovery. Only trusted HTTPS hosts are requested; redirects
//! and HTML media references are checked independently. No scripts, cookies, API
//! credentials, referrer, browser session or authentication bypass are used.

use super::{
    fetch::{Failure, Http, MAX_BYTES},
    link::GifId,
    sources,
};
use scraper::{Html, Selector};
use url::Url;

const HOPS: usize = 4;
pub(crate) const HTML_BYTES: usize = 256 << 10;

pub(crate) struct Response {
    pub(crate) status: u16,
    pub(crate) location: Option<String>,
    pub(crate) content_type: String,
    pub(crate) bytes: Vec<u8>,
}
pub(crate) trait Request {
    fn request(&mut self, url: &str) -> Result<Response, Failure>;
}
impl Request for Http {
    fn request(&mut self, url: &str) -> Result<Response, Failure> {
        self.media_request(url)
    }
}
pub(crate) fn fetch(
    http: &mut impl Request,
    id: &GifId,
    revision: u64,
) -> Result<Vec<u8>, Failure> {
    resolve(
        http,
        id.as_str(),
        || super::enabled() && revision == sources::revision(),
        sources::permits,
        super::video::available(),
    )
}
/// Kept separate from HTTP to pin that rejected redirects/CDNs are never requested.
fn resolve(
    http: &mut impl Request,
    start: &str,
    current: impl Fn() -> bool,
    permits: impl Fn(&Url) -> bool,
    video: bool,
) -> Result<Vec<u8>, Failure> {
    let mut url = sources::checked_url(start).ok_or(Failure::Refused)?;
    let mut page_seen = false;
    for _ in 0..HOPS {
        if !current() {
            return Err(Failure::Refused);
        }
        if !permits(&url) {
            sources::blocked(url.host_str().unwrap_or_default());
            return Err(Failure::Refused);
        }
        let response = http.request(url.as_str())?;
        if !current() {
            return Err(Failure::Refused);
        }
        if matches!(response.status, 301 | 302 | 303 | 307 | 308) {
            let next = url
                .join(response.location.as_deref().ok_or(Failure::Refused)?)
                .map_err(|_| Failure::Refused)?;
            url = sources::checked_url(next.as_str()).ok_or(Failure::Refused)?;
            continue;
        }
        if response.status != 200 {
            return Err(Failure::Status(response.status));
        }
        if response.bytes.len() as u64 > MAX_BYTES {
            return Err(Failure::TooLarge);
        }
        if super::decode::is_gif(&response.bytes)
            || super::decode::is_webp(&response.bytes)
            || super::video::is_mp4(&response.bytes)
        {
            return Ok(response.bytes);
        }
        if page_seen
            || !response.content_type.starts_with("text/html")
            || response.bytes.len() > HTML_BYTES
        {
            return Err(Failure::Refused);
        }
        page_seen = true;
        url = media_link(&url, &response.bytes, video).ok_or(Failure::Refused)?;
    }
    Err(Failure::Refused)
}
/// Standard HTML media declarations only. `<base>` and JavaScript never affect URLs.
/// Without `video` (no MP4 decoding here) the page's picture is taken instead of its
/// video.
fn media_link(base: &Url, bytes: &[u8], video: bool) -> Option<Url> {
    let text = std::str::from_utf8(bytes).ok()?;
    let html = Html::parse_document(text);
    let selector =
        Selector::parse("meta[property], video[src], video source[src], link[rel='image_src']")
            .ok()?;
    let mut image = None;
    for node in html.select(&selector) {
        let value = node.value();
        let candidate = match value.name() {
            "meta" => match value.attr("property")? {
                "og:video" | "og:video:url" | "og:video:secure_url" => value.attr("content"),
                "og:image" | "og:image:url" | "og:image:secure_url" => {
                    image = value.attr("content");
                    continue;
                }
                _ => continue,
            },
            "link" => {
                image = value.attr("href");
                continue;
            }
            _ => value.attr("src"),
        };
        if !video {
            continue;
        }
        if let Some(url) = candidate
            .and_then(|value| base.join(value).ok())
            .and_then(|url| sources::checked_url(url.as_str()))
        {
            return Some(url);
        }
    }
    image
        .and_then(|value| base.join(value).ok())
        .and_then(|url| sources::checked_url(url.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Script {
        asked: Vec<String>,
        answers: std::collections::VecDeque<Response>,
    }
    impl Request for Script {
        fn request(&mut self, url: &str) -> Result<Response, Failure> {
            self.asked.push(url.to_owned());
            Ok(self.answers.pop_front().expect("scripted answer"))
        }
    }
    fn reply(status: u16, location: Option<&str>, content_type: &str, bytes: &[u8]) -> Response {
        Response {
            status,
            location: location.map(str::to_owned),
            content_type: content_type.to_owned(),
            bytes: bytes.to_vec(),
        }
    }
    #[test]
    fn untrusted_initial_links_redirects_and_media_hosts_are_never_requested() {
        let mut http = Script {
            asked: vec![],
            answers: [].into(),
        };
        assert!(
            resolve(
                &mut http,
                "https://example.org/a.gif",
                || true,
                |_| false,
                true
            )
            .is_err()
        );
        assert!(http.asked.is_empty());
        for response in [
            reply(302, Some("https://cdn.example.org/a.gif"), "", b""),
            reply(
                200,
                None,
                "text/html",
                br#"<meta property="og:video" content="https://cdn.example.org/a.mp4">"#,
            ),
        ] {
            let mut http = Script {
                asked: vec![],
                answers: [response].into(),
            };
            assert!(
                resolve(
                    &mut http,
                    "https://example.org/page",
                    || true,
                    |url| url.host_str() == Some("example.org"),
                    true
                )
                .is_err()
            );
            assert_eq!(http.asked, ["https://example.org/page"]);
        }
    }
    #[test]
    fn trusted_page_media_resolves_but_revoked_jobs_do_not_start() {
        let page = reply(
            200,
            None,
            "text/html",
            br#"<video><source src="/clip.mp4?x=1&amp;y=2"></video>"#,
        );
        let mut http = Script {
            asked: vec![],
            answers: [page, reply(200, None, "video/mp4", b"0000ftypvideo")].into(),
        };
        assert_eq!(
            resolve(
                &mut http,
                "https://example.org/page",
                || true,
                |_| true,
                true
            )
            .unwrap(),
            b"0000ftypvideo"
        );
        assert_eq!(http.asked[1], "https://example.org/clip.mp4?x=1&y=2");
        http.asked.clear();
        assert!(
            resolve(
                &mut http,
                "https://example.org/page",
                || false,
                |_| true,
                true
            )
            .is_err()
        );
        assert!(http.asked.is_empty());
    }
    #[test]
    fn discovery_ignores_script_and_base_tags() {
        let base = Url::parse("https://example.org/a/page").unwrap();
        assert_eq!(media_link(&base, br#"<base href="https://other.example/"><meta property="og:video" content="clip.mp4">"#, true).unwrap().as_str(), "https://example.org/a/clip.mp4");
        assert!(media_link(&base, b"<script>fetch('/private-api')</script>", true).is_none());
    }
    #[test]
    fn without_mp4_decoding_the_page_picture_is_taken() {
        let base = Url::parse("https://example.org/a/page").unwrap();
        let page = br#"<meta property="og:video" content="clip.mp4"><meta property="og:image" content="still.webp">"#;
        assert_eq!(
            media_link(&base, page, true).unwrap().as_str(),
            "https://example.org/a/clip.mp4"
        );
        assert_eq!(
            media_link(&base, page, false).unwrap().as_str(),
            "https://example.org/a/still.webp"
        );
        assert!(media_link(&base, br#"<video src="clip.mp4"></video>"#, false).is_none());
    }
    #[test]
    fn with_clips_off_a_page_resolves_to_its_picture_not_its_video() {
        let page = reply(
            200,
            None,
            "text/html",
            br#"<meta property="og:video" content="/clip.mp4"><meta property="og:image" content="/still.gif">"#,
        );
        let mut http = Script {
            asked: vec![],
            answers: [page, reply(200, None, "image/gif", b"GIF89a")].into(),
        };
        let bytes = resolve(
            &mut http,
            "https://example.org/page",
            || true,
            |_| true,
            false,
        );
        assert_eq!(bytes.unwrap(), b"GIF89a");
        assert_eq!(http.asked[1], "https://example.org/still.gif");
        // The off switch also reaches the real resolutions: off by default.
        let _switch = super::super::video::SWITCH
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(!super::super::video::available());
    }
    #[test]
    fn webp_media_found_on_a_page_or_linked_directly_is_kept() {
        let webp = crate::chat_gifs::decode::test_webp([8, 8], 2, 50);
        let page = reply(
            200,
            None,
            "text/html",
            br#"<meta property="og:image" content="https://example.org/a.webp">"#,
        );
        let mut http = Script {
            asked: vec![],
            answers: [page, reply(200, None, "image/webp", &webp)].into(),
        };
        let bytes = resolve(
            &mut http,
            "https://example.org/page",
            || true,
            |_| true,
            true,
        )
        .unwrap();
        assert_eq!(http.asked[1], "https://example.org/a.webp");
        assert_eq!(
            crate::chat_gifs::decode::decode(&bytes)
                .unwrap()
                .frames
                .len(),
            2
        );
        let mut http = Script {
            asked: vec![],
            answers: [reply(200, None, "image/webp", &webp)].into(),
        };
        assert_eq!(
            resolve(
                &mut http,
                "https://example.org/a.webp",
                || true,
                |_| true,
                true
            )
            .unwrap(),
            webp
        );
    }
}
