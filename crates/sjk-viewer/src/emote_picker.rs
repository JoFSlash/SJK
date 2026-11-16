//! The emote picker (`docs/hub-chat.md`, "Emotes in the client"): `emotes` shows an
//! SJK pop-up card over the darkened screen with every installed emote as a tile,
//! its picture ([`crate::emote_icons`]), name and id, and a search field that takes
//! the typing at once. A click on a tile, or Enter on the chosen one, closes the card
//! and plays it as `sjkemote <id>` would. Escape clears the search, then closes.

use crate::emotes::EmoteDef;
use crate::menu::sjk::{Frame, TextTarget, color, key_hint, key_hint_width, kit, text};
use crate::menu_widgets::{MenuCanvas, TextFamily};
use sjk_ui::{Color, DrawCommand, FontWeight, InputEvent, TextAlign, UiEventKind};
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

/// Console command that opens the picker.
pub(crate) const COMMAND: &str = "emotes";
pub(crate) const HELP: &str = "Pick an emote to play from a searchable list of every installed one";

const CLOSE_TOKEN: u16 = 1;
const SEARCH_TOKEN: u16 = 2;
/// The shown tiles' tokens, from the grid's top left.
const TILE_TOKEN: u16 = 16;

const CARD_X: f32 = 330.0;
const CARD_WIDTH: f32 = 1260.0;
const CARD_TOP: f32 = 70.0;
const CARD_HEIGHT: f32 = 890.0;
const MARGIN: f32 = 44.0;
const INNER_X: f32 = CARD_X + MARGIN;
const INNER_WIDTH: f32 = CARD_WIDTH - MARGIN * 2.0;
const SEARCH_Y: f32 = CARD_TOP + 126.0;
const SEARCH_HEIGHT: f32 = 50.0;
const GRID_Y: f32 = SEARCH_Y + SEARCH_HEIGHT + 20.0;
pub(crate) const COLUMNS: usize = 7;
pub(crate) const ROWS: usize = 4;
const GAP: f32 = 12.0;
/// Room on the grid's right for the scroll bar.
const BAR_ROOM: f32 = 18.0;
const TILE_WIDTH: f32 = (INNER_WIDTH - BAR_ROOM - GAP * (COLUMNS as f32 - 1.0)) / COLUMNS as f32;
const TILE_HEIGHT: f32 = 134.0;
const ICON: f32 = 72.0;
const BUTTON_HEIGHT: f32 = 46.0;
const CLOSE_WIDTH: f32 = 150.0;
const FOOT_Y: f32 = CARD_TOP + CARD_HEIGHT - MARGIN - BUTTON_HEIGHT;
const KEYS_Y: f32 = 992.0;
const KEYS_RIGHT: f32 = 1824.0;
/// The longest search.
const SEARCH_MAX: usize = 32;
/// A name longer than this is cut with an ellipsis to fit its tile.
const NAME_CHARS: usize = 17;

// The grid fits between the search and the foot, the card above the key hints, and
// a tile's name has room beside its picture.
const _: () =
    assert!(GRID_Y + ROWS as f32 * TILE_HEIGHT + (ROWS as f32 - 1.0) * GAP < FOOT_Y - 10.0);
const _: () = assert!(CARD_TOP + CARD_HEIGHT < KEYS_Y - 10.0);
const _: () = assert!(TILE_WIDTH > ICON + 40.0);

/// What the player chose on the card.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Choice {
    Close,
    /// Play the emote with this id.
    Play(String),
}

/// One emote the card lists.
#[derive(Clone, Debug, PartialEq)]
struct Entry {
    id: String,
    name: String,
    /// The name and id in lower case, what the search matches.
    key: String,
}

#[derive(Default)]
pub(crate) struct State {
    pub(crate) card: Card,
    /// The card opened the game menu, which goes with it.
    opened_menu: bool,
}

pub(crate) struct Card {
    wanted: bool,
    /// Shown this frame.
    open: bool,
    entries: Vec<Entry>,
    query: String,
    /// The entries the search keeps, as indices into `entries`.
    shown: Vec<usize>,
    /// The chosen one, an index into `shown`.
    chosen: usize,
    /// The grid's first row on screen.
    first_row: usize,
    ui: MenuCanvas,
}

