//! JoF EJK's `clientlist`, `clientlistInfo` and `showPlayerId` console commands.
//!
//! The compact list is shared by `clientlist` and the older `showplayerid` alias.
//! `clientlistinfo` adds the public clientinfo fields that JoF EJK prints for human
//! players. The reference is JoF EJK `cg_consolecmds.c` at commit
//! `bd5e202dfedd49500f1b7aadd655619bac152167`.

use sjk_client::LegacyClientInfo;
use sjk_protocol::{GameState, Snapshot};

pub(crate) const CLIENT_LIST_COMMAND: &str = "clientlist";
pub(crate) const SHOW_PLAYER_ID_COMMAND: &str = "showplayerid";
pub(crate) const CLIENT_LIST_INFO_COMMAND: &str = "clientlistinfo";
pub(crate) const CLIENT_LIST_HELP: &str = "List each player's client number, team and name";
pub(crate) const SHOW_PLAYER_ID_HELP: &str =
    "Alias of clientlist (JoF EJK's showPlayerId spelling)";
pub(crate) const CLIENT_LIST_INFO_HELP: &str =
    "List human clients with model, gender, sabers, colours and team details";

const PLAYERS: usize = 1_131;
const SERVER_INFO: usize = 0;
const MAX_CLIENTS: usize = 32;
const GT_TEAM: i32 = 6;

pub(crate) fn client_list_lines(game_state: &GameState) -> Vec<String> {
    let mut lines = Vec::new();
    for client in 0..MAX_CLIENTS {
        let Some(info) = player_info(game_state, client) else {
            continue;
        };
        let bot = if info.bytes("skill").is_some_and(|skill| !skill.is_empty()) {
            " (bot)"
        } else {
            ""
        };
        lines.push(format!(
            "{client:2} {}   ^7{}^7{bot}",
            team_letter(legacy_integer(info.bytes("t"))),
            info.name().expect("player_info requires a name")
        ));
    }
    let count = lines.len();
    lines.push(format!("Listed {count:2} clients"));
    lines
}

pub(crate) fn client_list_info_lines(game_state: &GameState, snapshot: &Snapshot) -> Vec<String> {
    let team_game = game_state
        .config_string(SERVER_INFO)
        .map(LegacyClientInfo::new)
        .is_some_and(|info| legacy_integer(info.bytes("g_gametype")) >= GT_TEAM);
    (0..MAX_CLIENTS)
        .filter_map(|client| {
            let info = player_info(game_state, client)?;
            if info.bytes("skill").is_some_and(|skill| !skill.is_empty()) {
                return None;
            }
            detailed_line(client, info, team_game, snapshot_rgba(snapshot, client))
        })
        .collect()
}

fn player_info(game_state: &GameState, client: usize) -> Option<LegacyClientInfo<'_>> {
    let info = LegacyClientInfo::new(game_state.config_string(PLAYERS + client)?);
    info.name().is_some().then_some(info)
}

fn detailed_line(
    client: usize,
    info: LegacyClientInfo<'_>,
    team_game: bool,
    rgba: Option<[u8; 4]>,
) -> Option<String> {
    let name = info.name()?;
    let model = info.text("model")?;
    let (model, skin) = model.split_once('/').unwrap_or((model, "default"));
    let skin = skin.split('/').next().unwrap_or_default();
    if model.is_empty() || skin.is_empty() {
        return None;
    }

    let team = legacy_integer(info.bytes("t"));
    let mut line = format!(
        "^7{client:2} {}   ^7{name} ^7{model}",
        info_team_letter(team)
    );
    if !skin.eq_ignore_ascii_case("default") {
        line.push('/');
        line.push_str(skin);
    }
    line.push(' ');

    match info
        .bytes("ds")
        .or_else(|| info.bytes("sex"))
        .and_then(|sex| sex.first())
        .map(u8::to_ascii_lowercase)
    {
        Some(b'f') => line.push_str("^7(female) "),
        Some(b'n') => line.push_str("^7(gender neutral) "),
        _ => {}
    }

    let sabers = saber_names(info);
    if let Some(name) = sabers[0] {
        append_saber(&mut line, name, info, 0);
    }
    if let Some(name) = sabers[1] {
        append_saber(&mut line, name, info, 1);
    }

    if team_game {
        line.push_str(if team == 2 {
            "^7Rebellion "
        } else {
            "^7Empire "
        });
    }
    if let Some([red, green, blue, _]) = rgba
        && [red, green, blue]
            .into_iter()
            .any(|channel| channel > 0 && channel < 255)
    {
        line.push_str(&format!("^7RGBA: ^1{red:03} ^2{green:03} ^4{blue:03} "));
    }
    Some(line.trim_end().to_owned())
}

fn saber_names(info: LegacyClientInfo<'_>) -> [Option<&str>; 2] {
    let primary = info
        .text("saber1")
        .or_else(|| info.text("st"))
        .filter(|name| !name.is_empty());
    let secondary = info
        .text("saber2")
        .or_else(|| info.text("st2"))
        .filter(|name| {
            !name.is_empty()
                && !name.eq_ignore_ascii_case("none")
                && !name.eq_ignore_ascii_case("remove")
        });
    [primary, secondary]
}

