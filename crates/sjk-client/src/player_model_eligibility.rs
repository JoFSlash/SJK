//! Mount-scoped player model restrictions from JoF EJK #126.
//!
//! This is player loading/picker policy, never NPC loading policy. Content
//! authors declare restrictions through metadata rather than built-in names.
use sjk_vfs::VirtualFileSystem;
use std::collections::BTreeSet;

/// Archived player blacklist, with whitespace/comma/semicolon-separated names.
pub const BLACKLIST_CVAR: &str = "cg_modelBlacklist";
/// Archived override for content-authored restrictions (on by default).
pub const METADATA_CVAR: &str = "cg_modelEligibility";

/// Restrictions parsed once for an immutable mounted content set.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PlayerModelEligibility {
    restricted: BTreeSet<String>,
}

impl PlayerModelEligibility {
    /// Read visible settings and merged blacklist files, respecting VFS priority.
    /// Missing, unreadable or oversized optional files are ignored, as in EJK.
    pub fn load(vfs: &VirtualFileSystem) -> Self {
        let mut policy = Self::default();
        for path in vfs.paths() {
            let name = path.as_str();
            let setting = name
                .strip_prefix("models/players/")
                .and_then(|name| name.strip_suffix("/settings.txt"))
                .filter(|name| valid_name(name));
            let list = name.starts_with("ext_data/modelblacklist/")
                && name.ends_with(".txt")
                && !name["ext_data/modelblacklist/".len()..].contains('/');
            if setting.is_none() && !list {
                continue;
            }
            let Ok(Some(asset)) = vfs.read(name) else {
                continue;
            };
            // EJK #126's MODEL_SETTINGS_FILE_SIZE/MODEL_BLACKLIST_FILE_SIZE.
            let limit = if setting.is_some() { 4096 } else { 16384 };
            if asset.bytes.is_empty() || asset.bytes.len() >= limit {
                continue;
            }
            let tokens = crate::catalog_tokens::tokenize(&String::from_utf8_lossy(&asset.bytes));
            if let Some(model) = setting {
                let mut restricted = false;
                for pair in tokens.windows(2) {
                    if pair[0].eq_ignore_ascii_case("notInMP") {
                        restricted = crate::team_info::legacy_atoi(pair[1].as_bytes()) != 0;
                    }
                }
                if restricted {
                    policy.restricted.insert(model.to_ascii_lowercase());
                }
            } else {
                policy.restricted.extend(
                    tokens
                        .into_iter()
                        .filter(|name| valid_name(name))
                        .map(|name| name.to_ascii_lowercase()),
                );
            }
        }
        policy
    }

    /// Whether a player may use this directory or model/skin value.
    ///
    /// `respect_metadata` can bypass pack-authored restrictions; the player's
    /// explicit blacklist still applies. Call only when selecting/loading an
    /// appearance, not during actor submission. NPC callers bypass this policy.
    pub fn allows(&self, model: &str, blacklist: &str, respect_metadata: bool) -> bool {
        let model = model_name(model);
        (!respect_metadata || !self.restricted.contains(&model.to_ascii_lowercase()))
            && !blacklist
                .split(|c: char| c.is_ascii_whitespace() || matches!(c, ',' | ';'))
                .filter(|entry| !entry.is_empty())
                .any(|entry| model_name(entry).eq_ignore_ascii_case(model))
    }
}

fn model_name(value: &str) -> &str {
    let value = value
        .get(..15)
        .filter(|prefix| prefix.eq_ignore_ascii_case("models/players/"))
        .map_or(value, |_| &value[15..]);
    value.split('/').next().unwrap_or(value)
}

fn valid_name(value: &str) -> bool {
    !value.is_empty()
        && !matches!(value, "." | "..")
        && !value
            .chars()
            .any(|c| c.is_control() || matches!(c, '/' | '\\'))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn metadata_and_pack_lists_merge_with_visible_file_priority() {
        let mut vfs = VirtualFileSystem::new();
        vfs.mount_memory(
            "first",
            [
                (
                    "models/players/declared/settings.txt",
                    b"// comment\nnotInMP 1".to_vec(),
                ),
                ("models/players/cleared/settings.txt", b"notInMP 1".to_vec()),
                (
                    "ext_data/modelblacklist/one.txt",
                    b"listed /* reason */\nshared".to_vec(),
                ),
                ("ext_data/modelblacklist/two.txt", b"other".to_vec()),
            ],
        )
        .unwrap();
        vfs.mount_memory(
            "override",
            [
                ("models/players/cleared/settings.txt", b"notInMP 0".to_vec()),
                ("ext_data/modelblacklist/one.txt", b"replacement".to_vec()),
            ],
        )
        .unwrap();
        let policy = PlayerModelEligibility::load(&vfs);
        for name in ["declared", "replacement", "other"] {
            assert!(!policy.allows(name, "", true), "{name}");
            assert!(policy.allows(name, "", false), "player override: {name}");
        }
        for name in ["cleared", "listed", "shared", "unlisted"] {
            assert!(policy.allows(name, "", true), "{name}");
        }
    }
    #[test]
    fn player_lists_match_whole_names_and_skin_suffixes_case_insensitively() {
        let policy = PlayerModelEligibility::default();
        for list in ["ALPHA/default, beta", "beta;alpha/red", "beta\nalpha"] {
            assert!(!policy.allows("models/players/Alpha", list, false));
            assert!(policy.allows("alphabet", list, true));
        }
        assert!(policy.allows("alpha", "beta", true));
    }
    #[test]
    fn oversized_and_invalid_optional_metadata_cannot_hide_other_models() {
        let mut vfs = VirtualFileSystem::new();
        vfs.mount_memory(
            "pack",
            [
                ("models/players/large/settings.txt", vec![b'x'; 4096]),
                ("models/players/valid/settings.txt", b"notInMP -1".to_vec()),
                (
                    "ext_data/modelblacklist/invalid.txt",
                    b"../elsewhere models/players/name { }".to_vec(),
                ),
            ],
        )
        .unwrap();
        let policy = PlayerModelEligibility::load(&vfs);
        assert!(policy.allows("large", "", true));
        assert!(!policy.allows("valid", "", true));
        assert!(policy.allows("name", "", true));
    }
}
