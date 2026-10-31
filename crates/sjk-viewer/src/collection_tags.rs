//! The Collection's Tags tab (27/10/2026): the tags a player sticks on walls with
//! `toy_tag` (`tags.rs`). Every tag of the catalogue stands in a wall of tiles on the
//! left, its picture over its name and its tier's colour, the ones the player lacks
//! dimmed and the one `cg_tag` names marked; the tag chosen stands large on a card on the
//! right, with how it is got and its Choose button, which sets `cg_tag`.

use super::view::LEFT_X;
use super::*;
use crate::menu::sjk::{Frame, color, kit, text};
use crate::menu_widgets::TextFamily;
use sjk_ui::{Color, DrawCommand, FontWeight, TextAlign};

/// The wall: its columns, a tile's size and the room between tiles, and its top.
const COLUMNS: usize = 5;
const TILE_WIDTH: f32 = 168.0;
const TILE_HEIGHT: f32 = 196.0;
const TILE_GAP: f32 = 18.0;
const WALL_Y: f32 = 304.0;
/// The chosen tag's card: its left edge, width, top and foot, and the room inside it.
const CARD_X: f32 = 1_180.0;
const CARD_WIDTH: f32 = super::view::RIGHT_X - CARD_X;
const CARD_TOP: f32 = 304.0;
const CARD_BOTTOM: f32 = 990.0;
const CARD_PAD: f32 = 28.0;
/// Where new tags come from: proposals on SJK's Discord, which Sol picks from
/// (`docs/client.md`, "Tag").
const PROPOSE: &str = "New tag ideas: SJK's Discord, #tag-proposals";

/// How many copies of tag `tag` the player holds: the free ones count one, a dropped one
/// as many as the hub lists (0 when not held, or when the hub is not known).
fn held(inputs: &Inputs<'_>, tag: &crate::tags::Tag) -> u32 {
    if tag.free {
        return 1;
    }
    inputs
        .holdings
        .unlock(tag.id)
        .map_or(0, |unlock| unlock.count.max(1))
}

impl Panel {
    /// Whether Enter (or Choose) picks the tag chosen: one the player holds, not already
    /// the one `cg_tag` names.
    pub(super) fn tag_choosable(&self, inputs: &Inputs<'_>) -> bool {
        crate::tags::CATALOGUE
            .get(self.tag)
            .is_some_and(|tag| held(inputs, tag) > 0 && tag.id != inputs.tag.trim())
    }

    /// Enter, or the card's Choose button: pick the tag chosen for `toy_tag`. A tag the
    /// player lacks is not picked; `toy_tag` would refuse it.
    pub(super) fn choose_tag(&self) -> PanelAction {
        match crate::tags::CATALOGUE.get(self.tag) {
            Some(tag) if self.tag_held.get(self.tag).copied().unwrap_or(false) => {
                PanelAction::ChooseTag(tag.id)
            }
            _ => PanelAction::None,
        }
    }

    /// The Tags tab.
    pub(super) fn tags(&mut self, frame: &Frame, inputs: &Inputs<'_>) {
        let total = crate::tags::CATALOGUE.len();
        let counts: Vec<u32> = crate::tags::CATALOGUE
            .iter()
            .map(|tag| held(inputs, tag))
            .collect();
        self.tag_held = counts.iter().map(|count| *count > 0).collect();
        let owned = self.tag_held.iter().filter(|held| **held).count();
        let line = match inputs.holdings.reason() {
            Some(reason) => reason.to_owned(),
            None => format!(
                "{owned} of {total} held. Holocrons drop more; stick yours with toy_tag (the quick wheel's Toys, or a key)."
            ),
        };
        self.lead(frame, &format!("{total} tags"), &line);
        self.tag = self.tag.min(total - 1);
        let chosen_setting = inputs.tag.trim();
        let chosen_setting = if chosen_setting.is_empty() {
            crate::tags::DEFAULT
        } else {
            chosen_setting
        };
        for (index, tag) in crate::tags::CATALOGUE.iter().enumerate() {
            let column = index % COLUMNS;
            let row = index / COLUMNS;
            let x = LEFT_X + column as f32 * (TILE_WIDTH + TILE_GAP);
            let y = WALL_Y + row as f32 * (TILE_HEIGHT + TILE_GAP);
            self.tag_tile(
                frame,
                index,
                tag,
                [x, y],
                counts[index],
                index == self.tag,
                tag.id == chosen_setting,
            );
        }
        self.tag_card(frame, counts[self.tag], chosen_setting);
    }

