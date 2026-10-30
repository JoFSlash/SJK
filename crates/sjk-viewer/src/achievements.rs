//! Achievements: milestones of the player's own play, shown on the Profile page's
//! board (`docs/identity.md`, "Achievements"). Like medals they grant nothing.
//!
//! Most are counted here, from what the client sees in its matches on servers
//! ([`tracker`]): players defeated, duels won, flags captured, maps and servers played,
//! time played. The counts are kept in `achievements.json` in the settings folder, so
//! they work with the identity off, and are sent to the SJK hub with it on, which keeps
//! them with the player's profile (`PROTOCOL.md`, "Achievements"). A few the hub counts
//! itself, from what the player did there (a bio, bug reports, world notes, medals).
//!
//! The hub cannot see a match, so what the client counts is the player's own record:
//! the hub bounds how fast each count may rise, nothing more. With a hub it has the
//! last word all the same: one the client counts that reached its goal here waits for
//! the hub to unlock it (and grant its kyber crystal) before it is announced
//! ([`Standing::waiting`]); without one it unlocks here, with no crystal. Adding an achievement is
//! one entry in [`ALL`] and the same id in the hub's catalogue.
//!
//! A hidden achievement ([`Kind::hidden`]) shows as a secret until it is unlocked; a
//! retired one ([`Kind::retired`]) can no longer be earned and shows only to those who
//! hold it ([`Standing::listed`]).

pub(crate) mod medallion;
pub(crate) mod tracker;

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

/// The file the counts are kept in, in the settings folder.
const FILE: &str = "achievements.json";
/// Most maps, servers and duel partners remembered (their counts' goals are far below).
const SET_MAX: usize = 512;
/// Longest map name, server address or player name kept.
const NAME_MAX: usize = 64;
/// Shortest time between two writes of the file while counts change.
const SAVE_EVERY: Duration = Duration::from_secs(20);

/// What the client counts: each achievement it counts reads one of these.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Counter {
    /// Players defeated.
    Kills,
    /// Players defeated with a saber.
    SaberKills,
    /// The most players defeated in one life.
    BestStreak,
    /// Players defeated by Force lightning or drain.
    DarkSideKills,
    /// Players a fall, a pit or a hazard finished after the player hit or pushed them.
    LedgeKills,
    /// Kinds of weapon the player defeated someone with.
    Weapons,
    /// Duels won.
    DuelWins,
    /// Flags captured.
    Captures,
    /// Different maps played.
    Maps,
    /// Different servers played on.
    Servers,
    /// Minutes played on servers.
    Minutes,
    /// Minutes hidden in the cardboard box with another player close by.
    BoxMinutes,
    /// Duels won without taking damage.
    DuelFlawless,
    /// Duels won with less than 5 health left.
    DuelCloseCall,
    /// Different players duelled (by name).
    DuelPartners,
    /// Players defeated by a long saber throw.
    SaberThrowFar,
    /// Players defeated with their own thermal detonator.
    ThermalReturn,
    /// Duels won with the saber in each style (`fd.saberAnimLevel` at the kill).
    DuelStyleBlue,
    DuelStyleYellow,
    DuelStyleRed,
    DuelStyleDual,
    DuelStyleStaff,
    /// Fish landed with the Fishing toy ([`crate::fishing`]).
    FishCaught,
    /// Kinds of fish landed (bits of [`fish_kind`]).
    FishKinds,
    /// Echoes landed on the line.
    EchoesLanded,
    /// Old Whiskers, the great catfish, landed.
    GreatCatfish,
    /// Players struck with a monitor during the player's Screen Saver who raised it
    /// (`monitor/saver.rs`).
    MonitorPayback,
    /// Screen Savers during which the player struck three different players.
    MonitorSurge,
}

impl Counter {
    const COUNT: usize = 28;
    const ALL: [Self; Self::COUNT] = [
        Self::Kills,
        Self::SaberKills,
        Self::BestStreak,
        Self::DarkSideKills,
        Self::LedgeKills,
        Self::Weapons,
        Self::DuelWins,
        Self::Captures,
        Self::Maps,
        Self::Servers,
        Self::Minutes,
        Self::BoxMinutes,
        Self::DuelFlawless,
        Self::DuelCloseCall,
        Self::DuelPartners,
        Self::SaberThrowFar,
        Self::ThermalReturn,
        Self::DuelStyleBlue,
        Self::DuelStyleYellow,
        Self::DuelStyleRed,
        Self::DuelStyleDual,
        Self::DuelStyleStaff,
        Self::FishCaught,
        Self::FishKinds,
        Self::EchoesLanded,
        Self::GreatCatfish,
        Self::MonitorPayback,
        Self::MonitorSurge,
    ];

    /// Its name in `achievements.json`.
    const fn key(self) -> &'static str {
        match self {
            Self::Kills => "kills",
            Self::SaberKills => "saber_kills",
            Self::BestStreak => "best_streak",
            Self::DarkSideKills => "dark_side_kills",
            Self::LedgeKills => "ledge_kills",
            Self::Weapons => "weapons",
            Self::DuelWins => "duel_wins",
            Self::Captures => "captures",
            Self::Maps => "maps",
            Self::Servers => "servers",
            Self::Minutes => "minutes",
            Self::BoxMinutes => "box_minutes",
            Self::DuelFlawless => "duel_flawless",
            Self::DuelCloseCall => "duel_close_calls",
            Self::DuelPartners => "duel_partners",
            Self::SaberThrowFar => "saber_throws_far",
            Self::ThermalReturn => "thermal_returns",
            Self::DuelStyleBlue => "duel_style_blue",
            Self::DuelStyleYellow => "duel_style_yellow",
            Self::DuelStyleRed => "duel_style_red",
            Self::DuelStyleDual => "duel_style_dual",
            Self::DuelStyleStaff => "duel_style_staff",
            Self::FishCaught => "fish_caught",
            Self::FishKinds => "fish_kinds",
            Self::EchoesLanded => "echoes_landed",
            Self::GreatCatfish => "great_catfish",
            Self::MonitorPayback => "monitor_paybacks",
            Self::MonitorSurge => "monitor_surges",
        }
    }

    /// Whether the count is the size of a set the record keeps (maps, servers, weapons,
    /// duel partners, kinds of fish) rather than a number.
    const fn is_set(self) -> bool {
        matches!(
            self,
            Self::Maps | Self::Servers | Self::Weapons | Self::DuelPartners | Self::FishKinds
        )
    }
}

/// Where a board shows an achievement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Category {
    Combat,
    Duels,
    Journeys,
    Community,
    Collection,
    Toys,
}

impl Category {
    #[cfg(test)]
    pub(crate) const ALL: [Self; 6] = [
        Self::Combat,
        Self::Duels,
        Self::Journeys,
        Self::Community,
        Self::Collection,
        Self::Toys,
    ];

    /// The name a board reads.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Combat => "Combat",
            Self::Duels => "Duels and flags",
            Self::Journeys => "Journeys",
            Self::Community => "Community",
            Self::Collection => "Collection",
            Self::Toys => "Toys",
        }
    }
}

/// Who counts an achievement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Source {
    /// The client, from its matches.
    Client(Counter),
    /// The hub, from what the player did there.
    Hub,
}

/// One achievement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Kind {
    /// The hub's id.
    pub(crate) id: &'static str,
    pub(crate) name: &'static str,
    /// What to do, in a sentence.
    pub(crate) description: &'static str,
    /// The count that unlocks it.
    pub(crate) goal: u64,
    pub(crate) category: Category,
    pub(crate) source: Source,
    /// The edition it belongs to ([`crate::editions`]).
    pub(crate) edition: &'static str,
    /// A secret until unlocked: boards show it as [`SECRET_NAME`], with no goal or count.
    pub(crate) hidden: bool,
    /// No longer obtainable (Surveyor, 26/10/2026): shown only to those who hold it, and
    /// counted in a person's total only then.
    pub(crate) retired: bool,
}

