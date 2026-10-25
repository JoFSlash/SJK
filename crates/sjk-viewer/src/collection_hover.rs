//! The Collection's hover cards (`docs/sjk-ui.md`, "Hover cards"): resting the pointer on
//! a medal, an achievement, a shader, a toy or a card shows beside it what it is, how it
//! is obtained and where the player stands with it ([`crate::menu::sjk::hover`]). The
//! words come from the same tables as the rest of the page: a medal's
//! [`crate::medals::Medal::how_to_get`], an achievement's goal, a blade skin's
//! `how_to_get` ([`crate::unlockables`]), a card's odds ([`crate::cards::loot`]).

use super::*;
use crate::achievements::Source;
use crate::menu::sjk::hover::{Card, Line};
use crate::menu::sjk::{color, hover};

impl Panel {
    /// Follow the pointer and draw the card of the thing it has rested on, over the
    /// page: the page's last drawing.
    pub(super) fn hover_card(&mut self, frame: &crate::menu::sjk::Frame, inputs: &Inputs<'_>) {
        if self.tab == Tab::Forge && self.forge.ask.is_some() {
            // The card asking covers the page: nothing under it answers the pointer.
            return;
        }
        let now = Instant::now();
        self.hover.follow(self.ui.hovered(), now);
        let Some((token, alpha)) = self.hover.showing(now) else {
            return;
        };
        let (Some(card), Some(anchor)) = (self.card_of(token, inputs), self.ui.rect_for(token))
        else {
            return;
        };
        hover::draw(&mut self.ui, frame, anchor, &card, alpha);
    }

    /// What the card of the thing `token` stands for says; `None` for anything else (a
    /// tab, a button).
    pub(super) fn card_of(&self, token: u16, inputs: &Inputs<'_>) -> Option<Card> {
        let within = |base: u16, count: usize| {
            token
                .checked_sub(base)
                .map(usize::from)
                .filter(|index| *index < count)
        };
        if self.tab == Tab::Forge {
            return self.forge_card(token, inputs);
        }
        if let Some(index) = within(MEDAL_BASE, self.medals_shown) {
            let (_, shown) = medals::showing(inputs);
            return shown.get(index).map(medal_card);
        }
        if let Some(index) = within(ACHIEVEMENT_BASE, crate::achievements::ALL.len()) {
            let kind = &crate::achievements::ALL[index];
            let standing = inputs
                .standings
                .iter()
                .find(|standing| standing.kind.id == kind.id);
            return Some(achievement_card(kind, standing));
        }
        if let Some(row) = within(SHADER_BASE, self.shader_count) {
            return Some(shader_card(self.skin_at(row), &inputs.holdings));
        }
        if token == TOY_TOKEN {
            return Some(Card {
                title: "Illuminate holocron".to_owned(),
                colour: color::GOLD_BRIGHT,
                lines: vec![
                    Line::how(
                        "Every SJK player has it. Light it here, from the quick wheel's Toys \
                         page or with a key bound to toy_illuminate.",
                    ),
                    Line::status(crate::editions::name(crate::editions::ALPHA)),
                    Line::held("Yours"),
                ],
            });
        }
        if token == BOX_TOKEN {
            return Some(Card {
                title: "Cardboard box".to_owned(),
                colour: color::GOLD_BRIGHT,
                lines: vec![
                    Line::how(
                        "Every SJK player has it. Get in here, from the quick wheel's Toys \
                         page or with a key bound to toy_box.",
                    ),
                    Line::status(crate::editions::name(crate::editions::ALPHA)),
                    Line::held("Yours"),
                ],
            });
        }
        if token == RAGDOLL_TOKEN {
            return Some(Card {
                title: "Ragdoll".to_owned(),
                colour: color::GOLD_BRIGHT,
                lines: vec![
                    Line::how(
                        "Every SJK player has it. Go limp here, from the quick wheel's Toys \
                         page or with a key bound to toy_ragdoll; press it again to get up.",
                    ),
                    Line::status(crate::editions::name(crate::editions::ALPHA)),
                    Line::held("Yours"),
                ],
            });
        }
        if token == DRONE_TOKEN {
            return Some(Card {
                title: "Drone".to_owned(),
                colour: color::GOLD_BRIGHT,
                lines: vec![
                    Line::how(
                        "Every SJK player has it: freecam and peek fly it. Turn it on here,                          from the quick wheel's Toys page or with a key bound to toy_drone,                          to let others peek at you.",
                    ),
                    Line::status(crate::editions::name(crate::editions::ALPHA)),
                    Line::held("Yours"),
                ],
            });
        }
        if token == HAX_TOKEN {
            return Some(Card {
                title: "The Hax".to_owned(),
                colour: color::GOLD_BRIGHT,
                lines: vec![
                    Line::how(
                        "Every SJK player has it. Turn it on here, from the quick wheel's Toys \
                         page or with a key bound to toy_hax; hold Use on a player who has it \
                         on to hack them.",
                    ),
                    Line::status(crate::editions::name(crate::editions::ALPHA)),
                    Line::held("Yours"),
                ],
            });
        }
        if token == ORDER_TOKEN {
            return Some(Card {
                title: "Order 66".to_owned(),
                colour: color::GOLD_BRIGHT,
                lines: vec![
                    Line::how(
                        "Every SJK player has it. Turn it on here, from the quick wheel's Toys \
                         page or with a key bound to toy_order66; order66 executes it.",
                    ),
                    Line::status(crate::editions::name(crate::editions::ALPHA)),
                    Line::held("Yours"),
                ],
            });
        }
        if let Some(index) = within(CARD_BASE, self.cards_shown) {
            return self.card_tiles.get(index).map(tile_card);
        }
        None
    }
}

