//! The toy relay (`PROTOCOL.md`, "Toy relay"): toy events that must reach one player
//! at once, many times a second (the Hax toy's hacker steering the hacked player),
//! which the feed's toy events cannot carry. Two threads of their own: one sends what
//! the viewer hands it as soon as it comes, the other keeps a long poll open at the hub
//! for the events meant for the player, while the viewer says it listens.
//!
//! The same two threads carry encrypted whispers (`crate::whisper`): the sender seals
//! and posts the ones the viewer hands it, first, and says what became of each; the
//! reader asks for them while the viewer takes them, and opens them.
//!
//! Both read the hub and the game server from [`FeedShared`] (set only while the
//! identity is on, registered and on a server). The hub needs a live claim there, so
//! nothing goes or comes before it is in.

use crate::feed::{FeedShared, ForServer, QueuedToyEvent, lock};
use crate::hub::Hub;
use crate::keys::Identity;
use crate::service::HubFactory;
use crate::whisper;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

/// How long the hub may hold a read when nothing comes, in seconds.
pub const WAIT: u64 = 20;
/// The reading client's HTTP timeout: the hub's wait and room for a slow answer.
pub const TIMEOUT: Duration = Duration::from_secs(WAIT + 10);
/// The sending client's HTTP timeout: a later event is out of date anyway.
pub const SEND_TIMEOUT: Duration = Duration::from_secs(3);
/// How often an idle thread looks at what it is told.
const IDLE: Duration = Duration::from_millis(100);
/// First wait after a failed read, doubled up to [`RETRY_MAX`]: under the 3 seconds the
/// hub keeps an event, so one sent while the reads fail (the claim not in yet, after a
/// join or a map change) is still read once they work again.
const RETRY_MIN: Duration = Duration::from_millis(500);
const RETRY_MAX: Duration = Duration::from_secs(2);
/// Events waiting for the viewer, the newest kept.
const INBOX: usize = 256;
/// Events waiting to be sent, the newest kept.
const OUTBOX: usize = 64;
/// Whispers waiting to be sent, read or told about, the newest kept.
const WHISPERS: usize = 32;

/// The clients that relay: one for sends ([`SEND_TIMEOUT`]), one for the long poll
/// ([`TIMEOUT`]).
pub struct RelayHubs {
    /// Builds the sending client.
    pub send: HubFactory,
    /// Builds the reading client.
    pub read: HubFactory,
}

/// An event to relay.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Outgoing {
    pub(crate) toy: String,
    pub(crate) event: String,
    pub(crate) target: u8,
    /// A later event of the same toy and target that also replaces takes its place
    /// while it waits (a control frame: only the newest matters).
    pub(crate) replace: bool,
}

/// What the viewer and the relay threads share.
#[derive(Default)]
pub(crate) struct RelayState {
    outbox: VecDeque<Outgoing>,
    /// The viewer wants the events meant for the player.
    listen: bool,
    inbox: VecDeque<QueuedToyEvent>,
    /// The viewer takes encrypted whispers.
    whispers: bool,
    whisper_out: VecDeque<whisper::Outgoing>,
    whisper_sent: VecDeque<whisper::Sent>,
    whisper_in: VecDeque<ForServer<whisper::Received>>,
}

impl RelayState {
    /// Queue `outgoing`, in place of a waiting one it replaces.
    pub(crate) fn push(&mut self, outgoing: Outgoing) {
        if outgoing.replace
            && let Some(waiting) = self.outbox.iter_mut().find(|waiting| {
                waiting.replace && waiting.toy == outgoing.toy && waiting.target == outgoing.target
            })
        {
            *waiting = outgoing;
            return;
        }
        if self.outbox.len() >= OUTBOX {
            self.outbox.pop_front();
        }
        self.outbox.push_back(outgoing);
    }

    pub(crate) fn set_listen(&mut self, listen: bool) {
        self.listen = listen;
    }