fn append_saber(line: &mut String, name: &str, info: LegacyClientInfo<'_>, slot: usize) {
    let color = legacy_integer(info.bytes(if slot == 0 { "c1" } else { "c2" }));
    line.push_str("^7");
    line.push_str(name);
    line.push(' ');
    match color {
        1 => line.push_str("^8(Orange) "),
        2 => line.push_str("^3(Yellow) "),
        3 => line.push_str("^2(Green) "),
        4 => line.push_str("^4(Blue) "),
        5 => line.push_str("^6(Purple) "),
        6..=10 => {
            let packed = legacy_integer(info.bytes(if slot == 0 { "c3" } else { "c4" })) as u32;
            let [red, green, blue] = sjk_client::unpack_saber_rgb(packed);
            let kind = match color {
                7 => "Flame #1",
                8 => "Electric #1",
                9 => "Flame #2",
                10 => "Electric #2",
                _ => "RGB",
            };
            line.push_str(&format!("^7[{red} {green} {blue}] ^5({kind}) "));
        }
        11 => line.push_str("^9(Black) "),
        _ => line.push_str("^1(Red) "),
    }
}

fn snapshot_rgba(snapshot: &Snapshot, client: usize) -> Option<[u8; 4]> {
    let client = u16::try_from(client).ok()?;
    if snapshot.player.client_num() == client {
        return Some(snapshot.player.custom_rgba());
    }
    snapshot
        .entities
        .iter()
        .find(|entity| entity.number() == client)
        .map(|entity| entity.custom_rgba())
}

/// C `atoi` for the small numeric clientinfo fields used here.
fn legacy_integer(value: Option<&[u8]>) -> i32 {
    let value = value.unwrap_or_default();
    let (negative, digits) = match value.split_first() {
        Some((b'-', rest)) => (true, rest),
        Some((b'+', rest)) => (false, rest),
        _ => (false, value),
    };
    let number = digits
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .fold(0i32, |number, byte| {
            number.wrapping_mul(10).wrapping_add(i32::from(byte - b'0'))
        });
    if negative {
        number.wrapping_neg()
    } else {
        number
    }
}

fn team_letter(team: i32) -> &'static str {
    match team {
        0 => "^3F",
        1 => "^1R",
        2 => "^4B",
        _ => "^3S",
    }
}

fn info_team_letter(team: i32) -> &'static str {
    match team {
        0 => "^3F",
        1 => "^1R",
        2 => "^4B",
        _ => "^7S",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(players: &[(usize, &str)]) -> GameState {
        let mut game = GameState::empty_local(0);
        for (client, info) in players {
            game.replace_config_string(PLAYERS + client, info.as_bytes().to_vec())
                .unwrap();
        }
        game
    }

    #[test]
    fn compact_list_matches_jof_team_letters_bots_and_count() {
        let game = game(&[
            (0, "n\\^1Rosh\\t\\1"),
            (3, "n\\Kyle\\t\\2\\skill\\4"),
            (12, "n\\Watcher\\t\\3"),
        ]);
        assert_eq!(
            client_list_lines(&game),
            [
                " 0 ^1R   ^7^1Rosh^7",
                " 3 ^4B   ^7Kyle^7 (bot)",
                "12 ^3S   ^7Watcher^7",
                "Listed  3 clients",
            ]
        );
    }

    #[test]
    fn an_empty_server_lists_no_clients() {
        assert_eq!(client_list_lines(&game(&[])), ["Listed  0 clients"]);
    }

    #[test]
    fn detailed_line_has_model_gender_sabers_and_custom_colours() {
        let info = LegacyClientInfo::new(
            b"n\\^2Leia\\t\\2\\model\\jedi_hf/red\\ds\\f\\st\\single_1\\st2\\staff_1\\c1\\4\\c2\\6\\c4\\197121",
        );
        assert_eq!(
            detailed_line(7, info, true, Some([12, 255, 0, 255])).as_deref(),
            Some(
                "^7 7 ^4B   ^7^2Leia ^7jedi_hf/red ^7(female) ^7single_1 ^4(Blue) ^7staff_1 ^7[1 2 3] ^5(RGB) ^7Rebellion ^7RGBA: ^1012 ^2255 ^4000"
            )
        );
    }

    #[test]
    fn detailed_line_omits_default_skin_and_removed_second_saber() {
        assert_eq!(
            detailed_line(
                2,
                LegacyClientInfo::new(b"n\\Human\\model\\kyle/default\\st\\single_1\\st2\\none"),
                false,
                None,
            )
            .as_deref(),
            Some("^7 2 ^3F   ^7Human ^7kyle ^7single_1 ^1(Red)")
        );
        assert!(detailed_line(4, LegacyClientInfo::new(b"n\\No model"), false, None).is_none());
    }
}