impl Default for Card {
    fn default() -> Self {
        Self {
            wanted: false,
            open: false,
            entries: Vec::new(),
            query: String::new(),
            shown: Vec::new(),
            chosen: 0,
            first_row: 0,
            ui: MenuCanvas::with_capacities(80, 64, 320),
        }
    }
}

/// `name` cut to [`NAME_CHARS`] characters, with an ellipsis when it was longer.
fn short(name: &str) -> String {
    if name.chars().count() <= NAME_CHARS {
        return name.to_owned();
    }
    let mut cut: String = name.chars().take(NAME_CHARS - 1).collect();
    cut.push('…');
    cut
}

impl Card {
    /// Open on `catalogue`, by name, with an empty search.
    pub(crate) fn show(&mut self, catalogue: &[EmoteDef]) {
        let mut entries: Vec<Entry> = catalogue
            .iter()
            .map(|def| {
                let name = if def.name.trim().is_empty() {
                    def.id.clone()
                } else {
                    def.name.trim().to_owned()
                };
                Entry {
                    key: format!("{} {}", name.to_lowercase(), def.id),
                    id: def.id.clone(),
                    name,
                }
            })
            .collect();
        entries.sort_by(|a, b| a.key.cmp(&b.key));
        self.entries = entries;
        self.query.clear();
        crate::text_entry::deselect();
        self.filter();
        self.wanted = true;
    }

    /// Whether the card has something to show.
    pub(crate) fn wants(&self) -> bool {
        self.wanted
    }

    pub(crate) fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    pub(crate) fn is_open(&self) -> bool {
        self.open
    }

