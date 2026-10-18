//! Secrets the console never prints: the values of password and key cvars, and
//! the arguments of commands that carry a password.
//!
//! Every console line is kept in the scrollback, `qconsole.log`, the session log
//! and the crash trail a crash report or a log upload sends, so a password typed
//! as `rconpassword hunter2` or `amlogin hunter2` would travel with them. Three
//! layers keep it out:
//!
//! - a cvar named by [`SECRET_CVAR_PARTS`] gets [`crate::CvarFlags::SECRET`] when it
//!   is registered, and every description or listing shows its value through
//!   [`crate::Cvar::shown_value`]: [`MASK`] when it is set;
//! - the echo of a typed line goes through [`input_line`], which masks the
//!   arguments of [`SECRET_COMMANDS`], a secret cvar's new value, and a command line
//!   held in a cvar or a bind (`seta autologin "amlogin hunter2"`), also inside the
//!   [`WRAPPERS`] that forward a command line (`rcon`, `cmd`);
//! - every line the console keeps is scrubbed of the current values of the secret
//!   cvars ([`crate::CvarRegistry::scrub`]), so a path the first two miss (`strSub
//!   echo $rconPassword$`, a mod printing userinfo) still prints [`MASK`].
//!
//! The values themselves are unchanged and still reach the command or the server.
//!
//! The tables are mod-specific (`amlogin` and `loginpass` are JA+'s, `login` and
//! `register` JoF's), yet unlike other such tables (AGENTS.md, "No hardcoded special
//! cases") the player cannot turn an entry off: an override would make every log
//! that might carry a password unsafe to share, and a player who wants to read one
//! back has it in `config.cfg`. Adding a case is still one entry.

use crate::{split_commands, tokenize};

/// What a masked value shows in its place.
pub const MASK: &str = "********";

/// The shortest secret value [`crate::CvarRegistry::scrub`] looks for: a shorter
/// one (`1`, `on`) would mask ordinary text. The echo and the listings mask any
/// length.
pub const SCRUB_MIN_CHARS: usize = 4;

/// Parts of a cvar name (lowercase) that make its value secret.
pub const SECRET_CVAR_PARTS: &[&str] = &[
    // rconPassword, password (joining a locked server), g_password, sv_password,
    // cl_consoleSocketPassword.
    "password",
    // japlus.loginPass1-3, the JA+ admin passwords auto login sends.
    "loginpass",
    // cl_giphyKey, the player's own GIPHY API key.
    "giphykey",
];

/// Commands (lowercase) whose arguments are a password.
pub const SECRET_COMMANDS: &[&str] = &[
    // JA+ and JoF: `amlogin <password>` (sjk-mod-japlus, JoF EJK codemp/game).
    "amlogin", // JoF EJK and account mods: `login <name> <password>`.
    "login",   // JoF EJK and account mods: `register <name> <password>`.
    "register",
];

/// Commands whose arguments are themselves a command line, masked in turn:
/// `rcon <command>` runs it on the server, `cmd <command>` sends it as a client
/// command (`console_info.rs` in the viewer).
pub const WRAPPERS: &[&str] = &["rcon", "cmd"];

/// Commands whose first argument is a name (a cvar to set, a key to bind) and
/// whose other arguments are a value that may hold a command line.
const TARGETED: &[&str] = &["set", "seta", "sets", "setu", "bind"];

/// Whether a cvar called `name` holds a secret, by [`SECRET_CVAR_PARTS`].
pub fn is_secret_name(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    SECRET_CVAR_PARTS.iter().any(|part| name.contains(part))
}

/// What the echo needs to know about the cvars a line names.
pub trait Names {
    /// Whether `name` is a cvar whose value is secret.
    fn is_secret(&self, name: &str) -> bool;
    /// Whether `name` is a cvar (typing its name and a value sets it).
    fn is_cvar(&self, name: &str) -> bool;
}

/// Names known only by [`is_secret_name`], for a line read without a registry.
pub struct ByName;

impl Names for ByName {
    fn is_secret(&self, name: &str) -> bool {
        is_secret_name(name)
    }

    fn is_cvar(&self, _name: &str) -> bool {
        false
    }
}

/// `value` as a listing may show it: [`MASK`] for a set secret value; a command
/// line it holds with its passwords masked otherwise.
pub fn shown(secret: bool, value: &str) -> String {
    if value.is_empty() {
        String::new()
    } else if secret {
        MASK.to_owned()
    } else {
        input_line(value, &ByName)
    }
}

/// A typed console line as its echo may show it: each command that carries a
/// secret keeps its name (and the cvar or key it targets) with the rest masked; a
/// line with none comes back unchanged, quotes and spacing included.
pub fn input_line(line: &str, names: &dyn Names) -> String {
    let Ok(commands) = split_commands(line) else {
        return masked_unparsed(line, names);
    };
    let masked: Vec<Option<String>> = commands
        .iter()
        .map(|command| command_masked(command, names))
        .collect();
    if masked.iter().all(Option::is_none) {
        return line.to_owned();
    }
    commands
        .into_iter()
        .zip(masked)
        .map(|(command, masked)| masked.unwrap_or(command))
        .collect::<Vec<_>>()
        .join("; ")
}

