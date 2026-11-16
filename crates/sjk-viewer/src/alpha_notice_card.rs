//! The alpha notice (`docs/sjk-ui.md`, "Alpha notice"): the first time the game starts,
//! an SJK pop-up card over the darkened screen says SJK is an alpha build of a free and
//! open-source project, that bugs and unfinished parts are to be expected, how to report
//! them (the SJK Discord, or Report a bug, the game menu's icon, whose dialog the card
//! opens too), and that SJK comes with no warranty and owes the player nothing. Continue
//! (gold, Enter) only works once the player ticks all three promises ([`PROMISES`]: I
//! understand; I won't DM Sol how the game works; I won't DM Sol bugs or ideas but report
//! them); Quit leaves the game. There is no way past it but those two.
//!
//! `ui_alphaNotice` holds the notice's version the player accepted: raising
//! [`VERSION`] when the words change shows the card again. It shows over the menus, never
//! in the middle of play, before every other card and First setup, in the SJK UI's look
//! in every menu style; the menus under it are not drawn and take no input.

use crate::menu::sjk::{Frame, TextTarget, color, key_hint, key_hint_width, kit, text, wrap};
use crate::menu_widgets::{MenuCanvas, TextFamily};
use sjk_ui::{Color, DrawCommand, FontWeight, InputEvent, TextAlign, UiEventKind};
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

/// The notice's version the player accepted.
pub(crate) const CVAR: &str = "ui_alphaNotice";
/// This notice's version: raise it when the words change, to ask again.
pub(crate) const VERSION: i64 = 2;
/// The SJK Discord's permanent invite (to #welcome; it never expires).
pub(crate) const DISCORD_URL: &str = "https://discord.gg/dGBdMygvJw";

const CONTINUE_TOKEN: u16 = 1;
const QUIT_TOKEN: u16 = 2;
const DISCORD_TOKEN: u16 = 4;
const REPORT_TOKEN: u16 = 5;
/// The promises' ticks, `TICK_TOKEN + n`.
const TICK_TOKEN: u16 = 10;

/// The card's left edge and width; it is as tall as what it holds, centred.
const CARD_X: f32 = 510.0;
const CARD_WIDTH: f32 = 900.0;
const MARGIN: f32 = 44.0;
const TEXT_X: f32 = CARD_X + MARGIN;
const TEXT_WIDTH: f32 = CARD_WIDTH - MARGIN * 2.0;
/// The explanation's first line under the card's top, its step, and characters a line.
const BODY: f32 = 134.0;
const LINE: f32 = 28.0;
const LINE_CHARS: usize = 98;
const PARAGRAPH_GAP: f32 = 12.0;
/// The report row: the game menu's Report a bug icon (as big as there) with its name,
/// and Join the Discord.
const ROW_GAP: f32 = 22.0;
const ROW_HEIGHT: f32 = 54.0;
const REPORT_WIDTH: f32 = 330.0;
const DISCORD_WIDTH: f32 = 230.0;
/// The ticks' lines and the buttons.
const TICK_GAP: f32 = 30.0;
const PROMISE_GAP: f32 = 14.0;
const TICK_CHARS: usize = 94;
const FOOT_GAP: f32 = 26.0;
const BUTTON_HEIGHT: f32 = 46.0;
const BUTTON_GAP: f32 = 14.0;
const CONTINUE_WIDTH: f32 = 200.0;
const QUIT_WIDTH: f32 = 130.0;
const KEYS_Y: f32 = 1020.0;
const KEYS_RIGHT: f32 = 1824.0;

/// What the card says above the report row.
const PARAGRAPHS: [&str; 2] = [
    "This is an alpha build of Sol JK, a free and open-source Jedi Academy client. Expect \
     bugs, and parts that are not finished yet. Some collectibles show in the game but \
     cannot be unlocked yet: they are a work in progress, so there is no need to ask Sol \
     about them.",
    "Found a bug, or something that feels wrong? Tell us on the SJK Discord, or send it with \
     Report a bug, this icon in the game menu (Escape).",
];
/// What it says under the report row.
const TERMS: &str = "SJK is a fun project that Sol and its contributors make in their free \
     time. It comes with no warranty of any kind (as its GPL licence says), and nobody owes \
     you support, fixes or features.";
