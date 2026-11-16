//! Typing and pasting into an SJK chat field: the SJK chat page's
//! (`sjk_chat_panel.rs`) and the docks' (`menu/sjk/chat_dock.rs`, on the main
//! page and the in-game menu). The caret moves as in the console
//! ([`crate::console::line_edit`]): Left and Right by a character, with Ctrl by a word,
//! Home and End to the ends; typing and pasting go in at it, Backspace and Delete take
//! the character before or after it (Rollembones' report, 22/10/2026: the arrows did
//! not move it). The GIF picker's search ([`key_into`]) keeps its caret at the end.
//!
//! Ctrl+V (and Shift+Insert, as in the console and the game's chat) pastes the
//! clipboard's text as the hub would take it ([`sjk_identity::chat`]): line breaks and
//! tabs become spaces and runs of spaces one, characters outside the chat's alphabet are
//! dropped, a `^` is kept only before a digit (a colour code, counted as two characters
//! as the field counts them), and the paste stops at [`MAX`] characters, never splitting
//! a colour code. A GIPHY link is pasted as its shortest address
//! ([`crate::chat_gifs::link::shortened`]), so a share link with its query fits.

use crate::console::line_edit::{LineEdit, Motion};
use sjk_identity::chat::TEXT_MAX;
use winit::keyboard::KeyCode;

/// The longest draft, as the hub takes a message.
pub(crate) const MAX: usize = TEXT_MAX;

/// Ctrl+V on Windows types this character.
const CONTROL_V: &str = "\u{16}";

/// Whether a pressed `key` that typed `text` pastes: Ctrl+V, or Shift+Insert. With Ctrl
/// held, Windows hands the key's letter without Ctrl (`v`, the layout's own letter, as
/// the in-game chat reads it); the Ctrl character itself, or Ctrl with V where the layout
/// types nothing, count too.
pub(crate) fn is_paste(key: KeyCode, text: Option<&str>, shift: bool, control: bool) -> bool {
    text == Some(CONTROL_V)
        || (control && text.is_some_and(|text| text.eq_ignore_ascii_case("v")))
        || (control && key == KeyCode::KeyV && text.is_none())
        || (shift && !control && key == KeyCode::Insert)
}

/// A pressed `key` that typed `text` into `draft`: pasted from the clipboard
/// ([`is_paste`]) or typed, control characters left out, up to [`MAX`] characters.
pub(crate) fn key_into(
    draft: &mut String,
    key: KeyCode,
    text: Option<&str>,
    shift: bool,
    control: bool,
) {
    if is_paste(key, text, shift, control) {
        if let Some(clipboard) = crate::console::clipboard::paste() {
            paste_into(draft, &clipboard);
        }
    } else if let Some(text) = text {
        type_into(draft, text);
    }
}

/// A key that moves `edit`'s caret in `draft` or deletes at it: Left and Right (by a word
/// with Ctrl), Home, End, Backspace and Delete (a word with Ctrl). Whether it was one.
pub(crate) fn caret_key(
    draft: &mut String,
    edit: &mut LineEdit,
    key: KeyCode,
    control: bool,
) -> bool {
    let (left, right) = if control {
        (Motion::WordLeft, Motion::WordRight)
    } else {
        (Motion::Left, Motion::Right)
    };
    match key {
        KeyCode::ArrowLeft => edit.motion(draft, left, false),
        KeyCode::ArrowRight => edit.motion(draft, right, false),
        KeyCode::Home => edit.motion(draft, Motion::Home, false),
        KeyCode::End => edit.motion(draft, Motion::End, false),
        KeyCode::Backspace => edit.delete(draft, left),
        KeyCode::Delete => edit.delete(draft, right),
        _ => return false,
    }
    true
}

