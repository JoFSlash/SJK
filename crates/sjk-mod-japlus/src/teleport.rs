//! Teleports through JA+'s `amtele`, from JoF EternalJK's `teleGun`, `get`, `goto`,
//! `amTeleOffset`, `PTelemark` and `PTele` (`codemp/cgame/cg_consolecmds.c:88-162,
//! 2570-2696`). The client only works out where; the server checks the admin rights.

use sjk_mod::{Host, Me, Player, Trace, forward, number, resolve_player};

/// How far above the floor a teleport lands, as JA+'s own teleports do.
const LIFT: f32 = 24.0;
/// `get` and `goto` put the player this far in front by default.
const DISTANCE: f32 = 100.0;
/// `goto` trusts a drawn position for this long before asking the server.
const STALE_MS: i32 = 1_000;

fn me(host: &dyn Host) -> Result<Me, String> {
    host.me().ok_or_else(|| "not in a game".to_owned())
}

fn add(origin: [f32; 3], direction: [f32; 3], distance: f32) -> [f32; 3] {
    std::array::from_fn(|axis| origin[axis] + direction[axis] * distance)
}

fn tele(position: [f32; 3], yaw: f32) -> String {
    format!(
        "amtele {:.6} {:.6} {:.6} {:.6}",
        position[0], position[1], position[2], yaw
    )
}

/// `japlus.guntele [distance] [yaw offset]`: to the crosshair's point, or
/// `distance` straight ahead.
///
/// With a distance it departs from EternalJK's `teleGun`, which follows the
/// view pitch included and so lands under the floor when you look down, or
/// inside a wall in the way (hub report #218; the change was the player's own
/// decision): the way is level (yaw only) and the player's box is swept along it
/// through the map (`trace`, [`Host::trace_player`]), stopping [`WALL_GAP`]
/// short of the first wall or step. A box that starts inside something solid
/// (noclip in a wall) lands on the crosshair's point as without a distance, or
/// stays where it is without one.
pub(crate) fn gun_tele(
    args: &[String],
    me: Me,
    crosshair: Option<[f32; 3]>,
    trace: impl Fn([f32; 3], [f32; 3]) -> Option<Trace>,
) -> Result<String, String> {
    let yaw = me.view_angles[1];
    let on_crosshair = || {
        crosshair
            .filter(|point| *point != [0.0; 3])
            .map(|point| [point[0], point[1], point[2] + LIFT])
    };
    let Some(distance) = args.first() else {
        let point = on_crosshair().ok_or("aim at a surface first")?;
        return Ok(tele(point, yaw));
    };
    let offset = args.get(1).map_or(0.0, |text| number(text));
    let position = level_destination(me.origin, yaw, number(distance), Walls::Stop, trace)
        .or_else(on_crosshair)
        .unwrap_or(me.origin);
    Ok(tele(position, yaw + offset))
}

/// `japlus.guntelewall <distance> [yaw offset]`: `distance` straight ahead,
/// level as [`gun_tele`], but through walls as EternalJK's `teleGun` goes (hub
/// report #232). When the player's box would end inside something solid
/// (`trace`, [`Host::trace_player`]) it lands on the nearest free spot back along
/// the way, or else where `japlus.guntele` would, [`WALL_GAP`] short of the first
/// wall. From inside a wall (noclip) it goes to the crosshair's point, or stays.
pub(crate) fn gun_tele_wall(
    args: &[String],
    me: Me,
    crosshair: Option<[f32; 3]>,
    trace: impl Fn([f32; 3], [f32; 3]) -> Option<Trace>,
) -> Result<String, String> {
    let distance = args
        .first()
        .ok_or("usage: japlus.guntelewall <distance> [yaw offset]")?;
    let yaw = me.view_angles[1];
    let offset = args.get(1).map_or(0.0, |text| number(text));
    let position = level_destination(me.origin, yaw, number(distance), Walls::Through, trace)
        .or_else(|| {
            crosshair
                .filter(|point| *point != [0.0; 3])
                .map(|point| [point[0], point[1], point[2] + LIFT])
        })
        .unwrap_or(me.origin);
    Ok(tele(position, yaw + offset))
}

