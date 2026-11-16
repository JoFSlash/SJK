//! BaseJKA entity colour (`customRGBA`) resolution for rendered actors.
//!
//! The server fills `playerState_t::customRGBA` from the `char_color_*`
//! userinfo keys (`codemp/game/g_client.c:2177-2198`) and copies it onto
//! body-queue corpses (`g_client.c:1135-1138`). cgame forwards it as the
//! render entity's `shaderRGBA`:
//!
//! - `CG_Player` prefers `clientInfo_t::colorOverride` when it is non-zero and
//!   otherwise uses `entityState_t::customRGBA`
//!   (`codemp/cgame/cg_players.c:9016-9030`). `colorOverride` is produced by
//!   `BG_ValidateSkinForTeam` for `jedi_*` custom-species models in team
//!   gametypes other than siege and jedi-vs-merc
//!   (`cg_players.c:481-488`, `codemp/game/bg_misc.c:2643-2660`): red team
//!   `(1, 0, 0)`, blue team `(0, 0, 1)`.
//! - `CG_General`, which renders `ET_BODY` corpses, copies `customRGBA`
//!   verbatim (`codemp/cgame/cg_ents.c`, `CG_General`).
//!
//! Only material stages whose colour generator reads the entity colour
//! (`entity`, `oneMinusEntity`, `lightingDiffuseEntity`) observe the result;
//! that mapping lives in the renderer.

use sjk_protocol::GameState;

const CS_SERVERINFO: usize = 0;
const CS_PLAYERS: usize = 1_131;
/// `GT_TEAM` / `GT_SIEGE` from `codemp/game/bg_public.h` `gametype_t`.
const GT_TEAM: i32 = 6;
const GT_SIEGE: i32 = 7;
/// `TEAM_RED` / `TEAM_BLUE` from `team_t`.
const TEAM_RED: i32 = 1;
const TEAM_BLUE: i32 = 2;

/// Per-gamestate inputs to the team colour override rule, resolved once per
/// snapshot so entity resolution stays a fixed-cost lookup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TeamColorPolicy {
    /// `cgs.gametype >= GT_TEAM && !cgs.jediVmerc && cgs.gametype != GT_SIEGE`.
    pub team_override_active: bool,
}

impl TeamColorPolicy {
    /// Read `g_gametype` and `g_jediVmerc` from `CS_SERVERINFO`.
    pub fn from_game_state(game_state: &GameState) -> Self {
        Self::from_server_info(game_state.config_string(CS_SERVERINFO).unwrap_or(&[]))
    }

    /// Evaluate the rule over a raw `CS_SERVERINFO` string.
    pub fn from_server_info(serverinfo: &[u8]) -> Self {
        let gametype = info_int(serverinfo, "g_gametype");
        let jedi_vs_merc = info_int(serverinfo, "g_jediVmerc") != 0;
        Self {
            team_override_active: gametype >= GT_TEAM && !jedi_vs_merc && gametype != GT_SIEGE,
        }
    }
}

/// Resolve the colour `CG_Player` hands to the renderer for a live player.
///
/// `custom_rgba` is the entity's (or the local player state's) `customRGBA`.
pub fn legacy_player_color(
    game_state: &GameState,
    policy: TeamColorPolicy,
    client_num: u16,
    custom_rgba: [u8; 4],
) -> [u8; 4] {
    if !policy.team_override_active {
        return custom_rgba;
    }
    let Some(config) = game_state.config_string(CS_PLAYERS + usize::from(client_num)) else {
        return custom_rgba;
    };
    player_color_from_config(config, custom_rgba)
}

/// `colorOverride` selection over one raw `CS_PLAYERS` string.
fn player_color_from_config(config: &[u8], custom_rgba: [u8; 4]) -> [u8; 4] {
    let is_custom_species = info_value(config, "model")
        .is_some_and(|model| model.len() > 5 && model[..5].eq_ignore_ascii_case(b"jedi_"));
    if !is_custom_species {
        return custom_rgba;
    }
    match info_int(config, "t") {
        TEAM_RED => [255, 0, 0, custom_rgba[3]],
        TEAM_BLUE => [0, 0, 255, custom_rgba[3]],
        _ => custom_rgba,
    }
}

/// `GT_CTF` / `GT_CTY` from `gametype_t`.
const GT_CTF: i32 = 8;
const GT_CTY: i32 = 9;

