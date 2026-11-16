//! JoF EJK's Force wheel: the order Force selection walks and shows, with the
//! JoF JA+ abilities that are not Force powers (Stasis, Repulse, Dash).
//!
//! A JoF JA+ server grants those abilities through spare `forcePowersKnown`
//! bits past the 18 real powers (JoF EJK `q_shared.h`: `STASIS_KNOWN_BIT`,
//! `REPULSE_KNOWN_BIT`, `DASH_KNOWN_BIT`). The client gives each a pseudo-slot
//! of the same number in the wheel; a pseudo-slot is never sent as the
//! user command's Force selection. Selecting one turns `+useforce` into the
//! ability instead (JoF EJK `cl_input.cpp` `CL_CmdButtons`): Stasis sets the
//! usercmd button the server reads, Repulse and Dash send their server command
//! once per press. Everywhere else those bits are clear and the wheel is the
//! stock order (`bg_misc.c` `forcePowerSorted`).
//!
//! On JA+, merc mode replaces Force Lightning with a flamethrower; the wheel
//! shows it as such ([`FlamethrowerOverride`]).

use crate::selection::FORCE_ORDER;
use sjk_protocol::PlayerState;

/// Real Force powers (`NUM_FORCE_POWERS`).
pub const REAL_POWERS: u8 = 18;
/// Force Stasis (JoF JA+ V58): its known bit and wheel pseudo-slot.
pub const STASIS: u8 = 18;
/// Force Repulse (JoF JA+ V71): its known bit and wheel pseudo-slot.
pub const REPULSE: u8 = 19;
/// Force Dash (JoF JA+ V69): its known bit and wheel pseudo-slot.
pub const DASH: u8 = 20;
/// Fixed adapter storage for real powers and optional extension entries.
pub const MAX_SLOTS: usize = 22;
/// Known bits of the original three pseudo-slots a server grants.
pub const PSEUDO_MASK: u32 = (1 << STASIS) | (1 << REPULSE) | (1 << DASH);

/// A server-granted pseudo-power's command and position in the stock wheel.
/// Evidence: EJK #110/#112/#117 and #338 `CG_BuildForceWheel`/`CL_CmdButtons`.
#[derive(Debug)]
pub struct Ability {
    /// Local wheel slot, never a protocol force selection.
    pub slot: u8,
    /// Real power whose position anchors this ability, even when unknown.
    pub anchor: u8,
    /// Insert before the anchor rather than after it.
    pub before: bool,
    /// Reliable command for an instant ability; Stasis uses its held button.
    pub command: Option<&'static str>,
    /// Optional serverinfo flag required in addition to the known bit.
    pub server_flag: Option<&'static str>,
    /// Require a living player outside spectator/follow states.
    pub live_only: bool,
    /// Host label for this optional entry.
    pub label: &'static str,
    /// Installed image requested only while this extension is active.
    pub icon: Option<&'static str>,
    /// Stock icon index used when the optional image is absent.
    pub fallback: u8,
}

/// Compatibility contracts; adding a pseudo-power adds a row, not a server-name check.
pub const ABILITIES: &[Ability] = &[
    Ability {
        slot: DASH,
        label: "Dash",
        icon: None,
        fallback: 2,
        anchor: 2,
        before: true,
        command: Some("force_dash"),
        server_flag: None,
        live_only: false,
    },
    Ability {
        slot: STASIS,
        label: "Stasis",
        icon: None,
        fallback: 1,
        anchor: 14,
        before: false,
        command: None,
        server_flag: None,
        live_only: false,
    },
    Ability {
        slot: REPULSE,
        label: "Repulse",
        icon: None,
        fallback: 3,
        anchor: 14,
        before: false,
        command: Some("force_repulse"),
        server_flag: None,
        live_only: false,
    },
];

/// Known-bit mask contributed by optional extension metadata.
pub fn extension_mask(extensions: &[Ability]) -> u32 {
    extensions
        .iter()
        .filter(|entry| entry.slot < 32)
        .fold(0, |mask, entry| mask | (1 << entry.slot))
}

