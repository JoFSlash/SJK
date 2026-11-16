//! Server-reported and session-local death counts for EJK's scoreboard modes.
use super::*;

/// `CS_LEVEL_START_TIME`: set anew by every `map_restart` and level load.
const CS_LEVEL_START_TIME: usize = 21;
/// `GT_DUEL` and `GT_CTF`, which never show deaths (`cg_scoreboard.c:216`).
const DUEL: i32 = 3;
const CTF: i32 = 8;

/// Fixed counters, consuming the existing obituary feed exactly once.
pub(super) struct Deaths {
    counts: [i32; 64],
    /// Each slot's team (`t`) when last seen, `None` while the slot is empty.
    teams: [Option<u8>; 32],
    level_start: Option<i32>,
    consumed: u64,
    time: i32,
    mode: i64,
    gametype: i32,
}

impl Default for Deaths {
    fn default() -> Self {
        Self {
            counts: [0; 64],
            teams: [None; 32],
            level_start: None,
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
        self.server_intermission = self.intermission.observe(game);
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
        // Before this frame's obituaries, as EJK runs a snapshot's commands before
        // its events: the death of a team change counts on the new team.
        d.follow_roster(game);
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
    /// EJK's resets of the local counter: everyone's on `map_restart` or a new
    /// level (`CG_MapRestart`, `cg_servercmds.c:1245-1255`), one player's when
    /// they change team (`CG_NewClientInfo`, `cg_players.c:2312-2318`) or their
    /// slot empties (the client info cleared).
    fn follow_roster(&mut self, game: &GameState) {
        let start = game
            .config_string(CS_LEVEL_START_TIME)
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
            .and_then(|text| text.trim().parse::<i32>().ok());
        if start != self.level_start {
            self.level_start = start;
            self.counts.fill(0);
        }
        for (client, seen) in self.teams.iter_mut().enumerate() {
            let team = game
                .config_string(CS_PLAYERS + client)
                .filter(|bytes| !bytes.is_empty())
                .map(|bytes| {
                    sjk_client::LegacyClientInfo::new(bytes)
                        .integer("t")
                        .and_then(|team| u8::try_from(team).ok())
                        .unwrap_or(3)
                });
            if team != *seen {
                *seen = team;
                self.counts[client] = 0;
            }
        }
    }

    /// Apply JoF EJK's `cg_scoreDeaths` policy: mode 1 uses plugin data, mode 2
    /// falls back to the local counter, and mode 3 always uses the local counter.
    pub(super) fn count(&self, client: u8, reported: Option<i32>) -> Option<i32> {
        if self.mode == 0 || matches!(self.gametype, DUEL | CTF) {
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
    use super::{CS_LEVEL_START_TIME, CS_PLAYERS, Deaths, GameState};

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

        deaths.gametype = 3;
        assert_eq!(deaths.count(4, Some(9)), None, "duel");
        deaths.gametype = 8;
        assert_eq!(deaths.count(4, Some(9)), None, "capture the flag");
        // Siege and team FFA show them, as in EJK (mode 4 reads as 1).
        deaths.gametype = 7;
        assert_eq!(deaths.count(4, Some(9)), Some(9));
        deaths.gametype = 6;
        assert_eq!(deaths.count(4, Some(9)), Some(9));
    }

    fn game(level_start: &str, players: &[(usize, &str)]) -> GameState {
        let mut game = GameState::empty_local(0);
        let mut set = |index: usize, value: &str| {
            game.replace_config_string(index, value.as_bytes().to_vec())
                .unwrap();
        };
        set(CS_LEVEL_START_TIME, level_start);
        for (client, info) in players {
            set(CS_PLAYERS + client, info);
        }
        game
    }

    #[test]
    fn local_counts_reset_as_ejk_resets_them() {
        let mut deaths = Deaths::default();
        let red = r"\n\Kushina\t\1";
        let blue = r"\n\Sol\t\2";
        deaths.follow_roster(&game("1000", &[(4, red), (5, blue)]));
        deaths.counts[4] = 3;
        deaths.counts[5] = 2;

        // The same roster and level: kept.
        deaths.follow_roster(&game("1000", &[(4, red), (5, blue)]));
        assert_eq!((deaths.counts[4], deaths.counts[5]), (3, 2));

        // A team change resets that player alone.
        let moved = r"\n\Kushina\t\2";
        deaths.follow_roster(&game("1000", &[(4, moved), (5, blue)]));
        assert_eq!((deaths.counts[4], deaths.counts[5]), (0, 2));

        // An emptied slot resets; the next player in it starts from 0.
        deaths.counts[4] = 1;
        deaths.follow_roster(&game("1000", &[(5, blue)]));
        assert_eq!(deaths.counts[4], 0);
        deaths.counts[4] = 1;
        deaths.follow_roster(&game("1000", &[(4, r"\n\New\t\2"), (5, blue)]));
        assert_eq!(deaths.counts[4], 0);

        // `map_restart` (a new level start time) resets everyone.
        deaths.counts[4] = 4;
        deaths.counts[5] = 5;
        deaths.follow_roster(&game("61000", &[(4, r"\n\New\t\2"), (5, blue)]));
        assert_eq!((deaths.counts[4], deaths.counts[5]), (0, 0));
    }
}
