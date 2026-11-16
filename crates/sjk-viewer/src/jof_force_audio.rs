//! Audio host for the optional JoF mod; no community parser or policy in base SJK.
use super::*;
#[derive(Default)]
pub(super) struct State {
    gate: sjk_mod_jof::feedback::Gate,
    prepared: bool,
    source: Option<SourceId>,
}
impl GameAudio {
    pub(crate) fn sync_force_feedback(&mut self, enabled: bool) {
        if !enabled {
            if let Some(source) = self.force_feedback.source.take() {
                self.output
                    .send(AudioCommand::StopChannel(source, ChannelId(1)));
            }
            self.force_feedback.gate = Default::default();
        }
    }

    pub(crate) fn prepare_force_feedback(&mut self, vfs: &VirtualFileSystem) {
        self.register_effect_paths(
            vfs,
            sjk_mod_jof::feedback::FEEDBACK
                .iter()
                .map(|entry| entry.sound),
        );
        self.force_feedback.prepared = true;
    }
    pub(crate) fn force_feedback(&mut self, index: usize, snapshot: &Snapshot) {
        if !self.force_feedback.prepared {
            let Some(vfs) = self.legacy_vfs.clone() else {
                return;
            };
            self.prepare_force_feedback(&vfs);
        }
        if let Some(entry) = self.force_feedback.gate.accept(index, snapshot.server_time) {
            self.force_feedback.source = Some(SourceId(u32::from(snapshot.player.client_num())));
            self.play_local(
                entry.sound,
                1.0,
                SourceId(u32::from(snapshot.player.client_num())),
                ChannelId(1),
            );
        }
    }
}
