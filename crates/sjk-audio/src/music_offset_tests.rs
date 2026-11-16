//! Synthetic PCM phase checks; no files or game data.
use super::*;
fn mixer() -> (Mixer, SoundHandle, SoundHandle) {
    let mut mixer = Mixer::new(MixerConfig {
        voices: 1,
        loops: 0,
        sample_rate: 1000,
        loop_attenuation: Attenuation::None,
        doppler: DopplerConfig::disabled(),
        protected_channels: &[],
    });
    let intro = mixer
        .sound_bank_mut()
        .register_decoded(DecodedSound::from_samples(vec![0.1; 10]));
    let repeating = mixer
        .sound_bank_mut()
        .register_decoded(DecodedSound::from_samples(vec![0.2; 7]));
    (mixer, intro, repeating)
}
#[test]
fn music_offset_preserves_intro_then_wraps_the_loop() {
    let (mut mixer, intro, repeating) = mixer();
    for (elapsed, in_intro, cursor) in [
        (0, true, 0),
        (9, true, 9),
        (10, false, 0),
        (16, false, 6),
        (17, false, 0),
        (25, false, 1),
    ] {
        assert!(mixer.start_background_track_at(intro, repeating, 1.0, elapsed));
        assert_eq!(
            mixer.background_track_state(),
            Some(BackgroundTrackState { in_intro, cursor })
        );
    }
    assert!(mixer.start_background_track_at(intro, repeating, 1.0, u64::MAX));
    assert_eq!(
        mixer.background_track_state().unwrap().cursor,
        ((u128::from(u64::MAX) - 10) % 7) as usize
    );
}
#[test]
fn music_offset_is_one_start_only_and_invalid_handles_do_not_replace_the_track() {
    let (mut mixer, intro, repeating) = mixer();
    assert!(mixer.start_background_track_at(intro, repeating, 1.0, 25));
    assert!(!mixer.start_background_track_at(SoundHandle(u32::MAX), repeating, 1.0, 99));
    assert_eq!(mixer.background_track_state().unwrap().cursor, 1);
    assert!(mixer.start_background_track(intro, repeating, 1.0));
    assert_eq!(
        mixer.background_track_state(),
        Some(BackgroundTrackState {
            in_intro: true,
            cursor: 0
        })
    );
}
#[test]
fn music_offset_enters_loop_sample_exactly_during_render() {
    let (mut mixer, intro, repeating) = mixer();
    assert!(mixer.start_background_track_at(intro, repeating, 1.0, 9));
    mixer.render(&mut [0.0; 4]);
    assert_eq!(
        mixer.background_track_state(),
        Some(BackgroundTrackState {
            in_intro: false,
            cursor: 1
        })
    );
}
