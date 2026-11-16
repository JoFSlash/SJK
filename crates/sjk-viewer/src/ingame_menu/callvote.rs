//! Fixed-capacity call-vote catalogue and page controller.

use super::Page;
use sjk_protocol::{GameState, InfoString};
use sjk_vfs::VirtualFileSystem;

const CS_SERVERINFO: usize = 0;
const CS_PLAYERS: usize = 1_131;
pub(super) const PAGE_ITEMS: usize = 16;
const PLAYER_LIMIT: usize = 32;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Action {
    Open(Page),
    Send(String),
    Back,
    None,
}

/// Retained map/player catalogues used by call-vote sub-screens.
pub(crate) struct State {
    /// The maps a vote may name, as `callvote map` takes them.
    maps: Vec<String>,
    /// What each map's row shows: the arena's `longname` and the map, or the map.
    map_labels: Vec<String>,
    map_len: usize,
    players: [Player; PLAYER_LIMIT],
    player_len: usize,
    map_offset: usize,
    kick_offset: usize,
}

#[derive(Clone)]
struct Player {
    client_num: u8,
    name: String,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            maps: Vec::new(),
            map_labels: Vec::new(),
            map_len: 0,
            players: std::array::from_fn(|index| Player {
                client_num: index as u8,
                name: String::with_capacity(64),
            }),
            player_len: 0,
            map_offset: 0,
            kick_offset: 0,
        }
    }

    /// Refresh catalogues when the call-vote screen is opened, never per frame.
    ///
    /// The maps are the server's own list when its serverinfo carries one
    /// (`sv_maplist`), else the installed maps the arena files allow for the
    /// server's game type, as the stock in-game vote screen lists them
    /// (`UI_LoadArenas`, `UI_MapCountByGameType`): the retail maps first, then
    /// the community's, each by name.
    pub(crate) fn refresh(
        &mut self,
        game_state: Option<&GameState>,
        vfs: Option<&VirtualFileSystem>,
    ) {
        self.maps.clear();
        self.map_labels.clear();
        self.map_len = 0;
        self.player_len = 0;
        self.map_offset = 0;
        self.kick_offset = 0;
        let mut gametype = None;
        if let Some(game_state) = game_state {
            self.read_server_maps(game_state);
            self.read_players(game_state);
            gametype = server_gametype(game_state);
        }
        if self.map_len == 0
            && let Some(vfs) = vfs
        {
            for map in arena_maps(vfs, gametype) {
                self.push_map(&map.name, map.label.as_deref());
            }
        }
    }

    pub(crate) fn row_count(&self, page: Page) -> usize {
        match page {
            Page::CallVote => 10,
            Page::VoteMap => self.page_len(self.map_len, self.map_offset),
            Page::VoteGameType => 10,
            Page::VoteKick | Page::VoteClientKick => {
                self.page_len(self.player_len, self.kick_offset)
            }
            Page::VoteWarmup => 3,
            Page::VoteTimeLimit => 7,
            Page::VoteFragLimit => 7,
            _ => 0,
        }
    }

    pub(crate) fn prepare_rows(&self, page: Page, rows: &mut [String]) -> usize {
        for row in rows.iter_mut() {
            row.clear();
        }
        match page {
            Page::CallVote => fill(
                rows,
                &[
                    "Restart map",
                    "Next map",
                    "Change map...",
                    "Change game type...",
                    "Kick player by name...",
                    "Kick client number...",
                    "Warmup...",
                    "Time limit...",
                    "Frag limit...",
                    "Back",
                ],
            ),
            Page::VoteMap => self.map_rows(rows),
            Page::VoteGameType => fill(
                rows,
                &[
                    "Free for all  /  0",
                    "Holocron free for all  /  1",
                    "Jedi master  /  2",
                    "Duel  /  3",
                    "Power duel  /  4",
                    "Team free for all  /  6",
                    "Siege  /  7",
                    "Capture the flag  /  8",
                    "Capture the Ysalamiri  /  9",
                    "Back",
                ],
            ),
            Page::VoteKick | Page::VoteClientKick => self.player_rows(page, rows),
            Page::VoteWarmup => fill(rows, &["Enable warmup", "Disable warmup", "Back"]),
            Page::VoteTimeLimit => fill(rows, &["0", "5", "10", "15", "20", "30", "Back"]),
            Page::VoteFragLimit => fill(rows, &["0", "10", "20", "30", "40", "50", "Back"]),
            _ => 0,
        }
    }

    pub(crate) fn activate(&mut self, page: Page, row: usize) -> Action {
        match page {
            Page::CallVote => self.activate_root(row),
            Page::VoteMap => self.activate_map(row),
            Page::VoteGameType => self.activate_gametype(row),
            Page::VoteKick => self.activate_player(row, false),
            Page::VoteClientKick => self.activate_player(row, true),
            Page::VoteWarmup => self.activate_bool(row),
            Page::VoteTimeLimit => self.activate_number(row, true),
            Page::VoteFragLimit => self.activate_number(row, false),
            _ => Action::None,
        }
    }

    pub(crate) fn scroll(&mut self, page: Page, lines: isize) {
        let (offset, total) = match page {
            Page::VoteMap => (&mut self.map_offset, self.map_len),
            Page::VoteKick | Page::VoteClientKick => (&mut self.kick_offset, self.player_len),
            _ => return,
        };
        let last = total.saturating_sub(PAGE_ITEMS);
        *offset = (*offset as isize + lines).clamp(0, last as isize) as usize;
    }

    pub(crate) fn scroll_to(&mut self, page: Page, ratio: f32) {
        let (offset, total) = match page {
            Page::VoteMap => (&mut self.map_offset, self.map_len),
            Page::VoteKick | Page::VoteClientKick => (&mut self.kick_offset, self.player_len),
            _ => return,
        };
        let last = total.saturating_sub(PAGE_ITEMS);
        *offset = (ratio.clamp(0.0, 1.0) * last as f32).round() as usize;
    }

    pub(crate) fn scroll_metrics(&self, page: Page) -> Option<(usize, usize)> {
        match page {
            Page::VoteMap if self.map_len > PAGE_ITEMS => Some((self.map_offset, self.map_len)),
            Page::VoteKick | Page::VoteClientKick if self.player_len > PAGE_ITEMS => {
                Some((self.kick_offset, self.player_len))
            }
            _ => None,
        }
    }

    fn read_server_maps(&mut self, game_state: &GameState) {
        let Some(text) = game_state
            .config_string(CS_SERVERINFO)
            .and_then(|value| std::str::from_utf8(value).ok())
        else {
            return;
        };
        let Ok(info) = InfoString::parse(text) else {
            return;
        };
        let Some(list) = info.get("sv_maplist").or_else(|| info.get("maplist")) else {
            return;
        };
        for map in list.split_ascii_whitespace() {
            self.push_map(map, None);
        }
    }

    fn push_map(&mut self, map: &str, label: Option<&str>) {
        if self
            .maps
            .iter()
            .any(|known| known.eq_ignore_ascii_case(map))
        {
            return;
        }
        self.maps.push(map.to_owned());
        self.map_labels.push(match label {
            Some(label) => format!("{label}  /  {map}"),
            None => map.to_owned(),
        });
        self.map_len = self.maps.len();
    }

    fn read_players(&mut self, game_state: &GameState) {
        for client in 0..PLAYER_LIMIT {
            let Some(text) = game_state
                .config_string(CS_PLAYERS + client)
                .and_then(|value| std::str::from_utf8(value).ok())
            else {
                continue;
            };
            let Ok(info) = InfoString::parse(text) else {
                continue;
            };
            let Some(name) = info.get("n").or_else(|| info.get("name")) else {
                continue;
            };
            self.players[self.player_len].client_num = client as u8;
            self.players[self.player_len].name.clear();
            self.players[self.player_len].name.push_str(name);
            self.player_len += 1;
        }
    }

    fn page_len(&self, total: usize, offset: usize) -> usize {
        total.saturating_sub(offset).min(PAGE_ITEMS) + 1 + usize::from(total > offset + PAGE_ITEMS)
    }

    fn map_rows(&self, rows: &mut [String]) -> usize {
        let end = (self.map_offset + PAGE_ITEMS).min(self.map_len);
        let mut count = 0;
        for map in &self.map_labels[self.map_offset..end] {
            rows[count].push_str(map);
            count += 1;
        }
        if end < self.map_len {
            rows[count].push_str("More maps...");
            count += 1;
        }
        rows[count].push_str("Back");
        count + 1
    }

    fn player_rows(&self, page: Page, rows: &mut [String]) -> usize {
        let end = (self.kick_offset + PAGE_ITEMS).min(self.player_len);
        let mut count = 0;
        for player in &self.players[self.kick_offset..end] {
            if page == Page::VoteClientKick {
                use std::fmt::Write as _;
                let _ = write!(
                    rows[count],
                    "{}  /  client {}",
                    player.name, player.client_num
                );
            } else {
                rows[count].push_str(&player.name);
            }
            count += 1;
        }
        if end < self.player_len {
            rows[count].push_str("More players...");
            count += 1;
        }
        rows[count].push_str("Back");
        count + 1
    }

    fn activate_root(&self, row: usize) -> Action {
        match row {
            0 => send(sjk_client::LegacyCallVote::MapRestart),
            1 => send(sjk_client::LegacyCallVote::NextMap),
            2 => Action::Open(Page::VoteMap),
            3 => Action::Open(Page::VoteGameType),
            4 => Action::Open(Page::VoteKick),
            5 => Action::Open(Page::VoteClientKick),
            6 => Action::Open(Page::VoteWarmup),
            7 => Action::Open(Page::VoteTimeLimit),
            8 => Action::Open(Page::VoteFragLimit),
            9 => Action::Back,
            _ => Action::None,
        }
    }

    fn activate_map(&mut self, row: usize) -> Action {
        let remaining = self.map_len.saturating_sub(self.map_offset).min(PAGE_ITEMS);
        if row < remaining {
            return send(sjk_client::LegacyCallVote::Map(
                &self.maps[self.map_offset + row],
            ));
        }
        if self.map_offset + remaining < self.map_len && row == remaining {
            self.map_offset += remaining;
            return Action::Open(Page::VoteMap);
        }
        Action::Back
    }

    fn activate_gametype(&self, row: usize) -> Action {
        [0, 1, 2, 3, 4, 6, 7, 8, 9]
            .get(row)
            .copied()
            .map_or(Action::Back, |value| {
                send(sjk_client::LegacyCallVote::GameType(value))
            })
    }

    fn activate_player(&mut self, row: usize, numeric: bool) -> Action {
        let remaining = self
            .player_len
            .saturating_sub(self.kick_offset)
            .min(PAGE_ITEMS);
        if row < remaining {
            let player = &self.players[self.kick_offset + row];
            return if numeric {
                send(sjk_client::LegacyCallVote::ClientKick(player.client_num))
            } else {
                send(sjk_client::LegacyCallVote::Kick(&player.name))
            };
        }
        if self.kick_offset + remaining < self.player_len && row == remaining {
            self.kick_offset += remaining;
            return Action::Open(if numeric {
                Page::VoteClientKick
            } else {
                Page::VoteKick
            });
        }
        Action::Back
    }

    fn activate_bool(&self, row: usize) -> Action {
        match row {
            0 => send(sjk_client::LegacyCallVote::Warmup(true)),
            1 => send(sjk_client::LegacyCallVote::Warmup(false)),
            _ => Action::Back,
        }
    }

    fn activate_number(&self, row: usize, time: bool) -> Action {
        let values = if time {
            [0, 5, 10, 15, 20, 30]
        } else {
            [0, 10, 20, 30, 40, 50]
        };
        values.get(row).copied().map_or(Action::Back, |value| {
            if time {
                send(sjk_client::LegacyCallVote::TimeLimit(value))
            } else {
                send(sjk_client::LegacyCallVote::FragLimit(value))
            }
        })
    }
}

