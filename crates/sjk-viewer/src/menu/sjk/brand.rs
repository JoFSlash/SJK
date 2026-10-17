//! SlashJK's name and mark, as the menus show them: a rounded accent square
//! with a slash through it, and the name beside it, "Slash" in the text's
//! colour and "JK" in the accent. It is drawn from shapes and the display
//! family, so it needs no picture and scales with every window.

use super::{Frame, color, text};
use crate::menu_widgets::{MenuCanvas, TextFamily};
use sjk_ui::{DrawCommand, FontWeight, Rect, TextAlign};

/// The client's name in the menus.
pub(crate) const NAME: &str = "SlashJK";
/// The name's two parts, drawn in two colours.
const NAME_PARTS: [&str; 2] = ["Slash", "JK"];

/// The mark over window rectangle `rect` (a square): the accent square with its
/// corners rounded and a light slash through it.
pub(crate) fn mark(canvas: &mut MenuCanvas, rect: Rect) {
    let side = rect.width.min(rect.height);
    let list = canvas.draw_list_mut();
    let _ = list.push(DrawCommand::RoundedRect {
        rect,
        radius: side * 0.28,
        color: color::ACCENT,
    });
    let _ = list.push(DrawCommand::Line {
        from: [rect.x + side * 0.63, rect.y + side * 0.24],
        to: [rect.x + side * 0.37, rect.y + side * 0.76],
        width: side * 0.13,
        color: color::TEXT,
    });
}

/// The mark and the name from frame point (`x`, `y`), the top of the mark, with
/// the name's line `size` frame pixels tall; returns the frame x after it.
pub(crate) fn wordmark(canvas: &mut MenuCanvas, frame: &Frame, x: f32, y: f32, size: f32) -> f32 {
    let side = size * 1.05;
    mark(canvas, frame.rect(x, y, side, side));
    let mut cursor = x + side + size * 0.38;
    for (part, colour) in NAME_PARTS
        .into_iter()
        .zip([color::TEXT, color::ACCENT_BRIGHT])
    {
        let width = crate::text::display_width(part, size) * 1.04;
        text(
            canvas,
            TextFamily::Display,
            format_args!("{part}"),
            frame.rect(cursor, y, width + size, side),
            size * frame.s,
            colour,
            FontWeight::Semibold,
            TextAlign::Start,
        );
        cursor += width;
    }
    cursor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_name_is_its_two_parts() {
        assert_eq!(NAME_PARTS.concat(), NAME);
    }

    #[test]
    fn the_wordmark_draws_its_mark_and_both_parts() {
        let mut canvas = MenuCanvas::new();
        canvas.begin_transparent([1920.0, 1080.0]);
        let frame = Frame::new([1920.0, 1080.0]);
        let end = wordmark(&mut canvas, &frame, 96.0, 80.0, 34.0);
        assert!(end > 96.0 + 34.0 * 3.0, "{end}");
        let texts: Vec<_> = canvas
            .draw_list()
            .commands()
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { text, .. } => Some(canvas.stored_text(*text).to_owned()),
                _ => None,
            })
            .collect();
        assert_eq!(texts, ["Slash", "JK"]);
    }
}
