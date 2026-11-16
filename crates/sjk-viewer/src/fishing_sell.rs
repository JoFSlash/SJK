//! Selling the Fishing toy's junk to the SJK hub for credits (Sol's yes on SJK #278,
//! 03/11/2026): the Fishing card in Collection > Toys lists the catches and, when the hub
//! buys junk from this player, a "Sell junk" button; `fishing_sell` in the console does
//! the same. The hub pays a few credits a piece up to a small sum a day (`PROTOCOL.md`
//! there, "Fishing junk"; `sjk_identity::junk`), the most valuable pieces first, and answers what
//! it took: only that is counted sold in the fish log (`sold <id> <count>` lines), so the
//! counts of what was caught never change.

use crate::GpuState;
use crate::fishing::{CATCHES, Catch, Kind, Log};
use std::time::{Duration, Instant};

/// The console command that sells the junk held.
pub(crate) const COMMAND: &str = "fishing_sell";
/// The console command that sells the fish held at the catch market (15/11/2026).
pub(crate) const FISH_COMMAND: &str = "fishing_sellfish";
/// The console command that prints the fish log in full.
pub(crate) const LOG_COMMAND: &str = "fishing_log";
/// The console command that shares the last rare catch in the SJK chat
/// ([`crate::catch_book`]).
pub(crate) const BRAG_COMMAND: &str = "fishing_brag";
/// How often the card asks the hub what it still pays, while it is on show.
pub(crate) const READ_EVERY: Duration = Duration::from_secs(30);
/// How long a sale waits for the hub's answer before another may be asked.
pub(crate) const ASK_TIMEOUT: Duration = Duration::from_secs(20);

/// The sale's state, kept with the toy (Fishing's junk, and since 05/11/2026 Mining's
/// rubble, [`crate::mining_sell`], by the same hub routes: one sale runs at a time, and
/// each toy acts only on the answer to its own).
#[derive(Debug, Default)]
pub(crate) struct Sale {
    /// A sale was asked then and waits for its answer.
    pub(crate) asked: Option<Instant>,
    /// The last answer acted on (or passed over, the other toy's).
    pub(crate) seen: u64,
    /// The last sale's result, for the card.
    pub(crate) line: Option<String>,
    /// When the hub was last asked what it pays.
    pub(crate) read_at: Option<Instant>,
    /// What the sale waiting sells, for its result line: "" junk or rubble, "fish",
    /// "ore" (the catch market, 15/11/2026).
    pub(crate) goods: &'static str,
    /// The last kyber find told ([`GpuState::catch_news_frame`], Fishing's sale only).
    pub(crate) find_seen: u64,
    /// The day (its end) and goals last told won, so each is told once.
    pub(crate) goals_told: (i64, bool, bool),
}

/// What a kyber find says: "A kyber glint in the water! +3 kyber shards."
pub(crate) fn find_line(find: &sjk_identity::catch::KyberFind) -> String {
    let place = if find.fishing() {
        "in the water"
    } else {
        "in the rock"
    };
    let shard = if find.shards == 1 { "shard" } else { "shards" };
    format!(
        "A kyber glint {place}! +{} kyber {shard} ({} now).",
        find.shards, find.balance
    )
}

/// What newly won goal `day` tells after `told` (the day's end and the goals told won):
/// the line to say, if any, and what is told after.
pub(crate) fn goal_news(
    day: &sjk_identity::catch::CatchQuests,
    told: (i64, bool, bool),
) -> (Option<&'static str>, (i64, bool, bool)) {
    let (until, fishing, mining) = if told.0 == day.until {
        told
    } else {
        // A new day: what was won before it was told then (or not at all, before the
        // first answer).
        (day.until, false, false)
    };
    let line = if day.fishing_won && !fishing && day.won.as_deref() == Some("fishing") {
        Some("Today's fishing goal is done: an Echo is yours!")
    } else if day.mining_won && !mining && day.won.as_deref() == Some("mining") {
        Some("Today's mining goal is done: an Echo is yours!")
    } else {
        None
    };
    (
        line,
        (until, fishing || day.fishing_won, mining || day.mining_won),
    )
}