/// Filter raw grants by the host's active extension metadata. No mod's server
/// identification or feature flags are interpreted in base SJK.
pub fn known(player: &PlayerState, extensions: &[Ability]) -> u32 {
    let mut known = player.raw_field(51).unwrap_or(0)
        & (((1 << REAL_POWERS) - 1) | PSEUDO_MASK | extension_mask(extensions));
    if player.health() <= 0
        || matches!(player.movement_type(), 3 | 4)
        || player.movement_flags() & 4096 != 0
    {
        for entry in extensions
            .iter()
            .filter(|entry| entry.live_only && entry.slot < 32)
        {
            known &= !(1 << entry.slot);
        }
    }
    known
}

/// `BUTTON_FORCEPOWER`, the `+useforce` bit (`bg_public.h`).
const FORCE_POWER_BUTTON: u16 = 1 << 9;
/// `STASIS_ENGAGE_BTN`: the usercmd bit a JoF JA+ server reads to engage Stasis.
pub const STASIS_BUTTON: u16 = 1 << 14;

const LEVITATION: u8 = 1;
const LIGHTNING: u8 = 7;
const SEE: u8 = 14;
const SPEED: u8 = 2;
const SABER_POWERS: [u8; 3] = [15, 16, 17];

/// JA+ merc mode (`eFlags` bit 12, also read by `CG_DrawHolsteredSaber`).
const EF_MERC: u32 = 0x1000;
/// `EF_BOBAFIRE`: the flamethrower is firing.
const EF_BOBAFIRE: u32 = 1 << 17;
/// `EF_EMPOWERED` (JoF): `amempower` overrides merc mode.
const EF_EMPOWERED: u32 = 1 << 18;
/// `playerState_t::activeForcePass`, protocol-26 netfield 72.
const ACTIVE_FORCE_PASS: usize = 72;
/// `PERS_SPAWN_COUNT`.
const SPAWN_COUNT: usize = 4;

/// Whether `slot` is a pseudo-slot rather than a real Force power.
pub fn is_pseudo(slot: u8) -> bool {
    slot >= REAL_POWERS
}

/// `ForcePower_Valid` (JoF EJK `cg_draw.c`): a known power that can be
/// selected (not Jump or the saber powers), or a granted pseudo-slot.
pub fn valid(known: u32, slot: u8) -> bool {
    if slot >= 32 {
        return false;
    }
    if is_pseudo(slot) {
        return slot <= DASH && known & (1 << slot) != 0;
    }
    slot != LEVITATION && !SABER_POWERS.contains(&slot) && known & (1 << slot) != 0
}

/// `CG_BuildForceWheel` (JoF EJK `cg_main.c`): the valid powers in stock display
/// order, with Stasis and Repulse right after Sense, Dash right before Speed, and
/// a pseudo-slot whose anchor power is not known at the end. Returns the slots
/// and how many there are.
pub fn build(known: u32) -> ([u8; MAX_SLOTS], usize) {
    let (stasis, repulse, dash) = (
        valid(known, STASIS),
        valid(known, REPULSE),
        valid(known, DASH),
    );
    let mut slots = [0; MAX_SLOTS];
    let mut count = 0;
    let mut push = |slot: u8| {
        slots[count] = slot;
        count += 1;
    };
    let (mut placed, mut dash_placed) = (false, false);
    for power in FORCE_ORDER {
        if !valid(known, power) {
            continue;
        }
        if dash && power == SPEED && !dash_placed {
            push(DASH);
            dash_placed = true;
        }
        push(power);
        if (stasis || repulse) && power == SEE && !placed {
            if stasis {
                push(STASIS);
            }
            if repulse {
                push(REPULSE);
            }
            placed = true;
        }
    }
    if dash && !dash_placed {
        push(DASH);
    }
    if !placed {
        if stasis {
            push(STASIS);
        }
        if repulse {
            push(REPULSE);
        }
    }
    (slots, count)
}

