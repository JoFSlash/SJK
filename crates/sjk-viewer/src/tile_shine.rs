//! A finish's light over any rounded tile: a card, and any other tile that wants one (a
//! saber shader's, a medal's). [`push`] lays Holo's rainbow foil or Solar's corona over
//! the tile, shaded per pixel by the renderer ([`CardShine`]), its light sweeping across
//! while the tile rests and following the pointer while it rests on the tile, with a soft
//! glare under it. [`HoverEase`] eases the pointer's hold on each tile in and out, so the
//! light and any tilt glide to the pointer and back instead of jumping.

use sjk_ui::{CardShine, DrawCommand, DrawList, Rect, ShineKind};

/// The resting sweep: seconds from one pass to the next, and the share of them it takes
/// to cross the tile (the rest, it waits off the tile).
const SWEEP_PERIOD: f32 = 7.0;
const SWEEP_CROSSING: f32 = 0.6;
/// The finishes' clock wraps after this many seconds, so its motion keeps its precision
/// on a page left open.
const CLOCK: f32 = 3_600.0;
/// How quickly a hover eases, per second: it settles in about a third of a second.
const RATE: f32 = 9.0;
/// A tile not drawn for this many seconds is forgotten (scrolled away, page closed).
const FORGET: f32 = 1.0;

/// The pointer on a tile, eased: where it rests, -1 at the tile's left (top) edge to 1 at
/// its right (bottom) edge, and how firmly the tile answers it, from 0 (not yet, or gone)
/// to 1.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Hover {
    pub(crate) at: [f32; 2],
    pub(crate) hold: f32,
}

impl Hover {
    /// The pointer held fully at `at`.
    #[cfg(test)]
    pub(crate) fn held(at: [f32; 2]) -> Self {
        Self { at, hold: 1.0 }
    }

    /// Where the light falls, a share of the tile's width and height from its top left.
    pub(crate) fn light(self) -> [f32; 2] {
        self.at.map(|at| 0.5 + 0.5 * at.clamp(-1.0, 1.0))
    }

    /// How far a tile leans towards the pointer, -1 to 1 each way: the pointer's place
    /// as far as it holds.
    pub(crate) fn lean(self) -> [f32; 2] {
        let hold = self.hold.clamp(0.0, 1.0);
        self.at.map(|at| at.clamp(-1.0, 1.0) * hold)
    }
}

/// One tile's hover as it eases.
#[derive(Clone, Copy, Debug)]
struct Easing {
    key: usize,
    target: [f32; 2],
    value: [f32; 2],
    hold: f32,
    over: bool,
    /// The page's time when it was last drawn.
    at: f32,
}

/// The hover of each tile on a page, keyed by the page's own numbers: eased in while the
/// pointer rests on a tile, eased out after it leaves, so the tile left settles while the
/// next one rises.
#[derive(Clone, Debug, Default)]
pub(crate) struct HoverEase {
    tiles: Vec<Easing>,
}

impl HoverEase {
    /// Tile `key` at `rect` (window pixels, as laid out) with the pointer at `pointer`, at
    /// `seconds`: its hover, `None` once it has settled with the pointer away.
    pub(crate) fn update(
        &mut self,
        key: usize,
        rect: Rect,
        pointer: Option<[f32; 2]>,
        seconds: f32,
    ) -> Option<Hover> {
        self.tiles
            .retain(|tile| tile.key == key || seconds - tile.at < FORGET);
        let over = pointer
            .filter(|[px, py]| {
                *px >= rect.x && *px <= rect.right() && *py >= rect.y && *py <= rect.bottom()
            })
            .map(|[px, py]| {
                [
                    (px - rect.x - rect.width * 0.5) / (rect.width * 0.5).max(1.0),
                    (py - rect.y - rect.height * 0.5) / (rect.height * 0.5).max(1.0),
                ]
            });
        let index = match self.tiles.iter().position(|tile| tile.key == key) {
            Some(index) => index,
            None => {
                // A new hover starts at the pointer, holding nothing yet.
                let at = over?;
                self.tiles.push(Easing {
                    key,
                    target: at,
                    value: at,
                    hold: 0.0,
                    over: true,
                    at: seconds,
                });
                self.tiles.len() - 1
            }
        };
        let tile = &mut self.tiles[index];
        tile.over = over.is_some();
        if let Some(at) = over {
            tile.target = at;
        }
        let step = 1.0 - (-(seconds - tile.at).max(0.0) * RATE).exp();
        tile.at = seconds;
        for axis in 0..2 {
            tile.value[axis] += (tile.target[axis] - tile.value[axis]) * step;
        }
        let goal = if tile.over { 1.0 } else { 0.0 };
        tile.hold += (goal - tile.hold) * step;
        if !tile.over && tile.hold < 0.01 {
            self.tiles.swap_remove(index);
            return None;
        }
        Some(Hover {
            at: tile.value,
            hold: tile.hold,
        })
    }