/// A medal's card: how the SJK team gives it, the blade it brings, and whether the
/// player holds it.
fn medal_card(showing: &medals::Showing) -> Card {
    let medal = showing.medal;
    let mut lines = vec![Line::how(medal.how_to_get())];
    if let Some(blade) = medal.blade().and_then(crate::unlockables::find) {
        lines.push(Line::status(format!(
            "Brings the {} while you hold it.",
            blade.name
        )));
    }
    lines.push(Line::status(crate::editions::name(medal.edition())));
    lines.push(match &showing.award {
        Some(award) if award.given().is_empty() => Line::held("Yours"),
        Some(award) => Line::held(format!("Yours: {}", award.given().to_lowercase())),
        None => Line::status("Not given to you yet"),
    });
    Card {
        title: medal.name().to_owned(),
        colour: color::GOLD_BRIGHT,
        lines,
    }
}

/// An achievement's card: what to do, who counts it, how far the player is, and the kyber
/// crystal it brings, in the crystal's colour ([`crate::kyber::reward`]).
fn achievement_card(
    kind: &'static crate::achievements::Kind,
    standing: Option<&crate::achievements::Standing>,
) -> Card {
    let counted = match kind.source {
        Source::Client(_) => "Counted by SJK in your matches on servers.",
        Source::Hub => "Counted by the SJK hub.",
    };
    let counted = format!("{counted} {}.", crate::editions::name(kind.edition));
    let status = match standing {
        Some(standing) => match standing.unlocked {
            Some(at) if at > 0 => Line::held(format!("Unlocked {}", crate::medals::date_text(at))),
            Some(_) => Line::held("Unlocked"),
            None if kind.goal > 1 => Line::status(format!(
                "{} of {} so far",
                kind.amount(standing.progress.min(kind.goal)),
                kind.amount(kind.goal)
            )),
            None => Line::status("Not unlocked yet"),
        },
        None => Line::status("Not unlocked yet"),
    };
    let done = standing.is_some_and(|standing| standing.unlocked.is_some());
    Card {
        title: kind.name.to_owned(),
        colour: if done {
            color::GOLD_BRIGHT
        } else {
            crate::achievements::medallion::tint(kind.category)
        },
        lines: [Line::how(kind.description), Line::status(counted), status]
            .into_iter()
            .chain(crate::kyber::reward(kind.id).map(|tier| Line {
                words: crate::kyber::reward_line(tier).to_owned(),
                colour: tier.colour,
            }))
            .collect(),
    }
}

