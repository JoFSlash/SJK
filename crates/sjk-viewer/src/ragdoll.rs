//! The Ragdoll, SJK's meme toy (`toy_ragdoll`; Sol's idea, 23/10/2026): the player's
//! body goes limp and falls where physics takes it, as in the sandbox games, though the
//! player is not dead; `toy_ragdoll` again gets them up.
//!
//! No game server knows of it. The body is the client's: a simulated humanoid
//! ([`body`]) drawn with the player's own model ([`skin`]), the camera following it
//! from behind (the mouse turns it). Every SJK client on the server shows the same fall
//! without any pose being sent: the faller's client relays one start event to every SJK
//! player there (`fall_<moment>_<origin>_<speed>_<yaw>`, the toy relay, `PROTOCOL.md`
//! "Toy relay"), and each runs the same simulation from it in fixed steps of the game
//! server's clock, fast-forwarding a fall it hears late. Getting up relays `up_<moment>`.
//! A player who comes onto the server (or whose SJK client starts) during a fall is told
//! of it too.
//!
//! The player's real body stays on the game server: while limp, the client walks it to
//! where the ragdoll lies (its own user commands, as the Hax's steering), so that the
//! player gets up there. Every SJK client draws the ragdoll in its place; players on
//! stock clients see a player walking about. A limp player sends no button, so cannot
//! attack, and gets up at once when hurt, dead, spectating, in a duel or carrying a flag
//! ([`crate::order66::spared`]); nobody falls in one of those either.
//!
//! The fall is heard: an original flatline, the toy's own ([`FLATLINE`], made by
//! `scripts/ragdoll_assets.py`; a PK3 with the same path replaces it), at the body, and
//! every part's knocks and scrapes on the world, flesh and the surface's material
//! ([`sounds`]). Lifts, doors and trains carry and push the body ([`scene`]).

mod body;
mod scene;
mod skin;
mod sounds;

use crate::GpuState;
use crate::ghost_replay::Drive;
use body::{Body, JOINTS, PELVIS, STEP_MS, Start};
use glam::Vec3;
use sjk_protocol::UserCommand;
use sjk_runtime::EntityId;
use std::net::SocketAddr;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

pub(crate) use skin::Scratch;

/// The toy's console command: fall, or get up.
pub(crate) const COMMAND: &str = "toy_ragdoll";
/// The toy's id in relayed toy events.
pub(crate) const TOY: &str = "ragdoll";
/// The toy's icon (quick wheel, Collection, key bindings).
pub(crate) const ICON: &str = "gfx/sjk/toy_ragdoll.png";
/// Heard as the body goes limp.
pub(crate) const FLATLINE: &str = "sound/sjk/ragdoll/flatline.wav";
/// The start event: this, then the start ([`body::encode`]); the end: this, then the
/// fall's moment.
const FALL_PREFIX: &str = "fall_";
const UP_PREFIX: &str = "up_";
/// Between two start events sent (the relay takes 40 a second from a player).
const TELL_GAP: Duration = Duration::from_millis(35);
/// A fall heard this far after its moment, or before it, is dropped (ms).
const LATE_MS: i64 = 600_000;
const EARLY_MS: i64 = 5_000;
/// A fallen player whose real body is further than this from the ragdoll for
/// [`ASTRAY`] got up without the news reaching here: their ragdoll goes.
const ASTRAY_REACH: f32 = 320.0;
const ASTRAY: Duration = Duration::from_secs(3);
/// The limp player's real body walks to the ragdoll: it stops this close, and walks
/// (rather than runs) this close.
const WALK_STOP: f32 = 12.0;
const WALK_NEAR: f32 = 72.0;
/// `BUTTON_WALKING`.
const BUTTON_WALKING: u16 = 16;
/// The camera: above the pelvis, and how far behind.
const CAMERA_HEIGHT: f32 = 12.0;
const CAMERA_RANGE: f32 = 110.0;
const CAMERA_RISE: f32 = 10.0;
const SLOTS: usize = crate::looks::SLOTS;
const ET_PLAYER: u8 = 1;
const EF_DEAD: u32 = 1 << 1;
/// `EF_TELEPORT_BIT`, flipped at every teleport and respawn.
const EF_TELEPORT_BIT: u32 = 1 << 3;
/// `EF_NODRAW`.
const EF_NODRAW: u32 = 1 << 8;
/// `PMF_FOLLOW`.
const PMF_FOLLOW: u16 = 4096;

