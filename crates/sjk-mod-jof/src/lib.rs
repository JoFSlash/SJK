//! JoF tools: support for the JoF community's JA+ servers as an SJK mod (`jof.*`,
//! `docs/mods.md`).
//!
//! A JoF server is one of the `jof` family in SJK's table of known servers
//! (`sjk_mod::known_servers`): a JA+ server whose serverinfo `V` is `2.5B0`. Its extra
//! commands run on the server, which checks every right: the mod completes them
//! (`gcmds[]`, `cgame/cg_consolecmds.c:3262-3303`) and lists them with
//! `jof.commands`.

use sjk_mod::{ClientMod, Command, Host, Server, Setting, UserinfoField, command};
/// Destruction's community-owned capability and wheel metadata.
pub mod destruction;
/// Bounded JoF dialogue command/state/response contract.
pub mod dialogue;
/// JoF server firing-confirmation contracts and repeat policy.
pub mod feedback;
/// SJK chat said in All chat on JoF servers, and the notice that tells the player.
pub mod sjk_chat;
/// Player opt-in to the JoF EJK server extension, separate from client identity.
pub const EJK_OPT_IN: &str = "jof.ejkOptIn";
/// Local confirmation sound preference, kept while the mod is unloaded.
pub const ABILITY_SOUNDS: &str = "jof.abilitySounds";
/// Encrypted whispers on JoF servers, off by default: JoF moderates whispers there
/// (the client's `e2e_whispers::PLAIN_FAMILIES` reads it, whatever the mod's switch).
pub const E2E_WHISPERS: &str = "jof.e2eWhispers";
/// Medals called "Honors" and holocrons "Echoes" everywhere while the mod is on, on by
/// default (JoF's request; the client's `terms` module reads it).
pub const HONOR_NAMES: &str = "jof.honorNames";
const SETTINGS: &[Setting] = &[
    Setting {
        name: destruction::CVAR,
        default: sjk_mod::SettingValue::Integer(-1),
        help: "JoF Destruction support: -1 server flag, 0 off, 1 grant without flag; requires loaded JoF tools",
    },
    Setting {
        name: dialogue::CVAR,
        default: sjk_mod::SettingValue::Bool(true),
        help: "Show JoF server dialogue and installed node voices while JoF tools are loaded",
    },
    Setting {
        name: EJK_OPT_IN,
        default: sjk_mod::SettingValue::Bool(true),
        help: "Advertise jofejk on JoF servers while JoF tools are enabled; enables server EJK extensions",
    },
    Setting {
        name: ABILITY_SOUNDS,
        default: sjk_mod::SettingValue::Bool(true),
        help: "Play installed Stasis/Repulse confirmation sounds from a JoF server",
    },
    Setting {
        name: E2E_WHISPERS,
        default: sjk_mod::SettingValue::Bool(false),
        help: "Send and take encrypted whispers on JoF servers, where JoF asks that whispers go through their server so they can moderate them",
    },
    Setting {
        name: HONOR_NAMES,
        default: sjk_mod::SettingValue::Bool(true),
        help: "Call medals \"Honors\" and holocrons \"Echoes\" in every menu and message while JoF tools are on",
    },
    Setting {
        name: sjk_chat::NOTICE,
        default: sjk_mod::SettingValue::Bool(true),
        help: "Show the card saying SJK chat is said in All chat on the next JoF server",
    },
];
const USERINFO: &[UserinfoField] = &[UserinfoField {
    key: "jofejk",
    value: "1",
    enabled_by: EJK_OPT_IN,
}];

/// The mod's id and prefix.
pub const ID: &str = "jof";

const COMMANDS: &[Command] = &[command(
    "jof.commands",
    "List the JoF server commands, and whether this server is a JoF one",
)];

