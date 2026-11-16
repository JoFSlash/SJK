//! `cp_pluginDisable` options, each named after the feature its bit switches.
//!
//! `cp_pluginDisable` is the userinfo bit mask JA+ and jaPRO servers read, and the
//! clients made for them read too, to switch plugin features for one player. Most bits
//! switch a feature off when set, a few switch one on ([`Polarity`]); the player is
//! shown the feature and whether it is on ("Black sabers: On"), never the bit's own
//! name ("No black sabers: Off"). Each server family's mod holds its table,
//! [`PluginOptions`], in JoF EternalJK's `plugin` order (`japlusPluginDisables[]` and
//! `japroPluginDisables[]`, `codemp/cgame/cg_consolecmds.c`), so an option has the
//! number EternalJK's `plugin` command gives it. Options SJK cannot honour stay in
//! the table, [`Offer::Hidden`] with the reason, so they can be offered later. The
//! setting itself (`cp_pluginDisable`, its default, the userinfo) is the core
//! client's; Settings shows the offered options as switches (`docs/mods.md`).

use crate::Host;

/// The userinfo cvar holding the bits.
pub const CVAR: &str = "cp_pluginDisable";

/// Which way round an option's bit works.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Polarity {
    /// A set bit switches the feature off (`JAPRO_PLUGIN_BLACKSABERSDISABLE`).
    SetTurnsOff,
    /// A set bit switches the feature on (`JAPRO_PLUGIN_DUELSEEOTHERS`).
    SetTurnsOn,
}

/// Whether SJK offers an option.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Offer {
    /// Settings and the `plugin` command switch it.
    Offered,
    /// Not offered, and why, in the player's words: the `plugin` command says it
    /// when asked for the option.
    Hidden(&'static str),
}

/// Why the options that change how the server moves the player are hidden.
pub const MOVES_YOU: &str = "the server would move you in a way SJK's movement prediction \
     does not follow yet, so your view would snap back";

/// One bit of `cp_pluginDisable`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PluginOption {
    /// The bit, 0 to 31.
    pub bit: u8,
    /// The feature the bit switches, as the player reads it ("Ledge grab").
    pub name: &'static str,
    /// One line on what the player sees, or what the server does, while it is on.
    pub help: &'static str,
    pub polarity: Polarity,
    pub offer: Offer,
    /// Where the reference reads the bit, and so which way round it works.
    pub evidence: &'static str,
    /// Its Settings row's name (not a cvar): the command that switches it,
    /// `<command> <number>`.
    pub row: &'static str,
}

impl PluginOption {
    /// The option's bit as a mask.
    pub const fn mask(&self) -> u32 {
        1 << self.bit
    }

    /// Whether the feature is on under `bits`.
    pub const fn on(&self, bits: u32) -> bool {
        let set = bits & self.mask() != 0;
        match self.polarity {
            Polarity::SetTurnsOn => set,
            Polarity::SetTurnsOff => !set,
        }
    }

    /// `bits` with the feature switched the other way; every other bit is kept.
    pub const fn switched(&self, bits: u32) -> u32 {
        bits ^ self.mask()
    }

    /// Whether SJK offers it.
    pub const fn offered(&self) -> bool {
        matches!(self.offer, Offer::Offered)
    }
}

/// A server family's options and the command that lists and switches them.
#[derive(Clone, Copy, Debug)]
pub struct PluginOptions {
    /// The family, for messages ("JA+").
    pub family: &'static str,
    /// The mod's command (`japlus.plugin`), JoF EternalJK's `plugin`.
    pub command: &'static str,
    /// The options in JoF EternalJK's order for this family: an option's number is
    /// its place.
    pub options: &'static [PluginOption],
}

/// How the state of an option reads in the console.
fn state(on: bool) -> &'static str {
    if on { "^2On" } else { "^1Off" }
}

/// The bits a `cp_pluginDisable` value holds, `None` when it is not a mask (a
/// negative number or not a number), which the userinfo does not send either.
pub fn parse_bits(text: &str) -> Option<u32> {
    text.trim()
        .parse::<i64>()
        .ok()
        .and_then(|value| u32::try_from(value).ok())
}

