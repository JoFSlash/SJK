//! The opening's first moment (`docs/cards.md`, "The ceremony"): the tab's 3D holocron
//! ([`crate::holocrons::stage`]) leaves its place on the page for the middle of the
//! window while the hub answers, humming round and breathing its light; then it spins
//! up, swells, shakes and blazes, light breaking out of its seams, and is gone in a flash
//! of its tier's colour, out of which the crate strip runs. Every holocron opened on its
//! own cracks open so; an opening of many cracks the first and runs their strips
//! together ([`super::batch`]).
//!
//! The world shows round the holocron through a dark vignette ([`vignette`]); the page
//! under it is not drawn. The motion is worked out from the ceremony's clock alone
//! ([`motion`]), so a world shot draws any moment of it.

use super::opening::{Ceremony, UNSEAL};
use super::*;
use crate::audio::ui_cues::{self, Cue};
use crate::holocrons::stage::Motion;
use crate::menu::sjk::{Frame, color, text};
use crate::menu_widgets::TextFamily;
use sjk_ui::{Color, DrawCommand, FontWeight, TextAlign};

/// Where the holocron cracks open: the middle of the 1920 by 1080 frame, a little high.
pub(crate) const MIDDLE: [f32; 2] = [960.0, 470.0];
/// Seconds to glide from the page's place to the middle.
const GLIDE: f32 = 0.6;
/// Its turn while the hub answers, in radians a second on top of the idle one; the
/// spin-up adds up to [`SPIN_UP`] by the flash.
const HUM: f32 = 1.4;
const SPIN_UP: f32 = 42.0;
/// How far it shakes at the end (frame pixels), how much it swells, and how bright its
/// light grows against the tier's.
const SHAKE: f32 = 7.0;
const SWELL: f32 = 0.15;
const BLAZE: f32 = 2.5;
/// The pop before the flash: it starts this long before the end, swells this much more
/// and flares this much more, and the holocron is gone [`VANISH`] before the end.
const POP: f32 = 0.25;
const POP_SWELL: f32 = 0.35;
const POP_GLOW: f32 = 3.0;
const VANISH: f32 = 0.08;
/// The flash: it rises over [`FLASH_IN`] to the end of the unseal and fades over
/// [`FLASH_OUT`] into the strip.
const FLASH_IN: f32 = 0.2;
pub(super) const FLASH_OUT: f32 = 0.45;
/// The vignette's clear box about [`MIDDLE`] (frame pixels).
const CLEAR: [f32; 2] = [520.0, 420.0];
/// Rays out of the seams: the fewest at the start, the most at the end.
const SEAMS: [usize; 2] = [4, 16];

/// Ease in and out.
fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// How far along its glide the holocron is after `seconds`.
fn glide(seconds: f32) -> [f32; 2] {
    let g = smooth(seconds / GLIDE);
    [
        STAGE_AT[0] + (MIDDLE[0] - STAGE_AT[0]) * g,
        STAGE_AT[1] + (MIDDLE[1] - STAGE_AT[1]) * g,
    ]
}

/// The motion of the holocron `seconds` into its unseal (0 to [`UNSEAL`]): `glided`
/// seconds after it set off for the middle (the wait for the hub included), its turn
/// adding to `turned`.
pub(super) fn unsealing(seconds: f32, glided: f32, turned: f32) -> Motion {
    let charge = (seconds / UNSEAL).clamp(0.0, 1.0);
    // Its spin: the hum, then up to SPIN_UP a second by the flash, `charge` squared.
    let turn = turned + HUM * seconds + SPIN_UP * seconds * charge * charge / 3.0;
    let pop = ((seconds - (UNSEAL - POP)) / (POP - VANISH)).clamp(0.0, 1.0);
    let shake = SHAKE * charge.powi(3);
    let [x, y] = glide(glided);
    Motion {
        at: [
            x + shake * (seconds * 53.0).sin(),
            y + shake * (seconds * 41.0).cos(),
        ],
        turn,
        swell: (1.0 + SWELL * charge * charge) * (1.0 + POP_SWELL * pop),
        glow: 1.0 + BLAZE * charge * charge + POP_GLOW * pop,
        shown: seconds < UNSEAL - VANISH,
    }
}

