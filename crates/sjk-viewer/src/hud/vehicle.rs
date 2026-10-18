//! The vehicle HUD: what OpenJK codemp's `CG_DrawVehicleHud` (`cg_draw.c`) shows a
//! pilot, drawn in SJK's vector style instead of the `swoopvehiclehud` menu's pictures.
//!
//! Stock reads the maxima from the vehicle's `.veh` definition ([`Profile`]) and the
//! values from `cg.predictedVehicleState`: `stats[STAT_HEALTH]` against `armor` (hull,
//! twelve tics), `stats[STAT_ARMOR]` against `shields` (five tics), `speed` against
//! `speedMax` (five tics, red and flashing while the turbo burns), `ammo[0]` and
//! `ammo[1]` against each weapon's `ammoMax` (five tics, or two rows when the vehicle has
//! two weapons), the turbo recharge (a bar that fills, green once ready) and the
//! weapons-linked mark. A tic is drawn at full strength while the value covers it, faded
//! by the covered fraction when it does, and not at all beyond. When the vehicle hides its
//! rider the player's own status HUD is not drawn (`CG_DrawVehicleHud` returns false).
//!
//! Not drawn: the damage-direction icons of `vehicledamagehud` and
//! `enemyvehicledamagehud`, the no-ammo warning flash and the weapons-linked sound.

use super::*;
use sjk_game_jka::vehicle_fields::VehicleInfo;
use sjk_ui::{Color, DrawCommand, FontWeight, Rect, TextAlign, TextOverflow, TextureId};
use std::fmt::Write as _;

/// First and last text ids this HUD resolves (`HudOverlay::resolve_text`).
pub(super) const TEXT_FIRST: u32 = 320;
pub(super) const TEXT_LAST: u32 = 323;
const HULL_TEXT: u32 = TEXT_FIRST;
const SHIELD_TEXT: u32 = TEXT_FIRST + 1;
const SPEED_TEXT: u32 = TEXT_FIRST + 2;
const AMMO_TEXT: u32 = TEXT_FIRST + 3;

/// `MAX_VHUD_SHIELD_TICS`, `MAX_VHUD_ARMOR_TICS`, `MAX_VHUD_SPEED_TICS`,
/// `MAX_VHUD_AMMO_TICS`.
const HULL_TICS: usize = 12;
const SHIELD_TICS: usize = 5;
const SPEED_TICS: usize = 5;
const AMMO_TICS: usize = 5;
/// `CG_DrawVehicleAmmoUpper` and `Lower` loop `i < MAX_VHUD_AMMO_TICS`: four tics are
/// drawn although the value is divided by five, so a full weapon never fills the row.
const DUAL_AMMO_TICS: usize = 4;

/// The turbo flashes this often while it burns (`cg.VHUDFlashTime = cg.time + 200`).
const FLASH_MS: i32 = 200;

/// What the vehicle's `.veh` file gives the HUD, read once when the vehicle model loads.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Profile {
    /// `armor`: the hull's maximum (`STAT_HEALTH` of the vehicle).
    pub(crate) hull: i32,
    /// `shields` (`STAT_ARMOR` of the vehicle).
    pub(crate) shields: i32,
    /// `speedMax`.
    pub(crate) speed_max: f32,
    /// `turboRecharge`, in milliseconds.
    pub(crate) turbo_recharge: i32,
    /// `weap1AmmoMax`, `weap2AmmoMax`.
    pub(crate) ammo_max: [i32; 2],
    /// `weapon[n].ID` is set (stock tests it as a boolean, so -1 counts as a weapon).
    pub(crate) has_weapon: [bool; 2],
    /// A weapon's `linkable` is 2: the weapons are always linked.
    pub(crate) always_linked: bool,
    /// `hideRider`: the rider is not drawn, and the player's status HUD gives way.
    pub(crate) hide_rider: bool,
    /// `crosshairShader`, the picture drawn as the crosshair while riding.
    pub(crate) crosshair: Option<String>,
}

impl Profile {
    /// The HUD's view of a parsed vehicle definition.
    pub(crate) fn from_info(info: &VehicleInfo) -> Self {
        let [first, second] = &info.weapons;
        Self {
            hull: info.armor,
            shields: info.shields,
            speed_max: info.speed_max,
            turbo_recharge: info.turbo_recharge,
            ammo_max: [first.ammo_max, second.ammo_max],
            has_weapon: [first.id != 0, second.id != 0],
            always_linked: first.linkable == 2 || second.linkable == 2,
            hide_rider: info.hide_rider,
            crosshair: info
                .crosshair_shader
                .as_deref()
                .filter(|name| !name.is_empty())
                .map(|name| String::from_utf8_lossy(name).into_owned()),
        }
    }
}

