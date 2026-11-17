//! The console's find bar (headshotsmcgee117's idea, in Sol's ideas recap of
//! 16/11/2026: "console Ctrl+F").
//!
//! Ctrl+F in the open console puts a find field in place of the input line. What is
//! typed there is looked for in the scrollback, ignoring case and colour codes: every
//! match is marked, the current one brighter, and the field says which match it is of
//! how many, counted from the newest (`1/5`). Enter, F3 or Up goes to the next older
//! match and Shift+Enter, Shift+F3 or Down to the next newer one, wrapping round, and
//! the console scrolls to it. Escape closes the field and gives the input line back as
//! it was; closing the console closes it too, and the next Ctrl+F brings the last
//! search back, selected. With `con_chatOnly` on only the chat lines on show are
//! searched. Page Up and Down and Ctrl+Home and End still scroll, and the field takes
//! the caret, deletion and clipboard keys (Ctrl+A, C, V, X) as the input line does.
//!
//! Matches are found again only when the search, the scrollback or `con_chatOnly`
//! changed, not every frame.
use super::super::clipboard;
use super::super::line_edit::{LineEdit, Motion};
use super::{
    BLINK_SHIFT, Grid, Painter, Prompt, Span, ViewerConsole, byte_of_cell, cell_count, colour_code,
    column_of, cursor_rect, quake_color, sjk,
};
use crate::console_backdrop::{ConsoleFrame, SolidQuad};
use crate::input::dead_key::{DeadKey, TypingField, keep_caret};
use std::fmt::Write as _;
use std::ops::Range;
use winit::event::KeyEvent;
use winit::keyboard::{Key, KeyCode};

/// Longest search, in bytes.
const QUERY_LIMIT: usize = 128;
/// The field's label, where the prompt's mark is.
const LABEL: &str = "Find:";
/// A match, under its glyphs.
const MATCH: [f32; 4] = [1.0, 0.78, 0.25, 0.28];
/// The current match.
const CURRENT: [f32; 4] = [1.0, 0.62, 0.1, 0.6];
/// The count while nothing matches.
const NO_MATCH: [f32; 4] = [1.0, 0.55, 0.52, 1.0];

/// A match: bytes `start..end` of scrollback line `line`'s text (colour codes inside
/// it included).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Hit {
    pub(super) line: u64,
    pub(super) start: usize,
    pub(super) end: usize,
}

/// The find bar's state, kept with the classic view's.
#[derive(Default)]
pub(crate) struct Find {
    open: bool,
    query: String,
    edit: LineEdit,
    dead_key: DeadKey,
    /// Every match, oldest first.
    hits: Vec<Hit>,
    /// Index in `hits` of the current match.
    current: Option<usize>,
    /// What `hits` were found in: lines written, lines kept, `con_chatOnly`; `None`
    /// once the search changed.
    seen: Option<(u64, usize, bool)>,
    /// Scroll to the current match next frame.
    jump: bool,
    /// The count's text, `n/m`.
    count: String,
    /// The text of a line being searched, lowercase without its colour codes, each
    /// character with its bytes in the line; and the search, lowercase.
    plain: Vec<(char, usize, usize)>,
    needle: Vec<char>,
}

impl Find {
    pub(crate) fn is_open(&self) -> bool {
        self.open
    }

    /// Open the field on the last search, selected, and look for it again.
    fn show(&mut self) {
        self.open = true;
        self.edit.select_all(&self.query);
        self.searched_anew();
    }

    pub(crate) fn close(&mut self) {
        self.open = false;
        self.jump = false;
        self.dead_key.settle();
    }

    /// The search changed: find its matches again and go to the newest.
    fn searched_anew(&mut self) {
        self.seen = None;
        self.current = None;
    }

    /// Go to the next older match (`older`) or the next newer one, wrapping round.
    fn step(&mut self, older: bool) {
        let count = self.hits.len();
        if count == 0 {
            return;
        }
        let index = match self.current {
            Some(index) if older => index.checked_sub(1).unwrap_or(count - 1),
            Some(index) => (index + 1) % count,
            None => count - 1,
        };
        self.current = Some(index);
        self.jump = true;
    }

    /// Which match is current, counted from the newest, and how many there are; `None`
    /// while the field is empty.
    pub(super) fn count(&self) -> Option<(usize, usize)> {
        (!self.query.is_empty()).then(|| {
            let total = self.hits.len();
            (self.current.map_or(0, |index| total - index), total)
        })
    }

    fn current_hit(&self) -> Option<Hit> {
        self.hits.get(self.current?).copied()
    }

