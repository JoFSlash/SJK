//! Creyon's Blade Lab designs: the fourteen blade skins first drawn in Creyon's Blade Lab
//! (`SJK_Blade_Lab.html`, a three.js page), drawn in game the way the lab drew them.
//!
//! Each skin's file describes it with the skin parts every skin shares (corona, figures,
//! solids), which only come near the lab's look; players compared the two side by side
//! and found them far apart (Creyon's review, 17/10/2026: the serpent's head, the
//! obsidian blade's width, the comic blade's missing outline and words, the bubbles,
//! the monarchs, quantum's copies). So the skins this table names are drawn by the lab's
//! own shader instead, ported to `saber.wgsl` (`lab_*`): the blade, its particles, its
//! solid blade and the comic words, in the lab's units (a stock 40-unit blade is the
//! lab's 1.08 m one). Parts of the lab that grew out of the blade are left out on
//! purpose: quantum's ghost copies at other angles and the phoenix's wings.
//!
//! `cg_saberLabDesigns 0` (Settings > Game options, "Blade Lab designs") draws these
//! skins from their files again.

use crate::blade_skin_file::{BladeSkinDef, Rgb};
use std::sync::atomic::{AtomicBool, Ordering};

/// Archived, default 1: draw the skins of [`DESIGNS`] as Creyon's Blade Lab did.
pub(crate) const CVAR: &str = "cg_saberLabDesigns";
const LOOKUP: &str = "cg_saberlabdesigns";
/// Metres a unit: the lab's blade is 1.08 m, a stock blade 40 units (`saber.wgsl`
/// `LAB_METRE`).
pub(crate) const METRE: f32 = 1.08 / 40.0;

/// One design: the skin it draws, its number in `saber.wgsl` (`LAB_*`), how far its
/// glow reaches and its particles go (metres, the lab's `R` and their flight), whether it
/// has a solid pass (a solid blade, blended particles or words), and the lab's colour,
/// which its trail and light take.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Design {
    pub(crate) id: &'static str,
    pub(crate) number: u32,
    pub(crate) reach: f32,
    pub(crate) room: f32,
    pub(crate) solid: bool,
    pub(crate) color: Rgb,
}

const fn design(
    id: &'static str,
    number: u32,
    reach: f32,
    room: f32,
    solid: bool,
    color: Rgb,
) -> Design {
    Design {
        id,
        number,
        reach,
        room,
        solid,
        color,
    }
}

/// The designs, with the lab's numbers (its `FX` table: `R`, `color`).
pub(crate) const DESIGNS: [Design; 14] = [
    design("saber_peppermint", 1, 0.09, 0.1, false, [1.0, 0.2, 0.25]),
    design("saber_bubble", 2, 0.1, 0.5, false, [0.7, 0.85, 1.0]),
    design("saber_lavalamp", 3, 0.1, 0.16, false, [1.0, 0.45, 0.2]),
    design("saber_sketch", 4, 0.09, 0.16, false, [0.88, 0.92, 1.0]),
    design("saber_hourglass", 5, 0.1, 0.4, false, [1.0, 0.78, 0.35]),
    design("saber_comic", 6, 0.15, 0.6, true, [0.2, 0.7, 1.0]),
    design("saber_quicksilver", 7, 0.09, 0.3, true, [0.82, 0.86, 0.95]),
    design("saber_reactor", 8, 0.1, 0.3, false, [0.45, 0.8, 1.0]),
    design("saber_obsidian", 9, 0.09, 0.12, true, [0.92, 0.95, 1.0]),
    design("saber_bleeding", 10, 0.1, 0.35, true, [1.0, 0.1, 0.12]),
    design("saber_monarch", 11, 0.09, 0.3, true, [1.0, 0.58, 0.14]),
    design("saber_phoenix", 12, 0.13, 0.45, false, [1.0, 0.45, 0.08]),
    design("saber_serpent", 13, 0.11, 0.12, false, [0.15, 1.0, 0.5]),
    design("saber_quantum", 14, 0.1, 0.25, false, [0.35, 0.85, 1.0]),
];

