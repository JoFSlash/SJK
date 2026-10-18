//! Secrets the console never prints: the values of password and key cvars, and
//! the arguments of commands that carry a password.
//!
//! Every console line is kept in the scrollback, `qconsole.log`, the session log
//! and the crash trail a crash report or a log upload sends, so a password typed
//! as `rconpassword hunter2` or `amlogin hunter2` would travel with them. The
//! shell masks it where it prints: the echo of typed input ([`input_line`]) and
//! every cvar description or listing ([`shown_value`]). The values themselves are
//! unchanged and still reach the command or the server.
//!
//! The two tables below name what is secret, one entry per case. There is no
//! setting to show them again: a log that might carry a password cannot be
//! shared, and the value can still be read in `config.cfg` by its owner.

use crate::{split_commands, tokenize};

/// What a masked value shows in its place.
pub const MASK: &str = "********";

/// Parts of a cvar name (lowercase) that make its value secret.
const SECRET_CVAR_PARTS: &[(&str, &str)] = &[
    // rconPassword, password (joining a locked server), g_password, sv_password,
    // cl_consoleSocketPassword.
    ("password", "a server, rcon or console-socket password"),
    // japlus.loginPass1-3, the JA+ admin passwords auto login sends.
    ("loginpass", "a JA+ admin password"),
    // cl_giphyKey, the player's own GIPHY API key.
    ("giphykey", "a personal API key"),
];

/// Commands (lowercase) whose arguments are a password.
const SECRET_COMMANDS: &[(&str, &str)] = &[
    // JA+ and JoF: `amlogin <password>` (sjk-mod-japlus, JoF EJK codemp/game).
    ("amlogin", "an admin password"),
    // JoF EJK and account mods: `login <name> <password>`.
    ("login", "an account password"),
    // JoF EJK and account mods: `register <name> <password>`.
    ("register", "an account password"),
];

/// Commands that set the cvar named by their first argument.
const SETTERS: &[&str] = &["set", "seta", "sets", "setu"];

/// Commands whose arguments are themselves a command line, masked in turn.
const WRAPPERS: &[&str] = &["rcon"];

/// Whether the cvar `name`'s value is secret.
pub fn is_secret_cvar(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    SECRET_CVAR_PARTS
        .iter()
        .any(|(part, _)| name.contains(part))
}

/// The cvar `name`'s value as the console may print it: [`MASK`] for a secret
/// one that is set, the value otherwise.
pub fn shown_value<'a>(name: &str, value: &'a str) -> &'a str {
    if !value.is_empty() && is_secret_cvar(name) {
        MASK
    } else {
        value
    }
}

/// A typed console line as its echo may show it: each command that carries a
/// secret keeps its name (and the cvar it sets) with the rest masked; a line
/// with none comes back unchanged, quotes and spacing included.
pub fn input_line(line: &str) -> String {
    let Ok(commands) = split_commands(line) else {
        return masked_unparsed(line);
    };
    if !commands
        .iter()
        .any(|command| command_masked(command).is_some())
    {
        return line.to_owned();
    }
    commands
        .iter()
        .map(|command| command_masked(command).unwrap_or_else(|| command.clone()))
        .collect::<Vec<_>>()
        .join("; ")
}

/// `command` masked, or `None` when it carries no secret.
fn command_masked(command: &str) -> Option<String> {
    let tokens = tokenize(command).ok()?;
    let (name, arguments) = tokens.split_first()?;
    let lower = name.to_ascii_lowercase();
    if arguments.is_empty() {
        return None;
    }
    if SECRET_COMMANDS.iter().any(|(secret, _)| *secret == lower) {
        return Some(format!("{name} {MASK}"));
    }
    if SETTERS.contains(&lower.as_str()) {
        let (cvar, value) = arguments.split_first()?;
        return (!value.is_empty() && is_secret_cvar(cvar))
            .then(|| format!("{name} {cvar} {MASK}"));
    }
    if is_secret_cvar(name) {
        return Some(format!("{name} {MASK}"));
    }
    if WRAPPERS.contains(&lower.as_str()) {
        let inner = command_masked(&arguments.join(" "))?;
        return Some(format!("{name} {inner}"));
    }
    if lower == "bind" {
        let (key, script) = arguments.split_first()?;
        if script.is_empty() {
            return None;
        }
        let shown = input_line(&script.join(" "));
        return (shown != script.join(" ")).then(|| format!("{name} {key} \"{shown}\""));
    }
    None
}

