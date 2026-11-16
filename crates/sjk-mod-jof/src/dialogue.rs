//! EJK #301/#307/#342 server-authoritative dialogue presentation contract.
//! Quests and action effects remain entirely on the remote server. These are
//! existing reliable command operands, not additions to protocol-26 codecs.

/// Server command used by the documented dialogue extension.
/// Archived opt-out owned by the JoF mod.
pub const CVAR: &str = "jof.dialogue";

pub const COMMAND: &[u8] = b"jof_dialogue";
/// Matching client response command, with serial and signed choice operands.
pub const RESPONSE: &str = "dialogueresponse";
/// EJK's maximum responses per node.
pub const MAX_CHOICES: usize = 12;

/// One bounded, decoded dialogue command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// Replace any prior node, including its voice and pending response.
    Begin {
        serial: u32,
        speaker: String,
        text: String,
    },
    /// Supply one zero-based response label.
    Choice {
        serial: u32,
        index: usize,
        text: String,
    },
    /// Open the assembled node, optionally with installed voice audio.
    Show { serial: u32, sound: Option<String> },
    /// Close only the matching node, including a node still being assembled.
    Stop { serial: u32 },
}

fn text(bytes: &[u8], limit: usize) -> String {
    sjk_client::decode_legacy(&bytes[..bytes.len().min(limit - 1)]).replace("\\n", "\n")
}

/// Parse a tokenized reliable command; malformed operands never become responses
/// or shell scripts. Limits match `cg_dialogue.c` and `MAX_QPATH`.
pub fn parse(args: &[Vec<u8>]) -> Option<Event> {
    if args.first()?.as_slice() != COMMAND {
        return None;
    }
    let action = args.get(1)?;
    let serial = std::str::from_utf8(args.get(2)?).ok()?.parse().ok()?;
    if action.eq_ignore_ascii_case(b"begin") {
        Some(Event::Begin {
            serial,
            speaker: text(args.get(3)?, 64),
            text: text(args.get(4)?, 512),
        })
    } else if action.eq_ignore_ascii_case(b"choice") {
        let index = std::str::from_utf8(args.get(3)?)
            .ok()?
            .parse::<usize>()
            .ok()?;
        (index < MAX_CHOICES).then(|| Event::Choice {
            serial,
            index,
            text: text(args.get(4).map_or(&[], Vec::as_slice), 160),
        })
    } else if action.eq_ignore_ascii_case(b"show") {
        let sound = args
            .get(3)
            .filter(|p| !p.is_empty() && p.len() < 64)
            .and_then(|p| std::str::from_utf8(p).ok())
            .filter(|p| {
                !p.contains(['\\', ':'])
                    && !p.starts_with('/')
                    && !p.split('/').any(|c| c == ".." || c == ".")
                    && !p.chars().any(char::is_control)
            })
            .map(str::to_owned);
        Some(Event::Show { serial, sound })
    } else if action.eq_ignore_ascii_case(b"stop") {
        Some(Event::Stop { serial })
    } else {
        None
    }
}

/// Owned current node, bounded to twelve choices; no frame-time parsing or I/O.
#[derive(Default)]
pub struct Dialogue {
    /// Current assembly's serial; absent after closing/reset.
    pub serial: Option<u32>,
    /// Whether `show` made the current node interactive.
    pub active: bool,
    /// Server-provided speaker label.
    pub speaker: String,
    /// Server-provided body with decoded explicit newlines.
    pub text: String,
    /// Response labels in server order.
    pub choices: [String; MAX_CHOICES],
    /// Number of response slots, including sparse slots as EJK does.
    pub count: usize,
    /// Highlighted response, clamped at the ends (#307).
    pub selected: usize,
    /// Installed voice requested by `show` only.
    pub sound: Option<String>,
    /// Changes on begin, accepted show and close, for once-only voice lifecycle.
    pub revision: u64,
    submitted: Option<u64>,
}

impl Dialogue {
    /// Clear the node, including pending audio, without reusing its voice revision.
    pub fn reset(&mut self) {
        let revision = self.revision.wrapping_add(1);
        *self = Self {
            revision,
            ..Self::default()
        };
    }

    /// Apply a reliable command. Returns true when its serial/state was accepted.
    pub fn apply(&mut self, event: Event) -> bool {
        match event {
            Event::Begin {
                serial,
                speaker,
                text,
            } => {
                self.reset();
                self.serial = Some(serial);
                self.speaker = speaker;
                self.text = text;
            }
            Event::Choice {
                serial,
                index,
                text,
            } if self.serial == Some(serial) && index < MAX_CHOICES => {
                self.choices[index] = text;
                self.count = self.count.max(index + 1);
            }
            Event::Show { serial, sound }
                if self.serial == Some(serial) && self.count > 0 && !self.active =>
            {
                self.active = true;
                self.selected = 0;
                self.sound = sound;
                self.revision = self.revision.wrapping_add(1);
            }
            Event::Stop { serial } if self.serial == Some(serial) => self.reset(),
            _ => return false,
        }
        true
    }

