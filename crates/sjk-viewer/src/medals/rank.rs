//! Medal ranks (`docs/identity.md`, "Medal ranks"; Sol's decision, 27/10/2026): every
//! medal a player holds has a rank from I to V, set by hand by the SJK team, which says
//! how far they went in what the medal is for (a Tester at rank V is a top tester). The
//! hub sends it with the medal (`rank`, 1 from hubs before ranks).
//!
//! Each rank keeps the lower ranks' looks and adds one, in the medal's own colour
//! ([`Medal::glow`]) where it makes sense:
//!
//! - I Issued: the picture as it is.
//! - II Polished: a band of metal light sweeping across the medal every few seconds,
//!   on the Legendary sheen's clock ([`crate::rarity_fx::sheen_travel`]), drawn as the
//!   picture's own strips brighter so it keeps to the medal's shape
//!   ([`crate::medal_popup::award::band`]).
//! - III Engraved: the medal leans slightly towards the pointer, as a card does
//!   ([`crate::cards::look::tilt_matrix`]), a soft glare follows it over the medallion
//!   ([`crate::tile_shine`]) and a faint rim of its colour breathes round it. Without a
//!   pointer on it, it sways gently by itself.
//! - IV Holo: the cards' Holo rainbow foil over the medallion
//!   ([`sjk_ui::ShineKind::Holo`]).
//! - V Solar: the cards' Solar corona burning round the medallion's rim
//!   ([`sjk_ui::ShineKind::Solar`]), and Mythical's motes rising over it
//!   ([`crate::rarity_fx::rising_motes`]).
//!
//! These play only where a medal is drawn large ([`draw_art`]); with
//! `ui_rarityEffects 0` the medal stays still and wears its rank's device instead. Small
//! places (ribbon bars, medallions under about 64 pixels) show the device: none for I,
//! a bronze, silver or gold pip for II to IV and a small sun for V, as a real ribbon's
//! devices ([`device`]). Text names a rank "Rank III Engraved" ([`Rank::title`]).

use super::Medal;
use crate::medal_popup::award::{MEDALLION, MEDALLION_RADIUS};
use crate::tile_shine::Hover;
use sjk_ui::{Color, DrawCommand, DrawList, Rect, ShineKind};
use std::f32::consts::{PI, TAU};

/// A medal's rank, I to V.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum Rank {
    #[default]
    Issued,
    Polished,
    Engraved,
    Holo,
    Solar,
}

/// The rim's breath, seconds.
const RIM_BREATH: f32 = 3.6;
/// How far a rank III medal leans, against a card's lean.
const LEAN: f32 = 0.7;
/// The sway of a rank III medal no pointer rests on: seconds of each way's swing, and
/// how far (shares of the medal from its middle).
const SWAY_SECONDS: [f32; 2] = [9.0, 13.0];
const SWAY: [f32; 2] = [0.55, 0.35];
const SWAY_HOLD: f32 = 0.45;
/// The device on the whole picture's ribbon when the effects are off: its middle and
/// its size, shares of the picture's side.
const ART_DEVICE: [f32; 2] = [0.5, 0.2];
const ART_DEVICE_SIZE: f32 = 0.11;

impl Rank {
    pub(crate) const COUNT: usize = 5;
    /// Every rank, lowest first.
    pub(crate) const ALL: [Self; Self::COUNT] = [
        Self::Issued,
        Self::Polished,
        Self::Engraved,
        Self::Holo,
        Self::Solar,
    ];

    /// The rank the hub's number names, brought into 1..=5.
    pub(crate) fn from_wire(rank: u8) -> Self {
        Self::ALL[usize::from(rank.clamp(1, 5) - 1)]
    }

    /// Its number, 1 to 5 (the hub's).
    pub(crate) const fn number(self) -> u8 {
        self as u8 + 1
    }

