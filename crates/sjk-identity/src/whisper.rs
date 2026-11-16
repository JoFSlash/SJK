//! End-to-end encrypted whispers (`PROTOCOL.md`, "Whispers"): a whisper between two
//! SJK players goes through the hub sealed to the recipient's identity key, so the
//! hub carries it without being able to read it.
//!
//! A box is sealed with the recipient's Ed25519 public key turned into X25519 (its
//! Montgomery form): a fresh X25519 key per message, HKDF-SHA256 of the shared secret
//! for a ChaCha20-Poly1305 key, used once (so its nonce is zero). Inside, the sender
//! signs the recipient's key, the game server, the time and the text with their
//! identity key, so neither the hub nor anyone else can write a whisper in their name
//! or move one to another player or server. The hub hands out the keys: a hub that
//! lied about a key could read what is sealed to it, which is why the client
//! remembers each player's key ([`KeyPins`]) and says when it changes.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use curve25519_dalek::montgomery::MontgomeryPoint;
use ed25519_dalek::{Signature, VerifyingKey};
use hkdf::Hkdf;
use sha2::Sha256;
use std::collections::BTreeMap;

use crate::keys::{Identity, random_bytes};

/// The box format's version, its first byte.
const VERSION: u8 = 1;
/// What the key derivation and the signature are bound to.
const CONTEXT: &[u8] = b"SJK whisper v1";
/// The longest whisper text, in bytes: the game's own `tell` limit is shorter.
pub const TEXT_MAX: usize = 900;
/// How far a whisper's time may be from the reader's clock, in seconds.
pub const CLOCK_WINDOW: i64 = 600;

/// Why a box could not be opened.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OpenError {
    /// Not a box of this format, or not sealed to this key.
    Unreadable,
    /// The signature is not the sender's, or names another player or server.
    Forged,
    /// Sealed too long ago or too far ahead (a replay, or a wrong clock).
    Stale,
}

/// Why a whisper could not be sealed.
#[derive(Debug, Eq, PartialEq)]
pub enum SealError {
    /// The text is empty or longer than [`TEXT_MAX`].
    Length,
    /// The recipient's key is not a key.
    BadKey,
    /// The system gave no randomness.
    NoRandom,
}

fn signed_message(
    sender: &[u8; 32],
    recipient: &[u8; 32],
    server: &str,
    time: i64,
    text: &str,
) -> Vec<u8> {
    let mut message = Vec::with_capacity(CONTEXT.len() + 80 + server.len() + text.len());
    message.extend_from_slice(CONTEXT);
    message.extend_from_slice(sender);
    message.extend_from_slice(recipient);
    message.extend_from_slice(server.as_bytes());
    message.push(0);
    message.extend_from_slice(&time.to_le_bytes());
    message.extend_from_slice(text.as_bytes());
    message
}

fn cipher(
    shared: &MontgomeryPoint,
    ephemeral: &[u8; 32],
    recipient: &[u8; 32],
) -> ChaCha20Poly1305 {
    let mut salt = [0_u8; 64];
    salt[..32].copy_from_slice(ephemeral);
    salt[32..].copy_from_slice(recipient);
    let mut key = [0_u8; 32];
    Hkdf::<Sha256>::new(Some(&salt), shared.as_bytes())
        .expand(CONTEXT, &mut key)
        .expect("32 bytes is a valid HKDF-SHA256 length");
    ChaCha20Poly1305::new(&key.into())
}

/// The recipient's public key in X25519 form; `None` when it is not a key.
fn montgomery(key: &[u8; 32]) -> Option<MontgomeryPoint> {
    Some(VerifyingKey::from_bytes(key).ok()?.to_montgomery())
}

