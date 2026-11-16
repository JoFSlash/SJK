//! The JA+ tools mod's anti-kiss bridge (`sjk_mod_japlus::anti_kiss`, SJK #190):
//! while the mod is loaded and `cg_antiAmKiss` is on, the mod says each frame
//! whether to answer a kiss, and the response it names goes in the next user command
//! as a bound generic command would. The client knows nothing of JA+'s `amkiss`;
//! a build without `mod-japlus` has neither the setting nor the bridge.

/// The interaction the mod is tracking.
#[derive(Default)]
pub(crate) struct State {
    #[cfg(feature = "mod-japlus")]
    latch: sjk_mod_japlus::anti_kiss::Latch,
}

#[cfg(feature = "mod-japlus")]
impl crate::GpuState {
    /// Queue the mod's response before the next user command is assembled.
    pub(crate) fn apply_anti_emote(&mut self) {
        use sjk_mod_japlus::anti_kiss;
        let enabled = self.console.as_ref().is_some_and(|console| {
            console.mods.loaded(sjk_mod_japlus::ID)
                && console.bool_cvar(anti_kiss::CVAR).unwrap_or(false)
        });
        let Some(session) = self.live_session.as_ref().filter(|session| {
            enabled && anti_kiss::applies_on(crate::mods::server_kind(session.compat_profile()))
        }) else {
            self.anti_emote.latch.clear();
            return;
        };
        let predicted = self
            .local_prediction
            .predicted_state()
            .map(|state| [state.legs_anim, state.torso_anim]);
        let kissed = anti_kiss::kiss_is_active(&session.latest_snapshot().player, predicted);
        // A command the player pressed this frame goes first; push on the next one.
        if kissed && self.pending_generic_command != 0 {
            return;
        }
        if self.anti_emote.latch.observe(kissed)
            && let Some(command) =
                crate::input::generic_commands::generic_command(anti_kiss::RESPONSE)
        {
            self.pending_generic_command = command;
        }
    }
}

#[cfg(not(feature = "mod-japlus"))]
impl crate::GpuState {
    /// Without JA+ tools there is no anti-kiss.
    pub(crate) fn apply_anti_emote(&mut self) {
        let State {} = self.anti_emote;
    }
}

#[cfg(all(test, feature = "mod-japlus"))]
mod tests {
    use sjk_mod_japlus::anti_kiss;

    /// The mod's wire numbers are the client's.
    #[test]
    fn the_mods_constants_match_the_game_tables() {
        assert_eq!(
            sjk_game_jka::legacy_animation_index("BOTH_KISSEE"),
            Some(usize::from(anti_kiss::KISS_VICTIM_ANIMATION))
        );
        assert_eq!(anti_kiss::PM_SPECTATOR, sjk_game_jka::PM_SPECTATOR);
        assert_eq!(anti_kiss::PMF_FOLLOW, sjk_game_jka::follow::PMF_FOLLOW);
        assert_eq!(
            crate::input::generic_commands::generic_command(anti_kiss::RESPONSE),
            Some(sjk_game_jka::generic_commands::GENCMD_FORCE_THROW)
        );
    }

    #[test]
    fn the_switch_is_archived_and_off_by_default_whatever_the_server() {
        let mut shell = sjk_shell::Shell::new(sjk_shell::CvarRegistry::new(), Default::default());
        crate::mods::Mods::new().register(&mut shell).unwrap();
        let cvar = shell.cvars.get(anti_kiss::CVAR).expect("registered");
        assert_eq!(cvar.value.as_text(), "0");
        assert!(cvar.flags.contains(sjk_shell::CvarFlags::ARCHIVE));
    }
}
