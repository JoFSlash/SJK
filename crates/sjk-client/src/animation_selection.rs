//! BaseJKA humanoid animation selection compatibility.
//!
//! The selection path mirrors OpenJK `codemp/cgame/cg_players.c`:
//! `CG_PlayerAnimation` passes `entityState.legsAnim` and `torsoAnim` directly
//! to `CG_RunLerpFrame`; `legsFlip`/`torsoFlip` force a restart without changing
//! the clip; and `forceFrame` freezes both tracks. `CG_SetLerpFrameAnimation`
//! applies locomotion/rage speed scaling, saber transition scaling, synchronized
//! torso/legs starts, and death/flip blend rules. For the predicted local player,
//! `codemp/game/bg_misc.c::BG_PlayerStateToEntityState` copies the same animation
//! and flip fields before cgame presents it.
//!
//! `playerState.legsTimer` and `torsoTimer` are game-side guards that decide
//! when bg_pmove may select a new animation. They are not copied into
//! `entityState` and are not consulted by `CG_PlayerAnimation`; presentation
//! therefore selects the declared clips rather than deriving clips from timers.
//! View pitch is likewise a bone-angle contribution in `CG_G2PlayerAngles`, not
//! an animation-clip selection rule.
//!
//! Presentation additionally replaces the stock Tavion/Desann idle clips with
//! JoF EJK #47's poses; [`crate::idle_stances`] never edits the simulation input.

use sjk_protocol::{EntityState, PlayerState};
use sjk_runtime::{AnimationState, AnimationTrackInput, EntityId, World};

use crate::pmove::MovementState;
/// Fully decoded input to codemp's two-track player animation selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LegacyAnimationSelection {
    lower: LegacyTrackSelection,
    upper: LegacyTrackSelection,
    forced_frame: Option<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LegacyTrackSelection {
    clip: usize,
    revision: u64,
    speed_milli: u16,
}

impl LegacyAnimationSelection {
    /// Decode an on-wire entity, then apply the client presentation remaps.
    pub(crate) fn from_entity(state: &EntityState) -> Self {
        let (legs, torso) = ejk_animation_fixes(
            state.leg_animation(),
            state.torso_animation(),
            state.weapon(),
            state.saber_in_flight(),
            state.entity_type() == crate::npc_identity::ET_NPC,
        );
        let (legs, torso) = crate::idle_stances::present(
            legs,
            torso,
            state.weapon(),
            state.saber_in_flight(),
            state.saber_holstered(),
            state.fire_flag(),
        );
        let lower_clip = usize::from(legs);
        let upper_clip = usize::from(torso);
        Self {
            lower: LegacyTrackSelection {
                clip: lower_clip,
                revision: animation_revision(legs, state.leg_flip()),
                speed_milli: legacy_animation_speed(
                    lower_clip,
                    state.force_powers_active(),
                    false,
                    state.fire_flag(),
                    state.broken_limbs(),
                ),
            },
            upper: LegacyTrackSelection {
                clip: upper_clip,
                revision: animation_revision(torso, state.torso_flip()),
                speed_milli: legacy_animation_speed(
                    upper_clip,
                    state.force_powers_active(),
                    true,
                    state.fire_flag(),
                    state.broken_limbs(),
                ),
            },
            forced_frame: (state.force_frame() != 0).then_some(usize::from(state.force_frame())),
        }
    }

    /// Decode the local player fields copied by `BG_PlayerStateToEntityState`.
    pub(crate) fn from_player(state: &PlayerState) -> Self {
        let (legs, torso) = ejk_animation_fixes(
            state.leg_animation(),
            state.torso_animation(),
            state.weapon(),
            state.saber_in_flight(),
            false,
        );
        let (legs, torso) = crate::idle_stances::present(
            legs,
            torso,
            state.weapon(),
            state.saber_in_flight(),
            state.saber_holstered(),
            state.saber_style(),
        );
        let lower_clip = usize::from(legs);
        let upper_clip = usize::from(torso);
        Self {
            lower: LegacyTrackSelection {
                clip: lower_clip,
                revision: animation_revision(legs, state.leg_flip()),
                speed_milli: legacy_animation_speed(
                    lower_clip,
                    state.force_powers_active(),
                    false,
                    state.saber_style(),
                    state.broken_limbs(),
                ),
            },
            upper: LegacyTrackSelection {
                clip: upper_clip,
                revision: animation_revision(torso, state.torso_flip()),
                speed_milli: legacy_animation_speed(
                    upper_clip,
                    state.force_powers_active(),
                    true,
                    state.saber_style(),
                    state.broken_limbs(),
                ),
            },
            forced_frame: None,
        }
    }

