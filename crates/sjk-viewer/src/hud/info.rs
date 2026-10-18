//! Match-information projection into retained HUD storage.

use super::*;
use sjk_client::{CrosshairName, ObituaryTracker, WarmupText};
use std::fmt::Write as _;

impl HudOverlay {
    /// `local_client` is the viewed player's client number, whose kills and
    /// deaths the kill feed marks. `following` says whether `CG_DrawFollow` names
    /// that player ([`super::follow::shown`]); the warmup line then gives way, as
    /// `CG_Draw2D` skips `CG_DrawWarmup` while the name shows.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn update_match_information(
        &mut self,
        game: &GameState,
        server_time: i32,
        obituaries: &ObituaryTracker,
        local_client: u16,
        lagometer: &sjk_client::LagometerSamples,
        crosshair: Option<CrosshairName>,
        interrupted: bool,
        following: bool,
    ) {
        self.kill_feed
            .observe(obituaries, game, server_time, local_client);
        self.crosshair_name.clear();
        self.crosshair_alpha = 0.0;
        self.crosshair_teammate = false;
        if let Some(target) = crosshair {
            family::name(
                &mut self.crosshair_name,
                &client_name_and_team(game, target.client_num).0,
                self.family.name_colors,
            );
            self.crosshair_alpha = target.alpha;
            self.crosshair_teammate = target.teammate;
        }
        self.match_timer.clear();
        family::timer(&mut self.match_timer, game, server_time, self.family);
        self.followed_name.clear();
        if following {
            self.followed_name
                .push_str(&client_name_and_team(game, local_client).0);
        }
        self.warmup_text.clear();
        let warmup = if following {
            WarmupText::Hidden
        } else {
            sjk_client::legacy_warmup_text(game, server_time)
        };
        match warmup {
            // `CG_DrawSiegeDeathTimer` (`cg_draw.c:8319-8335`): a siege player waiting
            // for its respawn wave sees the whole seconds left until it comes
            // (`timeRemaining /= 1000`, as the reference truncates).
            WarmupText::Hidden => {
                let left = obituaries.siege_respawn_at() - server_time;
                let (hack_done, hack_length) = obituaries.hack();
                if left > 0 {
                    let _ = write!(self.warmup_text, "RESPAWN IN: {}", left / 1_000);
                } else if hack_done != 0 && hack_length > 0 {
                    // `CG_DrawHaqrBar` (`cg_draw.c:4766-4803`): the hack's progress while
                    // the use key is held on a `usetime` trigger, shown as a share done.
                    let remaining = (hack_done - server_time).clamp(0, hack_length);
                    let done = 100 - remaining * 100 / hack_length;
                    let _ = write!(self.warmup_text, "HACKING: {done}%");
                }
            }
            WarmupText::WaitingForPlayers => {
                self.warmup_text.push_str("WAITING FOR PLAYERS");
            }
            WarmupText::StartsIn(seconds) => {
                let _ = write!(self.warmup_text, "STARTS IN: {seconds}");
            }
        }
        let (hack_done, hack_length) = obituaries.hack();
        self.hack_left = hack_bar::left(hack_done, hack_length, server_time);
        self.interrupted = interrupted;
        self.lagometer.clone_from(lagometer);
    }
}
