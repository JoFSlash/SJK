//! The Fishing and Mining toys' log book and their rare catches (Slash, 15/11/2026, after
//! adwè's #ideas post asking for more reason to fish and mine): the cosmetic half of
//! that plan, nothing of the economy.
//!
//! - **The log book.** The toy's card in Collection > Toys opens on its log book (the
//!   "Log book" button by the log's head): every fish and junk, or every ore and rubble,
//!   one to a cell, the ones never caught as "???", the others with their count and the
//!   day the first came up. The day is kept in the toy's own log, a `first <id>
//!   <yyyymmdd>` line (three words, which logs before it and older clients skip, as
//!   they do `sold`); catches from before it have no day.
//! - **Rare catches.** A rare fish ([`crate::fishing::is_rare`]) or ore
//!   ([`crate::mining::is_rare`]) brings the unlock pop-up's card ("RARE CATCH", "RARE
//!   FIND", [`crate::unlock_toast::Unlock::RareCatch`]) besides its centre line, with
//!   the single-player game's secret-area sound in the toy's own sounds.
//! - **Sharing.** `fishing_brag` and `mining_brag` (and the log book's Share button) say
//!   the last rare catch of the session, else the log's rarest, in the SJK chat as the
//!   player's own message ([`crate::player_identity::chat_sjk_only`]: the hub's chat
//!   rules and quota hold, and nothing goes to the game server's chat), at most once in
//!   [`BRAG_EVERY`]. Nothing is ever said for the player unasked.
//!
//! Everything here is the player's own log and words: the hub neither reads nor pays
//! for any of it.

use std::time::{Duration, Instant};

/// The least time between two shares of a toy's catch.
pub(crate) const BRAG_EVERY: Duration = Duration::from_secs(60);

/// The log's line of the day a catch first came up: `first <id> <yyyymmdd>`.
pub(crate) const FIRST: &str = "first";

/// Today on this PC, as the log keeps a day: `yyyymmdd`.
pub(crate) fn today() -> u32 {
    let now = sjk_shell::local_time::LocalTime::now();
    day_of(now.year, now.month, now.day)
}

/// The day `year`/`month`/`day` as the log keeps it.
pub(crate) fn day_of(year: i32, month: u8, day: u8) -> u32 {
    (year.max(0) as u32) * 10_000 + u32::from(month) * 100 + u32::from(day)
}

/// A kept day as the card writes it: `dd/mm/yyyy`; `None` for one that is no day.
pub(crate) fn day_text(day: u32) -> Option<String> {
    let (year, month, date) = (day / 10_000, day / 100 % 100, day % 100);
    ((1..=12).contains(&month) && (1..=31).contains(&date) && year >= 2_000)
        .then(|| format!("{date:02}/{month:02}/{year}"))
}

/// Read a `first <id> <yyyymmdd>` line's day; `None` for one that is no day.
pub(crate) fn parse_day(text: &str) -> Option<u32> {
    text.parse::<u32>()
        .ok()
        .filter(|day| day_text(*day).is_some())
}

/// `n` as an ordinal: 1st, 2nd, 3rd, 4th, 11th, 21st...
pub(crate) fn ordinal(n: u32) -> String {
    let suffix = match (n % 10, n % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

/// A catch's name without its article ("a Kyber minnow" is Kyber minnow, "the Heart of
/// the Mountain" Heart of the Mountain); what follows a comma stays ("Old Whiskers, the
/// great catfish").
pub(crate) fn bare_name(name: &str) -> &str {
    ["a ", "an ", "the "]
        .iter()
        .find_map(|article| name.strip_prefix(article))
        .unwrap_or(name)
}

/// The SJK chat line sharing a catch: `verb` (caught, dug out) the catch `name`, the
/// `count`th of it, with the log's kinds caught of all there are.
pub(crate) fn brag_line(verb: &str, name: &str, count: u32, kinds: (usize, usize)) -> String {
    let (caught, all) = kinds;
    let which = if count <= 1 {
        "my first!".to_owned()
    } else {
        format!("my {}", ordinal(count))
    };
    format!(
        "{verb} {} ({which}) - {caught} of {all} kinds in my log",
        bare_name(name)
    )
}

/// One cell of the log book.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Entry {
    /// Its name, without the article; "???" when never caught.
    pub(crate) name: String,
    /// How many were caught (0: never).
    pub(crate) count: u32,
    /// The day the first came up (`dd/mm/yyyy`), when the log knows it.
    pub(crate) first: Option<String>,
    /// A rare catch's cell, drawn in gold.
    pub(crate) rare: bool,
}

impl Entry {
    /// The cell of a catch named `name`, caught `count` times, the first on `first`.
    pub(crate) fn of(name: &str, count: u32, first: Option<u32>, rare: bool) -> Self {
        if count == 0 {
            return Self {
                name: "???".to_owned(),
                count,
                first: None,
                rare,
            };
        }
        Self {
            name: bare_name(name).split(',').next().unwrap_or(name).to_owned(),
            count,
            first: first.and_then(day_text),
            rare,
        }
    }

    /// The cell's second words: its count and first day ("x3 · 12/11/2026"), nothing
    /// for one never caught.
    pub(crate) fn note(&self) -> String {
        match (&self.first, self.count) {
            (_, 0) => String::new(),
            (Some(day), count) => format!("x{count}  {day}"),
            (None, count) => format!("x{count}"),
        }
    }
}

/// The log's first days, kept beside its counts: one `(id, yyyymmdd)` a catch.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Firsts(Vec<(String, u32)>);

impl Firsts {
    /// Read a `first <id> <day>` line's words, `id` and `day`.
    pub(crate) fn read(&mut self, id: &str, day: &str) {
        if let Some(day) = parse_day(day) {
            self.0.retain(|(known, _)| known != id);
            self.0.push((id.to_owned(), day));
        }
    }

    /// Note `day` as the first of `id` unless one is known.
    pub(crate) fn mark(&mut self, id: &str, day: u32) {
        if self.of(id).is_none() {
            self.0.push((id.to_owned(), day));
        }
    }

    /// Count in `other`'s days, where none is known: what was caught before the log was
    /// read.
    pub(crate) fn merge(&mut self, other: &Firsts) {
        for (id, day) in &other.0 {
            self.mark(id, *day);
        }
    }

    /// The first day of `id`, when known.
    pub(crate) fn of(&self, id: &str) -> Option<u32> {
        self.0
            .iter()
            .find(|(known, _)| known == id)
            .map(|(_, day)| *day)
    }

    /// The lines to write: `first <id> <day>` each.
    pub(crate) fn push_text(&self, text: &mut String) {
        for (id, day) in &self.0 {
            text.push_str(&format!("{FIRST} {id} {day}\n"));
        }
    }
}

/// When a toy's catch was last shared, and the last rare catch of the session.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Brag {
    pub(crate) at: Option<Instant>,
}