/// The motion while the hub is asked, `seconds` since: it glides to the middle, hums
/// round and breathes its light.
pub(super) fn waiting(seconds: f32) -> Motion {
    Motion {
        at: glide(seconds),
        turn: HUM * seconds,
        swell: 1.0,
        glow: 1.0 + 0.3 * (0.5 + 0.5 * (seconds * 5.0).sin()),
        shown: true,
    }
}

impl Panel {
    /// How the 3D holocron moves at `now` while the opening shows; `None` without one.
    pub(crate) fn stage_motion(&self, now: Instant) -> Option<Motion> {
        let ceremony = self.ceremony.as_ref()?;
        let elapsed = self.ceremony_elapsed(ceremony, now);
        Some(match ceremony {
            Ceremony::Waiting { .. } => waiting(self.glide_from.map_or(0.0, |from| {
                now.saturating_duration_since(from).as_secs_f32()
            })),
            Ceremony::Showing { index, .. } if elapsed < UNSEAL => {
                if *index == 0 {
                    unsealing(elapsed, self.waited + elapsed, HUM * self.waited)
                } else {
                    // The next holocron grows in the middle.
                    unsealing(elapsed, GLIDE, 0.0)
                }
            }
            Ceremony::Batch { .. } if elapsed < UNSEAL => {
                unsealing(elapsed, self.waited + elapsed, HUM * self.waited)
            }
            _ => Motion {
                at: MIDDLE,
                turn: 0.0,
                swell: 1.0,
                glow: 1.0,
                shown: false,
            },
        })
    }

    /// While the hub is asked: the world round the holocron through the vignette, a halo
    /// and "Opening...".
    pub(super) fn draw_waiting(&mut self, frame: &Frame, viewport: [f32; 2], tier: usize) {
        let s = frame.s;
        vignette(&mut self.ui, frame, viewport, 0.97);
        let colour = holocrons::slot(tier).colour;
        halo(&mut self.ui, frame, colour, 0.2);
        text(
            &mut self.ui,
            TextFamily::Display,
            format_args!("Opening..."),
            frame.rect(MIDDLE[0] - 400.0, 690.0, 800.0, 44.0),
            34.0 * s,
            color::TEXT,
            FontWeight::Semibold,
            TextAlign::Center,
        );
    }

    /// The holocron of tier `tier` cracking open, `elapsed` seconds in: `title` (its name,
    /// or how many open) in its colour, the halo, light out of its seams, then the flash.
    /// `of` is "2 of 3" when several open one by one.
    pub(super) fn draw_unseal(
        &mut self,
        frame: &Frame,
        viewport: [f32; 2],
        tier: usize,
        elapsed: f32,
        title: std::fmt::Arguments<'_>,
        of: Option<(usize, usize)>,
    ) {
        let s = frame.s;
        if !self.played.charged {
            self.played.charged = true;
            ui_cues::post(Cue::HolocronCharge);
        }
        if elapsed >= UNSEAL - FLASH_IN && !self.played.cracked {
            self.played.cracked = true;
            ui_cues::post(Cue::HolocronOpen);
        }
        let tier = holocrons::slot(tier);
        let charge = (elapsed / UNSEAL).clamp(0.0, 1.0);
        vignette(&mut self.ui, frame, viewport, 0.97);
        halo(
            &mut self.ui,
            frame,
            tier.colour,
            0.2 + 0.5 * charge * charge,
        );
        seams(&mut self.ui, frame, tier.colour, elapsed, charge);
        let words = (elapsed / 0.3).clamp(0.0, 1.0);
        text(
            &mut self.ui,
            TextFamily::Display,
            title,
            frame.rect(MIDDLE[0] - 500.0, 170.0, 1_000.0, 50.0),
            40.0 * s,
            tier.colour_alpha(words),
            FontWeight::Semibold,
            TextAlign::Center,
        );
        if let Some((index, count)) = of {
            text(
                &mut self.ui,
                TextFamily::Display,
                format_args!("{} of {count}", index + 1),
                frame.rect(MIDDLE[0] - 300.0, 124.0, 600.0, 34.0),
                24.0 * s,
                color::alpha(color::MUTED, words),
                FontWeight::Regular,
                TextAlign::Center,
            );
        }
        let rise = ((elapsed - (UNSEAL - FLASH_IN)) / FLASH_IN).clamp(0.0, 1.0);
        flash(&mut self.ui, viewport, tier.colour, rise);
    }
}

