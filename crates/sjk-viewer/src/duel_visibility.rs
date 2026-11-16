//! Local world visibility during a private duel.

/// Archived player preference for hiding the players outside the duel, and what they
/// bring, while the local (or followed) player is in a private duel
/// ([`sjk_client::DuelHideMode`]): 0 never, 1 on the servers that expect it (JoF EJK's
/// rule, the default), 2 on every server.
pub(crate) const HIDE_OTHERS_CVAR: &str = "cg_duelHideOthers";

/// Archived player preference for hiding `ET_BODY` corpses while the local
/// player is in a private duel.
pub(crate) const CORPSES_CVAR: &str = "cg_hideCorpsesInDuel";

/// Archived player preference for hiding `ET_ITEM` and `ET_HOLOCRON` pickups
/// while the local player is in a private duel.
pub(crate) const PICKUPS_CVAR: &str = "cg_hidePickupsInDuel";

/// One presented snapshot without what [`sjk_client::DuelHide`] hides, made again only
/// when the snapshot or the setting changes.
#[derive(Default)]
pub(crate) struct HiddenView {
    key: Option<(i32, i32, sjk_client::DuelHideSettings)>,
    snapshot: Option<sjk_protocol::Snapshot>,
}

impl HiddenView {
    /// Follow `presented` (the snapshot and its game state) under `settings`.
    pub(crate) fn update(
        &mut self,
        presented: Option<(&sjk_protocol::Snapshot, &sjk_protocol::GameState)>,
        settings: sjk_client::DuelHideSettings,
    ) {
        let Some((snapshot, game)) = presented else {
            self.key = None;
            self.snapshot = None;
            return;
        };
        let key = (snapshot.message_sequence, snapshot.server_time, settings);
        if self.key != Some(key) {
            self.key = Some(key);
            self.snapshot = sjk_client::duel_hidden_snapshot(snapshot, game, settings);
        }
    }

    /// The snapshot without the hidden players, when some are hidden.
    pub(crate) fn get(&self) -> Option<&sjk_protocol::Snapshot> {
        self.snapshot.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjk_client::{DuelHideMode, DuelHideSettings};
    use sjk_protocol::{EntityState, GameState, LEGACY_ENTITY_FIELDS, PlayerState, Snapshot};

    fn player(number: u16) -> EntityState {
        let mut state = EntityState::zero(number, &LEGACY_ENTITY_FIELDS);
        state.set_raw_field(8, 1);
        state
    }

    /// Client 0 duelling client 1, with client 2 looking on.
    fn duel(sequence: i32) -> Snapshot {
        let mut player_state = PlayerState::default();
        player_state.set_raw_field(119, 1);
        player_state.set_raw_field(44, 1);
        Snapshot {
            message_sequence: sequence,
            reliable_acknowledge: 0,
            server_commands: Vec::new(),
            server_time: 1_000 + sequence,
            delta_from: None,
            flags: 0,
            area_mask: Vec::new(),
            player: player_state,
            vehicle_player: None,
            entities: vec![player(1), player(2)],
            consumed_bits: 0,
        }
    }

    #[test]
    fn the_cvar_and_the_plugin_bits_make_the_setting() {
        let directory = tempfile::tempdir().unwrap();
        let mut console =
            crate::console::ViewerConsole::new(directory.path().join("config.cfg")).unwrap();
        assert_eq!(console.duel_hide(), DuelHideSettings::default());
        assert!(console.set_cvar(HIDE_OTHERS_CVAR, "2"));
        assert!(console.set_cvar("cp_pluginDisable", "2"));
        assert_eq!(
            console.duel_hide(),
            DuelHideSettings {
                mode: DuelHideMode::Always,
                plugin_disable: 2,
            }
        );
        assert!(console.set_cvar(HIDE_OTHERS_CVAR, "0"));
        assert_eq!(console.duel_hide().mode, DuelHideMode::Off);
    }

    #[test]
    fn the_frame_view_drops_bystanders_and_follows_the_setting() {
        let game = GameState::empty_local(0);
        let always = DuelHideSettings {
            mode: DuelHideMode::Always,
            plugin_disable: 0,
        };
        let mut view = HiddenView::default();
        let snapshot = duel(1);
        view.update(Some((&snapshot, &game)), always);
        let numbers: Vec<u16> = view
            .get()
            .unwrap()
            .entities
            .iter()
            .map(EntityState::number)
            .collect();
        assert_eq!(numbers, [1]);
        // Auto on a server no family lists: the snapshot as received.
        view.update(Some((&snapshot, &game)), DuelHideSettings::default());
        assert!(view.get().is_none());
        view.update(Some((&duel(2), &game)), always);
        assert!(view.get().is_some());
        view.update(None, always);
        assert!(view.get().is_none());
    }
}
