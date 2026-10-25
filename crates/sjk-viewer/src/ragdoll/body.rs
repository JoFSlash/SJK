//! The Ragdoll toy's body: a limp humanoid of [`JOINTS`] points held together by sticks
//! (Verlet integration, as the sandbox games' first ragdolls), falling and tumbling
//! against the world.
//!
//! Every SJK client that hears of a fall runs this same simulation from the same start
//! ([`Start`]), in fixed steps of [`STEP_MS`] game server milliseconds counted from the
//! fall's moment, so all of them show the body in the same place without any pose being
//! sent: the start is one short relay event. The body's proportions are fixed, not the
//! player model's (a client that lacks the model draws another). The world's brushes
//! and its movers are in its way: a mover stands where its trajectory puts it at the
//! step's own moment ([`World::at`]), which every client that sees the mover reads the
//! same, so a lift carries the body and a closing door pushes it the same way everywhere.
//! Once every point has lain still for [`SLEEP_STEPS`] steps the body sleeps, until a
//! mover carries or pushes it again.

use glam::Vec3;

/// The points, by name.
pub(crate) const PELVIS: usize = 0;
pub(crate) const CHEST: usize = 1;
pub(crate) const HEAD: usize = 2;
pub(crate) const L_SHOULDER: usize = 3;
pub(crate) const L_ELBOW: usize = 4;
pub(crate) const L_HAND: usize = 5;
pub(crate) const R_SHOULDER: usize = 6;
pub(crate) const R_ELBOW: usize = 7;
pub(crate) const R_HAND: usize = 8;
pub(crate) const L_HIP: usize = 9;
pub(crate) const L_KNEE: usize = 10;
pub(crate) const L_FOOT: usize = 11;
pub(crate) const R_HIP: usize = 12;
pub(crate) const R_KNEE: usize = 13;
pub(crate) const R_FOOT: usize = 14;
pub(crate) const JOINTS: usize = 15;

/// Where each point stands on a body upright at the origin (its feet at height 0),
/// facing +X with its left at +Y: the stock `_humanoid` skeleton's joints (pelvis,
/// thoracic, cranium, humerus, radius, hand, femur, tibia, talus), arms hanging.
const STANDING: [[f32; 3]; JOINTS] = [
    [0.0, 0.0, 37.0],
    [0.0, 0.0, 51.5],
    [1.0, 0.0, 61.5],
    [-1.0, 6.0, 54.5],
    [-1.5, 7.0, 43.8],
    [0.5, 7.5, 35.2],
    [-1.0, -6.0, 54.5],
    [-1.5, -7.0, 43.8],
    [0.5, -7.5, 35.2],
    [0.0, 3.6, 35.0],
    [1.5, 4.0, 21.3],
    [0.0, 4.5, 7.5],
    [0.0, -3.6, 35.0],
    [1.5, -4.0, 21.3],
    [0.0, -4.5, 7.5],
];

/// The sticks: pairs kept at their standing length. Bones first, then the braces that
/// keep the trunk a solid block (shoulders and hips crossed) and the head on the
/// shoulders.
const STICKS: [(usize, usize); 26] = [
    (PELVIS, CHEST),
    (CHEST, HEAD),
    (CHEST, L_SHOULDER),
    (CHEST, R_SHOULDER),
    (L_SHOULDER, L_ELBOW),
    (L_ELBOW, L_HAND),
    (R_SHOULDER, R_ELBOW),
    (R_ELBOW, R_HAND),
    (PELVIS, L_HIP),
    (PELVIS, R_HIP),
    (L_HIP, L_KNEE),
    (L_KNEE, L_FOOT),
    (R_HIP, R_KNEE),
    (R_KNEE, R_FOOT),
    (L_SHOULDER, R_SHOULDER),
    (L_HIP, R_HIP),
    (L_SHOULDER, L_HIP),
    (R_SHOULDER, R_HIP),
    (L_SHOULDER, R_HIP),
    (R_SHOULDER, L_HIP),
    (L_SHOULDER, PELVIS),
    (R_SHOULDER, PELVIS),
    (HEAD, L_SHOULDER),
    (HEAD, R_SHOULDER),
    (CHEST, L_HIP),
    (CHEST, R_HIP),
];

