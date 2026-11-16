//! The rules for a profile's name emoji (`PROTOCOL.md`, "Profile fields"), shared word
//! for word with the hub (`src/name_emoji.rs` there): the one emoji picture a player
//! picks to stand at the right of their name, wherever SJK draws the name with its
//! marks (chat lines, cards, the scoreboard).
//!
//! A name emoji is exactly one Twemoji picture ([`crate::emoji`]): one pictograph, with
//! the marks that change it (a skin tone, U+FE0F, a subdivision flag's tags) or the
//! pictographs U+200D joins to it (a family), or two regional indicators (a flag).
//! Keycaps (a digit, `#` or `*` with U+20E3) are not taken, nor the pictures of
//! [`REFUSED`]. The empty string is no emoji.

/// Longest name emoji, in characters (a subdivision flag is 7, a family 7 or 11).
pub const EMOJI_MAX: usize = 16;

/// Pictures a name emoji may not be: rude gestures. Each is listed by its pictograph,
/// any skin tone included.
pub const REFUSED: &[char] = &['\u{1F595}'];

/// Why a name emoji was refused; the hub answers it with the `error` code
/// `emoji_invalid`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmojiError;

impl EmojiError {
    /// The hub's `error` code for it.
    pub const fn code(self) -> &'static str {
        "emoji_invalid"
    }

    /// What the player is told.
    pub const fn message(self) -> &'static str {
        "a name emoji is one emoji picture"
    }
}

impl std::fmt::Display for EmojiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}

/// Whether `c` is a regional indicator, two of which make a flag.
fn regional(c: char) -> bool {
    matches!(u32::from(c), 0x1F1E6..=0x1F1FF)
}

/// How many pictures `text` draws: each pictograph not joined to the one before by
/// U+200D, regional indicators counted in pairs.
fn pictures(text: &str) -> usize {
    let mut count = 0;
    let mut regionals: usize = 0;
    let mut joined = false;
    for c in text.chars() {
        if regional(c) {
            regionals += 1;
        } else if crate::emoji::pictograph(c) && !joined {
            count += 1;
        }
        joined = c == '\u{200D}';
    }
    count + regionals.div_ceil(2)
}

/// The name emoji ready to store (trimmed, empty for none), or why it is refused.
pub fn check(raw: &str) -> Result<String, EmojiError> {
    let emoji = raw.trim();
    if emoji.is_empty() {
        return Ok(String::new());
    }
    let only_emoji = emoji
        .chars()
        .all(|c| crate::emoji::pictograph(c) || crate::emoji::mark(c))
        && !emoji.contains('\u{20E3}');
    let regionals = emoji.chars().filter(|&c| regional(c)).count();
    if emoji.chars().count() > EMOJI_MAX
        || !only_emoji
        || !crate::emoji::marks_in_place(emoji)
        || !(regionals == 0 || regionals == 2 && emoji.chars().count() == 2)
        || pictures(emoji) != 1
        || emoji.chars().any(|c| REFUSED.contains(&c))
    {
        return Err(EmojiError);
    }
    Ok(emoji.to_owned())
}

/// A name emoji as a hub sent it, to show: itself when it passes [`check`], else
/// nothing, so a hub that kept an older or foreign value cannot put anything else
/// beside a name.
pub fn for_display(sent: &str) -> &str {
    match check(sent) {
        Ok(_) => sent.trim(),
        Err(_) => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_picture_with_its_marks_passes() {
        for emoji in [
            "🦊",
            " 🦊 ",
            "❤️",
            "👍🏽",
            "👨\u{200D}👩\u{200D}👧",
            "🇫🇷",
            "🏴\u{E0067}\u{E0062}\u{E0065}\u{E006E}\u{E0067}\u{E007F}",
        ] {
            assert_eq!(check(emoji), Ok(emoji.trim().to_owned()), "{emoji:?}");
        }
        assert_eq!(check("   "), Ok(String::new()), "nothing is no emoji");
    }

    #[test]
    fn anything_but_one_picture_is_refused() {
        for text in [
            "🦊🦊",
            "a",
            "🦊a",
            "1\u{FE0F}\u{20E3}",
            "\u{200D}🦊",
            "🏽",
            "🇫",
            "🇫🇷🇩🇪",
            "🖕",
            "🖕🏽",
            "^1🦊",
        ] {
            assert_eq!(check(text), Err(EmojiError), "{text:?}");
        }
        assert_eq!(for_display("🦊🦊"), "");
        assert_eq!(for_display("🦊"), "🦊");
    }
}
