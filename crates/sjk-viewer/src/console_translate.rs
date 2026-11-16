//! Click a chat line in the console to translate it (`docs/client.md`, "Chat
//! translation"), as a click on a line's text does in the chat box.
//!
//! The console keeps, beside its bounded scrollback, the plain text of each chat line
//! another player sent ([`ViewerConsole::offer_translation`]): never the player's own
//! lines, end-to-end encrypted whispers, holocron drops nor the SJK team's lines. A
//! faint band shows under the pointer resting on such a line; a click on it (a press
//! and release with nothing selected, not the first press of a double click) hands its
//! text to the frame, which asks the hub as `chat/translate.rs` does.
//!
//! The translation, `» (fr) ...` (or why there is none), shows right under the line
//! clicked, however much chat came since: the scrollback only grows at its end, so it
//! is no scrollback line of its own but kept here beside the line's number
//! ([`ViewerConsole::put_under`]) and drawn as that line's next rows
//! (`console_classic.rs`).

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Chat lines kept for a click, the newest.
const MAX_LINES: usize = 256;
/// Translations kept under their lines, the newest.
const MAX_UNDER: usize = 64;
/// Translations asked from the console waiting for the hub.
const MAX_ASKED: usize = 8;
/// A click waits this long for a second press (a double click selects a word).
const CLICK_WAIT: Duration = super::selection::DOUBLE_CLICK;

/// A chat line that may be translated.
struct Offer {
    line: u64,
    text: String,
    /// Its translation was asked for: a click does nothing more.
    asked: bool,
}

#[derive(Default)]
pub(super) struct State {
    offers: VecDeque<Offer>,
    /// A line clicked, waiting out [`CLICK_WAIT`] for a second press.
    pending: Option<(u64, Instant)>,
    /// A line whose click is ready for the frame: its number and text.
    ready: Option<(u64, String)>,
    /// Translations asked for a line: the serial of the ask and the line's number.
    asked: VecDeque<(u64, u64)>,
    /// What shows under a line: its number and the text.
    under: VecDeque<(u64, String)>,
}

impl State {
    fn offer(&mut self, line: u64, text: String) {
        if self.offers.len() == MAX_LINES {
            self.offers.pop_front();
        }
        self.offers.push_back(Offer {
            line,
            text,
            asked: false,
        });
    }

    /// Whether a click on scrollback line `line` translates it.
    pub(super) fn offered(&self, line: u64) -> bool {
        self.offers
            .iter()
            .rev()
            .any(|offer| offer.line == line && !offer.asked)
    }

    /// A click on scrollback line `line` came at `now`.
    pub(super) fn clicked(&mut self, line: u64, now: Instant) {
        if self.offered(line) {
            self.pending = Some((line, now));
        }
    }

    /// Another press came: a waiting click was the start of a double click.
    pub(super) fn pressed(&mut self) {
        self.pending = None;
    }

    /// The text of the line clicked, once its wait is over; it is asked for then.
    fn take(&mut self, now: Instant) -> Option<(u64, String)> {
        if let Some((line, at)) = self.pending
            && now.duration_since(at) >= CLICK_WAIT
        {
            self.pending = None;
            if let Some(offer) = self
                .offers
                .iter_mut()
                .rev()
                .find(|offer| offer.line == line && !offer.asked)
            {
                offer.asked = true;
                self.ready = Some((line, offer.text.clone()));
            }
        }
        self.ready.take()
    }

    /// The line may be clicked again: its translation was not asked after all.
    fn unask(&mut self, line: u64) {
        if let Some(offer) = self
            .offers
            .iter_mut()
            .rev()
            .find(|offer| offer.line == line)
        {
            offer.asked = false;
        }
    }

    fn note_asked(&mut self, serial: u64, line: u64) {
        if self.asked.len() == MAX_ASKED {
            self.asked.pop_front();
        }
        self.asked.push_back((serial, line));
    }

    /// The line the ask of `serial` was for, once.
    fn answered(&mut self, serial: u64) -> Option<u64> {
        let index = self.asked.iter().position(|(asked, _)| *asked == serial)?;
        self.asked.remove(index).map(|(_, line)| line)
    }

    /// Show `text` under line `line`, in place of what showed there.
    fn put_under(&mut self, line: u64, text: String) {
        self.under.retain(|(number, _)| *number != line);
        if self.under.len() == MAX_UNDER {
            self.under.pop_front();
        }
        self.under.push_back((line, text));
    }

