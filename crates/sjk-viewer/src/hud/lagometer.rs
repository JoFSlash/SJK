//! The lagometer (`cg_lagometer`) and JoF EJK's modes 2 and 3 of it.
//!
//! Mode 1 is the graph SJK has always drawn: frame interpolation (yellow above the
//! middle line, blue below) and snapshot latency (green, yellow when the server held
//! the snapshot back for rate, red when one was dropped) over a backdrop, where
//! `cg_lagometerX`, `cg_lagometerY` and `cg_lagometerScale` put it
//! ([`super::targeting::Policy::lagometer_rect`]).
//!
//! JoF EJK (`cg_draw.c`, `CG_DrawLagometer` and `CG_DrawSpeedGraph`) adds two modes:
//! 2 writes the average ping and interpolation of the graph's 48 columns along its top
//! (SJK: just above it), and draws a speed graph above it (56 of the graph's 48 units
//! up) with the current speed at its top left; 3 does the same without the backdrops,
//! the speed graph three times as tall, its number only while moving. EJK draws the speed graph only with
//! its `cg_speedometer` speed graph bit; SJK has no such bit and draws it with the
//! modes (Sol's decision, 29/10/2026). The graphs are in units of the graph's own height
//! over 48, EJK's size, at any `cg_lagometerScale`.
//!
//! The numbers are not: SJK's graph is 45 pixels at 1080 lines with the default HUD scale
//! where EJK's is 108, so at EJK's proportion they were 10 pixels tall. They are drawn at
//! EJK's own size instead ([`TEXT_LINE`]: digits about 13 pixels tall at 1080p, as the
//! HUD's "Score" capitals), grown by the HUD scale and `cg_lagometerScale`, and the average
//! ping and interpolation stand on a line just above the graph rather than over its top,
//! wider than the graph and centred on it, so the graph keeps its size (Sol's decision,
//! 29/10/2026). The speed graph rises as far as that line needs ([`places`]).

use super::*;
use sjk_ui::{Color, DrawCommand, DrawList, FontWeight, HudWidget, Rect, TextAlign, TextOverflow};
use std::fmt::Write as _;

/// The setting: 0 off, 1 the graph, 2 with numbers and the speed graph, 3 the same
/// without backdrops.
pub(crate) const CVAR: &str = "cg_lagometer";
/// The highest mode; a larger value draws it.
pub(crate) const MAX_MODE: u8 = 3;
/// The text of the average ping, the average interpolation and the speed.
pub(super) const PING_TEXT: TextId = TextId(325);
pub(super) const INTERPOLATION_TEXT: TextId = TextId(326);
pub(super) const SPEED_TEXT: TextId = TextId(327);
/// EJK's graph is 48 virtual units square, and its averages read its 48 columns (`aw`).
const UNITS: f32 = 48.0;
const AVERAGED: usize = 48;
/// Speed samples kept: as many as the widest graph draws columns.
const SPEED_SAMPLES: usize = 64;
/// `MAX_LAGOMETER_RANGE` and `MAX_LAGOMETER_PING`: the milliseconds the two graphs span.
const FRAME_RANGE: f32 = 300.0;
const PING_RANGE: f32 = 900.0;
/// `CG_DrawSpeedGraph`'s `vscale = range / 3000`: the speed the graph's height spans.
const SPEED_RANGE: f32 = 3_000.0;
/// Where the speed graph stands: 56 units above the lagometer, 48 tall; in mode 3,
/// 96 more up and three times as tall, so its bottom stays.
const SPEED_GAP: f32 = 56.0;
const TALL_SPEED_RISE: f32 = 96.0;
/// The numbers' line box at 1080 lines and `cg_lagometerScale 1`, in the HUD's logical
/// pixels (before `cg_hudScale`, 0.7 by default): with the default HUD scale a 23.8-pixel
/// line, EJK's at 1080p (`CG_Text_Paint(..., 0.5f, ..., FONT_SMALL)`, `ocr_a`'s 21-unit
/// line at half scale, 10.5 of 480 lines: 23.6 pixels). `ocr_a`'s digits (SJK Chat with
/// `ui_gameFont`) fill 11.4 of its 21-pixel line, so they stand about 13 pixels tall, as
/// the HUD's "Score" capitals (Sol's request, 29/10/2026).
pub(super) const TEXT_LINE: f32 = 34.0;
/// The space between the numbers' line and the graphs, in the same pixels.
pub(super) const TEXT_GAP: f32 = 2.0;
/// The narrowest the numbers' line is, in lines: a three-digit ping and an interpolation
/// such as "-12.5" side by side with a digit's room between them, wider than the graph.
const ROW_LINES: f32 = 4.7;
const YELLOW: Color = Color::new(1.0, 0.82, 0.20, 0.9);
const BLUE: Color = Color::new(0.25, 0.55, 1.0, 0.9);
const GREEN: Color = Color::new(0.28, 0.95, 0.48, 0.9);
const RED: Color = Color::new(1.0, 0.22, 0.18, 0.9);
const WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);