    /// The matches on scrollback line `line`.
    fn on_line(&self, line: u64) -> &[Hit] {
        let start = self.hits.partition_point(|hit| hit.line < line);
        let end = self.hits.partition_point(|hit| hit.line <= line);
        &self.hits[start..end]
    }

    /// Mark the matches on a drawn row: `text` is bytes `span` of line `line`, drawn
    /// from column `text_column` of the row whose top is `y`.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn highlight(
        &self,
        frame: &mut ConsoleFrame,
        grid: &Grid,
        line: u64,
        span: Span,
        text: &str,
        text_column: usize,
        y: f32,
    ) {
        if !self.open {
            return;
        }
        let current = self.current_hit();
        for hit in self.on_line(line) {
            let (start, end) = (hit.start.max(span.start), hit.end.min(span.end));
            if start >= end {
                continue;
            }
            let left = column_of(text, start - span.start);
            let right = column_of(text, end - span.start);
            if right <= left {
                continue;
            }
            frame.quads.push(SolidQuad {
                rect: [
                    grid.x(text_column + left),
                    y,
                    (right - left) as f32 * grid.width,
                    grid.pitch,
                ],
                color: if current == Some(*hit) {
                    CURRENT
                } else {
                    MATCH
                },
            });
        }
    }

    /// Type `text` at the caret, as a paste does.
    fn type_text(&mut self, text: &str) {
        self.edit.insert(&mut self.query, text, QUERY_LIMIT);
        self.searched_anew();
    }
}

/// Call `found` with the byte range of each match of `needle` (lowercase) in `text`,
/// ignoring case and colour codes, left to right without overlaps. `plain` is
/// scratch room.
pub(super) fn find_matches(
    text: &str,
    needle: &[char],
    plain: &mut Vec<(char, usize, usize)>,
    mut found: impl FnMut(Range<usize>),
) {
    if needle.is_empty() {
        return;
    }
    plain.clear();
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if colour_code(bytes, index).is_some() {
            index += 2;
            continue;
        }
        let Some(character) = text[index..].chars().next() else {
            break;
        };
        let end = index + character.len_utf8();
        for lower in character.to_lowercase() {
            plain.push((lower, index, end));
        }
        index = end;
    }
    let length = needle.len();
    let mut at = 0;
    while at + length <= plain.len() {
        let window = &plain[at..at + length];
        if window
            .iter()
            .map(|(character, _, _)| *character)
            .eq(needle.iter().copied())
        {
            found(window[0].1..window[length - 1].2);
            at += length;
        } else {
            at += 1;
        }
    }
}

/// The letter of a Ctrl+letter press: its control character where the platform
/// reports one (it follows the keyboard layout), otherwise the layout's character
/// under a held Ctrl.
fn control_letter(event: &KeyEvent, control: bool) -> Option<char> {
    if let Some(text) = event.text.as_deref() {
        let mut characters = text.chars();
        if let (Some(code @ '\u{1}'..='\u{1a}'), None) = (characters.next(), characters.next()) {
            return char::from_u32('a' as u32 + code as u32 - 1);
        }
    }
    match &event.logical_key {
        Key::Character(text) if control => text.chars().next().map(|c| c.to_ascii_lowercase()),
        _ => None,
    }
}

/// The find field as a field dead-key composition types into.
struct FindLine<'a> {
    text: &'a mut String,
    edit: &'a mut LineEdit,
}

impl TypingField for FindLine<'_> {
    fn line(&self) -> &str {
        self.text
    }

    fn caret(&self) -> usize {
        self.edit.cursor(self.text)
    }

    fn insert(&mut self, text: &str) {
        self.edit.insert(self.text, text, QUERY_LIMIT);
    }

    fn remove(&mut self, range: Range<usize>) {
        let caret = keep_caret(self.edit.cursor(self.text), &range);
        self.text.replace_range(range, "");
        self.edit.place(self.text, caret, false);
    }
}

