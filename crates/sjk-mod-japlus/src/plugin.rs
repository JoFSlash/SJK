//! JA+'s plugin options: `japlus.plugin`, JoF EternalJK's `plugin` on a JA+ server,
//! and the switches Settings shows under JA+ plugin options (`docs/mods.md`).
//!
//! The table is EternalJK's JA+ list (`japlusPluginDisables[]`,
//! `codemp/cgame/cg_consolecmds.c`): bits 0 to 14, the JA+ 1.4B4 plugin's, then
//! bit 29, EternalJK's own. Each entry's polarity comes from where JoF EternalJK
//! (`S:\Git\Not mine\JoF_EJK`, `ebd2a782`) reads the bit; where nothing reads it, from
//! the bit's name, and the evidence says so. Options SJK cannot honour are kept,
//! hidden, with the reason.

use sjk_mod::plugin_options::{MOVES_YOU, Offer, PluginOption, PluginOptions, Polarity};

/// JA+'s options in EternalJK's `plugin` order.
pub const OPTIONS: PluginOptions = PluginOptions {
    family: "JA+",
    command: "japlus.plugin",
    options: &[
        PluginOption {
            bit: 0,
            name: "New drain effect",
            help: "Drain's wide arc drawn as jaPRO draws it.",
            polarity: Polarity::SetTurnsOn,
            offer: Offer::Hidden("SJK draws the stock Drain only"),
            evidence: "JAPRO_PLUGIN_NEWDRAINEFX: cg_players.c:13121 plays jaPRO's wide Drain \
                       while the bit is set",
            row: "japlus.plugin 0",
        },
        PluginOption {
            bit: 1,
            name: "See others in duels",
            help: "In a private duel, keeps showing the players outside it, which JA+ leaves \
                   the client to hide (Hide others in duel on Auto).",
            polarity: Polarity::SetTurnsOn,
            offer: Offer::Offered,
            evidence: "JAPRO_PLUGIN_DUELSEEOTHERS: cg_players.c:12018 and cg_ents.c:933 hide \
                       bystanders during a duel unless the bit is set; SJK: duel_hide.rs",
            row: "japlus.plugin 1",
        },
        PluginOption {
            bit: 2,
            name: "End-of-duel camera",
            help: "The camera turns around the winner when a duel ends.",
            polarity: Polarity::SetTurnsOff,
            offer: Offer::Hidden("it is the JA+ plugin's own camera, which SJK does not have"),
            evidence: "JAPRO_PLUGIN_ENDDUELROTATION: nothing in EternalJK reads it; a set bit \
                       taken to switch it off, as the mask's other JA+ bits do (unverified)",
            row: "japlus.plugin 2",
        },
        PluginOption {
            bit: 3,
            name: "Black sabers",
            help: "Shows blades set to black as black; off, they show orange. JA+ black \
                   sabers must be on too.",
            polarity: Polarity::SetTurnsOff,
            offer: Offer::Offered,
            evidence: "JAPRO_PLUGIN_BLACKSABERSDISABLE: cg_players.c:6525 drops black while \
                       the bit is set; SJK: black_sabers.rs",
            row: "japlus.plugin 3",
        },
        PluginOption {
            bit: 4,
            name: "Auto replier",
            help: "The JA+ plugin's automatic replies.",
            polarity: Polarity::SetTurnsOff,
            offer: Offer::Hidden("it is the JA+ plugin's own, which SJK does not have"),
            evidence: "JAPRO_PLUGIN_AUTOREPLYDISABLE (by its name; the server's \
                       jp_noAutoReplier speaks of the plugin's integrated AutoReplier); nothing \
                       in EternalJK reads it",
            row: "japlus.plugin 4",
        },
        PluginOption {
            bit: 5,
            name: "New Force effects",
            help: "Puffs on Rage, Heal, Quick Heal and Grip victims, and cyan when Protect \
                   and Absorb are both up.",
            polarity: Polarity::SetTurnsOff,
            offer: Offer::Offered,
            evidence: "JAPRO_PLUGIN_NEWFORCEEFFECT: cg_players.c:6226, 6293, 13254 and 14899 \
                       draw them while the bit is clear; SJK: force_fx_policy.rs",
            row: "japlus.plugin 5",
        },
        PluginOption {
            bit: 6,
            name: "New death messages",
            help: "JA+'s new death messages.",
            polarity: Polarity::SetTurnsOff,
            offer: Offer::Hidden(
                "neither SJK nor JoF EJK reads it, so what it changes is unverified",
            ),
            evidence: "JAPRO_PLUGIN_NEWDEATHMSG_DISABLE (by its name); nothing in EternalJK \
                       reads it",
            row: "japlus.plugin 6",
        },
        PluginOption {
            bit: 7,
            name: "Force Sense screen effect",
            help: "A purple ring over the screen while your Force Sense is up.",
            polarity: Polarity::SetTurnsOn,
            offer: Offer::Hidden("SJK does not draw the Sense ring over the screen yet"),
            evidence: "JAPRO_PLUGIN_NEWSIGHTEFFECT: cg_draw.c:10755 draws gfx/2d/jsense only \
                       while the bit is set",
            row: "japlus.plugin 7",
        },
        PluginOption {
            bit: 8,
            name: "Alternate dimension effect",
            help: "The look of players in JA+'s other dimension.",
            polarity: Polarity::SetTurnsOff,
            offer: Offer::Hidden("it is the JA+ plugin's own look, which SJK does not draw"),
            evidence: "JAPRO_PLUGIN_NOALTDIMEFFECT (by its name); nothing in EternalJK reads it",
            row: "japlus.plugin 8",
        },
        PluginOption {
            bit: 9,
            name: "Staff on the back",
            help: "A holstered staff goes across the back, and the server plays your reach \
                   over the shoulder; off keeps it at the hip.",
            polarity: Polarity::SetTurnsOff,
            offer: Offer::Offered,
            evidence: "JAPRO_PLUGIN_HOLSTEREDSABER, a disable bit despite its name: \
                       cg_players.c:11288 puts a staff on the back while the bit is clear, and \
                       a JA+ server then plays the reach (cg_xcvar.h:236); SJK: \
                       holstered_sabers.rs",
            row: "japlus.plugin 9",
        },
        PluginOption {
            bit: 10,
            name: "Ledge grab",
            help: "Grab and pull yourself up the ledges you jump at, where the server allows it.",
            polarity: Polarity::SetTurnsOff,
            offer: Offer::Hidden(MOVES_YOU),
            evidence: "JAPRO_PLUGIN_LEDGEGRAB: bg_pmove.c:73 allows it while the bit is clear",
            row: "japlus.plugin 10",
        },
        PluginOption {
            bit: 11,
            name: "New DFA (attack)",
            help: "JA+'s new death from above, with the attack button.",
            polarity: Polarity::SetTurnsOff,
            offer: Offer::Hidden(MOVES_YOU),
            evidence: "JAPRO_PLUGIN_NEWDFAPRIM, EternalJK's \"Disable New DFA Primary\"; a \
                       server saber move EternalJK's own code does not read",
            row: "japlus.plugin 11",
        },
        PluginOption {
            bit: 12,
            name: "New DFA (alt attack)",
            help: "JA+'s new death from above, with the alternate attack button.",
            polarity: Polarity::SetTurnsOff,
            offer: Offer::Hidden(MOVES_YOU),
            evidence: "JAPRO_PLUGIN_NEWDFAALT, EternalJK's \"Disable New DFA Alt\"; a server \
                       saber move EternalJK's own code does not read",
            row: "japlus.plugin 12",
        },
        PluginOption {
            bit: 13,
            name: "Single-player cartwheel",
            help: "The single player game's cartwheel.",
            polarity: Polarity::SetTurnsOff,
            offer: Offer::Hidden(MOVES_YOU),
            evidence: "JAPRO_PLUGIN_NOSPCARTWHEEL (by its name), a server move EternalJK's \
                       own code does not read",
            row: "japlus.plugin 13",
        },
        PluginOption {
            bit: 14,
            name: "Download redirect",
            help: "Downloads a server's missing files from its web address.",
            polarity: Polarity::SetTurnsOff,
            offer: Offer::Hidden(
                "SJK downloads from the game server only and does not follow a download address",
            ),
            evidence: "JAPRO_PLUGIN_ALLOWLIBCURL, EternalJK's \"No Auto DL Redirect\": the \
                       name and the label disagree, so the polarity is unverified",
            row: "japlus.plugin 14",
        },
        PluginOption {
            bit: 29,
            name: "Holstered sabers",
            help: "Shows the hilts players carry on the hip or back; Holstered saber hilts \
                   must be on too.",
            polarity: Polarity::SetTurnsOff,
            offer: Offer::Offered,
            evidence: "JAPRO_PLUGIN_HOLSTEREDSABERS, EternalJK's own: cg_players.c:10828 and \
                       11209 hide every holstered hilt while the bit is set; SJK: \
                       holstered_sabers.rs",
            row: "japlus.plugin 15",
        },
    ],
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The core client's default, 1536: bits 9 and 10.
    const DEFAULT: u32 = 1_536;

    fn named(name: &str) -> PluginOption {
        *OPTIONS
            .options
            .iter()
            .find(|option| option.name == name)
            .unwrap()
    }

    #[test]
    fn the_bits_are_ejk_s_ja_plus_list_in_its_order() {
        let bits: Vec<u8> = OPTIONS.options.iter().map(|option| option.bit).collect();
        let mut expected: Vec<u8> = (0..=14).collect();
        expected.push(29);
        assert_eq!(bits, expected);
        for (number, option) in OPTIONS.options.iter().enumerate() {
            assert_eq!(option.row, format!("japlus.plugin {number}"));
        }
    }

    #[test]
    fn offered_options_and_their_state_under_the_default() {
        let offered: Vec<(&str, bool)> = OPTIONS
            .options
            .iter()
            .filter(|option| option.offered())
            .map(|option| (option.name, option.on(DEFAULT)))
            .collect();
        assert_eq!(
            offered,
            [
                ("See others in duels", false),
                ("Black sabers", true),
                ("New Force effects", true),
                ("Staff on the back", false),
                ("Holstered sabers", true),
            ]
        );
    }

    /// Each option's state against its bit, as the reference reads it.
    #[test]
    fn the_polarity_follows_the_reference() {
        for (name, bit, on_when_set) in [
            ("New drain effect", 0, true),
            ("See others in duels", 1, true),
            ("Black sabers", 3, false),
            ("New Force effects", 5, false),
            ("Force Sense screen effect", 7, true),
            ("Staff on the back", 9, false),
            ("Ledge grab", 10, false),
            ("Holstered sabers", 29, false),
        ] {
            let option = named(name);
            assert_eq!(option.bit, bit, "{name}");
            assert_eq!(option.on(1 << bit), on_when_set, "{name}");
            assert_eq!(option.on(0), !on_when_set, "{name}");
        }
    }

    #[test]
    fn movement_bits_stay_hidden() {
        for name in [
            "Ledge grab",
            "New DFA (attack)",
            "New DFA (alt attack)",
            "Single-player cartwheel",
        ] {
            assert_eq!(named(name).offer, Offer::Hidden(MOVES_YOU), "{name}");
        }
    }

    #[test]
    fn the_listing_numbers_options_as_ejk_does() {
        let lines = OPTIONS.listing(DEFAULT);
        assert_eq!(
            lines[1..6],
            [
                "^5 1 ^7See others in duels: ^1Off",
                "^5 3 ^7Black sabers: ^2On",
                "^5 5 ^7New Force effects: ^2On",
                "^5 9 ^7Staff on the back: ^1Off",
                "^515 ^7Holstered sabers: ^2On",
            ]
        );
        assert!(lines[6].contains("10 Ledge grab"));
        // EJK's `plugin 15` is bit 29 on JA+.
        assert_eq!(
            OPTIONS.switch(DEFAULT, "15"),
            Ok((DEFAULT | 1 << 29, "^5Holstered sabers^7: ^1Off".to_owned()))
        );
        // Turning ledge grab on is refused, the default kept.
        assert!(OPTIONS.switch(DEFAULT, "10").is_err());
    }
}
