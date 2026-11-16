//! JA+ flamethrower presentation contracts, separate from native server gameplay.
//!
//! Evidence: JoF EJK #31, revision f887203414a11e6c9723fbaa6ea129eddbbf7bd8,
//! codemp/game/bg_public.h and cgame/cg_players.c/cg_ents.c. A new server
//! contract adds a table entry; generic consumers never compare server names.
use super::Caster;
use crate::console::ViewerConsole;
use sjk_protocol::GameState;
use sjk_shell::{CvarDefinition, CvarFlags, CvarRegistry};

/// -1 detects a documented contract, 0 disables it, positive values select a row.
pub(crate) const CVAR: &str = "cg_flameThrower";
/// Protocol-26 compatibility identifiers (`bg_public.h`).
const CLASS_VEHICLE: u8 = 53;
const EF_DEAD: u32 = 2;

/// One server family's existing snapshot markers and presentation assets.
pub(crate) struct Contract {
    pub(crate) game_key: &'static str,
    pub(crate) accepts: fn(&str) -> bool,
    pub(crate) firing_flag: u32,
    pub(crate) hit_flag: u32,
    pub(crate) stream: &'static str,
    /// Retail fallback from JoF EJK #67; its forward axis is opposite the custom graph.
    pub(crate) fallback_stream: &'static str,
    pub(crate) hit: &'static str,
    pub(crate) sound: &'static str,
    pub(crate) pose: &'static str,
    /// Legacy `animation.cfg` ordinal, pinned to its name by the contract test.
    pub(crate) pose_index: usize,
    pub(crate) hit_millis: i32,
    pub(crate) sound_millis: i32,
}

/// JA+ reuses Lightning passes 1..=3; EJK #31 supplies the flag/asset contract.
pub(crate) const CONTRACTS: &[Contract] = &[Contract {
    game_key: "gamename",
    accepts: ja_plus_itself,
    firing_flag: 1 << 17,
    hit_flag: 1 << 8,
    stream: "mp/fthrw_bobafire",
    fallback_stream: "boba/fthrw",
    hit: "env/fire_wall",
    sound: "sound/weapons/boba/bf_flame.mp3",
    pose: "BOTH_FORCELIGHTNING_HOLD",
    pose_index: 1337,
    hit_millis: 100,
    sound_millis: 2900,
}];

/// JA+ proper only (`ja+`, `japlus`, `ja_plus` gamenames, spelled as
/// `sjk_protocol::is_ja_plus_game_name` spells them). That helper also accepts
/// abyssmod and UA Galaxy, whose eFlags bit 17 / eFlags2 bit 8 meanings are
/// unverified, so they get no flame contract until an entry documents them.
pub(crate) fn ja_plus_itself(game_name: &str) -> bool {
    ["ja+", "japlus", "ja_plus"].iter().any(|prefix| {
        game_name
            .as_bytes()
            .get(..prefix.len())
            .is_some_and(|head| head.eq_ignore_ascii_case(prefix.as_bytes()))
    })
}

impl Contract {
    /// Choose only preloaded definitions; a missing graph never retries disk I/O
    /// in actor submission. The custom graph always wins when both are installed.
    pub(crate) fn stream_effect(
        &self,
        mut available: impl FnMut(&str) -> bool,
    ) -> Option<(&'static str, bool)> {
        if available(self.stream) {
            Some((self.stream, false))
        } else if available(self.fallback_stream) {
            Some((self.fallback_stream, true))
        } else {
            None
        }
    }
    /// `CG_General`'s eFlags2 marker, already decoded at protocol-26 netfield 96.
    pub(crate) fn hit_marker(&self, entity: &sjk_protocol::EntityState) -> bool {
        entity.entity_type() == 0 && entity.raw_field(96).unwrap_or(0) & self.hit_flag != 0
    }
    /// Only the Lightning branch is replaced; Drain and vehicle effects keep theirs.
    pub(crate) fn firing(&self, caster: Caster) -> bool {
        caster.npc_class != CLASS_VEHICLE
            && caster.e_flags & EF_DEAD == 0
            && (1..=3).contains(&caster.active_force_pass)
            && caster.e_flags & self.firing_flag != 0
    }
}

