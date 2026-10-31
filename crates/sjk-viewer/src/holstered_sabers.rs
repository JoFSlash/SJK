//! JoF EJK #42/#81/#95/#267/#300: stowed hilt policy, with shared catalog meshes.
//! JA+ meanings live in CONTRACTS; `cg_holsteredSaberSupport` overrides detection.
use crate::console::ViewerConsole;
use sjk_protocol::GameState;

#[path = "holstered_saber_pose.rs"]
pub(crate) mod pose;
#[path = "holstered_staff.rs"]
pub(crate) mod staff;

pub(crate) const CVAR: &str = "cg_drawholsteredsaber";
pub(crate) const SUPPORT_CVAR: &str = "cg_holsteredsabersupport";
pub(crate) const SWAP_CVAR: &str = "cg_holsteredstaffswap";

/// Server-specific evidence: JoF EJK f21eee8, CG_HolsteredSaberVisible and
/// CG_StaffHolsteredOnBack. JA+ repurposes EF_NOT_USED_3 as no owned saber;
/// plugin disable bits 29 (`JAPRO_PLUGIN_HOLSTEREDSABERS`) and 9
/// (`JAPRO_PLUGIN_HOLSTEREDSABER`) suppress all holsters and staff back reach. Bit 13
/// is JA+'s single-player cartwheel, a server move, not a holster switch.
struct Contract {
    matches: fn(&str) -> bool,
    absent_flag: u32,
    disable_all: i64,
    disable_back: i64,
    npc_classes: &'static [u8],
}
const CONTRACTS: &[Contract] = &[Contract {
    // JA+ proper only, as the flamethrower's contract: abyssmod's and UA Galaxy's
    // ownership flags are unverified.
    matches: crate::actor_world_submission::force_powers::flame::ja_plus_itself,
    absent_flag: 0x1000,
    disable_all: 1 << 29,
    disable_back: 1 << 9,
    // teams.h: Desann, Jedi, Kyle, Luke, Reborn, Tavion; EJK #267 visibility list.
    npc_classes: &[6, 18, 19, 22, 37, 47],
}];

/// The serverinfo part of the policy: which contract the gamename matches, read
/// again only after the configstring dispatch saw slot 0 change.
#[derive(Clone, Copy, Default)]
pub(crate) struct ServerCache {
    valid: bool,
    detected: Option<usize>,
}

impl ServerCache {
    /// Called by the configstring dispatch for `CS_SERVERINFO`.
    pub(crate) fn invalidate(&mut self) {
        self.valid = false;
    }

    fn detected(&mut self, game: Option<&GameState>) -> Option<usize> {
        let game = game?;
        if !self.valid {
            let name = game
                .config_string(0)
                .and_then(|info| sjk_client::LegacyClientInfo::new(info).text("gamename"))
                .unwrap_or("");
            *self = Self {
                valid: true,
                detected: detect(name),
            };
        }
        self.detected
    }
}

/// The contract a gamename matches, by its table row.
fn detect(name: &str) -> Option<usize> {
    CONTRACTS.iter().position(|entry| (entry.matches)(name))
}

#[derive(Clone, Copy)]
pub(crate) struct Policy {
    pub(crate) enabled: bool,
    pub(crate) back: bool,
    pub(crate) swap: f32,
    absent_flag: u32,
    npc_classes: &'static [u8],
}

impl Policy {
    /// 0 disables compatibility support, 1 detects it, 2 forces the named contract.
    /// The cvars are read each frame, the serverinfo through `server`.
    pub(crate) fn read(
        console: Option<&ViewerConsole>,
        server: &mut ServerCache,
        game: Option<&GameState>,
    ) -> Self {
        let integer = |name, fallback| {
            console
                .and_then(|c| c.integer_cvar(name))
                .unwrap_or(fallback)
        };
        Self::resolve(
            console.and_then(|c| c.bool_cvar(CVAR)).unwrap_or(true),
            integer(SUPPORT_CVAR, 1),
            server.detected(game),
            integer("cp_plugindisable", 1536),
            console
                .and_then(|c| c.float_cvar(SWAP_CVAR))
                .unwrap_or(0.45) as f32,
        )
    }

    fn resolve(
        enabled: bool,
        support: i64,
        detected: Option<usize>,
        disabled: i64,
        swap: f32,
    ) -> Self {
        let index = match support {
            1 => detected,
            2.. => usize::try_from(support - 2).ok(),
            _ => None,
        };
        let contract = index.and_then(|index| CONTRACTS.get(index));
        Self {
            enabled: enabled && contract.is_some_and(|c| disabled & c.disable_all == 0),
            back: contract.is_some_and(|c| disabled & c.disable_back == 0),
            absent_flag: contract.map_or(0, |c| c.absent_flag),
            npc_classes: contract.map_or(&[], |c| c.npc_classes),
            swap: if swap.is_finite() {
                swap.clamp(0.0, 1.0)
            } else {
                0.45
            },
        }
    }