/// [`key_into`] at `edit`'s caret: a paste or typed text goes in there, up to [`MAX`]
/// characters for the whole draft, and the caret follows it.
pub(crate) fn key_at(
    draft: &mut String,
    edit: &mut LineEdit,
    key: KeyCode,
    text: Option<&str>,
    shift: bool,
    control: bool,
) {
    if is_paste(key, text, shift, control) {
        if let Some(clipboard) = crate::console::clipboard::paste() {
            paste_at(draft, edit, &clipboard);
        }
    } else if let Some(text) = text {
        edit.insert_counted(draft, text, MAX, |_| 1);
    }
}

/// Pasted `text` put in `draft` at `edit`'s caret, as [`paste_into`] takes it, the caret
/// after it.
pub(crate) fn paste_at(draft: &mut String, edit: &mut LineEdit, text: &str) {
    let tail = draft.split_off(edit.cursor(draft));
    paste_limited(draft, text, MAX.saturating_sub(tail.chars().count()));
    edit.place(draft, draft.len(), false);
    draft.push_str(&tail);
}

/// `draft` as a field shows it, `mark` at the byte `caret`, cut to about `room`
/// characters around the caret: the end of a long draft, or the part from the caret on
/// when the caret is further back, with "..." where text is left out.
pub(crate) fn with_caret(draft: &str, caret: usize, mark: &str, room: usize) -> String {
    let mut caret = caret.min(draft.len());
    while !draft.is_char_boundary(caret) {
        caret -= 1;
    }
    let caret = draft[..caret].chars().count();
    let count = draft.chars().count();
    let (start, end) = if count <= room {
        (0, count)
    } else if caret + room.saturating_sub(3) >= count {
        (count - room.saturating_sub(3), count)
    } else {
        (caret, caret + room.saturating_sub(6))
    };
    let mut shown = String::with_capacity(draft.len() + mark.len() + 6);
    if start > 0 {
        shown.push_str("...");
    }
    for (index, c) in draft.chars().enumerate().take(end).skip(start) {
        if index == caret {
            shown.push_str(mark);
        }
        shown.push(c);
    }
    if caret >= end {
        shown.push_str(mark);
    }
    if end < count {
        shown.push_str("...");
    }
    shown
}

/// Typed `text` added to `draft`, control characters left out, up to [`MAX`] characters.
pub(crate) fn type_into(draft: &mut String, text: &str) {
    let room = MAX.saturating_sub(draft.chars().count());
    draft.extend(text.chars().filter(|c| !c.is_control()).take(room));
}

/// Pasted `text` added to `draft` as the chat takes it (see the module's rules).
pub(crate) fn paste_into(draft: &mut String, text: &str) {
    paste_limited(draft, text, MAX);
}

