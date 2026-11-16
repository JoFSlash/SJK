//! Rate-limited in-session userinfo command tracking.
//!
//! OpenJK sends `userinfo "<CVAR_USERINFO>"` through the reliable command
//! queue when any userinfo cvar changes (`codemp/client/cl_main.cpp:2157-2169`).
//!
//! The server takes at most [`INFO_CHANGE_COUNT`] changes in a window of
//! [`INFO_CHANGE_WINDOW`] from the first (`SV_UpdateUserinfo_f`,
//! `INFO_CHANGE_MIN_INTERVAL`/`INFO_CHANGE_MAX_COUNT`), and ignores the rest with
//! "Too many info changes, last info ignored". A slider dragged for a few seconds
//! (the saber's RGB) used to send one a second, so the server kept an early value
//! and every player saw the wrong blade (Kushina's report, 16/10/2026). The tracker
//! keeps to the server's window, so the latest value always arrives, and sends it
//! again when the server says it ignored one.

use sjk_network::{LegacyUserInfo, NetworkError, legacy_userinfo_payload_with_extensions};
use std::time::{Duration, Instant};

const MIN_UPDATE_INTERVAL: Duration = Duration::from_secs(1);
/// `INFO_CHANGE_MAX_COUNT`: changes the server takes in one window.
pub(crate) const INFO_CHANGE_COUNT: u32 = 3;
/// `INFO_CHANGE_MIN_INTERVAL` (6 s), and a little more for the way to the server.
pub(crate) const INFO_CHANGE_WINDOW: Duration = Duration::from_millis(6_250);
/// The server's print when it ignores a change (`SV_UpdateUserinfo_f`).
pub(crate) const INFO_CHANGE_REFUSED: &str = "Too many info changes";

/// Result of checking a prospective userinfo payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserinfoUpdateStatus {
    /// The payload equals the last successfully sent value.
    Unchanged,
    /// A changed payload is waiting for the one-second send interval.
    RateLimited,
    /// A changed payload was sent on the reliable channel.
    Sent,
}

pub(crate) struct UserinfoUpdateTracker {
    last_payload: String,
    last_sent_at: Instant,
    /// When the server's current window of changes ends, and how many it took in it.
    window_end: Option<Instant>,
    window_count: u32,
    /// The server said it ignored the last change: send the latest again.
    refused: bool,
    /// Connection identity is not part of the mutable player profile.
    guid: Option<String>,
}

impl UserinfoUpdateTracker {
    pub(crate) fn new(initial_payload: String, guid: Option<String>, now: Instant) -> Self {
        Self {
            last_payload: initial_payload,
            last_sent_at: now,
            window_end: None,
            window_count: 0,
            refused: false,
            guid,
        }
    }

    pub(crate) fn payload(
        &self,
        userinfo: &LegacyUserInfo,
        extensions: &[(&str, &str)],
    ) -> Result<String, NetworkError> {
        let mut userinfo = userinfo.clone();
        userinfo.guid.clone_from(&self.guid);
        legacy_userinfo_payload_with_extensions(&userinfo, extensions)
    }

    pub(crate) fn status(&self, payload: &str, now: Instant) -> UserinfoUpdateStatus {
        let window_full =
            self.window_count >= INFO_CHANGE_COUNT && self.window_end.is_some_and(|end| now < end);
        if payload == self.last_payload && !self.refused {
            UserinfoUpdateStatus::Unchanged
        } else if now.duration_since(self.last_sent_at) < MIN_UPDATE_INTERVAL || window_full {
            UserinfoUpdateStatus::RateLimited
        } else {
            UserinfoUpdateStatus::Sent
        }
    }

    /// The server ignored a change (its [`INFO_CHANGE_REFUSED`] print): its window is
    /// full, so the latest value goes again once the window ends.
    pub(crate) fn refuse(&mut self) {
        self.refused = true;
        self.window_count = INFO_CHANGE_COUNT;
    }

    /// Whether a refused change still waits to go again.
    pub(crate) fn refused(&self) -> bool {
        self.refused
    }

    pub(crate) fn command(payload: &str) -> Vec<u8> {
        let mut command = Vec::with_capacity(payload.len() + 11);
        command.extend_from_slice(b"userinfo \"");
        command.extend_from_slice(payload.as_bytes());
        command.push(b'"');
        command
    }

    pub(crate) fn mark_sent(&mut self, payload: String, now: Instant) {
        self.last_payload = payload;
        self.last_sent_at = now;
        self.refused = false;
        if self.window_end.is_some_and(|end| now < end) {
            self.window_count += 1;
        } else {
            self.window_end = Some(now + INFO_CHANGE_WINDOW);
            self.window_count = 1;
        }
    }
}

