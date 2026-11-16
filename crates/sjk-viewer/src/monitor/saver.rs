//! The Screen Saver (Sol's idea, 29/10/2026): a player struck by [`HITS`] monitors of
//! other players within [`WINDOW_MS`] is untouchable by monitors for [`LASTS_MS`], and
//! still throws their own. A gold shell (Force Absorb's, gold) wraps them meanwhile, a
//! CRT's degauss is heard as it rises and its power-off as it ends.
//!
//! The struck player's own client counts the hits (it alone decides them) and raises
//! it; everyone else learns of it through a feed toy event the player sends
//! (`saver_<until>`, the game server time it ends, then `saver_off` if it ends sooner),
//! so their clients draw the shell and foresee no hit on them. It ends at its time, or
//! sooner with the player's death or the map.
//!
//! Two achievements come with it: Payback (strike the player whose monitor raised it,
//! while it holds) and the hidden Power Surge (strike three different players while it
//! holds), counted from the hits the struck players' own clients tell.

/// How many monitors must strike the player within [`WINDOW_MS`] to raise it.
pub(crate) const HITS: usize = 5;
/// The time those hits must fall within (game server milliseconds).
pub(crate) const WINDOW_MS: i64 = 20_000;
/// How long it holds (game server milliseconds).
pub(crate) const LASTS_MS: i64 = 30_000;
/// Its shell flickers this long before it ends.
const WARN_MS: i64 = 3_000;
/// The shell fades in over this long.
const FADE_IN_MS: i64 = 250;
/// One pulse of the shell, and one flicker as it is about to end.
const PULSE_MS: f32 = 900.0;
const FLICKER_MS: i64 = 70;
/// Players struck while it holds for Power Surge.
pub(crate) const SURGE_PLAYERS: u32 = 3;
/// The feed's events: `saver_<until>` and `saver_off`.
const PREFIX: &str = "saver_";
const OFF: &str = "saver_off";

/// The recent hits on the player that count towards it: when, and by which monitor
/// (thrower, number).
#[derive(Debug, Default)]
pub(super) struct Count {
    hits: Vec<(i64, u8, u8)>,
}

impl Count {
    /// Monitor `monitor` struck the player at `at`; whether that raises the Screen
    /// Saver. A monitor counts once however often it strikes (a body it juggles); the
    /// count starts afresh once it rises.
    pub(super) fn hit(&mut self, at: i64, monitor: (u8, u8)) -> bool {
        self.hits
            .retain(|(when, _, _)| *when <= at && at - when < WINDOW_MS);
        if !self
            .hits
            .iter()
            .any(|(_, thrower, number)| (*thrower, *number) == monitor)
        {
            self.hits.push((at, monitor.0, monitor.1));
        }
        if self.hits.len() >= HITS {
            self.hits.clear();
            return true;
        }
        false
    }

    pub(super) fn clear(&mut self) {
        self.hits.clear();
    }
}

/// The player's own Screen Saver: from when to when (game server milliseconds), whose
/// monitor raised it, whom the player's monitors struck since (bits by slot), and the
/// achievements it already counted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Own {
    pub(super) since: i64,
    pub(super) until: i64,
    pub(super) by: u8,
    struck: u64,
    paid_back: bool,
    surged: bool,
}

/// What a hit of the player's during their Screen Saver earned.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct Earned {
    pub(super) payback: bool,
    pub(super) surge: bool,
}

impl Own {
    pub(super) fn new(since: i64, by: u8) -> Self {
        Self {
            since,
            until: since + LASTS_MS,
            by,
            struck: 0,
            paid_back: false,
            surged: false,
        }
    }

    /// Whether it holds at `at`.
    pub(super) fn holds(&self, at: i64) -> bool {
        (self.since..self.until).contains(&at)
    }

    /// The player's monitor struck the player in `slot` at `at`: what that earns, each
    /// once a Screen Saver.
    pub(super) fn struck(&mut self, slot: u8, at: i64) -> Earned {
        if !self.holds(at) || usize::from(slot) >= 64 {
            return Earned::default();
        }
        self.struck |= 1 << slot;
        let payback = slot == self.by && !self.paid_back;
        let surge = self.struck.count_ones() >= SURGE_PLAYERS && !self.surged;
        self.paid_back |= payback;
        self.surged |= surge;
        Earned { payback, surge }
    }
}

/// The feed event telling others the player's Screen Saver holds until `until`, or
/// (`None`) ended.
pub(super) fn event(until: Option<i64>) -> String {
    until.map_or_else(|| OFF.to_owned(), |until| format!("{PREFIX}{until}"))
}

