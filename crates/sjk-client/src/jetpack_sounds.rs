//! JoF EJK #43 sound choices (`cg_main.c`, `cg_players.c`, `cg_xcvar.h`).

/// Both ignition choices, then the fixed shutdown, preloaded with a world.
pub const LEGACY_JETPACK_SWITCH_SOUNDS: [&str; 3] = [
    "sound/chars/boba/bf_land.mp3",
    "sound/chars/boba/jeton.wav",
    "sound/chars/boba/bf_blast-off.mp3",
];

/// Both hover choices, registered before frame-loop selection.
pub const LEGACY_JETPACK_HOVER_SOUNDS: [&str; 2] = [
    "sound/chars/boba/jethover.wav",
    "sound/chars/boba/bf_jetpack_lp.wav",
];

/// Archived `cg_jetpackOnSound` and `cg_jetpackHoverSound` presentation choices.
/// EJK selects the default for every value <= 1, the alternate for values > 1.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LegacyJetpackSounds {
    alternate_ignition: bool,
    alternate_hover: bool,
}

impl LegacyJetpackSounds {
    /// Interpret the original integer cvars without introducing an off mode.
    pub const fn from_cvars(ignition: i64, hover: i64) -> Self {
        Self {
            alternate_ignition: ignition > 1,
            alternate_hover: hover > 1,
        }
    }

    /// Ignition for the first active frame, or shutdown after the jets stop.
    pub fn switch_path(self, ignition: bool) -> &'static str {
        LEGACY_JETPACK_SWITCH_SOUNDS[if ignition {
            usize::from(self.alternate_ignition)
        } else {
            2
        }]
    }

    /// EJK #87: start at the wearer's origin on CHAN_AUTO, with normal falloff.
    pub fn switch_request(
        self,
        ignition: bool,
        wearer: u16,
        origin: [f32; 3],
    ) -> (&'static str, sjk_audio::PlayRequest) {
        (
            self.switch_path(ignition),
            sjk_audio::PlayRequest {
                source: sjk_audio::SourceId(u32::from(wearer)),
                origin: Some(origin),
                channel: sjk_audio::ChannelId(0),
                volume: 1.0,
                attenuation: crate::legacy_sound_attenuation(0),
            },
        )
    }

    pub(crate) fn hover_index(self) -> usize {
        usize::from(self.alternate_hover)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choices_follow_ejk_integer_branches_and_shutdown_is_fixed() {
        for value in [i64::MIN, -1, 0, 1, 2, 3, i64::MAX] {
            let options = LegacyJetpackSounds::from_cvars(value, value);
            let index = usize::from(value > 1);
            assert_eq!(
                options.switch_path(true),
                LEGACY_JETPACK_SWITCH_SOUNDS[index]
            );
            assert_eq!(options.hover_index(), index);
            assert_eq!(options.switch_path(false), LEGACY_JETPACK_SWITCH_SOUNDS[2]);
        }
        assert_eq!(
            LegacyJetpackSounds::default(),
            LegacyJetpackSounds::from_cvars(1, 1)
        );
    }

    #[test]
    fn switches_follow_the_actual_wearer_on_auto_channel() {
        for wearer in [0, 7, 31] {
            for ignition in [true, false] {
                let options = LegacyJetpackSounds::from_cvars(2, 2);
                let (path, request) = options.switch_request(ignition, wearer, [80., 20., 4.]);
                assert_eq!(path, options.switch_path(ignition));
                assert_eq!(request.source, sjk_audio::SourceId(u32::from(wearer)));
                assert_eq!(request.channel, sjk_audio::ChannelId(0));
                assert_eq!(request.origin, Some([80., 20., 4.]));
                assert_eq!(request.attenuation, crate::legacy_sound_attenuation(0));
            }
        }
    }

    #[test]
    fn hover_choice_changes_live_for_local_and_remote_without_changing_fire() {
        use crate::{LegacyLoopKind, LegacySoundAdapter};
        use sjk_protocol::{EntityState, GameState, LEGACY_ENTITY_FIELDS, PlayerState, Snapshot};
        let mut vfs = sjk_vfs::VirtualFileSystem::new();
        vfs.mount_memory(
            "synthetic-jetpack-sounds",
            LEGACY_JETPACK_HOVER_SOUNDS
                .into_iter()
                .chain(["sound/effects/fire_lp.wav"])
                .map(|path| (path, vec![1])),
        )
        .unwrap();
        let mut handle = 0;
        let mut adapter = LegacySoundAdapter::new(&GameState::empty_local(0), &vfs, |_, _| {
            handle += 1;
            Some(sjk_audio::SoundHandle(handle))
        });
        let flags = (1 << 29) | (1 << 11) | (1 << 30); // worn, active, flaming
        let e_flags = LEGACY_ENTITY_FIELDS
            .iter()
            .position(|field| field.name == "eFlags")
            .unwrap();
        let mut remote = EntityState::zero(7, &LEGACY_ENTITY_FIELDS);
        remote.set_raw_field(8, 1); // ET_PLAYER
        remote.set_raw_field(e_flags, flags);
        let mut player = PlayerState::zero();
        player.stats[0] = 100;
        player.set_raw_field(17, flags); // playerState eFlags
        let mut snapshot = Snapshot {
            message_sequence: 0,
            reliable_acknowledge: 0,
            server_commands: vec![],
            server_time: 1000,
            delta_from: None,
            flags: 0,
            area_mask: vec![],
            player,
            vehicle_player: None,
            entities: vec![remote],
            consumed_bits: 0,
        };
        let storage = adapter.loop_decisions().as_ptr();
        for choice in [1, 2, 1] {
            adapter.set_jetpack_sounds(LegacyJetpackSounds::from_cvars(1, choice));
            adapter.observe_loops(&snapshot, 1000, [0., 0., 64.], |_| [400., 0., 64.]);
            let jetpack = adapter
                .loop_decisions()
                .iter()
                .filter(|decision| {
                    matches!(
                        decision.kind,
                        LegacyLoopKind::JetpackHover | LegacyLoopKind::JetpackFire
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(jetpack.len(), 4);
            for decision in jetpack {
                let expected = if decision.kind == LegacyLoopKind::JetpackHover {
                    LEGACY_JETPACK_HOVER_SOUNDS[usize::from(choice > 1)]
                } else {
                    "sound/effects/fire_lp.wav"
                };
                assert_eq!(
                    &*adapter.loop_sound(decision.sound.unwrap()).unwrap().path,
                    expected
                );
                assert!(decision.handle.is_some());
                let remote = decision.request.source == sjk_audio::SourceId(7);
                assert!(remote || decision.request.source == sjk_audio::SourceId(0));
                assert_eq!(
                    decision.request.origin,
                    Some(if remote {
                        [400., 0., 64.]
                    } else {
                        [0., 0., 64.]
                    })
                );
            }
            assert_eq!(adapter.loop_decisions().as_ptr(), storage);
        }
        snapshot.player.set_raw_field(17, flags | 2); // dead wearers have no loops
        snapshot.entities[0].set_raw_field(e_flags, flags | 2);
        adapter.observe_loops(&snapshot, 1000, [0.; 3], |_| [400., 0., 64.]);
        assert!(!adapter.loop_decisions().iter().any(|d| matches!(
            d.kind,
            LegacyLoopKind::JetpackHover | LegacyLoopKind::JetpackFire
        )));
    }
}