/// A bundled file's bytes, by its name in `assets/ragdoll`.
macro_rules! bundled {
    ($name:literal) => {
        include_bytes!(concat!("../assets/ragdoll/", $name))
    };
}

/// The bundled files at their game paths (made by `scripts/ragdoll_assets.py`).
const FILES: [(&str, &[u8]); 2] = [
    (ICON, bundled!("toy_ragdoll.png")),
    (FLATLINE, bundled!("flatline.wav")),
];

/// Mount the bundled files.
pub(crate) fn mount(vfs: &mut sjk_vfs::VirtualFileSystem) -> Result<(), sjk_vfs::VfsError> {
    vfs.mount_memory("SJK Ragdoll", FILES.iter().copied())?;
    Ok(())
}

/// A relayed event of the toy, as read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Event {
    Fall(Start),
    Up(i32),
}

impl Event {
    fn parse(event: &str) -> Option<Self> {
        if let Some(start) = event.strip_prefix(FALL_PREFIX) {
            return body::decode(start).map(Self::Fall);
        }
        event.strip_prefix(UP_PREFIX)?.parse().ok().map(Self::Up)
    }

    fn text(&self) -> String {
        match self {
            Self::Fall(start) => format!("{FALL_PREFIX}{}", body::encode(start)),
            Self::Up(moment) => format!("{UP_PREFIX}{moment}"),
        }
    }
}

/// One body falling or lying, the player's own or another's.
struct Fall {
    start: Start,
    body: Body,
    /// When its parts' knocks were last heard.
    heard: sounds::Heard,
    /// The `EF_TELEPORT_BIT` the faller had (another's): a flip is a respawn.
    teleport: Option<u32>,
    /// Since when their real body has been far from the ragdoll (another's).
    astray: Option<Instant>,
}

impl Fall {
    fn new(start: Start) -> Self {
        Self {
            body: Body::new(&start),
            start,
            heard: sounds::Heard::default(),
            teleport: None,
            astray: None,
        }
    }
}

/// The player's own fall: the moment, who has been told (bits by slot), when the next
/// may be, the health it began with, and the camera's angles to give back.
struct Mine {
    moment: i32,
    told: u64,
    tell_at: Instant,
    health: i32,
    camera: (f32, f32),
}

/// The toy's state.
#[derive(Default)]
pub(crate) struct Ragdoll {
    server: Option<SocketAddr>,
    /// `toy_ragdoll` was pressed since the last frame.
    toggled: bool,
    falls: Vec<Option<Fall>>,
    mine: Option<Mine>,
    /// The player's own slot, and where its real body is to walk to.
    own: Option<u8>,
    walk_to: Option<Vec3>,
    /// The sounds to play.
    cues: Vec<Cue>,
    /// The movers in the bodies' way.
    movers: scene::Movers,
    sounds_registered: Option<u64>,
    /// The game server time last stepped to.
    time: i64,
    /// For the world shots: what is shown is left as a shot set it.
    #[cfg(test)]
    shot_frozen: bool,
}

/// Each slot's ragdoll as shown this frame (world space), for the actors' posing and
/// placing, which run without the viewer's state.
static SHOWN: LazyLock<Mutex<[Option<[Vec3; JOINTS]>; SLOTS]>> =
    LazyLock::new(|| Mutex::new([None; SLOTS]));

fn shown_lock() -> std::sync::MutexGuard<'static, [Option<[Vec3; JOINTS]>; SLOTS]> {
    SHOWN
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The ragdoll of the player in `slot` this frame, world space, if they are limp.
pub(crate) fn shown(slot: u8) -> Option<[Vec3; JOINTS]> {
    shown_lock().get(usize::from(slot)).copied().flatten()
}

/// The ragdoll of the player whose actor is `entity` (`slot + 1`), if any.
pub(crate) fn shown_entity(entity: EntityId) -> Option<[Vec3; JOINTS]> {
    let slot = u8::try_from(entity.get().checked_sub(1)?).ok()?;
    shown(slot)
}

