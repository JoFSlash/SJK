//! The world in a ragdoll's way ([`World`]): the map's brushes, and the brush movers
//! the game shows (lifts, doors, trains), each standing where its trajectory puts it at
//! the step's own moment.
//!
//! A step's movers are read from the first snapshot at or after its moment, the one
//! holding the trajectories in effect then, so every client that saw that snapshot
//! places a lift at the same height at the same step, however late it presents the
//! world. A lift carries the points lying on it, and a mover that moves into a point
//! pushes it along ([`World::carry`], [`World::pusher`]).

use super::body::{Hit, STEP_MS, World};
use crate::local_prediction::movers::Collider;
use crate::movement_collision::solid_trace;
use glam::Vec3;
use sjk_bsp::{Aabb, Bsp, TraceScratch};
use sjk_client::ClientSession;
use sjk_client::animation_events::footsteps;

/// `CONTENTS_SOLID`.
const CONTENTS_SOLID: u32 = 1;
/// A mover further than this beyond the body's box is out of its reach for a step.
const REACH: f32 = 96.0;

/// The brush movers of the snapshot last read, and those near the body at the step.
#[derive(Default)]
pub(crate) struct Movers {
    /// The server time of the snapshot `solids` were read from.
    snapshot: Option<i32>,
    solids: Vec<Collider>,
    /// Where each mover near the body stands at the step's end, and a step before.
    near: Vec<(Collider, Collider)>,
}

/// The world for one frame's steps.
pub(crate) struct Scene<'a> {
    bsp: &'a Bsp,
    scratch: &'a mut TraceScratch,
    session: Option<&'a ClientSession>,
    movers: &'a mut Movers,
}

impl<'a> Scene<'a> {
    /// The map's brushes, and the movers of `session`'s snapshots (none without one).
    pub(crate) fn new(
        bsp: &'a Bsp,
        scratch: &'a mut TraceScratch,
        session: Option<&'a ClientSession>,
        movers: &'a mut Movers,
    ) -> Self {
        // The map may have changed since the last frame: read the solids again.
        movers.snapshot = None;
        movers.solids.clear();
        movers.near.clear();
        Self {
            bsp,
            scratch,
            session,
            movers,
        }
    }

    /// Whether `solid` comes within [`REACH`] of the box from `low` to `high`.
    fn reaches(&self, solid: &Collider, low: Vec3, high: Vec3) -> bool {
        let Some(model) = self.bsp.render().models().get(solid.model) else {
            return false;
        };
        let origin = Vec3::from_array(solid.origin);
        let (mins, maxs) = (
            Vec3::from_array(model.minimums),
            Vec3::from_array(model.maximums),
        );
        let (from, to) = if solid.rotated {
            let radius = mins.abs().max(maxs.abs()).length();
            (origin - Vec3::splat(radius), origin + Vec3::splat(radius))
        } else {
            (origin + mins, origin + maxs)
        };
        from.cmple(high + REACH).all() && to.cmpge(low - REACH).all()
    }
}

/// The footstep family of what a trace hit: its material, else guessed from its
/// shader's name, else stone (as the footsteps, `footprints.rs`).
fn family(bsp: &Bsp, flags: u32, shader: Option<usize>) -> u8 {
    let family = if flags & 31 != 0 {
        footsteps::material(flags)
    } else {
        shader
            .and_then(|index| bsp.shaders().get(index))
            .and_then(|shader| footsteps::material_from_name(&shader.name_lossy()))
            .unwrap_or(0)
    };
    family as u8
}

/// `point` in the space of a solid with `axes` (rows: its forward, left and up).
fn into(axes: [[f32; 3]; 3], point: Vec3) -> Vec3 {
    Vec3::from_array(axes.map(|axis| Vec3::from_array(axis).dot(point)))
}

/// `point` of a solid's space with `axes`, back in the world's.
fn out_of(axes: [[f32; 3]; 3], point: Vec3) -> Vec3 {
    Vec3::from_array(axes[0]) * point.x
        + Vec3::from_array(axes[1]) * point.y
        + Vec3::from_array(axes[2]) * point.z
}

