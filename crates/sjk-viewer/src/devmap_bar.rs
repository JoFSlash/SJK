//! The devmap bar (`docs/client.md`, "Devmap bar"): on a map with cheats on (`devmap
//! mp/ffa3`) a strip of buttons along the bottom or the top of the screen holds the
//! cheat commands ([`ENTRIES`]): noclip, god mode, no target, the jetpack, `give`'s
//! kinds, the spawn list, removing the NPCs, their bounds, third person and a respawn.
//!
//! While play has the mouse, holding the key bound to `+devbar` (Ctrl unless the player
//! moved it) lends it to the bar, as the quick wheel does: the view stops turning, a
//! pointer shows over the bar, and a left click runs the button under it as if typed
//! in the console. Letting go of the key gives the mouse back; Escape does too.
//! Movement keys keep working meanwhile, and the buttons never attack.
//!
//! `cg_devBar` ([`CVAR`]) chooses where it shows: 0 only while the key is held, at the
//! bottom (the default: stock shows nothing), 1 always at the bottom, 2 always at the
//! top. The bar shows only in a game whose server has cheats on (`sv_cheats` in its
//! system info), never over a menu, the console or the scoreboard, and is drawn in the
//! SJK UI's look in every menu style.

use crate::menu::sjk::{Frame, color, text};
use crate::menu_widgets::{MenuCanvas, TextFamily};
use crate::text::{TextStyle, TextVertex, UiFont};
use sjk_ui::{Color, DrawCommand, FontWeight, Rect, TextAlign};
use std::time::{Duration, Instant};

/// Held, the bar takes the mouse; `-devbar` (the key's release) gives it back.
pub(crate) const HOLD_COMMAND: &str = "+devbar";
pub(crate) const RELEASE_COMMAND: &str = "-devbar";
pub(crate) const HOLD_HELP: &str =
    "Hold to click the devmap bar's cheat buttons (a game with cheats on, devmap)";
pub(crate) const RELEASE_HELP: &str = "Give the mouse back from the devmap bar";
/// Where the bar shows: 0 while `+devbar` is held (bottom), 1 always at the bottom,
/// 2 always at the top (archived).
pub(crate) const CVAR: &str = "cg_devBar";
/// The key `+devbar` is bound to by default.
pub(crate) const DEFAULT_KEY: &str = "CTRL";
/// Marks a saved profile [`migrate_bind`] has seen (archived).
pub(crate) const BIND_VERSION_CVAR: &str = "cl_devBarBindVersion";

/// Bind [`DEFAULT_KEY`] to `+devbar` in a profile saved before the bar, when that key
/// is free and no key holds the bar.
pub(crate) fn migrate_bind(binds: &mut sjk_shell::BindTable) {
    let bound = binds
        .iter()
        .any(|(_, command)| command.trim().eq_ignore_ascii_case(HOLD_COMMAND));
    if !bound && binds.get(DEFAULT_KEY).is_none() {
        let _ = binds.bind(DEFAULT_KEY, HOLD_COMMAND);
    }
}

/// What a button does, which sets its colour.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Kind {
    /// An on/off cheat: run again to turn it off.
    Toggle,
    /// Something given at once.
    Give,
    /// The NPCs.
    Npc,
    /// The player's own view or life.
    Player,
}

/// One button: its label, the line it runs and what it does (shown while hovered).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Entry {
    pub(crate) label: &'static str,
    pub(crate) command: &'static str,
    pub(crate) about: &'static str,
    pub(crate) kind: Kind,
}

const fn entry(
    kind: Kind,
    label: &'static str,
    command: &'static str,
    about: &'static str,
) -> Entry {
    Entry {
        label,
        command,
        about,
        kind,
    }
}