/// [`paste_into`], up to `max` characters for `draft`.
fn paste_limited(draft: &mut String, text: &str, max: usize) {
    // Spaces first, so a link split from its neighbours by a line break is found.
    let spaced: String = text
        .chars()
        .map(|c| if c.is_whitespace() { ' ' } else { c })
        .collect();
    let text = crate::chat_gifs::link::shortened(&spaced);
    let mut count = draft.chars().count();
    // A space after a space (or at the start) would be tidied away by the hub anyway.
    let mut after_space = draft.is_empty() || draft.ends_with(' ');
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        let c = if c.is_whitespace() { ' ' } else { c };
        let (first, second) = match c {
            ' ' if after_space => continue,
            '^' => match chars.peek() {
                Some(&digit) if digit.is_ascii_digit() => {
                    let _ = chars.next();
                    ('^', Some(digit))
                }
                _ => continue,
            },
            c if sjk_identity::bio::allowed(c) => (c, None),
            _ => continue,
        };
        let width = 1 + usize::from(second.is_some());
        if count + width > max {
            break;
        }
        draft.push(first);
        draft.extend(second);
        count += width;
        after_space = first == ' ';
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pasted(draft: &str, text: &str) -> String {
        let mut draft = draft.to_owned();
        paste_into(&mut draft, text);
        draft
    }

    #[test]
    fn ctrl_v_and_shift_insert_paste() {
        assert!(is_paste(KeyCode::KeyV, Some("\u{16}"), false, true));
        assert!(is_paste(KeyCode::KeyV, None, false, true));
        // What Windows sends for Ctrl+V: the letter, Ctrl left out of the text.
        assert!(is_paste(KeyCode::KeyV, Some("v"), false, true));
        assert!(is_paste(KeyCode::KeyV, Some("V"), true, true));
        // A layout whose V sits on another key (Dvorak's `.` key) pastes there.
        assert!(is_paste(KeyCode::Period, Some("v"), false, true));
        assert!(is_paste(KeyCode::Insert, None, true, false));
        assert!(!is_paste(KeyCode::KeyV, Some("v"), false, false));
        // Another layout's letter on that key is typed, not a paste.
        assert!(!is_paste(KeyCode::KeyV, Some("k"), false, true));
        assert!(!is_paste(KeyCode::Insert, None, false, false));
        assert!(!is_paste(KeyCode::Insert, None, true, true));
    }

    #[test]
    fn a_giphy_share_link_is_pasted_short_enough_to_send() {
        let share = concat!(
            "https://media4.giphy.com/media/v1.Y2lkPTc5MGI3NjExbWx2dTRqZ2NrOXJ5c3B3ZDhvN2x6",
            "ZWZ3NHN4bjd2bWJ2Z3V0aGh6bSZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/3o7TKSjRrfIPjeiVyM/",
            "giphy.gif?cid=790b7611&ep=v1_internal_gif_by_id&rid=giphy.gif&ct=g"
        );
        assert_eq!(
            pasted("look ", &format!("{share}!")),
            "look https://giphy.com/gifs/3o7TKSjRrfIPjeiVyM!"
        );
        assert_eq!(
            pasted(
                "",
                "https://giphy.com/gifs/funny-cat-3o7TKSjRrfIPjeiVyM?utm_source=x"
            ),
            "https://giphy.com/gifs/3o7TKSjRrfIPjeiVyM"
        );
        // Other links are pasted as they are.
        assert_eq!(pasted("", "https://example.com/a"), "https://example.com/a");
    }

    #[test]
    fn line_breaks_and_tabs_become_one_space() {
        assert_eq!(
            pasted("", "one\r\ntwo\tthree\n\nfour"),
            "one two three four"
        );
        assert_eq!(pasted("hi ", "  there"), "hi there");
        assert_eq!(pasted("hi", " there"), "hi there");
        assert_eq!(pasted("", "\n lead"), "lead");
        assert_eq!(pasted("", "a\u{a0}b"), "a b");
    }

    #[test]
    fn what_the_hub_refuses_is_dropped() {
        assert_eq!(pasted("", "smile \u{221E}!"), "smile !");
        assert_eq!(pasted("", "a\u{200b}b\u{202e}c"), "abc");
        assert_eq!(pasted("", r"{tag} \o/ `x`"), "tag o/ x");
        assert_eq!(
            sjk_identity::chat::check(&pasted("", "Ça va? Très bien.")),
            Ok("Ça va? Très bien.".to_owned())
        );
    }

    #[test]
    fn colour_codes_are_kept_and_stray_carets_dropped() {
        assert_eq!(pasted("", "^1red ^xno ^"), "^1red xno ");
        assert_eq!(pasted("^", "2"), "^2");
    }

    #[test]
    fn the_paste_stops_at_the_limit() {
        let long = "word ".repeat(MAX / 4);
        let draft = pasted("", &long);
        assert_eq!(draft.chars().count(), MAX);
        assert_eq!(pasted(&draft, "more"), draft);
        let near = "a".repeat(MAX - 3);
        assert_eq!(pasted(&near, "bcdef"), format!("{near}bcd"));
        // A colour code is never cut in half.
        let one_left = "a".repeat(MAX - 1);
        assert_eq!(pasted(&one_left, "^1x"), one_left);
        let two_left = "a".repeat(MAX - 2);
        assert_eq!(pasted(&two_left, "^1x"), format!("{two_left}^1"));
        // Characters count, not bytes.
        assert_eq!(pasted("", &"é".repeat(MAX + 50)).chars().count(), MAX);
    }

    #[test]
    fn the_caret_moves_and_edits_inside_the_draft() {
        let mut draft = String::new();
        let mut edit = LineEdit::default();
        key_at(
            &mut draft,
            &mut edit,
            KeyCode::KeyH,
            Some("helo"),
            false,
            false,
        );
        assert!(caret_key(&mut draft, &mut edit, KeyCode::ArrowLeft, false));
        key_at(
            &mut draft,
            &mut edit,
            KeyCode::KeyL,
            Some("l"),
            false,
            false,
        );
        assert_eq!(draft, "hello");
        assert_eq!(edit.cursor(&draft), 4);
        assert!(caret_key(&mut draft, &mut edit, KeyCode::Home, false));
        key_at(
            &mut draft,
            &mut edit,
            KeyCode::KeyO,
            Some("oh "),
            false,
            false,
        );
        assert_eq!(draft, "oh hello");
        assert!(caret_key(&mut draft, &mut edit, KeyCode::Backspace, false));
        assert!(caret_key(&mut draft, &mut edit, KeyCode::Delete, false));
        assert_eq!(draft, "ohello");
        assert!(caret_key(&mut draft, &mut edit, KeyCode::End, false));
        assert_eq!(edit.cursor(&draft), draft.len());
        // Ctrl jumps a word.
        let mut draft = "one two three".to_owned();
        let mut edit = LineEdit::default();
        edit.to_end(&draft);
        assert!(caret_key(&mut draft, &mut edit, KeyCode::ArrowLeft, true));
        assert_eq!(edit.cursor(&draft), 8);
        assert!(caret_key(&mut draft, &mut edit, KeyCode::ArrowLeft, true));
        assert_eq!(edit.cursor(&draft), 4);
        assert!(caret_key(&mut draft, &mut edit, KeyCode::ArrowRight, true));
        assert_eq!(edit.cursor(&draft), 8);
        assert!(!caret_key(&mut draft, &mut edit, KeyCode::KeyA, false));
        // A caret left past a cleared draft starts again at its start.
        draft.clear();
        key_at(
            &mut draft,
            &mut edit,
            KeyCode::KeyA,
            Some("a"),
            false,
            false,
        );
        assert_eq!(draft, "a");
    }

    #[test]
    fn typing_and_pasting_at_the_caret_keep_the_limit() {
        let mut draft = "a".repeat(MAX - 2);
        let mut edit = LineEdit::default();
        key_at(
            &mut draft,
            &mut edit,
            KeyCode::KeyB,
            Some("bcd"),
            false,
            false,
        );
        assert_eq!(draft, format!("bc{}", "a".repeat(MAX - 2)));
        let mut draft = format!("x{}", "a".repeat(MAX - 4));
        let mut edit = LineEdit::default();
        edit.place(&draft, 1, false);
        paste_at(&mut draft, &mut edit, " ^1red");
        assert_eq!(draft, format!("x ^1{}", "a".repeat(MAX - 4)));
        assert_eq!(edit.cursor(&draft), 4);
    }

    #[test]
    fn the_field_shows_its_caret_where_it_is() {
        assert_eq!(with_caret("hello", 2, "|", 96), "he|llo");
        assert_eq!(with_caret("hello", 5, "|", 96), "hello|");
        assert_eq!(with_caret("f\u{ea}te", 3, "|", 96), "f\u{ea}|te");
        let long = "abcdefghij";
        // The end shows while the caret is in it, else the part from the caret on.
        assert_eq!(with_caret(long, 10, "|", 8), "...fghij|");
        assert_eq!(with_caret(long, 6, "|", 8), "...f|ghij");
        assert_eq!(with_caret(long, 2, "|", 8), "...|cd...");
        assert_eq!(with_caret(long, 0, "|", 8), "|ab...");
    }

    #[test]
    fn typing_keeps_its_rules() {
        let mut draft = String::new();
        type_into(&mut draft, "a\u{16}b\r");
        assert_eq!(draft, "ab");
        let mut full = "a".repeat(MAX);
        type_into(&mut full, "b");
        assert_eq!(full.chars().count(), MAX);
    }
}
