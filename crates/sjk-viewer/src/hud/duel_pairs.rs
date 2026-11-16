//! Who duels whom, so the two duellists of one duel wear the same badge colour
//! ([`crate::ui_renderer::duel_badge::COLOURS`]) on their nameplates.
//!
//! A player entity's `bolt1` only says that it duels, not with whom; the client
//! learns its own opponent alone (`ps.duelIndex`). The pairs come from:
//!
//! - the server's broadcast when a duel starts: `Cmd_EngageDuel_f`
//!   (`codemp/game/g_cmds.c`) prints to every client
//!   `"<challenged> @@@PLDUELACCEPT <challenger>!"`, jaPRO and TaystJK (as JoF runs)
//!   `"<challenged>^7 @@@PLDUELACCEPT <challenger>^7! (Saber)"` with the duel type
//!   after; the names are matched against the players' `n` in `CS_PLAYERS`;
//! - the local player's own duel, from its player state.
//!
//! Nothing is guessed: a duellist whose opponent the client was never told (a duel
//! that began before it joined, or a mod that words the message otherwise) wears the
//! neutral badge ([`UNKNOWN`]) rather than a colour that might pair the wrong two.
//!
//! A pair keeps its colour while it lasts, and ends when the snapshot shows one of
//! the two no longer duelling (a player out of view keeps it). A broadcast pair
//! shown not duelling is dropped once a short grace has passed. A new gamestate (map
//! or server) or time running back (a demo seek) forgets every pair.

use sjk_protocol::{GameState, Snapshot};

/// `CS_PLAYERS`.
const CS_PLAYERS: usize = 1_131;
/// Client slots.
const CLIENTS: usize = 32;
/// `ET_PLAYER`.
const ET_PLAYER: u8 = 1;
/// The token the duel-start broadcast carries between the two names.
pub(crate) const ACCEPT: &str = " @@@PLDUELACCEPT ";
/// The badge colour of a duellist whose opponent is unknown: the last of
/// [`crate::ui_renderer::duel_badge::COLOURS`], grey, which no pair wears.
pub(crate) const UNKNOWN: u8 = crate::ui_renderer::duel_badge::COLOURS.len() as u8 - 1;
/// Server milliseconds a broadcast pair waits to be seen duelling.
const GRACE: i32 = 3_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Pair {
    a: u8,
    b: u8,
    /// Index into the duel colours.
    colour: u8,
    /// Both have been seen duelling.
    seen: bool,
    /// Server time of the first snapshot after the pair was made, for the grace.
    since: Option<i32>,
}

impl Pair {
    fn has(&self, slot: u8) -> bool {
        self.a == slot || self.b == slot
    }
}

/// The duels known to the client.
#[derive(Default)]
pub(crate) struct DuelPairs {
    pairs: Vec<Pair>,
    /// The gamestate the pairs belong to (its `checksumFeed`, new with every map and
    /// server) and the last snapshot's server time: a new gamestate, or time running
    /// back (a demo seek), forgets every pair.
    feed: Option<i32>,
    last_time: i32,
}

impl DuelPairs {
    /// Read one server `print` (raw, before localisation) for a duel start.
    pub(crate) fn observe_print(&mut self, text: &str, game: &GameState) {
        let Some((challenged, rest)) = text.split_once(ACCEPT) else {
            return;
        };
        let rest = rest.trim_end_matches(['\n', '\r']);
        // The challenger's name ends at a `!`: the last one in stock, but jaPRO and
        // TaystJK (as JoF runs) follow it with the duel type,
        // `"<challenged>^7 @@@PLDUELACCEPT <challenger>^7! (Saber) [Ranked]"`, and a
        // name may hold a `!` of its own, so each `!` is tried from the last.
        let challenger = rest
            .rmatch_indices('!')
            .find_map(|(at, _)| slot_named(game, &rest[..at]))
            .or_else(|| slot_named(game, rest));
        let challenged = challenged.trim_start_matches(['\n', '\r']);
        if let (Some(a), Some(b)) = (slot_named(game, challenged), challenger)
            && a != b
        {
            self.add(a, b);
        }
    }