    /// Decode the predicted fields copied by `BG_PlayerStateToEntityState`.
    fn from_movement(state: &MovementState) -> Self {
        let (legs, torso) = ejk_animation_fixes(
            state.legs_anim,
            state.torso_anim,
            state.weapon,
            state.saber_in_flight,
            false,
        );
        let (legs, torso) = crate::idle_stances::present(
            legs,
            torso,
            state.weapon,
            state.saber_in_flight,
            state.saber_holstered,
            state.saber_anim_level,
        );
        let lower_clip = usize::from(legs);
        let upper_clip = usize::from(torso);
        Self {
            lower: LegacyTrackSelection {
                clip: lower_clip,
                revision: animation_revision(legs, state.legs_flip),
                speed_milli: legacy_animation_speed(
                    lower_clip,
                    state.force_powers_active,
                    false,
                    state.saber_anim_level,
                    state.broken_limbs,
                ),
            },
            upper: LegacyTrackSelection {
                clip: upper_clip,
                revision: animation_revision(torso, state.torso_flip),
                speed_milli: legacy_animation_speed(
                    upper_clip,
                    state.force_powers_active,
                    true,
                    state.saber_anim_level,
                    state.broken_limbs,
                ),
            },
            forced_frame: None,
        }
    }

    /// Apply both selected tracks at the snapshot's cgame presentation time.
    pub(crate) fn apply(self, world: &mut World, entity_id: EntityId, time_millis: i64) {
        let previous = world
            .entity(entity_id)
            .and_then(|entity| entity.animation());
        trace_selection(entity_id, previous, self);
        world.set_animation(
            entity_id,
            animation_track(
                self.lower,
                previous.map(|animation| animation.lower.clip),
                self.forced_frame,
                synchronized_phase(previous, self.lower.clip, false, time_millis),
            ),
            animation_track(
                self.upper,
                previous.map(|animation| animation.upper.clip),
                self.forced_frame,
                synchronized_phase(previous, self.upper.clip, true, time_millis),
            ),
            time_millis,
        );
    }
}

/// Build renderer-neutral animation tracks from `cg.predictedPlayerState`.
///
/// OpenJK copies `legsAnim`, `torsoAnim`, `legsFlip`, and `torsoFlip` in
/// `BG_PlayerStateToEntityState` (`codemp/game/bg_misc.c:2762-2830`) before
/// presenting the local entity (`codemp/cgame/cg_ents.c:3457`).
/// SJK's default Tavion/Desann idle remaps affect only these output tracks.
pub fn legacy_predicted_animation_inputs(
    state: &MovementState,
    previous: Option<AnimationState>,
    command_time: i64,
) -> [AnimationTrackInput; 2] {
    let selection = LegacyAnimationSelection::from_movement(state);
    [
        animation_track(
            selection.lower,
            previous.map(|animation| animation.lower.clip),
            None,
            synchronized_phase(previous, selection.lower.clip, false, command_time),
        ),
        animation_track(
            selection.upper,
            previous.map(|animation| animation.upper.clip),
            None,
            synchronized_phase(previous, selection.upper.clip, true, command_time),
        ),
    ]
}

