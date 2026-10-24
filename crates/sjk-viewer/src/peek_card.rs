//! The peek card (`docs/client.md`, "Timed player peek"): a small SJK card at the top
//! centre of play, under the unlock pop-up, while a peek concerns the player. It asks
//! when someone wants to watch them (Allow, gold, Insert; Deny, Delete; Block, which
//! also blocks the sender on this PC), and otherwise
//! says whom they watch, whom they asked or who watches them, with the seconds left and
//! Stop (Delete). It is not modal: play goes on under it and only those keys are taken
//! (`GpuState::peek_key`). It stays over the game menu, at the frame's top, so Escape
//! frees the pointer to click its buttons. It hides while the console, the main menus
//! or a pop-up card is over the frame.

use crate::menu::sjk::{Frame, TextTarget, color, key_hint, key_hint_width, kit, text};
use crate::menu_widgets::{MenuCanvas, TextFamily};
use sjk_ui::{FontWeight, InputEvent, TextAlign, UiEventKind};

const ALLOW_TOKEN: u16 = 1;
const DENY_TOKEN: u16 = 2;
const STOP_TOKEN: u16 = 3;
const BLOCK_TOKEN: u16 = 4;

/// The card's left edge, top and width in frame pixels: centred, under the unlock
/// pop-up (`unlock_toast.rs`), above the centre prints; over the game menu, at the top,
/// clear of its arc and match card.
const CARD_WIDTH: f32 = 720.0;
const CARD_X: f32 = (1920.0 - CARD_WIDTH) * 0.5;
const PLAY_TOP: f32 = 236.0;
const MENU_TOP: f32 = 14.0;
const MARGIN: f32 = 28.0;
const TEXT_X: f32 = CARD_X + MARGIN;
const TEXT_WIDTH: f32 = CARD_WIDTH - MARGIN * 2.0;
/// The card's height with a question's buttons row, and without.
const ASK_HEIGHT: f32 = 178.0;
const SHORT_HEIGHT: f32 = 112.0;
const BUTTON_HEIGHT: f32 = 40.0;
const BUTTON_GAP: f32 = 12.0;
const ALLOW_WIDTH: f32 = 140.0;
const DENY_WIDTH: f32 = 120.0;
const BLOCK_WIDTH: f32 = 120.0;
const STOP_WIDTH: f32 = 120.0;

/// What the card shows; names keep their colour codes, which the card leaves out.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum View {
    /// `name` asks to watch the player for `seconds`; `left` seconds to answer.
    Request {
        name: String,
        seconds: i64,
        left: i64,
    },
    /// The player asked `name` and waits for the answer.
    Asking { name: String, left: i64 },
    /// The player watches `name`.
    Watching { name: String, left: i64 },
    /// `name` watches the player.
    Watched { name: String, left: i64 },
}

impl View {
    /// The small line over the headline, the headline, and the explanation if any.
    fn words(&self) -> (String, String, Option<String>) {
        let plain = |name: &str| {
            let name = crate::medals::plain_note(name);
            if name.trim().is_empty() {
                "Someone".to_owned()
            } else {
                name
            }
        };
        match self {
            Self::Request {
                name,
                seconds,
                left,
            } => (
                format!("PEEK REQUEST  \u{b7}  {left} s"),
                format!("{} wants to watch you", plain(name)),
                Some(format!(
                    "For {seconds} s their camera follows you from behind. Nothing changes \
                     unless you allow it."
                )),
            ),
            Self::Asking { name, left } => (
                format!("PEEK  \u{b7}  {left} s"),
                format!("Asking {}\u{2026}", plain(name)),
                None,
            ),
            Self::Watching { name, left } => (
                format!("PEEK  \u{b7}  {left} s"),
                format!("Watching {}", plain(name)),
                None,
            ),
            Self::Watched { name, left } => (
                format!("PEEK  \u{b7}  {left} s"),
                format!("{} is watching you", plain(name)),
                None,
            ),
        }
    }
}

/// What the player chose on the card.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Choice {
    Allow,
    Deny,
    /// Deny and block the sender.
    Block,
    Stop,
}

pub(crate) struct Card {
    view: Option<View>,
    /// Drawn this frame.
    shown: bool,
    ui: MenuCanvas,
}

impl Default for Card {
    fn default() -> Self {
        Self {
            view: None,
            shown: false,
            ui: MenuCanvas::with_capacities(12, 256, 48),
        }
    }
}

impl Card {
    /// Whether the card was drawn this frame, so its keys and buttons count.
    pub(crate) fn is_shown(&self) -> bool {
        self.shown
    }

    pub(crate) fn draw_list(&self) -> &sjk_ui::DrawList {
        self.ui.draw_list()
    }

    /// A click on a button while the card shows.
    pub(crate) fn handle_pointer(&mut self, event: InputEvent) -> Option<Choice> {
        if !self.shown {
            return None;
        }
        let event = self.ui.pointer(event)?;
        if event.kind != UiEventKind::Activate {
            return None;
        }
        match event.token {
            Some(ALLOW_TOKEN) => Some(Choice::Allow),
            Some(DENY_TOKEN) => Some(Choice::Deny),
            Some(STOP_TOKEN) => Some(Choice::Stop),
            Some(BLOCK_TOKEN) => Some(Choice::Block),
            _ => None,
        }
    }

    /// Show `view` this frame, or nothing.
    pub(crate) fn set(&mut self, view: Option<View>) {
        self.shown = view.is_some();
        self.view = view;
    }

