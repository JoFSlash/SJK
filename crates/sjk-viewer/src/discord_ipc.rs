//! Discord Rich Presence over the Discord desktop app's local IPC pipe.
//!
//! The desktop app listens on `\\.\pipe\discord-ipc-N` on Windows and on a
//! `discord-ipc-N` socket in the runtime or temporary directory elsewhere. Each
//! message is a frame: opcode and length as little-endian `u32`, then JSON.
//! Nothing leaves the computer: Discord itself shows the activity. A background
//! thread owns the pipe, so a missing or slow Discord never stalls a frame; it
//! sends only the latest activity, at most once per [`SEND_GAP`], and retries a
//! closed Discord every [`RETRY_GAP`].

use std::io::{Read, Write};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

/// Discord allows five activity updates per 20 seconds; one every 5 s stays under it.
const SEND_GAP: Duration = Duration::from_secs(5);
/// How long to wait before looking for a Discord that was not running.
const RETRY_GAP: Duration = Duration::from_secs(20);
/// Longest text Discord accepts in an activity field.
const MAX_TEXT: usize = 128;

const OP_HANDSHAKE: u32 = 0;
const OP_FRAME: u32 = 1;
const OP_CLOSE: u32 = 2;
const OP_PING: u32 = 3;
const OP_PONG: u32 = 4;

/// A link button under the activity (Discord shows at most two).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivityButton {
    /// Button text.
    pub label: String,
    /// The https address it opens.
    pub url: String,
}

/// What Discord shows on the player's profile.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Activity {
    /// First line under the game name.
    pub details: String,
    /// Second line.
    pub state: String,
    /// Big picture: an uploaded asset key or an https address.
    pub large_image: String,
    /// Tooltip of the big picture.
    pub large_text: String,
    /// Small round picture over the big one's corner.
    pub small_image: String,
    /// Tooltip of the small picture.
    pub small_text: String,
    /// Unix time, in seconds, the elapsed timer counts from.
    pub start: Option<u64>,
    /// Party size as (current, maximum), shown as "(3 of 32)" after the state.
    pub party: Option<(u32, u32)>,
    /// Link buttons; Discord keeps the first two.
    pub buttons: Vec<ActivityButton>,
}

impl Activity {
    /// The activity as Discord's `SET_ACTIVITY` JSON.
    pub fn to_json(&self) -> Value {
        let mut activity = serde_json::Map::new();
        put_text(&mut activity, "details", &self.details);
        put_text(&mut activity, "state", &self.state);
        let mut assets = serde_json::Map::new();
        put_text(&mut assets, "large_image", &self.large_image);
        put_text(&mut assets, "large_text", &self.large_text);
        put_text(&mut assets, "small_image", &self.small_image);
        put_text(&mut assets, "small_text", &self.small_text);
        if !assets.is_empty() {
            activity.insert("assets".into(), Value::Object(assets));
        }
        if let Some(start) = self.start {
            activity.insert("timestamps".into(), json!({ "start": start }));
        }
        // A party size needs a state to sit next to and a non-zero maximum.
        if let Some((size, max)) = self.party
            && max > 0
            && activity.contains_key("state")
        {
            activity.insert(
                "party".into(),
                json!({ "id": "sjk-server", "size": [size.min(max), max] }),
            );
        }
        let buttons: Vec<Value> = self
            .buttons
            .iter()
            .filter(|button| button.url.starts_with("https://") && !button.label.is_empty())
            .take(2)
            .map(|button| json!({ "label": clip(&button.label, 32), "url": button.url }))
            .collect();
        if !buttons.is_empty() {
            activity.insert("buttons".into(), Value::Array(buttons));
        }
        Value::Object(activity)
    }
}

/// Inserts `text` clipped to Discord's limit, or nothing when it is shorter than
/// the two characters Discord requires.
fn put_text(map: &mut serde_json::Map<String, Value>, key: &str, text: &str) {
    let text = clip(text.trim(), MAX_TEXT);
    if text.chars().count() >= 2 {
        map.insert(key.into(), Value::String(text));
    }
}

fn clip(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut clipped: String = text.chars().take(max - 1).collect();
    clipped.push('…');
    clipped
}

/// Owner of the presence thread; dropping it clears the activity and stops the thread.
pub struct DiscordIpc {
    sender: Sender<Option<Activity>>,
    last: Option<Option<Activity>>,
}

impl DiscordIpc {
    /// Starts the thread for the Discord application `client_id`.
    pub fn start(client_id: &'static str) -> Option<Self> {
        let (sender, receiver) = mpsc::channel();
        std::thread::Builder::new()
            .name("discord-presence".into())
            .spawn(move || run(client_id, receiver))
            .ok()?;
        Some(Self { sender, last: None })
    }

    /// Shows `activity`, or clears it with `None`. Repeating the same one is free.
    pub fn set(&mut self, activity: Option<Activity>) {
        if self.last.as_ref() == Some(&activity) {
            return;
        }
        let _ = self.sender.send(activity.clone());
        self.last = Some(activity);
    }
}

