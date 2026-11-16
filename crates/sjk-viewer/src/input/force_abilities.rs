//! Host bridge for optional mod-owned Force wheel entries.
use sjk_client::force_wheel::Ability;

/// Every wheel entry a built-in mod may supply, whether or not it is active now.
fn known_entries() -> &'static [Ability] {
    #[cfg(feature = "mod-jof")]
    return sjk_mod_jof::destruction::ENTRIES;
    #[cfg(not(feature = "mod-jof"))]
    &[]
}

/// The wheel slot whose command `command` is (lower case), active or not.
pub(super) fn command_slot(command: &str) -> Option<u8> {
    known_entries()
        .iter()
        .find(|entry| entry.command == Some(command))
        .map(|entry| entry.slot)
}

/// The command a mod's entry sends. While that entry is not active (mod off or
/// unloaded, another server, the support setting off), a typed or bound command
/// goes to the server unchanged, as any other unknown command would.
pub(super) fn command_for(slot: u8) -> Option<&'static str> {
    entry(slot).and_then(|entry| entry.command)
}

/// A built-in mod's wheel entry for `slot`, active or not.
pub(crate) fn entry(slot: u8) -> Option<&'static Ability> {
    known_entries().iter().find(|entry| entry.slot == slot)
}

pub(crate) fn extensions(
    game: Option<&sjk_protocol::GameState>,
    console: Option<&crate::console::ViewerConsole>,
) -> &'static [Ability] {
    #[cfg(feature = "mod-jof")]
    if let Some(console) = console
        && console.mods.loaded(sjk_mod_jof::ID)
        && console.bool_cvar("mod_jof").unwrap_or(false)
    {
        return sjk_mod_jof::destruction::entries(
            game,
            console.integer_cvar("jof.destructionsupport").unwrap_or(-1),
        );
    }
    let _ = (game, console);
    &[]
}

impl crate::GpuState {
    pub(crate) fn refresh_force_support(&mut self) {
        let game = self
            .live_session
            .as_ref()
            .map(|s| s.game_state())
            .or_else(|| self.demo_session.as_ref().map(|s| s.game_state()));
        let entries = extensions(game, self.console.as_ref());
        self.gameplay_input.selection.set_extensions(entries);
        if !entries.is_empty()
            && let Some(vfs) = &self.vfs
        {
            self.hud.load_extension_icons(
                entries,
                vfs,
                &self.shaders,
                &self.ui_shapes,
                &self.queue,
            );
        }
    }
}
#[cfg(all(test, feature = "mod-jof"))]
mod tests {
    use super::*;
    use crate::input::{GameplayInput, InputAction};
    #[test]
    fn loaded_jof_switch_and_server_are_required_even_for_override() {
        let dir = tempfile::tempdir().unwrap();
        let mut console =
            crate::console::ViewerConsole::new(dir.path().join("config.cfg")).unwrap();
        console.set_cvar(sjk_mod_jof::destruction::CVAR, "1");
        let game = sjk_protocol::GameState::empty_local(0);
        assert!(extensions(Some(&game), Some(&console)).is_empty());
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
            console.execute_buffered_frame(None, |_, _| false);
            assert_eq!(
                !extensions(Some(&game), Some(&console)).is_empty(),
                expected
            );
        }
        console.set_cvar("mod_jof", "0");
        assert!(extensions(Some(&game), Some(&console)).is_empty());
        let mut input = GameplayInput::default();
        assert!(GameplayInput::recognizes(sjk_mod_jof::destruction::COMMAND));
        // Inactive (mod off here): still the server command, forwarded unchanged.
        assert_eq!(
            command_for(sjk_mod_jof::destruction::SLOT),
            Some(sjk_mod_jof::destruction::COMMAND)
        );
        assert_eq!(command_for(sjk_client::force_wheel::DASH), None);
        assert_eq!(
            input.apply(sjk_mod_jof::destruction::COMMAND),
            Some(InputAction::ForceAbility(sjk_mod_jof::destruction::SLOT))
        );
    }
}