/// Designs whose words outlast their blade (Creyon's round 14: the Clone Wars blade's
/// "VMMMM!" on switching off went with the blade before it could be read): seconds the
/// word keeps floating after the blade has gone in. One entry per design, read by
/// [`linger`]; the blade itself is not drawn meanwhile (`saber.wgsl` `lab_lingering`).
const LINGERS: [(&str, f32); 1] = [("saber_comic", 1.6)];

/// Designs that answer each swing of a chain on its own (the Clone Wars blade's words, one
/// per swing): their swing's start is marked again every second of a chain. The others
/// count a chain as one swing from its first (Creyon's round 14: Felucia's spores fell back
/// onto the blade at every mark).
const EACH_SWING: [&str; 1] = ["saber_comic"];

/// Designs whose effects answer how much the blade moves in the world, running as much as
/// swinging, rather than a swing alone (Creyon's round 16: Felucia's spores rest on the blade
/// while its wearer stands idle, fly off and follow it when they run or swing, and settle back
/// once they stand still again): their motion's last lane carries
/// [`crate::saber_motion::BladeMotion::stir`] instead of the seconds since a swing.
const STIRRED: [&str; 1] = ["saber_monarch"];

/// Whether `id`'s swing trail is comic speed lines (Creyon's round 19: the Clone Wars blade
/// leaves comic lines when it moves, not the blur's afterimage).
pub(crate) fn comic_trail(id: &str) -> bool {
    id == "saber_comic"
}

/// Whether `id` answers the blade's movement in the world ([`STIRRED`]).
pub(crate) fn stirred(id: &str) -> bool {
    STIRRED.contains(&id)
}

/// Whether `id` answers each swing of a chain on its own ([`EACH_SWING`]).
pub(crate) fn each_swing(id: &str) -> bool {
    EACH_SWING.contains(&id)
}

/// How long `id`'s words float on after its blade has gone in (seconds; 0 for none).
pub(crate) fn linger(id: &str) -> f32 {
    LINGERS
        .iter()
        .find(|(design, _)| *design == id)
        .map_or(0.0, |(_, seconds)| *seconds)
}

/// Quantum's number: its ghost copies are an option (`saber.wgsl` `lab_quantum_ghosts`,
/// drawn unless the wearer's `copies` section is off).
const QUANTUM: u32 = 14;
/// The Clone Wars blade's number: its own parts are options (round 19).
const COMIC: u32 = 6;

/// `cg_saberLabDesigns`, as last read ([`set_enabled`]); on until the console says.
static ENABLED: AtomicBool = AtomicBool::new(true);

/// Whether the designs are drawn.
pub(crate) fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Take `cg_saberLabDesigns` from the console; true when it changed (the skins' uniforms
/// are then uploaded again).
pub(crate) fn set_enabled(console: Option<&crate::console::ViewerConsole>) -> bool {
    let on = console
        .and_then(|console| console.bool_cvar(LOOKUP))
        .unwrap_or(true);
    ENABLED.swap(on, Ordering::Relaxed) != on
}

/// The design of skin `id`, if it has one.
pub(crate) fn of(id: &str) -> Option<&'static Design> {
    DESIGNS.iter().find(|design| design.id == id)
}

impl Design {
    /// Its `design` lane in `saber.wgsl`'s `Skin`.
    pub(crate) fn lane(&self) -> [f32; 4] {
        [
            self.number as f32,
            self.reach,
            self.room,
            if self.solid { 1.0 } else { 0.0 },
        ]
    }

