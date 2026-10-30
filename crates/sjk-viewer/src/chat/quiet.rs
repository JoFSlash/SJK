//! Quiet mode (Lex Deifaber's idea, 30/10/2026): until it is turned off or the map
//! changes, the chat feed hides public, team, SJK and admin chat and their sounds, but
//! for the player's private duel partner, whispers (server tells and encrypted SJK
//! whispers) and the player's own lines. What no player sent (the server's own chat
//! lines, the SJK team's announcements, refusals) and centre prints still show; the
//! console keeps every line, as it does under `pmonly`.
//!
//! Whether a line passes is decided when it arrives ([`super::ChatLine::loud`]), from
//! the duel partner of that moment, so a duel partner's lines stay after the duel and
//! turning quiet mode on hides the history it would have hidden; turning it off shows
//! that history again.
use crate::console::ViewerConsole;

/// `cg_quietMode`: quiet mode on. Not archived: every map starts with it off.
pub(crate) const CVAR: &str = "cg_quietMode";
/// The console command that toggles it (bindable as any command).
pub(crate) const COMMAND: &str = "quiet";

/// `quiet [on|off]`: toggle quiet mode, or set it; says ON or OFF.
pub(crate) fn command(console: &mut ViewerConsole, args: &[String]) -> Result<Vec<String>, String> {
    let value = match args {
        [] => !console.bool_cvar(CVAR).unwrap_or(false),
        [arg] if arg == "1" || arg.eq_ignore_ascii_case("on") => true,
        [arg] if arg == "0" || arg.eq_ignore_ascii_case("off") => false,
        _ => return Err("Usage: quiet [on|off]".into()),
    };
    console.set_cvar(CVAR, if value { "1" } else { "0" });
    Ok(vec![
        if value {
            "Quiet mode ON: only your duel partner and whispers until the map changes."
        } else {
            "Quiet mode OFF."
        }
        .to_owned(),
    ])
}

/// A new map: quiet mode ends. Graphics reloads keep it.
pub(crate) fn reset(console: &mut ViewerConsole) {
    console.set_cvar(CVAR, "0");
}

/// Whether a game chat line passes quiet mode: a whisper, a line no player is named as
/// sending (the server's own), the player's own or one from `partner`, the private duel
/// partner when it arrives.
pub(super) fn game_line_loud(
    private: bool,
    sender: Option<u16>,
    own: Option<u16>,
    partner: Option<u16>,
) -> bool {
    private || sender.is_none() || sender == own || (sender.is_some() && sender == partner)
}

/// Whether quiet mode silences a chat beep: a public or team line's (never a tell's)
/// whose sender the sound adapter named (`LegacySoundDecision::cause`) and who is
/// neither the player (`player`'s slot) nor their private duel partner. A beep naming
/// nobody plays, as its line shows.
pub(crate) fn silences_beep(
    quiet: bool,
    event: sjk_client::LegacySoundEvent,
    cause: Option<u16>,
    player: &sjk_protocol::PlayerState,
) -> bool {
    use sjk_client::LegacySoundEvent::{ChatBeep, TeamChatBeep};
    let partner = player.duel_in_progress().then(|| player.duel_index());
    quiet
        && matches!(event, ChatBeep | TeamChatBeep)
        && !game_line_loud(false, cause, Some(player.client_num()), partner)
}

impl super::ChatOverlay {
    /// The private duel partner's slot from the latest snapshot (`ps.duelIndex` while
    /// `ps.duelInProgress`), `None` outside a duel.
    pub(crate) fn set_duel_partner(&mut self, partner: Option<u16>) {
        self.duel_partner = partner;
    }

    /// While quiet mode is on, the identity keys whose SJK chat still passes: the
    /// player's own and the duel partner's, as the hub knows their slots; `None` while
    /// it is off.
    pub(crate) fn quiet_keys(&self) -> Option<[String; 2]> {
        self.options
            .quiet
            .then(|| [self.own_slot, self.duel_partner].map(|slot| self.slot_key(slot).to_owned()))
    }