    /// What shows under line `line`.
    pub(super) fn under(&self, line: u64) -> Option<&str> {
        self.under
            .iter()
            .find(|(number, _)| *number == line)
            .map(|(_, text)| text.as_str())
    }
}

impl super::ViewerConsole {
    /// Scrollback line `line` (from [`Self::push_chat_line`]) is chat another player
    /// sent, `text` its plain words: a click on it may translate them.
    pub(crate) fn offer_translation(&mut self, line: u64, text: String) {
        if !text.is_empty() {
            self.chat_translate.offer(line, text);
        }
    }

    /// Whether clicks translate chat lines (`cl_chatTranslate`).
    pub(super) fn translate_clicks_on(&self) -> bool {
        self.bool_cvar(crate::chat::translate::CVAR) != Some(false)
    }

    /// The chat line clicked to translate, once: its number and its text.
    pub(crate) fn take_translate_click(&mut self, now: Instant) -> Option<(u64, String)> {
        let click = self.chat_translate.take(now)?;
        self.translate_clicks_on().then_some(click)
    }

    /// The line of `line` may be clicked again (nothing was asked).
    pub(crate) fn unask_translation(&mut self, line: u64) {
        self.chat_translate.unask(line);
    }

    /// The hub was asked, under `serial`, for the translation of line `line`.
    pub(crate) fn translation_asked(&mut self, serial: u64, line: u64) {
        self.chat_translate.note_asked(serial, line);
    }

    /// Show `text` (with its colour codes) right under line `line`.
    pub(crate) fn put_under(&mut self, line: u64, text: String) {
        self.chat_translate.put_under(line, text);
    }

    /// The answer `text` to the ask of `serial`: under its line when a click in the
    /// console asked it, otherwise given back for the end of the scrollback.
    pub(crate) fn translation_answered(&mut self, serial: u64, text: String) -> Option<String> {
        let text = whiter(text);
        match self.chat_translate.answered(serial) {
            Some(line) => {
                self.chat_translate.put_under(line, text);
                None
            }
            None => Some(text),
        }
    }
}

/// A translation in the console is white, not the feed's dim grey (`^9`): the console's
/// grey on its dark backdrop was hard to read.
fn whiter(text: String) -> String {
    match text.strip_prefix("^9") {
        Some(rest) => format!("^7{rest}"),
        None => text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_translation_is_white_in_the_console() {
        assert_eq!(whiter("^9\u{bb} (fr) hi".into()), "^7\u{bb} (fr) hi");
        assert_eq!(whiter("^3Translate: off".into()), "^3Translate: off");
    }

    #[test]
    fn a_click_waits_out_a_double_click_and_asks_once() {
        let mut state = State::default();
        let t0 = Instant::now();
        state.offer(7, "salut".into());
        assert!(!state.offered(6));
        state.clicked(7, t0);
        assert_eq!(state.take(t0), None, "a second press may still come");
        let later = t0 + CLICK_WAIT;
        assert_eq!(state.take(later), Some((7, "salut".to_owned())));
        assert!(!state.offered(7), "asked already");
        state.clicked(7, later);
        assert_eq!(state.take(later + CLICK_WAIT), None);
        state.unask(7);
        assert!(state.offered(7));
    }

    #[test]
    fn a_second_press_makes_it_a_double_click() {
        let mut state = State::default();
        let t0 = Instant::now();
        state.offer(3, "hola amigos".into());
        state.clicked(3, t0);
        state.pressed();
        assert_eq!(state.take(t0 + CLICK_WAIT), None);
        assert!(state.offered(3));
    }

    #[test]
    fn an_answer_goes_under_the_line_asked_for() {
        let mut state = State::default();
        state.note_asked(5, 40);
        assert_eq!(state.answered(4), None, "another ask's");
        assert_eq!(state.answered(5), Some(40));
        assert_eq!(state.answered(5), None, "once");
        state.put_under(40, "^3Translate: limit".into());
        state.put_under(40, "^9\u{bb} (fr) hi".into());
        assert_eq!(state.under(40), Some("^9\u{bb} (fr) hi"));
        assert_eq!(state.under(41), None);
    }

    #[test]
    fn lines_not_offered_are_never_clicked() {
        let mut state = State::default();
        let t0 = Instant::now();
        state.clicked(1, t0);
        assert_eq!(state.take(t0 + CLICK_WAIT), None);
    }
}
