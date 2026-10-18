//! The siege hack's progress under the crosshair, after `CG_DrawHaqrBar`
//! (`cg_draw.c:4766-4803`): it shows while the player hacks a `usetime` trigger and goes
//! when the hack ends, measured from `ps.hackingTime` and `ps.hackingBaseTime` as stock
//! measures it.
//!
//! The look is SJK's, not stock's (stock draws a black-framed yellow bar that shrinks
//! from the right as the hack goes on): a slim dark pill holding the SJK meters' white
//! track, filled left to right in cyan as the hack is done, with a quiet caption under it,
//! "Overriding... N%". It replaced an all-capitals "HACKING: N%" in the warmup line at the
//! top of the screen, away from where the player is looking.
use sjk_ui::{Color, DrawCommand, DrawList, FontWeight, Rect, TextAlign, TextId, TextOverflow};
use std::fmt::Write as _;

/// The caption's text ([`super::HudOverlay::resolve_text`]).
pub(super) const TEXT: TextId = TextId(325);
/// The pill and the track in it, in 640 by 480 units.
const WIDTH: f32 = 64.0;
const HEIGHT: f32 = 6.0;
const INSET: f32 = 1.5;
/// Below the screen's middle: half the stock 24-unit crosshair and the 8 units under it.
const BELOW_CENTRE: f32 = 12.0 + 8.0;
/// The SJK vehicle HUD's panel and meter track, and its cyan.
const PANEL: Color = Color::new(0.0, 0.0, 0.0, 0.5);
const TRACK: Color = Color::new(1.0, 1.0, 1.0, 0.14);
const FILL: Color = Color::new(0.3, 0.95, 1.0, 1.0);
/// The caption, in 640 by 480 units: its type size, the gap above it and its box.
const CAPTION_SIZE: f32 = 7.5;
const CAPTION_GAP: f32 = 3.0;
const CAPTION_WIDTH: f32 = 120.0;
/// A soft white, a little see-through, so the caption reads without shouting.
const CAPTION: Color = Color::new(0.92, 0.97, 1.0, 0.85);

/// The share of the hack still to go: `ps.hackingTime` (when it is done) less `time`,
/// against `ps.hackingBaseTime`; `None` while nothing is hacked.
pub(super) fn left(done_at: i32, length: i32, time: i32) -> Option<f32> {
    (done_at != 0 && length > 0).then(|| (done_at - time) as f32 / length as f32)
}

/// The share of the hack done, 0 to 1, for `left` (see [`left`]).
fn done(left: f32) -> f32 {
    1.0 - left.clamp(0.0, 1.0)
}

/// Write the caption for `left` (see [`left`]) into `text`, which is left empty while
/// nothing is hacked. Reuses `text`'s storage.
pub(super) fn caption(text: &mut String, left: Option<f32>) {
    text.clear();
    if let Some(left) = left {
        let percent = (done(left) * 100.0).round() as u32;
        let _ = write!(text, "Overriding... {percent}%");
    }
}

/// Draw the bar for `left` (see [`left`]) in a `viewport` of physical pixels, with its
/// caption under it. Nothing is drawn while nothing is hacked, nor for a hack already
/// over or not yet begun (stock's bounds).
pub(super) fn emit(list: &mut DrawList, viewport: [f32; 2], left: Option<f32>) {
    let Some(left) = left.filter(|left| (0.0..=1.0).contains(left)) else {
        return;
    };
    let unit = viewport[1] / 480.0;
    let pill = Rect::new(
        viewport[0] * 0.5 - WIDTH * 0.5 * unit,
        viewport[1] * 0.5 + BELOW_CENTRE * unit,
        WIDTH * unit,
        HEIGHT * unit,
    );
    let _ = list.push(DrawCommand::RoundedRect {
        rect: pill,
        radius: pill.height * 0.5,
        color: PANEL,
    });
    let track = Rect::new(
        pill.x + INSET * unit,
        pill.y + INSET * unit,
        pill.width - 2.0 * INSET * unit,
        pill.height - 2.0 * INSET * unit,
    );
    let _ = list.push(DrawCommand::RoundedRect {
        rect: track,
        radius: track.height * 0.5,
        color: TRACK,
    });
    let filled = track.width * done(left);
    if filled > 0.0 {
        let _ = list.push(DrawCommand::RoundedRect {
            rect: Rect::new(track.x, track.y, filled, track.height),
            radius: (track.height * 0.5).min(filled * 0.5),
            color: FILL,
        });
    }
    let _ = list.push(DrawCommand::Text {
        rect: Rect::new(
            viewport[0] * 0.5 - CAPTION_WIDTH * 0.5 * unit,
            pill.y + pill.height + CAPTION_GAP * unit,
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

    /// The fill's rectangle in the list `emit` drew.
    fn fill(list: &DrawList) -> Option<Rect> {
        list.commands().iter().find_map(|command| match command {
            DrawCommand::RoundedRect { rect, color, .. } if *color == FILL => Some(*rect),
            _ => None,
        })
    }

    #[test]
    fn the_bar_fills_from_the_left_as_the_hack_goes_on() {
        assert_eq!(left(0, 10_000, 5_000), None);
        assert_eq!(left(15_000, 10_000, 10_000), Some(0.5));
        let mut early = DrawList::new(8);
        emit(&mut early, [640.0, 480.0], left(19_000, 10_000, 10_000));
        let mut late = DrawList::new(8);
        emit(&mut late, [640.0, 480.0], left(12_000, 10_000, 10_000));
        let (early, late) = (fill(&early).expect("a fill"), fill(&late).expect("a fill"));
        assert_eq!(early.x, late.x, "both start at the left");
        assert!(late.width > early.width * 4.0);
        // A hack already over draws nothing.
        let mut over = DrawList::new(8);
        emit(&mut over, [640.0, 480.0], Some(-0.1));
        assert!(over.commands().is_empty());
    }

    #[test]
    fn the_caption_says_how_much_is_overridden() {
        let mut text = String::new();
        caption(&mut text, left(17_200, 10_000, 10_000));
        assert_eq!(text, "Overriding... 28%");
        caption(&mut text, None);
        assert!(text.is_empty());
        let mut list = DrawList::new(8);
        emit(&mut list, [640.0, 480.0], Some(0.5));
        assert!(
            list.commands()
                .iter()
                .any(|command| matches!(command, DrawCommand::Text { text: TEXT, .. }))
        );
    }
}
