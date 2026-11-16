//! EJK #342 node-local voice: ENTITYNUM_NONE/CHAN_VOICE_GLOBAL, no attenuation.
//! One desired revision replaces or stops the channel, including pending decodes.
use super::{AudioCommand, ChannelId, GameAudio, PlayRequest, SourceId, VirtualFileSystem};
const SOURCE: SourceId = SourceId(1023);
const CHANNEL: ChannelId = ChannelId(12);

#[derive(Default)]
pub(super) struct Voice {
    revision: Option<u64>,
    sounding: bool,
}
impl GameAudio {
    pub(super) fn sync_dialogue(
        &mut self,
        desired: Option<(u64, Option<&str>)>,
        vfs: Option<&VirtualFileSystem>,
    ) {
        let revision = desired.map(|(revision, _)| revision);
        let path = desired.and_then(|(_, path)| path);
        if self.dialogue.revision == revision && self.dialogue.sounding == path.is_some() {
            return;
        }
        self.output.send(AudioCommand::StopChannel(SOURCE, CHANNEL));
        self.dialogue.revision = revision;
        // Mark the revision consumed even if its optional installed sound is missing.
        self.dialogue.sounding = path.is_some();
        if let (Some(path), Some(vfs)) = (path, vfs) {
            if let Some(handle) = self.register_vfs_async(vfs, path) {
                self.output.send(AudioCommand::Play(
                    handle,
                    PlayRequest {
                        origin: None,
                        source: SOURCE,
                        channel: CHANNEL,
                        volume: 1.0,
                        attenuation: sjk_audio::Attenuation::None,
                    },
                ));
            } else {
                crate::log::progress(format_args!("Dialogue: missing installed sound '{path}'"));
            }
        }
    }
}
