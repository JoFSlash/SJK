//! jaPRO command policy from TaystJK `2d60c0fa5`, multiplayer `codemp/game/bg_pmove.c`.
//! These rules interpret existing snapshot slots only for the advertised jaPRO dialect.
use sjk_protocol::{GameState, InfoString, PlayerState, ServerDialect, ServerProfile};

use crate::pmove::MovementState;
use crate::pmove_roll::{BOTH_ROLL_F, BOTH_ROLL_R};

/// TaystJK `codemp/game/bg_public.h` stat and movement-style ordinals.
const STAT_RACEMODE: usize = 11;
const STAT_MOVEMENTSTYLE: usize = 13;
const STAT_RESTRICTIONS: usize = 12;
const STAT_JUMPTIME: usize = 14;
const STAT_LASTJUMPSPEED: usize = 10;
pub(crate) const RESTRICT_BHOP: u32 = 1;
pub(crate) const RESTRICT_CROUCHJUMP: u32 = 1 << 1;
pub(crate) const RESTRICT_SUPERJUMP: u32 = 1 << 4;
/// jaPRO's existing `cp_pluginDisable` only-bhop preference bit.
pub const PLUGIN_BHOP: u32 = 1 << 19;
const MV_OCPM: u32 = 16;
const TEAM_SPECTATOR: u8 = 3;
/// `JAPRO_CINFO_HIGHFPSFIX` in that header's advertised capability table.
const HIGH_FPS_FIX: u32 = 1 << 21;

/// Snapshot metadata used by jaPRO's command wrapper. Other dialects may reuse
/// these stat slots; callers must select [`JaProRules`] before interpreting them.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct JaProState {
    /// `STAT_RACEMODE`, stat slot 11.
    pub race: bool,
    /// `STAT_MOVEMENTSTYLE`, stat slot 13; `MV_OCPM` is 16.
    pub style: u32,
    /// `STAT_RESTRICTIONS`, stat slot 12; interpreted only for jaPRO.
    pub restrictions: u32,
    /// `STAT_JUMPTIME`, stat slot 14, in milliseconds.
    pub jump_time: i32,
    /// `STAT_LASTJUMPSPEED`, stat slot 10: integer takeoff speed before superjump.
    pub last_jump_speed: i32,
    /// Existing protocol-26 jetpackFuel field (39), read only from snapshots.
    pub jetpack_fuel: i32,
    /// Runtime ownership of the new active flag, selected by current server policy.
    /// This is never inferred from the snapshot's flags or stat-slot contents.
    pub predicts_jetpack_flag: bool,
}

impl JaProState {
    /// Retain the existing decoded snapshot slots without changing the wire format.
    pub fn from_player_state(player: &PlayerState) -> Self {
        Self {
            race: player.stats[STAT_RACEMODE] != 0,
            style: player.stats[STAT_MOVEMENTSTYLE],
            restrictions: player.stats[STAT_RESTRICTIONS],
            jump_time: player.stats[STAT_JUMPTIME] as i32,
            last_jump_speed: player.stats[STAT_LASTJUMPSPEED] as i32,
            jetpack_fuel: player.raw_field(39).unwrap_or(0) as i32,
            predicts_jetpack_flag: false,
        }
    }
}

/// Advertised jaPRO policy. The default leaves the stock predictor unchanged.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct JaProRules {
    /// Interpret jaPRO movement metadata and run its command policy.
    pub enabled: bool,
    /// `JAPRO_CINFO_HIGHFPSFIX`, `jcinfo` bit 21.
    pub high_fps_fix: bool,
    /// `JAPRO_CINFO_HEADSLIDE`, `jcinfo` bit 5: standing on a player has no friction.
    pub head_slide: bool,
    /// `JAPRO_CINFO_BHOP1`, bit 10: the server forces ordinary-height bunnyhops.
    pub forced_bhop: bool,
    /// `JAPRO_CINFO_BHOP2`, bit 26: the server supports the player's opt-in.
    pub optional_bhop: bool,
    /// Existing `cp_pluginDisable` bit 19; refreshed from the player's setting.
    pub player_bhop: bool,
    /// `JAPRO_CINFO_JETPACK`, bit 12, enables the new pack outside race mode.
    pub jetpack: bool,
}

impl JaProRules {
    /// Server-forced or player-selected only-bhop policy; foreign stat slots
    /// and player settings never activate it without the current jaPRO dialect.
    pub(crate) fn only_bhop(self, state: JaProState) -> bool {
        self.enabled
            && (self.forced_bhop
                || self.optional_bhop && self.player_bhop
                || state.restrictions & RESTRICT_BHOP != 0)
    }

    /// Resolve the existing server dialect once when serverinfo changes.
    pub fn from_game_state(game: &GameState) -> Self {
        let Some(info) = game
            .config_string(0)
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
            .and_then(|text| InfoString::parse(text).ok())
        else {
            return Self::default();
        };
        match ServerProfile::from_server_info(&info).dialect {
            ServerDialect::TaystJk {
                ja_pro_capabilities,
                ..
            } => Self {
                enabled: true,
                high_fps_fix: ja_pro_capabilities.contains(HIGH_FPS_FIX),
                head_slide: ja_pro_capabilities.contains(1 << 5),
                forced_bhop: ja_pro_capabilities.contains(1 << 10),
                optional_bhop: ja_pro_capabilities.contains(1 << 26),
                player_bhop: false,
                jetpack: ja_pro_capabilities.contains(1 << 12),
            },
            _ => Self::default(),
        }
    }

    /// `Pmove`: race commands bypass fixed/66 ms chopping, except active rolls
    /// and OCPM, which use at most 8 ms. Re-evaluate after every slice.
    pub(crate) fn slice_millis(
        self,
        state: &MovementState,
        remaining: i32,
        fixed: Option<i32>,
    ) -> i32 {
        let maximum = if self.enabled && state.ja_pro.race {
            if state.ja_pro.style == MV_OCPM
                || state.legs_timer > 0 && matches!(state.legs_anim, BOTH_ROLL_F..=BOTH_ROLL_R)
            {
                8
            } else {
                remaining
            }
        } else {
            fixed.map_or(66, |value| value.clamp(8, 33))
        };
        remaining.min(maximum)
    }

    /// `PmoveSingle` final velocity policy. This runs only on paths that reach
    /// the end of the move; spectator/noclip early returns remain early returns.
    pub(crate) fn snap_velocity(
        self,
        state: &MovementState,
        millis: i32,
        stock_snap: bool,
    ) -> bool {
        if !self.enabled {
            return stock_snap;
        }
        if state.team == TEAM_SPECTATOR {
            return true;
        }
        if state.ja_pro.race && state.ja_pro.style != MV_OCPM {
            return false;
        }
        if self.high_fps_fix && !(4..=25).contains(&millis) {
            return false;
        }
        stock_snap
    }

    /// Race wallbug prevention happens before snapping: an unchanged origin
    /// cannot retain an accumulating velocity. Spectators are excluded.
    pub(crate) fn finish_velocity(
        self,
        state: &mut MovementState,
        previous_origin: [f32; 3],
        millis: i32,
        stock_snap: bool,
    ) {
        if self.enabled
            && state.ja_pro.race
            && state.team != TEAM_SPECTATOR
            && state.origin == previous_origin
        {
            state.velocity = [0.0; 3];
        }
        if self.snap_velocity(state, millis, stock_snap) {
            state.velocity = state.velocity.map(f32::round_ties_even);
        }
    }
}

#[cfg(test)]
#[path = "pmove_japro_tests.rs"]
mod tests;