/// Pairs that may come closer than they stand but not further, and not closer than this
/// share of it: a limb folds at its middle joint, never inside out to nothing; a hand
/// never sinks into its own shoulder.
const REACH: [(usize, usize, f32); 6] = [
    (L_HIP, L_FOOT, 0.45),
    (R_HIP, R_FOOT, 0.45),
    (L_SHOULDER, L_HAND, 0.25),
    (R_SHOULDER, R_HAND, 0.25),
    (HEAD, PELVIS, 0.92),
    (L_FOOT, R_FOOT, 0.25),
];

/// The hinges: (upper, middle, lower) of a limb, and whether the middle bends forward
/// (a knee) or back (an elbow) of the line from upper to lower.
const HINGES: [(usize, usize, usize, bool); 4] = [
    (L_HIP, L_KNEE, L_FOOT, true),
    (R_HIP, R_KNEE, R_FOOT, true),
    (L_SHOULDER, L_ELBOW, L_HAND, false),
    (R_SHOULDER, R_ELBOW, R_HAND, false),
];

/// One step of the simulation, in game server milliseconds, and in seconds.
pub(crate) const STEP_MS: i32 = 16;
const STEP: f32 = STEP_MS as f32 / 1000.0;
/// `g_gravity`.
const GRAVITY: f32 = 800.0;
/// How much of its speed a point keeps from one step to the next, in the air.
const DRAG: f32 = 0.995;
/// Rounds of the sticks per step: more is stiffer.
const ROUNDS: usize = 6;
/// Half the size of each point's box in the world's collision: about how thick the body
/// is there, so that it lies on a floor rather than in it.
const HALVES: [f32; JOINTS] = [
    4.5, 5.0, 5.0, 3.0, 2.5, 2.0, 3.0, 2.5, 2.0, 3.5, 3.0, 2.5, 3.5, 3.0, 2.5,
];
/// Off a surface a point keeps this much of its speed along it, and bounces back with
/// this much of its speed into it.
const FRICTION_KEPT: f32 = 0.55;
const BOUNCE: f32 = 0.2;
/// Slower into a surface than this (units a second), a point does not bounce, and its
/// knock is not heard.
const BOUNCE_SPEED: f32 = 60.0;
/// A point that moves less than this in a step is still (the sticks and the floor keep
/// a lying body trembling by a tenth of a unit); the body sleeps once all are, for this
/// many steps in a row (a second).
const STILL: f32 = 0.2;
pub(crate) const SLEEP_STEPS: u32 = 60;
/// The body sleeps after this many steps whatever it does (40 seconds).
pub(crate) const MAX_STEPS: u32 = 2_500;
/// A point that falls this far below where the body started stops: the void under a map.
const FLOOR_DEPTH: f32 = 8_192.0;
/// The fastest a point may go, in units a step (what the trace can follow).
const TOP_SPEED: f32 = 40.0;
/// A point sliding along a surface faster than this (units a second) scrapes.
const SCRAPE_SPEED: f32 = 80.0;

/// How a fall starts: where the player stood (their origin, 24 units above their feet,
/// as the game keeps it), how fast they went, which way they faced (degrees), and the
/// moment (game server milliseconds), which also varies how the body crumples.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Start {
    pub(crate) moment: i32,
    pub(crate) origin: [i32; 3],
    pub(crate) velocity: [i32; 3],
    pub(crate) yaw: u16,
}

/// What a point met: where it stopped, the surface's normal, what the surface is made
/// of (a footstep family, for its sound), and the mover it is, if one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Hit {
    pub(crate) stop: Vec3,
    pub(crate) normal: Vec3,
    pub(crate) surface: u8,
    pub(crate) mover: Option<u16>,
}

/// What the world is like round a point.
pub(crate) trait World {
    /// What a box of half size `half` meets moving from one point to another.
    fn trace(&mut self, from: Vec3, to: Vec3, half: f32) -> Option<Hit>;
    /// Stand the movers near the body (its points between `low` and `high`) where they
    /// are at `time` (game server milliseconds), the end of a step, and a step before.
    fn at(&mut self, _time: i32, _low: Vec3, _high: Vec3) {}
    /// Where mover `mover`'s move over the step takes a point at `at` that rides it.
    fn carry(&mut self, _mover: u16, at: Vec3) -> Vec3 {
        at
    }
    /// The mover that moved into a box of half size `half` at `at` over the step.
    fn pusher(&mut self, _at: Vec3, _half: f32) -> Option<u16> {
        None
    }
}

