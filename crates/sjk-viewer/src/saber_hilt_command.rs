//! JoF EJK's `saber` console command (`CG_Saber_f`, `cg_consolecmds.c:1051-1085`;
//! `docs/client.md`, "Useful console commands"): the hilts the player holds, as JoF EJK
//! players type them. `saber kyle kyle` holds two sabers (`saber1` and `saber2`),
//! `saber dual_4` one (a staff's hilt makes it a staff; `saber2` goes to `none`), and
//! `saber` alone says what is held. It writes the `saber1`/`saber2` cvars, the one place
//! the menu reads and the userinfo sends, so a choice made from the console or a
//! quick-wheel custom command shows in Profile > Saber and the Collection's stage, and
//! stays on the next map. Sent to the server as is, the command swapped the saber on JA+
//! servers but left the cvars on the menu's choice, which the next map's userinfo brought
//! back (Kushina's report #126).
//!
//! Both names are checked against the installed `.sab` sabers before either cvar is set,
//! so a misspelt second hilt changes nothing (review of 22/10/2026). On JA+ (JoF's
//! servers among them) and jaPRO-lineage servers the change also goes to the server as
//! its own `saber` command, so it applies without a respawn: that is
//! `sjk_client::SaberChangeNotifier`, which follows the two cvars whoever sets them (the
//! menus too), with its table of the servers that take the command
//! (`profile_accepts_saber_command`). This command sets the cvars, then asks the
//! notifier to send even an unchanged pick (a switch the server refused can be asked
//! again, #132).

use crate::console::ViewerConsole;

/// Completion and help entry.
pub(crate) const COMMANDS: &[(&str, &str)] = &[(
    COMMAND,
    "Hold one saber, a staff or two sabers: saber <hilt> [<second hilt>] (saber kyle kyle, saber dual_4)",
)];

/// Lowercase name the client's command dispatch matches.
pub(crate) const COMMAND: &str = "saber";

const USAGE: &str = "saber <hilt> [<second hilt> | none]";