    /// Tag `index`'s tile at `[x, y]`: its picture (its name while the pack is not
    /// here), its name and tier; dimmed when not held (`count` 0), "x2" over a duplicate,
    /// framed gold when chosen here, and marked when it is the one `toy_tag` sticks
    /// (`picked`).
    #[allow(clippy::too_many_arguments)]
    fn tag_tile(
        &mut self,
        frame: &Frame,
        index: usize,
        tag: &crate::tags::Tag,
        [x, y]: [f32; 2],
        count: u32,
        chosen: bool,
        picked: bool,
    ) {
        let s = frame.s;
        let token = TAG_BASE + index as u16;
        let hovered = self.ui.token_hovered(token);
        let tier = tag.tier.colour();
        let area = frame.rect(x, y, TILE_WIDTH, TILE_HEIGHT);
        let _ = self.ui.draw_list_mut().push(DrawCommand::RoundedRect {
            rect: area,
            radius: 12.0 * s,
            color: if chosen {
                color::alpha(color::GOLD, 0.09)
            } else {
                color::alpha(color::SPACE, if hovered { 0.78 } else { 0.6 })
            },
        });
        let _ = self.ui.draw_list_mut().push(DrawCommand::RoundedRect {
            rect: frame.rect(x + 16.0, y, TILE_WIDTH - 32.0, 3.0),
            radius: 1.5 * s,
            color: Color::new(tier.r, tier.g, tier.b, if count > 0 { 1.0 } else { 0.4 }),
        });
        let _ = self.ui.draw_list_mut().push(DrawCommand::Border {
            rect: area,
            radius: 12.0 * s,
            width: if chosen { 1.6 } else { 1.0 } * s,
            color: if chosen {
                color::GOLD_BRIGHT
            } else {
                color::alpha(color::HOLO, if hovered { 0.35 } else { 0.16 })
            },
        });
        let art = 120.0;
        let art_rect = [x + (TILE_WIDTH - art) * 0.5, y + 16.0, art, art];
        let alpha = if count > 0 { 1.0 } else { 0.32 };
        self.tag_picture(frame, tag, art_rect, alpha, 15.0);
        text(
            &mut self.ui,
            TextFamily::Display,
            format_args!("{}", tag.name.trim_end_matches(" tag")),
            frame.rect(x + 8.0, y + 142.0, TILE_WIDTH - 16.0, 24.0),
            17.0 * s,
            if chosen {
                color::GOLD_BRIGHT
            } else if count > 0 {
                color::TEXT
            } else {
                color::QUIET
            },
            FontWeight::Semibold,
            TextAlign::Center,
        );
        let status = if picked {
            "In use".to_owned()
        } else if count == 0 {
            "Locked".to_owned()
        } else if count > 1 {
            format!("{}  ·  x{count}", tag.tier.label())
        } else {
            tag.tier.label().to_owned()
        };
        text(
            &mut self.ui,
            TextFamily::Body,
            format_args!("{status}"),
            frame.rect(x + 8.0, y + 168.0, TILE_WIDTH - 16.0, 20.0),
            14.0 * s,
            if picked {
                color::HOLO
            } else {
                Color::new(tier.r, tier.g, tier.b, if count > 0 { 0.9 } else { 0.5 })
            },
            FontWeight::Regular,
            TextAlign::Center,
        );
        self.ui
            .hit_region(token, frame.rect(x, y, TILE_WIDTH, TILE_HEIGHT));
    }

    /// Tag `tag`'s picture over `[x, y, width, height]` at `alpha`, or its name in its
    /// tier's colour (`size`) while the hub's pack has not brought the picture.
    fn tag_picture(
        &mut self,
        frame: &Frame,
        tag: &crate::tags::Tag,
        [x, y, width, height]: [f32; 4],
        alpha: f32,
        size: f32,
    ) {
        match crate::tag_icons::texture(tag.id) {
            Some(texture) => {
                let _ = self.ui.draw_list_mut().push(DrawCommand::TexturedQuad {
                    rect: frame.rect(x, y, width, height),
                    texture,
                    color: Color::new(1.0, 1.0, 1.0, alpha),
                });
            }
            None => {
                let tier = tag.tier.colour();
                let _ = self.ui.draw_list_mut().push(DrawCommand::Border {
                    rect: frame.rect(x, y, width, height),
                    radius: 10.0 * frame.s,
                    width: 1.2 * frame.s,
                    color: Color::new(tier.r, tier.g, tier.b, 0.5 * alpha),
                });
                text(
                    &mut self.ui,
                    TextFamily::Display,
                    format_args!("{}", tag.name.trim_end_matches(" tag")),
                    frame.rect(x, y + height * 0.5 - size * 0.7, width, size * 1.4),
                    size * frame.s,
                    Color::new(tier.r, tier.g, tier.b, alpha),
                    FontWeight::Semibold,
                    TextAlign::Center,
                );
            }
        }
    }