/// `command` masked, or `None` when it carries no secret.
fn command_masked(command: &str, names: &dyn Names) -> Option<String> {
    let tokens = tokenize(command).ok()?;
    let (name, arguments) = tokens.split_first()?;
    if arguments.is_empty() {
        return None;
    }
    let lower = name.to_ascii_lowercase();
    if SECRET_COMMANDS.contains(&lower.as_str()) || names.is_secret(name) {
        return Some(format!("{name} {MASK}"));
    }
    let (first, rest) = arguments.split_first()?;
    // `set`, `toggle`, `ifCvar`, `cvarAdd` ... naming a secret cvar first.
    if !rest.is_empty() && names.is_secret(first) {
        return Some(format!("{name} {first} {MASK}"));
    }
    if WRAPPERS.contains(&lower.as_str()) {
        let inner = arguments.join(" ");
        let shown = input_line(&inner, names);
        return (shown != inner).then(|| format!("{name} {shown}"));
    }
    // A value that is a command line: `seta autologin "amlogin hunter2"`,
    // `bind F5 "amlogin hunter2"`, or `autologin "amlogin hunter2"` for a cvar.
    let (prefix, value) = if TARGETED.contains(&lower.as_str()) {
        (format!("{name} {first}"), rest.join(" "))
    } else if names.is_cvar(name) {
        (name.clone(), arguments.join(" "))
    } else {
        return None;
    };
    if value.is_empty() {
        return None;
    }
    let shown = input_line(&value, names);
    (shown != value).then(|| format!("{prefix} \"{shown}\""))
}

/// A line the tokenizer refuses (an open quote): kept up to the first secret word
/// (a word ends at a space, `;` or `"`), the rest masked, so a broken line never
/// shows a password either.
fn masked_unparsed(line: &str, names: &dyn Names) -> String {
    let separator = |c: char| c.is_whitespace() || c == ';' || c == '"';
    let mut start = None;
    for (at, c) in line
        .char_indices()
        .chain(std::iter::once((line.len(), ' ')))
    {
        match (start, separator(c)) {
            (None, false) => start = Some(at),
            (Some(from), true) => {
                start = None;
                let word = &line[from..at];
                let lower = word.to_ascii_lowercase();
                let secret = SECRET_COMMANDS.contains(&lower.as_str())
                    || names.is_secret(word)
                    || WRAPPERS.contains(&lower.as_str());
                if secret && !line[at..].trim_matches(separator).is_empty() {
                    return format!("{} {MASK}", &line[..at]);
                }
            }
            _ => {}
        }
    }
    line.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The registry's answers for a shell that has `autologin` and `rconPassword`.
    struct Registered;

    impl Names for Registered {
        fn is_secret(&self, name: &str) -> bool {
            is_secret_name(name)
        }

        fn is_cvar(&self, name: &str) -> bool {
            ["autologin", "rconpassword"].contains(&name.to_ascii_lowercase().as_str())
        }
    }

    fn echo(line: &str) -> String {
        input_line(line, &Registered)
    }

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
            assert!(is_secret_name(name), "{name}");
        }
        for name in [
            "name",
            "cg_fov",
            "rconAddress",
            "japlus.loginServer1",
            "cl_hubUrl",
        ] {
            assert!(!is_secret_name(name), "{name}");
        }
    }

    #[test]
    fn a_shown_value_hides_a_secret_or_the_password_in_a_script() {
        assert_eq!(shown(true, "hunter2"), MASK);
        assert_eq!(shown(true, ""), "");
        assert_eq!(shown(false, "110"), "110");
        assert_eq!(shown(false, "amlogin hunter2"), "amlogin ********");
        assert_eq!(
            shown(false, "say hi; amlogin hunter2"),
            "say hi; amlogin ********"
        );
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
            ("cmd amlogin hunter2", "cmd amlogin ********"),
            ("cmd login sol hunter2", "cmd login ********"),
            (
                "toggle rconPassword hunter2 other",
                "toggle rconPassword ********",
            ),
            (
                "ifCvar rconPassword hunter2 echo ok",
                "ifCvar rconPassword ********",
            ),
            (
                "bind F5 \"amlogin hunter2\"",
                "bind F5 \"amlogin ********\"",
            ),
            ("bind P amlogin afxg", "bind P \"amlogin ********\""),
            (
                "seta autologin \"amlogin hunter2\"",
                "seta autologin \"amlogin ********\"",
            ),
            (
                "set x \"say hi; amlogin hunter2\"",
                "set x \"say hi; amlogin ********\"",
            ),
            (
                "autologin \"amlogin hunter2\"",
                "autologin \"amlogin ********\"",
            ),
        ] {
            assert_eq!(echo(typed), shown, "{typed}");
        }
    }

    #[test]
    fn every_command_of_a_line_is_masked() {
        assert_eq!(
            echo("cg_fov 110; rconpassword hunter2;amlogin hunter2"),
            "cg_fov 110; rconpassword ********; amlogin ********"
        );
    }

    #[test]
    fn lines_without_a_secret_stay_as_typed() {
        for typed in [
            "cg_fov 110",
            "say \"hello  there\"",
            "say I will login later",
            "rconpassword",
            "amlogin",
            "set password",
            "rcon kick 3",
            "cmd say hi",
            "seta japlus.loginServer1 127.0.0.1",
            "bind F5 \"say gg\"",
            "seta autologin \"say gg\"",
        ] {
            assert_eq!(echo(typed), typed);
        }
    }

    #[test]
    fn an_unterminated_quote_still_hides_the_password() {
        for (typed, shown) in [
            ("amlogin \"hunter2", "amlogin ********"),
            ("seta rconpassword \"hunter2", "seta rconpassword ********"),
            (
                "cg_fov 110;amlogin \"hunter2",
                "cg_fov 110;amlogin ********",
            ),
            ("bind F5 \"amlogin hunter2", "bind F5 \"amlogin ********"),
            ("rcon \"g_password hunter2", "rcon ********"),
        ] {
            assert_eq!(echo(typed), shown, "{typed}");
        }
        assert_eq!(echo("say \"hi"), "say \"hi");
        assert_eq!(echo("say \"amlogin"), "say \"amlogin");
    }
}
