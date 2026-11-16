use super::*;
use sjk_protocol::{LEGACY_ENTITY_FIELDS, PlayerState};

fn fixture() -> (GameState, VirtualFileSystem, LegacySoundAdapter) {
    let mut game = GameState::empty_local(0);
    let mut files = Vec::new();
    for (bank, names) in NAMES.iter().enumerate() {
        let directory = format!("npc_bank{bank}");
        game.replace_config_string(CS_SOUNDS + bank + 1, format!("*${directory}").into_bytes())
            .unwrap();
        for name in *names {
            files.push((
                format!("sound/chars/{directory}/misc/{name}.wav"),
                b"synthetic voice".to_vec(),
            ));
        }
    }
    for (slot, name) in [
        (10, "anger1"),
        (11, "taunt1"),
        (12, "deflect1"),
        (13, "choke1"),
        (14, "suspicious5"),
        (15, "falling1"),
    ] {
        game.replace_config_string(CS_SOUNDS + slot, format!("*{name}.wav").into_bytes())
            .unwrap();
    }
    game.replace_config_string(
        1131 + MAX_CLIENTS - 1,
        br"\model\test_player/default".to_vec(),
    )
    .unwrap();
    files.push((
        "models/players/test_player/sounds.cfg".into(),
        b"player_voice\nm\n".to_vec(),
    ));
    files.push((
        "sound/chars/player_voice/misc/anger1.wav".into(),
        b"synthetic player".to_vec(),
    ));
    files.push((
        "sound/chars/replacement/misc/anger1.mp3".into(),
        b"synthetic replacement".to_vec(),
    ));
    files.push((
        "sound/player/roll1.wav".into(),
        b"synthetic roll body cue".to_vec(),
    ));
    let mut vfs = VirtualFileSystem::new();
    vfs.mount_memory("synthetic", files).unwrap();
    let adapter = LegacySoundAdapter::new(&game, &vfs, |_, _| Some(SoundHandle(7)));
    (game, vfs, adapter)
}

fn npc(event: u16, parameter: u8) -> EntityState {
    let mut entity = EntityState::zero(100, &LEGACY_ENTITY_FIELDS);
    for (field, value) in [
        (8, 13),
        (28, u32::from(event)),
        (42, u32::from(parameter)),
        (32, (MAX_CLIENTS - 1) as u32),
    ] {
        assert!(entity.set_raw_field(field, value));
    }
    for (field, value) in BANK_FIELDS.into_iter().zip(1..=4) {
        assert!(entity.set_raw_field(field, value));
    }
    entity
}

fn snapshot(time: i32, entities: Vec<EntityState>) -> Snapshot {
    Snapshot {
        message_sequence: 0,
        reliable_acknowledge: 0,
        server_commands: Vec::new(),
        server_time: time,
        delta_from: None,
        flags: 0,
        area_mask: Vec::new(),
        player: PlayerState::zero(),
        vehicle_player: None,
        entities,
        consumed_bits: 0,
    }
}

fn path(adapter: &LegacySoundAdapter) -> Option<&str> {
    let sound = adapter.decisions().last()?.sound?;
    let registered = adapter.sound(sound)?;
    registered.handle.map(|_| registered.path.as_ref())
}

#[test]
fn npc_voice_fields_and_all_vocal_events_use_their_own_banks() {
    for (field, name) in BANK_FIELDS.into_iter().zip([
        "csSounds_Std",
        "csSounds_Combat",
        "csSounds_Extra",
        "csSounds_Jedi",
    ]) {
        assert_eq!(LEGACY_ENTITY_FIELDS[field].name, name);
    }
    let (_, _, mut adapter) = fixture();
    let count = adapter.sounds().len();
    for event in 116..=190 {
        let Some(key) = event_key(event) else {
            continue;
        };
        adapter.observe_snapshot(&snapshot(1000 + i32::from(event), vec![npc(event, 0)]));
        let expected = format!(
            "sound/chars/npc_bank{}/misc/{}.wav",
            key.bank, NAMES[key.bank][key.index]
        );
        assert_eq!(path(&adapter), Some(expected.as_str()), "event {event}");
        let decision = adapter.decisions().last().unwrap();
        assert_eq!(decision.request.source, SourceId(100));
        assert_eq!(decision.request.channel, ChannelId(CHAN_VOICE));
    }
    assert_eq!(
        adapter.sounds().len(),
        count,
        "playback never registers assets"
    );
}

