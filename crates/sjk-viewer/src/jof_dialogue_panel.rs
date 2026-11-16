//! Server dialogue modal in SJK's UI in every menu style. Server owns quests.
use super::ViewerConsole;
use crate::menu_widgets::MenuCanvas;
use sjk_mod_jof::dialogue::{Dialogue, Event};
use sjk_ui::{InputEvent, UiEventKind};
use std::time::Instant;
use winit::event::KeyEvent;
use winit::keyboard::{KeyCode, PhysicalKey};
#[path = "dialogue_view.rs"]
mod view;

pub(crate) const CVAR: &str = sjk_mod_jof::dialogue::CVAR;

pub(super) struct Panel {
    node: Dialogue,
    canvas: MenuCanvas,
    pending: Option<String>,
    owns_console: bool,
    epoch: Instant,
}
impl Panel {
    pub(super) fn new() -> Self {
        Self {
            node: Dialogue::default(),
            canvas: MenuCanvas::with_text_capacity(48),
            pending: None,
            owns_console: false,
            epoch: Instant::now(),
        }
    }
    pub(super) fn is_open(&self) -> bool {
        self.node.active
    }
    fn respond(&mut self, choice: i32) {
        if let Some(command) = self
            .node
            .respond(choice, self.epoch.elapsed().as_millis() as u64)
        {
            self.pending = Some(command);
        }
    }
}
impl ViewerConsole {
    /// Accept only already-decoded bounded dialogue events from the current session.
    pub(crate) fn dialogue_event(&mut self, event: Event) {
        if !self.mods.loaded(sjk_mod_jof::ID) || !self.bool_cvar("mod_jof").unwrap_or(false) {
            return;
        }
        let begin = matches!(event, Event::Begin { .. });
        let stop = matches!(event, Event::Stop { .. });
        if !self.dialogue.node.apply(event) {
            return;
        }
        if stop {
            self.dialogue.pending = None;
        }
        if begin {
            self.dialogue.pending = None;
            // A release over a replacement node must not activate an old press.
            self.dialogue.canvas = MenuCanvas::with_text_capacity(48);
        }
        if self.dialogue.is_open() {
            if !self.bool_cvar(CVAR).unwrap_or(true) {
                self.dialogue.respond(-1);
                self.dialogue_finished_locally();
                return;
            }
            if !self.open {
                self.dialogue.owns_console = true;
                self.set_open(true);
            }
        } else if stop && self.dialogue.owns_console {
            self.dialogue.owns_console = false;
            self.set_open(false);
        }
    }
    /// A new connection/map or disconnect discards old serials and unsent responses.
    pub(super) fn reset_unloaded_dialogue(&mut self) {
        if !self.mods.loaded(sjk_mod_jof::ID) || !self.bool_cvar("mod_jof").unwrap_or(false) {
            self.reset_dialogue();
        }
    }
    pub(crate) fn reset_dialogue(&mut self) {
        if self.dialogue.node.serial.is_none()
            && self.dialogue.pending.is_none()
            && !self.dialogue.owns_console
        {
            return;
        }
        self.dialogue.node.reset();
        self.dialogue.pending = None;
        if std::mem::take(&mut self.dialogue.owns_console) {
            self.set_open(false);
        }
    }
    /// Closing the console while a dialogue owns input cancels its current serial.
    pub(super) fn dialogue_closed(&mut self) {
        if self.dialogue.is_open() {
            self.dialogue.respond(-1);
        }
        self.dialogue.owns_console = false;
    }
    fn dialogue_finished_locally(&mut self) {
        if !self.dialogue.is_open() && std::mem::take(&mut self.dialogue.owns_console) {
            self.set_open(false);
        }
    }
    pub(super) fn flush_dialogue(&mut self, session: &mut sjk_client::ClientSession) {
        if !self.mods.loaded(sjk_mod_jof::ID) || !self.bool_cvar("mod_jof").unwrap_or(false) {
            self.reset_dialogue();
            return;
        }
        if !self.bool_cvar(CVAR).unwrap_or(true) && self.dialogue.is_open() {
            self.dialogue.respond(-1);
            self.dialogue_finished_locally();
        }
        if let Some(command) = self.dialogue.pending.take()
            && let Err(error) = session.send_reliable_command_from(
                command.as_bytes(),
                sjk_client::ReliableCommandOrigin::Input,
            )
        {
            self.push_log(format!("^3Dialogue response failed: {error}"));
        }
    }
    pub(super) fn dialogue_key(&mut self, event: &KeyEvent) -> bool {
        if !self.dialogue.is_open() {
            return false;
        }
        let PhysicalKey::Code(key) = event.physical_key else {
            return true;
        };
        match key {
            KeyCode::Escape => self.dialogue.respond(-1),
            KeyCode::ArrowUp | KeyCode::Numpad8 => self.dialogue.node.navigate(-1),
            KeyCode::ArrowDown | KeyCode::Numpad2 => self.dialogue.node.navigate(1),
            KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space if !event.repeat => {
                self.dialogue.respond(self.dialogue.node.selected as i32)
            }
            KeyCode::Digit1 => self.dialogue.respond(0),
            KeyCode::Digit2 => self.dialogue.respond(1),
            KeyCode::Digit3 => self.dialogue.respond(2),
            KeyCode::Digit4 => self.dialogue.respond(3),
            KeyCode::Digit5 => self.dialogue.respond(4),
            KeyCode::Digit6 => self.dialogue.respond(5),
            KeyCode::Digit7 => self.dialogue.respond(6),
            KeyCode::Digit8 => self.dialogue.respond(7),
            KeyCode::Digit9 => self.dialogue.respond(8),
            KeyCode::Digit0 => self.dialogue.respond(9),
            _ => {}
        }
        self.dialogue_finished_locally();
        true
    }
    pub(super) fn dialogue_pointer(&mut self, event: InputEvent) -> bool {
        if !self.dialogue.is_open() {
            return false;
        }
        if let InputEvent::PointerWheel { delta, .. } = event {
            if delta.y != 0.0 {
                self.dialogue
                    .node
                    .navigate(if delta.y > 0.0 { -1 } else { 1 });
            }
            return true;
        }
        if let Some(event) = self.dialogue.canvas.pointer(event)
            && let Some(token) = event.token
        {
            if token == crate::menu_widgets::BACK_TOKEN && event.kind == UiEventKind::Activate {
                self.dialogue.respond(-1);
            } else if let Some(index) = token
                .checked_sub(1)
                .map(usize::from)
                .filter(|&i| i < self.dialogue.node.count)
            {
                match event.kind {
                    UiEventKind::HoverEnter | UiEventKind::Focus => {
                        self.dialogue.node.selected = index
                    }
                    UiEventKind::Activate => self.dialogue.respond(index as i32),
                    _ => {}
                }
            }
        }
        self.dialogue_finished_locally();
        true
    }
    pub(super) fn append_dialogue(
        &mut self,
        target: crate::menu::sjk::TextTarget<'_>,
        viewport: [f32; 2],
    ) {
        self.dialogue.build(viewport);
        target.append(&self.dialogue.canvas, viewport);
    }
    pub(super) fn dialogue_draw_list(&self) -> Option<&sjk_ui::DrawList> {
        (self.server_dialogue_active()).then(|| self.dialogue.canvas.draw_list())
    }
    /// Audio observes a revision, not a repeated per-frame play request.
    pub(crate) fn dialogue_voice(&self) -> (u64, Option<&str>) {
        (
            self.dialogue.node.revision,
            (self.server_dialogue_active())
                .then_some(self.dialogue.node.sound.as_deref())
                .flatten(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn event(text: &str) -> Event {
        // Parser state is covered in sjk-mod-jof; these are UI integration operands.
        match text {
            "begin" => Event::Begin {
                serial: 12,
                speaker: "Guide".into(),
                text: "First\nSecond".into(),
            },
            "show" => Event::Show {
                serial: 12,
                sound: Some("sound/guide.wav".into()),
            },
            _ => unreachable!(),
        }
    }
    #[test]
    fn unrecognized_servers_and_mod_off_never_accept_dialogue_and_unload_clears_it() {
        let dir = tempfile::tempdir().unwrap();
        let mut console = ViewerConsole::new(dir.path().join("config.cfg")).unwrap();
        for (kind, info, expected) in [
            (sjk_mod::ServerKind::BaseJka, "", false),
            (sjk_mod::ServerKind::JaPlus, r"\V\2.4B7", false),
            (sjk_mod::ServerKind::JaPro, r"\V\2.5B0", false),
            (sjk_mod::ServerKind::JaPlus, r"\V\2.5B0", true),
        ] {
            console.mods.set_server(Some(sjk_mod::Server {
                kind,
                address: "fixture".into(),
                info: info.into(),
            }));
            console.mods.sync(&mut console.shell);
            console.dialogue_event(event("begin"));
            assert_eq!(console.dialogue.node.serial.is_some(), expected);
        }
        console.dialogue_event(Event::Choice {
            serial: 12,
            index: 0,
            text: "Continue".into(),
        });
        console.dialogue_event(event("show"));
        assert!(console.server_dialogue_active());
        console.dialogue.respond(0);
        console.set_cvar("mod_jof", "0");
        assert!(!console.server_dialogue_active());
        assert!(console.dialogue_voice().1.is_none());
        console.execute_buffered_frame(None, |_, _| false);
        assert!(console.dialogue.node.serial.is_none() && console.dialogue.pending.is_none());
        let revision = console.dialogue.node.revision;
        console.reset_dialogue();
        assert_eq!(console.dialogue.node.revision, revision);
        console.dialogue_event(event("begin"));
        assert!(console.dialogue.node.serial.is_none());
    }

    #[test]
    fn all_twelve_choices_fit_five_visible_rows_and_scroll_without_overflow() {
        let mut panel = Panel::new();
        panel.node.apply(event("begin"));
        for index in 0..12 {
            panel.node.apply(Event::Choice {
                serial: 12,
                index,
                text: format!("Answer {index}"),
            });
        }
        panel.node.apply(event("show"));
        for viewport in [[640.0, 480.0], [1920.0, 1080.0], [3440.0, 1440.0]] {
            for selected in 0..12 {
                panel.node.selected = selected;
                panel.build(viewport);
                assert!(!panel.canvas.overflowed());
                assert_eq!(panel.canvas.widget_count(), 6);
                assert!(
                    panel
                        .canvas
                        .widget_tokens()
                        .contains(&(selected as u16 + 1))
                );
            }
        }
    }
    #[test]
    fn modal_owns_console_only_when_it_opened_it_and_closing_queues_cancel() {
        let dir = tempfile::tempdir().unwrap();
        let mut console = ViewerConsole::new(dir.path().join("config.cfg")).unwrap();
        console.mods.set_server(Some(sjk_mod::Server {
            kind: sjk_mod::ServerKind::JaPlus,
            address: "fixture".into(),
            info: r"\V\2.5B0".into(),
        }));
        console.mods.sync(&mut console.shell);
        console.dialogue_event(event("begin"));
        console.dialogue_event(Event::Choice {
            serial: 12,
            index: 0,
            text: "Continue".into(),
        });
        console.dialogue_event(event("show"));
        assert!(console.is_open() && console.sjk_page_open());
        assert!(console.dialogue_voice().1.is_some());
        console.set_open(false);
        assert_eq!(
            console.dialogue.pending.as_deref(),
            Some("dialogueresponse 12 -1")
        );
        assert!(console.dialogue_voice().1.is_none());
        console.reset_dialogue();
        assert!(console.dialogue.pending.is_none());
        console.set_open(true);
        console.dialogue_event(event("begin"));
        console.dialogue_event(Event::Choice {
            serial: 12,
            index: 0,
            text: "Continue".into(),
        });
        console.dialogue_event(event("show"));
        console.dialogue_event(Event::Stop { serial: 11 });
        assert!(console.dialogue.is_open());
        console.dialogue_event(Event::Stop { serial: 12 });
        assert!(!console.dialogue.is_open());
        assert!(console.is_open());
    }
}
