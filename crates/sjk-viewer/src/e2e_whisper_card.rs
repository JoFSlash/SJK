//! The encrypted whisper card (`docs/sjk-ui.md`, "Encrypted whisper"): an SJK pop-up
//! card over the game menu for the encrypted whispers (`e2e_whispers.rs`), in two kinds.
//!
//! - **A whisper the SJK hub did not take** ([`Failure`]): it says to whom, what and why,
//!   and offers Send unencrypted (Enter), which sends it through the game server as a
//!   plain whisper, or Don't send (Escape). Nothing goes unencrypted without Send
//!   unencrypted.
//! - **A safety code** ([`Safety`], from the chat's player card or `whisper_verify`):
//!   the code of the two players' keys to compare outside the game, with Mark verified
//!   (Enter) and Close (Escape), or, once verified, Remove the check. A verified player
//!   whose key changed says so first.
//!
//! What waits shows one after the other, oldest first. It is drawn in the SJK UI's look
//! in every menu style, and the menus under it are not drawn and take no input, as under
//! the other cards.

use crate::menu::sjk::{Frame, TextTarget, color, key_hint, key_hint_width, kit, text, wrap};
use crate::menu_widgets::{MenuCanvas, TextFamily};
use sjk_identity::whisper::Checked;
use sjk_ui::{FontWeight, InputEvent, TextAlign, UiEventKind};
use std::collections::VecDeque;
use std::net::SocketAddr;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

/// The gold button (Send unencrypted, Mark verified) and the outlined one (Don't send,
/// Close).
const MAIN_TOKEN: u16 = 1;
const OTHER_TOKEN: u16 = 2;

/// The card's left edge, top and width; it is as tall as what it holds.
const CARD_X: f32 = 560.0;
const CARD_TOP: f32 = 330.0;
const CARD_WIDTH: f32 = 800.0;
const MARGIN: f32 = 44.0;
const TEXT_X: f32 = CARD_X + MARGIN;
const TEXT_WIDTH: f32 = CARD_WIDTH - MARGIN * 2.0;
/// The explanation's first line, its step and most lines, and characters a line.
const BODY_TOP: f32 = CARD_TOP + 134.0;
const LINE: f32 = 28.0;
const LINES: usize = 7;
const LINE_CHARS: usize = 90;
/// A safety code's rows, above the explanation: four groups of five digits a row,
/// each row this tall, and the room they take with the gap under them.
const CODE_GROUPS_A_ROW: usize = 4;
const CODE_ROW: f32 = 46.0;
const CODE_HEIGHT: f32 = CODE_ROW * 3.0 + 18.0;
/// The most of the whisper quoted.
const QUOTE_CHARS: usize = 120;
/// The buttons along the card's foot, this far under the explanation.
const BUTTONS_GAP: f32 = 30.0;
const BUTTON_HEIGHT: f32 = 46.0;
const BUTTON_GAP: f32 = 14.0;
const MAIN_WIDTH: f32 = 250.0;
const OTHER_WIDTH: f32 = 170.0;
/// The keys' line, as on the other screens.
const KEYS_Y: f32 = 992.0;
const KEYS_RIGHT: f32 = 1824.0;
/// The most asks waiting.
const WAITING: usize = 8;

/// An encrypted whisper the hub did not take, or would not be sent encrypted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Failure {
    /// The game server it was written on.
    pub(crate) server: SocketAddr,
    /// The recipient's name as the game showed it.
    pub(crate) name: String,
    pub(crate) text: String,
    /// The `tell` the game server would have had.
    pub(crate) command: String,
    /// Why it did not go, in a player's words.
    pub(crate) why: String,
}

/// A player's safety code, to compare.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Safety {
    pub(crate) server: SocketAddr,
    /// Their name as the game shows it.
    pub(crate) name: String,
    /// Their key id and the key they take whispers with.
    pub(crate) key_id: String,
    pub(crate) key: String,
    /// The code ([`sjk_identity::whisper::safety_code`]).
    pub(crate) code: String,
    /// Whether this PC checked them before, and with this key.
    pub(crate) checked: Checked,
}

/// What the card asks about.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Ask {
    Failure(Failure),
    Safety(Safety),
}

impl Ask {
    fn server(&self) -> SocketAddr {
        match self {
            Self::Failure(failure) => failure.server,
            Self::Safety(safety) => safety.server,
        }
    }
}

/// What the player chose on the card.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Choice {
    SendPlain(Failure),
    Cancel,
    /// Mark this key as checked for the player.
    Verify(Safety),
    /// Forget the player's check.
    Unverify(Safety),
}