/// What a board calls a hidden achievement not unlocked yet.
pub(crate) const SECRET_NAME: &str = "Secret achievement";
/// What a board says a hidden achievement not unlocked yet asks.
pub(crate) const SECRET_DESCRIPTION: &str = "A secret. Unlock it to find out what it is.";
/// What a board says of an achievement reached here that the hub has not unlocked yet.
pub(crate) const WAITING_NOTE: &str = "Waiting for the SJK hub";
/// What a board says of a retired achievement its holder keeps.
pub(crate) const RETIRED_NOTE: &str = "No longer obtainable.";

const fn kind(
    id: &'static str,
    name: &'static str,
    description: &'static str,
    goal: u64,
    category: Category,
    source: Source,
) -> Kind {
    Kind {
        id,
        name,
        description,
        goal,
        category,
        source,
        edition: crate::editions::ALPHA,
        hidden: false,
        retired: false,
    }
}

impl Kind {
    /// The same achievement, hidden until unlocked.
    const fn secret(self) -> Self {
        Self {
            hidden: true,
            ..self
        }
    }

    /// The same achievement, retired.
    const fn retire(self) -> Self {
        Self {
            retired: true,
            ..self
        }
    }
}

use Category::{Collection, Combat, Community, Duels, Journeys, Toys};
use Counter as C;
use Source::{Client, Hub};

/// Every achievement, in the hub's catalogue order.
pub(crate) const ALL: [Kind; 47] = [
    kind(
        "first_blood",
        "First Blood",
        "Defeat another player on a server.",
        1,
        Combat,
        Client(C::Kills),
    ),
    kind(
        "kills_100",
        "Centurion",
        "Defeat 100 players.",
        100,
        Combat,
        Client(C::Kills),
    ),
    kind(
        "kills_1000",
        "Legend of the Arena",
        "Defeat 1000 players.",
        1000,
        Combat,
        Client(C::Kills),
    ),
    kind(
        "saber_kills_100",
        "Blademaster",
        "Defeat 100 players with your saber.",
        100,
        Combat,
        Client(C::SaberKills),
    ),
    kind(
        "streak_5",
        "Rampage",
        "Defeat 5 players without falling.",
        5,
        Combat,
        Client(C::BestStreak),
    ),
    kind(
        "streak_10",
        "Unstoppable",
        "Defeat 10 players without falling.",
        10,
        Combat,
        Client(C::BestStreak),
    ),
    kind(
        "dark_side_25",
        "Unlimited Power",
        "Defeat 25 players with Force lightning or drain.",
        25,
        Combat,
        Client(C::DarkSideKills),
    ),
    kind(
        "ledge_10",
        "Watch Your Step",
        "Send 10 players to a fall, a pit or a hazard.",
        10,
        Combat,
        Client(C::LedgeKills),
    ),
    kind(
        "arsenal",
        "Arsenal",
        "Defeat players with 8 different weapons.",
        8,
        Combat,
        Client(C::Weapons),
    ),
    kind(
        "duel_wins_10",
        "Duelist",
        "Win 10 duels.",
        10,
        Duels,
        Client(C::DuelWins),
    ),
    kind(
        "duel_wins_100",
        "Duel Master",
        "Win 100 duels.",
        100,
        Duels,
        Client(C::DuelWins),
    ),
    kind(
        "captures_10",
        "Flag Runner",
        "Capture 10 flags.",
        10,
        Duels,
        Client(C::Captures),
    ),
    kind(
        "maps_10",
        "Traveller",
        "Play on 10 different maps.",
        10,
        Journeys,
        Client(C::Maps),
    ),
    kind(
        "maps_25",
        "Galaxy Tour",
        "Play on 25 different maps.",
        25,
        Journeys,
        Client(C::Maps),
    ),
    kind(
        "servers_5",
        "Server Hopper",
        "Play on 5 different servers.",
        5,
        Journeys,
        Client(C::Servers),
    ),
    kind(
        "hours_10",
        "Regular",
        "Play 10 hours on servers.",
        600,
        Journeys,
        Client(C::Minutes),
    ),
    kind(
        "hours_100",
        "Veteran",
        "Play 100 hours on servers.",
        6000,
        Journeys,
        Client(C::Minutes),
    ),
    kind(
        "storyteller",
        "Storyteller",
        "Write a bio on your profile.",
        1,
        Community,
        Hub,
    ),
    kind(
        "bug_reporter",
        "Bug Reporter",
        "Send a bug report to the SJK team.",
        1,
        Community,
        Hub,
    ),
    // Retired 26/10/2026 (Sol): its holders keep it, nobody earns it any more.
    kind(
        "surveyor",
        "Surveyor",
        "Pin 5 world notes for the SJK team.",
        5,
        Community,
        Hub,
    )
    .retire(),
    kind(
        "decorated",
        "Decorated",
        "Receive a medal from the SJK team.",
        1,
        Community,
        Hub,
    ),
    // 26/10/2026 (Sol): the fourteen after Decorated, with the hub's ids and goals.
    kind(
        "holocrons_10",
        "Collector",
        "Open 10 holocrons.",
        10,
        Collection,
        Hub,
    ),
    kind(
        "holocrons_50",
        "Archivist",
        "Open 50 holocrons.",
        50,
        Collection,
        Hub,
    ),
    kind(
        "holocrons_200",
        "Keeper of the Holocrons",
        "Open 200 holocrons.",
        200,
        Collection,
        Hub,
    ),
    kind(
        "mythical_find",
        "Touched by the Force",
        "Find a Mythical in a holocron.",
        1,
        Collection,
        Hub,
    )
    .secret(),
    kind(
        "craft_blade",
        "Craftsman",
        "Craft a blade from a card.",
        1,
        Collection,
        Hub,
    ),
    // Every finish a blade can be crafted from: Normal, Holo and Solar.
    kind(
        "craft_finishes",
        "Master Craftsman",
        "Craft a blade from a Normal, a Holo and a Solar card.",
        crate::cards::FINISHES.len() as u64,
        Collection,
        Hub,
    ),
    kind(
        "full_set",
        "Full Set",
        "Hold every Common and Uncommon card of one set.",
        1,
        Collection,
        Hub,
    ),
    kind(
        "order66_kill",
        "Executor",
        "Defeat a player with Order 66.",
        1,
        Toys,
        Hub,
    ),
    kind(
        "box_minutes_10",
        "Tactical Espionage",
        "Spend 10 minutes hidden in the cardboard box with players close by.",
        10,
        Toys,
        Client(C::BoxMinutes),
    ),
    kind(
        "duel_flawless",
        "Flawless",
        "Win a duel without taking damage.",
        1,
        Duels,
        Client(C::DuelFlawless),
    ),
    kind(
        "duel_close_call",
        "Close Call",
        "Win a duel with less than 5 health left.",
        1,
        Duels,
        Client(C::DuelCloseCall),
    ),
    kind(
        "duel_partners_25",
        "Well Met",
        "Duel 25 different players.",
        25,
        Duels,
        Client(C::DuelPartners),
    ),
    kind(
        "saber_throw_far",
        "Long Throw",
        "Defeat a player with a long-range saber throw.",
        1,
        Combat,
        Client(C::SaberThrowFar),
    ),
    kind(
        "thermal_return",
        "Return to Sender",
        "Defeat a player with their own thermal detonator.",
        1,
        Combat,
        Client(C::ThermalReturn),
    ),
    kind(
        "style_blue_50",
        "Blue Master",
        "Win 50 duels with the saber in the Fast (blue) style.",
        50,
        Duels,
        Client(C::DuelStyleBlue),
    ),
    kind(
        "style_yellow_50",
        "Yellow Master",
        "Win 50 duels with the saber in the Medium (yellow) style.",
        50,
        Duels,
        Client(C::DuelStyleYellow),
    ),
    kind(
        "style_red_50",
        "Red Master",
        "Win 50 duels with the saber in the Strong (red) style.",
        50,
        Duels,
        Client(C::DuelStyleRed),
    ),
    kind(
        "style_dual_50",
        "Dual Sabers Master",
        "Win 50 duels with two sabers.",
        50,
        Duels,
        Client(C::DuelStyleDual),
    ),
    kind(
        "style_staff_50",
        "Staff Master",
        "Win 50 duels with a saber staff.",
        50,
        Duels,
        Client(C::DuelStyleStaff),
    ),
    // 29/10/2026 (Slash): the Fishing toy's, counted on servers as the rest.
    kind(
        "fish_first",
        "First Bite",
        "Land a fish with the Fishing toy.",
        1,
        Toys,
        Client(C::FishCaught),
    ),
    kind(
        "fish_50",
        "Angler",
        "Land 50 fish.",
        50,
        Toys,
        Client(C::FishCaught),
    ),
    kind(
        "fish_kinds",
        "Every Fish in the Pond",
        "Land every kind of fish.",
        crate::fishing::FISH_KINDS as u64,
        Toys,
        Client(C::FishKinds),
    ),
    kind(
        "echo_angler",
        "Echo Angler",
        "Land 10 Echoes on the line.",
        10,
        Toys,
        Client(C::EchoesLanded),
    ),
    kind(
        "old_whiskers",
        "Old Whiskers",
        "Land Old Whiskers, the great catfish.",
        1,
        Toys,
        Client(C::GreatCatfish),
    )
    .secret(),
    // Since 29/10/2026 (Sol's Screen Saver, `monitor/saver.rs`).
    kind(
        "monitor_payback",
        "Payback",
        "During your Screen Saver, hit the player whose monitor set it off.",
        1,
        Toys,
        Client(C::MonitorPayback),
    ),
    kind(
        "monitor_surge",
        "Power Surge",
        "Hit 3 different players with monitors during one Screen Saver.",
        1,
        Toys,
        Client(C::MonitorSurge),
    )
    .secret(),
];

