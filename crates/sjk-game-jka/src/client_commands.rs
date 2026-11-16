//! `ClientCommand`'s table (OpenJK `codemp/game/g_cmds.c`, `commands[]`) as far as the
//! server needs it to answer a name it does not know, the two flag cheats in it, `god`
//! (`Cmd_God_f`, `FL_GODMODE`) and `notarget` (`Cmd_Notarget_f`, `FL_NOTARGET`), and
//! what the smaller commands print or check: `where`, `forcechanged`, `voice_cmd`,
//! `levelshot`, `giveother`, `killother` and the bots' `debugBMove_*`.
//!
//! Both cheats are `CMD_CHEAT | CMD_ALIVE | CMD_NOINTERMISSION`, like `noclip`
//! ([`crate::noclip`]); the server checks those gates before it toggles. God mode keeps
//! the player out of `G_Damage` unless the blow is `DAMAGE_NO_PROTECTION`
//! ([`crate::damage`]); the push still lands. `notarget` keeps NPCs and turrets from
//! taking the player for an enemy. `ClientSpawn` clears `ent->flags`, so a respawn or a
//! team change ends both.

/// Every name in OpenJK's `commands[]`, which `ClientCommand` searches before it gives
/// up. (JoF EJK's table adds its mod's commands, which this server does not have.)
pub const COMMANDS: &[&str] = &[
    "addbot",
    "callteamvote",
    "callvote",
    "debugBMove_Back",
    "debugBMove_Forward",
    "debugBMove_Left",
    "debugBMove_Right",
    "debugBMove_Up",
    "duelteam",
    "follow",
    "follownext",
    "followprev",
    "forcechanged",
    "gc",
    "give",
    "giveother",
    "god",
    "kill",
    "killother",
    "levelshot",
    "noclip",
    "notarget",
    "npc",
    "say",
    "say_team",
    "score",
    "setviewpos",
    "siegeclass",
    "team",
    "teamvote",
    "tell",
    "thedestroyer",
    "t_use",
    "voice_cmd",
    "vote",
    "where",
];

/// Whether `name` is in [`COMMANDS`] (`cmdcmp` is `Q_stricmp`).
pub fn is_command(name: &[u8]) -> bool {
    COMMANDS
        .iter()
        .any(|command| name.eq_ignore_ascii_case(command.as_bytes()))
}

/// `Cmd_AddBot_f`: a client cannot add bots (`MP_SVGAME_ONLY_ADD_BOTS_AS_SERVER`).
pub const ADDBOT_REFUSED: &[u8] = b"print \"@@@ONLY_ADD_BOTS_AS_SERVER.\n\"";

/// What `ClientCommand` prints for a name not in [`COMMANDS`]:
/// `print "Unknown command <name>\n"`.
pub fn unknown(name: &[u8]) -> Vec<u8> {
    let mut text = b"print \"Unknown command ".to_vec();
    // The client reads the command's text up to its closing quote.
    text.extend(name.iter().filter(|byte| **byte != b'"'));
    text.extend_from_slice(b"\n\"");
    text
}

/// `Cmd_God_f`: the flag toggled, and the line the player is printed.
pub fn toggle_god(god: &mut bool) -> &'static [u8] {
    *god = !*god;
    if *god {
        b"print \"godmode ON\n\""
    } else {
        b"print \"godmode OFF\n\""
    }
}

/// `Cmd_Notarget_f`: the flag toggled, and the line the player is printed.
pub fn toggle_notarget(notarget: &mut bool) -> &'static [u8] {
    *notarget = !*notarget;
    if *notarget {
        b"print \"notarget ON\n\""
    } else {
        b"print \"notarget OFF\n\""
    }
}

/// `Cmd_Where_f`: the player's origin as `vtos` prints it, `(x y z)` in whole units.
pub fn where_print(origin: [f32; 3]) -> Vec<u8> {
    let [x, y, z] = origin.map(|value| value as i32);
    format!("print \"({x} {y} {z})\n\"").into_bytes()
}

