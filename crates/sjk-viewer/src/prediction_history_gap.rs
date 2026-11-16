//! Command-history completeness, independent of queue capacity and packet timing.

/// Latest discarded command not yet covered by the authoritative command time.
#[derive(Default)]
pub(super) struct HistoryGap(Option<i32>);

impl HistoryGap {
    pub(super) fn discarded(&mut self, command_time: i32, acknowledged: i32) {
        if command_time > acknowledged {
            self.0 = Some(self.0.map_or(command_time, |old| old.max(command_time)));
        }
    }

    pub(super) fn acknowledge(&mut self, command_time: i32) {
        if self.0.is_some_and(|missing| command_time >= missing) {
            self.0 = None;
        }
    }

    pub(super) fn incomplete(&self) -> bool {
        self.0.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_recovers_only_with_ack_coverage_or_an_explicit_reset() {
        for step in [8, 7, 4, 3] {
            let mut history = HistoryGap::default();
            history.discarded(step, 0);
            history.discarded(2 * step, 0);
            history.acknowledge(step);
            assert!(history.incomplete());
            history.acknowledge(2 * step);
            assert!(!history.incomplete());
            history.discarded(2 * step, 2 * step);
            assert!(!history.incomplete());
            history.discarded(3 * step, 2 * step);
            history.acknowledge(0);
            assert!(
                history.incomplete(),
                "a clock change does not recover missing commands"
            );
            history = HistoryGap::default(); // an explicit session/prediction reset
            assert!(!history.incomplete());
        }
    }
}