    /// Draw the card, over the game menu when `in_menu`; its text goes to `target`.
    pub(crate) fn append(&mut self, target: TextTarget<'_>, viewport: [f32; 2], in_menu: bool) {
        let Some(view) = self.view.as_ref() else {
            return;
        };
        let top = if in_menu { MENU_TOP } else { PLAY_TOP };
        let frame = Frame::new(viewport);
        let s = frame.s;
        let (eyebrow, headline, body) = view.words();
        let asks = matches!(view, View::Request { .. });
        let height = if asks { ASK_HEIGHT } else { SHORT_HEIGHT };
        let ui = &mut self.ui;
        ui.begin_transparent(viewport);
        kit::card(ui, &frame, [CARD_X, top, CARD_WIDTH, height]);
        // Stop sits at the right of the short card, beside the words.
        let words_width = if asks {
            TEXT_WIDTH
        } else {
            TEXT_WIDTH - STOP_WIDTH - BUTTON_GAP
        };
        text(
            ui,
            TextFamily::Display,
            format_args!("{eyebrow}"),
            frame.rect(TEXT_X, top + 18.0, words_width, 24.0),
            17.0 * s,
            color::GOLD_BRIGHT,
            FontWeight::Semibold,
            TextAlign::Start,
        );
        text(
            ui,
            TextFamily::Display,
            format_args!("{headline}"),
            frame.rect(TEXT_X, top + 44.0, words_width, 40.0),
            28.0 * s,
            color::TEXT,
            FontWeight::Semibold,
            TextAlign::Start,
        );
        let right = CARD_X + CARD_WIDTH - MARGIN;
        if asks {
            if let Some(body) = body {
                text(
                    ui,
                    TextFamily::Body,
                    format_args!("{body}"),
                    frame.rect(TEXT_X, top + 86.0, TEXT_WIDTH, 24.0),
                    16.0 * s,
                    color::MUTED,
                    FontWeight::Regular,
                    TextAlign::Start,
                );
            }
            let buttons_y = top + height - MARGIN - BUTTON_HEIGHT + 6.0;
            kit::button(
                ui,
                &frame,
                [right - ALLOW_WIDTH, buttons_y, ALLOW_WIDTH, BUTTON_HEIGHT],
                "Allow",
                true,
                true,
                false,
                ALLOW_TOKEN,
            );
            kit::button(
                ui,
                &frame,
                [
                    right - ALLOW_WIDTH - BUTTON_GAP - DENY_WIDTH,
                    buttons_y,
                    DENY_WIDTH,
                    BUTTON_HEIGHT,
                ],
                "Deny",
                false,
                true,
                false,
                DENY_TOKEN,
            );
            kit::button(
                ui,
                &frame,
                [
                    right - ALLOW_WIDTH - DENY_WIDTH - BLOCK_WIDTH - BUTTON_GAP * 2.0,
                    buttons_y,
                    BLOCK_WIDTH,
                    BUTTON_HEIGHT,
                ],
                "Block",
                false,
                true,
                false,
                BLOCK_TOKEN,
            );
            let [x, y] = frame.point(TEXT_X, buttons_y + (BUTTON_HEIGHT - 24.0) * 0.5);
            let x = key_hint(ui, &["Ins"], "allow", x, y, s);
            key_hint(ui, &["Del"], "deny", x + 14.0 * s, y, s);
        } else {
            let button_y = top + (height - BUTTON_HEIGHT) * 0.5 - 8.0;
            kit::button(
                ui,
                &frame,
                [right - STOP_WIDTH, button_y, STOP_WIDTH, BUTTON_HEIGHT],
                "Stop",
                true,
                true,
                false,
                STOP_TOKEN,
            );
            let width = key_hint_width(&["Del"], "stop", s);
            let [x, y] = frame.point(right - STOP_WIDTH * 0.5, button_y + BUTTON_HEIGHT + 6.0);
            key_hint(ui, &["Del"], "stop", x - width * 0.5, y, s);
        }
        // No button starts focused: a stray Enter or Space answers nothing.
        ui.finish(0);
        target.append(&self.ui, viewport);
    }
}

impl crate::GpuState {
    /// Draw the peek card over play or the game menu when a peek concerns the player and
    /// nothing covers the frame (`covered`: the console or a pop-up card). Returns
    /// whether it shows.
    pub(crate) fn append_peek_card(&mut self, viewport: [f32; 2], covered: bool) -> bool {
        let menu_visible = self
            .client_menu
            .as_ref()
            .is_some_and(crate::menu::ClientMenu::is_visible);
        let console_open = self
            .console
            .as_ref()
            .is_some_and(crate::console::ViewerConsole::is_open);
        let place = !covered && !menu_visible && !console_open;
        let view = place.then(|| self.peek_card_view()).flatten();
        self.peek.card.set(view);
        if !self.peek.card.is_shown() {
            return false;
        }
        let target = crate::ingame_menu::sjk_view::text_target(
            &mut self.game_fonts,
            &mut self.text_vertices,
            &self.ui_font,
        );
        self.peek.card.append(target, viewport, self.game_menu);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_words_name_the_other_player_without_colours() {
        let (eyebrow, headline, body) = View::Request {
            name: "^1Tox".into(),
            seconds: 60,
            left: 12,
        }
        .words();
        assert_eq!(eyebrow, "PEEK REQUEST  \u{b7}  12 s");
        assert_eq!(headline, "Tox wants to watch you");
        assert!(body.unwrap().starts_with("For 60 s"));
        let (_, headline, body) = View::Watched {
            name: "^3^7".into(),
            left: 4,
        }
        .words();
        assert_eq!(headline, "Someone is watching you");
        assert!(body.is_none());
    }

    #[test]
    fn the_card_shows_only_with_something_to_say() {
        let mut card = Card::default();
        card.set(None);
        assert!(!card.is_shown());
        card.set(Some(View::Watching {
            name: "Tox".into(),
            left: 3,
        }));
        assert!(card.is_shown());
    }
}