/// What a feed event of the toy says of its sender's Screen Saver: until when it holds,
/// or (`Some(None)`) that it ended; `None` for another event.
pub(super) fn parse(event: &str) -> Option<Option<i64>> {
    if event == OFF {
        return Some(None);
    }
    event.strip_prefix(PREFIX)?.parse().ok().map(Some)
}

/// How bright the gold shell is at `now`, for a Screen Saver from `since` to `until`
/// (0 to 1; 0 when it does not hold): it fades in, pulses gently, and flickers over
/// its last [`WARN_MS`].
pub(crate) fn glow(now: i64, since: i64, until: i64) -> f32 {
    if now < since || now >= until {
        return 0.0;
    }
    let left = until - now;
    if left < WARN_MS && (left / FLICKER_MS) % 2 == 1 {
        return 0.15;
    }
    let fade = ((now - since) as f32 / FADE_IN_MS as f32).min(1.0);
    let pulse = 0.8 + 0.2 * (std::f32::consts::TAU * (now - since) as f32 / PULSE_MS).cos();
    fade * pulse
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn five_monitors_within_twenty_seconds_raise_it() {
        let mut count = Count::default();
        for number in 0..4 {
            assert!(!count.hit(1_000 * i64::from(number), (3, number)));
        }
        assert!(count.hit(4_000, (4, 0)));
        // It starts afresh after.
        assert!(!count.hit(4_100, (4, 1)));
    }

    #[test]
    fn one_monitor_juggling_a_body_counts_once_and_old_hits_drop_out() {
        let mut count = Count::default();
        for at in 0..10 {
            assert!(!count.hit(at * 300, (3, 7)));
        }
        let mut count = Count::default();
        for number in 0..4 {
            assert!(!count.hit(0, (3, number)));
        }
        // 20 seconds on, the first four are gone.
        assert!(!count.hit(WINDOW_MS, (3, 9)));
        // A clock gone back (a new map) forgets the hits after it.
        let mut count = Count::default();
        for number in 0..4 {
            assert!(!count.hit(50_000, (3, number)));
        }
        assert!(!count.hit(1_000, (3, 9)));
    }

    #[test]
    fn payback_strikes_the_one_who_raised_it_and_a_surge_three_players() {
        let mut own = Own::new(10_000, 5);
        assert!(own.holds(10_000) && own.holds(10_000 + LASTS_MS - 1));
        assert!(!own.holds(9_999) && !own.holds(10_000 + LASTS_MS));
        assert_eq!(own.struck(2, 11_000), Earned::default());
        assert_eq!(
            own.struck(5, 12_000),
            Earned {
                payback: true,
                surge: false
            }
        );
        assert_eq!(own.struck(5, 12_500), Earned::default(), "once");
        assert_eq!(
            own.struck(9, 13_000),
            Earned {
                payback: false,
                surge: true
            }
        );
        assert_eq!(own.struck(11, 14_000), Earned::default(), "once");
        // After it ended, nothing.
        let mut own = Own::new(0, 5);
        assert_eq!(own.struck(5, LASTS_MS), Earned::default());
    }

    #[test]
    fn the_feed_says_until_when_it_holds_or_that_it_ended() {
        assert_eq!(parse(&event(Some(123_456))), Some(Some(123_456)));
        assert_eq!(parse(&event(None)), Some(None));
        let longest = event(Some(i64::from(i32::MAX)));
        assert!(longest.len() <= 32, "{longest}");
        for other in ["saver_", "saver_x", "throw_1", "hacked_5"] {
            assert_eq!(parse(other), None, "{other}");
        }
    }

    #[test]
    fn the_shell_fades_in_pulses_and_flickers_before_it_ends() {
        assert_eq!(glow(-1, 0, LASTS_MS), 0.0);
        assert_eq!(glow(LASTS_MS, 0, LASTS_MS), 0.0);
        assert_eq!(glow(0, 0, LASTS_MS), 0.0);
        let steady = glow(FADE_IN_MS, 0, LASTS_MS);
        assert!((0.6..=1.0).contains(&steady), "{steady}");
        let flickers: Vec<f32> = (0..6)
            .map(|step| glow(LASTS_MS - WARN_MS + step * FLICKER_MS, 0, LASTS_MS))
            .collect();
        assert!(flickers.iter().any(|glow| *glow < 0.2), "{flickers:?}");
        assert!(flickers.iter().any(|glow| *glow > 0.5), "{flickers:?}");
    }
}
