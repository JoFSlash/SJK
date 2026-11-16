//! World shot of the emote help card ([`crate::emote_help`]) over duel6 under the SJK
//! UI, opened by `help_emotes`. An off-screen render (no game window), ignored like the
//! other world shots: it needs a GPU adapter and the installed game data named by
//! `JKA_GAME_DATA`.
//!
//! Run: `JKA_GAME_DATA=<GameData> cargo test -p sjk-viewer emote_help_card --
//! --ignored --test-threads=1 --nocapture`; the picture is in `target/world-shots`.

use super::*;
use std::time::Duration;

#[test]
#[ignore = "renders with the GPU and the installed game data named by JKA_GAME_DATA"]
fn emote_help_card() {
    on_big_stack(|| {
        let size = [1920, 1080];
        let menu = menu::ClientMenu::new(true, String::new());
        let cvars = [
            ("ui_menuStyle", "sjk"),
            (crate::settings::quick::HIDE_CVAR, "1"),
        ];
        let Some((mut gpu, _profile)) = open("maps/mp/duel6.bsp", size, Some(menu), &cvars) else {
            return;
        };
        let _ = frame(&mut gpu, 20);
        gpu.ui_epoch -= Duration::from_millis(2_000);
        gpu.emote_help_command()
            .expect("help_emotes opens the card");
        shoot(&mut gpu, 4, "emote-help");
        assert!(gpu.emote_help.card.is_open(), "the card shows");
        gpu.emote_help_choice(Some(crate::emote_help::Choice::Close));
    });
}