impl ViewerConsole {
    /// Ctrl+F opens the find bar; while it is open it takes the keys it uses.
    /// `false` leaves the key to the console.
    pub(in crate::console) fn find_key(&mut self, event: &KeyEvent, key: KeyCode) -> bool {
        let (control, shift) = (self.control, self.shift);
        let letter = control_letter(event, control);
        if !self.classic.find.open {
            if letter == Some('f') {
                self.dead_key.settle();
                self.classic.find.show();
                return true;
            }
            return false;
        }
        let find = &mut self.classic.find;
        let text = event.text.as_deref();
        let word = |plain, by_word| if control { by_word } else { plain };
        let before = find.query.len();
        match key {
            // Scrolling stays the console's.
            KeyCode::PageUp | KeyCode::PageDown => return false,
            KeyCode::Home | KeyCode::End if control => return false,
            KeyCode::Escape => {
                find.dead_key.other_key(text);
                find.close();
            }
            KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::F3 => {
                find.dead_key.other_key(text);
                find.step(!shift);
            }
            KeyCode::ArrowUp => find.step(true),
            KeyCode::ArrowDown => find.step(false),
            KeyCode::Tab => find.dead_key.other_key(text),
            KeyCode::ArrowLeft => {
                find.edit
                    .motion(&find.query, word(Motion::Left, Motion::WordLeft), shift);
            }
            KeyCode::ArrowRight => {
                find.edit
                    .motion(&find.query, word(Motion::Right, Motion::WordRight), shift);
            }
            KeyCode::Home => find.edit.motion(&find.query, Motion::Home, shift),
            KeyCode::End => find.edit.motion(&find.query, Motion::End, shift),
            KeyCode::Backspace => {
                find.dead_key.other_key(text);
                find.edit
                    .delete(&mut find.query, word(Motion::Left, Motion::WordLeft));
                find.searched_anew();
            }
            KeyCode::Delete => {
                find.edit
                    .delete(&mut find.query, word(Motion::Right, Motion::WordRight));
                find.searched_anew();
            }
            KeyCode::Insert if shift => {
                if let Some(pasted) = clipboard::paste() {
                    find.type_text(&pasted);
                }
            }
            KeyCode::Insert => {
                if control {
                    copy_query(find);
                }
            }
            _ => match letter {
                Some('a' | 'f') => find.edit.select_all(&find.query),
                Some('c') => copy_query(find),
                Some('x') => {
                    if let Some(range) = find.edit.selection(&find.query)
                        && clipboard::copy(&find.query[range])
                    {
                        find.edit.delete(&mut find.query, Motion::Left);
                        find.searched_anew();
                    }
                }
                Some('v') => {
                    if let Some(pasted) = clipboard::paste() {
                        find.type_text(&pasted);
                    }
                }
                // A held dead key only arms its accent once.
                _ if !event.repeat || !matches!(event.logical_key, Key::Dead(_)) => {
                    let mut dead = find.dead_key;
                    dead.type_key(
                        &mut FindLine {
                            text: &mut find.query,
                            edit: &mut find.edit,
                        },
                        &event.logical_key,
                        text,
                    );
                    find.dead_key = dead;
                    if find.query.len() != before {
                        find.searched_anew();
                    }
                }
                _ => {}
            },
        }
        true
    }

    /// Find the search's matches again when the search, the scrollback or
    /// `con_chatOnly` changed, keeping the current match where it still is.
    pub(super) fn refresh_find(&mut self, chat_only: bool) {
        if !self.classic.find.open {
            return;
        }
        let end = self.shell.lines_written();
        let kept = self.shell.lines().count();
        let seen = Some((end, kept, chat_only));
        if self.classic.find.seen == seen {
            return;
        }
        let mut find = std::mem::take(&mut self.classic.find);
        let kept_hit = find.current_hit();
        find.seen = seen;
        find.hits.clear();
        find.needle.clear();
        find.needle
            .extend(find.query.chars().flat_map(char::to_lowercase));
        let first = end - kept as u64;
        for (line, entry) in (first..).zip(self.shell.lines()) {
            if chat_only && !self.is_chat_line(line) {
                continue;
            }
            let hits = &mut find.hits;
            find_matches(&entry.text, &find.needle, &mut find.plain, |range| {
                hits.push(Hit {
                    line,
                    start: range.start,
                    end: range.end,
                });
            });
        }
        find.current = match kept_hit {
            Some(hit) => find
                .hits
                .iter()
                .position(|other| *other == hit)
                .or_else(|| find.hits.len().checked_sub(1)),
            None => {
                find.jump = !find.hits.is_empty();
                find.hits.len().checked_sub(1)
            }
        };
        self.classic.find = find;
    }

