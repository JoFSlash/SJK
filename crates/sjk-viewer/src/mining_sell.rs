//! Selling the Mining toy's rubble to the SJK hub for credits (vinry's toy, Slash,
//! 05/11/2026; phase 2
//! of [`crate::mining`]): the hub buys it by the Fishing junk's routes and under its cap
//! (`PROTOCOL.md` there, "Fishing junk": one daily sum for fished junk and dug rubble
//! together, so mining earns nothing on top of fishing). The Mining card in Collection >
//! Toys lists the finds and, when the hub buys rubble from this player, a "Sell rubble"
//! button; `mining_sell` in the console does the same, `mining_log` prints the log. Ore
//! is never sold, as fish are not. Only what the hub's answer names is counted sold in
//! the mining log (`sold <id> <count>` lines), so the counts of what was dug never
//! change; the sale state is [`crate::fishing_sell::Sale`], one sale at a time across
//! both toys.

use crate::GpuState;
use crate::fishing_sell::Card;
use crate::mining::{FINDS, Find, Kind, Log};
use std::time::Instant;

/// The console command that sells the rubble held.
pub(crate) const COMMAND: &str = "mining_sell";
/// The console command that sells the ore held at the catch market (15/11/2026).
pub(crate) const ORE_COMMAND: &str = "mining_sellore";
/// The console command that prints the mining log in full.
pub(crate) const LOG_COMMAND: &str = "mining_log";
/// The console command that shares the last rare find in the SJK chat
/// ([`crate::catch_book`]).
pub(crate) const BRAG_COMMAND: &str = "mining_brag";

/// A find's name for the card: without its article ("a chunk of iron ore" is chunk of
/// iron ore, "the Heart of the Mountain" Heart of the Mountain).
fn short_name(find: &Find) -> &'static str {
    let name = find.name;
    ["a ", "an ", "the "]
        .iter()
        .find_map(|article| name.strip_prefix(article))
        .unwrap_or(name)
}