/// Reverse the fallback's forward direction in SJK's proper-rotation EFX frame.
/// Keep up fixed; the side axis necessarily turns too (EJK flips forward alone).
/// Live comparison of asymmetric community replacements remains unverified.
pub(crate) fn stream_rotation(frame: glam::Quat, reverse_forward: bool) -> glam::Quat {
    if reverse_forward {
        frame * glam::Quat::from_rotation_z(std::f32::consts::PI)
    } else {
        frame
    }
}

/// Register the archived player override (also exposed by both settings views).
pub(crate) fn register(cvars: &mut CvarRegistry) -> Result<(), sjk_shell::CvarError> {
    cvars.register(CvarDefinition::new(
        CVAR,
        -1_i64,
        CvarFlags::ARCHIVE,
        "Flamethrower presentation: -1 detect server, 0 off, 1 JA+ snapshot contract",
    ))?;
    Ok(())
}

/// Recognise server capabilities from the documented table.
pub(crate) fn detect(game: Option<&GameState>) -> Option<&'static Contract> {
    let info = sjk_client::LegacyClientInfo::new(game?.config_string(0)?);
    CONTRACTS
        .iter()
        .find(|rule| info.text(rule.game_key).is_some_and(rule.accepts))
}

/// Apply the explicit override to an automatically detected contract.
pub(crate) fn overridden(
    detected: Option<&'static Contract>,
    value: i64,
) -> Option<&'static Contract> {
    match value {
        -1 => detected,
        0 => None,
        value => usize::try_from(value)
            .ok()
            .and_then(|index| index.checked_sub(1))
            .and_then(|index| CONTRACTS.get(index)),
    }
}

/// Resolve the frame's contract once, before actor submission/pose preparation.
pub(crate) fn policy(
    game: Option<&GameState>,
    console: Option<&ViewerConsole>,
) -> Option<&'static Contract> {
    overridden(
        detect(game),
        console.and_then(|c| c.integer_cvar(CVAR)).unwrap_or(-1),
    )
}

/// An entity's presentation-only timer, reset on stop, disappearance or time rewind.
#[derive(Clone, Copy, Default)]
pub(crate) struct Timer {
    last: Option<i32>,
    next: i32,
}

impl Timer {
    pub(crate) fn due(&mut self, active: bool, time: i32, interval: i32) -> bool {
        if !active {
            *self = Self::default();
            return false;
        }
        let due = self.last.is_none_or(|last| time < last) || time > self.next;
        self.last = Some(time);
        if due {
            self.next = time.saturating_add(interval);
        }
        due
    }
}

/// Per-mesh upper-body override; the world and predicted movement stay untouched.
#[derive(Default)]
pub(crate) struct Pose {
    identity: Option<(sjk_runtime::EntityId, usize)>,
    started: i64,
    last: i64,
    outgoing: Option<sjk_runtime::AnimationTrackState>,
}

impl Pose {
    pub(crate) fn apply(
        &mut self,
        entity: Option<sjk_runtime::EntityId>,
        clip: Option<usize>,
        time: i64,
        state: &mut sjk_runtime::AnimationState,
    ) {
        let Some(identity) = entity.zip(clip) else {
            self.identity = None;
            return;
        };
        if self.identity != Some(identity) || time < self.last {
            self.identity = Some(identity);
            self.started = time;
            self.outgoing = Some(state.upper);
        }
        self.last = time;
        state.upper.clip = identity.1;
        state.upper.started_at_millis = self.started;
        state.upper.phase_millis = 0;
        state.upper.forced_frame = None;
        // CG_SetLerpFrameAnimation's ordinary 100 ms upper-body blend.
        state.upper.transition = self
            .outgoing
            .filter(|_| time < self.started.saturating_add(100))
            .map(|track| sjk_runtime::AnimationTrackTransition {
                clip: track.clip,
                revision: track.revision,
                started_at_millis: track.started_at_millis,
                phase_millis: track.phase_millis,
                speed_milli: track.speed_milli,
                forced_frame: track.forced_frame,
                blend_started_at_millis: self.started,
                blend_duration_millis: 100,
            });
    }
}