/// Whether the game is one where SJK's optional team sabers apply (the viewer's
/// `cg_ctfTeamSabers`): capture the flag and capture the ysalamiri. It is stricter than
/// cgame's `cg_saberTeamColors 0`, which lets a red player keep a yellow blade
/// (`cg_players.c:8329-8342`).
pub fn legacy_team_sabers(game_state: &GameState) -> bool {
    let serverinfo = game_state.config_string(CS_SERVERINFO).unwrap_or(&[]);
    matches!(info_int(serverinfo, "g_gametype"), GT_CTF | GT_CTY)
}

/// The blade colour `client_num`'s team forces on both sabers, if
/// [`legacy_team_sabers`] holds and the client is on red or blue: the stock
/// `SABER_RED` / `SABER_BLUE` tints, whatever `c1`..`c4` say.
pub fn legacy_team_blade(game_state: &GameState, client_num: u16) -> Option<[u8; 3]> {
    if !legacy_team_sabers(game_state) {
        return None;
    }
    let config = game_state.config_string(CS_PLAYERS + usize::from(client_num))?;
    team_blade_from_config(config)
}

/// The team blade of one raw `CS_PLAYERS` string.
fn team_blade_from_config(config: &[u8]) -> Option<[u8; 3]> {
    use crate::presentation_equipment::legacy_blade_rgb;
    match info_int(config, "t") {
        TEAM_RED => Some(legacy_blade_rgb(0)),
        TEAM_BLUE => Some(legacy_blade_rgb(4)),
        _ => None,
    }
}

/// The blade colours `client_num`'s Siege class forces on its two sabers, `None` for a
/// saber that keeps the player's own colour.
///
/// cgame `CG_NewClientInfo` (`codemp/cgame/cg_players.c:2492-2537`): in Siege the
/// class named by the player's `siegeclass` (`BG_SiegeFindClassByName`, any case)
/// replaces `icolor1` with its `sabercolor` and `icolor2` with its `saber2color`
/// (`bg_saga.c:874-893`) when it has them, before any other colour rule. Only cgame
/// does this; the server sends `c1`/`c2` as the player set them. The forced number is
/// read as a `c1` without a custom tint would be (`saber_color_from_config`).
pub fn legacy_siege_class_blades(
    game_state: &GameState,
    registry: &sjk_game_jka::siege_class::SiegeRegistry,
    client_num: u16,
) -> [Option<[u8; 3]>; 2] {
    let serverinfo = game_state.config_string(CS_SERVERINFO).unwrap_or(&[]);
    if info_int(serverinfo, "g_gametype") != GT_SIEGE {
        return [None; 2];
    }
    let class = game_state
        .config_string(CS_PLAYERS + usize::from(client_num))
        .and_then(|config| info_value(config, "siegeclass"))
        .and_then(|name| std::str::from_utf8(name).ok())
        .filter(|name| !name.is_empty())
        .and_then(|name| registry.class_index(name))
        .and_then(|index| registry.classes.get(index));
    let Some(class) = class else {
        return [None; 2];
    };
    [class.saber_color, class.saber2_color].map(|color| color.map(forced_blade_rgb))
}

/// A forced `saber_colors_t` number as a blade tint: past `SABER_PURPLE` it rolls
/// over as a `c1` without a custom tint does (`ClampSaberColor`).
fn forced_blade_rgb(color: i32) -> [u8; 3] {
    crate::presentation_equipment::legacy_blade_rgb(color.rem_euclid(12) % (SABER_PURPLE + 1))
}

/// `SABER_PURPLE`, the last stock entry of `saber_colors_t`.
const SABER_PURPLE: i32 = 5;
/// `SABER_RED` / `SABER_BLUE` from `saber_colors_t`.
const SABER_RED: i32 = 0;
const SABER_BLUE: i32 = 4;

