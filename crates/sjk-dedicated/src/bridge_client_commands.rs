//! The rest of `ClientCommand`'s table (OpenJK `codemp/game/g_cmds.c`): `where`,
//! `forcechanged`, `voice_cmd`, `levelshot`, `giveother`, `killother`, `thedestroyer`
//! and the bots' `debugBMove_*`, with what they print and check in
//! [`sjk_game_jka::client_commands`]; and `give`'s item by name, which `giveother`
//! shares.

use super::*;
use sjk_game_jka::client_commands as commands;
use sjk_game_jka::client_view::client_number_from_string;
use sjk_game_jka::give::Given;

/// `CMD_CHEAT`, `CMD_ALIVE`, `CMD_NOINTERMISSION`: the table's gates.
const CHEAT: u8 = 1;
const ALIVE: u8 = 2;
const NO_INTERMISSION: u8 = 4;
/// `EV_VOICECMD_SOUND`, and the field its speaker is in (`s.groundEntityNum`).
const EV_VOICECMD_SOUND: u32 = 75;
const ES_GROUND_ENTITY: usize = 22;
/// `WP_SABER`.
const WP_SABER: u8 = 3;
/// `GT_SINGLE_PLAYER`.
const GT_SINGLE_PLAYER: i32 = 5;

impl NativeGame {
    /// `ClientCommand` for the commands here: whether `command` was one of them.
    /// `arguments` are the words after it.
    pub(super) fn table_command(
        &mut self,
        client: usize,
        command: &[u8],
        arguments: &[&[u8]],
        server_time: i32,
    ) -> bool {
        let is = |name: &[u8]| command.eq_ignore_ascii_case(name);
        let bot_move = commands::debug_bot_move(command);
        let gates = if is(b"where") || is(b"voice_cmd") {
            NO_INTERMISSION
        } else if is(b"levelshot") || is(b"giveother") {
            CHEAT | ALIVE | NO_INTERMISSION
        } else if is(b"killother") || is(b"thedestroyer") || bot_move.is_some() {
            CHEAT | ALIVE
        } else {
            return false;
        };
        if self.gated(client, command, gates) {
            return true;
        }
        let level_time = self.last_frame_time;
        if let Some(movement) = bot_move {
            // `Cmd_BotMove*_f`: the bot by number (`atoi`, 0 without one).
            let bot = sjk_game_jka::userinfo::atoi(arguments.first().copied().unwrap_or(b""));
            if let Some(mind) = usize::try_from(bot)
                .ok()
                .and_then(|bot| self.bots.minds.get_mut(bot))
                .and_then(Option::as_mut)
            {
                commands::toggle_forced_movement(&mut mind.force_move, movement);
            }
        } else if is(b"where") {
            if let Some(peer) = self.peer(client) {
                let text = commands::where_print(peer.state.origin());
                self.told.push(Told::One(client, text));
            }
        } else if is(b"voice_cmd") {
            self.voice_command(client, arguments.first().copied(), level_time);
        } else if is(b"levelshot") {
            self.level_shot(client, server_time);
        } else if is(b"giveother") {
            self.give_other(client, arguments);
        } else if is(b"killother") {
            self.kill_other(client, arguments, server_time);
        } else {
            self.the_destroyer(client, level_time);
        }
        true
    }

    /// `ClientCommand`'s gates in its order: the intermission, the cheats, the living.
    /// Whether the command was refused (and the player told why).
    fn gated(&mut self, client: usize, command: &[u8], gates: u8) -> bool {
        if gates & NO_INTERMISSION != 0 && self.refused_at_intermission(client, command) {
            return true;
        }
        if gates & CHEAT != 0 && !self.settings.cheats {
            self.told
                .push(Told::One(client, b"print \"@@@NOCHEATS\n\"".to_vec()));
            return true;
        }
        if gates & ALIVE != 0
            && self
                .peer(client)
                .is_none_or(|peer| peer.health <= 0 || !peer.playing())
        {
            self.told
                .push(Told::One(client, b"print \"@@@MUSTBEALIVE\n\"".to_vec()));
            return true;
        }
        false
    }

