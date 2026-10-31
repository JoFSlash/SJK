//! Bounded wrapping and viewport-relative conversation geometry.

use crate::text::{self, Carry, TextFace, UiFont};
use std::ops::Range;

pub(super) const WRAP_LINES: usize = 4;

/// Virtual 640x480 HUD pixels to 1080p pixels: the HUD's 480 lines fill the height.
const VIRTUAL_TO_1080: f32 = 1080.0 / 480.0;
/// Line box of the chat's body text in 1080p pixels at `cg_chatBoxFontSize` 1, the
/// size EternalJK draws it: `CG_ChatBox_DrawStrings` (JoF EJK `cg_draw.c:10672`)
/// draws `ocr_a` at scale `0.65 * cg_chatBoxFontSize`, and `ocr_a`'s line is 21
/// retail pixels (`mHeight`), which SJK Chat keeps (`text::retail_font::CHAT`). So
/// its 12-pixel capitals stand 17.6 px tall at 1080 lines, as Inter's do (0.60 of
/// this box). Before 19/10/2026 the box was 18 px, 0.59 of this.
pub(super) const FONT_1080: f32 = 21.0 * 0.65 * VIRTUAL_TO_1080;
/// Pitch of a message's wrapped body rows as a multiple of the body text size:
/// EternalJK's rows are `CHATBOX_FONT_HEIGHT` 20 x 0.65 virtual pixels apart, to
/// [`FONT_1080`]'s 21 x 0.65 line. A descender still clears the next row's capitals
/// by 0.15 of the box in Inter.
pub(super) const ROW_PITCH: f32 = 20.0 / 21.0;

pub(super) struct Geometry {
    pub(super) scale: f32,
    pub(super) left: f32,
    pub(super) top: f32,
    pub(super) bottom: f32,
    pub(super) width: f32,
    pub(super) row: f32,
    pub(super) font: f32,
    /// Extra room after each message character, in pixels (`cg_chatBoxLetterSpacing`).
    pub(super) spacing: f32,
    /// Space between consecutive messages, in pixels (`cg_chatBoxLineSpacing`).
    pub(super) gap: f32,
}

impl Geometry {
    pub(super) fn new(viewport: [f32; 2]) -> Self {
        let scale = crate::ui_scale::height_scale(viewport[1]);
        let left = 40.0 * scale;
        Self {
            scale,
            left,
            top: 150.0 * scale,
            bottom: (viewport[1] * 0.64).min(viewport[1] - 210.0 * scale),
            width: (620.0 * scale).min(viewport[0] - left * 2.0).max(1.0),
            row: FONT_1080 * scale * ROW_PITCH,
            font: FONT_1080 * scale,
            spacing: 0.0,
            gap: super::options::LINE_SPACING_DEFAULT as f32 * scale,
        }
    }
}

pub(super) struct Wrapped {
    pub(super) rows: [Range<usize>; WRAP_LINES],
    /// The colour code in force where each row starts, which that row is drawn
    /// after, so a row goes on in the colour the one before it ended in.
    pub(super) carry: [Carry; WRAP_LINES],
    pub(super) len: usize,
    width: f32,
    /// Room the first row leaves at its start (an SJK chat line's tag and name).
    indent: f32,
    size: f32,
    spacing: f32,
    font_height: f32,
    font_modern: bool,
}

impl Default for Wrapped {
    fn default() -> Self {
        Self {
            rows: std::array::from_fn(|_| 0..0),
            carry: [Carry::NONE; WRAP_LINES],
            len: 0,
            width: 0.0,
            indent: 0.0,
            size: 0.0,
            spacing: 0.0,
            font_height: 0.0,
            font_modern: true,
        }
    }
}

impl Wrapped {
    /// [`Self::update_spaced`] with no indent or spacing.
    #[cfg(test)]
    pub(super) fn update(&mut self, value: &str, font: &UiFont, width: f32, size: f32) {
        self.update_spaced(value, font, width, size, 0.0, 0.0);
    }