/// What the player ticks, every one before Continue.
pub(crate) const PROMISES: [&str; 3] = [
    "I understand that SJK is a work in progress that may have bugs, that it comes with no \
     warranty, and that I am owed nothing.",
    "I won't DM Sol to ask how the game works: I will do my best to figure it out myself.",
    "I won't DM Sol bugs or ideas: I will report them with Report a bug or on the SJK Discord.",
];

/// What the player chose on the card.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Choice {
    /// Flip promise `n`.
    Tick(usize),
    /// Accepted: the card goes for good.
    Continue,
    Quit,
    Discord,
    /// Open the Report a bug dialog; the card waits under it.
    Report,
}

pub(crate) struct Card {
    /// Not accepted yet.
    pending: bool,
    ticked: [bool; PROMISES.len()],
    /// Shown this frame.
    open: bool,
    ui: MenuCanvas,
}

impl Default for Card {
    fn default() -> Self {
        Self {
            pending: false,
            ticked: [false; PROMISES.len()],
            open: false,
            ui: MenuCanvas::with_capacities(48, 2048, 256),
        }
    }
}

impl Card {
    /// The version the player accepted, read from `ui_alphaNotice`.
    pub(crate) fn offer(&mut self, accepted: i64) {
        self.pending = accepted < VERSION;
    }

    /// Whether the card has something to show.
    pub(crate) fn wants(&self) -> bool {
        self.pending
    }

    pub(crate) fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    pub(crate) fn is_open(&self) -> bool {
        self.open
    }

    /// Space ticks the first promise not ticked yet (unticks the last when all are), 1 to 3
    /// flip a promise, Enter continues once all are ticked; every other key, Escape too, is
    /// swallowed while the card shows.
    pub(crate) fn handle_key(&mut self, event: &KeyEvent) -> Option<Choice> {
        if event.state != ElementState::Pressed || event.repeat {
            return None;
        }
        let PhysicalKey::Code(code) = event.physical_key else {
            return None;
        };
        self.key(code)
    }

    /// [`Self::handle_key`] for a key code, for the world shots.
    #[cfg(test)]
    pub(crate) fn handle_key_code_for_test(&mut self, code: KeyCode) -> Option<Choice> {
        self.key(code)
    }

    fn key(&mut self, code: KeyCode) -> Option<Choice> {
        let choice = match code {
            KeyCode::Space => Choice::Tick(
                self.ticked
                    .iter()
                    .position(|ticked| !ticked)
                    .unwrap_or(PROMISES.len() - 1),
            ),
            KeyCode::Digit1 | KeyCode::Numpad1 => Choice::Tick(0),
            KeyCode::Digit2 | KeyCode::Numpad2 => Choice::Tick(1),
            KeyCode::Digit3 | KeyCode::Numpad3 => Choice::Tick(2),
            KeyCode::Enter | KeyCode::NumpadEnter => Choice::Continue,
            _ => return None,
        };
        self.choose(choice)
    }

    pub(crate) fn handle_pointer(&mut self, event: InputEvent) -> Option<Choice> {
        let event = self.ui.pointer(event)?;
        if event.kind != UiEventKind::Activate {
            return None;
        }
        let choice = match event.token {
            Some(CONTINUE_TOKEN) => Choice::Continue,
            Some(QUIT_TOKEN) => Choice::Quit,
            Some(DISCORD_TOKEN) => Choice::Discord,
            Some(REPORT_TOKEN) => Choice::Report,
            Some(token) if (TICK_TOKEN..TICK_TOKEN + PROMISES.len() as u16).contains(&token) => {
                Choice::Tick(usize::from(token - TICK_TOKEN))
            }
            _ => return None,
        };
        self.choose(choice)
    }

    /// Whether every promise is ticked.
    fn all_ticked(&self) -> bool {
        self.ticked.iter().all(|ticked| *ticked)
    }

