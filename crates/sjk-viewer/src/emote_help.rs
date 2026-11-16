//! The emote help card (`docs/hub-chat.md`, "Emotes in the client"): `help_emotes` (or
//! `sjkemote help`) shows an SJK pop-up card over the darkened screen, as the low-FPS
//! card, that lists the `sjkemote` commands and says how to put emotes on the quick
//! wheel. One command to give a player who asks how emotes work. Close (Enter or
//! Escape) ends it.

use crate::menu::sjk::{Frame, TextTarget, color, key_hint, key_hint_width, kit, text, wrap};
use crate::menu_widgets::{MenuCanvas, TextFamily};
use sjk_ui::{FontWeight, InputEvent, TextAlign, UiEventKind};
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

/// Console command that opens the card.
pub(crate) const COMMAND: &str = "help_emotes";
pub(crate) const HELP: &str = "Show how SJK emotes work: the sjkemote commands and the quick wheel";
/// `sjkemote help` opens it too.
pub(crate) const WORD: &str = "help";

const CLOSE_TOKEN: u16 = 1;

/// The card's left edge and width; it is as tall as what it holds, centred.
const CARD_X: f32 = 440.0;
const CARD_WIDTH: f32 = 1040.0;
const SCREEN_HEIGHT: f32 = 1080.0;
const MARGIN: f32 = 44.0;
const TEXT_X: f32 = CARD_X + MARGIN;
const TEXT_WIDTH: f32 = CARD_WIDTH - MARGIN * 2.0;
/// Below the kicker and headline.
const BODY_OFFSET: f32 = 130.0;
const COMMAND_WIDTH: f32 = 270.0;
const LINE: f32 = 27.0;
/// The descriptions' column, and the paragraphs', in characters.
const ROW_CHARS: usize = 92;
const PARAGRAPH_CHARS: usize = 132;
const ROW_GAP: f32 = 8.0;
const SECTION_GAP: f32 = 22.0;
const FOOT_GAP: f32 = 26.0;
const BUTTON_HEIGHT: f32 = 46.0;
const CLOSE_WIDTH: f32 = 150.0;
const KEYS_Y: f32 = 992.0;
const KEYS_RIGHT: f32 = 1824.0;

/// The commands, as typed, and what each does.
const COMMANDS: [(&str, &str); 8] = [
    (
        "emotes",
        "Opens a window of every emote you have: type to search, click one to play it.",
    ),
    ("sjkemote", "Lists the emotes you have installed."),
    (
        "sjkemote <id>",
        "Plays one. The SJK players here see it when they have the same emote.",
    ),
    ("sjkemote stop", "Ends yours, for everyone."),
    (
        "sjkemote sync [player]",
        "Joins a player's emote in step, music too: the one named, else the one under \
         your crosshair, else the nearest.",
    ),
    (
        "sjkemote stop all",
        "Ends everyone else's emotes on your screen, once: their next one plays.",
    ),
    ("sjkemote stop <player>", "The same for one player only."),
    (
        "sjkemote mute <player>",
        "Hides a player's emotes on your screen until sjkemote unmute <player> (or \
         unmute all) or you quit. sjkemote mute alone lists them.",
    ),
];

/// What follows the commands.
const PARAGRAPHS: [&str; 3] = [
    "<player> is a slot number, part of a name, or target: the player under your crosshair.",
    "On the quick wheel: Settings > Quick wheel, pick a page, add choices from the Emotes \
     group (your emotes, Sync emote, Stop emote), then hold the wheel key (Q by default), \
     move towards one and let go.",
    "Others see your emotes with the SJK identity on (cl_identity 1). Moving ends most \
     emotes. Their music: Settings > Sound > Emotes.",
];

/// What the player chose on the card.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Choice {
    Close,
}

#[derive(Default)]
pub(crate) struct State {
    pub(crate) card: Card,
    /// The card opened the game menu, which goes with it.
    opened_menu: bool,
}

pub(crate) struct Card {
    wanted: bool,
    /// Shown this frame.
    open: bool,
    ui: MenuCanvas,
}

impl Default for Card {
    fn default() -> Self {
        Self {
            wanted: false,
            open: false,
            ui: MenuCanvas::with_capacities(48, 140, 96),
        }
    }
}

/// One line the card draws, from the body's top: a command (in the code font) with
/// the first line of its words, or a line of words alone.
#[derive(Debug, PartialEq)]
struct Line {
    y: f32,
    command: Option<&'static str>,
    words: String,
    /// A paragraph's line, across the whole card.
    wide: bool,
}

