//! A body's fade: `EV_BODYFADE` and the frames `CG_General` draws for an `ET_BODY`
//! while it fades (OpenJK `codemp/cgame/cg_event.c` `EV_BODYFADE`,
//! `codemp/cgame/cg_ents.c` the `ET_BODY` block of `CG_General`; EternalJK, which JoF
//! players use, draws the same frames).
//!
//! `BodySink` (`g_client.c`) sends the event 30 s after the body fell. cgame then sets
//! `bodyFadeTime` 60 s ahead, turns off the `head_eyes_mouth` surface, stops the
//! ragdoll, and on every frame draws the body in up to three layers by the time since
//! the event (`curTimeDif`, with `tMult = curTimeDif * 0.08` truncated):
//!
//! 1. the body itself, `RF_FORCE_ENT_ALPHA` at `254 - tMult`, until `curTimeDif * 0.1`
//!    passes 254;
//! 2. the body again with the side's shader (light: `gfx/misc/personalshield`, dark:
//!    `gfx/effects/sabers/red_glow`) and `RF_RGB_TINT` at `tMult` (at least 1); once
//!    `tMult` passes 200 it burns away from the body's origin (`RF_DISINTEGRATE2` from
//!    that moment, with the force-sight sound for light, lightning for dark);
//! 3. for the first 3.4 s, light: the saber hum (EternalJK plays it once per 100 ms,
//!    base every frame) for 2.2 s; dark: the body red with a random one of the two
//!    electric shaders and, now and then, the energy crackle.
//!
//! The side is `ircg`'s fourth operand (`CG_RestoreClientGhoul_f`).

use sjk_protocol::Snapshot;

const ENTITY_LIMIT: usize = 1_024;
/// `ET_BODY` and `ET_EVENTS` (`bg_public.h`).
const ET_BODY: u8 = 15;
const ET_EVENTS: u8 = 18;
/// `EV_BODYFADE` (`bg_public.h`), under `EV_EVENT_BITS`.
pub const EV_BODYFADE: u16 = 100;
const EVENT_MASK: u16 = 0xff;
/// `cent->bodyFadeTime = cg.time + 60000`.
pub const FADE_MILLIS: i32 = 60_000;
/// The light side's hum stops (`curTimeDif < 2200`).
const HUM_MILLIS: i32 = 2_200;
/// The third layer stops (`curTimeDif < 3400`).
const EXTRA_MILLIS: i32 = 3_400;
/// EternalJK's fix for the loud hum: at most once per 100 ms of game time.
pub const HUM_PERIOD_MILLIS: i32 = 100;
/// `gfx/misc/personalshield`, `cgs.media.playerShieldDamage`.
pub const LIGHT_SHADER: &str = "gfx/misc/personalshield";
/// `gfx/effects/sabers/red_glow`, `cgs.media.redSaberGlowShader`.
pub const DARK_SHADER: &str = "gfx/effects/sabers/red_glow";
/// `cgs.media.electricBodyShader`, `electricBody2Shader`.
pub const ELECTRIC_SHADERS: [&str; 2] = ["gfx/misc/electric", "gfx/misc/fullbodyelectric2"];
/// Played as the burn starts.
pub const LIGHT_START_SOUND: &str = "sound/weapons/force/see.wav";
pub const DARK_START_SOUND: &str = "sound/weapons/force/lightning";
/// The light side's hum and the dark side's crackle (`cgs.media.crackleSound`).
pub const HUM_SOUND: &str = "sound/weapons/saber/saberhum1.wav";
pub const CRACKLE_SOUND: &str = "sound/effects/energy_crackle.wav";

/// What `CG_General` draws of a fading body `elapsed` ms after `EV_BODYFADE`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BodyFadeFrame {
    /// Layer 1: the body at this forced alpha, or not at all.
    pub body_alpha: Option<u8>,
    /// Layer 2's `RF_RGB_TINT` grey, 1 to 200.
    pub glow: u8,
    /// Layer 2 burns away (`RF_DISINTEGRATE2`): it then draws in the burn's own colours.
    pub burning: bool,
    /// Layer 3 is drawn (the dark side's electric body) or heard.
    pub extra: bool,
    /// The light side's hum may sound.
    pub hum: bool,
}

