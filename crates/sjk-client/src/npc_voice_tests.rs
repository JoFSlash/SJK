use super::*;
use sjk_protocol::{LEGACY_ENTITY_FIELDS, PlayerState};

const CS_PLAYERS: usize = 1_131;
const EV_JUMP: u16 = 16;
const EV_PAIN: u16 = 89;
const EV_ENTITY_SOUND: u16 = 79;
const EV_GENERAL_SOUND: u16 = 76;
/// `CS_SOUNDS` slots the test server fills.
const STD: u8 = 10;
const JEDI: u8 = 11;
const DEATH: u8 = 12;
const FALLING: u8 = 13;
const CHOKE: u8 = 14;
const GASP: u8 = 15;
const NPC: u16 = 100;
const SILENT_NPC: u16 = 101;

fn vfs() -> VirtualFileSystem {
    let mut vfs = VirtualFileSystem::new();
    let files: Vec<(String, Vec<u8>)> = [
        "sound/chars/stormtrooper/misc/pain25.wav",
        "sound/chars/stormtrooper/misc/death1.wav",
        // In the trooper's folder, but `*deflect1` belongs to the Jedi set.
        "sound/chars/stormtrooper/misc/deflect1.wav",
        "sound/chars/jedi2/misc/deflect1.wav",
        "sound/chars/kyle/misc/pain25.wav",
        "sound/chars/kyle/misc/jump1.wav",
        "sound/chars/kyle/misc/death1.wav",
        "sound/chars/kyle/misc/falling1.wav",
        "sound/chars/kyle/misc/choke1.wav",
        "sound/chars/kyle/misc/gasp.wav",
        "sound/chars/mp_generic_male/misc/pain25.wav",
        "sound/chars/mp_generic_male/misc/jump1.wav",
        "sound/chars/mp_generic_male/misc/death1.wav",
    ]
    .into_iter()
    .map(|path| (path.to_owned(), b"wav".to_vec()))
    .collect();
    vfs.mount_memory("test", files).unwrap();
    vfs
}

fn game_state() -> GameState {
    let mut game = GameState::empty_local(0);
    for (slot, path) in [
        (STD, "*$stormtrooper"),
        (JEDI, "*$jedi2"),
        (DEATH, "*death1.wav"),
        (FALLING, "*falling1.wav"),
        (CHOKE, "*choke1.wav"),
        (GASP, "*gasp.wav"),
    ] {
        game.replace_config_string(CS_SOUNDS + usize::from(slot), path.as_bytes().to_vec())
            .unwrap();
    }
    // The last player slot, whose voice NPCs used to borrow.
    game.replace_config_string(CS_PLAYERS + 31, b"n\\Kyle\\model\\kyle/default".to_vec())
        .unwrap();
    game
}

fn adapter() -> LegacySoundAdapter {
    let mut next = 0;
    LegacySoundAdapter::new(&game_state(), &vfs(), |_, _| {
        next += 1;
        Some(SoundHandle(next))
    })
}

/// An `ET_NPC` whose clientNum, like stock's, is its own number.
fn npc(number: u16, std: u8, jedi: u8) -> EntityState {
    let mut state = EntityState::zero(number, &LEGACY_ENTITY_FIELDS);
    state.set_raw_field(8, u32::from(crate::npc_identity::ET_NPC));
    state.set_raw_field(32, u32::from(number));
    state.set_raw_field(80, u32::from(std));
    state.set_raw_field(105, u32::from(jedi));
    state
}

fn with_event(mut state: EntityState, event: u16, parameter: u8) -> EntityState {
    state.set_raw_field(28, u32::from(event));
    state.set_raw_field(42, u32::from(parameter));
    state
}

/// `G_EntitySound`'s temporary entity: `EV_ENTITY_SOUND` naming `client`.
fn entity_sound(number: u16, client: u16, slot: u8) -> EntityState {
    temp_event(number, EV_ENTITY_SOUND, client, slot)
}

fn temp_event(number: u16, event: u16, client: u16, slot: u8) -> EntityState {
    let mut state = EntityState::zero(number, &LEGACY_ENTITY_FIELDS);
    state.set_raw_field(8, u32::from(ET_EVENTS) + u32::from(event));
    state.set_raw_field(32, u32::from(client));
    state.set_raw_field(42, u32::from(slot));
    state
}

fn snapshot(entities: Vec<EntityState>) -> Snapshot {
    let mut player = PlayerState::zero();
    player.set_client_num(0);
    Snapshot {
        message_sequence: 1,
        reliable_acknowledge: 0,
        server_commands: Vec::new(),
        server_time: 1_000,
        delta_from: None,
        flags: 0,
        area_mask: Vec::new(),
        player,
        vehicle_player: None,
        entities,
        consumed_bits: 0,
    }
}

/// The audible paths the snapshot's events play, by source.
fn played(adapter: &mut LegacySoundAdapter, entities: Vec<EntityState>) -> Vec<(u16, String)> {
    adapter.observe_snapshot(&snapshot(entities));
    adapter
        .decisions()
        .iter()
        .filter_map(|decision| {
            let sound = adapter.sound(decision.sound?)?;
            sound.handle?;
            Some((decision.request.source.0 as u16, sound.path.to_string()))
        })
        .collect()
}

fn path(adapter: &LegacySoundAdapter, index: Option<u16>) -> Option<String> {
    Some(adapter.sound(index?)?.path.to_string())
}

#[test]
fn an_npc_cries_in_its_own_voice() {
    let mut adapter = adapter();
    let sounds = played(
        &mut adapter,
        vec![with_event(npc(NPC, STD, JEDI), EV_PAIN, 10)],
    );
    assert_eq!(
        sounds,
        [(NPC, "sound/chars/stormtrooper/misc/pain25.wav".to_owned())]
    );
}