    /// `def` as the design draws it: none of the file's sections (the design draws its
    /// own parts, and the CPU's afterimages, echo, glints and pulse go with them), and
    /// the lab's colour for its trail and light. The core and corona stay, unused by the
    /// shader, for a chroma's base hue; the sounds stay the file's.
    pub(crate) fn shown(&self, def: &BladeSkinDef) -> BladeSkinDef {
        let mut shown = bare(def);
        // The design's own parts its wearer may switch off.
        let option =
            |id: &str, name: &str, section: &str| crate::blade_skin_file::extras::SkinOption {
                id: id.to_owned(),
                name: name.to_owned(),
                sections: vec![section.to_owned()],
            };
        // Every design's particles and bursts (on bits it has no other use for: `saber.wgsl`
        // `lab_glow` and `lab_place`), and Quantum's ghost copies.
        shown.options = vec![
            option("particles", "Particles", "arcs"),
            option("bursts", "Swing and clash bursts", "motes"),
        ];
        if self.number == COMIC {
            // Round 19 (Creyon): the Clone Wars blade's own parts, each switched on its own;
            // its particles go with the cartoon smoke and sparks (`saber.wgsl`
            // `lab_comic_*`), on bits a design has no other use for.
            shown.options = vec![
                option("comic_panel", "Comic panel", "sputter"),
                option("action_burst", "Action burst", "glitch"),
                option("clone_markings", "Clone markings", "scan"),
                option("holo_flicker", "Holo flicker", "bands"),
                option("smoke_sparks", "Cartoon smoke and sparks", "arcs"),
                option("bursts", "Swing and clash bursts", "motes"),
            ];
        }
        if self.number == QUANTUM {
            shown
                .options
                .push(option("ghosts", "Ghost copies", "copies"));
        }
        shown.trail = self.color;
        shown.light.color = self.color;
        shown
    }
}

/// `def` without any of its sections (a design or signature draws its own parts, and the
/// CPU's afterimages, echo, glints and pulse go with them) or options. The core, corona,
/// colours and sounds stay.
pub(crate) fn bare(def: &BladeSkinDef) -> BladeSkinDef {
    {
        let mut shown = def.clone();
        shown.arcs = None;
        shown.motes = None;
        shown.hue = None;
        shown.sputter = None;
        shown.glitch = None;
        shown.scan = None;
        shown.pulse = None;
        shown.ghosts = None;
        shown.embers = None;
        shown.veins = None;
        shown.team = None;
        shown.ambient = None;
        shown.glyphs = None;
        shown.star = None;
        shown.orbit = None;
        shown.holes = None;
        shown.wisps = None;
        shown.haze = None;
        shown.echo = None;
        shown.bands = None;
        shown.segments = None;
        shown.film = None;
        shown.blobs = None;
        shown.front = None;
        shown.sketch = None;
        shown.hourglass = None;
        shown.serpent = None;
        shown.strands = None;
        shown.butterflies = None;
        shown.wings = None;
        shown.copies = None;
        shown.ink = None;
        shown.chrome = None;
        shown.options = Vec::new();
        shown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_design_is_a_blade_skin_of_the_catalogue_with_its_own_number() {
        for (index, design) in DESIGNS.iter().enumerate() {
            assert!(
                crate::unlockables::blade_skin(design.id).is_some(),
                "{} is not a blade skin",
                design.id
            );
            assert_eq!(design.number as usize, index + 1);
            assert!(design.reach > 0.0 && design.room >= 0.0);
        }
    }

    #[test]
    fn the_shader_numbers_its_designs_as_the_table_does() {
        let shader = include_str!("saber.wgsl");
        let names = [
            "PEPPERMINT",
            "BUBBLE",
            "LAVALAMP",
            "SKETCH",
            "HOURGLASS",
            "COMIC",
            "QUICKSILVER",
            "REACTOR",
            "OBSIDIAN",
            "BLEEDING",
            "MONARCH",
            "PHOENIX",
            "SERPENT",
            "QUANTUM",
        ];
        for (design, name) in DESIGNS.iter().zip(names) {
            assert!(
                design.id.to_uppercase().ends_with(name),
                "{} is not {name}",
                design.id
            );
            assert!(shader.contains(&format!("const LAB_{name}: u32 = {}u;", design.number)));
            assert!(shader.contains(&format!("case {}u: {{ return lab_", design.number)));
        }
        assert!(shader.contains(&format!("const LAB_METRE: f32 = {METRE};")));
    }
}

#[cfg(test)]
#[path = "blade_lab_designs_gpu_test.rs"]
mod gpu_test;
