//! Snapshot-driven JA+ flame sample repeats and immediate channel stops.
use super::*;
use crate::actor_world_submission::force_powers::{Caster, flame};
/// OpenJK codemp CHAN_WEAPON, ET_PLAYER and ET_NPC.
const WEAPON_CHANNEL: ChannelId = ChannelId(2);
const ACTOR_TYPES: [u8; 2] = [1, 13];

#[derive(Clone, Copy, Default)]
struct Voice {
    timer: flame::Timer,
    playing: bool,
    seen: bool,
}

#[derive(Debug, PartialEq, Eq)]
enum Cue {
    None,
    Start,
    Stop,
}
impl Voice {
    fn advance(&mut self, active: bool, time: i32, interval: i32) -> Cue {
        self.seen = true;
        let cue = if self.playing && !active {
            Cue::Stop
        } else if self.timer.due(active, time, interval) {
            Cue::Start
        } else {
            Cue::None
        };
        if !active {
            self.timer = Default::default();
        }
        self.playing = active;
        cue
    }
}

pub(super) struct State {
    pub(super) detected: Option<&'static flame::Contract>,
    pub(super) override_value: i64,
    voices: Box<[Voice]>,
    any_playing: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            detected: None,
            override_value: -1,
            voices: vec![Voice::default(); 1024].into_boxed_slice(),
            any_playing: false,
        }
    }
}

#[derive(Clone, Copy)]
struct Frame<'a> {
    rule: &'static flame::Contract,
    clock: i32,
    time: i32,
    world: &'a World,
}

impl GameAudio {
    /// Repeating one-shots on CHAN_WEAPON, as EJK #31; never decode in this path.
    pub(super) fn update_flame(
        &mut self,
        snapshot: &Snapshot,
        time: i32,
        world: &World,
        predicted: Option<&sjk_client::pmove::MovementState>,
    ) {
        let Some(rule) = flame::overridden(self.flame.detected, self.flame.override_value) else {
            self.stop_flame();
            return;
        };
        let frame = Frame {
            rule,
            clock: snapshot.server_time,
            time,
            world,
        };
        self.flame.any_playing = false;
        for voice in &mut self.flame.voices {
            voice.seen = false;
        }
        let local = snapshot.player.client_num();
        let viewer_powers = predicted.map_or(snapshot.player.force_powers_active(), |p| {
            p.force_powers_active
        });
        self.flame_actor(
            local,
            Caster::from_player(&snapshot.player, predicted),
            predicted.map_or(snapshot.player.origin(), |p| p.origin),
            frame,
        );
        for entity in &snapshot.entities {
            if entity.number() == local || !ACTOR_TYPES.contains(&entity.entity_type()) {
                continue;
            }
            self.flame_actor(
                entity.number(),
                Caster::from_entity(entity, local, viewer_powers),
                entity.trajectory_base(),
                frame,
            );
        }
        // Disappearing players, time gaps and disabled support must not leave a tail.
        for (slot, voice) in self.flame.voices.iter_mut().enumerate() {
            if !voice.seen {
                if voice.playing {
                    self.output.send(AudioCommand::StopChannel(
                        SourceId(slot as u32),
                        WEAPON_CHANNEL,
                    ));
                }
                *voice = Voice::default();
            }
        }
    }

    fn flame_actor(&mut self, slot: u16, caster: Caster, fallback: [f32; 3], frame: Frame<'_>) {
        let Frame {
            rule,
            clock,
            time,
            world,
        } = frame;
        let audible = !self.mute.silences(SourceId(u32::from(slot)), None);
        let Some(voice) = self.flame.voices.get_mut(usize::from(slot)) else {
            return;
        };
        let active = flame_audible(rule, caster, audible);
        let cue = voice.advance(active, clock, rule.sound_millis);
        self.flame.any_playing |= active;
        if cue == Cue::Stop {
            self.output.send(AudioCommand::StopChannel(
                SourceId(u32::from(slot)),
                WEAPON_CHANNEL,
            ));
        }
        if cue == Cue::Start {
            let origin = if caster.local {
                fallback
            } else {
                world
                    .entity(EntityId::new(u64::from(slot) + 1))
                    .map_or(fallback, |entity| {
                        entity.sample(i64::from(time)).translation
                    })
            };
            self.play_on_channel(
                rule.sound,
                1.0,
                origin,
                SourceId(u32::from(slot)),
                WEAPON_CHANNEL,
            );
        }
    }

    pub(super) fn stop_flame(&mut self) {
        if !self.flame.any_playing {
            return;
        }
        self.flame.any_playing = false;
        for (slot, voice) in self.flame.voices.iter_mut().enumerate() {
            if voice.playing {
                self.output.send(AudioCommand::StopChannel(
                    SourceId(slot as u32),
                    WEAPON_CHANNEL,
                ));
            }
            *voice = Voice::default();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flame_sound_repeats_and_stops_at_integer_frame_steps() {
        for step in [8, 7, 4, 3] {
            let mut voice = Voice::default();
            let interval = flame::CONTRACTS[0].sound_millis;
            assert_eq!(voice.advance(true, 1000, interval), Cue::Start);
            for time in (1000 + step..=3900).step_by(step as usize) {
                assert_eq!(voice.advance(true, time, interval), Cue::None);
            }
            assert_eq!(voice.advance(true, 3900 + step, interval), Cue::Start);
            assert_eq!(voice.advance(false, 3901 + step, interval), Cue::Stop);
            assert_eq!(voice.advance(false, 3902 + step, interval), Cue::None);
            assert_eq!(voice.advance(true, 3903 + step, interval), Cue::Start);
            assert_eq!(
                voice.advance(true, 100, interval),
                Cue::Start,
                "rewound clock"
            );
        }
    }
}

/// A hidden caster's flamethrower must not advertise its position through audio.
fn flame_audible(rule: &flame::Contract, caster: Caster, audible: bool) -> bool {
    rule.firing(caster) && !caster.tricks_viewer && audible
}

#[cfg(test)]
mod mindtrick_tests {
    use super::*;
    use sjk_protocol::{EntityState, LEGACY_ENTITY_FIELDS};
    #[test]
    fn hidden_flames_stop_and_force_sight_can_resume_them() {
        let rule = flame::overridden(None, 1).unwrap();
        let mut entity = EntityState::zero(2, &LEGACY_ENTITY_FIELDS);
        entity.set_raw_field(19, 1 << 17);
        entity.set_raw_field(68, 1);
        entity.set_raw_field(58, 1);
        let hidden = Caster::from_entity(&entity, 0, 0);
        let seen = Caster::from_entity(&entity, 0, 1 << 14);
        assert!(!flame_audible(rule, hidden, true));
        assert!(flame_audible(rule, seen, true));
        let mut voice = Voice::default();
        assert_eq!(
            voice.advance(flame_audible(rule, seen, true), 1000, rule.sound_millis),
            Cue::Start
        );
        assert_eq!(
            voice.advance(flame_audible(rule, hidden, true), 1008, rule.sound_millis),
            Cue::Stop
        );
        assert_eq!(
            voice.advance(flame_audible(rule, seen, true), 1016, rule.sound_millis),
            Cue::Start
        );
    }
}
