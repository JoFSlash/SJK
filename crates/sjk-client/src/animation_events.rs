//! Multiplayer `animevents.cfg` audio cues, independent of movement and the wire.
use sjk_model::{AnimationConfig, AnimationSequence};
use sjk_vfs::VirtualFileSystem;

pub mod footsteps;
mod parser;

/// One model-authored audio action. Paths are normalized once during loading.
#[derive(Clone, Debug)]
pub enum Cue {
    /// A random registered sound on a legacy channel.
    Sound { paths: Vec<String>, channel: u8 },
    /// Ground contact from a named Ghoul2 foot bolt. `extra` marks SJK's own
    /// contacts ([`footsteps::JUMP_LAND_STEPS`]), which the model's table does not have.
    Footstep {
        right: bool,
        heavy: bool,
        extra: bool,
    },
}

/// Absolute animation frame and probability (`0` means always).
#[derive(Clone, Debug)]
pub struct Event {
    /// Absolute GLA frame, including the animation's configured first frame.
    pub frame: i32,
    /// Percentage chance; zero means always, as in codemp.
    pub probability: u8,
    /// Audio-only payload; gameplay animation events are not executed here.
    pub cue: Cue,
    kind: String,
    order: usize,
}

/// Upper and lower event tables from one mounted model's configuration.
#[derive(Default, Debug)]
pub struct Events {
    /// Legs first, torso second; sorted by absolute keyframe.
    pub tracks: [Vec<Event>; 2],
}

impl Events {
    /// Load the model's event file, falling back to its skeleton's shared table.
    /// Includes remain within the mounted VFS and are bounded against cycles.
    pub fn load(
        vfs: &VirtualFileSystem,
        model: &str,
        skeleton: &str,
        config: &AnimationConfig,
    ) -> Self {
        let mut result = Self::default();
        let model_path = format!("{model}/animevents.cfg");
        let path = if vfs.contains(&model_path).unwrap_or(false) {
            model_path
        } else {
            format!("{skeleton}/animevents.cfg")
        };
        parser::load(&mut result, vfs, &path, config, &mut Vec::new());
        result.add_jump_land_steps(config);
        for track in &mut result.tracks {
            track.sort_by_key(|event| event.frame);
        }
        result
    }

    /// SJK's jump and landing contacts ([`footsteps::JUMP_LAND_STEPS`]): both feet at
    /// the listed frame of each listed animation the model has, unless its own table
    /// already steps somewhere in that animation.
    fn add_jump_land_steps(&mut self, config: &AnimationConfig) {
        let legs = &mut self.tracks[0];
        for &(name, key) in footsteps::JUMP_LAND_STEPS {
            let Some(sequence) = config.get_exact(name) else {
                continue;
            };
            if key as usize >= sequence.frame_count {
                continue;
            }
            let first = sequence.first_frame as i32;
            let last = first + sequence.frame_count as i32;
            if legs.iter().any(|event| {
                matches!(event.cue, Cue::Footstep { .. }) && (first..last).contains(&event.frame)
            }) {
                continue;
            }
            for right in [false, true] {
                let order = legs.len();
                legs.push(Event {
                    frame: first + key,
                    probability: 0,
                    cue: Cue::Footstep {
                        right,
                        heavy: true,
                        extra: true,
                    },
                    kind: "AEV_FOOTSTEP".to_owned(),
                    order,
                });
            }
        }
    }

    /// Sound assets to read on the appearance loader, before frame playback.
    pub fn paths(&self) -> impl Iterator<Item = &str> {
        self.tracks
            .iter()
            .flatten()
            .flat_map(|event| match &event.cue {
                Cue::Sound { paths, .. } => paths.as_slice(),
                Cue::Footstep { .. } => &[],
            })
            .map(String::as_str)
    }
}

/// Small replay-stable random stream; advances per occurrence, never per frame.
#[derive(Clone, Debug)]
pub struct Random(u64);
impl Default for Random {
    fn default() -> Self {
        Self(0x853c49e6748fea9b)
    }
}
impl Random {
    /// Select a bounded variant without allocation or shared state.
    pub fn index(&mut self, count: usize) -> usize {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        (self.0.wrapping_mul(0x2545f4914f6cdd1d) >> 32) as usize % count.max(1)
    }
}

