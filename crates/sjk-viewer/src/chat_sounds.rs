//! Chat sounds per kind of message (`docs/client.md`, "Chat sounds"; Sol, after
//! Turin's and Nib's ideas, 23/10/2026): public chat, team chat, private messages
//! and the SJK chat each have a switch and a sound picked from [`SOUNDS`], with a
//! preview, under the master switch `cg_chatSounds` (EternalJK's; 0 silences every
//! chat sound, 2 gives team chat and private messages EJK's own sounds by default).
//!
//! A private message is a server's tell, which arrives as ordinary chat with the stock
//! private marker (`sjk_client` `chat::private_chat_command`, the
//! `LegacySoundEvent::PrivateChatBeep` beep), or an SJK encrypted whisper
//! (`e2e_whispers.rs`). Every server family SJK knows sends tells the stock way, so
//! one test serves them all; a family that marked them differently would get an
//! entry in a table beside that test, not a check here.
//!
//! The routing is [`Options::route`]: the audio owner reads the settings once a frame
//! ([`Options::read`]) and asks it for each chat beep and SJK chat or whisper cue.

use crate::console::ViewerConsole;
use sjk_shell::{CvarDefinition, CvarFlags, CvarRegistry};
use std::sync::atomic::{AtomicU32, Ordering};

/// `cg_chatSounds`: the master switch, 0 off, 1 on, 2 on with EJK's distinct team and
/// private sounds as the defaults (`cgame_options.rs` registers it).
pub(crate) const MASTER_CVAR: &str = "cg_chatSounds";

/// `cl_privateChatFlash`: a private message flashes the taskbar while the window is
/// not focused.
pub(crate) const FLASH_CVAR: &str = "cl_privateChatFlash";

/// The console command that plays a kind's chosen sound, as the Settings' preview
/// rows do: `chatsoundpreview <public|team|private|sjk>`.
pub(crate) const PREVIEW_COMMAND: &str = "chatsoundpreview";

/// One kind of chat message with a sound of its own.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Kind {
    /// A game server's public chat (`chat`, `lchat`).
    Public,
    /// A game server's team chat (`tchat`, `ltchat`).
    Team,
    /// A server's tell, or an SJK encrypted whisper.
    Private,
    /// A message on the SJK chat (`sjk_chat_frame.rs`).
    Sjk,
}

impl Kind {
    pub(crate) const ALL: [Self; 4] = [Self::Public, Self::Team, Self::Private, Self::Sjk];

    /// Its on/off switch; the first two are the game's own cvars, the last the SJK
    /// chat's sound switch from before the kinds had sounds of their own.
    pub(crate) const fn switch_cvar(self) -> &'static str {
        match self {
            Self::Public => "cg_chatBeep",
            Self::Team => "cg_teamChatBeep",
            Self::Private => "cg_privateChatBeep",
            Self::Sjk => crate::sjk_chat_frame::SOUND_CVAR,
        }
    }

    /// The cvar of its sound, an id of [`SOUNDS`].
    pub(crate) const fn sound_cvar(self) -> &'static str {
        match self {
            Self::Public => "cl_chatSoundPublic",
            Self::Team => "cl_chatSoundTeam",
            Self::Private => "cl_chatSoundPrivate",
            Self::Sjk => "cl_chatSoundSjk",
        }
    }

    /// Its Settings preview row's name: the console command that plays its sound.
    pub(crate) const fn preview_row(self) -> &'static str {
        match self {
            Self::Public => "chatsoundpreview public",
            Self::Team => "chatsoundpreview team",
            Self::Private => "chatsoundpreview private",
            Self::Sjk => "chatsoundpreview sjk",
        }
    }

    /// Its word in [`PREVIEW_COMMAND`].
    const fn word(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Team => "team",
            Self::Private => "private",
            Self::Sjk => "sjk",
        }
    }

    /// The kind of a chat beep the game's sound adapter made; `None` for any other.
    pub(crate) fn of_event(event: sjk_client::LegacySoundEvent) -> Option<Self> {
        use sjk_client::LegacySoundEvent as Event;
        match event {
            Event::ChatBeep => Some(Self::Public),
            Event::TeamChatBeep => Some(Self::Team),
            Event::PrivateChatBeep => Some(Self::Private),
            _ => None,
        }
    }

    /// The sound [`Source::Default`] plays for this kind at master `mode`: the game's
    /// chat sound, EJK's team and private sounds under `cg_chatSounds 2` (JoF EJK
    /// `cg_main.c:916-919`, `cg_servercmds.c:1814-1839`), the SJK chat's own variant.
    /// An encrypted whisper's default is the whisper variant ([`Self::whisper_default`]).
    pub(crate) const fn default_key(self, mode: i64) -> &'static str {
        match (self, mode) {
            (Self::Team, 2) => SWITCH_TEAM,
            (Self::Private, 2) => SWITCH_PRIVATE,
            (Self::Public | Self::Team | Self::Private, _) => TALK,
            (Self::Sjk, _) => crate::audio::sjk_chat_sound::HANDLE_KEY,
        }
    }

    /// The default sound of an SJK encrypted whisper, a private message.
    pub(crate) const fn whisper_default() -> &'static str {
        crate::audio::sjk_chat_sound::WHISPER_KEY
    }
}