    /// A tick flips; Continue needs them all and puts the card away.
    fn choose(&mut self, choice: Choice) -> Option<Choice> {
        match choice {
            Choice::Tick(index) => self.ticked[index] = !self.ticked[index],
            Choice::Continue if !self.all_ticked() => return None,
            Choice::Continue => self.pending = false,
            Choice::Quit | Choice::Discord | Choice::Report => {}
        }
        Some(choice)
    }

    pub(crate) fn draw_list(&self) -> &sjk_ui::DrawList {
        self.ui.draw_list()
    }

    /// Draw the card over the whole frame.
    pub(crate) fn append(&mut self, target: TextTarget<'_>, viewport: [f32; 2]) {
        let frame = Frame::new(viewport);
        let s = frame.s;
        // Lay out from the card's top, then centre it.
        let mut y = BODY;
        let mut lines: Vec<(f32, &str)> = Vec::new();
        for paragraph in PARAGRAPHS {
            for line in wrap(paragraph, LINE_CHARS) {
                lines.push((y, line));
                y += LINE;
            }
            y += PARAGRAPH_GAP;
        }
        let row = y - PARAGRAPH_GAP + ROW_GAP;
        y = row + ROW_HEIGHT + ROW_GAP;
        for line in wrap(TERMS, LINE_CHARS) {
            lines.push((y, line));
            y += LINE;
        }
        // Each promise's top and lines.
        let mut promises: Vec<(f32, Vec<&str>)> = Vec::new();
        y += TICK_GAP;
        for promise in PROMISES {
            let promise_lines: Vec<&str> = wrap(promise, TICK_CHARS).collect();
            let next = y + promise_lines.len() as f32 * LINE + PROMISE_GAP;
            promises.push((y, promise_lines));
            y = next;
        }
        let foot = y - PROMISE_GAP + FOOT_GAP;
        let height = foot + BUTTON_HEIGHT + MARGIN;
        let top = ((1080.0 - height) * 0.5).max(20.0);
        let ticks = self.ticked;
        let all_ticked = self.all_ticked();
        let ui = &mut self.ui;
        ui.begin_transparent(viewport);
        kit::scrim(ui, viewport);
        kit::card(ui, &frame, [CARD_X, top, CARD_WIDTH, height]);
        text(
            ui,
            TextFamily::Display,
            format_args!("Alpha build"),
            frame.rect(TEXT_X, top + 30.0, TEXT_WIDTH, 30.0),
            22.0 * s,
            color::GOLD_BRIGHT,
            FontWeight::Semibold,
            TextAlign::Start,
        );
        text(
            ui,
            TextFamily::Display,
            format_args!("Welcome to Sol JK"),
            frame.rect(TEXT_X, top + 66.0, TEXT_WIDTH, 48.0),
            36.0 * s,
            color::TEXT,
            FontWeight::Semibold,
            TextAlign::Start,
        );
        for (line_y, line) in &lines {
            text(
                ui,
                TextFamily::Body,
                format_args!("{line}"),
                frame.rect(TEXT_X, top + line_y, TEXT_WIDTH, LINE),
                18.0 * s,
                color::MUTED,
                FontWeight::Regular,
                TextAlign::Start,
            );
        }
        // Report a bug as the game menu's row of icons draws it: its picture, ringed gold
        // while hovered, its name beside it; a click opens the dialog.
        let row = top + row;
        let lit = ui.token_hovered(REPORT_TOKEN);
        let icon = frame.rect(TEXT_X, row, ROW_HEIGHT, ROW_HEIGHT);
        let picture = crate::ingame_menu::sjk_focus::Icon::ReportBug.picture();
        if let Some(texture) = crate::settings_icons::texture(picture) {
            let _ = ui.draw_list_mut().push(DrawCommand::TexturedQuad {
                rect: icon,
                texture,
                color: Color::new(1.0, 1.0, 1.0, if lit { 1.0 } else { 0.78 }),
            });
        }
        if lit {
            let ring = frame.rect(TEXT_X - 3.0, row - 3.0, ROW_HEIGHT + 6.0, ROW_HEIGHT + 6.0);
            let _ = ui.draw_list_mut().push(DrawCommand::Border {
                rect: ring,
                radius: ring.height * 0.5,
                width: 2.0 * s,
                color: color::alpha(color::GOLD, 0.7),
            });
        }
        let name_x = TEXT_X + ROW_HEIGHT + 16.0;
        text(
            ui,
            TextFamily::Display,
            format_args!("Report a bug"),
            frame.rect(name_x, row + 2.0, 240.0, 28.0),
            22.0 * s,
            if lit { color::GOLD_BRIGHT } else { color::TEXT },
            FontWeight::Regular,
            TextAlign::Start,
        );
        text(
            ui,
            TextFamily::Body,
            format_args!("In the game menu (Escape)"),
            frame.rect(name_x, row + 28.0, 240.0, 24.0),
            16.0 * s,
            color::QUIET,
            FontWeight::Regular,
            TextAlign::Start,
        );
        ui.hit_region(
            REPORT_TOKEN,
            frame.rect(TEXT_X - 6.0, row - 6.0, REPORT_WIDTH, ROW_HEIGHT + 12.0),
        );
        let right = CARD_X + CARD_WIDTH - MARGIN;
        kit::button(
            ui,
            &frame,
            [right - DISCORD_WIDTH, row, DISCORD_WIDTH, ROW_HEIGHT],
            "Join the SJK Discord",
            false,
            true,
            false,
            DISCORD_TOKEN,
        );
        text(
            ui,
            TextFamily::Body,
            format_args!("discord.gg/dGBdMygvJw"),
            frame.rect(right - DISCORD_WIDTH - 236.0, row + 13.0, 220.0, LINE),
            16.0 * s,
            color::QUIET,
            FontWeight::Regular,
            TextAlign::End,
        );
        // The promises, which Continue needs.
        for (index, (promise_top, promise_lines)) in promises.iter().enumerate() {
            let promise_top = top + promise_top;
            let ticked = ticks[index];
            let after = kit::tick(ui, &frame, TEXT_X, promise_top + LINE * 0.5, ticked, false);
            for (line_index, line) in promise_lines.iter().enumerate() {
                text(
                    ui,
                    TextFamily::Body,
                    format_args!("{line}"),
                    frame.rect(
                        after + 14.0,
                        promise_top + line_index as f32 * LINE,
                        TEXT_WIDTH,
                        LINE,
                    ),
                    18.0 * s,
                    if ticked {
                        color::TEXT
                    } else {
                        color::GOLD_BRIGHT
                    },
                    FontWeight::Regular,
                    TextAlign::Start,
                );
            }
            ui.hit_region(
                TICK_TOKEN + index as u16,
                frame.rect(
                    TEXT_X - 8.0,
                    promise_top - 6.0,
                    TEXT_WIDTH + 16.0,
                    promise_lines.len() as f32 * LINE + 12.0,
                ),
            );
        }
        let foot = top + foot;
        kit::button(
            ui,
            &frame,
            [right - CONTINUE_WIDTH, foot, CONTINUE_WIDTH, BUTTON_HEIGHT],
            "Continue",
            true,
            all_ticked,
            false,
            CONTINUE_TOKEN,
        );
        kit::button(
            ui,
            &frame,
            [
                right - CONTINUE_WIDTH - BUTTON_GAP - QUIT_WIDTH,
                foot,
                QUIT_WIDTH,
                BUTTON_HEIGHT,
            ],
            "Quit",
            false,
            true,
            false,
            QUIT_TOKEN,
        );
        let [right, keys_y] = frame.point(KEYS_RIGHT, KEYS_Y);
        let hints: [(&[&str], &str); 2] = [(&["Space"], "tick the next"), (&["Enter"], "continue")];
        let width: f32 = hints
            .iter()
            .map(|(keys, label)| key_hint_width(keys, label, s) + 18.0 * s)
            .sum();
        let mut x = right - width;
        for (keys, label) in hints {
            x = key_hint(ui, keys, label, x, keys_y, s) + 18.0 * s;
        }
        let first_open = ticks.iter().position(|ticked| !ticked);
        ui.finish(match first_open {
            None => CONTINUE_TOKEN,
            Some(index) => TICK_TOKEN + index as u16,
        });
        target.append(&self.ui, viewport);
    }
}

