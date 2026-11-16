//! Open all's question (`docs/cards.md`, "In the client"; Sol's request of 21/10/2026):
//! before more than one holocron opens at once, a card over the darkened tab asks "Open
//! all 7 holocrons?". More than [`ONE_BY_ONE`] open together in the batch's quicker,
//! simpler animation ([`super::batch`]: five small strips at a time, then the finds laid
//! out, without each card's own pack, flip and words), and the card says so; two or three
//! still play their full ceremony one after another, and the card says that instead. One
//! holocron opens at once, as Open does.
//!
//! Open all (gold) and Cancel: Enter or Space presses the one with the focus (Open all
//! first), Tab, Shift+Tab, Left and Right move it, Escape cancels; a click on either
//! answers. The page under the card is not drawn: text draws over every shape.

use super::opening::ONE_BY_ONE;
use super::*;
use crate::audio::ui_cues::{self, Cue};
use crate::menu::sjk::{Frame, color, key_hint, key_hint_width, kit, text, wrap};
use crate::menu_widgets::TextFamily;
use sjk_ui::{FontWeight, TextAlign};

/// The card's Open all and Cancel.
pub(crate) const ASK_ALL_OPEN_TOKEN: u16 = 1_327;
pub(crate) const ASK_ALL_CANCEL_TOKEN: u16 = 1_328;
/// The card (frame pixels): its left and width, the middle it is centred on, and its
/// height without the words' lines, each of which adds [`LINE`].
const CARD_X: f32 = 560.0;
const CARD_WIDTH: f32 = 800.0;
const CARD_MIDDLE: f32 = 535.0;
const CARD_BASE: f32 = 212.0;
const LINE: f32 = 27.0;
/// The words' width in characters, and the most lines they take.
const WRAP: usize = 88;
const LINES: usize = 4;

/// Which of the card's buttons has the focus.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AskAll {
    Open,
    Cancel,
}

/// The card's title and words for `opening` holocrons (or kyber crystals, `crystals`) of
/// the `held` the page can open.
pub(super) fn words(opening: usize, held: usize, crystals: bool) -> (String, Vec<String>) {
    let things = crate::terms::say(if crystals {
        "kyber crystals"
    } else {
        "holocrons"
    });
    let title = if held > opening {
        format!("Open {opening} of your {held} {things}?")
    } else {
        format!("Open all {opening} {things}?")
    };
    let mut lines: Vec<String> = Vec::new();
    let body = if opening > ONE_BY_ONE && crystals {
        "They open together, in a quicker and simpler animation than opening them one by \
         one: the strips run five at a time and every shader found is laid out at the end, \
         without each shader's own reveal."
    } else if opening > ONE_BY_ONE {
        "They open together, in a quicker and simpler animation than opening them one by \
         one: the strips run five at a time and everything found is laid out at the end, \
         without each card's own reveal."
    } else {
        "Each one plays its full opening, one after another."
    };
    lines.extend(wrap(body, WRAP).map(str::to_owned));
    if held > opening {
        let rest = held - opening;
        let line = format!(
            "{} open at most at once; the other {rest} wait for the next time.",
            sjk_identity::OPEN_MAX
        );
        lines.extend(wrap(&line, WRAP).map(str::to_owned));
    }
    lines.truncate(LINES);
    (title, lines)
}

impl Panel {
    /// Open all (the button or A): more than one holocron asks first; one opens at once.
    pub(super) fn open_all(&mut self, now: Instant) -> PanelAction {
        let Some(ids) = self.open_every() else {
            return PanelAction::None;
        };
        if ids.len() > 1 {
            self.ask_all = Some(AskAll::Open);
            ui_cues::post(Cue::Click);
            return PanelAction::None;
        }
        self.ask(ids, now)
    }

    /// Whether Open all's question shows.
    #[cfg(test)]
    pub(crate) fn asking_all(&self) -> bool {
        self.ask_all.is_some()
    }

    /// Show Open all's question, for a world shot.
    #[cfg(test)]
    pub(crate) fn ask_all_for_shot(&mut self) {
        self.ask_all = Some(AskAll::Open);
    }

    /// The question answered: Open all asks the hub for every one it would open now.
    fn answer_all(&mut self, open: bool, now: Instant) -> PanelAction {
        self.ask_all = None;
        if !open {
            ui_cues::post(Cue::Click);
            return PanelAction::None;
        }
        self.open_every()
            .map_or(PanelAction::None, |ids| self.ask(ids, now))
    }