/// Seal `text` from `identity` to the key `recipient` for a whisper on `server` at
/// `time` (unix seconds): the box, unpadded base64url.
pub fn seal(
    identity: &Identity,
    recipient: &[u8; 32],
    server: &str,
    time: i64,
    text: &str,
) -> Result<String, SealError> {
    if text.is_empty() || text.len() > TEXT_MAX {
        return Err(SealError::Length);
    }
    let their = montgomery(recipient).ok_or(SealError::BadKey)?;
    let secret: [u8; 32] = random_bytes().map_err(|_| SealError::NoRandom)?;
    let ephemeral = MontgomeryPoint::mul_base_clamped(secret).to_bytes();
    let shared = their.mul_clamped(secret);
    let sender = identity.public_key();
    let signature = identity.sign_bytes(&signed_message(&sender, recipient, server, time, text));
    let mut plain = Vec::with_capacity(72 + text.len());
    plain.extend_from_slice(&time.to_le_bytes());
    plain.extend_from_slice(&signature);
    plain.extend_from_slice(text.as_bytes());
    let sealed = cipher(&shared, &ephemeral, recipient)
        .encrypt(
            &Nonce::default(),
            Payload {
                msg: &plain,
                aad: &sender,
            },
        )
        .map_err(|_| SealError::Length)?;
    let mut boxed = Vec::with_capacity(33 + sealed.len());
    boxed.push(VERSION);
    boxed.extend_from_slice(&ephemeral);
    boxed.extend_from_slice(&sealed);
    Ok(B64.encode(boxed))
}

/// Open `boxed`, sealed by the key `sender` to `identity` for a whisper on `server`,
/// read at `now` (unix seconds): the text.
pub fn open(
    identity: &Identity,
    sender: &[u8; 32],
    server: &str,
    boxed: &str,
    now: i64,
) -> Result<String, OpenError> {
    open_signed(identity, sender, server, boxed, now).map(|opened| opened.text)
}

/// A whisper opened with what proves it the sender's: the time and signature found
/// inside, which a player report hands the hub (`PROTOCOL.md`, "Player reports").
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Opened {
    /// The whisper's text.
    pub text: String,
    /// When the sender sealed it, unix seconds.
    pub time: i64,
    /// The sender's signature over the whisper's signed message.
    pub signature: [u8; 64],
}

/// [`open`], keeping the time and signature.
pub fn open_signed(
    identity: &Identity,
    sender: &[u8; 32],
    server: &str,
    boxed: &str,
    now: i64,
) -> Result<Opened, OpenError> {
    let bytes = B64.decode(boxed).map_err(|_| OpenError::Unreadable)?;
    let (Some(&VERSION), Some(ephemeral), Some(sealed)) =
        (bytes.first(), bytes.get(1..33), bytes.get(33..))
    else {
        return Err(OpenError::Unreadable);
    };
    let ephemeral: [u8; 32] = ephemeral.try_into().map_err(|_| OpenError::Unreadable)?;
    let own = identity.public_key();
    let shared = MontgomeryPoint(ephemeral).mul_clamped(identity.x25519_scalar());
    let plain = cipher(&shared, &ephemeral, &own)
        .decrypt(
            &Nonce::default(),
            Payload {
                msg: sealed,
                aad: sender,
            },
        )
        .map_err(|_| OpenError::Unreadable)?;
    let (Some(time), Some(signature), Some(text)) =
        (plain.get(..8), plain.get(8..72), plain.get(72..))
    else {
        return Err(OpenError::Unreadable);
    };
    let time = i64::from_le_bytes(time.try_into().map_err(|_| OpenError::Unreadable)?);
    let text = std::str::from_utf8(text).map_err(|_| OpenError::Unreadable)?;
    let signed: [u8; 64] = signature.try_into().map_err(|_| OpenError::Forged)?;
    let signature = Signature::from_bytes(&signed);
    VerifyingKey::from_bytes(sender)
        .and_then(|key| {
            key.verify_strict(
                &signed_message(sender, &own, server, time, text),
                &signature,
            )
        })
        .map_err(|_| OpenError::Forged)?;
    if (now - time).abs() > CLOCK_WINDOW {
        return Err(OpenError::Stale);
    }
    Ok(Opened {
        text: text.to_owned(),
        time,
        signature: signed,
    })
}