/// EternalJK's "hack to fix bugged player animations" (`CG_Player`,
/// `codemp/cgame/cg_players.c:10665-10730` at EternalJK a40e793),
/// applied to a player's `legsAnim`/`torsoAnim` before they are presented. For a
/// player (not an NPC) without a saber in hand (any other weapon, or the saber thrown):
/// the two-handed and dual runs and walks play as the ordinary ones; the staff ones
/// too (EternalJK keeps them only in jaPRO race mode, which this client does not
/// play); a thrown saber's standing torso follows the legs; the old Bryar's
/// `BOTH_STAND1` (servers without `g_fixWeaponAttackAnim` fire it with that, arm
/// hanging) shows as `BOTH_ATTACK2`, its arm raised; and the concussion rifle's
/// `BOTH_ATTACK2` as `BOTH_ATTACK3`. The revision follows the mapped animation, so
/// the Bryar's mapped stand does not restart its attack.
pub(crate) fn ejk_animation_fixes(
    legs: u16,
    torso: u16,
    weapon: u8,
    saber_in_flight: bool,
    npc: bool,
) -> (u16, u16) {
    const WP_SABER: u8 = 3;
    const WP_CONCUSSION: u8 = 15;
    const WP_BRYAR_OLD: u8 = 16;
    const BOTH_STAND1: u16 = 915;
    const BOTH_ATTACK2: u16 = 114;
    const BOTH_ATTACK3: u16 = 115;
    const RUN1: u16 = 1111;
    const RUNBACK1: u16 = 1136;
    const WALK1: u16 = 1102;
    const WALKBACK1: u16 = 1134;
    if npc || (weapon == WP_SABER && !saber_in_flight) {
        return (legs, torso);
    }
    let ordinary = |animation: u16| match animation {
        // BOTH_RUN2, BOTH_RUN_DUAL, BOTH_RUN_STAFF.
        1114 | 1120 | 1118 => RUN1,
        // BOTH_RUNBACK2, BOTH_RUNBACK_DUAL, BOTH_RUNBACK_STAFF.
        1137 | 1121 | 1119 => RUNBACK1,
        // BOTH_WALK2, BOTH_WALK_DUAL, BOTH_WALK_STAFF.
        1103 | 1106 | 1104 => WALK1,
        // BOTH_WALKBACK2, BOTH_WALKBACK_DUAL, BOTH_WALKBACK_STAFF.
        1135 | 1107 | 1105 => WALKBACK1,
        other => other,
    };
    let (mut legs, mut torso) = (ordinary(legs), ordinary(torso));
    if saber_in_flight && torso == BOTH_STAND1 {
        torso = legs;
    }
    if weapon == WP_BRYAR_OLD && torso == BOTH_STAND1 {
        torso = BOTH_ATTACK2;
    } else if weapon == WP_CONCUSSION {
        if legs == BOTH_ATTACK2 {
            legs = BOTH_ATTACK3;
        }
        if torso == BOTH_ATTACK2 {
            torso = BOTH_ATTACK3;
        }
    }
    (legs, torso)
}

fn animation_revision(animation: u16, flip: bool) -> u64 {
    u64::from(animation) | (u64::from(flip) << 16)
}

/// Mirror `CG_SetLerpFrameAnimation`'s synchronized lower/upper start rule
/// (`codemp/cgame/cg_players.c`): a track starting the clip the other one already
/// plays picks up the other's frame. A torso restarting the clip it already plays (a
/// `torsoFlip`) starts from its first frame instead (`ci->torsoAnim == newAnimation`
/// clears `beginFrame`): a tauntaun rider's repeated `BOTH_VT_ATR_S`, whose legs play the
/// same finished swing, otherwise stayed frozen on its last frame.
fn synchronized_phase(
    previous: Option<AnimationState>,
    clip: usize,
    upper: bool,
    time_millis: i64,
) -> Option<i64> {
    if upper && previous.is_some_and(|animation| animation.upper.clip == clip) {
        return None;
    }
    let other = previous.map(|animation| {
        if upper {
            animation.lower
        } else {
            animation.upper
        }
    })?;
    (other.clip == clip).then(|| other.elapsed_millis(time_millis))
}

/// Mirror `CG_SetLerpFrameAnimation` death/flip blend selection.
fn animation_track(
    selection: LegacyTrackSelection,
    previous_clip: Option<usize>,
    forced_frame: Option<usize>,
    resume_phase_millis: Option<i64>,
) -> AnimationTrackInput {
    let blend_millis = legacy_blend_millis(selection.clip, previous_clip, forced_frame.is_some());
    AnimationTrackInput {
        clip: selection.clip,
        revision: selection.revision,
        speed_milli: selection.speed_milli,
        blend_millis,
        forced_frame,
        resume_phase_millis,
    }
}

pub(crate) fn legacy_blend_millis(
    clip: usize,
    previous_clip: Option<usize>,
    forced_frame: bool,
) -> u16 {
    if forced_frame {
        150
    } else if death_animation(clip) || previous_clip.is_some_and(death_animation) {
        0
    } else if flipping_animation(clip) || previous_clip.is_some_and(flipping_animation) {
        200
    } else {
        100
    }
}

