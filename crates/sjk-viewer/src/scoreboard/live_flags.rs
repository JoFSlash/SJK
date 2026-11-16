//! Carried flags on the scoreboard rows from live state, not only from `scores`.
//!
//! `CG_DrawClientScore` draws the flag from `clientInfo_t::powerups`, which
//! `CG_ParseScores` and `CG_ParseTeamInfo` fill (`cg_servercmds.c`). A `scores`
//! row is only as fresh as the last request (one per scoreboard press), so a flag
//! taken or captured while the board is open never showed. Each frame the rows'
//! flag bits are corrected from the freshest source the client has, in order:
//! the viewed player's own state, a player entity in the snapshot, a teammate's
//! `tinfo` row, then the `scores` row. `CS_FLAGSTATUS` clears a flag nobody can
//! still be holding (its status is not taken).

use super::ScoreRow;
use sjk_client::TeamInfo;
use sjk_protocol::{GameState, Snapshot};

/// `CS_FLAGSTATUS` (`bg_public.h`): red then blue, `'1'` while carried.
const CS_FLAGSTATUS: usize = 23;
/// `PW_REDFLAG`, `PW_BLUEFLAG`, `PW_NEUTRALFLAG`.
const PW_REDFLAG: u32 = 4;
const PW_BLUEFLAG: u32 = 5;
const PW_NEUTRALFLAG: u32 = 6;
const FLAG_BITS: u32 = 1 << PW_REDFLAG | 1 << PW_BLUEFLAG | 1 << PW_NEUTRALFLAG;
/// `entityType_t::ET_PLAYER`.
const ET_PLAYER: u8 = 1;

/// Correct every row's flag bits from `snapshot`, the teammates' `team` rows and
/// the flag status in `game`. Allocates nothing; at most 32 rows.
pub(super) fn apply(
    rows: &mut [ScoreRow],
    snapshot: &Snapshot,
    team: &[TeamInfo],
    game: &GameState,
) {
    let allowed = allowed_bits(game.config_string(CS_FLAGSTATUS));
    let viewed = snapshot.player.client_num();
    for row in rows {
        let live = if u16::from(row.client_num) == viewed {
            Some(player_bits(&snapshot.player.powerups))
        } else if let Some(state) = snapshot
            .entities
            .binary_search_by_key(&u16::from(row.client_num), |state| state.number())
            .ok()
            .map(|index| &snapshot.entities[index])
            .filter(|state| state.entity_type() == ET_PLAYER)
        {
            Some(state.powerups())
        } else {
            team.iter()
                .find(|info| info.client_num == row.client_num)
                .map(|info| info.powerups as u32)
        };
        let flags = live.unwrap_or(row.powerups) & FLAG_BITS & allowed;
        row.powerups = (row.powerups & !FLAG_BITS) | flags;
    }
}

/// `BG_PlayerStateToEntityState`'s powerup bits: any powerup with a time set.
fn player_bits(powerups: &[u32]) -> u32 {
    powerups
        .iter()
        .enumerate()
        .filter(|(_, until)| **until != 0)
        .fold(0, |bits, (index, _)| bits | 1 << index)
}

/// The flag bits a carrier may still show: a red or blue flag whose status is not
/// taken is at its base or dropped, so nobody holds it. Without a status (no
/// capture mode, or an older server) every flag bit stays.
fn allowed_bits(status: Option<&[u8]>) -> u32 {
    let Some(status) = status.filter(|status| status.len() >= 2) else {
        return FLAG_BITS;
    };
    let mut bits = 1 << PW_NEUTRALFLAG;
    if status[0] == b'1' {
        bits |= 1 << PW_REDFLAG;
    }
    if status[1] == b'1' {
        bits |= 1 << PW_BLUEFLAG;
    }
    bits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_bits_follow_set_times() {
        let mut powerups = [0_u32; 16];
        powerups[PW_BLUEFLAG as usize] = i32::MAX as u32;
        assert_eq!(player_bits(&powerups), 1 << PW_BLUEFLAG);
    }

    #[test]
    fn flag_status_clears_flags_nobody_holds() {
        assert_eq!(allowed_bits(None), FLAG_BITS);
        assert_eq!(allowed_bits(Some(b"")), FLAG_BITS);
        // Red at base, blue taken.
        assert_eq!(
            allowed_bits(Some(b"01")),
            1 << PW_BLUEFLAG | 1 << PW_NEUTRALFLAG
        );
        // Red taken, blue dropped.
        assert_eq!(
            allowed_bits(Some(b"12")),
            1 << PW_REDFLAG | 1 << PW_NEUTRALFLAG
        );
    }
}