const TALK: &str = crate::audio::sjk_chat_sound::TALK;
const SWITCH_TEAM: &str = "sound/movers/switches/button_11.mp3";
const SWITCH_PRIVATE: &str = "sound/movers/switches/button_15.mp3";

/// Where a choice's sound comes from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Source {
    /// The kind's own ([`Kind::default_key`]).
    Default,
    /// A sound file of the game's data, looked up as every sound is.
    File(&'static str),
    /// A sound SJK makes from the game's talk sound (`audio/sjk_chat_sound.rs`), by
    /// its name among the handles.
    Made(&'static str),
}

/// One sound a chat kind can play: its id (the cvar's value), the name the Settings
/// show and where it comes from.
pub(crate) struct Sound {
    pub(crate) id: &'static str,
    pub(crate) name: &'static str,
    pub(crate) source: Source,
}

/// The sounds a chat kind can play, the default first. The files are short sounds of
/// the retail game's shared `assets0.pk3`, so every install has them (a missing one
/// plays nothing); SJK bundles no sound of its own, and its two made sounds are
/// variants of the game's talk sound.
pub(crate) const SOUNDS: [Sound; 10] = [
    Sound {
        id: "default",
        name: "Default",
        source: Source::Default,
    },
    Sound {
        id: "talk",
        name: "JKA chat",
        source: Source::File(TALK),
    },
    Sound {
        id: "sjk",
        name: "SJK chat",
        source: Source::Made(crate::audio::sjk_chat_sound::HANDLE_KEY),
    },
    Sound {
        id: "whisper",
        name: "SJK whisper",
        source: Source::Made(crate::audio::sjk_chat_sound::WHISPER_KEY),
    },
    Sound {
        id: "team",
        name: "EJK team beep",
        source: Source::File(SWITCH_TEAM),
    },
    Sound {
        id: "private",
        name: "EJK private beep",
        source: Source::File(SWITCH_PRIVATE),
    },
    Sound {
        id: "blip",
        name: "Blip",
        source: Source::File("sound/interface/update.mp3"),
    },
    Sound {
        id: "select",
        name: "Select",
        source: Source::File("sound/interface/sub_select.mp3"),
    },
    Sound {
        id: "button",
        name: "Button",
        source: Source::File("sound/interface/button1.mp3"),
    },
    Sound {
        id: "tick",
        name: "Tick",
        source: Source::File("sound/interface/menuroam.mp3"),
    },
];

/// The ids of [`SOUNDS`], in order: the Settings' choices.
pub(crate) const SOUND_IDS: [&str; SOUNDS.len()] = {
    let mut ids = [""; SOUNDS.len()];
    let mut index = 0;
    while index < SOUNDS.len() {
        ids[index] = SOUNDS[index].id;
        index += 1;
    }
    ids
};

/// The sound files of [`SOUNDS`] and the kinds' defaults, for registering with the
/// other interface sounds.
pub(crate) fn files() -> impl Iterator<Item = &'static str> {
    SOUNDS
        .iter()
        .filter_map(|sound| match sound.source {
            Source::File(path) => Some(path),
            _ => None,
        })
        .chain([TALK, SWITCH_TEAM, SWITCH_PRIVATE])
}

