//! 16-bit sample storage: what the mixer hears matches the float samples it
//! replaced, released sounds free their PCM, and mixing never allocates.
//! Synthetic PCM only; no game data.

use super::*;

const RATE: u32 = 44_100;
/// Interleaved stereo samples in one rendered block.
const BLOCK: usize = 256;
/// The largest error 16-bit storage may add to a rendered sample.
const STEP: f32 = 1.0 / 32_768.0;

fn mixer() -> Mixer {
    Mixer::new(MixerConfig {
        voices: 8,
        loops: 4,
        sample_rate: RATE,
        loop_attenuation: Attenuation::None,
        doppler: DopplerConfig::disabled(),
        protected_channels: &[],
    })
}

fn request(source: u32, volume: f32) -> PlayRequest {
    PlayRequest {
        origin: None,
        source: SourceId(source),
        channel: ChannelId(0),
        volume,
        attenuation: Attenuation::None,
    }
}

/// A float sound no 16-bit value holds exactly: two detuned sines.
fn float_sound(length: usize) -> Vec<f32> {
    (0..length)
        .map(|n| {
            let t = n as f32;
            0.6 * (t * 0.031).sin() + 0.25 * (t * 0.177 + 1.0).sin()
        })
        .collect()
}

fn render(mixer: &mut Mixer, blocks: usize) -> Vec<f32> {
    let mut out = vec![0.0; BLOCK * blocks];
    for block in out.chunks_exact_mut(BLOCK) {
        mixer.render(block);
    }
    out
}

fn assert_within_a_step(rendered: &[f32], expected: impl Fn(usize) -> f32) {
    for (index, frame) in rendered.chunks_exact(2).enumerate() {
        let want = expected(index);
        for (ear, sample) in frame.iter().enumerate() {
            assert!(
                (sample - want).abs() <= STEP,
                "frame {index} ear {ear}: {sample} against {want}"
            );
        }
    }
}

/// A 16-bit mono WAV of `samples` at `rate` Hz.
fn wav(samples: &[i16], rate: u32) -> Vec<u8> {
    let size = (samples.len() * 2) as u32;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + size).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&rate.to_le_bytes());
    bytes.extend_from_slice(&(rate * 2).to_le_bytes());
    bytes.extend_from_slice(&2_u16.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&size.to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    bytes
}

#[test]
fn a_16_bit_wav_is_kept_exactly_and_in_half_the_bytes_of_floats() {
    let source: Vec<i16> = (0..4_000)
        .map(|n| ((n * 7_919) % 65_536 - 32_768) as i16)
        .chain([i16::MIN, -1, 0, 1, i16::MAX])
        .collect();
    let decoded = decode_encoded(&wav(&source, RATE), "wav", RATE).unwrap();
    assert_eq!(decoded.samples(), &source[..]);
    assert_eq!(decoded.byte_len(), source.len() * 2);
    // The float a stored sample stands for is the one the old decoder made.
    for sample in source {
        assert_eq!(sample_value(sample), f32::from(sample) / 32_768.0);
        assert_eq!(quantize_sample(sample_value(sample)), sample);
    }
}

#[test]
fn a_resampled_wav_stays_within_half_a_step_of_the_float_resampling() {
    let source: Vec<i16> = float_sound(2_205)
        .into_iter()
        .map(quantize_sample)
        .collect();
    let decoded = decode_encoded(&wav(&source, 22_050), "wav", RATE).unwrap();
    let floats = decode_mono(&wav(&source, 22_050), "wav", RATE).unwrap();
    assert_eq!(decoded.samples().len(), 4_410);
    assert_eq!(floats.len(), 4_410);
    for (stored, float) in decoded.samples().iter().zip(&floats) {
        assert!((sample_value(*stored) - float).abs() <= STEP / 2.0);
    }
}

#[test]
fn a_channel_gain_replaces_the_master_gain_for_that_channel_only() {
    let source = float_sound(2_000);
    let floats = resample(&source, 22_050, RATE, std::convert::identity, |x| x);
    let voice_channel = ChannelId(3);
    let emote_channel = ChannelId(13);
    let on_voice = PlayRequest {
        channel: voice_channel,
        ..request(1, 1.0)
    };
    // Two buses at once: each channel takes its own gain (the voice volume, and the
    // emotes' full gain while their loudness is their own, #163).
    for (channel, gain) in [(voice_channel, 0.2), (emote_channel, 1.0)] {
        let mut voices = mixer();
        voices.set_master_gain(0.5);
        voices.set_channel_gain(voice_channel, Some(0.2));
        voices.set_channel_gain(emote_channel, Some(1.0));
        let sound = voices.sound_bank_mut().register_pcm(&source, 22_050);
        assert!(voices.play(
            sound,
            PlayRequest {
                channel,
                ..on_voice
            }
        ));
        let rendered = render(&mut voices, 60);
        assert_within_a_step(&rendered, |n| {
            floats.get(n).map_or(0.0, |sample| sample * gain)
        });
    }
    // Another channel keeps the master gain, and so does every voice once the bus ends.
    for bus in [Some(0.2), None] {
        let mut others = mixer();
        others.set_master_gain(0.5);
        others.set_channel_gain(voice_channel, bus);
        let sound = others.sound_bank_mut().register_pcm(&source, 22_050);
        let request = if bus.is_some() {
            request(1, 1.0)
        } else {
            on_voice
        };
        assert!(others.play(sound, request));
        let rendered = render(&mut others, 60);
        assert_within_a_step(&rendered, |n| {
            floats.get(n).map_or(0.0, |sample| sample * 0.5)
        });
    }
}