    pub(crate) fn inbox(&mut self) -> &mut VecDeque<QueuedToyEvent> {
        &mut self.inbox
    }

    pub(crate) fn set_whispers(&mut self, whispers: bool) {
        self.whispers = whispers;
    }

    /// Queue a whisper to seal and send; the oldest waiting gives way when full, and is
    /// told as not sent.
    pub(crate) fn push_whisper(&mut self, outgoing: whisper::Outgoing) {
        if self.whisper_out.len() >= WHISPERS
            && let Some(dropped) = self.whisper_out.pop_front()
        {
            self.tell(dropped.serial, Err("too many whispers waited".to_owned()));
        }
        self.whisper_out.push_back(outgoing);
    }

    fn tell(&mut self, serial: u64, result: Result<(), String>) {
        if self.whisper_sent.len() >= WHISPERS {
            self.whisper_sent.pop_front();
        }
        self.whisper_sent
            .push_back(whisper::Sent { serial, result });
    }

    /// What became of the whispers sent since the last call.
    pub(crate) fn take_sent(&mut self) -> Vec<whisper::Sent> {
        self.whisper_sent.drain(..).collect()
    }

    /// The whispers read for `server` since the last call (others are dropped).
    pub(crate) fn take_received(
        &mut self,
        server: Option<std::net::SocketAddr>,
    ) -> Vec<whisper::Received> {
        let queue = std::sync::Mutex::new(std::mem::take(&mut self.whisper_in));
        crate::feed::take_for_server(&queue, server)
    }
}

/// The relay's shared state and the bell that wakes the sender.
pub(crate) type Shared = Arc<(Mutex<RelayState>, Condvar)>;

/// The hub client a thread keeps, rebuilt when the hub's address changes.
struct Client {
    make_hub: HubFactory,
    hub: Option<Box<dyn Hub>>,
    url: Option<String>,
}

impl Client {
    /// The client for `url`, and whether it is new.
    fn for_url(&mut self, url: Option<&str>) -> (Option<&mut Box<dyn Hub>>, bool) {
        let changed = url != self.url.as_deref();
        if changed {
            self.hub = url.and_then(|url| (self.make_hub)(url).ok());
            self.url = url.map(str::to_owned);
        }
        (self.hub.as_mut(), changed)
    }
}

/// Send what is queued, oldest first, until `stop`.
pub(crate) fn run_sender(
    identity: Identity,
    make_hub: HubFactory,
    feed: Arc<Mutex<FeedShared>>,
    shared: Shared,
    stop: &AtomicBool,
) {
    let mut client = Client {
        make_hub,
        hub: None,
        url: None,
    };
    let (state, bell) = &*shared;
    while !stop.load(Ordering::Relaxed) {
        // A whisper goes before the toys' frames.
        let (whisper, next) = {
            let mut state = lock(state);
            if state.outbox.is_empty() && state.whisper_out.is_empty() {
                state = bell
                    .wait_timeout(state, IDLE)
                    .map_or_else(|poisoned| poisoned.into_inner().0, |(state, _)| state);
            }
            match state.whisper_out.pop_front() {
                Some(whisper) => (Some(whisper), None),
                None => (None, state.outbox.pop_front()),
            }
        };
        if let Some(outgoing) = whisper {
            let place = lock(&feed).clone();
            let serial = outgoing.serial;
            let result = match client.for_url(place.url.as_deref()) {
                (Some(hub), _) => send_whisper(&identity, hub.as_mut(), &place, outgoing),
                (None, _) => Err("the SJK hub is off or not set".to_owned()),
            };
            lock(state).tell(serial, result);
            continue;
        }
        let Some(outgoing) = next else {
            continue;
        };
        let place = lock(&feed).clone();
        let (Some(hub), _) = client.for_url(place.url.as_deref()) else {
            continue;
        };
        if let Some(server) = place.server.as_deref() {
            // Refused or lost: the next one goes all the same, and the toys time out.
            let _ = hub.relay_send(
                &identity,
                server,
                &outgoing.toy,
                &outgoing.event,
                outgoing.target,
            );
        }
    }
}

