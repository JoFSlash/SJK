//! World shots of the emote picker ([`crate::emote_picker`]) over duel6 under the SJK
//! UI: every installed emote of the game data with its picture, then a search typed.
//! An off-screen render (no game window), ignored like the other world shots: it needs
//! a GPU adapter and the installed game data named by `JKA_GAME_DATA`.
//!
//! Run: `JKA_GAME_DATA=<GameData> cargo test -p sjk-viewer emote_picker_card --
//! --ignored --test-threads=1 --nocapture`; the pictures are in `target/world-shots`.

use super::*;
use std::time::Duration;
use winit::keyboard::KeyCode;

#[test]
#[ignore = "renders with the GPU and the installed game data named by JKA_GAME_DATA"]
fn emote_picker_card() {
    on_big_stack(|| {
        let menu = menu::ClientMenu::new(true, String::new());
        let cvars = [
            ("ui_menuStyle", "sjk"),
            (crate::settings::quick::HIDE_CVAR, "1"),
        ];
        let Some((mut gpu, _profile)) = open("maps/mp/duel6.bsp", [1920, 1080], Some(menu), &cvars)
        else {
            return;
        };
        if crate::emotes::lock().catalogue.is_none()
            && let Some(vfs) = gpu.vfs.as_deref()
        {
            crate::emotes::lock().catalogue = Some(crate::emotes::load(vfs));
        }
        // The frames put the catalogue's pictures into the atlas.
        let _ = frame(&mut gpu, 20);
        gpu.ui_epoch -= Duration::from_millis(2_000);
        let catalogue = crate::emotes::lock().catalogue.clone().unwrap_or_default();
        println!("installed emotes: {}", catalogue.len());
        gpu.emote_picker.card.show(&catalogue);
        println!("{}", shoot(&mut gpu, 4, "emote-picker").display());
        assert!(gpu.emote_picker.card.is_open(), "the card shows");
        for character in ["d", "a", "n"] {
            let _ = gpu
                .emote_picker
                .card
                .key(KeyCode::KeyA, Some(character), false, false);
        }
        let _ = gpu
            .emote_picker
            .card
            .key(KeyCode::ArrowRight, None, false, false);
        println!("{}", shoot(&mut gpu, 4, "emote-picker-search").display());
        gpu.emote_picker_choice(Some(crate::emote_picker::Choice::Close));
    });
}
