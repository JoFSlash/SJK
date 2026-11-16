//! Recipient metadata for outgoing whispers echoed into console scrollback.
//!
//! Protocol 26 echoes a `tell` as the sender's private chat line without naming
//! its recipient. Remember the destination at send time, pair the reliable
//! server echo by its normalized body, and retain the name beside the bounded
//! scrollback line. Presentation can then explain an old line without guessing
//! a player from its coloured text or from the current roster.

use std::collections::VecDeque;

const MAX_PENDING: usize = 64;
const MAX_LINES: usize = 256;

#[derive(Debug, Eq, PartialEq)]
struct Pending {
    slot: u16,
    name: String,
    body: String,
}

#[derive(Default)]
pub(super) struct State {
    pending: VecDeque<Pending>,
    lines: VecDeque<(u64, String)>,
}

impl State {
    /// Remember one final, escaped `tell <slot> <body>` command after it was sent.
    pub(super) fn note_sent(&mut self, command: &str, name: String) {
        let Some(pending) = sent_whisper(command, name) else {
            return;
        };
        if self.pending.len() == MAX_PENDING {
            self.pending.pop_front();
        }
        self.pending.push_back(pending);
    }

    /// Attach a private echo from our own slot to the sent whisper it confirms.
    pub(super) fn note_echo(&mut self, line: u64, body: &str) {
        let body = normalized(body);
        let Some(index) = self.pending.iter().position(|sent| sent.body == body) else {
            return;
        };
        // Reliable commands and their server echoes keep order. Missing earlier
        // echoes must not label a later line with a stale recipient.
        self.pending.drain(..index);
        let Some(sent) = self.pending.pop_front() else {
            return;
        };
        if self.lines.len() == MAX_LINES {
            self.lines.pop_front();
        }
        self.lines.push_back((
            line,
            format!("Whisper sent to {} (#{}).", sent.name, sent.slot),
        ));
    }

    pub(super) fn label(&self, line: u64) -> Option<&str> {
        self.lines
            .iter()
            .rev()
            .find_map(|(number, label)| (*number == line).then_some(label.as_str()))
    }
}

fn sent_whisper(command: &str, name: String) -> Option<Pending> {
    let tokens = sjk_shell::tokenize(command).ok()?;
    let [verb, slot, body @ ..] = tokens.as_slice() else {
        return None;
    };
    if !verb.eq_ignore_ascii_case("tell") || body.is_empty() {
        return None;
    }
    Some(Pending {
        slot: slot.parse().ok()?,
        name,
        body: normalized(&body.join(" ")),
    })
}

fn normalized(body: &str) -> String {
    sjk_client::chat_plain_text(&sjk_client::chat_unescape(body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_echo_keeps_the_recipient_on_its_console_line() {
        let mut state = State::default();
        state.note_sent("tell 7 hello ^1there", "Fox".to_owned());
        state.note_echo(42, "^6hello ^2there");
        assert_eq!(state.label(42), Some("Whisper sent to Fox (#7)."));
    }

    #[test]
    fn only_the_matching_private_echo_consumes_a_pending_whisper() {
        let mut state = State::default();
        state.note_sent("tell 3 \"first\"", "Sol".to_owned());
        state.note_echo(8, "another message");
        assert_eq!(state.label(8), None);
        state.note_echo(9, "first");
        assert_eq!(state.label(9), Some("Whisper sent to Sol (#3)."));
    }

    #[test]
    fn non_tell_commands_have_no_recipient_metadata() {
        let mut state = State::default();
        state.note_sent("say \"hello\"", "Fox".to_owned());
        state.note_echo(1, "hello");
        assert_eq!(state.label(1), None);
    }
}
