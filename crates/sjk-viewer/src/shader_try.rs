//! Trying a shader (Blep's idea in #ideas, 04/11/2026; `sjk_identity::shader_try`, the
//! hub's `PROTOCOL.md` "Trying a shader"): the Shaders tab's card offers "Try for 10 min"
//! on a shader the player does not hold, when the hub lets them. The hub starts the trial
//! and decides when it ends; while it runs the look wears the skin as if held
//! (`player_identity::wears_unlock`), so every SJK player on the server sees it. Once the
//! hub says it started, the saber wears it (`cg_saberSkin`) and what it wore before is kept
//! in [`BEFORE_CVAR`]; when the trial ends (on this PC's clock from the hub's times, or as
//! the hub says after a restart) the saber goes back to it. The hub takes the skin off the
//! live look itself at the end, whatever this client does, and then that skin cools down
//! (a day at first) before it may be tried again.

use crate::console::ViewerConsole;
use crate::unlockables::SABER_SKIN_CVAR;
use std::time::{Duration, Instant};

/// What the saber wore before a trial put its skin on: empty while no trial is followed,
/// [`STOCK`] for the stock blade, else a blade skin id.
pub(crate) const BEFORE_CVAR: &str = "cg_saberSkinBeforeTry";
/// [`BEFORE_CVAR`]'s word for the stock blade.
const STOCK: &str = "stock";
/// How often the trial is looked at.
const EVERY: Duration = Duration::from_millis(500);

/// The console's side of trials: the trial asked for and the last answer acted on.
#[derive(Debug, Default)]
pub(crate) struct State {
    /// The blade skin asked for, until the hub answers.
    asked: Option<&'static str>,
    /// The serial of the last answer acted on.
    seen: u64,
    /// When the trial was last looked at.
    looked: Option<Instant>,
}

/// What a shader's card offers about trying it, from the hub's last answer at `now`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Offer {
    /// "Try for 10 min": the hub lets the player try it now.
    Try(String),
    /// Its trial runs, this long still.
    Running(Duration),
    /// It cools down, this long still.
    Cooling(Duration),
    /// Another trial runs.
    Busy,
}

impl Offer {
    /// The card's button: its label, and whether it may be pressed.
    pub(crate) fn button(&self) -> (String, bool) {
        use sjk_identity::shader_try::time_left;
        match self {
            Self::Try(label) => (label.clone(), true),
            Self::Running(left) => (format!("Trying, {} left", time_left(*left)), false),
            Self::Cooling(left) => (format!("Try again in {}", time_left(*left)), false),
            Self::Busy => ("Another trial runs".to_owned(), false),
        }
    }
}

/// What the card offers for `skin` (`None`: no button: the hub has no trials, does not
/// let this player try, or `skin` may not be tried: held, or Unique).
pub(crate) fn offer(
    trials: Option<&sjk_identity::shader_try::Trials>,
    skin: &crate::unlockables::Unlockable,
    held: bool,
    now: Instant,
) -> Option<Offer> {
    let trials = trials?;
    if held || skin.tier == crate::unlockables::Rarity::Unique {
        return None;
    }
    if let Some((worn, left)) = trials.running(now) {
        return Some(if worn == skin.id {
            Offer::Running(left)
        } else {
            Offer::Busy
        });
    }
    if let Some(left) = trials.cooling(skin.id, now) {
        return Some(Offer::Cooling(left));
    }
    trials.status.open.then(|| {
        let minutes = (trials.status.secs.max(60) / 60).max(1);
        Offer::Try(format!("Try for {minutes} min"))
    })
}

/// The Try button was pressed for `skin`: ask the hub.
pub(crate) fn ask(console: &mut ViewerConsole, skin: &'static str) {
    console.shader_try.asked = Some(skin);
    crate::player_identity::start_shader_try(skin);
}

/// Follow the trial, twice a second: wear the skin once the hub says it started, and put
/// back what the saber wore before once it ended.
pub(crate) fn follow(console: &mut ViewerConsole) {
    let now = Instant::now();
    if console
        .shader_try
        .looked
        .is_some_and(|looked| now < looked + EVERY)
    {
        return;
    }
    console.shader_try.looked = Some(now);
    if let Some((serial, outcome)) = crate::player_identity::shader_try_started()
        && serial != console.shader_try.seen
    {
        console.shader_try.seen = serial;
        let asked = console.shader_try.asked.take();
        match outcome {
            Ok(started) if asked == Some(started.blade.as_str()) => {
                let worn = console
                    .text_cvar(SABER_SKIN_CVAR)
                    .unwrap_or_default()
                    .to_owned();
                if before(console).is_none() {
                    let kept = if worn.is_empty() { STOCK } else { &worn };
                    console.set_cvar(BEFORE_CVAR, kept);
                }
                console.set_cvar(SABER_SKIN_CVAR, &started.blade);
            }
            Ok(_) => {}
            Err(why) if asked.is_some() => console.push_log(format!("^3Shader trial: {why}")),
            Err(_) => {}
        }
    }
    let Some(kept) = before(console) else {
        return;
    };
    // Only on the hub's word: before it answered, a trial cannot be told from none.
    let Some(trials) = crate::player_identity::shader_trials() else {
        return;
    };
    let worn = console
        .text_cvar(SABER_SKIN_CVAR)
        .unwrap_or_default()
        .to_owned();
    match settle(
        &worn,
        &kept,
        |id| trials.wears(id, now),
        crate::player_identity::owns_unlock,
    ) {
        Settle::Keep => {}
        Settle::Forget => {
            console.set_cvar(BEFORE_CVAR, "");
        }
        Settle::PutBack(skin) => {
            console.set_cvar(SABER_SKIN_CVAR, &skin);
            console.set_cvar(BEFORE_CVAR, "");
        }
    }
}