/// The buttons, left to right, grouped by kind. Each runs a command the server takes
/// with cheats on (`sjk-dedicated`'s `bridge_cheats.rs`, `give.rs` after
/// `codemp/game/g_cmds.c:Cmd_Give_f`, the `npc` command) or a client one.
pub(crate) const ENTRIES: [Entry; 15] = [
    entry(
        Kind::Toggle,
        "Noclip",
        "noclip",
        "Fly through walls and floors. Click again to walk.",
    ),
    entry(
        Kind::Toggle,
        "God",
        "god",
        "Take no damage. Click again to turn it off.",
    ),
    entry(
        Kind::Toggle,
        "No target",
        "notarget",
        "NPCs do not see you. Click again to turn it off.",
    ),
    entry(
        Kind::Toggle,
        "Jetpack",
        "jetpack",
        "Put on a jetpack with a full tank, or take it off.",
    ),
    entry(
        Kind::Give,
        "Give all",
        "give all",
        "Every weapon, full ammo, health, armor, Force and items.",
    ),
    entry(
        Kind::Give,
        "Health",
        "give health",
        "Health back to its maximum.",
    ),
    entry(Kind::Give, "Armor", "give armor", "Armor to its maximum."),
    entry(
        Kind::Give,
        "Ammo",
        "give ammo",
        "999 of every kind of ammo.",
    ),
    entry(
        Kind::Give,
        "Weapons",
        "give weapons",
        "Every weapon there is.",
    ),
    entry(
        Kind::Give,
        "Force",
        "give force",
        "Your Force power full again.",
    ),
    entry(
        Kind::Npc,
        "Spawn list",
        crate::console::spawn_panel::COMMAND,
        "Pick an NPC or a vehicle to spawn from a list.",
    ),
    entry(
        Kind::Npc,
        "Clear NPCs",
        "npc kill all",
        "Kill every NPC on the map, vehicles too.",
    ),
    entry(
        Kind::Npc,
        "NPC bounds",
        "npc showbounds",
        "Draw every NPC's bounding box. Click again to hide them.",
    ),
    entry(
        Kind::Player,
        "3rd person",
        "toggle cg_thirdPerson",
        "Switch between first and third person.",
    ),
    entry(Kind::Player, "Respawn", "kill", "Kill yourself to respawn."),
];

/// The bar's height and its distance from the screen's edge, in frame pixels.
const BAR_HEIGHT: f32 = 50.0;
const EDGE: f32 = 14.0;
/// A button's height, its padding beside the label and the gaps.
const BUTTON_HEIGHT: f32 = 32.0;
const BUTTON_PADDING: f32 = 10.0;
const GAP: f32 = 6.0;
/// The gap between two kinds, with a thin line in its middle.
const KIND_GAP: f32 = 22.0;
/// The bar's padding at either end, and the room of its "DEVMAP" label.
const END_PADDING: f32 = 16.0;
const LABEL_WIDTH: f32 = 72.0;
/// The labels' size, and about how wide one character is at it (Body SemiBold).
const LABEL_SIZE: f32 = 14.0;
const CHAR_WIDTH: f32 = 6.8;
/// The line under (or over) the bar: the hovered button's about, or the key hint.
const NOTE_HEIGHT: f32 = 24.0;
const NOTE_SIZE: f32 = 14.0;
/// How long a clicked button stays lit.
const FLASH: Duration = Duration::from_millis(350);
/// The pointer's size, in frame pixels.
const POINTER: f32 = 12.0;

/// Where the bar is.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Place {
    Bottom,
    Top,
}

/// `cg_devBar`'s value: where the bar shows, and whether only while held.
pub(crate) fn placement(value: i64) -> (Place, bool) {
    match value {
        1 => (Place::Bottom, true),
        2 => (Place::Top, true),
        _ => (Place::Bottom, false),
    }
}

/// A button's width for its label.
fn button_width(label: &str) -> f32 {
    BUTTON_PADDING * 2.0 + CHAR_WIDTH * label.chars().count() as f32
}