    /// Its Roman numeral.
    pub(crate) const fn numeral(self) -> &'static str {
        match self {
            Self::Issued => "I",
            Self::Polished => "II",
            Self::Engraved => "III",
            Self::Holo => "IV",
            Self::Solar => "V",
        }
    }

    /// Its name.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Issued => "Issued",
            Self::Polished => "Polished",
            Self::Engraved => "Engraved",
            Self::Holo => "Holo",
            Self::Solar => "Solar",
        }
    }

    /// How text names it: "Rank III Engraved", everywhere a medal's rank is written.
    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::Issued => "Rank I Issued",
            Self::Polished => "Rank II Polished",
            Self::Engraved => "Rank III Engraved",
            Self::Holo => "Rank IV Holo",
            Self::Solar => "Rank V Solar",
        }
    }

    /// The next rank up, if there is one.
    pub(crate) fn next(self) -> Option<Self> {
        Self::ALL.get(self as usize + 1).copied()
    }
}

/// `colour` at `alpha`.
fn with_alpha(colour: Color, alpha: f32) -> Color {
    Color::new(colour.r, colour.g, colour.b, alpha.clamp(0.0, 1.0))
}

/// `colour` brought `amount` of the way to white.
fn lit(colour: Color, amount: f32) -> Color {
    Color::new(
        colour.r + (1.0 - colour.r) * amount,
        colour.g + (1.0 - colour.g) * amount,
        colour.b + (1.0 - colour.b) * amount,
        colour.a,
    )
}

/// A disc of `radius` round `centre`.
fn disc(push: &mut impl FnMut(DrawCommand), centre: [f32; 2], radius: f32, colour: Color) {
    push(DrawCommand::RoundedRect {
        rect: Rect::new(
            centre[0] - radius,
            centre[1] - radius,
            radius * 2.0,
            radius * 2.0,
        ),
        radius,
        color: colour,
    });
}

/// The metals of the devices: bronze, silver and gold, and the sun's gold.
const BRONZE: Color = Color::new(0.80, 0.50, 0.24, 1.0);
const SILVER: Color = Color::new(0.82, 0.85, 0.90, 1.0);
const GOLD: Color = Color::new(0.98, 0.78, 0.30, 1.0);
const SUN: Color = Color::new(1.0, 0.86, 0.36, 1.0);
const EDGE: Color = Color::new(0.06, 0.05, 0.04, 0.75);

/// Rays round the sun device.
const SUN_RAYS: usize = 8;

/// The most commands one device draws.
#[cfg(test)]
const DEVICE_COMMANDS: usize = 3 + SUN_RAYS;

/// `rank`'s device, `size` pixels across, its middle at `centre`, to `push`: nothing for
/// Issued, a bronze, silver or gold pip for Polished, Engraved and Holo, a small sun for
/// Solar. A dark edge keeps it apart from any ribbon.
pub(crate) fn device(rank: Rank, centre: [f32; 2], size: f32, mut push: impl FnMut(DrawCommand)) {
    let radius = size * 0.5;
    if radius < 0.5 {
        return;
    }
    let metal = match rank {
        Rank::Issued => return,
        Rank::Polished => BRONZE,
        Rank::Engraved => SILVER,
        Rank::Holo => GOLD,
        Rank::Solar => SUN,
    };
    if rank == Rank::Solar {
        // The rays, then the sun's disc over them: everything within `radius`.
        let width = (size * 0.12).max(1.0);
        let reach = radius - width * 0.5;
        for ray in 0..SUN_RAYS {
            let angle = TAU * ray as f32 / SUN_RAYS as f32 - PI * 0.5;
            let (sin, cos) = angle.sin_cos();
            push(DrawCommand::Line {
                from: [
                    centre[0] + cos * radius * 0.4,
                    centre[1] + sin * radius * 0.4,
                ],
                to: [centre[0] + cos * reach, centre[1] + sin * reach],
                width,
                color: metal,
            });
        }
        disc(&mut push, centre, radius * 0.56, EDGE);
        disc(&mut push, centre, radius * 0.46, metal);
        disc(
            &mut push,
            [centre[0] - radius * 0.12, centre[1] - radius * 0.12],
            radius * 0.18,
            lit(metal, 0.7),
        );
        return;
    }
    disc(&mut push, centre, radius, EDGE);
    disc(&mut push, centre, radius * 0.78, metal);
    disc(
        &mut push,
        [centre[0] - radius * 0.2, centre[1] - radius * 0.2],
        radius * 0.3,
        lit(metal, 0.6),
    );
}

