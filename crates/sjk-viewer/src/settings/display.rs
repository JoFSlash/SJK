//! Display mode of the client window and the monitor facts the settings
//! screen offers choices from.
//!
//! Stock `r_fullscreen` keeps its meaning, fullscreen on or off, and Alt+Enter
//! still toggles it. SJK adds `r_exclusiveFullscreen` to say which kind of
//! fullscreen that is: 0 (the default) is borderless at the desktop size, 1 is
//! an exclusive video mode at `r_resolution`. Stock JA's fullscreen is always
//! the exclusive kind; SJK keeps borderless as its default because it does
//! not change the monitor's mode and switches away instantly.

use crate::console::ViewerConsole;
use std::time::Duration;
use winit::dpi::{LogicalSize, PhysicalSize, Size};
use winit::window::Window;

/// Cvar choosing the kind of fullscreen `r_fullscreen 1` gives.
pub(crate) const EXCLUSIVE_CVAR: &str = "r_exclusiveFullscreen";

/// Cvar: whether the window's size (`r_resolution`) and the world's resolution count the
/// screen's pixels (1) or its points (0), the units its display scaling sizes things in.
pub(crate) const HIGH_DPI_CVAR: &str = "r_highDpi";

/// [`HIGH_DPI_CVAR`]'s default: points on a Mac, pixels elsewhere. Mac apps size their
/// windows in points, EternalJK's too (SDL without high DPI), so `1280x720` is the same
/// window on a Retina screen as on any other, not a quarter of it; and the world drawn at
/// the points' resolution costs a quarter of a Retina screen's pixels, as in EternalJK
/// (Glen's frame rate in fullscreen, 29/10/2026). JA's sizes on Windows count pixels.
pub(crate) const HIGH_DPI_DEFAULT: bool = !cfg!(target_os = "macos");

/// Whether the window and the world count the screen's pixels ([`HIGH_DPI_CVAR`]).
pub(crate) fn high_dpi(console: &ViewerConsole) -> bool {
    console.bool_cvar(HIGH_DPI_CVAR).unwrap_or(HIGH_DPI_DEFAULT)
}

/// The window size `r_resolution`'s `size` asks winit for: pixels with high DPI, else
/// points.
pub(crate) fn window_size([width, height]: [u32; 2], high_dpi: bool) -> Size {
    if high_dpi {
        PhysicalSize::new(width, height).into()
    } else {
        LogicalSize::new(f64::from(width), f64::from(height)).into()
    }
}

/// How the client window covers the screen.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DisplayMode {
    /// A decorated (or `r_noborder`) window of `r_resolution`.
    Windowed,
    /// A borderless window covering the monitor at the desktop size.
    Borderless,
    /// The monitor switched to the `r_resolution` video mode.
    Exclusive,
}

impl DisplayMode {
    /// Every mode, in the order the settings row steps through them.
    pub(crate) const ALL: [Self; 3] = [Self::Windowed, Self::Borderless, Self::Exclusive];

    /// The mode the two cvars describe.
    pub(crate) fn from_cvars(fullscreen: bool, exclusive: bool) -> Self {
        match (fullscreen, exclusive) {
            (false, _) => Self::Windowed,
            (true, false) => Self::Borderless,
            (true, true) => Self::Exclusive,
        }
    }

    /// The mode the console's cvars ask for.
    pub(crate) fn requested(console: &ViewerConsole) -> Self {
        Self::from_cvars(
            console.bool_cvar("r_fullscreen").unwrap_or(false),
            console.bool_cvar(EXCLUSIVE_CVAR).unwrap_or(false),
        )
    }

    /// The mode that is applied when exclusive fullscreen is unavailable:
    /// it falls back to borderless.
    pub(crate) fn effective(self, exclusive_available: bool) -> Self {
        if self == Self::Exclusive && !exclusive_available {
            Self::Borderless
        } else {
            self
        }
    }

    /// The mode to apply while the window is `suspended` (see
    /// [`suspended_after_focus`]): exclusive fullscreen becomes windowed, which
    /// restores the desktop's video mode and drops the topmost window level
    /// winit gives an exclusive window; the other modes are unaffected.
    pub(crate) fn while_suspended(self, suspended: bool) -> Self {
        if suspended && self == Self::Exclusive {
            Self::Windowed
        } else {
            self
        }
    }