/// The body's lines and its height.
fn layout() -> (Vec<Line>, f32) {
    let mut lines = Vec::new();
    let mut y = 0.0;
    for (command, words) in COMMANDS {
        for (index, line) in wrap(words, ROW_CHARS).enumerate() {
            lines.push(Line {
                y,
                command: (index == 0).then_some(command),
                words: line.to_owned(),
                wide: false,
            });
            y += LINE;
        }
        y += ROW_GAP;
    }
    y += SECTION_GAP - ROW_GAP;
    for paragraph in PARAGRAPHS {
        for line in wrap(paragraph, PARAGRAPH_CHARS) {
            lines.push(Line {
                y,
                command: None,
                words: line.to_owned(),
                wide: true,
            });
            y += LINE;
        }
        y += ROW_GAP;
    }
    (lines, y - ROW_GAP)
}

impl Card {
    pub(crate) fn show(&mut self) {
        self.wanted = true;
    }

    /// Whether the card has something to show.
    pub(crate) fn wants(&self) -> bool {
        self.wanted
    }

    pub(crate) fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    pub(crate) fn is_open(&self) -> bool {
        self.open
    }

    /// Enter, Space, Escape or Backspace close; every other key is swallowed while
    /// the card shows.
    pub(crate) fn handle_key(&mut self, event: &KeyEvent) -> Option<Choice> {
        if event.state != ElementState::Pressed || event.repeat {
            return None;
        }
        let PhysicalKey::Code(code) = event.physical_key else {
            return None;
        };
        self.key(code)
    }

    fn key(&mut self, code: KeyCode) -> Option<Choice> {
        match code {
            KeyCode::Enter
            | KeyCode::NumpadEnter
            | KeyCode::Space
            | KeyCode::Escape
            | KeyCode::Backspace => Some(self.close()),
            _ => None,
        }
    }

    pub(crate) fn handle_pointer(&mut self, event: InputEvent) -> Option<Choice> {
        let event = self.ui.pointer(event)?;
        (event.kind == UiEventKind::Activate && event.token == Some(CLOSE_TOKEN))
            .then(|| self.close())
    }

    fn close(&mut self) -> Choice {
        self.wanted = false;
        Choice::Close
    }

    pub(crate) fn draw_list(&self) -> &sjk_ui::DrawList {
        self.ui.draw_list()
    }

    /// Draw the card over the whole frame.
    pub(crate) fn append(&mut self, target: TextTarget<'_>, viewport: [f32; 2]) {
        let frame = Frame::new(viewport);
        let s = frame.s;
        let (lines, body) = layout();
        let height = BODY_OFFSET + body + FOOT_GAP + BUTTON_HEIGHT + MARGIN;
        let top = ((SCREEN_HEIGHT - height) * 0.5).max(24.0);
        let body_top = top + BODY_OFFSET;
        let foot = body_top + body + FOOT_GAP;
        let ui = &mut self.ui;
        ui.begin_transparent(viewport);
        kit::scrim(ui, viewport);
        kit::card(ui, &frame, [CARD_X, top, CARD_WIDTH, height]);
        text(
            ui,
            TextFamily::Display,
            format_args!("EMOTES"),
            frame.rect(TEXT_X, top + 30.0, TEXT_WIDTH, 30.0),
            22.0 * s,
            color::GOLD_BRIGHT,
            FontWeight::Semibold,
            TextAlign::Start,
        );
        text(
            ui,
            TextFamily::Display,
            format_args!("How emotes work"),
            frame.rect(TEXT_X, top + 66.0, TEXT_WIDTH, 48.0),
            36.0 * s,
            color::TEXT,
            FontWeight::Semibold,
            TextAlign::Start,
        );
        for line in &lines {
            let y = body_top + line.y;
            if let Some(command) = line.command {
                text(
                    ui,
                    TextFamily::Code,
                    format_args!("{command}"),
                    frame.rect(TEXT_X, y, COMMAND_WIDTH, LINE),
                    17.0 * s,
                    color::GOLD_BRIGHT,
                    FontWeight::Regular,
                    TextAlign::Start,
                );
            }
            let (x, width) = if line.wide {
                (TEXT_X, TEXT_WIDTH)
            } else {
                (TEXT_X + COMMAND_WIDTH, TEXT_WIDTH - COMMAND_WIDTH)
            };
            text(
                ui,
                TextFamily::Body,
                format_args!("{}", line.words),
                frame.rect(x, y, width, LINE),
                18.0 * s,
                if line.wide { color::MUTED } else { color::TEXT },
                FontWeight::Regular,
                TextAlign::Start,
            );
        }
        let right = CARD_X + CARD_WIDTH - MARGIN;
        kit::button(
            ui,
            &frame,
            [right - CLOSE_WIDTH, foot, CLOSE_WIDTH, BUTTON_HEIGHT],
            "Close",
            true,
            true,
            false,
            CLOSE_TOKEN,
        );
        let [right, keys_y] = frame.point(KEYS_RIGHT, KEYS_Y);
        let keys: &[&str] = &["Esc"];
        let x = right - key_hint_width(keys, "close", s);
        key_hint(ui, keys, "close", x, keys_y, s);
        ui.finish(CLOSE_TOKEN);
        target.append(&self.ui, viewport);
    }
}