#[test]
fn voices_loops_and_music_sound_as_their_float_samples_did() {
    // Resampled from 22.05 kHz like most retail effects: the float path kept
    // exactly these resampled floats; 16-bit storage may differ by one step.
    let source = float_sound(3_000);
    let floats = resample(&source, 22_050, RATE, std::convert::identity, |x| x);
    let (master, music) = (0.8, 0.6);

    let mut voice = mixer();
    voice.set_master_gain(master);
    let sound = voice.sound_bank_mut().register_pcm(&source, 22_050);
    assert_eq!(voice.sound_bank().sample_count(sound), Some(floats.len()));
    assert!(voice.play(sound, request(1, 0.7)));
    // Past the sound's end, which is silence.
    let rendered = render(&mut voice, 60);
    assert_within_a_step(&rendered, |n| {
        floats.get(n).map_or(0.0, |sample| sample * master * 0.7)
    });

    let mut looping = mixer();
    looping.set_master_gain(master);
    let sound = looping.sound_bank_mut().register_pcm(&source, 22_050);
    looping.clear_loops();
    assert!(looping.set_loop_request(sound, request(2, 0.9), [0.0; 3]));
    looping.commit_loops();
    // Longer than the sound: the loop wraps.
    let rendered = render(&mut looping, 60);
    assert_within_a_step(&rendered, |n| floats[n % floats.len()] * 0.9 * master);

    let mut track = mixer();
    track.set_music_gain(music);
    let sound = track.sound_bank_mut().register_pcm(&source, 22_050);
    assert!(track.start_background_track(sound, sound, 0.5));
    let rendered = render(&mut track, 60);
    assert_within_a_step(&rendered, |n| floats[n % floats.len()] * 0.5 * music);
}

#[test]
fn a_released_sound_hands_back_its_samples_and_plays_no_more() {
    let mut mixer = mixer();
    let effect = mixer.sound_bank_mut().register_pcm(&[0.25; 1_000], RATE);
    let music = mixer
        .sound_bank_mut()
        .register_decoded(DecodedSound::from_pcm16(vec![1_000; 50_000]));
    assert_eq!(mixer.sound_bank().sample_bytes(), (1_000 + 50_000) * 2);
    assert!(mixer.start_background_track(music, music, 1.0));
    assert!(mixer.play(music, request(1, 1.0)));
    assert!(mixer.play(effect, request(2, 1.0)));

    let released = mixer.release_sound(music).expect("registered");
    assert_eq!(released.byte_len(), 100_000);
    assert_eq!(released.samples().len(), 50_000);
    assert_eq!(mixer.sound_bank().sample_bytes(), 2_000);
    // Everything that played it stopped; the other sound plays on.
    assert_eq!(mixer.background_track_state(), None);
    assert_eq!(mixer.active_voice_count(), 1);
    assert_eq!(mixer.sound_bank().sample_count(music), None);
    assert!(!mixer.play(music, request(3, 1.0)));
    assert!(!mixer.start_background_track(music, music, 1.0));
    assert!(mixer.release_sound(music).is_none(), "released once");
    // Handles keep their slots: the next one is new, the old ones unchanged.
    let next = mixer
        .sound_bank_mut()
        .register_decoded(DecodedSound::silence());
    assert_eq!(next, SoundHandle(2));
    assert_eq!(mixer.sound_bank().len(), 3);
    assert_eq!(mixer.sound_bank().sample_count(effect), Some(1_000));
    let block = render(&mut mixer, 1);
    assert!((block[0] - 0.25).abs() <= STEP);
}

/// The audio thread's whole block: voices, a Doppler-shifted loop and music,
/// then releasing a sound, without one allocator call on this thread.
#[cfg(debug_assertions)]
#[test]
fn mixing_and_releasing_never_allocate() {
    let mut mixer = Mixer::new(MixerConfig {
        voices: 8,
        loops: 4,
        sample_rate: RATE,
        loop_attenuation: Attenuation::Linear {
            full_volume_distance: 256.0,
            falloff_per_unit: 0.0008,
        },
        doppler: DopplerConfig {
            enabled: true,
            distance_ratio_divisor: 1.0,
            maximum_scale: 2.0,
        },
        protected_channels: &[ChannelId(13)],
    });
    let source = float_sound(5_000);
    let effect = mixer.sound_bank_mut().register_pcm(&source, 22_050);
    let music = mixer.sound_bank_mut().register_pcm(&source, RATE);
    let mut out = vec![0.0; BLOCK];
    let before = assert_no_alloc::violation_count();
    let released = assert_no_alloc::assert_no_alloc(|| {
        for block in 0..40_u32 {
            mixer.play(effect, request(block % 3, 0.8));
            mixer.play(
                effect,
                PlayRequest {
                    origin: Some([300.0, 50.0, 0.0]),
                    channel: ChannelId(13),
                    attenuation: Attenuation::Linear {
                        full_volume_distance: 256.0,
                        falloff_per_unit: 0.0008,
                    },
                    ..request(9, 1.0)
                },
            );
            mixer.clear_loops();
            mixer.set_loop(SourceId(4), effect, [400.0, 0.0, 0.0], [-900.0, 0.0, 0.0]);
            mixer.commit_loops();
            if block == 0 {
                mixer.start_background_track(music, music, 0.5);
            }
            mixer.render(&mut out);
        }
        mixer.release_sound(music)
    });
    assert_eq!(assert_no_alloc::violation_count(), before);
    // Freed here, outside the guarded block.
    assert!(released.is_some());
    assert!(out.iter().any(|sample| *sample != 0.0));
}
