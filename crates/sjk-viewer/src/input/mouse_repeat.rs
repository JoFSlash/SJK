//! A held mouse 4 or 5 presses its key again, 20 times a second (`in_mouseRepeat`).
//!
//! EternalJK's `in_mouserepeat` (JoF EJK `sdl_input.cpp`, `IN_ProcessEvents`, archived,
//! default 0): while it is on, every other frame queues a key-down for a held mouse 4,
//! and for a held mouse 5 unless it is 2 ("don't spam mouse5"), as a held keyboard key's
//! system repeats do. A key-down of a held key runs a plain bind command again, while a
//! `+` command ignores it (`IN_KeyDown`'s "repeating key"; SJK's button state does the
//! same, `input/state.rs`).
//!
//! One deliberate difference (Sol's decision, 29/10/2026): EJK repeats once every other
//! frame, 125 presses a second at 250 FPS and more above, and each can be a command to
//! the server, which servers that cap commands (JoF's) see as a flood. SJK repeats at a
//! fixed [`RATE`] instead, the same at any frame rate: a press is due [`INTERVAL_MILLIS`]
//! after the last, at most one a frame, so a slow frame never sends a burst.
//!
//! The console tracks the buttons pressed into the bind stream
//! (`ViewerConsole::queue_bound_script`) and presses the due ones again once a frame,
//! while the pointer is the game's and no text field, menu or console has the keyboard
//! (`GpuState::run_console_command_buffer`). A release stops a button wherever it went,
//! and losing the window stops both.

/// The setting: 0 off, 1 mouse 4 and 5, 2 mouse 4 only (EternalJK's values).
pub(crate) const CVAR: &str = "in_mouseRepeat";
/// Repeated presses a second, whatever the frame rate.
pub(crate) const RATE: u64 = 20;
/// Milliseconds from one repeated press to the next.
pub(crate) const INTERVAL_MILLIS: u64 = 1_000 / RATE;
/// The buttons that repeat, by their bind names.
const KEYS: [&str; 2] = ["MOUSE4", "MOUSE5"];

/// Whether setting `mode` repeats button `slot` of [`KEYS`]: any value but 0 repeats
/// mouse 4, and mouse 5 unless it is 2, as EternalJK reads it.
fn repeats(mode: i64, slot: usize) -> bool {
    mode != 0 && (slot == 0 || mode != 2)
}

fn slot(key: &str) -> Option<usize> {
    KEYS.iter().position(|name| name.eq_ignore_ascii_case(key))
}

/// The held mouse 4 and 5 and when each is due its next press, on the console's
/// millisecond clock.
#[derive(Debug, Default)]
pub(crate) struct MouseRepeat {
    due: [Option<u64>; 2],
}

impl MouseRepeat {
    /// Bind key `key` went down at `now`; a press of a button already held (a repeat)
    /// keeps its schedule.
    pub(crate) fn press(&mut self, key: &str, now: u64) {
        if let Some(slot) = slot(key) {
            self.due[slot].get_or_insert(now.saturating_add(INTERVAL_MILLIS));
        }
    }

    /// Bind key `key` went up.
    pub(crate) fn release(&mut self, key: &str) {
        if let Some(slot) = slot(key) {
            self.due[slot] = None;
        }
    }

    /// Every button went up (the window lost the pointer).
    pub(crate) fn clear(&mut self) {
        self.due = [None; 2];
    }

    /// The held buttons due a repeated press at `now` under setting `mode`, each at
    /// most once; the next press is due one interval after this one was, or after
    /// `now` when the frame came later than that.
    pub(crate) fn due(&mut self, mode: i64, now: u64) -> [Option<&'static str>; 2] {
        std::array::from_fn(|slot| {
            let due = self.due[slot]?;
            if !repeats(mode, slot) || now < due {
                return None;
            }
            let next = due.saturating_add(INTERVAL_MILLIS);
            self.due[slot] = Some(if next > now {
                next
            } else {
                now.saturating_add(INTERVAL_MILLIS)
            });
            Some(KEYS[slot])
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Presses of each button over `millis` ms of frames `frame` ms apart, held from 0.
    fn presses(mode: i64, frame: u64, millis: u64) -> [usize; 2] {
        let mut repeat = MouseRepeat::default();
        repeat.press("MOUSE4", 0);
        repeat.press("mouse5", 0);
        let mut counts = [0; 2];
        let mut now = 0;
        while now < millis {
            now += frame;
            for (slot, key) in repeat.due(mode, now).into_iter().enumerate() {
                counts[slot] += usize::from(key.is_some());
            }
        }
        counts
    }

    #[test]
    fn twenty_presses_a_second_at_any_frame_rate() {
        // 1000, 333, 250, 142, 125 and 60 FPS: the same 20 a second, where EJK's every
        // other frame would make 500, 166, 125, 71, 62 and 30.
        for frame in [1, 3, 4, 7, 8, 16] {
            assert_eq!(presses(1, frame, 1_000), [20, 20], "{frame} ms frames");
            assert_eq!(presses(1, frame, 5_000), [100, 100], "{frame} ms frames");
        }
        // Below 20 FPS, one a frame at most: no burst after a slow frame.
        assert_eq!(presses(1, 100, 1_000), [10, 10]);
        let mut repeat = MouseRepeat::default();
        repeat.press("MOUSE4", 0);
        assert_eq!(repeat.due(1, 1_000), [Some("MOUSE4"), None]);
        assert_eq!(repeat.due(1, 1_001), [None, None]);
        assert_eq!(repeat.due(1, 1_050), [Some("MOUSE4"), None]);
    }

    #[test]
    fn the_setting_picks_the_buttons() {
        assert_eq!(presses(0, 4, 1_000), [0, 0], "0: off");
        assert_eq!(presses(1, 4, 1_000), [20, 20], "1: mouse 4 and 5");
        assert_eq!(presses(2, 4, 1_000), [20, 0], "2: mouse 4 only");
        assert_eq!(
            presses(3, 4, 1_000),
            [20, 20],
            "any other value: both, as EJK"
        );
    }

    #[test]
    fn only_mouse_4_and_5_held_repeat() {
        let mut repeat = MouseRepeat::default();
        for key in ["MOUSE1", "MOUSE2", "MOUSE3", "MWHEELUP", "SPACE"] {
            repeat.press(key, 0);
        }
        assert_eq!(repeat.due(1, 1_000), [None, None]);
        repeat.press("MOUSE4", 0);
        // A repeated press keeps the schedule: the first repeat is still at 50 ms.
        repeat.press("MOUSE4", 40);
        assert_eq!(repeat.due(1, 49), [None, None]);
        assert_eq!(repeat.due(1, 50), [Some("MOUSE4"), None]);
        repeat.release("mouse4");
        assert_eq!(repeat.due(1, 1_000), [None, None], "let go");
        repeat.press("MOUSE4", 0);
        repeat.press("MOUSE5", 0);
        repeat.clear();
        assert_eq!(repeat.due(1, 1_000), [None, None], "window lost");
    }
}