/// `cg_lagometer` as the HUD draws it, 0 to [`MAX_MODE`]; 0 when unset.
pub(crate) fn mode(console: Option<&ViewerConsole>) -> u8 {
    console
        .and_then(|console| console.integer_cvar(CVAR))
        .map_or(0, |mode| mode.clamp(0, i64::from(MAX_MODE)) as u8)
}

/// What modes 2 and 3 add: the numbers' retained text and the speed history.
pub(crate) struct Readout {
    pub(super) ping: String,
    pub(super) interpolation: String,
    pub(super) speed: String,
    /// The current speed is not 0, for mode 3's speed number.
    moving: bool,
    /// Horizontal speed, one sample a frame (`CG_AddSpeedGraphFrameInfo`).
    speeds: [f32; SPEED_SAMPLES],
    speed_count: u64,
}

impl Default for Readout {
    fn default() -> Self {
        Self {
            ping: String::with_capacity(16),
            interpolation: String::with_capacity(16),
            speed: String::with_capacity(16),
            moving: false,
            speeds: [0.0; SPEED_SAMPLES],
            speed_count: 0,
        }
    }
}

impl Readout {
    /// Record this frame's speed: `cg.currentSpeed`, the horizontal part of the
    /// (predicted) velocity, as the graph and EJK's number read it.
    pub(super) fn add_speed(&mut self, velocity: [f32; 3]) {
        let speed = velocity[0].hypot(velocity[1]);
        let speed = if speed.is_finite() { speed } else { 0.0 };
        self.speeds[self.speed_count as usize % SPEED_SAMPLES] = speed;
        self.speed_count += 1;
    }

    /// A speed sample, where offset zero is the newest; 0 before there was one.
    fn speed_newest(&self, offset: usize) -> f32 {
        if offset >= SPEED_SAMPLES || offset as u64 >= self.speed_count {
            return 0.0;
        }
        self.speeds[(self.speed_count - 1 - offset as u64) as usize % SPEED_SAMPLES]
    }

    /// Write the numbers `mode` shows from `samples` and the newest speed, into the
    /// retained text; modes 0 and 1 show none.
    pub(super) fn update(&mut self, samples: &sjk_client::LagometerSamples, mode: u8) {
        self.ping.clear();
        self.interpolation.clear();
        self.speed.clear();
        let speed = self.speed_newest(0);
        self.moving = speed != 0.0;
        if mode < 2 {
            return;
        }
        let (ping, interpolation) = averages(samples);
        let _ = write!(self.ping, "{ping:.0}");
        let _ = write!(self.interpolation, "{interpolation:04.1}");
        let _ = write!(self.speed, "{speed:.0}");
    }
}

