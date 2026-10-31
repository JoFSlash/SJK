//! `cg_fleshSparks` silences only the special saber hit's sound, below 2, as JoF EJK.
use super::*;
use sjk_protocol::{LEGACY_ENTITY_FIELDS, PlayerState};

fn adapter(level: u8) -> LegacySoundAdapter {
    let mut vfs = VirtualFileSystem::new();
    vfs.mount_memory(
        "test",
        (1..=3).map(|variant| {
            (
                format!("sound/weapons/saber/saberhit{variant}.mp3"),
                b"fixture".to_vec(),
            )
        }),
    )
    .unwrap();
    let mut adapter = LegacySoundAdapter::new(&GameState::empty_local(0), &vfs, |_, _| {
        Some(SoundHandle(1))
    });
    adapter.set_flesh_sparks(level);
    adapter
}

/// The hit sounds one `EV_SABER_HIT` temporary entity with `parameter` plays.
fn hit_sounds(level: u8, parameter: u8) -> usize {
    let mut entity = EntityState::zero(300, &LEGACY_ENTITY_FIELDS);
    entity.set_raw_field(8, 18 + 30); // ET_EVENTS + EV_SABER_HIT
    entity.set_raw_field(42, u32::from(parameter)); // eventParm
    let snapshot = Snapshot {
        message_sequence: 1,
        reliable_acknowledge: 0,
        server_commands: Vec::new(),
        server_time: 1_000,
        delta_from: None,
        flags: 0,
        area_mask: Vec::new(),
        player: PlayerState::zero(),
        vehicle_player: None,
        entities: vec![entity],
        consumed_bits: 0,
    };
    let mut adapter = adapter(level);
    adapter.observe_snapshot(&snapshot);
    adapter
        .decisions()
        .iter()
        .filter(|decision| {
            decision.event == LegacySoundEvent::SaberHit && decision.handle.is_some()
        })
        .count()
}

#[test]
fn flesh_sparks_below_two_silence_only_the_special_saber_hit() {
    for level in 0..=7 {
        assert_eq!(
            hit_sounds(level, 16),
            usize::from(level > 1),
            "eventParm 16 at {level}"
        );
        for parameter in [1, 2, 3] {
            assert_eq!(
                hit_sounds(level, parameter),
                1,
                "eventParm {parameter} at {level}"
            );
        }
    }
}