    /// The identity key the hub knows in `slot`, empty when none.
    pub(super) fn slot_key(&self, slot: Option<u16>) -> &str {
        slot.and_then(|slot| self.slot_marks.get(usize::from(slot)))
            .map_or("", |mark| mark.key_id.as_str())
    }

    /// Whether an SJK chat line from `key` passes quiet mode: the player's own or the
    /// duel partner's, or the SJK team's (`official`).
    pub(super) fn sjk_line_loud(&self, key: &str, official: bool) -> bool {
        official
            || (!key.is_empty()
                && (key == self.slot_key(self.own_slot) || key == self.slot_key(self.duel_partner)))
    }

    /// Whether `line` shows under quiet mode.
    pub(super) fn quiet_visible(&self, line: &super::ChatLine) -> bool {
        !self.options.quiet || line.loud
    }

    /// Whether quiet mode hides server chat `text` (from `sender`, when the server named
    /// one; `private` for the server's tell marker), decided as the feed decides it: the
    /// console's alert words leave such a line out.
    pub(crate) fn quiets(&self, text: &str, sender: Option<u16>, private: bool) -> bool {
        if !self.options.quiet {
            return false;
        }
        let display = sjk_client::chat_display_text(&sjk_client::chat_unescape(text));
        let target = self
            .roster
            .target(sender)
            .or_else(|| self.line_sender(&display));
        let whisper = private
            || target
                .and_then(|target| self.roster.display_name(target))
                .is_some_and(|name| sjk_client::chat_body(&display, name).1);
        !game_line_loud(
            whisper,
            target.map(|target| target.slot()),
            self.own_slot,
            self.duel_partner,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat::ChatOverlay;
    use crate::chat::marks::SlotMark;
    use sjk_client::ServerEventKind;
    use std::time::Instant;

    #[test]
    fn the_command_toggles_parses_says_on_or_off_and_resets() {
        let directory = tempfile::tempdir().unwrap();
        let mut console = ViewerConsole::new(directory.path().join("config.cfg")).unwrap();
        assert_eq!(console.bool_cvar(CVAR), Some(false));
        let said = command(&mut console, &[]).unwrap();
        assert!(said[0].contains("ON"));
        assert_eq!(console.bool_cvar(CVAR), Some(true));
        assert!(command(&mut console, &["maybe".into()]).is_err());
        assert_eq!(console.bool_cvar(CVAR), Some(true));
        assert!(command(&mut console, &["off".into()]).unwrap()[0].contains("OFF"));
        assert_eq!(console.bool_cvar(CVAR), Some(false));
        command(&mut console, &["1".into()]).unwrap();
        reset(&mut console);
        assert_eq!(console.bool_cvar(CVAR), Some(false));
    }

    #[test]
    fn game_lines_pass_for_whispers_the_server_ourselves_and_the_partner() {
        assert!(game_line_loud(true, Some(4), Some(0), None), "a whisper");
        assert!(game_line_loud(false, None, Some(0), None), "the server's");
        assert!(game_line_loud(false, Some(0), Some(0), None), "our own");
        assert!(
            game_line_loud(false, Some(3), Some(0), Some(3)),
            "the partner"
        );
        assert!(!game_line_loud(false, Some(4), Some(0), Some(3)));
        assert!(!game_line_loud(false, Some(4), None, None));
    }

    #[test]
    fn only_public_beeps_of_others_than_the_partner_are_silenced() {
        use sjk_client::LegacySoundEvent::{ChatBeep, PrivateChatBeep, TeamChatBeep};
        let mut player = sjk_protocol::PlayerState::default();
        player.set_client_num(0);
        // In a private duel with slot 3 (`duelInProgress`, `duelIndex`).
        player.set_raw_field(119, 1);
        player.set_raw_field(44, 3);
        assert!(!silences_beep(false, ChatBeep, Some(4), &player), "off");
        assert!(silences_beep(true, ChatBeep, Some(4), &player));
        assert!(silences_beep(true, TeamChatBeep, Some(4), &player));
        assert!(!silences_beep(true, PrivateChatBeep, Some(4), &player));
        assert!(
            !silences_beep(true, ChatBeep, Some(3), &player),
            "the partner"
        );
        assert!(!silences_beep(true, ChatBeep, Some(0), &player), "our own");
        assert!(
            !silences_beep(true, ChatBeep, None, &player),
            "nobody named"
        );
        player.set_raw_field(119, 0);
        assert!(
            silences_beep(true, ChatBeep, Some(3), &player),
            "the duel is over"
        );
    }

    fn visible(chat: &ChatOverlay) -> Vec<String> {
        chat.lines
            .iter()
            .filter(|line| chat.shown(line))
            .map(|line| line.body.clone())
            .collect()
    }

    #[test]
    fn quiet_mode_hides_public_chat_but_the_partner_and_whispers() {
        let mut chat = ChatOverlay::new();
        let mut game = sjk_protocol::GameState::empty_local(0);
        for (slot, player) in [(3, &b"n\\Duelist\\t\\0"[..]), (4, &b"n\\Fox\\t\\0"[..])] {
            game.replace_config_string(1_131 + slot, player.to_vec())
                .expect("a player");
        }
        chat.update_roster(&game);
        let now = Instant::now();
        chat.set_duel_partner(Some(3));
        for (kind, text, sender, private) in [
            (ServerEventKind::Chat, "Fox: public", Some(4), false),
            (ServerEventKind::TeamChat, "(Fox): team", Some(4), false),
            (ServerEventKind::Chat, "Duelist: gg", Some(3), false),
            (ServerEventKind::Chat, "\u{19}[Fox]: tell", Some(4), true),
            (ServerEventKind::Chat, "Server notice", None, false),
        ] {
            chat.receive_classified(kind, text.into(), sender, private, now);
        }
        assert!(chat.receive_encrypted(None, (4, "Fox"), "sealed", false, now));
        assert_eq!(visible(&chat).len(), 6, "all show while it is off");
        chat.options.quiet = true;
        let shown = visible(&chat);
        assert_eq!(shown.len(), 4, "{shown:?}");
        assert!(
            !shown
                .iter()
                .any(|body| body.contains("public") || body.contains("team"))
        );
        // The duel ended: the partner's line stays, a new line of theirs is hidden.
        chat.set_duel_partner(None);
        chat.receive_classified(
            ServerEventKind::Chat,
            "Duelist: again".into(),
            Some(3),
            false,
            now,
        );
        assert!(chat.quiets("Duelist: again", Some(3), false));
        assert!(!chat.quiets("\u{19}[Fox]: tell", Some(4), true));
        assert_eq!(visible(&chat).len(), 4);
        chat.options.quiet = false;
        assert_eq!(visible(&chat).len(), 7, "history comes back");
    }

    #[test]
    fn sjk_lines_pass_for_the_partners_key_ours_and_the_teams() {
        let mut chat = ChatOverlay::new();
        assert_eq!(chat.quiet_keys(), None);
        let mark = |key: &str| SlotMark {
            sjk: true,
            verified: false,
            tick: sjk_identity::Tick::None,
            trust: None,
            emoji: String::new(),
            name: None,
            key_id: key.to_owned(),
        };
        chat.set_slot_marks(vec![mark("me"), mark(""), mark("partner"), mark("other")]);
        chat.own_slot = Some(0);
        chat.set_duel_partner(Some(2));
        assert!(chat.sjk_line_loud("me", false));
        assert!(chat.sjk_line_loud("partner", false));
        assert!(chat.sjk_line_loud("", true), "the SJK team's");
        assert!(!chat.sjk_line_loud("other", false));
        assert!(!chat.sjk_line_loud("", false));
        chat.options.quiet = true;
        assert_eq!(
            chat.quiet_keys(),
            Some(["me".to_owned(), "partner".to_owned()])
        );
    }
}