/// The bit of the fish `id` in [`Record`]'s kinds of fish: its place among the fish of
/// [`crate::fishing::CATCHES`] (junk has none).
pub(crate) fn fish_kind(id: &str) -> Option<u32> {
    crate::fishing::CATCHES
        .iter()
        .filter(|catch| catch.kind == crate::fishing::Kind::Fish)
        .position(|catch| catch.id == id)
        .map(|index| 1 << index)
}

/// How many achievements can still be earned (every one but the retired).
pub(crate) const OBTAINABLE: usize = {
    let mut count = 0;
    let mut index = 0;
    while index < ALL.len() {
        if !ALL[index].retired {
            count += 1;
        }
        index += 1;
    }
    count
};

/// The achievement called `id`, if the client knows it.
pub(crate) fn find(id: &str) -> Option<&'static Kind> {
    ALL.iter().find(|kind| kind.id == id)
}

impl Kind {
    /// `count` as the board writes it: hours for time played, a plain number otherwise.
    pub(crate) fn amount(&self, count: u64) -> String {
        if self.source == Client(C::Minutes) {
            let hours = count as f64 / 60.0;
            if hours < 10.0 && !count.is_multiple_of(60) {
                format!("{hours:.1} h")
            } else {
                format!("{} h", count / 60)
            }
        } else if self.source == Client(C::BoxMinutes) {
            format!("{count} min")
        } else {
            count.to_string()
        }
    }
}

/// An achievement as a board shows it: its count (at most the goal) and when it was
/// unlocked (unix seconds), if it was.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Standing {
    pub(crate) kind: &'static Kind,
    pub(crate) progress: u64,
    pub(crate) unlocked: Option<i64>,
    /// Reached here, waiting for the hub to unlock it (and grant its crystal): not
    /// unlocked yet, nothing announced.
    pub(crate) waiting: bool,
}

impl Standing {
    /// How far towards the goal, from 0 to 1 (0 for a secret: nothing of it is shown).
    pub(crate) fn fraction(&self) -> f32 {
        if self.secret() {
            return 0.0;
        }
        (self.progress as f32 / self.kind.goal.max(1) as f32).clamp(0.0, 1.0)
    }

    /// Whether it shows as a secret: hidden and not unlocked yet.
    pub(crate) fn secret(&self) -> bool {
        self.kind.hidden && self.unlocked.is_none()
    }

    /// Whether boards list it at all: a retired achievement only once unlocked.
    pub(crate) fn listed(&self) -> bool {
        !self.kind.retired || self.unlocked.is_some()
    }

    /// The name a board shows.
    pub(crate) fn name(&self) -> std::borrow::Cow<'static, str> {
        if self.secret() {
            SECRET_NAME.into()
        } else {
            crate::terms::say(self.kind.name)
        }
    }

    /// What a board says it asks.
    pub(crate) fn description(&self) -> std::borrow::Cow<'static, str> {
        if self.secret() {
            SECRET_DESCRIPTION.into()
        } else {
            crate::terms::say(self.kind.description)
        }
    }
}

/// How many of `standings` are unlocked, of how many are listed: every obtainable
/// achievement and the retired ones held ("12 of 34").
pub(crate) fn tally(standings: &[Standing]) -> (usize, usize) {
    let listed = standings.iter().filter(|standing| standing.listed());
    let total = listed.clone().count();
    let unlocked = listed
        .filter(|standing| standing.unlocked.is_some())
        .count();
    (unlocked, total)
}

/// How many achievements a person's total counts: every obtainable one and the retired
/// ones `holds` says they hold unlocked.
pub(crate) fn total_for(holds: impl Fn(&Kind) -> bool) -> usize {
    OBTAINABLE
        + ALL
            .iter()
            .filter(|kind| kind.retired && holds(kind))
            .count()
}

/// The counts the client keeps, as `achievements.json` holds them.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Record {
    counts: [u64; Counter::COUNT],
    /// Milliseconds played beyond the whole minutes in `counts`.
    play_ms: u64,
    /// Milliseconds in the box beyond the whole minutes in `counts`.
    box_ms: u64,
    /// Maps and servers played, lower case.
    maps: BTreeSet<String>,
    servers: BTreeSet<String>,
    /// Players duelled, by their name without colours, lower case.
    partners: BTreeSet<String>,
    /// Kinds of weapon (bits of [`tracker::weapon_kind`]).
    weapons: u32,
    /// Kinds of fish landed (bits of [`fish_kind`]).
    fish: u32,
    /// What the hub held for the counts kept as sets (maps, servers, weapons), when
    /// that was more: after a reinstall the hub's count stands until the set passes it.
    floors: [u64; Counter::COUNT],
    /// When each achievement the client counts was unlocked here, unix seconds.
    unlocked: BTreeMap<String, i64>,
    /// The achievements the client counts that reached their goal here while the hub
    /// was on, waiting for the hub to unlock them: announced when it does.
    awaiting: BTreeSet<String>,
}

impl Record {
    /// The count `counter` stands at.
    pub(crate) fn count(&self, counter: Counter) -> u64 {
        let raw = match counter {
            Counter::Maps => self.maps.len() as u64,
            Counter::Servers => self.servers.len() as u64,
            Counter::DuelPartners => self.partners.len() as u64,
            Counter::Weapons => u64::from(self.weapons.count_ones()),
            Counter::FishKinds => u64::from(self.fish.count_ones()),
            _ => self.counts[counter as usize],
        };
        raw.max(self.floors[counter as usize])
    }

