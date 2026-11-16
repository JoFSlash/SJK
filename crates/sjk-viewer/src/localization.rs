//! Retail string-table localization kept outside rendering/UI layout code.

use sjk_vfs::VirtualFileSystem;
use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct Localization {
    pub(crate) strings: HashMap<String, String>,
}

impl Localization {
    pub(crate) fn load(vfs: &VirtualFileSystem) -> Self {
        let mut localization = Self::default();
        for path in [
            "strings/english/mp_ingame.str",
            "strings/english/mp_svgame.str",
        ] {
            let Some(asset) = vfs.read(path).ok().flatten() else {
                continue;
            };
            let text = String::from_utf8_lossy(&asset.bytes);
            sjk_client::string_table::parse_into(&mut localization.strings, &text);
        }
        // Every table's entries as `SE_GetStringTextString` names them (`<FILE>_<KEY>`),
        // the map's own among them: what a `cps` centre print, such as a siege map's
        // `target_print`, refers to.
        let tables: Vec<String> = vfs
            .list_files("strings/english", ".str")
            .into_iter()
            .map(|name| format!("strings/english/{name}"))
            .collect();
        let paths: Vec<&str> = tables.iter().map(String::as_str).collect();
        // `cps` asks in upper case; the string package matches references without it.
        for (key, value) in sjk_client::string_table::load_referenced(vfs, &paths) {
            localization
                .strings
                .entry(key.to_ascii_uppercase())
                .or_insert(value);
        }
        localization
    }

    pub(crate) fn translate(&self, text: &str) -> String {
        let mut translated = String::with_capacity(text.len());
        let mut remaining = text;
        while let Some(position) = remaining.find("@@@") {
            translated.push_str(&remaining[..position]);
            let token = &remaining[position + 3..];
            let length = token
                .bytes()
                .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
                .count();
            if length == 0 {
                translated.push_str("@@@");
                remaining = token;
                continue;
            }
            let key = &token[..length];
            translated.push_str(self.strings.get(key).map_or(key, String::as_str));
            remaining = &token[length..];
        }
        translated.push_str(remaining);
        translated
    }
}

#[cfg(test)]
mod tests {
    use super::Localization;
    use sjk_vfs::VirtualFileSystem;

    #[test]
    fn a_map_table_entry_is_found_by_file_and_key() {
        let mut vfs = VirtualFileSystem::new();
        vfs.mount_memory(
            "maps.pk3",
            [(
                "strings/english/siege_test.str",
                b"REFERENCE Gate_Down\nLANG_ENGLISH \"The gate is down\"\n".to_vec(),
            )],
        )
        .unwrap();
        let localization = Localization::load(&vfs);
        assert_eq!(
            localization.translate("@@@SIEGE_TEST_GATE_DOWN"),
            "The gate is down"
        );
    }
}
