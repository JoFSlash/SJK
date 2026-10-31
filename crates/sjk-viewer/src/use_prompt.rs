//! The Use prompt (Sol's idea, 30/10/2026): what the player's Use key does at what they
//! face, drawn to the right of the crosshair with a gap, as many games show it: a key cap
//! naming the key bound to `+use`, then the action and a line under it. A feature sets
//! the prompt each frame it applies ([`GpuState::want_use_prompt`]); the frame draws the
//! last one set and forgets it, so a prompt nobody sets again is gone the next frame.
//! The mystery boxes are its first user (`mystery_box`).

use crate::GpuState;
use sjk_ui::{Color, DrawCommand, DrawList, FontWeight, Rect, TextAlign, TextId, TextOverflow};

/// The gap between the crosshair's middle and the key cap, in pixels at 1080 lines (Sol,
/// 30/10/2026: 34 first, too close).
const GAP: f32 = 56.0;
/// The key cap's height and its key's size, and the action's and the line's sizes.
const CAP: f32 = 30.0;
const KEY_SIZE: f32 = 17.0;
const ACTION_SIZE: f32 = 21.0;
const LINE_SIZE: f32 = 15.0;
/// The widest the words beside the cap may run.
const WORDS_WIDTH: f32 = 520.0;
/// The command whose key the cap names.
const USE: &str = "+use";

/// What Use does at what the player faces.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Prompt {
    /// What pressing Use does ("Open mystery box").
    pub(crate) action: String,
    /// A line under it ("100 credits"), or none.
    pub(crate) line: Option<String>,
    /// Use does it now; false greys the cap (it waits, or cannot).
    pub(crate) ready: bool,
}

/// The prompt asked for this frame and its draw list.
pub(crate) struct UsePrompt {
    wanted: Option<Prompt>,
    list: DrawList,
    lines: Vec<String>,
}

impl UsePrompt {
    /// The prompt's key cap and words, for the frame's shape pass.
    pub(crate) fn list(&self) -> &DrawList {
        &self.list
    }
}

impl Default for UsePrompt {
    fn default() -> Self {
        Self {
            wanted: None,
            // The cap, its edge, and three lines over their shadows.
            list: DrawList::new(8),
            lines: Vec::new(),
        }
    }
}

/// The key cap's words for the keys bound to `+use` (the first one), "Use" when none is.
fn key_words(keys: &[String]) -> String {
    keys.first()
        .map(|key| crate::keybind_editor::sjk_view::key_label(key))
        .filter(|label| !label.is_empty())
        .unwrap_or_else(|| "Use".to_owned())
}

/// The key cap's width for `key`, in pixels at 1080 lines: square for one character,
/// wider for a word.
fn cap_width(key: &str) -> f32 {
    (12.0 + key.chars().count() as f32 * 11.0).max(CAP)
}

/// Where the cap and the words stand, by the crosshair's middle `at` (pixels), `unit`
/// pixels a 1080-line pixel, in a `viewport`: the cap's rectangle and the words' left
/// and top, kept on the screen.
fn layout(
    at: [f32; 2],
    key: &str,
    unit: f32,
    viewport: [f32; 2],
    two_lines: bool,
) -> (Rect, [f32; 2]) {
    let cap = Rect::new(
        (at[0] + GAP * unit)
            .min(viewport[0] - (cap_width(key) + 120.0) * unit)
            .max(0.0),
        at[1] - CAP * 0.5 * unit,
        cap_width(key) * unit,
        CAP * unit,
    );
    let words_top = if two_lines {
        at[1] - (ACTION_SIZE * 1.15) * unit
    } else {
        at[1] - ACTION_SIZE * 0.65 * unit
    };
    (cap, [cap.x + cap.width + 10.0 * unit, words_top])
}

impl GpuState {
    /// Show `prompt` by the crosshair this frame.
    pub(crate) fn want_use_prompt(&mut self, prompt: Prompt) {
        self.use_prompt.wanted = Some(prompt);
    }