/// A whisper the viewer hands the relay to seal and send.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Outgoing {
    /// The viewer's number for it, which its [`Sent`] carries back.
    pub serial: u64,
    /// The game server it is for: it is not sent from another one.
    pub server: String,
    /// The recipient's slot there.
    pub target: u8,
    /// The key it is sealed to: the recipient's `whisper_key` (unpadded base64url).
    pub to: String,
    /// The text.
    pub text: String,
}

/// What became of an [`Outgoing`] whisper.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sent {
    /// The whisper's [`Outgoing::serial`].
    pub serial: u64,
    /// `Ok` when the hub took it, else why it did not go, in a player's words.
    pub result: Result<(), String>,
}

/// A whisper the relay read and opened (or could not).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Received {
    /// The sender's slot and the name their claim was made under.
    pub slot: u8,
    /// The name the sender's claim was made under.
    pub claimed_name: String,
    /// The sender's key id (their person's).
    pub key_id: String,
    /// The key that sealed it (unpadded base64url).
    pub from: String,
    /// The text, with its time and signature, or why it could not be opened.
    pub text: Result<Opened, OpenError>,
}

/// Why `failure` kept a whisper from going, in a player's words.
pub(crate) fn failure_words(failure: &crate::HubError) -> String {
    match failure {
        crate::HubError::Rejected { code, .. } if code == "not_listening" => {
            "the player no longer takes encrypted whispers".to_owned()
        }
        crate::HubError::Rejected { code, .. } if code == "whisper_quota" => {
            "you whispered too often; wait a moment".to_owned()
        }
        crate::HubError::Rejected { code, .. } if code == "whisper_not_verified" => {
            "only verified players start encrypted whispers; you may answer one for 30 \
             minutes"
                .to_owned()
        }
        crate::HubError::Rejected { code, .. } if code == "not_on_server" => {
            "the hub does not see you on this server yet".to_owned()
        }
        crate::HubError::Network(_) => "the SJK hub could not be reached".to_owned(),
        other => other.to_string(),
    }
}

/// A key as the hub writes it (unpadded base64url); `None` when it is not 32 bytes.
pub fn parse_key(text: &str) -> Option<[u8; 32]> {
    B64.decode(text).ok()?.try_into().ok()
}

/// Hash rounds of a [`fingerprint`]: they make each try of a key search cost 5200
/// SHA-512s, as Signal's safety numbers do.
const FINGERPRINT_ROUNDS: usize = 5200;
/// Digits of one key's [`fingerprint`]: six groups of five.
const FINGERPRINT_DIGITS: usize = 30;

/// One key's half of the [`safety_code`]: 30 digits (about 100 bits) from
/// [`FINGERPRINT_ROUNDS`] rounds of SHA-512 over the key alone, so it depends on no
/// other key and a key made to match it must hit all 30 digits.
fn fingerprint(key: &[u8; 32]) -> String {
    use sha2::{Digest, Sha512};
    let mut hash = Sha512::new()
        .chain_update(b"SJK safety code v2")
        .chain_update(key)
        .finalize();
    for _ in 0..FINGERPRINT_ROUNDS {
        hash = Sha512::new()
            .chain_update(hash)
            .chain_update(key)
            .finalize();
    }
    let mut digits = String::with_capacity(FINGERPRINT_DIGITS);
    for chunk in hash[..30].chunks(5) {
        let number = chunk
            .iter()
            .fold(0_u64, |number, &byte| (number << 8) | u64::from(byte));
        digits.push_str(&format!("{:05}", number % 100_000));
    }
    digits
}

