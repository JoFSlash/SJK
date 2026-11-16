//! The `block` and `unblock` console commands (`docs/hub-chat.md`, "Blocking a
//! player"): `block` alone lists the players blocked on this PC, `block <player>` blocks
//! one of the server being played (`target` for the one under the crosshair, a slot
//! number, or part of the name the game shows), and `unblock <player|all>` lifts a block,
//! by a name of the list (so it works away from the server too) or a player here. The
//! list is the one the sender cards' Block keeps ([`crate::player_mutes`]).
//!
//! `block <player> map` blocks them for this player until the map changes, and, for SJK
//! staff, `block <player> everyone` for every SJK player on the server who follows staff
//! blocks ([`crate::map_blocks`]); `unblock <player>` lifts this player's blocks of them
//! and a staff block for this player alone, and `unblock <player> everyone` (staff) a
//! staff block for everyone.

use crate::chat_mutes::Mute;
use crate::emotes::plain_name;

/// Console command names.
pub(crate) const COMMAND: &str = "block";
pub(crate) const UNBLOCK_COMMAND: &str = "unblock";
/// Help texts for completion and `cmdlist`.
pub(crate) const HELP: &str = "Block a player (no chat, Kyle, no sounds, no emotes, named BLOCKED): block <player|target> on this PC, block <player> map until the map changes; block alone lists them";
pub(crate) const UNBLOCK_HELP: &str = "Lift a block: unblock <player|all>";
/// The last words after a player that make a block last the map only, or (staff)
/// reach everyone on the server.
const MAP: &str = "map";
const EVERYONE: &str = "everyone";

/// How long and for whom a block is.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Scope {
    /// On this PC, until lifted.
    Pc,
    /// For this player, until the map changes.
    Map,
    /// For every SJK player on the server following staff blocks, until the map
    /// changes (staff only).
    Everyone,
}

/// The player words and the scope `word` asks for: a last word `map` or `everyone`
/// after a player.
pub(crate) fn scope(word: &str) -> (&str, Scope) {
    match word.rsplit_once(' ') {
        Some((player, last)) if last.eq_ignore_ascii_case(MAP) => (player.trim(), Scope::Map),
        Some((player, last)) if last.eq_ignore_ascii_case(EVERYONE) => {
            (player.trim(), Scope::Everyone)
        }
        _ => (word, Scope::Pc),
    }
}

/// The console's lines for the list.
pub(crate) fn listing(blocked: &[Mute]) -> Vec<String> {
    if blocked.is_empty() {
        return vec!["Nobody is blocked on this PC".to_owned()];
    }
    std::iter::once(format!("Blocked on this PC ({}):", blocked.len()))
        .chain(blocked.iter().map(|mute| {
            // How it matches, never the key id itself: key ids never show in the game.
            let how = if mute.key_id.is_some() {
                "by key"
            } else {
                "by name"
            };
            format!("  {}^7  ({how})", mute.name)
        }))
        .collect()
}