/// One frame's inputs, from the snapshot (and the ride prediction where it runs).
pub(crate) struct Sample<'a> {
    /// The local player state's `m_iVehicleNum`; zero on foot.
    pub(crate) number: u16,
    /// The snapshot carries the vehicle's own player state, which only its pilot gets.
    pub(crate) piloted: bool,
    /// The riding vehicle's definition, once its model has loaded.
    pub(crate) profile: Option<&'a Profile>,
    /// The local player is alive and not a spectator.
    pub(crate) alive: bool,
    /// `stats[STAT_HEALTH]` of the vehicle's player state.
    pub(crate) hull: i32,
    /// `stats[STAT_ARMOR]` of the vehicle's player state.
    pub(crate) shield: i32,
    /// `speed` of the predicted vehicle state, or the snapshot's.
    pub(crate) speed: f32,
    /// `ammo[0]`, `ammo[1]`.
    pub(crate) ammo: [i32; 2],
    /// `vehWeaponsLinked`.
    pub(crate) linked: bool,
    /// `m_iTurboTime` of the predicted vehicle, where prediction runs.
    pub(crate) turbo_time: Option<i32>,
    /// `cg.time`.
    pub(crate) time: i32,
    /// `cg_dynamicCrosshair`.
    pub(crate) dynamic_crosshair: i64,
}

/// Opacity of each tic of a meter: `value` against `max` split into `divisions` tics,
/// of which `N` are drawn. A tic the value covers is 1, the one it only part covers fades
/// by that part, and the rest are 0 (the loops of `CG_DrawVehicleArmor` and its kin).
pub(super) fn tic_alphas<const N: usize>(value: f32, max: f32, divisions: usize) -> [f32; N] {
    let mut alphas = [0.0; N];
    let mut value = if value.is_finite() { value } else { 0.0 };
    let max = if max.is_finite() { max } else { 0.0 };
    let each = max / divisions.max(1) as f32;
    for alpha in &mut alphas {
        if value <= 0.0 {
            break;
        }
        *alpha = if value < each { value / each } else { 1.0 };
        value -= each;
    }
    alphas
}

/// How charged the turbo is, `0..=1`, and whether it is ready: `CG_DrawVehicleTurboRecharge`
/// measures the time since `m_iTurboTime` against `turboRecharge`.
pub(super) fn turbo_charge(time: i32, turbo_time: i32, recharge: i32) -> (f32, bool) {
    let elapsed = i64::from(time) - i64::from(turbo_time);
    if elapsed > i64::from(recharge) {
        return (1.0, true);
    }
    // A vehicle without a recharge time has no fraction to show before it is ready.
    let fraction = if recharge > 0 {
        (elapsed as f32 / recharge as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (fraction, false)
}

/// The turbo burns while `cg.time` has not passed `m_iTurboTime` (`CG_DrawVehicleSpeed`).
pub(super) fn turbo_burning(time: i32, turbo_time: i32) -> bool {
    time <= turbo_time
}

/// Whether the HUD draws: a living pilot whose vehicle definition is known.
pub(super) fn shown(sample: &Sample) -> bool {
    sample.number != 0 && sample.piloted && sample.profile.is_some() && sample.alive
}

/// The crosshair size stock draws while riding, `CG_DrawCrosshair`: twice `cg_crosshairSize`
/// when the crosshair is not the selective one (`cg_dynamicCrosshair` 2 and above).
pub(super) fn crosshair_doubled(number: u16, dynamic_crosshair: i64) -> bool {
    number != 0 && dynamic_crosshair < 2
}

/// Panel-local rectangles (`x`, `y`, width, height) in units of the 640 by 480 screen:
/// one dark pill at the bottom of the screen, as the SJK HUD's own panels, holding the
/// hull's segmented bar with its value above it and a row of three segmented meters
/// (speed, shield, ammunition) with their values under them.
mod strip {
    pub(super) const WIDTH: f32 = 300.0;
    pub(super) const HEIGHT: f32 = 62.0;
    /// Gap kept under the panel.
    pub(super) const MARGIN: f32 = 8.0;
    pub(super) const RADIUS: f32 = 14.0;
    /// The inner margin of the panel's contents.
    pub(super) const PAD: f32 = 14.0;
    pub(super) const HULL_LABEL: [f32; 4] = [PAD, 6.0, 160.0, 11.0];
    pub(super) const HULL_BAR: [f32; 4] = [PAD, 19.0, WIDTH - 2.0 * PAD, 9.0];
    /// The three meters of the lower row, `GROUP_GAP` apart.
    pub(super) const GROUP_Y: f32 = 35.0;
    pub(super) const GROUP_WIDTH: f32 = (WIDTH - 2.0 * PAD - 2.0 * GROUP_GAP) / 3.0;
    pub(super) const GROUP_GAP: f32 = 10.0;
    pub(super) const GROUP_HEIGHT: f32 = 7.0;
    /// Two weapons: two thinner rows in the ammunition meter's place.
    pub(super) const ROW_HEIGHT: f32 = 3.0;
    pub(super) const LOWER_Y: f32 = GROUP_Y + GROUP_HEIGHT - ROW_HEIGHT;
    pub(super) const LABEL_Y: f32 = 45.0;
    pub(super) const LABEL_HEIGHT: f32 = 10.0;
    /// Gap between two segments of a meter.
    pub(super) const SEGMENT_GAP: f32 = 2.0;
    pub(super) const SEGMENT_RADIUS: f32 = 1.5;
    /// The turbo's charge and the weapons-linked mark: thin upright pills either side of
    /// the lower row, in the panel's margin.
    pub(super) const TURBO: [f32; 4] = [6.0, GROUP_Y, 3.0, GROUP_HEIGHT];
    pub(super) const LINKED: [f32; 4] = [WIDTH - 9.0, GROUP_Y, 3.0, GROUP_HEIGHT];

    /// The left edge of lower-row meter `index` (speed, shield, ammunition).
    pub(super) const fn group_x(index: usize) -> f32 {
        PAD + index as f32 * (GROUP_WIDTH + GROUP_GAP)
    }
}

/// Colours, from the SJK HUD's own: its panels, its meters' track, and the health,
/// armour, Force and ammunition colours of its arcs and values.
mod tint {
    use sjk_ui::Color;
    pub(super) const PANEL: Color = Color::new(0.0, 0.0, 0.0, 0.5);
    pub(super) const TRACK: Color = Color::new(1.0, 1.0, 1.0, 0.14);
    pub(super) const HULL: Color = Color::new(1.0, 0.3, 0.22, 1.0);
    pub(super) const HULL_TEXT: Color = Color::new(1.0, 0.46, 0.4, 1.0);
    pub(super) const SHIELD: Color = Color::new(0.32, 1.0, 0.4, 1.0);
    pub(super) const SHIELD_TEXT: Color = Color::new(0.5, 1.0, 0.56, 1.0);
    pub(super) const SPEED: Color = Color::new(0.34, 0.64, 1.0, 1.0);
    pub(super) const SPEED_TEXT: Color = Color::new(0.52, 0.74, 1.0, 1.0);
    /// `colorTable[CT_LTRED1]`-like flash of the speed while the turbo burns.
    pub(super) const BOOST: Color = Color::new(1.0, 0.4, 0.38, 1.0);
    pub(super) const AMMO: Color = Color::new(1.0, 0.76, 0.16, 1.0);
    pub(super) const AMMO_TEXT: Color = Color::new(1.0, 0.82, 0.36, 1.0);
    /// `CT_GREEN` ready, `CT_RED` charging.
    pub(super) const READY: Color = Color::new(0.32, 1.0, 0.4, 1.0);
    pub(super) const CHARGING: Color = Color::new(1.0, 0.3, 0.22, 1.0);
    /// `CT_CYAN`.
    pub(super) const LINKED: Color = Color::new(0.3, 0.95, 1.0, 1.0);
}

/// Retained vehicle HUD state; every string is reserved once.
pub(crate) struct State {
    shown: bool,
    hides_player: bool,
    doubled: bool,
    hull: [f32; HULL_TICS],
    shield: [f32; SHIELD_TICS],
    speed: [f32; SPEED_TICS],
    ammo: [f32; AMMO_TICS],
    ammo_upper: [f32; DUAL_AMMO_TICS],
    ammo_lower: [f32; DUAL_AMMO_TICS],
    /// Weapons the vehicle has: none, one (a single row), or two (upper and lower).
    weapons: [bool; 2],
    shield_present: bool,
    boost_flash: bool,
    /// Turbo recharge fraction and readiness, where prediction gives the turbo time.
    turbo: Option<(f32, bool)>,
    linked: bool,
    hull_text: String,
    shield_text: String,
    speed_text: String,
    ammo_text: String,
    /// `crosshairShader` of the riding vehicle, and the one last tried to load.
    wanted: Option<String>,
    attempted: Option<String>,
    picture: Option<TextureId>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            shown: false,
            hides_player: false,
            doubled: false,
            hull: [0.0; HULL_TICS],
            shield: [0.0; SHIELD_TICS],
            speed: [0.0; SPEED_TICS],
            ammo: [0.0; AMMO_TICS],
            ammo_upper: [0.0; DUAL_AMMO_TICS],
            ammo_lower: [0.0; DUAL_AMMO_TICS],
            weapons: [false; 2],
            shield_present: false,
            boost_flash: false,
            turbo: None,
            linked: false,
            hull_text: String::with_capacity(24),
            shield_text: String::with_capacity(24),
            speed_text: String::with_capacity(24),
            ammo_text: String::with_capacity(24),
            wanted: None,
            attempted: None,
            picture: None,
        }
    }
}

