//! Which of the player's own medals the new medal pop-up has shown (`medal_popup.rs`):
//! `medals_seen.txt` in the settings folder, beside `identity.key`. A medal shows once,
//! a repeatable one again each time its count rises, and any again once as a promotion
//! each time its rank rises (`rank.rs`). The file names the key it belongs to, so a new
//! identity sees its own medals; a missing file means nothing was shown, so an install
//! that already holds medals when it first reads its profile still shows them.
//!
//! ```text
//! # Medals SJK has shown this identity: "<id> <count>" a line.
//! key 0123456789abcdef
//! early_tester 1
//! early_tester.rank 3
//! bug_hunter 2
//! ```
//!
//! The rank shown is a line of its own, `<id>.rank <rank>`, written only above rank I:
//! a medal shown before ranks (27/10/2026) has none and was seen at rank I. Lines for ids
//! this build does not know are kept as they are, so a newer build's medals are not shown
//! twice after switching builds; an older build keeps the rank lines the same way.

use super::{Award, Medal, Rank};
use std::path::Path;

/// The file, in the settings folder.
pub(crate) const FILE: &str = "medals_seen.txt";
const HEADER: &str = "# Medals SJK has shown this identity: \"<id> <count>\" a line.";
/// What follows a medal's id on the line of the rank shown.
const RANK_SUFFIX: &str = ".rank";

/// The counts shown so far, for one key.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Seen {
    key_id: String,
    /// The count last shown of each known medal; 0 when never shown.
    counts: [u32; Medal::COUNT],
    /// The rank last shown of each; Issued when never shown or shown before ranks.
    ranks: [Rank; Medal::COUNT],
    /// Lines naming ids this build does not know, kept for a newer one.
    unknown: Vec<String>,
}

impl Seen {
    /// What `text` says was shown to `key_id`; nothing when it is another key's.
    pub(crate) fn parse(text: &str, key_id: &str) -> Self {
        let mut seen = Self {
            key_id: key_id.to_owned(),
            ..Self::default()
        };
        let mut lines = text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'));
        if lines.next().and_then(|line| line.strip_prefix("key ")) != Some(key_id) {
            return seen;
        }
        for line in lines {
            let mut words = line.split_whitespace();
            let (Some(id), Some(count), None) = (words.next(), words.next(), words.next()) else {
                continue;
            };
            let Ok(count) = count.parse::<u32>() else {
                continue;
            };
            let ranked = id.strip_suffix(RANK_SUFFIX);
            match (Medal::from_id(ranked.unwrap_or(id)), ranked) {
                (Some(medal), Some(_)) => {
                    seen.ranks[medal.index()] = Rank::from_wire(count.min(5) as u8);
                }
                (Some(medal), None) => seen.counts[medal.index()] = count,
                (None, _) => seen.unknown.push(format!("{id} {count}")),
            }
        }
        seen
    }

    /// The file `directory` holds for `key_id`; nothing shown when there is none.
    pub(crate) fn load(directory: &Path, key_id: &str) -> Self {
        let text = std::fs::read_to_string(directory.join(FILE)).unwrap_or_default();
        Self::parse(&text, key_id)
    }

    /// The key these counts belong to.
    pub(crate) fn key_id(&self) -> &str {
        &self.key_id
    }

    /// The file's text.
    pub(crate) fn text(&self) -> String {
        let mut text = format!("{HEADER}\nkey {}\n", self.key_id);
        for medal in Medal::ALL {
            let count = self.counts[medal.index()];
            if count > 0 {
                text.push_str(&format!("{} {count}\n", medal.id()));
            }
            let rank = self.ranks[medal.index()];
            if rank > Rank::Issued {
                text.push_str(&format!("{}{RANK_SUFFIX} {}\n", medal.id(), rank.number()));
            }
        }
        for line in &self.unknown {
            text.push_str(line);
            text.push('\n');
        }
        text
    }

    /// Write the file into `directory`.
    pub(crate) fn save(&self, directory: &Path) -> std::io::Result<()> {
        std::fs::write(directory.join(FILE), self.text())
    }

    /// Whether `award` is new: never shown, given again since, or promoted since.
    pub(crate) fn is_new(&self, award: &Award) -> bool {
        award.count > self.counts[award.medal.index()] || self.is_promotion(award)
    }

    /// Whether `award` was shown already and only its rank rose since: the pop-up
    /// says it was promoted.
    pub(crate) fn is_promotion(&self, award: &Award) -> bool {
        let index = award.medal.index();
        award.count <= self.counts[index] && award.rank > self.ranks[index]
    }

