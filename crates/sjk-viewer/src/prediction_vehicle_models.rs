//! Snapshot-time vehicle-definition cache for prediction collision hulls.

use super::{CS_MODELS, Unregistered};
use sjk_game_jka::{vehicle_bounds::VehicleBounds, vehicle_parms::VehicleTable};
use sjk_protocol::{GameState, Snapshot};

/// Protocol-26 `MAX_MODELS`; contained in this compatibility cache.
const MAX_MODELS: usize = 512;
const CLASS_VEHICLE: u8 = 53;

pub(super) struct Models {
    names: Box<[Option<Box<[u8]>>]>,
    bounds: Box<[Option<VehicleBounds>]>,
    revision: u64,
}

impl Models {
    pub(super) fn new() -> Self {
        Self {
            names: std::iter::repeat_with(|| None).take(MAX_MODELS).collect(),
            bounds: vec![None; MAX_MODELS].into_boxed_slice(),
            revision: 0,
        }
    }

    pub(super) fn refresh(
        &mut self,
        snapshot: &Snapshot,
        game: &GameState,
        table: &mut VehicleTable,
    ) {
        for state in &snapshot.entities {
            if state.entity_type() != 13 || state.npc_class() != CLASS_VEHICLE {
                continue;
            }
            let model = state.model_index() as usize;
            let Some(cached_name) = self.names.get_mut(model) else {
                continue;
            };
            let name = game.config_string(CS_MODELS + model).unwrap_or_default();
            if cached_name.as_deref() == Some(name) {
                continue;
            }
            self.bounds[model] = name
                .strip_prefix(b"$")
                .and_then(|name| table.index_for_name(name, &mut Unregistered))
                .and_then(|index| table.vehicle(index))
                .and_then(VehicleBounds::from_info);
            *cached_name = Some(name.into());
            self.revision = self.revision.wrapping_add(1);
        }
    }

    pub(super) fn bounds(&self) -> (u64, &[Option<VehicleBounds>]) {
        (self.revision, &self.bounds)
    }
}