/// The bar's rectangle and every button's, in frame pixels (1920 x 1080), for `place`.
pub(crate) fn layout(place: Place) -> ([f32; 4], [[f32; 4]; ENTRIES.len()]) {
    let mut width = END_PADDING * 2.0 + LABEL_WIDTH;
    for (index, entry) in ENTRIES.iter().enumerate() {
        if index > 0 {
            width += if ENTRIES[index - 1].kind == entry.kind {
                GAP
            } else {
                KIND_GAP
            };
        }
        width += button_width(entry.label);
    }
    let x = (1920.0 - width) * 0.5;
    let y = match place {
        Place::Bottom => 1080.0 - EDGE - BAR_HEIGHT,
        Place::Top => EDGE,
    };
    let mut buttons = [[0.0; 4]; ENTRIES.len()];
    let mut left = x + END_PADDING + LABEL_WIDTH;
    let top = y + (BAR_HEIGHT - BUTTON_HEIGHT) * 0.5;
    for (index, entry) in ENTRIES.iter().enumerate() {
        if index > 0 {
            left += if ENTRIES[index - 1].kind == entry.kind {
                GAP
            } else {
                KIND_GAP
            };
        }
        let button = button_width(entry.label);
        buttons[index] = [left, top, button, BUTTON_HEIGHT];
        left += button;
    }
    ([x, y, width, BAR_HEIGHT], buttons)
}

/// A kind's colour.
fn tint(kind: Kind) -> Color {
    match kind {
        Kind::Toggle => color::GOLD,
        Kind::Give => color::HOLO,
        Kind::Npc => color::EMBER,
        Kind::Player => color::MUTED,
    }
}

fn push(canvas: &mut MenuCanvas, command: DrawCommand) {
    let _ = canvas.draw_list_mut().push(command);
}

/// The bar's state: whether it holds the mouse, where its pointer is, and its drawing.
pub(crate) struct DevBar {
    canvas: MenuCanvas,
    /// `+devbar` is held: the bar has the mouse.
    held: bool,
    /// The pointer, in window pixels, while held.
    pointer: [f32; 2],
    /// Where the bar was laid out last, in window pixels: its buttons, for the pointer.
    buttons: [Rect; ENTRIES.len()],
    bar: Rect,
    /// The window the bar was laid out for, which bounds the pointer.
    viewport: [f32; 2],
    /// The button under the pointer.
    hovered: Option<usize>,
    /// The button clicked last, lit until then.
    flash: Option<(usize, Instant)>,
    /// Drawn this frame.
    shown: bool,
    /// The keys bound to `+devbar`, as players read them (kept between frames).
    keys: String,
}

impl Default for DevBar {
    fn default() -> Self {
        Self {
            canvas: MenuCanvas::with_text_capacity(64),
            held: false,
            pointer: [0.0; 2],
            buttons: [Rect::new(0.0, 0.0, 0.0, 0.0); ENTRIES.len()],
            bar: Rect::new(0.0, 0.0, 0.0, 0.0),
            viewport: [0.0; 2],
            hovered: None,
            flash: None,
            shown: false,
            keys: String::new(),
        }
    }
}

impl DevBar {
    /// Whether the bar has the mouse.
    pub(crate) fn is_held(&self) -> bool {
        self.held
    }

    /// Whether the bar was drawn this frame.
    pub(crate) fn shown(&self) -> bool {
        self.shown
    }

    pub(crate) fn draw_list(&self) -> &sjk_ui::DrawList {
        self.canvas.draw_list()
    }

    /// Take the mouse, the pointer starting over the bar's middle.
    pub(crate) fn hold(&mut self) {
        if !self.held {
            self.held = true;
            self.pointer = [
                self.bar.x + self.bar.width * 0.5,
                self.bar.y + self.bar.height * 0.5,
            ];
            self.hovered = self.under(self.pointer);
        }
    }

    /// Give the mouse back.
    pub(crate) fn release(&mut self) {
        self.held = false;
        self.hovered = None;
    }

    /// Mouse movement while held, in raw counts: moves the pointer within the window.
    pub(crate) fn moved(&mut self, delta: [f32; 2]) {
        let viewport = self.viewport;
        if !self.held {
            return;
        }
        self.pointer = [
            (self.pointer[0] + delta[0]).clamp(0.0, viewport[0]),
            (self.pointer[1] + delta[1]).clamp(0.0, viewport[1]),
        ];
        self.hovered = self.under(self.pointer);
    }