    /// Move the selected row without wrapping (EJK #307).
    pub fn navigate(&mut self, direction: i8) {
        self.selected = if direction < 0 {
            self.selected.saturating_sub(1)
        } else {
            (self.selected + 1).min(self.count.saturating_sub(1))
        };
    }

    /// Format one response at most once per second until the server changes node.
    /// Cancellation uses the current serial too, then closes locally immediately.
    pub fn respond(&mut self, choice: i32, now_ms: u64) -> Option<String> {
        if !self.active || choice < -1 || choice >= self.count as i32 {
            return None;
        }
        if choice != -1
            && self
                .submitted
                .is_some_and(|at| now_ms.saturating_sub(at) < 1000)
        {
            return None;
        }
        let response = format!("{RESPONSE} {} {choice}", self.serial?);
        if choice == -1 {
            self.reset();
        } else {
            self.submitted = Some(now_ms);
        }
        Some(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn command(value: &str) -> Event {
        let args = sjk_shell::tokenize(&value.replace("\\", "\\\\"))
            .unwrap()
            .into_iter()
            .map(String::into_bytes)
            .collect::<Vec<_>>();
        parse(&args).unwrap()
    }
    #[test]
    fn stale_stop_cannot_close_pending_or_visible_node_and_show_is_once() {
        let mut d = Dialogue::default();
        d.apply(command(r#"jof_dialogue begin 8 "Guide" "First\nSecond""#));
        assert_eq!(d.text, "First\nSecond");
        assert!(!d.apply(command("jof_dialogue stop 7")));
        d.apply(command(r#"jof_dialogue choice 8 0 "Continue""#));
        assert!(d.apply(command(r#"jof_dialogue show 8 "sound/guide.wav""#)));
        let revision = d.revision;
        assert!(!d.apply(command("jof_dialogue show 8")));
        assert_eq!(d.revision, revision);
        assert!(!d.apply(command("jof_dialogue stop 7")));
        assert!(d.active);
        assert_eq!(d.respond(0, 0).as_deref(), Some("dialogueresponse 8 0"));
        assert_eq!(d.respond(0, 999), None);
        assert!(d.respond(0, 1000).is_some());
        assert_eq!(
            d.respond(-1, 1001).as_deref(),
            Some("dialogueresponse 8 -1")
        );
        assert!(!d.active && d.sound.is_none());
        assert_ne!(d.revision, revision);
    }
    #[test]
    fn rows_clamp_all_choices_remain_reachable_and_node_replacement_stops_voice() {
        let mut d = Dialogue::default();
        d.apply(command(r#"jof_dialogue begin 1 "" """#));
        for index in 0..12 {
            d.apply(command(&format!("jof_dialogue choice 1 {index} Answer")));
        }
        d.apply(command("jof_dialogue show 1"));
        for _ in 0..20 {
            d.navigate(1);
        }
        assert_eq!(d.selected, 11);
        assert!(d.respond(11, 0).is_some());
        for _ in 0..20 {
            d.navigate(-1);
        }
        assert_eq!(d.selected, 0);
        d.apply(command(r#"jof_dialogue begin 2 "Next" "Other""#));
        assert!(!d.active && d.sound.is_none());
        assert_eq!(d.respond(-1, 1000), None);
        assert!(!d.apply(command("jof_dialogue stop 1")));
        assert!(d.apply(command("jof_dialogue stop 2")));
        assert!(d.serial.is_none());
    }
    #[test]
    fn limits_and_sound_paths_are_bounded_and_older_silent_show_works() {
        let args = vec![
            COMMAND.to_vec(),
            b"begin".to_vec(),
            b"4".to_vec(),
            vec![b'x'; 200],
            vec![b'y'; 2000],
        ];
        let Event::Begin { speaker, text, .. } = parse(&args).unwrap() else {
            panic!()
        };
        assert_eq!((speaker.len(), text.len()), (63, 511));
        assert!(
            parse(&[
                COMMAND.to_vec(),
                b"choice".to_vec(),
                b"4".to_vec(),
                b"12".to_vec()
            ])
            .is_none()
        );
        for path in [
            "../voice.wav",
            "/voice.wav",
            "C:/voice.wav",
            "sound/../voice.wav",
        ] {
            assert!(matches!(
                command(&format!("jof_dialogue show 4 {path}")),
                Event::Show { sound: None, .. }
            ));
        }
        assert!(matches!(
            command("jof_dialogue show 4"),
            Event::Show { sound: None, .. }
        ));
    }
}