/// Seal `outgoing` and hand it to `hub`, from the place the feed reads: why not, in a
/// player's words.
fn send_whisper(
    identity: &Identity,
    hub: &mut dyn Hub,
    place: &FeedShared,
    outgoing: whisper::Outgoing,
) -> Result<(), String> {
    if place.server.as_deref() != Some(outgoing.server.as_str()) {
        return Err("you are no longer on that server with the SJK hub".to_owned());
    }
    let to = whisper::parse_key(&outgoing.to).ok_or("the player's key is not a key")?;
    let time = chrono::Utc::now().timestamp();
    let boxed = whisper::seal(identity, &to, &outgoing.server, time, &outgoing.text).map_err(
        |failure| match failure {
            whisper::SealError::Length => "the message is too long to encrypt".to_owned(),
            whisper::SealError::BadKey => "the player's key is not a key".to_owned(),
            whisper::SealError::NoRandom => "the system gave no randomness".to_owned(),
        },
    )?;
    hub.whisper_send(
        identity,
        &outgoing.server,
        outgoing.target,
        &outgoing.to,
        &boxed,
    )
    .map(|_| ())
    .map_err(|failure| whisper::failure_words(&failure))
}

/// The reading thread's logic, advanced by [`Reader::step`] with an explicit clock.
pub(crate) struct Reader {
    identity: Identity,
    client: Client,
    feed: Arc<Mutex<FeedShared>>,
    shared: Shared,
    server: Option<String>,
    /// Whether the reading asks for whispers.
    whispers: bool,
    /// The id to read after; 0 starts afresh.
    after: u64,
    due: Instant,
    backoff: Duration,
}

impl Reader {
    pub(crate) fn new(
        identity: Identity,
        make_hub: HubFactory,
        feed: Arc<Mutex<FeedShared>>,
        shared: Shared,
        now: Instant,
    ) -> Self {
        Self {
            identity,
            client: Client {
                make_hub,
                hub: None,
                url: None,
            },
            feed,
            shared,
            server: None,
            whispers: false,
            after: 0,
            due: now,
            backoff: RETRY_MIN,
        }
    }

    /// Read if the viewer listens and it is time to; how long to wait before the next
    /// step. A read blocks for as long as the hub holds it (at most [`WAIT`] seconds).
    pub(crate) fn step(&mut self, now: Instant) -> Duration {
        let place = lock(&self.feed).clone();
        let (listen, whispers) = {
            let state = lock(&self.shared.0);
            (state.listen || state.whispers, state.whispers)
        };
        let (hub, new_hub) = self.client.for_url(place.url.as_deref());
        // Asking for whispers or not is another reading: it starts afresh.
        if new_hub || place.server != self.server || !listen || whispers != self.whispers {
            self.whispers = whispers;
            self.server = place.server;
            self.after = 0;
            self.due = now;
            self.backoff = RETRY_MIN;
        }
        let (Some(hub), Some(server), true) = (hub, self.server.as_deref(), listen) else {
            return IDLE;
        };
        if now < self.due {
            return (self.due - now).min(IDLE);
        }
        // The first read only learns where the relay is: nothing older is wanted.
        let wait = if self.after == 0 { 0 } else { WAIT };
        match hub.relay_read(&self.identity, server, self.after, wait, whispers) {
            Ok(batch) => {
                let read_for = server.parse().ok();
                let now = chrono::Utc::now().timestamp();
                let opened: Vec<_> = batch
                    .whispers
                    .into_iter()
                    .map(|sealed| whisper::Received {
                        text: whisper::parse_key(&sealed.from)
                            .ok_or(whisper::OpenError::Unreadable)
                            .and_then(|from| {
                                whisper::open_signed(
                                    &self.identity,
                                    &from,
                                    server,
                                    &sealed.boxed,
                                    now,
                                )
                            }),
                        slot: sealed.slot,
                        claimed_name: sealed.claimed_name,
                        key_id: sealed.key_id,
                        from: sealed.from,
                    })
                    .collect();
                let mut state = lock(&self.shared.0);
                for event in batch.events {
                    if state.inbox.len() >= INBOX {
                        state.inbox.pop_front();
                    }
                    state.inbox.push_back(QueuedToyEvent::new(read_for, event));
                }
                for received in opened {
                    if state.whisper_in.len() >= WHISPERS {
                        state.whisper_in.pop_front();
                    }
                    state
                        .whisper_in
                        .push_back(ForServer::new(read_for, received));
                }
                self.after = batch.next;
                self.backoff = RETRY_MIN;
                Duration::ZERO
            }
            // No claim yet (just joined), an old hub, or the network.
            Err(_) => {
                self.due = now + self.backoff;
                self.backoff = (self.backoff * 2).min(RETRY_MAX);
                self.backoff.min(IDLE)
            }
        }
    }
}