    /// `Cmd_ForceChanged_f`: a spectator's Force is read again at once; a player's with
    /// its next spawn (`forceDoInit`), and it is told so. Returns whether the command
    /// goes on as `Cmd_Team_f`: the menu sends the team with it, outside the duels.
    pub(super) fn force_changed(&mut self, client: usize, has_argument: bool) -> bool {
        let settings = self.settings;
        let gametype = self.gametype;
        let Some(peer) = self.peer_mut(client) else {
            return false;
        };
        if peer.session.team == i32::from(TEAM_SPECTATOR) {
            // "if it's a spec, just make the changes now", without a word: "the UI calls
            // this a lot".
            let mut force = sjk_game_jka::client_begin::read_force_powers(
                &mut peer.session,
                &peer.userinfo,
                settings,
            );
            // The profile menu and the rank (`spc`, `nfr`), or why the string was
            // refused, as a spawn sends them.
            let commands = std::mem::take(&mut force.commands);
            peer.session.force = Some(force);
            self.told
                .extend(commands.into_iter().map(|text| Told::One(client, text)));
        } else {
            peer.session.force_do_init = true;
            self.told
                .push(Told::One(client, commands::FORCE_CHANGED.to_vec()));
        }
        let duel = gametype == sjk_game_jka::match_end::GT_DUEL
            || gametype == sjk_game_jka::match_end::GT_POWERDUEL;
        has_argument && !duel
    }

    /// `Cmd_VoiceCommand_f`: one of the siege voice sounds, heard by everyone from the
    /// speaker (`EV_VOICECMD_SOUND`), in team games only.
    fn voice_command(&mut self, client: usize, argument: Option<&[u8]>, level_time: i32) {
        if self.gametype < sjk_game_jka::match_end::GT_TEAM {
            return;
        }
        let Some(argument) = argument else { return };
        if self.peer(client).is_none_or(|peer| !peer.playing()) {
            self.told
                .push(Told::One(client, commands::NO_VOICE_AS_SPECTATOR.to_vec()));
            return;
        }
        let Some(sound) = commands::voice_sound(argument) else {
            return;
        };
        let (sounds, told) = (&mut self.sounds, &mut self.told);
        let index = sounds.index(sound.as_bytes(), &mut |index, value| {
            told.push(Told::ConfigString {
                index,
                previous: Vec::new(),
                value: value.to_vec(),
            })
        });
        let mut event = EventEntity {
            event: EV_VOICECMD_SOUND,
            parameter: u32::from(index),
            origin: [0.0; 3],
            client: None,
            broadcast: true,
            extra: [(0, 0); 12],
        };
        event.extra[0] = (ES_GROUND_ENTITY, client as u32);
        let _ = self.pool.spawn_temporary(event.state(), level_time, None);
    }

    /// `Cmd_LevelShot_f`: for a player on the server's own machine, the intermission
    /// begins and its client is asked for the shot (`clientLevelShot`).
    fn level_shot(&mut self, client: usize, server_time: i32) {
        if self
            .peer(client)
            .is_none_or(|peer| !commands::is_local_address(&peer.address))
        {
            self.told
                .push(Told::One(client, commands::LEVELSHOT_NOT_LOCAL.to_vec()));
            return;
        }
        if self.gametype == GT_SINGLE_PLAYER {
            self.told.push(Told::One(
                client,
                commands::LEVELSHOT_SINGLE_PLAYER.to_vec(),
            ));
            return;
        }
        self.begin_intermission(server_time);
        self.told
            .push(Told::One(client, commands::CLIENT_LEVELSHOT.to_vec()));
    }