impl Sale {
    /// Whether a sale asked waits for the hub's answer at `now`.
    pub(crate) fn waiting(&self, now: Instant) -> bool {
        self.asked
            .is_some_and(|asked| now.duration_since(asked) < ASK_TIMEOUT)
    }
}

/// What a toy's card shows of its log and the sale (the Fishing card's fish log, the
/// Mining card's mining log).
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Card {
    /// The log's heading: "Fish log", "Mining log".
    pub(crate) title: &'static str,
    /// What sells, for the button and its notes: "junk", "rubble".
    pub(crate) goods: &'static str,
    /// "23 fish, 7 of 12 kinds", or that none were caught yet.
    pub(crate) summary: String,
    /// The fish caught, with their counts.
    pub(crate) fish: String,
    /// The junk held, with its counts (empty: none).
    pub(crate) junk: String,
    /// The card's one line of junk: how many pieces, and what they sell for now.
    pub(crate) junk_line: String,
    /// How many pieces of junk are held.
    pub(crate) pieces: u32,
    /// The Sell button: what it would pay now; `None` hides it (the hub does not buy
    /// junk from this player, has no junk selling, or cannot be reached).
    pub(crate) sell: Option<u32>,
    /// A sale waits for the hub.
    pub(crate) selling: bool,
    /// The last sale's result, or why the button does nothing.
    pub(crate) line: Option<String>,
    /// Lines under the junk's (15/11/2026, at most [`EXTRA_LINES`]): the day's catch
    /// quest, the catch market's wanted fish or ore.
    pub(crate) extra: Vec<String>,
    /// What the catch market buys of this toy, for its button: "fish", "ore".
    pub(crate) catch_goods: &'static str,
    /// How many fish (or ores) are held unsold.
    pub(crate) catch_pieces: u32,
    /// The Sell fish / Sell ore button: what it would pay now; `None` hides it (the
    /// market is closed to this player, or the hub has none).
    pub(crate) catch_sell: Option<u32>,
    /// The log book ([`crate::catch_book`]): the fish (or ores) in the table's order,
    /// then the junk (or rubble).
    pub(crate) book: Vec<crate::catch_book::Entry>,
    /// The kinds of the book's first part (fish, ores): the rest is junk or rubble.
    pub(crate) book_kinds: usize,
    /// What the log book's Share button would say in the SJK chat (`None`: nothing worth
    /// sharing yet).
    pub(crate) brag: Option<String>,
}

/// The most lines a card shows under its junk line (its height keeps room for them).
pub(crate) const EXTRA_LINES: usize = 1;

/// The card's lines for the catch market and the day's quest, and its catch button:
/// `goal` the day's goal line (from the hub's claim answers), `held` the fish or ore
/// held unsold, `junk` what the hub said of selling, `name_of` a catch's name by id.
/// Fills [`Card::extra`], [`Card::catch_pieces`] and [`Card::catch_sell`].
pub(crate) fn add_market(
    card: &mut Card,
    goal: Option<String>,
    held: &std::collections::BTreeMap<String, u32>,
    junk: Option<&sjk_identity::junk::JunkStatus>,
    name_of: impl Fn(&str) -> Option<&'static str>,
) {
    card.extra.clear();
    card.catch_pieces = held.values().sum();
    card.catch_sell = None;
    let market = junk
        .filter(|junk| junk.open)
        .and_then(|junk| junk.market.as_ref().map(|market| (junk, market)));
    let wanted = market.map(|(junk, market)| {
        card.catch_sell = Some(junk.payable(held));
        match name_of(&market.wanted) {
            Some(name) => {
                let price = market.prices.get(&market.wanted).copied().unwrap_or(0);
                let times = market.wanted_times.max(1);
                format!("Wanted: {name} x{times} ({price} credits)")
            }
            None => format!("The market buys {} too", card.catch_goods),
        }
    });
    let line = [goal, wanted]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" \u{b7} ");
    if !line.is_empty() {
        card.extra.push(line);
    }
}

/// The name of the fish `id` for the market's line ("Moon eel").
pub(crate) fn fish_name(id: &str) -> Option<&'static str> {
    CATCHES
        .iter()
        .find(|c| c.kind == Kind::Fish && c.id == id)
        .map(short_name)
}