    /// A left click while held: the line of the button under the pointer, lit a moment.
    pub(crate) fn click(&mut self, now: Instant) -> Option<&'static str> {
        if !self.held {
            return None;
        }
        let index = self.hovered?;
        self.flash = Some((index, now));
        Some(ENTRIES[index].command)
    }

    /// The button at window point `point`.
    fn under(&self, [x, y]: [f32; 2]) -> Option<usize> {
        self.buttons.iter().position(|rect| {
            rect.width > 0.0
                && x >= rect.x
                && x <= rect.x + rect.width
                && y >= rect.y
                && y <= rect.y + rect.height
        })
    }

    /// Lay out and draw the bar at `place` for `viewport`, or nothing when not `shown`.
    pub(crate) fn build(&mut self, viewport: [f32; 2], place: Place, shown: bool, now: Instant) {
        self.shown = shown;
        self.viewport = viewport;
        self.canvas.begin_transparent(viewport);
        if !shown {
            self.canvas.finish(0);
            return;
        }
        let frame = Frame::new(viewport);
        let s = frame.s;
        let (bar, buttons) = layout(place);
        self.bar = frame.rect(bar[0], bar[1], bar[2], bar[3]);
        for (rect, button) in self.buttons.iter_mut().zip(&buttons) {
            *rect = frame.rect(button[0], button[1], button[2], button[3]);
        }
        if self.held {
            self.hovered = self.under(self.pointer);
        }
        if self
            .flash
            .is_some_and(|(_, at)| now.saturating_duration_since(at) > FLASH)
        {
            self.flash = None;
        }
        let canvas = &mut self.canvas;
        // The glass strip, its edge brighter while it has the mouse.
        push(
            canvas,
            DrawCommand::RoundedRect {
                rect: frame.rect(bar[0] + 3.0, bar[1] + 6.0, bar[2], bar[3]),
                radius: bar[3] * 0.5 * s,
                color: color::alpha(color::SPACE, 0.45),
            },
        );
        push(
            canvas,
            DrawCommand::RoundedRect {
                rect: self.bar,
                radius: bar[3] * 0.5 * s,
                color: Color::new(0.04, 0.06, 0.12, 0.86),
            },
        );
        push(
            canvas,
            DrawCommand::Border {
                rect: self.bar,
                radius: bar[3] * 0.5 * s,
                width: 1.5 * s,
                color: color::alpha(color::HOLO, if self.held { 0.7 } else { 0.35 }),
            },
        );
        text(
            canvas,
            TextFamily::Display,
            format_args!("DEVMAP"),
            frame.rect(bar[0] + END_PADDING, bar[1], LABEL_WIDTH, bar[3]),
            17.0 * s,
            color::GOLD,
            FontWeight::Semibold,
            TextAlign::Start,
        );
        for (index, (entry, button)) in ENTRIES.iter().zip(&buttons).enumerate() {
            // A thin line between two kinds.
            if index > 0 && ENTRIES[index - 1].kind != entry.kind {
                push(
                    canvas,
                    DrawCommand::SolidRect {
                        rect: frame.rect(
                            button[0] - KIND_GAP * 0.5 - 0.75,
                            button[1] + 6.0,
                            1.5,
                            button[3] - 12.0,
                        ),
                        color: color::alpha(color::HOLO, 0.25),
                    },
                );
            }
            let tint = tint(entry.kind);
            let lit = self.flash.is_some_and(|(flashed, _)| flashed == index);
            let hovered = self.held && self.hovered == Some(index);
            let rect = frame.rect(button[0], button[1], button[2], button[3]);
            push(
                canvas,
                DrawCommand::RoundedRect {
                    rect,
                    radius: rect.height * 0.5,
                    color: color::alpha(
                        tint,
                        if lit {
                            0.85
                        } else if hovered {
                            0.26
                        } else {
                            0.08
                        },
                    ),
                },
            );
            push(
                canvas,
                DrawCommand::Border {
                    rect,
                    radius: rect.height * 0.5,
                    width: 1.5 * s,
                    color: color::alpha(tint, if hovered || lit { 0.95 } else { 0.55 }),
                },
            );
            text(
                canvas,
                TextFamily::Body,
                format_args!("{}", entry.label),
                rect,
                LABEL_SIZE * s,
                if lit { color::SPACE } else { tint },
                FontWeight::Semibold,
                TextAlign::Center,
            );
        }
        // The note beside the bar, on its inner side: what the hovered button does, or
        // how to click while the mouse turns the view.
        let note_y = match place {
            Place::Bottom => bar[1] - NOTE_HEIGHT - 4.0,
            Place::Top => bar[1] + bar[3] + 4.0,
        };
        let note = frame.rect(bar[0], note_y, bar[2], NOTE_HEIGHT);
        match (self.held, self.hovered) {
            (true, Some(index)) => text(
                canvas,
                TextFamily::Body,
                format_args!("{} \u{b7} {}", ENTRIES[index].about, ENTRIES[index].command),
                note,
                NOTE_SIZE * s,
                color::TEXT,
                FontWeight::Regular,
                TextAlign::Center,
            ),
            (true, None) => text(
                canvas,
                TextFamily::Body,
                format_args!("Click a button to run it"),
                note,
                NOTE_SIZE * s,
                color::MUTED,
                FontWeight::Regular,
                TextAlign::Center,
            ),
            (false, _) if self.keys == "unbound" => text(
                canvas,
                TextFamily::Body,
                format_args!("Bind a key to {HOLD_COMMAND} to click the bar"),
                note,
                NOTE_SIZE * s,
                color::QUIET,
                FontWeight::Regular,
                TextAlign::Center,
            ),
            (false, _) => text(
                canvas,
                TextFamily::Body,
                format_args!("Hold {} to click the bar", self.keys),
                note,
                NOTE_SIZE * s,
                color::QUIET,
                FontWeight::Regular,
                TextAlign::Center,
            ),
        }
        // The pointer: a white dot ringed dark, over everything.
        if self.held {
            let size = POINTER * s;
            let [x, y] = self.pointer;
            push(
                canvas,
                DrawCommand::RoundedRect {
                    rect: Rect::new(
                        x - size * 0.5 - 2.0 * s,
                        y - size * 0.5 - 2.0 * s,
                        size + 4.0 * s,
                        size + 4.0 * s,
                    ),
                    radius: size * 0.5 + 2.0 * s,
                    color: color::alpha(color::SPACE, 0.9),
                },
            );
            push(
                canvas,
                DrawCommand::RoundedRect {
                    rect: Rect::new(x - size * 0.5, y - size * 0.5, size, size),
                    radius: size * 0.5,
                    color: Color::new(1.0, 1.0, 1.0, 0.95),
                },
            );
        }
        self.canvas.finish(0);
    }

    fn append_text(
        &self,
        fonts: Option<crate::game_font::SjkFonts<'_>>,
        vertices: &mut Vec<TextVertex>,
        font: &UiFont,
        viewport: [f32; 2],
    ) {
        match fonts {
            Some(fonts) => self
                .canvas
                .append_text_families(fonts, viewport, TextStyle::NEUTRAL),
            None => self
                .canvas
                .append_text_styled(vertices, font, viewport, TextStyle::NEUTRAL),
        }
    }
}