    /// Scroll the view to the current match after a step or a new search, putting it
    /// mid-way up the `visible` rows when there are rows enough under it.
    pub(super) fn find_jump(&mut self, visible: usize) {
        if !std::mem::take(&mut self.classic.find.jump) {
            return;
        }
        let Some(hit) = self.classic.find.current_hit() else {
            return;
        };
        let rows = &self.classic.rows;
        let first = rows.partition_point(|row| row.line < hit.line);
        let Some(slot) = (first..rows.len())
            .take_while(|&slot| rows[slot].line == hit.line)
            .find(|&slot| !rows[slot].under && rows[slot].span.end > hit.start)
            .or_else(|| (first < rows.len() && rows[first].line == hit.line).then_some(first))
        else {
            return;
        };
        let from_bottom = rows.len() - 1 - slot;
        self.scroll_offset = from_bottom.saturating_sub(visible / 2);
    }

    /// The find field at `y`, in place of the input row: the clock (classic),
    /// "Find:" where the prompt's mark is, the search with its caret and selection,
    /// and the count at the right.
    pub(super) fn find_row(
        &mut self,
        frame: &mut ConsoleFrame,
        painter: &Painter<'_>,
        y: f32,
        prompt: Prompt,
    ) {
        let grid = painter.grid;
        if prompt.clock {
            let clock = self.classic.clock;
            for (column, byte) in clock.iter().enumerate() {
                painter.glyph(frame, *byte, grid.x(column), y, quake_color(2));
            }
        }
        let blink = (self.shell.command_clock_millis() >> BLINK_SHIFT) & 1 == 1;
        let find = &mut self.classic.find;
        painter.raw(frame, LABEL, prompt.mark_column, y, prompt.mark_color);
        let input_column = prompt.mark_column + LABEL.len() + 1;

        find.count.clear();
        let count = find.count();
        if let Some((current, total)) = count {
            let _ = write!(find.count, "{current}/{total}");
        }
        let count_column = (grid.columns + 1).saturating_sub(find.count.len());
        let color = if count.is_some_and(|(_, total)| total == 0) {
            NO_MATCH
        } else {
            prompt.mark_color
        };
        painter.raw(frame, &find.count, count_column, y, color);

        let room = count_column.saturating_sub(input_column + 2).max(1);
        let query = &find.query;
        let cursor = find.edit.cursor(query);
        let caret = cell_count(&query[..cursor]);
        let prestep = caret.saturating_sub(room - 1);
        let start = byte_of_cell(query, prestep);
        let shown_end = byte_of_cell(query, prestep + room);
        if let Some(range) = find.edit.selection(query) {
            let left = cell_count(&query[..range.start]).saturating_sub(prestep);
            let right = cell_count(&query[..range.end])
                .saturating_sub(prestep)
                .min(room);
            if right > left {
                frame.quads.push(SolidQuad {
                    rect: [
                        grid.x(input_column + left),
                        y,
                        (right - left) as f32 * grid.width,
                        grid.pitch,
                    ],
                    color: painter.ink.highlight,
                });
            }
        }
        painter.raw(
            frame,
            &query[start..shown_end],
            input_column,
            y,
            prompt.text_color,
        );
        if !blink {
            let x = grid.x(input_column + caret - prestep);
            if prompt.gold_caret {
                frame
                    .quads
                    .push(sjk::caret(&grid, x, y + grid.inset(), false));
            } else if painter.font.is_modern() {
                painter.glyph(frame, b'_', x, y, prompt.text_color);
            } else {
                frame.quads.push(SolidQuad {
                    rect: cursor_rect(&grid, x, y, false),
                    color: prompt.text_color,
                });
            }
        }
    }
}

