//! The box hunt (Sol's idea, 27/10/2026; `docs/mystery-boxes.md`, "Box hunt"): players
//! with the Box hunt toy on (`crate::box_hunt`) take part. Every 10 minutes at most the hub
//! picks one of them on the server as the mark; another who kills the mark wins a free
//! mystery box roll. Only players with it on see the mark or can be it. The hub never
//! sees kills: this client tells it who killed its player when its player is the mark,
//! so no one can claim a kill they did not make.

use super::label::{Label, Tone};
use crate::GpuState;
use glam::Vec3;
use sjk_identity::mystery_box::{self as wire, HuntEvent, HuntState};
use std::time::{Duration, Instant};

/// How often the hunt is read while the player takes part (a new mark is also told at
/// once by the hub's toy event).
const READ_EVERY: Duration = Duration::from_secs(20);
/// Soonest a read follows another asked by an event.
const READ_GAP: Duration = Duration::from_secs(2);
/// How high over a player's origin the mark's label stands, and the point it is seen by.
const MARK_RISE: f32 = 70.0;
const MARK_SIGHT: f32 = 20.0;
/// `MAX_CLIENTS`: slots of players.
const SLOTS: u16 = 32;

/// The hunt as this client follows it.
#[derive(Default)]
pub(crate) struct Hunt {
    /// The hunt last read, and the server it was read on.
    state: Option<(String, HuntState)>,
    /// The last read taken from the identity snapshot.
    seen: u64,
    /// When the hunt was last asked for.
    asked: Option<Instant>,
    /// Asked for at once (an event changed it).
    stale: bool,
    /// The obituaries counted so far.
    obituaries: u64,
    /// The mark last told to the player, so each is told once.
    told: Option<u8>,
    /// The mark the card at the top of the screen names, while they are the mark.
    card: Option<Card>,
    /// Where a box's price went when it blew up, told when its roll stops on the teddy
    /// bear rather than at the opening (Sol, 30/10/2026): when, centred or not, the text.
    laughs: Vec<(Instant, bool, String)>,
}

/// The card at the top of the screen naming the mark (Sol's idea, 28/10/2026), from the
/// moment the player is told of them until they are hunted or the hub picks another.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Card {
    pub(super) slot: u8,
    /// Their worn name, colour codes and all.
    pub(super) name: String,
    /// The player is the mark.
    pub(super) own: bool,
    /// When they were told: the card pulses as it arrives.
    pub(super) since: Instant,
}

impl Hunt {
    fn mark(&self) -> Option<&wire::HuntMark> {
        self.state
            .as_ref()
            .filter(|(_, state)| state.hunting)
            .and_then(|(_, state)| state.mark.as_ref())
    }

    /// The card to draw: the told mark's, while the player takes part.
    pub(super) fn card(&self) -> Option<&Card> {
        self.card.as_ref()
    }

    /// A new mark told at `now`: the card names them.
    fn tell(&mut self, slot: u8, own: u8, name: &str, now: Instant) -> (bool, String) {
        self.told = Some(slot);
        self.card = Some(Card {
            slot,
            name: name.to_owned(),
            own: slot == own,
            since: now,
        });
        mark_words(slot, own, name)
    }

    /// The mark was hunted: no mark until the hub picks the next one, so the last read
    /// does not tell the hunted one again.
    fn hunted(&mut self) {
        self.told = None;
        self.card = None;
        if let Some((_, state)) = &mut self.state {
            state.mark = None;
        }
    }

    /// Whether the hunt should be read at `now`.
    fn due(&self, now: Instant) -> bool {
        self.asked.is_none_or(|asked| {
            let since = now.saturating_duration_since(asked);
            since >= READ_EVERY || (self.stale && since >= READ_GAP)
        })
    }
}

/// What the player is told when slot `slot` (named `name`) becomes the mark; `own` is
/// the player's slot.
fn mark_words(slot: u8, own: u8, name: &str) -> (bool, String) {
    if slot == own {
        (
            true,
            "You are the mark!\nBox hunters win a free mystery box roll by killing you".to_owned(),
        )
    } else {
        (
            false,
            format!("Box hunt: {name}^7 is the mark. Kill them for a free mystery box roll"),
        )
    }
}

/// What a hunter is told when the box the player on slot `opener` (named `opener_name`)
/// opened blew up and `credits` went to the hunter on `to` (named `to_name`); `own` is
/// the player's slot. Nothing for the opener: their own box says it.
fn laugh_words(
    opener: u8,
    to: Option<u8>,
    credits: u64,
    own: u8,
    opener_name: &str,
    to_name: &str,
) -> Option<(bool, String)> {
    if opener == own {
        return None;
    }
    Some(match to {
        Some(to) if to == own && credits > 0 => (
            true,
            format!("{opener_name}^7's box blew up!\nYou get their {credits} credits"),
        ),
        Some(_) if credits > 0 => (
            false,
            format!(
                "Box hunt: {opener_name}^7's box blew up, and {to_name}^7 gets their {credits} credits"
            ),
        ),
        _ => (false, format!("Box hunt: {opener_name}^7's box blew up")),
    })
}

