//! Starting and joining the client's own server (Create game): the binary
//! and the log are found here, the process is owned by the menu, and the
//! join goes through the ordinary address join once the server answers.

use super::local_server::{self, HostSettings, LocalServer};

#[path = "local_devmap.rs"]
mod devmap;

impl crate::GpuState {
    /// Start `sjk-server` for `settings`; the menu follows its start-up.
    pub(crate) fn start_local_game(&mut self, settings: HostSettings) {
        let Some(menu) = &mut self.client_menu else {
            return;
        };
        let Some(program) = local_server::locate_server_binary() else {
            menu.local_server_failed(
                "sjk-server was not found beside the client or on PATH (set JKA_DEDICATED).",
            );
            return;
        };
        let config_directory = self.console.as_ref().map_or_else(
            || {
                crate::platform::user_config_file()
                    .ok()
                    .and_then(|file| file.parent().map(std::path::Path::to_path_buf))
                    .unwrap_or_else(std::env::temp_dir)
            },
            |console| console.config_directory().to_path_buf(),
        );
        let log = local_server::server_log_path(
            std::env::var_os(local_server::SERVER_LOG_ENV).map(Into::into),
            local_server::client_log_file(),
            &config_directory,
        );
        let port = local_server::choose_port(&settings);
        let mut arguments = settings.arguments(&self.game_data, port);
        // A listen server shares the console's server cvars, as in EternalJK; the
        // owned server is a separate process, so they are handed over at launch.
        let shared = hosted_cvars(self.console.as_ref());
        for (name, value) in &shared {
            arguments.extend(["--set".into(), (*name).into(), value.to_string().into()]);
        }
        crate::log::progress(format_args!(
            "local server: {} {}",
            program.display(),
            arguments
                .iter()
                .map(|argument| argument.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ")
        ));
        match LocalServer::start(&program, &arguments, settings.console_lines(), &log) {
            Ok(mut server) => {
                for (name, value) in shared {
                    server.share_cvar(name, value);
                }
                // Detach before replacing the old owned child, so its disconnect
                // cannot be mistaken for a failure of the new local game.
                self.leave_session();
                self.game_menu = false;
                self.gameplay_input.clear();
                self.release_pointer();
                if let Some(console) = &mut self.console {
                    console.close_for_connection();
                }
                if let Some(menu) = &mut self.client_menu {
                    menu.local_server_started(server, &settings.map);
                    menu.state_loading(&settings.map);
                }
            }
            Err(error) => {
                menu.local_server_failed(&format!("Could not start {}: {error}", program.display()))
            }
        }
    }

    /// Type into the hosted server the server cvars changed in the console since
    /// (each frame).
    pub(crate) fn share_hosted_cvars(&mut self) {
        let shared = hosted_cvars(self.console.as_ref());
        if let Some(server) = self
            .client_menu
            .as_mut()
            .and_then(crate::menu::ClientMenu::local_server_mut)
        {
            for (name, value) in shared {
                server.share_cvar(name, value);
            }
        }
    }

    /// Join the hosted server once it answers (each frame).
    pub(crate) fn join_local_game(&mut self) {
        if let Some(address) = self
            .client_menu
            .as_mut()
            .and_then(crate::menu::ClientMenu::take_local_join)
        {
            self.begin_address_join(address);
        }
    }
}

/// The console's values of the hosted server's cvars.
fn hosted_cvars(console: Option<&crate::console::ViewerConsole>) -> Vec<(&'static str, i64)> {
    let Some(console) = console else {
        return Vec::new();
    };
    local_server::HOSTED_CVARS
        .into_iter()
        .filter_map(|name| Some((name, console.integer_cvar(name)?)))
        .collect()
}