/// The flash at `strength` (0 to 1) over the whole window: the tier's colour with a white
/// heart.
pub(super) fn flash(
    canvas: &mut crate::menu_widgets::MenuCanvas,
    viewport: [f32; 2],
    colour: Color,
    strength: f32,
) {
    if strength <= 0.0 {
        return;
    }
    let whole = sjk_ui::Rect::new(0.0, 0.0, viewport[0], viewport[1]);
    let _ = canvas.draw_list_mut().push(DrawCommand::SolidRect {
        rect: whole,
        color: color::alpha(colour, 0.8 * strength),
    });
    let _ = canvas.draw_list_mut().push(DrawCommand::SolidRect {
        rect: whole,
        color: color::alpha(Color::new(1.0, 1.0, 1.0, 1.0), 0.45 * strength * strength),
    });
}

/// The dark over the window with a clear middle about [`MIDDLE`], where the holocron
/// floats in the world: four fades from the window's edges, dark `alpha` there, to clear
/// at a box [`CLEAR`] across about the middle (darker in the corners, where two fades
/// meet, which rounds it). Straight fades only, so no seam shows: the UI renderer's arcs
/// are strokes, not fills, and take no clip.
pub(super) fn vignette(
    canvas: &mut crate::menu_widgets::MenuCanvas,
    frame: &Frame,
    viewport: [f32; 2],
    alpha: f32,
) {
    let s = frame.s;
    let [cx, cy] = frame.point(MIDDLE[0], MIDDLE[1]);
    let (half_x, half_y) = (CLEAR[0] * 0.5 * s, CLEAR[1] * 0.5 * s);
    let dark = color::alpha(color::SPACE, alpha);
    let clear = color::alpha(color::SPACE, 0.0);
    let (left, top) = ((cx - half_x).max(0.0), (cy - half_y).max(0.0));
    let (right, bottom) = (
        (cx + half_x).min(viewport[0]),
        (cy + half_y).min(viewport[1]),
    );
    let list = canvas.draw_list_mut();
    for (rect, start, end, vertical) in [
        (
            sjk_ui::Rect::new(0.0, 0.0, left, viewport[1]),
            dark,
            clear,
            false,
        ),
        (
            sjk_ui::Rect::new(right, 0.0, viewport[0] - right, viewport[1]),
            clear,
            dark,
            false,
        ),
        (
            sjk_ui::Rect::new(0.0, 0.0, viewport[0], top),
            dark,
            clear,
            true,
        ),
        (
            sjk_ui::Rect::new(0.0, bottom, viewport[0], viewport[1] - bottom),
            clear,
            dark,
            true,
        ),
    ] {
        let _ = list.push(DrawCommand::GradientRect {
            rect,
            radius: 0.0,
            gradient: sjk_ui::Gradient {
                start,
                end,
                vertical,
            },
        });
    }
}

