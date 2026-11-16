//! Siege class saber colours, as stock cgame forces them: in Siege a class file's
//! `sabercolor` / `saber2color` replace the colours its players chose
//! (`CG_NewClientInfo`, `codemp/cgame/cg_players.c:2492-2537`;
//! `sjk_client::legacy_siege_class_blades`). The server takes no part: it sends
//! `c1`/`c2` as the players set them.
//!
//! The map's class files are read once per map and each player's colours are
//! worked out again only when `CS_SERVERINFO` or a `CS_PLAYERS` slot changes
//! ([`Cache::invalidate`]); the frame reads the stored result. The colours go in
//! with the team blades (`team_sabers.rs`), so a class colour replaces a black
//! blade too and a blade shader is still worn over it.

use sjk_game_jka::siege_class::{MAX_SIEGE_CLASSES, SiegeRegistry};
use sjk_protocol::{GameState, InfoString, LEGACY_MAX_CLIENTS};
use sjk_vfs::VirtualFileSystem;

/// Each client slot's class-forced colours (primary, secondary); `None` keeps the
/// player's own.
pub(crate) type Forced = [[Option<[u8; 3]>; 2]; LEGACY_MAX_CLIENTS];

/// The map's classes and the colours they force, rebuilt on configstring changes.
pub(crate) struct Cache {
    valid: bool,
    /// The Siege map whose class files `registry` holds; empty before any.
    map: String,
    registry: SiegeRegistry,
    forced: Forced,
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            valid: false,
            map: String::new(),
            registry: SiegeRegistry::default(),
            forced: [[None; 2]; LEGACY_MAX_CLIENTS],
        }
    }
}

impl Cache {
    /// Called by the configstring dispatch for `CS_SERVERINFO` and `CS_PLAYERS`.
    pub(crate) fn invalidate(&mut self) {
        self.valid = false;
    }

    /// The colours each client's class forces; reworked only after
    /// [`Self::invalidate`].
    pub(crate) fn forced(
        &mut self,
        game: Option<&GameState>,
        vfs: Option<&VirtualFileSystem>,
    ) -> &Forced {
        if !self.valid
            && let (Some(game), Some(vfs)) = (game, vfs)
        {
            self.rebuild(game, |map| load(vfs, map));
        }
        &self.forced
    }

    /// Work the colours out again from `game`, reading a new map's classes with `load`.
    fn rebuild(&mut self, game: &GameState, load: impl FnOnce(&str) -> SiegeRegistry) {
        self.valid = true;
        self.forced = [[None; 2]; LEGACY_MAX_CLIENTS];
        let Some(map) = siege_map(game) else {
            return;
        };
        if !self.map.eq_ignore_ascii_case(&map) {
            self.registry = load(&map);
            self.map = map;
        }
        for (client, forced) in (0u16..).zip(self.forced.iter_mut()) {
            *forced = sjk_client::legacy_siege_class_blades(game, &self.registry, client);
        }
    }
}

/// The map's name when the game is Siege.
fn siege_map(game: &GameState) -> Option<String> {
    if !crate::ingame_menu::siege_data::is_siege(Some(game)) {
        return None;
    }
    let text = std::str::from_utf8(game.config_string(0)?).ok()?;
    let info = InfoString::parse(text).ok()?;
    info.get("mapname").map(str::to_owned)
}

/// Every installed class file in the game's listing order (`BG_SiegeLoadClasses`, as
/// SJK's server reads them), at most the game's limit.
fn load(vfs: &VirtualFileSystem, map: &str) -> SiegeRegistry {
    const DIRECTORY: &str = "ext_data/Siege/Classes";
    let classes: Vec<String> = vfs
        .list_files(DIRECTORY, ".scl")
        .into_iter()
        .take(MAX_SIEGE_CLASSES)
        .filter_map(|file| vfs.read(&format!("{DIRECTORY}/{file}")).ok().flatten())
        .map(|asset| String::from_utf8_lossy(&asset.bytes).into_owned())
        .collect();
    crate::log::progress(format_args!(
        "Siege class saber colours: {} class files for {map}",
        classes.len()
    ));
    SiegeRegistry::load(classes.iter().map(String::as_str), [])
}

#[cfg(test)]
mod tests {
    use super::*;

    const PURPLE: [u8; 3] = [230, 51, 255];

    fn registry() -> SiegeRegistry {
        let class = |name: &str, colors: &str| {
            format!(
                "ClassInfo\n{{\n\tname \"{name}\"\n\tweapons WP_SABER\n\tuishader \"x\"\n{colors}}}\n"
            )
        };
        let files = [
            class("Jedi Knight", "\tsabercolor 5\n"),
            class("Imperial Infantry", ""),
        ];
        SiegeRegistry::load(files.iter().map(String::as_str), [])
    }

    fn game(gametype: u8) -> GameState {
        let mut game = GameState::empty_local(0);
        game.replace_config_string(
            0,
            format!(r"\g_gametype\{gametype}\mapname\siege_hoth").into_bytes(),
        )
        .unwrap();
        game.replace_config_string(
            1131 + 3,
            br"\n\Jedi\t\2\c1\3\siegeclass\Jedi Knight".to_vec(),
        )
        .unwrap();
        game.replace_config_string(
            1131 + 4,
            br"\n\Trooper\t\1\c1\2\siegeclass\Imperial Infantry".to_vec(),
        )
        .unwrap();
        game
    }

    #[test]
    fn classes_load_once_per_map_and_colours_follow_changes() {
        let mut cache = Cache::default();
        let mut loads = 0;
        let mut rebuild = |cache: &mut Cache, game: &GameState| {
            cache.rebuild(game, |_| {
                loads += 1;
                registry()
            });
        };
        let mut siege = game(7);
        rebuild(&mut cache, &siege);
        assert_eq!(cache.forced[3], [Some(PURPLE), None], "sabercolor 5");
        assert_eq!(cache.forced[4], [None; 2], "a class without one");
        // A class change is picked up on the next rebuild, the classes not read again.
        siege
            .replace_config_string(1131 + 4, br"\n\Trooper\t\2\siegeclass\Jedi Knight".to_vec())
            .unwrap();
        rebuild(&mut cache, &siege);
        assert_eq!(cache.forced[4], [Some(PURPLE), None]);
        // Other gametypes are untouched.
        rebuild(&mut cache, &game(8));
        assert_eq!(cache.forced, [[None; 2]; LEGACY_MAX_CLIENTS]);
        assert_eq!(loads, 1);
    }
}
