//! Trying a shader (`PROTOCOL.md` "Trying a shader", since 04/11/2026, Blep's idea in
//! #ideas): the player may wear a blade skin they do not hold for a short while (the hub
//! says how long, ten minutes at first), which every SJK player on their server sees.
//! When the time is up the hub takes it off their look, and that skin cannot be tried
//! again by them for a cooldown (a day at first). One trial runs at a time. The hub
//! decides every step, behind the operator's switch.
//!
//! `GET /v1/shaders/try` says whether this key may try shaders, the trial running and the
//! skins cooling down; `POST /v1/shaders/try` starts a trial.

use crate::hub::HubError;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

/// A trial running, as the hub says it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct Trial {
    /// The blade skin worn.
    pub blade: String,
    /// When it ends, in the hub's unix seconds.
    pub ends: i64,
}

/// `GET /v1/shaders/try`'s answer.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct TryStatus {
    /// Whether the hub lets this key try shaders now (the operator's switch).
    #[serde(default)]
    pub open: bool,
    /// How long a trial lasts, in seconds.
    #[serde(default)]
    pub secs: i64,
    /// How long a skin cools down after its trial, in seconds.
    #[serde(default)]
    pub cooldown: i64,
    /// The hub's clock when it answered, in unix seconds: the times below count from it.
    #[serde(default)]
    pub now: i64,
    /// The trial running, if one is.
    #[serde(default)]
    pub trial: Option<Trial>,
    /// The skins that cannot be tried again yet, each with the hub's unix seconds from
    /// which it may.
    #[serde(default)]
    pub cooling: BTreeMap<String, i64>,
}

/// `POST /v1/shaders/try`'s answer.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct Started {
    /// The blade skin now worn.
    pub blade: String,
    /// When the trial started, in the hub's unix seconds.
    pub started: i64,
    /// When it ends, in the hub's unix seconds.
    pub ends: i64,
}

/// What the client knows of shader trials: the hub's last answer and when it came, so
/// times read on this PC's clock.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trials {
    /// The hub's last answer.
    pub status: TryStatus,
    /// When [`Self::status`] arrived.
    pub read: Instant,
}

impl Trials {
    /// The hub's unix seconds at `now`.
    fn hub_now(&self, now: Instant) -> i64 {
        let passed = now.saturating_duration_since(self.read).as_secs();
        self.status
            .now
            .saturating_add(i64::try_from(passed).unwrap_or(i64::MAX))
    }

    /// The skin a trial lets the player wear at `now`, and how long it still runs.
    pub fn running(&self, now: Instant) -> Option<(&str, Duration)> {
        let trial = self.status.trial.as_ref()?;
        let left = trial.ends - self.hub_now(now);
        (left > 0).then(|| {
            (
                trial.blade.as_str(),
                Duration::from_secs(left.unsigned_abs()),
            )
        })
    }

    /// Whether a trial lets the player wear `blade` at `now`.
    pub fn wears(&self, blade: &str, now: Instant) -> bool {
        self.running(now).is_some_and(|(worn, _)| worn == blade)
    }

    /// How long until `blade` may be tried again at `now`: its cooldown, or the running
    /// trial's time and then its cooldown when that is the skin running.
    pub fn cooling(&self, blade: &str, now: Instant) -> Option<Duration> {
        let hub_now = self.hub_now(now);
        let until = match self.status.trial.as_ref() {
            Some(trial) if trial.blade == blade => trial.ends + self.status.cooldown,
            _ => *self.status.cooling.get(blade)?,
        };
        let left = until - hub_now;
        (left > 0).then(|| Duration::from_secs(left.unsigned_abs()))
    }

    /// Note a trial just started (`POST /v1/shaders/try`'s answer) at `now`.
    pub fn started(&mut self, started: &Started, now: Instant) {
        self.status.now = started.started;
        self.read = now;
        self.status.trial = Some(Trial {
            blade: started.blade.clone(),
            ends: started.ends,
        });
    }
}

/// The body of `POST /v1/shaders/try` for `blade`.
pub fn start_body(blade: &str) -> serde_json::Value {
    serde_json::json!({ "blade": blade })
}

/// A short time left, as players read it: "9:05", "23 h", "45 min".
pub fn time_left(left: Duration) -> String {
    let secs = left.as_secs();
    if secs >= 3_600 {
        format!("{} h", secs.div_ceil(3_600))
    } else if secs >= 600 {
        format!("{} min", secs.div_ceil(60))
    } else {
        format!("{}:{:02}", secs / 60, secs % 60)
    }
}