/// Lay out and draw the bar for this frame; `visible` when the HUD shows (a game, no
/// menu, no console, no intermission).
pub(crate) fn append(gpu: &mut crate::GpuState, viewport: [f32; 2], visible: bool) {
    let cheats = gpu.devmap_bar_cheats();
    if gpu.devmap_bar.held && (!cheats || gpu.text_has_keyboard()) {
        gpu.devmap_bar.release();
    }
    let (place, always) = placement(
        gpu.console
            .as_ref()
            .and_then(|console| console.integer_cvar(CVAR))
            .unwrap_or(0),
    );
    let shown = visible && cheats && (always || gpu.devmap_bar.held);
    if shown
        && !gpu.devmap_bar.held
        && let Some(console) = &gpu.console
    {
        console.write_keys_for_command(HOLD_COMMAND, &mut gpu.devmap_bar.keys);
    }
    gpu.devmap_bar.build(viewport, place, shown, Instant::now());
    if shown {
        gpu.devmap_bar.append_text(
            gpu.game_fonts.sjk(),
            &mut gpu.text_vertices,
            &gpu.ui_font,
            viewport,
        );
    }
}

impl crate::GpuState {
    /// Whether the live game's server has cheats on: `sv_cheats` in its
    /// `CS_SYSTEMINFO` (as a stock server sends it), or a server this client hosts with
    /// cheats (`devmap`), since SJK's own server leaves `sv_cheats` out of it.
    pub(crate) fn devmap_bar_cheats(&self) -> bool {
        let Some(session) = self.live_session.as_ref() else {
            return false;
        };
        let told = session
            .game_state()
            .config_string(1)
            .and_then(|info| sjk_protocol::info_value(info, b"sv_cheats"))
            .is_some_and(|value| value.first() == Some(&b'1'));
        told || self
            .client_menu
            .as_ref()
            .is_some_and(crate::menu::ClientMenu::hosting_with_cheats)
    }

