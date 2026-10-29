//! jaPRO's plugin options: `japro.plugin`, JoF EternalJK's `plugin` on a jaPRO server,
//! and the switches Settings shows under jaPRO plugin options (`docs/mods.md`).
//!
//! The table is EternalJK's jaPRO list (`japroPluginDisables[]`,
//! `codemp/cgame/cg_consolecmds.c`): bit 3 and bits 19 to 28, jaPRO's own. Each
//! entry's polarity comes from where JoF EternalJK (`S:\Git\Not mine\JoF_EJK`,
//! `ebd2a782`) reads the bit: its cgame, and jaPRO's server code it carries
//! (`g_client.c:2308-2370` turns the bits into the player's `pers` flags). Options SJK
//! cannot honour are kept, hidden, with the reason. Bits 15 to 18 (ja++'s no kata,
//! butterfly, stab and DFA) are on neither of EternalJK's lists and are left out.

use sjk_mod::plugin_options::{MOVES_YOU, Offer, PluginOption, PluginOptions, Polarity};

/// jaPRO's options in EternalJK's `plugin` order.
pub const OPTIONS: PluginOptions = PluginOptions {
    family: "jaPRO",
    command: "japro.plugin",
    options: &[
        PluginOption {
            bit: 3,
            name: "Black sabers",
            help: "Shows blades set to black as black; off, they show orange. JA+ black \
                   sabers must be on too.",
            polarity: Polarity::SetTurnsOff,
            offer: Offer::Offered,
            evidence: "JAPRO_PLUGIN_BLACKSABERSDISABLE: cg_players.c:6525 drops black while \
                       the bit is set; SJK: black_sabers.rs",
            row: "japro.plugin 0",
        },
        PluginOption {
            bit: 19,
            name: "Force jumps",
            help: "Holding jump makes a Force jump; off, it bunny hops instead, where the \
                   server lets players choose.",
            polarity: Polarity::SetTurnsOff,
            offer: Offer::Hidden(MOVES_YOU),
            evidence: "JAPRO_PLUGIN_BHOP, EternalJK's \"Disable forcejumps\": bg_pmove.c:2670 \
                       and 2836 bunny hop instead while the bit is set and the server allows \
                       it (JAPRO_CINFO_BHOP2, pers.onlyBhop)",
            row: "japro.plugin 1",
        },
        PluginOption {
            bit: 20,
            name: "Rolls",
            help: "Rolls when you crouch while moving.",
            polarity: Polarity::SetTurnsOff,
            offer: Offer::Hidden(MOVES_YOU),
            evidence: "JAPRO_PLUGIN_NOROLL: bg_pmove.c:5110 stops rolls while the bit is set",
            row: "japro.plugin 2",
        },
        PluginOption {
            bit: 21,
            name: "Cartwheels",
            help: "Saber cartwheels to the side.",
            polarity: Polarity::SetTurnsOff,
            offer: Offer::Hidden(MOVES_YOU),
            evidence: "JAPRO_PLUGIN_NOCART: bg_saber.c:2434 stops cartwheels while the bit is \
                       set",
            row: "japro.plugin 3",
        },
        PluginOption {
            bit: 22,
            name: "Jawa run",
            help: "You run as a Jawa does.",
            polarity: Polarity::SetTurnsOn,
            offer: Offer::Hidden(
                "the server picks your run in movement code SJK's prediction does not follow \
                 yet, so your legs would switch between both runs",
            ),
            evidence: "JAPRO_PLUGIN_JAWARUN, EternalJK's \"New run animation\": \
                       bg_pmove.c:7328-7544 play the Jawa run while the bit is set",
            row: "japro.plugin 4",
        },
        PluginOption {
            bit: 23,
            name: "Respawn where you fell",
            help: "On servers with duel respawn, dying in a private duel brings you back \
                   where you fell, not at a spawn point.",
            polarity: Polarity::SetTurnsOff,
            offer: Offer::Offered,
            evidence: "JAPRO_PLUGIN_NODUELTELE, EternalJK's \"Disable duel tele\": \
                       g_combat.c:2192 keeps the spot unless the bit is set, g_client.c:3819 \
                       respawns there; a server teleport, nothing predicted",
            row: "japro.plugin 5",
        },
        PluginOption {
            bit: 24,
            name: "Checkpoints on screen",
            help: "In race mode, the server shows each checkpoint's time in the middle of \
                   your screen.",
            polarity: Polarity::SetTurnsOff,
            offer: Offer::Offered,
            evidence: "JAPRO_PLUGIN_NOCENTERCP: g_trigger.c:1778 centre-prints the time while \
                       the bit is clear (pers.showCenterCP)",
            row: "japro.plugin 6",
        },
        PluginOption {
            bit: 25,
            name: "Checkpoints in chat",
            help: "In race mode, the server also writes each checkpoint's time in the chat.",
            polarity: Polarity::SetTurnsOn,
            offer: Offer::Offered,
            evidence: "JAPRO_PLUGIN_CHATBOXCP: g_trigger.c:1782 writes the time in the chat \
                       while the bit is set (pers.showChatCP)",
            row: "japro.plugin 7",
        },
        PluginOption {
            bit: 26,
            name: "Damage numbers",
            help: "The server tells you the damage your hits do, as it is set to: chat, \
                   console, mid-screen or over the target.",
            polarity: Polarity::SetTurnsOff,
            offer: Offer::Offered,
            evidence: "JAPRO_PLUGIN_NODMGNUMBERS: g_combat.c:5692 and 5711 send them while the \
                       bit is clear, as prints, centre prints, chat or score plums",
            row: "japro.plugin 8",
        },
        PluginOption {
            bit: 27,
            name: "Shots from the centre",
            help: "Your shots leave from the middle of your view instead of the gun, on \
                   servers that allow it.",
            polarity: Polarity::SetTurnsOn,
            offer: Offer::Offered,
            evidence: "JAPRO_PLUGIN_CENTERMUZZLE: g_weapon.c:4850 fires from the centre while \
                       the bit is set and the server allows it (WT_CENTER_MUZZLEPOINT); SJK \
                       predicts no shots, so only the server's part applies",
            row: "japro.plugin 9",
        },
        PluginOption {
            bit: 28,
            name: "Checkpoints in console",
            help: "In race mode, the server writes each checkpoint's time in the console, in \
                   place of the chat.",
            polarity: Polarity::SetTurnsOn,
            offer: Offer::Offered,
            evidence: "JAPRO_PLUGIN_CONSOLECP: g_trigger.c:1780 prints the time in the console \
                       while the bit is set, and then not in the chat (pers.showConsoleCP)",
            row: "japro.plugin 10",
        },
    ],
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The core client's default, 1536: bits 9 and 10.
    const DEFAULT: u32 = 1_536;

    #[test]
    fn the_bits_are_ejk_s_japro_list_in_its_order() {
        let bits: Vec<u8> = OPTIONS.options.iter().map(|option| option.bit).collect();
        let mut expected = vec![3];
        expected.extend(19..=28);
        assert_eq!(bits, expected);
        for (number, option) in OPTIONS.options.iter().enumerate() {
            assert_eq!(option.row, format!("japro.plugin {number}"));
        }
    }

    /// Each option's state against its bit, as jaPRO's code reads it, and which ones
    /// SJK offers.
    #[test]
    fn the_polarity_follows_the_reference() {
        for (name, bit, on_when_set, offered) in [
            ("Black sabers", 3, false, true),
            ("Force jumps", 19, false, false),
            ("Rolls", 20, false, false),
            ("Cartwheels", 21, false, false),
            ("Jawa run", 22, true, false),
            ("Respawn where you fell", 23, false, true),
            ("Checkpoints on screen", 24, false, true),
            ("Checkpoints in chat", 25, true, true),
            ("Damage numbers", 26, false, true),
            ("Shots from the centre", 27, true, true),
            ("Checkpoints in console", 28, true, true),
        ] {
            let option = OPTIONS
                .options
                .iter()
                .find(|option| option.name == name)
                .unwrap();
            assert_eq!(option.bit, bit, "{name}");
            assert_eq!(option.on(1 << bit), on_when_set, "{name}");
            assert_eq!(option.on(0), !on_when_set, "{name}");
            assert_eq!(option.offered(), offered, "{name}");
        }
    }

    #[test]
    fn the_listing_under_the_default() {
        let lines = OPTIONS.listing(DEFAULT);
        assert_eq!(
            lines[1..8],
            [
                "^5 0 ^7Black sabers: ^2On",
                "^5 5 ^7Respawn where you fell: ^2On",
                "^5 6 ^7Checkpoints on screen: ^2On",
                "^5 7 ^7Checkpoints in chat: ^1Off",
                "^5 8 ^7Damage numbers: ^2On",
                "^5 9 ^7Shots from the centre: ^1Off",
                "^510 ^7Checkpoints in console: ^1Off",
            ]
        );
        assert_eq!(
            lines[8],
            "^3Not in SJK yet:^7 1 Force jumps, 2 Rolls, 3 Cartwheels, 4 Jawa run \
             (japro.plugin <number> says why)"
        );
        assert_eq!(
            OPTIONS.switch(DEFAULT, "7"),
            Ok((
                DEFAULT | 1 << 25,
                "^5Checkpoints in chat^7: ^2On".to_owned()
            ))
        );
    }
}
