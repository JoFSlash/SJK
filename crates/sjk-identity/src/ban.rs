//! Chat bans (`PROTOCOL.md` in Sol-Vulpes/SJK-hub, "Chat bans"): a verified or staff
//! player bans a player the hub does not vouch for from the SJK chat for 24 hours,
//! with no reason asked and no wait between bans. The hub bans the person's key and
//! the address it last saw them use, so a new key from there cannot write either; a
//! verified key from that address still can. Staff lift bans
//! ([`crate::StaffRequest::ChatUnban`]).
//!
//! What the player reads of a ban is worded here: the time a ban ends, in local time
//! as `dd/mm HH:MM` (SJK writes dates day first and times on the 24-hour clock), and
//! why the hub did not take one.

use crate::HubError;
use chrono::{DateTime, Datelike, Local, TimeZone, Timelike};
use std::borrow::Cow;

/// How long a ban lasts, in seconds (the hub's rule; shown, never sent).
pub const BAN_SECS: i64 = 24 * 60 * 60;

/// The hub's answer to a ban: when it ends, and whether the target's address was
/// banned too (the hub saw them lately).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ChatBan {
    /// Unix seconds when the ban ends.
    pub until: i64,
    /// The address the hub last saw them use is banned too.
    pub address: bool,
}

/// What became of the last ban the player asked for.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BanOutcome {
    /// Counts bans asked for, so a reader can tell a new outcome from the last one.
    pub serial: u64,
    /// The key the ban was for.
    pub key_id: String,
    /// The hub's answer, or why there is none.
    pub result: Result<ChatBan, HubError>,
}

impl BanOutcome {
    /// The line the player reads about it.
    pub fn line(&self) -> String {
        match &self.result {
            Ok(ban) => format!("Banned from SJK chat until {}", local_day_time(ban.until)),
            Err(error) => refusal(error).into_owned(),
        }
    }
}

/// `unix` in local time as `dd/mm HH:MM`, or `?` for a time chrono cannot place.
pub fn local_day_time(unix: i64) -> String {
    DateTime::from_timestamp(unix, 0)
        .map(|time| day_time(&time.with_timezone(&Local)))
        .unwrap_or_else(|| "?".to_owned())
}

/// `time` as `dd/mm HH:MM`.
fn day_time<T: TimeZone>(time: &DateTime<T>) -> String {
    format!(
        "{:02}/{:02} {:02}:{:02}",
        time.day(),
        time.month(),
        time.hour(),
        time.minute()
    )
}

/// What the player reads when the hub refuses their message because they are banned
/// until `until` (unix seconds).
pub fn banned_message(until: i64) -> String {
    format!(
        "A verified player banned you from SJK chat until {}",
        local_day_time(until)
    )
}

/// Why the hub did not take a ban, in the player's words.
pub fn refusal(error: &HubError) -> Cow<'_, str> {
    let why = match error {
        HubError::Rejected { code, .. } => match code.as_str() {
            "not_verified" => "Only verified players can ban from SJK chat",
            "target_verified" => "Verified players and staff cannot be banned",
            "target_trusted" => "Only staff can ban a trusted player",
            "not_found" => "The hub does not know this player",
            "bad_target" => "You cannot ban yourself",
            "not_registered" => "Your identity is not registered at the SJK hub yet",
            "unauthorized" | "clock" => "The hub did not take the request; try again",
            code if code.ends_with("quota") || code == "rate_limited" => {
                "Too many requests; wait a moment"
            }
            _ => "The hub did not take the ban",
        },
        // A hub from before bans has no such route.
        HubError::Protocol(why)
            if why.starts_with("status 404") || why.starts_with("status 405") =>
        {
            "This SJK hub cannot ban from the chat yet"
        }
        HubError::Network(_) => "Cannot reach the SJK hub",
        HubError::Protocol(_) => "The hub did not take the ban",
        // Never sent: why, as the service words it (identity off, no hub, not yet
        // registered, with what to do).
        HubError::NotConnected(why) => return Cow::Borrowed(why),
    };
    Cow::Borrowed(why)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::FixedOffset;

    fn rejected(code: &str) -> HubError {
        HubError::Rejected {
            status: 403,
            code: code.to_owned(),
            message: String::new(),
        }
    }

    #[test]
    fn times_are_day_first_on_the_24_hour_clock() {
        // 17/10/2026 12:32 UTC is 14:32 in Paris's summer time.
        let paris = FixedOffset::east_opt(2 * 3600).unwrap();
        let time = DateTime::from_timestamp(1_792_240_320, 0)
            .unwrap()
            .with_timezone(&paris);
        assert_eq!(day_time(&time), "17/10 14:32");
        let early = DateTime::from_timestamp(1_791_688_860, 0)
            .unwrap()
            .with_timezone(&FixedOffset::east_opt(0).unwrap());
        assert_eq!(day_time(&early), "11/10 03:21");
        assert!(
            banned_message(1_792_240_320)
                .starts_with("A verified player banned you from SJK chat until ")
        );
    }

    #[test]
    fn refusals_are_worded_for_players() {
        assert_eq!(
            refusal(&rejected("target_verified")),
            "Verified players and staff cannot be banned"
        );
        assert_eq!(
            refusal(&rejected("not_verified")),
            "Only verified players can ban from SJK chat"
        );
        assert_eq!(refusal(&rejected("bad_target")), "You cannot ban yourself");
        assert_eq!(
            refusal(&rejected("not_found")),
            "The hub does not know this player"
        );
        assert_eq!(
            refusal(&rejected("chat_quota")),
            "Too many requests; wait a moment"
        );
        assert_eq!(
            refusal(&HubError::Protocol("status 404, not JSON".to_owned())),
            "This SJK hub cannot ban from the chat yet"
        );
        // Not sent at all: the service's reason, with what to do.
        let off = "the SJK identity is off: turn it on with cl_identity 1";
        assert_eq!(refusal(&HubError::NotConnected(off.to_owned())), off);
        let banned = BanOutcome {
            serial: 1,
            key_id: "0123456789abcdef".to_owned(),
            result: Ok(ChatBan {
                until: 1_792_240_320,
                address: true,
            }),
        };
        assert!(banned.line().starts_with("Banned from SJK chat until "));
    }
}
