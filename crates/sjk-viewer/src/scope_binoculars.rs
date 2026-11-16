//! The electrobinoculars' view (zoom mode 2), as `CG_DrawZoomMask` draws it (OpenJK
//! and JoF EJK `codemp/cgame/cg_draw.c`, the `zoomMode == 2` branch): the blue static
//! window, the heading strip under it, the pulsing circle, the frame (`gfx/2d/binMask`),
//! the zoom arrow down the left edge and the triangle at the top that flips now and then.
//! Everything is placed on the 640x480 virtual screen and stretched to the window, as
//! the disruptor's mask is.
//!
//! [`super::gpu::Mask`] draws the pictures with their shaders' blends. The heading
//! numbers, chunky bitmap digits in stock, are drawn by the HUD in its vector font
//! (SJK draws no bitmap fonts), each digit centred in the cell stock gives it, and only
//! the cells inside the strip's black box: stock lets the frame cover the rest. JoF's
//! team readout beside the binoculars and its JA Pro race-mode exception are not
//! ported.

/// `level`'s scale: the arrow's travel in virtual pixels over the zoom range.
const ARROW_TRAVEL: f32 = 162.0;
/// The black box behind the heading numbers (`CG_DrawPic(212, 367, 200, 40, white)`).
pub(crate) const NUMBER_BOX: [f32; 4] = [212.0, 367.0, 200.0, 40.0];
/// A heading digit's cell (`CG_DrawNumField(..., 24, 14, NUM_FONT_CHUNKY, ...)`).
pub(crate) const DIGIT_SIZE: [f32; 2] = [24.0, 14.0];
/// `NUM_FONT_CHUNKY`'s advance, `charWidth / 1.2 + 2` truncated to an int.
const DIGIT_ADVANCE: f32 = 22.0;
/// The numbers' row and the markers' row.
const NUMBER_Y: f32 = 374.0;
const MARKER_Y: f32 = 376.0;
/// The numbers' colour, a faint green (`color1` before `CG_DrawNumField`).
pub(crate) const NUMBER_COLOR: [f32; 4] = [0.2, 0.4, 0.2, 0.3];

/// How far down the arrow sits for `zoom_fov`, the player state's `zoomFov`.
pub(crate) fn arrow_level(zoom_fov: f32) -> f32 {
    ((80.0 - zoom_fov) / 80.0).clamp(0.0, 1.0) * ARROW_TRAVEL
}

/// One number of the heading strip: its field's left edge, the value shown, and the
/// 6x6 marker's left edge, all on the virtual screen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Mark {
    pub(crate) number_x: f32,
    pub(crate) value: i32,
    pub(crate) marker_x: f32,
}

/// The heading strip for a view turned `yaw` degrees: numbers every ten degrees with
/// 200 added ("just to bump the range up"), sliding with the fraction of ten. The
/// leftmost shows only past three degrees, as stock keeps it from poking out of the
/// mask. `yaw` is wrapped to stock's -180 to 180 first.
pub(crate) fn strip(yaw: f32) -> impl Iterator<Item = Mark> {
    let heading = (yaw + 180.0).rem_euclid(360.0);
    // C's `(int)` truncates; the heading is never negative here.
    let mut value = (heading / 10.0) as i32 * 10;
    let off = heading - value as f32;
    (-10..30).step_by(10).filter_map(move |i: i32| {
        value -= 10;
        if value < 0 {
            value += 360;
        }
        ((off > 3.0 && i == -10) || i > -10).then(|| Mark {
            number_x: 155.0 + i as f32 * 10.0 + off * 10.0,
            value: value + 200,
            marker_x: 245.0 + (i - 1) as f32 * 10.0 + off * 10.0,
        })
    })
}

/// The digit cells of one strip number (`CG_DrawNumField` width 3, zero-filled: no
/// leading offset), as (left edge, digit), keeping only cells inside [`NUMBER_BOX`].
pub(crate) fn digits(mark: Mark) -> impl Iterator<Item = (f32, u8)> {
    // Width 3 clamps the value to 999 (the strip's values stay 200 to 559).
    let value = mark.value.clamp(0, 999) as u32;
    [value / 100, value / 10 % 10, value % 10]
        .into_iter()
        .enumerate()
        .map(move |(index, digit)| (mark.number_x + index as f32 * DIGIT_ADVANCE, digit as u8))
        .filter(|&(x, _)| x >= NUMBER_BOX[0] && x + DIGIT_SIZE[0] <= NUMBER_BOX[0] + NUMBER_BOX[2])
}

