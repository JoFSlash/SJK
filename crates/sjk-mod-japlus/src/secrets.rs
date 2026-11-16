//! The passwords JA+ servers and the EternalJK family of clients made for them
//! take, masked by the console in every line it prints or keeps (`docs/mods.md`,
//! `sjk_shell::secret`), and their chat commands, whose typed lines stay out of the
//! session log, crash reports and log uploads as received chat does.
//!
//! The client masks this table from startup whatever the server and `mod_japlus`:
//! `amlogin <password>` typed offline, held in a bind or run from a config at
//! startup never reaches the scrollback, `qconsole.log`, the session log, a crash
//! report or a log upload. There is no player override, for the reason the base
//! table has none: an entry turned off would make every log unsafe to share.
//! Adding a case is one entry with its source.

use sjk_mod::Secrets;

/// Commands whose arguments are a password.
const COMMANDS: &[&str] = &[
    // `amlogin <password>`: JA+ admin login, which JoF's server takes too
    // (JA+ `g_cmds.c`; JoF EternalJK's `autoLogin`, `cg_consolecmds.c:1088`).
    "amlogin",
    // `login <name> <password>`: JoF EternalJK's accounts (`g_account.c`), the
    // JA+ family's account mods.
    "login",
    // `register <name> <password>`: the same accounts.
    "register",
    // `changepassword <name> <password> <new password>`: the same accounts.
    "changepassword",
    // `clanpass <password>`: the password of JA+ clan chat (`g_cmds.c`
    // `Cmd_Clanpass_f`).
    "clanpass",
    // `clanwhois <clan password>`: JoF's list of a clan's members (`g_cmds.c`).
    "clanwhois",
];

/// Parts of a cvar name that make its value secret.
const CVAR_PARTS: &[&str] = &[
    // japlus.loginPass1-3, the admin passwords `japlus.autologin` sends.
    "loginpass",
];

/// Commands whose arguments are a chat message (JoF EJK `codemp/game/g_cmds.c`).
const CHAT_COMMANDS: &[&str] = &[
    // Team chat as a mod sends it.
    "say_team_mod",
    // Clan chat.
    "clansay",
    // Admin chat, to every admin and to one player.
    "amsay",
    "ampsay",
];

/// The table the client applies from startup.
pub(crate) const SECRETS: Secrets = Secrets {
    commands: COMMANDS,
    cvar_parts: CVAR_PARTS,
    chat_commands: CHAT_COMMANDS,
};

#[cfg(test)]
mod tests {
    use super::*;
    use sjk_shell::secret::{ByName, MASK, Table, add, input_line, is_secret_name, shown};

    fn masked() {
        add(Table {
            cvar_parts: SECRETS.cvar_parts,
            commands: SECRETS.commands,
        });
    }

    fn echo(line: &str) -> String {
        masked();
        input_line(line, &ByName)
    }

    #[test]
    fn every_word_is_lowercase() {
        for word in SECRETS
            .commands
            .iter()
            .chain(SECRETS.cvar_parts)
            .chain(SECRETS.chat_commands)
        {
            assert_eq!(*word, word.to_ascii_lowercase());
        }
    }

    #[test]
    fn login_password_settings_are_secret_and_their_servers_are_not() {
        masked();
        for slot in 1..=crate::login::SLOTS {
            let (server, password) = crate::login::names(slot);
            assert!(is_secret_name(&password), "{password}");
            assert!(!is_secret_name(&server), "{server}");
        }
    }

    #[test]
    fn typed_passwords_are_masked() {
        for (typed, expected) in [
            ("amlogin hunter2", "amlogin ********"),
            ("AMLOGIN hunter2", "AMLOGIN ********"),
            ("login sol hunter2", "login ********"),
            ("register sol hunter2", "register ********"),
            ("changepassword sol hunter2 new2", "changepassword ********"),
            ("clanPass hunter2", "clanPass ********"),
            ("clanwhois hunter2", "clanwhois ********"),
            ("cmd amlogin hunter2", "cmd amlogin ********"),
            ("delay 500 amlogin hunter2", "delay 500 amlogin ********"),
            (
                "bind F5 \"amlogin hunter2\"",
                "bind F5 \"amlogin ********\"",
            ),
            (
                "seta japlus.loginPass1 hunter2",
                "seta japlus.loginPass1 ********",
            ),
            (
                "seta autologin \"say hi; amlogin hunter2\"",
                "seta autologin \"say hi; amlogin ********\"",
            ),
        ] {
            assert_eq!(echo(typed), expected, "{typed}");
        }
        assert_eq!(shown(false, "amlogin hunter2"), format!("amlogin {MASK}"));
    }

    #[test]
    fn lines_without_a_password_stay_as_typed() {
        for typed in [
            "amlogin",
            "clanwhois",
            "clanjoin Jedi",
            "say I will login later",
            "seta japlus.loginServer1 127.0.0.1",
        ] {
            assert_eq!(echo(typed), typed);
        }
    }
}