impl crate::GpuState {
    /// Whether the card shows this frame: not accepted yet, and the player on a menu or
    /// the game menu. Only the console covers it; every other card waits for it.
    pub(crate) fn prepare_alpha_card(&mut self) -> bool {
        let accepted = self
            .console
            .as_ref()
            .and_then(|console| console.integer_cvar(CVAR))
            .unwrap_or(0);
        self.alpha_card.offer(accepted);
        let menu_visible = self
            .client_menu
            .as_ref()
            .is_some_and(crate::menu::ClientMenu::is_visible);
        let console_open = self
            .console
            .as_ref()
            .is_some_and(crate::console::ViewerConsole::is_open);
        let open = self.alpha_card.wants()
            && (menu_visible || self.game_menu)
            && !console_open
            && !self.text_dialog.is_open()
            && self.world_load_task.is_none()
            && self.world_install_task.is_none()
            && !self.pending_map_reload;
        self.alpha_card.set_open(open);
        open
    }

    /// Draw the card. It darkens everything, so every font batch appended before it is
    /// dropped (text draws above all shapes).
    pub(crate) fn append_alpha_card(&mut self, viewport: [f32; 2]) {
        self.text_vertices.clear();
        self.classic_text_vertices.clear();
        self.game_fonts.clear_text();
        let target = crate::ingame_menu::sjk_view::text_target(
            &mut self.game_fonts,
            &mut self.text_vertices,
            &self.ui_font,
        );
        self.alpha_card.append(target, viewport);
    }

