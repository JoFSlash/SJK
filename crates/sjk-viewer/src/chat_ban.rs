//! Ban 24 h (`docs/hub-chat.md`, "Chat bans"): a player the hub vouches for (verified
//! or staff) bans a player it does not vouch for from the SJK chat for 24 hours, from
//! the sender card or the SJK chat page's chosen message. No reason is asked and there
//! is no wait between bans; the hub bans the player's key and the address it last saw
//! them use ([`sjk_identity::ban`]).
//!
//! One click could ban a player by mistake, so the first click arms the button
//! ("Confirm ban") and only a second one on the same player within [`ARM`] sends it.
//! The state is the process's own: the sender cards of the game's chat, the docks and
//! the page show the same button.

use sjk_identity::BanOutcome;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// How long an armed button waits for the second click.
pub(crate) const ARM: Duration = Duration::from_secs(4);

/// The button's words.
pub(crate) const BAN: &str = "Ban 24 h";
pub(crate) const CONFIRM: &str = "Confirm ban";
pub(crate) const BANNING: &str = "Banning...";
/// The line under an armed button.
pub(crate) const ARMED_LINE: &str = "Click again: no SJK chat for them for 24 hours";

/// What a Ban button shows for a player.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Shown<'a> {
    /// The button's words.
    pub(crate) label: &'static str,
    /// The button waits for its second click.
    pub(crate) armed: bool,
    /// A line to show under it, and whether it is good news; `None`: none.
    pub(crate) line: Option<(&'a str, bool)>,
}

/// Which player is armed, which ban is on its way and what the last one came to.
#[derive(Debug, Default)]
pub(crate) struct Bans {
    armed: Option<(String, Instant)>,
    /// The key sent last and the ban outcomes' serial before it was sent.
    waiting: Option<(String, u64)>,
    /// The last answer: its key, its line and whether it banned.
    said: Option<(String, String, bool)>,
}

impl Bans {
    const fn new() -> Self {
        Self {
            armed: None,
            waiting: None,
            said: None,
        }
    }

    /// A click on `key_id`'s button at `now`, the outcomes' serial being `serial`:
    /// true when the ban should go now (the second click on an armed button).
    pub(crate) fn press(&mut self, key_id: &str, now: Instant, serial: u64) -> bool {
        if self.waiting.as_ref().is_some_and(|(key, _)| key == key_id) {
            return false;
        }
        let confirmed = self
            .armed
            .as_ref()
            .is_some_and(|(key, at)| key == key_id && now.duration_since(*at) <= ARM);
        if confirmed {
            self.armed = None;
            self.waiting = Some((key_id.to_owned(), serial));
            if self.said.as_ref().is_some_and(|(key, ..)| key == key_id) {
                self.said = None;
            }
        } else {
            self.armed = Some((key_id.to_owned(), now));
        }
        confirmed
    }

    /// Whether a ban is on its way, so the outcome needs reading.
    pub(crate) fn waiting(&self) -> bool {
        self.waiting.is_some()
    }

    /// Take in the service's last ban outcome, when it is the one waited for.
    pub(crate) fn answered(&mut self, outcome: &BanOutcome) {
        if let Some((_, after)) = &self.waiting
            && outcome.serial > *after
        {
            self.said = Some((
                outcome.key_id.clone(),
                outcome.line(),
                outcome.result.is_ok(),
            ));
            self.waiting = None;
        }
    }

    /// What `key_id`'s button shows at `now`.
    pub(crate) fn shown(&self, key_id: &str, now: Instant) -> Shown<'_> {
        if self.waiting.as_ref().is_some_and(|(key, _)| key == key_id) {
            return Shown {
                label: BANNING,
                armed: false,
                line: None,
            };
        }
        if self
            .armed
            .as_ref()
            .is_some_and(|(key, at)| key == key_id && now.duration_since(*at) <= ARM)
        {
            return Shown {
                label: CONFIRM,
                armed: true,
                line: Some((ARMED_LINE, false)),
            };
        }
        Shown {
            label: BAN,
            armed: false,
            line: self
                .said
                .as_ref()
                .filter(|(key, ..)| key == key_id)
                .map(|(_, line, banned)| (line.as_str(), *banned)),
        }
    }
}

static BANS: Mutex<Bans> = Mutex::new(Bans::new());

fn bans() -> std::sync::MutexGuard<'static, Bans> {
    BANS.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Whether the player's own key may ban this sender: the hub vouches for the player
/// and not for the sender, who has a key and is not the player. A trusted sender
/// (`trust`, the hub's green tick) only staff may ban.
pub(crate) fn offered(
    key_id: Option<&str>,
    verified: bool,
    staff: bool,
    trust: Option<sjk_identity::Trust>,
) -> bool {
    !verified
        && !staff
        && key_id.is_some_and(crate::player_identity::is_others_key)
        && trusted()
        && (trust != Some(sjk_identity::Trust::Trusted) || crate::player_identity::is_staff())
}