/// The entry of `blocked` that `word` names: a whole name (colours and case aside) or
/// the key first, else the only one whose name holds it.
pub(crate) fn find<'a>(blocked: &'a [Mute], word: &str) -> Result<Option<&'a Mute>, String> {
    let wanted = plain_name(word);
    if wanted.is_empty() {
        return Ok(None);
    }
    if let Some(exact) = blocked
        .iter()
        .find(|mute| plain_name(&mute.name) == wanted || mute.key_id.as_deref() == Some(word))
    {
        return Ok(Some(exact));
    }
    let matching: Vec<&Mute> = blocked
        .iter()
        .filter(|mute| plain_name(&mute.name).contains(&wanted))
        .collect();
    match matching[..] {
        [] => Ok(None),
        [one] => Ok(Some(one)),
        _ => Err(format!(
            "{word} matches {}: say more of the name",
            matching
                .iter()
                .map(|mute| crate::emotes::plain_name_shown(&mute.name))
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

impl crate::GpuState {
    /// The player `word` names on the server being played: their slot, the name the
    /// game shows (colours kept) and their SJK key when the hub's claim covers them.
    fn blockable(&self, word: &str) -> Result<(u8, String, String, Option<String>), String> {
        let Some(session) = self.live_session.as_ref() else {
            return Err("Join a server to block a player there".to_owned());
        };
        let own = u8::try_from(session.latest_snapshot().player.client_num()).ok();
        let (slot, plain) = self.emote_player(own, word)?;
        let shown = crate::player_identity::shown_name(session.game_state(), usize::from(slot))
            .unwrap_or_else(|| plain.clone());
        let key = crate::player_identity::hub_mark(slot, &shown).map(|mark| mark.key_id);
        Ok((slot, plain, shown, key))
    }

    /// Ask the hub to block (or stop blocking) the player in `slot`, shown as `shown`,
    /// for every SJK player on this server until the map changes; SJK staff only. What
    /// became of it shows on the Staff page's line.
    pub(crate) fn staff_map_block(
        &self,
        slot: u8,
        shown: &str,
        blocked: bool,
    ) -> Result<(), String> {
        if !crate::player_identity::is_staff() {
            return Err("Only SJK staff block players for everyone".to_owned());
        }
        let session = self
            .live_session
            .as_ref()
            .ok_or("Join a server to block a player there")?;
        let server = crate::player_identity::claimed_server()
            .ok_or("The SJK hub does not see you on this server yet")?;
        let map = session
            .game_state()
            .config_string(0)
            .map(sjk_client::LegacyClientInfo::new)
            .and_then(|info| info.bytes("mapname"))
            .map(|bytes| String::from_utf8_lossy(bytes).to_ascii_lowercase())
            .filter(|map| !map.is_empty())
            .ok_or("This server names no map")?;
        let request = sjk_identity::StaffRequest::MapBlock {
            server: server.to_string(),
            map,
            serverid: i64::from(session.server_id()),
            slot,
            name: shown.to_owned(),
            blocked,
        };
        if crate::player_identity::staff(request) {
            Ok(())
        } else {
            Err("The SJK hub is off (cl_identity 0)".to_owned())
        }
    }

    /// The console's lines for `block` alone: the PC's list, then the map's blocks.
    fn block_listing(&self) -> Vec<String> {
        let mut lines = listing(&crate::player_mutes::entries());
        let (own, staff) = (self.map_blocks.own_slots(), self.map_blocks.shared_slots());
        for (slots, what) in [(own, "by you"), (staff, "by SJK staff, for everyone")] {
            for slot in (0..crate::chat_mutes::SLOTS as u16).filter(|slot| slots & 1 << slot != 0) {
                lines.push(format!(
                    "  {}^7  (this map, {what})",
                    self.chat.player_label(slot)
                ));
            }
        }
        lines
    }

    /// `block [player] [map|everyone]`.
    pub(crate) fn block_command(&mut self, args: &[String]) -> Result<Vec<String>, String> {
        let word = args.get(1..).unwrap_or_default().join(" ");
        let word = word.trim();
        if word.is_empty() {
            return Ok(self.block_listing());
        }
        let (word, scope) = scope(word);
        let (slot, plain, shown, key) = self.blockable(word)?;
        match scope {
            Scope::Pc => {}
            Scope::Map => {
                self.map_blocks.block(usize::from(slot), &shown);
                return Ok(vec![format!(
                    "{plain} is blocked for you until the map changes (unblock {plain})"
                )]);
            }
            Scope::Everyone => {
                self.staff_map_block(slot, &shown, true)?;
                return Ok(vec![format!(
                    "Asked the SJK hub to block {plain} for everyone on this server until the map changes"
                )]);
            }
        }
        if crate::player_mutes::is_muted(key.as_deref(), &shown) {
            return Err(format!("{plain} is already blocked"));
        }
        crate::player_mutes::set_muted(key.as_deref(), &shown, true);
        Ok(vec![match key {
            Some(_) => format!("{plain} is blocked on this PC (unblock {plain})"),
            None => format!(
                "{plain} is blocked on this PC by name: the hub does not know them, so anyone wearing the name is (unblock {plain})"
            ),
        }])
    }

    /// `unblock <player|all> [everyone]`.
    pub(crate) fn unblock_command(&mut self, args: &[String]) -> Result<Vec<String>, String> {
        let word = args.get(1..).unwrap_or_default().join(" ");
        let word = word.trim();
        if word.is_empty() {
            return Err(UNBLOCK_HELP.to_owned());
        }
        if word.eq_ignore_ascii_case(crate::emotes::ALL) {
            let count = crate::player_mutes::clear() + self.map_blocks.clear();
            return Ok(vec![format!("Unblocked {count} player(s)")]);
        }
        let (word, scope) = scope(word);
        if scope == Scope::Everyone {
            let (slot, plain, shown, _) = self.blockable(word)?;
            self.staff_map_block(slot, &shown, false)?;
            return Ok(vec![format!(
                "Asked the SJK hub to stop blocking {plain} for everyone"
            )]);
        }
        // A player here blocked for the map, by this player or by staff (lifted for
        // this player alone), and on this PC too if they are.
        if self.live_session.is_some()
            && let Ok((slot, plain, shown, key)) = self.blockable(word)
            && self.map_blocks.unblock(usize::from(slot))
        {
            if crate::player_mutes::is_muted(key.as_deref(), &shown) {
                crate::player_mutes::set_muted(key.as_deref(), &shown, false);
            }
            return Ok(vec![format!("{plain} is unblocked")]);
        }
        let blocked = crate::player_mutes::entries();
        let entry = match find(&blocked, word)? {
            Some(entry) => entry.clone(),
            None => {
                // A player here, by the name they show now.
                if self.live_session.is_none() {
                    return Err(format!("Nobody blocked is called {word}"));
                }
                let (_, plain, shown, key) = self.blockable(word)?;
                if !crate::player_mutes::is_muted(key.as_deref(), &shown) {
                    return Err(format!("{plain} is not blocked"));
                }
                Mute {
                    key_id: key,
                    name: shown,
                }
            }
        };
        crate::player_mutes::set_muted(entry.key_id.as_deref(), &entry.name, false);
        Ok(vec![format!(
            "{} is unblocked",
            crate::emotes::plain_name_shown(&entry.name)
        )])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mute(key: Option<&str>, name: &str) -> Mute {
        Mute {
            key_id: key.map(str::to_owned),
            name: name.to_owned(),
        }
    }

    #[test]
    fn a_name_finds_its_entry_whole_names_first() {
        let list = [
            mute(Some("0123456789abcdef"), "^1Troll"),
            mute(None, "Trollface"),
            mute(None, "Sol"),
        ];
        assert_eq!(
            find(&list, "troll").unwrap(),
            Some(&list[0]),
            "whole name first"
        );
        assert_eq!(find(&list, "face").unwrap(), Some(&list[1]));
        assert_eq!(
            find(&list, "0123456789abcdef").unwrap(),
            Some(&list[0]),
            "by key"
        );
        assert_eq!(find(&list, "nobody").unwrap(), None);
        assert!(find(&list, "o").is_err(), "several match");
    }

    #[test]
    fn a_last_word_says_how_long_and_for_whom() {
        assert_eq!(scope("troll"), ("troll", Scope::Pc));
        assert_eq!(scope("troll map"), ("troll", Scope::Map));
        assert_eq!(scope("big troll EVERYONE"), ("big troll", Scope::Everyone));
        assert_eq!(scope("map"), ("map", Scope::Pc), "a player called map");
    }

    #[test]
    fn the_listing_says_how_each_is_matched() {
        assert_eq!(listing(&[]), ["Nobody is blocked on this PC"]);
        let lines = listing(&[mute(Some("0123456789abcdef"), "^1Troll"), mute(None, "Sol")]);
        assert_eq!(lines[0], "Blocked on this PC (2):");
        assert!(lines[1].contains("by key"));
        assert!(!lines[1].contains("0123456789abcdef"), "key ids never show");
        assert!(lines[2].contains("by name"));
    }
}