/// Fixed entity-slot hit timers owned by the current effect world.
pub(crate) struct Hits {
    timers: Box<[Timer]>,
    enabled: bool,
}
impl Default for Hits {
    fn default() -> Self {
        Self {
            timers: vec![Timer::default(); 1024].into_boxed_slice(),
            enabled: false,
        }
    }
}

impl Hits {
    /// A marker's timer uses the presented trajectory clock, never snapshot arrival.
    fn due(&mut self, rule: &Contract, entity: &sjk_protocol::EntityState, time: i32) -> bool {
        self.timers
            .get_mut(usize::from(entity.number()))
            .is_some_and(|timer| timer.due(rule.hit_marker(entity), time, rule.hit_millis))
    }
}

/// Play ET_GENERAL hit markers once per reference interval, independently of actors.
pub(in crate::actor_world_submission) fn submit_hits(
    sinks: &mut super::Sinks<'_>,
    snapshot: &sjk_protocol::Snapshot,
    time: i32,
    now: std::time::Instant,
) {
    let Some(rule) = sinks.flame else {
        if sinks.effect_aux.flame_hits.enabled {
            sinks.effect_aux.flame_hits.timers.fill(Timer::default());
            sinks.effect_aux.flame_hits.enabled = false;
        }
        return;
    };
    sinks.effect_aux.flame_hits.enabled = true;
    // JoF EJK #67: advance hit effects between snapshots, using the same
    // presentation clock that evaluates their trajectories (including demos).
    let clock = time;
    for entity in &snapshot.entities {
        if !sinks.effect_aux.flame_hits.due(rule, entity, clock)
            || !sinks.effects.contains_definition(rule.hit)
        {
            continue;
        }
        let origin = sjk_client::legacy_evaluate_trajectory(
            entity.trajectory_base(),
            entity.trajectory_delta(),
            entity.trajectory_type(),
            entity.trajectory_time(),
            entity.trajectory_duration(),
            time,
        );
        let angles = entity.angular_trajectory_base().map(f32::to_radians);
        let rotation = glam::Quat::from_rotation_z(angles[1])
            * glam::Quat::from_rotation_y(angles[0])
            * glam::Quat::from_rotation_x(angles[2]);
        crate::effect_runtime::spawn_effect(
            sinks.particles,
            sinks.effect_aux,
            sinks.effects,
            sinks.vfs,
            rule.hit,
            glam::Vec3::from_array(origin),
            now,
            u32::from(entity.number()) ^ time as u32,
            0,
            sinks.game_audio,
            rotation,
        );
    }
    for timer in &mut sinks.effect_aux.flame_hits.timers {
        if timer.last != Some(clock) {
            *timer = Timer::default();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjk_protocol::{EntityState, LEGACY_ENTITY_FIELDS, PlayerState};

    #[test]
    #[ignore = "release-only synthetic flame selection/timer measurement"]
    fn measure_flame_policy() {
        let rule = &CONTRACTS[0];
        let available: std::collections::HashSet<&str> =
            [rule.fallback_stream].into_iter().collect();
        let mut timers = vec![Timer::default(); 128];
        let start = std::time::Instant::now();
        let mut bursts = 0_u64;
        for frame in 0..20_000 {
            for timer in &mut timers {
                let selection =
                    rule.stream_effect(|name| available.contains(std::hint::black_box(name)));
                std::hint::black_box(selection);
                bursts += u64::from(timer.due(true, frame * 3, rule.hit_millis));
            }
        }
        eprintln!(
            "128 flame graph selections/timers: {:.3} us/frame; bursts={}",
            start.elapsed().as_secs_f64() * 1e6 / 20_000.0,
            std::hint::black_box(bursts)
        );
    }

    #[test]
    fn stream_prefers_custom_graph_and_falls_back_only_when_missing() {
        let rule = &CONTRACTS[0];
        assert_eq!(rule.stream_effect(|_| true), Some((rule.stream, false)));
        assert_eq!(
            rule.stream_effect(|name| name == rule.fallback_stream),
            Some((rule.fallback_stream, true))
        );
        assert_eq!(rule.stream_effect(|_| false), None);
        for pitch in [-70.0, 0.0, 70.0] {
            for yaw in [0.0, 90.0, 180.0, 270.0] {
                let frame = super::super::beam_rotation(pitch, yaw);
                assert_eq!(stream_rotation(frame, false), frame);
                let fallback = stream_rotation(frame, true);
                assert!((fallback * glam::Vec3::X).abs_diff_eq(-(frame * glam::Vec3::X), 0.0001));
                assert!((fallback * glam::Vec3::Z).abs_diff_eq(frame * glam::Vec3::Z, 0.0001));
            }
        }
    }

    #[test]
    fn impact_clock_advances_between_snapshots_at_all_command_steps() {
        let rule = &CONTRACTS[0];
        for step in [8, 7, 4, 3] {
            let mut hits = Hits::default();
            let mut marker = EntityState::zero(77, &LEGACY_ENTITY_FIELDS);
            marker.set_raw_field(96, rule.hit_flag);
            let mut bursts = Vec::new();
            for presentation_time in (1000..=1500).step_by(step) {
                if hits.due(rule, &marker, presentation_time) {
                    bursts.push(presentation_time);
                }
                assert!(!hits.due(rule, &marker, presentation_time));
            }
            assert_eq!(bursts.len(), 5);
            assert!(
                bursts
                    .windows(2)
                    .all(|pair| (101..=108).contains(&(pair[1] - pair[0])))
            );
            marker.set_raw_field(96, 0);
            assert!(!hits.due(rule, &marker, 1501));
            marker.set_raw_field(96, rule.hit_flag);
            assert!(hits.due(rule, &marker, 1502));
            assert!(hits.due(rule, &marker, 900));
        }
    }

    #[test]
    fn flame_contract_is_detected_and_can_be_overridden() {
        let mut game = GameState::empty_local(0);
        game.replace_config_string(0, br"\gamename\JA+ Mod v2.5".to_vec())
            .unwrap();
        let rule = detect(Some(&game));
        assert!(rule.is_some());
        for rule in CONTRACTS {
            assert_eq!(
                sjk_game_jka::legacy_animation_name(rule.pose_index),
                Some(rule.pose)
            );
        }
        assert!(overridden(rule, 0).is_none());
        assert!(overridden(None, 1).is_some());
        for value in [i64::MIN, -2, 2, i64::MAX] {
            assert!(overridden(rule, value).is_none());
        }
        // abyssmod and UA Galaxy share JA+'s dialect but not a verified flame contract.
        for other in [
            &br"\gamename\basejka"[..],
            br"\gamename\abyssmod",
            br"\gamename\^4U^3A^5Galaxy",
        ] {
            game.replace_config_string(0, other.to_vec()).unwrap();
            assert!(detect(Some(&game)).is_none());
        }
        assert!(detect(None).is_none());
    }

    #[test]
    fn flame_flags_replace_only_lightning_for_local_and_remote_players() {
        let rule = &CONTRACTS[0];
        let mut player = PlayerState::zero();
        player.set_raw_field(17, rule.firing_flag);
        player.set_raw_field(72, 3);
        let local = Caster::from_player(&player, None);
        let mut entity = EntityState::zero(4, &LEGACY_ENTITY_FIELDS);
        entity.set_raw_field(19, rule.firing_flag);
        entity.set_raw_field(68, 3);
        let remote = Caster::from_entity(&entity, 0, 0);
        for caster in [local, remote] {
            let selected = |caster, rule| {
                super::super::select_for_contract(caster, false, rule, |_| true)
                    .0
                    .beam
            };
            assert_eq!(selected(caster, Some(rule)), Some(rule.stream));
            assert_eq!(
                super::super::select_for_contract(caster, false, Some(rule), |_| false)
                    .0
                    .beam,
                Some("force/lightningwide"),
                "without the JoF and retail flames the caster keeps stock Lightning"
            );
            let (fallback, reversed) =
                super::super::select_for_contract(caster, false, Some(rule), |name| {
                    name == rule.fallback_stream
                });
            assert_eq!(fallback.beam, Some(rule.fallback_stream));
            assert!(reversed, "the retail Boba flame is reversed");
            assert_eq!(selected(caster, None), Some("force/lightningwide"));
            assert_eq!(
                selected(
                    Caster {
                        e_flags: 0,
                        ..caster
                    },
                    Some(rule)
                ),
                Some("force/lightningwide")
            );
            assert_eq!(
                selected(
                    Caster {
                        active_force_pass: 4,
                        ..caster
                    },
                    Some(rule)
                ),
                Some("mp/drain")
            );
            assert_eq!(
                selected(
                    Caster {
                        active_force_pass: 0,
                        ..caster
                    },
                    Some(rule)
                ),
                None
            );
            assert_eq!(
                selected(
                    Caster {
                        npc_class: CLASS_VEHICLE,
                        ..caster
                    },
                    Some(rule)
                ),
                None
            );
            assert!(!rule.firing(Caster {
                e_flags: rule.firing_flag | EF_DEAD,
                ..caster
            }));
        }
        entity.set_raw_field(96, rule.hit_flag);
        assert!(rule.hit_marker(&entity));
        entity.set_raw_field(8, 13);
        assert!(!rule.hit_marker(&entity));
    }

    #[test]
    fn hit_cadence_uses_once_only_server_times_and_resets() {
        for step in [8, 7, 4, 3] {
            let mut timer = Timer::default();
            assert!(timer.due(true, 1000, 100));
            assert!(!timer.due(true, 1000, 100));
            for time in (1000 + step..=1100).step_by(step as usize) {
                assert!(!timer.due(true, time, 100));
            }
            assert!(timer.due(true, 1100 + step, 100));
            assert!(!timer.due(false, 1101 + step, 100));
            assert!(timer.due(true, 1102 + step, 100));
            assert!(timer.due(true, 100, 100));
        }
    }

    #[test]
    fn flame_pose_keeps_its_clock_and_releases_without_changing_legs() {
        use sjk_runtime::{AnimationState, AnimationTrackState, EntityId};
        let track = |clip| AnimationTrackState {
            clip,
            revision: 1,
            started_at_millis: 0,
            phase_millis: 0,
            speed_milli: 1000,
            forced_frame: None,
            transition: None,
        };
        let mut pose = Pose::default();
        let original = AnimationState {
            lower: track(1),
            upper: track(2),
        };
        let id = Some(EntityId::new(1));
        for step in [8, 7, 4, 3] {
            let mut clocked = Pose::default();
            for time in (1000..1200).step_by(step) {
                let mut state = original;
                clocked.apply(id, Some(3), time, &mut state);
                assert_eq!(state.lower, original.lower);
                assert_eq!(state.upper.elapsed_millis(time), time - 1000);
                assert_eq!(state.upper.transition.is_some(), time < 1100);
            }
        }
        for (time, start) in [(1000, 1000), (1100, 1000), (1050, 1050)] {
            let mut state = original;
            pose.apply(id, Some(3), time, &mut state);
            assert_eq!(state.lower, original.lower);
            assert_eq!(state.upper.clip, 3);
            assert_eq!(state.upper.started_at_millis, start);
        }
        let mut state = original;
        pose.apply(id, None, 1200, &mut state);
        assert_eq!(state, original);
        pose.apply(id, Some(3), 1300, &mut state);
        assert_eq!(state.upper.started_at_millis, 1300);
    }
}