/// The safety code of two players' keys (`a` and `b`, in either order): 60 digits in
/// twelve groups of five (`48210 93755 ...`), the same on both screens: the
/// [`fingerprint`] of the lower key, then the higher's. Compared outside the game, it
/// proves each has the other's real key, so not even the hub can read their whispers.
///
/// Each half belongs to one key, so a hub handing out keys of its own must find a key
/// whose 30-digit fingerprint equals a real one's, not just two keys whose codes meet:
/// the twelve-digit code of v1 (SHA-256 of both keys) fell to such a meet in seconds.
pub fn safety_code(a: &[u8; 32], b: &[u8; 32]) -> String {
    let (low, high) = if a <= b { (a, b) } else { (b, a) };
    let digits = fingerprint(low) + &fingerprint(high);
    let groups: Vec<&str> = (0..digits.len())
        .step_by(5)
        .map(|at| &digits[at..at + 5])
        .collect();
    groups.join(" ")
}

/// The players whose key the player checked with the safety code, by key id: the key
/// they checked (`whisper-verified.txt`, one `key_id key` per line).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Verified {
    keys: BTreeMap<String, String>,
}

/// Where a player stands with [`Verified`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Checked {
    /// Never checked.
    No,
    /// Checked, and this is the key.
    Yes,
    /// Checked, but their key is not the one checked any more.
    Changed,
}

impl Verified {
    /// Read the file's text; lines that do not parse are dropped.
    pub fn parse(text: &str) -> Self {
        let keys = text
            .lines()
            .filter_map(|line| line.trim().split_once(' '))
            .filter(|(id, key)| !id.is_empty() && parse_key(key).is_some())
            .map(|(id, key)| (id.to_owned(), key.to_owned()))
            .collect();
        Self { keys }
    }

    /// The file's text.
    pub fn to_text(&self) -> String {
        self.keys
            .iter()
            .map(|(id, key)| {
                format!(
                    "{id} {key}
"
                )
            })
            .collect()
    }

    /// Whether `key` is the one checked for `key_id`.
    pub fn check(&self, key_id: &str, key: &str) -> Checked {
        match self.keys.get(key_id) {
            None => Checked::No,
            Some(checked) if checked == key => Checked::Yes,
            Some(_) => Checked::Changed,
        }
    }

    /// Mark `key` as checked for `key_id`, or forget the check (`key` `None`).
    pub fn set(&mut self, key_id: &str, key: Option<&str>) {
        match key {
            Some(key) => {
                self.keys.insert(key_id.to_owned(), key.to_owned());
            }
            None => {
                self.keys.remove(key_id);
            }
        }
    }
}

/// The keys this PC has seen each player's whispers come from and go to, by key id
/// (`whisper-keys.txt`, one `key_id key` per line): a key that changes is said, since
/// the hub could hand out its own. A player's keys are those of each of their PCs.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct KeyPins {
    keys: BTreeMap<String, Vec<String>>,
}

/// What [`KeyPins::check`] found.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PinCheck {
    /// The key was seen for this player before.
    Known,
    /// The first key seen for this player.
    First,
    /// A key this PC has not seen for a player it has keys for: another PC of theirs,
    /// a key they made again, or the hub handing out one of its own.
    Changed,
}

impl KeyPins {
    /// The most keys kept for one player.
    const PER_PLAYER: usize = 8;

    /// Read the file's text; lines that do not parse are dropped.
    pub fn parse(text: &str) -> Self {
        let mut pins = Self::default();
        for line in text.lines() {
            if let Some((id, key)) = line.trim().split_once(' ')
                && parse_key(key).is_some()
                && !id.is_empty()
            {
                pins.remember(id, key);
            }
        }
        pins
    }

    /// The file's text.
    pub fn to_text(&self) -> String {
        let mut text = String::new();
        for (id, keys) in &self.keys {
            for key in keys {
                text.push_str(id);
                text.push(' ');
                text.push_str(key);
                text.push('\n');
            }
        }
        text
    }

    /// Whether `key` is one seen for `key_id`, remembering it.
    pub fn check(&mut self, key_id: &str, key: &str) -> PinCheck {
        let check = match self.keys.get(key_id) {
            None => PinCheck::First,
            Some(keys) if keys.iter().any(|known| known == key) => return PinCheck::Known,
            Some(_) => PinCheck::Changed,
        };
        self.remember(key_id, key);
        check
    }

