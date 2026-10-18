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
        // A listen server shares the console's `g_dismember`, as in EternalJK; the
        // owned server is a separate process, so it is handed over at launch.
        if let Some(chance) = self
            .console
            .as_ref()
            .and_then(|console| console.integer_cvar(crate::dismember::SERVER_CVAR))
            .filter(|chance| *chance > 0)
        {
            arguments.extend([
                "--set".into(),
                crate::dismember::SERVER_CVAR.into(),
                chance.to_string().into(),
            ]);
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
        // The process is created on a helper thread; the current game is kept
        // until it exists, so a server that cannot start costs the player nothing.
        match LocalServer::start(&program, &arguments, settings.console_lines(), &log) {
            Ok(server) => menu.local_server_spawning(server, &settings.map),
            Err(error) => {
                menu.local_server_failed(&format!("Could not start {}: {error}", program.display()))
            }
        }
    }

    /// Leave the current game for the hosted server `server`, whose process exists.
    fn adopt_local_server(&mut self, server: LocalServer, map: &str) {
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
            menu.local_server_started(server, map);
            menu.state_loading(map);
        }
    }

    /// Take over the hosted server once its process exists, and join it once it
    /// answers (each frame).
    pub(crate) fn join_local_game(&mut self) {
        if let Some((server, map)) = self
            .client_menu
            .as_mut()
            .and_then(crate::menu::ClientMenu::take_spawned_local_server)
        {
            self.adopt_local_server(server, &map);
        }
        if let Some(address) = self
            .client_menu
            .as_mut()
            .and_then(crate::menu::ClientMenu::take_local_join)
        {
            self.begin_address_join(address);
        }
    }
}
