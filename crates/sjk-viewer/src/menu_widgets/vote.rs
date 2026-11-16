//! Shared vote card geometry for the HUD and headless evidence.

use sjk_ui::Rect;

/// One vote's card, sized in hero design pixels (1080 lines).
pub(crate) struct VoteLayout {
    /// The dark plate behind the card.
    pub(crate) plate: Rect,
    /// Time left, as a thin bar along the top.
    pub(crate) time: Rect,
    /// Vote kind at the left, seconds left at the right.
    pub(crate) label: Rect,
    /// The proposal, centred.
    pub(crate) proposal: Rect,
    /// YES / NO share of the votes cast.
    pub(crate) split: Rect,
    /// YES count at the left, NO count at the right.
    pub(crate) counts: Rect,
    /// The keys for YES at the left and NO at the right.
    pub(crate) keys: Rect,
}

impl VoteLayout {
    /// Card width in design pixels.
    pub(crate) const WIDTH: f32 = 560.0;
    /// Card height in design pixels.
    pub(crate) const HEIGHT: f32 = 186.0;

    /// Lay out a card whose top left is `rect`'s, `rect.width` wide.
    pub(crate) fn new(rect: Rect, scale: f32) -> Self {
        let pad = 18.0 * scale;
        let x = rect.x + pad;
        let width = rect.width - 2.0 * pad;
        let row = |top: f32, height: f32| Rect::new(x, rect.y + top * scale, width, height * scale);
        Self {
            plate: Rect::new(rect.x, rect.y, rect.width, Self::HEIGHT * scale),
            time: row(12.0, 4.0),
            label: row(22.0, 28.0),
            proposal: row(54.0, 40.0),
            split: row(104.0, 8.0),
            counts: row(118.0, 30.0),
            keys: row(150.0, 22.0),
        }
    }
}

/// A voted card folded to one row: the YES count, the proposal (an `ampoll` question)
/// over the YES / NO bar, and the NO count.
pub(crate) struct VoteRow {
    /// The dark plate behind the row.
    pub(crate) plate: Rect,
    /// Time left, as a thin bar along the top.
    pub(crate) time: Rect,
    /// The proposal, centred over the bar.
    pub(crate) question: Rect,
    /// The YES / NO share of the votes cast, between the counts.
    pub(crate) split: Rect,
    /// YES count at the left, NO count at the right.
    pub(crate) counts: Rect,
}

impl VoteRow {
    /// Row height in design pixels.
    pub(crate) const HEIGHT: f32 = 64.0;
    /// Room for each count beside the bar, in design pixels.
    const COUNT: f32 = 96.0;

    /// Lay out a row whose top left is `rect`'s, `rect.width` wide.
    pub(crate) fn new(rect: Rect, scale: f32) -> Self {
        let pad = 18.0 * scale;
        let x = rect.x + pad;
        let width = rect.width - 2.0 * pad;
        let count = Self::COUNT * scale;
        Self {
            plate: Rect::new(rect.x, rect.y, rect.width, Self::HEIGHT * scale),
            time: Rect::new(x, rect.y + 8.0 * scale, width, 3.0 * scale),
            question: Rect::new(
                x + count,
                rect.y + 16.0 * scale,
                width - 2.0 * count,
                24.0 * scale,
            ),
            split: Rect::new(
                x + count,
                rect.y + 44.0 * scale,
                width - 2.0 * count,
                8.0 * scale,
            ),
            counts: Rect::new(x, rect.y + 22.0 * scale, width, 28.0 * scale),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_stay_inside_the_plate_in_order() {
        let layout = VoteLayout::new(Rect::new(100.0, 40.0, VoteLayout::WIDTH, 0.0), 1.0);
        let rows = [
            layout.time,
            layout.label,
            layout.proposal,
            layout.split,
            layout.counts,
            layout.keys,
        ];
        let bottom = layout.plate.y + layout.plate.height;
        for pair in rows.windows(2) {
            assert!(pair[0].y + pair[0].height <= pair[1].y);
        }
        for row in rows {
            assert!(row.x >= layout.plate.x && row.y >= layout.plate.y);
            assert!(row.x + row.width <= layout.plate.x + layout.plate.width);
            assert!(row.y + row.height <= bottom);
        }
    }

    #[test]
    fn the_voted_row_keeps_its_bar_between_the_counts() {
        let row = VoteRow::new(Rect::new(100.0, 40.0, VoteLayout::WIDTH, 0.0), 1.0);
        assert!(row.plate.height < VoteLayout::HEIGHT / 2.0);
        assert!(row.question.y + row.question.height <= row.split.y);
        for inner in [row.time, row.question, row.split, row.counts] {
            assert!(inner.x >= row.plate.x && inner.y >= row.plate.y);
            assert!(inner.x + inner.width <= row.plate.x + row.plate.width);
            assert!(inner.y + inner.height <= row.plate.y + row.plate.height);
        }
        assert!(row.split.x > row.counts.x);
        assert!(row.split.x + row.split.width < row.counts.x + row.counts.width);
    }
}
