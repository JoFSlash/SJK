//! A player's jetpack, as `CG_Player` draws it (`codemp/cgame/cg_players.c`, JoF EJK
//! 11843-11935).
//!
//! Stock bolts `models/weapons2/jetpack/model.glm` into the player's Ghoul2 instance
//! at slot 3, on the body's third bolt, `*chestg` (`CG_InitJetpackGhoul2`,
//! `cg_players.c:676-678,10230-10250`). While `EF_JETPACK_ACTIVE` is set each of the
//! pack's `torso_ljet` and `torso_rjet` bolts plays `effects/boba/jet.efx` (twice
//! with `EF_JETPACK_FLAMING`), pushed back from the bolt along its game-facing axes;
//! the first frame plays `jetpackOnSound` and the first frame after plays
//! `jetpackOffSound`. The hover and fire loops are `sjk_client`'s loop adapter's.

use crate::bolt::{self, BoltMatrix};
use crate::saber::{self, Attachment};
use glam::{Quat, Vec3};
use sjk_vfs::VirtualFileSystem;
use std::cell::Cell;

/// `JETPACK_MODEL`.
pub(crate) const MODEL: &str = "models/weapons2/jetpack/model.glm";
/// `cgs.effects.mBobaJet`.
pub(crate) const EFFECT: &str = "boba/jet";
/// Both `cg_jetpackOnSound` choices and the fixed `jetpackOffSound`.
pub(crate) const SOUNDS: [&str; 3] = sjk_client::LEGACY_JETPACK_SWITCH_SOUNDS;
// Lower case, so the per-frame lookups need no allocation; registered as
// `cg_jetpackOnSound` and `cg_jetpackHoverSound` (console_cvars.rs).
pub(crate) const ON_SOUND_CVAR: &str = "cg_jetpackonsound";
pub(crate) const HOVER_SOUND_CVAR: &str = "cg_jetpackhoversound";

/// Read the shared archived choices once per audio frame, without allocating.
pub(crate) fn sound_options(
    console: Option<&crate::console::ViewerConsole>,
) -> sjk_client::LegacyJetpackSounds {
    sjk_client::LegacyJetpackSounds::from_cvars(
        console
            .and_then(|c| c.integer_cvar(ON_SOUND_CVAR))
            .unwrap_or(1),
        console
            .and_then(|c| c.integer_cvar(HOVER_SOUND_CVAR))
            .unwrap_or(1),
    )
}
/// The body bolt the pack hangs on.
const CHEST_BOLT: &str = "*chestg";
/// The pack's jet bolts, in `G2API_AddBolt` order.
const JET_BOLTS: [&str; 2] = ["torso_ljet", "torso_rjet"];
/// `MAX_GENTITIES`.
const MAX_ENTITIES: usize = 1_024;

/// The pack's mesh and its jet bolts in pack space, read once per world.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Catalog {
    pub(crate) mesh: Option<usize>,
    jets: [Option<BoltMatrix>; 2],
}

impl Catalog {
    /// Find the loaded pack and read its bolts in the bind pose, as weapon
    /// `*flash` bolts are (`object_meshes::load_one`).
    pub(crate) fn build(vfs: &VirtualFileSystem, meshes: &[crate::StaticModelMesh]) -> Self {
        let mesh = meshes
            .iter()
            .position(|mesh| mesh.appearance.variant.is_empty() && mesh.appearance.model == MODEL);
        let jets = vfs
            .read(MODEL)
            .ok()
            .flatten()
            .and_then(|asset| sjk_model::Glm::parse(&asset.bytes).ok())
            .map(|model| {
                let pose = vec![
                    [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0]
                    ];
                    model.bone_count
                ];
                JET_BOLTS.map(|name| model.surface_bolt_matrix(name, 0, &pose).ok().flatten())
            })
            .unwrap_or_default();
        Self { mesh, jets }
    }

    /// The two jet flames in world space for a pack at `origin`/`rotation`: origin
    /// and direction, as `CG_Player` moves them off the bolts.
    pub(crate) fn flames(&self, origin: Vec3, rotation: Quat) -> [Option<(Vec3, Vec3)>; 2] {
        std::array::from_fn(|jet| {
            let game = bolt::game_facing(self.jets[jet]?);
            let world = |column| rotation * Vec3::from_array(bolt::column(&game, column));
            let position = origin + world(3);
            // `NEGATIVE_Y` and `POSITIVE_X`; the left jet steps back along -Y first.
            let negative_y = -world(1).normalize_or_zero();
            let positive_x = world(0).normalize_or_zero();
            Some(if jet == 0 {
                (position - 9.5 * negative_y - 13.5 * positive_x, positive_x)
            } else {
                (position - 9.5 * positive_x - 13.5 * negative_y, negative_y)
            })
        })
    }
}

