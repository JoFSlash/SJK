//! Retained siege selection in two steps, as the team page of the other team games:
//! the side first (Auto-join, Red team, Blue team, Spectate, Back), then that side's
//! classes. Choosing a class sends the stock class command, not userinfo; Auto-join
//! sends `team free`, which puts the player on the side with fewer players in the
//! class the server settles for it.
use super::{InGameMenu, siege_data::Classes};
use sjk_protocol::GameState;
use sjk_vfs::VirtualFileSystem;
use std::fmt::Write as _;

const PAGE_SIZE: usize = 8;

/// The side step's rows.
const AUTO: usize = 0;
const RED: usize = 1;
const BLUE: usize = 2;
const SPECTATE: usize = 3;
const SIDE_ROWS: usize = 5;

#[derive(Default)]
/// Selection cursor and load-time catalogue retained between menu frames.
pub(super) struct State {
    classes: Classes,
    /// The side whose classes are listed (0 red, 1 blue); `None` on the side step.
    side: Option<usize>,
    first: usize,
    /// The row the cursor takes when an action keeps the page.
    cursor: usize,
    /// The player's own team when the page opened (1 red, 2 blue, 3 spectator).
    team: u8,
}

/// What the side step needs from the frame: how many play on each side.
#[derive(Clone, Copy, Debug)]
pub(super) struct Sides {
    pub(super) red_players: usize,
    pub(super) blue_players: usize,
}

impl State {
    /// The side step over `classes` for a player on `team`, the cursor on the
    /// player's own side, or on Auto-join for one who watches.
    fn new(classes: Classes, team: u8) -> Self {
        Self {
            classes,
            side: None,
            first: 0,
            cursor: match team {
                1 => RED,
                2 => BLUE,
                _ => AUTO,
            },
            team,
        }
    }

    /// The side step over made-up class names (world shots and tests).
    #[cfg(test)]
    pub(super) fn for_shot(teams: [&[&str]; 2], team: u8) -> Self {
        let names = |names: &[&str]| names.iter().map(|name| (*name).to_owned()).collect();
        Self::new(
            Classes {
                teams: [names(teams[0]), names(teams[1])],
            },
            team,
        )
    }

    fn visible(&self, side: usize) -> &[String] {
        let all = &self.classes.teams[side];
        &all[self.first..(self.first + PAGE_SIZE).min(all.len())]
    }

    fn paged(&self, side: usize) -> bool {
        self.classes.teams[side].len() > PAGE_SIZE
    }

    /// The side step's five rows, or a side's visible classes plus optional
    /// pagination and Back.
    pub fn row_count(&self) -> usize {
        match self.side {
            None => SIDE_ROWS,
            Some(side) => self.visible(side).len() + 1 + usize::from(self.paged(side)),
        }
    }

    /// Fill already allocated menu text slots without allocating; a row's detail
    /// follows its label after "  /  ".
    pub fn prepare(&self, rows: &mut [String; 24], sides: Sides) -> usize {
        let Some(side) = self.side else {
            let _ = write!(
                rows[AUTO],
                "Auto-join  /  The side with fewer players: {} red, {} blue",
                sides.red_players, sides.blue_players
            );
            for (row, label, players, team) in [
                (RED, "Red team", sides.red_players, 1),
                (BLUE, "Blue team", sides.blue_players, 2),
            ] {
                let mine = if self.team == team { "Your team, " } else { "" };
                let _ = write!(
                    rows[row],
                    "{label}  /  {mine}{}",
                    super::sjk_view::Players(players)
                );
            }
            rows[SPECTATE].push_str(if self.team == 3 {
                "Spectate  /  You are watching"
            } else {
                "Spectate  /  Watch the match"
            });
            rows[SPECTATE + 1].push_str("Back");
            return SIDE_ROWS;
        };
        // The step already names the side, so a class row is its name alone:
        // the classic pop-up's 240 units fit the long stock names.
        for (row, name) in self.visible(side).iter().enumerate() {
            rows[row].push_str(name);
        }
        let mut next = self.visible(side).len();
        if self.paged(side) {
            rows[next].push_str("Next page");
            next += 1;
        }
        rows[next].push_str("Back  /  Choose the other team");
        next + 1
    }

    /// The page's name: the side step's, or the listed side's classes.
    pub(super) fn title(&self) -> &'static str {
        match self.side {
            None => "Choose your team",
            Some(0) => "Red team classes",
            Some(_) => "Blue team classes",
        }
    }

    /// Whether the side step is showing (rather than a side's classes).
    pub(super) fn on_sides(&self) -> bool {
        self.side.is_none()
    }

    /// Whether `row` is the side step's Spectate while the player already
    /// watches: it cannot be taken, as the Team page's current team.
    pub(super) fn current_row(&self, row: usize) -> bool {
        self.side.is_none() && row == SPECTATE && self.team == 3
    }

    /// The side step's team marks: row `row`'s team (1 red, 2 blue), if any.
    pub(super) fn mark(&self, row: usize) -> Option<u8> {
        match (self.side, row) {
            (None, RED) => Some(1),
            (None, BLUE) => Some(2),
            _ => None,
        }
    }
}

impl InGameMenu {
    /// Refresh from the mounted map and server's theme overrides on each open, on
    /// the side step; `team` is the player's own (not a followed player's).
    pub(crate) fn refresh_siege(
        &mut self,
        game: &GameState,
        vfs: &VirtualFileSystem,
        team: u8,
    ) -> Result<(), String> {
        self.siege = State::new(Classes::load(vfs, game)?, team);
        Ok(())
    }