/// Read until `stop`.
pub(crate) fn run_reader(mut reader: Reader, stop: &AtomicBool) {
    while !stop.load(Ordering::Relaxed) {
        let wait = reader.step(Instant::now());
        if !wait.is_zero() {
            std::thread::sleep(wait);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hub::HubError;
    use crate::wire::{Profile, RelayBatch, SealedWhisper, ToyEvent};

    fn out(event: &str, target: u8, replace: bool) -> Outgoing {
        Outgoing {
            toy: "hax".to_owned(),
            event: event.to_owned(),
            target,
            replace,
        }
    }

    #[test]
    fn a_waiting_control_frame_is_replaced_by_the_next_one() {
        let mut state = RelayState::default();
        state.push(out("hack", 4, false));
        state.push(out("in_1", 4, true));
        state.push(out("in_1", 5, true));
        state.push(out("in_2", 4, true));
        state.push(out("stop", 4, false));
        let events: Vec<_> = state
            .outbox
            .iter()
            .map(|out| (out.event.as_str(), out.target))
            .collect();
        assert_eq!(events, [("hack", 4), ("in_2", 4), ("in_1", 5), ("stop", 4)]);
    }

    /// A hub whose relay reads are scripted, recording what was asked.
    #[derive(Clone, Default)]
    struct Scripted {
        answers: Arc<Mutex<VecDeque<Result<RelayBatch, HubError>>>>,
        asked: Arc<Mutex<Vec<(String, u64, u64)>>>,
        /// Whether each read asked for whispers.
        asked_whispers: Arc<Mutex<Vec<bool>>>,
        /// The whispers sent: target, key and box; refused with `refuse` when set.
        whispered: Arc<Mutex<Vec<(u8, String, String)>>>,
        refuse: Arc<Mutex<Option<HubError>>>,
    }

    impl Hub for Scripted {
        fn register(&mut self, _: &Identity, _: Option<&str>) -> Result<Profile, HubError> {
            unreachable!()
        }
        fn set_bio(&mut self, _: &Identity, _: &str) -> Result<Profile, HubError> {
            unreachable!()
        }
        fn profile(&mut self, _: &str) -> Result<Profile, HubError> {
            unreachable!()
        }
        fn claim(
            &mut self,
            _: &Identity,
            _: &str,
            _: u8,
            _: &str,
            _: bool,
        ) -> Result<(), HubError> {
            unreachable!()
        }
        fn release(&mut self, _: &Identity, _: &str) -> Result<(), HubError> {
            unreachable!()
        }
        fn presence(&mut self, _: &str) -> Result<Vec<crate::Presence>, HubError> {
            unreachable!()
        }
        fn relay_read(
            &mut self,
            _: &Identity,
            server: &str,
            after: u64,
            wait: u64,
            whispers: bool,
        ) -> Result<RelayBatch, HubError> {
            lock(&self.asked).push((server.to_owned(), after, wait));
            lock(&self.asked_whispers).push(whispers);
            lock(&self.answers)
                .pop_front()
                .unwrap_or_else(|| Err(HubError::Network("down".to_owned())))
        }
        fn whisper_send(
            &mut self,
            _: &Identity,
            _: &str,
            target: u8,
            to: &str,
            boxed: &str,
        ) -> Result<u64, HubError> {
            if let Some(refusal) = lock(&self.refuse).take() {
                return Err(refusal);
            }
            let mut whispered = lock(&self.whispered);
            whispered.push((target, to.to_owned(), boxed.to_owned()));
            Ok(whispered.len() as u64)
        }
    }

    const SERVER: &str = "1.2.3.4:29070";

    fn place() -> FeedShared {
        FeedShared {
            url: Some("https://hub.example".to_owned()),
            server: Some(SERVER.to_owned()),
            ..FeedShared::default()
        }
    }

    fn key_text(identity: &Identity) -> String {
        identity.public_key_text()
    }

    #[test]
    fn a_whisper_is_sealed_to_the_recipient_and_refusals_are_told() {
        let (sol, fox) = (Identity::generate().unwrap(), Identity::generate().unwrap());
        let mut hub = Scripted::default();
        let outgoing = |server: &str| whisper::Outgoing {
            serial: 1,
            server: server.to_owned(),
            target: 4,
            to: key_text(&fox),
            text: "meet at ^1the bridge".to_owned(),
        };
        assert_eq!(
            send_whisper(&sol, &mut hub, &place(), outgoing(SERVER)),
            Ok(())
        );
        let (target, to, boxed) = lock(&hub.whispered).pop().unwrap();
        assert_eq!((target, to), (4, key_text(&fox)));
        let now = chrono::Utc::now().timestamp();
        assert_eq!(
            whisper::open(&fox, &sol.public_key(), SERVER, &boxed, now).unwrap(),
            "meet at ^1the bridge"
        );
        // Not from another server than the one it was written on.
        assert!(send_whisper(&sol, &mut hub, &place(), outgoing("5.6.7.8:29070")).is_err());
        assert!(lock(&hub.whispered).is_empty());
        *lock(&hub.refuse) = Some(HubError::Rejected {
            status: 409,
            code: "not_listening".to_owned(),
            message: String::new(),
        });
        assert_eq!(
            send_whisper(&sol, &mut hub, &place(), outgoing(SERVER)),
            Err("the player no longer takes encrypted whispers".to_owned())
        );
        // The state tells each outcome once, under its serial.
        let mut state = RelayState::default();
        state.tell(1, Ok(()));
        state.tell(2, Err("no".to_owned()));
        let sent = state.take_sent();
        assert_eq!(sent.iter().map(|s| s.serial).collect::<Vec<_>>(), [1, 2]);
        assert!(state.take_sent().is_empty());
    }

    #[test]
    fn the_reader_asks_for_whispers_and_opens_them() {
        let (sol, fox) = (Identity::generate().unwrap(), Identity::generate().unwrap());
        let hub = Scripted::default();
        let shared: Shared = Arc::default();
        let made = hub.clone();
        let now = Instant::now();
        let mut reader = Reader::new(
            fox.clone(),
            Box::new(move |_| Ok(Box::new(made.clone()))),
            Arc::new(Mutex::new(place())),
            Arc::clone(&shared),
            now,
        );
        lock(&shared.0).set_whispers(true);
        let time = chrono::Utc::now().timestamp();
        let sealed = |id, boxed: String, from: &Identity| SealedWhisper {
            id,
            at: 0,
            slot: 3,
            claimed_name: "Sol".to_owned(),
            key_id: "k".to_owned(),
            from: key_text(from),
            boxed,
        };
        let good = whisper::seal(&sol, &fox.public_key(), SERVER, time, "hi").unwrap();
        // Sealed by Sol, said to come from Fox's own key: it does not open.
        let moved = whisper::seal(&sol, &fox.public_key(), SERVER, time, "x").unwrap();
        lock(&hub.answers).extend([
            Ok(RelayBatch {
                next: 7,
                ..RelayBatch::default()
            }),
            Ok(RelayBatch {
                whispers: vec![sealed(8, good, &sol), sealed(9, moved, &fox)],
                next: 9,
                ..RelayBatch::default()
            }),
        ]);
        reader.step(now);
        reader.step(now);
        assert_eq!(*lock(&hub.asked_whispers), [true, true]);
        let received = lock(&shared.0).take_received(Some(SERVER.parse().unwrap()));
        assert_eq!(received.len(), 2);
        assert_eq!(
            received[0].text.as_ref().map(|opened| opened.text.as_str()),
            Ok("hi")
        );
        assert_eq!(received[0].from, key_text(&sol));
        assert!(received[1].text.is_err());
        // Read for another server than the viewer's: dropped.
        assert!(lock(&shared.0).take_received(None).is_empty());
    }

    fn event(id: u64) -> ToyEvent {
        ToyEvent {
            id,
            at: 0,
            slot: 3,
            claimed_name: "Sol".to_owned(),
            key_id: "k".to_owned(),
            toy: "hax".to_owned(),
            event: "hack".to_owned(),
            target: 4,
        }
    }

    #[test]
    fn the_reader_listens_only_when_asked_and_reads_on_from_the_last_id() {
        let hub = Scripted::default();
        let feed = Arc::new(Mutex::new(FeedShared {
            url: Some("https://hub.example".to_owned()),
            server: Some("1.2.3.4:29070".to_owned()),
            ..FeedShared::default()
        }));
        let shared: Shared = Arc::default();
        let made = hub.clone();
        let now = Instant::now();
        let mut reader = Reader::new(
            Identity::generate().unwrap(),
            Box::new(move |_| Ok(Box::new(made.clone()))),
            Arc::clone(&feed),
            Arc::clone(&shared),
            now,
        );
        assert_eq!(reader.step(now), IDLE);
        assert!(lock(&hub.asked).is_empty(), "not listening");
        lock(&shared.0).set_listen(true);
        lock(&hub.answers).extend([
            Ok(RelayBatch {
                events: Vec::new(),
                next: 7,
                ..RelayBatch::default()
            }),
            Ok(RelayBatch {
                events: vec![event(8), event(9)],
                next: 9,
                ..RelayBatch::default()
            }),
        ]);
        assert_eq!(reader.step(now), Duration::ZERO);
        assert_eq!(reader.step(now), Duration::ZERO);
        assert_eq!(
            *lock(&hub.asked),
            [
                ("1.2.3.4:29070".to_owned(), 0, 0),
                ("1.2.3.4:29070".to_owned(), 7, WAIT)
            ]
        );
        let taken = crate::feed::take_toy_events(
            &Mutex::new(std::mem::take(lock(&shared.0).inbox())),
            Some("1.2.3.4:29070".parse().unwrap()),
        );
        assert_eq!(taken.iter().map(|e| e.id).collect::<Vec<_>>(), [8, 9]);
        // A failure waits, then reads on from 9; another server starts afresh.
        assert!(reader.step(now) > Duration::ZERO);
        assert_eq!(
            reader.step(now + RETRY_MIN).min(IDLE),
            IDLE.min(RETRY_MIN * 2)
        );
        lock(&feed).server = Some("5.6.7.8:29070".to_owned());
        lock(&hub.answers).push_back(Ok(RelayBatch::default()));
        reader.step(now + RETRY_MAX);
        let asked = lock(&hub.asked);
        assert_eq!(asked[2], ("1.2.3.4:29070".to_owned(), 9, WAIT));
        assert_eq!(asked.last().unwrap(), &("5.6.7.8:29070".to_owned(), 0, 0));
    }
}