/// A line the tokenizer refuses (an open quote): masked from the first secret
/// word on, so a broken line never shows a password either.
fn masked_unparsed(line: &str) -> String {
    let mut words = line.split_whitespace();
    let mut shown = Vec::new();
    while let Some(word) = words.next() {
        shown.push(word);
        let lower = word.trim_matches(['"', ';']).to_ascii_lowercase();
        let secret =
            SECRET_COMMANDS.iter().any(|(name, _)| *name == lower) || is_secret_cvar(&lower);
        if secret {
            if words.next().is_some() {
                shown.push(MASK);
            }
            break;
        }
    }
    shown.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_cvars_are_secret_and_others_are_not() {
        for name in [
            "rconPassword",
            "rconpassword",
            "password",
            "g_password",
            "sv_password",
            "cl_consoleSocketPassword",
            "japlus.loginPass1",
            "japlus.loginpass3",
            "cl_giphyKey",
        ] {
            assert!(is_secret_cvar(name), "{name}");
        }
        for name in [
            "name",
            "cg_fov",
            "rconAddress",
            "japlus.loginServer1",
            "cl_hubUrl",
        ] {
            assert!(!is_secret_cvar(name), "{name}");
        }
    }

    #[test]
    fn a_secret_value_is_masked_unless_it_is_empty() {
        assert_eq!(shown_value("rconPassword", "hunter2"), MASK);
        assert_eq!(shown_value("rconPassword", ""), "");
        assert_eq!(shown_value("cg_fov", "110"), "110");
    }

    #[test]
    fn typed_passwords_are_masked() {
        for (typed, shown) in [
            ("rconpassword hunter2", "rconpassword ********"),
            ("rconPassword \"two words\"", "rconPassword ********"),
            (
                "seta japlus.loginPass1 hunter2",
                "seta japlus.loginPass1 ********",
            ),
            ("set password hunter2", "set password ********"),
            ("setu password hunter2", "setu password ********"),
            ("amlogin hunter2", "amlogin ********"),
            ("AMLOGIN hunter2", "AMLOGIN ********"),
            ("login sol hunter2", "login ********"),
            ("register sol hunter2", "register ********"),
            ("rcon g_password hunter2", "rcon g_password ********"),
            (
                "rcon set sv_password hunter2",
                "rcon set sv_password ********",
            ),
            (
                "bind F5 \"amlogin hunter2\"",
                "bind F5 \"amlogin ********\"",
            ),
        ] {
            assert_eq!(input_line(typed), shown, "{typed}");
        }
    }

    #[test]
    fn every_command_of_a_line_is_masked() {
        assert_eq!(
            input_line("cg_fov 110; rconpassword hunter2;amlogin hunter2"),
            "cg_fov 110; rconpassword ********; amlogin ********"
        );
    }

    #[test]
    fn lines_without_a_secret_stay_as_typed() {
        for typed in [
            "cg_fov 110",
            "say \"hello  there\"",
            "rconpassword",
            "amlogin",
            "set password",
            "rcon kick 3",
            "seta japlus.loginServer1 127.0.0.1",
            "bind F5 \"say gg\"",
        ] {
            assert_eq!(input_line(typed), typed);
        }
    }

    #[test]
    fn an_unterminated_quote_still_hides_the_password() {
        assert_eq!(input_line("amlogin \"hunter2"), "amlogin ********");
        assert_eq!(
            input_line("seta rconpassword \"hunter2"),
            "seta rconpassword ********"
        );
        assert_eq!(input_line("say \"hi"), "say \"hi");
    }
}
