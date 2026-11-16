//! World shots of Settings' Chat sounds (`chat_sounds.rs`) over duel6: Advanced's
//! Sound focused on the private messages' sound, at 1080p and 4K, with Turin's choice
//! (public chat off, a sound of its own for private messages) so the changed dots
//! show. Ignored like the other world shots.

use super::*;

#[test]
#[ignore = "renders with the GPU and the installed game data named by JKA_GAME_DATA"]
fn duel6_settings_chat_sounds() {
    for (size, suffix) in [([1920, 1080], "1080p"), ([3840, 2160], "4k")] {
        on_big_stack(move || {
            let menu = menu::ClientMenu::new(true, String::new());
            let cvars = [
                ("ui_menuStyle", "sjk"),
                (crate::settings::quick::HIDE_CVAR, "1"),
                ("cg_chatBeep", "0"),
                ("cl_chatSoundPrivate", "whisper"),
            ];
            let Some((mut gpu, _profile)) = open("maps/mp/duel6.bsp", size, Some(menu), &cvars)
            else {
                return;
            };
            let _ = frame(&mut gpu, 4);
            if let (Some(menu), Some(console)) = (gpu.client_menu.as_mut(), gpu.console.as_ref()) {
                // Advanced's Sound.
                menu.open_sjk_settings(console, 3, crate::player_menu::ReturnTarget::MainMenu);
                menu.select_setting_for_shot("cl_chatSoundPrivate");
            }
            gpu.ui_epoch -= std::time::Duration::from_millis(2_000);
            let name = format!("settings-chat-sounds-{suffix}");
            println!("{}", shoot(&mut gpu, 6, &name).display());
        });
    }
}