/// A world of brushes alone, giving the stop and the normal.
impl<F: FnMut(Vec3, Vec3, f32) -> Option<(Vec3, Vec3)>> World for F {
    fn trace(&mut self, from: Vec3, to: Vec3, half: f32) -> Option<Hit> {
        self(from, to, half).map(|(stop, normal)| Hit {
            stop,
            normal,
            surface: 0,
            mover: None,
        })
    }
}

/// A point that hit a surface (how hard, into it) or slid along one (how fast), in units
/// a second, and what the surface is made of ([`Hit::surface`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Knock {
    pub(crate) joint: usize,
    pub(crate) speed: f32,
    pub(crate) surface: u8,
}

/// The body, as it falls.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Body {
    /// Each point now, and a step ago (the speed is the difference).
    points: [Vec3; JOINTS],
    previous: [Vec3; JOINTS],
    /// Each point before the last step, where it was shown (unlike `previous`, which
    /// a hit changes to change the speed).
    earlier: [Vec3; JOINTS],
    lengths: [f32; STICKS.len()],
    reach: [f32; REACH.len()],
    /// The fall's moment (game server milliseconds).
    moment: i32,
    /// Steps taken since the moment, how many of the last in a row all were still, and
    /// how many since the start or since a mover last woke the body.
    steps: u32,
    still: u32,
    restless: u32,
    /// The mover each point lay on at the last step, and how far that step carried it.
    riding: [Option<u16>; JOINTS],
    ride: [Vec3; JOINTS],
    /// Since [`Body::run_to`] began: the points that hit the world, and that slid along it.
    pub(crate) knocks: Vec<Knock>,
    pub(crate) scrapes: Vec<Knock>,
    lowest: f32,
}

/// A number in 0..1 from `seed` and `salt`, the same on every client.
fn unit(seed: u32, salt: u32) -> f32 {
    let mut mixed = seed ^ salt.wrapping_mul(0x9e37_79b9);
    mixed ^= mixed >> 16;
    mixed = mixed.wrapping_mul(0x85eb_ca6b);
    mixed ^= mixed >> 13;
    mixed = mixed.wrapping_mul(0xc2b2_ae35);
    mixed ^= mixed >> 16;
    (mixed >> 8) as f32 / (1 << 24) as f32
}

/// The cosine and sine of `yaw` (a whole number of degrees), from a table of whole
/// degrees so that every client gets the same, whatever its maths library.
fn turn(yaw: u16) -> (f32, f32) {
    let degrees = i32::from(yaw % 360);
    // Quarter turns are exact; within one, Bhaskara's approximation is enough to
    // place a body, and it is plain arithmetic.
    let quarter = degrees / 90;
    let within = (degrees % 90) as f32;
    let sine = |d: f32| {
        // sin(d degrees) for 0 <= d <= 180.
        4.0 * d * (180.0 - d) / (40_500.0 - d * (180.0 - d))
    };
    let (s, c) = (sine(within), sine(90.0 - within));
    match quarter {
        0 => (c, s),
        1 => (-s, c),
        2 => (-c, -s),
        _ => (s, -c),
    }
}

impl Body {
    /// The body at the start of a fall: standing where the player stood, turned their
    /// way, with their speed, given a push off balance (back and to one side, the knees
    /// buckling) that the moment varies.
    pub(crate) fn new(start: &Start) -> Self {
        let (cos, sin) = turn(start.yaw);
        let feet = Vec3::new(
            start.origin[0] as f32,
            start.origin[1] as f32,
            start.origin[2] as f32 - 24.0,
        );
        let speed = Vec3::new(
            start.velocity[0] as f32,
            start.velocity[1] as f32,
            start.velocity[2] as f32,
        ) * STEP;
        let seed = start.moment as u32;
        let forward = Vec3::new(cos, sin, 0.0);
        let left = Vec3::new(-sin, cos, 0.0);
        // The topple, in units a second: the trunk goes back, the knees forward.
        let back = -forward * (70.0 + 50.0 * unit(seed, 1)) + left * (60.0 * unit(seed, 2) - 30.0);
        let buckle = forward * (40.0 + 30.0 * unit(seed, 3));
        let mut points = [Vec3::ZERO; JOINTS];
        let mut previous = [Vec3::ZERO; JOINTS];
        for (joint, standing) in STANDING.iter().enumerate() {
            let [x, y, z] = *standing;
            let at = feet + forward * x + left * y + Vec3::Z * z;
            let push = match joint {
                CHEST | HEAD | L_SHOULDER | R_SHOULDER => back,
                L_ELBOW | R_ELBOW | L_HAND | R_HAND => back * 0.6,
                L_KNEE | R_KNEE => buckle,
                _ => Vec3::ZERO,
            } * STEP;
            points[joint] = at;
            previous[joint] = at - speed - push;
        }
        let standing = |joint: usize| Vec3::from_array(STANDING[joint]);
        let lengths = STICKS.map(|(a, b)| standing(a).distance(standing(b)));
        let reach = REACH.map(|(a, b, share)| standing(a).distance(standing(b)) * share);
        Self {
            points,
            previous,
            earlier: points,
            lengths,
            reach,
            moment: start.moment,
            steps: 0,
            still: 0,
            restless: 0,
            riding: [None; JOINTS],
            ride: [Vec3::ZERO; JOINTS],
            knocks: Vec::with_capacity(JOINTS),
            scrapes: Vec::with_capacity(JOINTS),
            lowest: feet.z - FLOOR_DEPTH,
        }
    }