    fn add(&mut self, counter: Counter, amount: u64) {
        let slot = &mut self.counts[counter as usize];
        *slot = slot.saturating_add(amount);
    }

    fn raise(&mut self, counter: Counter, value: u64) {
        let slot = &mut self.counts[counter as usize];
        *slot = (*slot).max(value);
    }

    /// Add `ms` of play to the minutes played.
    fn play(&mut self, ms: u64) {
        let total = self.play_ms + ms;
        self.add(Counter::Minutes, total / 60_000);
        self.play_ms = total % 60_000;
    }

    /// Add `ms` hidden in the box, with someone close by, to its minutes.
    fn hide(&mut self, ms: u64) {
        let total = self.box_ms + ms;
        self.add(Counter::BoxMinutes, total / 60_000);
        self.box_ms = total % 60_000;
    }

    /// A catch the Fishing toy landed: `catch` (none when the hook came up empty) or an
    /// Echo (`echo`). Whether a count changed.
    pub(crate) fn fished(&mut self, catch: Option<&crate::fishing::Catch>, echo: bool) -> bool {
        if echo {
            self.add(Counter::EchoesLanded, 1);
            return true;
        }
        let Some(catch) = catch else {
            return false;
        };
        match catch.kind {
            crate::fishing::Kind::Junk => false,
            crate::fishing::Kind::Fish => {
                self.add(Counter::FishCaught, 1);
                self.fish |= fish_kind(catch.id).unwrap_or(0);
                if catch.id == crate::fishing::GREAT_CATFISH {
                    self.add(Counter::GreatCatfish, 1);
                }
                true
            }
        }
    }

    /// Remember `name` in `set` (lower case, bounded); whether it was new.
    fn remember(set: &mut BTreeSet<String>, name: &str) -> bool {
        let name = name.trim().to_ascii_lowercase();
        if name.is_empty() || name.len() > NAME_MAX || set.len() >= SET_MAX {
            return false;
        }
        set.insert(name)
    }

    /// Forget achievement `kind` as cleared: its counter goes just below its goal, so
    /// the next count unlocks it again, and every achievement on that counter that the
    /// lower count no longer reaches is unlocked no more. A hub-counted one only loses
    /// its local mark.
    fn forget(&mut self, kind: &Kind) {
        self.unlocked.remove(kind.id);
        self.awaiting.remove(kind.id);
        let Client(counter) = kind.source else {
            return;
        };
        let below = kind.goal.saturating_sub(1);
        let index = counter as usize;
        self.floors[index] = self.floors[index].min(below);
        match counter {
            Counter::Maps | Counter::Servers | Counter::DuelPartners => {
                let set = match counter {
                    Counter::Maps => &mut self.maps,
                    Counter::Servers => &mut self.servers,
                    _ => &mut self.partners,
                };
                while set.len() as u64 > below {
                    set.pop_last();
                }
            }
            Counter::Weapons => {
                while u64::from(self.weapons.count_ones()) > below {
                    self.weapons &= self.weapons - 1;
                }
            }
            Counter::FishKinds => {
                while u64::from(self.fish.count_ones()) > below {
                    self.fish &= self.fish - 1;
                }
            }
            _ => {
                self.counts[index] = self.counts[index].min(below);
                if counter == Counter::Minutes {
                    self.play_ms = 0;
                }
                if counter == Counter::BoxMinutes {
                    self.box_ms = 0;
                }
            }
        }
        for other in &ALL {
            if other.source == kind.source && other.goal > below {
                self.unlocked.remove(other.id);
                self.awaiting.remove(other.id);
            }
        }
    }

    /// The counts of the achievements the client counts, by id, as the hub takes them.
    pub(crate) fn hub_counts(&self) -> BTreeMap<String, u64> {
        ALL.iter()
            .filter_map(|kind| match kind.source {
                Client(counter) => Some((kind.id.to_owned(), self.count(counter))),
                Hub => None,
            })
            .filter(|(_, count)| *count > 0)
            .collect()
    }

    /// Take in what the hub holds: a count above the one kept here (another PC, a
    /// reinstall) raises it, and an unlock it holds is taken. `None` when nothing
    /// changed, else the achievements that were waiting here for the hub
    /// ([`Self::unlock`]) and that it has now unlocked, to announce.
    pub(crate) fn merge_hub(
        &mut self,
        held: &[sjk_identity::Achievement],
    ) -> Option<Vec<&'static Kind>> {
        let before = self.clone();
        let mut confirmed = Vec::new();
        for achievement in held {
            let Some(kind) = find(&achievement.id) else {
                continue;
            };
            let Client(counter) = &kind.source else {
                continue;
            };
            match counter {
                _ if counter.is_set() => {
                    let floor = &mut self.floors[*counter as usize];
                    *floor = (*floor).max(achievement.progress);
                }
                _ => self.raise(*counter, achievement.progress),
            }
            if achievement.unlocked > 0 && !self.unlocked.contains_key(kind.id) {
                self.unlocked
                    .insert(kind.id.to_owned(), achievement.unlocked);
                if self.awaiting.remove(kind.id) {
                    confirmed.push(kind);
                }
            }
        }
        (*self != before).then_some(confirmed)
    }

    /// Mark the achievements the client counts that reached their goal at `now`
    /// (unix seconds); the ones newly unlocked here, to announce. With the `hub` on they
    /// only wait for it ([`Self::merge_hub`] gives them once it unlocks them), since the
    /// hub raises a count at its own pace and grants the crystal when it unlocks; without
    /// one they unlock here, with what was waiting when the hub went off.
    fn unlock(&mut self, now: i64, hub: bool) -> Vec<&'static Kind> {
        let mut new = Vec::new();
        for kind in &ALL {
            let Client(counter) = kind.source else {
                continue;
            };
            if kind.retired
                || self.count(counter) < kind.goal
                || self.unlocked.contains_key(kind.id)
            {
                continue;
            }
            if hub {
                self.awaiting.insert(kind.id.to_owned());
            } else {
                self.awaiting.remove(kind.id);
                self.unlocked.insert(kind.id.to_owned(), now);
                new.push(kind);
            }
        }
        new
    }

    /// Every achievement as a board shows it, the hub's word taken for those it counts
    /// and for any it says are further along.
    pub(crate) fn standings(&self, held: &[sjk_identity::Achievement]) -> Vec<Standing> {
        ALL.iter()
            .map(|kind| {
                let hub = held.iter().find(|held| held.id == kind.id);
                let local = match kind.source {
                    Client(counter) => self.count(counter),
                    Hub => 0,
                };
                let progress = local.max(hub.map_or(0, |hub| hub.progress)).min(kind.goal);
                let confirmed = self
                    .unlocked
                    .get(kind.id)
                    .copied()
                    .or(hub.map(|hub| hub.unlocked).filter(|at| *at > 0));
                let waiting = confirmed.is_none() && self.awaiting.contains(kind.id);
                let unlocked = confirmed.or((progress >= kind.goal && !waiting).then_some(0));
                Standing {
                    kind,
                    progress,
                    unlocked,
                    waiting,
                }
            })
            .collect()
    }

    /// The file's JSON.
    fn to_json(&self) -> Value {
        let counts: serde_json::Map<String, Value> = Counter::ALL
            .iter()
            .filter(|counter| !counter.is_set())
            .map(|counter| {
                (
                    counter.key().to_owned(),
                    json!(self.counts[*counter as usize]),
                )
            })
            .collect();
        let floors: serde_json::Map<String, Value> = Counter::ALL
            .iter()
            .filter(|counter| self.floors[**counter as usize] > 0)
            .map(|counter| {
                (
                    counter.key().to_owned(),
                    json!(self.floors[*counter as usize]),
                )
            })
            .collect();
        json!({
            "version": 1,
            "counts": counts,
            "play_ms": self.play_ms,
            "box_ms": self.box_ms,
            "maps": self.maps,
            "servers": self.servers,
            "partners": self.partners,
            "weapons": self.weapons,
            "fish": self.fish,
            "floors": floors,
            "unlocked": self.unlocked,
            "awaiting": self.awaiting,
        })
    }

    /// The record `json` holds; anything missing or malformed counts as nothing.
    fn from_json(json: &Value) -> Self {
        let mut record = Self::default();
        let number = |value: Option<&Value>| value.and_then(Value::as_u64).unwrap_or(0);
        for counter in Counter::ALL {
            record.counts[counter as usize] = number(json["counts"].get(counter.key()));
            record.floors[counter as usize] = number(json["floors"].get(counter.key()));
        }
        record.play_ms = number(json.get("play_ms")).min(59_999);
        record.box_ms = number(json.get("box_ms")).min(59_999);
        for (field, set) in [
            ("maps", &mut record.maps),
            ("servers", &mut record.servers),
            ("partners", &mut record.partners),
        ] {
            if let Some(names) = json[field].as_array() {
                for name in names.iter().filter_map(Value::as_str) {
                    Self::remember(set, name);
                }
            }
        }
        record.weapons = u32::try_from(number(json.get("weapons"))).unwrap_or(0);
        record.fish = u32::try_from(number(json.get("fish"))).unwrap_or(0);
        if let Some(unlocked) = json["unlocked"].as_object() {
            for (id, at) in unlocked {
                if let (Some(kind), Some(at)) = (find(id), at.as_i64()) {
                    record.unlocked.insert(kind.id.to_owned(), at);
                }
            }
        }
        if let Some(awaiting) = json["awaiting"].as_array() {
            for kind in awaiting.iter().filter_map(Value::as_str).filter_map(find) {
                if !record.unlocked.contains_key(kind.id) {
                    record.awaiting.insert(kind.id.to_owned());
                }
            }
        }
        record
    }
}

