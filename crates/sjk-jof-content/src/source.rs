//! Where the manifest and the files come from: the table of JoF's repository
//! hosts. Supporting another host is a new entry here, not new code.

/// One host of JoF's repository.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Source {
    /// The short name the player chooses it by (`jof.contentSource`) and logs show.
    pub name: &'static str,
    pub manifest: &'static str,
    /// The repository's root: a file is `<files>/<localfolder>/<name>`.
    pub files: &'static str,
}

/// JoF Cloud (`cloud.jof.app`, S3 behind CloudFront): an hourly byte-for-byte
/// copy of the origin's manifest and files, run by JoF to take the load off the
/// origin. JoF's launcher tries it first (`lib/models/repository_source.dart`,
/// 16/10/2026).
pub const MIRROR: Source = Source {
    name: "mirror",
    manifest: "https://cloud.jof.app/cloud/manifest/getclouddata.json",
    files: "https://cloud.jof.app/cloud/repository",
};

/// jofhub.eu, the repository itself: used when the mirror is down or has not
/// copied a new file yet.
pub const ORIGIN: Source = Source {
    name: "origin",
    manifest: "https://api.jofhub.eu/cloud/filemanager/getclouddata",
    files: "https://jofhub.eu/apps/cloud/repository",
};

/// Where a collection's hosts are listed: the SJK hub's `GET /v1/content-sources`
/// (`PROTOCOL.md` in the hub, "Content sources"), so SJK can move a collection to
/// another host, or switch it off, without a release.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Directory {
    /// The hub's address (`cl_hubUrl`); empty when the player has none.
    pub hub: String,
    /// The collection's key in the hub's answer: `jof` or `sith`.
    pub collection: &'static str,
    /// Keep only the host of this name (the player's choice, `jof.contentSource`), while
    /// the hub still lists it.
    pub only: Option<String>,
}

/// The hosts of `collection` in the hub's answer (`{"jof":[{"name","manifest",
/// "files"}],..}`), in its order; `None` when the answer does not say. An empty list
/// means the collection is off.
pub fn listed(answer: &[u8], collection: &str) -> Option<Vec<Source>> {
    let answer: serde_json::Value = serde_json::from_slice(answer).ok()?;
    let mut sources = Vec::new();
    for host in answer.get(collection)?.as_array()? {
        let text = |key: &str| host.get(key).and_then(serde_json::Value::as_str);
        let (name, manifest, files) = (text("name")?, text("manifest")?, text("files")?);
        let web = |url: &str| url.starts_with("https://") || url.starts_with("http://");
        if name.is_empty() || !web(manifest) || !web(files) {
            return None;
        }
        sources.push(Source {
            name: interned(name),
            manifest: interned(manifest),
            files: interned(files.trim_end_matches('/')),
        });
    }
    Some(sources)
}

/// `text` kept for the program's life; the same text is kept once, so asking the hub
/// again does not grow it.
fn interned(text: &str) -> &'static str {
    static KEPT: std::sync::Mutex<Vec<&'static str>> = std::sync::Mutex::new(Vec::new());
    let mut kept = KEPT.lock().unwrap_or_else(|error| error.into_inner());
    if let Some(known) = kept.iter().find(|known| **known == text) {
        return known;
    }
    let leaked: &'static str = Box::leak(text.to_owned().into_boxed_str());
    kept.push(leaked);
    leaked
}

/// Every known host, in the order `auto` tries them.
pub const SOURCES: &[Source] = &[MIRROR, ORIGIN];

/// The player's choice of host (`jof.contentSource`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SourceChoice {
    /// The mirror, then the origin.
    #[default]
    Auto,
    /// One host only.
    Only(Source),
}

impl SourceChoice {
    /// `auto`, or a host's name; anything else is `auto`.
    pub fn parse(value: &str) -> Self {
        SOURCES
            .iter()
            .find(|source| source.name.eq_ignore_ascii_case(value.trim()))
            .map_or(Self::Auto, |source| Self::Only(*source))
    }

    /// The hosts to try, in order.
    pub fn sources(self) -> Vec<Source> {
        match self {
            Self::Auto => SOURCES.to_vec(),
            Self::Only(source) => vec![source],
        }
    }
}

impl Source {
    /// The file's address, each path segment percent-encoded on its own, so
    /// names with spaces, braces, quotes or `&` survive.
    pub fn file_url(&self, asset: &crate::Asset) -> String {
        let mut url = self.files.to_owned();
        for segment in asset
            .repository_path
            .iter()
            .chain(std::iter::once(&asset.name))
        {
            url.push('/');
            encode_segment(segment, &mut url);
        }
        url
    }
}

fn encode_segment(segment: &str, out: &mut String) {
    for byte in segment.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_addresses_encode_each_segment() {
        let manifest = crate::Manifest::parse(crate::manifest::tests::SAMPLE.as_bytes()).unwrap();
        let asset = &manifest.group(2).unwrap().assets[0];
        assert_eq!(
            MIRROR.file_url(asset),
            "https://cloud.jof.app/cloud/repository/assets/%7BJoF%7D%27s-GWing.pk3"
        );
        assert_eq!(
            ORIGIN.file_url(&manifest.group(2).unwrap().assets[1]),
            "https://jofhub.eu/apps/cloud/repository/assets/maps%20two.pk3"
        );
    }

    #[test]
    fn the_choice_names_a_host_or_tries_them_all() {
        assert_eq!(SourceChoice::parse("auto").sources(), [MIRROR, ORIGIN]);
        assert_eq!(SourceChoice::parse("Origin").sources(), [ORIGIN]);
        assert_eq!(SourceChoice::parse("nonsense"), SourceChoice::Auto);
    }
}