    /// Lay out this frame's prompt by the crosshair and add its words to the HUD's text;
    /// nothing while `shown` is false (menus, console, scoreboard, intermission). The
    /// prompt asked for is forgotten either way.
    pub(crate) fn append_use_prompt(&mut self, viewport: [f32; 2], shown: bool) {
        let wanted = self.use_prompt.wanted.take();
        self.use_prompt.list.clear();
        self.use_prompt.lines.clear();
        let Some(prompt) = wanted.filter(|_| shown) else {
            return;
        };
        let keys = self
            .console
            .as_ref()
            .map(|console| console.keys_for_command(USE))
            .unwrap_or_default();
        let key = key_words(&keys);
        let unit = crate::ui_scale::height_scale(viewport[1]);
        let [x, y] = self.hud.targeting.center(viewport);
        let at = [x * viewport[0], y * viewport[1]];
        let (cap, [left, top]) = layout(at, &key, unit, viewport, prompt.line.is_some());
        let state = &mut self.use_prompt;
        let edge = if prompt.ready {
            Color::new(1.0, 0.84, 0.42, 1.0)
        } else {
            Color::new(0.55, 0.55, 0.55, 0.9)
        };
        let radius = 6.0 * unit;
        let _ = state.list.push(DrawCommand::RoundedRect {
            rect: cap,
            radius,
            color: Color::new(0.06, 0.06, 0.08, 0.86),
        });
        let _ = state.list.push(DrawCommand::Border {
            rect: cap,
            radius,
            width: 1.5 * unit,
            color: edge,
        });
        let words = [
            Some((key, cap, KEY_SIZE, edge, TextAlign::Center)),
            Some((
                prompt.action,
                Rect::new(left, top, WORDS_WIDTH * unit, ACTION_SIZE * 1.3 * unit),
                ACTION_SIZE,
                if prompt.ready {
                    Color::new(1.0, 1.0, 1.0, 1.0)
                } else {
                    Color::new(0.75, 0.75, 0.75, 1.0)
                },
                TextAlign::Start,
            )),
            prompt.line.map(|line| {
                (
                    line,
                    Rect::new(
                        left,
                        top + ACTION_SIZE * 1.25 * unit,
                        WORDS_WIDTH * unit,
                        LINE_SIZE * 1.3 * unit,
                    ),
                    LINE_SIZE,
                    Color::new(0.82, 0.82, 0.82, 1.0),
                    TextAlign::Start,
                )
            }),
        ];
        for (text, rect, size, colour, align) in words.into_iter().flatten() {
            let id = state.lines.len() as u32;
            state.lines.push(text);
            // The key sits in its cap; the words over a dark shadow, to read on any wall.
            let shadow = if align == TextAlign::Center {
                None
            } else {
                Some(Color::new(0.0, 0.0, 0.0, 0.75))
            };
            let size = size * unit;
            let rect = if align == TextAlign::Center {
                // Its line box about the cap's middle; its glyphs are centred on it once
                // drawn, below.
                Rect::new(
                    rect.x,
                    rect.y + (rect.height - size) * 0.5,
                    rect.width,
                    size,
                )
            } else {
                rect
            };
            for (offset, color) in shadow
                .map(|shadow| (1.5 * unit, shadow))
                .into_iter()
                .chain([(0.0, colour)])
            {
                let _ = state.list.push(DrawCommand::Text {
                    rect: Rect::new(rect.x + offset, rect.y + offset, rect.width, rect.height),
                    text: TextId(id),
                    size,
                    color,
                    align,
                    overflow: TextOverflow::Ellipsis,
                    weight: FontWeight::Semibold,
                    letter_spacing: 0.0,
                });
            }
        }
        let state = &self.use_prompt;
        // In the headline font, as centre prints are (Inter until it loads).
        let (vertices, font) = self.game_fonts.target(
            crate::game_font::RetailFont::Headline,
            &mut self.text_vertices,
            &self.ui_font,
        );
        let line = |id: TextId| state.lines.get(id.0 as usize).map_or("", String::as_str);
        // The key first, alone, its ink centred in the cap whatever the font's ascent and
        // descent (Sol, 30/10/2026: its letter sat off the middle); then the words.
        let key_from = vertices.len();
        crate::ui_renderer::append_text_commands_where(
            &state.list,
            line,
            |id, _| id.0 == 0,
            vertices,
            font,
            viewport,
            crate::text::TextStyle::NEUTRAL,
            crate::text::CodePalette::Legible,
        );
        crate::text::centre_vertices(
            &mut vertices[key_from..],
            [cap.x + cap.width * 0.5, cap.y + cap.height * 0.5],
            viewport,
        );
        crate::ui_renderer::append_text_commands_where(
            &state.list,
            line,
            |id, _| id.0 != 0,
            vertices,
            font,
            viewport,
            crate::text::TextStyle::NEUTRAL,
            crate::text::CodePalette::Legible,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cap_names_the_use_key_or_says_use() {
        assert_eq!(key_words(&["e".to_owned()]), "E");
        assert_eq!(key_words(&["MOUSE2".to_owned(), "f".to_owned()]), "Mouse 2");
        assert_eq!(key_words(&[]), "Use");
        // A letter's cap is square; a word's wider.
        assert_eq!(cap_width("E"), CAP);
        assert!(cap_width("Mouse 2") > CAP);
    }

    #[test]
    fn the_prompt_stands_right_of_the_crosshair_and_on_the_screen() {
        let viewport = [1920.0, 1080.0];
        let (cap, [left, top]) = layout([960.0, 540.0], "E", 1.0, viewport, true);
        assert_eq!(cap.x, 960.0 + GAP);
        assert!(
            (cap.y + cap.height * 0.5 - 540.0).abs() < 1e-3,
            "centred on it"
        );
        assert!(left > cap.x + cap.width && top < 540.0);
        // A crosshair at the right edge keeps the cap on the screen.
        let (cap, _) = layout([1915.0, 540.0], "E", 1.0, viewport, false);
        assert!(cap.x + cap.width <= viewport[0]);
    }
}