/// Pose an actor's bones for the ragdoll `points` (world space) into `scratch`: false
/// when its skeleton is not the stock humanoid's (it is then drawn as animated).
pub(crate) fn pose(
    scratch: &mut Scratch,
    gla: &std::sync::Arc<sjk_model::Gla>,
    points: &[Vec3; JOINTS],
) -> bool {
    scratch.pose(gla, &skin::model_space(points))
}

/// Where an actor posed by [`pose`] is placed: at the pelvis, unturned, unscaled.
pub(crate) fn place(transform: &mut sjk_runtime::Transform, points: &[Vec3; JOINTS]) {
    transform.translation = points[PELVIS].to_array();
    transform.rotation = glam::Quat::IDENTITY.to_array();
    transform.scale = [1.0; 3];
}

/// The game server's 16-bit angle for `degrees` (`ANGLE2SHORT`).
fn short(degrees: f32) -> i32 {
    (degrees * 65_536.0 / 360.0) as i32 & 0xFFFF
}

impl Ragdoll {
    /// `toy_ragdoll`: fall, or get up (at the next frame).
    pub(crate) fn toggle(&mut self) {
        self.toggled = true;
    }

    /// Go limp (`true`) or get up, at the next frame, unless already so.
    pub(crate) fn set_on(&mut self, on: bool) {
        self.toggled = on != self.on();
    }

    /// Whether the player's own body is limp.
    pub(crate) fn on(&self) -> bool {
        self.mine.is_some()
    }

    /// The command to send in place of `input` while the player is limp: no button, the
    /// real body walking to the ragdoll; `delta` the player's `delta_angles` and `from`
    /// where the real body is.
    pub(crate) fn drive(
        &self,
        input: &UserCommand,
        delta: [i32; 3],
        from: [f32; 3],
    ) -> Option<Drive> {
        self.mine.as_ref()?;
        let mut command = UserCommand {
            forward_move: 0,
            right_move: 0,
            up_move: 0,
            buttons: 0,
            generic_command: 0,
            ..*input
        };
        let way = self
            .walk_to
            .map(|to| (to - Vec3::from_array(from)).truncate())
            .unwrap_or_default();
        let distance = way.length();
        let yaw = if distance > 0.5 {
            way.y.atan2(way.x).to_degrees()
        } else {
            0.0
        };
        let view = [0, short(yaw), input.angles[2].wrapping_add(delta[2])];
        command.angles = std::array::from_fn(|i| view[i].wrapping_sub(delta[i]) & 0xFFFF);
        if distance > WALK_STOP {
            command.forward_move = 127;
            if distance < WALK_NEAR {
                command.buttons = BUTTON_WALKING;
            }
        }
        Some(Drive {
            command,
            view: None,
        })
    }

    /// Shape a command that is only shown (the prediction's preview), as
    /// [`Ragdoll::drive`] would.
    pub(crate) fn shape_preview(&self, command: &mut UserCommand, delta: [i32; 3], from: [f32; 3]) {
        if let Some(drive) = self.drive(command, delta, from) {
            *command = drive.command;
        }
    }

    /// Forget every fall: no game, or another server.
    fn leave(&mut self) {
        self.server = None;
        self.falls.clear();
        self.mine = None;
        self.walk_to = None;
        *shown_lock() = [None; SLOTS];
    }

    fn fall_mut(&mut self, slot: u8) -> Option<&mut Fall> {
        self.falls.get_mut(usize::from(slot))?.as_mut()
    }

    fn set_fall(&mut self, slot: u8, fall: Option<Fall>) {
        let index = usize::from(slot);
        if index >= SLOTS {
            return;
        }
        if self.falls.len() <= index {
            self.falls.resize_with(index + 1, || None);
        }
        self.falls[index] = fall;
    }
}

/// Send `event` to the player in `slot`.
fn relay(event: Event, slot: u8) {
    crate::player_identity::relay_toy_event(TOY, event.text(), slot, false);
}

/// A sound of the toy to play: its path, where (`None`: the player's own, not placed),
/// whose body, and how loud.
struct Cue {
    path: &'static str,
    at: Option<Vec3>,
    slot: u8,
    volume: f32,
}

impl Cue {
    fn flatline(at: Option<Vec3>, slot: u8) -> Self {
        Self {
            path: FLATLINE,
            at,
            slot,
            volume: 1.0,
        }
    }
}

