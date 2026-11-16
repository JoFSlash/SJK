//! SJK dialogue presentation: bounded text and five scrolling response rows.
use super::Panel;
use crate::menu::sjk::{self, Frame, color};
use crate::menu_widgets::{BACK_TOKEN, TextFamily};
use sjk_ui::{FontWeight, TextAlign};

impl Panel {
    pub(super) fn build(&mut self, viewport: [f32; 2]) {
        let frame = Frame::new(viewport);
        self.canvas.begin_transparent(viewport);
        // Use the SJK kit's world backdrop and typography, independent of menu style.
        crate::settings::sjk_view::backdrop(&mut self.canvas, viewport);
        sjk::top_bar(
            &mut self.canvas,
            &frame,
            "Leave conversation",
            BACK_TOKEN,
            "Conversation",
            None,
        );
        sjk::text(
            &mut self.canvas,
            TextFamily::Display,
            format_args!("{}", self.node.speaker),
            frame.rect(180.0, 520.0, 1560.0, 50.0),
            36.0 * frame.s,
            color::GOLD,
            FontWeight::Semibold,
            TextAlign::Start,
        );
        for (line, text) in self
            .node
            .text
            .lines()
            .flat_map(|line| sjk::wrap(line, 105))
            .take(3)
            .enumerate()
        {
            sjk::text(
                &mut self.canvas,
                TextFamily::Body,
                format_args!("{text}"),
                frame.rect(180.0, 580.0 + line as f32 * 33.0, 1560.0, 33.0),
                26.0 * frame.s,
                color::TEXT,
                FontWeight::Regular,
                TextAlign::Start,
            );
        }
        let first = self
            .node
            .selected
            .saturating_sub(4)
            .min(self.node.count.saturating_sub(5));
        for index in first..(first + 5).min(self.node.count) {
            let y = 710.0 + (index - first) as f32 * 48.0;
            let rect = frame.rect(180.0, y, 1560.0, 44.0);
            self.canvas.hit_region(index as u16 + 1, rect);
            if index == self.node.selected {
                self.canvas
                    .accent_bar(frame.rect(168.0, y + 6.0, 3.0, 30.0), color::GOLD);
            }
            sjk::text(
                &mut self.canvas,
                TextFamily::Body,
                format_args!("{}. {}", index + 1, self.node.choices[index]),
                rect,
                25.0 * frame.s,
                if index == self.node.selected {
                    color::GOLD
                } else {
                    color::TEXT
                },
                FontWeight::Regular,
                TextAlign::Start,
            );
        }
        sjk::text(
            &mut self.canvas,
            TextFamily::Body,
            format_args!(
                "Arrows / wheel to choose    Enter to answer    Esc to leave    {} / {}",
                self.node.selected + 1,
                self.node.count
            ),
            frame.rect(180.0, 990.0, 1560.0, 35.0),
            20.0 * frame.s,
            color::MUTED,
            FontWeight::Regular,
            TextAlign::Start,
        );
        self.canvas.finish(self.node.selected as u16 + 1);
    }
}