/// Mirror `CG_PlayerAnimation` and `BG_SaberStartTransAnim` speed rules.
fn legacy_animation_speed(
    clip: usize,
    force_powers_active: u32,
    torso: bool,
    saber_style: u8,
    broken_limbs: u8,
) -> u16 {
    const FP_SPEED: u32 = 1 << 2;
    const FP_RAGE: u32 = 1 << 8;
    let mut speed = if torso {
        if force_powers_active & FP_RAGE != 0 {
            1_700
        } else {
            1_000
        }
    } else if !locomotion_animation(clip) {
        1_000
    } else if force_powers_active & FP_RAGE != 0 {
        1_300
    } else if force_powers_active & FP_SPEED != 0 {
        1_700
    } else {
        1_000
    };

    let saber_transition = animation_in_range(clip, "BOTH_T1_BR__R", "BOTH_T1_BL_TL")
        || animation_in_range(clip, "BOTH_T2_BR__R", "BOTH_T2_BL_TL")
        || animation_in_range(clip, "BOTH_T3_BR__R", "BOTH_T3_BL_TL");
    if saber_transition {
        speed = match saber_style {
            1 => speed * 3 / 2,
            3 => speed * 3 / 4,
            _ => speed,
        };
    }
    let saber_animation = animation_in_range(clip, "BOTH_A1_T__B_", "BOTH_H1_S1_BR");
    if saber_transition || saber_animation {
        const BROKEN_LEFT_ARM: u8 = 1 << 1;
        const BROKEN_RIGHT_ARM: u8 = 1 << 2;
        if broken_limbs & BROKEN_RIGHT_ARM != 0 {
            speed /= 2;
        } else if broken_limbs & BROKEN_LEFT_ARM != 0 {
            speed = speed * 65 / 100;
        }
    }
    speed
}

fn animation_in_range(clip: usize, first: &str, last: &str) -> bool {
    let first = crate::legacy_animation::NAMES
        .iter()
        .position(|name| *name == first);
    let last = crate::legacy_animation::NAMES
        .iter()
        .position(|name| *name == last);
    matches!((first, last), (Some(first), Some(last)) if (first..=last).contains(&clip))
}

fn locomotion_animation(clip: usize) -> bool {
    matches!(
        crate::legacy_animation_name(clip),
        Some(
            "BOTH_WALK1"
                | "BOTH_WALK2"
                | "BOTH_WALK_STAFF"
                | "BOTH_WALK_DUAL"
                | "BOTH_WALK5"
                | "BOTH_WALK6"
                | "BOTH_WALK7"
                | "BOTH_WALKBACK1"
                | "BOTH_WALKBACK2"
                | "BOTH_WALKBACK_STAFF"
                | "BOTH_WALKBACK_DUAL"
                | "BOTH_RUN1"
                | "BOTH_RUN2"
                | "BOTH_RUN_STAFF"
                | "BOTH_RUN_DUAL"
                | "BOTH_RUNBACK1"
                | "BOTH_RUNBACK2"
                | "BOTH_RUNBACK_STAFF"
                | "BOTH_RUNBACK_DUAL"
                | "BOTH_RUN1START"
                | "BOTH_RUN1STOP"
                | "BOTH_RUNSTRAFE_LEFT1"
                | "BOTH_RUNSTRAFE_RIGHT1"
                | "BOTH_RUN4"
        )
    )
}

pub(crate) fn death_animation(clip: usize) -> bool {
    let Some(name) = crate::legacy_animation_name(clip) else {
        return false;
    };
    name.starts_with("BOTH_DEATH")
        || name.starts_with("BOTH_DEAD")
        || matches!(
            name,
            "BOTH_LYINGDEATH1"
                | "BOTH_STUMBLEDEATH1"
                | "BOTH_FALLDEATH1"
                | "BOTH_FALLDEATH1INAIR"
                | "BOTH_FALLDEATH1LAND"
                | "BOTH_LYINGDEAD1"
                | "BOTH_STUMBLEDEAD1"
                | "BOTH_FALLDEAD1LAND"
                | "BOTH_DISMEMBER_HEAD1"
                | "BOTH_DISMEMBER_TORSO1"
                | "BOTH_DISMEMBER_LLEG"
                | "BOTH_DISMEMBER_RLEG"
                | "BOTH_DISMEMBER_RARM"
                | "BOTH_DISMEMBER_LARM"
        )
}