/// The circle's pulse: `sin(time * 0.01) * 0.5 + 0.5`, squared, as its grey.
pub(crate) fn circle_grey(time: i32) -> f32 {
    let pulse = (time as f32 * 0.01).sin() * 0.5 + 0.5;
    pulse * pulse
}

/// Whether the top triangle flips this frame: stock flips on a 2% chance while
/// `cg.time & 1024`; `chance` stands in for `Q_flrand(0, 1)`.
pub(crate) fn flips(time: i32, chance: f32) -> bool {
    chance > 0.98 && time & 1024 != 0
}

/// The text the HUD draws for digit `id` (see [`TEXT_FIRST`]).
pub(crate) const DIGIT_TEXT: [&str; 10] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
/// The HUD's text ids for the ten digits.
pub(crate) const TEXT_FIRST: u32 = 330;
pub(crate) const TEXT_LAST: u32 = TEXT_FIRST + 9;

/// This frame's heading strip for the HUD, set while the binocular view shows.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Heading {
    pub(crate) yaw: Option<f32>,
}

impl Heading {
    /// Append the strip's digits to `list` over a `viewport` (physical pixels).
    pub(crate) fn emit(&self, list: &mut sjk_ui::DrawList, viewport: [f32; 2]) {
        use sjk_ui::{Color, DrawCommand, FontWeight, Rect, TextAlign, TextId, TextOverflow};
        let Some(yaw) = self.yaw else {
            return;
        };
        let [sx, sy] = [viewport[0] / 640.0, viewport[1] / 480.0];
        let [r, g, b, a] = NUMBER_COLOR;
        for mark in strip(yaw) {
            for (x, digit) in digits(mark) {
                let _ = list.push(DrawCommand::Text {
                    rect: Rect::new(
                        x * sx,
                        NUMBER_Y * sy,
                        DIGIT_SIZE[0] * sx,
                        DIGIT_SIZE[1] * sy,
                    ),
                    text: TextId(TEXT_FIRST + u32::from(digit)),
                    size: DIGIT_SIZE[1] * sy,
                    color: Color::new(r, g, b, a),
                    align: TextAlign::Center,
                    overflow: TextOverflow::Clip,
                    weight: FontWeight::Semibold,
                    letter_spacing: 0.0,
                });
            }
        }
    }
}

/// The marker squares' left edges for `yaw`.
pub(crate) fn markers(yaw: f32) -> impl Iterator<Item = f32> {
    strip(yaw).map(|mark| mark.marker_x)
}

/// The marker row and size (`CG_DrawPic(x, 376, 6, 6, white)`).
pub(crate) const MARKER: [f32; 2] = [MARKER_Y, 6.0];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_strip_reads_as_stock_draws_it() {
        // Facing yaw 0: heading 180, no fraction, so the leftmost number is skipped.
        let marks: Vec<Mark> = strip(0.0).collect();
        assert_eq!(marks.len(), 3);
        assert_eq!(
            marks[0],
            Mark {
                number_x: 155.0,
                value: 360,
                marker_x: 235.0,
            }
        );
        assert_eq!(marks[2].value, 340);
        // Five degrees past: the leftmost shows and everything slides 50 to the right.
        let marks: Vec<Mark> = strip(5.0).collect();
        assert_eq!(marks.len(), 4);
        assert_eq!(marks[0].number_x, 105.0);
        assert_eq!(marks[0].value, 370);
        // Facing -180 wraps the value below zero back round.
        assert!(strip(-180.0).all(|mark| (200..560).contains(&mark.value)));
        assert_eq!(
            strip(180.0).collect::<Vec<_>>(),
            strip(-180.0).collect::<Vec<_>>()
        );
    }

    #[test]
    fn only_digits_inside_the_box_are_drawn() {
        let mark = Mark {
            number_x: 200.0,
            value: 345,
            marker_x: 0.0,
        };
        // 200 is left of the box; 222 and 244 are inside.
        assert_eq!(digits(mark).collect::<Vec<_>>(), [(222.0, 4), (244.0, 5)]);
        assert_eq!(arrow_level(80.0), 0.0);
        assert_eq!(arrow_level(0.0), 162.0);
        assert_eq!(arrow_level(40.0), 81.0);
        assert!(flips(1024, 0.99) && !flips(1023, 0.99) && !flips(1024, 0.5));
        assert!((circle_grey(0) - 0.25).abs() < 1e-6);
    }
}
