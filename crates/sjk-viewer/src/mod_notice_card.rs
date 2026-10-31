//! A client mod's notice (`docs/mods.md`): a card a mod shows once on its servers, to
//! tell the player something the mod changes there. JoF tools use it to say that SJK
//! chat is said in All chat on JoF servers (`sjk-mod-jof`, `sjk_chat.rs`). The mod gives
//! the words and an archived on/off setting; the card shows once the player is in a
//! match on such a server, over the game menu, in the SJK UI's look in every menu style,
//! and Got it (Enter or Escape) turns the setting off for good. The core has no
//! words of its own here: without a mod asking, the card never shows.

use crate::menu::sjk::{Frame, TextTarget, color, key_hint, key_hint_width, kit, text, wrap};
use crate::menu_widgets::{MenuCanvas, TextFamily};
use sjk_ui::{FontWeight, InputEvent, TextAlign, UiEventKind};
use std::time::{Duration, Instant};
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

/// What a mod's notice says, and the setting that is on until it was shown.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Notice {
    pub(crate) cvar: &'static str,
    pub(crate) kicker: &'static str,
    pub(crate) title: &'static str,
    pub(crate) paragraphs: &'static [&'static str],
}

const CLOSE_TOKEN: u16 = 1;

/// The card's left edge and width; it is as tall as what it holds, centred.
const CARD_X: f32 = 560.0;
const CARD_WIDTH: f32 = 800.0;
const SCREEN_HEIGHT: f32 = 1080.0;
const MARGIN: f32 = 44.0;
const TEXT_X: f32 = CARD_X + MARGIN;
const TEXT_WIDTH: f32 = CARD_WIDTH - MARGIN * 2.0;
/// Below the kicker and headline.
const BODY_OFFSET: f32 = 130.0;
const LINE: f32 = 28.0;
const LINE_CHARS: usize = 90;
const PARAGRAPH_GAP: f32 = 12.0;
const FOOT_GAP: f32 = 26.0;
const BUTTON_HEIGHT: f32 = 46.0;
const CLOSE_WIDTH: f32 = 150.0;
const KEYS_Y: f32 = 992.0;
const KEYS_RIGHT: f32 = 1824.0;
/// How long the card takes no input after it opens. It opens by itself in the middle of
/// a match, so a key or click meant for the game at that moment (a jump) must not close
/// it for good, as on the low-FPS card (hub report #194).
const SETTLE: Duration = Duration::from_millis(600);

/// What the player chose on the card.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Choice {
    /// Got it: the notice's setting goes off.
    Close(&'static str),
}

#[derive(Default)]
pub(crate) struct State {
    pub(crate) card: Card,
    /// The card opened the game menu, which goes with it.
    opened_menu: bool,
}

pub(crate) struct Card {
    notice: Option<Notice>,
    /// Shown this frame.
    open: bool,
    /// When the card last opened, for [`SETTLE`].
    opened_at: Option<Instant>,
    ui: MenuCanvas,
}

impl Default for Card {
    fn default() -> Self {
        Self {
            notice: None,
            open: false,
            opened_at: None,
            ui: MenuCanvas::with_capacities(32, 1024, 64),
        }
    }
}

