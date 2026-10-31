use super::*;

#[test]
fn japro_player_bhop_preference_survives_switches_without_enabling_foreign_rules() {
    let mut prediction = LocalPrediction::new(None, None, None, &VirtualFileSystem::new());
    let mut game = GameState::empty_local(0);
    let mask = sjk_game_jka::pmove_japro::PLUGIN_BHOP;
    prediction.set_ja_pro_plugin_disable(mask);
    assert!(prediction.movement_config.ja_pro.player_bhop);
    assert!(!prediction.movement_config.ja_pro.enabled);
    for (name, enabled) in [
        ("japro", true),
        ("basejka", false),
        ("JA+ Mod", false),
        ("lugormod", false),
        ("TaystJK", true),
    ] {
        game.replace_config_string(
            0,
            format!("\\gamename\\{name}\\jcinfo\\67108864").into_bytes(),
        )
        .unwrap();
        prediction.refresh_roll_rules(&game);
        assert_eq!(prediction.movement_config.ja_pro.enabled, enabled, "{name}");
        assert!(prediction.movement_config.ja_pro.player_bhop, "{name}");
    }
    prediction.set_ja_pro_plugin_disable(0);
    assert!(!prediction.movement_config.ja_pro.player_bhop);
    prediction.refresh_roll_rules(&game);
    assert!(!prediction.movement_config.ja_pro.player_bhop);
}