#[test]
fn configured_npc_voices_do_not_hit_the_player_duel_bank() {
    let (_, _, mut adapter) = fixture();
    for (slot, directory, name) in [
        (10, 1, "anger1"),
        (11, 3, "taunt1"),
        (12, 3, "deflect1"),
        (13, 0, "choke1"),
        (14, 2, "suspicious5"),
        (15, 0, "falling1"),
    ] {
        adapter.observe_snapshot(&snapshot(
            1000 + i32::from(slot),
            vec![npc(76 | (u16::from(slot % 4) << 8), slot)],
        ));
        let expected = format!("sound/chars/npc_bank{directory}/misc/{name}.wav");
        assert_eq!(path(&adapter), Some(expected.as_str()));
    }
}

#[test]
fn npc_standard_reactions_use_standard_and_combat_and_jedi_directories() {
    let (_, _, mut adapter) = fixture();
    for (event, parameter, bank, name) in [
        (16, 0, 0, "jump1"),
        (89, 100, 0, "pain100"),
        (90, 0, 0, "death1"),
        (125, 0, 1, "pushed1"),
        (128, 0, 0, "choke1"),
        (190, 0, 3, "pushfail"),
    ] {
        adapter.observe_snapshot(&snapshot(
            1000 + i32::from(event),
            vec![npc(event, parameter)],
        ));
        assert_eq!(
            path(&adapter),
            Some(format!("sound/chars/npc_bank{bank}/misc/{name}.wav").as_str())
        );
    }
    // Voice-bank support must not replace the independent roll body cue.
    adapter.observe_snapshot(&snapshot(1300, vec![npc(17, 0)]));
    let body = adapter.decisions().last().unwrap();
    assert_eq!(body.request.channel, ChannelId(CHAN_BODY));
    assert_eq!(path(&adapter), Some("sound/player/roll1.wav"));
    let voice = adapter
        .decisions()
        .iter()
        .find(|decision| decision.request.channel == ChannelId(CHAN_VOICE))
        .unwrap();
    assert_eq!(
        adapter.sound(voice.sound.unwrap()).unwrap().path.as_ref(),
        "sound/chars/npc_bank0/misc/jump1.wav"
    );
}

#[test]
fn missing_npc_banks_stay_silent_without_borrowing_a_player_voice() {
    let (_, _, mut adapter) = fixture();
    let mut entity = npc(116, 0);
    for field in BANK_FIELDS {
        entity.set_raw_field(field, 0);
    }
    adapter.observe_snapshot(&snapshot(1000, vec![entity]));
    assert_eq!(path(&adapter), None);
    assert!(adapter.decisions().last().unwrap().handle.is_none());
    // The player bank remains valid and independent.
    let mut player = EntityState::zero((MAX_CLIENTS - 1) as u16, &LEGACY_ENTITY_FIELDS);
    player.set_raw_field(32, (MAX_CLIENTS - 1) as u32);
    player.set_raw_field(28, 116);
    adapter.observe_snapshot(&snapshot(1100, vec![player]));
    assert_eq!(
        path(&adapter),
        Some("sound/chars/player_voice/misc/anger1.wav")
    );
    // Standard choke takes precedence even when only combat has that sample.
    let mut missing_standard = npc(128, 0);
    missing_standard.set_raw_field(BANK_FIELDS[0], 0);
    adapter.observe_snapshot(&snapshot(1200, vec![missing_standard]));
    assert_eq!(path(&adapter), None);
}