/// Extension-aware validation; base callers retain the original three slots.
pub fn valid_with(known: u32, slot: u8, extensions: &[Ability]) -> bool {
    valid(known, slot)
        || (slot < 32
            && known & (1 << slot) != 0
            && extensions.iter().any(|entry| entry.slot == slot))
}

/// Build an extended wheel using documented insertion metadata. With no loaded
/// extension, the original base order (including orphan placement) is retained.
pub fn build_with(known: u32, extensions: &[Ability]) -> ([u8; MAX_SLOTS], usize) {
    if extensions.is_empty() {
        return build(known);
    }
    let mut slots = [0; MAX_SLOTS];
    let mut count = 0;
    let mut push = |slot| {
        if count < MAX_SLOTS && valid_with(known, slot, extensions) {
            slots[count] = slot;
            count += 1;
        }
    };
    for power in FORCE_ORDER {
        for entry in ABILITIES
            .iter()
            .chain(extensions)
            .filter(|entry| entry.anchor == power && entry.before)
        {
            push(entry.slot);
        }
        push(power);
        for entry in ABILITIES
            .iter()
            .chain(extensions)
            .filter(|entry| entry.anchor == power && !entry.before)
        {
            push(entry.slot);
        }
    }
    (slots, count)
}

/// The wheel entry `direction` steps away from `current`, as
/// `CG_NextForcePower_f` and `CG_PrevForcePower_f` walk the wheel; `None` when
/// `current` is not on the wheel or the wheel is empty.
pub fn step(known: u32, current: u8, direction: i8) -> Option<u8> {
    step_with(known, current, direction, &[])
}

/// Step a host-selected extended wheel, without changing real Force selection.
pub fn step_with(known: u32, current: u8, direction: i8, extensions: &[Ability]) -> Option<u8> {
    let (slots, count) = build_with(known, extensions);
    let index = slots[..count].iter().position(|&slot| slot == current)?;
    let next = if direction > 0 {
        (index + 1) % count
    } else {
        (index + count - 1) % count
    };
    Some(slots[next])
}

/// `+useforce` while a pseudo-slot is selected (JoF EJK `CL_CmdButtons`), with
/// the press edges it needs to send Repulse and Dash once per press.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UseRemap {
    command_down: Option<u8>,
}

impl UseRemap {
    /// Rewrite one user command's `buttons` for the selected wheel entry and the
    /// held `+force_stasis`; returns the buttons and the server command to send.
    pub fn apply(
        &mut self,
        buttons: u16,
        selected: Option<u8>,
        known: u32,
        stasis_held: bool,
    ) -> (u16, Option<&'static str>) {
        self.apply_with(buttons, selected, known, stasis_held, &[])
    }

    /// Apply optional loaded-mod metadata to the same press latch.
    pub fn apply_with(
        &mut self,
        buttons: u16,
        selected: Option<u8>,
        known: u32,
        stasis_held: bool,
        extensions: &[Ability],
    ) -> (u16, Option<&'static str>) {
        let mut buttons = buttons;
        // The dedicated bind engages Stasis only where the server granted it.
        if stasis_held && valid(known, STASIS) {
            buttons |= STASIS_BUTTON;
        }
        let selected =
            selected.filter(|&slot| is_pseudo(slot) && valid_with(known, slot, extensions));
        let down = buttons & FORCE_POWER_BUTTON != 0;
        let mut command = None;
        let mut command_down = None;
        if selected == Some(STASIS) {
            if down {
                buttons = (buttons & !FORCE_POWER_BUTTON) | STASIS_BUTTON;
            }
        } else if let Some(ability) = ABILITIES
            .iter()
            .chain(extensions)
            .find(|a| Some(a.slot) == selected && a.command.is_some())
        {
            buttons &= !FORCE_POWER_BUTTON;
            if down {
                if self.command_down != Some(ability.slot) {
                    command = ability.command;
                }
                command_down = Some(ability.slot);
            }
        }
        self.command_down = command_down;
        (buttons, command)
    }
}