/// Per-actor event latch. A stopped frame cannot replay a cue.
#[derive(Default)]
pub struct Cursor {
    previous: [Option<(usize, i32)>; 2],
    random: Random,
}
impl Cursor {
    /// Forget an absent actor, teleport or backwards seek without firing old cues.
    pub fn reset(&mut self) {
        self.previous = [None; 2];
    }

    /// Visit only nearby keyframes, matching codemp's three-frame skip tolerance.
    pub fn advance(
        &mut self,
        events: &Events,
        config: &AnimationConfig,
        frames: [Option<(usize, i32)>; 2],
        mut emit: impl FnMut(&Cue, usize),
    ) {
        self.advance_tracks(events, config, frames, |_, cue, variant| emit(cue, variant));
    }

    /// [`Self::advance`], telling each cue's track (legs 0, torso 1).
    fn advance_tracks(
        &mut self,
        events: &Events,
        config: &AnimationConfig,
        frames: [Option<(usize, i32)>; 2],
        mut emit: impl FnMut(usize, &Cue, usize),
    ) {
        for (track, current) in frames.into_iter().enumerate() {
            let previous = std::mem::replace(&mut self.previous[track], current);
            let (Some((clip, frame)), Some((old_clip, old))) = (current, previous) else {
                continue;
            };
            if frame == old {
                continue;
            }
            let sequence =
                crate::legacy_animation_name(clip).and_then(|name| config.get_exact(name));
            let entries = &events.tracks[track];
            let mut selected = [0usize; 42];
            let mut count = 0;
            // Every eligible event is within three frames of one endpoint.
            for (range, (lo, hi)) in [
                (old.saturating_sub(3), old.saturating_add(3)),
                (frame.saturating_sub(3), frame.saturating_add(3)),
            ]
            .into_iter()
            .enumerate()
            {
                let start = entries.partition_point(|event| event.frame < lo);
                for (offset, event) in entries[start..]
                    .iter()
                    .take_while(|event| event.frame <= hi)
                    .enumerate()
                {
                    if range == 1 && event.frame.abs_diff(old) <= 3 {
                        continue;
                    }
                    if matches_frame(event.frame, old, frame, clip == old_clip, sequence) {
                        // Two seven-frame intervals, at most three supported event kinds per key.
                        selected[count] = start + offset;
                        count += 1;
                    }
                }
            }
            selected[..count].sort_unstable_by_key(|&i| entries[i].order);
            for &i in &selected[..count] {
                let event = &entries[i];
                if event.probability == 0 || self.random.index(100) < usize::from(event.probability)
                {
                    emit(track, &event.cue, self.random.index(1 << 16));
                }
            }
        }
    }

    /// Play `track`'s cues in a non-looping, forward `clip` keyed after `after` and up
    /// to `frame`, at most [`CATCH_UP_FRAMES`] back.
    fn catch_up(
        &mut self,
        events: &Events,
        config: &AnimationConfig,
        track: usize,
        (clip, after, frame): (usize, i32, i32),
        emit: &mut impl FnMut(usize, &Cue, usize),
    ) {
        let Some(sequence) = one_shot(config, clip) else {
            return;
        };
        let first = sequence.first_frame as i32;
        let last = first + sequence.frame_count as i32;
        let after = after.max(first - 1).max(frame - CATCH_UP_FRAMES);
        let entries = &events.tracks[track];
        let start = entries.partition_point(|event| event.frame <= after);
        for event in entries[start..]
            .iter()
            .take_while(|event| event.frame <= frame && event.frame < last)
        {
            if event.probability == 0 || self.random.index(100) < usize::from(event.probability) {
                emit(track, &event.cue, self.random.index(1 << 16));
            }
        }
    }
}

/// `clip`'s sequence when it plays once forward: the only kind whose cues a
/// hand-over keeps back or catches up, since a loop's frames come round again.
fn one_shot(config: &AnimationConfig, clip: usize) -> Option<&AnimationSequence> {
    crate::legacy_animation_name(clip)
        .and_then(|name| config.get_exact(name))
        .filter(|sequence| sequence.loop_frame == -1 && sequence.frames_per_second >= 0.0)
}