    /// The pairs after this snapshot of `game`, and each client slot's duel colour, if
    /// it duels.
    pub(crate) fn update(
        &mut self,
        snapshot: &Snapshot,
        game: &GameState,
    ) -> [Option<u8>; CLIENTS] {
        let now = snapshot.server_time;
        if self.feed.is_some_and(|feed| feed != game.checksum_feed) || now < self.last_time {
            self.pairs.clear();
        }
        self.feed = Some(game.checksum_feed);
        self.last_time = now;
        let player = &snapshot.player;
        let local = player.client_num();
        // Who the snapshot shows duelling or not; a player out of view is unknown.
        let mut state = [None::<bool>; CLIENTS];
        for entity in &snapshot.entities {
            let slot = usize::from(entity.number());
            if entity.entity_type() == ET_PLAYER && slot < CLIENTS {
                state[slot] = Some(entity.bolt1());
            }
        }
        if usize::from(local) < CLIENTS {
            state[usize::from(local)] = Some(player.duel_in_progress());
        }
        let duelling = |slot: u8| state.get(usize::from(slot)).copied().flatten();
        for pair in &mut self.pairs {
            pair.since.get_or_insert(now);
            if duelling(pair.a) == Some(true) && duelling(pair.b) != Some(false)
                || duelling(pair.b) == Some(true) && duelling(pair.a) != Some(false)
            {
                pair.seen = true;
            }
        }
        // A pair ends when one of the two shows not duelling. One named but never yet
        // seen duelling waits out the grace first (the print may come before the
        // snapshot that sets the flag); out of view, it waits for them to come into view.
        self.pairs.retain(|pair| {
            let ended = duelling(pair.a) == Some(false) || duelling(pair.b) == Some(false);
            let waited = pair.since.is_some_and(|since| now - since > GRACE);
            !(ended && (pair.seen || waited))
        });
        // The local duel is known for certain.
        if player.duel_in_progress()
            && player.duel_index() != local
            && let (Ok(a), Ok(b)) = (u8::try_from(local), u8::try_from(player.duel_index()))
            && usize::from(b) < CLIENTS
        {
            self.add(a, b);
        }
        for pair in &mut self.pairs {
            pair.since.get_or_insert(now);
        }
        // Duellists in view whose opponent is unknown: the neutral badge.
        let mut colours: [Option<u8>; CLIENTS] =
            std::array::from_fn(|slot| (state[slot] == Some(true)).then_some(UNKNOWN));
        for pair in &self.pairs {
            for slot in [pair.a, pair.b] {
                if let Some(colour) = colours.get_mut(usize::from(slot)) {
                    *colour = Some(pair.colour);
                }
            }
        }
        colours
    }

    /// Pair `a` with `b`, ending any other pair either was in. The colour is kept
    /// when the same two are paired again, otherwise the first one no pair wears.
    fn add(&mut self, a: u8, b: u8) {
        if self.pairs.iter().any(|pair| pair.has(a) && pair.has(b)) {
            return;
        }
        self.pairs.retain(|pair| !pair.has(a) && !pair.has(b));
        let colour = (0..UNKNOWN)
            .find(|colour| !self.pairs.iter().any(|pair| pair.colour == *colour))
            .unwrap_or((self.pairs.len() as u8) % UNKNOWN);
        self.pairs.push(Pair {
            a,
            b,
            colour,
            seen: false,
            since: None,
        });
    }
}

/// The one client slot whose name (`n` in `CS_PLAYERS`) is `name`; none when no
/// player or more than one has it. An exact match first, then one without colour
/// codes, as a mod may colour the names in its message.
fn slot_named(game: &GameState, name: &str) -> Option<u8> {
    // Decoded as the print was ([`sjk_client::decode_legacy`]): a name with bytes past
    // ASCII reaches the print as text, not as the configstring's raw bytes.
    let names: Vec<(u8, String)> = (0..CLIENTS)
        .filter_map(|slot| {
            let config = game.config_string(CS_PLAYERS + slot)?;
            let name = info_value(config, b"n")?;
            Some((slot as u8, sjk_client::decode_legacy(name).into_owned()))
        })
        .collect();
    let only = |matches: &mut dyn Iterator<Item = u8>| {
        let first = matches.next()?;
        matches.next().is_none().then_some(first)
    };
    only(
        &mut names
            .iter()
            .filter(|(_, n)| n == name)
            .map(|(slot, _)| *slot),
    )
    .or_else(|| {
        let plain = uncoloured(name.as_bytes());
        only(
            &mut names
                .iter()
                .filter(|(_, n)| uncoloured(n.as_bytes()) == plain)
                .map(|(slot, _)| *slot),
        )
    })
}

/// `name` without its colour codes: `^` and the letter or digit after it.
fn uncoloured(name: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(name.len());
    let mut bytes = name.iter().peekable();
    while let Some(&byte) = bytes.next() {
        if byte == b'^'
            && bytes
                .peek()
                .is_some_and(|next| next.is_ascii_alphanumeric())
        {
            bytes.next();
        } else {
            out.push(byte);
        }
    }
    out
}

