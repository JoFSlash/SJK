//! Server-reported and session-local death counts for EJK's scoreboard modes.
use super::*;

/// Fixed counters, consuming the existing obituary feed exactly once.
pub(super) struct Deaths {
    counts: [i32; 64],
    consumed: u64,
    time: i32,
    mode: i64,
    gametype: i32,
}

impl Default for Deaths {
    fn default() -> Self {
        Self {
            counts: [0; 64],
            consumed: 0,
            time: 0,
            mode: 1,
            gametype: 0,
        }
    }
}

impl Scoreboard {
    /// Sample the cvar and accept new obituary counters without rebuilding the name cache.
    pub(crate) fn observe_deaths(
        &mut self,
        tracker: &sjk_client::ObituaryTracker,
        snapshot: &sjk_protocol::Snapshot,
        game: &GameState,
        console: Option<&crate::console::ViewerConsole>,
    ) {
        if snapshot.server_time < self.deaths.time {
            self.siege_round_over = None;
        }
        let d = &mut self.deaths;
        if snapshot.server_time < d.time || tracker.decoded() < d.consumed {
            d.counts.fill(0);
            d.consumed = 0;
        }
        d.time = snapshot.server_time;
        d.mode = console
            .and_then(|c| c.integer_cvar("cg_scoredeaths"))
            .unwrap_or(1);
        d.gametype = game
            .config_string(0)
            .and_then(|b| sjk_client::LegacyClientInfo::new(b).integer("g_gametype"))
            .unwrap_or(0);
        let mut killer = None;
        // Oldest first, so the newest kill of the viewing player wins.
        for offset in (0..(tracker.decoded() - d.consumed).min(8) as usize).rev() {
            let Some(event) = tracker.feed().newest(offset) else {
                continue;
            };
            if let Some(count) = d.counts.get_mut(usize::from(event.target)) {
                *count = count.saturating_add(1);
            }
            // `cg.killerName`: set when another player kills you.
            if event.local_was_killed && event.attacker < 32 && event.attacker != event.target {
                killer = Some(event.attacker);
            }
        }
        d.consumed = tracker.decoded();
        // A round won since the last frame; the clock running back (a new level or
        // server, a demo sought back) forgets the one before.
        let round = tracker.siege_round_over();
        if round != self.siege_round_seen {
            self.siege_round_seen = round;
            self.siege_round_over = round;
        }
        if let Some(client) = killer {
            self.killer = Some(client);
            self.killer_name.clear();
            self.killer_name
                .push_str(&client_identity(game, client as u8).0);
        }
    }
}

impl Deaths {
    /// Apply JoF EJK's `cg_scoreDeaths` policy: mode 1 uses plugin data, mode 2
    /// falls back to the local counter, and mode 3 always uses the local counter.
    pub(super) fn count(&self, client: u8, reported: Option<i32>) -> Option<i32> {
        if self.mode == 0 || matches!(self.gametype, 3 | 7) {
            return None;
        }
        match self.mode {
            2 => reported.or_else(|| self.counts.get(usize::from(client)).copied()),
            3 => self.counts.get(usize::from(client)).copied(),
            _ => reported,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Deaths;

    #[test]
    fn ejk_modes_choose_reported_or_local_deaths() {
        let mut deaths = Deaths::default();
        deaths.counts[4] = 6;

        deaths.mode = 0;
        assert_eq!(deaths.count(4, Some(9)), None);
        deaths.mode = 1;
        assert_eq!(deaths.count(4, Some(9)), Some(9));
        assert_eq!(deaths.count(4, None), None);
        deaths.mode = 2;
        assert_eq!(deaths.count(4, Some(9)), Some(9));
        assert_eq!(deaths.count(4, None), Some(6));
        deaths.mode = 3;
        assert_eq!(deaths.count(4, Some(9)), Some(6));
        deaths.mode = 4;
        assert_eq!(deaths.count(4, Some(9)), Some(9));

        deaths.gametype = 7;
        assert_eq!(deaths.count(4, Some(9)), None);
    }
}