/// One map the vote list offers.
#[derive(Debug, Eq, PartialEq)]
struct VoteMap {
    /// `mp/ffa3`, as `callvote map` takes it.
    name: String,
    /// The arena's `longname`, when it has one other than the name.
    label: Option<String>,
}

/// The game type in the server's serverinfo (`g_gametype`).
fn server_gametype(game_state: &GameState) -> Option<i32> {
    let text = std::str::from_utf8(game_state.config_string(CS_SERVERINFO)?).ok()?;
    InfoString::parse(text)
        .ok()?
        .get("g_gametype")?
        .trim()
        .parse()
        .ok()
}

/// The installed maps (`maps/**.bsp`) the arena files (`scripts/arenas.txt`,
/// `scripts/*.arena`) allow for `gametype` (every arena map when it is unknown),
/// retail ones first, each part by name.
fn arena_maps(vfs: &VirtualFileSystem, gametype: Option<i32>) -> Vec<VoteMap> {
    let read = |path: &str| {
        vfs.read(path)
            .ok()
            .flatten()
            .map(|asset| asset.bytes.to_vec())
    };
    let mut arenas: Vec<Vec<u8>> = read("scripts/arenas.txt").into_iter().collect();
    arenas.extend(
        vfs.list_files("scripts", ".arena")
            .iter()
            .filter_map(|name| read(&format!("scripts/{name}"))),
    );
    let installed: Vec<String> = vfs
        .list_files("maps", ".bsp")
        .iter()
        .filter_map(|file| file.get(..file.len().checked_sub(4)?))
        .map(str::to_ascii_lowercase)
        .collect();
    let retail = sjk_client::retail_view(vfs);
    let is_retail = |name: &str| {
        retail
            .as_ref()
            .is_some_and(|view| view.contains(&format!("maps/{name}.bsp")).unwrap_or(false))
    };
    let mut maps = vote_maps(&installed, &arenas, gametype);
    maps.sort_by_cached_key(|map| (!is_retail(&map.name), map.name.clone()));
    maps
}