/// An unlock to announce.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Announcement {
    pub(crate) kind: &'static Kind,
    /// Whether the hub unlocked it, so it grants the kyber crystal and the card says so.
    /// Without a hub it unlocks here and brings none.
    pub(crate) crystal: bool,
}

/// The shared state: the record, where it is kept, and the unlocks waiting to be told.
#[derive(Default)]
struct Shared {
    record: Record,
    file: Option<PathBuf>,
    dirty: bool,
    saved_at: Option<Instant>,
    /// Counts as the identity service was last given them.
    sent: Option<BTreeMap<String, u64>>,
    /// Unlocked (in a match, or by the hub after waiting here), not yet announced.
    announce: Vec<Announcement>,
    /// Whether the hub is on, as [`sync`] last heard it.
    hub: bool,
}

impl Shared {
    /// After the counts changed at `now`: unlock what reached its goal (or let it wait
    /// for the hub) and queue what unlocked to announce.
    fn counted(&mut self, now: i64) {
        self.dirty = true;
        let new = self.record.unlock(now, self.hub);
        self.announce_all(&new, false);
    }

    /// Queue `kinds` to announce, each once.
    fn announce_all(&mut self, kinds: &[&'static Kind], crystal: bool) {
        for &kind in kinds {
            if !self.announce.iter().any(|told| told.kind.id == kind.id) {
                self.announce.push(Announcement { kind, crystal });
            }
        }
    }

    /// [`sync`]'s turn with the record: take in what the hub holds (`held`), queue what
    /// it unlocked of what waited here, and with the `hub` off unlock here what waited.
    fn follow_hub(&mut self, held: Option<&[sjk_identity::Achievement]>, hub: bool, now: i64) {
        self.hub = hub;
        if let Some(held) = held
            && let Some(confirmed) = self.record.merge_hub(held)
        {
            self.dirty = true;
            self.announce_all(&confirmed, true);
            // Reached on another PC and not unlocked yet: it waits here too.
            let new = self.record.unlock(now, hub);
            self.announce_all(&new, false);
        }
        if !hub && !self.record.awaiting.is_empty() {
            self.dirty = true;
            let new = self.record.unlock(now, false);
            self.announce_all(&new, false);
        }
    }
}

static SHARED: Mutex<Shared> = Mutex::new(Shared {
    record: Record {
        counts: [0; Counter::COUNT],
        play_ms: 0,
        box_ms: 0,
        maps: BTreeSet::new(),
        servers: BTreeSet::new(),
        partners: BTreeSet::new(),
        weapons: 0,
        fish: 0,
        floors: [0; Counter::COUNT],
        unlocked: BTreeMap::new(),
        awaiting: BTreeSet::new(),
    },
    file: None,
    dirty: false,
    saved_at: None,
    sent: None,
    announce: Vec::new(),
    hub: false,
});

fn lock() -> MutexGuard<'static, Shared> {
    SHARED
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_secs() as i64)
}

/// Read `achievements.json` from the settings folder `directory`, once.
pub(crate) fn load(directory: &Path) {
    let mut shared = lock();
    if shared.file.is_some() {
        return;
    }
    let file = directory.join(FILE);
    if let Ok(text) = std::fs::read_to_string(&file) {
        match serde_json::from_str::<Value>(&text) {
            Ok(json) => shared.record = Record::from_json(&json),
            Err(error) => crate::log::progress(format_args!("achievements: {FILE}: {error}")),
        }
    }
    shared.file = Some(file);
}

/// Change the record with `change` (the tracker's counts), then unlock what reached
/// its goal; newly unlocked achievements wait for [`take_announcements`]. With the hub
/// on they wait for the hub instead ([`Record::unlock`]).
pub(crate) fn update(change: impl FnOnce(&mut Record) -> bool) {
    let mut shared = lock();
    if shared.file.is_none() {
        // Not loaded yet: counting now would overwrite the file with less.
        return;
    }
    if !change(&mut shared.record) {
        return;
    }
    shared.counted(unix_now());
}

/// Achievements unlocked since the last call, to announce.
pub(crate) fn take_announcements() -> Vec<Announcement> {
    std::mem::take(&mut lock().announce)
}

/// Twice a second: take in what the hub holds (`hub` whether the identity is on with a
/// hub, [`crate::player_identity::hub_on`]): announce what it unlocked of what waited
/// here, while one unlocked on another PC has nothing to announce here. Then give the
/// identity service the counts when they changed, and write the file when it is due
/// (`force` on the way out).
pub(crate) fn sync(held: Option<&[sjk_identity::Achievement]>, hub: bool, force: bool) {
    let mut shared = lock();
    if shared.file.is_none() {
        return;
    }
    shared.follow_hub(held, hub, unix_now());
    let counts = shared.record.hub_counts();
    if shared.sent.as_ref() != Some(&counts)
        && crate::player_identity::set_achievement_counts(&counts)
    {
        shared.sent = Some(counts);
    }
    let due = shared
        .saved_at
        .is_none_or(|saved| saved.elapsed() >= SAVE_EVERY);
    if shared.dirty && (due || force) {
        shared.dirty = false;
        shared.saved_at = Some(Instant::now());
        if let Some(file) = shared.file.clone() {
            save(&file, &shared.record);
        }
    }
}

