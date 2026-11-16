//! JoF EJK #287/#374 player voice holds. Fixed storage; no mixer or simulation changes.
use sjk_client::LegacyVoiceLine;

const CS_PLAYERS: usize = 1_131;

#[derive(Clone, Copy, Default)]
struct Timer {
    start: Option<i32>,
    hold_ms: u32,
}

pub(super) struct VoiceThrottle {
    pub(super) enabled: bool,
    timers: [[Timer; 5]; 32],
    /// Each slot's occupant as seen in CS_PLAYERS: its name, or None when empty.
    owners: [Option<Vec<u8>>; 32],
    last_time: Option<i32>,
}

impl Default for VoiceThrottle {
    fn default() -> Self {
        Self {
            enabled: true,
            timers: [[Timer::default(); 5]; 32],
            owners: Default::default(),
            last_time: None,
        }
    }
}

impl VoiceThrottle {
    /// A backwards seek resets every slot, including slots absent from that snapshot.
    pub(super) fn observe_time(&mut self, time: i32) {
        if self.last_time.is_some_and(|last| time < last) {
            self.reset();
        }
        self.last_time = Some(time);
    }

    pub(super) fn reset(&mut self) {
        self.timers = [[Timer::default(); 5]; 32];
        self.last_time = None;
    }

    /// A slot that empties or passes to another player (a new name in its
    /// CS_PLAYERS string) drops the previous occupant's holds, so the newcomer's
    /// first taunt is not silenced by them.
    pub(super) fn observe_owner(&mut self, client: usize, info: &[u8]) {
        let Some(owner) = self.owners.get_mut(client) else {
            return;
        };
        let name = (!info.is_empty()).then(|| info_name(info));
        if owner.as_deref() != name {
            *owner = name.map(<[u8]>::to_vec);
            self.timers[client] = [Timer::default(); 5];
        }
    }

    /// [`Self::observe_owner`] for every CS_PLAYERS slot; allocates only on a change.
    pub(super) fn observe_clients(&mut self, game: &sjk_protocol::GameState) {
        for client in 0..self.owners.len() {
            self.observe_owner(
                client,
                game.config_string(CS_PLAYERS + client).unwrap_or_default(),
            );
        }
    }

    /// NPC/world sources bypass the reference's MAX_CLIENTS player-only policy.
    pub(super) fn allows(
        &mut self,
        source: u32,
        family: Option<LegacyVoiceLine>,
        time: i32,
        length_ms: u32,
    ) -> bool {
        if !self.enabled {
            return true;
        }
        let Some(family) = family else {
            return true;
        };
        let Some(slot) = self.timers.get_mut(source as usize) else {
            return true;
        };
        let timer = &mut slot[family as usize];
        if timer.start.is_some_and(|start| {
            time >= start && i64::from(time) - i64::from(start) < i64::from(timer.hold_ms)
        }) {
            return false;
        }
        *timer = Timer {
            start: Some(time),
            hold_ms: if family == LegacyVoiceLine::Taunt && length_ms > 0 {
                length_ms
            } else {
                family.fallback_ms()
            },
        };
        true
    }
}