/// `rank`'s device on a ribbon bar at `ribbon`, to `push`: in its middle, as tall as most
/// of it.
pub(crate) fn ribbon_device(rank: Rank, ribbon: Rect, push: impl FnMut(DrawCommand)) {
    let size = (ribbon.height * 0.78).min(ribbon.width * 0.6);
    device(
        rank,
        [
            ribbon.x + ribbon.width * 0.5,
            ribbon.y + ribbon.height * 0.5,
        ],
        size,
        push,
    );
}

/// `rank`'s device on a medallion's small picture at `icon`, to `list`: at its lower
/// right, as a badge.
pub(crate) fn icon_device(list: &mut DrawList, rank: Rank, icon: Rect) {
    let size = (icon.width * 0.42).max(6.0);
    device(
        rank,
        [icon.right() - size * 0.42, icon.bottom() - size * 0.42],
        size,
        |command| {
            let _ = list.push(command);
        },
    );
}

/// A medal drawn whole, large, at its rank.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Art {
    pub(crate) medal: Medal,
    pub(crate) rank: Rank,
    /// The whole picture's square, window pixels.
    pub(crate) rect: Rect,
    /// Opacity, 0 to 1.
    pub(crate) alpha: f32,
    /// The pointer resting on it, eased ([`crate::tile_shine::HoverEase`]); `None` when
    /// the place does not follow the pointer or it is away.
    pub(crate) hover: Option<Hover>,
    /// The page's clock, for the motion.
    pub(crate) seconds: f32,
}

impl Art {
    /// `medal` at `rank` over `rect`, opaque, with no pointer on it.
    pub(crate) fn new(medal: Medal, rank: Rank, rect: Rect, seconds: f32) -> Self {
        Self {
            medal,
            rank,
            rect,
            alpha: 1.0,
            hover: None,
            seconds,
        }
    }

    /// The medallion's middle and radius in the picture.
    fn medallion(&self) -> ([f32; 2], f32) {
        (
            [
                self.rect.x + self.rect.width * MEDALLION[0],
                self.rect.y + self.rect.height * MEDALLION[1],
            ],
            self.rect.width * MEDALLION_RADIUS,
        )
    }
}

/// The most commands [`draw_art`] draws (the tests hold every rank to it).
#[cfg(test)]
const ART_COMMANDS: usize = 4 // opacity, transform and their ends
    + 1 // the picture
    + 2 // the rim
    + 1 // the corona
    + crate::medal_popup::award::SHINE_STRIPS // the sweep
    + 2 // the foil and the glare
    + 4; // the motes

/// Draw `art`: the medal's whole picture and, with the rarity effects on, its rank's
/// light (nothing more for Issued); with them off, the picture still and its rank's
/// device on its ribbon.
pub(crate) fn draw_art(list: &mut DrawList, art: Art) {
    draw_art_as(list, art, crate::rarity_fx::enabled());
}