/// One actor's `*chestg` bolt, evaluated only while it wears a pack.
#[derive(Clone, Debug, Default)]
pub(crate) struct Worn {
    /// The actor wore a pack when last drawn; set by the submission.
    pub(crate) wanted: Cell<bool>,
    /// `*chestg` in the latest pose, model space.
    pub(crate) chest: Option<Attachment>,
}

impl Worn {
    /// Keep a valid resting bolt even for the shared, unanimated fallback body.
    pub(crate) fn new(preview: &crate::PlayerPreview, matrices: &[[[f32; 4]; 3]]) -> Self {
        Self {
            wanted: Cell::new(false),
            chest: saber::attachment_from_matrices(preview, matrices, CHEST_BOLT),
        }
    }

    /// Equipping happens after pose evaluation. Resolve this frame's bolt immediately
    /// on that edge, rather than waiting for another pose upload.
    pub(crate) fn attachment(
        &self,
        worn: bool,
        current_pose: impl FnOnce() -> Option<Attachment>,
    ) -> Option<Attachment> {
        let previously_worn = self.wanted.replace(worn);
        if !worn {
            None
        } else if !previously_worn {
            current_pose().or(self.chest)
        } else {
            self.chest
        }
    }
    /// Refresh the chest bolt from one evaluated pose; nothing is computed for an
    /// actor without a pack.
    pub(crate) fn update(&mut self, preview: &crate::PlayerPreview, matrices: &[[[f32; 4]; 3]]) {
        if self.wanted.get() {
            self.chest = saber::attachment_from_matrices(preview, matrices, CHEST_BOLT);
        }
    }
}

/// `centity_t::hasPlayedJetpackSounds`, per entity.
pub(crate) struct Sounds {
    played: Box<[bool]>,
}

impl Default for Sounds {
    fn default() -> Self {
        Self {
            played: vec![false; MAX_ENTITIES].into_boxed_slice(),
        }
    }
}

/// A jetpack one-shot to start.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Cue {
    On,
    Off,
}

