//! Duel lines (Ric's idea, extended by Lex Deifaber, #ideas 04/11/2026): a line of the
//! player's own whispered to their private duel opponent as the duel starts, and another
//! as it ends, one for a win and one for a loss.
//!
//! The lines are three archived text cvars, all empty (off) by default, so stock
//! behaviour is unchanged: [`START_CVAR`], [`WIN_CVAR`] and [`LOSE_CVAR`]. Each goes as a
//! `tell` to the opponent, never to everyone: a private duel is between two players,
//! and a whisper keeps the line out of everybody else's chat, works on every server
//! (stock `Cmd_Tell_f`) and is never taken for spam by a server's flood guard. JA++
//! has no built-in setting for it (its players script it on the `PrivateDuel` event of
//! its client Lua, `bg_luaevent.h`), and EternalJK's nearest is `tell_attacker` bound to
//! a key, which SJK has too (`console_client.rs`).
//!
//! The duel is read from the player's own state (`ps.duelInProgress`, `ps.duelIndex`),
//! never a followed player's. Its end is told by the obituaries the snapshots bring,
//! as the achievements' tracker does: the opponent's means a win, the player's own (or
//! the duel ending with the player dead) a loss. A duel that ends any other way (the
//! opponent leaves, a map change, a spectator switch) sends no end line. Each duel
//! sends at most one start line and one end line, and two lines of the same kind are
//! at least [`GAP`] apart, so a run of duels accepted and ended back to back cannot
//! turn it into spam.
//!
//! [`TO_ALL_CVAR`] (off by default) sends them to all chat instead, for players who
//! want the server to read their `gl hf` and `gg`; the same once-a-duel and gap limits
//! apply.

use sjk_client::ObituaryEvent;
use std::time::{Duration, Instant};

/// Archived, empty by default: the line whispered to the opponent as a duel starts.
pub(crate) const START_CVAR: &str = "cg_duelStartLine";
/// Archived, empty by default: the line whispered to the opponent after a duel won.
pub(crate) const WIN_CVAR: &str = "cg_duelWinLine";
/// Archived, empty by default: the line whispered to the opponent after a duel lost.
pub(crate) const LOSE_CVAR: &str = "cg_duelLoseLine";
/// Archived, off by default: the lines go to all chat (`say`) instead of a whisper to
/// the opponent.
pub(crate) const TO_ALL_CVAR: &str = "cg_duelLinesToAll";

/// The least time between two lines of the same kind (two start lines, or two end
/// lines); a line that would come sooner is not sent.
pub(crate) const GAP: Duration = Duration::from_secs(10);

/// How long an ended duel waits for the obituary that tells how it ended (it may come
/// with the snapshot after the one that ends the duel).
const LINGER: Duration = Duration::from_secs(2);

/// Client slots; a duel index past them is no player.
const PLAYERS: u16 = 32;

/// The moment of a duel a line belongs to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Moment {
    Start,
    Won,
    Lost,
}

impl Moment {
    /// The cvar holding this moment's line.
    pub(crate) fn cvar(self) -> &'static str {
        match self {
            Self::Start => START_CVAR,
            Self::Won => WIN_CVAR,
            Self::Lost => LOSE_CVAR,
        }
    }
}

/// A line due: the moment and the opponent's slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Due {
    pub(crate) moment: Moment,
    pub(crate) to: u16,
}

/// What one live snapshot shows of the player's own private duel.
pub(crate) struct Seen<'a> {
    /// The player's own slot.
    pub(crate) me: u16,
    /// The opponent while the player's own state shows a private duel; `None` when it
    /// shows none or is a followed player's.
    pub(crate) opponent: Option<u16>,
    /// The player's own health is gone.
    pub(crate) dead: bool,
    /// The obituaries this snapshot brought, oldest first.
    pub(crate) obituaries: &'a [ObituaryEvent],
    pub(crate) now: Instant,
}