fn run(client_id: &str, receiver: Receiver<Option<Activity>>) {
    // Every Discord app that answered: Discord, PTB and Canary can run side by side,
    // each signed in to its own account, and each shows the status on its own.
    let mut pipes: Vec<Pipe> = Vec::new();
    let mut wanted: Option<Activity> = None;
    let mut shown: Option<Option<Activity>> = None;
    let mut last_send: Option<Instant> = None;
    let mut last_attempt: Option<Instant> = None;
    let mut nonce = 0u64;
    // Log a missing Discord once, not every retry, and the first status it takes.
    let mut told_missing = false;
    let mut told_shown = false;
    loop {
        match receiver.recv_timeout(Duration::from_secs(1)) {
            Ok(activity) => wanted = activity,
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                for pipe in &mut pipes {
                    nonce += 1;
                    let _ = pipe.set_activity(None, nonce);
                }
                return;
            }
        }
        while let Ok(activity) = receiver.try_recv() {
            wanted = activity;
        }
        if shown.as_ref() == Some(&wanted) {
            continue;
        }
        if last_send.is_some_and(|at| at.elapsed() < SEND_GAP) {
            continue;
        }
        if pipes.is_empty() {
            if last_attempt.is_some_and(|at| at.elapsed() < RETRY_GAP) {
                continue;
            }
            last_attempt = Some(Instant::now());
            pipes = Pipe::connect_all(client_id);
            if pipes.is_empty() {
                if !told_missing {
                    eprintln!("Discord status: no Discord app answered; looking again every 20 s");
                    told_missing = true;
                }
                continue;
            }
            let apps: Vec<&str> = pipes.iter().map(|pipe| pipe.app).collect();
            eprintln!("Discord status: connected to {}", apps.join(", "));
            told_missing = false;
            told_shown = false;
        }
        last_send = Some(Instant::now());
        pipes.retain_mut(|pipe| {
            nonce += 1;
            match pipe.set_activity(wanted.as_ref(), nonce) {
                Ok(()) => true,
                Err(error) => {
                    eprintln!("Discord status: lost {}: {error}", pipe.app);
                    false
                }
            }
        });
        if pipes.is_empty() {
            shown = None;
            last_attempt = Some(Instant::now());
            continue;
        }
        if !told_shown && let Some(activity) = wanted.as_ref() {
            eprintln!(
                "Discord status: showing \"{}\" / \"{}\"",
                activity.details, activity.state
            );
            told_shown = true;
        }
        shown = Some(wanted.clone());
    }
}

/// One open connection to a Discord desktop app.
struct Pipe {
    stream: Box<dyn Stream>,
    /// Which app it is (Discord, Discord PTB, Discord Canary), for the log.
    app: &'static str,
}

trait Stream: Read + Write + Send {}
impl<T: Read + Write + Send> Stream for T {}

impl Pipe {
    /// Opens every Discord pipe that answers the handshake.
    fn connect_all(client_id: &str) -> Vec<Self> {
        let handshake = json!({ "v": 1, "client_id": client_id });
        candidate_streams()
            .filter_map(|stream| {
                let mut pipe = Self {
                    stream,
                    app: "Discord",
                };
                pipe.write(OP_HANDSHAKE, &handshake).ok()?;
                let ready = pipe.read_reply().ok()?;
                pipe.app = app_name(&ready);
                Some(pipe)
            })
            .collect()
    }

    fn set_activity(&mut self, activity: Option<&Activity>, nonce: u64) -> std::io::Result<()> {
        let message = json!({
            "cmd": "SET_ACTIVITY",
            "args": {
                "pid": std::process::id(),
                "activity": activity.map(Activity::to_json),
            },
            "nonce": nonce.to_string(),
        });
        self.write(OP_FRAME, &message)?;
        let reply = self.read_reply()?;
        #[cfg(test)]
        println!("discord reply: {reply}");
        if reply.get("evt").and_then(Value::as_str) == Some("ERROR") {
            eprintln!("Discord status refused: {}", reply["data"]);
        }
        Ok(())
    }

    fn write(&mut self, opcode: u32, payload: &Value) -> std::io::Result<()> {
        self.stream.write_all(&encode_frame(opcode, payload))?;
        self.stream.flush()
    }

    /// Reads frames until a reply arrives, answering pings on the way.
    fn read_reply(&mut self) -> std::io::Result<Value> {
        loop {
            let mut header = [0u8; 8];
            self.stream.read_exact(&mut header)?;
            let opcode = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
            let length = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;
            if length > 1 << 20 {
                return Err(std::io::Error::other("oversized discord frame"));
            }
            let mut body = vec![0u8; length];
            self.stream.read_exact(&mut body)?;
            let value: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
            match opcode {
                OP_FRAME => return Ok(value),
                OP_PING => self.write(OP_PONG, &value)?,
                OP_CLOSE => return Err(std::io::Error::other(format!("discord closed: {value}"))),
                _ => {}
            }
        }
    }
}