/// The index in [`SOUNDS`] of id `value`; an unknown id is the default.
fn sound_index(value: &str) -> usize {
    SOUNDS
        .iter()
        .position(|sound| sound.id.eq_ignore_ascii_case(value.trim()))
        .unwrap_or(0)
}

/// What a sound row's value is called in the Settings (`settings/help.rs`).
pub(crate) fn choice_name(cvar: &str, value: &str) -> Option<&'static str> {
    Kind::ALL
        .iter()
        .any(|kind| kind.sound_cvar().eq_ignore_ascii_case(cvar))
        .then(|| {
            SOUNDS
                .iter()
                .find(|sound| sound.id.eq_ignore_ascii_case(value.trim()))
                .map(|sound| sound.name)
        })
        .flatten()
}

/// What a chat beep or cue plays.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Route {
    /// Nothing: the master switch or the kind's is off.
    Muted,
    /// What it would have played without these settings: the game's beep as the
    /// adapter chose it, the SJK chat's or the whisper's own sound.
    Default,
    /// The chosen sound, by its name among the handles.
    Sound(&'static str),
}

/// The chat sound settings, read once a frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Options {
    /// `cg_chatSounds`.
    mode: i64,
    /// Each [`Kind`]'s switch, in [`Kind::ALL`]'s order.
    on: [bool; 4],
    /// Each kind's sound, an index of [`SOUNDS`].
    sound: [usize; 4],
}

impl Default for Options {
    fn default() -> Self {
        Self {
            mode: 1,
            on: [true; 4],
            sound: [0; 4],
        }
    }
}

impl Options {
    /// The settings as `console` holds them; every switch on and every sound the
    /// default without one.
    pub(crate) fn read(console: Option<&ViewerConsole>) -> Self {
        let Some(console) = console else {
            return Self::default();
        };
        Self {
            mode: console.integer_cvar(MASTER_CVAR).unwrap_or(1),
            on: Kind::ALL.map(|kind| console.bool_cvar(kind.switch_cvar()).unwrap_or(true)),
            sound: Kind::ALL
                .map(|kind| console.text_value(kind.sound_cvar()).map_or(0, sound_index)),
        }
    }

    /// What a `kind` message plays.
    pub(crate) fn route(&self, kind: Kind) -> Route {
        if self.mode == 0 || !self.on[kind as usize] {
            return Route::Muted;
        }
        match SOUNDS[self.sound[kind as usize]].source {
            Source::Default => Route::Default,
            Source::File(key) | Source::Made(key) => Route::Sound(key),
        }
    }

    /// What `kind`'s preview plays: its chosen sound, the default resolved, whatever
    /// its switch says.
    pub(crate) fn preview(&self, kind: Kind) -> &'static str {
        match SOUNDS[self.sound[kind as usize]].source {
            Source::Default => kind.default_key(self.mode),
            Source::File(key) | Source::Made(key) => key,
        }
    }
}

/// Whether a `kind` message plays a sound now (the master switch and its own).
pub(crate) fn enabled(console: Option<&ViewerConsole>, kind: Kind) -> bool {
    Options::read(console).route(kind) != Route::Muted
}

/// Previews asked for since the audio last played them, one bit per [`Kind`].
static PREVIEWS: AtomicU32 = AtomicU32::new(0);

/// Play `kind`'s chosen sound on the next frame (a preview row, a choice made in the
/// Settings, [`PREVIEW_COMMAND`]).
pub(crate) fn preview(kind: Kind) {
    PREVIEWS.fetch_or(1 << kind as u32, Ordering::Relaxed);
}