/// Whether JA+ merc mode turns Force Lightning into the flamethrower, latched
/// across snapshots as JoF EJK's `CG_UpdateFlamethrowerOverride`
/// (`cg_snapshot.c`) does: real lightning seen while merc is set means the merc
/// bit is stale, until the flamethrower fires again or the player respawns.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FlamethrowerOverride {
    lightning_seen: bool,
    /// Client and spawn count of the last snapshot observed.
    last: Option<(u16, u32)>,
}

impl FlamethrowerOverride {
    /// Observe the latest snapshot's player; `ja_plus` is a JA+ server.
    pub fn observe(&mut self, player: &PlayerState, ja_plus: bool) {
        let identity = (player.client_num(), player.persistent[SPAWN_COUNT]);
        let flags = player.entity_flags();
        let merc = ja_plus && flags & EF_MERC != 0;
        if self.last != Some(identity) || !merc {
            self.lightning_seen = false;
        }
        self.last = Some(identity);
        if !merc {
            return;
        }
        let pass = player.raw_field(ACTIVE_FORCE_PASS).unwrap_or(0);
        if flags & EF_BOBAFIRE != 0 {
            self.lightning_seen = false;
        } else if flags & EF_EMPOWERED == 0 && (1..=3).contains(&pass) {
            // CG_Player's lightning branch; higher passes are Drain.
            self.lightning_seen = true;
        }
    }

