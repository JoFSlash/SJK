//! Choosing which emotes this screen shows and hears (mg.perso's idea, 16/11/2026;
//! Settings > Sound > Emotes, all off or empty by default, so emotes play as before):
//!
//! - [`OTHERS_CVAR`] hides every other player's emote: their animation, sound and
//!   console line never play here. The player's own emotes are untouched, and the
//!   others still see theirs.
//! - [`HIDE_CVAR`] lists emote ids hidden the same way, one by one (`emote_hide <id>`
//!   adds or removes one): the others' plays of them are not shown here, the player's
//!   own still are.
//! - [`MUTE_CVAR`] lists emote ids played silent here (`emote_mute <id>`): the dance
//!   shows, its music does not play, for everyone's plays of it, the player's own too.
//!   Unmuted while it plays, the music comes back where it is, as after an away emote.
//!
//! Only what this client draws and plays changes: nothing is sent to the hub. The ids
//! are the catalogue's (`sjkemote` alone lists them), separated by spaces or commas,
//! case aside. A player's emotes are hidden one by one with `sjkemote mute <player>`
//! (`emotes.rs`), apart from these.

use crate::emotes::{ActiveEmotes, EmoteDef};
use std::time::Instant;

/// Whether other players' emotes are hidden here.
pub(crate) const OTHERS_CVAR: &str = "cg_hideOthersEmotes";
/// [`OTHERS_CVAR`]'s default: shown, as before.
pub(crate) const DEFAULT_OTHERS: bool = false;
/// The emote ids whose plays by other players are hidden here.
pub(crate) const HIDE_CVAR: &str = "cg_hiddenEmotes";
/// The emote ids played without their sound here.
pub(crate) const MUTE_CVAR: &str = "s_mutedEmotes";
/// Console command names.
pub(crate) const HIDE_COMMAND: &str = "emote_hide";
pub(crate) const MUTE_COMMAND: &str = "emote_mute";
/// Help texts for completion and `cmdlist`.
pub(crate) const HIDE_HELP: &str = "Hide an emote when other players play it, on your screen only: emote_hide <id> (again shows it); alone, list them and the installed ids";
pub(crate) const MUTE_HELP: &str = "Play an emote without its sound on your screen: emote_mute <id> (again hears it); alone, list them and the installed ids";
/// The list both commands' first argument completes from on Tab: the installed ids.
pub(crate) const ARGUMENTS: &str = "emote_id";

/// The settings as last read, kept so a frame reads them without allocating.
#[derive(Debug, Default)]
pub(crate) struct Filters {
    pub(crate) hide_others: bool,
    pub(crate) hidden: String,
    pub(crate) muted: String,
}

impl Filters {
    /// Follow the console's settings; the strings keep their buffers.
    pub(crate) fn sync(&mut self, console: Option<&crate::console::ViewerConsole>) {
        let Some(console) = console else {
            *self = Self::default();
            return;
        };
        self.hide_others = console.bool_cvar(OTHERS_CVAR).unwrap_or(DEFAULT_OTHERS);
        follow(&mut self.hidden, console.text_value(HIDE_CVAR));
        follow(&mut self.muted, console.text_value(MUTE_CVAR));
    }

    /// Whether `slot`'s play of `emote` is hidden here (`own`: the player's slot).
    pub(crate) fn hides(&self, slot: u8, own: Option<u8>, emote: &str) -> bool {
        own != Some(slot) && (self.hide_others || listed(&self.hidden, emote))
    }

    /// Whether `emote` plays without its sound here.
    pub(crate) fn mutes(&self, emote: &str) -> bool {
        listed(&self.muted, emote)
    }