    /// A key while the question shows.
    pub(super) fn ask_all_key(&mut self, key: KeyCode, now: Instant) -> PanelAction {
        let Some(focus) = self.ask_all else {
            return PanelAction::None;
        };
        match key {
            KeyCode::Escape => self.answer_all(false, now),
            KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space => {
                self.answer_all(focus == AskAll::Open, now)
            }
            KeyCode::Tab | KeyCode::ArrowLeft | KeyCode::ArrowRight => {
                self.ask_all = Some(match focus {
                    AskAll::Open => AskAll::Cancel,
                    AskAll::Cancel => AskAll::Open,
                });
                PanelAction::None
            }
            _ => PanelAction::None,
        }
    }

    /// A pointer event while the question shows: only its two buttons answer.
    pub(super) fn ask_all_pointer(&mut self, event: InputEvent, now: Instant) -> PanelAction {
        if matches!(event, InputEvent::PointerWheel { .. }) {
            return PanelAction::None;
        }
        match self.ui.pointer(event) {
            Some(event) if event.kind == UiEventKind::Activate => match event.token {
                Some(ASK_ALL_OPEN_TOKEN) => self.answer_all(true, now),
                Some(ASK_ALL_CANCEL_TOKEN) => self.answer_all(false, now),
                _ => PanelAction::None,
            },
            _ => PanelAction::None,
        }
    }

    /// Draw the question over the darkened tab.
    pub(super) fn draw_ask_all(&mut self, frame: &Frame, viewport: [f32; 2]) {
        let s = frame.s;
        let focus = self.ask_all.unwrap_or(AskAll::Open);
        let opening = self.open_every().map_or(0, |ids| ids.len());
        let (title, lines) = words(
            opening,
            self.data.openable(self.selected),
            self.selected().is_kyber(),
        );
        let height = CARD_BASE + LINE * lines.len() as f32;
        let card = [CARD_X, CARD_MIDDLE - height * 0.5, CARD_WIDTH, height];
        kit::scrim(&mut self.ui, viewport);
        kit::card(&mut self.ui, frame, card);
        let [x, y, width, height] = card;
        text(
            &mut self.ui,
            TextFamily::Display,
            format_args!("{title}"),
            frame.rect(x + 40.0, y + 30.0, width - 80.0, 50.0),
            36.0 * s,
            color::TEXT,
            FontWeight::Semibold,
            TextAlign::Start,
        );
        for (at, line) in lines.iter().enumerate() {
            text(
                &mut self.ui,
                TextFamily::Body,
                format_args!("{line}"),
                frame.rect(x + 40.0, y + 96.0 + at as f32 * LINE, width - 80.0, 26.0),
                18.0 * s,
                color::MUTED,
                FontWeight::Regular,
                TextAlign::Start,
            );
        }
        let buttons = y + height - 40.0 - 46.0;
        kit::button(
            &mut self.ui,
            frame,
            [
                x + width - 40.0 - 180.0 - 14.0 - 150.0,
                buttons,
                150.0,
                46.0,
            ],
            "Cancel",
            false,
            true,
            focus == AskAll::Cancel,
            ASK_ALL_CANCEL_TOKEN,
        );
        kit::button(
            &mut self.ui,
            frame,
            [x + width - 40.0 - 180.0, buttons, 180.0, 46.0],
            "Open all",
            true,
            true,
            focus == AskAll::Open,
            ASK_ALL_OPEN_TOKEN,
        );
        let enter = match focus {
            AskAll::Open => "open all",
            AskAll::Cancel => "cancel",
        };
        let keys: [(&[&str], &str); 3] = [
            (&["Tab"], "switch"),
            (&["Enter"], enter),
            (&["Esc"], "cancel"),
        ];
        let gap = 30.0 * s;
        let width: f32 = keys
            .iter()
            .map(|(caps, action)| key_hint_width(caps, action, s))
            .sum::<f32>()
            + gap * (keys.len() - 1) as f32;
        let [right, y] = frame.point(1_824.0, super::view::KEYS_Y);
        let mut x = right - width;
        for (caps, action) in keys {
            x = key_hint(&mut self.ui, caps, action, x, y, s) + gap;
        }
    }

    /// The token the question's focus is on, for the canvas.
    pub(super) fn ask_all_token(&self) -> u16 {
        match self.ask_all {
            Some(AskAll::Cancel) => ASK_ALL_CANCEL_TOKEN,
            _ => ASK_ALL_OPEN_TOKEN,
        }
    }
}
