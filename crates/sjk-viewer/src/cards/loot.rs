//! The loot table's odds (`docs/holocrons.md`, "Loot table"), worked out from the hub's
//! tables ([`crate::loot_odds`]): how often a drop is each tier, the card inside a
//! holocron of each tier and its credits. A card's rarity and its finish
//! are rolled apart, so the chance of a kind is the two multiplied; "per holocron found"
//! weighs each tier by how often it drops, and counts the openings that bring a saber
//! shader or a pile of kyber shards instead of the card and its credits ([`card_in`]).
//! Every chance is 0 to 1.

use super::{FINISHES, RARITIES};
use crate::holocrons::COUNT;
use crate::loot_odds::current;

/// The chance a drop is tier `tier`.
pub(crate) fn drop(tier: usize) -> f64 {
    f64::from(current().drop[tier]) / 1_000.0
}

/// The chance an opening of a holocron of tier `tier` brings a card (and its credits),
/// not a saber shader, a tag or a pile of kyber shards.
pub(crate) fn card_in(tier: usize) -> f64 {
    let table = current();
    1.0 - f64::from(table.shader[tier] + table.tag[tier] + table.shard_pile[tier]) / 1_000.0
}

/// The chance an opening of a holocron of tier `tier` brings a pile of kyber shards.
pub(crate) fn shards_in(tier: usize) -> f64 {
    f64::from(current().shard_pile[tier]) / 1_000.0
}

/// The kyber shards a holocron of tier `tier` brings on average: a pile's size rolled
/// evenly between the least and the most, times how often it brings one.
pub(crate) fn shards_mean(tier: usize) -> f64 {
    let (least, most) = current().shards[tier];
    shards_in(tier) * f64::from(least + most) / 2.0
}

/// The chance a holocron found brings a pile of kyber shards.
pub(crate) fn shards_any() -> f64 {
    (0..COUNT).map(|tier| drop(tier) * shards_in(tier)).sum()
}

/// The kyber shards a holocron found brings on average.
pub(crate) fn shards_any_mean() -> f64 {
    (0..COUNT).map(|tier| drop(tier) * shards_mean(tier)).sum()
}

/// The chance an opening of a holocron of tier `tier` brings one given saber shader of
/// shader tier `rarity` (0 Common to 4 Mythical), one of the `pool` droppable shaders of
/// that tier the hub picks among evenly.
pub(crate) fn shader_in(tier: usize, rarity: usize, pool: usize) -> f64 {
    if pool == 0 {
        return 0.0;
    }
    let table = current();
    let tier_share = table.shader_tier[tier]
        .get(rarity)
        .map_or(0.0, |share| f64::from(*share) / 1_000.0);
    f64::from(table.shader[tier]) / 1_000.0 * tier_share / pool as f64
}

/// The chance a holocron found brings a card.
fn card_any() -> f64 {
    (0..COUNT).map(|tier| drop(tier) * card_in(tier)).sum()
}

/// The chance the card inside a holocron of tier `tier` is of rarity `rarity`.
pub(crate) fn rarity_in(tier: usize, rarity: usize) -> f64 {
    f64::from(current().card[tier].rarity[rarity]) / 1_000.0
}

/// The chance the card inside a holocron of tier `tier` has finish `finish`.
pub(crate) fn finish_in(tier: usize, finish: usize) -> f64 {
    f64::from(current().card[tier].finish[finish]) / 1_000.0
}

/// The share of the cards found that are of rarity `rarity`, any finish.
pub(crate) fn rarity_any(rarity: usize) -> f64 {
    (0..COUNT)
        .map(|tier| drop(tier) * card_in(tier) * rarity_in(tier, rarity))
        .sum::<f64>()
        / card_any()
}

/// The share of the cards found that have finish `finish`, any rarity.
pub(crate) fn finish_any(finish: usize) -> f64 {
    (0..COUNT)
        .map(|tier| drop(tier) * card_in(tier) * finish_in(tier, finish))
        .sum::<f64>()
        / card_any()
}

/// The chance a holocron found holds a card of rarity `rarity` with finish `finish`.
pub(crate) fn kind_any(rarity: usize, finish: usize) -> f64 {
    (0..COUNT)
        .map(|tier| drop(tier) * card_in(tier) * rarity_in(tier, rarity) * finish_in(tier, finish))
        .sum()
}

/// The credits a holocron of tier `tier` brings on average: rolled evenly between the
/// least and the most when it holds a card, none when it holds a shader.
pub(crate) fn credits_mean(tier: usize) -> f64 {
    let (least, most) = current().credits[tier];
    card_in(tier) * f64::from(least + most) / 2.0
}

/// The credits a holocron found brings on average.
pub(crate) fn credits_any() -> f64 {
    (0..COUNT).map(|tier| drop(tier) * credits_mean(tier)).sum()
}

/// The rarest kind of card a holocron can hold, (rarity, finish) as positions in
/// [`RARITIES`] and [`FINISHES`], and its chance per holocron found.
pub(crate) fn rarest() -> (usize, usize, f64) {
    let mut rarest = (0, 0, 1.0);
    for rarity in 0..RARITIES.len() {
        for finish in 0..FINISHES.len() {
            let chance = kind_any(rarity, finish);
            if chance > 0.0 && chance < rarest.2 {
                rarest = (rarity, finish, chance);
            }
        }
    }
    rarest
}

