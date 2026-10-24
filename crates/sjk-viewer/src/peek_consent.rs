//! Asking before a peek (`docs/client.md`, "Timed player peek"): a player who runs SJK
//! online is asked first and sees who watches them. The question and the answers ride
//! the hub's toy events (`docs/hub-chat.md`, "Toy events"): toy [`TOY`], event
//! `request_<milliseconds>` from the watcher, [`ALLOW`] or [`DENY`] back, and [`END`]
//! from either side when they stop early. Both sides keep their own timers in their
//! own snapshots' server time, so no event has to say when a peek is over.
//!
//! This is the pure state; `peek.rs` sends the events and moves the camera.

/// `cl_peekRequests`: whether other players may ask to watch this one (0 answers every
/// request no at once, unseen).
pub(crate) const CVAR: &str = "cl_peekRequests";
/// The toy id of peek events.
pub(crate) const TOY: &str = "peek";
/// A request's event, before its length in milliseconds.
const REQUEST: &str = "request_";
/// The asked player lets the sender watch.
pub(crate) const ALLOW: &str = "allow";
/// The asked player says no.
pub(crate) const DENY: &str = "deny";
/// Either side stops: the watcher returns, or the watched player ends it.
pub(crate) const END: &str = "end";
/// How long a request waits for its answer, in server milliseconds.
pub(crate) const ANSWER_WINDOW: i64 = 15_000;
/// A sender's next request is heard only this long after their last.
const REPEAT: i64 = 10_000;
/// The longest peek, as `/peek` clamps it.
const LONGEST: i64 = 60_000;
/// Slack on the watched side's timer, for the two clients' snapshots arriving apart.
const SLACK: i64 = 1_000;

/// The event that asks to watch for `milliseconds`.
pub(crate) fn request_event(milliseconds: i64) -> String {
    format!("{REQUEST}{milliseconds}")
}

/// The length a request event asks for, if it is one and in range.
fn request_length(event: &str) -> Option<i64> {
    let milliseconds = event.strip_prefix(REQUEST)?.parse::<i64>().ok()?;
    (1..=LONGEST)
        .contains(&milliseconds)
        .then_some(milliseconds)
}

/// What becomes of a request from a sender, by the player's choice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Gate {
    /// Show the question.
    Ask,
    /// Answer no at once: the player takes no requests (`cl_peekRequests 0`).
    Refuse,
    /// Say nothing, so the request lapses: the sender is muted.
    Ignore,
}

/// The other player of a question or a peek.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Party {
    pub(crate) slot: u16,
    /// Their name as the game shows it, colours kept.
    pub(crate) name: String,
    /// The peek's length.
    pub(crate) milliseconds: i64,
    /// When the question lapses, or the watching ends (server time).
    pub(crate) until: i64,
}

/// What an event from another player changed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Heard {
    /// Someone asks to watch the local player; the card shows the question.
    Request,
    /// A request the player takes none of: answer it with [`DENY`].
    Refused,
    /// The asked player allowed: watch them for this long.
    Allowed(Party),
    /// The asked player said no.
    Denied(Party),
    /// `slot` stopped: the watched player ended the peek, or the watcher (or asker)
    /// left. The caller ends its own peek of `slot` if it was asked for.
    Ended {
        slot: u16,
        /// The watcher or asker who left, if one was.
        watcher: Option<Party>,
    },
}

#[derive(Default)]
pub(crate) struct Consent {
    /// The player asked to be watched, waiting for their answer.
    pub(crate) asking: Option<Party>,
    /// Someone's request to watch the local player, waiting for an answer.
    pub(crate) request: Option<Party>,
    /// The player allowed to watch the local player now.
    pub(crate) watcher: Option<Party>,
    /// When each slot last asked, against a stream of requests.
    last_request: Vec<(u16, i64)>,
}

impl Consent {
    /// Forget everything: another server, map or game.
    pub(crate) fn clear(&mut self) {
        *self = Self::default();
    }

    /// Wait for `slot`'s answer to watching them for `milliseconds`.
    pub(crate) fn ask(&mut self, slot: u16, name: String, milliseconds: i64, now: i64) {
        self.asking = Some(Party {
            slot,
            name,
            milliseconds,
            until: now + ANSWER_WINDOW,
        });
    }