    /// The name the settings screen shows.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Windowed => "Windowed",
            Self::Borderless => "Borderless fullscreen",
            Self::Exclusive => "Exclusive fullscreen",
        }
    }

    /// The next mode `direction` steps away, wrapping, skipping exclusive
    /// fullscreen when it is unavailable.
    pub(crate) fn step(self, direction: i32, exclusive_available: bool) -> Self {
        let modes: &[Self] = if exclusive_available {
            &Self::ALL
        } else {
            &Self::ALL[..2]
        };
        let index = modes
            .iter()
            .position(|mode| *mode == self.effective(exclusive_available))
            .unwrap_or(0) as i32;
        modes[(index + direction.signum()).rem_euclid(modes.len() as i32) as usize]
    }

    /// Write this mode into the cvars. Windowed leaves the fullscreen kind
    /// alone, so Alt+Enter returns to whichever kind was last chosen.
    pub(crate) fn store(self, console: &mut ViewerConsole) {
        match self {
            Self::Windowed => {
                console.set_cvar("r_fullscreen", "0");
            }
            Self::Borderless | Self::Exclusive => {
                let exclusive = if self == Self::Exclusive { "1" } else { "0" };
                console.set_cvar(EXCLUSIVE_CVAR, exclusive);
                console.set_cvar("r_fullscreen", "1");
            }
        }
    }
}

/// How soon after entering a display mode a focus loss is taken for a side
/// effect of the switch itself, not the player leaving (Alt+Tab, Win+D).
pub(crate) const FOCUS_LOSS_GRACE: Duration = Duration::from_millis(750);

/// Whether exclusive fullscreen is suspended after a focus change. An exclusive
/// window is topmost and owns the video mode, so it would stay over the desktop
/// after Alt+Tab or Win+D: losing focus suspends it, and gaining focus (the
/// window restored from the taskbar) resumes it. A loss within
/// [`FOCUS_LOSS_GRACE`] of the last display change is ignored, so a focus
/// flicker caused by the switch cannot loop suspend and resume.
pub(crate) fn suspended_after_focus(
    suspended: bool,
    exclusive_active: bool,
    since_display_change: Duration,
    focused: bool,
) -> bool {
    if focused {
        false
    } else {
        suspended || (exclusive_active && since_display_change >= FOCUS_LOSS_GRACE)
    }
}

/// Whether a focus change that just suspended exclusive fullscreen also minimises the
/// window: always when it lost focus. Going windowed restores the desktop's video mode,
/// but the window stayed on it (Alt+Tab and Win+D "only into a smaller window", hub
/// report #131). SDL, which OpenJK's window runs on, minimises a fullscreen window on
/// focus loss (`SDL_HINT_VIDEO_MINIMIZE_ON_FOCUS_LOSS`'s default for real, not
/// desktop, fullscreen); the taskbar button brings it back, and the focus gained then
/// resumes the exclusive mode.
pub(crate) fn minimise_after_focus(suspended: bool, focused: bool) -> bool {
    suspended && !focused
}

/// The size to ask the window for, if any: `r_resolution` (`resolution`, counted in
/// pixels when `high_dpi`) while the window is windowed and not already at the size last
/// asked for (`applied`, with how it was counted). A
/// fullscreen window keeps the screen's size: borderless covers the monitor and an
/// exclusive mode is the resolution. Asking a fullscreen window for another size shrank
/// it on a Mac, a small game in the middle of a black screen (Glen's report, 29/10/2026).
/// `fullscreen` is the window's own state, which the Mac's green button changes without
/// the cvars, and still set while a Mac fullscreen animation runs; suspended exclusive
/// fullscreen keeps its size for the return.
pub(crate) fn window_size_request(
    display: DisplayMode,
    fullscreen: bool,
    suspended: bool,
    resolution: Option<[u32; 2]>,
    high_dpi: bool,
    applied: ([u32; 2], bool),
) -> Option<[u32; 2]> {
    resolution.filter(|size| {
        display == DisplayMode::Windowed
            && !fullscreen
            && !suspended
            && (*size, high_dpi) != applied
    })
}

