//! Match timer and warmup projections from configstring-backed cgame state.

use sjk_protocol::GameState;

const CS_WARMUP: usize = 5;
const CS_LEVEL_START_TIME: usize = 21;

/// Elapsed match clock projected exactly as `CG_DrawTimer`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MatchClock {
    /// Whole elapsed minutes.
    pub minutes: i32,
    /// Seconds within the current minute.
    pub seconds: i32,
}

/// Warmup text state from `CG_DrawWarmup`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WarmupText {
    /// No warmup message is active.
    Hidden,
    /// The server is waiting for enough players.
    WaitingForPlayers,
    /// Match start countdown in displayed seconds.
    StartsIn(i32),
}

/// Read `CS_LEVEL_START_TIME` on every projection, so `cs` updates take effect.
pub fn legacy_match_clock(game: &GameState, server_time: i32) -> MatchClock {
    let start = config_i32(game, CS_LEVEL_START_TIME);
    let total = server_time.wrapping_sub(start) / 1_000;
    MatchClock {
        minutes: total / 60,
        seconds: total % 60,
    }
}

/// Read `CS_WARMUP` and reproduce the displayed `sec + 1` countdown.
///
/// With no warmup, a siege round still to begin is told the same way, from
/// `CS_SIEGE_STATE` (`CG_ParseSiegeState`, `cg_draw.c:8168-8226`): `1` waits for
/// players on both teams, `2|time` counts down `SIEGE_ROUND_BEGIN_TIME` from `time`.
pub fn legacy_warmup_text(game: &GameState, server_time: i32) -> WarmupText {
    let warmup = config_i32(game, CS_WARMUP);
    if warmup == 0 {
        siege_round_text(game, server_time)
    } else if warmup < 0 {
        WarmupText::WaitingForPlayers
    } else {
        let seconds = warmup.wrapping_sub(server_time) / 1_000;
        if seconds < 0 {
            WarmupText::Hidden
        } else {
            WarmupText::StartsIn(seconds + 1)
        }
    }
}

/// `CS_SIEGE_STATE` before the round begins (see [`legacy_warmup_text`]).
fn siege_round_text(game: &GameState, server_time: i32) -> WarmupText {
    let Some((state, time)) = game
        .config_string(sjk_game_jka::siege_class::CS_SIEGE_STATE)
        .and_then(parse_siege_state)
    else {
        return WarmupText::Hidden;
    };
    match state {
        1 => WarmupText::WaitingForPlayers,
        2 => WarmupText::StartsIn(siege_round_seconds(
            server_time.wrapping_sub(time.unwrap_or(0)),
        )),
        _ => WarmupText::Hidden,
    }
}

/// `CG_ParseSiegeState` (`cg_main.c:540-578`): `CS_SIEGE_STATE`'s round state and,
/// after a `|`, the server time it took effect. `None` for an empty string, which
/// the reference never parses (`cg_servercmds.c:871-877`, `cg_main.c:964-967`).
pub(crate) fn parse_siege_state(text: &[u8]) -> Option<(i32, Option<i32>)> {
    if text.is_empty() {
        return None;
    }
    let mut parts = text.splitn(2, |byte| *byte == b'|');
    let state = crate::local_sounds::atoi(parts.next().unwrap_or_default());
    Some((state, parts.next().map(crate::local_sounds::atoi)))
}

/// The whole seconds `CG_Draw2D` shows, and counts aloud from three, `since`
/// milliseconds after a siege round began counting in (`cg_draw.c:8180-8199`).
pub(crate) fn siege_round_seconds(since: i32) -> i32 {
    let begin = sjk_game_jka::siege::SIEGE_ROUND_BEGIN_TIME;
    (begin.wrapping_sub(since).clamp(0, begin) / 1_000 + 1).max(1)
}

fn config_i32(game: &GameState, index: usize) -> i32 {
    game.config_string(index)
        .and_then(|value| std::str::from_utf8(value).ok())
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjk_game_jka::siege_class::CS_SIEGE_STATE;

    fn with_siege_state(state: &str) -> GameState {
        let mut game = GameState::empty_local(0);
        game.replace_config_string(CS_SIEGE_STATE, state.as_bytes().to_vec())
            .expect("a configstring");
        game
    }

    #[test]
    fn siege_round_state_shows_as_warmup() {
        assert_eq!(
            legacy_warmup_text(&with_siege_state("1"), 10_000),
            WarmupText::WaitingForPlayers
        );
        // `2|time`: five seconds counted down from `time`, never below one.
        let counting = with_siege_state("2|10000");
        assert_eq!(
            legacy_warmup_text(&counting, 10_000),
            WarmupText::StartsIn(6)
        );
        assert_eq!(
            legacy_warmup_text(&counting, 12_500),
            WarmupText::StartsIn(3)
        );
        assert_eq!(
            legacy_warmup_text(&counting, 20_000),
            WarmupText::StartsIn(1)
        );
        assert_eq!(
            legacy_warmup_text(&with_siege_state("0|10000"), 10_000),
            WarmupText::Hidden
        );
        assert_eq!(
            legacy_warmup_text(&GameState::empty_local(0), 10_000),
            WarmupText::Hidden
        );
    }
}
