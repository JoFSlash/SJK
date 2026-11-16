//! World shots of the game chat box and a centre print over duel6 at the default
//! sizes (`cg_chatBoxFontSize` 0.85, `cg_centerSize` 1, EternalJK's centre size;
//! [`crate::chat::text_sizes`]), with a draft in the chat field, in the game fonts
//! and in Inter, at 1080p.
//! Off-screen renders (no game window), ignored like the other world shots: they
//! need a GPU adapter and the installed game data named by `JKA_GAME_DATA`.
//!
//! Run: `JKA_GAME_DATA=<GameData> cargo test -p sjk-viewer text_sizes_shot --
//! --ignored --test-threads=1 --nocapture`; the pictures are in `target/world-shots`.

use super::*;

#[test]
#[ignore = "renders with the GPU and the installed game data named by JKA_GAME_DATA"]
fn duel6_text_sizes_shot() {
    for (game_font, suffix) in [("1", "game-font"), ("0", "inter")] {
        on_big_stack(move || {
            let cvars = [
                ("cg_chatbox", "60000"),
                ("cg_centerTime", "60"),
                (crate::game_font::CVAR, game_font),
            ];
            let Some((mut gpu, _profile)) = open("maps/mp/duel6.bsp", [1920, 1080], None, &cvars)
            else {
                return;
            };
            let _ = frame(&mut gpu, 1);
            let shots = menu_backdrop::tour_for("Yavin Training Grounds").expect("duel6's tour");
            let (yaw, pitch) = look(shots[0].from, shots[0].at);
            aim(&mut gpu, shots[0].from, yaw, pitch);
            if let Some(console) = gpu.console.as_mut() {
                console.set_open(false);
            }
            let now = std::time::Instant::now();
            let chat = &mut gpu.chat;
            chat.for_shot = true;
            for line in [
                "^7Kyle^7: ^2anyone up for a duel?",
                "^1Sol^7Vulpes^7: ^2HELLO there, the quick brown fox jumps over the lazy dog",
                "^7Fox^7: ^2gg",
            ] {
                chat.receive(
                    sjk_client::ServerEventKind::Chat,
                    line.to_owned(),
                    None,
                    now,
                );
            }
            chat.receive(
                sjk_client::ServerEventKind::CenterPrint,
                "Sol^7Vulpes has won the duel!\nNEXT ROUND".to_owned(),
                None,
                now,
            );
            chat.draft_for_shot("the typed text matches the chat's size");
            let name = format!("duel6-text-sizes-{suffix}-1080p");
            println!("{}", shoot(&mut gpu, 12, &name).display());
        });
    }
}