/// `chance` as players read it: "60%", "10.5%", "0.18%", "0.0176%": three figures at
/// most and no needless zeros after the point; "0%" for none.
pub(crate) fn percent(chance: f64) -> String {
    let percent = chance * 100.0;
    if percent <= 0.0 {
        return "0%".to_owned();
    }
    let decimals = (2 - percent.log10().floor() as i32).clamp(0, 4) as usize;
    // Halves round up: 1.715 is worked out as 1.71499..., and reads 1.72.
    let factor = 10_f64.powi(decimals as i32);
    let rounded = (percent * factor * (1.0 + 1e-9)).round() / factor;
    let mut words = format!("{rounded:.decimals$}");
    if words.contains('.') {
        words.truncate(words.trim_end_matches('0').trim_end_matches('.').len());
    }
    words.push('%');
    words
}

/// "1 in 5 698": how rare `chance` is, to the nearest whole; empty for none and for
/// anything likelier than one in two.
pub(crate) fn one_in(chance: f64) -> String {
    if chance <= 0.0 || chance > 0.5 {
        return String::new();
    }
    format!("1 in {}", super::grouped((1.0 / chance).round() as u64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_shader_is_its_tiers_share_split_among_its_pool() {
        // A Mythical holocron: 500 in 1000 bring a shader, 120 in 1000 of those Mythical.
        let mythical = crate::holocrons::COUNT - 1;
        assert!(close(shader_in(mythical, 4, 1), 0.5 * 0.12));
        assert!(close(shader_in(mythical, 4, 4), 0.5 * 0.12 / 4.0));
        // An Uncommon holocron never brings a Mythical shader, and an empty pool nothing.
        assert_eq!(shader_in(0, 4, 3), 0.0);
        assert_eq!(shader_in(mythical, 2, 0), 0.0);
        assert_eq!(shader_in(mythical, 9, 2), 0.0);
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    /// Every table sums to one, so the loot table's columns do too.
    #[test]
    fn the_chances_sum_to_one() {
        assert!(close((0..COUNT).map(drop).sum(), 1.0));
        for tier in 0..COUNT {
            assert!(close(
                (0..RARITIES.len()).map(|r| rarity_in(tier, r)).sum(),
                1.0
            ));
            assert!(close(
                (0..FINISHES.len()).map(|f| finish_in(tier, f)).sum(),
                1.0
            ));
        }
        assert!(close((0..RARITIES.len()).map(rarity_any).sum(), 1.0));
        assert!(close((0..FINISHES.len()).map(finish_any).sum(), 1.0));
        // The kinds of card sum to the chance a holocron brings a card at all; the rest
        // brings a shader or a pile of kyber shards (one in ten).
        let kinds: f64 = (0..RARITIES.len())
            .flat_map(|r| (0..FINISHES.len()).map(move |f| kind_any(r, f)))
            .sum();
        assert!(close(kinds, card_any()));
        assert!(close(card_any(), 0.55325));
        assert!(close(shards_any(), 0.1));
    }

    /// The figures `docs/holocrons.md` gives, worked by hand.
    #[test]
    fn the_figures_are_the_docs() {
        let rarities: Vec<String> = (0..5).map(|r| percent(rarity_any(r))).collect();
        assert_eq!(rarities, ["60.9%", "27.5%", "10.1%", "1.33%", "0.18%"]);
        let finishes: Vec<String> = (0..3).map(|f| percent(finish_any(f))).collect();
        assert_eq!(finishes, ["92.3%", "7.56%", "0.14%"]);
        assert!(close(credits_any(), 17.525));
        assert!(close(credits_mean(0), 9.0));
        assert!(close(shards_any_mean(), 1.70875));
        assert!(close(shards_mean(3), 15.0));
        let (rarity, finish, chance) = rarest();
        assert_eq!(
            (RARITIES[rarity].id, FINISHES[finish].id),
            ("mythical", "solar")
        );
        assert_eq!(one_in(chance), "1 in 119 581");
        assert_eq!(percent(chance), "0.0008%");
        assert_eq!(one_in(kind_any(3, 2)), "1 in 15 388");
        assert_eq!(percent(kind_any(3, 2)), "0.0065%");
        assert_eq!(percent(kind_any(0, 2)), "0.0172%");
    }

    #[test]
    fn chances_read_as_words() {
        assert_eq!(percent(0.6), "60%");
        assert_eq!(percent(0.105), "10.5%");
        assert_eq!(percent(0.015), "1.5%");
        assert_eq!(percent(0.005), "0.5%");
        assert_eq!(percent(1.0), "100%");
        assert_eq!(percent(0.0), "0%");
        assert_eq!(one_in(0.6), "");
        assert_eq!(one_in(0.0), "");
        assert_eq!(one_in(0.28), "1 in 4");
        assert_eq!(one_in(0.0018), "1 in 556");
    }
}
