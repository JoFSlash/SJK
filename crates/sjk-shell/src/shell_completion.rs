//! Tab completion of the command or cvar name being typed (`Field_CompleteCommand`,
//! `qcommon/common.cpp`): a unique name completes with a trailing space; several
//! complete to their longest shared prefix and are listed in the scrollback, with
//! descriptions while the listing is short. The file argument of `exec` and the
//! other commands in [`FILE_ARGUMENTS`] completes the same way from the files the
//! caller lists (`Cmd_CompleteCfgName`, `qcommon/cmd.cpp`).

use super::{ConsoleLineKind, Shell, ascii_starts_with_ignore_case, builtin_commands};

/// Commands whose first argument names a file, with the extension of the files it
/// completes from, as the completion functions `Cmd_Init` and `Com_Init` register.
const FILE_ARGUMENTS: &[(&str, &str)] = &[
    ("exec", "cfg"),
    ("execq", "cfg"),
    ("writeconfig", "cfg"),
    ("write", "cfg"),
];

/// Longest cvar value shown in a completion listing, as `TRUNCATE_LENGTH`.
const LISTED_VALUE_CHARS: usize = 64;
/// Most matches listed with their descriptions. Longer listings drop them and end with
/// the match count, so one listing fills as little of the bounded scrollback as it can.
const DESCRIBED_MATCHES: usize = 16;

/// The key that asked for a completion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompletionKey {
    /// Tab: candidates are listed whenever more than one remains.
    Tab,
    /// Enter with `cl_allowEnterCompletion`: the line runs next, so candidates are
    /// listed only when the completed text is not itself a full name.
    Enter,
}

/// One completion candidate.
struct Candidate<'a> {
    name: &'a str,
    description: &'a str,
    /// Current value of a cvar; `None` for a command.
    value: Option<String>,
}

impl<'a> Candidate<'a> {
    fn command(name: &'a str, description: &'a str) -> Self {
        Self {
            name,
            description,
            value: None,
        }
    }
}

impl Shell {
    /// Complete the name at the end of `line`: the first word after its last unquoted
    /// `;`, without a leading `/` or `\`. Such a slash is kept on the line's first
    /// command, which the console strips when the line is entered, and dropped from a
    /// later one, which nothing strips. Returns the completed line, or `None` when
    /// nothing matches or the line already continues past the name.
    pub fn complete_line(&mut self, line: &str, key: CompletionKey) -> Option<String> {
        self.complete_line_with_files(line, key, |_| Vec::new())
    }

    /// [`Self::complete_line`], and on Tab also the file argument of `exec` and the
    /// other commands in `FILE_ARGUMENTS`: `files` lists the names, with their
    /// extension, of the files of the given extension the command can open, highest
    /// priority first. Enter leaves arguments alone, as `Field_CompleteCommand` does.
    pub fn complete_line_with_files(
        &mut self,
        line: &str,
        key: CompletionKey,
        files: impl FnOnce(&str) -> Vec<String>,
    ) -> Option<String> {
        let (replaced, name, listing) = match typed_file_argument(line) {
            Some(argument) if key == CompletionKey::Tab => {
                let names = files(argument.extension);
                let (name, listing) = file_completion(&line[argument.start..], &names)?;
                (argument.start, name, listing)
            }
            Some(_) => return None,
            None => {
                let typed = typed_name(line)?;
                let (name, listing) = self.completion(&line[typed.name..], key)?;
                (typed.replaced, name, listing)
            }
        };
        let completed = format!("{}{name}", &line[..replaced]);
        if !listing.is_empty() {
            self.push_line(
                ConsoleLineKind::Input,
                format!("{}{completed}", super::PROMPT),
            );
            for text in listing {
                self.push_line(ConsoleLineKind::Output, text);
            }
        }
        Some(completed)
    }

    /// The unique name `line` would complete to, for a prompt hint; does not allocate.
    pub fn completion_hint(&self, line: &str) -> Option<&str> {
        self.unique_command_completion(&line[typed_name(line)?.name..])
    }