/// `CG_DrawLagometer`'s two averages over the newest 48 columns: the ping, a dropped
/// snapshot counting as the highest ping seen so far in the walk (1 at least), and
/// the interpolation, the frames' offset from the latest snapshot negated (how far
/// behind it the view is drawn, in milliseconds). A column with no sample yet counts
/// as 0, as EJK's zeroed arrays do.
pub(super) fn averages(samples: &sjk_client::LagometerSamples) -> (f32, f32) {
    let mut interpolation = 0.0;
    for offset in 0..AVERAGED {
        interpolation += samples.frame_newest(offset) as f32;
    }
    let mut ping = 0.0;
    let mut highest = 1;
    for offset in 0..AVERAGED {
        let sample = samples.snapshot_newest(offset).ping;
        highest = highest.max(sample);
        if sample > 0 {
            ping += sample as f32;
        } else if sample < 0 {
            ping += highest as f32;
        }
    }
    (ping / AVERAGED as f32, -(interpolation / AVERAGED as f32))
}

/// What one frame's lagometer draws: the samples, the readout and the mode.
pub(super) struct Graph<'a> {
    pub(super) samples: &'a sjk_client::LagometerSamples,
    pub(super) readout: &'a Readout,
    pub(super) mode: u8,
    /// Columns drawn, whatever the graph's size.
    pub(super) columns: usize,
    /// The numbers' scale: the HUD's (the screen's height over 1080, with `cg_hudScale`)
    /// times `cg_lagometerScale`.
    pub(super) text_scale: f32,
    /// The screen's size in pixels, which the numbers stay inside.
    pub(super) screen: [f32; 2],
}

/// Where modes 2 and 3 put the numbers and the speed graph around the graph.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Places {
    /// The average ping's (from its left) and interpolation's (to its right) line: just
    /// above the graph, or just below it when the graph is too near the screen's top. As
    /// wide as the graph, or [`ROW_LINES`] lines when that is wider, centred on it and
    /// inside the screen.
    pub(super) numbers: Rect,
    /// The speed graph: EJK's 56 units above the graph, higher when the numbers' line
    /// needs the room; in mode 3 three times as tall, rising.
    pub(super) speed: Rect,
    /// The speed number's line, at the top left of the speed graph's lowest 48 units
    /// (its top in mode 2), where EJK writes it.
    pub(super) speed_number: Rect,
}

/// Where the numbers and the speed graph stand around the graph in `rect` for `mode`,
/// with the numbers `text_scale` times [`TEXT_LINE`] tall on a `screen`-sized screen.
pub(super) fn places(rect: Rect, mode: u8, text_scale: f32, screen: [f32; 2]) -> Places {
    let unit = rect.height / UNITS;
    let line = TEXT_LINE * text_scale;
    let gap = TEXT_GAP * text_scale;
    let width = rect
        .width
        .max(ROW_LINES * line)
        .min(screen[0].max(rect.width));
    let x = (rect.x + (rect.width - width) * 0.5).clamp(0.0, (screen[0] - width).max(0.0));
    let above = rect.y - gap - line;
    let fits_above = above >= 0.0;
    let numbers = Rect::new(
        x,
        if fits_above {
            above
        } else {
            rect.bottom() + gap
        },
        width,
        line,
    );
    // EJK leaves 8 units between the graph and the speed graph; the numbers' line and a
    // gap on each side of it may need more.
    let room = if fits_above { line + 2.0 * gap } else { 0.0 };
    let lift = (room - (SPEED_GAP - UNITS) * unit).max(0.0);
    let top = rect.y - SPEED_GAP * unit - lift;
    let speed = if mode >= 3 {
        Rect::new(
            rect.x,
            top - TALL_SPEED_RISE * unit,
            rect.width,
            rect.height * 3.0,
        )
    } else {
        Rect::new(rect.x, top, rect.width, rect.height)
    };
    let speed_x = rect.x + 2.0 * unit;
    let speed_number = Rect::new(
        speed_x,
        top,
        (width - 2.0 * unit).min(screen[0] - speed_x).max(0.0),
        line,
    );
    Places {
        numbers,
        speed,
        speed_number,
    }
}