    /// [`Self::update_spaced`] with no spacing.
    #[cfg(test)]
    pub(super) fn update_indented(
        &mut self,
        value: &str,
        font: &UiFont,
        width: f32,
        size: f32,
        indent: f32,
    ) {
        self.update_spaced(value, font, width, size, indent, 0.0);
    }

    /// Cache boundaries, never strings. Long words break at a character boundary,
    /// never between a colour code's `^` and its digit ([`fitting_end`]), and each
    /// row keeps the colour code in force where it starts ([`Self::carry`]).
    ///
    /// The first row starts `indent` in, after what is drawn before it on that row
    /// (an SJK chat line's tag and name): the body goes on along the name's row and
    /// wraps only when it is too long. A first word that does not fit after the name
    /// but fits a whole row starts the next row instead, the first row then empty.
    /// Each character but an emoji picture takes `spacing` more pixels, as the body
    /// is drawn with `cg_chatBoxLetterSpacing`.
    pub(super) fn update_spaced(
        &mut self,
        value: &str,
        font: &UiFont,
        width: f32,
        size: f32,
        indent: f32,
        spacing: f32,
    ) {
        if self.len > 0
            && self.width == width
            && self.indent == indent
            && self.size == size
            && self.spacing == spacing
            && self.font_height == font.height
            && self.font_modern == font.is_modern()
        {
            return;
        }
        self.width = width;
        self.indent = indent;
        self.size = size;
        self.spacing = spacing;
        self.font_height = font.height;
        self.font_modern = font.is_modern();
        self.len = 0;
        let mut start = 0;
        let mut carry = Carry::NONE;
        while start < value.len() && self.len < WRAP_LINES {
            let room = if self.len == 0 {
                (width - indent).max(0.0)
            } else {
                width
            };
            let mut end = start + fitting_end_spaced(&value[start..], font, room, size, spacing);
            if end < value.len() {
                match value[start..end].rfind(' ') {
                    Some(space) if space > 0 => end = start + space,
                    // The first word does not fit after the name: when it fits a
                    // whole row, the body starts on the next one.
                    _ if self.len == 0 && indent > 0.0 => {
                        let word = value[start..]
                            .find(' ')
                            .map_or(value.len(), |at| start + at);
                        if word > end
                            && word
                                <= start
                                    + fitting_end_spaced(
                                        &value[start..],
                                        font,
                                        width,
                                        size,
                                        spacing,
                                    )
                        {
                            end = start;
                        }
                    }
                    _ => {}
                }
            }
            self.rows[self.len] = start..end;
            self.carry[self.len] = carry;
            self.len += 1;
            carry = carry.after(&value[start..end]);
            start = end;
            while value.as_bytes().get(start) == Some(&b' ') {
                start += 1;
            }
        }
        if self.len == 0 {
            self.len = 1;
            self.rows[0] = 0..0;
            self.carry[0] = Carry::NONE;
        }
    }
}

/// How wide a body row of `value` is drawn at `size` with `spacing` after each
/// character: its text runs as they measure, each emoji mark the room of its picture
/// (as `view::body_row` lays them out).
pub(super) fn drawn_width(value: &str, font: &UiFont, size: f32, spacing: f32) -> f32 {
    let scale = size / font.height.max(1.0);
    let is_mark = |character: char| super::emoji::mark_index(character).is_some();
    let mut width = 0.0;
    let mut rest = value;
    loop {
        let (run, after) = rest.split_at(rest.find(is_mark).unwrap_or(rest.len()));
        width += text::visible_text_width_style(font, run, scale, TextFace::Regular, spacing);
        let Some(mark) = after.chars().next() else {
            return width;
        };
        width += super::emoji::advance(size);
        rest = &after[mark.len_utf8()..];
    }
}

/// Work is linear in the bounded text length, with no temporary substring copies.
pub(super) fn fitting_end(value: &str, font: &UiFont, width: f32, size: f32) -> usize {
    fitting_end_spaced(value, font, width, size, 0.0)
}

