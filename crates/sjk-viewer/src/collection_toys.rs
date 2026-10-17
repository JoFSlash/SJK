//! The Collection's Toys tab: things to use in a match, everyone's. SJK's Illuminate
//! holocron (`docs/client.md`, Illuminate) and the cardboard box (`docs/client.md`,
//! Cardboard box) stand on plinths on the left; the chosen one is described beside the
//! model with the switch that runs it (`toy_illuminate`, `toy_box`). The holocron
//! floats lit by the model while it is chosen.

use super::view::{LEFT_X, STAGE_TEXT_WIDTH, STAGE_TEXT_X, glow, left_lines};
use super::*;
use crate::menu::sjk::{Frame, color, kit, text};
use crate::menu_widgets::TextFamily;
use sjk_ui::{Color, DrawCommand, FontWeight, TextAlign, TextureId};

/// The first toy's niche: its corner and size; the next stands [`NICHE_STEP`] to the
/// right.
const NICHE: [f32; 3] = [LEFT_X, 300.0, 260.0];
const NICHE_STEP: f32 = 300.0;

/// One toy as the tab shows it.
struct Toy {
    /// Its quick wheel icon's name (the pictures are uploaded into the atlas at start).
    icon: &'static str,
    /// Under its niche.
    name: &'static str,
    /// Beside the model.
    title: &'static str,
    what: &'static str,
    how: &'static str,
    /// The switch's words.
    switch: &'static str,
    token: u16,
}

/// The toys, in the order of [`Panel::toy`].
const TOYS: [Toy; TOY_COUNT] = [
    Toy {
        icon: "illuminate",
        name: "Illuminate",
        title: "Illuminate holocron",
        what: "A holocron floats by your left shoulder and lights the way in dark maps. Other SlashJK players on your server see it lit.",
        how: "Bind a key to toy_illuminate, or use the quick wheel's Toys page.",
        switch: "Holocron lit",
        token: TOY_TOKEN,
    },
    Toy {
        icon: "cardboard_box",
        name: "Cardboard box",
        title: "Cardboard box",
        what: "Hide in a carton: you crouch in it with its music. Other SlashJK players see the box; one who walks up and presses Use lifts it. Die in it and everyone hears.",
        how: "Bind a key to toy_box, or use the quick wheel's Toys page.",
        switch: "In the box",
        token: BOX_TOKEN,
    },
];

/// A toy's picture: the quick wheel's, uploaded into the atlas at start.
fn icon(name: &str) -> TextureId {
    crate::ui_renderer::wheel_icon(
        crate::quick_wheel::catalog::icon_index(name)
            .expect("the quick wheel's icons hold every toy's"),
    )
}

/// A row of dots across the left side at `y`: the room for more.
pub(super) fn room_for_more(
    canvas: &mut crate::menu_widgets::MenuCanvas,
    frame: &Frame,
    y: f32,
    words: &str,
) {
    let mut x = LEFT_X;
    while x < LEFT_X + 820.0 {
        let _ = canvas.draw_list_mut().push(DrawCommand::SolidRect {
            rect: frame.rect(x, y, 8.0, 1.0),
            color: color::alpha(color::LINE, 0.22),
        });
        x += 16.0;
    }
    text(
        canvas,
        TextFamily::Body,
        format_args!("{words}"),
        frame.rect(LEFT_X, y + 16.0, 820.0, 24.0),
        15.0 * frame.s,
        color::QUIET,
        FontWeight::Regular,
        TextAlign::Start,
    );
}

impl Panel {
    /// The Toys tab.
    pub(super) fn toys(&mut self, frame: &Frame, inputs: &Inputs<'_>) {
        self.lead(
            frame,
            "2 toys",
            "Things to use in a match, from the quick wheel or a key.",
        );
        let chosen = self.toy.min(TOY_COUNT - 1);
        for (index, toy) in TOYS.iter().enumerate() {
            self.toy_niche(frame, toy, index as f32 * NICHE_STEP, index == chosen);
        }
        room_for_more(
            &mut self.ui,
            frame,
            716.0,
            "More toys will stand here as SlashJK adds them.",
        );
        let on = if chosen == 1 {
            inputs.cardboard_box
        } else {
            inputs.illuminate
        };
        self.toy_beside_model(frame, &TOYS[chosen], on);
    }