/// How far back a hand-over to the predicted animation plays the cues it skipped.
pub const CATCH_UP_FRAMES: i32 = 12;
/// The longest a hand-over to the server's animation keeps back cues already heard.
pub const HOLD_MILLIS: i64 = 1_000;
/// How many of a track's last animations a hand-over remembers as heard.
const HEARD: usize = 3;

/// One animation as played on a track: its clip, its revision (a restart flips it)
/// and the furthest frame its cues were heard to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Played {
    pub clip: usize,
    pub revision: u64,
    pub frame: i32,
}

impl Played {
    fn same(self, other: Self) -> bool {
        self.clip == other.clip && self.revision == other.revision
    }
}

/// A track's last [`HEARD`] animations, the newest first.
#[derive(Clone, Copy, Default)]
struct Heard([Option<Played>; HEARD]);

impl Heard {
    /// The animation `now` shows, if it was heard.
    fn find(&self, now: Played) -> Option<Played> {
        self.0
            .iter()
            .flatten()
            .copied()
            .find(|heard| heard.same(now))
    }

    /// Note `now` as heard up to its frame.
    fn note(&mut self, now: Played) {
        if let Some(heard) = self.0[0].filter(|heard| heard.same(now)) {
            self.0[0] = Some(Played {
                frame: heard.frame.max(now.frame),
                ..now
            });
        } else if let Some(index) = self
            .0
            .iter()
            .position(|heard| heard.is_some_and(|heard| heard.same(now)))
        {
            // The same clip and revision after another animation is a new run: a
            // restart flips the revision only when the same animation plays again
            // (A, R, A keeps it), so the older run's frame does not carry over.
            self.0[..=index].rotate_right(1);
            self.0[0] = Some(now);
        } else {
            self.0.rotate_right(1);
            self.0[0] = Some(now);
        }
    }
}

/// The animation data a hand-over plays cues from.
#[derive(Clone, Copy)]
pub struct Clips<'a> {
    pub events: &'a Events,
    pub config: &'a AnimationConfig,
}

/// The local player's cues across the hand-over between the predicted animation and
/// the server's, a ping behind, which the player is shown during saber moves that are
/// not predicted (hub report #171). Without it, the server's animation replayed the
/// swing just heard, and the way back to the predicted one landed past the new swing's
/// key, so it stayed silent. Handed to the server's, a track keeps back its cues while
/// that animation is one the prediction played, until it reaches the frame heard (or
/// [`HOLD_MILLIS`] pass); one the prediction never had (a parry, a saber lock) is
/// heard at once. Handed back to the predicted one, it plays the keys the predicted
/// animation passed unheard. Looping clips (running, standing) are left as they are:
/// their frames come round again, so nothing is held back or caught up.
#[derive(Default)]
pub struct Handover {
    predicted: Option<bool>,
    heard: [Heard; 2],
    hold: [Option<(Played, i64)>; 2],
}

impl Handover {
    /// Forget an absent actor, as [`Cursor::reset`] does.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Advance `cursor` to `played` at `now_millis`, the predicted animation when
    /// `predicted`, else the server's.
    pub fn advance(
        &mut self,
        cursor: &mut Cursor,
        clips: Clips<'_>,
        played: [Option<Played>; 2],
        predicted: bool,
        now_millis: i64,
        mut emit: impl FnMut(&Cue, usize),
    ) {
        let Clips { events, config } = clips;
        let switched = self.predicted.is_some_and(|was| was != predicted);
        self.predicted = Some(predicted);
        let mut quiet = [false; 2];
        let mut catch_up = [None; 2];
        for track in 0..2 {
            if switched {
                self.hold[track] = None;
                if let Some(now) = played[track].filter(|now| one_shot(config, now.clip).is_some())
                {
                    let heard = self.heard[track].find(now);
                    if predicted {
                        // The cursor's last frame is the server's; the keys between
                        // are played from what was heard instead. A one-shot clip only
                        // moves forward, so one heard past the predicted frame is an
                        // older run of the same clip (A, R, A), not this one.
                        quiet[track] = true;
                        let after = heard
                            .filter(|heard| heard.frame <= now.frame)
                            .map_or(i32::MIN, |heard| heard.frame);
                        catch_up[track] = Some((now.clip, after, now.frame));
                    } else if let Some(heard) = heard.filter(|heard| now.frame < heard.frame) {
                        self.hold[track] = Some((heard, now_millis + HOLD_MILLIS));
                    }
                }
            }
            if let Some((heard, until)) = self.hold[track] {
                quiet[track] = true;
                let shown = played[track].filter(|now| heard.same(*now));
                let reached = shown.is_some_and(|now| now.frame >= heard.frame);
                if reached || shown.is_none() || now_millis >= until {
                    // Reaching the frame heard still repeats its keys; once the
                    // server's animation goes elsewhere, or the time is up, it is heard.
                    self.hold[track] = None;
                    quiet[track] = reached;
                }
            }
        }
        let frames = played.map(|track| track.map(|played| (played.clip, played.frame)));
        cursor.advance_tracks(events, config, frames, |track, cue, variant| {
            if !quiet[track] {
                emit(cue, variant);
            }
        });
        for (track, range) in catch_up.into_iter().enumerate() {
            if let Some(range) = range {
                cursor.catch_up(events, config, track, range, &mut |_, cue, variant| {
                    emit(cue, variant)
                });
            }
        }
        for (track, now) in played.into_iter().enumerate() {
            if self.hold[track].is_none()
                && let Some(now) = now
            {
                self.heard[track].note(now);
            }
        }
    }
}

