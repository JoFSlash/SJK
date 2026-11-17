//! BaseJKA facial animation: blinking eyes and the `face` bone's expressions.
//!
//! A port of `CG_G2PlayerHeadAnims`, `CG_G2SetHeadBlink` and `CG_G2SetHeadAnim`
//! (`codemp/cgame/cg_players.c`). Every frame, for a humanoid with a `face` bone, cgame
//! steps three per-client timers (`clientInfo_t::facial_blink`, `facial_frown`,
//! `facial_aux`, floats in milliseconds where a negative value means "in progress
//! until minus this"):
//!
//! - blink: the eyes shut every 4 to 8 seconds for 300 ms, by turning the `leye` and
//!   `reye` bones `(0, -50, 0)` with `BONE_ANGLES_POSTMULT` about `POSITIVE_Y`,
//!   `POSITIVE_Z`, `POSITIVE_X`; one blink in twenty is a wink (the left eye only);
//! - aux and frown: while nobody talks, every 6 to 10 seconds the face plays
//!   `FACE_ALERT` for 2 seconds, or `FACE_FROWN` when the frown timer is due too;
//! - a dead player's face plays `FACE_DEAD`, held on its frame (eyes shut).
//!
//! The talking faces (`FACE_TALK1` to `FACE_TALK4`) follow `S_GetVoiceVolume`, the
//! lip-sync amplitude of the entity's voice channel, which SJK's mixer does not
//! measure: it reads as silent (0), as it is for a player who says nothing.
//! `SET_FACEEYESCLOSED` (`facial_blink == 1`) is a single-player ICARUS command and is
//! never sent in multiplayer.
//!
//! This module is the timer state machine only; [`crate::LegacyGhoul2Animator`]
//! installs what it decides on the skeleton.

/// How long the eyes stay shut in a blink (`cg_players.c`, `-(cg.time + 300.0f)`).
pub const BLINK_SHUT_MILLIS: f32 = 300.0;
/// Range of the wait between blinks (`flrand(4000.0, 8000.0)`).
pub const BLINK_INTERVAL_MILLIS: (f32, f32) = (4_000.0, 8_000.0);
/// The yaw `CG_G2SetHeadBlink` turns an eye bone by to shut it.
pub const BLINK_EYE_YAW_DEGREES: f32 = -50.0;
/// `CG_G2SetHeadBlink`'s blend time, a third of it for a wink. Ghoul2 records it with
/// the angles (`boneBlendTime`) but never reads it outside ragdolls, so the turn is
/// immediate in the stock game as here.
pub const BLINK_BLEND_MILLIS: u16 = 80;
/// A blink is a wink when the random draw exceeds this (`Q_flrand(0, 1) > 0.95f`).
pub const WINK_THRESHOLD: f32 = 0.95;
/// How long an aux (`FACE_ALERT`) or frown lasts.
pub const FACE_EXPRESSION_MILLIS: f32 = 2_000.0;
/// The first aux and frown waits (`flrand(6000.0, 10000.0)`).
pub const FACE_FIRST_INTERVAL_MILLIS: (f32, f32) = (6_000.0, 10_000.0);
/// Later aux and frown waits (`flrand(7000.0, 10000.0)`).
pub const FACE_INTERVAL_MILLIS: (f32, f32) = (7_000.0, 10_000.0);

/// What a blink does to the eye bones this frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlinkChange {
    /// `CG_G2SetHeadBlink(cent, qtrue)`: shut `leye`, and `reye` unless it is a wink.
    Shut {
        /// Only the left eye shuts.
        wink: bool,
    },
    /// `CG_G2SetHeadBlink(cent, qfalse)`: both eyes back at `(0, 0, 0)`.
    Open,
}

impl BlinkChange {
    /// The angles `G2API_SetBoneAngles` gets for each eye bone, `[left, right]`;
    /// `None` leaves that eye as it is.
    pub fn eye_angles(self) -> [Option<[f32; 3]>; 2] {
        match self {
            Self::Shut { wink } => {
                let shut = [0.0, BLINK_EYE_YAW_DEGREES, 0.0];
                [Some(shut), (!wink).then_some(shut)]
            }
            Self::Open => [Some([0.0; 3]); 2],
        }
    }

    /// The blend time cgame passes along (80 ms, a third of it for a wink).
    pub fn blend_millis(self) -> u16 {
        match self {
            Self::Shut { wink: true } => BLINK_BLEND_MILLIS / 3,
            _ => BLINK_BLEND_MILLIS,
        }
    }
}

/// The face animation `CG_G2SetHeadAnim` sets on the `face` bone this frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FaceAnimation {
    /// `FACE_ALERT`, the aux expression.
    Alert,
    /// `FACE_FROWN`.
    Frown,
    /// `FACE_DEAD`, played with `BONE_ANIM_OVERRIDE_FREEZE`.
    Dead,
}