/// What a distance teleport does about a wall in the way.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Walls {
    /// Stop [`WALL_GAP`] short of it (`japlus.guntele`).
    Stop,
    /// Go through it, landing out of solid (`japlus.guntelewall`).
    Through,
}

/// How far short of a wall a distance teleport stops: the server keeps whole
/// units (`atoi`), which can move the player up to one unit towards it.
pub(crate) const WALL_GAP: f32 = 2.0;

/// How far back along the way each try for a free spot moves when a teleport
/// through walls would end inside something solid.
pub(crate) const FREE_STEP: f32 = 4.0;

/// `distance` level along `yaw` from `origin` (nothing in the way without a map).
/// [`Walls::Stop`] stops [`WALL_GAP`] short of what the player's box meets on the
/// way (`trace`). [`Walls::Through`] goes the whole way, and when the box would
/// end inside something solid (`trace` from the point to itself starts solid)
/// takes the free spot nearest the end going back in [`FREE_STEP`]s, down to where
/// the box first meets a wall; with none, it stops as [`Walls::Stop`] does.
/// `None` when the box starts inside something solid.
fn level_destination(
    origin: [f32; 3],
    yaw: f32,
    distance: f32,
    walls: Walls,
    trace: impl Fn([f32; 3], [f32; 3]) -> Option<Trace>,
) -> Option<[f32; 3]> {
    let direction = forward([0.0, yaw, 0.0]);
    let wanted = add(origin, direction, distance);
    let Some(hit) = trace(origin, wanted) else {
        return Some(wanted);
    };
    if hit.start_solid {
        return None;
    }
    if hit.fraction >= 1.0 {
        return Some(wanted);
    }
    let along = |travelled: f32| add(origin, direction, travelled.copysign(distance));
    let first_wall = distance.abs() * hit.fraction;
    if walls == Walls::Through {
        let free = |point: [f32; 3]| trace(point, point).is_some_and(|stay| !stay.start_solid);
        let mut travelled = distance.abs();
        while travelled > first_wall {
            let point = along(travelled);
            if free(point) {
                return Some(point);
            }
            travelled -= FREE_STEP;
        }
    }
    Some(along((first_wall - WALL_GAP).max(0.0)))
}

/// The player `get`/`goto` act on: the crosshair's without a name or with `gun`.
fn target(args: &[String], host: &dyn Host, players: &[Player]) -> Result<u16, String> {
    match args.first() {
        Some(query) if !query.eq_ignore_ascii_case("gun") => resolve_player(players, query),
        _ => host
            .crosshair_player()
            .ok_or_else(|| "aim at a player, or name one".to_owned()),
    }
}

/// The distance and yaw offset after the player argument.
fn placement(args: &[String]) -> (f32, f32) {
    (
        args.get(1).map_or(DISTANCE, |text| number(text)),
        args.get(2).map_or(0.0, |text| number(text)),
    )
}

/// `japlus.bring [id|name|gun] [distance] [yaw offset]`: the player to in front
/// of you, facing you.
pub(crate) fn bring(args: &[String], host: &dyn Host) -> Result<String, String> {
    let me = me(host)?;
    let client = target(args, host, &host.players())?;
    let (distance, offset) = placement(args);
    let mut angles = me.view_angles;
    // Without a distance the player lands level with you, lifted off the floor.
    let level = args.len() <= 1;
    if level {
        angles[0] = 0.0;
        angles[2] = 0.0;
    }
    let mut position = add(me.origin, forward(angles), distance);
    if level {
        position[2] = me.origin[2] + LIFT;
    }
    Ok(format!(
        "amtele {client} {:.6} {:.6} {:.6} {:.6}",
        position[0],
        position[1],
        position[2],
        me.view_angles[1] + 180.0 + offset
    ))
}