impl crate::GpuState {
    /// `help_emotes`, `sjkemote help`: show the card, over the game menu when on a map.
    pub(crate) fn emote_help_command(&mut self) -> Result<Vec<String>, String> {
        if let Some(console) = &mut self.console {
            console.set_open(false);
        }
        let menu_visible = self
            .client_menu
            .as_ref()
            .is_some_and(crate::menu::ClientMenu::is_visible);
        if !menu_visible && self.live_session.is_some() && !self.game_menu {
            self.release_pointer();
            self.game_menu = true;
            self.game_menu_page = crate::ingame_menu::Page::Main;
            self.game_menu_row = 0;
            self.emote_help.opened_menu = true;
        }
        self.emote_help.card.show();
        self.sync_cursor_policy();
        Ok(Vec::new())
    }

    /// Whether the card shows this frame: asked for, and the player on a menu or the
    /// game menu with nothing else over it. `covered` is another card or the console.
    pub(crate) fn prepare_emote_help_card(&mut self, covered: bool) -> bool {
        let menu_visible = self
            .client_menu
            .as_ref()
            .is_some_and(crate::menu::ClientMenu::is_visible);
        let console_open = self
            .console
            .as_ref()
            .is_some_and(crate::console::ViewerConsole::is_open);
        let open = self.emote_help.card.wants()
            && (menu_visible || self.game_menu)
            && !covered
            && !console_open
            && !self.text_dialog.is_open()
            && self.world_load_task.is_none()
            && self.world_install_task.is_none()
            && !self.pending_map_reload;
        self.emote_help.card.set_open(open);
        open
    }

    /// Draw the card. It darkens everything, so every font batch appended before it is
    /// dropped (text draws above all shapes).
    pub(crate) fn append_emote_help_card(&mut self, viewport: [f32; 2]) {
        self.text_vertices.clear();
        self.classic_text_vertices.clear();
        self.game_fonts.clear_text();
        let target = crate::ingame_menu::sjk_view::text_target(
            &mut self.game_fonts,
            &mut self.text_vertices,
            &self.ui_font,
        );
        self.emote_help.card.append(target, viewport);
    }

    /// Back to play when the card opened the game menu for itself.
    pub(crate) fn emote_help_choice(&mut self, choice: Option<Choice>) {
        if choice.is_some() && std::mem::take(&mut self.emote_help.opened_menu) {
            self.game_menu = false;
            self.sync_cursor_policy();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_close_keys_end_the_card_and_others_are_swallowed() {
        let mut card = Card::default();
        assert!(!card.wants());
        card.show();
        assert_eq!(card.key(KeyCode::KeyA), None);
        assert!(card.wants());
        assert_eq!(card.key(KeyCode::Escape), Some(Choice::Close));
        assert!(!card.wants());
        card.show();
        assert_eq!(card.key(KeyCode::Enter), Some(Choice::Close));
        assert!(!card.wants());
    }

    #[test]
    fn every_command_shows_and_the_card_fits_the_screen() {
        let (lines, body) = layout();
        for (command, _) in COMMANDS {
            assert!(
                lines.iter().any(|line| line.command == Some(command)),
                "{command}"
            );
            let width = command.chars().count() as f32 * 17.0 * 0.6;
            assert!(width < COMMAND_WIDTH - 12.0, "{command} is too wide");
        }
        let height = BODY_OFFSET + body + FOOT_GAP + BUTTON_HEIGHT + MARGIN;
        // Centred, clear of the key hints at the foot.
        assert!((SCREEN_HEIGHT + height) * 0.5 < KEYS_Y - 20.0, "{height}");
    }
}