impl FaceAnimation {
    /// The `animation.cfg` name of the sequence.
    pub fn sequence_name(self) -> &'static str {
        match self {
            Self::Alert => "FACE_ALERT",
            Self::Frown => "FACE_FROWN",
            Self::Dead => "FACE_DEAD",
        }
    }

    /// The protocol animation number (`anims.h`), the override's identity.
    pub fn animation_number(self) -> usize {
        match self {
            Self::Alert => 5,
            Self::Frown => 7,
            Self::Dead => 8,
        }
    }

    /// Whether the animation holds its last frame (`FACE_DEAD` only).
    pub fn freezes(self) -> bool {
        self == Self::Dead
    }
}

/// One frame's decision of `CG_G2PlayerHeadAnims`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HeadAnimStep {
    /// A change to the eye bones, on the frame a blink starts or ends.
    pub blink: Option<BlinkChange>,
    /// The face animation to (re)set this frame; `None` leaves the `face` bone alone.
    pub face: Option<FaceAnimation>,
}

/// Raven's `flrand` (`q_math.c`): the same LCG as `irand`, its upper 15 bits scaled.
#[derive(Clone, Copy, Debug)]
struct FloatRandom(u32);

impl FloatRandom {
    fn flrand(&mut self, (min, max): (f32, f32)) -> f32 {
        self.0 = self.0.wrapping_mul(214_013).wrapping_add(2_531_011);
        let unit = (self.0 >> 17) as f32 / 32_768.0;
        unit * (max - min) + min
    }
}

/// One actor's facial timers (`clientInfo_t::facial_*`), zero until the first frame.
#[derive(Clone, Copy, Debug)]
pub struct LegacyHeadAnimTimers {
    blink: f32,
    frown: f32,
    aux: f32,
    random: FloatRandom,
}

impl LegacyHeadAnimTimers {
    /// Fresh timers (a new client info); `seed` keeps actors from blinking in step.
    pub const fn new(seed: u32) -> Self {
        Self {
            blink: 0.0,
            frown: 0.0,
            aux: 0.0,
            random: FloatRandom(seed),
        }
    }

    /// Whether the eyes are shut by a blink now.
    pub fn blinking(&self) -> bool {
        self.blink < 0.0
    }