    /// `CG_ForceSelectUsesFlamethrower`: the wheel shows `power` as the flamethrower.
    pub fn shows_flamethrower(&self, power: u8, player: &PlayerState, ja_plus: bool) -> bool {
        let flags = player.entity_flags();
        power == LIGHTNING
            && ja_plus
            && flags & EF_MERC != 0
            && flags & EF_EMPOWERED == 0
            && !self.lightning_seen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every selectable real power.
    const ALL: u32 = ((1 << 18) - 1) & !(1 << 1) & !(7 << 15);

    fn wheel(known: u32) -> Vec<u8> {
        let (slots, count) = build(known);
        slots[..count].to_vec()
    }

    #[test]
    fn without_jof_bits_the_wheel_is_the_stock_order() {
        let expected: Vec<u8> = FORCE_ORDER
            .into_iter()
            .filter(|&p| p != 1 && p < 15)
            .collect();
        assert_eq!(wheel(ALL), expected);
        // Jump and the saber powers are never selectable, even when known.
        assert_eq!(wheel((1 << 1) | (7 << 15)), Vec::<u8>::new());
    }

    #[test]
    fn pseudo_slots_sit_after_sense_and_before_speed() {
        let known = ALL | PSEUDO_MASK;
        let order = wheel(known);
        let see = order.iter().position(|&p| p == SEE).unwrap();
        assert_eq!(&order[see..see + 3], &[SEE, STASIS, REPULSE]);
        let speed = order.iter().position(|&p| p == SPEED).unwrap();
        assert_eq!(order[speed - 1], DASH);
        assert_eq!(order.len(), 14 + 3);
    }

    #[test]
    fn pseudo_slots_without_their_anchor_go_last() {
        let known = (1 << 3) | (1 << 4) | PSEUDO_MASK;
        assert_eq!(wheel(known), [3, 4, DASH, STASIS, REPULSE]);
        assert_eq!(wheel(1 << REPULSE), [REPULSE]);
    }

    #[test]
    fn stepping_wraps_around_the_wheel_and_through_pseudo_slots() {
        let known = (1 << 14) | (1 << 3) | (1 << STASIS);
        let order = wheel(known);
        assert_eq!(order, [3, 14, STASIS]);
        assert_eq!(step(known, 14, 1), Some(STASIS));
        assert_eq!(step(known, STASIS, 1), Some(3));
        assert_eq!(step(known, 3, -1), Some(STASIS));
        assert_eq!(step(known, 0, 1), None);
        assert_eq!(step(0, 3, 1), None);
    }

    #[test]
    fn stasis_turns_useforce_into_its_button() {
        let mut remap = UseRemap::default();
        let known = 1 << STASIS;
        let (buttons, command) = remap.apply(FORCE_POWER_BUTTON | 1, Some(STASIS), known, false);
        assert_eq!(buttons, STASIS_BUTTON | 1);
        assert_eq!(command, None);
        // The dedicated bind works without the wheel, only where it is granted.
        assert_eq!(remap.apply(0, None, known, true).0, STASIS_BUTTON);
        assert_eq!(remap.apply(0, None, 0, true).0, 0);
    }

    #[test]
    fn repulse_and_dash_send_once_per_press() {
        let mut remap = UseRemap::default();
        let known = (1 << REPULSE) | (1 << DASH);
        let held = FORCE_POWER_BUTTON;
        assert_eq!(
            remap.apply(held, Some(REPULSE), known, false),
            (0, Some("force_repulse"))
        );
        assert_eq!(remap.apply(held, Some(REPULSE), known, false), (0, None));
        assert_eq!(remap.apply(0, Some(REPULSE), known, false), (0, None));
        assert_eq!(
            remap.apply(held, Some(REPULSE), known, false),
            (0, Some("force_repulse"))
        );
        assert_eq!(
            remap.apply(held, Some(DASH), known, false),
            (0, Some("force_dash"))
        );
    }

    #[test]
    fn nothing_but_the_game_and_the_server_puts_an_entry_on_the_wheel() {
        // Slot 21 was SJK's Illuminate; it is a toy now, so the bit means nothing.
        assert_eq!(wheel(1 << 21), Vec::<u8>::new());
        assert!(!valid(u32::MAX, 21));
        assert_eq!(build(u32::MAX).1, 14 + 3);
        let mut remap = UseRemap::default();
        let held = FORCE_POWER_BUTTON;
        assert_eq!(remap.apply(held, Some(21), u32::MAX, false), (held, None));
    }

    #[test]
    fn real_powers_and_revoked_slots_leave_useforce_alone() {
        let mut remap = UseRemap::default();
        let held = FORCE_POWER_BUTTON;
        assert_eq!(remap.apply(held, Some(3), u32::MAX, false), (held, None));
        assert_eq!(remap.apply(held, Some(REPULSE), 0, false), (held, None));
        assert_eq!(remap.apply(held, None, 1 << DASH, false), (held, None));
    }

    fn merc_player(flags: u32, pass: u32, spawn: u32) -> PlayerState {
        let mut player = PlayerState::default();
        player.set_raw_field(17, flags);
        player.set_raw_field(ACTIVE_FORCE_PASS, pass);
        player.persistent[SPAWN_COUNT] = spawn;
        player
    }

    #[test]
    fn merc_lightning_shows_as_the_flamethrower_until_real_lightning_is_seen() {
        let mut latch = FlamethrowerOverride::default();
        let merc = merc_player(EF_MERC, 0, 1);
        latch.observe(&merc, true);
        assert!(latch.shows_flamethrower(LIGHTNING, &merc, true));
        assert!(!latch.shows_flamethrower(LIGHTNING, &merc, false));
        assert!(!latch.shows_flamethrower(3, &merc, true));
        // Real lightning while the merc bit is set: the bit is stale.
        let zapping = merc_player(EF_MERC, 2, 1);
        latch.observe(&zapping, true);
        assert!(!latch.shows_flamethrower(LIGHTNING, &zapping, true));
        // Firing the flamethrower brings it back.
        let firing = merc_player(EF_MERC | EF_BOBAFIRE, 0, 1);
        latch.observe(&firing, true);
        assert!(latch.shows_flamethrower(LIGHTNING, &firing, true));
        // A respawn forgets what was seen.
        latch.observe(&zapping, true);
        latch.observe(&merc_player(EF_MERC, 0, 2), true);
        assert!(latch.shows_flamethrower(LIGHTNING, &merc, true));
        // Empower overrides merc.
        let empowered = merc_player(EF_MERC | EF_EMPOWERED, 0, 2);
        assert!(!latch.shows_flamethrower(LIGHTNING, &empowered, true));
    }
}