    /// The completed name and, when it is not unique, the lines listing the candidates.
    fn completion(&self, prefix: &str, key: CompletionKey) -> Option<(String, Vec<String>)> {
        let candidates = self.candidates(prefix);
        if let [only] = candidates.as_slice() {
            return Some((format!("{} ", only.name), Vec::new()));
        }
        let shared = shared_prefix(candidates.iter().map(|candidate| candidate.name))?;
        let full_name = candidates
            .iter()
            .any(|candidate| candidate.name.len() == shared.len());
        if key == CompletionKey::Enter && full_name {
            return Some((shared.to_owned(), Vec::new()));
        }
        let described = candidates.len() <= DESCRIBED_MATCHES;
        let mut listing = Vec::with_capacity(candidates.len() * 2 + 1);
        for candidate in &candidates {
            listing.push(listed(candidate));
            if described && !candidate.description.is_empty() {
                listing.push(format!("^2      {}", candidate.description));
            }
        }
        if !described {
            listing.push(format!(
                "^3{} matches; type more of the name to narrow them",
                candidates.len()
            ));
        }
        Some((shared.to_owned(), listing))
    }

    /// Commands, then cvars, whose names start with `prefix`, each sorted; a name that
    /// is both a command and a cvar is listed once, as the command.
    fn candidates(&self, prefix: &str) -> Vec<Candidate<'_>> {
        let mut commands = builtin_commands()
            .map(|(name, description)| Candidate::command(name, description))
            .chain(
                self.commands
                    .iter()
                    .map(|(name, description)| Candidate::command(name, description)),
            )
            .chain(
                self.external_command_help
                    .iter()
                    .map(|(name, description)| Candidate::command(name, description)),
            )
            .filter(|candidate| ascii_starts_with_ignore_case(candidate.name, prefix))
            .collect::<Vec<_>>();
        commands.sort_by_cached_key(|candidate| candidate.name.to_ascii_lowercase());
        commands.dedup_by(|later, earlier| later.name.eq_ignore_ascii_case(earlier.name));
        let mut cvars = self
            .cvars
            .iter()
            .filter(|cvar| ascii_starts_with_ignore_case(&cvar.name, prefix))
            .filter(|cvar| {
                !commands
                    .iter()
                    .any(|command| command.name.eq_ignore_ascii_case(&cvar.name))
            })
            .map(|cvar| Candidate {
                name: &cvar.name,
                description: &cvar.description,
                value: Some(listed_value(cvar.shown_value())),
            })
            .collect::<Vec<_>>();
        cvars.sort_by_cached_key(|candidate| candidate.name.to_ascii_lowercase());
        commands.append(&mut cvars);
        commands
    }
}

/// Byte offsets in a line of the name being typed.
struct TypedName {
    /// Start of the name itself.
    name: usize,
    /// Start of the text its completion replaces: the name, or the slash before it on a
    /// command after a `;`.
    replaced: usize,
}

/// The name being typed: after the last `;` that [`split_commands`](crate::split_commands)
/// splits on, leading whitespace and one `/` or `\`. `None` when that name is empty, an
/// argument follows it or a quote is still open.
fn typed_name(line: &str) -> Option<TypedName> {
    // The splitter's own view of the line; normalizing only rewrites the first
    // command's argument, so the name at the end is a suffix of both spellings.
    let normalized = crate::key_names::normalize_stock_backslash(line);
    let segment = last_command(&normalized)?;
    let command = normalized[segment..].trim_start_matches(|c: char| c.is_ascii_whitespace());
    let name = command.strip_prefix(['/', '\\']).unwrap_or(command);
    let continues = name
        .bytes()
        .any(|byte| byte.is_ascii_whitespace() || byte == b'"');
    if name.is_empty() || continues {
        return None;
    }
    let start = line.len() - name.len();
    Some(TypedName {
        name: start,
        replaced: if segment == 0 {
            start
        } else {
            line.len() - command.len()
        },
    })
}