/// `japlus.goto [id|name|gun] [distance] [yaw offset]`: you to in front of the
/// player, facing it; to the player itself when it is not drawn.
pub(crate) fn go_to(args: &[String], host: &dyn Host) -> Result<(String, Option<String>), String> {
    me(host)?;
    let players = host.players();
    let client = target(args, host, &players)?;
    let drawn = players
        .iter()
        .find(|player| player.client == client)
        .and_then(|player| player.drawn)
        .filter(|drawn| drawn.age_ms < STALE_MS);
    let Some(drawn) = drawn else {
        return Ok((
            format!("amtele {client}"),
            Some("^3Player not in view; teleporting to it through the server".into()),
        ));
    };
    let (distance, offset) = placement(args);
    let mut angles = drawn.angles;
    if args.len() <= 1 {
        angles[0] = 0.0;
        angles[2] = 0.0;
    }
    let mut position = add(drawn.origin, forward(angles), distance);
    if args.len() == 1 {
        position[2] = drawn.base_z + LIFT;
    }
    Ok((tele(position, drawn.angles[1] + 180.0 + offset), None))
}

/// `japlus.teleoffset x [y [z]]`: by whole units from where you are.
pub(crate) fn offset(args: &[String], me: Me) -> Result<String, String> {
    if args.is_empty() || args.len() > 3 {
        return Err("usage: japlus.teleoffset x [y [z]]".into());
    }
    let mut position = me.origin;
    for (axis, text) in args.iter().enumerate() {
        position[axis] += number(text).trunc();
    }
    Ok(tele(position, me.view_angles[1]))
}

/// A position kept by `japlus.mark`, in whole units as `PTelemark` keeps it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Mark([i32; 4]);

impl Mark {
    pub(crate) fn of(me: Me) -> Self {
        let [x, y, z] = me.origin.map(|value| value as i32);
        Self([x, y, z, me.view_angles[1] as i32])
    }

    pub(crate) fn describe(self) -> String {
        let [x, y, z, yaw] = self.0;
        format!("Mark set ({x} {y} {z}) : {yaw}")
    }