/// A catch's name for the card: without its article, and before any comma ("Old
/// Whiskers, the great catfish" is Old Whiskers).
pub(crate) fn short_name(catch: &Catch) -> &'static str {
    let name = catch.name;
    let name = name
        .strip_prefix("a ")
        .or_else(|| name.strip_prefix("an "))
        .unwrap_or(name);
    name.split(',').next().unwrap_or(name)
}

/// The kinds of `kind` in `log` with a count above 0 by `count`, as "name n" joined.
fn listed(log: &Log, kind: Kind, count: impl Fn(&Log, &str) -> u32) -> String {
    CATCHES
        .iter()
        .filter(|c| c.kind == kind)
        .filter_map(|c| {
            let n = count(log, c.id);
            (n > 0).then(|| format!("{} {n}", short_name(c)))
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

/// The card for `log`, the hub paying `payable` now and still paying so much today
/// (`None`: no Sell button), a sale waiting (`selling`) and the last result `line`.
pub(crate) fn card(
    log: &Log,
    payable: Option<(u32, u32)>,
    selling: bool,
    line: Option<String>,
) -> Card {
    let fish: u32 = CATCHES
        .iter()
        .filter(|c| c.kind == Kind::Fish)
        .map(|c| log.count(c.id))
        .sum();
    let (kinds, all) = log.species();
    let summary = if fish == 0 {
        "No fish caught yet.".to_owned()
    } else {
        format!("{fish} fish, {kinds} of {all} kinds")
    };
    let pieces: u32 = log.junk_held().values().sum();
    let piece = if pieces == 1 { "piece" } else { "pieces" };
    let junk_line = match payable {
        _ if pieces == 0 => "Junk: none".to_owned(),
        Some((pays, left)) => {
            format!("Junk: {pieces} {piece}, worth {pays} credits ({left} left today)")
        }
        None => format!("Junk: {pieces} {piece}"),
    };
    let entry = |c: &Catch| {
        crate::catch_book::Entry::of(
            c.name,
            log.count(c.id),
            log.first(c.id),
            crate::fishing::is_rare(c),
        )
    };
    let mut book: Vec<_> = CATCHES
        .iter()
        .filter(|c| c.kind == Kind::Fish)
        .map(entry)
        .collect();
    let book_kinds = book.len();
    book.extend(CATCHES.iter().filter(|c| c.kind == Kind::Junk).map(entry));
    Card {
        title: "Fish log",
        goods: "junk",
        catch_goods: "fish",
        summary,
        fish: listed(log, Kind::Fish, Log::count),
        junk: listed(log, Kind::Junk, Log::held),
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

/// What `fishing_brag` says for `log`: the session's last rare catch (`last_rare`, with
/// the count the log then held), else the log's rarest fish; `None` with no fish.
pub(crate) fn brag_line(log: &Log, last_rare: Option<(&'static Catch, u32)>) -> Option<String> {
    let (catch, count) = last_rare.or_else(|| log.rarest())?;
    Some(crate::catch_book::brag_line(
        "Caught",
        catch.name,
        count,
        log.species(),
    ))
}

/// The result line of a sale that took `pieces` for `credits`.
pub(crate) fn sold_line(pieces: u32, credits: u32) -> String {
    let piece = if pieces == 1 { "piece" } else { "pieces" };
    format!("Sold {pieces} {piece} of junk for {credits} credits.")
}

/// The result line of a market sale that took `pieces` fish (or ores) for `credits`.
pub(crate) fn sold_catch_line(goods: &str, pieces: u32, credits: u32) -> String {
    format!("Sold {pieces} {goods} for {credits} credits.")
}

impl GpuState {
    /// `fishing_log`: print the fish log in full to the console, every kind with its
    /// count, the junk still held and what was sold.
    pub(crate) fn print_fishing_log(&mut self) {
        let Some(directory) = self
            .console
            .as_ref()
            .map(|c| c.config_directory().to_owned())
        else {
            return;
        };
        let log = self.fishing.log_in(&directory);
        let shown = card(log, None, false, None);
        let sold = listed(log, Kind::Junk, Log::sold_count);
        let mut lines = vec![format!("Fish log: {}", shown.summary)];
        if !shown.fish.is_empty() {
            lines.push(format!("Fish: {}", shown.fish));
        }
        lines.push(if shown.junk.is_empty() {
            "Junk held: none".to_owned()
        } else {
            format!("Junk held: {}", shown.junk)
        });
        if !sold.is_empty() {
            lines.push(format!("Junk sold: {sold}"));
        }
        if let Some(console) = self.console.as_mut() {
            for line in lines {
                console.push_log(line);
            }
        }
    }

    /// `fishing_brag`, or the log book's Share button: say the session's last rare catch
    /// (else the rarest fish of the log) in the SJK chat as the player's own message,
    /// once in [`crate::catch_book::BRAG_EVERY`].
    pub(crate) fn brag_fishing(&mut self) {
        let Some(directory) = self
            .console
            .as_ref()
            .map(|c| c.config_directory().to_owned())
        else {
            return;
        };
        let last_rare = self.fishing.last_rare;
        let line = brag_line(self.fishing.log_in(&directory), last_rare);
        let said = match line {
            None => Some("Nothing to share yet: catch a fish first."),
            Some(_) if !self.fishing.brag.take(Instant::now()) => {
                Some("You shared a catch a moment ago.")
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

    /// `fishing_sell`, or the card's Sell button: sell the junk the log holds, once a sale
    /// is not waiting already.
    pub(crate) fn sell_fishing_junk(&mut self) {
        self.sell_fishing(false);
    }

    /// `fishing_sell fish`, or the card's Sell fish button (the catch market,
    /// 15/11/2026): sell the fish the log holds unsold, while the hub buys them.
    pub(crate) fn sell_fishing_catch(&mut self) {
        self.sell_fishing(true);
    }

    /// Sell the junk held, or the fish (`fish`).
    fn sell_fishing(&mut self, fish: bool) {
        let now = Instant::now();
        // One sale at a time, the Mining card's too: the hub answers one.
        if self.fishing.sale.waiting(now) || self.mining.sale.waiting(now) {
            return;
        }
        let Some(directory) = self
            .console
            .as_ref()
            .map(|c| c.config_directory().to_owned())
        else {
            return;
        };
        let log = self.fishing.log_in(&directory);
        let held = if fish {
            log.fish_held()
        } else {
            log.junk_held()
        };
        let goods = if fish { "fish" } else { "junk" };
        let said = if held.is_empty() {
            format!("No {goods} to sell: fish some up first.")
        } else {
            let open = crate::player_identity::with_snapshot(|snapshot| {
                snapshot.junk.as_ref().map(|junk| {
                    (
                        junk.open && (!fish || junk.market.is_some()),
                        junk.payable(&held),
                    )
                })
            })
            .flatten();
            match open {
                Some((true, paid)) if paid > 0 => {
                    if crate::player_identity::with_service(|service| service.sell_junk(held))
                        .is_some()
                    {
                        self.fishing.sale.asked = Some(now);
                        self.fishing.sale.goods = goods;
                        self.fishing.sale.line = Some("Selling...".to_owned());
                        return;
                    }
                    format!("SJK online is off: the hub buys the {goods}.")
                }
                Some((true, _)) => {
                    format!("The hub buys no more {goods} from you today: come back tomorrow.")
                }
                _ => {
                    // Not known yet, or closed: ask again for the next try.
                    let _ = crate::player_identity::with_service(|service| service.read_junk());
                    format!("The hub does not buy {goods} from you now.")
                }
            }
        };
        self.fishing.sale.line = Some(said.clone());
        self.chat.receive(
            sjk_client::ServerEventKind::CenterPrint,
            said,
            None,
            Instant::now(),
        );
    }

    /// Tell a kyber find and a daily goal won, once each, as the claims' answers bring
    /// them (`sjk_identity::catch`, since 15/11/2026).
    fn catch_news_frame(&mut self) {
        let seen = self.fishing.sale.find_seen;
        let told = self.fishing.sale.goals_told;
        let Some((find, goal)) = crate::player_identity::with_snapshot(|snapshot| {
            let find = snapshot
                .kyber_find
                .as_ref()
                .filter(|(count, _)| *count != seen)
                .cloned();
            let goal = snapshot
                .catch_quests
                .as_ref()
                .map(|day| goal_news(day, told));
            (find, goal)
        }) else {
            return;
        };
        let mut said = Vec::new();
        if let Some((count, find)) = find {
            self.fishing.sale.find_seen = count;
            said.push(find_line(&find));
        }
        if let Some((line, after)) = goal {
            self.fishing.sale.goals_told = after;
            said.extend(line.map(str::to_owned));
        }
        for line in said {
            if let Some(console) = self.console.as_mut() {
                console.push_log(line.clone());
            }
            self.chat.receive(
                sjk_client::ServerEventKind::CenterPrint,
                line,
                None,
                Instant::now(),
            );
        }
    }

    /// The sale's frame: act on the hub's answer once, keep what it pays read while the
    /// card is on show, and give the console the card.
    pub(super) fn fishing_sale_frame(&mut self, now: Instant) {
        self.read_fishing_sale_answer();
        self.catch_news_frame();
        let shown = self
            .console
            .as_ref()
            .is_some_and(|console| console.fishing_card_shown());
        if !shown {
            return;
        }
        if self
            .fishing
            .sale
            .read_at
            .is_none_or(|at| now.duration_since(at) >= READ_EVERY)
            && crate::player_identity::with_service(|service| service.read_junk()).is_some()
        {
            self.fishing.sale.read_at = Some(now);
        }
        let Some(directory) = self
            .console
            .as_ref()
            .map(|c| c.config_directory().to_owned())
        else {
            return;
        };
        let selling = self.fishing.sale.waiting(now);
        let line = self.fishing.sale.line.clone();
        let last_rare = self.fishing.last_rare;
        let log = self.fishing.log_in(&directory);
        let held = log.junk_held();
        let payable = crate::player_identity::with_snapshot(|snapshot| {
            snapshot
                .junk
                .as_ref()
                .filter(|junk| junk.open)
                .map(|junk| (junk.payable(&held), junk.left))
        })
        .flatten();
        let fish = log.fish_held();
        let mut card = card(log, payable, selling, line);
        crate::player_identity::with_snapshot(|snapshot| {
            let goal = snapshot.catch_quests.as_ref().map(|day| day.fishing_line());
            add_market(&mut card, goal, &fish, snapshot.junk.as_ref(), fish_name);
        });
        card.brag = brag_line(log, last_rare);
        if let Some(console) = self.console.as_mut() {
            console.set_fishing_card(card);
        }
    }

    /// Act on the hub's last answer to a sale, once: count what it took sold and say so.
    fn read_fishing_sale_answer(&mut self) {
        let seen = self.fishing.sale.seen;
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
        self.fishing.sale.seen = serial;
        // The Mining card's sale, not this one's (`mining_sell`).
        if self.fishing.sale.asked.take().is_none() {
            return;
        }
        let said = match outcome {
            Ok(sold) => {
                let Some(directory) = self
                    .console
                    .as_ref()
                    .map(|c| c.config_directory().to_owned())
                else {
                    return;
                };
                let pieces = self.fishing.log_in(&directory).sell(&sold.sold);
                self.fishing.save_log(&directory);
                match std::mem::take(&mut self.fishing.sale.goods) {
                    "fish" => sold_catch_line("fish", pieces, sold.credits),
                    _ => sold_line(pieces, sold.credits),
                }
            }
            Err(why) => why,
        };
        self.fishing.sale.line = Some(said.clone());
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
    fn the_card_lists_catches_and_what_is_held() {
        let mut log = Log::parse("silver_darter 3\nold_whiskers 1\nboot 4\nhilt 1\nsold boot 3\n");
        let shown = card(&log, Some((6, 9)), false, None);
        assert_eq!(
            shown.summary,
            format!("4 fish, 2 of {} kinds", crate::fishing::FISH_KINDS)
        );
        assert_eq!(shown.fish, "Silver darter 3 · Old Whiskers 1");
        assert_eq!(shown.junk, "old boot 1 · broken saber hilt 1");
        assert_eq!(shown.sell, Some(6));
        assert_eq!(shown.pieces, 2);
        assert_eq!(
            shown.junk_line,
            "Junk: 2 pieces, worth 6 credits (9 left today)"
        );
        assert_eq!(card(&log, None, false, None).junk_line, "Junk: 2 pieces");
        let empty = card(&Log::default(), None, false, None);
        assert_eq!(empty.summary, "No fish caught yet.");
        assert!(empty.fish.is_empty() && empty.junk.is_empty());
        assert!(empty.brag.is_none(), "nothing to share");
        // The log book holds every kind, fish first, the ones never caught hidden.
        assert_eq!(shown.book.len(), CATCHES.len());
        assert_eq!(shown.book_kinds, crate::fishing::FISH_KINDS);
        assert_eq!(shown.book[0].name, "Silver darter");
        assert_eq!(shown.book[1].name, "???");
        let whiskers = &shown.book[crate::fishing::FISH_KINDS - 1];
        assert_eq!(
            (whiskers.name.as_str(), whiskers.rare),
            ("Old Whiskers", true)
        );
        // With no rare catch this session, the share names the rarest in the log.
        assert_eq!(
            shown.brag.as_deref(),
            Some("Caught Old Whiskers, the great catfish (my first!) - 2 of 12 kinds in my log")
        );
        // Selling everything leaves no junk listed.
        let all = log.junk_held();
        assert_eq!(log.sell(&all), 2);
        let sold = card(&log, Some((0, 0)), false, None);
        assert!(sold.junk.is_empty());
        assert_eq!(sold.junk_line, "Junk: none");
    }

    #[test]
    fn finds_and_goals_are_told_once() {
        use sjk_identity::catch::{CatchQuests, KyberFind};
        let find = KyberFind {
            toy: "mining".to_owned(),
            shards: 3,
            balance: 40,
        };
        assert_eq!(
            find_line(&find),
            "A kyber glint in the rock! +3 kyber shards (40 now)."
        );
        let mut day = CatchQuests {
            until: 100,
            ..CatchQuests::default()
        };
        let (line, told) = goal_news(&day, (0, false, false));
        assert_eq!((line, told), (None, (100, false, false)));
        day.fishing_won = true;
        day.won = Some("fishing".to_owned());
        let (line, told) = goal_news(&day, told);
        assert!(line.unwrap().contains("fishing goal"));
        assert_eq!(goal_news(&day, told).0, None, "told once");
        // A goal won before the first answer seen is not told again later.
        let (line, _) = goal_news(
            &CatchQuests {
                won: None,
                ..day.clone()
            },
            (0, false, false),
        );
        assert_eq!(line, None);
    }

    #[test]
    fn the_market_line_names_the_wanted_catch() {
        let junk = sjk_identity::junk::JunkStatus {
            open: true,
            left: 15,
            market: Some(sjk_identity::junk::Market {
                prices: [("moon_eel".to_owned(), 6), ("sun_carp".to_owned(), 3)]
                    .into_iter()
                    .collect(),
                wanted: "moon_eel".to_owned(),
                wanted_times: 2,
                until: 0,
            }),
            ..Default::default()
        };
        let log = Log::parse("moon_eel 2\nsun_carp 1\n");
        let mut shown = card(&log, None, false, None);
        add_market(
            &mut shown,
            Some("Goal".to_owned()),
            &log.fish_held(),
            Some(&junk),
            fish_name,
        );
        assert_eq!(shown.catch_sell, Some(15));
        assert_eq!(shown.catch_pieces, 3);
        assert_eq!(
            shown.extra,
            vec!["Goal \u{b7} Wanted: Moon eel x2 (6 credits)".to_owned()]
        );
        // Closed market: no button, the goal still shows.
        add_market(&mut shown, None, &log.fish_held(), None, fish_name);
        assert_eq!((shown.catch_sell, shown.extra.len()), (None, 0));
    }

    #[test]
    fn a_sale_says_what_it_took() {
        assert_eq!(sold_line(1, 5), "Sold 1 piece of junk for 5 credits.");
        assert_eq!(sold_line(4, 8), "Sold 4 pieces of junk for 8 credits.");
    }
}