    /// `+devbar`: the bar takes the mouse, in a game with cheats on.
    pub(crate) fn hold_devmap_bar(&mut self) -> Result<Vec<String>, String> {
        if !self.devmap_bar_cheats() {
            return Ok(Vec::new());
        }
        self.devmap_bar.hold();
        Ok(Vec::new())
    }

    /// `-devbar`: the mouse turns the view again.
    pub(crate) fn release_devmap_bar(&mut self) -> Result<Vec<String>, String> {
        self.devmap_bar.release();
        Ok(Vec::new())
    }

    /// A left click while the bar has the mouse: run the button under its pointer.
    pub(crate) fn click_devmap_bar(&mut self) {
        let Some(command) = self.devmap_bar.click(Instant::now()) else {
            return;
        };
        if let Some(console) = &mut self.console
            && let Err(error) = console.queue_command(command)
        {
            console.push_log(format!("^1devbar: {error}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bar_fits_between_the_hud_corners() {
        for place in [Place::Bottom, Place::Top] {
            let (bar, buttons) = layout(place);
            // The HUD's dials take about 300 frame pixels at either bottom corner.
            assert!(bar[0] >= 300.0, "bar starts at {}", bar[0]);
            assert!(bar[0] + bar[2] <= 1620.0, "bar ends at {}", bar[0] + bar[2]);
            assert!(bar[1] >= 0.0 && bar[1] + bar[3] <= 1080.0);
            for pair in buttons.windows(2) {
                assert!(pair[0][0] + pair[0][2] < pair[1][0], "buttons overlap");
            }
            for button in buttons {
                assert!(button[0] >= bar[0] && button[0] + button[2] <= bar[0] + bar[2]);
                assert!(button[1] >= bar[1] && button[1] + button[3] <= bar[1] + bar[3]);
            }
        }
    }

    #[test]
    fn the_cvar_chooses_the_place_and_whether_always() {
        assert_eq!(placement(0), (Place::Bottom, false));
        assert_eq!(placement(1), (Place::Bottom, true));
        assert_eq!(placement(2), (Place::Top, true));
        assert_eq!(placement(7), (Place::Bottom, false));
    }

    #[test]
    fn every_line_runs_one_command() {
        for entry in ENTRIES {
            assert!(!entry.command.contains(';') && !entry.command.contains('"'));
            assert!(!entry.label.is_empty() && !entry.about.is_empty());
        }
    }

    #[test]
    fn a_click_runs_the_button_under_the_pointer_only_while_held() {
        let mut bar = DevBar::default();
        let viewport = [1920.0, 1080.0];
        bar.build(viewport, Place::Bottom, true, Instant::now());
        let now = Instant::now();
        assert_eq!(bar.click(now), None, "not held: the mouse turns the view");
        bar.hold();
        // From the middle, onto the first button.
        let first = bar.buttons[0];
        let target = [first.x + first.width * 0.5, first.y + first.height * 0.5];
        bar.moved([target[0] - bar.pointer[0], target[1] - bar.pointer[1]]);
        assert_eq!(bar.click(now), Some("noclip"));
        // Off the bar, nothing.
        bar.moved([0.0, -400.0]);
        assert_eq!(bar.click(now), None);
        bar.release();
        assert!(!bar.is_held());
    }

    #[test]
    fn the_pointer_stays_on_screen() {
        let mut bar = DevBar::default();
        let viewport = [1280.0, 720.0];
        bar.build(viewport, Place::Top, true, Instant::now());
        bar.hold();
        bar.moved([-99_999.0, 99_999.0]);
        assert_eq!(bar.pointer, [0.0, 720.0]);
    }
}