impl State {
    /// Settle this frame's meters. Allocates only when the riding vehicle changes.
    pub(crate) fn update(&mut self, sample: &Sample) {
        self.doubled = crosshair_doubled(sample.number, sample.dynamic_crosshair);
        match sample.profile.filter(|_| sample.number != 0) {
            Some(profile) => {
                if self.wanted.as_deref() != profile.crosshair.as_deref() {
                    self.wanted.clone_from(&profile.crosshair);
                }
            }
            None => self.wanted = None,
        }
        self.shown = shown(sample);
        let Some(profile) = sample.profile.filter(|_| self.shown) else {
            self.hides_player = false;
            return;
        };
        self.hides_player = profile.hide_rider;
        self.hull = tic_alphas(sample.hull as f32, profile.hull as f32, HULL_TICS);
        self.shield = tic_alphas(sample.shield as f32, profile.shields as f32, SHIELD_TICS);
        self.speed = tic_alphas(sample.speed, profile.speed_max, SPEED_TICS);
        self.weapons = profile.has_weapon;
        let ammo = |slot: usize| sample.ammo[slot] as f32;
        let max = |slot: usize| profile.ammo_max[slot] as f32;
        match profile.has_weapon {
            [true, false] => self.ammo = tic_alphas(ammo(0), max(0), AMMO_TICS),
            [true, true] => {
                self.ammo_upper = tic_alphas(ammo(0), max(0), AMMO_TICS);
                self.ammo_lower = tic_alphas(ammo(1), max(1), AMMO_TICS);
            }
            _ => {}
        }
        self.shield_present = profile.shields > 0;
        self.turbo = sample
            .turbo_time
            .map(|turbo| turbo_charge(sample.time, turbo, profile.turbo_recharge));
        self.boost_flash = sample.turbo_time.is_some_and(|turbo| {
            turbo_burning(sample.time, turbo) && (sample.time.div_euclid(FLASH_MS) & 1) == 1
        });
        self.linked = profile.always_linked || sample.linked;
        self.hull_text.clear();
        self.shield_text.clear();
        self.speed_text.clear();
        self.ammo_text.clear();
        let _ = write!(self.hull_text, "HULL {}", sample.hull.max(0));
        let _ = write!(self.shield_text, "SHIELD {}", sample.shield.max(0));
        // The speed is shown in game units a second, as the `cg_speedometer` default.
        let speed = if sample.speed.is_finite() {
            sample.speed.clamp(-99_999.0, 99_999.0)
        } else {
            0.0
        };
        let _ = write!(self.speed_text, "SPD {}", speed.round() as i32);
        match profile.has_weapon {
            [true, false] => {
                let _ = write!(self.ammo_text, "AMMO {}", sample.ammo[0].max(0));
            }
            [true, true] => {
                let _ = write!(
                    self.ammo_text,
                    "{} / {}",
                    sample.ammo[0].max(0),
                    sample.ammo[1].max(0)
                );
            }
            _ => {}
        }
    }