impl GpuState {
    /// `toy_ragdoll`: fall, or get up.
    pub(crate) fn toggle_ragdoll(&mut self) {
        self.ragdoll.toggle();
    }

    /// Say `text` on the centre of the screen.
    fn ragdoll_centre(&mut self, text: &str) {
        self.chat.receive(
            sjk_client::ServerEventKind::CenterPrint,
            text.to_owned(),
            None,
            Instant::now(),
        );
    }

    /// The frame's toy logic, from the relay's events: falls heard and ended, the
    /// player's own fall begun or ended, and those to tell of it.
    pub(crate) fn update_ragdoll(&mut self, relay_events: &[sjk_identity::ToyEvent]) {
        let Some(session) = self
            .live_session
            .as_ref()
            .filter(|session| !session.is_local())
        else {
            if self.ragdoll.server.is_some() {
                self.ragdoll_get_up(false);
                self.ragdoll.leave();
            }
            self.ragdoll.toggled = false;
            return;
        };
        let server = session.server();
        if self.ragdoll.server != Some(server) {
            self.ragdoll.leave();
            self.ragdoll.server = Some(server);
        }
        let snapshot = session.latest_snapshot();
        let game_state = session.game_state();
        let own = u8::try_from(game_state.client_num).ok();
        self.ragdoll.own = own;
        let gametype = game_state
            .config_string(0)
            .and_then(|info| sjk_client::LegacyClientInfo::new(info).integer("g_gametype"))
            .unwrap_or(0);
        let spared = crate::order66::spared(&snapshot.player, gametype);
        let health = snapshot.player.health();
        let now = self.ragdoll.time;
        // The relay's events: a sender counts only when the game shows the name they
        // claimed, and never one blocked on this PC.
        let heard: Vec<(u8, Event)> = relay_events
            .iter()
            .filter(|event| {
                event.toy == TOY
                    && Some(event.slot) != own
                    && usize::from(event.slot) < SLOTS
                    && !self.muted_players.contains(u16::from(event.slot))
                    && crate::player_identity::shown_name(game_state, usize::from(event.slot))
                        .is_some_and(|shown| sjk_identity::names_match(&event.claimed_name, &shown))
            })
            .filter_map(|event| Some((event.slot, Event::parse(&event.event)?)))
            .collect();
        for (slot, event) in heard {
            match event {
                Event::Fall(start) => {
                    let moment = i64::from(start.moment);
                    let known = self
                        .ragdoll
                        .fall_mut(slot)
                        .is_some_and(|fall| fall.start == start);
                    if !known && moment <= now + EARLY_MS && moment >= now - LATE_MS {
                        self.ragdoll.set_fall(slot, Some(Fall::new(start)));
                        if moment >= now - 1_000 {
                            self.ragdoll
                                .cues
                                .push(Cue::flatline(Some(start_point(&start)), slot));
                        }
                    }
                }
                Event::Up(moment) => {
                    if self
                        .ragdoll
                        .fall_mut(slot)
                        .is_some_and(|fall| fall.start.moment == moment)
                    {
                        self.ragdoll.set_fall(slot, None);
                    }
                }
            }
        }
        // The player's own fall.
        let toggled = std::mem::take(&mut self.ragdoll.toggled);
        if let Some(mine) = &self.ragdoll.mine {
            let why = if toggled {
                None
            } else if spared.is_some() {
                Some("")
            } else if health < mine.health {
                Some("^3Ouch!^7 You got up")
            } else {
                None
            };
            if toggled || why.is_some() {
                self.ragdoll_get_up(true);
                if let Some(why) = why.filter(|why| !why.is_empty()) {
                    self.ragdoll_centre(why);
                }
            }
        } else if toggled {
            match spared {
                Some(crate::order66::Spared::Duel) => self.ragdoll_centre("No ragdoll in a duel"),
                Some(crate::order66::Spared::Flag) => {
                    self.ragdoll_centre("No ragdoll while carrying the flag");
                }
                Some(_) => {}
                None if self.hax.busy() => self.ragdoll_centre("No ragdoll in a hack"),
                None => self.ragdoll_fall(health),
            }
        }
        self.ragdoll_tell();
    }

