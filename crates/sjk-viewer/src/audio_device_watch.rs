//! Following the system's default sound output: reopen the game's output when the
//! default device changes (headphones plugged in, a headset switched on) or when the
//! device it plays on stopped taking sound.
//!
//! cpal (under rodio) opens one device, the default of the moment, and keeps playing
//! on it whatever the system default becomes; with cpal 0.15 there is no notification
//! of a default change. So a small thread asks for the default device's name every
//! [`POLL`] and publishes it; the event loop compares it at most once per poll with
//! the device the output was opened on ([`decide`]) and reopens the output as
//! `snd_restart` does. Each frame costs one atomic load; the lock and the string
//! comparison happen once per poll, and nothing is allocated unless the output is
//! reopened. Hub report #210 (Metan).

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// How often the watcher asks for the default output device.
pub(crate) const POLL: Duration = Duration::from_millis(1_500);
/// The least time between two reopens for a stalled output on an unchanged default,
/// so a device that keeps failing is not reopened at every poll.
pub(crate) const STALL_RETRY: Duration = Duration::from_secs(10);

/// Why the output should be opened again.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reopen {
    /// The system's default output is another device than the one the output was
    /// opened on (or there was none and now there is one).
    DefaultChanged,
    /// The default is the same device, but its stream stopped taking sound (the device
    /// went away and came back, or its format changed).
    Stalled,
}

/// Whether to reopen the output: `opened` is the default device's name when the output
/// was last opened (or tried), `current` the default's name now, `stalled` whether the
/// open output stopped draining its queue, and `since_reopen` how long ago the watcher
/// last reopened it (`None`: never). With no default device there is nothing to open.
pub(crate) fn decide(
    opened: Option<&str>,
    current: Option<&str>,
    stalled: bool,
    since_reopen: Option<Duration>,
) -> Option<Reopen> {
    let current = current?;
    if opened != Some(current) {
        return Some(Reopen::DefaultChanged);
    }
    (stalled && since_reopen.is_none_or(|since| since >= STALL_RETRY)).then_some(Reopen::Stalled)
}

/// The name of the system's default output device, `None` with none (or no name).
pub(crate) fn default_output_name() -> Option<String> {
    use rodio::cpal::traits::{DeviceTrait, HostTrait};
    rodio::cpal::default_host()
        .default_output_device()?
        .name()
        .ok()
}

/// What the polling thread publishes.
#[derive(Default)]
struct Shared {
    /// Bumped after every poll: the event loop looks at `current` only when it moved.
    polls: AtomicU64,
    current: Mutex<Option<String>>,
    stop: AtomicBool,
}

/// The polling thread and what the event loop remembers between polls.
pub(crate) struct Watch {
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
    seen: u64,
    /// The default's name when the watcher last reopened (or tried to reopen) the
    /// output, for when no output is open to say which device it was opened on.
    attempted: Option<String>,
    reopened_at: Option<Instant>,
}

impl Watch {
    /// Start the polling thread. Without a thread nothing is followed (`snd_restart`
    /// still reopens the output by hand).
    pub(crate) fn start() -> Self {
        let shared = Arc::new(Shared::default());
        let thread = {
            let shared = Arc::clone(&shared);
            thread::Builder::new()
                .name("sjk-audio-device".into())
                .spawn(move || poll_default(&shared))
                .map_err(|error| {
                    crate::log::progress(format_args!(
                        "audio device watcher could not start: {error}"
                    ))
                })
                .ok()
        };
        Self {
            shared,
            thread,
            seen: 0,
            attempted: None,
            reopened_at: None,
        }
    }