    /// No vehicle this frame (no snapshot, or a menu without a game).
    pub(crate) fn clear(&mut self) {
        self.shown = false;
        self.hides_player = false;
        self.doubled = false;
        self.wanted = None;
    }

    /// The vehicle HUD is up and the vehicle hides its rider, so stock draws no player
    /// status HUD (`CG_DrawVehicleHud` returns false).
    pub(crate) fn hides_player(&self) -> bool {
        self.hides_player
    }

    /// Hide the player's own status and weapon HUD where stock does; the vehicle's own
    /// HUD (`visibility.vehicle`) stays.
    pub(crate) fn apply(&self, mut visibility: HudVisibility) -> HudVisibility {
        if self.hides_player {
            visibility.status = false;
            visibility.weapon = false;
        }
        visibility
    }

    /// Load the vehicle's crosshair picture when the riding vehicle changed. `load`
    /// decodes and uploads the named picture and answers its texture, if any; it runs
    /// once per vehicle, never per frame.
    pub(crate) fn ensure_picture(&mut self, load: impl FnOnce(&str) -> Option<TextureId>) {
        let Some(name) = self.wanted.as_deref() else {
            return;
        };
        if self.attempted.as_deref() == Some(name) {
            return;
        }
        self.picture = load(name);
        self.attempted = Some(name.to_owned());
    }

    /// The riding vehicle's crosshair picture, if its `.veh` names one that loaded.
    pub(crate) fn crosshair_picture(&self) -> Option<TextureId> {
        self.picture.filter(|_| self.wanted.is_some())
    }

    /// The crosshair look to draw: doubled in a vehicle as `CG_DrawCrosshair` does.
    pub(crate) fn crosshair_look(&self, look: super::crosshair::Look) -> super::crosshair::Look {
        if self.doubled {
            look.in_vehicle()
        } else {
            look
        }
    }

    /// Factor on the procedural cross's size, which `hud.wgsl` draws without a picture.
    pub(crate) fn crosshair_factor(&self, look: super::crosshair::Look) -> f32 {
        self.crosshair_look(look).size / look.size.max(f32::EPSILON)
    }

    /// Strings the HUD's text commands name.
    pub(super) fn text(&self, id: u32) -> &str {
        match id {
            HULL_TEXT => &self.hull_text,
            SHIELD_TEXT => &self.shield_text,
            SPEED_TEXT => &self.speed_text,
            AMMO_TEXT => &self.ammo_text,
            _ => "",
        }
    }