/// The `n` value of a player info string (empty when absent).
fn info_name(info: &[u8]) -> &[u8] {
    let mut fields = info.split(|byte| *byte == b'\\').skip(1);
    while let (Some(key), Some(value)) = (fields.next(), fields.next()) {
        if key == b"n" {
            return value;
        }
    }
    &[]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_taunts_hold_until_actual_previous_sample_ends_including_time_zero() {
        let mut state = VoiceThrottle::default();
        assert!(state.allows(1, Some(LegacyVoiceLine::Taunt), 0, 4300));
        assert!(!state.allows(1, Some(LegacyVoiceLine::Taunt), 4299, 10));
        assert!(state.allows(1, Some(LegacyVoiceLine::Taunt), 4300, 200));
        assert!(!state.allows(1, Some(LegacyVoiceLine::Taunt), 4499, 9000));
        assert!(state.allows(1, Some(LegacyVoiceLine::Taunt), 4500, 0));
        assert!(!state.allows(1, Some(LegacyVoiceLine::Taunt), 6999, 1));
        assert!(state.allows(1, Some(LegacyVoiceLine::Taunt), 7000, 1));
    }
    #[test]
    fn independent_sources_families_and_fixed_grunt_holds() {
        let mut state = VoiceThrottle::default();
        for family in [
            LegacyVoiceLine::Gasp,
            LegacyVoiceLine::Jump,
            LegacyVoiceLine::Roll,
            LegacyVoiceLine::Land,
        ] {
            assert!(state.allows(2, Some(family), 100, 99999));
            assert!(!state.allows(2, Some(family), 100 + family.fallback_ms() as i32 - 1, 1));
            assert!(state.allows(3, Some(family), 100, 1));
            assert!(state.allows(2, Some(family), 100 + family.fallback_ms() as i32, 1));
        }
        for _ in 0..2 {
            assert!(state.allows(32, Some(LegacyVoiceLine::Taunt), 100, 999));
            assert!(state.allows(2, None, 100, 999));
        }
    }
    #[test]
    fn disabled_map_reset_and_backwards_seek_do_not_keep_old_holds() {
        let mut state = VoiceThrottle::default();
        state.observe_time(10000);
        assert!(state.allows(2, Some(LegacyVoiceLine::Taunt), 10000, 9999));
        state.enabled = false;
        assert!(state.allows(2, Some(LegacyVoiceLine::Taunt), 10001, 9999));
        state.enabled = true;
        state.observe_time(9000);
        assert!(state.allows(2, Some(LegacyVoiceLine::Taunt), 10000, 10));
        state.reset();
        assert!(state.allows(2, Some(LegacyVoiceLine::Taunt), 10000, 10));
    }
    #[test]
    fn a_reused_slot_does_not_inherit_the_previous_players_hold() {
        let mut state = VoiceThrottle::default();
        state.observe_owner(4, br"\n\Alice\t\1\model\kyle");
        assert!(state.allows(4, Some(LegacyVoiceLine::Taunt), 1000, 5000));
        // The same player changing team or model keeps the hold.
        state.observe_owner(4, br"\n\Alice\t\2\model\jan");
        assert!(!state.allows(4, Some(LegacyVoiceLine::Taunt), 2000, 10));
        // Alice leaves (the string empties) and Bob takes the slot.
        state.observe_owner(4, b"");
        state.observe_owner(4, br"\n\Bob\t\1\model\kyle");
        assert!(state.allows(4, Some(LegacyVoiceLine::Taunt), 2000, 10));
        // A replacement seen in one refresh, without the empty string between.
        state.observe_owner(5, br"\n\Carol");
        assert!(state.allows(5, Some(LegacyVoiceLine::Taunt), 1000, 5000));
        state.observe_owner(5, br"\n\Dave");
        assert!(state.allows(5, Some(LegacyVoiceLine::Taunt), 2000, 10));
        // Other slots keep their holds; slots past MAX_CLIENTS are ignored.
        assert!(!state.allows(4, Some(LegacyVoiceLine::Taunt), 2005, 10));
        state.observe_owner(40, b"");
    }
    #[test]
    #[ignore = "release microbenchmark; run explicitly with --release --ignored --nocapture"]
    fn release_voice_throttle_cost() {
        let mut state = VoiceThrottle::default();
        let start = std::time::Instant::now();
        for time in 0..20000 {
            state.observe_time(time * 8);
            for source in 0..128 {
                std::hint::black_box(state.allows(
                    std::hint::black_box(source),
                    Some(LegacyVoiceLine::Taunt),
                    time * 8,
                    2500,
                ));
            }
        }
        eprintln!(
            "voice throttle: {:.3} us / 128 decisions",
            start.elapsed().as_secs_f64() * 1e6 / 20000.0
        );
    }
}