/// A hilt's name as the cvars keep it: its own spelling, or `None` when it is not a name.
fn hilt(word: &str) -> Option<&str> {
    let word = word.trim();
    (!word.is_empty()
        && word
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.')))
    .then_some(word)
}

/// Run `saber` with `args` on `console`; `known` says whether a hilt name is an installed
/// saber (any case).
pub(crate) fn run(
    console: &mut ViewerConsole,
    args: &[String],
    known: impl Fn(&str) -> bool,
) -> Result<Vec<String>, String> {
    let (first, second) = match args {
        [] => {
            let first = console
                .text_value("saber1")
                .unwrap_or("single_1")
                .to_owned();
            let second = console.text_value("saber2").unwrap_or("none").to_owned();
            return Ok(vec![if second.eq_ignore_ascii_case("none") {
                format!("You hold {first}. ({USAGE})")
            } else {
                format!("You hold {first} and {second}. ({USAGE})")
            }]);
        }
        [first] => (first.as_str(), "none"),
        [first, second] => (first.as_str(), second.as_str()),
        _ => return Err(format!("usage: {USAGE}")),
    };
    let first = hilt(first).ok_or_else(|| format!("usage: {USAGE}"))?;
    if first.eq_ignore_ascii_case("none") {
        return Err("The first saber cannot be none.".to_owned());
    }
    let second = hilt(second).ok_or_else(|| format!("usage: {USAGE}"))?;
    let second = if second.eq_ignore_ascii_case("none") {
        "none"
    } else {
        second
    };
    // Both names are checked before either is set.
    for name in [first, second] {
        if name != "none" && !known(name) {
            return Err(format!(
                "No saber named {name}: the Profile's Saber tab lists the installed ones."
            ));
        }
    }
    let before = console
        .text_value("saber1")
        .unwrap_or("single_1")
        .to_owned();
    if !console.set_cvar("saber1", first) {
        return Err("saber1 could not be set".to_owned());
    }
    if !console.set_cvar("saber2", second) {
        // The first goes back, so the two never disagree with what was asked.
        console.set_cvar("saber1", &before);
        return Err("saber2 could not be set; nothing changed".to_owned());
    }
    // As `CG_Saber_f`, every run asks the server, the same pick too: a switch it
    // refused (moving, not idle) left the cvars on the pick already (#132).
    console.request_saber_command();
    Ok(vec![if second == "none" {
        format!("You hold {first}.")
    } else {
        format!("You hold {first} and {second}.")
    }])
}

impl crate::GpuState {
    /// `saber [<hilt> [<second hilt>]]`.
    pub(crate) fn saber_hilt_command(&mut self, args: &[String]) -> Result<Vec<String>, String> {
        // The installed sabers, as the stage reads them (loaded once, on first need).
        if self.menu_stage.saber_definitions.is_none()
            && let Some(vfs) = self.vfs.clone()
        {
            // Unreadable, they stay unknown and the names go unchecked.
            self.menu_stage.saber_definitions = crate::saber_defs::load(&vfs).ok();
        }
        let definitions = self.menu_stage.saber_definitions.as_ref();
        let Some(console) = self.console.as_mut() else {
            return Err("The console is not ready".to_owned());
        };
        // Without game data to read them from, no name can be checked: they go as typed.
        run(console, args, |name| {
            definitions
                .is_none_or(|definitions| definitions.contains_key(&name.to_ascii_lowercase()))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(text: &str) -> Vec<String> {
        text.split_whitespace().map(str::to_owned).collect()
    }

    #[test]
    fn the_command_holds_two_sabers_a_staff_or_one() {
        let directory = tempfile::tempdir().unwrap();
        let mut console =
            ViewerConsole::new(directory.path().join("config.cfg")).expect("a console");
        let run = |console: &mut ViewerConsole, args: &[String]| run(console, args, |_| true);
        let lines = run(&mut console, &words("kyle kyle")).unwrap();
        assert_eq!(lines, ["You hold kyle and kyle."]);
        assert_eq!(console.text_value("saber1"), Some("kyle"));
        assert_eq!(console.text_value("saber2"), Some("kyle"));
        run(&mut console, &words("dual_4")).unwrap();
        assert_eq!(console.text_value("saber1"), Some("dual_4"));
        assert_eq!(console.text_value("saber2"), Some("none"));
        // The userinfo sends what the cvars hold.
        let userinfo = console.userinfo().unwrap();
        assert_eq!(userinfo.saber1, "dual_4");
        assert_eq!(userinfo.saber2, "none");
        run(&mut console, &words("single_1 NONE")).unwrap();
        assert_eq!(console.text_value("saber2"), Some("none"));
        assert!(run(&mut console, &[]).unwrap()[0].contains("You hold single_1."));
        assert!(run(&mut console, &words("none")).is_err());
        assert!(run(&mut console, &words("a b c")).is_err());
        assert!(run(&mut console, &words("kyle;quit")).is_err());
        assert_eq!(console.text_value("saber1"), Some("single_1"));
    }

    #[test]
    fn an_unknown_hilt_changes_neither_saber() {
        let directory = tempfile::tempdir().unwrap();
        let mut console =
            ViewerConsole::new(directory.path().join("config.cfg")).expect("a console");
        let known = |name: &str| ["kyle", "dual_4", "single_1"].contains(&name);
        run(&mut console, &words("kyle kyle"), known).unwrap();
        // A misspelt second hilt: the first stays as it was too.
        let refused = run(&mut console, &words("dual_4 kyel"), known).unwrap_err();
        assert!(refused.contains("kyel"), "{refused}");
        assert_eq!(console.text_value("saber1"), Some("kyle"));
        assert_eq!(console.text_value("saber2"), Some("kyle"));
        assert!(run(&mut console, &words("nosuch"), known).is_err());
        assert_eq!(console.text_value("saber1"), Some("kyle"));
        // `none` needs no saber of that name.
        run(&mut console, &words("dual_4 none"), known).unwrap();
        assert_eq!(console.text_value("saber2"), Some("none"));
        // What the server is told on JA+ and jaPRO servers, from the two cvars.
        assert_eq!(
            sjk_client::saber_command("dual_4", "none").as_deref(),
            Some("saber dual_4")
        );
    }
}