    /// The player falls, now.
    fn ragdoll_fall(&mut self, health: i32) {
        let Some(own) = self.ragdoll.own else {
            return;
        };
        let Some(state) = self.local_prediction.predicted_state() else {
            return;
        };
        let round = |v: f32| v.round() as i32;
        let yaw = self.camera_yaw.to_degrees().rem_euclid(360.0).round() as u16 % 360;
        let start = Start {
            moment: self.ragdoll.time as i32,
            origin: state.origin.map(round),
            velocity: state.velocity.map(|v| round(v.clamp(-3_999.0, 3_999.0))),
            yaw,
        };
        self.ragdoll.set_fall(own, Some(Fall::new(start)));
        self.ragdoll.mine = Some(Mine {
            moment: start.moment,
            told: 0,
            tell_at: Instant::now(),
            health,
            camera: (self.camera_pitch, self.camera_yaw),
        });
        self.ragdoll.cues.push(Cue::flatline(None, own));
    }

    /// The player gets up: everyone told hears of it (`tell`), the camera's angles are
    /// given back.
    fn ragdoll_get_up(&mut self, tell: bool) {
        let Some(mine) = self.ragdoll.mine.take() else {
            return;
        };
        if tell {
            for slot in 0..SLOTS as u8 {
                if mine.told & (1 << slot) != 0 {
                    relay(Event::Up(mine.moment), slot);
                }
            }
        }
        if let Some(own) = self.ragdoll.own {
            self.ragdoll.set_fall(own, None);
        }
        self.ragdoll.walk_to = None;
        (self.camera_pitch, self.camera_yaw) = mine.camera;
    }

    /// Tell the next SJK player on the server not yet told of the player's fall.
    fn ragdoll_tell(&mut self) {
        let Some(own) = self.ragdoll.own else {
            return;
        };
        let start = match self.ragdoll.fall_mut(own) {
            Some(fall) => fall.start,
            None => return,
        };
        let sjk = self.looks.sjk_slots() & !(1 << own);
        let Some(mine) = self.ragdoll.mine.as_mut() else {
            return;
        };
        // One who left is told again should they come back.
        mine.told &= sjk;
        let now = Instant::now();
        if now < mine.tell_at {
            return;
        }
        let untold = sjk & !mine.told;
        if untold == 0 {
            return;
        }
        let slot = untold.trailing_zeros() as u8;
        mine.told |= 1 << slot;
        mine.tell_at = now + TELL_GAP;
        relay(Event::Fall(start), slot);
    }

    /// Step every fall on to `time` (game server milliseconds, the presentation's), end
    /// the falls of players who are gone, and set what is shown this frame.
    pub(crate) fn step_ragdolls(&mut self, time: i64) {
        #[cfg(test)]
        if self.ragdoll.shot_frozen {
            return;
        }
        self.ragdoll.time = time;
        let Some(session) = self
            .live_session
            .as_ref()
            .filter(|session| !session.is_local())
        else {
            *shown_lock() = [None; SLOTS];
            return;
        };
        let snapshot = session.snapshot_at_or_before(time as i32);
        let own = self.ragdoll.own;
        let mut shown = [None; SLOTS];
        let mut movers = std::mem::take(&mut self.ragdoll.movers);
        let mut world = scene::Scene::new(
            &self.bsp,
            &mut self.trace_scratch,
            Some(session),
            &mut movers,
        );
        for (slot, entry) in self.ragdoll.falls.iter_mut().enumerate() {
            let Some(fall) = entry else {
                continue;
            };
            let slot = slot as u8;
            // Another's: gone, dead, hidden or respawned ends it; so does their real
            // body staying far from the ragdoll (the end did not reach here).
            if Some(slot) != own {
                let entity = snapshot.entities.iter().find(|entity| {
                    entity.number() == u16::from(slot) && entity.entity_type() == ET_PLAYER
                });
                let ended = entity.is_none_or(|entity| {
                    let flags = entity.e_flags();
                    let respawned = fall
                        .teleport
                        .replace(flags & EF_TELEPORT_BIT)
                        .is_some_and(|bit| bit != flags & EF_TELEPORT_BIT);
                    flags & (EF_DEAD | EF_NODRAW) != 0 || respawned
                });
                let far = self
                    .live_world
                    .entity(EntityId::new(u64::from(slot) + 1))
                    .map(|entity| Vec3::from_array(entity.sample(time).translation))
                    .is_some_and(|at| at.distance(fall.body.centre()) > ASTRAY_REACH);
                let astray = if far {
                    *fall.astray.get_or_insert_with(Instant::now)
                } else {
                    fall.astray = None;
                    Instant::now()
                };
                if ended || astray.elapsed() > ASTRAY {
                    *entry = None;
                    continue;
                }
            }
            let since = (time - i64::from(fall.start.moment)).max(0);
            let steps = (since / i64::from(STEP_MS)) as u32;
            let fraction = (since % i64::from(STEP_MS)) as f32 / STEP_MS as f32;
            let before = fall.body.steps();
            fall.body.run_to(steps, &mut world);
            // The knocks and scrapes of this frame's steps (not while fast-forwarding a
            // fall heard late).
            if fall.body.steps() > before && fall.body.steps() - before < 8 {
                sounds::hear(
                    &mut fall.heard,
                    &fall.body,
                    time,
                    slot,
                    &mut self.ragdoll.cues,
                );
            }
            shown[usize::from(slot)] = Some(fall.body.between(fraction));
        }
        self.ragdoll.movers = movers;
        // Where the player's own real body walks to: the ragdoll's pelvis.
        self.ragdoll.walk_to = own
            .and_then(|own| shown[usize::from(own)])
            .filter(|_| self.ragdoll.mine.is_some())
            .map(|points| points[PELVIS]);
        *shown_lock() = shown;
    }

