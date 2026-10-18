//! `CG_DrawHaqrBar` (`cg_draw.c:4766-4803`): while the player hacks a `usetime` trigger, a
//! bar under the crosshair shows how much of the hack is left: a black frame, the part
//! still to go in yellow and the part done greyed out.
use sjk_ui::{Color, DrawCommand, DrawList, Rect};

/// `HEALTH_WIDTH`, `HEALTH_HEIGHT`, in the 640 by 480 screen.
const WIDTH: f32 = 50.0;
const HEIGHT: f32 = 5.0;
/// Below the screen's middle: half the stock 24-unit crosshair and the 8 units under it.
const BELOW_CENTRE: f32 = 12.0 + 8.0;
const FRAME: Color = Color::new(0.0, 0.0, 0.0, 1.0);
const LEFT: Color = Color::new(1.0, 1.0, 0.0, 0.4);
const DONE: Color = Color::new(0.5, 0.5, 0.5, 0.1);

/// The share of the hack still to go: `ps.hackingTime` (when it is done) less `time`,
/// against `ps.hackingBaseTime`; `None` while nothing is hacked.
pub(super) fn left(done_at: i32, length: i32, time: i32) -> Option<f32> {
    (done_at != 0 && length > 0).then(|| (done_at - time) as f32 / length as f32)
}

/// Draw the bar for `left` (see [`left`]) in a `viewport` of physical pixels. Stock draws
/// nothing for less than a unit of it, or for more than the whole.
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
        assert!(!list.is_empty());
    }
}