    pub(crate) fn visible(self, body: Body) -> bool {
        self.enabled && body.owned && !body.dead && !body.mounted && !body.hidden
            && body.flags & (self.absent_flag | 0x100) == 0 // EF_NODRAW
            && (!body.cloaked || (body.sight && !body.viewed_player))
            && (!body.viewed_player || body.third_person)
    }
    pub(crate) fn allows_subject(self, entity_type: u8, npc_class: u8) -> bool {
        entity_type == 1 || (entity_type == 13 && self.npc_classes.contains(&npc_class))
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct Body {
    pub(crate) owned: bool,
    pub(crate) dead: bool,
    pub(crate) mounted: bool,
    pub(crate) hidden: bool,
    pub(crate) flags: u32,
    pub(crate) cloaked: bool,
    pub(crate) sight: bool,
    pub(crate) viewed_player: bool,
    pub(crate) third_person: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn the_archived_menu_switch_reaches_the_frame_policy() {
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/parity-reports");
        std::fs::create_dir_all(&root).unwrap();
        let profile = tempfile::Builder::new()
            .prefix("holsters-")
            .tempdir_in(root)
            .unwrap();
        let path = profile.path().join("config.cfg");
        let mut console = ViewerConsole::new(path.clone()).unwrap();
        console.try_set_cvar(SUPPORT_CVAR, "2").unwrap();
        let mut server = ServerCache::default();
        assert!(Policy::read(Some(&console), &mut server, None).enabled);
        console.try_set_cvar(CVAR, "0").unwrap();
        assert!(!Policy::read(Some(&console), &mut server, None).enabled);
        console.flush_config();
        let console = ViewerConsole::new(path).unwrap();
        assert!(!Policy::read(Some(&console), &mut server, None).enabled);
    }
    #[test]
    fn contract_detection_overrides_and_plugin_bits() {
        assert!(!Policy::resolve(true, 1, detect("basejka"), 0, 0.45).enabled);
        assert!(Policy::resolve(true, 1, detect("JA+ Mod"), 0, 0.45).enabled);
        assert!(Policy::resolve(true, 2, detect("basejka"), 0, 0.45).enabled);
        assert!(!Policy::resolve(true, 3, None, 0, 0.45).enabled);
        assert!(!Policy::resolve(true, 0, detect("JA+ Mod"), 0, 0.45).enabled);
        assert!(!Policy::resolve(true, 2, None, 1 << 29, 0.45).enabled);
        // Bit 13 (no single-player cartwheel) leaves the holsters alone.
        assert!(Policy::resolve(true, 2, None, 1 << 13, 0.45).enabled);
        assert!(!Policy::resolve(true, 2, None, 512, 0.45).back);
        assert!(!Policy::resolve(false, 2, None, 0, 0.45).enabled);
    }
    #[test]
    fn the_server_part_is_cached_until_serverinfo_changes() {
        let mut game = GameState::empty_local(0);
        game.replace_config_string(0, br"\gamename\JA+ Mod".to_vec())
            .unwrap();
        let mut server = ServerCache::default();
        assert_eq!(server.detected(Some(&game)), Some(0));
        game.replace_config_string(0, br"\gamename\basejka".to_vec())
            .unwrap();
        assert_eq!(server.detected(Some(&game)), Some(0), "cached");
        server.invalidate();
        assert_eq!(server.detected(Some(&game)), None);
        assert_eq!(server.detected(None), None);
    }
    #[test]
    fn visibility_never_reveals_hidden_or_first_person_hilts() {
        let p = Policy::resolve(true, 2, None, 0, 0.45);
        let b = Body {
            owned: true,
            ..Body::default()
        };
        assert!(p.visible(b));
        for hidden in [
            Body { owned: false, ..b },
            Body { dead: true, ..b },
            Body { mounted: true, ..b },
            Body { hidden: true, ..b },
            Body { flags: 0x1000, ..b },
            Body { flags: 0x100, ..b },
            Body { cloaked: true, ..b },
            Body {
                viewed_player: true,
                ..b
            },
        ] {
            assert!(!p.visible(hidden));
        }
        assert!(p.visible(Body {
            cloaked: true,
            sight: true,
            ..b
        }));
        assert!(!p.visible(Body {
            cloaked: true,
            sight: true,
            viewed_player: true,
            third_person: true,
            ..b
        }));
        assert!(p.visible(Body {
            viewed_player: true,
            third_person: true,
            ..b
        }));
    }
}
