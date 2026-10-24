//! The saber sounds a player hears for their sabers, chosen apart from the saber shader
//! they wear (`docs/unlockables.md`, "Saber sounds"; Nib's idea, 18/10/2026): follow
//! the shader (the default, the sounds came with the blade before), the stock sounds,
//! or the sound set of any saber shader the player owns.
//!
//! `cg_saberSounds` (archived) holds the choice: empty to follow, `stock`, or a blade
//! skin's unlock id. The own choice is gated like the blade ([`SoundChoice::own`]: a
//! shader not owned follows the blade), travels in the look (`saber_sounds`) through
//! the SJK hub, and other SJK players hear it ([`SoundChoice::sound_skin`]). A look
//! without the field (an older client or hub) follows the blade, as before.
//!
//! The Saber tab's Sounds row previews a choice: [`post_preview`] leaves it in a
//! mailbox the frame plays once ([`GpuState::play_sound_preview`]), as the interface
//! cues are.

#[cfg(not(test))]
use std::sync::Mutex;

/// Archived: the saber sounds chosen apart from the blade.
pub(crate) const CVAR: &str = "cg_saberSounds";

/// The value of [`CVAR`] (and of a look's `saber_sounds`) for the stock sounds.
pub(crate) const STOCK: &str = "stock";

/// Which saber sounds a player hears for their sabers.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum SoundChoice {
    /// The sounds of the shader worn (none for the stock blade): what every look meant
    /// before the choice, and what an older client's look still means.
    #[default]
    Follow,
    /// The stock saber sounds, whatever the blade.
    Stock,
    /// The sound set of this blade skin (a catalogue id), whatever the blade.
    Shader(&'static str),
}

impl SoundChoice {
    /// The choice `value` names (`cg_saberSounds`, a look's `saber_sounds`): empty or
    /// an id this client does not know (a newer client's shader) follows the blade.
    pub(crate) fn parse(value: &str) -> Self {
        let value = value.trim();
        if value.eq_ignore_ascii_case(STOCK) {
            return Self::Stock;
        }
        crate::unlockables::blade_skin(value).map_or(Self::Follow, |skin| Self::Shader(skin.id))
    }

    /// The local player's own choice from `setting`: a shader's sounds only while their
    /// profile lists it (`owned`), as for the blade; one not owned follows the blade.
    pub(crate) fn own(setting: &str, owned: impl Fn(&str) -> bool) -> Self {
        match Self::parse(setting) {
            Self::Shader(id) if !owned(id) => Self::Follow,
            choice => choice,
        }
    }

    /// The wire and setting form: empty, [`STOCK`] or the unlock id.
    pub(crate) fn wire(self) -> &'static str {
        match self {
            Self::Follow => "",
            Self::Stock => STOCK,
            Self::Shader(id) => id,
        }
    }

    /// The blade skin whose sounds play for a player wearing `blade` (`None` the stock
    /// blade) with this choice; `None` for the stock sounds. Whether that skin's pack
    /// is loaded is the caller's to check (a set not loaded plays the stock sounds).
    pub(crate) fn sound_skin(self, blade: Option<&'static str>) -> Option<&'static str> {
        match self {
            Self::Follow => blade,
            Self::Stock => None,
            Self::Shader(id) => Some(id),
        }
    }

    /// The choice's name, as the Saber tab shows it.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Follow => "Follow the shader",
            Self::Stock => "Stock sounds",
            Self::Shader(id) => crate::unlockables::blade_skin(id).map_or(id, |skin| skin.name),
        }
    }
}

/// The sound set a preview plays: a blade skin's, or `None` for the stock one.
#[cfg(not(test))]
static PREVIEW: Mutex<Option<Option<&'static str>>> = Mutex::new(None);

#[cfg(test)]
thread_local! {
    /// Each test thread's own mailbox, as tests run side by side.
    static PREVIEW: std::cell::Cell<Option<Option<&'static str>>> =
        const { std::cell::Cell::new(None) };
}

