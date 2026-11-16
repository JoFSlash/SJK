//! SJK Symbols (`docs/sjk-ui.md`, "Symbols"): SJK's own symbol font, marks for qualities
//! drawn in the colour of the text around them. Each symbol is a character in Unicode's
//! Private Use Area, so any text can hold one ("\u{E003} Legendary"); the font
//! (`assets/fonts/SJKSymbols.ttf`, built by `scripts/sjk_symbols.py`) draws it through
//! the wide-glyph atlas ([`crate::text::wide`]) after every face's own fonts.
//!
//! A symbol keeps its code point for good: a new one takes the next free point of its
//! group in the script and gets its name here.

/// The rarity marks, Common to Unique: one holocron octagon for every rank, the marks
/// inside it growing (empty, a core, an axis, a cross round a core, a second frame), then
/// Unique's (25/10/2026): the two frames round a solid four-point star.
pub(crate) const RARITY: [char; 6] = [
    '\u{E000}', '\u{E001}', '\u{E002}', '\u{E003}', '\u{E004}', '\u{E005}',
];

/// The cards' finishes: Normal (a card), Holo (a card with slanting bands), Solar (a
/// card with a sun at its heart).
pub(crate) const FINISH: [char; 3] = ['\u{E010}', '\u{E011}', '\u{E012}'];

/// The items' icons (`docs/sjk-ui.md`, "Symbols"), line pictures in the rarity marks'
/// stroke: a holocron (found in one), a medal (comes with one), a padlock (locked), a
/// tick (yours), two copies (held more than once), a smith's hammer (who made it), a
/// turning arrow (drag to turn), a lightsaber (worn), a gift (from the team) and a star
/// (its edition).
pub(crate) const HOLOCRON: char = '\u{E020}';
pub(crate) const MEDAL: char = '\u{E021}';
pub(crate) const LOCK: char = '\u{E022}';
pub(crate) const TICK: char = '\u{E023}';
pub(crate) const COPIES: char = '\u{E024}';
pub(crate) const SMITH: char = '\u{E025}';
pub(crate) const TURN: char = '\u{E026}';
pub(crate) const SABER: char = '\u{E027}';
pub(crate) const GIFT: char = '\u{E028}';
pub(crate) const EDITION: char = '\u{E029}';

/// Every icon, for the font's test.
#[cfg(test)]
const ICONS: [char; 10] = [
    HOLOCRON, MEDAL, LOCK, TICK, COPIES, SMITH, TURN, SABER, GIFT, EDITION,
];
/// The kyber shard (17/10/2026): a long crystal with a ridge down its middle, as the
/// shapes the client draws it with ([`crate::cards::shard`]). The credits' sign is not in
/// the font: it is Lucasfilm's, drawn from shapes ([`crate::cards::mark`]).
pub(crate) const SHARD: char = '\u{E030}';

/// The rarity mark of a card rarity (`crate::cards::RARITIES` order, Common to
/// Mythical): cards have no Unique, so an index past Mythical keeps Mythical's.
pub(crate) fn card_rarity(index: usize) -> char {
    RARITY[index.min(crate::cards::RARITIES.len() - 1)]
}

/// The mark of a card finish (`crate::cards::FINISHES` order).
pub(crate) fn finish(index: usize) -> char {
    FINISH[index.min(FINISH.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ab_glyph::{Font, FontRef};

    /// Every symbol the client names is in the font, and the font is the script's
    /// (`python scripts/sjk_symbols.py --check` checks the bytes).
    #[test]
    fn every_symbol_is_in_the_font() {
        let font = FontRef::try_from_slice(include_bytes!("../assets/fonts/SJKSymbols.ttf"))
            .expect("SJK Symbols reads");
        for symbol in RARITY
            .iter()
            .chain(FINISH.iter())
            .chain(ICONS.iter())
            .chain([SHARD].iter())
        {
            assert_ne!(font.glyph_id(*symbol).0, 0, "U+{:04X}", u32::from(*symbol));
        }
        // Not a letter the faces have: a symbol never stands in for one.
        assert_eq!(font.glyph_id('A').0, 0);
    }

    #[test]
    fn the_ranks_line_up() {
        assert_eq!(card_rarity(0), '\u{E000}');
        assert_eq!(card_rarity(3), '\u{E003}');
        assert_eq!(card_rarity(4), '\u{E004}');
        assert_eq!(card_rarity(9), '\u{E004}', "a card is never Unique");
        use crate::unlockables::Rarity;
        assert_eq!(Rarity::Common.symbol(), '\u{E000}');
        assert_eq!(
            Rarity::Uncommon.symbol(),
            '\u{E001}',
            "an Uncommon holocron"
        );
        assert_eq!(Rarity::Mythical.symbol(), '\u{E004}', "a Mythical one");
        assert_eq!(Rarity::Unique.symbol(), '\u{E005}', "rarityUnique");
        assert_eq!(finish(2), '\u{E012}');
    }
}