/// [`BEFORE_CVAR`]: `None` while no trial is followed, `Some("")` for the stock blade.
fn before(console: &ViewerConsole) -> Option<String> {
    let kept = console.text_cvar(BEFORE_CVAR).unwrap_or_default().trim();
    match kept {
        "" => None,
        STOCK => Some(String::new()),
        skin => Some(skin.to_owned()),
    }
}

/// What to do with a followed trial.
#[derive(Debug, PartialEq, Eq)]
enum Settle {
    /// The trial still runs on the worn skin.
    Keep,
    /// The player wears something of their own now: nothing to put back.
    Forget,
    /// The trial ended: wear this again (empty for the stock blade).
    PutBack(String),
}

/// The saber wears `worn`, and wore `kept` before the trial; `trying` says whether a
/// trial lets it wear a skin, `held` whether the player holds one.
fn settle(
    worn: &str,
    kept: &str,
    trying: impl Fn(&str) -> bool,
    held: impl Fn(&str) -> bool,
) -> Settle {
    if !worn.is_empty() && trying(worn) {
        Settle::Keep
    } else if worn.is_empty() || held(worn) {
        Settle::Forget
    } else {
        Settle::PutBack(kept.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjk_identity::shader_try::{Trials, TryStatus};

    fn trials(text: &str) -> (Trials, Instant) {
        let read = Instant::now();
        let status: TryStatus = serde_json::from_str(text).unwrap();
        (Trials { status, read }, read)
    }

    fn skin(id: &str) -> &'static crate::unlockables::Unlockable {
        crate::unlockables::find(id).unwrap()
    }

    #[test]
    fn the_card_offers_a_try_only_when_the_hub_lets_and_the_skin_is_not_held() {
        let (open, now) = trials(r#"{"open":true,"secs":600,"cooldown":86400,"now":1000}"#);
        assert_eq!(
            offer(Some(&open), skin("saber_sun"), false, now),
            Some(Offer::Try("Try for 10 min".to_owned()))
        );
        assert_eq!(offer(Some(&open), skin("saber_sun"), true, now), None);
        assert_eq!(offer(Some(&open), skin("saber_hearth"), false, now), None);
        assert_eq!(offer(None, skin("saber_sun"), false, now), None);
        let (closed, now) = trials(r#"{"open":false,"secs":600,"cooldown":86400,"now":1000}"#);
        assert_eq!(offer(Some(&closed), skin("saber_sun"), false, now), None);
    }

    #[test]
    fn the_card_says_a_trial_runs_or_cools_down() {
        let (running, now) = trials(
            r#"{"open":true,"secs":600,"cooldown":86400,"now":1000,"trial":{"blade":"saber_sun","ends":1300},"cooling":{"saber_void":4600}}"#,
        );
        assert_eq!(
            offer(Some(&running), skin("saber_sun"), false, now),
            Some(Offer::Running(Duration::from_secs(300)))
        );
        assert_eq!(
            offer(Some(&running), skin("saber_frost"), false, now),
            Some(Offer::Busy)
        );
        let later = now + Duration::from_secs(300);
        assert_eq!(
            offer(Some(&running), skin("saber_sun"), false, later),
            Some(Offer::Cooling(Duration::from_secs(86_400)))
        );
        assert_eq!(
            offer(Some(&running), skin("saber_void"), false, later),
            Some(Offer::Cooling(Duration::from_secs(3_300)))
        );
        assert_eq!(
            Offer::Running(Duration::from_secs(300)).button(),
            ("Trying, 5:00 left".to_owned(), false)
        );
        assert_eq!(
            Offer::Cooling(Duration::from_secs(86_000)).button(),
            ("Try again in 24 h".to_owned(), false)
        );
    }

    #[test]
    fn the_saber_goes_back_once_the_trial_ends() {
        let none = |_: &str| false;
        let sun = |id: &str| id == "saber_sun";
        // Still running.
        assert_eq!(settle("saber_sun", "saber_frost", sun, none), Settle::Keep);
        // Ended: the skin worn before goes back on, the stock blade too.
        assert_eq!(
            settle("saber_sun", "saber_frost", none, none),
            Settle::PutBack("saber_frost".to_owned())
        );
        assert_eq!(
            settle("saber_sun", "", none, none),
            Settle::PutBack(String::new())
        );
        // The player chose one of their own (or the stock blade) meanwhile.
        assert_eq!(
            settle("saber_frost", "saber_void", none, |id| id == "saber_frost"),
            Settle::Forget
        );
        assert_eq!(settle("", "saber_void", none, none), Settle::Forget);
        // Given the skin for good during the trial: it stays.
        assert_eq!(settle("saber_sun", "saber_void", none, sun), Settle::Forget);
    }
}
