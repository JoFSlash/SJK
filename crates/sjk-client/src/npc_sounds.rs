//! NPC voice directories advertised by `csSounds_*` through `CS_SOUNDS`.
//!
//! Evidence: JoF EJK #55 (`289e14ce`), `CG_CustomSound` in `cg_players.c`,
//! `CG_HandleNPCSounds`/`CG_RegisterCustomSounds` in `cg_servercmds.c`, and
//! `CG_EntityEvent` in `cg_event.c`. NPCs own four banks; the player duel
//! bank is not an NPC fallback. Registration is lifecycle work, never playback.

use super::*;

/// Original bank name order, without the leading `*`. Standard choke sounds
/// take precedence over the duplicate names in the combat bank.
const NAMES: [&[&str]; 4] = [
    &[
        "death1", "death2", "death3", "jump1", "pain25", "pain50", "pain75", "pain100", "falling1",
        "choke1", "choke2", "choke3", "gasp", "land1", "taunt", "roll",
    ],
    &[
        "anger1", "anger2", "anger3", "victory1", "victory2", "victory3", "confuse1", "confuse2",
        "confuse3", "pushed1", "pushed2", "pushed3", "choke1", "choke2", "choke3", "ffwarn",
        "ffturn",
    ],
    &[
        "chase1",
        "chase2",
        "chase3",
        "cover1",
        "cover2",
        "cover3",
        "cover4",
        "cover5",
        "detected1",
        "detected2",
        "detected3",
        "detected4",
        "detected5",
        "lost1",
        "outflank1",
        "outflank2",
        "escaping1",
        "escaping2",
        "escaping3",
        "giveup1",
        "giveup2",
        "giveup3",
        "giveup4",
        "look1",
        "look2",
        "sight1",
        "sight2",
        "sight3",
        "sound1",
        "sound2",
        "sound3",
        "suspicious1",
        "suspicious2",
        "suspicious3",
        "suspicious4",
        "suspicious5",
    ],
    &[
        "combat1",
        "combat2",
        "combat3",
        "jdetected1",
        "jdetected2",
        "jdetected3",
        "taunt1",
        "taunt2",
        "taunt3",
        "jchase1",
        "jchase2",
        "jchase3",
        "jlost1",
        "jlost2",
        "jlost3",
        "deflect1",
        "deflect2",
        "deflect3",
        "gloat1",
        "gloat2",
        "gloat3",
        "pushfail",
    ],
];

/// The protocol-26 fields [`EntityState::npc_sound_sets`] reads (Std, Combat,
/// Extra, Jedi), which the tests check by name. No codec changes.
#[cfg(test)]
const BANK_FIELDS: [usize; 4] = [80, 103, 104, 105];

/// An `ET_NPC`'s `csSounds_*` indices ([`EntityState::npc_sound_sets`]).
pub(super) fn entity_banks(state: &EntityState) -> Option<[u8; 4]> {
    (state.entity_type() == crate::npc_identity::ET_NPC
        && usize::from(state.number()) >= MAX_CLIENTS)
        .then(|| state.npc_sound_sets())
}

/// Whose custom sounds an event speaks with: `CG_CustomSound`'s `ci`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Voice {
    /// A player slot's `clientinfo_t`.
    Player(usize),
    /// An NPC's `npcClient` banks; all zero (silent) when the speaker is past
    /// the player slots but no NPC in the snapshot (`npcClient` is `NULL`).
    Npc([u8; 4]),
}

impl LegacySoundAdapter {
    /// The voice of `speaker`: a player below `MAX_CLIENTS`, else that entity's
    /// NPC banks, from the event's own subject or the per-snapshot source table
    /// (an `EV_ENTITY_SOUND` naming an NPC, a forwarded event).
    pub(super) fn voice_of(&self, entity: &EventSubject, speaker: u16) -> Voice {
        if usize::from(speaker) < MAX_CLIENTS {
            return Voice::Player(usize::from(speaker));
        }
        let own = if speaker == entity.number {
            entity.npc_banks
        } else {
            None
        };
        Voice::Npc(
            own.or_else(|| self.npc.source_banks(speaker))
                .unwrap_or([0; 4]),
        )
    }

