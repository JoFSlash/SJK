//! The use key (`TryUse`, OpenJK `codemp/game/g_utils.c:1588-1760`): what a player
//! pressing `BUTTON_USE` reaches, and whether that thing answers.
//!
//! This is one of the shared pieces of the game rather than any one entity's: siege
//! objectives, `func_usable`, the buttons and every `PLAYER_USE` door and lift all wait
//! on it, which is why it lives on its own rather than inside one of them.

/// `USE_DISTANCE` (`g_utils.c:1588`): how far in front of its eyes a player can reach.
pub const USE_DISTANCE: f32 = 64.0;
/// `SVF_PLAYER_USABLE`, the flag that says the use key may reach a thing at all.
pub const SVF_PLAYER_USABLE: u32 = 0x0010;
/// `FL_INACTIVE`, which `target_deactivate` sets.
pub const FL_INACTIVE: u32 = 0x10000;
/// `BUTTON_USE`.
pub const BUTTON_USE: u16 = 32;
/// `MASK_OPAQUE | CONTENTS_SOLID | CONTENTS_BODY | CONTENTS_ITEM | CONTENTS_CORPSE`, the
/// mask `TryUse` traces with — so the reach is stopped by a wall, a player or a body.
pub const USE_MASK: u32 = 0x1 | 0x40 | 0x2000_0000 | 0x100 | 0x4000_0000;

/// `G_ValidUseEnt` (`g_utils.c:1428-1444`): whether the thing a player is looking at
/// answers the use key at all. Three separate refusals, in the reference's own order: it
/// has nothing to do when used, something deactivated it, or it was never usable.
pub fn answers_use(has_use: bool, flags: u32, svflags: u32) -> bool {
    if !has_use {
        return false;
    }
    if flags & FL_INACTIVE != 0 {
        return false;
    }
    svflags & SVF_PLAYER_USABLE != 0
}

/// Where `TryUse`'s trace runs from and to: a player's eyes, and `USE_DISTANCE` along
/// the way it is looking. The angles are degrees, as a player state keeps them.
pub fn reach(eyes: [f32; 3], view_angles: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let (yaw_sin, yaw_cos) = view_angles[1].to_radians().sin_cos();
    let (pitch_sin, pitch_cos) = view_angles[0].to_radians().sin_cos();
    // `AngleVectors`' forward: pitch is measured downwards, which is why it is negated.
    let forward = [pitch_cos * yaw_cos, pitch_cos * yaw_sin, -pitch_sin];
    (
        eyes,
        std::array::from_fn(|axis| eyes[axis] + forward[axis] * USE_DISTANCE),
    )
}

/// `func_usable` (`SP_func_usable`, `g_mover.c:3160-3215`): a brush a player uses to
/// fire its targets, and which a map may start hidden and bring back solid.
#[derive(Clone, Debug, PartialEq)]
pub struct Usable {
    /// The inline model it is made of, and that model's box around [`Self::origin`].
    pub model: usize,
    pub bounds: ([f32; 3], [f32; 3]),
    /// `s.origin`, copied to `s.pos.trBase` and `r.currentOrigin`: where the brush and
    /// its second model stand (an origin brush compiles the model around it).
    pub origin: [f32; 3],
    /// `model2`: an md3 every client draws with the brush (`modelindex2`, `CG_Mover`),
    /// such as a table over a clip-only brush; empty for none, and for a `.glm`, which
    /// the reference leaves out in multiplayer.
    pub model2: String,
    /// What it fires when used.
    pub target: String,
    /// What it fires when it comes back solid (`func_wait_return_solid`).
    pub target2: String,
    pub targetname: String,
    pub spawnflags: u32,
    /// `count`: whether it is there at all. A START_OFF brush is not.
    pub present: bool,
    pub contents: u32,
    pub svflags: u32,
    pub eflags: u32,
    /// `wait`, in milliseconds: how long an `ALWAYS_ON` one is out of use after a use.
    pub wait: i32,
    /// An `ALWAYS_ON` one used: when it answers again (`func_usable_think`), `None` while it
    /// answers, `i32::MAX` for never (`use = 0` with no `wait`).
    pub disabled_until: Option<i32>,
}

/// The wire fields [`Usable::project`] writes (`msg.cpp`'s entity fields).
mod es {
    pub const TYPE: usize = 8;
    pub const EFLAGS: usize = 19;
    pub const MODEL2: usize = 41;
    pub const POS_TYPE: usize = 23;
    pub const POS_BASE: [usize; 3] = [2, 1, 4];
}