impl World for Scene<'_> {
    fn trace(&mut self, from: Vec3, to: Vec3, half: f32) -> Option<Hit> {
        let bounds = Aabb::new([-half; 3], [half; 3]).unwrap_or(Aabb::POINT);
        let (start, end) = (from.to_array(), to.to_array());
        // A point caught in a brush (the sticks pulled it in) may leave it for open
        // space, else stays: never pushed out some way, which would walk it up a wall.
        let stuck = |mover| Hit {
            stop: from,
            normal: Vec3::ZERO,
            surface: 0,
            mover,
        };
        let mut best: Option<(f32, Hit)> = None;
        let trace = self
            .bsp
            .trace_box_with(self.scratch, start, end, bounds, CONTENTS_SOLID);
        if trace.start_solid {
            let free = !self
                .bsp
                .trace_box_with(self.scratch, end, end, bounds, CONTENTS_SOLID)
                .start_solid;
            if !free {
                return Some(stuck(None));
            }
        } else if let Some(plane) = trace.plane.filter(|_| trace.fraction < 1.0) {
            best = Some((
                trace.fraction,
                Hit {
                    stop: Vec3::from_array(trace.end_position),
                    normal: Vec3::from_array(plane.normal),
                    surface: family(self.bsp, trace.surface_flags, trace.shader),
                    mover: None,
                },
            ));
        }
        for (solid, _) in &self.movers.near {
            let trace = solid_trace(self.bsp, solid, start, end, bounds, CONTENTS_SOLID);
            if trace.start_solid {
                let free =
                    !solid_trace(self.bsp, solid, end, end, bounds, CONTENTS_SOLID).start_solid;
                if !free {
                    return Some(stuck(Some(solid.entity)));
                }
                continue;
            }
            if trace.fraction < 1.0 && best.is_none_or(|(fraction, _)| trace.fraction < fraction) {
                best = Some((
                    trace.fraction,
                    Hit {
                        stop: Vec3::from_array(trace.end_position),
                        normal: Vec3::from_array(trace.plane_normal),
                        surface: family(self.bsp, trace.surface_flags, None),
                        mover: Some(solid.entity),
                    },
                ));
            }
        }
        best.map(|(_, hit)| hit)
    }

    fn at(&mut self, time: i32, low: Vec3, high: Vec3) {
        let snapshot = self.session.map(|session| {
            session
                .snapshot_after(time.wrapping_sub(1))
                .unwrap_or_else(|| session.snapshot_at_or_before(time))
        });
        let read = snapshot.map(|snapshot| snapshot.server_time);
        if read != self.movers.snapshot {
            self.movers.snapshot = read;
            self.movers.solids.clear();
            if let Some(snapshot) = snapshot {
                self.movers.solids.extend(
                    snapshot
                        .entities
                        .iter()
                        .filter_map(|entity| Collider::from_entity(entity, time, time, false))
                        .filter(|solid| solid.bounds.is_none()),
                );
            }
        }
        let mut near = std::mem::take(&mut self.movers.near);
        near.clear();
        for solid in &self.movers.solids {
            let (now, before) = if solid.moves() {
                (
                    solid.placed_at(time),
                    solid.placed_at(time.wrapping_sub(STEP_MS)),
                )
            } else {
                (*solid, *solid)
            };
            if self.reaches(&now, low, high) || self.reaches(&before, low, high) {
                near.push((now, before));
            }
        }
        self.movers.near = near;
    }

    fn carry(&mut self, mover: u16, at: Vec3) -> Vec3 {
        let Some((now, before)) = self.movers.near.iter().find(|(now, _)| now.entity == mover)
        else {
            return at;
        };
        let mut local = at - Vec3::from_array(before.origin);
        if before.rotated {
            local = into(before.axes, local);
        }
        if now.rotated {
            local = out_of(now.axes, local);
        }
        local + Vec3::from_array(now.origin)
    }

    fn pusher(&mut self, at: Vec3, half: f32) -> Option<u16> {
        let bounds = Aabb::new([-half; 3], [half; 3]).unwrap_or(Aabb::POINT);
        let point = at.to_array();
        let inside = |solid: &Collider| {
            solid_trace(self.bsp, solid, point, point, bounds, CONTENTS_SOLID).start_solid
        };
        self.movers
            .near
            .iter()
            .filter(|(now, before)| {
                now.origin != before.origin
                    || now.rotated != before.rotated
                    || (now.rotated && now.axes != before.axes)
            })
            .find(|(now, before)| inside(now) && !inside(before))
            .map(|(now, _)| now.entity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_prediction::movers::rotation_axes;

    #[test]
    fn a_point_on_a_turning_mover_turns_with_it() {
        // A quarter turn to the left carries a point ahead of the hinge to its left.
        let (before, now) = (rotation_axes([0.0; 3]), rotation_axes([0.0, 90.0, 0.0]));
        let carried = out_of(now, into(before, Vec3::new(10.0, 0.0, 5.0)));
        assert!(
            carried.distance(Vec3::new(0.0, 10.0, 5.0)) < 1e-4,
            "{carried}"
        );
        let back = out_of(before, into(now, carried));
        assert!(back.distance(Vec3::new(10.0, 0.0, 5.0)) < 1e-4, "{back}");
    }
}