    /// Where each point is now.
    pub(crate) fn points(&self) -> &[Vec3; JOINTS] {
        &self.points
    }

    /// Where each point is `fraction` of the way from the last step to the next, for
    /// a frame between two steps (from the last step's start to its end: the body is
    /// shown a step late, which no one sees).
    pub(crate) fn between(&self, fraction: f32) -> [Vec3; JOINTS] {
        std::array::from_fn(|joint| self.earlier[joint].lerp(self.points[joint], fraction))
    }

    /// Steps taken since the moment.
    pub(crate) fn steps(&self) -> u32 {
        self.steps
    }

    /// Whether it lies still (or has run for [`MAX_STEPS`] since it last woke).
    pub(crate) fn asleep(&self) -> bool {
        self.still >= SLEEP_STEPS || self.restless >= MAX_STEPS
    }

    /// Where the body is, for the camera and the player's own body: its pelvis.
    pub(crate) fn centre(&self) -> Vec3 {
        self.points[PELVIS]
    }

    /// Step on to `steps` steps since the moment, against `world`. Asleep, the steps go
    /// by with the body lying still, unless a mover wakes it.
    pub(crate) fn run_to(&mut self, steps: u32, world: &mut impl World) {
        self.knocks.clear();
        self.scrapes.clear();
        while self.steps < steps {
            if self.asleep() && !self.stirred(world) {
                self.steps += 1;
                continue;
            }
            self.step(world);
        }
        if self.asleep() {
            self.earlier = self.points;
        }
    }

    /// The game server time at the end of step `steps`.
    fn time_of(&self, steps: u32) -> i32 {
        self.moment
            .wrapping_add((steps as i32).wrapping_mul(STEP_MS))
    }

    /// The box round every point.
    fn bounds(&self) -> (Vec3, Vec3) {
        self.points
            .iter()
            .fold((self.points[0], self.points[0]), |(low, high), point| {
                (low.min(*point), high.max(*point))
            })
    }

    /// Whether a mover wakes the sleeping body at its next step: one carrying a point
    /// that lies on it, or moving into one.
    fn stirred(&mut self, world: &mut impl World) -> bool {
        let (low, high) = self.bounds();
        world.at(self.time_of(self.steps + 1), low, high);
        let stirred = (0..JOINTS).any(|joint| {
            let at = self.points[joint];
            self.riding[joint].is_some_and(|mover| world.carry(mover, at) != at)
                || world.pusher(at, half(joint)).is_some()
        });
        if stirred {
            self.still = 0;
            self.restless = 0;
        }
        stirred
    }

