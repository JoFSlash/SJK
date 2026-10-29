//! The JA+ and jaPRO plugin options as Settings switches (`docs/mods.md`, Plugin
//! options). Each row stands for one bit of `cp_pluginDisable`, named after the
//! feature and shown On while the feature is on, whichever way its bit works; the
//! tables, with each bit's polarity and evidence, are the mods'
//! ([`sjk_mod_japlus::plugin`], [`sjk_mod_japro::plugin`]). A row is a derived switch
//! ([`super::DerivedSwitch`]): reading and flipping it read and write
//! `cp_pluginDisable`, a userinfo cvar, so a change reaches the server as any
//! userinfo change does. Bits no row names are kept as they are.

use crate::console::ViewerConsole;
use sjk_client::PLUGIN_DISABLE_DEFAULT;
use sjk_mod::plugin_options::{CVAR, PluginOption, PluginOptions};

/// The families whose options Settings shows: those of the mods this build carries.
const FAMILIES: &[PluginOptions] = &[
    #[cfg(feature = "mod-japlus")]
    sjk_mod_japlus::plugin::OPTIONS,
    #[cfg(feature = "mod-japro")]
    sjk_mod_japro::plugin::OPTIONS,
];

/// The offered option whose Settings row is `row`, if it is one.
pub(super) fn option(row: &str) -> Option<&'static PluginOption> {
    FAMILIES.iter().find_map(|family| family.by_row(row))
}

/// The bits the server is sent: `cp_pluginDisable`, or the default when it is not a
/// mask, as the userinfo has it (`console_profile.rs`, `CompatProfile::userinfo_for`).
fn bits(console: &ViewerConsole) -> u32 {
    console
        .integer_cvar(CVAR)
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or(PLUGIN_DISABLE_DEFAULT)
}

/// Whether row `row`'s feature is on.
pub(super) fn on(console: &ViewerConsole, row: &str) -> bool {
    option(row).is_some_and(|option| option.on(bits(console)))
}

/// Switch row `row`'s feature the other way, keeping every other bit.
pub(super) fn switch(console: &mut ViewerConsole, row: &str) {
    if let Some(option) = option(row) {
        let bits = option.switched(bits(console));
        console.set_cvar(CVAR, &bits.to_string());
    }
}

/// Whether row `row`'s feature is on under the default mask.
pub(super) fn on_by_default(row: &str) -> bool {
    option(row).is_some_and(|option| option.on(PLUGIN_DISABLE_DEFAULT))
}

/// What row `row` does, in the player's words.
pub(super) fn help(row: &str) -> Option<&'static str> {
    option(row).map(|option| option.help)
}