/// What the window's monitor offers, gathered when the settings screen opens.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct MonitorModes {
    /// Distinct sizes of the monitor's video modes, largest first.
    pub(crate) sizes: Vec<[u32; 2]>,
    /// The monitor's current (desktop) size.
    pub(crate) desktop: Option<[u32; 2]>,
    /// Whether exclusive fullscreen can be used on this window.
    pub(crate) exclusive: bool,
    /// The monitor's pixels per point (2 on a Retina screen); 0 or 1 when unscaled.
    pub(crate) scale: f64,
}

impl MonitorModes {
    /// Ask the window's current monitor for its modes.
    pub(crate) fn query(window: &Window) -> Self {
        let Some(monitor) = window.current_monitor() else {
            return Self::default();
        };
        let mut sizes: Vec<[u32; 2]> = monitor
            .video_modes()
            .map(|mode| [mode.size().width, mode.size().height])
            .filter(|[width, height]| *width > 0 && *height > 0)
            .collect();
        sizes.sort_unstable_by(|a, b| b.cmp(a));
        sizes.dedup();
        let desktop = Some([monitor.size().width, monitor.size().height])
            .filter(|[width, height]| *width > 0 && *height > 0);
        let exclusive = exclusive_supported(window) && !sizes.is_empty();
        Self {
            sizes,
            desktop,
            exclusive,
            scale: monitor.scale_factor(),
        }
    }

    /// The sizes counted in points when `points`, for a window measured in points
    /// ([`HIGH_DPI_CVAR`] off): each divided by the monitor's scale, so a 3024x1964
    /// Retina desktop is 1512x982. Exclusive video modes always count pixels.
    pub(crate) fn counted_in_points(&self, points: bool) -> Self {
        if !points || self.scale <= 1.0 {
            return self.clone();
        }
        let to_points = |size: [u32; 2]| size.map(|n| (f64::from(n) / self.scale).round() as u32);
        let mut sizes: Vec<[u32; 2]> = self.sizes.iter().map(|size| to_points(*size)).collect();
        sizes.sort_unstable_by(|a, b| b.cmp(a));
        sizes.dedup();
        Self {
            sizes,
            desktop: self.desktop.map(to_points),
            ..self.clone()
        }
    }
}

/// Whether the windowing system honours exclusive fullscreen: Wayland
/// ignores the request (winit `Window::set_fullscreen`), and macOS is left to
/// borderless: winit captures the display there and asserts that capturing it
/// and setting its video mode succeed, a panic inside AppKit's redraw that
/// aborts the game (a Mac player's crash on choosing fullscreen, 25/10/2026),
/// and a captured display also takes Cmd+Tab away.
pub(crate) fn exclusive_supported(window: &Window) -> bool {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    window.window_handle().is_ok_and(|handle| {
        !matches!(
            handle.as_raw(),
            RawWindowHandle::Wayland(_) | RawWindowHandle::AppKit(_)
        )
    })
}

