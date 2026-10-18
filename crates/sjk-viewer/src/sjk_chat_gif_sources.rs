//! SJK UI for this player's exact-host media trust list. Selecting a host only fills
//! the shared text-entry field; Trust is a separate explicit action.
use super::*;
use crate::{
    chat_gifs::sources,
    menu::sjk::{Frame, color, kit, text},
    menu_widgets::TextFamily,
};
use sjk_ui::{FontWeight, TextAlign};
pub(super) const OPEN: u16 = 1550;
pub(super) const FIELD: u16 = 1551;
pub(super) const TRUST: u16 = 1552;
pub(super) const REMOVE: u16 = 1553;
const PREV: u16 = 1554;
const NEXT: u16 = 1555;
const HOST: u16 = 1560;
const BLOCKED: u16 = 1580;
const ROWS: usize = 6;
#[derive(Default)]
pub(super) struct Editor {
    pub(super) typed: String,
    notice: Option<&'static str>,
    page: usize,
    made: Option<(u64, u64, usize)>,
    shown: Vec<String>,
    blocked: Vec<String>,
}
impl Editor {
    pub(super) fn activate(&mut self, token: u16) -> bool {
        match token {
            FIELD => {}
            TRUST | REMOVE => {
                self.notice = Some(match sources::set(&self.typed, token == TRUST) {
                    Ok(()) => {
                        if token == TRUST {
                            "Source trusted. Its media can now load automatically."
                        } else {
                            "Source removed. Cached media and pending work were cleared."
                        }
                    }
                    Err(error) => error,
                });
            }
            PREV => self.page = self.page.saturating_sub(1),
            NEXT => self.page = (self.page + 1).min(sources::MAX.div_ceil(ROWS) - 1),
            HOST..=1565 => {
                if let Some(host) = self.shown.get((token - HOST) as usize) {
                    self.typed.clone_from(host);
                    self.notice = None;
                }
            }
            BLOCKED..=1585 => {
                if let Some(host) = self.blocked.get((token - BLOCKED) as usize) {
                    self.typed.clone_from(host);
                    self.notice = None;
                }
            }
            _ => return false,
        }
        true
    }
    pub(super) fn edit(&mut self, key: KeyCode, text: Option<&str>, shift: bool, control: bool) {
        if crate::text_entry::edit(&mut self.typed, key, text, shift, control, 2048, true) {
            self.notice = None;
        }
    }
}
impl Panel {
    pub(super) fn open_sources(&mut self, candidate: Option<String>) {
        self.gifs.open = true;
        self.gifs.key_card = None;
        self.gifs.choose_tab(gif_picker::Tab::Sources);
        if let Some(candidate) = candidate {
            self.gifs.sources.typed = candidate;
        }
        self.focus = FIELD;
    }
    pub(super) fn gif_sources(&mut self, frame: &Frame) {
        use super::view::{LIST_WIDTH, LIST_X};
        let x = LIST_X;
        let width = LIST_WIDTH;
        let mut y = 284.0;
        for line in [
            "Trust only sites whose media you want to see automatically in chat.",
            "Each exact host needs approval, including separate media CDNs. No wildcard domains.",
            "Trusting a site contacts it directly and may show any content it hosts.",
            "MP4 video clips also need Play chat video clips (Windows) in Settings, off by default.",
        ] {
            text(
                &mut self.ui,
                TextFamily::Body,
                format_args!("{line}"),
                frame.rect(x, y, width, 24.0),
                17.0 * frame.s,
                color::MUTED,
                FontWeight::Regular,
                TextAlign::Start,
            );
            y += 28.0;
        }
        let focused = self.focus == FIELD;
        kit::field(
            &mut self.ui,
            frame,
            [x, y, width - 380.0, 46.0],
            format_args!(
                "{}{}",
                self.gifs.sources.typed,
                if focused && (self.epoch.elapsed().as_millis() / 500).is_multiple_of(2) {
                    "|"
                } else {
                    ""
                }
            ),
            focused,
            false,
        );
        self.ui
            .hit_region(FIELD, frame.rect(x, y, width - 380.0, 46.0));
        self.order.push(FIELD);
        for (dx, label, token) in [
            (width - 360.0, "Trust source", TRUST),
            (width - 180.0, "Remove", REMOVE),
        ] {
            kit::button(
                &mut self.ui,
                frame,
                [x + dx, y, 170.0, 46.0],
                label,
                false,
                !self.gifs.sources.typed.trim().is_empty(),
                self.focus == token,
                token,
            );
            self.order.push(token);
        }
        y += 62.0;
        text(
            &mut self.ui,
            TextFamily::Body,
            format_args!(
                "{}",
                self.gifs.sources.notice.unwrap_or(
                    "Paste an HTTPS link or enter a hostname, then choose Trust source."
                )
            ),
            frame.rect(x, y, width, 38.0),
            16.0 * frame.s,
            color::GOLD_BRIGHT,
            FontWeight::Regular,
            TextAlign::Start,
        );
        y += 48.0;
        let mark = (
            sources::revision(),
            sources::blocked_revision(),
            self.gifs.sources.page,
        );
        if self.gifs.sources.made != Some(mark) {
            self.gifs.sources.made = Some(mark);
            self.gifs.sources.shown = sources::list()
                .into_iter()
                .skip(self.gifs.sources.page * ROWS)
                .take(ROWS)
                .collect();
            self.gifs.sources.blocked = sources::blocked_list()
                .into_iter()
                .skip(self.gifs.sources.page * ROWS)
                .take(ROWS)
                .collect();
        }
        let col = (width - 32.0) * 0.5;
        for (column, title, base, hosts) in [
            (0.0, "Trusted on this PC", HOST, &self.gifs.sources.shown),
            (
                col + 32.0,
                "Blocked media hosts discovered from pages",
                BLOCKED,
                &self.gifs.sources.blocked,
            ),
        ] {
            kit::heading(&mut self.ui, frame, x + column, y, col, title);
            for (row, host) in hosts.iter().enumerate() {
                let token = base + row as u16;
                kit::button(
                    &mut self.ui,
                    frame,
                    [x + column, y + 32.0 + row as f32 * 38.0, col, 34.0],
                    host,
                    false,
                    true,
                    self.focus == token,
                    token,
                );
                self.order.push(token);
            }
        }
        y += 276.0;
        for (dx, label, token) in [(0.0, "Previous", PREV), (180.0, "Next", NEXT)] {
            kit::button(
                &mut self.ui,
                frame,
                [x + dx, y, 160.0, 38.0],
                label,
                false,
                true,
                self.focus == token,
                token,
            );
            self.order.push(token);
        }
        text(
            &mut self.ui,
            TextFamily::Body,
            format_args!(
                "Page {}. Select a host above to edit it; Trust and Remove apply only after you press them.",
                self.gifs.sources.page + 1
            ),
            frame.rect(x + 360.0, y, width - 360.0, 44.0),
            15.0 * frame.s,
            color::QUIET,
            FontWeight::Regular,
            TextAlign::Start,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selecting_a_host_only_fills_the_field_and_typing_uses_shared_selection() {
        let before = sources::revision();
        let mut editor = Editor {
            shown: vec!["media.example.org".to_owned()],
            notice: Some("Previous source was trusted"),
            ..Editor::default()
        };
        assert!(editor.activate(HOST));
        assert_eq!(editor.typed, "media.example.org");
        assert!(editor.notice.is_none());
        assert_eq!(sources::revision(), before);
        editor.edit(KeyCode::KeyA, Some("a"), false, true);
        editor.edit(KeyCode::KeyX, Some("example.org"), false, false);
        assert_eq!(editor.typed, "example.org");
        editor.edit(KeyCode::Backspace, None, false, false);
        editor.edit(KeyCode::Backspace, None, false, false);
        assert_eq!(editor.typed, "example.o");
    }
    #[test]
    fn chat_source_action_opens_confirmation_without_trusting() {
        let before = sources::revision();
        let mut panel = Panel::new();
        panel.shown.chosen_source =
            crate::chat_gifs::GifId::from_url("https://media.example.org/clip.mp4");
        assert_eq!(panel.activate(OPEN), PanelAction::None);
        assert_eq!(panel.gifs.tab, gif_picker::Tab::Sources);
        assert_eq!(panel.focus, FIELD);
        assert_eq!(panel.gifs.sources.typed, "media.example.org");
        assert_eq!(sources::revision(), before);
    }
}
