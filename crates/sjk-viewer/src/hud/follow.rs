//! The followed player's name in the top left while spectating them, after JoF
//! EternalJK's `CG_DrawFollow` (`cg_draw.c`): with `PMF_FOLLOW` set, or during demo
//! playback, it paints the viewed client's name, colour codes and all, at (4, 27) of
//! the 640x480 screen with `FONT_MEDIUM` at scale 0.85, and `CG_Draw2D` then skips
//! `CG_DrawWarmup`. Stock Jedi Academy centred "following" and the name at the top;
//! EternalJK moved it to the corner, out of the way of the warmup and vote lines.

use sjk_ui::{Color, DrawCommand, DrawList, FontWeight, Rect, TextAlign, TextId, TextOverflow};

/// `PMF_FOLLOW` (`bg_public.h`): the player state is the followed player's.
pub(super) const PMF_FOLLOW: u16 = 0x1000;

/// The name's text id: retail drew it with `FONT_MEDIUM`
/// ([`super::text_values::retail_font`]).
pub(super) const TEXT: TextId = TextId(324);

/// Line top-left in the 640x480 screen, and `FONT_MEDIUM`'s (`ergoec`) point size
/// times `CG_Text_Paint`'s 0.85.
const ORIGIN: [f32; 2] = [4.0, 27.0];
const LINE: f32 = 20.0 * 0.85;

/// Units of the 480-line screen per pixel of the HUD's 1080-line frame.
const UNIT: f32 = 1_080.0 / 480.0;

/// Whether `CG_DrawFollow` draws: the view follows someone (`movement_flags` holds
/// `PMF_FOLLOW`) or a demo plays.
pub(crate) fn shown(movement_flags: u16, demo: bool) -> bool {
    demo || movement_flags & PMF_FOLLOW != 0
}

/// The name's line box and line height in physical pixels, at the HUD's scale
/// (`dpi_scale`: viewport height over 1080, clamped, times `cg_hudScale`), so it sits
/// where EternalJK puts it wherever that scale equals the height over 1080, and grows
/// with the other HUD text otherwise. The box runs to the same margin on the right.
fn placement(viewport: [f32; 2], dpi_scale: f32) -> (Rect, f32) {
    let unit = UNIT * dpi_scale;
    let [x, y] = ORIGIN.map(|value| value * unit);
    let size = LINE * unit;
    (
        Rect::new(x, y, (viewport[0] - 2.0 * x).max(0.0), size * 1.25),
        size,
    )
}

/// Push the name's text command when `name` is not empty.
pub(super) fn emit(draw_list: &mut DrawList, name: &str, viewport: [f32; 2], dpi_scale: f32) {
    if name.is_empty() {
        return;
    }
    let (rect, size) = placement(viewport, dpi_scale);
    let _ = draw_list.push(DrawCommand::Text {
        rect,
        text: TEXT,
        size,
        // `colorWhite`: the name's own colour codes recolour it.
        color: Color::new(1.0, 1.0, 1.0, 1.0),
        align: TextAlign::Start,
        overflow: TextOverflow::Ellipsis,
        weight: FontWeight::Regular,
        letter_spacing: 0.0,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shown_while_following_or_in_a_demo() {
        assert!(!shown(0, false));
        assert!(shown(PMF_FOLLOW, false));
        assert!(shown(PMF_FOLLOW | 0x40, false));
        assert!(shown(0, true));
    }

    #[test]
    fn placed_at_eternaljks_corner_in_the_hud_frame() {
        // At 1080 lines the HUD frame is the 640x480 screen at 2.25 px a unit.
        let (rect, size) = placement([1920.0, 1080.0], 1.0);
        assert!((rect.x - 9.0).abs() < 1e-3);
        assert!((rect.y - 60.75).abs() < 1e-3);
        assert!((size - 38.25).abs() < 1e-3);
        assert!((rect.right() - (1920.0 - 9.0)).abs() < 1e-3);
        // 1440 lines: a third larger, still in the corner.
        let (rect, size) = placement([2560.0, 1440.0], 1440.0 / 1080.0);
        assert!((rect.x - 12.0).abs() < 1e-3);
        assert!((rect.y - 81.0).abs() < 1e-3);
        assert!((size - 51.0).abs() < 1e-3);
    }

    #[test]
    fn emits_the_name_only_when_there_is_one() {
        let mut list = DrawList::new(4);
        emit(&mut list, "", [1920.0, 1080.0], 1.0);
        assert!(list.commands().is_empty());
        emit(&mut list, "^1Sol^7JK", [1920.0, 1080.0], 1.0);
        let [DrawCommand::Text { text, align, .. }] = list.commands() else {
            panic!("one text command: {:?}", list.commands());
        };
        assert_eq!(*text, TEXT);
        assert_eq!(*align, TextAlign::Start);
    }
}