    /// End the emotes playing that are hidden now, and silence the muted ones (or give
    /// back the sound of those no longer muted); `catalogue` knows their music.
    pub(crate) fn apply(
        &self,
        active: &mut ActiveEmotes,
        catalogue: Option<&[EmoteDef]>,
        own: Option<u8>,
        now: Instant,
    ) {
        let (mut hide, mut mute, mut unmute) = (0u64, 0u64, 0u64);
        for (slot, emote) in active.playing() {
            let bit = 1u64.checked_shl(u32::from(slot)).unwrap_or(0);
            if self.hides(slot, own, &emote.emote) {
                hide |= bit;
            } else if self.mutes(&emote.emote) != emote.muted {
                if emote.muted {
                    unmute |= bit;
                } else {
                    mute |= bit;
                }
            }
        }
        for slot in slots(hide) {
            active.stop(slot);
        }
        for slot in slots(mute) {
            active.set_muted(slot, true, None, now);
        }
        for slot in slots(unmute) {
            let def = active
                .get(slot)
                .and_then(|playing| catalogue?.iter().find(|def| def.id == playing.emote));
            active.set_muted(slot, false, def, now);
        }
    }
}

/// Copy `value` into `kept` when it changed.
fn follow(kept: &mut String, value: Option<&str>) {
    let value = value.unwrap_or_default();
    if kept != value {
        kept.clear();
        kept.push_str(value);
    }
}

/// The slots of the bits set in `bits`.
fn slots(mut bits: u64) -> impl Iterator<Item = u8> {
    std::iter::from_fn(move || {
        (bits != 0).then(|| {
            let slot = bits.trailing_zeros() as u8;
            bits &= bits - 1;
            slot
        })
    })
}

/// The ids of `list`: words separated by spaces or commas.
fn ids(list: &str) -> impl Iterator<Item = &str> {
    list.split(|c: char| c.is_ascii_whitespace() || c == ',')
        .filter(|id| !id.is_empty())
}

/// Whether `list` names `emote`, case aside.
pub(crate) fn listed(list: &str, emote: &str) -> bool {
    ids(list).any(|id| id.eq_ignore_ascii_case(emote))
}

/// `list` with `emote` added, or taken out when it was there (every spelling of it),
/// and whether it is listed now.
pub(crate) fn toggled(list: &str, emote: &str) -> (String, bool) {
    let present = listed(list, emote);
    let mut kept: Vec<&str> = ids(list)
        .filter(|id| !id.eq_ignore_ascii_case(emote))
        .collect();
    let lower = emote.to_ascii_lowercase();
    if !present {
        kept.push(&lower);
    }
    (kept.join(" "), !present)
}