    /// At most once per [`POLL`]: whether `audio` should be opened again on the
    /// current default device. One atomic load when no new poll came in.
    pub(crate) fn poll(
        &mut self,
        audio: Option<&crate::GameAudio>,
        now: Instant,
    ) -> Option<Reopen> {
        let polls = self.shared.polls.load(Ordering::Acquire);
        if polls == self.seen {
            return None;
        }
        self.seen = polls;
        let current = self.shared.current.lock().ok()?;
        let (opened, stalled) = match audio {
            Some(audio) => (audio.output_device(), audio.output_stalled()),
            None => (self.attempted.as_deref(), false),
        };
        let since = self.reopened_at.map(|at| now.saturating_duration_since(at));
        let reopen = decide(opened, current.as_deref(), stalled, since)?;
        self.attempted.clone_from(&current);
        self.reopened_at = Some(now);
        Some(reopen)
    }
}

impl Drop for Watch {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            thread.thread().unpark();
            let _ = thread.join();
        }
    }
}

/// The thread: publish the default device's name every [`POLL`] until stopped.
fn poll_default(shared: &Shared) {
    let mut last: Option<String> = None;
    while !shared.stop.load(Ordering::Acquire) {
        thread::park_timeout(POLL);
        if shared.stop.load(Ordering::Acquire) {
            break;
        }
        let name = default_output_name();
        if name != last {
            if let Ok(mut current) = shared.current.lock() {
                current.clone_from(&name);
            }
            last = name;
        }
        shared.polls.fetch_add(1, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEAKERS: Option<&str> = Some("Speakers (High Definition Audio)");
    const HEADPHONES: Option<&str> = Some("Headphones (High Definition Audio)");

    #[test]
    fn a_new_default_device_reopens_the_output() {
        assert_eq!(
            decide(SPEAKERS, HEADPHONES, false, None),
            Some(Reopen::DefaultChanged)
        );
        // Even just after another reopen: the player switched again.
        assert_eq!(
            decide(HEADPHONES, SPEAKERS, false, Some(Duration::ZERO)),
            Some(Reopen::DefaultChanged)
        );
    }

    #[test]
    fn an_unchanged_default_keeps_the_output() {
        assert_eq!(decide(SPEAKERS, SPEAKERS, false, None), None);
        assert_eq!(decide(SPEAKERS, SPEAKERS, false, Some(STALL_RETRY)), None);
    }

    #[test]
    fn no_default_device_opens_nothing() {
        assert_eq!(decide(SPEAKERS, None, false, None), None);
        assert_eq!(decide(SPEAKERS, None, true, None), None);
        assert_eq!(decide(None, None, false, None), None);
    }

    #[test]
    fn a_device_appearing_opens_an_output() {
        assert_eq!(
            decide(None, HEADPHONES, false, None),
            Some(Reopen::DefaultChanged)
        );
    }

    #[test]
    fn a_stalled_output_on_the_same_device_is_reopened_at_a_slow_rate() {
        assert_eq!(
            decide(SPEAKERS, SPEAKERS, true, None),
            Some(Reopen::Stalled)
        );
        assert_eq!(
            decide(
                SPEAKERS,
                SPEAKERS,
                true,
                Some(STALL_RETRY - Duration::from_millis(1))
            ),
            None
        );
        assert_eq!(
            decide(SPEAKERS, SPEAKERS, true, Some(STALL_RETRY)),
            Some(Reopen::Stalled)
        );
    }

    #[test]
    fn the_watch_looks_once_per_poll_and_remembers_what_it_tried() {
        let mut watch = Watch {
            shared: Arc::new(Shared::default()),
            thread: None,
            seen: 0,
            attempted: None,
            reopened_at: None,
        };
        let now = Instant::now();
        // No poll yet: nothing to decide on.
        assert_eq!(watch.poll(None, now), None);
        *watch.shared.current.lock().unwrap() = HEADPHONES.map(str::to_owned);
        watch.shared.polls.fetch_add(1, Ordering::Release);
        assert_eq!(watch.poll(None, now), Some(Reopen::DefaultChanged));
        // The same poll is not looked at twice.
        assert_eq!(watch.poll(None, now), None);
        // The open failed (no output): the next poll on the same default does not
        // try again.
        watch.shared.polls.fetch_add(1, Ordering::Release);
        assert_eq!(watch.poll(None, now), None);
        assert_eq!(watch.attempted.as_deref(), HEADPHONES);
    }
}