/// The team colours cgame forces on `client_num`'s two sabers (`c1`, `c2`), `None`
/// for a saber that keeps its own colour.
///
/// JoF EJK `CG_AddSaberBlade` (`codemp/cgame/cg_players.c:8096-8111`): in team
/// gametypes other than siege and jedi-vs-merc ([`TeamColorPolicy`]), a red player's
/// blades turn `SABER_RED` and a blue player's `SABER_BLUE`. With `always` false
/// (`cg_saberTeamColors 0`) only a blade in the other team's colour or past
/// `SABER_PURPLE` (an RGB, black or other extended colour) is replaced. The custom
/// RGB of `c3`/`c4` goes with the index it belonged to. NPCs never reach this: they
/// have no `CS_PLAYERS` slot.
pub fn legacy_saber_team_colors(
    game_state: &GameState,
    policy: TeamColorPolicy,
    client_num: u16,
    always: bool,
) -> [Option<[u8; 3]>; 2] {
    if !policy.team_override_active {
        return [None; 2];
    }
    game_state
        .config_string(CS_PLAYERS + usize::from(client_num))
        .map_or([None; 2], |config| {
            saber_team_colors_from_config(config, always)
        })
}

/// [`legacy_saber_team_colors`] over one raw `CS_PLAYERS` string.
fn saber_team_colors_from_config(config: &[u8], always: bool) -> [Option<[u8; 3]>; 2] {
    use crate::presentation_equipment::legacy_blade_rgb;
    let (own, opposite) = match info_int(config, "t") {
        TEAM_RED => (SABER_RED, SABER_BLUE),
        TEAM_BLUE => (SABER_BLUE, SABER_RED),
        _ => return [None; 2],
    };
    ["c1", "c2"].map(|key| {
        // `icolor1`/`icolor2` are `atoi` of the raw value, before `ClampSaberColor`;
        // an absent key reads as 0 (red), as `Info_ValueForKey` returns "".
        let color = info_int(config, key);
        (always || color == opposite || color > SABER_PURPLE).then(|| legacy_blade_rgb(own))
    })
}

/// Resolve the colour `CG_General` hands to the renderer for a corpse.
pub fn legacy_body_color(custom_rgba: [u8; 4]) -> [u8; 4] {
    custom_rgba
}

/// Allocation-free `Info_ValueForKey` over a raw configstring.
fn info_value<'a>(info: &'a [u8], key: &str) -> Option<&'a [u8]> {
    let mut fields = info
        .split(|byte| *byte == b'\\')
        .skip_while(|field| field.is_empty());
    while let (Some(candidate), Some(value)) = (fields.next(), fields.next()) {
        if candidate.eq_ignore_ascii_case(key.as_bytes()) {
            return Some(value);
        }
    }
    None
}