impl Brag {
    /// Whether a share may go now (none in [`BRAG_EVERY`]); notes it when it may.
    pub(crate) fn take(&mut self, now: Instant) -> bool {
        if self
            .at
            .is_some_and(|at| now.duration_since(at) < BRAG_EVERY)
        {
            return false;
        }
        self.at = Some(now);
        true
    }
}

/// Send `line` to the SJK chat as the player's own message, and say what came of it on
/// the centre of the screen: `None` when it went, else why not.
pub(crate) fn share(line: String) -> Option<&'static str> {
    if crate::player_identity::chat_sjk_only(line) {
        None
    } else {
        Some("SJK online is off: the SJK chat shares your catch.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_day_is_kept_and_written_day_first() {
        assert_eq!(day_of(2026, 11, 5), 20261105);
        assert_eq!(day_text(20261105).as_deref(), Some("05/11/2026"));
        assert_eq!(parse_day("20261105"), Some(20261105));
        assert_eq!(parse_day("2026115"), None, "no month 1 day 15 of year 202");
        assert_eq!(parse_day("20261341"), None);
        assert_eq!(parse_day("x"), None);
    }

    #[test]
    fn ordinals_read_as_english() {
        let words: Vec<String> = [1, 2, 3, 4, 11, 12, 13, 21, 22, 101, 111]
            .into_iter()
            .map(ordinal)
            .collect();
        assert_eq!(
            words,
            [
                "1st", "2nd", "3rd", "4th", "11th", "12th", "13th", "21st", "22nd", "101st",
                "111th"
            ]
        );
    }

    #[test]
    fn a_share_names_the_catch_and_the_log() {
        assert_eq!(
            brag_line("Caught", "Old Whiskers, the great catfish", 1, (9, 12)),
            "Caught Old Whiskers, the great catfish (my first!) - 9 of 12 kinds in my log"
        );
        assert_eq!(
            brag_line("Dug out", "the Heart of the Mountain", 3, (9, 9)),
            "Dug out Heart of the Mountain (my 3rd) - 9 of 9 kinds in my log"
        );
        // The hub's chat takes it: one line, its alphabet, under 300 characters.
        let line = brag_line("Caught", "a Kyber minnow", 22, (12, 12));
        assert!(sjk_identity::chat::check(&line).is_ok(), "{line}");
    }

    #[test]
    fn a_cell_hides_what_was_never_caught() {
        let never = Entry::of("a Ghost ray", 0, Some(20261101), true);
        assert_eq!(never.name, "???");
        assert!(never.note().is_empty() && never.first.is_none());
        let caught = Entry::of("Old Whiskers, the great catfish", 2, Some(20261112), true);
        assert_eq!(caught.name, "Old Whiskers");
        assert_eq!(caught.note(), "x2  12/11/2026");
        assert_eq!(Entry::of("a Sun carp", 4, None, false).note(), "x4");
    }

    #[test]
    fn the_first_day_is_kept_once() {
        let mut firsts = Firsts::default();
        firsts.mark("sun_carp", 20261101);
        firsts.mark("sun_carp", 20261115);
        firsts.read("moon_eel", "20261102");
        firsts.read("moon_eel", "nonsense");
        assert_eq!(firsts.of("sun_carp"), Some(20261101));
        assert_eq!(firsts.of("moon_eel"), Some(20261102));
        let mut text = String::new();
        firsts.push_text(&mut text);
        assert_eq!(text, "first sun_carp 20261101\nfirst moon_eel 20261102\n");
    }

    #[test]
    fn a_share_waits_a_minute_after_the_last() {
        let mut brag = Brag::default();
        let now = Instant::now();
        assert!(brag.take(now));
        assert!(!brag.take(now + Duration::from_secs(30)));
        assert!(brag.take(now + BRAG_EVERY));
    }
}