    /// The custom set a voice plays its standard reactions from.
    pub(super) fn voice_set(&self, voice: Voice) -> CustomSet {
        match voice {
            Voice::Player(client) => self.custom[client],
            Voice::Npc(refs) => self.npc.custom_set(refs),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Key {
    bank: usize,
    index: usize,
}

impl Key {
    fn parse(path: &str) -> Option<Self> {
        let name = path.strip_prefix('*')?.split('.').next()?;
        NAMES.iter().enumerate().find_map(|(bank, names)| {
            names
                .iter()
                .position(|candidate| candidate.eq_ignore_ascii_case(name))
                .map(|index| Self { bank, index })
        })
    }
}

type Banks = [[Option<u16>; 36]; 4];

pub(super) struct NpcSounds {
    banks: Box<[Option<Box<Banks>>]>,
    keys: [Option<Key>; MAX_SOUNDS],
    sources: Box<[Option<[u8; 4]>]>,
}

impl NpcSounds {
    pub(super) fn new(
        game: &GameState,
        vfs: &VirtualFileSystem,
        intern: &mut impl FnMut(&str) -> u16,
    ) -> Self {
        let mut this = Self {
            banks: (0..MAX_SOUNDS).map(|_| None).collect(),
            keys: [None; MAX_SOUNDS],
            sources: vec![None; MAX_ENTITIES].into_boxed_slice(),
        };
        for slot in 0..MAX_SOUNDS {
            this.refresh(slot, game, vfs, intern);
        }
        this
    }

    /// A fixed lookup for forwarded events, rebuilt once per snapshot. It avoids
    /// searching the entity list for every event and clears despawned identities.
    pub(super) fn observe(&mut self, snapshot: &Snapshot) {
        self.sources.fill(None);
        for entity in &snapshot.entities {
            if let Some(slot) = self.sources.get_mut(usize::from(entity.number())) {
                *slot = entity_banks(entity);
            }
        }
    }

    pub(super) fn source_banks(&self, source: u16) -> Option<[u8; 4]> {
        self.sources.get(usize::from(source)).copied().flatten()
    }

    pub(super) fn refresh(
        &mut self,
        slot: usize,
        game: &GameState,
        vfs: &VirtualFileSystem,
        intern: &mut impl FnMut(&str) -> u16,
    ) {
        self.banks[slot] = None;
        let path = game
            .config_string(CS_SOUNDS + slot)
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
            .unwrap_or_default();
        self.keys[slot] = Key::parse(path);
        let Some(directory) = path
            .strip_prefix("*$")
            .filter(|dir| !dir.is_empty() && slot != 0)
        else {
            return;
        };
        let mut banks = Box::new([[None; 36]; 4]);
        for (bank, names) in NAMES.iter().enumerate() {
            for (index, name) in names.iter().enumerate() {
                let path = format!("sound/chars/{directory}/misc/{name}.wav");
                if sound_exists(vfs, &path) {
                    banks[bank][index] = Some(intern(&path));
                }
            }
        }
        self.banks[slot] = Some(banks);
    }

    pub(super) fn get(&self, refs: [u8; 4], key: Key) -> Option<u16> {
        let slot = usize::from(refs[key.bank]);
        self.banks
            .get(slot)?
            .as_ref()?
            .get(key.bank)?
            .get(key.index)
            .copied()
            .flatten()
    }

    pub(super) fn configured(&self, refs: [u8; 4], slot: u8) -> Option<u16> {
        self.get(refs, self.keys[usize::from(slot)]?)
    }

    pub(super) fn custom_set(&self, refs: [u8; 4]) -> CustomSet {
        let at = |bank, index| self.get(refs, Key { bank, index });
        CustomSet {
            pushed: std::array::from_fn(|i| at(1, 9 + i)),
            choke: std::array::from_fn(|i| at(0, 9 + i)),
            push_fail: at(3, 21),
            jump: at(0, 3),
            pain: std::array::from_fn(|i| at(0, 4 + i)),
            death: std::array::from_fn(|i| at(0, i)),
            land: at(0, 13),
            falling: at(0, 8),
            gasp: at(0, 12),
            taunt: at(0, 14),
            anger: std::array::from_fn(|i| at(1, i)),
            victory: std::array::from_fn(|i| at(1, 3 + i)),
            taunt_numbered: std::array::from_fn(|i| at(3, 6 + i)),
            deflect: std::array::from_fn(|i| at(3, 15 + i)),
            gloat: std::array::from_fn(|i| at(3, 18 + i)),
        }
    }
}

/// NPC vocal event families, excluding the pushed/choke/pushfail branches
/// already handled individually by the ordinary resolver.
pub(super) fn event_key(event: u16) -> Option<Key> {
    let (bank, index) = match event {
        116..=124 => (1, event - 116),
        131..=132 => (1, event - 131 + 15),
        133..=168 => (2, event - 133),
        169..=189 => (3, event - 169),
        _ => return None,
    };
    Some(Key {
        bank,
        index: usize::from(index),
    })
}

/// Player duel voices for the vocal events players can also receive.
pub(super) fn player_event(set: CustomSet, event: u16) -> Option<u16> {
    match event {
        116..=118 => set.anger[usize::from(event - 116)],
        119..=121 => set.victory[usize::from(event - 119)],
        175..=177 => set.taunt_numbered[usize::from(event - 175)],
        184..=186 => set.deflect[usize::from(event - 184)],
        187..=189 => set.gloat[usize::from(event - 187)],
        _ => None,
    }
}

#[cfg(test)]
#[path = "npc_sound_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "npc_voice_tests.rs"]
mod voice_tests;