    /// Step the timers at cgame's `cg.time` (`CG_G2PlayerHeadAnims`).
    pub fn step(&mut self, time_millis: i64, dead: bool) -> HeadAnimStep {
        let time = time_millis as f32;
        let mut step = HeadAnimStep::default();
        if dead {
            // "Dead people close their eyes and don't make faces!" The eye bones keep
            // what they have; the blink ends on the first living frame.
            self.blink = -1.0;
            step.face = Some(FaceAnimation::Dead);
            return step;
        }
        if self.blink == 0.0 {
            self.blink = time + self.random.flrand(BLINK_INTERVAL_MILLIS);
            self.frown = time + self.random.flrand(FACE_FIRST_INTERVAL_MILLIS);
            self.aux = time + self.random.flrand(FACE_FIRST_INTERVAL_MILLIS);
        }
        if self.blink < 0.0 {
            if -self.blink < time {
                self.blink = time + self.random.flrand(BLINK_INTERVAL_MILLIS);
                step.blink = Some(BlinkChange::Open);
            }
        } else if self.blink < time {
            let wink = self.random.flrand((0.0, 1.0)) > WINK_THRESHOLD;
            step.blink = Some(BlinkChange::Shut { wink });
            self.blink = -(time + BLINK_SHUT_MILLIS);
        }
        // `S_GetVoiceVolume` is 0: not talking, so the aux and frown timers run.
        if self.aux < 0.0 {
            if -self.aux < time {
                self.aux = time + self.random.flrand(FACE_INTERVAL_MILLIS);
            } else {
                step.face = Some(FaceAnimation::Alert);
            }
        } else if self.aux < time {
            step.face = Some(FaceAnimation::Alert);
            self.aux = -(time + FACE_EXPRESSION_MILLIS);
        }
        if step.face.is_some() {
            if self.frown < 0.0 {
                if -self.frown < time {
                    self.frown = time + self.random.flrand(FACE_INTERVAL_MILLIS);
                } else {
                    step.face = Some(FaceAnimation::Frown);
                }
            } else if self.frown < time {
                step.face = Some(FaceAnimation::Frown);
                self.frown = -(time + FACE_EXPRESSION_MILLIS);
            }
        }
        step
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Step every millisecond from `from` to `to`, collecting the blink changes.
    fn blinks(timers: &mut LegacyHeadAnimTimers, from: i64, to: i64) -> Vec<(i64, BlinkChange)> {
        (from..to)
            .filter_map(|time| timers.step(time, false).blink.map(|change| (time, change)))
            .collect()
    }

    #[test]
    fn eyes_shut_every_four_to_eight_seconds_for_300_ms() {
        for seed in [1, 7, 12_345, 0xdead_beef] {
            let mut timers = LegacyHeadAnimTimers::new(seed);
            let changes = blinks(&mut timers, 1_000, 120_000);
            assert!(
                changes.len() >= 20,
                "seed {seed}: {} changes",
                changes.len()
            );
            let mut last_open = 1_000;
            let (shuts, opens): (Vec<_>, Vec<_>) = changes
                .iter()
                .enumerate()
                .partition(|(index, _)| index % 2 == 0);
            for (shut, open) in shuts.into_iter().zip(opens) {
                let pair = [*shut.1, *open.1];
                let (shut_at, BlinkChange::Shut { .. }) = pair[0] else {
                    panic!("seed {seed}: blink did not start with a shut: {pair:?}");
                };
                let (open_at, BlinkChange::Open) = pair[1] else {
                    panic!("seed {seed}: blink did not end open: {pair:?}");
                };
                let wait = shut_at - last_open;
                assert!((4_000..=8_001).contains(&wait), "seed {seed}: wait {wait}");
                // Shut at t, open on the first frame past t + 300.
                assert_eq!(open_at - shut_at, 301, "seed {seed}");
                last_open = open_at;
            }
        }
    }

    #[test]
    fn about_one_blink_in_twenty_is_a_wink() {
        let mut timers = LegacyHeadAnimTimers::new(99);
        let shuts: Vec<bool> = blinks(&mut timers, 1_000, 20_000_000)
            .into_iter()
            .filter_map(|(_, change)| match change {
                BlinkChange::Shut { wink } => Some(wink),
                BlinkChange::Open => None,
            })
            .collect();
        let winks = shuts.iter().filter(|wink| **wink).count();
        let share = winks as f32 / shuts.len() as f32;
        assert!((0.03..0.07).contains(&share), "{winks} of {}", shuts.len());
    }

    #[test]
    fn a_blink_turns_the_eye_bones_as_cgame_does() {
        let shut = [0.0, -50.0, 0.0];
        assert_eq!(
            BlinkChange::Shut { wink: false }.eye_angles(),
            [Some(shut), Some(shut)]
        );
        assert_eq!(
            BlinkChange::Shut { wink: true }.eye_angles(),
            [Some(shut), None]
        );
        assert_eq!(BlinkChange::Open.eye_angles(), [Some([0.0; 3]); 2]);
        assert_eq!(BlinkChange::Shut { wink: false }.blend_millis(), 80);
        assert_eq!(BlinkChange::Shut { wink: true }.blend_millis(), 26);
    }

    #[test]
    fn the_dead_hold_face_dead_and_reopen_their_eyes_when_alive() {
        let mut timers = LegacyHeadAnimTimers::new(5);
        // Find a frame where the eyes are shut, then die.
        let mut time = 1_000;
        while !timers.blinking() {
            timers.step(time, false);
            time += 1;
        }
        for _ in 0..5_000 {
            let step = timers.step(time, true);
            assert_eq!(step.face, Some(FaceAnimation::Dead));
            assert_eq!(step.blink, None, "the eyes keep their state while dead");
            time += 1;
        }
        // The first living frame ends the blink and starts a new wait.
        assert_eq!(timers.step(time, false).blink, Some(BlinkChange::Open));
        assert!(!timers.blinking());
    }

    #[test]
    fn the_face_shows_alert_or_frown_for_two_seconds_every_few_seconds() {
        let mut timers = LegacyHeadAnimTimers::new(42);
        let mut runs = Vec::new();
        let mut current: Option<(i64, i64)> = None;
        for time in 1_000..200_000 {
            if timers.step(time, false).face.is_some() {
                current = Some(current.map_or((time, time), |(start, _)| (start, time)));
            } else if let Some(run) = current.take() {
                runs.push(run);
            }
        }
        assert!(runs.len() >= 15, "{} expressions", runs.len());
        for (start, end) in &runs {
            // Set at t and every frame up to t + 2000 inclusive.
            assert_eq!(end - start, 2_000);
        }
        for pair in runs.windows(2) {
            let gap = pair[1].0 - pair[0].1;
            assert!((7_000..=10_002).contains(&gap), "gap {gap}");
        }
    }

    #[test]
    fn a_frown_only_replaces_an_aux() {
        let mut timers = LegacyHeadAnimTimers::new(3);
        let mut frowns = 0;
        for time in 1_000..400_000 {
            if timers.step(time, false).face == Some(FaceAnimation::Frown) {
                frowns += 1;
            }
        }
        assert!(frowns > 0);
        assert_eq!(FaceAnimation::Dead.sequence_name(), "FACE_DEAD");
        assert!(FaceAnimation::Dead.freezes());
        assert!(!FaceAnimation::Alert.freezes());
        assert_eq!(
            sjk_game_jka::legacy_animation_name(FaceAnimation::Frown.animation_number()),
            Some("FACE_FROWN")
        );
    }
}