impl Usable {
    /// Its wire state (`InitMover`, then `SP_func_usable`): an `ET_MOVER` of its inline
    /// model standing still (`TR_STATIONARY`) at its origin, with `model2` the registered
    /// index of [`Self::model2`] (zero for none).
    pub fn project(&self, state: &mut sjk_protocol::EntityState, model2: u16) {
        state.set_raw_field(es::TYPE, crate::path_movers::ET_MOVER);
        crate::triggers::set_brush_model(state, self.model);
        state.set_raw_field(es::MODEL2, u32::from(model2));
        state.set_raw_field(es::EFLAGS, self.eflags);
        state.set_raw_field(es::POS_TYPE, crate::path_movers::TR_STATIONARY);
        for (field, value) in es::POS_BASE.into_iter().zip(self.origin) {
            state.set_raw_field(field, value.to_bits());
        }
    }

    /// `MOVER_INACTIVE`: `GlobalUse` passes it by.
    pub fn inactive(&self) -> bool {
        self.spawnflags & USABLE_INACTIVE != 0
    }

    /// Whether the use key reaches it and fires it (`ValidUseTarget`,
    /// `g_utils.c:1628-1646`): a mover the map made `PLAYER_USE`, not deactivated.
    pub fn answers_use(&self) -> bool {
        let flags = if self.spawnflags & USABLE_INACTIVE != 0 {
            FL_INACTIVE
        } else {
            0
        };
        answers_use(true, flags, self.svflags)
    }
}

/// `SP_func_usable`'s own spawnflags.
const USABLE_START_OFF: u32 = 1;
/// `ALWAYS_ON` (8): using it fires its targets and it never goes away.
pub const USABLE_ALWAYS_ON: u32 = 8;
/// `MOVER_PLAYER_USE` (64, `g_mover.c:52`): `InitMover` makes it `SVF_PLAYER_USABLE`.
const USABLE_PLAYER_USE: u32 = 64;
/// `MOVER_INACTIVE` (128, `g_mover.c:53`): `InitMover` makes it `FL_INACTIVE`.
const USABLE_INACTIVE: u32 = 128;
/// `SVF_NOCLIENT` and `EF_NODRAW` (`bg_public.h`: bit 8; bit 7 is `EF_PERMANENT`),
/// which a hidden one wears.
const SVF_NOCLIENT: u32 = 0x0001;
const EF_NODRAW: u32 = 1 << 8;
/// `CONTENTS_SOLID`.
const CONTENTS_SOLID: u32 = 1;

/// `SP_func_usable`. A brush that starts off is not in the world at all — no contents,
/// not drawn, not sent — until something brings it back.
pub fn spawn_usable(entity: &sjk_entity::Entity, bounds: ([f32; 3], [f32; 3])) -> Option<Usable> {
    if entity.get("classname") != Some("func_usable") {
        return None;
    }
    let model = entity
        .get("model")
        .and_then(|name| name.strip_prefix('*'))
        .and_then(|index| index.parse().ok())?;
    let number = |key: &str| {
        entity
            .get(key)
            .and_then(|text| text.trim().parse::<f32>().ok())
            .unwrap_or(0.0)
    };
    let spawnflags = number("spawnflags") as u32;
    let off = spawnflags & USABLE_START_OFF != 0;
    let origin = std::array::from_fn(|axis| {
        entity
            .get("origin")
            .and_then(|text| text.split_whitespace().nth(axis))
            .and_then(|value| value.parse::<f32>().ok())
            .unwrap_or(0.0)
    });
    let model2 = entity.get("model2").unwrap_or_default();
    let player_use = if spawnflags & USABLE_PLAYER_USE != 0 {
        SVF_PLAYER_USABLE
    } else {
        0
    };
    Some(Usable {
        model,
        bounds,
        origin,
        model2: if model2.contains(".glm") {
            String::new()
        } else {
            model2.to_owned()
        },
        target: entity.get("target").unwrap_or_default().to_owned(),
        target2: entity.get("target2").unwrap_or_default().to_owned(),
        targetname: entity.get("targetname").unwrap_or_default().to_owned(),
        spawnflags,
        present: !off,
        contents: if off { 0 } else { CONTENTS_SOLID },
        svflags: player_use | if off { SVF_NOCLIENT } else { 0 },
        eflags: if off { EF_NODRAW } else { 0 },
        wait: (number("wait") * 1_000.0) as i32,
        disabled_until: None,
    })
}

impl Usable {
    /// `func_usable_think` at `level_time`: an `ALWAYS_ON` one whose `wait` is over is
    /// usable again, by the use key too.
    pub fn wake(&mut self, level_time: i32) {
        if self.disabled_until.is_some_and(|until| until <= level_time) {
            self.disabled_until = None;
            self.svflags |= SVF_PLAYER_USABLE;
        }
    }

