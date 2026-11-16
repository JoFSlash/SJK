//! JoF EJK #123 snapshot external-event cases; no game assets or simulation harness.
use super::*;
use sjk_protocol::PlayerState;

fn adapter() -> LegacySoundAdapter {
    LegacySoundAdapter::new(
        &GameState::empty_local(0),
        &VirtualFileSystem::new(),
        |_, _| None,
    )
}
fn snapshot(time: i32, client: u16, event: u16) -> Snapshot {
    let mut player = PlayerState::zero();
    player.set_raw_field(43, u32::from(client));
    player.set_raw_field(56, u32::from(event));
    player.set_raw_field(64, 51); // authoritative EV_FALL strength, including land voice
    Snapshot {
        message_sequence: 1,
        reliable_acknowledge: 0,
        server_commands: vec![],
        server_time: time,
        delta_from: None,
        flags: 0,
        area_mask: vec![],
        player,
        vehicle_player: None,
        entities: vec![],
        consumed_bits: 0,
    }
}
fn falls(adapter: &LegacySoundAdapter) -> usize {
    adapter
        .decisions()
        .iter()
        .filter(|decision| decision.event == LegacySoundEvent::Fall)
        .count()
}

#[test]
fn authoritative_fall_plays_once_and_clearing_rearms_at_each_command_step() {
    for step in [8, 7, 4, 3] {
        let mut adapter = adapter();
        adapter.observe_snapshot(&snapshot(1000, 0, 0));
        adapter.observe_snapshot(&snapshot(1000 + step, 0, 0x100 | 11));
        assert_eq!(falls(&adapter), 2, "impact and voice at {step} ms");
        adapter.observe_snapshot(&snapshot(1000 + 2 * step, 0, 0x100 | 11));
        assert_eq!(falls(&adapter), 0);
        adapter.observe_snapshot(&snapshot(1000 + 3 * step, 0, 0));
        adapter.observe_snapshot(&snapshot(1000 + 4 * step, 0, 0x100 | 11));
        assert_eq!(falls(&adapter), 2);
    }
}

#[test]
fn follow_switch_seeds_the_new_players_pending_event_then_accepts_new_events() {
    for step in [8, 7, 4, 3] {
        let mut adapter = adapter();
        adapter.observe_snapshot(&snapshot(1000, 0, 0x100 | 11));
        adapter.observe_snapshot(&snapshot(1000 + step, 2, 0x200 | 11));
        assert_eq!(falls(&adapter), 0, "follow switch at {step} ms");
        adapter.observe_snapshot(&snapshot(1000 + 2 * step, 2, 0x200 | 11));
        assert_eq!(falls(&adapter), 0);
        adapter.observe_snapshot(&snapshot(1000 + 3 * step, 2, 0x300 | 11));
        assert_eq!(falls(&adapter), 2);
        assert!(
            adapter
                .decisions()
                .iter()
                .all(|decision| decision.request.source == SourceId(2))
        );
    }
}

#[test]
fn follow_switch_seeds_the_new_players_event_ring() {
    let ring = |time, client: u16, sequence: u32| {
        let mut state = snapshot(time, client, 0);
        state.player.set_raw_field(19, sequence);
        // EV_FALL in the slot of event sequence - 1, with its strength.
        let (event, parameter) = if sequence & 1 == 0 {
            (28, 60)
        } else {
            (27, 65)
        };
        state.player.set_raw_field(event, 11);
        state.player.set_raw_field(parameter, 51);
        state
    };
    let mut adapter = adapter();
    adapter.observe_snapshot(&ring(1000, 0, 100));
    adapter.observe_snapshot(&ring(1008, 2, 500));
    assert_eq!(falls(&adapter), 0);
    adapter.observe_snapshot(&ring(1016, 2, 501));
    assert_eq!(falls(&adapter), 2);
}

#[test]
fn explicit_synchronization_seeds_both_event_and_view_identity() {
    let mut adapter = adapter();
    adapter.synchronize(&snapshot(1000, 2, 0x100 | 11));
    adapter.observe_snapshot(&snapshot(1008, 2, 0x100 | 11));
    assert_eq!(falls(&adapter), 0);
    adapter.observe_snapshot(&snapshot(1016, 0, 0x200 | 11));
    assert_eq!(falls(&adapter), 0);
    adapter.observe_snapshot(&snapshot(1024, 0, 0x300 | 11));
    assert_eq!(falls(&adapter), 2);
}