/// Start of the last command on `line`: after its last `;` that
/// [`split_commands`](crate::split_commands) splits on. `None` while a quote is open.
fn last_command(line: &str) -> Option<usize> {
    let mut segment = 0;
    let mut quoted = false;
    let mut escaped = false;
    for (index, byte) in line.bytes().enumerate() {
        if escaped {
            escaped = false;
        } else if byte == b'\\' && quoted {
            escaped = true;
        } else if byte == b'"' {
            quoted = !quoted;
        } else if byte == b';' && !quoted {
            segment = index + 1;
        }
    }
    (!quoted && !escaped).then_some(segment)
}

/// The file argument being typed to a command in [`FILE_ARGUMENTS`].
struct TypedFileArgument {
    /// Byte offset in the line of the argument; it runs to the end of the line.
    start: usize,
    /// Extension of the files it completes from.
    extension: &'static str,
}

/// The first argument of the last command on `line` when that command takes a file
/// and nothing follows the argument. The argument may still be empty.
fn typed_file_argument(line: &str) -> Option<TypedFileArgument> {
    let command = line[last_command(line)?..].trim_start_matches(|c: char| c.is_ascii_whitespace());
    let command = command.strip_prefix(['/', '\\']).unwrap_or(command);
    let (name, rest) = command.split_once(|c: char| c.is_ascii_whitespace())?;
    let argument = rest.trim_start_matches(|c: char| c.is_ascii_whitespace());
    if argument
        .bytes()
        .any(|byte| byte.is_ascii_whitespace() || byte == b'"')
    {
        return None;
    }
    let &(_, extension) = FILE_ARGUMENTS
        .iter()
        .find(|(command, _)| command.eq_ignore_ascii_case(name))?;
    Some(TypedFileArgument {
        start: line.len() - argument.len(),
        extension,
    })
}

/// The completed file name and, when it is not unique, the lines listing the
/// candidates, as `PrintFileMatches` (`qcommon/common.cpp`) prints them. A name met
/// twice, in any letter case, is listed once, as first met. A unique name replaces
/// the typed text; several only extend it, keeping the letter case typed so far.
fn file_completion(prefix: &str, names: &[String]) -> Option<(String, Vec<String>)> {
    let mut candidates = names
        .iter()
        .filter(|name| ascii_starts_with_ignore_case(name, prefix))
        .collect::<Vec<_>>();
    // A stable sort keeps the first met of the names equal without case before
    // the others, so dedup keeps it.
    candidates.sort_by_cached_key(|name| name.to_ascii_lowercase());
    candidates.dedup_by(|later, earlier| later.eq_ignore_ascii_case(earlier));
    if let [only] = candidates.as_slice() {
        return Some((format!("{only} "), Vec::new()));
    }
    let shared = shared_prefix(candidates.iter().map(|name| name.as_str()))?;
    // Every candidate starts with `prefix` without case, so its bytes end on a
    // boundary of `shared` too.
    let completed = format!("{prefix}{}", &shared[prefix.len()..]);
    let listing = candidates
        .iter()
        .map(|name| format!("^9File  ^7{name}"))
        .collect();
    Some((completed, listing))
}

/// Longest case-insensitive prefix shared by every name, spelled as the first.
fn shared_prefix<'a>(mut names: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    let first = names.next()?;
    let mut length = first.len();
    for name in names {
        length = first
            .bytes()
            .zip(name.bytes())
            .take(length)
            .take_while(|(left, right)| left.eq_ignore_ascii_case(right))
            .count();
    }
    while !first.is_char_boundary(length) {
        length -= 1;
    }
    Some(&first[..length])
}

/// One candidate's listing line, coloured as `PrintMatches` and
/// `PrintCvarMatches` (`qcommon/common.cpp`) print it: a grey `Cmd` or `Cvar`
/// label, the white name and, for a cvar, its value in grey quotes.
fn listed(candidate: &Candidate<'_>) -> String {
    match &candidate.value {
        None => format!("^9Cmd   ^7{}", candidate.name),
        Some(value) => format!("^9Cvar  ^7{} = ^9\"^7{value}^9\"^7", candidate.name),
    }
}