    /// `func_usable_use` (`g_mover.c:3070-3126`) at `level_time`: an `ALWAYS_ON` one fires
    /// its `target` and is out of use for its `wait` (for good without one); any other
    /// toggles, going away (not drawn, not solid) and firing its `target`, or coming back
    /// solid and firing its `target2`. What it fires, empty for nothing.
    ///
    /// Not ported: a shader-animating one (`endframe`) steps its frame instead, a use
    /// script runs, and one with spawnflag 16 waits until nobody stands in it before it
    /// comes back.
    pub fn used(&mut self, level_time: i32) -> String {
        self.wake(level_time);
        if self.disabled_until.is_some() {
            // `use = 0`: nothing answers.
            return String::new();
        }
        if self.spawnflags & USABLE_ALWAYS_ON != 0 {
            self.svflags &= !SVF_PLAYER_USABLE;
            self.disabled_until = Some(if self.wait != 0 {
                level_time + self.wait
            } else {
                i32::MAX
            });
            return self.target.clone();
        }
        if self.present {
            self.present = false;
            self.contents = 0;
            self.svflags |= SVF_NOCLIENT;
            self.eflags |= EF_NODRAW;
            self.target.clone()
        } else {
            self.present = true;
            self.contents = CONTENTS_SOLID;
            self.svflags &= !SVF_NOCLIENT;
            self.eflags &= !EF_NODRAW;
            self.target2.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usable(pairs: &[(&str, &str)]) -> Usable {
        let mut text = String::from(
            "{
\"classname\" \"func_usable\"
",
        );
        for (key, value) in pairs {
            text.push_str(&format!(
                "\"{key}\" \"{value}\"
"
            ));
        }
        text.push_str(
            "}
",
        );
        let entity = sjk_entity::parse_entity_lump(text.as_bytes())
            .unwrap()
            .remove(0);
        spawn_usable(&entity, ([-22.0, -20.0, -1.0], [20.0, 20.0, 43.0])).unwrap()
    }

    #[test]
    fn a_table_stands_at_its_origin_with_its_model2() {
        // `t2_rancor`'s entity 231: a clip-only brush with a table md3.
        let table = usable(&[
            ("model", "*27"),
            ("model2", "models/map_objects/nar_shaddar/table01.md3"),
            ("origin", "-2839 2228 1072"),
        ]);
        assert_eq!(table.origin, [-2839.0, 2228.0, 1072.0]);
        assert_eq!(table.model2, "models/map_objects/nar_shaddar/table01.md3");
        assert!(table.present);
        assert_eq!(table.eflags, 0);
        let mut state = sjk_protocol::EntityState::zero(0, &sjk_protocol::LEGACY_ENTITY_FIELDS);
        table.project(&mut state, 5);
        assert_eq!(state.entity_type(), 6);
        assert_eq!(state.model_index(), 27);
        assert_eq!(state.model_index2(), 5);
        assert_eq!(state.trajectory_base(), [-2839.0, 2228.0, 1072.0]);
    }

    #[test]
    fn a_start_off_usable_is_hidden_by_nodraw_not_permanent() {
        let hidden = usable(&[("model", "*17"), ("spawnflags", "1")]);
        assert!(!hidden.present);
        assert_eq!(hidden.eflags, 1 << 8);
        assert_eq!(hidden.contents, 0);
        assert_eq!(hidden.svflags & SVF_NOCLIENT, SVF_NOCLIENT);
    }

    #[test]
    fn only_player_use_usables_answer_the_use_key() {
        assert!(!usable(&[("model", "*1")]).answers_use());
        assert!(usable(&[("model", "*1"), ("spawnflags", "64")]).answers_use());
        assert!(!usable(&[("model", "*1"), ("spawnflags", "192")]).answers_use());
    }

    #[test]
    fn a_used_wall_goes_away_and_comes_back() {
        let mut wall = usable(&[
            ("model", "*9"),
            ("targetname", "shield"),
            ("target", "down"),
            ("target2", "up"),
        ]);
        assert_eq!(wall.used(1_000), "down");
        assert!(!wall.present);
        assert_eq!(wall.contents, 0);
        assert_eq!(wall.eflags & EF_NODRAW, EF_NODRAW);
        assert_eq!(wall.svflags & SVF_NOCLIENT, SVF_NOCLIENT);
        assert_eq!(wall.used(2_000), "up");
        assert!(wall.present);
        assert_eq!(wall.contents, CONTENTS_SOLID);
        assert_eq!(wall.eflags & EF_NODRAW, 0);
    }

    #[test]
    fn an_always_on_usable_fires_and_rests_for_its_wait() {
        let mut console = usable(&[
            ("model", "*3"),
            ("spawnflags", "72"),
            ("target", "relay"),
            ("wait", "2"),
        ]);
        assert_eq!(console.used(1_000), "relay");
        assert!(console.present);
        assert!(!console.answers_use());
        assert_eq!(console.used(2_000), "");
        console.wake(3_000);
        assert!(console.answers_use());
        assert_eq!(console.used(3_000), "relay");
        let mut once = usable(&[("model", "*3"), ("spawnflags", "8"), ("target", "t")]);
        assert_eq!(once.used(0), "t");
        assert_eq!(once.used(1_000_000), "");
    }

    #[test]
    fn a_glm_model2_is_left_out_as_in_multiplayer() {
        let usable = usable(&[("model", "*1"), ("model2", "models/x/model.glm")]);
        assert!(usable.model2.is_empty());
    }
}