/// `words` with how many are held after it when there are several ("Yours, 2 held").
fn held_count(words: &str, count: u32) -> String {
    if count > 1 {
        format!("{words}, {count} held")
    } else {
        words.to_owned()
    }
}

/// A shader's card: the stock blade (`None`) or a blade skin, how it is obtained, and
/// whether the player holds it.
pub(super) fn shader_card(
    skin: Option<&'static crate::unlockables::Unlockable>,
    holdings: &crate::unlockables::Holdings<'_>,
) -> Card {
    let Some(skin) = skin else {
        return Card {
            title: "Stock blade".to_owned(),
            colour: color::HOLO,
            lines: vec![
                Line::how("Every player has it: the plain blade, in your saber's colour."),
                Line::held("Yours"),
            ],
        };
    };
    let status = match (holdings.unlock(skin.id), holdings.reason()) {
        (Some(unlock), _) if crate::debug_collectibles::added(unlock) => {
            Line::held(crate::debug_collectibles::NOTE.to_owned())
        }
        (Some(unlock), _) => match crate::medals::date_text(unlock.granted) {
            date if date.is_empty() => Line::held(held_count("Yours", unlock.count)),
            date => Line::held(held_count(&format!("Yours since {date}"), unlock.count)),
        },
        (None, Some(reason)) => Line::status(reason),
        (None, None) => Line::status("Locked: you can still try it on here."),
    };
    Card {
        title: skin.name.to_owned(),
        colour: skin.tier.colour(),
        lines: vec![
            Line::how(skin.how_to_get),
            // The blade number only while `debug_collectibles` is on (Sol, 21/10/2026).
            Line::status(if holdings.debug() {
                format!(
                    "Blade {}. {} blade skin. {}.",
                    skin.number_text(),
                    skin.tier.label(),
                    crate::editions::name(skin.edition)
                )
            } else {
                format!(
                    "{} blade skin. {}.",
                    skin.tier.label(),
                    crate::editions::name(skin.edition)
                )
            }),
            Line::status(format!("Saber smith: {}", skin.smith)),
            status,
        ],
    }
}