    /// From a side's classes back to the side step, returning that side's row;
    /// `None` on the side step already (Escape then leaves the page).
    pub(crate) fn siege_back(&mut self) -> Option<usize> {
        let state = &mut self.siege;
        let side = state.side.take()?;
        state.first = 0;
        state.cursor = RED + side;
        Some(state.cursor)
    }

    /// Resolve a visible row through the same path for mouse and keyboard
    /// activation; [`super::CallVoteAction::None`] keeps the page, the cursor on
    /// [`Self::siege_row`].
    pub(crate) fn siege_action(&mut self, row: usize) -> super::CallVoteAction {
        use super::CallVoteAction as Action;
        let state = &mut self.siege;
        let Some(side) = state.side else {
            return match row {
                // `PickTeam` on the server: the side with fewer players.
                AUTO => Action::Send("team free".into()),
                RED | BLUE => {
                    state.side = Some(row - RED);
                    state.first = 0;
                    state.cursor = 0;
                    Action::None
                }
                SPECTATE => Action::Send("team spectator".into()),
                _ => Action::Back,
            };
        };
        if let Some(name) = state.visible(side).get(row) {
            // Cmd_SiegeClass_f chooses the team from this name and enforces cooldowns.
            return Action::Send(format!("siegeclass \"{name}\""));
        }
        if state.paged(side) && row == state.visible(side).len() {
            state.first += PAGE_SIZE;
            if state.first >= state.classes.teams[side].len() {
                state.first = 0;
            }
            state.cursor = 0;
            return Action::None;
        }
        self.siege_back();
        Action::None
    }

    /// The row the cursor takes after [`Self::siege_action`] kept the page: the
    /// first class after choosing a side or turning a page, the side's row after
    /// coming back from its classes.
    pub(crate) fn siege_row(&self) -> usize {
        self.siege.cursor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingame_menu::CallVoteAction as Action;

    fn menu(team: u8) -> InGameMenu {
        let mut menu = InGameMenu::new();
        menu.siege = State::for_shot([&["Jedi Warrior", "Jedi Knight"], &["Sith Warrior"]], team);
        menu
    }

    fn labels(menu: &InGameMenu) -> Vec<String> {
        let mut rows: [String; 24] = std::array::from_fn(|_| String::new());
        let sides = Sides {
            red_players: 4,
            blue_players: 1,
        };
        let count = menu.siege.prepare(&mut rows, sides);
        assert_eq!(count, menu.siege.row_count());
        rows[..count].to_vec()
    }

    #[test]
    fn the_side_comes_first_as_in_the_other_team_games() {
        let menu = menu(1);
        assert_eq!(
            labels(&menu),
            [
                "Auto-join  /  The side with fewer players: 4 red, 1 blue",
                "Red team  /  Your team, 4 players",
                "Blue team  /  1 player",
                "Spectate  /  Watch the match",
                "Back",
            ]
        );
        assert_eq!(menu.siege.title(), "Choose your team");
        assert_eq!(menu.siege_row(), RED, "opens on the player's side");
        assert_eq!(self::menu(2).siege_row(), BLUE);
        assert_eq!(self::menu(3).siege_row(), AUTO, "a spectator on Auto-join");
        assert_eq!(
            (0..5).map(|row| menu.siege.mark(row)).collect::<Vec<_>>(),
            [None, Some(1), Some(2), None, None]
        );
    }

    #[test]
    fn spectate_cannot_be_taken_while_watching() {
        let watching = menu(3);
        assert!(watching.siege.current_row(SPECTATE));
        assert!(!watching.siege.current_row(RED) && !watching.siege.current_row(BLUE));
        assert!(!watching.siege.current_row(AUTO));
        assert!(!menu(1).siege.current_row(SPECTATE));
        let mut listing = menu(3);
        listing.siege_action(RED);
        assert!(
            !listing.siege.current_row(SPECTATE),
            "a class row, not Spectate"
        );
    }

    #[test]
    fn a_side_lists_its_classes_and_back_returns_to_it() {
        let mut menu = menu(3);
        assert_eq!(menu.siege_action(RED), Action::None);
        assert_eq!(menu.siege_row(), 0);
        assert_eq!(menu.siege.title(), "Red team classes");
        assert_eq!(
            labels(&menu),
            [
                "Jedi Warrior",
                "Jedi Knight",
                "Back  /  Choose the other team"
            ]
        );
        assert_eq!(menu.siege.mark(0), None);
        assert_eq!(
            menu.siege_action(1),
            Action::Send("siegeclass \"Jedi Knight\"".into())
        );
        // Back (and Escape) go to the side step, the cursor on the side.
        assert_eq!(menu.siege_action(2), Action::None);
        assert_eq!(menu.siege.title(), "Choose your team");
        assert_eq!(menu.siege_row(), RED);
        assert_eq!(menu.siege_back(), None);
        assert_eq!(menu.siege_action(BLUE), Action::None);
        assert_eq!(menu.siege.title(), "Blue team classes");
        assert_eq!(labels(&menu)[0], "Sith Warrior");
        assert_eq!(menu.siege_back(), Some(BLUE));
        assert_eq!(menu.siege_row(), BLUE);
        assert_eq!(
            menu.siege_action(SPECTATE),
            Action::Send("team spectator".into())
        );
        assert_eq!(menu.siege_action(SPECTATE + 1), Action::Back);
    }

    #[test]
    fn auto_join_takes_the_side_with_fewer_players() {
        let mut menu = menu(3);
        assert_eq!(menu.siege_action(AUTO), Action::Send("team free".into()));
        assert!(menu.siege.on_sides(), "no class list: the server picks");
    }
}