pub(crate) struct Card {
    asks: VecDeque<Ask>,
    /// Shown this frame.
    open: bool,
    ui: MenuCanvas,
    /// A world shot's asks, kept with no server.
    #[cfg(test)]
    pub(crate) preview: bool,
}

impl Default for Card {
    fn default() -> Self {
        Self {
            asks: VecDeque::new(),
            open: false,
            ui: MenuCanvas::with_capacities(16, 200, 64),
            #[cfg(test)]
            preview: false,
        }
    }
}

impl Card {
    /// Ask about `failure`, after what already waits.
    pub(crate) fn offer(&mut self, failure: Failure) {
        self.push(Ask::Failure(failure));
    }

    /// Show `safety`'s code, after what already waits.
    pub(crate) fn show_safety(&mut self, safety: Safety) {
        self.push(Ask::Safety(safety));
    }

    fn push(&mut self, ask: Ask) {
        if self.asks.len() >= WAITING {
            self.asks.pop_front();
        }
        self.asks.push_back(ask);
    }

    /// Keep only what was asked on `server`, the one the player is on.
    pub(crate) fn keep_for(&mut self, server: Option<SocketAddr>) {
        #[cfg(test)]
        if self.preview {
            return;
        }
        self.asks.retain(|ask| Some(ask.server()) == server);
    }

    /// Drop everything waiting (world shots).
    #[cfg(test)]
    pub(crate) fn clear(&mut self) {
        self.asks.clear();
    }

    /// Whether the card has something to show.
    pub(crate) fn wants(&self) -> bool {
        !self.asks.is_empty()
    }

    pub(crate) fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    pub(crate) fn is_open(&self) -> bool {
        self.open
    }