    /// Hold tile `key` fully at `at`, as though the pointer had rested there a while, for
    /// a world shot.
    #[cfg(test)]
    pub(crate) fn rest(&mut self, key: usize, at: [f32; 2], seconds: f32) {
        self.tiles.retain(|tile| tile.key != key);
        self.tiles.push(Easing {
            key,
            target: at,
            value: at,
            hold: 1.0,
            over: true,
            at: seconds,
        });
    }
}

/// Lays `kind`'s light (`None` for a plain tile) and the pointer's glare over the tile at
/// `rect` (window pixels, inside any transform and clip already pushed) with corners of
/// `radius` pixels, at `seconds`. While `hover` holds, the light glides from its sweep to
/// the pointer.
pub(crate) fn push(
    list: &mut DrawList,
    rect: Rect,
    radius: f32,
    kind: Option<ShineKind>,
    hover: Option<Hover>,
    seconds: f32,
) {
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }
    let hold = hover.map_or(0.0, |hover| hover.hold.clamp(0.0, 1.0));
    let size = [rect.width, rect.height];
    let time = seconds.rem_euclid(CLOCK);
    if let Some(kind) = kind {
        let resting = [sweep(kind, size, seconds), 0.5];
        let light = hover.map_or(resting, |hover| {
            let [across, down] = hover.light();
            [
                resting[0] + (across - resting[0]) * hold,
                resting[1] + (down - resting[1]) * hold,
            ]
        });
        let _ = list.push(DrawCommand::CardShine {
            rect,
            shine: CardShine {
                kind,
                size,
                radius,
                time,
                light,
                hold,
                alpha: 1.0,
            },
        });
    }
    if let Some(hover) = hover.filter(|_| hold > 0.0) {
        let _ = list.push(DrawCommand::CardShine {
            rect,
            shine: CardShine {
                kind: ShineKind::Glare,
                size,
                radius,
                time,
                light: hover.light(),
                hold,
                alpha: 1.0,
            },
        });
    }
}

/// Where `kind`'s resting light crosses a tile of `size` pixels at `seconds`, a share of
/// its width: from off its left to off its right over the first [`SWEEP_CROSSING`] of
/// each pass, eased, then off its right until the next.
fn sweep(kind: ShineKind, [w, h]: [f32; 2], seconds: f32) -> f32 {
    // Off the tile: far enough that the light's whole slant has left it.
    let (period, off) = match kind {
        ShineKind::Holo => (SWEEP_PERIOD, h * 0.42 + w * 0.9),
        ShineKind::Solar => (SWEEP_PERIOD * 1.4, h * 0.35 + w * 0.4),
        ShineKind::Glare => return 0.5,
    };
    let off = off / w.max(1.0);
    let (from, to) = (-off, 1.0 + off);
    let phase = (seconds / period).fract() / SWEEP_CROSSING;
    if phase >= 1.0 {
        return to;
    }
    // Eased in and out, so it does not start or stop all at once.
    let eased = phase * phase * (3.0 - 2.0 * phase);
    from + (to - from) * eased
}

#[cfg(test)]
mod tests {
    use super::*;

    const TILE: Rect = Rect {
        x: 100.0,
        y: 100.0,
        width: 200.0,
        height: 280.0,
    };