    /// Keep the entries whose name or id holds every word of the search, back to the
    /// top.
    fn filter(&mut self) {
        let query = self.query.to_lowercase();
        let words: Vec<&str> = query.split_whitespace().collect();
        self.shown = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| words.iter().all(|word| entry.key.contains(word)))
            .map(|(index, _)| index)
            .collect();
        self.chosen = 0;
        self.first_row = 0;
    }

    fn rows(&self) -> usize {
        self.shown.len().div_ceil(COLUMNS)
    }

    /// Scroll so the chosen tile is on screen.
    fn reveal(&mut self) {
        let row = self.chosen / COLUMNS;
        if row < self.first_row {
            self.first_row = row;
        } else if row >= self.first_row + ROWS {
            self.first_row = row + 1 - ROWS;
        }
    }

    /// Move the choice by `by` tiles, stopping at the ends.
    fn step(&mut self, by: isize) {
        let Some(last) = self.shown.len().checked_sub(1) else {
            return;
        };
        self.chosen = self.chosen.saturating_add_signed(by).min(last);
        self.reveal();
    }

    /// The tile under shown slot `slot` (from the grid's top left), as an index into
    /// `shown`.
    fn at_slot(&self, slot: usize) -> Option<usize> {
        let index = self.first_row * COLUMNS + slot;
        (index < self.shown.len()).then_some(index)
    }

    fn play(&mut self, index: usize) -> Option<Choice> {
        let entry = self.entries.get(*self.shown.get(index)?)?;
        let id = entry.id.clone();
        self.close();
        Some(Choice::Play(id))
    }

    /// Typing goes into the search; arrows, Page Up and Page Down move the choice,
    /// Enter plays it, Escape clears the search and then closes. Ctrl or Shift come
    /// from the caller (the console tracks them).
    pub(crate) fn handle_key(
        &mut self,
        event: &KeyEvent,
        shift: bool,
        control: bool,
    ) -> Option<Choice> {
        if event.state != ElementState::Pressed {
            return None;
        }
        let PhysicalKey::Code(code) = event.physical_key else {
            return None;
        };
        if event.repeat
            && !matches!(
                code,
                KeyCode::Backspace
                    | KeyCode::ArrowLeft
                    | KeyCode::ArrowRight
                    | KeyCode::ArrowUp
                    | KeyCode::ArrowDown
            )
        {
            return None;
        }
        self.key(code, event.text.as_deref(), shift, control)
    }

    /// A pressed key by its code and what it typed ([`Self::handle_key`]).
    pub(crate) fn key(
        &mut self,
        code: KeyCode,
        typed: Option<&str>,
        shift: bool,
        control: bool,
    ) -> Option<Choice> {
        match code {
            KeyCode::Escape => {
                if self.query.is_empty() {
                    return Some(self.close());
                }
                self.query.clear();
                crate::text_entry::deselect();
                self.filter();
            }
            KeyCode::Enter | KeyCode::NumpadEnter => return self.play(self.chosen),
            KeyCode::ArrowLeft => self.step(-1),
            KeyCode::ArrowRight => self.step(1),
            KeyCode::ArrowUp => self.step(-(COLUMNS as isize)),
            KeyCode::ArrowDown => self.step(COLUMNS as isize),
            KeyCode::PageUp => self.step(-((COLUMNS * ROWS) as isize)),
            KeyCode::PageDown => self.step((COLUMNS * ROWS) as isize),
            _ => {
                if crate::text_entry::edit(
                    &mut self.query,
                    code,
                    typed,
                    shift,
                    control,
                    SEARCH_MAX,
                    true,
                ) {
                    self.filter();
                }
            }
        }
        None
    }

    /// A click on a tile plays it, on Close closes; the wheel scrolls a row a notch.
    pub(crate) fn handle_pointer(&mut self, event: InputEvent) -> Option<Choice> {
        if let InputEvent::PointerWheel { delta, .. } = event {
            let last = self.rows().saturating_sub(ROWS);
            if delta.y > 0.0 {
                self.first_row = self.first_row.saturating_sub(1);
            } else if delta.y < 0.0 {
                self.first_row = (self.first_row + 1).min(last);
            }
            return None;
        }
        let event = self.ui.pointer(event)?;
        if event.kind != UiEventKind::Activate {
            return None;
        }
        match event.token? {
            CLOSE_TOKEN => Some(self.close()),
            token if token >= TILE_TOKEN => {
                let index = self.at_slot(usize::from(token - TILE_TOKEN))?;
                self.play(index)
            }
            _ => None,
        }
    }

    fn close(&mut self) -> Choice {
        self.wanted = false;
        crate::text_entry::deselect();
        Choice::Close
    }

    pub(crate) fn draw_list(&self) -> &sjk_ui::DrawList {
        self.ui.draw_list()
    }

    /// Draw the card over the whole frame.
    pub(crate) fn append(&mut self, target: TextTarget<'_>, viewport: [f32; 2]) {
        let frame = Frame::new(viewport);
        let s = frame.s;
        let rows = self.rows();
        let ui = &mut self.ui;
        ui.begin_transparent(viewport);
        kit::scrim(ui, viewport);
        kit::card(ui, &frame, [CARD_X, CARD_TOP, CARD_WIDTH, CARD_HEIGHT]);
        text(
            ui,
            TextFamily::Display,
            format_args!("EMOTES"),
            frame.rect(INNER_X, CARD_TOP + 30.0, INNER_WIDTH, 30.0),
            22.0 * s,
            color::GOLD_BRIGHT,
            FontWeight::Semibold,
            TextAlign::Start,
        );
        text(
            ui,
            TextFamily::Display,
            format_args!("Play an emote"),
            frame.rect(INNER_X, CARD_TOP + 66.0, INNER_WIDTH, 48.0),
            36.0 * s,
            color::TEXT,
            FontWeight::Semibold,
            TextAlign::Start,
        );
        let searching = !self.query.trim().is_empty();
        kit::search(
            ui,
            &frame,
            [INNER_X, SEARCH_Y, INNER_WIDTH, SEARCH_HEIGHT],
            &self.query,
            true,
            "Type a name",
            searching.then_some(self.shown.len()),
            SEARCH_TOKEN,
        );
        if self.shown.is_empty() {
            let words = if self.entries.is_empty() {
                "No emotes are installed yet (emotes/<id>.emote in the game's files).".to_owned()
            } else {
                format!("No emote matches \"{}\".", self.query.trim())
            };
            text(
                ui,
                TextFamily::Body,
                format_args!("{words}"),
                frame.rect(INNER_X, GRID_Y + 40.0, INNER_WIDTH, 30.0),
                19.0 * s,
                color::MUTED,
                FontWeight::Regular,
                TextAlign::Center,
            );
        }
        for slot in 0..COLUMNS * ROWS {
            let index = self.first_row * COLUMNS + slot;
            let Some(entry) = self.shown.get(index).map(|&at| &self.entries[at]) else {
                break;
            };
            let x = INNER_X + (slot % COLUMNS) as f32 * (TILE_WIDTH + GAP);
            let y = GRID_Y + (slot / COLUMNS) as f32 * (TILE_HEIGHT + GAP);
            let token = TILE_TOKEN + slot as u16;
            let lit = index == self.chosen || ui.token_hovered(token);
            tile(ui, &frame, [x, y, TILE_WIDTH, TILE_HEIGHT], lit);
            let icon = frame.rect(x + (TILE_WIDTH - ICON) * 0.5, y + 12.0, ICON, ICON);
            let picture = match crate::emote_icons::texture(&entry.id) {
                Some(texture) => DrawCommand::TexturedQuad {
                    rect: icon,
                    texture,
                    color: Color::new(1.0, 1.0, 1.0, 0.95),
                },
                None => DrawCommand::Border {
                    rect: icon,
                    radius: icon.width * 0.5,
                    width: (1.5 * s).max(1.0),
                    color: color::alpha(color::HOLO, 0.6),
                },
            };
            let _ = ui.draw_list_mut().push(picture);
            text(
                ui,
                TextFamily::Body,
                format_args!("{}", short(&entry.name)),
                frame.rect(x + 6.0, y + 90.0, TILE_WIDTH - 12.0, 22.0),
                17.0 * s,
                if lit { color::GOLD_BRIGHT } else { color::TEXT },
                FontWeight::Regular,
                TextAlign::Center,
            );
            text(
                ui,
                TextFamily::Code,
                format_args!("{}", short(&entry.id)),
                frame.rect(x + 6.0, y + 111.0, TILE_WIDTH - 12.0, 18.0),
                13.0 * s,
                color::QUIET,
                FontWeight::Regular,
                TextAlign::Center,
            );
            ui.hit_region(token, frame.rect(x, y, TILE_WIDTH, TILE_HEIGHT));
        }
        scroll_bar(ui, &frame, self.first_row, rows);
        text(
            ui,
            TextFamily::Body,
            format_args!(
                "{} emote{} · click one to play it",
                self.entries.len(),
                if self.entries.len() == 1 { "" } else { "s" }
            ),
            frame.rect(
                INNER_X,
                FOOT_Y + 11.0,
                INNER_WIDTH - CLOSE_WIDTH - 20.0,
                24.0,
            ),
            17.0 * s,
            color::MUTED,
            FontWeight::Regular,
            TextAlign::Start,
        );
        let right = CARD_X + CARD_WIDTH - MARGIN;
        kit::button(
            ui,
            &frame,
            [right - CLOSE_WIDTH, FOOT_Y, CLOSE_WIDTH, BUTTON_HEIGHT],
            "Close",
            false,
            true,
            false,
            CLOSE_TOKEN,
        );
        let [right, keys_y] = frame.point(KEYS_RIGHT, KEYS_Y);
        let close_width = key_hint_width(&["Esc"], "close", s);
        let play_width = key_hint_width(&["Enter"], "play", s);
        let x = right - close_width;
        key_hint(ui, &["Esc"], "close", x, keys_y, s);
        key_hint(ui, &["Enter"], "play", x - play_width - 20.0 * s, keys_y, s);
        ui.finish(SEARCH_TOKEN);
        target.append(&self.ui, viewport);
    }
}