/// Console transport diagnostics are read without changing the wire codec.
impl crate::ClientSession {
    /// Send the optional stock team-status subscription using the existing userinfo codec.
    /// TaystJK cg_cvar.c:46-54; OpenJK g_client.c:2294-2304.
    pub fn update_userinfo_options(
        &mut self,
        userinfo: &LegacyUserInfo,
        team_overlay: Option<bool>,
        now: Instant,
    ) -> Result<UserinfoUpdateStatus, crate::ClientError> {
        self.update_userinfo_with_mod_fields(userinfo, team_overlay, &[], now)
    }

    /// Apply host-selected loaded-mod fields through the existing userinfo codec.
    pub fn update_userinfo_with_mod_fields(
        &mut self,
        userinfo: &LegacyUserInfo,
        team_overlay: Option<bool>,
        mod_fields: &[(&str, &str)],
        now: Instant,
    ) -> Result<UserinfoUpdateStatus, crate::ClientError> {
        let mut extensions = self.compat_profile.userinfo_extensions().to_vec();
        extensions.extend_from_slice(mod_fields);
        if let Some(enabled) = team_overlay {
            extensions.push(("teamoverlay", if enabled { "1" } else { "0" }));
        }
        let userinfo = self.compat_profile.userinfo_for(userinfo);
        let payload = self.userinfo_updates.payload(&userinfo, &extensions)?;
        let status = self.userinfo_updates.status(&payload, now);
        if status != UserinfoUpdateStatus::Sent {
            return Ok(status);
        }
        self.send_reliable_command(&UserinfoUpdateTracker::command(&payload))?;
        self.userinfo_updates.mark_sent(payload, now);
        Ok(UserinfoUpdateStatus::Sent)
    }

    /// Whether the server ignored a userinfo change that must go again: the host
    /// sends its userinfo again (it goes once the server's window allows).
    pub fn userinfo_refused(&self) -> bool {
        self.userinfo_updates.refused()
    }

    /// Take an expected-server OOB print for the frontend console.
    pub fn pop_server_print(&mut self) -> Option<String> {
        self.connection.as_mut().and_then(|c| c.pop_server_print())
    }

    /// How long the connected server has been silent on the sequenced channel.
    pub fn packet_silence(&self) -> Option<std::time::Duration> {
        self.connection.as_ref().and_then(|c| c.packet_silence())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drag a slider for `seconds`, a new value every 100 ms, sending whenever the
    /// tracker allows; returns the values sent and when (ms after the start).
    fn drag(
        tracker: &mut UserinfoUpdateTracker,
        start: Instant,
        seconds: u64,
    ) -> Vec<(u64, String)> {
        let mut sent = Vec::new();
        let mut latest = String::new();
        for tick in 0..=(seconds * 10 + 100) {
            let now = start + Duration::from_millis(tick * 100);
            if tick <= seconds * 10 {
                latest = format!("color {tick}");
            }
            if tracker.status(&latest, now) == UserinfoUpdateStatus::Sent {
                tracker.mark_sent(latest.clone(), now);
                sent.push((tick * 100, latest.clone()));
            }
        }
        sent
    }

    /// The server's rule (`SV_UpdateUserinfo_f`): a window opens at the first change
    /// it takes and lasts 6 s; a fourth change in it is ignored.
    fn server_takes(sent: &[(u64, String)]) -> Vec<String> {
        let (mut end, mut count, mut taken) = (0_u64, 0_u32, Vec::new());
        for (at, value) in sent {
            if *at < end {
                count += 1;
                if count >= INFO_CHANGE_COUNT {
                    continue;
                }
            } else {
                count = 0;
                end = at + 6_000;
            }
            taken.push(value.clone());
        }
        taken
    }

    /// Kushina's report: a long RGB slider drag ends with the server on the last value.
    #[test]
    fn a_long_slider_drag_reaches_the_server_with_its_last_value() {
        let start = Instant::now();
        let mut tracker = UserinfoUpdateTracker::new(String::new(), None, start);
        let sent = drag(&mut tracker, start + Duration::from_secs(2), 8);
        let taken = server_takes(&sent);
        assert_eq!(
            taken.len(),
            sent.len(),
            "the server ignored a change: {sent:?}"
        );
        assert_eq!(taken.last().map(String::as_str), Some("color 80"));
    }

    #[test]
    fn a_refused_change_goes_again_after_the_window() {
        let start = Instant::now();
        let mut tracker = UserinfoUpdateTracker::new(String::new(), None, start);
        let later = start + Duration::from_secs(2);
        tracker.mark_sent("a".into(), later);
        tracker.refuse();
        assert!(tracker.refused());
        let soon = later + Duration::from_secs(3);
        assert_eq!(tracker.status("a", soon), UserinfoUpdateStatus::RateLimited);
        let after = later + INFO_CHANGE_WINDOW;
        assert_eq!(tracker.status("a", after), UserinfoUpdateStatus::Sent);
        tracker.mark_sent("a".into(), after);
        assert!(!tracker.refused());
        assert_eq!(tracker.status("a", after), UserinfoUpdateStatus::Unchanged);
    }
}