fn flipping_animation(clip: usize) -> bool {
    matches!(
        crate::legacy_animation_name(clip),
        Some(
            "BOTH_FLIP_F"
                | "BOTH_FLIP_B"
                | "BOTH_FLIP_L"
                | "BOTH_FLIP_R"
                | "BOTH_WALL_RUN_RIGHT_FLIP"
                | "BOTH_WALL_RUN_LEFT_FLIP"
                | "BOTH_WALL_FLIP_RIGHT"
                | "BOTH_WALL_FLIP_LEFT"
                | "BOTH_FLIP_BACK1"
                | "BOTH_FLIP_BACK2"
                | "BOTH_FLIP_BACK3"
                | "BOTH_WALL_FLIP_BACK1"
                | "BOTH_WALL_RUN_RIGHT"
                | "BOTH_WALL_RUN_LEFT"
                | "BOTH_WALL_RUN_RIGHT_STOP"
                | "BOTH_WALL_RUN_LEFT_STOP"
                | "BOTH_BUTTERFLY_LEFT"
                | "BOTH_BUTTERFLY_RIGHT"
                | "BOTH_BUTTERFLY_FL1"
                | "BOTH_BUTTERFLY_FR1"
                | "BOTH_ARIAL_LEFT"
                | "BOTH_ARIAL_RIGHT"
                | "BOTH_ARIAL_F1"
                | "BOTH_CARTWHEEL_LEFT"
                | "BOTH_CARTWHEEL_RIGHT"
                | "BOTH_JUMPFLIPSLASHDOWN1"
                | "BOTH_JUMPFLIPSTABDOWN"
                | "BOTH_JUMPATTACK6"
                | "BOTH_JUMPATTACK7"
                | "BOTH_FORCEWALLRUNFLIP_END"
                | "BOTH_FORCEWALLRUNFLIP_ALT"
                | "BOTH_FLIP_ATTACK7"
                | "BOTH_A7_SOULCAL"
        )
    )
}

fn trace_selection(
    entity_id: EntityId,
    previous: Option<AnimationState>,
    selection: LegacyAnimationSelection,
) {
    if std::env::var_os("SJK_TRACE_ANIMATIONS").is_none()
        || previous.is_some_and(|animation| {
            animation.lower.clip == selection.lower.clip
                && animation.upper.clip == selection.upper.clip
        })
    {
        return;
    }
    eprintln!(
        "animation entity={} legs={}({}) torso={}({})",
        entity_id.get().saturating_sub(1),
        selection.lower.clip,
        crate::legacy_animation_name(selection.lower.clip).unwrap_or("<unknown>"),
        selection.upper.clip,
        crate::legacy_animation_name(selection.upper.clip).unwrap_or("<unknown>"),
    );
}

#[cfg(test)]
mod synchronized_phase_tests {
    use super::*;
    use sjk_runtime::AnimationTrackState;

    fn playing(clip: usize, started_at_millis: i64) -> AnimationTrackState {
        AnimationTrackState {
            clip,
            revision: clip as u64,
            started_at_millis,
            phase_millis: 0,
            speed_milli: 1_000,
            forced_frame: None,
            transition: None,
        }
    }

    #[test]
    fn a_restarted_torso_swing_starts_from_its_first_frame() {
        // Slash #123: a tauntaun rider holding attack restarts `BOTH_VT_ATR_S` (1087) on
        // both tracks every second (`PM_VehicleWeaponAnimate`, `SETANIM_FLAG_RESTART`).
        // The torso picked up the finished legs' frame and showed one swing only.
        let previous = AnimationState {
            lower: playing(1_087, 0),
            upper: playing(1_087, 0),
        };
        for step in [8, 7, 4, 3] {
            let restart = 1_000 + step;
            assert_eq!(
                synchronized_phase(Some(previous), 1_087, true, restart),
                None
            );
            // The legs still pick up the torso's frame (`lower_lumbar`), as codemp's do.
            assert_eq!(
                synchronized_phase(Some(previous), 1_087, false, restart),
                Some(restart)
            );
        }
        // A torso entering the clip the legs already play still joins their frame.
        let entering = AnimationState {
            lower: playing(1_087, 0),
            upper: playing(1_083, 0),
        };
        assert_eq!(
            synchronized_phase(Some(entering), 1_087, true, 400),
            Some(400)
        );
    }
}