#[test]
fn an_npc_without_a_sound_is_silent_not_a_player() {
    let mut adapter = adapter();
    let sounds = played(
        &mut adapter,
        vec![
            with_event(npc(NPC, STD, 0), EV_JUMP, 0),
            with_event(npc(SILENT_NPC, 0, 0), EV_PAIN, 10),
        ],
    );
    // Neither Kyle in slot 31 nor `mp_generic_male`: the trooper has no jump1 and
    // the other NPC no set at all.
    assert_eq!(sounds, []);
}

#[test]
fn each_name_comes_from_its_own_set() {
    let adapter = adapter();
    let state = npc(NPC, STD, JEDI);
    let voice = adapter.voice_set(adapter.voice_of(&EventSubject::entity(&state), NPC));
    assert_eq!(
        path(&adapter, voice.death[0]).as_deref(),
        Some("sound/chars/stormtrooper/misc/death1.wav")
    );
    // `*deflect1` is a Jedi-set name: the trooper's own file is not used.
    assert_eq!(
        path(&adapter, voice.deflect[0]).as_deref(),
        Some("sound/chars/jedi2/misc/deflect1.wav")
    );
    assert_eq!(voice.jump, None);
}

#[test]
fn an_entity_sound_naming_an_npc_uses_its_voice() {
    let mut adapter = adapter();
    let sounds = played(
        &mut adapter,
        vec![npc(NPC, STD, JEDI), entity_sound(400, NPC, DEATH)],
    );
    assert_eq!(
        sounds,
        [(NPC, "sound/chars/stormtrooper/misc/death1.wav".to_owned())]
    );
    // A number past the player slots that is no NPC has no voice (`npcClient` NULL).
    let mut adapter = self::adapter();
    assert_eq!(
        played(&mut adapter, vec![entity_sound(400, 200, DEATH)]),
        []
    );
}

#[test]
fn players_keep_their_own_voice() {
    let mut adapter = adapter();
    let mut player = EntityState::zero(31, &LEGACY_ENTITY_FIELDS);
    player.set_raw_field(8, 1);
    player.set_raw_field(32, 31);
    let sounds = played(&mut adapter, vec![with_event(player, EV_PAIN, 10)]);
    assert_eq!(
        sounds,
        [(31, "sound/chars/kyle/misc/pain25.wav".to_owned())]
    );
}

#[test]
fn a_changed_slot_reloads_its_set() {
    let mut adapter = adapter();
    let vfs = vfs();
    let mut game = game_state();
    game.replace_config_string(CS_SOUNDS + usize::from(STD), b"*$kyle".to_vec())
        .unwrap();
    let mut next = 100;
    adapter.refresh_sound_table(CS_SOUNDS + usize::from(STD), &game, &vfs, |_, _| {
        next += 1;
        Some(SoundHandle(next))
    });
    let sounds = played(&mut adapter, vec![with_event(npc(NPC, STD, 0), EV_JUMP, 0)]);
    assert_eq!(
        sounds,
        [(NPC, "sound/chars/kyle/misc/jump1.wav".to_owned())]
    );
}

/// Pits (`trigger_hurt`'s `*falling1`) and Grip (`*choke1`, `*gasp`) voice a
/// player through `G_EntitySound`: a temporary entity past the player slots
/// whose `clientNum` is the speaker (`cg_event.c:4174`), never an NPC lookup.
#[test]
fn entity_sounds_naming_a_player_use_the_players_voice() {
    let mut adapter = adapter();
    for (number, slot, name) in [
        (400, FALLING, "falling1"),
        (401, CHOKE, "choke1"),
        (402, GASP, "gasp"),
    ] {
        let sounds = played(&mut adapter, vec![entity_sound(number, 31, slot)]);
        // The source is the player's own number, which blocking silences.
        assert_eq!(
            sounds,
            [(31, format!("sound/chars/kyle/misc/{name}.wav"))],
            "{name}"
        );
    }
}

/// `EV_GENERAL_SOUND` voices `es->number`: a custom name on a temporary
/// entity (`G_Sound`) has no `clientinfo` and stays silent, as in stock,
/// rather than borrowing whichever player its `clientNum` names.
#[test]
fn a_general_sound_on_a_temporary_entity_has_no_voice() {
    let mut adapter = adapter();
    assert_eq!(
        played(
            &mut adapter,
            vec![temp_event(400, EV_GENERAL_SOUND, 31, FALLING)]
        ),
        []
    );
    // On the player's own entity it is that player's voice.
    let mut player = EntityState::zero(31, &LEGACY_ENTITY_FIELDS);
    player.set_raw_field(8, 1);
    player.set_raw_field(32, 31);
    let sounds = played(
        &mut adapter,
        vec![with_event(player, EV_GENERAL_SOUND, FALLING)],
    );
    assert_eq!(
        sounds,
        [(31, "sound/chars/kyle/misc/falling1.wav".to_owned())]
    );
}

/// `EV_TAUNT1`-`3` share the taunt anti-spam hold (JoF EJK #287); the other
/// vocal events have none.
#[test]
fn numbered_taunts_carry_the_taunt_voice_line() {
    let mut adapter = adapter();
    for (number, event, line) in [
        (100, 175, Some(crate::LegacyVoiceLine::Taunt)),
        (101, 177, Some(crate::LegacyVoiceLine::Taunt)),
        (102, 116, None),
    ] {
        adapter.observe_snapshot(&snapshot(vec![with_event(
            npc(number, STD, JEDI),
            event,
            0,
        )]));
        let decision = adapter.decisions().last().unwrap();
        assert_eq!(decision.voice_line, line, "event {event}");
    }
}
