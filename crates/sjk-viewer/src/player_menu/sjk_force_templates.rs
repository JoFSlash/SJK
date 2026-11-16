//! SJK Force page selectors for the normal, shared template catalogue.

use super::PlayerMenu;
use super::rows::{FORCE_COMMUNITY_TEMPLATE_ROW, FORCE_RETAIL_TEMPLATE_ROW};
use crate::menu::sjk::{Frame, color, kit};

impl PlayerMenu {
    /// Retail first, then community. Left/Right or either half of a selector
    /// loads a template into the draft; Apply remains the player's choice.
    pub(super) fn sjk_force_templates(&mut self, frame: &Frame) {
        self.force_templates.list.ensure(self.icon_vfs.as_ref());
        let side = self.force.allocation().side;
        let templates = self.force_templates.list.of(side);
        let retail = self.force_templates.list.retail_count(side);
        let chosen = self.template_row();
        let x = 1_384.0;
        let width = 440.0;
        kit::heading_in(
            &mut self.canvas,
            frame,
            x,
            292.0,
            width,
            "Force templates",
            color::GOLD,
        );
        for (row, label, range, top) in [
            (FORCE_RETAIL_TEMPLATE_ROW, "Jedi Academy", 0..retail, 326.0),
            (
                FORCE_COMMUNITY_TEMPLATE_ROW,
                "Community",
                retail..templates.len(),
                430.0,
            ),
        ] {
            kit::heading_in(&mut self.canvas, frame, x, top, width, label, color::MUTED);
            let name = chosen
                .filter(|index| range.contains(index))
                .map(|index| templates[index].name.as_str())
                .unwrap_or(if range.is_empty() {
                    "None installed"
                } else {
                    "Choose a template"
                });
            let area = [x, top + 24.0, width, 44.0];
            kit::cycler(
                &mut self.canvas,
                frame,
                area,
                format_args!("{name}"),
                None,
                self.selected == row,
            );
            if !range.is_empty() {
                self.canvas
                    .hit_region(row as u16, frame.rect(area[0], area[1], area[2], area[3]));
            }
        }
    }
}