    /// `ClientNumberFromString` for a command naming another player, telling `client`
    /// when nobody answers to `argument`.
    fn other_player(&mut self, client: usize, argument: &[u8]) -> Option<usize> {
        let views = self.client_views();
        let found = client_number_from_string(&views, argument, false).map(|view| view.client);
        if found.is_none() {
            let text = [
                b"print \"User ".as_slice(),
                argument,
                b" is not on the server\n\"",
            ]
            .concat();
            self.told.push(Told::One(client, text));
        }
        found
    }

    /// Whether `target` is alive and in the game, as `giveother` and `killother` ask.
    fn alive_in_game(&self, target: usize) -> bool {
        self.peer(target)
            .is_some_and(|peer| peer.health > 0 && peer.playing())
    }

    /// `Cmd_GiveOther_f`: `give` for another living player.
    fn give_other(&mut self, client: usize, arguments: &[&[u8]]) {
        let [who, name, rest @ ..] = arguments else {
            self.told
                .push(Told::One(client, commands::GIVEOTHER_USAGE.to_vec()));
            return;
        };
        let Some(target) = self.other_player(client, who) else {
            return;
        };
        if !self.alive_in_game(target) {
            self.told
                .push(Told::One(client, commands::GIVEOTHER_NOT_ALIVE.to_vec()));
            return;
        }
        self.give_to(target, name, rest);
    }

    /// `Cmd_KillOther_f`: `G_Kill` for another living player.
    fn kill_other(&mut self, client: usize, arguments: &[&[u8]], server_time: i32) {
        let Some(who) = arguments.first() else {
            self.told
                .push(Told::One(client, commands::KILLOTHER_USAGE.to_vec()));
            return;
        };
        let Some(target) = self.other_player(client, who) else {
            return;
        };
        if !self.alive_in_game(target) {
            self.told
                .push(Told::One(client, b"print \"@@@MUSTBEALIVE\n\"".to_vec()));
            return;
        }
        self.kill(target, server_time);
    }

    /// `Cmd_TheDestroyer_f`: a holstered saber in hand is lit (`Cmd_ToggleSaber_f`).
    fn the_destroyer(&mut self, client: usize, level_time: i32) {
        let Some(origin) = self
            .peer(client)
            .filter(|peer| peer.state.saber_holstered() != 0 && peer.state.weapon() == WP_SABER)
            .map(|peer| peer.state.origin())
        else {
            return;
        };
        let command = sjk_protocol::UserCommand {
            generic_command: sjk_game_jka::generic_commands::GENCMD_SABERSWITCH,
            ..Default::default()
        };
        self.generic_command(client, &command, origin, level_time);
    }

    /// `G_Give` on the living player `target`: `name` and the words after it. A name
    /// that is an item's classname (`BG_FindItem`, without case) is touched as if the
    /// player had walked over it (`Touch_Item`); the reference's item entity exists only
    /// for that touch, so nothing is seen of it, nor its pickup event.
    pub(super) fn give_to(&mut self, target: usize, name: &[u8], rest: &[&[u8]]) {
        let Some(peer) = self.peer_mut(target) else {
            return;
        };
        if give(&mut peer.state, &mut peer.health, name, rest) == Given::Item {
            let item = std::str::from_utf8(name).ok().and_then(|name| {
                sjk_game_jka::items::ITEMS
                    .iter()
                    .skip(1)
                    .position(|item| item.classname.eq_ignore_ascii_case(name))
                    .map(|index| index + 1)
            });
            if let Some(item) = item
                && sjk_game_jka::items::ITEMS[item].kind != sjk_game_jka::items::Kind::Team
                && sjk_game_jka::items::can_be_grabbed(item, &peer.state, false)
            {
                let _ =
                    sjk_game_jka::items::pick_up(item, &mut peer.state, &mut peer.health, 0, false);
            }
        }
        // The weapons and the ammo are the movement's to spend: it restarts from the
        // state as given.
        peer.movement = peer.movement.reseeded(&peer.state);
        peer.movement.set_health(peer.health);
    }
}