    /// A toy's plinth, `offset` right of the first: brighter when it is the one chosen.
    fn toy_niche(&mut self, frame: &Frame, toy: &Toy, offset: f32, chosen: bool) {
        let s = frame.s;
        let [x, y, size] = NICHE;
        let x = x + offset;
        let strength = if chosen { 1.0 } else { 0.45 };
        glow(
            &mut self.ui,
            frame,
            [x + size * 0.5, y + size * 0.55],
            size * 0.62,
            color::ACCENT,
            0.42 * strength,
        );
        let rect = frame.rect(x, y, size, size);
        let _ = self.ui.draw_list_mut().push(DrawCommand::Border {
            rect,
            radius: 14.0 * s,
            width: 2.0 * s,
            color: if chosen {
                color::ACCENT_BRIGHT
            } else {
                color::alpha(color::ACCENT, 0.5)
            },
        });
        let art = size * 0.72;
        let _ = self.ui.draw_list_mut().push(DrawCommand::TexturedQuad {
            rect: frame.rect(x + (size - art) * 0.5, y + (size - art) * 0.5, art, art),
            texture: icon(toy.icon),
            color: Color::new(1.0, 1.0, 1.0, 0.55 + 0.45 * strength),
        });
        let _ = self.ui.draw_list_mut().push(DrawCommand::SolidRect {
            rect: frame.rect(x, y + size + 16.0, size, 2.0),
            color: color::alpha(color::ACCENT, strength),
        });
        text(
            &mut self.ui,
            TextFamily::Display,
            format_args!("{}", toy.name),
            frame.rect(x, y + size + 30.0, size, 30.0),
            24.0 * s,
            if chosen {
                color::ACCENT_BRIGHT
            } else {
                color::ACCENT
            },
            FontWeight::Semibold,
            TextAlign::Center,
        );
        text(
            &mut self.ui,
            TextFamily::Body,
            format_args!("Everyone's"),
            frame.rect(x, y + size + 62.0, size, 22.0),
            15.0 * s,
            color::ACCENT,
            FontWeight::Regular,
            TextAlign::Center,
        );
        self.ui.hit_region(toy.token, rect);
    }

    /// What the chosen toy is, beside the model, and its switch (`on`: it is on now).
    fn toy_beside_model(&mut self, frame: &Frame, toy: &Toy, on: bool) {
        let s = frame.s;
        let x = STAGE_TEXT_X;
        let width = STAGE_TEXT_WIDTH;
        if self.backstage == Backstage::None {
            glow(
                &mut self.ui,
                frame,
                [x + 110.0, 400.0],
                150.0,
                color::ACCENT,
                0.3,
            );
            let _ = self.ui.draw_list_mut().push(DrawCommand::TexturedQuad {
                rect: frame.rect(x + 10.0, 300.0, 200.0, 200.0),
                texture: icon(toy.icon),
                color: Color::new(1.0, 1.0, 1.0, 1.0),
            });
        }
        let mut y = 618.0;
        text(
            &mut self.ui,
            TextFamily::Body,
            format_args!("Toy"),
            frame.rect(x, y, width, 22.0),
            16.0 * s,
            color::LINE,
            FontWeight::Semibold,
            TextAlign::Start,
        );
        y += 26.0;
        text(
            &mut self.ui,
            TextFamily::Display,
            format_args!("{}", toy.title),
            frame.rect(x, y, width, 62.0),
            52.0 * s,
            color::ACCENT_BRIGHT,
            FontWeight::Semibold,
            TextAlign::Start,
        );
        y += 70.0;
        y = left_lines(
            &mut self.ui,
            frame,
            toy.what,
            [x, y, width],
            (52, 3),
            18.0,
            color::TEXT,
            TextAlign::Start,
        );
        y += 4.0;
        y = left_lines(
            &mut self.ui,
            frame,
            toy.how,
            [x, y, width],
            (60, 2),
            16.0,
            color::MUTED,
            TextAlign::Start,
        );
        y += 18.0;
        text(
            &mut self.ui,
            TextFamily::Display,
            format_args!("{}", toy.switch),
            frame.rect(x, y, 300.0, 30.0),
            22.0 * s,
            color::TEXT,
            FontWeight::Semibold,
            TextAlign::Start,
        );
        let right = x + 330.0;
        let left = kit::switch(&mut self.ui, frame, right, y + 15.0, on, true);
        self.ui.hit_region(
            TOY_SWITCH_TOKEN,
            frame.rect(left - 4.0, y - 4.0, right - left + 8.0, 38.0),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each toy, its words and the switch fit over the keys and answer the pointer,
    /// with and without a model, at 1080 lines, 4K, 4:3 and 21:9.
    #[test]
    fn the_toys_fit_over_the_keys() {
        let font = crate::text::load_modern(1.0, None).expect("Inter").font;
        for viewport in [
            [1_920.0, 1_080.0],
            [3_840.0, 2_160.0],
            [1_440.0, 1_080.0],
            [2_560.0, 1_080.0],
        ] {
            for backstage in [Backstage::None, Backstage::Preview { ready: true }] {
                for toy in 0..TOY_COUNT {
                    let inputs = super::super::tests::inputs(unlockables::Holdings::Known(&[]), "");
                    let mut panel = Panel::new();
                    panel.open(Tab::Toys, true, true, ReturnTarget::InGame);
                    panel.set_backstage(backstage);
                    panel.toy = toy;
                    panel.build(&inputs, &font, viewport);
                    assert!(!panel.ui.overflowed(), "{viewport:?}");
                    let frame = Frame::new(viewport);
                    let keys = frame.point(0.0, super::super::view::KEYS_Y)[1];
                    for token in [TOY_TOKEN, BOX_TOKEN, TOY_SWITCH_TOKEN] {
                        let area = panel.ui.rect_for(token).expect("a target");
                        assert!(area.bottom() <= keys, "{viewport:?}");
                    }
                    for command in panel.ui.draw_list().commands() {
                        if let DrawCommand::Text { rect, .. } = command {
                            assert!(rect.bottom() <= keys || rect.y >= keys, "{rect:?}");
                        }
                    }
                }
            }
        }
    }
}
