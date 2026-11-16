//! A siege round's result as the legacy wire tells it: `CS_SIEGE_STATE` ("3|time" once
//! `SiegeRoundComplete` ended the round), the `EV_SIEGE_ROUNDOVER` naming its winner, and
//! `CS_SIEGE_WINTEAM`. Siege scores no team, so the team scores cannot tell.

use crate::SiegeRoundOver;
use sjk_game_jka::siege_class::{
    CS_SIEGE_STATE, CS_SIEGE_WINTEAM, SIEGETEAM_TEAM1, SIEGETEAM_TEAM2,
};
use sjk_protocol::GameState;

/// How far the round-over event may be from the time `CS_SIEGE_STATE` gives the round's
/// end, both the server's clock, for the event to be this round's.
const SAME_ROUND_MS: u32 = 2_000;

/// The siege team that won the round now over (`SIEGETEAM_TEAM1`, red, or
/// `SIEGETEAM_TEAM2`, blue): while `CS_SIEGE_STATE` says the round ended, the winner of
/// the `EV_SIEGE_ROUNDOVER` seen at that time; else, after the second round of a match
/// whose teams switch, the match's winner (`CS_SIEGE_WINTEAM`, "0" otherwise). A player
/// who joined after the event of any other round learns no winner.
pub fn siege_round_winner(game: &GameState, round_over: Option<SiegeRoundOver>) -> Option<u8> {
    let text = |index| {
        game.config_string(index)
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
    };
    let ended = text(CS_SIEGE_STATE)?.strip_prefix('3')?;
    let ended_at = ended
        .strip_prefix('|')
        .and_then(|time| time.parse::<i32>().ok());
    let side = |team: u8| matches!(team, SIEGETEAM_TEAM1 | SIEGETEAM_TEAM2).then_some(team);
    round_over
        .filter(|round| {
            ended_at.is_some_and(|time| round.server_time.abs_diff(time) <= SAME_ROUND_MS)
        })
        .and_then(|round| side(round.winner))
        .or_else(|| {
            text(CS_SIEGE_WINTEAM)
                .and_then(|team| team.parse::<u8>().ok())
                .and_then(side)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_siege_round_names_its_winner_once_it_is_over() {
        let mut game = GameState::empty_local(0);
        let won = |winner, server_time| {
            Some(SiegeRoundOver {
                winner,
                server_time,
            })
        };
        // Still running: no winner, whatever was seen before.
        game.replace_config_string(CS_SIEGE_STATE, b"0|1000".to_vec())
            .unwrap();
        assert_eq!(siege_round_winner(&game, won(2, 90_000)), None);
        // Over at 90 s: the event of that moment names the side.
        game.replace_config_string(CS_SIEGE_STATE, b"3|90000".to_vec())
            .unwrap();
        assert_eq!(siege_round_winner(&game, won(2, 90_050)), Some(2));
        assert_eq!(siege_round_winner(&game, won(1, 90_000)), Some(1));
        // An earlier round's event is not this one's; nor is a team that is no side.
        assert_eq!(siege_round_winner(&game, won(2, 30_000)), None);
        assert_eq!(siege_round_winner(&game, won(3, 90_000)), None);
        // A time at the edge of the integers does not overflow.
        game.replace_config_string(CS_SIEGE_STATE, b"3|-2147483648".to_vec())
            .unwrap();
        assert_eq!(siege_round_winner(&game, won(2, i32::MAX)), None);
        // After the switch, the match's winner even without the event.
        game.replace_config_string(CS_SIEGE_WINTEAM, b"1".to_vec())
            .unwrap();
        assert_eq!(siege_round_winner(&game, None), Some(1));
    }
}
