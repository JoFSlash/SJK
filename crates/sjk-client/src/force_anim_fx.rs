//! JoF EJK #58/#89 animation-driven Force presentation (`CG_RunTimedForceAnimFX`).

/// Effect selected by the full-body animation, independent of sound events.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LegacyForceAnimFx {
    /// `force/rage2`, attached to the chest.
    Rage,
    /// `force/heal2`, attached to the chest.
    Heal,
    /// Push-blur puffs at the left hand.
    QuickHeal,
}

impl LegacyForceAnimFx {
    /// Local presentation uses prediction (or player state), since snapshots omit
    /// the local entity. Other players and NPCs use their own entity state.
    pub fn from_snapshot(
        snapshot: &sjk_protocol::Snapshot,
        number: u16,
        predicted: Option<&crate::pmove::MovementState>,
    ) -> Option<Self> {
        if number == snapshot.player.client_num() {
            return predicted.map_or_else(
                || {
                    Self::select(
                        snapshot.player.leg_animation(),
                        snapshot.player.torso_animation(),
                        snapshot.player.velocity(),
                    )
                },
                |state| Self::select(state.legs_anim, state.torso_anim, state.velocity),
            );
        }
        let index = snapshot
            .entities
            .binary_search_by_key(&number, |state| state.number())
            .ok()?;
        let state = &snapshot.entities[index];
        Self::select(
            state.leg_animation(),
            state.torso_animation(),
            state.trajectory_delta(),
        )
    }

    /// Both animation tracks must match, with speed strictly below 250 units/s.
    pub fn select(legs: u16, torso: u16, velocity: [f32; 3]) -> Option<Self> {
        if legs != torso || velocity.iter().map(|v| v * v).sum::<f32>() >= 250.0 * 250.0 {
            return None;
        }
        match torso {
            1350 => Some(Self::Rage),      // BOTH_FORCE_RAGE
            1339 => Some(Self::Heal),      // BOTH_FORCEHEAL_START
            1341 => Some(Self::QuickHeal), // BOTH_FORCEHEAL_QUICK
            _ => None,
        }
    }
}

/// Map-lifetime timers indexed by protocol-26 entity number. No frame allocations.
pub struct LegacyForceAnimFxTimers {
    slots: Box<[Option<(i32, i32)>]>,
}

impl Default for LegacyForceAnimFxTimers {
    fn default() -> Self {
        Self {
            slots: vec![None; 1024].into_boxed_slice(),
        }
    }
}