    fn remember(&mut self, key_id: &str, key: &str) {
        let keys = self.keys.entry(key_id.to_owned()).or_default();
        if !keys.iter().any(|known| known == key) {
            if keys.len() >= Self::PER_PLAYER {
                keys.remove(0);
            }
            keys.push(key.to_owned());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SERVER: &str = "1.2.3.4:29070";

    #[test]
    fn only_the_recipient_opens_a_whisper() {
        let (sol, fox, other) = (
            Identity::generate().unwrap(),
            Identity::generate().unwrap(),
            Identity::generate().unwrap(),
        );
        let boxed = seal(&sol, &fox.public_key(), SERVER, 1_000, "hello ^1there").unwrap();
        assert_eq!(
            open(&fox, &sol.public_key(), SERVER, &boxed, 1_010).unwrap(),
            "hello ^1there"
        );
        assert_eq!(
            open(&other, &sol.public_key(), SERVER, &boxed, 1_010),
            Err(OpenError::Unreadable)
        );
        // Two boxes of the same text differ.
        assert_ne!(
            boxed,
            seal(&sol, &fox.public_key(), SERVER, 1_000, "hello ^1there").unwrap()
        );
    }

    #[test]
    fn an_opened_whisper_keeps_the_senders_proof() {
        let (sol, fox) = (Identity::generate().unwrap(), Identity::generate().unwrap());
        let boxed = seal(&sol, &fox.public_key(), SERVER, 1_000, "buy credits").unwrap();
        let opened = open_signed(&fox, &sol.public_key(), SERVER, &boxed, 1_010).unwrap();
        assert_eq!((opened.text.as_str(), opened.time), ("buy credits", 1_000));
        // What the hub checks of a reported whisper: Sol's signature, to Fox, there.
        let message = signed_message(
            &sol.public_key(),
            &fox.public_key(),
            SERVER,
            opened.time,
            &opened.text,
        );
        let key = VerifyingKey::from_bytes(&sol.public_key()).unwrap();
        assert!(
            key.verify_strict(&message, &Signature::from_bytes(&opened.signature))
                .is_ok()
        );
    }

    #[test]
    fn a_whisper_cannot_be_moved_or_put_in_another_name() {
        let (sol, fox, hub) = (
            Identity::generate().unwrap(),
            Identity::generate().unwrap(),
            Identity::generate().unwrap(),
        );
        let boxed = seal(&sol, &fox.public_key(), SERVER, 1_000, "hi").unwrap();
        // Another server.
        assert_eq!(
            open(&fox, &sol.public_key(), "5.6.7.8:29070", &boxed, 1_000),
            Err(OpenError::Forged)
        );
        // Said to come from another key: the sender is bound to the box.
        assert_eq!(
            open(&fox, &hub.public_key(), SERVER, &boxed, 1_000),
            Err(OpenError::Unreadable)
        );
        // Sealed by someone else in Sol's name: the signature is not Sol's.
        let forged = seal(&hub, &fox.public_key(), SERVER, 1_000, "hi").unwrap();
        assert!(open(&fox, &sol.public_key(), SERVER, &forged, 1_000).is_err());
        // A tampered byte.
        let mut bytes = B64.decode(&boxed).unwrap();
        *bytes.last_mut().unwrap() ^= 1;
        assert_eq!(
            open(&fox, &sol.public_key(), SERVER, &B64.encode(bytes), 1_000),
            Err(OpenError::Unreadable)
        );
    }

    #[test]
    fn an_old_whisper_is_refused() {
        let (sol, fox) = (Identity::generate().unwrap(), Identity::generate().unwrap());
        let boxed = seal(&sol, &fox.public_key(), SERVER, 1_000, "hi").unwrap();
        assert_eq!(
            open(
                &fox,
                &sol.public_key(),
                SERVER,
                &boxed,
                1_000 + CLOCK_WINDOW + 1
            ),
            Err(OpenError::Stale)
        );
    }

    #[test]
    fn text_length_is_checked() {
        let (sol, fox) = (Identity::generate().unwrap(), Identity::generate().unwrap());
        assert_eq!(
            seal(&sol, &fox.public_key(), SERVER, 0, ""),
            Err(SealError::Length)
        );
        assert_eq!(
            seal(
                &sol,
                &fox.public_key(),
                SERVER,
                0,
                &"a".repeat(TEXT_MAX + 1)
            ),
            Err(SealError::Length)
        );
        assert!(seal(&sol, &fox.public_key(), SERVER, 0, &"a".repeat(TEXT_MAX)).is_ok());
    }

    #[test]
    fn both_players_see_the_same_safety_code() {
        let (sol, fox, hub) = (
            Identity::generate().unwrap(),
            Identity::generate().unwrap(),
            Identity::generate().unwrap(),
        );
        let code = safety_code(&sol.public_key(), &fox.public_key());
        assert_eq!(code, safety_code(&fox.public_key(), &sol.public_key()));
        assert_eq!(code.len(), 71);
        assert_eq!(code.split(' ').count(), 12);
        assert!(
            code.split(' ')
                .all(|group| group.len() == 5 && group.bytes().all(|b| b.is_ascii_digit()))
        );
        // Each half is one key's own: the lower key's first, then the higher's.
        let (low, high) = if sol.public_key() <= fox.public_key() {
            (sol.public_key(), fox.public_key())
        } else {
            (fox.public_key(), sol.public_key())
        };
        assert_eq!(
            code.replace(' ', ""),
            fingerprint(&low) + &fingerprint(&high)
        );
        // A hub handing out its own key shows another code.
        assert_ne!(code, safety_code(&sol.public_key(), &hub.public_key()));
        // Pinned: the format must not change under players who compared.
        assert_eq!(
            safety_code(&[2; 32], &[1; 32]),
            "84017 83486 07110 59323 10299 40656 37217 30494 56215 28645 39432 28934"
        );
    }

    #[test]
    fn a_key_shows_the_same_half_in_every_code() {
        // A hub's own key changes the half it stands for, never the real key's half.
        let (low, mid, high) = ([1; 32], [2; 32], [3; 32]);
        let with_mid = safety_code(&low, &mid);
        let with_high = safety_code(&low, &high);
        assert_eq!(with_mid[..35], with_high[..35]);
        assert_ne!(with_mid[36..], with_high[36..]);
        assert_eq!(safety_code(&mid, &high)[..35], with_mid[36..]);
    }

    #[test]
    fn a_checked_key_is_remembered_and_a_new_one_is_said() {
        let key = B64.encode([7_u8; 32]);
        let other = B64.encode([8_u8; 32]);
        let mut verified = Verified::default();
        assert_eq!(verified.check("fox", &key), Checked::No);
        verified.set("fox", Some(&key));
        assert_eq!(verified.check("fox", &key), Checked::Yes);
        assert_eq!(verified.check("fox", &other), Checked::Changed);
        let read = Verified::parse(&verified.to_text());
        assert_eq!(read, verified);
        verified.set("fox", None);
        assert_eq!(verified.check("fox", &key), Checked::No);
        assert_eq!(
            Verified::parse(
                "fox notakey
"
            ),
            Verified::default()
        );
    }

    #[test]
    fn pins_say_when_a_players_key_changes() {
        let mut pins = KeyPins::default();
        assert_eq!(pins.check("aaaa", "k1"), PinCheck::First);
        assert_eq!(pins.check("aaaa", "k1"), PinCheck::Known);
        assert_eq!(pins.check("aaaa", "k2"), PinCheck::Changed);
        assert_eq!(pins.check("aaaa", "k2"), PinCheck::Known);
        let key = B64.encode([7_u8; 32]);
        let mut pins = KeyPins::default();
        pins.check("bbbb", &key);
        assert_eq!(KeyPins::parse(&pins.to_text()), pins);
        assert_eq!(KeyPins::parse("bbbb notakey\n\n"), KeyPins::default());
    }
}