/// Write `record` to `file` through a temporary file, so a crash cannot leave half.
fn save(file: &Path, record: &Record) {
    let text = match serde_json::to_string_pretty(&record.to_json()) {
        Ok(text) => text,
        Err(_) => return,
    };
    let temporary = file.with_extension("json.tmp");
    let written = std::fs::write(&temporary, text).and_then(|()| std::fs::rename(&temporary, file));
    if let Err(error) = written {
        crate::log::progress(format_args!("achievements: cannot save {FILE}: {error}"));
    }
}

/// Every achievement as the board shows it, with what the hub holds (`held`).
pub(crate) fn standings(held: &[sjk_identity::Achievement]) -> Vec<Standing> {
    lock().record.standings(held)
}

/// The achievements that clearing `id` takes with it on this client: `id`, and for
/// one the client counts, every one on the same counter with a goal at least as high
/// (their count goes below `id`'s goal). Empty for an unknown id.
pub(crate) fn cleared_with(id: &str) -> Vec<&'static str> {
    let Some(kind) = find(id) else {
        return Vec::new();
    };
    match kind.source {
        Hub => vec![kind.id],
        Client(_) => ALL
            .iter()
            .filter(|other| other.source == kind.source && other.goal >= kind.goal)
            .map(|other| other.id)
            .collect(),
    }
}

/// Forget achievements on this client, after the player's own were cleared at the
/// hub: `Some(id)` as [`Record::forget`] does, `None` every count and unlock.
pub(crate) fn forget(id: Option<&str>) {
    let mut shared = lock();
    if shared.file.is_none() {
        return;
    }
    match id.and_then(find) {
        Some(kind) => shared.record.forget(kind),
        None if id.is_none() => shared.record = Record::default(),
        None => return,
    }
    shared.dirty = true;
}

/// One more of `counter`, a moment a toy saw rather than the snapshots' tracker (the
/// Screen Saver's achievements); the caller keeps to live matches on servers, as the
/// tracker does.
pub(crate) fn note(counter: Counter) {
    update(|record| {
        record.add(counter, 1);
        true
    });
}