    /// Act on the player's choice on the card.
    pub(crate) fn alpha_card_choice(&mut self, choice: Option<Choice>) {
        match choice {
            Some(Choice::Continue) => {
                if let Some(console) = &mut self.console {
                    console.set_cvar(CVAR, &VERSION.to_string());
                }
                crate::log::progress(format_args!("alpha notice: accepted"));
            }
            Some(Choice::Quit) => self.quit_requested = true,
            Some(Choice::Discord) => crate::update::open_page(DISCORD_URL),
            // The dialog takes the screen; the card comes back when it closes.
            Some(Choice::Report) => self.open_bug_report(),
            Some(Choice::Tick(_)) | None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continue_needs_every_tick_and_puts_the_card_away() {
        let mut card = Card::default();
        card.offer(0);
        assert!(card.wants());
        // Escape and other keys do nothing; Enter does nothing before the ticks.
        assert_eq!(card.key(KeyCode::Escape), None);
        assert_eq!(card.key(KeyCode::KeyA), None);
        assert_eq!(card.key(KeyCode::Enter), None);
        assert!(card.wants());
        // Space ticks the next one; a number flips its own.
        assert_eq!(card.key(KeyCode::Space), Some(Choice::Tick(0)));
        assert_eq!(card.key(KeyCode::Digit3), Some(Choice::Tick(2)));
        assert_eq!(card.key(KeyCode::Enter), None);
        assert_eq!(card.key(KeyCode::Space), Some(Choice::Tick(1)));
        assert!(card.all_ticked());
        // All ticked, Space unticks the last.
        assert_eq!(card.key(KeyCode::Space), Some(Choice::Tick(2)));
        assert_eq!(card.key(KeyCode::Enter), None);
        assert_eq!(card.key(KeyCode::Digit3), Some(Choice::Tick(2)));
        assert_eq!(card.key(KeyCode::Enter), Some(Choice::Continue));
        assert!(!card.wants());
    }

    #[test]
    fn an_accepted_version_hides_the_card_and_a_newer_one_asks_again() {
        let mut card = Card::default();
        card.offer(VERSION);
        assert!(!card.wants());
        card.offer(VERSION - 1);
        assert!(card.wants());
    }

    #[test]
    fn the_cvar_is_defined_and_starts_unaccepted() {
        let directory = tempfile::tempdir().unwrap();
        let console =
            crate::console::ViewerConsole::new(directory.path().join("config.cfg")).unwrap();
        assert_eq!(console.integer_cvar(CVAR), Some(0));
    }

    #[test]
    fn the_invite_is_the_permanent_one_and_https() {
        assert!(DISCORD_URL.starts_with("https://discord.gg/"));
    }
}