/// The kinds of `kind` in `log` with a count above 0 by `count`, as "name n" joined.
fn listed(log: &Log, kind: Kind, count: impl Fn(&Log, &str) -> u32) -> String {
    FINDS
        .iter()
        .filter(|f| f.kind == kind)
        .filter_map(|f| {
            let n = count(log, f.id);
            (n > 0).then(|| format!("{} {n}", short_name(f)))
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

/// Whether the hub buys rubble: an older hub's prices name none of it.
pub(crate) fn hub_buys_rubble(prices: &std::collections::BTreeMap<String, u32>) -> bool {
    FINDS
        .iter()
        .any(|f| f.kind == Kind::Rubble && prices.contains_key(f.id))
}

/// The Mining card for `log`, the hub paying `payable` now and still paying so much today
/// (`None`: no Sell button), a sale waiting (`selling`) and the last result `line`.
pub(crate) fn card(
    log: &Log,
    payable: Option<(u32, u32)>,
    selling: bool,
    line: Option<String>,
) -> Card {
    let ores: u32 = FINDS
        .iter()
        .filter(|f| f.kind == Kind::Ore)
        .map(|f| log.count(f.id))
        .sum();
    let (kinds, all) = log.kinds();
    let summary = if ores == 0 {
        "No ore dug yet.".to_owned()
    } else {
        format!("{ores} ores, {kinds} of {all} kinds")
    };
    let pieces: u32 = log.rubble_held().values().sum();
    let piece = if pieces == 1 { "piece" } else { "pieces" };
    let junk_line = match payable {
        _ if pieces == 0 => "Rubble: none".to_owned(),
        Some((pays, left)) => {
            format!("Rubble: {pieces} {piece}, worth {pays} credits ({left} left today)")
        }
        None => format!("Rubble: {pieces} {piece}"),
    };
    let entry = |f: &Find| {
        crate::catch_book::Entry::of(
            f.name,
            log.count(f.id),
            log.first(f.id),
            crate::mining::is_rare(f),
        )
    };
    let mut book: Vec<_> = FINDS
        .iter()
        .filter(|f| f.kind == Kind::Ore)
        .map(entry)
        .collect();
    let book_kinds = book.len();
    book.extend(FINDS.iter().filter(|f| f.kind == Kind::Rubble).map(entry));
    Card {
        title: "Mining log",
        goods: "rubble",
        catch_goods: "ore",
        summary,
        fish: listed(log, Kind::Ore, Log::count),
        junk: listed(log, Kind::Rubble, Log::held),
        junk_line,
        pieces,
        sell: payable.map(|(pays, _)| pays),
        selling,
        line,
        book,
        book_kinds,
        brag: brag_line(log, None),
        ..Card::default()
    }
}

/// The name of the ore `id` for the market's line ("void crystal").
pub(crate) fn ore_name(id: &str) -> Option<&'static str> {
    FINDS
        .iter()
        .find(|f| f.kind == Kind::Ore && f.id == id)
        .map(short_name)
}

/// What `mining_brag` says for `log`: the session's last rare find (`last_rare`, with
/// the count the log then held), else the log's rarest ore; `None` with no ore.
pub(crate) fn brag_line(log: &Log, last_rare: Option<(&'static Find, u32)>) -> Option<String> {
    let (find, count) = last_rare.or_else(|| log.rarest())?;
    Some(crate::catch_book::brag_line(
        "Dug out",
        find.name,
        count,
        log.kinds(),
    ))
}

/// The result line of a sale that took `pieces` for `credits`.
pub(crate) fn sold_line(pieces: u32, credits: u32) -> String {
    let piece = if pieces == 1 { "piece" } else { "pieces" };
    format!("Sold {pieces} {piece} of rubble for {credits} credits.")
}

impl GpuState {
    fn config_directory(&self) -> Option<std::path::PathBuf> {
        self.console
            .as_ref()
            .map(|c| c.config_directory().to_owned())
    }

    /// `mining_brag`, or the log book's Share button: say the session's last rare find
    /// (else the rarest ore of the log) in the SJK chat as the player's own message, once
    /// in [`crate::catch_book::BRAG_EVERY`].
    pub(crate) fn brag_mining(&mut self) {
        let Some(directory) = self.config_directory() else {
            return;
        };
        let last_rare = self.mining.last_rare;
        let line = brag_line(self.mining.log_in(&directory), last_rare);
        let said = match line {
            None => Some("Nothing to share yet: dig out some ore first."),
            Some(_) if !self.mining.brag.take(Instant::now()) => {
                Some("You shared a find a moment ago.")
            }
            Some(line) => crate::catch_book::share(line),
        };
        if let Some(said) = said {
            self.chat.receive(
                sjk_client::ServerEventKind::CenterPrint,
                said.to_owned(),
                None,
                Instant::now(),
            );
        }
    }

    /// `mining_log`: print the mining log in full to the console, every kind with its
    /// count, the rubble still held and what was sold.
    pub(crate) fn print_mining_log(&mut self) {
        let Some(directory) = self.config_directory() else {
            return;
        };
        let log = self.mining.log_in(&directory);
        let shown = card(log, None, false, None);
        let sold = listed(log, Kind::Rubble, Log::sold_count);
        let mut lines = vec![format!("Mining log: {}", shown.summary)];
        if !shown.fish.is_empty() {
            lines.push(format!("Ore: {}", shown.fish));
        }
        lines.push(if shown.junk.is_empty() {
            "Rubble held: none".to_owned()
        } else {
            format!("Rubble held: {}", shown.junk)
        });
        if !sold.is_empty() {
            lines.push(format!("Rubble sold: {sold}"));
        }
        if let Some(console) = self.console.as_mut() {
            for line in lines {
                console.push_log(line);
            }
        }
    }

    /// `mining_sell`, or the card's Sell button: sell the rubble the log holds, once no
    /// sale (this toy's or Fishing's) waits already.
    pub(crate) fn sell_mining_rubble(&mut self) {
        self.sell_mining(false);
    }

    /// `mining_sell ore`, or the card's Sell ore button (the catch market, 15/11/2026):
    /// sell the ore the log holds unsold, while the hub buys it.
    pub(crate) fn sell_mining_ore(&mut self) {
        self.sell_mining(true);
    }

    /// Sell the rubble held, or the ore (`ore`).
    fn sell_mining(&mut self, ore: bool) {
        let now = Instant::now();
        if self.mining.sale.waiting(now) || self.fishing.sale.waiting(now) {
            return;
        }
        let Some(directory) = self.config_directory() else {
            return;
        };
        let log = self.mining.log_in(&directory);
        let held = if ore {
            log.ore_held()
        } else {
            log.rubble_held()
        };
        let goods = if ore { "ore" } else { "rubble" };
        let said = if held.is_empty() {
            format!("No {goods} to sell: break some boulders first.")
        } else {
            let open = crate::player_identity::with_snapshot(|snapshot| {
                snapshot.junk.as_ref().map(|junk| {
                    let buys = if ore {
                        junk.market.is_some()
                    } else {
                        hub_buys_rubble(&junk.prices)
                    };
                    (junk.open && buys, junk.payable(&held))
                })
            })
            .flatten();
            match open {
                Some((true, paid)) if paid > 0 => {
                    if crate::player_identity::with_service(|service| service.sell_junk(held))
                        .is_some()
                    {
                        self.mining.sale.asked = Some(now);
                        self.mining.sale.goods = goods;
                        self.mining.sale.line = Some("Selling...".to_owned());
                        return;
                    }
                    format!("SJK online is off: the hub buys the {goods}.")
                }
                Some((true, _)) => {
                    "The hub buys no more today (one sum for all it buys): come back tomorrow."
                        .to_owned()
                }
                _ => {
                    // Not known yet, or closed: ask again for the next try.
                    let _ = crate::player_identity::with_service(|service| service.read_junk());
                    format!("The hub does not buy {goods} from you now.")
                }
            }
        };
        self.mining.sale.line = Some(said.clone());
        self.chat.receive(
            sjk_client::ServerEventKind::CenterPrint,
            said,
            None,
            Instant::now(),
        );
    }

    /// The sale's frame: act on the hub's answer to this toy's sale once, keep what it
    /// pays read while the card is on show, and give the console the card.
    pub(crate) fn mining_sale_frame(&mut self, now: Instant) {
        self.read_mining_sale_answer();
        let shown = self
            .console
            .as_ref()
            .is_some_and(|console| console.mining_card_shown());
        if !shown {
            return;
        }
        if self
            .mining
            .sale
            .read_at
            .is_none_or(|at| now.duration_since(at) >= crate::fishing_sell::READ_EVERY)
            && crate::player_identity::with_service(|service| service.read_junk()).is_some()
        {
            self.mining.sale.read_at = Some(now);
        }
        let Some(directory) = self.config_directory() else {
            return;
        };
        let selling = self.mining.sale.waiting(now);
        let line = self.mining.sale.line.clone();
        let last_rare = self.mining.last_rare;
        let log = self.mining.log_in(&directory);
        let held = log.rubble_held();
        let payable = crate::player_identity::with_snapshot(|snapshot| {
            snapshot
                .junk
                .as_ref()
                .filter(|junk| junk.open && hub_buys_rubble(&junk.prices))
                .map(|junk| (junk.payable(&held), junk.left))
        })
        .flatten();
        let ore = log.ore_held();
        let mut card = card(log, payable, selling, line);
        crate::player_identity::with_snapshot(|snapshot| {
            let goal = snapshot.catch_quests.as_ref().map(|day| day.mining_line());
            crate::fishing_sell::add_market(
                &mut card,
                goal,
                &ore,
                snapshot.junk.as_ref(),
                ore_name,
            );
        });
        card.brag = brag_line(log, last_rare);
        if let Some(console) = self.console.as_mut() {
            console.set_mining_card(card);
        }
    }

    /// Act on the hub's last answer to a sale, once, when it answers this toy's: count
    /// what it took sold and say so (an answer to Fishing's is passed over).
    fn read_mining_sale_answer(&mut self) {
        let seen = self.mining.sale.seen;
        let Some((serial, outcome)) = crate::player_identity::with_snapshot(|snapshot| {
            snapshot
                .junk_sale
                .as_ref()
                .filter(|(serial, _)| *serial != seen)
                .cloned()
        })
        .flatten() else {
            return;
        };
        self.mining.sale.seen = serial;
        if self.mining.sale.asked.take().is_none() {
            return;
        }
        let said = match outcome {
            Ok(sold) => {
                let Some(directory) = self.config_directory() else {
                    return;
                };
                let pieces = self.mining.log_in(&directory).sell(&sold.sold);
                self.mining.save_log(&directory);
                match std::mem::take(&mut self.mining.sale.goods) {
                    "ore" => crate::fishing_sell::sold_catch_line("ores", pieces, sold.credits),
                    _ => sold_line(pieces, sold.credits),
                }
            }
            Err(why) => why,
        };
        self.mining.sale.line = Some(said.clone());
        self.chat.receive(
            sjk_client::ServerEventKind::CenterPrint,
            said,
            None,
            Instant::now(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_card_lists_finds_and_the_rubble_held() {
        let mut log =
            Log::parse("iron_ore 3\nheart_of_the_mountain 1\nrubble 4\nfossil 1\nsold rubble 3\n");
        let shown = card(&log, Some((4, 9)), false, None);
        assert_eq!(
            shown.summary,
            format!("4 ores, 2 of {} kinds", crate::mining::ORE_KINDS)
        );
        assert_eq!(shown.fish, "chunk of iron ore 3 · Heart of the Mountain 1");
        assert_eq!(shown.junk, "handful of rubble 1 · fossil shell 1");
        assert_eq!((shown.title, shown.goods), ("Mining log", "rubble"));
        assert_eq!((shown.sell, shown.pieces), (Some(4), 2));
        assert_eq!(
            shown.junk_line,
            "Rubble: 2 pieces, worth 4 credits (9 left today)"
        );
        assert_eq!(card(&log, None, false, None).junk_line, "Rubble: 2 pieces");
        let empty = card(&Log::default(), None, false, None);
        assert_eq!(empty.summary, "No ore dug yet.");
        let all = log.rubble_held();
        assert_eq!(log.sell(&all), 2);
        assert_eq!(
            card(&log, Some((0, 0)), false, None).junk_line,
            "Rubble: none"
        );
    }

    #[test]
    fn only_a_hub_pricing_rubble_buys_it() {
        let mut prices = std::collections::BTreeMap::new();
        prices.insert("boot".to_owned(), 1);
        assert!(!hub_buys_rubble(&prices));
        prices.insert("slag".to_owned(), 1);
        assert!(hub_buys_rubble(&prices));
    }

    #[test]
    fn a_sale_says_what_it_took() {
        assert_eq!(sold_line(1, 3), "Sold 1 piece of rubble for 3 credits.");
        assert_eq!(sold_line(5, 7), "Sold 5 pieces of rubble for 7 credits.");
    }
}