    /// Enter takes the gold button (Send unencrypted, Mark verified; Close once
    /// verified), Escape or Backspace the other; every other key is swallowed while the
    /// card shows. Space does nothing: the gold button needs a deliberate key.
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
        let verified = matches!(
            self.asks.front(),
            Some(Ask::Safety(Safety {
                checked: Checked::Yes,
                ..
            }))
        );
        match code {
            KeyCode::Enter | KeyCode::NumpadEnter if verified => self.choose(false),
            KeyCode::Enter | KeyCode::NumpadEnter => self.choose(true),
            KeyCode::Escape | KeyCode::Backspace => self.choose(false),
            _ => None,
        }
    }

    pub(crate) fn handle_pointer(&mut self, event: InputEvent) -> Option<Choice> {
        let event = self.ui.pointer(event)?;
        if event.kind != UiEventKind::Activate {
            return None;
        }
        match event.token {
            Some(MAIN_TOKEN) => self.choose(true),
            Some(OTHER_TOKEN) => self.choose(false),
            _ => None,
        }
    }

    /// What is on show is answered (`main`: its gold button, or a verified code's
    /// Remove): the next one shows, if any.
    fn choose(&mut self, main: bool) -> Option<Choice> {
        Some(match (self.asks.pop_front()?, main) {
            (Ask::Failure(failure), true) => Choice::SendPlain(failure),
            (Ask::Failure(_), false) => Choice::Cancel,
            (Ask::Safety(safety), true) if safety.checked == Checked::Yes => {
                Choice::Unverify(safety)
            }
            (Ask::Safety(safety), true) => Choice::Verify(safety),
            (Ask::Safety(_), false) => Choice::Cancel,
        })
    }

    pub(crate) fn draw_list(&self) -> &sjk_ui::DrawList {
        self.ui.draw_list()
    }

    /// What the card says under its headline: to whom, why it failed, what it says, and
    /// what sending it unencrypted means.
    fn failure_words(failure: &Failure) -> String {
        let name = crate::medals::plain_note(&failure.name);
        let quoted = crate::medals::plain_note(&failure.text);
        let quoted = match quoted.char_indices().nth(QUOTE_CHARS) {
            Some((end, _)) => format!("{}...", &quoted[..end]),
            None => quoted,
        };
        format!(
            "Your whisper to {name} could not go encrypted: {}. \u{201c}{quoted}\u{201d} \
             Send it unencrypted through the game server instead? The server, and whoever \
             runs it, can then read it.",
            failure.why
        )
    }

    /// What a safety code's card says under the code.
    fn safety_words(safety: &Safety) -> String {
        let name = crate::medals::plain_note(&safety.name);
        let compare = format!(
            "Compare this code with {name} outside the game: by voice, on Discord or in \
             person. If you both see the same code, nobody else can read your whispers, \
             not even the SJK hub."
        );
        match safety.checked {
            Checked::No => format!(
                "{compare} Mark them verified once it matches: SJK then warns you if their \
                 key ever changes."
            ),
            Checked::Yes => format!(
                "You verified {name} on this PC with this code. SJK warns you if their key \
                 changes."
            ),
            Checked::Changed => format!(
                "{name}'s key changed since you verified them: a new PC or key of theirs, or \
                 someone in between. Whispers to them do not go encrypted until you check \
                 again. {compare}"
            ),
        }
    }

    /// Draw the card over the whole frame.
    pub(crate) fn append(&mut self, target: TextTarget<'_>, viewport: [f32; 2]) {
        let Some(ask) = self.asks.front().cloned() else {
            return;
        };
        let frame = Frame::new(viewport);
        let s = frame.s;
        let (kicker, headline, code, explanation, main, other, main_key, other_key) = match &ask {
            Ask::Failure(failure) => (
                "Encrypted whisper",
                "Send it unencrypted?".to_owned(),
                None,
                Self::failure_words(failure),
                Some("Send unencrypted"),
                "Don't send",
                "send unencrypted",
                "don't send",
            ),
            Ask::Safety(safety) => (
                "Encrypted whispers",
                match safety.checked {
                    Checked::Changed => {
                        format!("{}'s key changed", crate::medals::plain_note(&safety.name))
                    }
                    _ => format!(
                        "Safety code with {}",
                        crate::medals::plain_note(&safety.name)
                    ),
                },
                Some(safety.code.clone()),
                Self::safety_words(safety),
                Some(if safety.checked == Checked::Yes {
                    "Remove the check"
                } else {
                    "Mark verified"
                }),
                "Close",
                if safety.checked == Checked::Yes {
                    "close"
                } else {
                    "mark verified"
                },
                "close",
            ),
        };
        let verified = matches!(
            &ask,
            Ask::Safety(Safety {
                checked: Checked::Yes,
                ..
            })
        );
        let lines: Vec<&str> = wrap(&explanation, LINE_CHARS).take(LINES).collect();
        let body_top = BODY_TOP + if code.is_some() { CODE_HEIGHT } else { 0.0 };
        let body_bottom = body_top + lines.len() as f32 * LINE;
        let buttons_y = body_bottom + BUTTONS_GAP;
        let height = buttons_y + BUTTON_HEIGHT + MARGIN - CARD_TOP;
        let ui = &mut self.ui;
        ui.begin_transparent(viewport);
        kit::scrim(ui, viewport);
        kit::card(ui, &frame, [CARD_X, CARD_TOP, CARD_WIDTH, height]);
        text(
            ui,
            TextFamily::Display,
            format_args!("{kicker}"),
            frame.rect(TEXT_X, CARD_TOP + 30.0, TEXT_WIDTH, 30.0),
            22.0 * s,
            color::GOLD_BRIGHT,
            FontWeight::Semibold,
            TextAlign::Start,
        );
        text(
            ui,
            TextFamily::Display,
            format_args!("{headline}"),
            frame.rect(TEXT_X, CARD_TOP + 66.0, TEXT_WIDTH, 48.0),
            36.0 * s,
            color::TEXT,
            FontWeight::Semibold,
            TextAlign::Start,
        );
        if let Some(code) = &code {
            let groups: Vec<&str> = code.split_whitespace().collect();
            for (row, groups) in groups.chunks(CODE_GROUPS_A_ROW).enumerate() {
                text(
                    ui,
                    TextFamily::Display,
                    format_args!("{}", groups.join("  ")),
                    frame.rect(
                        TEXT_X,
                        BODY_TOP - 6.0 + row as f32 * CODE_ROW,
                        TEXT_WIDTH,
                        CODE_ROW,
                    ),
                    40.0 * s,
                    if verified {
                        color::GOLD_BRIGHT
                    } else {
                        color::TEXT
                    },
                    FontWeight::Semibold,
                    TextAlign::Start,
                );
            }
        }
        for (index, line) in lines.iter().enumerate() {
            text(
                ui,
                TextFamily::Body,
                format_args!("{line}"),
                frame.rect(TEXT_X, body_top + index as f32 * LINE, TEXT_WIDTH, LINE),
                18.0 * s,
                color::MUTED,
                FontWeight::Regular,
                TextAlign::Start,
            );
        }
        let right = CARD_X + CARD_WIDTH - MARGIN;
        if let Some(main) = main {
            kit::button(
                ui,
                &frame,
                [right - MAIN_WIDTH, buttons_y, MAIN_WIDTH, BUTTON_HEIGHT],
                main,
                !verified,
                true,
                false,
                MAIN_TOKEN,
            );
        }
        kit::button(
            ui,
            &frame,
            [
                right - MAIN_WIDTH - BUTTON_GAP - OTHER_WIDTH,
                buttons_y,
                OTHER_WIDTH,
                BUTTON_HEIGHT,
            ],
            other,
            verified,
            true,
            false,
            OTHER_TOKEN,
        );
        let [right, keys_y] = frame.point(KEYS_RIGHT, KEYS_Y);
        if verified {
            let x = right - key_hint_width(&["Esc"], "close", s);
            key_hint(ui, &["Esc"], "close", x, keys_y, s);
        } else {
            let other_width = key_hint_width(&["Esc"], other_key, s);
            let main_width = key_hint_width(&["Enter"], main_key, s);
            let x = right - other_width - main_width - 18.0 * s;
            let x = key_hint(ui, &["Enter"], main_key, x, keys_y, s);
            key_hint(ui, &["Esc"], other_key, x + 18.0 * s, keys_y, s);
        }
        // Focus starts on the outlined button: a stray click or key sends nothing.
        ui.finish(OTHER_TOKEN);
        target.append(&self.ui, viewport);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failure(name: &str, text: &str, server: &str) -> Failure {
        Failure {
            server: server.parse().unwrap(),
            name: name.into(),
            text: text.into(),
            command: format!("tell 4 \"{text}\""),
            why: "the SJK hub could not be reached".into(),
        }
    }

    fn safety(checked: Checked) -> Safety {
        Safety {
            server: "1.2.3.4:29070".parse().unwrap(),
            name: "^5Creyon".into(),
            key_id: "c0ffee00c0ffee01".into(),
            key: "AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8".into(),
            code: "84017 83486 07110 59323 10299 40656 37217 30494 56215 28645 39432 28934".into(),
            checked,
        }
    }

    #[test]
    fn asks_are_answered_oldest_first() {
        let mut card = Card::default();
        assert!(!card.wants());
        card.offer(failure("Fox", "one", "1.2.3.4:29070"));
        card.offer(failure("Fox", "two", "1.2.3.4:29070"));
        card.show_safety(safety(Checked::No));
        assert_eq!(card.key(KeyCode::Space), None);
        let Some(Choice::SendPlain(sent)) = card.key(KeyCode::Enter) else {
            panic!("Enter sends");
        };
        assert_eq!(sent.command, "tell 4 \"one\"");
        assert_eq!(card.key(KeyCode::Escape), Some(Choice::Cancel));
        assert_eq!(
            card.key(KeyCode::Enter),
            Some(Choice::Verify(safety(Checked::No)))
        );
        assert!(!card.wants());
        assert_eq!(card.key(KeyCode::Enter), None);
    }

    #[test]
    fn a_verified_code_closes_on_enter_and_is_removed_only_by_its_button() {
        let mut card = Card::default();
        card.show_safety(safety(Checked::Yes));
        assert_eq!(card.key(KeyCode::Enter), Some(Choice::Cancel));
        card.show_safety(safety(Checked::Yes));
        assert_eq!(
            card.choose(true),
            Some(Choice::Unverify(safety(Checked::Yes)))
        );
        card.show_safety(safety(Checked::Changed));
        assert_eq!(
            card.key(KeyCode::Enter),
            Some(Choice::Verify(safety(Checked::Changed)))
        );
    }

    #[test]
    fn leaving_the_server_drops_its_asks() {
        let mut card = Card::default();
        card.offer(failure("Fox", "one", "1.2.3.4:29070"));
        card.offer(failure("Fox", "two", "5.6.7.8:29070"));
        card.keep_for(Some("5.6.7.8:29070".parse().unwrap()));
        assert_eq!(card.asks.len(), 1);
        card.keep_for(None);
        assert!(!card.wants());
    }

    #[test]
    fn the_words_say_to_whom_why_and_what() {
        let words = Card::failure_words(&failure("^1Fox", "^3meet me", "1.2.3.4:29070"));
        assert!(
            words.starts_with(
                "Your whisper to Fox could not go encrypted: the SJK hub could not be \
                 reached. \u{201c}meet me\u{201d}"
            ),
            "{words}"
        );
        let longest =
            Card::failure_words(&failure(&"N".repeat(36), &"a ".repeat(450), "1.2.3.4:1"));
        assert!(wrap(&longest, LINE_CHARS).count() <= LINES, "{longest}");
        for checked in [Checked::No, Checked::Yes, Checked::Changed] {
            let words = Card::safety_words(&Safety {
                name: "N".repeat(36),
                ..safety(checked)
            });
            assert!(wrap(&words, LINE_CHARS).count() <= LINES, "{words}");
        }
    }
}
