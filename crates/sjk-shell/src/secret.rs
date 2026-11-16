//! Secrets the console never prints: the values of password and key cvars, and
//! the arguments of commands that carry a password.
//!
//! Every console line is kept in the scrollback, `qconsole.log`, the session log
//! and the crash trail a crash report or a log upload sends, so a password typed
//! as `rconpassword hunter2` would travel with them. Three layers keep it out:
//!
//! - a cvar named by a secret cvar part ([`is_secret_name`]) gets
//!   [`crate::CvarFlags::SECRET`] when it is registered, and every description or
//!   listing shows its value through [`crate::Cvar::shown_value`]: [`MASK`] when it
//!   is set;
//! - the echo of a typed line goes through [`input_line`], which masks the
//!   arguments of a secret command, a secret cvar's new value, and a command line
//!   held in a cvar or a bind (`seta autologin "<command> <password>"`), also inside
//!   the [`COMMAND_LINES`] that carry one (`rcon`, `cmd`, `strSub`, `delay`,
//!   `ifCvar`);
//! - every line the console keeps is scrubbed of the current values of the secret
//!   cvars ([`crate::CvarRegistry::scrub`]), so a path the first two miss (`strSub
//!   echo $rconPassword$`, a mod printing userinfo) still prints [`MASK`].
//!
//! The values themselves are unchanged and still reach the command or the server.
//!
//! The base tables ([`BASE`]) hold only stock Jedi Academy's and SJK's own secrets.
//! A server family's commands and cvars (JA+'s `amlogin`, for one) belong to its
//! client mod, which declares them in a table of its own; the client [`add`]s every
//! table its build carries at startup, before any cvar is registered or any line is
//! run, so they are masked offline, in binds and scripts run at startup, on every
//! server and with the mod switched off. Unlike other per-server tables (AGENTS.md,
//! "No hardcoded special cases") the player cannot turn an entry off: an override
//! would make every log that might carry a password unsafe to share, and a player
//! who wants to read one back has it in `config.cfg`. Adding a case is still one
//! entry.

use std::sync::{PoisonError, RwLock};

use crate::{split_commands, tokenize};

/// What a masked value shows in its place.
pub const MASK: &str = "********";

/// The shortest secret value [`crate::CvarRegistry::scrub`] looks for: a shorter
/// one (`1`, `on`) would mask ordinary text. The echo and the listings mask any
/// length.
pub const SCRUB_MIN_CHARS: usize = 4;

/// A table of secrets: what makes a cvar's value secret and which commands take a
/// password. Every word is lowercase.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Table {
    /// Parts of a cvar name that make its value secret.
    pub cvar_parts: &'static [&'static str],
    /// Commands whose arguments are a password.
    pub commands: &'static [&'static str],
}

/// Stock Jedi Academy's and SJK's own secrets, always in force.
pub const BASE: Table = Table {
    cvar_parts: SECRET_CVAR_PARTS,
    commands: SECRET_COMMANDS,
};

/// Parts of a cvar name (lowercase) that make its value secret.
pub const SECRET_CVAR_PARTS: &[&str] = &[
    // rconPassword, password (joining a locked server), g_password, sv_password,
    // cl_consoleSocketPassword.
    "password", // cl_giphyKey, the player's own GIPHY API key.
    "giphykey",
];

/// Commands (lowercase) whose arguments are a password. Stock Jedi Academy has
/// none: its passwords are cvars. A mod's server commands come with its table.
pub const SECRET_COMMANDS: &[&str] = &[];

/// The tables [`add`]ed after [`BASE`]; only ever grows.
static ADDED: RwLock<Vec<Table>> = RwLock::new(Vec::new());

/// Mask `table`'s secrets from now on, in every registry and line, whatever the
/// server. Adding a table twice changes nothing. Call it before registering the
/// cvars it names, so they get [`crate::CvarFlags::SECRET`].
pub fn add(table: Table) {
    let mut added = ADDED.write().unwrap_or_else(PoisonError::into_inner);
    if !added.contains(&table) {
        added.push(table);
    }
}

/// Whether any table in force answers `test`.
fn any_table(test: impl Fn(&Table) -> bool) -> bool {
    test(&BASE)
        || ADDED
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .any(test)
}

/// Whether `lower` (a lowercase command name) takes a password.
fn is_secret_command(lower: &str) -> bool {
    any_table(|table| table.commands.contains(&lower))
}

/// Commands (lowercase) whose arguments from the given one on are a command line,
/// masked in turn:
///
/// - `rcon <command>`: runs it on the server;
/// - `cmd <command>`: sends it as a client command (`console_info.rs` in the viewer);
/// - `strSub <command>`: runs it with `$cvar$` substituted;
/// - `delay <ms> <command>` and `waitf <frames> <command>`: the command typed on
///   the same line by mistake (the shell refuses it, but echoes it first);
/// - `ifCvar <cvar> <condition> <argc> <command ...>`: the branches' commands.
pub const COMMAND_LINES: &[(&str, usize)] = &[
    ("rcon", 0),
    ("cmd", 0),
    ("strsub", 0),
    ("delay", 1),
    ("waitf", 1),
    ("ifcvar", 3),
];