/// `atoi(Info_ValueForKey(...))` with a zero default.
fn info_int(info: &[u8], key: &str) -> i32 {
    info_value(info, key).map_or(0, crate::team_info::legacy_atoi)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation_equipment::legacy_blade_rgb;

    fn registry() -> sjk_game_jka::siege_class::SiegeRegistry {
        let class = |name: &str, colors: &str| {
            format!(
                "ClassInfo\n{{\n\tname \"{name}\"\n\tweapons WP_SABER\n\tuishader \"models/players/kyle/icon_default\"\n{colors}}}\n"
            )
        };
        let files = [
            class("Jedi Knight", "\tsabercolor 5\n"),
            class("Dark Jedi", "\tsabercolor 0\n\tsaber2color 1\n"),
            class("Imperial Infantry", ""),
        ];
        sjk_game_jka::siege_class::SiegeRegistry::load(files.iter().map(String::as_str), [])
    }

    fn siege_player(serverinfo: &[u8], class: &str) -> GameState {
        game(
            serverinfo,
            format!(r"n\a\t\1\c1\3\c2\2\siegeclass\{class}").as_bytes(),
        )
    }

    #[test]
    fn siege_classes_force_their_saber_colours() {
        let registry = registry();
        assert_eq!(registry.classes.len(), 3, "{registry:?}");
        let siege = br"\g_gametype\7";
        let purple = Some(legacy_blade_rgb(5));
        // A class with `sabercolor 5` draws purple whatever c1 says, any case.
        assert_eq!(
            legacy_siege_class_blades(&siege_player(siege, "jedi knight"), &registry, 2),
            [purple, None]
        );
        assert_eq!(
            legacy_siege_class_blades(&siege_player(siege, "Dark Jedi"), &registry, 2),
            [Some(legacy_blade_rgb(0)), Some(legacy_blade_rgb(1))]
        );
        // A class without one, an unknown class and no class keep the player's own.
        for class in ["Imperial Infantry", "Nobody", ""] {
            assert_eq!(
                legacy_siege_class_blades(&siege_player(siege, class), &registry, 2),
                [None; 2]
            );
        }
        assert_eq!(
            legacy_siege_class_blades(&siege_player(siege, "Jedi Knight"), &registry, 5),
            [None; 2],
            "an empty slot"
        );
        // Other gametypes ignore the class, as cgame reads it only in Siege.
        for gametype in [
            br"\g_gametype\0".as_slice(),
            br"\g_gametype\6",
            br"\g_gametype\8",
        ] {
            assert_eq!(
                legacy_siege_class_blades(&siege_player(gametype, "Jedi Knight"), &registry, 2),
                [None; 2]
            );
        }
    }

    #[test]
    fn red_and_blue_players_carry_only_their_teams_blade() {
        let yellow_rgb = br"n\a\t\1\c1\2\c2\8\c4\65280";
        assert_eq!(
            team_blade_from_config(yellow_rgb),
            Some(legacy_blade_rgb(0))
        );
        let red = br"n\b\t\2\c1\0\c2\0";
        assert_eq!(team_blade_from_config(red), Some(legacy_blade_rgb(4)));
        // Free and spectating players keep their own colours.
        assert_eq!(team_blade_from_config(br"n\c\t\0\c1\2"), None);
        assert_eq!(team_blade_from_config(br"n\d\t\3\c1\2"), None);
    }

    fn game(serverinfo: &[u8], player: &[u8]) -> GameState {
        let mut game = GameState::empty_local(0);
        game.replace_config_string(CS_SERVERINFO, serverinfo.to_vec())
            .unwrap();
        game.replace_config_string(CS_PLAYERS + 2, player.to_vec())
            .unwrap();
        game
    }

    fn forced(serverinfo: &[u8], player: &[u8], always: bool) -> [Option<[u8; 3]>; 2] {
        let game = game(serverinfo, player);
        let policy = TeamColorPolicy::from_game_state(&game);
        legacy_saber_team_colors(&game, policy, 2, always)
    }

    #[test]
    fn team_gametypes_force_red_and_blue_sabers() {
        let red = Some(legacy_blade_rgb(0));
        let blue = Some(legacy_blade_rgb(4));
        // Team FFA, CTF and CTY: both sabers, a custom RGB included.
        for gametype in [
            br"\g_gametype\6".as_slice(),
            br"\g_gametype\8",
            br"\g_gametype\9",
        ] {
            assert_eq!(
                forced(gametype, br"n\a\t\1\c1\3\c2\6\c4\65280", true),
                [red, red]
            );
            assert_eq!(forced(gametype, br"n\b\t\2\c1\2\c2\0", true), [blue, blue]);
            // Free and spectating players keep their own colours.
            assert_eq!(forced(gametype, br"n\c\t\0\c1\2", true), [None; 2]);
            assert_eq!(forced(gametype, br"n\d\t\3\c1\2", true), [None; 2]);
        }
    }

    #[test]
    fn ffa_duel_siege_and_jedi_vs_merc_keep_the_players_choice() {
        let player = br"n\a\t\1\c1\3\c2\4";
        for serverinfo in [
            br"\g_gametype\0".as_slice(),
            br"\g_gametype\3",
            br"\g_gametype\7",
            br"\g_gametype\6\g_jediVmerc\1",
        ] {
            assert_eq!(forced(serverinfo, player, true), [None; 2]);
        }
    }

    #[test]
    fn saber_team_colors_zero_replaces_only_the_other_team_and_extended_colours() {
        let red = Some(legacy_blade_rgb(0));
        let blue = Some(legacy_blade_rgb(4));
        let tffa = br"\g_gametype\6";
        // A red player keeps yellow and purple, loses blue.
        assert_eq!(forced(tffa, br"n\a\t\1\c1\2\c2\4", false), [None, red]);
        assert_eq!(forced(tffa, br"n\a\t\1\c1\5\c2\0", false), [None; 2]);
        // RGB (6), black (11) and the cosmetic suffix JoF appends (`8santahat`).
        assert_eq!(forced(tffa, br"n\a\t\1\c1\6\c2\11", false), [red, red]);
        assert_eq!(
            forced(tffa, br"n\b\t\2\c1\8santahat\c2\3", false),
            [blue, None]
        );
        // A blue player loses red, keeps green.
        assert_eq!(forced(tffa, br"n\b\t\2\c1\0\c2\3", false), [blue, None]);
    }
}