/// Whether the hub vouches for the player's own key.
fn trusted() -> bool {
    #[cfg(test)]
    if TRUSTED.with(std::cell::Cell::get) {
        return true;
    }
    crate::player_identity::can_ban()
}

#[cfg(test)]
thread_local! {
    static TRUSTED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
mod offered_tests {
    use sjk_identity::Trust;

    #[test]
    fn a_trusted_sender_is_not_offered_to_a_verified_player() {
        super::trust_for_test(true);
        let key = Some("0123456789abcdef");
        assert!(super::offered(key, false, false, None));
        assert!(super::offered(key, false, false, Some(Trust::New)));
        assert!(super::offered(key, false, false, Some(Trust::Banned)));
        // Only staff ban the trusted, and this test's player is not staff.
        assert!(!super::offered(key, false, false, Some(Trust::Trusted)));
        assert!(!super::offered(key, true, false, None));
        super::trust_for_test(false);
    }
}

/// Make this test thread's player one the hub vouches for, or not.
#[cfg(test)]
pub(crate) fn trust_for_test(on: bool) {
    TRUSTED.with(|trusted| trusted.set(on));
}

/// A click on `key_id`'s Ban button: arms it, or on the second click sends the ban.
pub(crate) fn press(key_id: &str) {
    // The serial is read before this module's lock is taken: the identity's lock is
    // never taken under it.
    let Some(serial) = crate::player_identity::with_chat(|chat| {
        chat.ban.as_ref().map_or(0, |outcome| outcome.serial)
    }) else {
        return;
    };
    let send = bans().press(key_id, Instant::now(), serial);
    if send {
        let _ = crate::player_identity::chat_ban(key_id.to_owned());
    }
}

/// Call `draw` with what `key_id`'s Ban button shows now, after taking in the answer
/// to a ban on its way.
pub(crate) fn with_shown<R>(key_id: &str, draw: impl FnOnce(Shown<'_>) -> R) -> R {
    let waiting = bans().waiting();
    if waiting
        && let Some(Some(outcome)) = crate::player_identity::with_chat(|chat| chat.ban.clone())
    {
        bans().answered(&outcome);
    }
    let bans = bans();
    draw(bans.shown(key_id, Instant::now()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjk_identity::{ChatBan, HubError};

    const TROLL: &str = "feedfacefeedface";

    #[test]
    fn the_first_click_arms_and_only_a_second_soon_after_sends() {
        let mut bans = Bans::default();
        let t0 = Instant::now();
        assert_eq!(bans.shown(TROLL, t0).label, BAN);
        assert!(!bans.press(TROLL, t0, 0), "the first click only arms");
        let armed = bans.shown(TROLL, t0);
        assert_eq!((armed.label, armed.armed), (CONFIRM, true));
        assert_eq!(armed.line, Some((ARMED_LINE, false)));
        // Another player's button is not armed by it.
        assert_eq!(bans.shown("0123456789abcdef", t0).label, BAN);
        // Too late: the click arms again rather than sending.
        let late = t0 + ARM + Duration::from_millis(1);
        assert_eq!(bans.shown(TROLL, late).label, BAN);
        assert!(!bans.press(TROLL, late, 0));
        assert!(bans.press(TROLL, late + Duration::from_secs(1), 0));
        assert_eq!(bans.shown(TROLL, late).label, BANNING);
        // On its way: more clicks send nothing more.
        assert!(!bans.press(TROLL, late, 0));
        assert!(!bans.press(TROLL, late, 0));
    }

    #[test]
    fn the_answer_shows_under_the_button_of_its_player() {
        let mut bans = Bans::default();
        let t0 = Instant::now();
        bans.press(TROLL, t0, 4);
        assert!(bans.press(TROLL, t0, 4));
        // An older outcome is not this ban's.
        bans.answered(&BanOutcome {
            serial: 4,
            key_id: TROLL.to_owned(),
            result: Err(HubError::Network("down".to_owned())),
        });
        assert!(bans.waiting());
        bans.answered(&BanOutcome {
            serial: 5,
            key_id: TROLL.to_owned(),
            result: Ok(ChatBan {
                until: 1_792_240_320,
                address: true,
            }),
        });
        assert!(!bans.waiting());
        let shown = bans.shown(TROLL, t0);
        assert_eq!(shown.label, BAN);
        let (line, banned) = shown.line.unwrap();
        assert!(line.starts_with("Banned from SJK chat until "), "{line}");
        assert!(banned);
        assert_eq!(bans.shown("0123456789abcdef", t0).line, None);
        // A refusal is worded for players.
        bans.press(TROLL, t0, 5);
        bans.press(TROLL, t0, 5);
        bans.answered(&BanOutcome {
            serial: 6,
            key_id: TROLL.to_owned(),
            result: Err(HubError::Rejected {
                status: 403,
                code: "target_verified".to_owned(),
                message: String::new(),
            }),
        });
        assert_eq!(
            bans.shown(TROLL, t0).line,
            Some(("Verified players and staff cannot be banned", false))
        );
    }
}
