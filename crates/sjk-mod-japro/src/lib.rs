//! jaPRO tools: support for jaPRO (TaystJK lineage) servers as an SJK mod (`japro.*`,
//! `docs/mods.md`).
//!
//! The mod loads on every server SJK sees as jaPRO ([`sjk_mod::ServerKind::JaPro`]).
//! It owns jaPRO's movement styles ([`movement_styles`]): which styles the client's
//! prediction models, and later the ports of the others (CPM, QW...). The viewer asks
//! it through a small bridge while the mod is loaded; with the mod off or another
//! server, nothing of it applies. It also lists jaPRO's plugin options
//! (`cp_pluginDisable`, [`plugin`]), which Settings switches.

use sjk_mod::{ClientMod, Command, Host, Server, ServerKind, Setting, command};

/// jaPRO's movement styles and which this client predicts.
pub mod movement_styles;
pub mod plugin;

/// The mod's id and prefix.
pub const ID: &str = "japro";

const SETTINGS: &[Setting] = &[Setting {
    name: movement_styles::PREDICT_ANYWAY_CVAR,
    default: sjk_mod::SettingValue::Bool(false),
    help: "Predict jaPRO movement styles SJK does not model yet (CPM, QW...) as JKA; off shows them as the server has them",
}];

const COMMANDS: &[Command] = &[
    command(
        "japro.styles",
        "List jaPRO's movement styles and which ones SJK predicts",
    ),
    command(
        "japro.plugin",
        "List the jaPRO plugin options (cp_pluginDisable) or switch one: japro.plugin [number]",
    ),
];

/// Whether `server` runs jaPRO or its TaystJK lineage.
pub fn is_japro(server: &Server) -> bool {
    server.kind == ServerKind::JaPro
}

/// The jaPRO tools mod.
#[derive(Default)]
pub struct JaPro;

impl JaPro {
    pub fn new() -> Self {
        Self
    }
}

fn listing() -> Vec<String> {
    let mut lines = vec!["^5jaPRO movement styles (/move on the server):".to_owned()];
    lines.extend(movement_styles::MOVE_STYLES.iter().map(|style| {
        format!(
            "^5{:<8}^7 {}",
            style.name,
            if style.predicted {
                "predicted"
            } else {
                "shown from the server (not ported yet)"
            }
        )
    }));
    lines
}

impl ClientMod for JaPro {
    fn id(&self) -> &'static str {
        ID
    }

    fn title(&self) -> &'static str {
        "jaPRO tools"
    }

    fn about(&self) -> &'static str {
        "jaPRO movement styles (what SJK predicts, what it shows from the server) and plugin options"
    }

    fn servers(&self) -> &'static str {
        "jaPRO"
    }

    fn loads_on(&self, server: &Server) -> bool {
        is_japro(server)
    }

    fn commands(&self) -> &'static [Command] {
        COMMANDS
    }

    fn settings(&self) -> &'static [Setting] {
        SETTINGS
    }

    fn run(
        &mut self,
        name: &str,
        args: &[String],
        host: &mut dyn Host,
    ) -> Result<Vec<String>, String> {
        match name {
            "japro.styles" => Ok(listing()),
            "japro.plugin" => plugin::OPTIONS.run(args, host),
            _ => Err(format!("Unknown jaPRO tools command {name}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server(kind: ServerKind) -> Server {
        Server {
            kind,
            address: "127.0.0.1:29070".into(),
            info: String::new(),
        }
    }

    #[test]
    fn loads_on_japro_servers_only() {
        assert!(JaPro.loads_on(&server(ServerKind::JaPro)));
        assert!(!JaPro.loads_on(&server(ServerKind::JaPlus)));
        assert!(!JaPro.loads_on(&server(ServerKind::BaseJka)));
    }

    #[test]
    fn its_names_carry_the_prefix() {
        assert!(
            COMMANDS
                .iter()
                .all(|command| command.name.starts_with("japro."))
        );
        assert!(
            SETTINGS
                .iter()
                .all(|setting| setting.name.starts_with("japro."))
        );
        assert_eq!(listing().len(), movement_styles::MOVE_STYLES.len() + 1);
    }
}
