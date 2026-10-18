//! `CG_DrawHaqrBar` (`cg_draw.c:4766-4803`): while the player hacks a `usetime` trigger, a
//! bar under the crosshair shows how much of the hack is left: a black frame, the part
//! still to go in yellow and the part done greyed out.
//!
//! SJK adds a quiet caption under the bar, "Overriding... N%", where the stock bar says
//! nothing; it replaced an all-capitals "HACKING: N%" in the warmup line at the top of
//! the screen, away from where the player is looking.
use sjk_ui::{Color, DrawCommand, DrawList, FontWeight, Rect, TextAlign, TextId, TextOverflow};
use std::fmt::Write as _;

/// The caption's text ([`super::HudOverlay::resolve_text`]).
pub(super) const TEXT: TextId = TextId(325);
/// `HEALTH_WIDTH`, `HEALTH_HEIGHT`, in the 640 by 480 screen.
const WIDTH: f32 = 50.0;
const HEIGHT: f32 = 5.0;
/// Below the screen's middle: half the stock 24-unit crosshair and the 8 units under it.
const BELOW_CENTRE: f32 = 12.0 + 8.0;
const FRAME: Color = Color::new(0.0, 0.0, 0.0, 1.0);
const LEFT: Color = Color::new(1.0, 1.0, 0.0, 0.4);
const DONE: Color = Color::new(0.5, 0.5, 0.5, 0.1);
/// The caption, in 640 by 480 units: its type size, the gap above it and its box.
const CAPTION_SIZE: f32 = 7.5;
const CAPTION_GAP: f32 = 3.0;
const CAPTION_WIDTH: f32 = 120.0;
/// A soft warm white, a little see-through, so the caption reads without shouting.
const CAPTION: Color = Color::new(1.0, 0.96, 0.86, 0.85);

/// The share of the hack still to go: `ps.hackingTime` (when it is done) less `time`,
/// against `ps.hackingBaseTime`; `None` while nothing is hacked.
pub(super) fn left(done_at: i32, length: i32, time: i32) -> Option<f32> {
    (done_at != 0 && length > 0).then(|| (done_at - time) as f32 / length as f32)
}

/// Write the caption for `left` (see [`left`]) into `text`, which is left empty while
/// nothing is hacked. Reuses `text`'s storage.
pub(super) fn caption(text: &mut String, left: Option<f32>) {
    text.clear();
    if let Some(left) = left {
        let done = ((1.0 - left.clamp(0.0, 1.0)) * 100.0).round() as u32;
        let _ = write!(text, "Overriding... {done}%");
    }
}

/// Draw the bar for `left` (see [`left`]) in a `viewport` of physical pixels, with its
/// caption under it. Stock draws nothing for less than a unit of it, or for more than
/// the whole.
pub(super) fn emit(list: &mut DrawList, viewport: [f32; 2], left: Option<f32>) {
    let Some(left) = left else {
        return;
    };
    let filled = left * WIDTH;
    if !(1.0..=WIDTH).contains(&filled) {
        return;
    }
    let unit = viewport[1] / 480.0;
    let x = viewport[0] * 0.5 - WIDTH * 0.5 * unit;
    let y = viewport[1] * 0.5 + BELOW_CENTRE * unit;
    let mut rect = |rx: f32, ry: f32, w: f32, h: f32, color: Color| {
        let _ = list.push(DrawCommand::SolidRect {
            rect: Rect::new(x + rx * unit, y + ry * unit, w * unit, h * unit),
            color,
        });
    };
    // `CG_DrawRect(x, y, HEALTH_WIDTH, HEALTH_HEIGHT, 1.0f, black)`: a one-unit frame.
    rect(0.0, 0.0, WIDTH, 1.0, FRAME);
    rect(0.0, HEIGHT - 1.0, WIDTH, 1.0, FRAME);
    rect(0.0, 0.0, 1.0, HEIGHT, FRAME);
    rect(WIDTH - 1.0, 0.0, 1.0, HEIGHT, FRAME);
    rect(1.0, 1.0, filled - 1.0, HEIGHT - 1.0, LEFT);
    rect(filled, 1.0, WIDTH - filled - 1.0, HEIGHT - 1.0, DONE);
    let _ = list.push(DrawCommand::Text {
        rect: Rect::new(
            viewport[0] * 0.5 - CAPTION_WIDTH * 0.5 * unit,
            y + (HEIGHT + CAPTION_GAP) * unit,
            CAPTION_WIDTH * unit,
            CAPTION_SIZE * 1.6 * unit,
        ),
        text: TEXT,
        size: CAPTION_SIZE * unit,
        color: CAPTION,
        align: TextAlign::Center,
        overflow: TextOverflow::Clip,
        weight: FontWeight::Regular,
        letter_spacing: 0.4,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bar_shrinks_as_the_hack_goes_on() {
        assert_eq!(left(0, 10_000, 5_000), None);
        assert_eq!(left(15_000, 10_000, 10_000), Some(0.5));
        let mut list = DrawList::new(8);
        emit(&mut list, [640.0, 480.0], left(15_000, 10_000, 10_000));
        assert!(
            list.commands()
                .iter()
                .any(|command| matches!(command, DrawCommand::Text { text: TEXT, .. }))
        );
    }

    #[test]
    fn the_caption_says_how_much_is_overridden() {
        let mut text = String::new();
        caption(&mut text, left(17_200, 10_000, 10_000));
        assert_eq!(text, "Overriding... 28%");
        caption(&mut text, None);
        assert!(text.is_empty());
    }
}