/// The monitor's video mode of exactly `size`, at its highest refresh rate
/// and colour depth, for exclusive fullscreen.
pub(crate) fn exclusive_video_mode(
    window: &Window,
    size: [u32; 2],
) -> Option<winit::monitor::VideoModeHandle> {
    window
        .current_monitor()?
        .video_modes()
        .filter(|mode| [mode.size().width, mode.size().height] == size)
        .max_by_key(|mode| (mode.refresh_rate_millihertz(), mode.bit_depth()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cvars_map_to_modes() {
        assert_eq!(DisplayMode::from_cvars(false, false), DisplayMode::Windowed);
        assert_eq!(DisplayMode::from_cvars(false, true), DisplayMode::Windowed);
        assert_eq!(
            DisplayMode::from_cvars(true, false),
            DisplayMode::Borderless
        );
        assert_eq!(DisplayMode::from_cvars(true, true), DisplayMode::Exclusive);
    }

    #[test]
    fn exclusive_falls_back_to_borderless() {
        assert_eq!(
            DisplayMode::Exclusive.effective(false),
            DisplayMode::Borderless
        );
        assert_eq!(
            DisplayMode::Exclusive.effective(true),
            DisplayMode::Exclusive
        );
        assert_eq!(
            DisplayMode::Windowed.effective(false),
            DisplayMode::Windowed
        );
    }

    #[test]
    fn suspended_exclusive_is_windowed() {
        use DisplayMode::*;
        assert_eq!(Exclusive.while_suspended(true), Windowed);
        assert_eq!(Exclusive.while_suspended(false), Exclusive);
        assert_eq!(Borderless.while_suspended(true), Borderless);
        assert_eq!(Windowed.while_suspended(true), Windowed);
    }

    #[test]
    fn focus_loss_suspends_exclusive_and_gain_resumes() {
        let settled = FOCUS_LOSS_GRACE;
        assert!(suspended_after_focus(false, true, settled, false));
        assert!(!suspended_after_focus(true, false, Duration::ZERO, true));
        assert!(!suspended_after_focus(false, true, settled, true));
    }

    #[test]
    fn suspending_on_focus_loss_minimises_and_resuming_does_not() {
        assert!(minimise_after_focus(true, false));
        assert!(!minimise_after_focus(false, true));
        // A change that did not suspend (borderless, windowed, the grace period) is not
        // asked to minimise: the caller acts only when the suspension changed.
        assert!(!minimise_after_focus(false, false));
    }

    #[test]
    fn focus_loss_leaves_other_modes_alone() {
        // Borderless, windowed and the exclusive request that fell back to
        // borderless are not exclusive on the window.
        assert!(!suspended_after_focus(
            false,
            false,
            FOCUS_LOSS_GRACE,
            false
        ));
    }

    #[test]
    fn focus_loss_right_after_a_switch_is_ignored() {
        let early = FOCUS_LOSS_GRACE - Duration::from_millis(1);
        assert!(!suspended_after_focus(false, true, early, false));
    }

    #[test]
    fn repeated_focus_loss_stays_suspended() {
        assert!(suspended_after_focus(true, false, Duration::ZERO, false));
    }

    #[test]
    fn only_a_windowed_window_takes_r_resolution() {
        let size = Some([1280, 720]);
        assert_eq!(
            window_size_request(
                DisplayMode::Windowed,
                false,
                false,
                size,
                true,
                ([0; 2], true)
            ),
            size
        );
        assert_eq!(
            window_size_request(
                DisplayMode::Windowed,
                false,
                false,
                size,
                true,
                ([1280, 720], true)
            ),
            None
        );
        // The same numbers in points are another size.
        assert_eq!(
            window_size_request(
                DisplayMode::Windowed,
                false,
                false,
                size,
                false,
                ([1280, 720], true)
            ),
            size
        );
        // Borderless, exclusive, the Mac's green button and suspended exclusive.
        for (display, fullscreen, suspended) in [
            (DisplayMode::Borderless, true, false),
            (DisplayMode::Exclusive, true, false),
            (DisplayMode::Windowed, true, false),
            (DisplayMode::Windowed, false, true),
        ] {
            assert_eq!(
                window_size_request(display, fullscreen, suspended, size, true, ([0; 2], true)),
                None
            );
        }
    }

    #[test]
    fn a_retina_monitor_counts_its_sizes_in_points() {
        let retina = MonitorModes {
            sizes: vec![[3024, 1964], [2560, 1600], [1512, 982]],
            desktop: Some([3024, 1964]),
            exclusive: false,
            scale: 2.0,
        };
        let points = retina.counted_in_points(true);
        assert_eq!(points.sizes, [[1512, 982], [1280, 800], [756, 491]]);
        assert_eq!(points.desktop, Some([1512, 982]));
        assert_eq!(retina.counted_in_points(false), retina);
        let unscaled = MonitorModes {
            scale: 1.0,
            ..retina.clone()
        };
        assert_eq!(unscaled.counted_in_points(true), unscaled);
    }

    #[test]
    fn stepping_wraps_and_skips_unavailable_modes() {
        use DisplayMode::*;
        assert_eq!(Windowed.step(1, true), Borderless);
        assert_eq!(Borderless.step(1, true), Exclusive);
        assert_eq!(Exclusive.step(1, true), Windowed);
        assert_eq!(Windowed.step(-1, true), Exclusive);
        assert_eq!(Borderless.step(1, false), Windowed);
        assert_eq!(Windowed.step(-1, false), Borderless);
        // A stored exclusive request steps from what is applied.
        assert_eq!(Exclusive.step(1, false), Windowed);
    }
}