/// The previews asked for since the last call.
pub(crate) fn take_previews() -> impl Iterator<Item = Kind> {
    let bits = PREVIEWS.swap(0, Ordering::Relaxed);
    Kind::ALL
        .into_iter()
        .filter(move |kind| bits & (1 << *kind as u32) != 0)
}

/// The kind whose preview row is `row`, if it is one.
pub(crate) fn preview_row_kind(row: &str) -> Option<Kind> {
    Kind::ALL
        .into_iter()
        .find(|kind| kind.preview_row().eq_ignore_ascii_case(row))
}

/// A choice was made in the Settings: a chat kind's sound plays as it is chosen.
pub(crate) fn choice_made(cvar: &str) {
    if let Some(kind) = Kind::ALL
        .into_iter()
        .find(|kind| kind.sound_cvar().eq_ignore_ascii_case(cvar))
    {
        preview(kind);
    }
}

/// [`PREVIEW_COMMAND`]: play a kind's chosen sound.
pub(crate) fn command(args: &[String]) -> Result<Vec<String>, String> {
    let usage = || format!("Usage: {PREVIEW_COMMAND} <public|team|private|sjk>");
    let [word] = args else {
        return Err(usage());
    };
    let kind = Kind::ALL
        .into_iter()
        .find(|kind| kind.word().eq_ignore_ascii_case(word))
        .ok_or_else(usage)?;
    preview(kind);
    Ok(Vec::new())
}

