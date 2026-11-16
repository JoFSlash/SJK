//! Stock blades for weak PCs (Sol's request, 25/10/2026): with `cg_stockBlades 1` other
//! players' blades are drawn as stock blades in the colour they set (a stock colour or
//! RGB), without their blade skin; with `2` every blade is, the local player's own
//! included ([`crate::saber_skins::SaberSkins::get`]). A skin's whole look goes with it:
//! its glow shading, afterimages, glints, petals, haze, black-hole lenses and solid
//! parts are all drawn from the skin a blade wears, so a stock blade draws none, and the
//! skin passes (`saber_gpu::Runtime::has_haze`, the horizons and solids) are skipped when
//! no skinned blade is left in the frame.
//!
//! Off by default: nothing changes until a player asks. It changes only what this
//! client draws in the world: sounds stay as each player chose them (a stock blade heard
//! with a skin's sounds is already a choice players make), other players see what their
//! own setting says, and the menus' previews (the Character and Collection stage, the
//! cards, the blade pictures) keep showing the skins, which they never take from here.

/// Archived, default 0: 1 draws other players' blades stock, 2 every blade.
pub(crate) const CVAR: &str = "cg_stockBlades";
const LOOKUP: &str = "cg_stockblades";

/// Which blades this client draws stock instead of with their skin.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum StockBlades {
    /// Every blade wears its skin (0, the default).
    #[default]
    Off,
    /// Other players' blades are stock; the local player's own keeps its skin (1).
    Others,
    /// Every blade is stock, the local player's own included (2).
    All,
}

impl StockBlades {
    /// The choice `value` of [`CVAR`] names: 1 and 2 as above, anything else off.
    pub(crate) fn of(value: Option<i64>) -> Self {
        match value {
            Some(1) => Self::Others,
            Some(2) => Self::All,
            _ => Self::Off,
        }
    }

    /// The player's choice; off without a console. One name lookup, no allocation.
    pub(crate) fn read(console: Option<&crate::console::ViewerConsole>) -> Self {
        Self::of(console.and_then(|console| console.integer_cvar(LOOKUP)))
    }

    /// Whether a blade is drawn stock: `own` when it is the local player's own (their
    /// client slot in a session; a followed player's is another's).
    pub(crate) fn plain(self, own: bool) -> bool {
        match self {
            Self::Off => false,
            Self::Others => !own,
            Self::All => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_choice_names_whose_blades_are_stock() {
        assert_eq!(StockBlades::of(None), StockBlades::Off);
        assert_eq!(StockBlades::of(Some(0)), StockBlades::Off);
        assert_eq!(StockBlades::of(Some(1)), StockBlades::Others);
        assert_eq!(StockBlades::of(Some(2)), StockBlades::All);
        // A value the row never writes draws the skins, as off.
        assert_eq!(StockBlades::of(Some(-1)), StockBlades::Off);
        assert_eq!(StockBlades::of(Some(3)), StockBlades::Off);
        assert!(!StockBlades::Off.plain(true) && !StockBlades::Off.plain(false));
        assert!(!StockBlades::Others.plain(true), "your own keeps its skin");
        assert!(StockBlades::Others.plain(false));
        assert!(StockBlades::All.plain(true) && StockBlades::All.plain(false));
        assert_eq!(LOOKUP, CVAR.to_ascii_lowercase());
    }

    /// Off in a new profile, archived, and read back from the console as set.
    #[test]
    fn off_by_default_and_read_from_the_console() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config.cfg");
        let mut console = crate::console::ViewerConsole::new(path.clone()).unwrap();
        assert_eq!(
            console.cvar_default(CVAR),
            Some(&sjk_shell::CvarValue::Integer(0))
        );
        assert_eq!(StockBlades::read(Some(&console)), StockBlades::Off);
        assert_eq!(StockBlades::read(None), StockBlades::Off);
        assert!(console.set_cvar(CVAR, "1"));
        assert_eq!(StockBlades::read(Some(&console)), StockBlades::Others);
        assert!(console.set_cvar(CVAR, "2"));
        assert_eq!(StockBlades::read(Some(&console)), StockBlades::All);
        drop(console);
        let console = crate::console::ViewerConsole::new(path).unwrap();
        assert_eq!(StockBlades::read(Some(&console)), StockBlades::All);
    }
}