/// The body's lines, each with its top from the body's, and its height.
fn layout(notice: &Notice) -> (Vec<(f32, &'static str)>, f32) {
    let mut lines = Vec::new();
    let mut y = 0.0;
    for paragraph in notice.paragraphs {
        for line in wrap(paragraph, LINE_CHARS) {
            lines.push((y, line));
            y += LINE;
        }
        y += PARAGRAPH_GAP;
    }
    (lines, (y - PARAGRAPH_GAP).max(0.0))
}

impl Card {
    pub(crate) fn show(&mut self, notice: Notice) {
        self.notice = Some(notice);
    }

    /// Whether the card has something to show.
    pub(crate) fn wants(&self) -> bool {
        self.notice.is_some()
    }

    pub(crate) fn set_open(&mut self, open: bool) {
        if open && !self.open {
            self.opened_at = Some(Instant::now());
        }
        self.open = open;
    }

    /// Whether the card has been open long enough to take input.
    fn settled(&self, now: Instant) -> bool {
        self.opened_at
            .is_none_or(|opened| now.saturating_duration_since(opened) >= SETTLE)
    }

    pub(crate) fn is_open(&self) -> bool {
        self.open
    }

    /// Enter or Escape close once the card has settled; every other key, Space among
    /// them, is swallowed while the card shows.
    pub(crate) fn handle_key(&mut self, event: &KeyEvent) -> Option<Choice> {
        if event.state != ElementState::Pressed || event.repeat || !self.settled(Instant::now()) {
            return None;
        }
        let PhysicalKey::Code(code) = event.physical_key else {
            return None;
        };
        self.key(code)
    }

    fn key(&mut self, code: KeyCode) -> Option<Choice> {
        match code {
            KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Escape => self.close(),
            _ => None,
        }
    }

    pub(crate) fn handle_pointer(&mut self, event: InputEvent) -> Option<Choice> {
        let event = self.ui.pointer(event)?;
        if event.kind == UiEventKind::Activate
            && event.token == Some(CLOSE_TOKEN)
            && self.settled(Instant::now())
        {
            self.close()
        } else {
            None
        }
    }

    fn close(&mut self) -> Option<Choice> {
        self.notice.take().map(|notice| Choice::Close(notice.cvar))
    }

    pub(crate) fn draw_list(&self) -> &sjk_ui::DrawList {
        self.ui.draw_list()
    }

    /// Draw the card over the whole frame.
    pub(crate) fn append(&mut self, target: TextTarget<'_>, viewport: [f32; 2]) {
        let Some(notice) = self.notice else {
            return;
        };
        let frame = Frame::new(viewport);
        let s = frame.s;
        let (lines, body) = layout(&notice);
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
            format_args!("{}", notice.kicker),
            frame.rect(TEXT_X, top + 30.0, TEXT_WIDTH, 30.0),
            22.0 * s,
            color::GOLD_BRIGHT,
            FontWeight::Semibold,
            TextAlign::Start,
        );
        text(
            ui,
            TextFamily::Display,
            format_args!("{}", notice.title),
            frame.rect(TEXT_X, top + 66.0, TEXT_WIDTH, 48.0),
            36.0 * s,
            color::TEXT,
            FontWeight::Semibold,
            TextAlign::Start,
        );
        for (y, line) in &lines {
            text(
                ui,
                TextFamily::Body,
                format_args!("{line}"),
                frame.rect(TEXT_X, body_top + y, TEXT_WIDTH, LINE),
                18.0 * s,
                color::MUTED,
                FontWeight::Regular,
                TextAlign::Start,
            );
        }
        let right = CARD_X + CARD_WIDTH - MARGIN;
        kit::button(
            ui,
            &frame,
            [right - CLOSE_WIDTH, foot, CLOSE_WIDTH, BUTTON_HEIGHT],
            "Got it",
            true,
            true,
            false,
            CLOSE_TOKEN,
        );
        let [right, keys_y] = frame.point(KEYS_RIGHT, KEYS_Y);
        let keys: &[&str] = &["Enter"];
        let x = right - key_hint_width(keys, "got it", s);
        key_hint(ui, keys, "got it", x, keys_y, s);
        ui.finish(CLOSE_TOKEN);
        target.append(&self.ui, viewport);
    }
}

impl crate::GpuState {
    /// The notice a loaded mod wants shown on this server now, if any: JoF tools' on
    /// SJK chat, while they say it in All chat here, the SJK chat is on and the player
    /// has not seen it yet.
    fn wanted_mod_notice(&self) -> Option<Notice> {
        #[cfg(feature = "mod-jof")]
        {
            use sjk_mod_jof::sjk_chat;
            let console = self.console.as_ref()?;
            (self.sjk_chat_said_in_all()
                && console.bool_cvar("cl_sjkChat") != Some(false)
                && console.bool_cvar(sjk_chat::NOTICE) != Some(false))
            .then_some(Notice {
                cvar: sjk_chat::NOTICE,
                kicker: sjk_chat::NOTICE_KICKER,
                title: sjk_chat::NOTICE_TITLE,
                paragraphs: &sjk_chat::NOTICE_PARAGRAPHS,
            })
        }
        #[cfg(not(feature = "mod-jof"))]
        {
            None
        }
    }

