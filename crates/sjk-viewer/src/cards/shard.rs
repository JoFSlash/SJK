//! The kyber shard's mark (`docs/cards.md`, "Shards and the shop"): a long crystal with a
//! point at each end, its two facets in two depths of the kyber cyan and a light ridge
//! between them, drawn from shapes as the credits' mark is ([`super::mark`]), so it is
//! sharp at every size.
//!
//! The crystal is a square turned on its point (one thick [`DrawCommand::Line`]),
//! stretched upright by a [`DrawCommand::PushTransform`]; the left facet is the same
//! square again, lighter, under a clip of the left half. A mark is a fixed number of
//! commands ([`COMMANDS`]).

use sjk_ui::{Color, DrawCommand, DrawList, Rect};

/// The kyber cyan: the shards' colour wherever they are counted or shown.
pub(crate) const KYBER: Color = Color::new(0.49, 0.93, 1.0, 1.0);
/// The crystal's deep side.
const DEEP: f32 = 0.55;
/// The crystal's width against its height.
const ASPECT: f32 = 0.42;

/// How many draw commands a mark takes.
#[cfg(test)]
pub(crate) const COMMANDS: usize = 8;

/// How wide a mark `height` tall is.
pub(crate) fn width(height: f32) -> f32 {
    height * ASPECT
}

/// Draw a mark `height` pixels tall with its top left at `at` (window pixels), in
/// `colour` (its lit facet; the other is deeper). Returns its width.
pub(crate) fn draw(list: &mut DrawList, at: [f32; 2], height: f32, colour: Color) -> f32 {
    let width = width(height);
    let half = width * 0.5;
    let [cx, cy] = [at[0] + half, at[1] + height * 0.5];
    // Stretched upright about its middle: a point's height from the middle times `k`.
    let k = height / width;
    let _ = list.push(DrawCommand::PushTransform([
        1.0,
        0.0,
        0.0,
        0.0,
        k,
        cy * (1.0 - k),
        0.0,
        0.0,
        1.0,
    ]));
    // The square on its point, `half` from its middle to each point: a stroke along
    // its diagonal as wide as its side.
    let square = |colour: Color| DrawCommand::Line {
        from: [cx - half * 0.5, cy - half * 0.5],
        to: [cx + half * 0.5, cy + half * 0.5],
        width: half * std::f32::consts::SQRT_2,
        color: colour,
    };
    let deep = Color::new(colour.r * DEEP, colour.g * DEEP, colour.b * DEEP, colour.a);
    let _ = list.push(square(deep));
    let _ = list.push(DrawCommand::PushClip(Rect::new(
        cx - half,
        cy - half,
        half,
        half * 2.0,
    )));
    let _ = list.push(square(colour));
    let _ = list.push(DrawCommand::PopClip);
    // The ridge between the facets, and a glint on the lit one near the top.
    let light = Color::new(1.0, 1.0, 1.0, 0.6 * colour.a);
    let _ = list.push(DrawCommand::Line {
        from: [cx, cy - half * 0.92],
        to: [cx, cy + half * 0.92],
        width: (width * 0.07).max(1.0),
        color: light,
    });
    let _ = list.push(DrawCommand::Line {
        from: [cx - half * 0.62, cy - half * 0.12],
        to: [cx - half * 0.28, cy - half * 0.5],
        width: (width * 0.06).max(1.0),
        color: Color::new(1.0, 1.0, 1.0, 0.45 * colour.a),
    });
    let _ = list.push(DrawCommand::PopTransform);
    width
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mark_is_a_fixed_number_of_commands_balanced() {
        let mut list = DrawList::new(64);
        let wide = draw(&mut list, [10.0, 20.0], 40.0, KYBER);
        assert!((wide - 16.8).abs() < 1e-4);
        assert_eq!(list.len(), COMMANDS);
        let commands = list.commands();
        assert!(matches!(commands[0], DrawCommand::PushTransform(_)));
        assert!(matches!(commands[COMMANDS - 1], DrawCommand::PopTransform));
        // The transform keeps the middle where it is and stretches the square upright:
        // its points land on the mark's top and foot.
        let DrawCommand::PushTransform(m) = commands[0] else {
            unreachable!()
        };
        let map = |[x, y]: [f32; 2]| {
            let w = m[6] * x + m[7] * y + m[8];
            [
                (m[0] * x + m[1] * y + m[2]) / w,
                (m[3] * x + m[4] * y + m[5]) / w,
            ]
        };
        let half = wide * 0.5;
        let [cx, cy] = [10.0 + half, 40.0];
        let top = map([cx, cy - half]);
        let foot = map([cx, cy + half]);
        assert!((top[1] - 20.0).abs() < 1e-3 && (foot[1] - 60.0).abs() < 1e-3);
        assert!((map([cx - half, cy])[0] - 10.0).abs() < 1e-3);
    }
}
