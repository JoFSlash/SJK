//! Neutral optional-modal host hooks; JoF state/UI is compiled with its mod only.
use super::ViewerConsole;
impl ViewerConsole {
    pub(crate) fn server_dialogue_active(&self) -> bool {
        #[cfg(feature = "mod-jof")]
        {
            self.open
                && self.dialogue.is_open()
                && self.mods.loaded(sjk_mod_jof::ID)
                && self.bool_cvar("mod_jof").unwrap_or(false)
        }
        #[cfg(not(feature = "mod-jof"))]
        {
            false
        }
    }
}
#[cfg(not(feature = "mod-jof"))]
impl ViewerConsole {
    pub(crate) fn reset_dialogue(&mut self) {}
    pub(super) fn reset_unloaded_dialogue(&mut self) {}
    pub(super) fn dialogue_closed(&mut self) {}
    pub(super) fn dialogue_key(&mut self, _: &winit::event::KeyEvent) -> bool {
        false
    }
    pub(super) fn dialogue_pointer(&mut self, _: sjk_ui::InputEvent) -> bool {
        false
    }
    pub(super) fn dialogue_draw_list(&self) -> Option<&sjk_ui::DrawList> {
        None
    }
    pub(super) fn append_dialogue(&mut self, _: crate::menu::sjk::TextTarget<'_>, _: [f32; 2]) {}
    pub(super) fn flush_dialogue(&mut self, _: &mut sjk_client::ClientSession) {}
}