/// [`draw_art`] with the effects `on` or off (tests and shots, and the setting's own
/// check).
pub(crate) fn draw_art_as(list: &mut DrawList, art: Art, on: bool) {
    let rect = art.rect;
    if rect.width <= 1.0 || art.alpha <= 0.0 {
        return;
    }
    let faded = art.alpha < 1.0;
    if faded {
        let _ = list.push(DrawCommand::PushOpacity(art.alpha));
    }
    let picture = |list: &mut DrawList| {
        let _ = list.push(DrawCommand::TexturedQuad {
            rect,
            texture: art.medal.art(),
            color: Color::new(1.0, 1.0, 1.0, 1.0),
        });
    };
    if !on || art.rank == Rank::Issued {
        picture(list);
        if art.rank > Rank::Issued {
            device(
                art.rank,
                [
                    rect.x + rect.width * ART_DEVICE[0],
                    rect.y + rect.height * ART_DEVICE[1],
                ],
                rect.width * ART_DEVICE_SIZE,
                |command| {
                    let _ = list.push(command);
                },
            );
        }
        if faded {
            let _ = list.push(DrawCommand::PopOpacity);
        }
        return;
    }
    let glow = art.medal.glow();
    let seconds = art.seconds;
    let (centre, radius) = art.medallion();
    let px = (rect.width / 340.0).max(0.25);
    // Engraved and up lean towards the pointer, or sway by themselves: the sway eases
    // into the pointer's hold and back, never jumping.
    let hover = (art.rank >= Rank::Engraved).then(|| {
        let idle = sway(seconds);
        art.hover.map_or(idle, |hover| {
            let hold = hover.hold.clamp(0.0, 1.0);
            Hover {
                at: [0, 1].map(|axis| idle.at[axis] + (hover.at[axis] - idle.at[axis]) * hold),
                hold: idle.hold + (1.0 - idle.hold) * hold,
            }
        })
    });
    if let Some(hover) = hover {
        let lean = Hover {
            at: hover.at.map(|at| at * LEAN),
            hold: hover.hold,
        };
        let _ = list.push(DrawCommand::PushTransform(crate::cards::look::tilt_matrix(
            centre,
            rect.height,
            lean,
        )));
    }
    // Under the picture: Solar's corona round the rim, Engraved's breathing rim.
    if art.rank >= Rank::Solar {
        let reach = radius * 1.2;
        crate::tile_shine::push(
            list,
            Rect::new(
                centre[0] - reach,
                centre[1] - reach,
                reach * 2.0,
                reach * 2.0,
            ),
            reach,
            Some(ShineKind::Solar),
            None,
            seconds,
        );
    }
    if art.rank >= Rank::Engraved {
        let breath = 0.5 - 0.5 * (seconds / RIM_BREATH * TAU).cos();
        let _ = list.push(DrawCommand::Arc {
            center: centre,
            radius: radius * 1.03,
            width: radius * 0.1,
            start: 0.0,
            sweep: TAU,
            color: with_alpha(glow, 0.18 + 0.2 * breath),
            knockout: None,
        });
        let _ = list.push(DrawCommand::Arc {
            center: centre,
            radius: radius * 1.0,
            width: (1.6 * px).max(1.0),
            start: 0.0,
            sweep: TAU,
            color: with_alpha(lit(glow, 0.4), 0.45 + 0.35 * breath),
            knockout: None,
        });
    }
    picture(list);
    // Polished: the sweep of metal light, in the medal's colour.
    if let Some(travel) = sheen(art.medal, seconds) {
        let tint = lit(glow, 0.55);
        crate::medal_popup::award::band(
            list,
            rect,
            art.medal.art(),
            -0.3 + 1.6 * travel,
            (travel * PI).sin(),
            [1.2 + 1.3 * tint.r, 1.2 + 1.3 * tint.g, 1.2 + 1.3 * tint.b],
        );
    }
    // Over the medallion: Holo's foil, and the glare under the pointer.
    let face = radius * 0.92;
    let disc_rect = Rect::new(centre[0] - face, centre[1] - face, face * 2.0, face * 2.0);
    let foil = (art.rank >= Rank::Holo).then_some(ShineKind::Holo);
    crate::tile_shine::push(list, disc_rect, face, foil, hover, seconds);
    // Solar's motes rising over it.
    if art.rank >= Rank::Solar {
        crate::rarity_fx::rising_motes(
            list,
            lit(glow, 0.3),
            Rect::new(
                centre[0] - radius,
                centre[1] - radius * 1.3,
                radius * 2.0,
                radius * 2.2,
            ),
            px,
            seconds,
        );
    }
    if hover.is_some() {
        let _ = list.push(DrawCommand::PopTransform);
    }
    if faded {
        let _ = list.push(DrawCommand::PopOpacity);
    }
}

