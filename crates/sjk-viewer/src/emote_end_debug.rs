//! `cg_debugEmoteEnd` (hub report #196, a JA+ `amsit5` or `ammeditate` ended while the
//! Collection was open, not seen again here): when the server lets go of the player from
//! a JA+ emote (`PM_FREEZE` ends), the console says what this client did just before
//! that could have ended it. Each line names the last command sent with a button other
//! than the talk balloon's, a move, a `generic_cmd` or another weapon, how long before
//! it went, the away emote's last step, the longest frame of the last few seconds (a
//! hitch), and whether the window was out of focus, the console open and the Collection
//! showing. Off by default and not archived, so it changes nothing until turned on.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use sjk_protocol::UserCommand;

/// The switch.
pub(crate) const CVAR: &str = "cg_debugEmoteEnd";
/// `PM_FREEZE`: a JA+ emote holds the player.
const PM_FREEZE: u8 = 6;
/// What a line looks back over.
const WINDOW: Duration = Duration::from_secs(3);

/// What the client was doing around the end, read when it is reported.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Context {
    pub(crate) unfocused: bool,
    pub(crate) console_open: bool,
    pub(crate) collection_open: bool,
}

#[derive(Debug, Default)]
struct Watch {
    frozen: bool,
    /// The last command that could have ended an emote, and when it went.
    input: Option<(Instant, UserCommand, u8)>,
    /// The away emote's last step, and when.
    away: Option<(Instant, String)>,
    /// The last frame's start and the longest frame since, with when it ended.
    last_frame: Option<Instant>,
    longest: Option<(Instant, Duration)>,
}

static WATCH: Mutex<Option<Watch>> = Mutex::new(None);
/// [`CVAR`]'s value, so that while it is off (the default) nothing is locked or kept.
static ENABLED: AtomicBool = AtomicBool::new(false);

/// Follow [`CVAR`]; turning it off forgets what was watched.
pub(crate) fn set_enabled(on: bool) {
    if ENABLED.swap(on, Ordering::Relaxed) && !on {
        *WATCH
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
    }
}

fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

fn with<T>(f: impl FnOnce(&mut Watch) -> T) -> T {
    let mut slot = WATCH
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    f(slot.get_or_insert_with(Watch::default))
}

/// Whether `command` could end a JA+ emote: a button besides the talk balloon's, a move,
/// a `generic_cmd`, or a weapon other than the one the player holds (`held`).
fn could_end(command: &UserCommand, held: u8) -> bool {
    command.buttons & !sjk_game_jka::pmove_talk::BUTTON_TALK != 0
        || command.forward_move != 0
        || command.right_move != 0
        || command.up_move != 0
        || command.generic_command != 0
        || command.weapon != held
}

/// A command went to the server while the player held weapon `held`.
pub(crate) fn observe_command(command: &UserCommand, held: u8, now: Instant) {
    if enabled() && could_end(command, held) {
        with(|watch| watch.input = Some((now, *command, held)));
    }
}

/// The away emote took `step`.
pub(crate) fn note_away_step(step: &crate::away_emote::Step) {
    if !enabled() {
        return;
    }
    let now = Instant::now();
    with(|watch| watch.away = Some((now, format!("{step:?}"))));
}

/// A frame began at `now`: the longest of the last [`WINDOW`] is kept.
pub(crate) fn observe_frame(now: Instant) {
    if !enabled() {
        return;
    }
    with(|watch| {
        if let Some(last) = watch.last_frame {
            let frame = now.saturating_duration_since(last);
            let stale = watch
                .longest
                .is_none_or(|(at, _)| now.saturating_duration_since(at) > WINDOW);
            if stale || watch.longest.is_some_and(|(_, longest)| frame > longest) {
                watch.longest = Some((now, frame));
            }
        }
        watch.last_frame = Some(now);
    });
}

/// The player's movement type is `movement_type` now; the line to print when a JA+
/// emote just let go of them, while [`CVAR`] is on.
pub(crate) fn report(movement_type: u8, context: Context, now: Instant) -> Option<String> {
    if !enabled() {
        return None;
    }
    with(|watch| {
        let frozen = movement_type == PM_FREEZE;
        let ended = watch.frozen && !frozen;
        watch.frozen = frozen;
        ended.then(|| line(watch, context, now))
    })
}

fn ago(now: Instant, at: Instant) -> u128 {
    now.saturating_duration_since(at).as_millis()
}

fn line(watch: &Watch, context: Context, now: Instant) -> String {
    let recent = |at: Instant| now.saturating_duration_since(at) <= WINDOW;
    let input = match watch.input.filter(|(at, ..)| recent(*at)) {
        Some((at, command, held)) => format!(
            "last input {} ms before: buttons {:#x} forward {} right {} up {} generic {} \
             weapon {} (held {held})",
            ago(now, at),
            command.buttons,
            command.forward_move,
            command.right_move,
            command.up_move,
            command.generic_command,
            command.weapon,
        ),
        None => "no input sent in the last 3 s".to_owned(),
    };
    let away = match &watch.away {
        Some((at, step)) if recent(*at) => format!("away {step} {} ms before", ago(now, *at)),
        _ => "no away step".to_owned(),
    };
    let hitch = watch
        .longest
        .filter(|(at, _)| recent(*at))
        .map_or(0, |(_, frame)| frame.as_millis());
    format!(
        "^3emote ended (PM_FREEZE let go): {input}; {away}; longest frame {hitch} ms; \
         unfocused {} console {} collection {}",
        u8::from(context.unfocused),
        u8::from(context.console_open),
        u8::from(context.collection_open),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn still() -> UserCommand {
        UserCommand {
            weapon: 3,
            buttons: sjk_game_jka::pmove_talk::BUTTON_TALK,
            ..UserCommand::default()
        }
    }

    #[test]
    fn only_what_could_end_an_emote_counts() {
        assert!(
            !could_end(&still(), 3),
            "the talk balloon and the saber held"
        );
        for command in [
            UserCommand {
                up_move: -127,
                ..still()
            },
            UserCommand {
                forward_move: 1,
                ..still()
            },
            UserCommand {
                buttons: 1,
                ..still()
            },
            UserCommand {
                generic_command: 4,
                ..still()
            },
            UserCommand {
                weapon: 1,
                ..still()
            },
        ] {
            assert!(could_end(&command, 3), "{command:?}");
        }
    }

    #[test]
    fn the_end_of_a_freeze_names_the_crouch_before_it() {
        let t0 = Instant::now();
        let quiet = Context::default();
        set_enabled(true);
        assert_eq!(report(PM_FREEZE, quiet, t0), None, "held");
        observe_command(&still(), 3, t0);
        observe_command(
            &UserCommand {
                up_move: -127,
                ..still()
            },
            3,
            t0,
        );
        let later = t0 + Duration::from_millis(40);
        let line = report(0, quiet, later).expect("let go");
        assert!(line.contains("40 ms before"), "{line}");
        assert!(line.contains("up -127"), "{line}");
        assert_eq!(report(0, quiet, later), None, "once");
        // Off, nothing is said or kept.
        set_enabled(false);
        assert_eq!(report(PM_FREEZE, quiet, later), None);
        assert_eq!(report(0, quiet, later), None);
        assert!(WATCH.lock().unwrap().is_none());
    }
}
