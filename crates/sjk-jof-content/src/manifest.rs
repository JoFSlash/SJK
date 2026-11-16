//! The repository's manifest (`getclouddata`): its asset groups and their files.
//!
//! The manifest is JSON, `{"status": ..., "data": {"assetgroups": [...], ...}}`.
//! Only `assetgroups` is read: `clientgroups` installs JoF's EternalJK client,
//! `modgroups` the JA+ and other mods' client plugins (SJK has its own support
//! for those servers) and `othergroups` holds withdrawn files. Ids and sizes
//! come as numbers or as strings, as JoF's launcher accepts them
//! (`lib/models/asset.dart`).

use serde_json::Value;

/// The manifest's asset groups, in the repository's order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Manifest {
    pub groups: Vec<Group>,
    /// Files left out because their name, folder or digest was unusable.
    pub skipped: usize,
}

/// One group of files, chosen or left out as a whole.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub kind: GroupKind,
    pub assets: Vec<Asset>,
}

/// How the repository rates a group (`type`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupKind {
    /// Needed to play on JoF's servers; chosen by default.
    Required,
    /// Chosen by default.
    Recommended,
    /// Left out unless the player chooses it.
    Optional,
}

impl GroupKind {
    /// Whether a player who has not chosen yet gets this group.
    pub fn default_chosen(self) -> bool {
        !matches!(self, Self::Optional)
    }

    /// The word the card shows.
    pub fn label(self) -> &'static str {
        match self {
            Self::Required => "Required",
            Self::Recommended => "Recommended",
            Self::Optional => "Optional",
        }
    }
}

/// One file of a group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Asset {
    pub id: u32,
    /// The file's name, a plain PK3 name with no folder.
    pub name: String,
    pub size: u64,
    pub md5: [u8; 16],
    pub description: String,
    pub target: Target,
    /// The folders under the repository's root the file is served from
    /// (`localfolder`), already split and checked.
    pub repository_path: Vec<String>,
    /// The repository's date for the file, as it writes it.
    pub updated: String,
}

/// Where the launcher puts a file (`targetfolder`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    /// The game's `base`: SJK keeps these in its JoF folder's `base`.
    Base,
    /// Another folder (`EternalJK`): content for JoF's own client, which SJK
    /// lists but does not fetch.
    Other(String),
}

/// Why a manifest could not be read.
#[derive(Debug, PartialEq, Eq)]
pub enum ManifestError {
    Json(String),
    /// The JSON has no `data.assetgroups` list.
    NoGroups,
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "the manifest is not valid JSON: {error}"),
            Self::NoGroups => formatter.write_str("the manifest lists no asset groups"),
        }
    }
}

impl std::error::Error for ManifestError {}

impl Manifest {
    /// Read the manifest. Groups of a kind SJK does not know (`DEPRECATED`) are
    /// left out, and so is a file whose name or folder is not a plain one, so
    /// nothing in the manifest can name a path outside SJK's folder.
    pub fn parse(bytes: &[u8]) -> Result<Self, ManifestError> {
        let root: Value = serde_json::from_slice(bytes)
            .map_err(|error| ManifestError::Json(error.to_string()))?;
        let groups = root
            .get("data")
            .and_then(|data| data.get("assetgroups"))
            .and_then(Value::as_array)
            .ok_or(ManifestError::NoGroups)?;
        let mut manifest = Manifest::default();
        for group in groups {
            let kind = match text(group, "type").to_ascii_uppercase().as_str() {
                "REQUIRED" => GroupKind::Required,
                "RECOMMENDED" => GroupKind::Recommended,
                "OPTIONAL" => GroupKind::Optional,
                _ => continue,
            };
            let mut assets = Vec::new();
            for asset in group
                .get("assets")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                match parse_asset(asset) {
                    Some(asset) => assets.push(asset),
                    None => manifest.skipped += 1,
                }
            }
            manifest.groups.push(Group {
                id: number(group, "id") as u32,
                name: text(group, "name"),
                description: text(group, "description"),
                kind,
                assets,
            });
        }
        Ok(manifest)
    }

    pub fn group(&self, id: u32) -> Option<&Group> {
        self.groups.iter().find(|group| group.id == id)
    }

    /// Every file of every group, with its group.
    pub fn assets(&self) -> impl Iterator<Item = (&Group, &Asset)> {
        self.groups
            .iter()
            .flat_map(|group| group.assets.iter().map(move |asset| (group, asset)))
    }
}

impl Group {
    /// Bytes of the files SJK would fetch (those for `base`).
    pub fn size(&self) -> u64 {
        self.assets
            .iter()
            .filter(|asset| asset.target == Target::Base)
            .map(|asset| asset.size)
            .sum()
    }
}

fn parse_asset(value: &Value) -> Option<Asset> {
    let name = text(value, "name");
    if !plain_name(&name) || !name.to_ascii_lowercase().ends_with(".pk3") {
        return None;
    }
    let folder = text(value, "localfolder");
    let repository_path: Vec<String> = folder
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(str::to_owned)
        .collect();
    if !repository_path.iter().all(|segment| plain_name(segment)) {
        return None;
    }
    let target = text(value, "targetfolder");
    let target = if target.is_empty() || target.eq_ignore_ascii_case("base") {
        Target::Base
    } else {
        Target::Other(target)
    };
    Some(Asset {
        id: number(value, "id") as u32,
        size: number(value, "size"),
        md5: crate::parse_md5(&text(value, "hash"))?,
        description: text(value, "description"),
        target,
        repository_path,
        updated: text(value, "lastupdatestr"),
        name,
    })
}

