//! What a claim's answer may add for SJK's Fishing and Mining toys (hub `PROTOCOL.md`,
//! "Holocrons" Kyber finds and "Quests" Daily catch quests, since 15/11/2026, adwè's
//! idea in #ideas): a kyber find the hub rolled on that claim (`"kyber_find"`) and the
//! day's catch quests (`"catch_quests"`). Both are the hub's own counts and rolls, never
//! the client's; an older hub sends neither, and the client then shows nothing of them.

use serde::Deserialize;

/// A few kyber shards the hub found for the player on a claim with a hooked bite or a
/// broken boulder (`"kyber_find"`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct KyberFind {
    /// `fishing` or `mining`.
    #[serde(default)]
    pub toy: String,
    /// The shards found.
    pub shards: u32,
    /// The player's shards after.
    #[serde(default)]
    pub balance: i64,
}

impl KyberFind {
    /// Whether it came from the Fishing toy (else Mining's).
    pub fn fishing(&self) -> bool {
        self.toy == "fishing"
    }
}

/// The day's two catch quests as the hub counts them (`"catch_quests"`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct CatchQuests {
    /// Bites hooked today, as the hub counted them.
    #[serde(default)]
    pub bites: u32,
    /// Bites the fishing goal wants.
    #[serde(default)]
    pub bites_goal: u32,
    /// Whether the fishing goal's Echo was won today.
    #[serde(default)]
    pub fishing_won: bool,
    /// Boulders broken today, as the hub counted them.
    #[serde(default)]
    pub rocks: u32,
    /// Boulders the mining goal wants.
    #[serde(default)]
    pub rocks_goal: u32,
    /// Whether the mining goal's Echo was won today.
    #[serde(default)]
    pub mining_won: bool,
    /// When the day ends, unix seconds.
    #[serde(default)]
    pub until: i64,
    /// The goal this claim won (`fishing`, `mining`), whose Echo comes as any holocron.
    #[serde(default)]
    pub won: Option<String>,
}

impl CatchQuests {
    /// The fishing goal's line: "Goal: 12/25 bites for an Echo", or that it was won.
    pub fn fishing_line(&self) -> String {
        line("bites", self.bites, self.bites_goal, self.fishing_won)
    }

    /// The mining goal's line: "Goal: 8/40 boulders for an Echo", or that it was won.
    pub fn mining_line(&self) -> String {
        line("boulders", self.rocks, self.rocks_goal, self.mining_won)
    }
}

fn line(what: &str, count: u32, goal: u32, won: bool) -> String {
    if won {
        format!("Goal done: {goal} {what}, Echo won")
    } else {
        format!("Goal: {}/{goal} {what} for an Echo", count.min(goal))
    }
}

/// What a claim's answer brought beside its `ttl`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct ClaimExtras {
    /// A kyber find rolled on the claim.
    #[serde(default)]
    pub kyber_find: Option<KyberFind>,
    /// The day's catch quests after the claim.
    #[serde(default)]
    pub catch_quests: Option<CatchQuests>,
}

impl ClaimExtras {
    /// The extras of a claim's answer `value`; none from an answer that holds none, or
    /// that does not read (the claim itself was accepted either way).
    pub fn of(value: &serde_json::Value) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_claims_answer_reads_as_the_protocol_writes_it() {
        let plain = ClaimExtras::of(&serde_json::json!({"ttl": 90}));
        assert_eq!(plain, ClaimExtras::default());
        let extras = ClaimExtras::of(&serde_json::json!({
            "ttl": 90,
            "kyber_find": {"toy": "fishing", "shards": 3, "balance": 120},
            "catch_quests": {"bites": 12, "bites_goal": 25, "fishing_won": false,
                "rocks": 0, "rocks_goal": 40, "mining_won": false, "until": 1_800_000_000,
                "won": "mining"}
        }));
        let find = extras.kyber_find.unwrap();
        assert!(find.fishing());
        assert_eq!((find.shards, find.balance), (3, 120));
        let day = extras.catch_quests.unwrap();
        assert_eq!(day.won.as_deref(), Some("mining"));
        assert_eq!(day.fishing_line(), "Goal: 12/25 bites for an Echo");
        let done = CatchQuests {
            rocks_goal: 40,
            mining_won: true,
            ..CatchQuests::default()
        };
        assert_eq!(done.mining_line(), "Goal done: 40 boulders, Echo won");
        // A malformed extra loses the extras, never the claim.
        let bad = ClaimExtras::of(&serde_json::json!({"ttl": 90, "kyber_find": 7}));
        assert_eq!(bad, ClaimExtras::default());
    }
}