/// [`arena_maps`] from the installed map names (`mp/ffa3`, lower case) and the
/// arena files' text, unordered. When no arena names an installed map, every
/// installed `maps/mp` map.
fn vote_maps(installed: &[String], arenas: &[Vec<u8>], gametype: Option<i32>) -> Vec<VoteMap> {
    // `UI_MapCountByGameType`: team FFA lists the FFA maps, Capture the Ysalamiri
    // the CTF ones.
    let wanted = gametype
        .map(|value| match value {
            6 => 0,
            9 => 8,
            other => other,
        })
        .map(|value| {
            u32::try_from(value)
                .ok()
                .and_then(|bit| 1u32.checked_shl(bit))
        });
    let mut maps: Vec<VoteMap> = Vec::new();
    let mut any_arena = false;
    let quiet = &mut |_: &[u8]| {};
    for info in arenas
        .iter()
        .flat_map(|text| sjk_game_jka::bots::parse_infos(text, 1_024, quiet))
    {
        let text = |key: &[u8]| {
            String::from_utf8_lossy(sjk_protocol::info_value(&info, key).unwrap_or_default())
                .into_owned()
        };
        let name = text(b"map").to_ascii_lowercase();
        any_arena |= installed.contains(&name);
        if name.is_empty()
            || !installed.contains(&name)
            || maps.iter().any(|known| known.name == name)
        {
            continue;
        }
        if let Some(wanted) = wanted {
            let types = text(b"type").to_ascii_lowercase();
            let bits = sjk_game_jka::arenas::map_type_bits(types.as_bytes());
            if wanted.is_none_or(|bit| bits & bit == 0) {
                continue;
            }
        }
        let longname = text(b"longname");
        let label =
            (!longname.is_empty() && longname != "<NULL>" && !longname.eq_ignore_ascii_case(&name))
                .then_some(longname);
        maps.push(VoteMap { name, label });
    }
    if !any_arena {
        maps.extend(
            installed
                .iter()
                .filter(|name| name.starts_with("mp/"))
                .map(|name| VoteMap {
                    name: name.clone(),
                    label: None,
                }),
        );
    }
    maps
}