    /// One step: the movers carry or push the points they hold, then every point moves
    /// on by its speed and gravity, out of the world's way, and the sticks hold.
    fn step(&mut self, world: &mut impl World) {
        self.steps += 1;
        self.restless += 1;
        let began = self.points;
        self.earlier = began;
        let (low, high) = self.bounds();
        world.at(self.time_of(self.steps), low, high);
        // A point on a mover goes with it, and one a mover moves into is pushed along:
        // both keep their own speed on top (the previous point moves as well).
        let mut carried = false;
        for joint in 0..JOINTS {
            let at = self.points[joint];
            let mover = self.riding[joint].or_else(|| world.pusher(at, half(joint)));
            let shift = mover.map_or(Vec3::ZERO, |mover| world.carry(mover, at) - at);
            self.points[joint] += shift;
            self.previous[joint] += shift;
            self.ride[joint] = shift;
            carried |= shift != Vec3::ZERO;
        }
        let fall = Vec3::new(0.0, 0.0, -GRAVITY * STEP * STEP);
        let mut touched = [None; JOINTS];
        for (joint, touch) in touched.iter_mut().enumerate() {
            let at = self.points[joint];
            let speed = ((at - self.previous[joint]) * DRAG + fall).clamp_length_max(TOP_SPEED);
            let mut to = at + speed;
            to.z = to.z.max(self.lowest);
            self.previous[joint] = at;
            let (point, hit) = self.collide(joint, at, to, world);
            self.points[joint] = point;
            *touch = hit;
        }
        let moved_to = self.points;
        for _ in 0..ROUNDS {
            self.hold();
        }
        // The sticks may have pushed points into the world: trace each from where it
        // was before them.
        let mut moved = 0.0_f32;
        for (joint, &from) in moved_to.iter().enumerate() {
            let to = self.points[joint];
            if let Some(hit) = world.trace(from, to, half(joint)) {
                touched[joint] = touched[joint].or(Some(hit));
                // What is left slides along the surface, as far as the world lets it.
                let rest = to - hit.stop;
                let slide = rest - hit.normal * rest.dot(hit.normal);
                let base = hit.stop + hit.normal * 0.125;
                self.points[joint] = match world.trace(base, base + slide, half(joint)) {
                    None => base + slide,
                    Some(hit) => hit.stop + hit.normal * 0.125,
                };
            }
            moved = moved.max(self.points[joint].distance(began[joint]));
        }
        // A point that leaves the mover it lay on takes the mover's speed with it.
        for (joint, touch) in touched.iter().enumerate() {
            if self.riding[joint].is_some() && touch.is_none() {
                self.previous[joint] -= self.ride[joint];
            }
            self.riding[joint] = touch.and_then(|hit| hit.mover);
        }
        if moved < STILL && !carried {
            self.still += 1;
        } else {
            self.still = 0;
        }
    }

    /// Where a point going from `from` to `to` ends up, and what it met: where it meets
    /// the world, its speed off the surface kept in part ([`FRICTION_KEPT`], [`BOUNCE`]).
    fn collide(
        &mut self,
        joint: usize,
        from: Vec3,
        to: Vec3,
        world: &mut impl World,
    ) -> (Vec3, Option<Hit>) {
        let Some(hit) = world.trace(from, to, half(joint)) else {
            return (to, None);
        };
        let normal = hit.normal;
        let speed = to - from;
        let into = speed.dot(normal);
        let along = speed - normal * into;
        let hard = -into.min(0.0) / STEP;
        let bounce = if hard > BOUNCE_SPEED { BOUNCE } else { 0.0 };
        let after = along * FRICTION_KEPT - normal * into.min(0.0) * bounce;
        let at = hit.stop + normal * 0.125;
        self.previous[joint] = at - after;
        let surface = hit.surface;
        if hard > BOUNCE_SPEED {
            self.knocks.push(Knock {
                joint,
                speed: hard,
                surface,
            });
        }
        let slide = along.length() / STEP;
        if slide > SCRAPE_SPEED && normal != Vec3::ZERO {
            self.scrapes.push(Knock {
                joint,
                speed: slide,
                surface,
            });
        }
        (at, Some(hit))
    }

    /// One round of the sticks, the reaches and the hinges.
    fn hold(&mut self) {
        for (index, &(a, b)) in STICKS.iter().enumerate() {
            self.keep(a, b, self.lengths[index], self.lengths[index]);
        }
        for (index, &(a, b, _)) in REACH.iter().enumerate() {
            let standing = Vec3::from_array(STANDING[a]).distance(Vec3::from_array(STANDING[b]));
            self.keep(a, b, self.reach[index], standing);
        }
        let forward = self.facing();
        for &(upper, middle, lower, forth) in &HINGES {
            let line = (self.points[upper] + self.points[lower]) * 0.5;
            let out = (self.points[middle] - line).dot(forward);
            let wrong = if forth { -out } else { out - 1.5 };
            if wrong > 0.0 {
                let push = forward * if forth { wrong } else { -wrong };
                self.points[middle] += push * 0.5;
                self.points[upper] -= push * 0.25;
                self.points[lower] -= push * 0.25;
            }
        }
    }