impl GpuState {
    /// The frame's box hunt: read the hunt, take the hub's events of it (`events`, this
    /// server's toy events), tell the hub who killed the player when they are the mark.
    pub(super) fn update_box_hunt(
        &mut self,
        events: &[sjk_identity::ToyEvent],
        server: &str,
        own: u8,
        now: Instant,
    ) {
        let on = self.box_hunt.on();
        let hunt = &mut self.mystery_boxes.hunt;
        // A read for this server.
        let read = crate::player_identity::with_snapshot(|snapshot| {
            snapshot
                .box_hunt
                .as_ref()
                .filter(|(serial, read_on, _)| *serial != hunt.seen && read_on == server)
                .cloned()
        })
        .flatten();
        if let Some((serial, read_on, state)) = read {
            hunt.seen = serial;
            self.mystery_boxes.free = state.free;
            hunt.state = Some((read_on, state));
            // The hub has no mark now, or another one (told below): not this card.
            let read_slot = hunt.mark().map(|mark| mark.slot);
            if hunt
                .card
                .as_ref()
                .is_some_and(|card| Some(card.slot) != read_slot)
            {
                hunt.card = None;
            }
        }
        if hunt
            .state
            .as_ref()
            .is_some_and(|(read_on, _)| read_on != server)
        {
            hunt.state = None;
            hunt.told = None;
            hunt.card = None;
        }
        let told_before = hunt.told;
        let mut lines = Vec::new();
        for event in events.iter().filter(|event| event.toy == wire::HUNT_TOY) {
            let Some(said) = wire::parse_hunt_event(&event.event) else {
                continue;
            };
            hunt.stale = true;
            if !on {
                continue;
            }
            match said {
                HuntEvent::Mark(slot) if hunt.told != Some(slot) => {
                    lines.push(hunt.tell(slot, own, &event.claimed_name, now));
                }
                HuntEvent::Hunted { slot, won } => {
                    hunt.hunted();
                    lines.push(if slot == own && won {
                        (
                            true,
                            "You hunted the mark!\nFree mystery box roll".to_owned(),
                        )
                    } else if slot == own {
                        (
                            false,
                            "Box hunt: you hunted the mark (no free roll: you hold the most \
                             or won enough today)"
                                .to_owned(),
                        )
                    } else {
                        (
                            false,
                            format!("Box hunt: {}^7 hunted the mark", event.claimed_name),
                        )
                    });
                }
                HuntEvent::Laugh {
                    opener,
                    to,
                    credits,
                } => {
                    let opener_name = self
                        .chat
                        .player_name(u16::from(opener))
                        .unwrap_or("Someone")
                        .to_owned();
                    if let Some((centre, text)) =
                        laugh_words(opener, to, credits, own, &opener_name, &event.claimed_name)
                    {
                        hunt.laughs.push((now + super::ROLL, centre, text));
                    }
                }
                HuntEvent::Mark(_) => {}
            }
        }
        // The mark read without its event (joining mid-hunt): told once too.
        let read_mark = hunt.mark().map(|mark| (mark.slot, mark.name.clone()));
        if on
            && let Some((slot, name)) = read_mark
            && hunt.told != Some(slot)
        {
            lines.push(hunt.tell(slot, own, &name, now));
        }
        // Where the price of a box that blew up went, once its roll has stopped (the box
        // laughs there, `super::MysteryBoxes::tick`).
        let due = hunt.laughs.iter().filter(|(at, ..)| *at <= now).count();
        lines.extend(
            hunt.laughs
                .drain(..due)
                .map(|(_, centre, text)| (centre, text)),
        );
        // A new mark rings the hunt's alarm, for the player alone.
        if hunt.told.is_some() && hunt.told != told_before {
            self.mystery_boxes.cues.push(super::Cue {
                path: super::HUNT_MARK,
                origin: None,
                source: super::HUNT_SOURCE,
            });
        }
        let hunt = &mut self.mystery_boxes.hunt;
        // The player, the mark, killed by another player: the hub is told by whom.
        let decoded = self.obituaries.decoded();
        let new = decoded.saturating_sub(hunt.obituaries).min(8) as usize;
        hunt.obituaries = decoded;
        let marked = on && hunt.mark().is_some_and(|mark| mark.slot == own);
        let killer = (0..new)
            .filter_map(|offset| self.obituaries.feed().newest(offset))
            .find(|kill| {
                kill.target == u16::from(own)
                    && kill.attacker < SLOTS
                    && kill.attacker != kill.target
            })
            .map(|kill| kill.attacker as u8);
        // Read when due: while taking part, while a box stands here (free rolls), or
        // when the hunt is not known yet.
        let wanted = on || !self.mystery_boxes.here.is_empty();
        let read_now = wanted && hunt.due(now);
        if read_now {
            hunt.asked = Some(now);
            hunt.stale = false;
        }
        if let (true, Some(by)) = (marked, killer) {
            hunt.asked = Some(now);
            crate::player_identity::with_service(|service| {
                service.box_hunt_killed(server.to_owned(), by);
            });
        } else if read_now {
            crate::player_identity::with_service(|service| {
                service.read_box_hunt(server.to_owned());
            });
        }
        if !on {
            self.mystery_boxes.hunt.told = None;
            self.mystery_boxes.hunt.card = None;
        }
        for (centre, text) in lines {
            if centre {
                self.mystery_box_centre(text);
            } else {
                self.mystery_box_print(text);
            }
        }
    }