    /// The chosen tag on its card: large, its name, tier and how it is got, and Choose
    /// (`picked` is the one `toy_tag` sticks now; `count` the copies held).
    fn tag_card(&mut self, frame: &Frame, count: u32, picked: &str) {
        let s = frame.s;
        let tag = &crate::tags::CATALOGUE[self.tag];
        let tier = tag.tier.colour();
        let height = CARD_BOTTOM - CARD_TOP;
        let area = frame.rect(CARD_X, CARD_TOP, CARD_WIDTH, height);
        let _ = self.ui.draw_list_mut().push(DrawCommand::RoundedRect {
            rect: area,
            radius: 16.0 * s,
            color: color::alpha(color::SPACE, 0.88),
        });
        let _ = self.ui.draw_list_mut().push(DrawCommand::RoundedRect {
            rect: frame.rect(CARD_X + 24.0, CARD_TOP, CARD_WIDTH - 48.0, 3.0),
            radius: 1.5 * s,
            color: tier,
        });
        let _ = self.ui.draw_list_mut().push(DrawCommand::Border {
            rect: area,
            radius: 16.0 * s,
            width: 1.2 * s,
            color: color::alpha(color::HOLO, 0.3),
        });
        let x = CARD_X + CARD_PAD;
        let width = CARD_WIDTH - CARD_PAD * 2.0;
        let mut y = CARD_TOP + CARD_PAD;
        text(
            &mut self.ui,
            TextFamily::Body,
            format_args!("Tag  ·  {}", tag.tier.label()),
            frame.rect(x, y, width, 22.0),
            16.0 * s,
            tier,
            FontWeight::Semibold,
            TextAlign::Start,
        );
        y += 26.0;
        text(
            &mut self.ui,
            TextFamily::Display,
            format_args!("{}", tag.name),
            frame.rect(x, y, width, 48.0),
            38.0 * s,
            color::GOLD_BRIGHT,
            FontWeight::Semibold,
            TextAlign::Start,
        );
        y += 60.0;
        // The picture is a 128-pixel atlas cell: no larger, or it blurs.
        let art = 224.0_f32.min(width);
        super::view::glow(
            &mut self.ui,
            frame,
            [CARD_X + CARD_WIDTH * 0.5, y + art * 0.5],
            art * 0.55,
            tier,
            0.22,
        );
        let alpha = if count > 0 { 1.0 } else { 0.4 };
        self.tag_picture(
            frame,
            tag,
            [CARD_X + (CARD_WIDTH - art) * 0.5, y, art, art],
            alpha,
            30.0,
        );
        y += art + 22.0;
        let held_line = match count {
            0 => "You do not hold it yet.".to_owned(),
            1 => "Yours.".to_owned(),
            count => format!("Yours, {count} copies."),
        };
        for (words, colour) in [
            (tag.how_to_get().to_owned(), color::TEXT),
            (held_line, color::MUTED),
            (PROPOSE.to_owned(), color::QUIET),
        ] {
            text(
                &mut self.ui,
                TextFamily::Body,
                format_args!("{words}"),
                frame.rect(x, y, width, 24.0),
                17.0 * s,
                colour,
                FontWeight::Regular,
                TextAlign::Start,
            );
            y += 30.0;
        }
        let in_use = tag.id == picked;
        let label = if in_use {
            "In use"
        } else if count > 0 {
            "Choose"
        } else {
            "Locked"
        };
        let button = [x, CARD_BOTTOM - CARD_PAD - 48.0, 200.0, 48.0];
        kit::button(
            &mut self.ui,
            frame,
            button,
            label,
            true,
            count > 0 && !in_use,
            false,
            TAG_CHOOSE_TOKEN,
        );
        let [bx, by, bw, _] = button;
        text(
            &mut self.ui,
            TextFamily::Body,
            format_args!("toy_tag sticks it where you look"),
            frame.rect(bx + bw + 18.0, by + 12.0, width - bw - 18.0, 24.0),
            15.0 * s,
            color::QUIET,
            FontWeight::Regular,
            TextAlign::Start,
        );
    }
}