    /// Append this frame's commands. `user_scale` is `cg_hudScale`; the panel stays
    /// centred and grows from the bottom edge.
    pub(super) fn emit(
        &self,
        draw: &mut DrawList,
        _theme: Theme,
        viewport: [f32; 2],
        user_scale: f32,
    ) {
        if !self.shown {
            return;
        }
        let unit = viewport[1] / 480.0 * user_scale.clamp(0.25, 2.0);
        let left = viewport[0] * 0.5 - strip::WIDTH * 0.5 * unit;
        let top = viewport[1] - (strip::MARGIN + strip::HEIGHT) * unit;
        let place =
            |[x, y, w, h]: [f32; 4]| Rect::new(left + x * unit, top + y * unit, w * unit, h * unit);
        let rounded = |draw: &mut DrawList, rect: [f32; 4], radius: f32, color: Color| {
            let _ = draw.push(DrawCommand::RoundedRect {
                rect: place(rect),
                radius: radius * unit,
                color,
            });
        };
        let faded =
            |color: Color, alpha: f32| Color::new(color.r, color.g, color.b, color.a * alpha);
        // A meter: its segments on the track, each lit as far as the value covers it.
        let meter = |draw: &mut DrawList, [x, y, w, h]: [f32; 4], alphas: &[f32], color: Color| {
            let count = alphas.len().max(1) as f32;
            let each = (w - strip::SEGMENT_GAP * (count - 1.0)) / count;
            for (index, alpha) in alphas.iter().enumerate() {
                let at = [x + index as f32 * (each + strip::SEGMENT_GAP), y, each, h];
                rounded(draw, at, strip::SEGMENT_RADIUS, tint::TRACK);
                if *alpha > 0.0 {
                    rounded(draw, at, strip::SEGMENT_RADIUS, faded(color, *alpha));
                }
            }
        };

        rounded(
            draw,
            [0.0, 0.0, strip::WIDTH, strip::HEIGHT],
            strip::RADIUS,
            tint::PANEL,
        );
        meter(draw, strip::HULL_BAR, &self.hull, tint::HULL);
        let group = |index: usize| {
            [
                strip::group_x(index),
                strip::GROUP_Y,
                strip::GROUP_WIDTH,
                strip::GROUP_HEIGHT,
            ]
        };
        let speed_color = if self.boost_flash {
            tint::BOOST
        } else {
            tint::SPEED
        };
        meter(draw, group(0), &self.speed, speed_color);
        if self.shield_present {
            meter(draw, group(1), &self.shield, tint::SHIELD);
        }
        let ammo_x = strip::group_x(2);
        match self.weapons {
            [true, false] => meter(draw, group(2), &self.ammo, tint::AMMO),
            [true, true] => {
                let row = |y: f32| [ammo_x, y, strip::GROUP_WIDTH, strip::ROW_HEIGHT];
                meter(draw, row(strip::GROUP_Y), &self.ammo_upper, tint::AMMO);
                meter(draw, row(strip::LOWER_Y), &self.ammo_lower, tint::AMMO);
            }
            _ => {}
        }

        // Turbo recharge fills from the bottom; the weapons-linked mark is a cyan pill.
        if let Some((fraction, ready)) = self.turbo {
            let [x, y, w, h] = strip::TURBO;
            rounded(draw, strip::TURBO, w * 0.5, tint::TRACK);
            let filled = h * fraction;
            if filled > 0.0 {
                let color = if ready { tint::READY } else { tint::CHARGING };
                rounded(draw, [x, y + h - filled, w, filled], w * 0.5, color);
            }
        }
        if self.linked {
            rounded(draw, strip::LINKED, strip::LINKED[2] * 0.5, tint::LINKED);
        }

        // Values: the hull's above its bar, the rest under their meters.
        let label = |draw: &mut DrawList, id: u32, rect: [f32; 4], color: Color| {
            let _ = draw.push(DrawCommand::Text {
                rect: place(rect),
                text: TextId(id),
                size: rect[3] * 0.9 * unit,
                color,
                align: TextAlign::Start,
                overflow: TextOverflow::Clip,
                weight: FontWeight::Semibold,
                letter_spacing: 0.0,
            });
        };
        label(draw, HULL_TEXT, strip::HULL_LABEL, tint::HULL_TEXT);
        let under = |index: usize| {
            [
                strip::group_x(index),
                strip::LABEL_Y,
                strip::GROUP_WIDTH,
                strip::LABEL_HEIGHT,
            ]
        };
        label(draw, SPEED_TEXT, under(0), tint::SPEED_TEXT);
        if self.shield_present {
            label(draw, SHIELD_TEXT, under(1), tint::SHIELD_TEXT);
        }
        if self.weapons[0] {
            label(draw, AMMO_TEXT, under(2), tint::AMMO_TEXT);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjk_game_jka::vehicle_fields::VehicleWeaponStats;

    fn profile() -> Profile {
        Profile {
            hull: 120,
            shields: 50,
            speed_max: 400.0,
            turbo_recharge: 3_000,
            ammo_max: [100, 40],
            has_weapon: [true, false],
            always_linked: false,
            hide_rider: true,
            crosshair: Some("gfx/hud/ship_crosshair".to_owned()),
        }
    }

    fn sample(profile: &Profile) -> Sample<'_> {
        Sample {
            number: 7,
            piloted: true,
            profile: Some(profile),
            alive: true,
            hull: 120,
            shield: 50,
            speed: 400.0,
            ammo: [100, 40],
            linked: false,
            turbo_time: None,
            time: 10_000,
            dynamic_crosshair: 1,
        }
    }

    #[test]
    fn tics_light_in_order_and_fade_the_partial_one() {
        // Five tics of 20 over 100: 50 covers two and a half.
        assert_eq!(tic_alphas::<5>(50.0, 100.0, 5), [1.0, 1.0, 0.5, 0.0, 0.0]);
        assert_eq!(tic_alphas::<5>(100.0, 100.0, 5), [1.0; 5]);
        assert_eq!(tic_alphas::<5>(0.0, 100.0, 5), [0.0; 5]);
        // More than the maximum is clamped by the tic count.
        assert_eq!(tic_alphas::<5>(250.0, 100.0, 5), [1.0; 5]);
        // Twelve hull tics.
        let hull = tic_alphas::<12>(30.0, 120.0, 12);
        assert_eq!(hull[..3], [1.0, 1.0, 1.0]);
        assert_eq!(hull[3..], [0.0; 9]);
    }

    #[test]
    fn tics_survive_bad_values() {
        assert_eq!(tic_alphas::<5>(-20.0, 100.0, 5), [0.0; 5]);
        assert_eq!(tic_alphas::<5>(f32::NAN, 100.0, 5), [0.0; 5]);
        assert_eq!(tic_alphas::<5>(f32::INFINITY, 100.0, 5), [0.0; 5]);
        // No maximum: any value covers every tic, as stock's zero-width tics do.
        assert_eq!(tic_alphas::<5>(10.0, 0.0, 5), [1.0; 5]);
        assert_eq!(tic_alphas::<5>(10.0, f32::NAN, 5), [1.0; 5]);
        // Zero divisions count as one.
        assert_eq!(tic_alphas::<5>(10.0, 100.0, 0), [0.1, 0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn two_weapon_rows_draw_four_tics_of_five() {
        // Stock divides by five but loops four times: a full weapon fills four tics.
        assert_eq!(tic_alphas::<DUAL_AMMO_TICS>(40.0, 40.0, 5), [1.0; 4]);
        assert_eq!(
            tic_alphas::<DUAL_AMMO_TICS>(16.0, 40.0, 5),
            [1.0, 1.0, 0.0, 0.0]
        );
    }

    #[test]
    fn turbo_charges_towards_ready() {
        assert_eq!(turbo_charge(1_000, 1_000, 3_000), (0.0, false));
        assert_eq!(turbo_charge(2_500, 1_000, 3_000), (0.5, false));
        assert_eq!(turbo_charge(4_000, 1_000, 3_000), (1.0, false));
        assert_eq!(turbo_charge(4_001, 1_000, 3_000), (1.0, true));
        // The turbo still burns (its time lies ahead): nothing has charged yet.
        assert_eq!(turbo_charge(500, 1_000, 3_000), (0.0, false));
        // A vehicle without a recharge time is always ready.
        assert_eq!(turbo_charge(100, 100, 0), (0.0, false));
        assert_eq!(turbo_charge(101, 100, 0), (1.0, true));
        assert_eq!(turbo_charge(i32::MAX, i32::MIN, 3_000), (1.0, true));
        assert!(turbo_burning(1_000, 1_000));
        assert!(!turbo_burning(1_001, 1_000));
    }

    #[test]
    fn the_hud_shows_for_a_living_pilot_with_a_known_vehicle() {
        let profile = profile();
        assert!(shown(&sample(&profile)));
        let edits: [fn(&mut Sample); 4] = [
            |s| s.number = 0,
            |s| s.piloted = false,
            |s| s.profile = None,
            |s| s.alive = false,
        ];
        for edit in edits {
            let mut changed = sample(&profile);
            edit(&mut changed);
            assert!(!shown(&changed));
        }
    }

    #[test]
    fn only_a_shown_hud_of_a_rider_hiding_vehicle_hides_the_status_hud() {
        let mut state = State::default();
        let mut open = profile();
        let hud = HudVisibility::from_console(None);
        state.update(&sample(&open));
        let hidden = state.apply(hud);
        assert!(!hidden.status && !hidden.weapon);
        assert!(hidden.crosshair && hidden.hud, "the rest of the HUD stays");
        open.hide_rider = false;
        state.update(&sample(&open));
        let kept = state.apply(hud);
        assert!(kept.status && kept.weapon);
        // A pilot whose vehicle is not known yet keeps the player's HUD.
        open.hide_rider = true;
        let mut unknown = sample(&open);
        unknown.profile = None;
        state.update(&unknown);
        assert!(state.apply(hud).status);
        state.update(&sample(&open));
        state.clear();
        assert!(state.apply(hud).status);
    }

    #[test]
    fn update_fills_meters_and_text_from_the_samples() {
        let profile = profile();
        let mut state = State::default();
        let mut s = sample(&profile);
        s.hull = 60;
        s.shield = 25;
        s.speed = 200.0;
        s.ammo = [50, 0];
        s.turbo_time = Some(8_500);
        s.linked = true;
        state.update(&s);
        assert!(state.shown);
        assert_eq!(state.hull[5], 1.0);
        assert_eq!(state.hull[6], 0.0);
        assert_eq!(state.shield, [1.0, 1.0, 0.5, 0.0, 0.0]);
        assert_eq!(state.speed, [1.0, 1.0, 0.5, 0.0, 0.0]);
        assert_eq!(state.ammo, [1.0, 1.0, 0.5, 0.0, 0.0]);
        // 1500 ms of 3000 since the turbo time.
        assert_eq!(state.turbo, Some((0.5, false)));
        assert!(state.linked);
        assert_eq!(state.text(HULL_TEXT), "HULL 60");
        assert_eq!(state.text(SHIELD_TEXT), "SHIELD 25");
        assert_eq!(state.text(SPEED_TEXT), "SPD 200");
        assert_eq!(state.text(AMMO_TEXT), "AMMO 50");
        // Prediction off: the turbo meter has no data and stays out.
        s.turbo_time = None;
        s.linked = false;
        state.update(&s);
        assert_eq!(state.turbo, None);
        assert!(!state.linked && !state.boost_flash);
    }

    #[test]
    fn the_turbo_flashes_the_speed_every_200_ms_while_it_burns() {
        let profile = profile();
        let mut state = State::default();
        let mut s = sample(&profile);
        s.turbo_time = Some(10_500);
        s.time = 10_000;
        state.update(&s);
        assert!(!state.boost_flash, "10000 / 200 is even");
        s.time = 10_200;
        state.update(&s);
        assert!(state.boost_flash);
        s.time = 10_400;
        state.update(&s);
        assert!(!state.boost_flash);
        s.time = 10_700;
        state.update(&s);
        assert!(!state.boost_flash, "the turbo has ended");
    }

    #[test]
    fn two_weapons_split_the_ammunition_rows_and_always_linked_shows_the_mark() {
        let mut two = profile();
        two.has_weapon = [true, true];
        two.always_linked = true;
        let mut state = State::default();
        let mut s = sample(&two);
        s.ammo = [100, 16];
        state.update(&s);
        assert_eq!(state.ammo_upper, [1.0; 4]);
        assert_eq!(state.ammo_lower, [1.0, 1.0, 0.0, 0.0]);
        assert_eq!(state.text(AMMO_TEXT), "100 / 16");
        assert!(state.linked);
        let mut none = profile();
        none.has_weapon = [false, false];
        state.update(&sample(&none));
        assert_eq!(state.text(AMMO_TEXT), "");
    }

    #[test]
    fn every_rectangle_stays_inside_the_panel() {
        let fits = |[x, y, w, h]: [f32; 4]| {
            x >= 0.0 && y >= 0.0 && x + w <= strip::WIDTH && y + h <= strip::HEIGHT
        };
        for rect in [
            strip::HULL_LABEL,
            strip::HULL_BAR,
            strip::TURBO,
            strip::LINKED,
        ] {
            assert!(fits(rect), "{rect:?}");
        }
        for index in 0..3 {
            let x = strip::group_x(index);
            for rect in [
                [x, strip::GROUP_Y, strip::GROUP_WIDTH, strip::GROUP_HEIGHT],
                [x, strip::LOWER_Y, strip::GROUP_WIDTH, strip::ROW_HEIGHT],
                [x, strip::LABEL_Y, strip::GROUP_WIDTH, strip::LABEL_HEIGHT],
            ] {
                assert!(fits(rect), "{rect:?}");
            }
        }
        // The turbo and linked pills stay clear of the meters between them.
        assert!(strip::TURBO[0] + strip::TURBO[2] <= strip::group_x(0));
        assert!(strip::group_x(2) + strip::GROUP_WIDTH <= strip::LINKED[0]);
    }

    #[test]
    fn emit_stays_centred_and_above_the_bottom_edge_at_any_size() {
        let profile = profile();
        let mut state = State::default();
        let mut s = sample(&profile);
        s.turbo_time = Some(9_000);
        s.linked = true;
        state.update(&s);
        for (viewport, scale) in [
            ([1_920.0, 1_080.0], 1.0),
            ([3_840.0, 2_160.0], 1.0),
            ([3_840.0, 2_160.0], 1.5),
            ([5_120.0, 1_440.0], 1.0),
        ] {
            let mut draw = DrawList::new(128);
            state.emit(&mut draw, Theme::default(), viewport, scale);
            assert!(draw.commands().len() > 20);
            let (mut left, mut right) = (f32::MAX, f32::MIN);
            for command in draw.commands() {
                let rect = match command {
                    DrawCommand::RoundedRect { rect, .. } | DrawCommand::Text { rect, .. } => rect,
                    other => panic!("unexpected {other:?}"),
                };
                assert!(rect.y + rect.height <= viewport[1], "{viewport:?}");
                assert!(rect.x >= 0.0 && rect.x + rect.width <= viewport[0]);
                left = left.min(rect.x);
                right = right.max(rect.x + rect.width);
            }
            // The panel is centred.
            let middle = (left + right) * 0.5;
            let unit = viewport[1] / 480.0 * scale;
            assert!(
                (middle - viewport[0] * 0.5).abs() < 6.0 * unit,
                "{viewport:?}"
            );
        }
        // Nothing is drawn while the HUD is not shown.
        let mut draw = DrawList::new(16);
        state.clear();
        state.emit(&mut draw, Theme::default(), [1_920.0, 1_080.0], 1.0);
        assert!(draw.commands().is_empty());
    }

    #[test]
    fn emitted_text_ids_resolve_to_this_states_strings() {
        let profile = profile();
        let mut state = State::default();
        state.update(&sample(&profile));
        let mut draw = DrawList::new(128);
        state.emit(&mut draw, Theme::default(), [1_920.0, 1_080.0], 1.0);
        let texts: Vec<_> = draw
            .commands()
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { text, .. } => Some(state.text(text.0)),
                _ => None,
            })
            .collect();
        assert_eq!(texts, ["HULL 120", "SPD 400", "SHIELD 50", "AMMO 100"]);
        assert!(TEXT_FIRST <= HULL_TEXT && AMMO_TEXT == TEXT_LAST);
    }

    #[test]
    fn the_crosshair_doubles_in_a_vehicle_unless_selective() {
        assert!(crosshair_doubled(7, 0));
        assert!(crosshair_doubled(7, 1));
        assert!(!crosshair_doubled(7, 2));
        assert!(!crosshair_doubled(7, 3));
        assert!(!crosshair_doubled(0, 1));
        let profile = profile();
        let mut state = State::default();
        let scaled = super::super::crosshair::Look::new(1, 24.0, true);
        assert_eq!(state.crosshair_look(scaled), scaled);
        assert_eq!(state.crosshair_factor(scaled), 1.0);
        state.update(&sample(&profile));
        assert_eq!(state.crosshair_look(scaled).size, 48.0);
        assert_eq!(state.crosshair_factor(scaled), 2.0);
        // Pixel-sized crosshairs (unscaled, or picture 10) keep their size.
        let unscaled = super::super::crosshair::Look::new(1, 24.0, false);
        let dot = super::super::crosshair::Look::new(10, 24.0, true);
        assert_eq!(state.crosshair_look(unscaled), unscaled);
        assert_eq!(state.crosshair_look(dot), dot);
        assert_eq!(state.crosshair_factor(dot), 1.0);
        // Riding without a known vehicle still doubles (it only needs m_iVehicleNum).
        let mut unknown = sample(&profile);
        unknown.profile = None;
        state.update(&unknown);
        assert_eq!(state.crosshair_factor(scaled), 2.0);
    }

    #[test]
    fn the_vehicle_picture_loads_once_per_name() {
        let profile = profile();
        let mut state = State::default();
        let mut loads = Vec::new();
        state.ensure_picture(|name| {
            loads.push(name.to_owned());
            Some(TextureId(1))
        });
        assert!(loads.is_empty(), "not riding yet");
        state.update(&sample(&profile));
        for _ in 0..3 {
            state.ensure_picture(|name| {
                loads.push(name.to_owned());
                None
            });
        }
        assert_eq!(
            loads,
            ["gfx/hud/ship_crosshair"],
            "a failed load is not retried"
        );
        assert_eq!(state.crosshair_picture(), None);
        let mut other = profile.clone();
        other.crosshair = Some("gfx/hud/other".to_owned());
        state.update(&sample(&other));
        state.ensure_picture(|name| {
            loads.push(name.to_owned());
            Some(TextureId(9))
        });
        assert_eq!(loads.len(), 2);
        assert_eq!(state.crosshair_picture(), Some(TextureId(9)));
        // On foot the picture is not used, and a vehicle without one uses the ordinary.
        state.clear();
        assert_eq!(state.crosshair_picture(), None);
        let mut plain = profile.clone();
        plain.crosshair = None;
        state.update(&sample(&plain));
        assert_eq!(state.crosshair_picture(), None);
    }

    #[test]
    fn the_profile_reads_the_definition_like_stock() {
        let mut info = VehicleInfo {
            armor: 800,
            shields: 400,
            speed_max: 1_000.0,
            turbo_recharge: 8_000,
            hide_rider: true,
            crosshair_shader: Some(b"gfx/hud/x".to_vec()),
            ..Default::default()
        };
        info.weapons[0] = VehicleWeaponStats {
            id: 3,
            ammo_max: 600,
            linkable: 2,
            ..Default::default()
        };
        info.weapons[1] = VehicleWeaponStats {
            id: -1,
            ammo_max: 8,
            ..Default::default()
        };
        let profile = Profile::from_info(&info);
        assert_eq!((profile.hull, profile.shields), (800, 400));
        assert_eq!(profile.speed_max, 1_000.0);
        assert_eq!(profile.turbo_recharge, 8_000);
        assert_eq!(profile.ammo_max, [600, 8]);
        // `weapon[n].ID` is tested as a boolean: -1 is a weapon, 0 is none.
        assert_eq!(profile.has_weapon, [true, true]);
        assert!(profile.always_linked && profile.hide_rider);
        assert_eq!(profile.crosshair.as_deref(), Some("gfx/hud/x"));
        info.weapons[1].id = 0;
        info.crosshair_shader = Some(Vec::new());
        let profile = Profile::from_info(&info);
        assert_eq!(profile.has_weapon, [true, false]);
        assert_eq!(profile.crosshair, None);
    }

    #[test]
    fn a_veh_block_gives_the_hud_its_maxima_and_crosshair() {
        use sjk_game_jka::vehicle_parms::{
            VehicleCapacity, VehicleFiles, VehicleRegistry, VehicleTable,
        };
        struct Quiet;
        impl VehicleRegistry for Quiet {
            fn model_index(&mut self, _: &[u8]) -> i32 {
                0
            }
            fn sound_index(&mut self, _: &[u8]) -> i32 {
                0
            }
            fn effect_index(&mut self, _: &[u8]) -> i32 {
                0
            }
            fn print(&mut self, _: &str) {}
        }
        let text: &[u8] = b"swoop
{
name swoop
type VH_SPEEDER
armor 150
shields 40
            speedMax 700
turboRecharge 5000
hideRider 1
            crosshairShader gfx/hud/swoop_crosshair
weap1AmmoMax 100
}
";
        let files = VehicleFiles::load([("swoop.veh", text)], []).unwrap();
        let mut table = VehicleTable::new(std::sync::Arc::new(files), VehicleCapacity::REFERENCE);
        let index = table.index_for_name(b"swoop", &mut Quiet).unwrap();
        let profile = Profile::from_info(table.vehicle(index).unwrap());
        assert_eq!((profile.hull, profile.shields), (150, 40));
        assert_eq!(profile.speed_max, 700.0);
        assert_eq!(profile.turbo_recharge, 5_000);
        assert_eq!(profile.ammo_max[0], 100);
        assert!(profile.hide_rider);
        assert_eq!(
            profile.crosshair.as_deref(),
            Some("gfx/hud/swoop_crosshair")
        );
    }
}
