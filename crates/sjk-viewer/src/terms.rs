//! The names players read for medals and holocrons (`docs/mods.md`, "Honors and
//! Echoes"): with the JoF tools on (`mod_jof`) and their [`CVAR`] on, the default,
//! medals are "Honors" and holocrons "Echoes" in every menu and message, on every
//! server. JoF asked for it, to keep their own names apart (Sol, 26/10/2026).
//!
//! Only the client's own words go through [`say`]: what players, the hub or a map
//! wrote (names, chat, achievements' hub text, "Holocron FFA", the lore holocrons in
//! shader names) keeps its words.

use std::borrow::Cow;
use std::sync::atomic::{AtomicBool, Ordering};

/// The JoF tools' setting, on by default (`sjk_mod_jof::HONOR_NAMES`).
pub(crate) const CVAR: &str = "jof.honorNames";

/// The words [`say`] changes, longest first so a plural is never read as the
/// singular, articles with the word they go before ("an Echo", "an Honor").
const WORDS: &[(&str, &str)] = &[
    ("HOLOCRONS", "ECHOES"),
    ("Holocrons", "Echoes"),
    ("holocrons", "echoes"),
    ("A HOLOCRON", "AN ECHO"),
    ("A Holocron", "An Echo"),
    ("A holocron", "An echo"),
    ("a Holocron", "an Echo"),
    ("a holocron", "an echo"),
    ("HOLOCRON", "ECHO"),
    ("Holocron", "Echo"),
    ("holocron", "echo"),
    ("MEDALS", "HONORS"),
    ("Medals", "Honors"),
    ("medals", "honors"),
    ("A MEDAL", "AN HONOR"),
    ("A Medal", "An Honor"),
    ("A medal", "An honor"),
    ("a Medal", "an Honor"),
    ("a medal", "an honor"),
    ("MEDAL", "HONOR"),
    ("Medal", "Honor"),
    ("medal", "honor"),
];

static RENAMED: AtomicBool = AtomicBool::new(false);

/// Follow `mod_jof` and [`CVAR`] (once a frame).
pub(crate) fn set_renamed(on: bool) {
    RENAMED.store(on, Ordering::Relaxed);
}

/// Whether medals and holocrons go by their JoF names.
pub(crate) fn renamed() -> bool {
    RENAMED.load(Ordering::Relaxed)
}

/// `text` as players read it: unchanged, or with the JoF names when [`renamed`].
pub(crate) fn say(text: &str) -> Cow<'_, str> {
    if renamed() {
        Cow::Owned(rename(text))
    } else {
        Cow::Borrowed(text)
    }
}

/// [`say`] for text already owned.
pub(crate) fn say_owned(text: String) -> String {
    if renamed() { rename(&text) } else { text }
}

/// [`say`] where only a `&'static str` fits (tables of hints and labels): each renamed
/// text is made once and kept for the rest of the run.
pub(crate) fn say_static(text: &'static str) -> &'static str {
    if !renamed() {
        return text;
    }
    static MADE: std::sync::Mutex<std::collections::BTreeMap<&str, &str>> =
        std::sync::Mutex::new(std::collections::BTreeMap::new());
    let mut made = MADE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    made.entry(text)
        .or_insert_with(|| Box::leak(rename(text).into_boxed_str()))
}

/// `text` with every whole word of [`WORDS`] changed ("Medallion" keeps its word).
fn rename(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut after_word = false;
    while let Some(c) = rest.chars().next() {
        let found = (!after_word)
            .then(|| {
                WORDS.iter().find(|(from, _)| {
                    rest.strip_prefix(from)
                        .is_some_and(|tail| !tail.starts_with(char::is_alphanumeric))
                })
            })
            .flatten();
        if let Some((from, to)) = found {
            out.push_str(to);
            rest = &rest[from.len()..];
            after_word = true;
        } else {
            out.push(c);
            rest = &rest[c.len_utf8()..];
            after_word = c.is_alphanumeric();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_renames_whole_words_with_their_articles() {
        assert_eq!(rename("Open 10 holocrons."), "Open 10 echoes.");
        assert_eq!(
            rename("You found a Rare Holocron."),
            "You found a Rare Echo."
        );
        assert_eq!(
            rename("Open a holocron on the Holocrons tab"),
            "Open an echo on the Echoes tab"
        );
        assert_eq!(
            rename("Receive a medal from the SJK team."),
            "Receive an honor from the SJK team."
        );
        assert_eq!(rename("NEW MEDAL"), "NEW HONOR");
        assert_eq!(rename("2 medals \u{b7} Medals"), "2 honors \u{b7} Honors");
        assert_eq!(
            rename("Medallion, holocronic, Bigmedal"),
            "Medallion, holocronic, Bigmedal"
        );
        assert_eq!(rename("A medal's blade"), "An honor's blade");
    }
}