/// Draw the lagometer in `rect` (already placed and scaled), with what `graph.mode`
/// adds; `panel` draws a backdrop.
pub(super) fn emit(
    draw_list: &mut DrawList,
    rect: Rect,
    graph: &Graph<'_>,
    panel: &mut dyn FnMut(&mut DrawList, Rect),
) {
    let backdrop = graph.mode < 3;
    if backdrop {
        panel(draw_list, rect);
    }
    emit_lag(draw_list, rect, graph);
    if graph.mode < 2 {
        return;
    }
    let places = places(rect, graph.mode, graph.text_scale, graph.screen);
    // The averages on their line: the ping from its left, the interpolation ending at
    // its right (EJK's `ax + 3` and `ax + aw - strW`).
    let numbers = places.numbers;
    let line = numbers.height;
    let unit = rect.height / UNITS;
    text(
        draw_list,
        Rect::new(
            numbers.x + 2.0 * unit,
            numbers.y,
            numbers.width - 2.0 * unit,
            line,
        ),
        PING_TEXT,
        line,
        TextAlign::Start,
    );
    text(
        draw_list,
        Rect::new(numbers.x, numbers.y, numbers.width - unit, line),
        INTERPOLATION_TEXT,
        line,
        TextAlign::End,
    );
    if backdrop {
        panel(draw_list, places.speed);
    }
    emit_speed(draw_list, places.speed, graph);
    if graph.mode == 2 || graph.readout.moving {
        text(
            draw_list,
            places.speed_number,
            SPEED_TEXT,
            line,
            TextAlign::Start,
        );
    }
}

fn text(draw_list: &mut DrawList, rect: Rect, id: TextId, size: f32, align: TextAlign) {
    let _ = draw_list.push(DrawCommand::Text {
        rect,
        text: id,
        size,
        color: WHITE,
        align,
        overflow: TextOverflow::Clip,
        weight: FontWeight::Regular,
        letter_spacing: 0.0,
    });
}

/// The interpolation and latency bars: mode 1's graph, unchanged.
fn emit_lag(draw_list: &mut DrawList, rect: Rect, graph: &Graph<'_>) {
    let width = graph.columns;
    let bar_width = rect.width / width as f32;
    let frame_range = rect.height / 3.0;
    let middle = rect.y + frame_range;
    let ping_range = rect.height / 2.0;
    for offset in 0..width {
        let x = rect.right() - (offset + 1) as f32 * bar_width;
        let frame = graph.samples.frame_newest(offset) as f32;
        let height = (frame.abs() * frame_range / FRAME_RANGE).min(frame_range);
        if height > 0.0 {
            let (y, color) = if frame > 0.0 {
                (middle - height, YELLOW)
            } else {
                (middle, BLUE)
            };
            let _ = draw_list.push(DrawCommand::SolidRect {
                rect: Rect::new(x, y, bar_width, height),
                color,
            });
        }
        let sample = graph.samples.snapshot_newest(offset);
        let (height, color) = if sample.ping < 0 {
            (ping_range, RED)
        } else {
            let color = if sample.flags & 1 != 0 { YELLOW } else { GREEN };
            (
                (sample.ping as f32 * ping_range / PING_RANGE).min(ping_range),
                color,
            )
        };
        if height > 0.0 {
            let _ = draw_list.push(DrawCommand::SolidRect {
                rect: Rect::new(x, rect.bottom() - height, bar_width, height),
                color,
            });
        }
    }
}

