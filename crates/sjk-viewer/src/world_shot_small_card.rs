//! World shots of the small-card notice ([`crate::small_card_notice`]) over duel6's main
//! menu, in the SJK UI and the classic style: what a 3 GB card's first start shows.
//! Off-screen renders (no game window), ignored like the other world shots: they need a
//! GPU adapter and the installed game data named by `JKA_GAME_DATA`.
//!
//! Run: `JKA_GAME_DATA=<GameData> cargo test -p sjk-viewer small_card_notice_card --
//! --ignored --test-threads=1 --nocapture`; the pictures are in `target/world-shots`.

use super::*;
use std::time::Duration;

#[test]
#[ignore = "renders with the GPU and the installed game data named by JKA_GAME_DATA"]
fn small_card_notice_card() {
    on_big_stack(|| {
        crate::video_memory::lighten_for_shot(crate::video_memory::Lightened {
            mib: 3072,
            vanilla: true,
            stock_blades: true,
            menu_cap: true,
            smaller_textures: true,
        });
        for style in ["sjk", "classic"] {
            let menu = menu::ClientMenu::new(true, String::new());
            let cvars = [
                ("ui_menuStyle", style),
                (crate::settings::quick::HIDE_CVAR, "1"),
            ];
            let Some((mut gpu, _profile)) =
                open("maps/mp/duel6.bsp", [1920, 1080], Some(menu), &cvars)
            else {
                return;
            };
            let _ = frame(&mut gpu, 20);
            gpu.ui_epoch -= Duration::from_millis(2_000);
            println!(
                "{}",
                shoot(&mut gpu, 4, &format!("small-card-notice-{style}")).display()
            );
            assert!(gpu.small_card_notice.is_open(), "the card shows");
        }
    });
}