/// `CG_General`'s `ET_BODY` fade for one frame; `None` once `bodyFadeTime` has passed.
pub fn legacy_body_fade(elapsed: i32) -> Option<BodyFadeFrame> {
    if !(0..FADE_MILLIS).contains(&elapsed) {
        return None;
    }
    // `int tMult = curTimeDif*0.08;`
    let t_mult = (f64::from(elapsed) * 0.08) as i32;
    let body = if f64::from(elapsed) * 0.1 > 254.0 {
        0
    } else {
        254 - t_mult
    };
    let burning = t_mult > 200;
    Some(BodyFadeFrame {
        body_alpha: (body >= 1).then_some(body as u8),
        glow: if burning { 200 } else { t_mult.max(1) as u8 },
        burning,
        extra: elapsed < EXTRA_MILLIS,
        hum: elapsed < HUM_MILLIS,
    })
}

/// When the burn of layer 2 starts: the first frame `tMult` passes 200
/// (`cent->dustTrailTime`), at `curTimeDif` 2513.
pub const BURN_START_MILLIS: i32 = 2_513;

/// The fade start of every body, kept from `EV_BODYFADE` in fixed storage.
///
/// The event is seen once per change of the entity's event field, as
/// `CG_CheckEvents` does, and only from an `ET_BODY` (`EV_BODYFADE event from a
/// non-corpse` is ignored). A body copied into the slot again (`ircg`) starts unfaded
/// ([`Self::reset`], `CG_BodyQueueCopy`).
pub struct LegacyBodyFadeTracker {
    started: [Option<i32>; ENTITY_LIMIT],
    signatures: [u16; ENTITY_LIMIT],
    active: [bool; ENTITY_LIMIT],
    /// The last snapshot's server time: one going back is a new gamestate.
    server_time: i32,
}

impl Default for LegacyBodyFadeTracker {
    fn default() -> Self {
        Self {
            started: [None; ENTITY_LIMIT],
            signatures: [0; ENTITY_LIMIT],
            active: [false; ENTITY_LIMIT],
            server_time: i32::MIN,
        }
    }
}

impl LegacyBodyFadeTracker {
    /// Observe one snapshot's events, at its server time (`cg.time` as it becomes
    /// the current snapshot).
    pub fn observe_snapshot(&mut self, snapshot: &Snapshot) {
        // A map change, `map_restart` or reconnect starts the server clock again:
        // no fade of the old world carries over to a body in the same slot.
        if snapshot.server_time < self.server_time {
            self.clear();
        }
        self.server_time = snapshot.server_time;
        let mut present = [false; ENTITY_LIMIT];
        for entity in &snapshot.entities {
            let index = usize::from(entity.number());
            if index >= ENTITY_LIMIT || entity.entity_type() >= ET_EVENTS {
                continue;
            }
            present[index] = true;
            if entity.entity_type() != ET_BODY {
                // The slot holds something else now; a body put back in it later
                // starts unfaded.
                self.started[index] = None;
            }
            // `previousEvent` is remembered even when it is no event (0), so the
            // next body's identical `EV_BODYFADE` in a reused slot is seen; an
            // entity back in the snapshot starts from 0 (`CG_ResetEntity`).
            let signature = entity.event();
            let previous = if self.active[index] {
                self.signatures[index]
            } else {
                0
            };
            self.active[index] = true;
            self.signatures[index] = signature;
            if signature == previous || signature & EVENT_MASK == 0 {
                continue;
            }
            if signature & EVENT_MASK == EV_BODYFADE && entity.entity_type() == ET_BODY {
                self.started[index] = Some(snapshot.server_time);
            }
        }
        for (active, present) in self.active.iter_mut().zip(present) {
            *active &= present;
        }
    }

    /// The time `EV_BODYFADE` arrived for body `entity`, if it is fading.
    pub fn started(&self, entity: u16) -> Option<i32> {
        self.started.get(usize::from(entity)).copied().flatten()
    }

    /// A new body in slot `entity` (`CG_BodyQueueCopy` clears `bodyFadeTime`).
    pub fn reset(&mut self, entity: u16) {
        if let Some(started) = self.started.get_mut(usize::from(entity)) {
            *started = None;
        }
    }

    /// Forget every body (a new gamestate).
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_body_thins_while_the_glow_rises() {
        let start = legacy_body_fade(0).unwrap();
        assert_eq!(start.body_alpha, Some(254));
        assert_eq!(start.glow, 1);
        assert!(!start.burning && start.extra && start.hum);
        let second = legacy_body_fade(1_000).unwrap();
        assert_eq!(second.body_alpha, Some(174));
        assert_eq!(second.glow, 80);
    }