    /// Play the toy's sounds of the frame.
    pub(crate) fn play_ragdoll_sounds(&mut self, audio: &mut Option<crate::GameAudio>) {
        let Some(audio) = audio.as_mut() else {
            self.ragdoll.cues.clear();
            return;
        };
        if self.ragdoll.sounds_registered != Some(audio.generation())
            && let Some(vfs) = self.vfs.as_deref()
        {
            audio.register_effect_paths(vfs, [FLATLINE].into_iter().chain(sounds::paths()));
            self.ragdoll.sounds_registered = Some(audio.generation());
        }
        for Cue {
            path,
            at,
            slot,
            volume,
        } in self.ragdoll.cues.drain(..)
        {
            let source = sjk_audio::SourceId(u32::from(slot));
            match at {
                None => audio.play_local(path, volume, source, crate::audio::TOY_CHANNEL),
                Some(at) => audio.play_on_channel(
                    path,
                    volume,
                    at.to_array(),
                    source,
                    if path == FLATLINE {
                        crate::audio::TOY_CHANNEL
                    } else {
                        sjk_audio::ChannelId(0)
                    },
                ),
            }
        }
    }
}

#[cfg(test)]
impl GpuState {
    /// For the world shots: the player in `slot` of the demo playing falls where they
    /// stand, with `velocity` and facing `yaw`, and is shown `steps` steps on.
    pub(crate) fn shot_ragdoll(&mut self, slot: u16, velocity: [i32; 3], yaw: u16, steps: u32) {
        let session = self.demo_session.as_ref().expect("a demo playing");
        let time = session.latest_snapshot().server_time;
        let origin = session
            .world()
            .entity(EntityId::new(u64::from(slot) + 1))
            .map(|entity| entity.sample(i64::from(time)).translation)
            .expect("the player");
        let start = Start {
            moment: time,
            origin: origin.map(|v| v.round() as i32),
            velocity,
            yaw,
        };
        let mut body = Body::new(&start);
        let mut movers = scene::Movers::default();
        body.run_to(
            steps,
            &mut scene::Scene::new(&self.bsp, &mut self.trace_scratch, None, &mut movers),
        );
        self.ragdoll.shot_frozen = true;
        shown_lock()[usize::from(slot)] = Some(*body.points());
    }
}

/// Where a fall starts, for its flatline: the player's origin.
fn start_point(start: &Start) -> Vec3 {
    Vec3::new(
        start.origin[0] as f32,
        start.origin[1] as f32,
        start.origin[2] as f32,
    )
}