    /// The server command `japlus.recall` sends.
    pub(crate) fn recall(self) -> String {
        let [x, y, z, yaw] = self.0;
        format!("setviewpos {x} {y} {z} {yaw}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjk_mod::{Drawn, Server};

    fn me() -> Me {
        Me {
            client: 0,
            origin: [100.0, 200.0, 24.0],
            view_angles: [30.0, 90.0, 0.0],
            intermission: false,
        }
    }

    fn args(text: &str) -> Vec<String> {
        text.split_whitespace().map(str::to_owned).collect()
    }

    struct Fake {
        crosshair: Option<u16>,
        players: Vec<Player>,
    }

    impl Host for Fake {
        fn send(&mut self, _: &str) -> Result<(), String> {
            Ok(())
        }
        fn server(&self) -> Option<Server> {
            None
        }
        fn me(&self) -> Option<Me> {
            Some(me())
        }
        fn crosshair_point(&self) -> Option<[f32; 3]> {
            None
        }
        fn crosshair_player(&self) -> Option<u16> {
            self.crosshair
        }
        fn players(&self) -> Vec<Player> {
            self.players.clone()
        }
        fn cvar(&self, _: &str) -> Option<String> {
            None
        }
        fn set_cvar(&mut self, _: &str, _: &str) -> Result<(), String> {
            Ok(())
        }
    }

    fn fake(age_ms: i32) -> Fake {
        Fake {
            crosshair: Some(4),
            players: vec![Player {
                client: 4,
                name: "Kyle".into(),
                drawn: Some(Drawn {
                    origin: [0.0, 0.0, 40.0],
                    angles: [10.0, 0.0, 0.0],
                    base_z: 32.0,
                    age_ms,
                }),
            }],
        }
    }

    /// No map: nothing in the way.
    fn open(_: [f32; 3], _: [f32; 3]) -> Option<Trace> {
        None
    }

    /// A map whose box sweeps stop at `fraction`, or start inside something.
    fn wall(fraction: f32, start_solid: bool) -> impl Fn([f32; 3], [f32; 3]) -> Option<Trace> {
        move |from, to| {
            Some(Trace {
                fraction,
                end: std::array::from_fn(|axis| from[axis] + (to[axis] - from[axis]) * fraction),
                start_solid,
            })
        }
    }

    #[test]
    fn gun_tele_lands_on_the_crosshair_point_or_ahead() {
        assert_eq!(
            gun_tele(&[], me(), Some([1.0, 2.0, 3.0]), open).unwrap(),
            "amtele 1.000000 2.000000 27.000000 90.000000"
        );
        assert!(gun_tele(&[], me(), Some([0.0; 3]), open).is_err());
        assert!(gun_tele(&[], me(), None, open).is_err());
        // Open space: 100 units straight ahead, level although the view is 30
        // degrees down (EternalJK went 50 units down, under the floor), turned 45.
        assert_eq!(
            gun_tele(&args("100 45"), me(), None, open).unwrap(),
            "amtele 99.999992 300.000000 24.000000 135.000000"
        );
        assert_eq!(
            gun_tele(&args("100"), me(), None, wall(1.0, false)).unwrap(),
            "amtele 99.999992 300.000000 24.000000 90.000000"
        );
    }

    /// Hub report #218: a wall in the way stops the teleport short of it, a
    /// negative distance included, and never through it.
    #[test]
    fn gun_tele_stops_short_of_a_wall_in_the_way() {
        // The box meets the wall at 120 of 300 units: it lands 118 ahead.
        assert_eq!(
            gun_tele(&args("300"), me(), None, wall(0.4, false)).unwrap(),
            "amtele 99.999992 318.000000 24.000000 90.000000"
        );
        // Backwards the same way.
        assert_eq!(
            gun_tele(&args("-300"), me(), None, wall(0.4, false)).unwrap(),
            "amtele 100.000008 82.000000 24.000000 90.000000"
        );
        // A wall right in front: it stays where it is.
        assert_eq!(
            gun_tele(&args("300"), me(), None, wall(0.0, false)).unwrap(),
            "amtele 100.000000 200.000000 24.000000 90.000000"
        );
    }

    /// A map of walls across the way north (+y), each solid from one y to
    /// another, swept with the standing box (15 units each side).
    fn walls(solid: &'static [(f32, f32)]) -> impl Fn([f32; 3], [f32; 3]) -> Option<Trace> {
        move |from, to| {
            let inside = |y: f32| {
                solid
                    .iter()
                    .any(|&(low, high)| y + 15.0 > low && y - 15.0 < high)
            };
            if inside(from[1]) {
                return Some(Trace {
                    fraction: 0.0,
                    end: from,
                    start_solid: true,
                });
            }
            let length = to[1] - from[1];
            let fraction = solid
                .iter()
                .filter_map(|&(low, high)| {
                    let contact = if length > 0.0 {
                        low - 15.0
                    } else {
                        high + 15.0
                    };
                    let fraction = (contact - from[1]) / length;
                    (0.0..1.0).contains(&fraction).then_some(fraction)
                })
                .fold(1.0, f32::min);
            Some(Trace {
                fraction,
                end: std::array::from_fn(|axis| from[axis] + (to[axis] - from[axis]) * fraction),
                start_solid: false,
            })
        }
    }

    /// Hub report #232: `japlus.guntelewall` goes the whole distance through a
    /// wall in the way, as EternalJK's `teleGun` does, level and turned by the
    /// offset, backwards too; it needs a distance.
    #[test]
    fn gun_tele_wall_goes_through_a_wall_in_the_way() {
        assert_eq!(
            gun_tele_wall(&args("300 45"), me(), None, walls(&[(300.0, 340.0)])).unwrap(),
            "amtele 99.999985 500.000000 24.000000 135.000000"
        );
        assert_eq!(
            gun_tele_wall(&args("-300"), me(), None, walls(&[(60.0, 100.0)])).unwrap(),
            "amtele 100.000015 -100.000000 24.000000 90.000000"
        );
        // A wall right in front makes no difference.
        assert_eq!(
            gun_tele_wall(&args("300"), me(), None, walls(&[(216.0, 260.0)])).unwrap(),
            "amtele 99.999985 500.000000 24.000000 90.000000"
        );
        // No map: straight there.
        assert_eq!(
            gun_tele_wall(&args("300"), me(), None, open).unwrap(),
            "amtele 99.999985 500.000000 24.000000 90.000000"
        );
        assert!(gun_tele_wall(&[], me(), Some([1.0, 2.0, 3.0]), open).is_err());
        // japlus.guntele still stops 2 units short of the same wall.
        assert_eq!(
            gun_tele(&args("300"), me(), None, walls(&[(300.0, 340.0)])).unwrap(),
            "amtele 100.000000 283.000000 24.000000 90.000000"
        );
    }

    /// Hub report #232: a `japlus.guntelewall` that would end inside a wall lands
    /// back out of it, on the free spot nearest the end, and short of the first
    /// wall only when nothing past it is free.
    #[test]
    fn gun_tele_wall_lands_back_out_of_a_wall_at_the_end() {
        // Through a wall 300 to 340 north, into one 450 to 600: the box at 500
        // is inside it, and the nearest free spot back is 432 (box edge at 447).
        assert_eq!(
            gun_tele_wall(
                &args("300"),
                me(),
                None,
                walls(&[(300.0, 340.0), (450.0, 600.0)])
            )
            .unwrap(),
            "amtele 99.999992 432.000000 24.000000 90.000000"
        );
        // Into a wall with nothing free past the one before it: 2 units short of
        // it, where japlus.guntele stops (the box meets it at 435).
        assert_eq!(
            gun_tele_wall(&args("300"), me(), None, walls(&[(450.0, 600.0)])).unwrap(),
            "amtele 99.999992 433.000000 24.000000 90.000000"
        );
        // From inside a wall: the crosshair's point, else where you are.
        assert_eq!(
            gun_tele_wall(&args("300"), me(), Some([1.0, 2.0, 3.0]), wall(0.0, true)).unwrap(),
            "amtele 1.000000 2.000000 27.000000 90.000000"
        );
        assert_eq!(
            gun_tele_wall(&args("300"), me(), None, wall(0.0, true)).unwrap(),
            "amtele 100.000000 200.000000 24.000000 90.000000"
        );
    }

    /// Starting inside something solid (noclip in a wall), the sweep says
    /// nothing: the crosshair's point as without a distance, else where you are.
    #[test]
    fn gun_tele_from_inside_a_wall_uses_the_crosshair_or_stays() {
        assert_eq!(
            gun_tele(&args("300"), me(), Some([1.0, 2.0, 3.0]), wall(0.0, true)).unwrap(),
            "amtele 1.000000 2.000000 27.000000 90.000000"
        );
        assert_eq!(
            gun_tele(&args("300"), me(), None, wall(0.0, true)).unwrap(),
            "amtele 100.000000 200.000000 24.000000 90.000000"
        );
    }

    #[test]
    fn bring_puts_the_target_level_in_front_facing_you() {
        let host = fake(0);
        assert_eq!(
            bring(&[], &host).unwrap(),
            "amtele 4 99.999992 300.000000 48.000000 270.000000"
        );
        // With a distance the view's pitch counts and nothing is lifted.
        assert_eq!(
            bring(&args("kyle 50 10"), &host).unwrap(),
            "amtele 4 100.000000 243.301270 -1.000000 280.000000"
        );
    }

    #[test]
    fn goto_uses_the_drawn_player_or_asks_the_server() {
        let (command, note) = go_to(&[], &fake(0)).unwrap();
        assert_eq!(command, "amtele 100.000000 0.000000 40.000000 180.000000");
        assert!(note.is_none());
        // A name alone lifts you above the player's sent position.
        let (command, _) = go_to(&args("gun"), &fake(0)).unwrap();
        assert_eq!(command, "amtele 100.000000 0.000000 56.000000 180.000000");
        let (command, note) = go_to(&[], &fake(1_000)).unwrap();
        assert_eq!(command, "amtele 4");
        assert!(note.is_some());
    }

    #[test]
    fn offset_moves_by_whole_units() {
        assert_eq!(
            offset(&args("10.9 -5"), me()).unwrap(),
            "amtele 110.000000 195.000000 24.000000 90.000000"
        );
        assert!(offset(&[], me()).is_err());
        assert!(offset(&args("1 2 3 4"), me()).is_err());
    }

    #[test]
    fn a_mark_recalls_through_setviewpos() {
        let mark = Mark::of(me());
        assert_eq!(mark.describe(), "Mark set (100 200 24) : 90");
        assert_eq!(mark.recall(), "setviewpos 100 200 24 90");
    }
}