const SERVER_COMMANDS: &[Command] = &[
    command(destruction::COMMAND, "JoF: use granted Force Destruction"),
    command(
        "gunorigin",
        "JoF admin: print the position of the player you aim at",
    ),
    command("gunprotect", "JoF admin: amprotect the player you aim at"),
    command(
        "gunshowmotd",
        "JoF admin: amshowmotd to the player you aim at",
    ),
    command("gunsleep", "JoF admin: amsleep the player you aim at"),
    command("gunwake", "JoF admin: amwake the player you aim at"),
    command("gunslap", "JoF admin: amslap the player you aim at"),
    command("gunkick", "JoF admin: amkick the player you aim at"),
    command("gunban", "JoF admin: amban the player you aim at"),
    command(
        "gunmindtrick",
        "JoF admin: ammindtrick the player you aim at",
    ),
    command("gunsilence", "JoF admin: amsilence the player you aim at"),
    command(
        "gununsilence",
        "JoF admin: amunsilence the player you aim at",
    ),
    command("gunempower", "JoF admin: amempower the player you aim at"),
    command("gunmerc", "JoF admin: ammerc the player you aim at"),
    command(
        "gunforcealtdim",
        "JoF admin: amforcealtdim the player you aim at",
    ),
    command(
        "gununforcealtdim",
        "JoF admin: amunforcealtdim the player you aim at",
    ),
    command("amorigin", "JoF admin: print a player's position"),
    command("amunsilence", "JoF admin: let a silenced player chat again"),
    command("jetpack", "JoF: jetpack"),
    command("toggleflame", "JoF: flame effect"),
    command("refusetele", "JoF: refuse admin teleports"),
    command("immortal", "JoF: immortal mode"),
    command("sleep", "JoF: sleep"),
    command("amhelp", "JoF: the JoF guide"),
    command("amtips", "JoF: tips"),
    command("amemotes", "JoF: list the emotes"),
    command("amshop", "JoF: the in-game shop"),
    command("amcredits", "JoF: the credits leaderboard"),
];

/// The family in `sjk_mod::known_servers` this mod is made for.
pub const FAMILY: &str = "jof";

/// Whether `server` is a JoF JA+ server.
pub fn is_jof(server: &Server) -> bool {
    server.family().is_some_and(|family| family.id == FAMILY)
}

/// The JoF tools mod.
#[derive(Default)]
pub struct JoF;

impl JoF {
    pub fn new() -> Self {
        Self
    }
}

fn listing(server: Option<&Server>) -> Vec<String> {
    let mut lines = vec![match server {
        Some(server) if is_jof(server) => "^5This is a JoF server.".to_owned(),
        Some(_) => "^3This is not a JoF server: these commands will not work here.".to_owned(),
        None => "^3Not connected; JoF servers accept these commands:".to_owned(),
    }];
    lines.extend(
        SERVER_COMMANDS
            .iter()
            .map(|command| format!("^5{:<18}^7 {}", command.name, command.help)),
    );
    lines
}

impl ClientMod for JoF {
    fn id(&self) -> &'static str {
        ID
    }

    fn title(&self) -> &'static str {
        "JoF tools"
    }

    fn about(&self) -> &'static str {
        "Completion and help for the JoF community's JA+ server commands"
    }

    fn servers(&self) -> &'static str {
        "JoF"
    }

    fn loads_on(&self, server: &Server) -> bool {
        is_jof(server)
    }

    fn commands(&self) -> &'static [Command] {
        COMMANDS
    }

    fn settings(&self) -> &'static [Setting] {
        SETTINGS
    }

    fn userinfo_fields(&self) -> &'static [UserinfoField] {
        USERINFO
    }

    fn server_commands(&self, server: &Server) -> &'static [Command] {
        if is_jof(server) { SERVER_COMMANDS } else { &[] }
    }

    fn run(
        &mut self,
        name: &str,
        _args: &[String],
        host: &mut dyn Host,
    ) -> Result<Vec<String>, String> {
        match name {
            "jof.commands" => Ok(listing(host.server().as_ref())),
            _ => Err(format!("Unknown JoF tools command {name}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjk_mod::ServerKind;

    fn server(kind: ServerKind, info: &str) -> Server {
        Server {
            kind,
            address: "127.0.0.1:29070".into(),
            info: info.into(),
        }
    }

    #[test]
    fn a_jof_server_is_ja_plus_2_5b0() {
        assert!(is_jof(&server(ServerKind::JaPlus, r"\V\2.5B0")));
        assert!(!is_jof(&server(ServerKind::JaPlus, r"\V\2.4B7")));
        assert!(!is_jof(&server(ServerKind::JaPro, r"\V\2.5B0")));
        assert_eq!(
            JoF.server_commands(&server(ServerKind::JaPlus, r"\V\2.5B0"))
                .len(),
            SERVER_COMMANDS.len()
        );
        assert!(
            JoF.server_commands(&server(ServerKind::BaseJka, ""))
                .is_empty()
        );
    }

    #[test]
    fn its_names_carry_the_prefix() {
        assert!(
            COMMANDS
                .iter()
                .all(|command| command.name.starts_with("jof."))
        );
        assert!(listing(None).len() == SERVER_COMMANDS.len() + 1);
    }
}
