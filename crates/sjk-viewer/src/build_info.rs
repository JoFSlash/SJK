//! Which build this is: its version, source commit and commit time, set at
//! compile time by `scripts/build_version.rs`. The version overlay, the menus'
//! version line and the startup notice show them.
//!
//! Dates read as SJK writes them for people: `dd/mm/yyyy` and a 24-hour time
//! (`docs/sjk.md`, "Dates and times").

use std::sync::OnceLock;

/// `0.1.0-alpha.3` in a release, the package version with `-dev` otherwise.
pub(crate) const VERSION: &str = env!("SJK_BUILD_VERSION");
/// Short hash of the source commit; empty when the build did not know it.
const COMMIT: &str = env!("SJK_BUILD_COMMIT");
/// Committer time of the source commit, ISO 8601; empty when unknown.
const COMMIT_TIME: &str = env!("SJK_BUILD_COMMIT_TIME");

/// `SJK <version> · <dd/mm/yyyy HH:MM> · <commit>`, built once.
pub(crate) fn label() -> &'static str {
    static LABEL: OnceLock<String> = OnceLock::new();
    LABEL.get_or_init(|| format_label(VERSION, COMMIT_TIME, COMMIT))
}

/// The `User-Agent` of SJK's requests to the hub: `SJK/<version> (<commit>)`, or
/// `SJK/<version>` when the build did not know its commit. The hub records it so staff
/// see which build a key plays with (the hub's `PROTOCOL.md`, "Client").
pub(crate) fn hub_agent() -> &'static str {
    static AGENT: OnceLock<String> = OnceLock::new();
    AGENT.get_or_init(|| format_agent(VERSION, COMMIT))
}

/// Whether this build is SJK `min` or newer, as the hub's minimum client counts it (the
/// hub's `PROTOCOL.md`, "Minimum client"): a dev build always is.
pub(crate) fn meets_min(min: &str) -> bool {
    version_meets(VERSION, min)
}

/// Whether `version` is `min` or newer: a dev build (`dev`, `-dev`) always is; release
/// versions compare number by number (`2026.1021.10` after `2026.1021.9`), trailing zeros
/// left out. A `min` that is no release version wants nothing; a `version` that is none
/// meets nothing, as the hub reads them.
fn version_meets(version: &str, min: &str) -> bool {
    fn numbers(text: &str) -> Option<Vec<u64>> {
        let mut numbers = text
            .trim()
            .split('.')
            .map(|part| {
                if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
                    return None;
                }
                part.parse().ok()
            })
            .collect::<Option<Vec<u64>>>()?;
        while numbers.last() == Some(&0) {
            numbers.pop();
        }
        Some(numbers)
    }
    if version == "dev" || version.ends_with("-dev") {
        return true;
    }
    let Some(min) = numbers(min) else {
        return true;
    };
    numbers(version).is_some_and(|version| version >= min)
}

fn format_agent(version: &str, commit: &str) -> String {
    if commit.is_empty() {
        format!("SJK/{version}")
    } else {
        format!("SJK/{version} ({commit})")
    }
}

/// The label of a build, leaving out a time or commit the build did not know.
fn format_label(version: &str, time: &str, commit: &str) -> String {
    let mut label = format!("SJK {version}");
    for part in [eu_date_time(time), Some(commit.to_owned())]
        .into_iter()
        .flatten()
        .filter(|part| !part.is_empty())
    {
        label.push_str(" \u{b7} ");
        label.push_str(&part);
    }
    label
}

/// `2026-10-05T19:56:05+02:00` as `05/10/2026 19:56`: the date and time as
/// written in the commit's own time zone, day first, on the 24-hour clock.
fn eu_date_time(iso: &str) -> Option<String> {
    let bytes = iso.as_bytes();
    let digits = |range: std::ops::Range<usize>| {
        let part = iso.get(range)?;
        part.bytes().all(|b| b.is_ascii_digit()).then_some(part)
    };
    let (year, month, day) = (digits(0..4)?, digits(5..7)?, digits(8..10)?);
    let (hour, minute) = (digits(11..13)?, digits(14..16)?);
    let separators = [(4, b'-'), (7, b'-'), (10, b'T'), (13, b':')];
    if separators
        .iter()
        .any(|&(index, byte)| bytes.get(index) != Some(&byte))
    {
        return None;
    }
    Some(format!("{day}/{month}/{year} {hour}:{minute}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hub_agent_names_the_commit_when_known() {
        assert_eq!(
            format_agent("2026.1017.2", "9dbbdf19"),
            "SJK/2026.1017.2 (9dbbdf19)"
        );
        assert_eq!(format_agent("0.1.0-dev", ""), "SJK/0.1.0-dev");
    }

    #[test]
    fn versions_meet_the_minimum_number_by_number() {
        assert!(version_meets("2026.1021.10", "2026.1021.9"));
        assert!(!version_meets("2026.1021.9", "2026.1021.10"));
        assert!(version_meets("2026.1022.1", "2026.1022.1"));
        assert!(version_meets("2026.1101.1", "2026.1031.3"));
        assert!(version_meets("2026.0901.1", "2026.901.1"));
        assert!(!version_meets("2026.1019.2", "2026.1022.1"));
        assert!(version_meets("dev", "2026.1022.1"));
        assert!(version_meets("0.1.0-dev", "2026.1022.1"));
        assert!(!version_meets("custom", "2026.1022.1"));
        assert!(version_meets("2026.1019.2", "soon"));
    }

    #[test]
    fn commit_time_reads_day_first_on_the_24_hour_clock() {
        assert_eq!(
            eu_date_time("2026-10-05T19:56:05+02:00").as_deref(),
            Some("05/10/2026 19:56")
        );
        assert_eq!(
            eu_date_time("2026-01-31T00:07:00Z").as_deref(),
            Some("31/01/2026 00:07")
        );
        for bad in [
            "",
            "yesterday",
            "2026-10-05",
            "2026/10/05T19:56",
            "2026-1x-05T19:56",
        ] {
            assert_eq!(eu_date_time(bad), None, "{bad}");
        }
    }

    #[test]
    fn label_leaves_out_what_the_build_did_not_know() {
        assert_eq!(
            format_label("2026.1005.1", "2026-10-05T19:56:05+02:00", "474caf6"),
            "SJK 2026.1005.1 \u{b7} 05/10/2026 19:56 \u{b7} 474caf6"
        );
        assert_eq!(format_label("dev", "", ""), "SJK dev");
        assert_eq!(format_label("dev", "", "474caf6"), "SJK dev \u{b7} 474caf6");
    }

    #[test]
    fn this_build_has_a_label() {
        assert!(label().starts_with(&format!("SJK {VERSION}")));
    }
}