    /// The pointer resting on a tile: its hold rises smoothly to 1, never jumping; once
    /// it leaves, the hold falls smoothly back and the tile lets go.
    #[test]
    fn a_hover_eases_in_and_out() {
        let mut ease = HoverEase::default();
        let pointer = Some([250.0, 240.0]);
        let first = ease.update(1, TILE, pointer, 0.0).unwrap();
        assert_eq!(first.hold, 0.0, "nothing held on the first frame");
        let mut last = 0.0;
        for frame in 1..=60 {
            let hover = ease.update(1, TILE, pointer, frame as f32 / 60.0).unwrap();
            assert!(
                hover.hold > last && hover.hold - last < 0.15,
                "{frame}: {last} -> {hover:?}"
            );
            last = hover.hold;
        }
        assert!(last > 0.99);
        let mut frame = 61;
        while let Some(hover) = ease.update(1, TILE, None, frame as f32 / 60.0) {
            assert!(hover.hold < last, "{frame}");
            last = hover.hold;
            frame += 1;
            assert!(frame < 240, "never let go");
        }
        assert!(frame > 70, "let go at once");
    }

    /// Moving from one tile to the next, the first settles back while the second rises.
    #[test]
    fn the_tile_left_settles_while_the_next_rises() {
        let mut ease = HoverEase::default();
        let next = Rect { x: 320.0, ..TILE };
        for frame in 0..30 {
            let _ = ease.update(1, TILE, Some([200.0, 240.0]), frame as f32 / 60.0);
        }
        let seconds = 31.0 / 60.0;
        let left = ease.update(1, TILE, Some([400.0, 240.0]), seconds).unwrap();
        let rising = ease.update(2, next, Some([400.0, 240.0]), seconds).unwrap();
        assert!(left.hold > 0.5, "the first still leans: {left:?}");
        assert_eq!(rising.hold, 0.0);
    }

    /// A tile that stops being drawn is forgotten.
    #[test]
    fn a_tile_no_longer_drawn_is_forgotten() {
        let mut ease = HoverEase::default();
        let _ = ease.update(1, TILE, Some([200.0, 240.0]), 0.0);
        let _ = ease.update(2, TILE, None, 5.0);
        assert!(ease.tiles.is_empty());
    }

    /// Lean is the pointer's place as far as it holds; light is the pointer's place.
    #[test]
    fn lean_grows_with_the_hold_and_light_stays_at_the_pointer() {
        let hover = Hover {
            at: [1.0, -0.5],
            hold: 0.5,
        };
        assert_eq!(hover.lean(), [0.5, -0.25]);
        assert_eq!(hover.light(), [1.0, 0.25]);
    }

    /// The sweep crosses from off one side to off the other, eased, then waits.
    #[test]
    fn the_sweep_crosses_then_waits() {
        let size = [200.0, 280.0];
        let start = sweep(ShineKind::Holo, size, 0.0);
        assert!(start < -0.5);
        let halfway = sweep(ShineKind::Holo, size, SWEEP_PERIOD * SWEEP_CROSSING * 0.5);
        assert!((halfway - 0.5).abs() < 1e-3, "{halfway}");
        let waiting = sweep(ShineKind::Holo, size, SWEEP_PERIOD * 0.9);
        assert!(waiting > 1.5);
        assert_eq!(
            sweep(ShineKind::Holo, size, SWEEP_PERIOD),
            start,
            "the next pass"
        );
    }

    /// A plain tile with the pointer on it takes the glare alone; a finished one its
    /// light too; a resting one, no glare. The light glides: half held, it lies halfway
    /// between its sweep and the pointer.
    #[test]
    fn push_lays_the_light_and_the_glare() {
        let shines = |kind, hover| {
            let mut list = DrawList::new(8);
            push(&mut list, TILE, 12.0, kind, hover, 1.0);
            list.commands()
                .iter()
                .filter_map(|command| match command {
                    DrawCommand::CardShine { shine, .. } => Some(*shine),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        let held = Some(Hover::held([0.0, 0.0]));
        assert_eq!(shines(None, None).len(), 0);
        assert_eq!(shines(None, held).len(), 1);
        assert_eq!(shines(Some(ShineKind::Solar), None).len(), 1);
        let both = shines(Some(ShineKind::Holo), held);
        assert_eq!(both[0].kind, ShineKind::Holo);
        assert_eq!(both[1].kind, ShineKind::Glare);
        let resting = shines(Some(ShineKind::Holo), None)[0].light[0];
        let half = shines(
            Some(ShineKind::Holo),
            Some(Hover {
                at: [0.0, 0.0],
                hold: 0.5,
            }),
        )[0]
        .light[0];
        assert!(
            (half - (resting + 0.5) * 0.5).abs() < 1e-4,
            "{resting} {half}"
        );
    }
}