/// Every offered option with its family, for the tests that pin the rows to them.
#[cfg(test)]
pub(super) fn offered() -> impl Iterator<Item = (&'static PluginOptions, &'static PluginOption)> {
    FAMILIES.iter().flat_map(|family| {
        family
            .options
            .iter()
            .filter(|option| option.offered())
            .map(move |option| (family, option))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn console() -> (tempfile::TempDir, ViewerConsole) {
        let directory = tempfile::tempdir().unwrap();
        let console = ViewerConsole::new(directory.path().join("config.cfg")).unwrap();
        (directory, console)
    }

    /// Every offered option has its row: a Settings switch labelled with the feature
    /// and its family, on the Mods page once.
    #[cfg(all(feature = "mod-japlus", feature = "mod-japro"))]
    #[test]
    fn every_offered_option_has_one_switch_and_nothing_else_does() {
        let rows: Vec<&super::super::Setting> = super::super::search::rows()
            .iter()
            .filter(|setting| setting.cvar.contains(".plugin "))
            .collect();
        assert_eq!(rows.len(), offered().count());
        for (family, option) in offered() {
            let row = rows
                .iter()
                .find(|setting| setting.cvar == option.row)
                .unwrap_or_else(|| panic!("{} has no row", option.row));
            assert_eq!(row.label, format!("{} ({})", option.name, family.family));
            assert!(matches!(row.kind, super::super::ValueKind::Bool));
            assert!(super::super::derived_switch(option.row).is_some());
            assert_eq!(super::super::help::help(option.row), Some(option.help));
        }
        // A hidden option has no row, even typed.
        assert!(option("japlus.plugin 10").is_none());
        assert!(super::super::derived_switch("japlus.plugin 10").is_none());
    }

    /// The default mask, 1536, shows the defaults EJK's JA+ list has with it, and a
    /// player's own mask is read as it is.
    #[cfg(feature = "mod-japlus")]
    #[test]
    fn rows_read_the_mask_whichever_way_the_bit_works() {
        let (_directory, mut console) = console();
        assert_eq!(bits(&console), PLUGIN_DISABLE_DEFAULT);
        assert!(!on(&console, "japlus.plugin 1"), "see others: bit 1 clear");
        assert!(on(&console, "japlus.plugin 3"), "black sabers: bit 3 clear");
        assert!(
            !on(&console, "japlus.plugin 9"),
            "staff on the back: bit 9 set"
        );
        assert!(
            on(&console, "JAPLUS.PLUGIN 15"),
            "holstered sabers: bit 29 clear"
        );
        for row in ["japlus.plugin 1", "japlus.plugin 3", "japlus.plugin 9"] {
            assert_eq!(on(&console, row), on_by_default(row), "{row}");
        }
        // An existing value: see others on (bit 1), black off (bit 3), staff on the
        // back on (bit 9 clear), bits no row names (20, 30) kept.
        console.set_cvar(
            CVAR,
            &((1 << 1) | (1 << 3) | (1 << 20) | (1 << 30)).to_string(),
        );
        assert!(on(&console, "japlus.plugin 1"));
        assert!(!on(&console, "japlus.plugin 3"));
        assert!(on(&console, "japlus.plugin 9"));
        // Not a mask: read as the default the server is then sent.
        console.set_cvar(CVAR, "-1");
        assert!(!on(&console, "japlus.plugin 9"));
        assert!(on(&console, "japlus.plugin 3"));
    }

    /// Switching a row flips its bit alone, and the userinfo then carries the new mask
    /// (a `cp_pluginDisable` change marks it for sending, so the server gets it in a
    /// match: `console_session::tests::plugin_mask_changes_mark_userinfo_dirty`).
    #[cfg(feature = "mod-japlus")]
    #[test]
    fn switching_a_row_flips_one_bit_and_updates_the_userinfo() {
        let (_directory, mut console) = console();
        console.set_cvar(CVAR, &(1_536 | (1 << 20)).to_string());
        switch(&mut console, "japlus.plugin 3");
        assert_eq!(
            console.integer_cvar(CVAR),
            Some(1_536 | (1 << 20) | (1 << 3))
        );
        assert!(!on(&console, "japlus.plugin 3"));
        let userinfo = console.userinfo().unwrap();
        assert_eq!(userinfo.plugin_disable, Some(1_536 | (1 << 20) | (1 << 3)));
        let sent = sjk_client::CompatProfile::JaPlus { version: None }.userinfo_for(&userinfo);
        assert_eq!(sent.plugin_disable, Some(1_536 | (1 << 20) | (1 << 3)));
        switch(&mut console, "japlus.plugin 3");
        assert_eq!(console.integer_cvar(CVAR), Some(1_536 | (1 << 20)));
        // A hidden option's row does nothing.
        switch(&mut console, "japlus.plugin 10");
        assert_eq!(console.integer_cvar(CVAR), Some(1_536 | (1 << 20)));
    }

    /// A profile that never touched the mask keeps sending 1536.
    #[test]
    fn an_untouched_profile_sends_the_default() {
        let (_directory, console) = console();
        let userinfo = console.userinfo().unwrap();
        assert_eq!(userinfo.plugin_disable, Some(PLUGIN_DISABLE_DEFAULT));
        let sent = sjk_client::CompatProfile::TaystJk.userinfo_for(&userinfo);
        assert_eq!(sent.plugin_disable, Some(1_536));
    }

    /// The jaPRO rows under the default and a switched one.
    #[cfg(feature = "mod-japro")]
    #[test]
    fn japro_rows_follow_their_bits() {
        let (_directory, mut console) = console();
        assert!(on(&console, "japro.plugin 0"), "black sabers");
        assert!(on(&console, "japro.plugin 6"), "checkpoints on screen");
        assert!(!on(&console, "japro.plugin 7"), "checkpoints in chat");
        assert!(!on(&console, "japro.plugin 9"), "shots from the centre");
        switch(&mut console, "japro.plugin 9");
        assert_eq!(console.integer_cvar(CVAR), Some(1_536 | (1 << 27)));
        assert!(on(&console, "japro.plugin 9"));
        // Black sabers is one bit on both families' lists.
        #[cfg(feature = "mod-japlus")]
        {
            switch(&mut console, "japro.plugin 0");
            assert!(!on(&console, "japlus.plugin 3"));
        }
    }
}