/// Copy the field's selection, or the whole search when nothing is selected.
fn copy_query(find: &Find) {
    let text = match find.edit.selection(&find.query) {
        Some(range) => &find.query[range],
        None => &find.query,
    };
    let _ = clipboard::copy(text);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matches(text: &str, needle: &str) -> Vec<Range<usize>> {
        let needle: Vec<char> = needle.chars().flat_map(char::to_lowercase).collect();
        let mut found = Vec::new();
        find_matches(text, &needle, &mut Vec::new(), |range| found.push(range));
        found
    }

    fn console() -> (tempfile::TempDir, ViewerConsole) {
        let directory = tempfile::tempdir().unwrap();
        let console = ViewerConsole::new(directory.path().join("config.cfg")).unwrap();
        (directory, console)
    }

    /// Open the bar on `query`, as typing it would.
    fn search(console: &mut ViewerConsole, query: &str, chat_only: bool) {
        console.classic.find.show();
        console.classic.find.type_text(query);
        console.refresh_find(chat_only);
    }

    #[test]
    fn matching_ignores_case_and_colour_codes() {
        assert_eq!(matches("Hello hello HELLO", "hello"), [0..5, 6..11, 12..17]);
        // The match covers the codes inside it, so its marks span them.
        assert_eq!(matches("^1He^7llo world", "hello"), [2..9]);
        assert_eq!(matches("Sol^7: GG", "sol: gg"), [0..9]);
        assert_eq!(matches("ÉCLAIR", "éclair"), [0..7]);
        assert!(matches("hello", "").is_empty());
        assert!(matches("hel", "hello").is_empty());
    }

    #[test]
    fn matches_do_not_overlap() {
        assert_eq!(matches("aaaa", "aa"), [0..2, 2..4]);
    }

    #[test]
    fn the_count_starts_at_the_newest_and_steps_wrap_round() {
        let (_directory, mut console) = console();
        console.push_log("first duel");
        console.push_log("nothing here");
        console.push_log("^2Duel^7 again");
        console.push_log("last DUEL");
        search(&mut console, "duel", false);
        let find = &mut console.classic.find;
        assert_eq!(find.count(), Some((1, 3)));
        assert!(find.jump, "a new search goes to its newest match");
        find.step(true);
        assert_eq!(find.count(), Some((2, 3)));
        find.step(true);
        find.step(true);
        assert_eq!(find.count(), Some((1, 3)), "older past the oldest wraps");
        find.step(false);
        assert_eq!(find.count(), Some((3, 3)), "newer past the newest wraps");
    }

    #[test]
    fn nothing_matching_counts_none_and_an_empty_field_shows_no_count() {
        let (_directory, mut console) = console();
        console.push_log("hello");
        search(&mut console, "", false);
        assert_eq!(console.classic.find.count(), None);
        console.classic.find.type_text("zz");
        console.refresh_find(false);
        assert_eq!(console.classic.find.count(), Some((0, 0)));
        console.classic.find.step(true);
        assert_eq!(console.classic.find.count(), Some((0, 0)));
    }

    #[test]
    fn new_lines_keep_the_current_match() {
        let (_directory, mut console) = console();
        console.push_log("kyber one");
        console.push_log("kyber two");
        search(&mut console, "kyber", false);
        console.classic.find.step(true);
        let current = console.classic.find.current_hit();
        console.push_log("kyber three");
        console.refresh_find(false);
        assert_eq!(console.classic.find.current_hit(), current);
        assert_eq!(console.classic.find.count(), Some((3, 3)));
    }

    #[test]
    fn chat_only_searches_the_chat_lines_alone() {
        let (_directory, mut console) = console();
        console.push_log("gg from the server");
        let chat = console.shell.lines_written();
        console.push_chat_line("Sol^7: gg".into(), "Sol^7\u{19}: gg", None);
        search(&mut console, "GG", true);
        assert_eq!(console.classic.find.count(), Some((1, 1)));
        assert_eq!(
            console.classic.find.current_hit().map(|hit| hit.line),
            Some(chat)
        );
        console.refresh_find(false);
        assert_eq!(
            console.classic.find.count().map(|(_, total)| total),
            Some(2)
        );
    }

    #[test]
    fn a_step_scrolls_the_match_into_view() {
        let (_directory, mut console) = console();
        console.push_log("needle at the top");
        for n in 0..40 {
            console.push_log(&format!("filler {n}"));
        }
        console.push_log("needle at the bottom");
        console.classic_rows(80, false);
        search(&mut console, "needle", false);
        console.find_jump(10);
        assert_eq!(console.scroll_offset, 0, "the newest match is on show");
        console.classic.find.step(true);
        console.find_jump(10);
        let total = console.classic.rows.len();
        let newest = total - 1 - console.scroll_offset;
        let hit = console.classic.find.current_hit().unwrap();
        let slot = console
            .classic
            .rows
            .iter()
            .position(|row| row.line == hit.line)
            .unwrap();
        assert!(slot <= newest && newest - slot < 10, "{slot} {newest}");
        // No step, no scroll: the player's own scrolling stays.
        console.scroll_offset = 3;
        console.find_jump(10);
        assert_eq!(console.scroll_offset, 3);
    }

    #[test]
    fn the_find_field_leaves_the_input_line_alone() {
        let (_directory, mut console) = console();
        console.input = "say hi".into();
        search(&mut console, "abc", false);
        assert_eq!(console.input, "say hi");
        assert!(console.classic.find.is_open());
        console.set_open(true);
        console.set_open(false);
        assert!(
            !console.classic.find.is_open(),
            "closing the console closes it"
        );
        console.classic.find.show();
        assert_eq!(
            console
                .classic
                .find
                .edit
                .selection(&console.classic.find.query),
            Some(0..3),
            "the last search comes back selected"
        );
    }
}