    /// Hold `a` and `b` between `shortest` and `longest` apart, each moved half the way.
    fn keep(&mut self, a: usize, b: usize, shortest: f32, longest: f32) {
        let gap = self.points[b] - self.points[a];
        let length = gap.length();
        if length < 1e-4 {
            return;
        }
        let target = length.clamp(shortest, longest);
        if target == length {
            return;
        }
        let shift = gap * ((length - target) / length * 0.5);
        self.points[a] += shift;
        self.points[b] -= shift;
    }

    /// The way the trunk faces: its left (hips, right to left) crossed with its up.
    pub(crate) fn facing(&self) -> Vec3 {
        let left = self.points[L_HIP] - self.points[R_HIP];
        let up = self.points[CHEST] - self.points[PELVIS];
        left.cross(up).normalize_or_zero()
    }
}

/// Half the size of `joint`'s box in the world's collision.
fn half(joint: usize) -> f32 {
    HALVES[joint]
}

/// `Start` as one relay event's text after its prefix: the moment, the origin, the
/// speed and the yaw, whole numbers parted by `_` (the relay takes 64 characters).
pub(crate) fn encode(start: &Start) -> String {
    let [x, y, z] = start.origin;
    let [vx, vy, vz] = start.velocity;
    format!("{}_{x}_{y}_{z}_{vx}_{vy}_{vz}_{}", start.moment, start.yaw)
}