    /// An event from `sender` (shown as `name`) to the local player; `gate` is what
    /// becomes of a request from them. A request counts once in [`REPEAT`] from a sender
    /// (each one, heard or not, starts the wait again), and is shown only while nobody
    /// else asks or watches.
    pub(crate) fn hear(
        &mut self,
        sender: u16,
        name: &str,
        event: &str,
        now: i64,
        gate: Gate,
    ) -> Option<Heard> {
        if let Some(milliseconds) = request_length(event) {
            let recent = self
                .last_request
                .iter()
                .any(|(slot, at)| *slot == sender && now - at < REPEAT && now >= *at);
            self.last_request.retain(|(slot, _)| *slot != sender);
            self.last_request.push((sender, now));
            if recent {
                return None;
            }
            match gate {
                Gate::Ask => {}
                Gate::Refuse => return Some(Heard::Refused),
                Gate::Ignore => return None,
            }
            if self.request.is_some() || self.watcher.is_some() {
                return None;
            }
            self.request = Some(Party {
                slot: sender,
                name: name.to_owned(),
                milliseconds,
                until: now + ANSWER_WINDOW,
            });
            return Some(Heard::Request);
        }
        match event {
            ALLOW | DENY => {
                let asking = self.asking.take_if(|party| party.slot == sender)?;
                Some(if event == ALLOW {
                    Heard::Allowed(asking)
                } else {
                    Heard::Denied(asking)
                })
            }
            END => {
                let watcher = self
                    .watcher
                    .take_if(|party| party.slot == sender)
                    .or_else(|| self.request.take_if(|party| party.slot == sender));
                Some(Heard::Ended {
                    slot: sender,
                    watcher,
                })
            }
            _ => None,
        }
    }

    /// Answer the request on show: the event to send its sender, and the sender. An
    /// allowed sender is the watcher until their peek is over.
    pub(crate) fn answer(&mut self, allow: bool, now: i64) -> Option<(&'static str, Party)> {
        let request = self.request.take()?;
        if allow {
            self.watcher = Some(Party {
                until: now + request.milliseconds + SLACK,
                ..request.clone()
            });
            Some((ALLOW, request))
        } else {
            Some((DENY, request))
        }
    }

    /// Put away what is over at `now` (or from before a server time rewound past it):
    /// the asked player who never answered is returned.
    pub(crate) fn expire(&mut self, now: i64) -> Option<Party> {
        let over = |party: &mut Party| now >= party.until || party.until - now > LONGEST * 2;
        self.request.take_if(over);
        self.watcher.take_if(over);
        self.last_request
            .retain(|(_, at)| now >= *at && now - at < REPEAT);
        self.asking.take_if(over)
    }