/// Rings of the tier's light round the holocron, `strength` their alpha.
fn halo(canvas: &mut crate::menu_widgets::MenuCanvas, frame: &Frame, colour: Color, strength: f32) {
    let s = frame.s;
    let [cx, cy] = frame.point(MIDDLE[0], MIDDLE[1]);
    for (ring, (radius, width)) in [(150.0, 26.0), (190.0, 40.0), (250.0, 70.0)]
        .into_iter()
        .enumerate()
    {
        let _ = canvas.draw_list_mut().push(DrawCommand::Arc {
            center: [cx, cy],
            radius: radius * s,
            width: width * s,
            start: 0.0,
            sweep: std::f32::consts::TAU,
            color: color::alpha(colour, strength * 0.35 / (ring as f32 + 1.0)),
            knockout: None,
        });
    }
}

/// Light breaking out of the holocron's seams, `elapsed` seconds into the unseal and
/// `charge` (0 to 1) of the way: more rays, longer and brighter, turning faster.
fn seams(
    canvas: &mut crate::menu_widgets::MenuCanvas,
    frame: &Frame,
    colour: Color,
    elapsed: f32,
    charge: f32,
) {
    let s = frame.s;
    let [cx, cy] = frame.point(MIDDLE[0], MIDDLE[1]);
    let count = SEAMS[0] + ((SEAMS[1] - SEAMS[0]) as f32 * charge).round() as usize;
    let inner = 110.0 * (1.0 + SWELL * charge * charge) * s;
    for ray in 0..count {
        // Each ray flickers on its own beat.
        let beat = 0.6 + 0.4 * (elapsed * (7.0 + ray as f32 * 1.3)).sin();
        let angle = ray as f32 / count as f32 * std::f32::consts::TAU
            + elapsed * (0.6 + 2.4 * charge)
            + ray as f32 * 0.37;
        let (sin, cos) = angle.sin_cos();
        let length = (40.0 + 420.0 * charge * charge) * beat * s;
        let _ = canvas.draw_list_mut().push(DrawCommand::Line {
            from: [cx + cos * inner, cy + sin * inner],
            to: [cx + cos * (inner + length), cy + sin * (inner + length)],
            width: (2.0 + 4.0 * charge) * s,
            color: color::alpha(colour, (0.25 + 0.6 * charge) * beat),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_glides_hums_spins_up_and_is_gone_in_the_flash() {
        let start = waiting(0.0);
        assert_eq!(start.at, STAGE_AT, "from its place on the page");
        assert!(start.shown);
        let there = waiting(GLIDE + 0.5);
        assert_eq!(there.at, MIDDLE, "in the middle once glided");
        // The unseal carries on the turn and the glide the wait left.
        let waited = 0.3;
        let handover = unsealing(0.0, waited, HUM * waited);
        assert!((handover.turn - waiting(waited).turn).abs() < 1e-5);
        assert_eq!(handover.at, waiting(waited).at);
        // Faster and faster, bigger and brighter.
        let speed = |t: f32| unsealing(t + 0.01, 1.0, 0.0).turn - unsealing(t, 1.0, 0.0).turn;
        assert!(speed(1.4) > 4.0 * speed(0.2));
        let late = unsealing(UNSEAL - 0.1, 1.0, 0.0);
        assert!(late.swell > 1.3 && late.glow > 4.0);
        assert!(late.shown);
        assert!(
            !unsealing(UNSEAL - 0.05, 1.0, 0.0).shown,
            "gone in the flash"
        );
        // It shakes only late, and never far.
        let early = unsealing(0.2, 1.0, 0.0);
        assert!((early.at[0] - MIDDLE[0]).abs() < 0.1);
        for tenth in 0..17 {
            let motion = unsealing(tenth as f32 * 0.1, 1.0, 0.0);
            assert!((motion.at[0] - MIDDLE[0]).abs() <= SHAKE + 1e-3);
            assert!((motion.at[1] - MIDDLE[1]).abs() <= SHAKE + 1e-3);
        }
    }
}