/// The start [`encode`] wrote; `None` for anything else or out of bounds.
pub(crate) fn decode(text: &str) -> Option<Start> {
    let mut parts = text.split('_');
    let mut number = || parts.next()?.parse::<i32>().ok();
    let moment = number()?;
    let origin = [number()?, number()?, number()?];
    let velocity = [number()?, number()?, number()?];
    let yaw = u16::try_from(number()?).ok().filter(|yaw| *yaw < 360)?;
    if parts.next().is_some()
        || origin.iter().any(|c| c.abs() > 65_536)
        || velocity.iter().any(|c| c.abs() > 4_000)
    {
        return None;
    }
    Some(Start {
        moment,
        origin,
        velocity,
        yaw,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A floor at height 0, and a wall at x = 100 if asked.
    fn floor(wall: bool) -> impl FnMut(Vec3, Vec3, f32) -> Option<(Vec3, Vec3)> {
        move |from: Vec3, to: Vec3, half: f32| {
            let mut best: Option<(f32, Vec3)> = None;
            let low = half;
            if to.z < low && from.z >= low - 1e-3 {
                let t = ((from.z - low) / (from.z - to.z)).clamp(0.0, 1.0);
                best = Some((t, Vec3::Z));
            } else if to.z < low {
                best = Some((0.0, Vec3::Z));
            }
            let face = 100.0 - half;
            if wall && to.x > face {
                let t = if from.x <= face {
                    ((face - from.x) / (to.x - from.x)).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                if best.is_none_or(|(bt, _)| t < bt) {
                    best = Some((t, -Vec3::X));
                }
            }
            best.map(|(t, normal)| {
                let mut at = from + (to - from) * t;
                if normal == Vec3::Z {
                    at.z = at.z.max(low);
                } else {
                    at.x = at.x.min(face);
                }
                (at, normal)
            })
        }
    }

    fn start(moment: i32) -> Start {
        Start {
            moment,
            origin: [0, 0, 24],
            velocity: [0, 0, 0],
            yaw: 90,
        }
    }

    #[test]
    fn a_start_reads_back_and_fits_a_relay_event() {
        let start = Start {
            moment: 2_147_000_000,
            origin: [-65_000, 12_345, -4_000],
            velocity: [-3_999, 250, -1_200],
            yaw: 359,
        };
        let text = format!("fall_{}", encode(&start));
        assert!(text.len() <= 64, "{text}");
        assert_eq!(decode(text.strip_prefix("fall_").unwrap()), Some(start));
        assert_eq!(decode("1_2_3"), None);
        assert_eq!(decode("1_0_0_0_0_0_0_360"), None);
        assert_eq!(decode("1_0_0_0_0_0_0_10_5"), None);
    }

    #[test]
    fn the_turn_table_is_close_to_the_real_one() {
        for yaw in 0..360_u16 {
            let (c, s) = turn(yaw);
            let r = f32::from(yaw).to_radians();
            assert!(
                (c - r.cos()).abs() < 0.002 && (s - r.sin()).abs() < 0.002,
                "{yaw}"
            );
        }
    }

    #[test]
    fn the_body_falls_onto_the_floor_and_falls_asleep_lying_down() {
        let mut body = Body::new(&start(1_000));
        let mut world = floor(false);
        body.run_to(MAX_STEPS, &mut world);
        assert!(body.asleep());
        assert!(body.still >= SLEEP_STEPS, "never came to rest");
        let points = body.points();
        for point in points {
            assert!(point.z >= 1.9 && point.z < 20.0, "{point}");
        }
        // Lying, not standing: the head is low, and away from the feet's spot.
        assert!(points[HEAD].z < 15.0);
        // The sticks held.
        let bone = points[L_HIP].distance(points[L_KNEE]);
        let standing =
            Vec3::from_array(STANDING[L_HIP]).distance(Vec3::from_array(STANDING[L_KNEE]));
        assert!((bone - standing).abs() < 1.0, "{bone} {standing}");
    }

    #[test]
    fn the_same_start_falls_the_same_way_however_the_steps_are_taken() {
        let mut world = floor(true);
        let mut once = Body::new(&start(77));
        once.run_to(400, &mut world);
        let mut piecemeal = Body::new(&start(77));
        for steps in (0..=400).step_by(7) {
            piecemeal.run_to(steps, &mut world);
        }
        piecemeal.run_to(400, &mut world);
        assert_eq!(once.points(), piecemeal.points());
        let mut other = Body::new(&start(78));
        other.run_to(400, &mut world);
        assert_ne!(once.points(), other.points());
    }

    /// A lift, mover 7: a floor whose top rises from 0 to 200 between 6 and 8 seconds.
    #[derive(Default)]
    struct Lift {
        top: f32,
        before: f32,
    }

    impl Lift {
        fn height(time: i32) -> f32 {
            ((time - 6_000) as f32 / 10.0).clamp(0.0, 200.0)
        }
    }

    impl World for Lift {
        fn trace(&mut self, from: Vec3, to: Vec3, half: f32) -> Option<Hit> {
            let low = self.top + half;
            if to.z >= low {
                return None;
            }
            let t = if from.z > low {
                (from.z - low) / (from.z - to.z)
            } else {
                0.0
            };
            let mut stop = from + (to - from) * t;
            stop.z = stop.z.max(low);
            Some(Hit {
                stop,
                normal: Vec3::Z,
                surface: 6,
                mover: Some(7),
            })
        }

        fn at(&mut self, time: i32, _low: Vec3, _high: Vec3) {
            self.before = Self::height(time - STEP_MS);
            self.top = Self::height(time);
        }

        fn carry(&mut self, mover: u16, at: Vec3) -> Vec3 {
            assert_eq!(mover, 7);
            at + Vec3::Z * (self.top - self.before)
        }

        fn pusher(&mut self, at: Vec3, half: f32) -> Option<u16> {
            let bottom = at.z - half;
            (bottom < self.top && bottom >= self.before).then_some(7)
        }
    }

    #[test]
    fn a_lift_carries_the_sleeping_body_up_and_it_sleeps_again_on_top() {
        let mut body = Body::new(&start(1_000));
        let mut lift = Lift::default();
        // Lying still on the lift before it moves.
        body.run_to(300, &mut lift);
        assert!(body.asleep());
        assert!(body.points().iter().all(|point| point.z < 20.0));
        assert!(body.knocks.iter().any(|knock| knock.surface == 6));
        // Up with it, still lying (no point left behind or thrown off), and asleep
        // again once it has stopped; the same however the steps are taken.
        let mut piecemeal = body.clone();
        body.run_to(600, &mut lift);
        for point in body.points() {
            assert!(point.z > 200.0 && point.z < 220.0, "{point}");
        }
        assert!(body.asleep());
        for steps in (300..=600).step_by(5) {
            piecemeal.run_to(steps, &mut lift);
        }
        assert_eq!(body.points(), piecemeal.points());
    }

    #[test]
    fn a_body_thrown_at_a_wall_stops_against_it() {
        let mut body = Body::new(&Start {
            velocity: [600, 0, 0],
            ..start(5)
        });
        let mut world = floor(true);
        body.run_to(MAX_STEPS, &mut world);
        for point in body.points() {
            assert!(point.x <= 100.0, "{point}");
        }
        assert!(body.knocks.is_empty() || body.asleep());
    }
}