/// The row of the call-vote page that opens `page`, one of its lists (the
/// SJK UI's Escape returns to it).
pub(super) fn opening_row(page: Page) -> Option<usize> {
    match page {
        Page::VoteMap => Some(2),
        Page::VoteGameType => Some(3),
        Page::VoteKick => Some(4),
        Page::VoteClientKick => Some(5),
        Page::VoteWarmup => Some(6),
        Page::VoteTimeLimit => Some(7),
        Page::VoteFragLimit => Some(8),
        _ => None,
    }
}

fn fill(rows: &mut [String], values: &[&str]) -> usize {
    for (row, value) in rows.iter_mut().zip(values) {
        row.push_str(value);
    }
    values.len()
}

fn send(vote: sjk_client::LegacyCallVote<'_>) -> Action {
    let mut command = String::with_capacity(96);
    match sjk_client::write_legacy_callvote(&mut command, vote) {
        Ok(()) => Action::Send(command),
        Err(_) => Action::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_list_opens_from_its_row() {
        let state = State::new();
        for row in 0..state.row_count(Page::CallVote) {
            if let Action::Open(page) = state.activate_root(row) {
                assert_eq!(opening_row(page), Some(row), "{page:?}");
            }
        }
        assert_eq!(opening_row(Page::CallVote), None);
    }

    fn names(maps: &[VoteMap]) -> Vec<&str> {
        maps.iter().map(|map| map.name.as_str()).collect()
    }

    // Report #136 (Kushina, a JoF server in team FFA): the list was every `.bsp`
    // in path order, cut at 128, so the community maps at `maps/*.bsp` filled it
    // before the retail `maps/mp` ones and single-player levels showed too.
    #[test]
    fn the_vote_lists_the_arena_maps_of_the_game_type() {
        let installed: Vec<String> = ["mp/ffa3", "mp/duel1", "mp/ctf1", "t1_sour", "jof_lobby"]
            .map(String::from)
            .into();
        let arenas = vec![
            b"{ map \"mp/ffa3\" longname \"Tatooine City\" type \"ffa holocron jedimaster\" }
              { map \"mp/duel1\" longname \"Bespin Exhaust\" type \"duel powerduel\" }
              { map \"mp/ctf1\" type \"ctf cty\" }
              { map \"mp/ffa9\" type \"ffa\" }"
                .to_vec(),
            b"{ map \"jof_lobby\" longname \"JoF Lobby\" type \"ffa team\" }".to_vec(),
        ];
        // Team FFA lists the FFA maps; a map without its .bsp is left out.
        let maps = vote_maps(&installed, &arenas, Some(6));
        assert_eq!(names(&maps), ["mp/ffa3", "jof_lobby"]);
        assert_eq!(maps[0].label.as_deref(), Some("Tatooine City"));
        assert_eq!(
            names(&vote_maps(&installed, &arenas, Some(4))),
            ["mp/duel1"]
        );
        assert_eq!(names(&vote_maps(&installed, &arenas, Some(9))), ["mp/ctf1"]);
        // An unknown game type lists every arena map.
        assert_eq!(vote_maps(&installed, &arenas, None).len(), 4);
        assert_eq!(
            names(&vote_maps(&installed, &arenas, Some(40))),
            [] as [&str; 0]
        );
        // No arena files at all: the installed multiplayer maps.
        assert_eq!(
            names(&vote_maps(&installed, &[], Some(0))),
            ["mp/ffa3", "mp/duel1", "mp/ctf1"]
        );
    }

    #[test]
    fn rows_show_the_long_name_and_votes_name_the_map() {
        let mut state = State::new();
        state.push_map("mp/ffa3", Some("Tatooine City"));
        state.push_map("MP/FFA3", None);
        state.push_map("mp/duel1", None);
        let mut rows: Vec<String> = vec![String::new(); PAGE_ITEMS + 2];
        assert_eq!(state.prepare_rows(Page::VoteMap, &mut rows), 3);
        assert_eq!(rows[0], "Tatooine City  /  mp/ffa3");
        assert_eq!(rows[1], "mp/duel1");
        assert_eq!(
            state.activate(Page::VoteMap, 0),
            send(sjk_client::LegacyCallVote::Map("mp/ffa3"))
        );
    }
}
