//! Bounded first-use starts waiting for the asynchronous decoder. A handle is
//! reserved before its PCM reaches the callback; dropping Play at that point
//! silently lost jump/weapon sounds. Expire old events instead of replaying a
//! loading backlog. All storage is reserved before the callback starts.
use super::*;
use std::collections::VecDeque;

/// The most long sounds waiting for their decoder at once (one boombox track a player).
const TRACKS: usize = 64;
/// A long sound still not decoded after this long is given up.
const TRACK_PATIENCE: Duration = Duration::from_secs(30);

pub(super) struct PendingStarts {
    starts: VecDeque<(SoundHandle, PlayRequest, u64)>,
    /// Long sounds started from a moment in, waiting for their decoder: they start as far
    /// in as the time they waited.
    tracks: Vec<(SoundHandle, PlayRequest, MusicOffset)>,
    block: u64,
}

/// Start `handle` as far in as `offset` says now; false while it is not decoded. One
/// whose offset is past its end plays nothing.
pub(super) fn start_track(
    mixer: &mut Mixer,
    handle: SoundHandle,
    request: PlayRequest,
    offset: MusicOffset,
) -> bool {
    let Some(length) = mixer.sound_bank().sample_count(handle) else {
        return false;
    };
    let samples = offset
        .position(Instant::now())
        .saturating_mul(u64::from(SAMPLE_RATE))
        / 1000;
    if let Ok(samples) = usize::try_from(samples)
        && samples < length
    {
        mixer.play_from(handle, request, samples);
    }
    true
}

impl PendingStarts {
    pub fn new() -> Self {
        Self {
            starts: VecDeque::with_capacity(VOICES),
            tracks: Vec::with_capacity(TRACKS),
            block: 0,
        }
    }

    pub fn play(&mut self, mixer: &mut Mixer, handle: SoundHandle, request: PlayRequest) {
        if request.channel != ChannelId(0) {
            self.cancel(request.source, request.channel);
        }
        if mixer.sound_bank().sample_count(handle).is_some() {
            mixer.play(handle, request);
        } else {
            if self.starts.len() == VOICES {
                self.starts.pop_front();
            }
            self.starts.push_back((handle, request, self.block));
        }
    }

    /// Start a long sound `offset` in, or once it is decoded, as far in as it is then.
    pub fn play_track(
        &mut self,
        mixer: &mut Mixer,
        handle: SoundHandle,
        request: PlayRequest,
        offset: MusicOffset,
    ) {
        self.cancel(request.source, request.channel);
        if !start_track(mixer, handle, request, offset) {
            if self.tracks.len() == TRACKS {
                self.tracks.remove(0);
            }
            self.tracks.push((handle, request, offset));
        }
    }

    pub fn cancel(&mut self, source: SourceId, channel: ChannelId) {
        self.starts
            .retain(|(_, r, _)| r.source != source || r.channel != channel);
        self.tracks
            .retain(|(_, r, _)| r.source != source || r.channel != channel);
    }

    pub fn clear(&mut self) {
        self.starts.clear();
        self.tracks.clear();
    }

    pub fn flush(&mut self, mixer: &mut Mixer) {
        // Event validity window (codemp EVENT_VALID_MSEC): 300 ms.
        let maximum_age = u64::from(SAMPLE_RATE) * 300 / 1000 / (MIX_SAMPLES as u64 / 2);
        self.starts.retain(|(handle, request, queued)| {
            if self.block.saturating_sub(*queued) > maximum_age {
                return false;
            }
            if mixer.sound_bank().sample_count(*handle).is_some() {
                mixer.play(*handle, *request);
                false
            } else {
                true
            }
        });
        let now = Instant::now();
        self.tracks.retain(|(handle, request, offset)| {
            now.saturating_duration_since(offset.queued_at) < TRACK_PATIENCE
                && !start_track(mixer, *handle, *request, *offset)
        });
        self.block = self.block.wrapping_add(1);
    }
}