impl PluginOptions {
    /// The offered option whose Settings row is `row` (any case).
    pub fn by_row(&self, row: &str) -> Option<&'static PluginOption> {
        self.options
            .iter()
            .find(|option| option.offered() && option.row.eq_ignore_ascii_case(row))
    }

    /// The listing the command prints with no argument: each offered option with its
    /// number and state, then the ones SJK does not offer yet.
    pub fn listing(&self, bits: u32) -> Vec<String> {
        let mut lines = vec![format!(
            "^5{} plugin options^7 ({CVAR} {bits}); {} <number> switches one:",
            self.family, self.command
        )];
        lines.extend(
            self.options
                .iter()
                .enumerate()
                .filter(|(_, option)| option.offered())
                .map(|(number, option)| {
                    format!("^5{number:2} ^7{}: {}", option.name, state(option.on(bits)))
                }),
        );
        let hidden: Vec<String> = self
            .options
            .iter()
            .enumerate()
            .filter(|(_, option)| !option.offered())
            .map(|(number, option)| format!("{number} {}", option.name))
            .collect();
        if !hidden.is_empty() {
            lines.push(format!(
                "^3Not in SJK yet:^7 {} ({} <number> says why)",
                hidden.join(", "),
                self.command
            ));
        }
        lines
    }

    /// Switch option `argument` (its number) under `bits`: the new bits and the line
    /// that says the option's new state, or why it cannot be switched.
    pub fn switch(&self, bits: u32, argument: &str) -> Result<(u32, String), String> {
        let option = argument
            .trim()
            .parse::<usize>()
            .ok()
            .and_then(|number| self.options.get(number))
            .ok_or_else(|| {
                format!(
                    "{}: no option {argument}; {} lists them (0 to {})",
                    self.command,
                    self.command,
                    self.options.len().saturating_sub(1)
                )
            })?;
        if let Offer::Hidden(why) = option.offer {
            return Err(format!("{} is not in SJK yet: {why}.", option.name));
        }
        let bits = option.switched(bits);
        Ok((
            bits,
            format!("^5{}^7: {}", option.name, state(option.on(bits))),
        ))
    }

    /// The command: list the options, or switch the one numbered by the argument,
    /// saving `cp_pluginDisable` (a userinfo cvar: the server gets the change).
    pub fn run(&self, args: &[String], host: &mut dyn Host) -> Result<Vec<String>, String> {
        let text = host.cvar(CVAR).unwrap_or_default();
        let bits = parse_bits(&text).ok_or_else(|| {
            format!("{CVAR} is {text}, not a bit mask: set it to 0 or more first")
        })?;
        match args {
            [] => Ok(self.listing(bits)),
            [argument] => {
                let (bits, line) = self.switch(bits, argument)?;
                host.set_cvar(CVAR, &bits.to_string())?;
                Ok(vec![line])
            }
            _ => Err(format!("usage: {} [number]", self.command)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OPTIONS: PluginOptions = PluginOptions {
        family: "Test",
        command: "test.plugin",
        options: &[
            PluginOption {
                bit: 3,
                name: "Black sabers",
                help: "",
                polarity: Polarity::SetTurnsOff,
                offer: Offer::Offered,
                evidence: "",
                row: "test.plugin 0",
            },
            PluginOption {
                bit: 10,
                name: "Ledge grab",
                help: "",
                polarity: Polarity::SetTurnsOff,
                offer: Offer::Hidden(MOVES_YOU),
                evidence: "",
                row: "test.plugin 1",
            },
            PluginOption {
                bit: 1,
                name: "See others in duels",
                help: "",
                polarity: Polarity::SetTurnsOn,
                offer: Offer::Offered,
                evidence: "",
                row: "test.plugin 2",
            },
        ],
    };

    #[test]
    fn the_state_follows_the_polarity() {
        let [black, _, others] = [0, 1, 2].map(|index| OPTIONS.options[index]);
        assert!(black.on(0) && !black.on(1 << 3));
        assert!(!others.on(0) && others.on(1 << 1));
        assert_eq!(black.switched(1_536), 1_544);
        assert_eq!(black.switched(1_544), 1_536);
    }

    #[test]
    fn listing_names_features_and_their_state() {
        let lines = OPTIONS.listing(1_536 | (1 << 1));
        assert_eq!(
            lines[0],
            "^5Test plugin options^7 (cp_pluginDisable 1538); test.plugin <number> switches one:"
        );
        assert_eq!(lines[1], "^5 0 ^7Black sabers: ^2On");
        assert_eq!(lines[2], "^5 2 ^7See others in duels: ^2On");
        assert_eq!(
            lines[3],
            "^3Not in SJK yet:^7 1 Ledge grab (test.plugin <number> says why)"
        );
        assert_eq!(lines.len(), 4);
    }

    #[test]
    fn switching_flips_one_bit_and_refuses_hidden_options() {
        assert_eq!(
            OPTIONS.switch(1_536, "0"),
            Ok((1_544, "^5Black sabers^7: ^1Off".to_owned()))
        );
        // Bits no option names are kept.
        assert_eq!(
            OPTIONS.switch(1 << 30, " 2 ").map(|(bits, _)| bits),
            Ok((1 << 30) | (1 << 1))
        );
        let hidden = OPTIONS.switch(1_536, "1").unwrap_err();
        assert!(hidden.starts_with("Ledge grab is not in SJK yet: the server"));
        for wrong in ["3", "-1", "ledge", ""] {
            assert!(OPTIONS.switch(0, wrong).is_err(), "{wrong}");
        }
    }

    #[test]
    fn only_a_mask_is_read() {
        assert_eq!(parse_bits(" 1536 "), Some(1_536));
        assert_eq!(parse_bits("-1"), None);
        assert_eq!(parse_bits("lots"), None);
        assert_eq!(parse_bits("4294967296"), None);
    }

    /// A host holding only `cp_pluginDisable`.
    struct Cvars(Option<String>);

    impl Host for Cvars {
        fn send(&mut self, _: &str) -> Result<(), String> {
            Err("offline".into())
        }
        fn server(&self) -> Option<crate::Server> {
            None
        }
        fn me(&self) -> Option<crate::Me> {
            None
        }
        fn crosshair_point(&self) -> Option<[f32; 3]> {
            None
        }
        fn crosshair_player(&self) -> Option<u16> {
            None
        }
        fn players(&self) -> Vec<crate::Player> {
            Vec::new()
        }
        fn cvar(&self, name: &str) -> Option<String> {
            (name == CVAR).then(|| self.0.clone()).flatten()
        }
        fn set_cvar(&mut self, name: &str, value: &str) -> Result<(), String> {
            assert_eq!(name, CVAR);
            self.0 = Some(value.to_owned());
            Ok(())
        }
    }

    #[test]
    fn the_command_lists_and_saves_the_mask() {
        let mut host = Cvars(Some("1536".into()));
        let listed = OPTIONS.run(&[], &mut host).unwrap();
        assert_eq!(listed[1], "^5 0 ^7Black sabers: ^2On");
        assert_eq!(
            OPTIONS.run(&["0".into()], &mut host).unwrap(),
            ["^5Black sabers^7: ^1Off"]
        );
        assert_eq!(host.0.as_deref(), Some("1544"));
        // A refused switch saves nothing.
        assert!(OPTIONS.run(&["1".into()], &mut host).is_err());
        assert!(OPTIONS.run(&["0".into(), "1".into()], &mut host).is_err());
        assert_eq!(host.0.as_deref(), Some("1544"));
        let mut broken = Cvars(Some("-5".into()));
        assert!(OPTIONS.run(&[], &mut broken).is_err());
    }

    #[test]
    fn rows_find_offered_options_only() {
        assert_eq!(OPTIONS.by_row("TEST.PLUGIN 0").map(|o| o.bit), Some(3));
        assert!(OPTIONS.by_row("test.plugin 1").is_none());
        assert!(OPTIONS.by_row("test.plugin 9").is_none());
    }
}