/// `Cmd_ForceChanged_f`'s word to a player (`S_COLOR_GREEN`, `FORCEPOWERCHANGED`): the
/// new Force comes with the next spawn.
pub const FORCE_CHANGED: &[u8] = b"print \"^2@@@FORCEPOWERCHANGED\n\"";

/// `Cmd_VoiceCommand_f`'s refusal to a spectator (`NOVOICECHATASSPEC`).
pub const NO_VOICE_AS_SPECTATOR: &[u8] = b"print \"@@@NOVOICECHATASSPEC\n\"";

/// `bg_customSiegeSoundNames` (`bg_misc.c`): the only sounds `voice_cmd` plays.
pub const SIEGE_SOUNDS: [&str; 29] = [
    "*att_attack",
    "*att_primary",
    "*att_second",
    "*def_guns",
    "*def_position",
    "*def_primary",
    "*def_second",
    "*reply_coming",
    "*reply_go",
    "*reply_no",
    "*reply_stay",
    "*reply_yes",
    "*req_assist",
    "*req_demo",
    "*req_hvy",
    "*req_medic",
    "*req_sup",
    "*req_tech",
    "*spot_air",
    "*spot_defenses",
    "*spot_emplaced",
    "*spot_sniper",
    "*spot_troops",
    "*tac_cover",
    "*tac_fallback",
    "*tac_follow",
    "*tac_hold",
    "*tac_split",
    "*tac_together",
];

/// `Cmd_VoiceCommand_f`: the sound `argument` names, with a `*` in front, when it is
/// one of [`SIEGE_SOUNDS`] (compared without case). One already starting with `*` is
/// refused ("maybe someone is trying to be sneaky").
pub fn voice_sound(argument: &[u8]) -> Option<&'static str> {
    if argument.first() == Some(&b'*') {
        return None;
    }
    SIEGE_SOUNDS.iter().copied().find(|sound| {
        sound.len() == argument.len() + 1 && sound.as_bytes()[1..].eq_ignore_ascii_case(argument)
    })
}

/// `Cmd_LevelShot_f`'s refusals: the command is for a client on the server's own
/// machine, and not for single player.
pub const LEVELSHOT_NOT_LOCAL: &[u8] =
    b"print \"The levelshot command must be executed by a local client\n\"";
pub const LEVELSHOT_SINGLE_PLAYER: &[u8] =
    b"print \"Must not be in singleplayer mode for levelshot\n\"";
/// What `Cmd_LevelShot_f` sends once the intermission has begun.
pub const CLIENT_LEVELSHOT: &[u8] = b"clientLevelShot";

/// Whether the host the engine saw (the userinfo's `ip`) is this machine
/// (`pers.localClient`): the reference's `localhost`, or a loopback address, which is
/// how a client on the same machine reaches this server.
pub fn is_local_address(address: &[u8]) -> bool {
    let host = address
        .iter()
        .rposition(|byte| *byte == b':')
        .filter(|_| !address.ends_with(b"]"))
        .map_or(address, |colon| &address[..colon]);
    host.eq_ignore_ascii_case(b"localhost") || host.starts_with(b"127.") || host == b"[::1]"
}

/// `Cmd_GiveOther_f` and `Cmd_KillOther_f`: their usage, and `giveother`'s word when
/// its target is not alive (`killother` says `MUSTBEALIVE`).
pub const GIVEOTHER_USAGE: &[u8] = b"print \"Usage: giveother <player id> <givestring>\n\"";
pub const KILLOTHER_USAGE: &[u8] = b"print \"Usage: killother <player id>\n\"";
pub const GIVEOTHER_NOT_ALIVE: &[u8] = b"print \"Target must be alive to use this command.\n\"";