/// How far the Polished sweep has crossed `medal` at `seconds` (`None` while it
/// rests): the Legendary sheen's clock, each medal at its own moment so a row of them
/// does not flash at once, wherever it is drawn.
fn sheen(medal: Medal, seconds: f32) -> Option<f32> {
    crate::rarity_fx::sheen_travel_from(medal.index() as f32 * 0.29, seconds)
}

/// An Engraved medal's sway while no pointer rests on it: a slow drift of its lean and
/// light, as though it turned on its ribbon.
fn sway(seconds: f32) -> Hover {
    Hover {
        at: [
            SWAY[0] * (seconds / SWAY_SECONDS[0] * TAU).sin(),
            SWAY[1] * (seconds / SWAY_SECONDS[1] * TAU).sin(),
        ],
        hold: SWAY_HOLD,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjk_ui::TextureId;

    /// The textures the pictures drawn use: the medal, then its sweep's strips.
    fn pictures(commands: &[DrawCommand]) -> Vec<TextureId> {
        commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::TexturedQuad { texture, .. }
                | DrawCommand::TexturedQuadUv { texture, .. } => Some(*texture),
                _ => None,
            })
            .collect()
    }

    const RECT: Rect = Rect::new(100.0, 50.0, 340.0, 340.0);

    fn drawn(rank: Rank, on: bool, seconds: f32) -> Vec<DrawCommand> {
        let mut list = DrawList::new(256);
        draw_art_as(&mut list, Art::new(Medal::Tester, rank, RECT, seconds), on);
        list.commands().to_vec()
    }

    fn shines(commands: &[DrawCommand]) -> Vec<ShineKind> {
        commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::CardShine { shine, .. } => Some(shine.kind),
                _ => None,
            })
            .collect()
    }

    /// A moment in the middle of the Polished sweep's crossing.
    fn sweeping() -> f32 {
        (0..800)
            .map(|step| step as f32 * 0.05)
            .find(|seconds| sheen(Medal::Tester, *seconds).is_some_and(|t| t > 0.3 && t < 0.7))
            .expect("a crossing")
    }

    #[test]
    fn ranks_read_from_the_wire_clamped_and_are_named() {
        assert_eq!(Rank::from_wire(0), Rank::Issued);
        assert_eq!(Rank::from_wire(1), Rank::Issued);
        assert_eq!(Rank::from_wire(3), Rank::Engraved);
        assert_eq!(Rank::from_wire(5), Rank::Solar);
        assert_eq!(Rank::from_wire(200), Rank::Solar);
        for (index, rank) in Rank::ALL.into_iter().enumerate() {
            assert_eq!(usize::from(rank.number()), index + 1);
            assert_eq!(Rank::from_wire(rank.number()), rank);
            assert_eq!(
                rank.title(),
                format!("Rank {} {}", rank.numeral(), rank.name())
            );
        }
        assert_eq!(Rank::Engraved.title(), "Rank III Engraved");
        assert_eq!(Rank::Holo.next(), Some(Rank::Solar));
        assert_eq!(Rank::Solar.next(), None);
        assert_eq!(Rank::default(), Rank::Issued);
    }

    #[test]
    fn each_rank_keeps_the_lower_ones_and_adds_its_own() {
        let at = sweeping();
        // Issued: the picture alone.
        assert_eq!(drawn(Rank::Issued, true, at).len(), 1);
        // Polished: the picture and its sweep, nothing leaning.
        let polished = drawn(Rank::Polished, true, at);
        assert!(pictures(&polished).len() > 1, "the sweep");
        assert!(shines(&polished).is_empty());
        // Engraved: it leans, a rim and the glare.
        let engraved = drawn(Rank::Engraved, true, at);
        assert!(matches!(engraved[0], DrawCommand::PushTransform(_)));
        assert!(matches!(engraved.last(), Some(DrawCommand::PopTransform)));
        assert_eq!(shines(&engraved), [ShineKind::Glare]);
        assert!(pictures(&engraved).len() > 1, "the sweep stays");
        // Holo: the foil too.
        assert_eq!(
            shines(&drawn(Rank::Holo, true, at)),
            [ShineKind::Holo, ShineKind::Glare]
        );
        // Solar: the corona under it all, and motes.
        let solar = drawn(Rank::Solar, true, at);
        assert_eq!(
            shines(&solar),
            [ShineKind::Solar, ShineKind::Holo, ShineKind::Glare]
        );
        assert!(solar.len() > drawn(Rank::Holo, true, at).len());
        for rank in Rank::ALL {
            for step in 0..80 {
                let commands = drawn(rank, true, step as f32 * 0.31);
                assert!(
                    commands.len() <= ART_COMMANDS,
                    "{rank:?}: {}",
                    commands.len()
                );
            }
        }
    }

    #[test]
    fn with_the_effects_off_a_medal_is_still_and_wears_its_device() {
        let at = sweeping();
        assert_eq!(drawn(Rank::Issued, false, at).len(), 1);
        for rank in [Rank::Polished, Rank::Engraved, Rank::Holo, Rank::Solar] {
            let still = drawn(rank, false, at);
            assert_eq!(pictures(&still), [Medal::Tester.art()], "{rank:?}");
            assert!(shines(&still).is_empty());
            assert!(
                !still
                    .iter()
                    .any(|c| matches!(c, DrawCommand::PushTransform(_)))
            );
            assert!(still.len() > 1, "{rank:?}: its device");
            assert_eq!(still, drawn(rank, false, at + 1.7), "nothing moves");
        }
    }

    #[test]
    fn devices_grow_with_the_rank_and_stay_on_their_ribbon() {
        let ribbon = Rect::new(10.0, 10.0, 30.0, 9.0);
        let mut counts = Vec::new();
        for rank in Rank::ALL {
            let mut commands = Vec::new();
            ribbon_device(rank, ribbon, |command| commands.push(command));
            assert!(commands.len() <= DEVICE_COMMANDS);
            for command in &commands {
                let inside = match *command {
                    DrawCommand::RoundedRect { rect, .. } => {
                        rect.x >= ribbon.x - 0.01
                            && rect.y >= ribbon.y - 0.01
                            && rect.right() <= ribbon.right() + 0.01
                            && rect.bottom() <= ribbon.bottom() + 0.01
                    }
                    DrawCommand::Line { from, to, .. } => [from, to].iter().all(|[x, y]| {
                        *x >= ribbon.x
                            && *x <= ribbon.right()
                            && *y >= ribbon.y
                            && *y <= ribbon.bottom()
                    }),
                    _ => false,
                };
                assert!(inside, "{rank:?}: {command:?}");
            }
            counts.push(commands.len());
        }
        assert_eq!(counts[0], 0, "Issued wears none");
        assert!(counts[1..4].iter().all(|count| *count == 3));
        assert!(counts[4] > counts[3], "Solar's sun has rays");
        // Each pip is its own metal.
        let metal = |rank| {
            let mut commands = Vec::new();
            ribbon_device(rank, ribbon, |command| commands.push(command));
            match commands[1] {
                DrawCommand::RoundedRect { color, .. } => color,
                _ => unreachable!(),
            }
        };
        assert_ne!(metal(Rank::Polished), metal(Rank::Engraved));
        assert_ne!(metal(Rank::Engraved), metal(Rank::Holo));
    }
}