/// Ask the next frame to play the ignition and hum of `skin`'s sound set (`None` the
/// stock sounds) once, on the interface's source.
pub(crate) fn post_preview(skin: Option<&'static str>) {
    #[cfg(not(test))]
    if let Ok(mut preview) = PREVIEW.lock() {
        *preview = Some(skin);
    }
    #[cfg(test)]
    PREVIEW.with(|preview| preview.set(Some(skin)));
}

/// The preview posted since the last call, if any.
pub(crate) fn take_preview() -> Option<Option<&'static str>> {
    #[cfg(not(test))]
    return PREVIEW.lock().ok().and_then(|mut preview| preview.take());
    #[cfg(test)]
    PREVIEW.with(std::cell::Cell::take)
}

/// The stock ignition and hum a preview of the stock sounds plays.
pub(crate) const STOCK_PREVIEW: [&str; 2] = [
    "sound/weapons/saber/saberon.wav",
    "sound/weapons/saber/saberhum1.wav",
];

impl crate::GpuState {
    /// Play a preview the Saber tab posted: the ignition and hum of blade skin `skin`'s
    /// sound set, read from its pack below the game data, or the stock ones (also for
    /// a skin whose pack is not loaded).
    pub(crate) fn play_sound_preview(
        &self,
        game_audio: &mut Option<crate::GameAudio>,
        skin: Option<&'static str>,
    ) {
        let (Some(audio), Some(vfs)) = (game_audio.as_mut(), self.vfs.as_ref()) else {
            return;
        };
        match skin.and_then(|id| self.blade_skins.get(id)) {
            Some(skin) => {
                let Ok(vfs) = vfs.with_lower(self.blade_skins.packs()) else {
                    return;
                };
                let sounds = &skin.def.sounds;
                audio.play_preview(&vfs, &[&sounds.on, &sounds.hum]);
            }
            None => audio.play_preview(vfs, &STOCK_PREVIEW),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Which sound set plays for a wearer: the blade's while following, none for stock,
    /// the chosen shader's whatever the blade; an older client's look (no field) and an
    /// unknown id follow the blade.
    #[test]
    fn the_sound_set_follows_the_blade_unless_chosen() {
        let sun = Some("saber_sun");
        assert_eq!(SoundChoice::parse("").sound_skin(sun), sun);
        assert_eq!(SoundChoice::parse("").sound_skin(None), None);
        assert_eq!(SoundChoice::parse("stock").sound_skin(sun), None);
        assert_eq!(SoundChoice::parse("STOCK").sound_skin(sun), None);
        assert_eq!(
            SoundChoice::parse("saber_void").sound_skin(sun),
            Some("saber_void")
        );
        assert_eq!(
            SoundChoice::parse("Saber_Void").sound_skin(None),
            Some("saber_void"),
            "a shader's sounds on the stock blade, by the catalogue's id"
        );
        assert_eq!(
            SoundChoice::parse("saber_from_a_newer_client").sound_skin(sun),
            sun
        );
        for choice in [
            SoundChoice::Follow,
            SoundChoice::Stock,
            SoundChoice::Shader("saber_void"),
        ] {
            assert_eq!(SoundChoice::parse(choice.wire()), choice, "{choice:?}");
        }
    }

    /// The own choice of a shader's sounds holds only while the profile lists it; stock
    /// and following need nothing.
    #[test]
    fn the_own_choice_of_a_shaders_sounds_needs_it_owned() {
        let owns_void = |id: &str| id == "saber_void";
        assert_eq!(
            SoundChoice::own("saber_void", owns_void),
            SoundChoice::Shader("saber_void")
        );
        assert_eq!(
            SoundChoice::own("saber_sun", owns_void),
            SoundChoice::Follow
        );
        assert_eq!(SoundChoice::own("stock", |_| false), SoundChoice::Stock);
        assert_eq!(SoundChoice::own("", |_| true), SoundChoice::Follow);
    }

    #[test]
    fn a_preview_is_taken_once() {
        post_preview(Some("saber_sun"));
        post_preview(None);
        assert_eq!(take_preview(), Some(None), "the latest wins");
        assert_eq!(take_preview(), None);
    }
}