impl Sounds {
    /// Advance one entity's pack for a drawn frame: the on sound the first frame its
    /// jets burn, the off sound the first frame after. A pack hidden by a mind trick
    /// stays silent but still keeps its state.
    pub(crate) fn advance(&mut self, entity: u16, active: bool, audible: bool) -> Option<Cue> {
        let played = self.played.get_mut(usize::from(entity))?;
        let cue = match (active, *played) {
            (true, false) => Some(Cue::On),
            (false, true) => Some(Cue::Off),
            _ => None,
        };
        *played = active;
        cue.filter(|_| audible)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sound_choices_are_archived_without_restarting_active_jets() {
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/parity-reports");
        std::fs::create_dir_all(&root).unwrap();
        let profile = tempfile::Builder::new()
            .prefix("jetpack-sounds-")
            .tempdir_in(root)
            .unwrap();
        let path = profile.path().join("config.cfg");
        let mut console = crate::console::ViewerConsole::new(path.clone()).unwrap();
        assert_eq!(
            sound_options(Some(&console)),
            sjk_client::LegacyJetpackSounds::from_cvars(1, 1)
        );
        let mut played = Sounds::default();
        assert_eq!(played.advance(7, true, true), Some(Cue::On));
        console.try_set_cvar(ON_SOUND_CVAR, "2").unwrap();
        console.try_set_cvar(HOVER_SOUND_CVAR, "2").unwrap();
        assert_eq!(
            sound_options(Some(&console)),
            sjk_client::LegacyJetpackSounds::from_cvars(2, 2)
        );
        assert_eq!(played.advance(7, true, true), None);
        console.flush_config();
        let console = crate::console::ViewerConsole::new(path).unwrap();
        assert_eq!(
            sound_options(Some(&console)),
            sjk_client::LegacyJetpackSounds::from_cvars(2, 2)
        );
        assert_eq!(played.advance(7, false, true), Some(Cue::Off));
    }

    #[test]
    fn equip_resolves_the_current_pose_without_waiting_for_upload() {
        let resting = Attachment {
            grip: [0.0; 3],
            rotation: Quat::IDENTITY.to_array(),
        };
        let current = Attachment {
            grip: [1.0, 2.0, 3.0],
            ..resting
        };
        let worn = Worn {
            chest: Some(resting),
            ..Worn::default()
        };
        assert_eq!(
            worn.attachment(true, || Some(current)).unwrap().grip,
            current.grip
        );
        assert!(
            worn.attachment(false, || panic!("unequipped pack never resolves a bolt"))
                .is_none()
        );
        assert_eq!(worn.attachment(true, || None).unwrap().grip, resting.grip);
        assert_eq!(
            worn.attachment(true, || panic!("cached pose reused"))
                .unwrap()
                .grip,
            resting.grip
        );
    }

    #[test]
    #[ignore = "requires installed game data and a GPU"]
    fn equipped_pack_on_duel6() {
        crate::world_shot::on_big_stack(|| {
            use sjk_runtime::{Appearance, EntityId, EntityKind, MotionSample, Transform};
            let (mut input, _profile) =
                crate::world_shot::world_input("maps/mp/duel6.bsp", None, &[]);
            // Keep the reference asset independent of personal replacement PK3s.
            let mut vfs = VirtualFileSystem::new();
            for name in ["assets0.pk3", "assets1.pk3", "assets2.pk3", "assets3.pk3"] {
                vfs.mount_pk3(input.game_data.join("base").join(name))
                    .unwrap();
            }
            input.shaders = crate::assets::load_shaders(&vfs);
            input.vfs = std::sync::Arc::new(vfs);
            input.build_game_state = Some(sjk_protocol::GameState::empty_local(0));
            let mut gpu =
                pollster::block_on(crate::GpuState::new_for_target(None, [1280, 720], input))
                    .expect("GPU startup");
            gpu.is_menu_world = true;
            crate::world_shot::attach_target(&mut gpu, [1280, 720]);
            let (spawn, yaw) = crate::assets::initial_camera(&gpu.bsp).unwrap();
            let turn = Quat::from_rotation_z(yaw);
            let origin = Vec3::from_array(spawn) + turn * Vec3::new(85.0, 0.0, -32.0);
            let body = Transform {
                translation: origin.to_array(),
                rotation: turn.to_array(),
                scale: [1.0; 3],
            };
            gpu.live_world.upsert(
                EntityId::new(1),
                EntityKind::Actor,
                MotionSample {
                    time_millis: 0,
                    transform: body,
                },
            );
            let fallback = gpu
                .actor_meshes
                .iter()
                .find(|actor| actor.entity_id.is_none() && !actor.corpse_pool)
                .unwrap();
            let chest = fallback
                .jetpack
                .attachment(true, || None)
                .expect("fallback pack attaches on its first frame");
            let (pack_origin, pack_rotation) = saber::world_attachment(
                origin,
                crate::weapon_view::actor_world_rotation(body.rotation),
                chest,
            );
            let mesh = gpu.jetpack.mesh.expect("stock jetpack mesh");
            assert!(
                gpu.jetpack.jets.iter().all(Option::is_some),
                "stock flame bolts"
            );
            let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/parity-reports");
            std::fs::create_dir_all(&directory).unwrap();
            let before = crate::world_shot::frame(&mut gpu, 1);
            before
                .save(directory.join("jetpack-unequipped.png"))
                .unwrap();
            gpu.live_world.upsert(
                EntityId::new(2),
                EntityKind::Other,
                MotionSample {
                    time_millis: 0,
                    transform: Transform {
                        translation: pack_origin.to_array(),
                        rotation: pack_rotation.to_array(),
                        scale: [1.0; 3],
                    },
                },
            );
            gpu.live_world.set_appearance(
                EntityId::new(2),
                Some(Appearance {
                    model: MODEL.to_owned(),
                    variant: String::new(),
                }),
            );
            let after = crate::world_shot::frame(&mut gpu, 1);
            assert_eq!(
                gpu.object_groups[mesh].len(),
                1,
                "pack reached the actual model pass"
            );
            assert!(
                before
                    .pixels()
                    .zip(after.pixels())
                    .filter(|(a, b)| a != b)
                    .count()
                    > 100
            );
            after.save(directory.join("jetpack-equipped.png")).unwrap();
            let definition = gpu
                .effects
                .definition(gpu.vfs.as_ref().unwrap(), EFFECT)
                .unwrap();
            let missing: Vec<_> = definition
                .components
                .iter()
                .flat_map(|component| component.shaders.iter())
                .filter(|name| {
                    !gpu.particle_atlas
                        .animations
                        .contains_key(&name.to_ascii_lowercase())
                })
                .collect();
            assert!(
                missing.is_empty(),
                "jetpack shaders fell back instead of being loaded: {missing:?}"
            );
            let now = std::time::Instant::now();
            for millis in (0..=200).step_by(8) {
                for (jet, flame) in gpu
                    .jetpack
                    .flames(pack_origin, pack_rotation)
                    .into_iter()
                    .enumerate()
                {
                    let (position, direction) = flame.unwrap();
                    for copy in 0..2 {
                        crate::effect_runtime::spawn_effect(
                            &mut gpu.particles,
                            &mut gpu.effect_aux,
                            &mut gpu.effects,
                            gpu.vfs.as_ref().unwrap(),
                            EFFECT,
                            position,
                            now - std::time::Duration::from_millis(200 - millis),
                            (millis as u32).wrapping_mul(7) + jet as u32 * 2 + copy,
                            0,
                            &mut None,
                            crate::combat_effects::rotation_from_direction(direction.to_array()),
                        );
                    }
                }
            }
            let exhaust = crate::world_shot::frame(&mut gpu, 1);
            assert!(
                gpu.particles.iter().any(|p| p.length.is_some()),
                "exhaust tails reached the effect pass"
            );
            exhaust.save(directory.join("jetpack-exhaust.png")).unwrap();
            let atlas = &mut gpu.particle_atlas;
            atlas
                .refresh_remaps(
                    &gpu.device,
                    &gpu.queue,
                    gpu.vfs.as_ref().unwrap(),
                    &gpu.shaders,
                    u64::MAX,
                    |name| {
                        (
                            if name == "gfx/effects/flamejet" {
                                "clear"
                            } else {
                                name
                            }
                            .to_owned(),
                            0.0,
                        )
                    },
                )
                .unwrap();
            assert!(
                atlas
                    .animations
                    .get("gfx/effects/flamejet")
                    .is_some_and(Vec::is_empty)
            );
            assert_eq!(
                atlas.layers_for("gfx/effects/flamejet", 0.0).iter().count(),
                0,
                "a suppressed exhaust shader must not become a fallback spark"
            );
            assert!(atlas.first_layer("gfx/effects/flamejet", 0.0).is_none());
            assert_eq!(
                atlas.layers_for("gfx/test/missing", 0.0).iter().count(),
                1,
                "a genuinely missing shader retains its fallback"
            );
        });
    }

    #[test]
    fn on_then_off_once_each() {
        let mut sounds = Sounds::default();
        assert_eq!(sounds.advance(7, true, true), Some(Cue::On));
        assert_eq!(sounds.advance(7, true, true), None);
        assert_eq!(sounds.advance(7, false, true), Some(Cue::Off));
        assert_eq!(sounds.advance(7, false, true), None);
    }

    #[test]
    fn a_tricked_pack_is_silent_but_tracked() {
        let mut sounds = Sounds::default();
        assert_eq!(sounds.advance(3, true, false), None);
        assert_eq!(sounds.advance(3, true, true), None);
        assert_eq!(sounds.advance(3, false, true), Some(Cue::Off));
    }

    #[test]
    fn flames_step_back_from_their_bolts() {
        let identity = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
        ];
        let catalog = Catalog {
            mesh: None,
            jets: [Some(identity); 2],
        };
        // Game facing turns raw -Y into +X and raw +X into +Y: POSITIVE_X is -Y here and
        // NEGATIVE_Y is -X.
        let [left, right] = catalog.flames(Vec3::ZERO, Quat::IDENTITY);
        let (origin, direction) = left.unwrap();
        assert!(direction.abs_diff_eq(-Vec3::Y, 1e-6));
        assert!(origin.abs_diff_eq(Vec3::new(9.5, 13.5, 0.0), 1e-5));
        let (origin, direction) = right.unwrap();
        assert!(direction.abs_diff_eq(-Vec3::X, 1e-6));
        assert!(origin.abs_diff_eq(Vec3::new(13.5, 9.5, 0.0), 1e-5));
    }
}
