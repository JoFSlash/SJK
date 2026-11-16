//! `CG_DrawHaqrBar` (JoF EJK `cg_draw.c:7264-7302`): while the player hacks a `usetime`
//! trigger, a bar under the crosshair shows how much of the hack is left (a black frame,
//! the part still to go in yellow and the part done greyed out), with the hacker icon
//! (`gfx/mp/c_icon_tech`) standing on it.
//!
//! Stock draws it from `CG_DrawCrosshair`, so it goes with the crosshair: not drawn when
//! `cg_drawCrosshair` is 0. Its caller pushes it down under a target's health bar or Siege
//! info (`cg_draw.c:7825-7862`); SJK draws neither under the crosshair, so the bar always
//! stands right under it.
use sjk_ui::{Color, DrawCommand, DrawList, Rect, TextureId};

/// `HEALTH_WIDTH`, `HEALTH_HEIGHT`, in the 640 by 480 screen.
const WIDTH: f32 = 50.0;
const HEIGHT: f32 = 5.0;
/// Units between the crosshair's bottom and the bar's top.
const GAP: f32 = 8.0;
const FRAME: Color = Color::new(0.0, 0.0, 0.0, 1.0);
const LEFT: Color = Color::new(1.0, 1.0, 0.0, 0.4);
const DONE: Color = Color::new(0.5, 0.5, 0.5, 0.1);
const ICON: Color = Color::new(1.0, 1.0, 1.0, 1.0);

/// The share of the hack still to go: `ps.hackingTime` (when it is done) less `time`,
/// against `ps.hackingBaseTime`; `None` while nothing is hacked.
pub(super) fn left(done_at: i32, length: i32, time: i32) -> Option<f32> {
    (done_at != 0 && length > 0).then(|| (done_at - time) as f32 / length as f32)
}

/// Draw the bar for `left` (see [`left`]) under `crosshair`, the drawn crosshair's
/// rectangle in a `viewport` of physical pixels, and `icon` on it when its picture
/// loaded. Stock draws nothing for less than a unit of it, or for more than the whole.
///
/// Units follow the 480-line height, so the bar and icon keep their shape on wide
/// screens as `widthRatioCoef` keeps them, centred on the crosshair as it is.
pub(super) fn emit(
    list: &mut DrawList,
    viewport: [f32; 2],
    left: Option<f32>,
    crosshair: Rect,
    icon: Option<TextureId>,
) {
    let Some(left) = left else {
        return;
    };
    let filled = left * WIDTH;
    if !(1.0..=WIDTH).contains(&filled) {
        return;
    }
    let unit = viewport[1] / 480.0;
    let x = crosshair.x + crosshair.width * 0.5 - WIDTH * 0.5 * unit;
    let y = crosshair.y + crosshair.height + GAP * unit;
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
    // `CG_DrawPic(x, y - HEALTH_WIDTH, HEALTH_WIDTH, HEALTH_WIDTH, hackerIconShader)`.
    if let Some(texture) = icon {
        let _ = list.push(DrawCommand::TexturedQuad {
            rect: Rect::new(x, y - WIDTH * unit, WIDTH * unit, WIDTH * unit),
            texture,
            color: ICON,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ICON_TEXTURE: TextureId = TextureId(7);

    /// A 24-unit crosshair in the middle of a 1920 by 1080 screen (54 pixels a side).
    fn crosshair() -> Rect {
        Rect::new(933.0, 513.0, 54.0, 54.0)
    }

    fn draw(left: Option<f32>, crosshair: Rect, icon: Option<TextureId>) -> DrawList {
        let mut list = DrawList::new(16);
        emit(&mut list, [1920.0, 1080.0], left, crosshair, icon);
        list
    }

    fn frame(list: &DrawList) -> Rect {
        match list.commands()[0] {
            DrawCommand::SolidRect { rect, .. } => rect,
            ref other => panic!("{other:?}"),
        }
    }

    #[test]
    fn the_share_left_follows_the_hack_times() {
        assert_eq!(left(0, 10_000, 5_000), None);
        assert_eq!(left(15_000, 0, 10_000), None);
        assert_eq!(left(15_000, 10_000, 10_000), Some(0.5));
    }

    #[test]
    fn the_bar_stands_eight_units_under_the_crosshair_with_the_icon_on_it() {
        let list = draw(Some(0.5), crosshair(), Some(ICON_TEXTURE));
        // 2.25 pixels a unit: 50 units wide, centred on the crosshair, 8 under it.
        let bar = frame(&list);
        assert_eq!((bar.x, bar.y, bar.width), (903.75, 585.0, 112.5));
        let icons: Vec<Rect> = list
            .commands()
            .iter()
            .filter_map(|command| match command {
                DrawCommand::TexturedQuad { rect, texture, .. } if *texture == ICON_TEXTURE => {
                    Some(*rect)
                }
                _ => None,
            })
            .collect();
        assert_eq!(icons, [Rect::new(903.75, 472.5, 112.5, 112.5)]);
    }

    #[test]
    fn the_bar_follows_a_moved_or_larger_crosshair() {
        let moved = Rect::new(1033.0, 413.0, 108.0, 108.0);
        let bar = frame(&draw(Some(0.5), moved, None));
        assert_eq!((bar.x, bar.y), (1087.0 - 56.25, 413.0 + 108.0 + 18.0));
    }

    #[test]
    fn without_its_picture_the_bar_draws_alone() {
        let list = draw(Some(0.5), crosshair(), None);
        assert_eq!(list.len(), 6);
        assert!(
            list.commands()
                .iter()
                .all(|command| matches!(command, DrawCommand::SolidRect { .. }))
        );
    }

    #[test]
    fn nothing_is_drawn_without_a_hack_or_out_of_range() {
        for left in [None, Some(0.01), Some(1.5), Some(-0.2)] {
            assert!(
                draw(left, crosshair(), Some(ICON_TEXTURE)).is_empty(),
                "{left:?}"
            );
        }
        // Just over one unit left, and the whole bar, are both drawn.
        assert!(!draw(Some(1.1 / WIDTH), crosshair(), None).is_empty());
        assert!(!draw(Some(1.0), crosshair(), None).is_empty());
    }
}