    /// Count `award` as shown, at its rank.
    pub(crate) fn mark(&mut self, award: &Award) {
        let index = award.medal.index();
        self.counts[index] = self.counts[index].max(award.count);
        self.ranks[index] = self.ranks[index].max(award.rank);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn award(medal: Medal, count: u32) -> Award {
        Award {
            medal,
            count,
            awarded: 0,
            note: String::new(),
            rank: Rank::Issued,
        }
    }

    fn ranked(medal: Medal, rank: Rank) -> Award {
        Award {
            rank,
            ..award(medal, 1)
        }
    }

    #[test]
    fn a_rank_rising_shows_once_as_a_promotion() {
        let mut seen = Seen::parse("", "aa");
        let holo = ranked(Medal::Tester, Rank::Holo);
        assert!(seen.is_new(&holo));
        assert!(!seen.is_promotion(&holo), "never shown: new, not promoted");
        seen.mark(&ranked(Medal::Tester, Rank::Polished));
        assert!(seen.is_new(&holo) && seen.is_promotion(&holo));
        seen.mark(&holo);
        assert!(!seen.is_new(&holo));
        assert!(
            !seen.is_new(&ranked(Medal::Tester, Rank::Engraved)),
            "lowered"
        );
        let solar = ranked(Medal::Tester, Rank::Solar);
        assert!(seen.is_promotion(&solar));
        let again = Seen::parse(&seen.text(), "aa");
        assert_eq!(again, seen);
        assert!(seen.text().contains("early_tester.rank 4\n"));
    }

    #[test]
    fn a_file_from_before_ranks_saw_rank_one() {
        // Written before 27/10/2026: no rank lines.
        let seen = Seen::parse("# x\nkey aa\nearly_tester 1\nbug_hunter 1\n", "aa");
        assert!(!seen.is_new(&ranked(Medal::Tester, Rank::Issued)));
        assert!(seen.is_promotion(&ranked(Medal::Tester, Rank::Engraved)));
        assert!(!seen.text().contains(".rank"), "rank I is not written");
        // A newer build's rank line for an id this one does not know is kept.
        let newer = Seen::parse("key aa\nfrom_the_future.rank 5\n", "aa");
        assert!(newer.text().contains("from_the_future.rank 5\n"));
        // A rank out of range is read into it.
        let high = Seen::parse("key aa\nearly_tester 1\nearly_tester.rank 9\n", "aa");
        assert!(!high.is_new(&ranked(Medal::Tester, Rank::Solar)));
    }

    #[test]
    fn a_medal_is_new_until_shown_and_again_when_its_count_rises() {
        let mut seen = Seen::parse("", "aa");
        let tester = award(Medal::Tester, 1);
        assert!(seen.is_new(&tester), "no file: everything held is new");
        seen.mark(&tester);
        assert!(!seen.is_new(&tester));
        seen.mark(&award(Medal::BugHunter, 2));
        assert!(!seen.is_new(&award(Medal::BugHunter, 2)));
        assert!(seen.is_new(&award(Medal::BugHunter, 3)));
    }

    #[test]
    fn the_file_round_trips_keeps_unknown_ids_and_belongs_to_one_key() {
        let text = "# x\nkey aa\nearly_tester 1\nfrom_the_future 4\nbug_hunter 2\nbroken\n";
        let seen = Seen::parse(text, "aa");
        assert!(!seen.is_new(&award(Medal::Tester, 1)));
        assert!(!seen.is_new(&award(Medal::BugHunter, 2)));
        assert!(seen.is_new(&award(Medal::Contributor, 1)));
        let again = Seen::parse(&seen.text(), "aa");
        assert_eq!(again, seen);
        assert!(seen.text().contains("from_the_future 4"));
        let other = Seen::parse(text, "bb");
        assert!(other.is_new(&award(Medal::Tester, 1)), "another key's file");
        assert_eq!(other.key_id(), "bb");
    }

    #[test]
    fn the_file_is_saved_in_the_settings_folder() {
        let directory = tempfile::tempdir().expect("a folder");
        let mut seen = Seen::load(directory.path(), "aa");
        seen.mark(&award(Medal::Contributor, 1));
        seen.save(directory.path()).expect("saved");
        let loaded = Seen::load(directory.path(), "aa");
        assert!(!loaded.is_new(&award(Medal::Contributor, 1)));
    }
}