/// A tile's ground: holo glass, gold-edged while chosen or hovered.
fn tile(canvas: &mut MenuCanvas, frame: &Frame, rect: [f32; 4], lit: bool) {
    let [x, y, width, height] = rect;
    let rect = frame.rect(x, y, width, height);
    let radius = 10.0 * frame.s;
    let draw = canvas.draw_list_mut();
    let _ = draw.push(DrawCommand::RoundedRect {
        rect,
        radius,
        color: color::alpha(color::HOLO, if lit { 0.12 } else { 0.04 }),
    });
    let _ = draw.push(DrawCommand::Border {
        rect,
        radius,
        width: (1.5 * frame.s).max(1.0),
        color: if lit {
            color::alpha(color::GOLD, 0.85)
        } else {
            color::alpha(color::HOLO, 0.16)
        },
    });
}

/// The thin bar right of the grid showing which rows are on screen, when they do not
/// all fit.
fn scroll_bar(canvas: &mut MenuCanvas, frame: &Frame, first: usize, rows: usize) {
    if rows <= ROWS {
        return;
    }
    let x = INNER_X + INNER_WIDTH - 5.0;
    let height = ROWS as f32 * TILE_HEIGHT + (ROWS as f32 - 1.0) * GAP;
    let track = frame.rect(x, GRID_Y, 4.0, height);
    let thumb_height = height * ROWS as f32 / rows as f32;
    let thumb_y = GRID_Y + (height - thumb_height) * first as f32 / (rows - ROWS) as f32;
    let thumb = frame.rect(x, thumb_y, 4.0, thumb_height);
    let draw = canvas.draw_list_mut();
    let _ = draw.push(DrawCommand::RoundedRect {
        rect: track,
        radius: track.width * 0.5,
        color: color::alpha(color::HOLO, 0.1),
    });
    let _ = draw.push(DrawCommand::RoundedRect {
        rect: thumb,
        radius: thumb.width * 0.5,
        color: color::alpha(color::GOLD, 0.8),
    });
}

