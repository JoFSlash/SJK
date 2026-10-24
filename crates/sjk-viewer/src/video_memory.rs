//! Small graphics cards found at start: SJK reads the card's video memory
//! ([`sjk_gpu_memory`]) and, on a card of 4 GB or less, lowers the texture detail
//! (`r_picmip`) and stands the menus on pictures ([`crate::menu_pictures`]) before the
//! first map is built, so a 3 GB card does not have to run out first (Vin's, 21/10/2026).
//! A built-in GPU (under 1 GB of its own) also draws the 3D view at 75% of the window
//! (`r_renderScale`): it is limited by the pixels it draws (Slash's, 24/10/2026).
//!
//! It chooses once a card: [`CVAR`] keeps the size it chose for, so a player who sets
//! the texture detail back is not overridden at the next start, and a new card is
//! looked at again. It only lowers a setting still at its default. The card's size and
//! budget also go into the log and into crash reports ([`summary`]).

use crate::console::ViewerConsole;
use std::sync::OnceLock;

/// The video memory (MiB) the small-card settings were last chosen for; 0 never.
pub(crate) const CVAR: &str = "r_videoMemoryChecked";
const MIB: u64 = 1 << 20;

static SUMMARY: OnceLock<String> = OnceLock::new();

/// Register [`CVAR`] (archived).
pub(crate) fn register(cvars: &mut sjk_shell::CvarRegistry) -> Result<(), sjk_shell::CvarError> {
    cvars.register(sjk_shell::CvarDefinition::new(
        CVAR,
        0_i64,
        sjk_shell::CvarFlags::ARCHIVE,
        "The graphics card's video memory (MiB) SJK last chose the small-card settings for; \
         0 to choose them again at the next start",
    ))
}

/// The texture detail a card with `dedicated` bytes of its own starts on: `None` when it
/// has room. An integrated GPU's few hundred MB count as a small card.
pub(crate) fn picmip_for(dedicated: u64) -> Option<i64> {
    match dedicated / MIB {
        0 => None,
        mib if mib < 3584 => Some(2),
        mib if mib < 4608 => Some(1),
        _ => None,
    }
}

/// The 3D resolution (`r_renderScale`) a card with `dedicated` bytes of its own starts
/// on: `None` for the window's. Built-in GPUs report a few hundred MB; a 1 GB discrete
/// card or more keeps it.
pub(crate) fn render_percent_for(dedicated: u64) -> Option<i64> {
    match dedicated / MIB {
        0 => None,
        mib if mib < 1024 => Some(75),
        _ => None,
    }
}

/// At start, before the first map: read the card and choose for it once
/// ([`apply`]); the menus' blade pictures follow the menu pictures either way.
pub(crate) fn at_start(console: &mut ViewerConsole) {
    if let Some(memory) = sjk_gpu_memory::high_performance() {
        let budget = memory.budget.map_or_else(String::new, |budget| {
            format!(", budget {} MiB", budget / MIB)
        });
        let summary = format!("{} MiB video memory{budget}", memory.dedicated / MIB);
        crate::log::progress(format_args!("graphics card: {}, {summary}", memory.name));
        let _ = SUMMARY.set(summary);
        for line in apply(console, memory.dedicated) {
            crate::log::progress(format_args!("{line}"));
            console.push_log_quiet(&line);
        }
    }
    crate::blade_pictures::set_live(!crate::menu_pictures::enabled(Some(console)));
}