    /// Whether the card shows this frame. A mod's notice is taken once the player is in
    /// a match with the world loaded and no menu up, and opens the game menu under it.
    /// `covered` is another card or the console.
    pub(crate) fn prepare_mod_notice_card(&mut self, covered: bool) -> bool {
        let menu_visible = self
            .client_menu
            .as_ref()
            .is_some_and(crate::menu::ClientMenu::is_visible);
        let console_open = self
            .console
            .as_ref()
            .is_some_and(crate::console::ViewerConsole::is_open);
        let settled = !covered
            && !console_open
            && !self.text_dialog.is_open()
            && self.world_load_task.is_none()
            && self.world_install_task.is_none()
            && !self.pending_map_reload;
        if settled
            && !self.mod_notice.card.wants()
            && !menu_visible
            && let Some(notice) = self.wanted_mod_notice()
        {
            if !self.game_menu {
                self.release_pointer();
                self.game_menu = true;
                self.game_menu_page = crate::ingame_menu::Page::Main;
                self.game_menu_row = 0;
                self.mod_notice.opened_menu = true;
                self.sync_cursor_policy();
            }
            self.mod_notice.card.show(notice);
        }
        let open = self.mod_notice.card.wants() && (menu_visible || self.game_menu) && settled;
        self.mod_notice.card.set_open(open);
        open
    }

    /// Draw the card. It darkens everything, so every font batch appended before it is
    /// dropped (text draws above all shapes).
    pub(crate) fn append_mod_notice_card(&mut self, viewport: [f32; 2]) {
        self.text_vertices.clear();
        self.classic_text_vertices.clear();
        self.game_fonts.clear_text();
        let target = crate::ingame_menu::sjk_view::text_target(
            &mut self.game_fonts,
            &mut self.text_vertices,
            &self.ui_font,
        );
        self.mod_notice.card.append(target, viewport);
    }

    /// The notice was read: its setting goes off, and play comes back when the card
    /// opened the game menu for itself.
    pub(crate) fn mod_notice_choice(&mut self, choice: Option<Choice>) {
        let Some(Choice::Close(cvar)) = choice else {
            return;
        };
        if let Some(console) = &mut self.console {
            console.set_cvar(cvar, "0");
        }
        crate::log::progress(format_args!("mod notice {cvar}: read"));
        if std::mem::take(&mut self.mod_notice.opened_menu) {
            self.game_menu = false;
            self.sync_cursor_policy();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOTICE: Notice = Notice {
        cvar: "test.notice",
        kicker: "TEST",
        title: "A notice",
        paragraphs: &["One paragraph.", "And another one."],
    };

    #[test]
    fn the_close_keys_end_the_card_with_its_setting_and_others_are_swallowed() {
        let mut card = Card::default();
        assert!(!card.wants());
        assert_eq!(card.key(KeyCode::Enter), None, "nothing to close");
        card.show(NOTICE);
        assert_eq!(card.key(KeyCode::KeyA), None);
        assert_eq!(card.key(KeyCode::Space), None, "a jump does not close it");
        assert!(card.wants());
        assert_eq!(
            card.key(KeyCode::Escape),
            Some(Choice::Close("test.notice"))
        );
        assert!(!card.wants());
    }

    #[test]
    fn the_card_takes_no_input_until_it_has_settled() {
        let mut card = Card::default();
        card.show(NOTICE);
        card.set_open(true);
        let opened = card.opened_at.expect("opened");
        assert!(!card.settled(opened + SETTLE / 2));
        assert!(card.settled(opened + SETTLE));
    }

    #[cfg(feature = "mod-jof")]
    #[test]
    fn the_jof_notice_fits_the_screen() {
        let notice = Notice {
            cvar: sjk_mod_jof::sjk_chat::NOTICE,
            kicker: sjk_mod_jof::sjk_chat::NOTICE_KICKER,
            title: sjk_mod_jof::sjk_chat::NOTICE_TITLE,
            paragraphs: &sjk_mod_jof::sjk_chat::NOTICE_PARAGRAPHS,
        };
        let (_, body) = layout(&notice);
        let height = BODY_OFFSET + body + FOOT_GAP + BUTTON_HEIGHT + MARGIN;
        assert!((SCREEN_HEIGHT + height) * 0.5 < KEYS_Y - 20.0, "{height}");
        // The headline in one line at 36 px.
        assert!(notice.title.chars().count() as f32 * 36.0 * 0.55 < TEXT_WIDTH);
    }

    #[cfg(feature = "mod-jof")]
    #[test]
    fn the_jof_notice_setting_exists_and_starts_on() {
        let directory = tempfile::tempdir().unwrap();
        let console =
            crate::console::ViewerConsole::new(directory.path().join("config.cfg")).unwrap();
        assert_eq!(console.bool_cvar(sjk_mod_jof::sjk_chat::NOTICE), Some(true));
    }
}