    /// Drop the parties whose slot no longer shows the name they had.
    pub(crate) fn keep_present(&mut self, shows: impl Fn(u16, &str) -> bool) {
        let gone = |party: &mut Party| !shows(party.slot, &party.name);
        self.asking.take_if(gone);
        self.request.take_if(gone);
        self.watcher.take_if(gone);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_events_carry_a_bounded_length() {
        assert_eq!(request_event(60_000), "request_60000");
        assert_eq!(request_length("request_5000"), Some(5000));
        assert_eq!(request_length("request_0"), None);
        assert_eq!(request_length("request_60001"), None);
        assert_eq!(request_length("request_x"), None);
        assert_eq!(request_length("allow"), None);
    }

    #[test]
    fn an_allowed_request_makes_a_watcher_until_the_peek_is_over() {
        let mut consent = Consent::default();
        assert_eq!(
            consent.hear(3, "Tox", "request_5000", 1000, Gate::Ask),
            Some(Heard::Request)
        );
        // Another request waits for nobody: one question at a time.
        assert_eq!(
            consent.hear(4, "Sol", "request_5000", 1100, Gate::Ask),
            None
        );
        let (event, sender) = consent.answer(true, 2000).unwrap();
        assert_eq!((event, sender.slot), (ALLOW, 3));
        assert_eq!(consent.watcher.as_ref().unwrap().until, 8000);
        assert!(consent.request.is_none());
        // Nobody else is heard while someone watches.
        assert_eq!(
            consent.hear(4, "Sol", "request_5000", 3000, Gate::Ask),
            None
        );
        assert_eq!(consent.expire(7999), None);
        assert!(consent.watcher.is_some());
        consent.expire(8000);
        assert!(consent.watcher.is_none());
    }

    #[test]
    fn a_denied_or_unanswered_request_lapses_and_repeats_wait() {
        let mut consent = Consent::default();
        consent.hear(3, "Tox", "request_5000", 1000, Gate::Ask);
        assert_eq!(consent.answer(false, 1500).unwrap().0, DENY);
        assert!(consent.watcher.is_none());
        // The same sender again too soon is not heard; later it is.
        assert_eq!(
            consent.hear(3, "Tox", "request_5000", 5000, Gate::Ask),
            None
        );
        assert_eq!(
            consent.hear(3, "Tox", "request_5000", 16_000, Gate::Ask),
            Some(Heard::Request)
        );
        consent.expire(16_000 + ANSWER_WINDOW);
        assert!(consent.request.is_none());
    }

    #[test]
    fn answers_count_only_from_the_player_asked() {
        let mut consent = Consent::default();
        consent.ask(5, "Tox".into(), 30_000, 1000);
        assert_eq!(consent.hear(6, "Sol", ALLOW, 1200, Gate::Ask), None);
        let Some(Heard::Allowed(party)) = consent.hear(5, "Tox", ALLOW, 1300, Gate::Ask) else {
            panic!("the asked player allowed");
        };
        assert_eq!((party.slot, party.milliseconds), (5, 30_000));
        assert_eq!(
            consent.hear(5, "Tox", ALLOW, 1400, Gate::Ask),
            None,
            "answered once"
        );
        consent.ask(5, "Tox".into(), 30_000, 2000);
        assert!(matches!(
            consent.hear(5, "Tox", DENY, 2100, Gate::Ask),
            Some(Heard::Denied(_))
        ));
        consent.ask(5, "Tox".into(), 30_000, 3000);
        assert_eq!(consent.expire(3000 + ANSWER_WINDOW).unwrap().slot, 5);
    }

    #[test]
    fn an_end_from_the_watcher_or_asker_puts_them_away() {
        let mut consent = Consent::default();
        consent.hear(3, "Tox", "request_5000", 1000, Gate::Ask);
        let Some(Heard::Ended { slot: 3, watcher }) = consent.hear(3, "Tox", END, 1100, Gate::Ask)
        else {
            panic!("ended");
        };
        assert_eq!(watcher.unwrap().slot, 3, "the request was called off");
        consent.hear(3, "Tox", "request_5000", 20_000, Gate::Ask);
        consent.answer(true, 20_100);
        let Some(Heard::Ended { watcher, .. }) = consent.hear(3, "Tox", END, 21_000, Gate::Ask)
        else {
            panic!("ended");
        };
        assert!(watcher.is_some() && consent.watcher.is_none());
        // An end from someone else touches nothing here; the caller checks its peek.
        assert_eq!(
            consent.hear(9, "Sol", END, 21_100, Gate::Ask),
            Some(Heard::Ended {
                slot: 9,
                watcher: None
            })
        );
    }

    #[test]
    fn refused_and_ignored_requests_show_nothing() {
        let mut consent = Consent::default();
        assert_eq!(
            consent.hear(3, "Tox", "request_5000", 0, Gate::Refuse),
            Some(Heard::Refused)
        );
        // A refusal is sent once in the wait, however often they ask.
        assert_eq!(
            consent.hear(3, "Tox", "request_5000", 500, Gate::Refuse),
            None
        );
        assert_eq!(
            consent.hear(4, "Sol", "request_5000", 0, Gate::Ignore),
            None
        );
        assert!(consent.request.is_none());
        // Asking again within the wait starts it again.
        assert_eq!(
            consent.hear(4, "Sol", "request_5000", 9_000, Gate::Ask),
            None
        );
        assert_eq!(
            consent.hear(4, "Sol", "request_5000", 18_000, Gate::Ask),
            None
        );
        assert_eq!(
            consent.hear(4, "Sol", "request_5000", 28_001, Gate::Ask),
            Some(Heard::Request)
        );
    }

    #[test]
    fn parties_leave_with_their_slot_name() {
        let mut consent = Consent::default();
        consent.ask(5, "Tox".into(), 5000, 0);
        consent.hear(3, "Sol", "request_5000", 0, Gate::Ask);
        consent.keep_present(|slot, name| slot == 3 && name == "Sol");
        assert!(consent.asking.is_none());
        assert!(consent.request.is_some());
    }
}