impl crate::GpuState {
    /// `emotes`: show the picker, over the game menu, on a server.
    pub(crate) fn emote_picker_command(&mut self) -> Result<Vec<String>, String> {
        if self.live_session.is_none() {
            return Err("Join a server to emote".to_owned());
        }
        if crate::emotes::lock().catalogue.is_none()
            && let Some(vfs) = self.vfs.as_deref()
        {
            crate::emotes::lock().catalogue = Some(crate::emotes::load(vfs));
        }
        if let Some(console) = &mut self.console {
            console.set_open(false);
        }
        let menu_visible = self
            .client_menu
            .as_ref()
            .is_some_and(crate::menu::ClientMenu::is_visible);
        if !menu_visible && !self.game_menu {
            self.release_pointer();
            self.game_menu = true;
            self.game_menu_page = crate::ingame_menu::Page::Main;
            self.game_menu_row = 0;
            self.emote_picker.opened_menu = true;
        }
        {
            let state = crate::emotes::lock();
            self.emote_picker
                .card
                .show(state.catalogue.as_deref().unwrap_or_default());
        }
        self.sync_cursor_policy();
        Ok(Vec::new())
    }

    /// Whether the picker shows this frame: asked for, and the player on a menu or the
    /// game menu with nothing else over it. `covered` is another card or the console.
    pub(crate) fn prepare_emote_picker(&mut self, covered: bool) -> bool {
        let menu_visible = self
            .client_menu
            .as_ref()
            .is_some_and(crate::menu::ClientMenu::is_visible);
        let console_open = self
            .console
            .as_ref()
            .is_some_and(crate::console::ViewerConsole::is_open);
        let open = self.emote_picker.card.wants()
            && (menu_visible || self.game_menu)
            && !covered
            && !console_open
            && !self.text_dialog.is_open()
            && self.world_load_task.is_none()
            && self.world_install_task.is_none()
            && !self.pending_map_reload;
        self.emote_picker.card.set_open(open);
        open
    }

    /// Draw the picker. It darkens everything, so every font batch appended before it
    /// is dropped (text draws above all shapes).
    pub(crate) fn append_emote_picker(&mut self, viewport: [f32; 2]) {
        self.text_vertices.clear();
        self.classic_text_vertices.clear();
        self.game_fonts.clear_text();
        let target = crate::ingame_menu::sjk_view::text_target(
            &mut self.game_fonts,
            &mut self.text_vertices,
            &self.ui_font,
        );
        self.emote_picker.card.append(target, viewport);
    }

    /// A key while the picker shows, with the modifiers the console tracks.
    pub(crate) fn emote_picker_key(&mut self, event: &KeyEvent) {
        let (shift, control) = self.console.as_ref().map_or((false, false), |console| {
            (console.shift_held(), console.control_held())
        });
        let choice = self.emote_picker.card.handle_key(event, shift, control);
        self.emote_picker_choice(choice);
    }