    /// The mark's client slot while the player takes part and someone else is the mark:
    /// their body is ringed in red ([`crate::hud::team_outline`]).
    pub(crate) fn box_hunt_outlined(&self) -> Option<u16> {
        if !self.box_hunt.on() {
            return None;
        }
        let slot = self.mystery_boxes.hunt.mark()?.slot;
        let own = self.live_session.as_ref()?.game_state().client_num;
        (i32::from(slot) != own).then_some(u16::from(slot))
    }

    /// The mark's label over their head, while the player takes part and someone else
    /// is the mark: the point it stands at, the point it is seen by, and the label.
    pub(super) fn box_hunt_mark_label(&mut self) -> Option<(Vec3, Vec3, Label)> {
        if !self.box_hunt.on() {
            return None;
        }
        let slot = self.mystery_boxes.hunt.mark()?.slot;
        let session = self.live_session.as_ref()?;
        if i32::from(slot) == session.game_state().client_num {
            return None;
        }
        let time = self.presentation_clock.sample(Instant::now());
        let presented = self
            .live_world
            .entity(sjk_runtime::EntityId::new(u64::from(slot) + 1))?;
        let origin = Vec3::from_array(presented.sample(time).translation);
        Some((
            origin + Vec3::Z * MARK_RISE,
            origin + Vec3::Z * MARK_SIGHT,
            Label {
                headline: "MARK".to_owned(),
                line: "Kill for a free box roll".to_owned(),
                tone: Tone::Mark,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_mark_is_told_to_themselves_and_to_hunters() {
        let (centre, words) = mark_words(3, 3, "Kyle");
        assert!(centre);
        assert!(words.starts_with("You are the mark!"));
        let (centre, words) = mark_words(3, 5, "Kyle");
        assert!(!centre);
        assert!(words.contains("Kyle^7 is the mark"));
    }

    #[test]
    fn a_box_that_blew_up_is_told_to_the_other_hunters() {
        // The hunter who got the credits, in the centre.
        let (centre, words) = laugh_words(2, Some(4), 100, 4, "Kyle", "Jan").unwrap();
        assert!(centre);
        assert_eq!(words, "Kyle^7's box blew up!\nYou get their 100 credits");
        // Another hunter, in the chat.
        let (centre, words) = laugh_words(2, Some(4), 100, 6, "Kyle", "Jan").unwrap();
        assert!(!centre);
        assert!(words.contains("Jan^7 gets their 100 credits"));
        // Nobody got them, or a free roll passed nothing.
        assert_eq!(
            laugh_words(2, None, 0, 6, "Kyle", "Kyle").unwrap().1,
            "Box hunt: Kyle^7's box blew up"
        );
        assert!(!laugh_words(2, Some(4), 0, 4, "Kyle", "Jan").unwrap().0);
        // The opener's own box tells them.
        assert_eq!(laugh_words(2, Some(4), 100, 2, "Kyle", "Jan"), None);
    }

    #[test]
    fn a_told_mark_gets_the_card_until_hunted() {
        let now = Instant::now();
        let mut hunt = Hunt::default();
        let (centre, _) = hunt.tell(4, 2, "^1Kyle", now);
        assert!(!centre);
        let card = hunt.card().expect("a card");
        assert_eq!(
            (card.slot, card.name.as_str(), card.own),
            (4, "^1Kyle", false)
        );
        // The player as the mark: their own card.
        hunt.tell(2, 2, "Sol", now);
        assert!(hunt.card().is_some_and(|card| card.own));
        // Hunted: no card, and the last read's mark is not told again.
        hunt.state = Some((
            "server".to_owned(),
            HuntState {
                hunting: true,
                mark: Some(wire::HuntMark {
                    slot: 2,
                    name: "Sol".to_owned(),
                    until: 0,
                }),
                ..HuntState::default()
            },
        ));
        hunt.hunted();
        assert!(hunt.card().is_none() && hunt.told.is_none() && hunt.mark().is_none());
    }

    #[test]
    fn the_hunt_is_read_every_while_and_soon_after_an_event() {
        let now = Instant::now();
        let mut hunt = Hunt::default();
        assert!(hunt.due(now));
        hunt.asked = Some(now);
        assert!(!hunt.due(now + Duration::from_secs(5)));
        assert!(hunt.due(now + READ_EVERY));
        hunt.stale = true;
        assert!(!hunt.due(now + Duration::from_secs(1)));
        assert!(hunt.due(now + READ_GAP));
    }
}