/// Register the new archived cvars: the private messages' switch, every kind's sound
/// and the taskbar flash (the other switches were already registered).
pub(crate) fn register(cvars: &mut CvarRegistry) -> Result<(), sjk_shell::CvarError> {
    let archive = CvarFlags::ARCHIVE;
    cvars.register(CvarDefinition::new(
        Kind::Private.switch_cvar(),
        true,
        archive,
        "Play a sound for a private message (a server's tell or an encrypted whisper); cg_chatSounds 0 silences it too",
    ))?;
    cvars.register(CvarDefinition::new(
        FLASH_CVAR,
        true,
        archive,
        "Flash the taskbar when a private message arrives while the game window is not focused",
    ))?;
    for (kind, what) in [
        (Kind::Public, "public chat"),
        (Kind::Team, "team chat"),
        (
            Kind::Private,
            "private messages (tells, encrypted whispers)",
        ),
        (Kind::Sjk, "SJK chat messages"),
    ] {
        cvars.register(CvarDefinition::new(
            kind.sound_cvar(),
            "default",
            archive,
            format!(
                "Sound of {what}: default, talk, sjk, whisper, team, private, blip, select, button or tick"
            ),
        ))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjk_client::LegacySoundEvent;

    fn console() -> (tempfile::TempDir, ViewerConsole) {
        let directory = tempfile::tempdir().unwrap();
        let console = ViewerConsole::new(directory.path().join("config.cfg")).unwrap();
        (directory, console)
    }

    #[test]
    fn beeps_are_told_apart_by_kind() {
        assert_eq!(
            Kind::of_event(LegacySoundEvent::ChatBeep),
            Some(Kind::Public)
        );
        assert_eq!(
            Kind::of_event(LegacySoundEvent::TeamChatBeep),
            Some(Kind::Team)
        );
        assert_eq!(
            Kind::of_event(LegacySoundEvent::PrivateChatBeep),
            Some(Kind::Private)
        );
        assert_eq!(Kind::of_event(LegacySoundEvent::Taunt), None);
    }

    #[test]
    fn by_default_every_kind_plays_its_own_sound() {
        let (_directory, console) = console();
        let options = Options::read(Some(&console));
        assert_eq!(options, Options::default());
        for kind in Kind::ALL {
            assert_eq!(options.route(kind), Route::Default, "{kind:?}");
        }
        assert_eq!(options.preview(Kind::Public), TALK);
        assert_eq!(options.preview(Kind::Team), TALK);
        assert_eq!(options.preview(Kind::Private), TALK);
        assert_eq!(
            options.preview(Kind::Sjk),
            crate::audio::sjk_chat_sound::HANDLE_KEY
        );
    }

    #[test]
    fn public_off_and_private_on_keeps_only_private_messages() {
        // Turin's case: no public chat sound, a sound for private messages.
        let (_directory, mut console) = console();
        assert!(console.set_cvar("cg_chatBeep", "0"));
        assert!(console.set_cvar("cl_chatSoundPrivate", "whisper"));
        let options = Options::read(Some(&console));
        assert_eq!(options.route(Kind::Public), Route::Muted);
        assert_eq!(options.route(Kind::Team), Route::Default);
        assert_eq!(
            options.route(Kind::Private),
            Route::Sound(crate::audio::sjk_chat_sound::WHISPER_KEY)
        );
        assert!(!enabled(Some(&console), Kind::Public));
        assert!(enabled(Some(&console), Kind::Private));
        // And the other way round: private messages muted, public chat heard.
        assert!(console.set_cvar("cg_chatBeep", "1"));
        assert!(console.set_cvar("cg_privateChatBeep", "0"));
        let options = Options::read(Some(&console));
        assert_eq!(options.route(Kind::Public), Route::Default);
        assert_eq!(options.route(Kind::Private), Route::Muted);
    }

    #[test]
    fn the_master_switch_silences_every_kind_and_2_is_ejk_s_distinct_sounds() {
        let (_directory, mut console) = console();
        assert!(console.set_cvar("cl_chatSoundTeam", "blip"));
        assert!(console.set_cvar(MASTER_CVAR, "0"));
        let options = Options::read(Some(&console));
        for kind in Kind::ALL {
            assert_eq!(options.route(kind), Route::Muted, "{kind:?}");
            assert!(!enabled(Some(&console), kind));
        }
        // A preview still plays: the player is choosing.
        assert_eq!(options.preview(Kind::Team), "sound/interface/update.mp3");
        assert!(console.set_cvar(MASTER_CVAR, "2"));
        assert!(console.set_cvar("cl_chatSoundTeam", "default"));
        let options = Options::read(Some(&console));
        assert_eq!(options.route(Kind::Team), Route::Default);
        assert_eq!(options.preview(Kind::Public), TALK);
        assert_eq!(options.preview(Kind::Team), SWITCH_TEAM);
        assert_eq!(options.preview(Kind::Private), SWITCH_PRIVATE);
    }

    #[test]
    fn every_kind_takes_every_sound_and_an_unknown_one_is_the_default() {
        let (_directory, mut console) = console();
        for kind in Kind::ALL {
            for sound in &SOUNDS {
                assert!(console.set_cvar(kind.sound_cvar(), sound.id));
                let route = Options::read(Some(&console)).route(kind);
                match sound.source {
                    Source::Default => assert_eq!(route, Route::Default),
                    Source::File(key) | Source::Made(key) => {
                        assert_eq!(route, Route::Sound(key));
                    }
                }
                assert_eq!(choice_name(kind.sound_cvar(), sound.id), Some(sound.name));
            }
            assert!(console.set_cvar(kind.sound_cvar(), "nonsense"));
            assert_eq!(Options::read(Some(&console)).route(kind), Route::Default);
        }
        assert_eq!(choice_name("cg_chatBeep", "talk"), None);
        // Handles are looked up by lowercase path.
        for path in files() {
            assert_eq!(path, path.to_ascii_lowercase());
        }
    }

    #[test]
    fn previews_come_from_rows_choices_and_the_command() {
        let _ = take_previews().count();
        assert_eq!(preview_row_kind("chatsoundpreview team"), Some(Kind::Team));
        assert_eq!(preview_row_kind("cg_chatBeep"), None);
        choice_made("cl_chatSoundSjk");
        choice_made("cg_chatBeep");
        assert!(command(&["PRIVATE".to_owned()]).is_ok());
        assert!(command(&[]).is_err());
        assert!(command(&["all".to_owned()]).is_err());
        // Other tests may preview at the same time (the mailbox is shared): ours came.
        let previews: Vec<Kind> = take_previews().collect();
        assert!(previews.contains(&Kind::Sjk) && previews.contains(&Kind::Private));
    }
}