/// `CG_DrawSpeedGraph`: a green column per frame from the bottom, the height's
/// worth 3000 units a second.
fn emit_speed(draw_list: &mut DrawList, rect: Rect, graph: &Graph<'_>) {
    let width = graph.columns;
    let bar_width = rect.width / width as f32;
    for offset in 0..width {
        let speed = graph.readout.speed_newest(offset);
        let height = (speed * rect.height / SPEED_RANGE).min(rect.height);
        if height > 0.0 {
            let _ = draw_list.push(DrawCommand::SolidRect {
                rect: Rect::new(
                    rect.right() - (offset + 1) as f32 * bar_width,
                    rect.bottom() - height,
                    bar_width,
                    height,
                ),
                color: GREEN,
            });
        }
    }
}

/// The backdrop the HUD layout gives the lagometer's widget.
pub(super) fn panel<'a>(
    theme: Theme,
    widget: &'a HudWidget,
) -> impl FnMut(&mut DrawList, Rect) + 'a {
    move |draw_list, rect| {
        super::widgets::emit_panel(
            draw_list,
            theme,
            widget,
            rect,
            widget.style.radius.unwrap_or(theme.radii.sm),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn samples() -> sjk_client::LagometerSamples {
        let mut samples = sjk_client::LagometerSamples::new();
        for sample in 0..64 {
            samples.add_frame(-20 - sample % 10);
            samples.add_snapshot(60 + sample % 5, 0);
        }
        samples
    }

    /// A 1080p screen.
    const SCREEN: [f32; 2] = [1920.0, 1080.0];

    fn drawn(mode: u8, readout: &Readout) -> Vec<DrawCommand> {
        drawn_at(mode, readout, Rect::new(500.0, 300.0, 96.0, 96.0), 1.0)
    }

    /// What `mode` draws for a graph in `rect`, the numbers at `text_scale`.
    fn drawn_at(mode: u8, readout: &Readout, rect: Rect, text_scale: f32) -> Vec<DrawCommand> {
        let samples = samples();
        let mut draw_list = DrawList::new(512);
        let graph = Graph {
            samples: &samples,
            readout,
            mode,
            columns: 48,
            text_scale,
            screen: SCREEN,
        };
        emit(&mut draw_list, rect, &graph, &mut |draw_list, rect| {
            let _ = draw_list.push(DrawCommand::RoundedRect {
                rect,
                radius: 0.0,
                color: Color::new(0.0, 0.0, 0.0, 0.5),
            });
        });
        draw_list.commands().to_vec()
    }

    fn panels(commands: &[DrawCommand]) -> Vec<Rect> {
        commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::RoundedRect { rect, .. } => Some(*rect),
                _ => None,
            })
            .collect()
    }

    fn texts(commands: &[DrawCommand]) -> Vec<TextId> {
        texts_placed(commands)
            .into_iter()
            .map(|(text, _, _)| text)
            .collect()
    }

    /// Each text drawn, with its line box and size.
    fn texts_placed(commands: &[DrawCommand]) -> Vec<(TextId, Rect, f32)> {
        commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text {
                    text, rect, size, ..
                } => Some((*text, *rect, *size)),
                _ => None,
            })
            .collect()
    }

    fn overlaps(a: Rect, b: Rect) -> bool {
        a.x < b.right() && b.x < a.right() && a.y < b.bottom() && b.y < a.bottom()
    }

    fn moving(speed: f32, mode: u8) -> Readout {
        let mut readout = Readout::default();
        for _ in 0..64 {
            readout.add_speed([speed * 0.6, speed * 0.8, 250.0]);
        }
        readout.update(&samples(), mode);
        readout
    }

    #[test]
    fn the_mode_reads_clamped_and_old_configs_keep_the_graph() {
        let directory = tempfile::tempdir().unwrap();
        let mut console = ViewerConsole::new(directory.path().join("config.cfg")).unwrap();
        assert_eq!(mode(None), 0);
        assert_eq!(mode(Some(&console)), 0, "off by default");
        // `seta cg_lagometer "1"`, as old SJK and EJK configs save it.
        for (value, expected) in [("1", 1), ("2", 2), ("3", 3), ("9", 3), ("-1", 0), ("0", 0)] {
            assert!(console.set_cvar(CVAR, value), "{value}");
            assert_eq!(mode(Some(&console)), expected, "{value}");
        }
    }

    /// Mode 1 draws what SJK always drew: one backdrop and the bars, no text, nothing
    /// above the graph.
    #[test]
    fn mode_1_is_the_graph_alone() {
        let readout = moving(300.0, 1);
        let commands = drawn(1, &readout);
        assert_eq!(panels(&commands), [Rect::new(500.0, 300.0, 96.0, 96.0)]);
        assert!(texts(&commands).is_empty());
        assert!(readout.ping.is_empty() && readout.speed.is_empty());
        let solids = commands
            .iter()
            .filter(|command| matches!(command, DrawCommand::SolidRect { .. }))
            .collect::<Vec<_>>();
        // A frame and a latency bar per column, all inside the graph.
        assert_eq!(solids.len(), 96);
        for command in solids {
            let DrawCommand::SolidRect { rect, .. } = command else {
                unreachable!()
            };
            assert!(rect.y >= 300.0 && rect.bottom() <= 396.0, "{rect:?}");
        }
    }

    #[test]
    fn mode_2_adds_the_numbers_and_the_speed_graph_on_backdrops() {
        let readout = moving(300.0, 2);
        let commands = drawn(2, &readout);
        // The graph's backdrop, as in mode 1, and the speed graph's, 56 units (of 96 / 48)
        // above it and 22 pixels higher, so the numbers' 34-pixel line and its 2-pixel
        // gaps fit between the two (EJK's 8 units leave 16 pixels).
        assert_eq!(
            panels(&commands),
            [
                Rect::new(500.0, 300.0, 96.0, 96.0),
                Rect::new(500.0, 300.0 - 112.0 - 22.0, 96.0, 96.0)
            ]
        );
        assert_eq!(
            texts(&commands),
            [PING_TEXT, INTERPOLATION_TEXT, SPEED_TEXT]
        );
        // 300 a second fills a tenth of the speed graph's height.
        let speed_bars: Vec<Rect> = commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::SolidRect { rect, .. } if rect.bottom() <= 300.0 - 16.0 => Some(*rect),
                _ => None,
            })
            .collect();
        assert_eq!(speed_bars.len(), 48);
        assert!(speed_bars.iter().all(|bar| (bar.height - 9.6).abs() < 1e-3));
        assert_eq!(readout.speed, "300", "horizontal speed only");
    }

    #[test]
    fn mode_3_has_no_backdrop_a_tall_speed_graph_and_a_number_only_while_moving() {
        let readout = moving(300.0, 3);
        let commands = drawn(3, &readout);
        assert!(panels(&commands).is_empty());
        assert_eq!(
            texts(&commands),
            [PING_TEXT, INTERPOLATION_TEXT, SPEED_TEXT]
        );
        let graph = Rect::new(500.0, 300.0, 96.0, 96.0);
        let tall = places(graph, 3, 1.0, SCREEN).speed;
        assert_eq!(tall, Rect::new(500.0, 300.0 - 304.0 - 22.0, 96.0, 288.0));
        // Its bottom is mode 2's.
        let short = places(graph, 2, 1.0, SCREEN).speed;
        assert!((tall.bottom() - short.bottom()).abs() < 1e-3);
        let still = moving(0.0, 3);
        assert_eq!(texts(&drawn(3, &still)), [PING_TEXT, INTERPOLATION_TEXT]);
        assert_eq!(texts(&drawn(2, &moving(0.0, 2))).len(), 3, "mode 2 shows 0");
    }

    /// The numbers are EJK's size (a 34-pixel line at 1080 lines before `cg_hudScale`)
    /// grown by the HUD scale and `cg_lagometerScale`, not the graph's EJK proportion (10.5
    /// pixels on
    /// SJK's 48-pixel graph): the ping and interpolation on a line just above the graph,
    /// ping left and interpolation right, the speed number at the speed graph's top left.
    /// Nothing overlaps, and the graph itself keeps its size.
    #[test]
    fn the_numbers_are_readable_above_the_graph_at_every_scale() {
        for scale in [1.0, 2.0] {
            // SJK's graph: 48 pixels at 1080 lines, times `cg_lagometerScale`.
            let side = 48.0 * scale;
            let graph = Rect::new(1_920.0 - 324.0 - side, 1_080.0 - 324.0 - side, side, side);
            for mode in [2, 3] {
                let readout = moving(300.0, mode);
                let commands = drawn_at(mode, &readout, graph, scale);
                let placed = texts_placed(&commands);
                assert_eq!(placed.len(), 3, "mode {mode} at {scale}");
                let line = TEXT_LINE * scale;
                assert!(
                    placed.iter().all(|(_, rect, size)| *size == line
                        && rect.height == line
                        && rect.x >= 0.0
                        && rect.right() <= SCREEN[0]),
                    "mode {mode} at {scale}: {placed:?}"
                );
                let (ping, interpolation, speed) = (placed[0], placed[1], placed[2]);
                assert_eq!(
                    (ping.0, interpolation.0, speed.0),
                    (PING_TEXT, INTERPOLATION_TEXT, SPEED_TEXT)
                );
                // Just above the graph, its gap scaled too.
                let row = places(graph, mode, scale, SCREEN);
                assert_eq!(row.numbers.bottom(), graph.y - TEXT_GAP * scale);
                for text in [ping.1, interpolation.1] {
                    assert_eq!(text.y, row.numbers.y);
                    assert!(text.x >= row.numbers.x && text.right() <= row.numbers.right());
                    assert!(!overlaps(text, graph) && !overlaps(text, row.speed));
                }
                // Centred on the graph, wide enough for both numbers on its 48 pixels.
                assert!(row.numbers.width >= graph.width);
                assert!(
                    (row.numbers.x + row.numbers.width / 2.0 - (graph.x + graph.width / 2.0)).abs()
                        < 1e-3
                );
                assert!(!overlaps(row.speed, graph));
                assert!(row.speed.bottom() <= row.numbers.y - TEXT_GAP * scale + 1e-3);
                // The speed number inside the speed graph's lowest 48 units, at its left.
                assert!((speed.1.y - (row.speed.bottom() - side)).abs() < 1e-3);
                assert!(speed.1.x > row.speed.x && speed.1.x < row.speed.x + side / 4.0);
                if mode == 2 {
                    // The graph's backdrop is mode 1's: its size does not change.
                    assert_eq!(panels(&commands)[0], graph);
                    assert_eq!(panels(&commands)[1], row.speed);
                }
            }
        }
        // `cg_hudScale` or a larger screen grows them as the HUD's other text: 1440 lines
        // at `cg_hudScale 1`.
        let commands = drawn_at(
            2,
            &moving(300.0, 2),
            Rect::new(1_600.0, 900.0, 64.0, 64.0),
            1_440.0 / 1_080.0,
        );
        assert!(
            texts_placed(&commands)
                .iter()
                .all(|(_, _, size)| (size - TEXT_LINE * 4.0 / 3.0).abs() < 1e-3)
        );
    }

    /// At 1080 lines with the default `cg_hudScale` (0.7), SJK Chat's digits (the font of
    /// `ui_gameFont`, on by default) stand about 13 pixels tall on EJK's 23.6-pixel line,
    /// and the widest numbers likely, a three-digit ping and "-99.9", fit the line side by
    /// side with room between them, on the 45-pixel graph or wider.
    #[test]
    fn the_digits_are_about_13_pixels_and_both_numbers_fit_their_line() {
        let font = crate::text::retail_font::load(&crate::text::retail_font::CHAT)
            .unwrap()
            .font;
        let hud = 0.7;
        let line = TEXT_LINE * hud;
        assert!((line - 23.6).abs() < 0.5, "{line}");
        let scale = line / font.height;
        let digit = font.glyph(crate::text::TextFace::Regular, b'8').height * scale;
        assert!((12.5..=13.5).contains(&digit), "{digit}");
        let graph = Rect::new(1_700.0, 700.0, 64.0 * hud, 64.0 * hud);
        let row = places(graph, 2, hud, SCREEN).numbers;
        assert!(row.width > graph.width);
        let width = |text| crate::text::visible_text_width(&font, text, scale);
        let room = row.width - 3.0 * graph.height / UNITS;
        assert!(
            width("999") + width("-99.9") + width("0") <= room,
            "{} + {} in {room}",
            width("999"),
            width("-99.9")
        );
    }

    /// A graph at the screen's top edge has no room above it: the numbers stand just
    /// below it instead, inside the screen; one at the right edge keeps them inside too.
    #[test]
    fn at_the_screen_edges_the_numbers_stay_inside() {
        for mode in [2, 3] {
            let top = Rect::new(800.0, 10.0, 48.0, 48.0);
            let row = places(top, mode, 1.0, SCREEN);
            assert_eq!(row.numbers.y, top.bottom() + TEXT_GAP);
            assert!(!overlaps(row.numbers, top));
            let commands = drawn_at(mode, &moving(300.0, mode), top, 1.0);
            let placed = texts_placed(&commands);
            assert!(
                placed[..2]
                    .iter()
                    .all(|(_, rect, _)| rect.y >= 0.0 && rect.y == row.numbers.y)
            );
            // Barely room above: they still go above, at the very top.
            let just = Rect::new(800.0, TEXT_LINE + TEXT_GAP, 48.0, 48.0);
            assert_eq!(places(just, mode, 1.0, SCREEN).numbers.y, 0.0);
            // At the right edge, wider than the graph: shifted left to stay on screen.
            let right = Rect::new(SCREEN[0] - 48.0, 500.0, 48.0, 48.0);
            let row = places(right, mode, 1.0, SCREEN);
            assert!(row.numbers.width > right.width);
            assert_eq!(row.numbers.right(), SCREEN[0]);
            let left = Rect::new(0.0, 500.0, 48.0, 48.0);
            assert_eq!(places(left, mode, 1.0, SCREEN).numbers.x, 0.0);
        }
    }

    /// EJK's averages: the 48 newest columns, a dropped snapshot as the highest ping
    /// seen so far, the interpolation negated, written `%.0f` and `%04.1f`.
    #[test]
    fn the_averages_follow_ejk() {
        let mut samples = sjk_client::LagometerSamples::new();
        for _ in 0..47 {
            samples.add_frame(-30);
            samples.add_snapshot(50, 0);
        }
        samples.add_frame(-6);
        samples.add_dropped();
        // The dropped one is the newest: walked first, it counts as 1.
        let (ping, interpolation) = averages(&samples);
        assert!((ping - (47.0 * 50.0 + 1.0) / 48.0).abs() < 1e-3, "{ping}");
        assert!((interpolation - (47.0 * 30.0 + 6.0) / 48.0).abs() < 1e-3);
        samples.add_snapshot(80, 0);
        // Now 80 is seen before it.
        let (ping, _) = averages(&samples);
        assert!(
            (ping - (46.0 * 50.0 + 80.0 + 80.0) / 48.0).abs() < 1e-3,
            "{ping}"
        );
        // Fewer samples than columns count as 0, as EJK's zeroed arrays.
        let mut few = sjk_client::LagometerSamples::new();
        few.add_frame(-48);
        few.add_snapshot(96, 0);
        assert_eq!(averages(&few), (2.0, 1.0));
        let mut readout = Readout::default();
        readout.update(&few, 2);
        assert_eq!(
            (readout.ping.as_str(), readout.interpolation.as_str()),
            ("2", "01.0")
        );
        readout.update(&few, 1);
        assert!(readout.ping.is_empty() && readout.interpolation.is_empty());
    }
}