/// The record's counts, for the profile's numbers.
pub(crate) fn count(counter: Counter) -> u64 {
    lock().record.count(counter)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fish counts towards Angler and its kind, Old Whiskers towards its own, an Echo
    /// on the line towards Echo Angler; junk and an empty hook count nothing, and the
    /// kinds of fish are kept in the file.
    #[test]
    fn fishing_counts_fish_their_kinds_echoes_and_the_catfish() {
        let catch = |id: &str| {
            crate::fishing::CATCHES
                .iter()
                .find(|catch| catch.id == id)
                .unwrap()
        };
        let mut record = Record::default();
        assert!(!record.fished(None, false));
        assert!(!record.fished(Some(catch("boot")), false));
        assert!(record.fished(Some(catch("silver_darter")), false));
        assert!(record.fished(Some(catch("silver_darter")), false));
        assert!(record.fished(Some(catch(crate::fishing::GREAT_CATFISH)), false));
        assert!(record.fished(None, true));
        assert_eq!(record.count(Counter::FishCaught), 3);
        assert_eq!(record.count(Counter::FishKinds), 2);
        assert_eq!(record.count(Counter::GreatCatfish), 1);
        assert_eq!(record.count(Counter::EchoesLanded), 1);
        assert_eq!(crate::fishing::FISH_KINDS, 12);
        assert_eq!(find("fish_kinds").unwrap().goal, 12);
        assert_eq!(Record::from_json(&record.to_json()), record);
    }

    fn held(id: &str, progress: u64, unlocked: i64) -> sjk_identity::Achievement {
        sjk_identity::Achievement {
            id: id.to_owned(),
            progress,
            goal: find(id).map_or(1, |kind| kind.goal),
            unlocked,
            reward: None,
        }
    }

    #[test]
    fn the_catalogue_has_unique_ids_and_sensible_goals() {
        let mut ids = BTreeSet::new();
        for kind in &ALL {
            assert!(ids.insert(kind.id), "{} twice", kind.id);
            assert!(kind.goal > 0);
            assert!(kind.description.ends_with('.'));
            assert!(
                kind.id
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
            );
            assert_eq!(find(kind.id), Some(kind));
        }
        assert!(find("nope").is_none());
        for category in Category::ALL {
            assert!(ALL.iter().any(|kind| kind.category == category));
        }
    }

    /// The fourteen of 26/10/2026 follow Decorated in the hub's order, the eight the hub
    /// counts are its own, Master Craftsman asks for every card finish, and only Touched
    /// by the Force and Old Whiskers are hidden, only Surveyor retired.
    #[test]
    fn the_new_achievements_follow_the_hubs_catalogue() {
        let ids: Vec<&str> = ALL[21..].iter().map(|kind| kind.id).collect();
        assert_eq!(
            ids,
            [
                "holocrons_10",
                "holocrons_50",
                "holocrons_200",
                "mythical_find",
                "craft_blade",
                "craft_finishes",
                "full_set",
                "order66_kill",
                "box_minutes_10",
                "duel_flawless",
                "duel_close_call",
                "duel_partners_25",
                "saber_throw_far",
                "thermal_return",
                "style_blue_50",
                "style_yellow_50",
                "style_red_50",
                "style_dual_50",
                "style_staff_50",
                "fish_first",
                "fish_50",
                "fish_kinds",
                "echo_angler",
                "old_whiskers",
                "monitor_payback",
                "monitor_surge",
            ]
        );
        assert_eq!(ALL[20].id, "decorated");
        let hub = ALL[21..].iter().filter(|kind| kind.source == Hub).count();
        assert_eq!(hub, 8);
        assert_eq!(find("craft_finishes").unwrap().goal, 3);
        assert_eq!(
            find("craft_finishes").unwrap().goal,
            crate::cards::FINISHES.len() as u64
        );
        let hidden: Vec<&str> = ALL.iter().filter(|k| k.hidden).map(|k| k.id).collect();
        assert_eq!(hidden, ["mythical_find", "old_whiskers", "monitor_surge"]);
        let retired: Vec<&str> = ALL.iter().filter(|k| k.retired).map(|k| k.id).collect();
        assert_eq!(retired, ["surveyor"]);
        assert_eq!(OBTAINABLE, ALL.len() - 1);
        assert_eq!(find("box_minutes_10").unwrap().amount(7), "7 min");
        assert_eq!(find("holocrons_200").unwrap().category, Collection);
        assert_eq!(find("order66_kill").unwrap().category, Toys);
    }

    /// A hidden achievement is a secret until unlocked: its name and what it asks are
    /// kept, and nothing of its count shows. Once unlocked it shows as any other.
    #[test]
    fn a_hidden_achievement_is_a_secret_until_unlocked() {
        let record = Record::default();
        let standings = record.standings(&[held("mythical_find", 0, 0)]);
        let secret = standings
            .iter()
            .find(|s| s.kind.id == "mythical_find")
            .unwrap();
        assert!(secret.secret() && secret.listed());
        assert_eq!(secret.name(), SECRET_NAME);
        assert_eq!(secret.description(), SECRET_DESCRIPTION);
        assert_eq!(secret.fraction(), 0.0);
        let standings = record.standings(&[held("mythical_find", 1, 1_791_336_225)]);
        let found = standings
            .iter()
            .find(|s| s.kind.id == "mythical_find")
            .unwrap();
        assert!(!found.secret());
        assert_eq!(found.name(), "Touched by the Force");
        assert_eq!(found.description(), "Find a Mythical in a holocron.");
        assert_eq!(found.fraction(), 1.0);
    }

    /// A retired achievement is listed and counted only for those who hold it, and its
    /// unfinished progress never shows.
    #[test]
    fn a_retired_achievement_shows_only_to_its_holders() {
        let record = Record::default();
        let none = record.standings(&[]);
        assert_eq!(tally(&none), (0, ALL.len() - 1));
        let partway = record.standings(&[held("surveyor", 3, 0)]);
        let surveyor = partway.iter().find(|s| s.kind.id == "surveyor").unwrap();
        assert!(!surveyor.listed());
        assert_eq!(tally(&partway), (0, ALL.len() - 1));
        let holder = record.standings(&[held("surveyor", 5, 1_791_336_225)]);
        let surveyor = holder.iter().find(|s| s.kind.id == "surveyor").unwrap();
        assert!(surveyor.listed() && !surveyor.secret());
        assert_eq!(tally(&holder), (1, ALL.len()));
        assert_eq!(total_for(|_| false), ALL.len() - 1);
        assert_eq!(total_for(|kind| kind.id == "surveyor"), ALL.len());
    }

    /// Duel partners are a set like maps: the hub's count is a floor after a reinstall,
    /// clearing one keeps the set just below its goal, and the file keeps them, with the
    /// time in the box beyond its whole minutes.
    #[test]
    fn duel_partners_and_box_time_are_kept_restored_and_cleared() {
        let mut record = Record::default();
        for name in ["sol", "bishop", "creyon"] {
            assert!(Record::remember(&mut record.partners, name));
        }
        assert!(!Record::remember(&mut record.partners, "sol"));
        assert_eq!(record.count(Counter::DuelPartners), 3);
        record.hide(61_500);
        assert_eq!(
            (record.count(Counter::BoxMinutes), record.box_ms),
            (1, 1_500)
        );
        record.add(Counter::DuelFlawless, 1);
        let back = Record::from_json(&record.to_json());
        assert_eq!(back, record);
        assert!(
            record
                .merge_hub(&[
                    held("duel_partners_25", 20, 0),
                    held("box_minutes_10", 4, 0)
                ])
                .is_some()
        );
        assert_eq!(record.count(Counter::DuelPartners), 20);
        assert_eq!(record.count(Counter::BoxMinutes), 4);
        assert_eq!(record.partners.len(), 3, "the floor, not names");
        let new: Vec<&str> = record.unlock(9, false).iter().map(|kind| kind.id).collect();
        assert_eq!(new, ["duel_flawless"]);
        for index in 0..10 {
            Record::remember(&mut record.partners, &format!("p{index}"));
        }
        record.floors[Counter::DuelPartners as usize] = 30;
        let _ = record.unlock(10, false);
        assert!(record.unlocked.contains_key("duel_partners_25"));
        record.forget(find("duel_partners_25").unwrap());
        assert_eq!(
            record.count(Counter::DuelPartners),
            24,
            "just below the goal"
        );
        assert_eq!(record.partners.len(), 13, "the set itself was below");
        record.forget(find("box_minutes_10").unwrap());
        assert_eq!((record.count(Counter::BoxMinutes), record.box_ms), (4, 0));
        let counts = record.hub_counts();
        assert_eq!(counts["duel_partners_25"], 24);
        assert_eq!(counts["duel_flawless"], 1);
    }

    #[test]
    fn counts_unlock_once_and_go_to_the_hub_by_id() {
        let mut record = Record::default();
        record.add(Counter::Kills, 1);
        let new: Vec<&str> = record
            .unlock(100, false)
            .iter()
            .map(|kind| kind.id)
            .collect();
        assert_eq!(new, ["first_blood"]);
        assert!(record.unlock(200, false).is_empty());
        record.add(Counter::Kills, 99);
        assert_eq!(
            record
                .unlock(300, false)
                .iter()
                .map(|k| k.id)
                .collect::<Vec<_>>(),
            ["kills_100"]
        );
        let counts = record.hub_counts();
        assert_eq!(counts["first_blood"], 100);
        assert_eq!(counts["kills_100"], 100);
        assert_eq!(counts["kills_1000"], 100);
        assert!(
            !counts.contains_key("storyteller"),
            "the hub counts its own"
        );
        assert!(!counts.contains_key("captures_10"), "nothing to send at 0");
    }

    /// The ids of what waits to be announced, with whether each brings its crystal.
    fn told(shared: &mut Shared) -> Vec<(&'static str, bool)> {
        std::mem::take(&mut shared.announce)
            .into_iter()
            .map(|told| (told.kind.id, told.crystal))
            .collect()
    }

    fn standing(record: &Record, held: &[sjk_identity::Achievement], id: &str) -> Standing {
        *record
            .standings(held)
            .iter()
            .find(|standing| standing.kind.id == id)
            .unwrap()
    }

    /// With the hub on, an achievement the client counts that reaches its goal waits for
    /// the hub (shown reached, "Waiting for the SJK hub", nothing announced) and is
    /// announced with its crystal once, when the hub holds it unlocked; one the hub
    /// unlocked that never waited here (another PC) is not announced.
    #[test]
    fn a_client_counted_achievement_waits_for_the_hub_before_it_is_announced() {
        let mut shared = Shared::default();
        shared.follow_hub(Some(&[]), true, 1);
        shared.record.add(Counter::Kills, 1);
        shared.counted(2);
        assert!(told(&mut shared).is_empty(), "not unlocked by the hub yet");
        let waiting = standing(&shared.record, &[], "first_blood");
        assert!(waiting.waiting);
        assert_eq!(waiting.unlocked, None);
        assert_eq!(waiting.fraction(), 1.0, "reached");
        // The hub took the count but has not unlocked it yet (its hourly allowance).
        let pending = [held("first_blood", 1, 0)];
        shared.follow_hub(Some(&pending), true, 3);
        assert!(told(&mut shared).is_empty());
        assert!(standing(&shared.record, &pending, "first_blood").waiting);
        // The hub unlocked it: announced once, with its crystal.
        let confirmed = [held("first_blood", 1, 500), held("kills_100", 100, 600)];
        shared.follow_hub(Some(&confirmed), true, 4);
        assert_eq!(told(&mut shared), [("first_blood", true)]);
        let done = standing(&shared.record, &confirmed, "first_blood");
        assert!(!done.waiting);
        assert_eq!(done.unlocked, Some(500));
        assert_eq!(
            standing(&shared.record, &confirmed, "kills_100").unlocked,
            Some(600),
            "unlocked on another PC: nothing to announce here"
        );
        shared.follow_hub(Some(&confirmed), true, 5);
        shared.record.add(Counter::Kills, 1);
        shared.counted(6);
        assert!(told(&mut shared).is_empty(), "never twice");
        assert!(shared.record.awaiting.is_empty());
    }

    /// What waits for the hub is kept in the file: when the hub unlocked it while the
    /// game was closed, the next sync announces it, once.
    #[test]
    fn a_hub_unlock_while_the_game_was_closed_is_announced_at_the_next_sync() {
        let mut record = Record::default();
        record.add(Counter::DuelWins, 10);
        assert!(record.unlock(1, true).is_empty());
        let back = Record::from_json(&record.to_json());
        assert_eq!(back, record);
        assert!(back.awaiting.contains("duel_wins_10"));
        let mut shared = Shared {
            record: back,
            ..Shared::default()
        };
        let held = [held("duel_wins_10", 10, 900)];
        shared.follow_hub(Some(&held), true, 2);
        assert_eq!(told(&mut shared), [("duel_wins_10", true)]);
        shared.follow_hub(Some(&held), true, 3);
        assert!(told(&mut shared).is_empty());
        let kept = Record::from_json(&shared.record.to_json());
        assert!(kept.awaiting.is_empty());
        assert_eq!(kept.unlocked["duel_wins_10"], 900);
    }

    /// With the identity off there is no hub to wait for: the achievement unlocks here
    /// and is announced at once, without a crystal; what was waiting for the hub when
    /// the identity went off unlocks the same way.
    #[test]
    fn with_the_identity_off_an_achievement_unlocks_here_without_a_crystal() {
        let mut shared = Shared::default();
        shared.follow_hub(None, false, 1);
        shared.record.add(Counter::Kills, 1);
        shared.counted(2);
        assert_eq!(told(&mut shared), [("first_blood", false)]);
        let open = standing(&shared.record, &[], "first_blood");
        assert_eq!((open.unlocked, open.waiting), (Some(2), false));
        shared.counted(3);
        assert!(told(&mut shared).is_empty(), "once");
        // Reached with the hub on, then the identity turned off.
        shared.follow_hub(Some(&[]), true, 4);
        shared.record.add(Counter::Captures, 10);
        shared.counted(5);
        assert!(told(&mut shared).is_empty());
        shared.follow_hub(None, false, 6);
        assert_eq!(told(&mut shared), [("captures_10", false)]);
        assert!(shared.record.awaiting.is_empty());
    }

    #[test]
    fn time_played_counts_whole_minutes() {
        let mut record = Record::default();
        record.play(59_000);
        assert_eq!(record.count(Counter::Minutes), 0);
        record.play(2_000);
        assert_eq!(record.count(Counter::Minutes), 1);
        assert_eq!(record.play_ms, 1_000);
        record.play(600_000);
        assert_eq!(record.count(Counter::Minutes), 11);
        let hours = find("hours_10").unwrap();
        assert_eq!(hours.amount(90), "1.5 h");
        assert_eq!(hours.amount(600), "10 h");
        assert_eq!(find("kills_100").unwrap().amount(37), "37");
    }

    #[test]
    fn maps_and_servers_are_counted_once_each_and_bounded() {
        let mut record = Record::default();
        assert!(Record::remember(&mut record.maps, "mp/FFA1"));
        assert!(!Record::remember(&mut record.maps, "mp/ffa1"));
        assert!(!Record::remember(&mut record.maps, "  "));
        assert!(!Record::remember(
            &mut record.maps,
            &"x".repeat(NAME_MAX + 1)
        ));
        assert_eq!(record.count(Counter::Maps), 1);
        for index in 0..SET_MAX + 10 {
            Record::remember(&mut record.servers, &format!("1.2.3.4:{index}"));
        }
        assert_eq!(record.count(Counter::Servers), SET_MAX as u64);
    }

    #[test]
    fn the_hub_restores_counts_after_a_reinstall() {
        let mut record = Record::default();
        assert!(
            record
                .merge_hub(&[
                    held("kills_100", 40, 0),
                    held("first_blood", 1, 1_000),
                    held("maps_10", 7, 0),
                    held("storyteller", 1, 5),
                    held("unknown", 9, 9),
                ])
                .is_some()
        );
        assert_eq!(record.count(Counter::Kills), 40);
        assert_eq!(record.count(Counter::Maps), 7);
        assert_eq!(record.unlocked["first_blood"], 1_000);
        // A new map does not count until the set passes the hub's count.
        Record::remember(&mut record.maps, "mp/ffa1");
        assert_eq!(record.count(Counter::Maps), 7);
        assert!(
            record.merge_hub(&[held("kills_100", 12, 0)]).is_none(),
            "lower changes nothing"
        );
    }

    #[test]
    fn standings_show_the_furthest_count_and_the_hubs_own() {
        let mut record = Record::default();
        record.add(Counter::Kills, 3);
        let _ = record.unlock(50, false);
        let standings = record.standings(&[held("kills_100", 10, 0), held("storyteller", 1, 77)]);
        let of = |id: &str| *standings.iter().find(|s| s.kind.id == id).unwrap();
        assert_eq!(of("kills_100").progress, 10);
        assert_eq!(of("kills_100").unlocked, None);
        assert_eq!(of("first_blood").unlocked, Some(50));
        assert_eq!(of("first_blood").progress, 1, "capped at the goal");
        assert_eq!(of("storyteller").unlocked, Some(77));
        assert_eq!(of("surveyor").progress, 0);
        assert!((of("kills_100").fraction() - 0.1).abs() < 1e-6);
        assert_eq!(standings.len(), ALL.len());
    }

    #[test]
    fn forgetting_sets_the_count_just_below_the_goal() {
        let mut record = Record::default();
        record.add(Counter::Kills, 150);
        for name in ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"] {
            Record::remember(&mut record.maps, name);
        }
        record.weapons = 0b1111_1111;
        record.play(700 * 60_000 + 5);
        let _ = record.unlock(1, false);
        assert!(record.unlocked.contains_key("kills_100"));
        record.forget(find("kills_100").unwrap());
        assert_eq!(record.count(Counter::Kills), 99);
        assert!(!record.unlocked.contains_key("kills_100"));
        assert!(record.unlocked.contains_key("first_blood"), "still reached");
        // The next count unlocks it again.
        record.add(Counter::Kills, 1);
        assert_eq!(
            record
                .unlock(2, false)
                .iter()
                .map(|k| k.id)
                .collect::<Vec<_>>(),
            ["kills_100"]
        );
        record.forget(find("first_blood").unwrap());
        assert_eq!(record.count(Counter::Kills), 0);
        assert!(
            !record.unlocked.contains_key("kills_100"),
            "no longer reached"
        );
        record.forget(find("maps_10").unwrap());
        assert_eq!(record.count(Counter::Maps), 9);
        record.forget(find("arsenal").unwrap());
        assert_eq!(record.count(Counter::Weapons), 7);
        record.forget(find("hours_10").unwrap());
        assert_eq!((record.count(Counter::Minutes), record.play_ms), (599, 0));
        record.unlocked.insert("storyteller".into(), 5);
        record.forget(find("storyteller").unwrap());
        assert!(!record.unlocked.contains_key("storyteller"));
    }

    #[test]
    fn clearing_takes_the_higher_goals_on_the_same_counter() {
        assert_eq!(
            cleared_with("first_blood"),
            ["first_blood", "kills_100", "kills_1000"]
        );
        assert_eq!(cleared_with("kills_100"), ["kills_100", "kills_1000"]);
        assert_eq!(cleared_with("saber_kills_100"), ["saber_kills_100"]);
        assert_eq!(cleared_with("maps_10"), ["maps_10", "maps_25"]);
        assert_eq!(cleared_with("storyteller"), ["storyteller"]);
        assert!(cleared_with("nope").is_empty());
    }

    #[test]
    fn the_file_round_trips_and_survives_damage() {
        let mut record = Record::default();
        record.add(Counter::Kills, 12);
        record.raise(Counter::BestStreak, 4);
        record.play(90_500);
        Record::remember(&mut record.maps, "mp/duel6");
        Record::remember(&mut record.servers, "1.2.3.4:29070");
        record.weapons = 0b101;
        record.floors[Counter::Maps as usize] = 3;
        let _ = record.unlock(42, false);
        let back = Record::from_json(&record.to_json());
        assert_eq!(back, record);
        // Wrong types and unknown ids count as nothing.
        let damaged = json!({
            "counts": {"kills": "many", "duel_wins": 3},
            "maps": [1, "mp/ffa3"],
            "unlocked": {"nope": 5, "first_blood": "x"},
            "play_ms": 999_999_999,
        });
        let read = Record::from_json(&damaged);
        assert_eq!(read.count(Counter::Kills), 0);
        assert_eq!(read.count(Counter::DuelWins), 3);
        assert_eq!(read.count(Counter::Maps), 1);
        assert!(read.unlocked.is_empty());
        assert!(read.play_ms < 60_000);
        assert_eq!(Record::from_json(&Value::Null), Record::default());
    }
}