impl<'a> Seen<'a> {
    /// Read the player's own private duel from `player`, for client `me`.
    pub(crate) fn from_player(
        player: &sjk_protocol::PlayerState,
        me: u16,
        obituaries: &'a [ObituaryEvent],
        now: Instant,
    ) -> Self {
        let own = player.client_num() == me
            && player.movement_flags() & sjk_game_jka::follow::PMF_FOLLOW == 0
            && !player.is_spectator();
        let opponent = (own && player.duel_in_progress())
            .then(|| player.duel_index())
            .filter(|&index| index < PLAYERS && index != me);
        Self {
            me,
            opponent,
            dead: own && player.health() <= 0,
            obituaries,
            now,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Duel {
    with: u16,
    /// When the snapshots stopped showing it, waiting for its obituary.
    ended: Option<Instant>,
}

/// The duel followed between snapshots and when the last lines went.
#[derive(Debug, Default)]
pub(crate) struct DuelLines {
    duel: Option<Duel>,
    last_start: Option<Instant>,
    last_end: Option<Instant>,
    due: Option<Due>,
}

impl DuelLines {
    /// Follow one snapshot; a line that falls due waits for [`Self::take_due`].
    pub(crate) fn observe(&mut self, seen: &Seen<'_>) {
        if let Some(due) = self.step(seen) {
            self.due = Some(due);
        }
    }

    /// The line due, once.
    pub(crate) fn take_due(&mut self) -> Option<Due> {
        self.due.take()
    }

    fn step(&mut self, seen: &Seen<'_>) -> Option<Due> {
        if let Some(duel) = self.duel {
            let outcome = seen.obituaries.iter().find_map(|event| {
                if event.target == seen.me {
                    Some(Moment::Lost)
                } else if event.target == duel.with {
                    Some(Moment::Won)
                } else {
                    None
                }
            });
            let outcome = outcome.or_else(|| {
                // Ended with the player dead and no obituary seen: lost.
                (duel.ended.is_none() && seen.opponent.is_none() && seen.dead)
                    .then_some(Moment::Lost)
            });
            if let Some(moment) = outcome {
                self.duel = None;
                return self.allow(moment, duel.with, seen.now);
            }
        }
        match (seen.opponent, self.duel.as_mut()) {
            (Some(with), Some(duel)) if duel.with == with && duel.ended.is_none() => None,
            (Some(with), _) => {
                self.duel = Some(Duel { with, ended: None });
                self.allow(Moment::Start, with, seen.now)
            }
            (None, Some(duel)) => {
                match duel.ended {
                    None => duel.ended = Some(seen.now),
                    // It ended some other way (the opponent left, a map change).
                    Some(at) if seen.now.duration_since(at) > LINGER => self.duel = None,
                    Some(_) => {}
                }
                None
            }
            (None, None) => None,
        }
    }

    /// `moment`'s line to `to`, unless one of its kind went less than [`GAP`] ago.
    fn allow(&mut self, moment: Moment, to: u16, now: Instant) -> Option<Due> {
        let last = match moment {
            Moment::Start => &mut self.last_start,
            Moment::Won | Moment::Lost => &mut self.last_end,
        };
        if last.is_some_and(|at| now.duration_since(at) < GAP) {
            return None;
        }
        *last = Some(now);
        Some(Due { moment, to })
    }
}

/// The text to whisper for a cvar's value: trimmed, nothing when empty.
pub(crate) fn line_text(value: &str) -> Option<&str> {
    let text = value.trim();
    (!text.is_empty()).then_some(text)
}

impl crate::GpuState {
    /// Whisper the duel line that fell due this frame, when its cvar holds one and the
    /// opponent is still in the game.
    pub(crate) fn send_duel_line(&mut self) {
        let Some(due) = self.duel_lines.take_due() else {
            return;
        };
        let Some(text) = self
            .console
            .as_ref()
            .and_then(|console| console.text_cvar(due.moment.cvar()).ok())
            .and_then(line_text)
            .map(str::to_owned)
        else {
            return;
        };
        let Some(session) = self.live_session.as_ref() else {
            return;
        };
        self.chat.update_roster(
            self.resident
                .session
                .as_ref()
                .unwrap_or(session)
                .game_state(),
        );
        if !self.chat.in_game(due.to) {
            return;
        }
        let to_all = self
            .console
            .as_ref()
            .and_then(|console| console.bool_cvar(TO_ALL_CVAR))
            .unwrap_or(false);
        let destination = if to_all {
            sjk_client::ChatDestination::Global
        } else {
            sjk_client::ChatDestination::Player(due.to)
        };
        if let Some(command) = sjk_client::chat_command_full(destination, &text) {
            self.send_chat_command(&command, None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ME: u16 = 2;
    const FOE: u16 = 5;

    fn kill(target: u16, attacker: u16) -> ObituaryEvent {
        ObituaryEvent {
            target,
            attacker,
            means_of_death: 3,
            message: "",
            attacker_message: None,
            local_fragged: attacker == ME,
            local_was_killed: target == ME && attacker != ME,
            server_time: 0,
            attacker_force: 0,
        }
    }

    fn seen(
        opponent: Option<u16>,
        dead: bool,
        obituaries: &[ObituaryEvent],
        now: Instant,
    ) -> Seen<'_> {
        Seen {
            me: ME,
            opponent,
            dead,
            obituaries,
            now,
        }
    }

    fn run(lines: &mut DuelLines, seen: &Seen<'_>) -> Option<Due> {
        lines.observe(seen);
        lines.take_due()
    }

    fn start() -> Due {
        Due {
            moment: Moment::Start,
            to: FOE,
        }
    }

    #[test]
    fn a_duel_sends_its_start_once_and_its_win() {
        let t = Instant::now();
        let mut lines = DuelLines::default();
        assert_eq!(run(&mut lines, &seen(None, false, &[], t)), None);
        assert_eq!(
            run(&mut lines, &seen(Some(FOE), false, &[], t)),
            Some(start())
        );
        // Every snapshot of the duel after the first sends nothing.
        for step in 1..20 {
            let now = t + Duration::from_millis(50 * step);
            assert_eq!(run(&mut lines, &seen(Some(FOE), false, &[], now)), None);
        }
        let end = t + Duration::from_secs(20);
        let won = Due {
            moment: Moment::Won,
            to: FOE,
        };
        assert_eq!(
            run(&mut lines, &seen(None, false, &[kill(FOE, ME)], end)),
            Some(won)
        );
        assert_eq!(run(&mut lines, &seen(None, false, &[], end)), None);
    }

    #[test]
    fn a_loss_is_told_by_the_obituary_or_by_dying() {
        let t = Instant::now();
        let lost = Due {
            moment: Moment::Lost,
            to: FOE,
        };
        let mut lines = DuelLines::default();
        run(&mut lines, &seen(Some(FOE), false, &[], t));
        let end = t + Duration::from_secs(15);
        assert_eq!(
            run(&mut lines, &seen(None, true, &[kill(ME, FOE)], end)),
            Some(lost)
        );

        // No obituary at all, but the duel ended with the player dead.
        let mut lines = DuelLines::default();
        run(&mut lines, &seen(Some(FOE), false, &[], t));
        assert_eq!(run(&mut lines, &seen(None, true, &[], end)), Some(lost));
    }

    #[test]
    fn the_obituary_may_come_a_snapshot_late() {
        let t = Instant::now();
        let mut lines = DuelLines::default();
        run(&mut lines, &seen(Some(FOE), false, &[], t));
        let end = t + Duration::from_secs(15);
        assert_eq!(run(&mut lines, &seen(None, false, &[], end)), None);
        let late = end + Duration::from_millis(100);
        assert_eq!(
            run(&mut lines, &seen(None, false, &[kill(FOE, 9)], late)),
            Some(Due {
                moment: Moment::Won,
                to: FOE
            })
        );
    }

    #[test]
    fn a_duel_ended_otherwise_sends_no_end_line() {
        let t = Instant::now();
        let mut lines = DuelLines::default();
        run(&mut lines, &seen(Some(FOE), false, &[], t));
        let end = t + Duration::from_secs(15);
        assert_eq!(run(&mut lines, &seen(None, false, &[], end)), None);
        let later = end + LINGER + Duration::from_millis(1);
        assert_eq!(run(&mut lines, &seen(None, false, &[], later)), None);
        // A death long after the duel is no loss of it.
        let death = later + Duration::from_secs(1);
        assert_eq!(
            run(&mut lines, &seen(None, true, &[kill(ME, FOE)], death)),
            None
        );
    }

    #[test]
    fn other_deaths_do_not_end_the_duel() {
        let t = Instant::now();
        let mut lines = DuelLines::default();
        run(&mut lines, &seen(Some(FOE), false, &[], t));
        let now = t + Duration::from_secs(1);
        assert_eq!(
            run(&mut lines, &seen(Some(FOE), false, &[kill(7, 8)], now)),
            None
        );
    }

    #[test]
    fn lines_of_a_kind_keep_their_gap() {
        let t = Instant::now();
        let mut lines = DuelLines::default();
        assert!(run(&mut lines, &seen(Some(FOE), false, &[], t)).is_some());
        let quick = t + Duration::from_secs(2);
        // A duel won at once: its end line still goes (another kind).
        assert!(run(&mut lines, &seen(None, false, &[kill(FOE, ME)], quick)).is_some());
        // The next duel at once: neither its start nor its end goes.
        let again = quick + Duration::from_secs(1);
        assert_eq!(run(&mut lines, &seen(Some(FOE), false, &[], again)), None);
        let over = again + Duration::from_secs(1);
        assert_eq!(
            run(&mut lines, &seen(None, false, &[kill(FOE, ME)], over)),
            None
        );
        // Once the gap has passed, both go again.
        let later = t + GAP + Duration::from_secs(5);
        assert!(run(&mut lines, &seen(Some(FOE), false, &[], later)).is_some());
        let end = later + Duration::from_secs(3);
        assert!(run(&mut lines, &seen(None, false, &[kill(ME, FOE)], end)).is_some());
    }

    #[test]
    fn a_new_opponent_is_a_new_duel() {
        let t = Instant::now();
        let mut lines = DuelLines::default();
        run(&mut lines, &seen(Some(FOE), false, &[], t));
        let later = t + GAP + Duration::from_secs(1);
        assert_eq!(
            run(&mut lines, &seen(Some(9), false, &[], later)),
            Some(Due {
                moment: Moment::Start,
                to: 9
            })
        );
    }

    #[test]
    fn empty_lines_are_off() {
        assert_eq!(line_text(""), None);
        assert_eq!(line_text("   "), None);
        assert_eq!(line_text(" gg wp "), Some("gg wp"));
    }

    #[test]
    fn a_followed_players_duel_is_not_ours() {
        let mut player = sjk_protocol::PlayerState::default();
        let seen = Seen::from_player(&player, ME, &[], Instant::now());
        assert_eq!(seen.opponent, None);
        player.set_client_num(ME);
        assert!(player.set_raw_field(119, 1));
        assert!(player.set_raw_field(44, u32::from(FOE)));
        player.stats[0] = 100;
        let seen = Seen::from_player(&player, ME, &[], Instant::now());
        assert_eq!(seen.opponent, Some(FOE));
        assert!(!seen.dead);
        player.set_movement_flags(sjk_game_jka::follow::PMF_FOLLOW);
        let seen = Seen::from_player(&player, ME, &[], Instant::now());
        assert_eq!(seen.opponent, None);
    }
}
