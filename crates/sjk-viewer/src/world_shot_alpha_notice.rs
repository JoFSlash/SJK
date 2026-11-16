//! World shots of the alpha notice ([`crate::alpha_notice_card`]) over duel6 under the
//! SJK UI, before and after the tick. Off-screen renders (no game window), ignored like
//! the other world shots: they need a GPU adapter and the installed game data named by
//! `JKA_GAME_DATA`.
//!
//! Run: `JKA_GAME_DATA=<GameData> cargo test -p sjk-viewer alpha_notice_card_shot --
//! --ignored --test-threads=1 --nocapture`; the pictures are in `target/world-shots`.

use super::*;
use crate::alpha_notice_card::Choice;
use std::time::Duration;

#[test]
#[ignore = "renders with the GPU and the installed game data named by JKA_GAME_DATA"]
fn alpha_notice_card_shot() {
    on_big_stack(|| {
        let size = [1920, 1080];
        let menu = menu::ClientMenu::new(true, String::new());
        let cvars = [
            ("ui_menuStyle", "sjk"),
            (crate::alpha_notice_card::CVAR, "0"),
        ];
        let Some((mut gpu, _profile)) = open("maps/mp/duel6.bsp", size, Some(menu), &cvars) else {
            return;
        };
        let _ = frame(&mut gpu, 20);
        gpu.ui_epoch -= Duration::from_millis(2_000);
        shoot(&mut gpu, 4, "alpha-notice");
        assert!(
            gpu.alpha_card.is_open(),
            "the card shows at the first start"
        );

        // One promise ticked: Continue stays grey.
        let _ = gpu
            .alpha_card
            .handle_key_code_for_test(winit::keyboard::KeyCode::Space);
        shoot(&mut gpu, 2, "alpha-notice-one-ticked");
        for _ in 1..crate::alpha_notice_card::PROMISES.len() {
            let _ = gpu
                .alpha_card
                .handle_key_code_for_test(winit::keyboard::KeyCode::Space);
        }
        shoot(&mut gpu, 2, "alpha-notice-ticked");
        let choice = gpu
            .alpha_card
            .handle_key_code_for_test(winit::keyboard::KeyCode::Enter);
        assert_eq!(choice, Some(Choice::Continue));
        gpu.alpha_card_choice(choice);
        shoot(&mut gpu, 2, "alpha-notice-accepted");
        assert!(!gpu.alpha_card.is_open(), "accepted, the card is gone");

        // Shown again, its Report a bug icon opens the dialog over it.
        if let Some(console) = &mut gpu.console {
            console.set_cvar(crate::alpha_notice_card::CVAR, "0");
        }
        let _ = frame(&mut gpu, 2);
        assert!(gpu.alpha_card.is_open(), "ui_alphaNotice 0 shows it again");
        gpu.alpha_card_choice(Some(Choice::Report));
        shoot(&mut gpu, 2, "alpha-notice-report");
        assert!(gpu.text_dialog.is_open(), "the report dialog opens");
        assert!(!gpu.alpha_card.is_open(), "the card waits under the dialog");
    });
}