/// The app behind a pipe, from the API address its `READY` reply names.
fn app_name(ready: &Value) -> &'static str {
    let endpoint = ready["data"]["config"]["api_endpoint"]
        .as_str()
        .unwrap_or("");
    if endpoint.contains("ptb.") {
        "Discord PTB"
    } else if endpoint.contains("canary.") {
        "Discord Canary"
    } else {
        "Discord"
    }
}

fn encode_frame(opcode: u32, payload: &Value) -> Vec<u8> {
    let body = payload.to_string();
    let mut frame = Vec::with_capacity(8 + body.len());
    frame.extend_from_slice(&opcode.to_le_bytes());
    frame.extend_from_slice(&(body.len() as u32).to_le_bytes());
    frame.extend_from_slice(body.as_bytes());
    frame
}

#[cfg(windows)]
fn candidate_streams() -> impl Iterator<Item = Box<dyn Stream>> {
    (0..10).filter_map(|index| {
        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(format!(r"\\.\pipe\discord-ipc-{index}"))
            .ok()
            .map(|file| Box::new(file) as Box<dyn Stream>)
    })
}

#[cfg(unix)]
fn candidate_streams() -> impl Iterator<Item = Box<dyn Stream>> {
    let base = ["XDG_RUNTIME_DIR", "TMPDIR", "TMP", "TEMP"]
        .iter()
        .find_map(|name| std::env::var_os(name))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| "/tmp".into());
    // The plain install, then the Flatpak and Snap packages' own folders.
    let folders = ["", "app/com.discordapp.Discord", "snap.discord"];
    folders.into_iter().flat_map(move |folder| {
        let dir = base.join(folder);
        (0..10).filter_map(move |index| {
            std::os::unix::net::UnixStream::connect(dir.join(format!("discord-ipc-{index}")))
                .ok()
                .map(|stream| Box::new(stream) as Box<dyn Stream>)
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_are_opcode_length_then_json() {
        let frame = encode_frame(OP_FRAME, &json!({ "a": 1 }));
        assert_eq!(&frame[..4], &1u32.to_le_bytes());
        assert_eq!(&frame[4..8], &7u32.to_le_bytes());
        assert_eq!(&frame[8..], br#"{"a":1}"#);
    }

    #[test]
    fn activity_json_drops_short_text_and_keeps_two_buttons() {
        let button = |label: &str| ActivityButton {
            label: label.into(),
            url: "https://example.org".into(),
        };
        let activity = Activity {
            details: "In menus".into(),
            state: "x".into(),
            large_image: "logo".into(),
            start: Some(10),
            party: Some((3, 32)),
            buttons: vec![button("One"), button("Two"), button("Three")],
            ..Activity::default()
        };
        let value = activity.to_json();
        assert_eq!(value["details"], "In menus");
        assert!(value.get("state").is_none());
        // No state, so no party size either.
        assert!(value.get("party").is_none());
        assert_eq!(value["assets"]["large_image"], "logo");
        assert_eq!(value["timestamps"]["start"], 10);
        assert_eq!(value["buttons"].as_array().map(Vec::len), Some(2));
    }

    /// Against the Discord app on this PC: `SJK_DISCORD_APP=<application id> cargo
    /// test -p sjk-viewer --bins discord_app_takes_an_activity -- --ignored`.
    #[test]
    #[ignore = "needs the Discord desktop app running"]
    fn discord_app_takes_an_activity() {
        let id = std::env::var("SJK_DISCORD_APP").unwrap_or_else(|_| "0".into());
        let mut pipes = Pipe::connect_all(&id);
        assert!(!pipes.is_empty(), "no Discord pipe answered the handshake");
        let activity = Activity {
            details: "Testing the pipe".into(),
            state: "ffa3 · FFA".into(),
            party: Some((3, 32)),
            large_image: "https://sol-vulpes.github.io/SJK/assets/sjk-logo-512.png".into(),
            large_text: "Sol JK".into(),
            start: Some(1_760_000_000),
            buttons: vec![ActivityButton {
                label: "Get SJK".into(),
                url: "https://sol-vulpes.github.io/SJK/".into(),
            }],
            ..Activity::default()
        };
        for pipe in &mut pipes {
            println!("{}:", pipe.app);
            pipe.set_activity(Some(&activity), 1)
                .expect("activity refused");
        }
        std::thread::sleep(Duration::from_secs(10));
    }

    #[test]
    fn the_ready_reply_names_the_app() {
        let ready = |endpoint: &str| json!({ "data": { "config": { "api_endpoint": endpoint } } });
        assert_eq!(app_name(&ready("//discord.com/api")), "Discord");
        assert_eq!(app_name(&ready("//ptb.discord.com/api")), "Discord PTB");
        assert_eq!(app_name(&Value::Null), "Discord");
    }

    #[test]
    fn long_text_is_clipped_to_discords_limit() {
        let activity = Activity {
            details: "a".repeat(300),
            state: "On a server".into(),
            party: Some((40, 32)),
            ..Activity::default()
        };
        let value = activity.to_json();
        assert_eq!(
            value["details"].as_str().map(|s| s.chars().count()),
            Some(MAX_TEXT)
        );
        assert_eq!(value["party"]["size"], json!([32, 32]));
    }
}