#[test]
fn refreshed_directories_tokens_and_removed_banks_take_effect_before_playback() {
    let (mut game, vfs, mut adapter) = fixture();
    game.replace_config_string(CS_SOUNDS + 2, b"*$replacement".to_vec())
        .unwrap();
    adapter.refresh_sound_table(CS_SOUNDS + 2, &game, &vfs, |_, _| Some(SoundHandle(8)));
    adapter.observe_snapshot(&snapshot(1000, vec![npc(116, 0)]));
    assert_eq!(
        path(&adapter),
        Some("sound/chars/replacement/misc/anger1.mp3")
    );
    adapter.remap_sound_handles(|handle| SoundHandle(handle.0 + 10));
    adapter.observe_snapshot(&snapshot(1005, vec![npc(116 | 0x100, 0)]));
    assert_eq!(
        adapter.decisions().last().unwrap().handle,
        Some(SoundHandle(18))
    );
    game.replace_config_string(CS_SOUNDS + 10, b"*taunt1.wav".to_vec())
        .unwrap();
    adapter.refresh_sound_table(CS_SOUNDS + 10, &game, &vfs, |_, _| Some(SoundHandle(8)));
    adapter.observe_snapshot(&snapshot(1010, vec![npc(76, 10)]));
    assert_eq!(
        path(&adapter),
        Some("sound/chars/npc_bank3/misc/taunt1.wav")
    );
    game.replace_config_string(CS_SOUNDS + 2, Vec::new())
        .unwrap();
    adapter.refresh_sound_table(CS_SOUNDS + 2, &game, &vfs, |_, _| Some(SoundHandle(8)));
    adapter.observe_snapshot(&snapshot(1020, vec![npc(116, 0)]));
    assert_eq!(path(&adapter), None);
}

#[test]
fn forwarded_events_and_entity_reuse_do_not_retain_old_npc_voices() {
    let (_, _, mut adapter) = fixture();
    let mut forwarded = EntityState::zero(900, &LEGACY_ENTITY_FIELDS);
    for (field, value) in [(8, u32::from(ET_EVENTS) + 116), (19, 1 << 5), (59, 100)] {
        forwarded.set_raw_field(field, value);
    }
    adapter.observe_snapshot(&snapshot(1000, vec![npc(0, 0), forwarded.clone()]));
    assert_eq!(
        path(&adapter),
        Some("sound/chars/npc_bank1/misc/anger1.wav")
    );
    assert_eq!(
        adapter.decisions().last().unwrap().request.source,
        SourceId(100)
    );
    adapter.observe_snapshot(&snapshot(1400, Vec::new()));
    forwarded.set_number(901).unwrap();
    adapter.observe_snapshot(&snapshot(1410, vec![forwarded]));
    assert_eq!(path(&adapter), None);
}

#[test]
#[ignore = "release-only NPC voice lookup measurement; not a game FPS benchmark"]
fn npc_voice_lookup_release_cost() {
    let (_, _, mut adapter) = fixture();
    let snap = snapshot(
        1000,
        (100..228)
            .map(|number| {
                let mut state = npc(116, 0);
                state.set_number(number).unwrap();
                state
            })
            .collect(),
    );
    let key = event_key(116).unwrap();
    let start = std::time::Instant::now();
    for _ in 0..20_000 {
        adapter.npc.observe(std::hint::black_box(&snap));
        for source in 100..228 {
            let refs = adapter.npc.source_banks(source).unwrap();
            std::hint::black_box(adapter.npc.custom_set(refs));
            std::hint::black_box(adapter.npc.get(refs, key));
        }
    }
    eprintln!(
        "128 NPC snapshot cache + voice lookups: {:.3} us/frame (20,000 synthetic frames)",
        start.elapsed().as_secs_f64() * 1_000_000. / 20_000.
    );
}