impl crate::GpuState {
    /// `emote_hide` and `emote_mute`: with an id, add it to the list or take it out;
    /// alone, say what the list holds.
    pub(crate) fn emote_filter_command(
        &mut self,
        command: &str,
        args: &[String],
    ) -> Result<Vec<String>, String> {
        let hide = command == HIDE_COMMAND;
        let (cvar, verb, undo) = if hide {
            (HIDE_CVAR, "hidden", "shown")
        } else {
            (MUTE_CVAR, "muted", "heard")
        };
        let console = self
            .console
            .as_mut()
            .ok_or_else(|| "The console is not ready".to_owned())?;
        let list = console.text_value(cvar).unwrap_or_default().to_owned();
        let Some(id) = args.first() else {
            let named: Vec<&str> = ids(&list).collect();
            let mut lines = vec![if named.is_empty() {
                format!("No emote is {verb} ({command} <id>)")
            } else {
                format!("Emotes {verb}: {}", named.join(", "))
            }];
            let installed = console.installed_emote_ids();
            if !installed.is_empty() {
                lines.push(format!("Emote ids: {}", installed.join(", ")));
            }
            return Ok(lines);
        };
        let id = id.to_ascii_lowercase();
        if !crate::emotes::valid_id(&id) {
            return Err("An emote id is 1 to 32 of a to z, 0 to 9 and _".to_owned());
        }
        let (list, now_listed) = toggled(&list, &id);
        if !console.set_cvar(cvar, &list) {
            return Err(format!("{cvar} could not be set"));
        }
        let whose = if hide { " when others play it" } else { "" };
        Ok(vec![if now_listed {
            format!("{id} is {verb} here{whose} ({command} {id} again: {undo})")
        } else {
            format!("{id} is {undo} again")
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn dance() -> EmoteDef {
        crate::emotes::parse(
            "dance",
            r#"{"name": "Dance", "torso": "BOTH_DANCE1", "length_ms": 9000,
                "loop": true, "sound": "sound/emotes/dance.mp3", "sound_ms": 4000}"#,
        )
        .unwrap()
    }

    #[test]
    fn lists_take_spaces_or_commas_and_any_case() {
        assert!(listed("wave, Dance  bow", "dance"));
        assert!(listed("wave,dance", "wave"));
        assert!(!listed("wave_two", "wave"));
        assert!(!listed("", "wave"));
    }

    #[test]
    fn a_command_adds_an_id_and_again_takes_it_out() {
        assert_eq!(toggled("", "Dance"), ("dance".to_owned(), true));
        assert_eq!(toggled("wave", "dance"), ("wave dance".to_owned(), true));
        assert_eq!(
            toggled("wave,DANCE bow", "dance"),
            ("wave bow".to_owned(), false)
        );
    }

    /// mg.perso's idea: with nothing set every emote plays as before; the switch hides
    /// the others' and never the player's own; a hidden id hides the others' plays of
    /// it only.
    #[test]
    fn hiding_leaves_the_own_emote_and_stock_shows_all() {
        let none = Filters::default();
        assert!(!none.hides(3, Some(1), "dance"));
        assert!(!none.mutes("dance"));
        let others = Filters {
            hide_others: true,
            ..Filters::default()
        };
        assert!(others.hides(3, Some(1), "dance"));
        assert!(!others.hides(1, Some(1), "dance"), "the own emote shows");
        let one = Filters {
            hidden: "dance".to_owned(),
            ..Filters::default()
        };
        assert!(one.hides(3, Some(1), "dance"));
        assert!(!one.hides(3, Some(1), "wave"));
        assert!(!one.hides(1, Some(1), "dance"), "the own emote shows");
    }

    #[test]
    fn a_muted_emote_dances_silently_and_its_music_comes_back_in_place() {
        let def = dance();
        let mut active = ActiveEmotes::default();
        let start = Instant::now();
        active.start(3, "dance", Some(&def), Some([0.0; 3]), start);
        let muted = Filters {
            muted: "dance".to_owned(),
            ..Filters::default()
        };
        muted.apply(
            &mut active,
            Some(std::slice::from_ref(&def)),
            Some(1),
            start,
        );
        assert!(
            !active
                .take_cues()
                .iter()
                .any(|cue| matches!(cue, crate::emotes::Cue::Start { .. })),
            "the music started this frame never plays"
        );
        assert!(
            active.get(3).is_some_and(|playing| playing.muted),
            "still dances"
        );
        // Its loop comes round: still silent.
        active.update(start + Duration::from_secs(5), |_| Some([0.0; 3]));
        assert!(active.take_cues().is_empty());
        // Unmuted: the music plays from where it is.
        let later = start + Duration::from_secs(6);
        Filters::default().apply(
            &mut active,
            Some(std::slice::from_ref(&def)),
            Some(1),
            later,
        );
        match active.take_cues().as_slice() {
            [
                crate::emotes::Cue::Start {
                    slot: 3, offset, ..
                },
            ] => {
                assert_eq!(*offset, Duration::from_secs(2));
            }
            cues => panic!("{cues:?}"),
        }
        // Muted while away, then back from away: still silent.
        muted.apply(
            &mut active,
            Some(std::slice::from_ref(&def)),
            Some(1),
            later,
        );
        let _ = active.take_cues();
        active.set_quiet(3, true, Some(&def), later);
        active.set_quiet(3, false, Some(&def), later);
        assert!(active.take_cues().is_empty(), "muted stays silent");
    }

    #[test]
    fn hiding_ends_the_others_emotes_playing_and_keeps_the_own() {
        let def = dance();
        let mut active = ActiveEmotes::default();
        let now = Instant::now();
        active.start(1, "dance", Some(&def), Some([0.0; 3]), now);
        active.start(3, "dance", Some(&def), Some([0.0; 3]), now);
        let _ = active.take_cues();
        let others = Filters {
            hide_others: true,
            ..Filters::default()
        };
        others.apply(&mut active, Some(&[def]), Some(1), now);
        assert!(active.get(1).is_some(), "own");
        assert!(active.get(3).is_none(), "another player's");
        assert_eq!(
            active.take_cues(),
            vec![crate::emotes::Cue::Stop { slot: 3 }]
        );
    }
}