/// A tile's card on the Cards tab: what the card is, how it is obtained and the odds,
/// then the copies held by finish and edition and when the first was found, or that it
/// is not found yet.
fn tile_card(tile: &crate::cards::binder::Tile) -> Card {
    use crate::cards::binder::Slot;
    let faces = crate::cards::faces::current();
    let odds = cards::rarity_odds(tile.rarity, &faces);
    let mut lines = Vec::with_capacity(4);
    match (&tile.slot, tile.face(&faces)) {
        (_, Some(face)) => {
            lines.push(Line::how(odds));
            // Each perk after its mark, as the tile's badges show it.
            for perk in face.perks() {
                lines.push(Line::status(format!("{}  {}", perk.mark, perk.words)));
            }
            // No number, whatever the kind (Sol, 20/10/2026): it stays the card's id.
            lines.push(Line::status(format!(
                "{} card. {}, {}.",
                tile.rarity.name,
                face.kind.name(),
                face.set_name
            )));
        }
        (Slot::Sealed, _) => {
            lines.push(Line::how(
                "Sealed: a card of this rarity whose face is picked when it is revealed,                  into the first set that has cards of it.",
            ));
            lines.push(Line::status(odds));
        }
        _ => {
            lines.push(Line::how("Its face is not in your card pack yet."));
            lines.push(Line::status(format!("{} card.", tile.rarity.name)));
        }
    }
    if tile.owned() {
        lines.push(Line::held(tile.breakdown()));
        let date = crate::medals::date_text(tile.first());
        if !date.is_empty() {
            lines.push(Line::status(format!("First found {date}")));
        }
    } else {
        lines.push(Line::status("Not found yet"));
    }
    Card {
        title: tile.name(&faces),
        colour: if tile.owned() {
            tile.rarity.colour
        } else {
            color::MUTED
        },
        lines,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_medal_says_how_it_is_given() {
        for medal in crate::medals::Medal::ALL {
            let card = medal_card(&medals::Showing { medal, award: None });
            assert!(card.lines[0].words.starts_with("Given by the SJK team"));
            let names_a_blade = card.lines.iter().any(|line| line.words.contains("blade"));
            assert_eq!(names_a_blade, medal.blade().is_some(), "{medal:?}");
            assert_eq!(card.lines.last().unwrap().words, "Not given to you yet");
        }
    }

    #[test]
    fn every_achievement_says_what_to_do_and_how_far() {
        for kind in &crate::achievements::ALL {
            let card = achievement_card(kind, None);
            assert_eq!(card.lines[0].words, kind.description);
            assert_eq!(card.lines.len(), 4);
            let reward = crate::kyber::reward(kind.id).expect("every achievement brings one");
            assert_eq!(card.lines[3].words, format!("Reward: {}", reward.name));
            assert_eq!(card.lines[3].colour, reward.colour);
        }
        let kind = crate::achievements::ALL
            .iter()
            .find(|kind| kind.goal == 100)
            .unwrap();
        let standing = crate::achievements::Standing {
            kind,
            progress: 37,
            unlocked: None,
        };
        let card = achievement_card(kind, Some(&standing));
        assert_eq!(card.lines[2].words, "37 of 100 so far");
        let done = crate::achievements::Standing {
            unlocked: Some(1_791_641_100),
            ..standing
        };
        let card = achievement_card(kind, Some(&done));
        assert_eq!(card.lines[2].words, "Unlocked 10/10/2026");
        assert_eq!(card.colour, color::GOLD_BRIGHT);
    }

    #[test]
    fn every_shader_says_how_to_get_it() {
        let card = shader_card(None, &crate::unlockables::Holdings::Known(&[]));
        assert_eq!(card.title, "Stock blade");
        for skin in crate::unlockables::blade_skins_by_tier() {
            let card = shader_card(Some(skin), &crate::unlockables::Holdings::Known(&[]));
            assert_eq!(card.lines[0].words, skin.how_to_get);
            assert_eq!(card.lines[2].words, format!("Saber smith: {}", skin.smith));
            assert!(card.lines[3].words.starts_with("Locked"));
        }
        let held = [sjk_identity::Unlock {
            id: "saber_sun".into(),
            granted: 1_791_336_225,
            note: String::new(),
            count: 1,
            medal: None,
            card: None,
        }];
        let sun = crate::unlockables::find("saber_sun").unwrap();
        let card = shader_card(Some(sun), &crate::unlockables::Holdings::Known(&held));
        assert!(card.lines[3].words.starts_with("Yours since "), "{card:?}");
        assert_eq!(card.lines[1].words, "Mythical blade skin. Alpha Edition.");
        // Debug collectibles on: the blade number shows as well.
        let held = [sjk_identity::Unlock {
            note: crate::debug_collectibles::NOTE.to_owned(),
            ..held[0].clone()
        }];
        let card = shader_card(Some(sun), &crate::unlockables::Holdings::Known(&held));
        assert_eq!(
            card.lines[1].words,
            "Blade 001. Mythical blade skin. Alpha Edition."
        );
    }

    #[test]
    fn a_tile_says_where_it_comes_from_and_what_is_held() {
        let stacks = crate::cards::stacks(&[
            sjk_identity::CardStack {
                card: None,
                rarity: "mythical".to_owned(),
                finish: "solar".to_owned(),
                edition: "alpha".to_owned(),
                count: 1,
                first: 1_791_641_100,
                last: 1_791_641_100,
            },
            sjk_identity::CardStack {
                card: None,
                rarity: "mythical".to_owned(),
                finish: "normal".to_owned(),
                edition: "alpha".to_owned(),
                count: 2,
                first: 1_791_700_000,
                last: 1_791_700_000,
            },
        ]);
        let faces = crate::cards::faces::Faces::of_list(Vec::new());
        let tiles = crate::cards::binder::tiles(&stacks, &faces);
        let card = tile_card(&tiles[0]);
        assert_eq!(card.title, "Sealed Mythical cards");
        assert_eq!(
            card.lines[1].words,
            "From holocrons: about 1 in 1 004 holocrons found holds a Mythical card (0.0996%)."
        );
        assert_eq!(
            card.lines[2].words,
            "3 copies: 1 Solar, 2 Normal · Alpha Edition"
        );
        assert_eq!(card.lines[3].words, "First found 10/10/2026");
    }
}
