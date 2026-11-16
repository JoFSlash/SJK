//! The cheats of a server with `sv_cheats` that live here: `t_use name`
//! (`Cmd_TargetUse_f`, `g_cmds.c`), which fires a name as the player
//! (`G_UseTargets2(ent, ent, name)`), as mappers test a map's targets and scripts;
//! `noclip` (`Cmd_Noclip_f`), which lets the player fly through the world
//! ([`sjk_game_jka::noclip`]; the think applies it); `god` and `notarget`
//! (`Cmd_God_f`, `Cmd_Notarget_f`, [`sjk_game_jka::client_commands`]), which `G_Damage`
//! and the enemy checks read; and SJK's own `jetpack`, which gives or takes away the
//! jetpack ([`sjk_game_jka::give::toggle_jetpack`]). (`setviewpos` is
//! `bridge_teleport`'s, `give` `bridge_commands`'.)

use super::*;

/// The cheats [`NativeGame::cheat_command`] runs.
#[derive(Clone, Copy, Eq, PartialEq)]
enum Cheat {
    Noclip,
    God,
    Notarget,
    TargetUse,
    Jetpack,
}

impl NativeGame {
    /// `ClientCommand` for `t_use`, `noclip`, `god`, `notarget` and `jetpack`: whether
    /// `text` was one of them.
    pub(super) fn cheat_command(&mut self, client: usize, text: &[u8]) -> bool {
        let mut words = text
            .split(|byte| byte.is_ascii_whitespace())
            .filter(|word| !word.is_empty());
        let Some(command) = words.next() else {
            return false;
        };
        let Some(cheat) = [
            (&b"noclip"[..], Cheat::Noclip),
            (b"god", Cheat::God),
            (b"notarget", Cheat::Notarget),
            (b"t_use", Cheat::TargetUse),
            (b"jetpack", Cheat::Jetpack),
        ]
        .into_iter()
        .find_map(|(name, cheat)| command.eq_ignore_ascii_case(name).then_some(cheat)) else {
            return false;
        };
        // `ClientCommand`'s gates (`g_cmds.c:3466-3480`), in its order: all but `t_use`
        // are also `CMD_NOINTERMISSION`.
        if cheat != Cheat::TargetUse && self.refused_at_intermission(client, command) {
            return true;
        }
        if !self.settings.cheats {
            self.told
                .push(Told::One(client, b"print \"@@@NOCHEATS\n\"".to_vec()));
            return true;
        }
        if self
            .peer(client)
            .is_none_or(|peer| peer.health <= 0 || !peer.playing())
        {
            self.told
                .push(Told::One(client, b"print \"@@@MUSTBEALIVE\n\"".to_vec()));
            return true;
        }
        if cheat == Cheat::TargetUse {
            // `Cmd_TargetUse_f`: needs a name.
            if let Some(name) = words.next() {
                let name = String::from_utf8_lossy(name).into_owned();
                self.fire_targets(&name, client, self.last_frame_time);
            }
            return true;
        }
        if cheat == Cheat::Jetpack {
            // The think puts the pack on the player (`EF_JETPACK`) or takes it off.
            if let Some(peer) = self.peer_mut(client) {
                let message =
                    sjk_game_jka::give::toggle_jetpack(&mut peer.state, &mut peer.gear.packs);
                peer.movement = peer.movement.reseeded(&peer.state);
                self.told.push(Told::One(client, message.to_vec()));
            }
            return true;
        }
        // The flag toggled and the player told. Noclip's next think puts it in (or takes
        // it out of) `PM_NOCLIP`; the other two are read where they apply.
        if let Some(peer) = self.peer_mut(client) {
            let message = match cheat {
                Cheat::Noclip => sjk_game_jka::noclip::toggle(&mut peer.noclip),
                Cheat::God => sjk_game_jka::client_commands::toggle_god(&mut peer.god),
                _ => sjk_game_jka::client_commands::toggle_notarget(&mut peer.notarget),
            };
            self.told.push(Told::One(client, message.to_vec()));
        }
        true
    }
}