/// [`fitting_end`] with `spacing` pixels after each character but an emoji picture.
pub(super) fn fitting_end_spaced(
    value: &str,
    font: &UiFont,
    width: f32,
    size: f32,
    spacing: f32,
) -> usize {
    let mut used = 0.0;
    let bytes = value.as_bytes();
    let mut i = 0;
    // Colour codes are zero-width, and truncating between `^` and its digit
    // would leave a stray caret in the drawn name. A glyph of several characters (an
    // emoji sequence, `👍🏽`) is measured and kept whole.
    while let Some(c) = value[i..].chars().next() {
        if c == '^' && bytes.get(i + 1).is_some_and(u8::is_ascii_digit) {
            i += 2;
            continue;
        }
        let end = if super::emoji::mark_index(c).is_some() {
            i + c.len_utf8()
        } else {
            i + text::glyph_at(value, i).1
        };
        used += if super::emoji::mark_index(c).is_some() {
            super::emoji::advance(size)
        } else {
            text::visible_text_width_style(
                font,
                &value[i..end],
                size / font.height,
                TextFace::Regular,
                spacing,
            )
        };
        if (used > width || end > 480) && i > 0 {
            return i;
        }
        i = end;
    }
    value.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::{code_per_char, test_font};

    /// `body` wrapped `chars` test-font characters wide: glyphs advance 8 units at
    /// height 12, so a row at size 12 holds `chars` of them.
    fn wrapped(body: &str, chars: usize) -> Wrapped {
        let mut wrapped = Wrapped::default();
        wrapped.update(body, &test_font(), chars as f32 * 8.0, 12.0);
        wrapped
    }

    /// Row `row` as the feed draws it: its carry, then its text.
    fn drawn(body: &str, wrapped: &Wrapped, row: usize) -> String {
        format!("{}{}", wrapped.carry[row], &body[wrapped.rows[row].clone()])
    }

    /// The rows drawn one by one hold, character for character, the colours the
    /// body has drawn as one line (the spaces a break eats left out); all of the
    /// body when it fits in the rows.
    fn assert_goes_on_as_one_line(body: &str, wrapped: &Wrapped) {
        let letters = |text: &str| -> Vec<(char, Option<u8>)> {
            code_per_char(text)
                .into_iter()
                .filter(|&(character, _)| character != ' ')
                .collect()
        };
        let whole = letters(body);
        let rows: Vec<_> = (0..wrapped.len)
            .flat_map(|row| letters(&drawn(body, wrapped, row)))
            .collect();
        assert!(!rows.is_empty());
        let complete = wrapped.rows[wrapped.len - 1].end == body.len();
        assert_eq!(rows, whole[..rows.len()], "{body:?}");
        assert_eq!(complete, rows.len() == whole.len(), "{body:?}");
    }

    #[test]
    fn a_long_green_message_is_green_on_every_row() {
        // The shape of a stock `say`: the server puts `^2` in front of the text.
        let body = "^2alpha bravo charlie delta echo foxtrot golf hotel";
        let wrapped = wrapped(body, 20);
        assert_eq!(wrapped.len, 3);
        assert_eq!(drawn(body, &wrapped, 0), "^2alpha bravo charlie");
        assert_eq!(drawn(body, &wrapped, 1), "^2delta echo foxtrot");
        assert_eq!(drawn(body, &wrapped, 2), "^2golf hotel");
        for row in 0..wrapped.len {
            assert!(
                code_per_char(&drawn(body, &wrapped, row))
                    .iter()
                    .all(|&(_, code)| code == Some(2)),
                "row {row}"
            );
        }
        assert_goes_on_as_one_line(body, &wrapped);
    }

    #[test]
    fn a_message_longer_than_the_rows_keeps_its_colour_to_the_last_one() {
        let body = format!(
            "^5{}",
            "the quick brown fox jumps over the lazy dog ".repeat(6)
        );
        let wrapped = wrapped(&body, 20);
        assert_eq!(wrapped.len, WRAP_LINES);
        assert_eq!(wrapped.carry[0], Carry::NONE);
        for row in 1..WRAP_LINES {
            assert_eq!(wrapped.carry[row].to_string(), "^5", "row {row}");
        }
        assert_goes_on_as_one_line(&body, &wrapped);
    }

    #[test]
    fn a_row_break_inside_a_coloured_name_goes_on_in_the_name_colour() {
        // "DarthVader" in two colours, the stock `^7: ` separator, then a green
        // message; the first row breaks inside the name.
        let body = "^1Darth^4Vader^7: ^2hello there general kenobi";
        let wrapped = wrapped(body, 8);
        assert_eq!(drawn(body, &wrapped, 0), "^1Darth^4Vad");
        assert_eq!(wrapped.carry[1].to_string(), "^4");
        assert_eq!(drawn(body, &wrapped, 1), "^4er^7:");
        // The separator's white goes on until the message says green, and from
        // there every row is green.
        assert_eq!(drawn(body, &wrapped, 2), "^7^2hello");
        assert_eq!(drawn(body, &wrapped, 3), "^2there");
        assert_goes_on_as_one_line(body, &wrapped);
        // The name's own colour reaches the first letter of the next row, and the
        // separator changes it where it says.
        let second = code_per_char(&drawn(body, &wrapped, 1));
        assert_eq!(second[0], ('e', Some(4)));
        assert_eq!(second[2], (':', Some(7)));
    }

    #[test]
    fn a_colour_change_in_the_message_carries_into_the_next_row() {
        let body = "^2all well here ^1but now red words ^3then yellow ones";
        let wrapped = wrapped(body, 16);
        assert_eq!(wrapped.carry[0], Carry::NONE);
        // Row 0 ends in green, row 1 starts in it and turns red, row 2 starts in red.
        assert_eq!(wrapped.carry[1].to_string(), "^2");
        assert_eq!(drawn(body, &wrapped, 1), "^2^1but now red");
        assert_eq!(wrapped.carry[2].to_string(), "^1");
        assert_eq!(drawn(body, &wrapped, 2), "^1words ^3then");
        assert_eq!(wrapped.carry[3].to_string(), "^3");
        assert_goes_on_as_one_line(body, &wrapped);
    }

    #[test]
    fn a_message_without_codes_keeps_the_base_colour_on_every_row() {
        let body = "no colour codes anywhere in this rather long message";
        let wrapped = wrapped(body, 16);
        assert!(wrapped.len >= 3);
        for row in 0..wrapped.len {
            assert_eq!(wrapped.carry[row], Carry::NONE, "row {row}");
            // Drawn as the row's own text: the base colour, as the first row.
            assert_eq!(drawn(body, &wrapped, row), &body[wrapped.rows[row].clone()]);
        }
        assert_goes_on_as_one_line(body, &wrapped);
    }

    #[test]
    fn a_code_set_on_a_row_ending_space_still_starts_the_next_row() {
        // The code sits before the space the break eats, and also after it.
        let body = "^2aaaaaa^1 bbbbbb ^4cccccc";
        let wrapped = wrapped(body, 6);
        assert_eq!(drawn(body, &wrapped, 1), "^1bbbbbb");
        assert_eq!(drawn(body, &wrapped, 2), "^1^4cccccc");
        assert_goes_on_as_one_line(body, &wrapped);
    }

    #[test]
    fn a_code_is_never_split_across_rows_and_takes_no_room() {
        // Codes between every pair of letters, at every width: no row ends on the
        // `^` of a code or starts on its digit, and the rows tile the body.
        let body = "a^1b^2c^3d^4e^5f^6g^7h^8i^9j^0kl";
        for chars in 1..=12 {
            let wrapped = wrapped(body, chars);
            let mut end = 0;
            for row in 0..wrapped.len {
                let range = wrapped.rows[row].clone();
                assert_eq!(range.start, end, "{chars} wide, row {row}");
                end = range.end;
                assert!(
                    !(body[..end].ends_with('^')
                        && body[end..].starts_with(|c: char| c.is_ascii_digit())),
                    "{chars} wide: row {row} ends inside a code"
                );
                // Only the letters are counted, so a row holds `chars` of them.
                let letters = code_per_char(&body[range]).len();
                assert!(letters <= chars.max(1), "{chars} wide: {letters} letters");
            }
            assert_goes_on_as_one_line(body, &wrapped);
        }
    }

    #[test]
    fn a_long_word_broken_at_the_edge_goes_on_in_its_colour() {
        let body = "^3abcdefghijklmnopqrstuvwxyz";
        let wrapped = wrapped(body, 10);
        assert_eq!(wrapped.len, 3);
        assert_eq!(drawn(body, &wrapped, 0), "^3abcdefghij");
        assert_eq!(drawn(body, &wrapped, 1), "^3klmnopqrst");
        assert_eq!(drawn(body, &wrapped, 2), "^3uvwxyz");
        assert_goes_on_as_one_line(body, &wrapped);
    }

    /// `body` wrapped `chars` characters wide, its first row `indent` characters in.
    fn indented(body: &str, chars: usize, indent: usize) -> Vec<&str> {
        let mut wrapped = Wrapped::default();
        wrapped.update_indented(
            body,
            &test_font(),
            chars as f32 * 8.0,
            12.0,
            indent as f32 * 8.0,
        );
        (0..wrapped.len)
            .map(|row| &body[wrapped.rows[row].clone()])
            .collect()
    }

    #[test]
    fn a_short_body_after_a_name_stays_on_its_row() {
        assert_eq!(indented("gg all", 40, 12), ["gg all"]);
    }

    #[test]
    fn a_long_body_after_a_name_wraps_only_where_it_must() {
        // 28 characters after a 12-character name, then whole rows of 40.
        let body = "the quick brown fox jumps over the lazy dog and runs far away from here";
        let rows = indented(body, 40, 12);
        assert_eq!(
            rows,
            [
                "the quick brown fox jumps",
                "over the lazy dog and runs far away",
                "from here"
            ]
        );
        // As wide as a line without the name, the same text wraps as a chat line.
        assert_eq!(indented(body, 40, 0), {
            let mut plain = Wrapped::default();
            plain.update(body, &test_font(), 320.0, 12.0);
            (0..plain.len)
                .map(|row| &body[plain.rows[row].clone()])
                .collect::<Vec<_>>()
        });
    }

    #[test]
    fn a_first_word_that_does_not_fit_after_the_name_starts_the_next_row() {
        assert_eq!(
            indented("extraordinarily so", 40, 30),
            ["", "extraordinarily so"]
        );
        // A word wider than any row is broken where the name's row ends.
        let word = "x".repeat(50);
        assert_eq!(indented(&word, 40, 30)[0], "x".repeat(10));
    }

    #[test]
    fn letter_spacing_wraps_sooner_and_is_part_of_the_cached_wrap() {
        let font = test_font();
        let body = "a".repeat(30);
        let mut wrapped = Wrapped::default();
        // Ten 8-unit characters fill 80 units; with 2 more each, eight do.
        wrapped.update_spaced(&body, &font, 80.0, 12.0, 0.0, 0.0);
        assert_eq!(wrapped.rows[0], 0..10);
        wrapped.update_spaced(&body, &font, 80.0, 12.0, 0.0, 2.0);
        assert_eq!(wrapped.rows[0], 0..8);
    }

    #[test]
    fn the_indent_is_part_of_the_cached_wrap() {
        let font = test_font();
        let body = "alpha bravo charlie delta echo";
        let mut wrapped = Wrapped::default();
        wrapped.update_indented(body, &font, 160.0, 12.0, 0.0);
        assert_eq!(wrapped.len, 2);
        wrapped.update_indented(body, &font, 160.0, 12.0, 120.0);
        assert_eq!(wrapped.len, 3, "a wider name moved words down");
        assert_eq!(&body[wrapped.rows[0].clone()], "alpha");
    }
}