// Separate predicate mirrors CG_PlayerAnimEvents and is reference-checkable.
/// Whether one keyframe is reached by a changed rendered animation frame.
pub fn matches_frame(
    key: i32,
    old: i32,
    frame: i32,
    same: bool,
    sequence: Option<&AnimationSequence>,
) -> bool {
    if frame == old {
        return false;
    }
    if key == frame {
        return true;
    }
    if !same || old.abs_diff(frame) <= 1 || (old.abs_diff(key) > 3 && frame.abs_diff(key) > 3) {
        return false;
    }
    let Some(sequence) = sequence else {
        return false;
    };
    let inside = key >= sequence.first_frame as i32
        && key < (sequence.first_frame + sequence.frame_count) as i32;
    if sequence.frames_per_second < 0.0 {
        old > key && (frame < key || (sequence.loop_frame != -1 && inside && frame > old))
    } else {
        old < key && (frame > key || (sequence.loop_frame != -1 && inside && frame < old))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn steps(events: &Events) -> Vec<(i32, bool, bool)> {
        events.tracks[0]
            .iter()
            .filter_map(|event| match event.cue {
                Cue::Footstep { right, extra, .. } => Some((event.frame, right, extra)),
                Cue::Sound { .. } => None,
            })
            .collect()
    }

    #[test]
    fn jumps_and_the_forward_landing_get_both_feet_unless_authored() {
        let config = AnimationConfig::parse(
            b"BOTH_JUMP1 100 8 -1 20
BOTH_LAND1 200 5 -1 20
BOTH_LANDBACK1 300 6 -1 20
",
        )
        .unwrap();
        let mut events = Events::default();
        // A model that authored its own landing step keeps it alone.
        events.tracks[0].push(Event {
            frame: 202,
            probability: 0,
            cue: Cue::Footstep {
                right: false,
                heavy: false,
                extra: false,
            },
            kind: "AEV_FOOTSTEP".to_owned(),
            order: 0,
        });
        events.add_jump_land_steps(&config);
        events.tracks[0].sort_by_key(|event| event.frame);
        let mut found = steps(&events);
        found.sort();
        assert_eq!(
            found,
            [(101, false, true), (101, true, true), (202, false, false)]
        );
        // The takeoff's frame is reached and both feet step.
        let mut cursor = Cursor::default();
        let mut cues = Vec::new();
        for frame in [100, 101] {
            cursor.advance(&events, &config, [Some((0, frame)), None], |cue, _| {
                cues.push(matches!(cue, Cue::Footstep { extra: true, .. }));
            });
        }
        assert_eq!(cues, [true, true]);
    }

    /// Two swings, each with a sound keyed at its fourth frame on the torso.
    fn swings() -> (AnimationConfig, Events, usize, usize) {
        let config = AnimationConfig::parse(
            b"BOTH_A1_T__B_ 100 10 -1 20
BOTH_A1_TL_BR 200 10 -1 20
BOTH_STAND1 300 10 0 20
",
        )
        .unwrap();
        let mut events = Events::default();
        for (order, frame) in [103, 203, 303].into_iter().enumerate() {
            events.tracks[1].push(Event {
                frame,
                probability: 0,
                cue: Cue::Sound {
                    paths: vec![frame.to_string()],
                    channel: 0,
                },
                kind: "AEV_SOUNDCHAN".to_owned(),
                order,
            });
        }
        let first = sjk_game_jka::legacy_animation_index("BOTH_A1_T__B_").unwrap();
        let second = sjk_game_jka::legacy_animation_index("BOTH_A1_TL_BR").unwrap();
        (config, events, first, second)
    }

    fn sound(cue: &Cue) -> String {
        match cue {
            Cue::Sound { paths, .. } => paths[0].clone(),
            Cue::Footstep { .. } => unreachable!(),
        }
    }

    /// The predicted swing to its end, the server's a ping behind replaying it, then
    /// the next swing predicted again, already past its key (#171).
    fn hand_over_frames(first: usize, second: usize) -> Vec<(bool, Played)> {
        let swing = |clip, revision, frame| Played {
            clip,
            revision,
            frame,
        };
        let mut frames = Vec::new();
        frames.extend((100..=109).map(|frame| (true, swing(first, 1, frame))));
        frames.extend((101..=106).map(|frame| (false, swing(first, 1, frame))));
        frames.extend((205..=207).map(|frame| (true, swing(second, 2, frame))));
        frames
    }

    #[test]
    fn a_bare_cursor_repeats_the_swing_and_misses_the_next_across_the_hand_over() {
        let (config, events, first, second) = swings();
        let mut cursor = Cursor::default();
        let mut heard = Vec::new();
        for (_, played) in hand_over_frames(first, second) {
            cursor.advance(
                &events,
                &config,
                [None, Some((played.clip, played.frame))],
                |cue, _| {
                    heard.push(sound(cue));
                },
            );
        }
        assert_eq!(heard, ["103", "103"]);
    }

    #[test]
    fn the_hand_over_hears_each_swing_once() {
        let (config, events, first, second) = swings();
        let mut cursor = Cursor::default();
        let mut handover = Handover::default();
        let mut heard = Vec::new();
        for (step, (predicted, played)) in hand_over_frames(first, second).into_iter().enumerate() {
            handover.advance(
                &mut cursor,
                Clips {
                    events: &events,
                    config: &config,
                },
                [None, Some(played)],
                predicted,
                step as i64 * 50,
                |cue, _| heard.push(sound(cue)),
            );
        }
        assert_eq!(heard, ["103", "203"]);
    }

    #[test]
    fn a_server_move_the_prediction_never_had_is_heard_at_once() {
        let (config, events, first, second) = swings();
        let swing = |clip, frame| Played {
            clip,
            revision: 1,
            frame,
        };
        // Predicted to the end of the first swing, then the server's shows the
        // second one the prediction never had (a parry, a saber lock): its key is
        // heard straight away, not held back.
        let mut cursor = Cursor::default();
        let mut handover = Handover::default();
        let mut heard = Vec::new();
        let mut step = |handover: &mut Handover, predicted, played, time: i64| {
            handover.advance(
                &mut cursor,
                Clips {
                    events: &events,
                    config: &config,
                },
                [None, Some(played)],
                predicted,
                time,
                |cue, _| heard.push((time, sound(cue))),
            );
        };
        let mut time = 0;
        for frame in 100..=109 {
            step(&mut handover, true, swing(first, frame), time);
            time += 50;
        }
        for frame in 200..=204 {
            step(&mut handover, false, swing(second, frame), time);
            time += 50;
        }
        assert_eq!(heard, [(150, "103".to_owned()), (650, "203".to_owned())]);
    }

    #[test]
    fn the_server_replaying_a_swing_is_held_until_it_leaves_it() {
        let (config, events, first, second) = swings();
        let swing = |clip, frame| Played {
            clip,
            revision: 1,
            frame,
        };
        // The server's replays the predicted swing a ping behind, then moves on to
        // the second before reaching the frame heard: the second is heard.
        let mut cursor = Cursor::default();
        let mut handover = Handover::default();
        let mut heard = Vec::new();
        let frames = (100..=109)
            .map(|frame| (true, swing(first, frame)))
            .chain((101..=104).map(|frame| (false, swing(first, frame))))
            .chain((200..=204).map(|frame| (false, swing(second, frame))));
        for (step, (predicted, played)) in frames.enumerate() {
            handover.advance(
                &mut cursor,
                Clips {
                    events: &events,
                    config: &config,
                },
                [None, Some(played)],
                predicted,
                step as i64 * 50,
                |cue, _| heard.push(sound(cue)),
            );
        }
        assert_eq!(heard, ["103", "203"]);
    }

    /// Cues heard through a hand-over of the second track's `frames`.
    fn heard_through(
        (config, events): (&AnimationConfig, &Events),
        frames: impl IntoIterator<Item = (bool, Played)>,
    ) -> Vec<String> {
        let mut cursor = Cursor::default();
        let mut handover = Handover::default();
        let mut heard = Vec::new();
        for (step, (predicted, played)) in frames.into_iter().enumerate() {
            handover.advance(
                &mut cursor,
                Clips { events, config },
                [None, Some(played)],
                predicted,
                step as i64 * 50,
                |cue, _| heard.push(sound(cue)),
            );
        }
        heard
    }

    #[test]
    fn a_swing_repeated_after_another_is_a_new_run_across_the_hand_over() {
        let (config, events, first, second) = swings();
        // A, R, A: the revision flips only when the same animation restarts, so the
        // second A has the first one's revision.
        let swing = |clip, revision, frame| Played {
            clip,
            revision,
            frame,
        };
        let a_r = || {
            (100..=109)
                .map(move |frame| (true, swing(first, 1, frame)))
                .chain((200..=209).map(move |frame| (true, swing(second, 2, frame))))
        };
        // Handed to the server's before the second A's key: the server's plays it.
        let to_server = a_r()
            .chain((100..=101).map(|frame| (true, swing(first, 1, frame))))
            .chain((100..=105).map(|frame| (false, swing(first, 1, frame))));
        assert_eq!(
            heard_through((&config, &events), to_server),
            ["103", "203", "103"]
        );
        // The combo ends on the server's R; the next opens with A predicted past its
        // key: it is caught up, not taken for the first A heard to its end (#171).
        let to_predicted = a_r()
            .chain((201..=209).map(|frame| (false, swing(second, 2, frame))))
            .chain((105..=107).map(|frame| (true, swing(first, 1, frame))));
        assert_eq!(
            heard_through((&config, &events), to_predicted),
            ["103", "203", "103"]
        );
    }

    #[test]
    fn footsteps_of_a_looping_clip_are_kept_across_the_hand_over() {
        let config = AnimationConfig::parse(b"BOTH_RUN1 400 8 0 20\n").unwrap();
        let mut events = Events::default();
        for (order, frame) in [401, 405].into_iter().enumerate() {
            events.tracks[0].push(Event {
                frame,
                probability: 0,
                cue: Cue::Footstep {
                    right: order == 1,
                    heavy: false,
                    extra: false,
                },
                kind: "AEV_FOOTSTEP".to_owned(),
                order,
            });
        }
        let run = sjk_game_jka::legacy_animation_index("BOTH_RUN1").unwrap();
        let running = |frame| Played {
            clip: run,
            revision: 1,
            frame,
        };
        // Predicted through a whole cycle, then the server's a few frames behind,
        // then the predicted again: every step the frames pass is heard.
        let mut cursor = Cursor::default();
        let mut handover = Handover::default();
        let mut steps = 0;
        let frames = (400..=407)
            .map(|frame| (true, running(frame)))
            .chain((404..=406).map(|frame| (false, running(frame))))
            .chain([(true, running(401)), (true, running(402))]);
        for (step, (predicted, played)) in frames.enumerate() {
            handover.advance(
                &mut cursor,
                Clips {
                    events: &events,
                    config: &config,
                },
                [Some(played), None],
                predicted,
                step as i64 * 50,
                |_, _| steps += 1,
            );
        }
        // 401 and 405 in the predicted cycle, 405 again from the server's (it
        // passes that frame), and 401 on the way back.
        assert_eq!(steps, 4);
    }
}