    #[test]
    fn the_glow_burns_away_after_two_and_a_half_seconds() {
        let before = legacy_body_fade(BURN_START_MILLIS - 1).unwrap();
        assert!(!before.burning);
        assert_eq!(before.glow, 200);
        let after = legacy_body_fade(BURN_START_MILLIS).unwrap();
        assert!(after.burning);
        assert_eq!(after.glow, 200);
        // The body layer still shows until curTimeDif * 0.1 passes 254.
        assert_eq!(after.body_alpha, Some(53));
        assert_eq!(legacy_body_fade(2_540).unwrap().body_alpha, Some(51));
        assert_eq!(legacy_body_fade(2_541).unwrap().body_alpha, None);
    }

    fn snapshot(server_time: i32, entities: Vec<sjk_protocol::EntityState>) -> Snapshot {
        Snapshot {
            message_sequence: 0,
            reliable_acknowledge: 0,
            server_commands: Vec::new(),
            server_time,
            delta_from: None,
            flags: 0,
            area_mask: Vec::new(),
            player: sjk_protocol::PlayerState::zero(),
            vehicle_player: None,
            entities,
            consumed_bits: 0,
        }
    }

    /// Entity `number` of type `kind` with event field `event` (`s.eType` is field 8,
    /// `s.event` field 28 of protocol 26).
    fn entity(number: u16, kind: u32, event: u32) -> sjk_protocol::EntityState {
        let mut state =
            sjk_protocol::EntityState::zero(number, &sjk_protocol::LEGACY_ENTITY_FIELDS);
        state.set_raw_field(8, kind);
        state.set_raw_field(28, event);
        state
    }

    #[test]
    fn a_body_fades_from_its_event_once() {
        let mut tracker = LegacyBodyFadeTracker::default();
        tracker.observe_snapshot(&snapshot(1_000, vec![entity(40, 15, 0)]));
        assert_eq!(tracker.started(40), None);
        tracker.observe_snapshot(&snapshot(2_000, vec![entity(40, 15, 0x100 | 100)]));
        assert_eq!(tracker.started(40), Some(2_000));
        // The same event in the next snapshot is not a new one.
        tracker.observe_snapshot(&snapshot(2_050, vec![entity(40, 15, 0x100 | 100)]));
        assert_eq!(tracker.started(40), Some(2_000));
        // A new body in the slot starts unfaded.
        tracker.reset(40);
        assert_eq!(tracker.started(40), None);
        // Not from a non-corpse, nor EV_PUSHED1 (125).
        tracker.observe_snapshot(&snapshot(3_000, vec![entity(41, 1, 0x100 | 100)]));
        tracker.observe_snapshot(&snapshot(3_000, vec![entity(42, 15, 0x100 | 125)]));
        assert_eq!(tracker.started(41), None);
        assert_eq!(tracker.started(42), None);
    }

    #[test]
    fn a_reused_slot_fades_again_after_its_event_clears() {
        // The body queue's slot stays in every snapshot: the next body's fade carries
        // the same value, seen again once the event field went back to 0.
        let mut tracker = LegacyBodyFadeTracker::default();
        tracker.observe_snapshot(&snapshot(1_000, vec![entity(40, 15, 0x100 | 100)]));
        assert_eq!(tracker.started(40), Some(1_000));
        tracker.reset(40);
        tracker.observe_snapshot(&snapshot(2_000, vec![entity(40, 15, 0)]));
        assert_eq!(tracker.started(40), None);
        tracker.observe_snapshot(&snapshot(3_000, vec![entity(40, 15, 0x100 | 100)]));
        assert_eq!(tracker.started(40), Some(3_000));
    }

    #[test]
    fn a_new_gamestate_forgets_every_fade() {
        let mut tracker = LegacyBodyFadeTracker::default();
        tracker.observe_snapshot(&snapshot(90_000, vec![entity(40, 15, 0x100 | 100)]));
        assert_eq!(tracker.started(40), Some(90_000));
        tracker.observe_snapshot(&snapshot(1_000, vec![entity(40, 15, 0)]));
        assert_eq!(tracker.started(40), None);
    }

    #[test]
    fn the_extras_stop_and_the_fade_ends() {
        assert!(!legacy_body_fade(2_200).unwrap().hum);
        assert!(legacy_body_fade(3_399).unwrap().extra);
        assert!(!legacy_body_fade(3_400).unwrap().extra);
        assert!(legacy_body_fade(59_999).is_some());
        assert_eq!(legacy_body_fade(60_000), None);
        assert_eq!(legacy_body_fade(-1), None);
    }
}