fn listed_value(value: String) -> String {
    match value.char_indices().nth(LISTED_VALUE_CHARS) {
        Some((end, _)) => format!("{}...", &value[..end]),
        None => value,
    }
}

#[cfg(test)]
mod tests {
    use crate::{BindTable, CommandRegistry, CvarDefinition, CvarFlags, CvarRegistry, Shell};

    use super::CompletionKey;

    fn shell() -> Shell {
        let mut cvars = CvarRegistry::new();
        for (name, help) in [("con_scale", "Console size"), ("con_style", "")] {
            cvars
                .register(CvarDefinition::new(name, 1.0, CvarFlags::NONE, help))
                .unwrap();
        }
        let mut shell = Shell::new(cvars, BindTable::new());
        shell.commands = CommandRegistry::new();
        shell
    }

    #[test]
    fn listing_colours_labels_values_and_descriptions_as_ejk() {
        let mut shell = shell();
        shell
            .commands
            .register("con_dump", "Dump the console", |_| Ok(Vec::new()))
            .unwrap();
        let completed = shell.complete_line("con_", CompletionKey::Tab).unwrap();
        assert_eq!(completed, "con_");
        let lines: Vec<_> = shell.lines().map(|line| line.text.as_str()).collect();
        assert_eq!(
            lines,
            [
                "]con_",
                "^9Cmd   ^7con_dump",
                "^2      Dump the console",
                "^9Cvar  ^7con_scale = ^9\"^71.0^9\"^7",
                "^2      Console size",
                "^9Cvar  ^7con_style = ^9\"^71.0^9\"^7",
            ]
        );
    }

    fn cfgs(extension: &str) -> Vec<String> {
        assert_eq!(extension, "cfg");
        ["moj.cfg", "Mod.cfg", "autoexec.cfg", "MOJ.cfg"]
            .map(str::to_owned)
            .to_vec()
    }

    #[test]
    fn exec_completes_a_unique_cfg_name() {
        let mut shell = shell();
        let completed = shell.complete_line_with_files("exec mo", CompletionKey::Tab, cfgs);
        assert_eq!(completed.as_deref(), Some("exec mo"));
        let completed = shell.complete_line_with_files("/exec mo", CompletionKey::Tab, cfgs);
        assert_eq!(completed.as_deref(), Some("/exec mo"));
        let completed =
            shell.complete_line_with_files("echo x; execq MOJ", CompletionKey::Tab, cfgs);
        assert_eq!(completed.as_deref(), Some("echo x; execq moj.cfg "));
        let completed = shell.complete_line_with_files("exec a", CompletionKey::Tab, cfgs);
        assert_eq!(completed.as_deref(), Some("exec autoexec.cfg "));
    }

    #[test]
    fn exec_lists_the_cfgs_it_could_complete_to() {
        let mut shell = shell();
        let completed = shell.complete_line_with_files("exec ", CompletionKey::Tab, cfgs);
        assert_eq!(completed.as_deref(), Some("exec "));
        let lines: Vec<_> = shell.lines().map(|line| line.text.as_str()).collect();
        assert_eq!(
            lines,
            [
                "]exec ",
                "^9File  ^7autoexec.cfg",
                "^9File  ^7Mod.cfg",
                "^9File  ^7moj.cfg",
            ]
        );
    }

    #[test]
    fn file_arguments_complete_only_on_tab_and_only_the_first_argument() {
        let mut shell = shell();
        for line in ["exec moj.cfg x", "exec \"mo", "bind mo"] {
            assert_eq!(
                shell.complete_line_with_files(line, CompletionKey::Tab, cfgs),
                None,
                "{line}"
            );
        }
        assert_eq!(
            shell.complete_line_with_files("exec mo", CompletionKey::Enter, cfgs),
            None
        );
        assert_eq!(
            shell.complete_line_with_files("exec nothing", CompletionKey::Tab, cfgs),
            None
        );
    }

    #[test]
    fn typed_lines_echo_after_a_bare_prompt() {
        let mut shell = shell();
        let _ = shell.execute_line("echo hi");
        assert_eq!(shell.lines().next().unwrap().text, "]echo hi");
    }
}