/// Why a trial could not start, in the player's words.
pub fn refusal(error: &HubError) -> String {
    let words = match error {
        HubError::Rejected { code, message, .. } => match code.as_str() {
            "shader_try_off" => "The hub does not let you try shaders yet.",
            "try_running" => "Another shader trial still runs: wait for it to end.",
            "try_cooldown" => "You tried that shader lately: try it again later.",
            "already_unlocked" => "You have that shader already.",
            "bad_blade" => "That shader cannot be tried.",
            _ => return message.clone(),
        },
        other if unsupported(other) => "This SJK hub does not let you try shaders yet.",
        other => return other.to_string(),
    };
    words.to_owned()
}

/// Whether `error` means the hub has no shader trials (an older hub): the client then
/// shows no Try button.
pub fn unsupported(error: &HubError) -> bool {
    matches!(error, HubError::Protocol(why)
        if why.starts_with("status 404") || why.starts_with("status 405"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status(text: &str) -> TryStatus {
        serde_json::from_str(text).unwrap()
    }

    #[test]
    fn answers_read_as_the_protocol_writes_them() {
        let open = status(
            r#"{"open":true,"secs":600,"cooldown":86400,"now":1000,"trial":{"blade":"saber_sun","ends":1600},"cooling":{"saber_void":90000}}"#,
        );
        assert!(open.open);
        assert_eq!(
            open.trial,
            Some(Trial {
                blade: "saber_sun".to_owned(),
                ends: 1_600
            })
        );
        assert_eq!(open.cooling["saber_void"], 90_000);
        let closed = status(r#"{"open":false,"trial":null,"cooling":{}}"#);
        assert!(!closed.open && closed.trial.is_none());
        let started: Started =
            serde_json::from_str(r#"{"blade":"saber_sun","started":1000,"ends":1600}"#).unwrap();
        assert_eq!(started.ends, 1_600);
    }

    #[test]
    fn a_trial_runs_on_this_pcs_clock_then_cools_down() {
        let read = Instant::now();
        let trials = Trials {
            status: status(
                r#"{"open":true,"secs":600,"cooldown":86400,"now":1000,"trial":{"blade":"saber_sun","ends":1600},"cooling":{"saber_void":5000}}"#,
            ),
            read,
        };
        assert_eq!(
            trials.running(read),
            Some(("saber_sun", Duration::from_secs(600)))
        );
        assert!(trials.wears("saber_sun", read + Duration::from_secs(599)));
        assert!(!trials.wears("saber_void", read));
        let after = read + Duration::from_secs(600);
        assert_eq!(trials.running(after), None);
        assert!(!trials.wears("saber_sun", after));
        // The Sun cools down a day from its end; the Void until its time.
        assert_eq!(
            trials.cooling("saber_sun", after),
            Some(Duration::from_secs(86_400))
        );
        assert_eq!(
            trials.cooling("saber_void", read),
            Some(Duration::from_secs(4_000))
        );
        assert_eq!(trials.cooling("saber_frost", read), None);
        assert_eq!(
            trials.cooling("saber_void", read + Duration::from_secs(4_000)),
            None
        );
    }

    #[test]
    fn a_start_is_noted_at_once() {
        let read = Instant::now();
        let mut trials = Trials {
            status: status(r#"{"open":true,"secs":600,"cooldown":86400,"now":1000}"#),
            read,
        };
        let later = read + Duration::from_secs(30);
        trials.started(
            &Started {
                blade: "saber_frost".to_owned(),
                started: 1_030,
                ends: 1_630,
            },
            later,
        );
        assert_eq!(
            trials.running(later),
            Some(("saber_frost", Duration::from_secs(600)))
        );
    }

    #[test]
    fn times_left_read_short() {
        assert_eq!(time_left(Duration::from_secs(545)), "9:05");
        assert_eq!(time_left(Duration::from_secs(59)), "0:59");
        assert_eq!(time_left(Duration::from_secs(2_700)), "45 min");
        assert_eq!(time_left(Duration::from_secs(82_000)), "23 h");
    }

    #[test]
    fn refusals_read_in_the_players_words() {
        let rejected = |code: &str| HubError::Rejected {
            status: 409,
            code: code.to_owned(),
            message: "hub words".to_owned(),
        };
        assert!(refusal(&rejected("try_cooldown")).contains("later"));
        assert_eq!(refusal(&rejected("other")), "hub words");
        let old = HubError::Protocol("status 404 Not Found".to_owned());
        assert!(unsupported(&old));
        assert!(refusal(&old).contains("does not let"));
        assert!(!unsupported(&rejected("try_running")));
    }
}