    /// Back to play when the picker opened the game menu for itself, or always after
    /// a pick, so the emote is seen; then the pick plays.
    pub(crate) fn emote_picker_choice(&mut self, choice: Option<Choice>) {
        let Some(choice) = choice else {
            return;
        };
        let opened = std::mem::take(&mut self.emote_picker.opened_menu);
        let play = matches!(choice, Choice::Play(_));
        if opened || play {
            self.game_menu = false;
            self.sync_cursor_policy();
        }
        if let Choice::Play(id) = choice
            && let Some(console) = &mut self.console
        {
            let _ = console.queue_command(&format!("{} {id}", crate::emotes::COMMAND));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def(id: &str, name: &str) -> EmoteDef {
        EmoteDef {
            id: id.to_owned(),
            name: name.to_owned(),
            torso: String::new(),
            legs: String::new(),
            length: std::time::Duration::ZERO,
            looping: false,
            sound: String::new(),
            clip: String::new(),
            local: false,
            moving: false,
            icon: String::new(),
            sound_length: std::time::Duration::ZERO,
            sound_loop: String::new(),
            sound_loop_length: std::time::Duration::ZERO,
            rig: String::new(),
        }
    }

    fn card(count: usize) -> Card {
        let catalogue: Vec<EmoteDef> = (0..count)
            .map(|index| def(&format!("emote_{index:02}"), &format!("Emote {index:02}")))
            .collect();
        let mut card = Card::default();
        card.show(&catalogue);
        card
    }

    fn type_text(card: &mut Card, words: &str) {
        for character in words.chars() {
            let typed = character.to_string();
            let _ = card.key(KeyCode::KeyA, Some(&typed), false, false);
        }
    }

    #[test]
    fn the_search_keeps_names_or_ids_holding_every_word() {
        let mut card = Card::default();
        card.show(&[
            def("hot_pink", "Hot Marat"),
            def("floss", "Floss"),
            def("take_the_l", "Take the L"),
        ]);
        assert_eq!(card.shown.len(), 3);
        // By name, the list is sorted.
        assert_eq!(card.entries[card.shown[0]].id, "floss");
        type_text(&mut card, "HOT");
        assert_eq!(card.shown.len(), 1);
        assert_eq!(card.entries[card.shown[0]].id, "hot_pink");
        // Escape clears the search first, then closes.
        assert_eq!(card.key(KeyCode::Escape, None, false, false), None);
        assert_eq!(card.shown.len(), 3);
        type_text(&mut card, "the l");
        assert_eq!(card.shown.len(), 1);
        type_text(&mut card, "x");
        assert!(card.shown.is_empty());
        assert_eq!(
            card.key(KeyCode::Enter, None, false, false),
            None,
            "nothing to play"
        );
        let _ = card.key(KeyCode::Escape, None, false, false);
        assert_eq!(
            card.key(KeyCode::Escape, None, false, false),
            Some(Choice::Close)
        );
        assert!(!card.wants());
    }

    #[test]
    fn enter_plays_the_chosen_tile_and_closes() {
        let mut card = card(20);
        let _ = card.key(KeyCode::ArrowRight, None, false, false);
        let _ = card.key(KeyCode::ArrowDown, None, false, false);
        assert_eq!(card.chosen, 1 + COLUMNS);
        assert_eq!(
            card.key(KeyCode::Enter, None, false, false),
            Some(Choice::Play(format!("emote_{:02}", 1 + COLUMNS)))
        );
        assert!(!card.wants());
    }

    #[test]
    fn moving_past_the_shown_rows_scrolls_and_stops_at_the_ends() {
        let mut card = card(60);
        let _ = card.key(KeyCode::ArrowLeft, None, false, false);
        assert_eq!(card.chosen, 0);
        for _ in 0..ROWS {
            let _ = card.key(KeyCode::ArrowDown, None, false, false);
        }
        assert_eq!(card.first_row, 1, "the chosen row is on screen");
        let _ = card.key(KeyCode::PageDown, None, false, false);
        let _ = card.key(KeyCode::PageDown, None, false, false);
        assert_eq!(card.chosen, 59, "stops on the last");
        assert_eq!(card.first_row, card.rows() - ROWS);
        assert_eq!(card.at_slot(0), Some(card.first_row * COLUMNS));
        assert_eq!(
            card.at_slot(COLUMNS * ROWS - 1),
            None,
            "past the last emote"
        );
    }

    #[test]
    fn long_names_are_cut_to_fit_a_tile() {
        assert_eq!(short("Floss"), "Floss");
        let cut = short("A very long emote name indeed");
        assert_eq!(cut.chars().count(), NAME_CHARS);
        assert!(cut.ends_with('…'));
    }
}