/// `Info_ValueForKey` over a raw configstring.
fn info_value<'a>(info: &'a [u8], key: &[u8]) -> Option<&'a [u8]> {
    let mut fields = info
        .split(|byte| *byte == b'\\')
        .skip_while(|field| field.is_empty());
    while let (Some(candidate), Some(value)) = (fields.next(), fields.next()) {
        if candidate.eq_ignore_ascii_case(key) {
            return Some(value);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjk_protocol::{EntityState, LEGACY_ENTITY_FIELDS, PlayerState};

    impl DuelPairs {
        /// [`DuelPairs::update`] against one fixed gamestate.
        fn update_test(&mut self, snapshot: &Snapshot) -> [Option<u8>; CLIENTS] {
            self.update(snapshot, &GameState::empty_local(0))
        }
    }

    fn game(names: &[&str]) -> GameState {
        let mut game = GameState::empty_local(0);
        for (slot, name) in names.iter().enumerate() {
            game.replace_config_string(CS_PLAYERS + slot, format!("n\\{name}\\t\\0").into_bytes())
                .unwrap();
        }
        game
    }

    fn player(number: u16, duelling: bool, x: f32) -> EntityState {
        let mut state = EntityState::zero(number, &LEGACY_ENTITY_FIELDS);
        state.set_raw_field(8, u32::from(ET_PLAYER));
        state.set_raw_field(91, u32::from(duelling));
        // `pos.trBase`, fields 2, 1 and 4 hold x, y and z.
        state.set_raw_field(2, x.to_bits());
        state
    }

    fn snapshot(time: i32, local: u16, entities: Vec<EntityState>) -> Snapshot {
        let mut player = PlayerState::zero();
        player.set_client_num(local);
        Snapshot {
            message_sequence: 0,
            reliable_acknowledge: 0,
            server_commands: Vec::new(),
            server_time: time,
            delta_from: None,
            flags: 0,
            area_mask: Vec::new(),
            player,
            vehicle_player: None,
            entities,
            consumed_bits: 0,
        }
    }

    #[test]
    fn the_broadcast_pairs_the_two_named_players_in_one_colour() {
        let game = game(&["Me", "^1Sol", "Kit", "Bounty", "Ogden"]);
        let mut pairs = DuelPairs::default();
        pairs.observe_print("^1Sol^7 @@@PLDUELACCEPT Kit!\n", &game);
        pairs.observe_print("Bounty @@@PLDUELACCEPT Ogden!\n", &game);
        let colours = pairs.update_test(&snapshot(
            1_000,
            0,
            vec![
                player(1, true, 0.0),
                player(2, true, 900.0),
                player(3, true, 10.0),
                player(4, true, 890.0),
            ],
        ));
        // Named pairs win over the nearer players.
        assert_eq!(colours[1], Some(0));
        assert_eq!(colours[2], Some(0));
        assert_eq!(colours[3], Some(1));
        assert_eq!(colours[4], Some(1));
        assert_eq!(colours[0], None);
    }

    #[test]
    fn a_duellist_with_an_unknown_opponent_wears_the_neutral_badge_until_named() {
        let game = game(&["Me", "A", "B", "C", "D"]);
        let mut pairs = DuelPairs::default();
        let view = |time| {
            snapshot(
                time,
                0,
                vec![
                    player(1, true, 0.0),
                    player(2, true, 20.0),
                    player(3, true, 500.0),
                    player(4, false, 520.0),
                ],
            )
        };
        // Standing close is no reason to pair anyone.
        let colours = pairs.update_test(&view(1_000));
        assert_eq!(colours[1], Some(UNKNOWN));
        assert_eq!(colours[2], Some(UNKNOWN));
        assert_eq!(colours[3], Some(UNKNOWN));
        assert_eq!(colours[4], None);
        pairs.observe_print("A @@@PLDUELACCEPT C!\n", &game);
        let colours = pairs.update_test(&view(1_100));
        assert_eq!(colours[1], Some(0));
        assert_eq!(colours[3], Some(0));
        assert_eq!(colours[2], Some(UNKNOWN));
    }

    #[test]
    fn the_local_duel_pairs_you_with_your_opponent() {
        let mut pairs = DuelPairs::default();
        let mut view = snapshot(1_000, 0, vec![player(5, true, 0.0), player(6, true, 10.0)]);
        // `duelInProgress` and `duelIndex` in the player state.
        view.player.set_raw_field(119, 1);
        view.player.set_raw_field(44, 5);
        let colours = pairs.update_test(&view);
        assert_eq!(colours[0], Some(0));
        assert_eq!(colours[5], Some(0));
        assert_eq!(colours[6], Some(UNKNOWN));
    }

    #[test]
    fn a_pair_ends_when_one_stops_duelling_and_a_stale_broadcast_lapses() {
        let game = game(&["Me", "A", "B", "C", "D"]);
        let mut pairs = DuelPairs::default();
        pairs.observe_print("A @@@PLDUELACCEPT B!\n", &game);
        let both = |time, b| snapshot(time, 0, vec![player(1, true, 0.0), player(2, b, 50.0)]);
        assert_eq!(pairs.update_test(&both(1_000, true))[1], Some(0));
        // B left the duel: the pair ends, and A, still flagged a moment, is unpaired.
        let after = pairs.update_test(&both(1_100, false));
        assert_eq!((after[1], after[2]), (Some(UNKNOWN), None));
        // Named while out of view: kept until they show up, and dropped once they
        // show up not duelling after the grace.
        pairs.observe_print("C @@@PLDUELACCEPT D!\n", &game);
        let unseen = pairs.update_test(&snapshot(2_000, 0, vec![]));
        assert_eq!((unseen[3], unseen[4]), (Some(0), Some(0)));
        let away = pairs.update_test(&snapshot(
            60_000,
            0,
            vec![player(3, true, 0.0), player(4, true, 9.0)],
        ));
        assert_eq!((away[3], away[4]), (Some(0), Some(0)));
        pairs.observe_print("A @@@PLDUELACCEPT B!\n", &game);
        pairs.update_test(&snapshot(61_000, 0, vec![]));
        let idle = pairs.update_test(&snapshot(65_000, 0, vec![player(1, false, 0.0)]));
        assert_eq!(idle[1], None);
        // A new gamestate (another map or server) forgets every pair.
        let colours = pairs.update(&snapshot(66_000, 0, vec![player(3, true, 0.0)]), &{
            let mut other = GameState::empty_local(0);
            other.checksum_feed = 7;
            other
        });
        assert_eq!(colours[3], Some(UNKNOWN));
    }

    #[test]
    fn an_unknown_or_shared_name_pairs_nobody() {
        let game = game(&["Me", "Twin", "Twin", "Kit"]);
        let mut pairs = DuelPairs::default();
        pairs.observe_print("Twin @@@PLDUELACCEPT Kit!\n", &game);
        pairs.observe_print("Nobody @@@PLDUELACCEPT Kit!\n", &game);
        pairs.observe_print("Kit has connected\n", &game);
        assert!(pairs.pairs.is_empty());
    }

    #[test]
    fn japros_message_with_the_duel_type_pairs_the_two() {
        let game = game(&["Me", "^1Kushina!", "JoF Ken", "A", "B"]);
        // `game` stores names as given; the accented one below goes in as Latin-1 bytes.
        let mut pairs = DuelPairs::default();
        pairs.observe_print(
            "^1Kushina!^7 @@@PLDUELACCEPT JoF Ken^7! (Saber) [Ranked]\n",
            &game,
        );
        pairs.observe_print("A^7 @@@PLDUELACCEPT B^7! (Force)\n", &game);
        let colours = pairs.update_test(&snapshot(
            1_000,
            0,
            vec![
                player(1, true, 0.0),
                player(2, true, 50.0),
                player(3, true, 100.0),
                player(4, true, 150.0),
            ],
        ));
        assert_eq!((colours[1], colours[2]), (Some(0), Some(0)));
        assert_eq!((colours[3], colours[4]), (Some(1), Some(1)));
    }

    #[test]
    fn a_name_past_ascii_matches_the_decoded_print() {
        let mut game = GameState::empty_local(0);
        for (slot, name) in [(1, &b"\xd7jof.jk\xd7"[..]), (2, b"Mara\xf1")] {
            let mut info = b"n\\".to_vec();
            info.extend_from_slice(name);
            info.extend_from_slice(b"\\t\\0");
            game.replace_config_string(CS_PLAYERS + slot, info).unwrap();
        }
        let mut pairs = DuelPairs::default();
        // The print as the client hands it on: decoded text.
        pairs.observe_print(
            "\u{d7}jof.jk\u{d7}^7 @@@PLDUELACCEPT Mara\u{f1}^7! (Saber)\n",
            &game,
        );
        let colours = pairs.update(
            &snapshot(1_000, 0, vec![player(1, true, 0.0), player(2, true, 9.0)]),
            &game,
        );
        assert_eq!((colours[1], colours[2]), (Some(0), Some(0)));
    }
}
