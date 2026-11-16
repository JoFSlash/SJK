//! Weapon-selection ownership at accepted snapshot boundaries.
//!
//! JoF EJK #39 (58de9cedaec5adc1fb427bf02e8674c66475fdb8), multiplayer
//! `CG_TransitionPlayerState`, resets weaponSelect/time when clientNum changes.
//! Native follow/spectator boundaries can retain clientNum, so track those too.
use sjk_protocol::PlayerState;
use std::time::Instant;

/// Multiplayer `bg_public.h`, `PMF_FOLLOW`.
const PMF_FOLLOW: u16 = 4096;

/// The viewed player's identity and the last accepted snapshot clock.
#[derive(Default)]
pub(crate) struct Tracker {
    previous: Option<(u16, bool, bool)>,
    time: Option<i32>,
}

impl Tracker {
    /// Rebase a retained weapon choice only when its owning view changes or rewinds.
    /// Ordinary weapon acknowledgements and duplicate snapshots preserve local input.
    pub(crate) fn observe(
        &mut self,
        player: &PlayerState,
        time: i32,
        selected: &mut Option<u8>,
        selected_at: &mut Option<Instant>,
        now: Instant,
    ) -> bool {
        let identity = (
            player.client_num(),
            player.movement_flags() & PMF_FOLLOW != 0,
            player.is_spectator(),
        );
        let changed = self
            .previous
            .replace(identity)
            .is_some_and(|old| old != identity)
            || self.time.is_some_and(|old| time < old);
        self.time = Some(time);
        if changed {
            *selected = Some(player.weapon());
            *selected_at = Some(now);
        }
        changed
    }
}

impl crate::GpuState {
    /// Run before prediction and command submission consume the newly accepted view.
    pub(crate) fn sync_weapon_selection_view(&mut self, snapshot: &sjk_protocol::Snapshot) {
        if self.weapon_selection_view.observe(
            &snapshot.player,
            snapshot.server_time,
            &mut self.selected_weapon,
            &mut self.weapon_selected_at,
            Instant::now(),
        ) {
            self.auto_switch
                .rebase_view(&snapshot.player, snapshot.server_time);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player(client: u16, weapon: u8, follow: bool, spectator: bool) -> PlayerState {
        let mut player = PlayerState::zero();
        player.set_client_num(client);
        player.set_raw_field(47, u32::from(weapon));
        player.set_movement_flags(if follow { PMF_FOLLOW } else { 0 });
        player.set_movement_type(if spectator { 4 } else { 0 });
        player
    }

    #[test]
    fn weapon_view_rebases_between_follow_targets_and_back_to_own_saber() {
        let mut tracker = Tracker::default();
        let start = Instant::now();
        let mut selected = Some(5);
        let mut at = Some(start);
        assert!(!tracker.observe(
            &player(7, 5, true, false),
            1000,
            &mut selected,
            &mut at,
            start
        ));
        let next = start + std::time::Duration::from_millis(8);
        assert!(tracker.observe(
            &player(9, 11, true, false),
            1008,
            &mut selected,
            &mut at,
            next
        ));
        assert_eq!((selected, at), (Some(11), Some(next)));
        assert!(tracker.observe(
            &player(3, 3, false, false),
            1016,
            &mut selected,
            &mut at,
            next
        ));
        assert_eq!(selected, Some(3));
    }

    #[test]
    fn weapon_view_handles_same_client_follow_and_spectator_boundaries() {
        for (old_follow, old_spec, new_follow, new_spec) in [
            (false, false, true, false),
            (true, false, false, false),
            (false, false, false, true),
            (false, true, false, false),
        ] {
            let mut tracker = Tracker::default();
            let now = Instant::now();
            let mut selected = Some(11);
            let mut at = None;
            tracker.observe(
                &player(3, 11, old_follow, old_spec),
                0,
                &mut selected,
                &mut at,
                now,
            );
            assert!(tracker.observe(
                &player(3, 3, new_follow, new_spec),
                8,
                &mut selected,
                &mut at,
                now
            ));
            assert_eq!((selected, at), (Some(3), Some(now)));
        }
    }

    #[test]
    fn weapon_view_keeps_pending_input_on_regular_snapshots_and_rebases_on_rewind() {
        for step in [8, 7, 4, 3] {
            let mut tracker = Tracker::default();
            let now = Instant::now();
            let mut selected = Some(5);
            let mut at = Some(now);
            for time in (1000..1200).step_by(step) {
                assert!(!tracker.observe(
                    &player(3, 3, false, false),
                    time,
                    &mut selected,
                    &mut at,
                    now
                ));
                assert!(!tracker.observe(
                    &player(3, 3, false, false),
                    time,
                    &mut selected,
                    &mut at,
                    now
                ));
                assert_eq!((selected, at), (Some(5), Some(now)));
            }
            assert!(tracker.observe(&player(3, 3, false, false), 0, &mut selected, &mut at, now));
            assert_eq!(selected, Some(3));
        }
    }
}
