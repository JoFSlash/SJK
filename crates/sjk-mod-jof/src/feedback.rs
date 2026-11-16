//! EJK #110/#112 firing confirmations, emitted by the server, never by a key press.

/// One reliable command and its optional installed local sound.
pub struct Feedback {
    /// Exact reliable server command token.
    pub command: &'static [u8],
    /// VFS asset supplied by the installed client pack; never bundled here.
    pub sound: &'static str,
    /// Minimum interval between duplicate confirmations, in server milliseconds.
    pub cooldown_ms: i32,
}
/// Evidence: `cg_servercmds.c::CG_Stasis_f` / `CG_Repulse_f` in EJK #110/#112.
/// No server-name comparison is needed: receipt of the command is the contract.
pub const FEEDBACK: &[Feedback] = &[
    Feedback {
        command: b"stasis",
        sound: "sound/jof/stasis.wav",
        cooldown_ms: 250,
    },
    Feedback {
        command: b"repulse",
        sound: "sound/jof/repulse.wav",
        cooldown_ms: 250,
    },
];

/// Recognize a confirmation without allocating or interpreting gameplay grants.
pub fn command(name: &[u8]) -> Option<usize> {
    FEEDBACK
        .iter()
        .position(|entry| name.eq_ignore_ascii_case(entry.command))
}

/// Per-map, per-ability repeat suppression, independent of the input press latch.
#[derive(Default)]
pub struct Gate {
    last: [Option<i32>; FEEDBACK.len()],
}
impl Gate {
    /// Accept the first confirmation and later ones after their delay. Rewinds
    /// start a fresh interval; a map install resets the whole gate.
    pub fn accept(&mut self, index: usize, time: i32) -> Option<&'static Feedback> {
        let entry = FEEDBACK.get(index)?;
        let last = &mut self.last[index];
        if last.is_some_and(|last| {
            time >= last && i64::from(time) - i64::from(last) < i64::from(entry.cooldown_ms)
        }) {
            return None;
        }
        *last = Some(time);
        Some(entry)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn confirmations_are_independent_throttled_and_resettable_at_all_command_steps() {
        for step in [8, 7, 4, 3] {
            let mut gate = Gate::default();
            assert_eq!(command(b"STASIS"), Some(0));
            assert_eq!(command(b"repulse"), Some(1));
            assert_eq!(command(b"force_repulse"), None);
            assert!(gate.accept(0, 0).is_some());
            assert!(gate.accept(1, 0).is_some());
            for time in (step..250).step_by(step as usize) {
                assert!(gate.accept(0, time).is_none());
            }
            assert!(gate.accept(0, 250).is_some());
            assert!(gate.accept(0, 250).is_none());
            assert!(gate.accept(0, 10).is_some());
            assert!(gate.accept(0, i32::MAX).is_some());
            assert!(gate.accept(0, i32::MIN).is_some());
            gate = Gate::default();
            assert!(gate.accept(0, 0).is_some());
            assert!(gate.accept(usize::MAX, 0).is_none());
        }
    }
}