/// The `debugBMove_*` command `name` is (`Cmd_BotMove*_f`): the axis of
/// `forceMove_Forward`, `_Right` and `_Up` it sets, and the value.
pub fn debug_bot_move(name: &[u8]) -> Option<(usize, i32)> {
    [
        (&b"debugBMove_Forward"[..], (0, 4_000)),
        (b"debugBMove_Back", (0, -4_000)),
        (b"debugBMove_Right", (1, 4_000)),
        (b"debugBMove_Left", (1, -4_000)),
        (b"debugBMove_Up", (2, 4_000)),
    ]
    .into_iter()
    .find_map(|(command, axis)| name.eq_ignore_ascii_case(command).then_some(axis))
}

/// `Bot_SetForcedMovement` on one axis: a bot already forced that way stops, another
/// starts.
pub fn toggle_forced_movement(force_move: &mut [i32; 3], (axis, value): (usize, i32)) {
    force_move[axis] = if force_move[axis] != 0 { 0 } else { value };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn where_prints_whole_units() {
        assert_eq!(where_print([12.9, -3.5, 24.0]), b"print \"(12 -3 24)\n\"");
    }

    #[test]
    fn only_the_siege_voice_sounds_play() {
        assert_eq!(voice_sound(b"req_medic"), Some("*req_medic"));
        assert_eq!(voice_sound(b"REQ_MEDIC"), Some("*req_medic"));
        assert_eq!(voice_sound(b"*req_medic"), None);
        assert_eq!(voice_sound(b"death1"), None);
        assert_eq!(voice_sound(b""), None);
    }

    #[test]
    fn local_clients_are_on_this_machine() {
        assert!(is_local_address(b"localhost"));
        assert!(is_local_address(b"127.0.0.1:29071"));
        assert!(is_local_address(b"[::1]:29071"));
        assert!(is_local_address(b"[::1]"));
        assert!(!is_local_address(b"192.168.1.4:29071"));
        assert!(!is_local_address(b"Bot"));
    }

    #[test]
    fn bot_moves_toggle_their_axis() {
        let mut force_move = [0; 3];
        toggle_forced_movement(&mut force_move, debug_bot_move(b"debugbmove_back").unwrap());
        assert_eq!(force_move, [-4_000, 0, 0]);
        toggle_forced_movement(
            &mut force_move,
            debug_bot_move(b"debugBMove_Forward").unwrap(),
        );
        assert_eq!(force_move, [0, 0, 0]);
        toggle_forced_movement(&mut force_move, debug_bot_move(b"debugBMove_Up").unwrap());
        assert_eq!(force_move, [0, 0, 4_000]);
        assert_eq!(debug_bot_move(b"debugBMove_Down"), None);
    }

    #[test]
    fn god_and_notarget_toggle_and_say_so() {
        let mut god = false;
        assert_eq!(toggle_god(&mut god), b"print \"godmode ON\n\"");
        assert!(god);
        assert_eq!(toggle_god(&mut god), b"print \"godmode OFF\n\"");
        assert!(!god);
        let mut notarget = false;
        assert_eq!(toggle_notarget(&mut notarget), b"print \"notarget ON\n\"");
        assert!(notarget);
        assert_eq!(toggle_notarget(&mut notarget), b"print \"notarget OFF\n\"");
        assert!(!notarget);
    }

    #[test]
    fn the_table_is_searched_without_case() {
        assert!(is_command(b"GOD"));
        assert!(is_command(b"debugbmove_up"));
        assert!(is_command(b"forcechanged"));
        assert!(!is_command(b"amtele"));
        assert!(!is_command(b"engage_duel"));
        assert!(!is_command(b"maplist"));
    }

    #[test]
    fn an_unknown_name_is_printed_back() {
        assert_eq!(unknown(b"amtele"), b"print \"Unknown command amtele\n\"");
        assert_eq!(unknown(b"a\"b"), b"print \"Unknown command ab\n\"");
    }
}
