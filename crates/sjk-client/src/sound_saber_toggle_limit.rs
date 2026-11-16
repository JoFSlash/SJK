//! A cap on how often one player's saber ignition and retraction sound, so a bound or
//! scripted toggle spam (the saber on and off every frame, or a weapon switch loop) does
//! not fill the air with them. The saber still lights and goes out as the server says;
//! only the sounds past the cap are dropped. Stock cgame has no such cap.
//!
//! It covers the server's toggle (`G_Sound` of a hilt's `soundOn`/`soundOff`, taken as
//! the nearest player's like [`super::saber_overrides`] does), `EV_SABER_UNHOLSTER` and
//! cgame's own switch sounds ([`super::saber_switch`]).

use super::MAX_CLIENTS;

/// Toggles one player may sound back to back before the cap holds.
const BURST: i32 = 4;
/// Once the burst is spent, one toggle sounds per this many milliseconds.
const INTERVAL: i32 = 500;

#[derive(Clone, Copy, Debug)]
struct Client {
    /// When the player's next toggle would be on schedule (a GCRA arrival time).
    due: i32,
    /// The server time of the player's latest toggle sound, and whether it played.
    last: Option<(i32, bool)>,
    /// The server time of the latest toggle given back ([`SaberToggleLimit::refund`]).
    refunded: Option<i32>,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            due: i32::MIN,
            last: None,
            refunded: None,
        }
    }
}

/// Each client's toggle sound budget.
#[derive(Debug, Default)]
pub(super) struct SaberToggleLimit {
    clients: [Client; MAX_CLIENTS],
}

impl SaberToggleLimit {
    /// Whether `client`'s toggle sound at server `time` plays. Every sound of one
    /// toggle (both hands, a blade skin's over them) shares its snapshot's time and
    /// gets one answer. Entities past the client range (NPCs) are not capped.
    pub(super) fn allow(&mut self, client: u16, time: i32) -> bool {
        let Some(state) = self.clients.get_mut(usize::from(client)) else {
            return true;
        };
        match state.last {
            Some((last, allowed)) if last == time => return allowed,
            // Time went back: a map restart or a demo seek.
            Some((last, _)) if time < last => *state = Client::default(),
            _ => {}
        }
        let allowed = state.due.saturating_sub((BURST - 1) * INTERVAL) <= time;
        if allowed {
            state.due = state.due.max(time).saturating_add(INTERVAL);
        }
        state.last = Some((time, allowed));
        allowed
    }

    /// Give back the toggle `client` paid for at server `time`, once however many of
    /// its sounds ask: a sound let through and then found to be a late copy of a
    /// toggle already paid for (a holstered staff's weapon switch shutdown after the
    /// one at its reach) is not a toggle of its own.
    pub(super) fn refund(&mut self, client: u16, time: i32) {
        let Some(state) = self.clients.get_mut(usize::from(client)) else {
            return;
        };
        if state.refunded != Some(time) {
            state.refunded = Some(time);
            state.due = state.due.saturating_sub(INTERVAL);
        }
    }
}

#[cfg(test)]
#[path = "sound_saber_toggle_limit_tests.rs"]
mod tests;
