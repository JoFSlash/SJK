//! The in-game SJK pop-up: SJK's own screens, reached from the SJK button
//! left of About on the classic bar. The SJK UI has them in its row of icons and
//! on its Profile screen instead (`sjk_focus.rs`, `profile_hub.rs`).
//! Add an entry to [`ENTRIES`] and its action in `game_menu_actions.rs`
//! (`activate_sjk_row`).

/// One entry: its label and the line describing it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Entry {
    pub(crate) label: &'static str,
    pub(crate) hint: &'static str,
}

/// The pop-up's entries, top to bottom.
pub(crate) const ENTRIES: [Entry; 7] = [
    Entry {
        label: "Changelog",
        hint: "What changed in each SlashJK release, and who made it",
    },
    Entry {
        label: "Credits",
        hint: "The people who make SlashJK",
    },
    Entry {
        label: "Profile",
        hint: "Your medals, bio, record and achievements",
    },
    Entry {
        label: "Identity",
        hint: "Your SlashJK key and profile, and the players the hub knows here",
    },
    Entry {
        label: "Report a bug",
        hint: "Tell the team what went wrong; it goes to the hub",
    },
    Entry {
        label: "Report a player",
        hint: "Everyone here; tell the team about a cheater or a troll",
    },
    Entry {
        label: "Global chat",
        hint: "Talk with everyone online, on any server or in the menus",
    },
];

/// Rows of the entries.
pub(crate) const CHANGELOG: usize = 0;
pub(crate) const CREDITS: usize = 1;
pub(crate) const PROFILE: usize = 2;
pub(crate) const IDENTITY: usize = 3;
pub(crate) const REPORT: usize = 4;
pub(crate) const REPORT_PLAYER: usize = 5;
pub(crate) const SJK_CHAT: usize = 6;
