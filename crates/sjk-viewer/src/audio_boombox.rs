//! The boombox toy's music (`boombox.rs`): a track per player, on its own source
//! ([`crate::boombox::source`]) on its own protected channel, started from where the
//! song is when its file is decoded, heard up to the carrier's range.
use super::*;

impl GameAudio {
    /// Play the track at `path` for player `slot` from `offset` in, at `origin`
    /// (`None`: the player's own, in both ears), fading out to nothing `range` units
    /// away; nothing for a player muted here or with the toy's sounds off. A track not
    /// decoded yet starts once it is, as far in as the song is by then.
    pub(crate) fn play_boombox(
        &mut self,
        vfs: &VirtualFileSystem,
        path: &str,
        slot: u8,
        origin: Option<[f32; 3]>,
        offset: std::time::Duration,
        range: f32,
    ) {
        let source = crate::boombox::source(slot);
        if self.mute.silences(SourceId(u32::from(slot)), None) {
            return;
        }
        let Some(volume) = self.toy_sounds.gain(crate::toy_sounds::Toy::Boombox, 1.0) else {
            return;
        };
        let Some(handle) = self.register_vfs_async(vfs, path) else {
            return;
        };
        let full = range * crate::boombox::FULL_SHARE;
        self.output.send(AudioCommand::PlayTrack(
            handle,
            PlayRequest {
                origin,
                source,
                channel: BOOMBOX_CHANNEL,
                volume,
                attenuation: match origin {
                    Some(_) => sjk_audio::Attenuation::Linear {
                        full_volume_distance: full,
                        falloff_per_unit: 1.0 / (range - full).max(1.0),
                    },
                    None => sjk_audio::Attenuation::None,
                },
            },
            u64::try_from(offset.as_millis()).unwrap_or(u64::MAX),
            Instant::now(),
        ));
    }

    /// Stop the track player `slot`'s boombox plays.
    pub(crate) fn stop_boombox(&mut self, slot: u8) {
        self.output.send(AudioCommand::StopChannel(
            crate::boombox::source(slot),
            BOOMBOX_CHANNEL,
        ));
    }

    /// Move player `slot`'s music to where they are.
    pub(crate) fn move_boombox(&mut self, slot: u8, origin: [f32; 3]) {
        self.output.send(AudioCommand::SourcePosition(
            crate::boombox::source(slot),
            origin,
        ));
    }

    /// How long the sound at `path` lasts, once decoded.
    pub(crate) fn sound_length(&self, path: &str) -> Option<std::time::Duration> {
        let length = self.output.sample_length_ms(self.find_handle(path)?);
        (length > 0).then(|| std::time::Duration::from_millis(u64::from(length)))
    }

    /// Free the decoded sound at `path`: nothing plays it any more. Asked for again, it
    /// is read and decoded again.
    pub(crate) fn forget_sound(&mut self, path: &str) {
        let Some(handle) = self.find_handle(path) else {
            return;
        };
        self.handles.retain(|_, kept| *kept != handle);
        self.output.release(handle);
    }
}