/// Choose the small-card settings for a card of `dedicated` bytes, unless they were
/// chosen for this size already: the lines saying what changed.
fn apply(console: &mut ViewerConsole, dedicated: u64) -> Vec<String> {
    let mib = (dedicated / MIB) as i64;
    if mib == 0 || console.integer_cvar(CVAR) == Some(mib) {
        return Vec::new();
    }
    console.set_cvar(CVAR, &mib.to_string());
    let Some(picmip) = picmip_for(dedicated) else {
        return Vec::new();
    };
    let mut lines = Vec::new();
    let detail = crate::world_materials::filtering::mips::PICMIP_CVAR;
    if console.integer_cvar(detail).unwrap_or(0) == 0
        && console.set_cvar(detail, &picmip.to_string())
    {
        lines.push(format!(
            "Your graphics card has {mib} MiB of video memory: Texture detail set to {picmip} \
             (Settings, Renderer)."
        ));
    }
    if !crate::menu_pictures::enabled(Some(console))
        && console.set_cvar(crate::menu_pictures::CVAR, "1")
    {
        lines.push("The menus use pictures to save video memory (Settings, Menu pictures).".into());
    }
    let scale = crate::frame_target::scale::PERCENT_CVAR;
    if let Some(percent) = render_percent_for(dedicated)
        && console.integer_cvar(scale).unwrap_or(100) == 100
        && console.set_cvar(scale, &percent.to_string())
    {
        lines.push(format!(
            "Built-in graphics: the 3D view is drawn at {percent}% of the window, HUD and text \
             stay sharp (Settings, Renderer, 3D resolution)."
        ));
    }
    lines
}

/// The card's video memory for crash reports, once read.
pub(crate) fn summary() -> Option<&'static str> {
    SUMMARY.get().map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    const GIB: u64 = 1 << 30;

    fn fresh() -> (tempfile::TempDir, ViewerConsole) {
        let directory = tempfile::tempdir().unwrap();
        let console = ViewerConsole::new(directory.path().join("config.cfg")).unwrap();
        (directory, console)
    }

    #[test]
    fn small_cards_start_on_smaller_textures() {
        assert_eq!(picmip_for(512 * MIB), Some(2), "an integrated GPU");
        assert_eq!(picmip_for(2 * GIB), Some(2));
        assert_eq!(picmip_for(3 * GIB), Some(2));
        assert_eq!(picmip_for(4 * GIB), Some(1));
        assert_eq!(picmip_for(6 * GIB), None);
        assert_eq!(picmip_for(0), None, "a driver that does not say");
    }

    #[test]
    fn built_in_graphics_start_on_a_smaller_3d_view() {
        assert_eq!(
            render_percent_for(495 * MIB),
            Some(75),
            "Slash's built-in Radeon"
        );
        assert_eq!(render_percent_for(GIB), None);
        assert_eq!(render_percent_for(3 * GIB), None);
        assert_eq!(render_percent_for(0), None, "a driver that does not say");
        let (_directory, mut console) = fresh();
        let scale = crate::frame_target::scale::PERCENT_CVAR;
        assert_eq!(apply(&mut console, 495 * MIB).len(), 3);
        assert_eq!(console.integer_cvar(scale), Some(75));
        let (_other, mut console) = fresh();
        console.set_cvar(scale, "90");
        apply(&mut console, 495 * MIB);
        assert_eq!(
            console.integer_cvar(scale),
            Some(90),
            "the player's own choice stays"
        );
    }

    #[test]
    fn it_chooses_once_a_card_and_keeps_what_the_player_set() {
        let (_directory, mut console) = fresh();
        let detail = crate::world_materials::filtering::mips::PICMIP_CVAR;
        assert_eq!(apply(&mut console, 3 * GIB).len(), 2);
        assert_eq!(console.integer_cvar(detail), Some(2));
        assert!(crate::menu_pictures::enabled(Some(&console)));
        // The player sets it back: the same card does not change it again.
        console.set_cvar(detail, "0");
        assert!(apply(&mut console, 3 * GIB).is_empty());
        assert_eq!(console.integer_cvar(detail), Some(0));
        // A large card changes nothing.
        let (_other, mut console) = fresh();
        assert!(apply(&mut console, 8 * GIB).is_empty());
        assert_eq!(console.integer_cvar(detail), Some(0));
        assert_eq!(console.integer_cvar(CVAR), Some(8192));
    }
}
