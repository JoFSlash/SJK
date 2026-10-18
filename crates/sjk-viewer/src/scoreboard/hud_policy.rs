//! JoF EJK bd5e202d `CG_Draw2D`: status is conditional, telemetry and votes are not.
use super::style::ScoreboardStyle;
use crate::{GpuState, hud::HudVisibility};

#[derive(Clone, Copy)]
struct Mask {
    all: bool,
    status: bool,
    crosshair: bool,
}

fn mask(style: ScoreboardStyle, held: bool, spectator: bool, alive: bool) -> Mask {
    if style == ScoreboardStyle::Sjk {
        return Mask {
            all: held,
            status: held,
            crosshair: held,
        };
    }
    Mask {
        all: false,
        status: held || !alive || spectator,
        crosshair: !spectator && (held || !alive),
    }
}

fn frame(gpu: &GpuState, time: i32) -> Mask {
    let player = crate::first_person_view::presented_snapshot(
        gpu.live_session.as_ref(),
        gpu.demo_session.as_ref(),
        time,
    )
    .map(|snapshot| &snapshot.player);
    let style = ScoreboardStyle::from_console(gpu.console.as_ref());
    let held = gpu.gameplay_input.held(crate::input::GameButton::Scores)
        && (style == ScoreboardStyle::Classic
            || gpu
                .console
                .as_ref()
                .is_none_or(|c| c.bool_cvar("cg_drawScores").unwrap_or(true)));
    mask(
        style,
        held,
        player.is_some_and(|p| p.team() == 3),
        player.is_none_or(|p| p.health() > 0),
    )
}

/// Apply the scoreboard's visibility before status widgets and ground HUD prepare.
pub(crate) fn visibility(
    gpu: &GpuState,
    mut visibility: HudVisibility,
    time: i32,
) -> HudVisibility {
    apply(frame(gpu, time), &mut visibility);
    visibility
}

fn apply(mask: Mask, visibility: &mut HudVisibility) {
    if mask.all {
        *visibility = HudVisibility::HIDDEN;
        return;
    }
    if mask.status {
        visibility.status = false;
        visibility.vehicle = false;
        visibility.weapon = false;
        visibility.selection = false;
    }
    if mask.crosshair {
        visibility.crosshair = false;
        visibility.crosshair_names = false;
    }
}

/// Crosshair scanning follows the spectator exception in `CG_Draw2D`.
pub(crate) fn hides_crosshair(gpu: &GpuState, time: i32) -> bool {
    frame(gpu, time).crosshair
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn classic_preserves_telemetry_and_spectator_crosshairs() {
        for held in [false, true] {
            for spectator in [false, true] {
                for alive in [false, true] {
                    let m = mask(ScoreboardStyle::Classic, held, spectator, alive);
                    assert!(!m.all);
                    assert_eq!(m.status, held || !alive || spectator);
                    assert_eq!(m.crosshair, !spectator && (held || !alive));
                    let mut visibility = HudVisibility::from_console(None);
                    visibility.lagometer = true;
                    apply(m, &mut visibility);
                    assert!(visibility.hud && visibility.timer && visibility.lagometer);
                    assert_eq!(visibility.status, !m.status);
                    assert_eq!(visibility.weapon, !m.status);
                    assert_eq!(visibility.selection, !m.status);
                    assert_eq!(visibility.crosshair, !m.crosshair);
                    let sjk = mask(ScoreboardStyle::Sjk, held, spectator, alive);
                    assert_eq!(sjk.all, held);
                }
            }
        }
    }
}
