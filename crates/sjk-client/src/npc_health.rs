//! JoF EJK #80's client-side NPC health disclosure rule (`CG_DrawCrosshair`).

use sjk_protocol::{EntityState, PlayerState};

const MAX_CLIENTS: u16 = 32;
const ET_NPC: u8 = 13;
const FP_SEE: u32 = 1 << 14;
/// Already-decoded protocol-26 `fd.forcePowerLevel[FP_SEE]` player field.
const SENSE_LEVEL_FIELD: usize = 109;

/// Frame-local health visibility policy, independent of HUD style and bar layout.
#[derive(Clone, Copy)]
pub struct LegacyNpcHealthView {
    sense_three_active: bool,
}

impl LegacyNpcHealthView {
    /// Use the server-supplied Sense rank and the presented local active-power bits.
    /// Demo playback and callers without prediction pass `None` for snapshot bits.
    pub fn new(player: &PlayerState, predicted_active: Option<u32>) -> Self {
        Self {
            sense_three_active: predicted_active.unwrap_or_else(|| player.force_powers_active())
                & FP_SEE
                != 0
                && player.raw_field(SENSE_LEVEL_FIELD).unwrap_or(0) >= 3,
        }
    }

    /// Ordinary NPC health requires active Sense 3. Mapper `showhealth` uses
    /// `shouldtarget` and bypasses that gate; players and non-NPC map entities
    /// retain their existing rules. This does not create or estimate health data.
    pub fn permits(self, entity: &EntityState) -> bool {
        entity.number() < MAX_CLIENTS
            || entity.entity_type() != ET_NPC
            || entity.should_target()
            || self.sense_three_active
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjk_protocol::LEGACY_ENTITY_FIELDS;

    fn npc() -> EntityState {
        let mut entity = EntityState::zero(77, &LEGACY_ENTITY_FIELDS);
        let index = LEGACY_ENTITY_FIELDS
            .iter()
            .position(|field| field.name == "eType")
            .unwrap();
        entity.set_raw_field(index, u32::from(ET_NPC));
        entity
    }

    fn player(level: u32, active: u32) -> PlayerState {
        let mut player = PlayerState::zero();
        player.set_raw_field(SENSE_LEVEL_FIELD, level);
        player.set_raw_field(82, active);
        player
    }

    #[test]
    fn ordinary_npc_needs_both_sense_three_and_active_sense() {
        for level in 0..=3 {
            for active in [0, FP_SEE, 1 << 8] {
                let view = LegacyNpcHealthView::new(&player(level, active), None);
                assert_eq!(view.permits(&npc()), level == 3 && active == FP_SEE);
            }
        }
    }

    #[test]
    fn mapper_showhealth_bypasses_sense_gate() {
        let mut entity = npc();
        assert_eq!(LEGACY_ENTITY_FIELDS[57].name, "shouldtarget");
        entity.set_raw_field(57, 1);
        assert!(LegacyNpcHealthView::new(&player(0, 0), None).permits(&entity));
    }

    #[test]
    fn players_and_non_npc_map_entities_keep_their_rules() {
        let view = LegacyNpcHealthView::new(&player(0, 0), None);
        let general = EntityState::zero(78, &LEGACY_ENTITY_FIELDS);
        assert!(view.permits(&general));
        let client = EntityState::zero(3, &LEGACY_ENTITY_FIELDS);
        assert!(view.permits(&client));
    }

    #[test]
    fn prediction_controls_activation_but_never_invents_a_rank() {
        let entity = npc();
        assert!(LegacyNpcHealthView::new(&player(3, 0), Some(FP_SEE)).permits(&entity));
        assert!(!LegacyNpcHealthView::new(&player(3, FP_SEE), Some(0)).permits(&entity));
        assert!(!LegacyNpcHealthView::new(&player(2, 0), Some(FP_SEE)).permits(&entity));
        assert!(LegacyNpcHealthView::new(&player(3, FP_SEE), None).permits(&entity));
    }

    #[test]
    #[ignore = "release-only synthetic NPC health policy measurement"]
    fn measure_npc_health_visibility() {
        let view = LegacyNpcHealthView::new(&player(3, FP_SEE), None);
        let entities: Vec<_> = (0..128).map(|_| npc()).collect();
        let start = std::time::Instant::now();
        let mut visible = 0_u64;
        for _ in 0..20_000 {
            for entity in &entities {
                visible +=
                    u64::from(std::hint::black_box(view).permits(std::hint::black_box(entity)));
            }
        }
        eprintln!(
            "128 NPC visibility checks: {:.3} us/frame; visible={}",
            start.elapsed().as_secs_f64() * 1e6 / 20_000.0,
            std::hint::black_box(visible)
        );
    }
}