impl LegacyForceAnimFxTimers {
    /// Chest effects repeat at 120 ms; quick-heal uses the renderer's 8 ms cadence.
    /// Inactivity resets the timer; a demo seek back restarts immediately.
    pub fn due(&mut self, entity: u16, effect: Option<LegacyForceAnimFx>, time: i32) -> bool {
        let Some(slot) = self.slots.get_mut(usize::from(entity)) else {
            return false;
        };
        let Some(effect) = effect else {
            *slot = None;
            return false;
        };
        if let Some((last, next)) = *slot
            && time >= last
            && time < next
        {
            return false;
        }
        let interval = if effect == LegacyForceAnimFx::QuickHeal {
            8
        } else {
            120
        };
        let next = if interval == 8 {
            match *slot {
                Some((last, next))
                    if time >= last && time >= next && time < next.saturating_add(8) =>
                {
                    next.saturating_add(8)
                }
                _ => time.saturating_add(8),
            }
        } else {
            time.saturating_add(interval)
        };
        *slot = Some((time, next));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_full_body_force_poses_and_speed_boundary() {
        for (anim, effect) in [
            (1350, LegacyForceAnimFx::Rage),
            (1339, LegacyForceAnimFx::Heal),
            (1341, LegacyForceAnimFx::QuickHeal),
        ] {
            assert_eq!(
                LegacyForceAnimFx::select(anim, anim, [249.0, 0.0, 0.0]),
                Some(effect)
            );
            assert_eq!(LegacyForceAnimFx::select(0, anim, [0.0; 3]), None);
            assert_eq!(
                LegacyForceAnimFx::select(anim, anim, [250.0, 0.0, 0.0]),
                None
            );
            assert_eq!(
                LegacyForceAnimFx::select(anim, anim, [200.0, 200.0, 0.0]),
                None
            );
        }
        assert_eq!(LegacyForceAnimFx::select(0, 0, [0.0; 3]), None);
    }

    #[test]
    fn timers_are_independent_reset_and_handle_demo_seeks() {
        let mut timers = LegacyForceAnimFxTimers::default();
        let effect = Some(LegacyForceAnimFx::Heal);
        assert!(timers.due(1, effect, 1000));
        assert!(!timers.due(1, effect, 1119));
        assert!(timers.due(2, effect, 1119));
        assert!(timers.due(1, effect, 1120));
        assert!(timers.due(1, effect, 900));
        assert!(!timers.due(1, None, 901));
        assert!(timers.due(1, effect, 902));
        assert!(!timers.due(1024, effect, 902));
    }

    #[test]
    fn quick_heal_is_bounded_at_all_command_steps() {
        for step in [8, 7, 4, 3] {
            let mut timers = LegacyForceAnimFxTimers::default();
            let emissions = (0..1000)
                .step_by(step)
                .filter(|time| timers.due(77, Some(LegacyForceAnimFx::QuickHeal), *time))
                .count();
            assert!(emissions <= 125);
            assert!(emissions >= 90);
        }
    }

    #[test]
    fn self_remote_player_and_npc_use_their_own_animation_state() {
        use sjk_protocol::{EntityState, LEGACY_ENTITY_FIELDS, PlayerState, Snapshot};
        let mut player = PlayerState::zero();
        player.set_client_num(3);
        player.set_raw_field(13, 1339);
        player.set_raw_field(15, 1339);
        let mut remote = EntityState::zero(7, &LEGACY_ENTITY_FIELDS);
        remote.set_raw_field(16, 1350);
        remote.set_raw_field(17, 1350);
        let mut npc = EntityState::zero(77, &LEGACY_ENTITY_FIELDS);
        npc.set_raw_field(16, 1341);
        npc.set_raw_field(17, 1341);
        let snapshot = Snapshot {
            message_sequence: 1,
            reliable_acknowledge: 0,
            server_commands: Vec::new(),
            server_time: 1000,
            delta_from: None,
            flags: 0,
            area_mask: Vec::new(),
            player,
            vehicle_player: None,
            entities: vec![remote, npc],
            consumed_bits: 0,
        };
        assert_eq!(
            LegacyForceAnimFx::from_snapshot(&snapshot, 3, None),
            Some(LegacyForceAnimFx::Heal)
        );
        assert_eq!(
            LegacyForceAnimFx::from_snapshot(&snapshot, 7, None),
            Some(LegacyForceAnimFx::Rage)
        );
        assert_eq!(
            LegacyForceAnimFx::from_snapshot(&snapshot, 77, None),
            Some(LegacyForceAnimFx::QuickHeal)
        );
        assert_eq!(LegacyForceAnimFx::from_snapshot(&snapshot, 78, None), None);
        let predicted = crate::pmove::MovementState {
            legs_anim: 1350,
            torso_anim: 1350,
            ..Default::default()
        };
        assert_eq!(
            LegacyForceAnimFx::from_snapshot(&snapshot, 3, Some(&predicted)),
            Some(LegacyForceAnimFx::Rage)
        );
        assert_eq!(
            LegacyForceAnimFx::from_snapshot(&snapshot, 77, Some(&predicted)),
            Some(LegacyForceAnimFx::QuickHeal)
        );
    }

    #[test]
    #[ignore = "release-only synthetic selection/timer measurement"]
    fn measure_force_anim_selection() {
        let mut timers = LegacyForceAnimFxTimers::default();
        let start = std::time::Instant::now();
        let mut emissions = 0_u64;
        for frame in 0..20_000 {
            for entity in 0..128 {
                let effect = LegacyForceAnimFx::select(std::hint::black_box(1350), 1350, [0.0; 3]);
                emissions += u64::from(timers.due(entity, effect, frame * 3));
            }
        }
        eprintln!(
            "128 actor selection/timers: {:.3} us/frame; emissions={}",
            start.elapsed().as_secs_f64() * 1e6 / 20_000.0,
            std::hint::black_box(emissions)
        );
    }
}