/// Commands whose first argument is a name (a cvar to set, a key to bind) and
/// whose other arguments are a value that may hold a command line.
const TARGETED: &[&str] = &["set", "seta", "sets", "setu", "bind"];

/// Whether a cvar called `name` holds a secret, by the cvar parts of the tables
/// in force.
pub fn is_secret_name(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    any_table(|table| table.cvar_parts.iter().any(|part| name.contains(part)))
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
/// line it holds with its passwords masked otherwise. Only a value that names a
/// password command or a secret cvar is parsed, so most cost a scan.
pub fn shown(secret: bool, value: &str) -> String {
    if value.is_empty() {
        String::new()
    } else if secret {
        MASK.to_owned()
    } else if mentions_secret(value) {
        input_line(value, &ByName)
    } else {
        value.to_owned()
    }
}

/// Whether `text` holds a secret command or cvar part of the tables in force, in
/// any case: the only text [`input_line`] can mask.
fn mentions_secret(text: &str) -> bool {
    let text = text.to_ascii_lowercase();
    any_table(|table| {
        table
            .commands
            .iter()
            .chain(table.cvar_parts)
            .any(|word| text.contains(word))
    })
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
    if is_secret_command(&lower) || names.is_secret(name) {
        return Some(format!("{name} {MASK}"));
    }
    let (first, rest) = arguments.split_first()?;
    // `set`, `toggle`, `ifCvar`, `cvarAdd` ... naming a secret cvar first.
    if !rest.is_empty() && names.is_secret(first) {
        return Some(format!("{name} {first} {MASK}"));
    }
    if let Some(&(_, at)) = COMMAND_LINES.iter().find(|(line, _)| *line == lower) {
        let (kept, inner) = arguments.split_at(at.min(arguments.len()));
        if inner.is_empty() {
            return None;
        }
        let inner = inner.join(" ");
        let shown = input_line(&inner, names);
        let prefix = std::iter::once(name.as_str())
            .chain(kept.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" ");
        return (shown != inner).then(|| format!("{prefix} {shown}"));
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
                let secret = is_secret_command(&lower)
                    || names.is_secret(word)
                    || COMMAND_LINES.iter().any(|(line, _)| *line == lower);
                if secret && !line[at..].trim_matches(separator).is_empty() {
                    return format!("{} {MASK}", &line[..at]);
                }
            }
            _ => {}
        }
    }
    line.to_owned()
}