/// One file or folder name: no separators, no drive, not `.` or `..`, no
/// characters Windows refuses.
pub(crate) fn plain_name(name: &str) -> bool {
    !name.is_empty()
        && name.trim() == name
        && name != "."
        && name != ".."
        && !name.ends_with('.')
        && !name.chars().any(|c| {
            c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
        })
}

fn text(value: &Value, key: &str) -> String {
    match value.get(key) {
        Some(Value::String(text)) => text.trim().to_owned(),
        Some(Value::Number(number)) => number.to_string(),
        _ => String::new(),
    }
}

fn number(value: &Value, key: &str) -> u64 {
    match value.get(key) {
        Some(Value::Number(number)) => number.as_u64().unwrap_or(0),
        Some(Value::String(text)) => text.trim().parse().unwrap_or(0),
        _ => 0,
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A manifest shaped as the repository's (16/10/2026), cut down.
    pub(crate) const SAMPLE: &str = r#"{"status":"OK","data":{"assetgroups":[
        {"id":2,"name":"JoF Essential Maps","description":"Pack of essential maps","category":"ASSETS","type":"RECOMMENDED","assets":[
            {"id":816,"name":"{JoF}'s-GWing.pk3","size":51516672,"hash":"7d26fb356d81ef10403f290d504517df","groupid":2,"description":"JoF Golden Wing V1","targetfolder":"Base","localfolder":"assets","lastupdatestr":"08.08.2025"},
            {"id":"817","name":"maps two.pk3","size":"12","hash":"0cc175b9c0f1b6a831c399e269772661","targetfolder":"Base","localfolder":"assets"}
        ]},
        {"id":6,"name":"JoF Cosmetics","description":"Cosmetics","type":"OPTIONAL","assets":[
            {"id":1640,"name":"JoF_CosmeticMod_Splash.pk3","size":4935370,"hash":"feb19b7add98ae53431523015aa91cfe","targetfolder":"EternalJK","localfolder":"assets"},
            {"id":1641,"name":"../evil.pk3","size":1,"hash":"feb19b7add98ae53431523015aa91cfe","targetfolder":"Base","localfolder":"assets"},
            {"id":1642,"name":"evil.pk3","size":1,"hash":"feb19b7add98ae53431523015aa91cfe","targetfolder":"Base","localfolder":"../.."},
            {"id":1643,"name":"readme.txt","size":1,"hash":"feb19b7add98ae53431523015aa91cfe","targetfolder":"Base","localfolder":"assets"},
            {"id":1644,"name":"nohash.pk3","size":1,"hash":"","targetfolder":"Base","localfolder":"assets"}
        ]},
        {"id":0,"name":"Deprecated","type":"DEPRECATED","assets":[]}
    ],"clientgroups":[],"modgroups":[]}}"#;

    #[test]
    fn the_repository_manifest_reads_its_groups_and_files() {
        let manifest = Manifest::parse(SAMPLE.as_bytes()).unwrap();
        assert_eq!(manifest.groups.len(), 2, "DEPRECATED is left out");
        let maps = manifest.group(2).unwrap();
        assert_eq!(maps.kind, GroupKind::Recommended);
        assert!(maps.kind.default_chosen());
        assert_eq!(maps.assets[0].name, "{JoF}'s-GWing.pk3");
        assert_eq!(maps.assets[0].repository_path, ["assets"]);
        assert_eq!(maps.assets[0].updated, "08.08.2025");
        assert_eq!(maps.assets[1].id, 817, "ids given as strings");
        assert_eq!(maps.assets[1].size, 12);
        assert_eq!(maps.size(), 51_516_684);
        let cosmetics = manifest.group(6).unwrap();
        assert!(!cosmetics.kind.default_chosen());
        assert_eq!(
            cosmetics.assets[0].target,
            Target::Other("EternalJK".into())
        );
        assert_eq!(cosmetics.size(), 0, "EternalJK files are not fetched");
    }

    #[test]
    fn unsafe_names_folders_and_digests_are_left_out() {
        let manifest = Manifest::parse(SAMPLE.as_bytes()).unwrap();
        assert_eq!(manifest.group(6).unwrap().assets.len(), 1);
        assert_eq!(manifest.skipped, 4);
        assert!(!plain_name("a/b.pk3"));
        assert!(!plain_name("C:x.pk3"));
        assert!(!plain_name(".."));
        assert!(plain_name("{JoF}'s-GWing.pk3"));
    }

    #[test]
    fn a_page_that_is_not_a_manifest_is_refused() {
        assert_eq!(
            Manifest::parse(br#"{"status":"OK"}"#),
            Err(ManifestError::NoGroups)
        );
        assert!(matches!(
            Manifest::parse(b"<html>"),
            Err(ManifestError::Json(_))
        ));
    }
}