#[cfg(test)]
mod ejk_fix_tests {
    use super::ejk_animation_fixes;

    #[test]
    fn the_old_bryar_keeps_its_arm_raised_between_shots() {
        // An unfixed server fires the old Bryar (16) with BOTH_STAND1 (915).
        assert_eq!(ejk_animation_fixes(915, 915, 16, false, false), (915, 114));
        // The legs still stand; an NPC is left alone.
        assert_eq!(ejk_animation_fixes(915, 915, 16, false, true), (915, 915));
    }

    #[test]
    fn the_concussion_fires_with_its_own_attack() {
        assert_eq!(ejk_animation_fixes(114, 114, 15, false, false), (115, 115));
    }

    #[test]
    fn a_held_saber_keeps_its_runs_and_a_gun_takes_the_ordinary_ones() {
        assert_eq!(
            ejk_animation_fixes(1114, 1114, 3, false, false),
            (1114, 1114)
        );
        assert_eq!(
            ejk_animation_fixes(1114, 1137, 5, false, false),
            (1111, 1136)
        );
        // A thrown saber's standing torso follows the legs.
        assert_eq!(ejk_animation_fixes(1102, 915, 3, true, false), (1102, 1102));
    }
}

#[cfg(test)]
mod idle_stance_tests {
    use super::*;

    #[test]
    fn snapshots_players_and_prediction_select_the_same_idle_pose() {
        for (style, stock, expected) in [(5, 850, 665), (4, 917, 942)] {
            let mut entity = EntityState::zero(1, &sjk_protocol::LEGACY_ENTITY_FIELDS);
            // Protocol-26 weapon, legsAnim, torsoAnim and fireflag (saber style).
            for (field, value) in [(14, 3), (16, stock), (17, stock), (27, style)] {
                assert!(entity.set_raw_field(field, value));
            }
            let mut player = PlayerState::zero();
            for (field, value) in [(47, 3), (13, stock), (15, stock), (23, style)] {
                assert!(player.set_raw_field(field, value));
            }
            let mut movement = MovementState {
                weapon: 3,
                legs_anim: stock as u16,
                torso_anim: stock as u16,
                saber_anim_level: style as u8,
                ..Default::default()
            };
            for step in [8, 7, 4, 3] {
                movement.command_time += step;
                let before = movement.clone();
                let remote = LegacyAnimationSelection::from_entity(&entity);
                let local = LegacyAnimationSelection::from_player(&player);
                let predicted = LegacyAnimationSelection::from_movement(&movement);
                assert_eq!(remote, local);
                assert_eq!(remote, predicted);
                assert_eq!(remote.lower.clip, expected);
                assert_eq!(remote.upper.clip, expected);
                assert_eq!(movement, before);
                assert_eq!(entity.leg_animation(), stock as u16);
                assert_eq!(player.leg_animation(), stock as u16);
            }
        }
    }

    #[test]
    fn flips_forced_frames_and_attacks_survive_the_presentation_remap() {
        let mut entity = EntityState::zero(1, &sjk_protocol::LEGACY_ENTITY_FIELDS);
        for (field, value) in [(14, 3), (16, 917), (17, 114), (27, 4), (50, 1), (62, 1)] {
            assert!(entity.set_raw_field(field, value));
        }
        let selection = LegacyAnimationSelection::from_entity(&entity);
        assert_eq!(selection.lower.clip, 942);
        assert_eq!(selection.lower.revision, 942 | (1 << 16));
        assert_eq!(selection.upper.clip, 114);
        assert_eq!(selection.upper.revision, 114 | (1 << 16));
        // The remapped pose retains the normal forced-frame override.
        let field = sjk_protocol::LEGACY_ENTITY_FIELDS
            .iter()
            .position(|field| field.name == "forceFrame")
            .unwrap();
        assert!(entity.set_raw_field(field, 123));
        assert_eq!(
            LegacyAnimationSelection::from_entity(&entity).forced_frame,
            Some(123)
        );
    }
}