/// A mod's table for the shell's own tests: `modlogin <password>`, `modregister
/// <name> <password>` and cvars named `*modpass*`. Real mod tables are tested in
/// their crates.
#[cfg(test)]
pub(crate) const TEST_MOD: Table = Table {
    cvar_parts: &["modpass"],
    commands: &["modlogin", "modregister"],
};

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
        add(TEST_MOD);
        input_line(line, &Registered)
    }

    #[test]
    fn password_cvars_are_secret_and_others_are_not() {
        add(TEST_MOD);
        for name in [
            "rconPassword",
            "rconpassword",
            "password",
            "g_password",
            "sv_password",
            "cl_consoleSocketPassword",
            "mod.modPass1",
            "mod.modpass3",
            "cl_giphyKey",
        ] {
            assert!(is_secret_name(name), "{name}");
        }
        for name in [
            "name",
            "cg_fov",
            "rconAddress",
            "mod.modServer1",
            "cl_hubUrl",
        ] {
            assert!(!is_secret_name(name), "{name}");
        }
    }

    #[test]
    fn a_shown_value_hides_a_secret_or_the_password_in_a_script() {
        add(TEST_MOD);
        assert_eq!(shown(true, "hunter2"), MASK);
        assert_eq!(shown(true, ""), "");
        assert_eq!(shown(false, "110"), "110");
        assert_eq!(shown(false, "modlogin hunter2"), "modlogin ********");
        assert_eq!(
            shown(false, "say hi; modlogin hunter2"),
            "say hi; modlogin ********"
        );
    }

    #[test]
    fn typed_passwords_are_masked() {
        for (typed, shown) in [
            ("rconpassword hunter2", "rconpassword ********"),
            ("rconPassword \"two words\"", "rconPassword ********"),
            ("seta mod.modPass1 hunter2", "seta mod.modPass1 ********"),
            ("set password hunter2", "set password ********"),
            ("setu password hunter2", "setu password ********"),
            ("modlogin hunter2", "modlogin ********"),
            ("MODLOGIN hunter2", "MODLOGIN ********"),
            ("modregister sol hunter2", "modregister ********"),
            ("rcon g_password hunter2", "rcon g_password ********"),
            (
                "rcon set sv_password hunter2",
                "rcon set sv_password ********",
            ),
            ("cmd modlogin hunter2", "cmd modlogin ********"),
            ("cmd modregister sol hunter2", "cmd modregister ********"),
            (
                "toggle rconPassword hunter2 other",
                "toggle rconPassword ********",
            ),
            (
                "ifCvar rconPassword hunter2 echo ok",
                "ifCvar rconPassword ********",
            ),
            (
                "bind F5 \"modlogin hunter2\"",
                "bind F5 \"modlogin ********\"",
            ),
            ("bind P modlogin afxg", "bind P \"modlogin ********\""),
            (
                "seta autologin \"modlogin hunter2\"",
                "seta autologin \"modlogin ********\"",
            ),
            (
                "set x \"say hi; modlogin hunter2\"",
                "set x \"say hi; modlogin ********\"",
            ),
            (
                "autologin \"modlogin hunter2\"",
                "autologin \"modlogin ********\"",
            ),
        ] {
            assert_eq!(echo(typed), shown, "{typed}");
        }
    }

    #[test]
    fn every_command_of_a_line_is_masked() {
        assert_eq!(
            echo("cg_fov 110; rconpassword hunter2;modlogin hunter2"),
            "cg_fov 110; rconpassword ********; modlogin ********"
        );
    }

    #[test]
    fn lines_without_a_secret_stay_as_typed() {
        for typed in [
            "cg_fov 110",
            "say \"hello  there\"",
            "say I will login later",
            "rconpassword",
            "modlogin",
            "modregister",
            "set password",
            "rcon kick 3",
            "cmd say hi",
            "seta mod.modServer1 127.0.0.1",
            "bind F5 \"say gg\"",
            "seta autologin \"say gg\"",
        ] {
            assert_eq!(echo(typed), typed);
        }
    }

    #[test]
    fn an_unterminated_quote_still_hides_the_password() {
        for (typed, shown) in [
            ("modlogin \"hunter2", "modlogin ********"),
            ("seta rconpassword \"hunter2", "seta rconpassword ********"),
            (
                "cg_fov 110;modlogin \"hunter2",
                "cg_fov 110;modlogin ********",
            ),
            ("bind F5 \"modlogin hunter2", "bind F5 \"modlogin ********"),
            ("rcon \"g_password hunter2", "rcon ********"),
        ] {
            assert_eq!(echo(typed), shown, "{typed}");
        }
        assert_eq!(echo("say \"hi"), "say \"hi");
        assert_eq!(echo("say \"modlogin"), "say \"modlogin");
    }

    #[test]
    fn command_lines_after_their_arguments_are_masked() {
        for (typed, shown) in [
            ("strSub modlogin hunter2", "strSub modlogin ********"),
            ("delay 500 modlogin hunter2", "delay 500 modlogin ********"),
            (
                "waitf 2 modregister sol hunter2",
                "waitf 2 modregister ********",
            ),
            (
                "ifCvar g_x =1 2 modlogin hunter2",
                "ifCvar g_x =1 2 modlogin ********",
            ),
        ] {
            assert_eq!(echo(typed), shown, "{typed}");
        }
        for typed in ["delay 500", "strSub echo $name$", "ifCvar g_x =1 2 say hi"] {
            assert_eq!(echo(typed), typed);
        }
    }

    #[test]
    fn a_value_without_a_secret_word_is_shown_as_it_is() {
        add(TEST_MOD);
        assert!(!mentions_secret("say gg; +attack"));
        assert!(mentions_secret("say gg; MODLOGIN x"));
        assert_eq!(shown(false, "say gg; +attack"), "say gg; +attack");
    }
}

#[cfg(test)]
mod table_tests {
    use super::*;

    /// No other test of this crate adds a table with these words.
    const ONLY_HERE: Table = Table {
        cvar_parts: &["tablepart"],
        commands: &["tablelogin"],
    };

    #[test]
    fn base_tables_hold_no_server_commands() {
        assert!(SECRET_COMMANDS.is_empty());
        assert!(!SECRET_CVAR_PARTS.contains(&"loginpass"));
    }

    #[test]
    fn an_added_table_masks_from_then_on_and_adding_it_again_changes_nothing() {
        assert_eq!(
            input_line("tablelogin hunter2", &ByName),
            "tablelogin hunter2"
        );
        assert!(!is_secret_name("x.tablePart1"));
        add(ONLY_HERE);
        add(ONLY_HERE);
        assert_eq!(
            input_line("tablelogin hunter2", &ByName),
            "tablelogin ********"
        );
        assert!(is_secret_name("x.tablePart1"));
        assert_eq!(
            shown(false, "say hi; tablelogin x"),
            "say hi; tablelogin ********"
        );
        let added = ADDED.read().unwrap();
        assert_eq!(added.iter().filter(|table| **table == ONLY_HERE).count(), 1);
    }
}