/// The limp player's camera: behind the ragdoll's pelvis, turned as the player's own
/// camera is; `None` otherwise.
pub(crate) fn camera(state: &mut GpuState, time: i64) -> Option<(Vec3, Vec3)> {
    state.ragdoll.mine.as_ref()?;
    let session = state.live_session.as_ref()?;
    let game = session.game_state();
    let snapshot = session.snapshot_at_or_before(time as i32);
    if snapshot.player.movement_flags() & PMF_FOLLOW != 0 {
        return None;
    }
    let points = shown(state.ragdoll.own?)?;
    let (pitch, yaw) = (state.camera_pitch, state.camera_yaw);
    let forward = Vec3::new(
        yaw.cos() * pitch.cos(),
        yaw.sin() * pitch.cos(),
        pitch.sin(),
    );
    let focus = points[PELVIS] + Vec3::Z * CAMERA_HEIGHT;
    let wanted = focus - forward * CAMERA_RANGE + Vec3::Z * CAMERA_RISE;
    let position = crate::movement_collision::camera_trace(
        &state.bsp,
        &mut state.trace_scratch,
        Some((game, snapshot)),
        time as i32,
        focus,
        wanted,
    );
    Some((position, position + forward * 256.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn start() -> Start {
        Start {
            moment: 123_456,
            origin: [-1_024, 512, 88],
            velocity: [320, -12, 270],
            yaw: 271,
        }
    }

    #[test]
    fn events_read_back_as_written_and_fit_the_relay() {
        for event in [Event::Fall(start()), Event::Up(123_456)] {
            let text = event.text();
            assert!(text.len() <= 64, "{text}");
            assert!(
                text.bytes().all(|b| b.is_ascii_lowercase()
                    || b.is_ascii_digit()
                    || b == b'_'
                    || b == b'-'),
                "{text}"
            );
            assert_eq!(Event::parse(&text), Some(event));
        }
        assert_eq!(Event::parse("fall_1_2"), None);
        assert_eq!(Event::parse("up_x"), None);
        assert_eq!(Event::parse("in_1"), None);
    }

    #[test]
    fn a_limp_player_walks_to_the_ragdoll_without_a_button() {
        let mut ragdoll = Ragdoll::default();
        let input = UserCommand {
            forward_move: -127,
            buttons: 1,
            ..UserCommand::default()
        };
        assert!(ragdoll.drive(&input, [0; 3], [0.0; 3]).is_none());
        ragdoll.mine = Some(Mine {
            moment: 0,
            told: 0,
            tell_at: Instant::now(),
            health: 100,
            camera: (0.0, 0.0),
        });
        // Far off along +Y: runs there, facing it.
        ragdoll.walk_to = Some(Vec3::new(0.0, 300.0, 0.0));
        let drive = ragdoll.drive(&input, [0; 3], [0.0; 3]).unwrap();
        assert_eq!(drive.command.forward_move, 127);
        assert_eq!(drive.command.buttons, 0);
        assert_eq!(drive.command.angles[1], 16_384);
        // Near: walks; there: stands.
        ragdoll.walk_to = Some(Vec3::new(40.0, 0.0, 0.0));
        let drive = ragdoll.drive(&input, [0; 3], [0.0; 3]).unwrap();
        assert_eq!(drive.command.buttons, BUTTON_WALKING);
        ragdoll.walk_to = Some(Vec3::new(5.0, 0.0, 30.0));
        let drive = ragdoll.drive(&input, [0; 3], [0.0; 3]).unwrap();
        assert_eq!(drive.command.forward_move, 0);
        // `delta_angles` are taken off, as the game adds them back.
        ragdoll.walk_to = Some(Vec3::new(0.0, 300.0, 0.0));
        let drive = ragdoll.drive(&input, [0, 100, 0], [0.0; 3]).unwrap();
        assert_eq!(drive.command.angles[1], 16_284);
    }

    #[test]
    fn a_ragdoll_places_its_actor_at_the_pelvis_unturned() {
        let body = Body::new(&start());
        let mut transform = sjk_runtime::Transform::IDENTITY;
        transform.scale = [1.2; 3];
        transform.rotation = glam::Quat::from_rotation_z(1.0).to_array();
        place(&mut transform, body.points());
        assert_eq!(transform.translation, body.centre().to_array());
        assert_eq!(transform.scale, [1.0; 3]);
        assert_eq!(transform.rotation, glam::Quat::IDENTITY.to_array());
    }
}
