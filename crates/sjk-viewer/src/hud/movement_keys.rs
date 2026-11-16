//! JoF EternalJK's movement keys (`cg_movementKeys`, jaPRO's `CG_MovementKeys` in JoF
//! EJK's `cg_draw.c`, called from `CG_DrawHUD`): six keys on screen, jump, forward and
//! crouch over left, back and right, each lit while it is held.
//!
//! For the player's own view the keys are the command being made this frame
//! (`trap->GetUserCmd` of the current command). Following someone, or in a demo, the
//! keys cannot be known, so they are guessed from the player state as JoF does: the
//! movement direction while moving faster than 9 units a second, jump while rising
//! more than 1 unit off the ground (or while the jump is held), crouch while ducked
//! or rolling.
//!
//! `cg_movementKeysX` and `cg_movementKeysY` place the keys' top left corner in the
//! 640 by 480 screen, `cg_movementKeysSize` scales their 16-unit side. As with
//! EternalJK's `widthRatioCoef`, the keys stay square on wide screens and keep their
//! distance from the screen's right edge. With one of JoF's numbered HUDs
//! (`cg_hudFiles` 1 to 3) they move next to it as JoF moves them.
//!
//! JoF draws jaPRO's pictures (`gfx/hud/keys`); SJK uses them where they are installed
//! (EternalJK's `japro-assets.pk3`, [`crate::asset_search_paths`]) and otherwise draws
//! keys of its own in the same places.

use super::icons::assets::decode;
use crate::console::ViewerConsole;
use sjk_protocol::{GameState, PlayerState};
use sjk_shader::ShaderCatalog;
use sjk_shell::{CvarDefinition, CvarFlags, CvarRegistry};
use sjk_ui::{Color, DrawCommand, DrawList, POLYGON_POINTS, Rect, TextureId};
use sjk_vfs::VirtualFileSystem;

/// Shows the keys: 0 hidden, anything else shown, as JoF reads `cg_movementKeys.integer`.
pub(crate) const CVAR: &str = "cg_movementKeys";
const X_CVAR: &str = "cg_movementKeysX";
const Y_CVAR: &str = "cg_movementKeysY";
const SIZE_CVAR: &str = "cg_movementKeysSize";
/// JoF EJK's defaults (`cg_xcvar.h`).
const DEFAULT_X: f64 = 465.0;
const DEFAULT_Y: f64 = 432.0;
const DEFAULT_SIZE: f64 = 1.0;
/// A key's side in the 640 by 480 screen at size 1.
const SIDE: f32 = 16.0;

/// The pictures JoF draws, two per key in [`Key`] order: released (`_off`), then
/// pressed (`_on`). Names without an extension, as `R_RegisterShaderNoMip` takes them.
pub(crate) const PICTURES: [&str; 12] = [
    "gfx/hud/keys/jump_off",
    "gfx/hud/keys/jump_on",
    "gfx/hud/keys/forward_off",
    "gfx/hud/keys/forward_on",
    "gfx/hud/keys/crouch_off",
    "gfx/hud/keys/crouch_on",
    "gfx/hud/keys/left_off",
    "gfx/hud/keys/left_on",
    "gfx/hud/keys/back_off",
    "gfx/hud/keys/back_on",
    "gfx/hud/keys/right_off",
    "gfx/hud/keys/right_on",
];

const _: () = assert!(PICTURES.len() == crate::ui_renderer::MOVEMENT_KEY_ICON_CELLS as usize);

/// Draw commands the keys push in a frame at most: SJK's own keys, a cap, its rim and
/// up to three shapes for the arrow, for each of the six.
pub(crate) const COMMANDS: usize = 6 * (2 + GLYPH_PARTS);

/// JoF moves the keys next to its numbered HUDs (`cgs.newHud`, `cg_hudFiles`):
/// (HUD number, units right, units down). Other numbers stay put.
const JOF_HUD_OFFSETS: &[(i64, [f32; 2])] =
    &[(1, [51.0, 0.0]), (2, [26.0, -3.0]), (3, [-18.0, 0.0])];
/// With a numbered HUD and no score line (`cg_drawScore 0`, or Power Duel), JoF moves
/// the keys down into its room.
const JOF_NO_SCORE_DOWN: f32 = 12.0;
/// With a numbered HUD and team scores beside it (`cg_drawScore` 2 or more in a team
/// game other than Siege), JoF moves the keys up and left of them: units left for
/// HUD 1, units left for the others, units up.
const JOF_TEAM_SCORE_SHIFT: (f32, f32, f32) = (23.0, 12.0, 14.0);
/// `g_gametype` values (`bg_public.h`).
const GT_POWERDUEL: i32 = 4;
const GT_TEAM: i32 = 6;
const GT_SIEGE: i32 = 7;

/// `pm_flags` (`bg_public.h`).
const PMF_DUCKED: u16 = 1;
const PMF_JUMP_HELD: u16 = 2;
const PMF_FOLLOW: u16 = 4096;
/// `playerState_t::fd.forceGripCripple`, the wire field.
const PS_FORCE_GRIP_CRIPPLE: usize = 111;
/// `TEAM_SPECTATOR`.
const TEAM_SPECTATOR: u8 = 3;
/// `BOTH_ROLL_F`, `_B`, `_L` and `_R` (`anims.h`), which `CG_InRollAnim` checks.
const ROLLS: std::ops::RangeInclusive<u16> = 1_167..=1_170;
/// Below this horizontal speed a guessed player is standing still (`xyspeed < 9`).
const STILL_SPEED: f32 = 9.0;

/// The six keys, in the order of [`PICTURES`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Key {
    Jump,
    Forward,
    Crouch,
    Left,
    Back,
    Right,
}

impl Key {
    const ALL: [Self; 6] = [
        Self::Jump,
        Self::Forward,
        Self::Crouch,
        Self::Left,
        Self::Back,
        Self::Right,
    ];

    /// Column and row in JoF's three by two block.
    const fn cell(self) -> (f32, f32) {
        match self {
            Self::Jump => (0.0, 0.0),
            Self::Forward => (1.0, 0.0),
            Self::Crouch => (2.0, 0.0),
            Self::Left => (0.0, 1.0),
            Self::Back => (1.0, 1.0),
            Self::Right => (2.0, 1.0),
        }
    }

    fn held(self, keys: Pressed) -> bool {
        match self {
            Self::Jump => keys.up > 0,
            Self::Forward => keys.forward > 0,
            Self::Crouch => keys.up < 0,
            Self::Left => keys.right < 0,
            Self::Back => keys.forward < 0,
            Self::Right => keys.right > 0,
        }
    }
}

/// Which way each movement axis is held: -1, 0 or 1, as a `usercmd_t`'s sign.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Pressed {
    forward: i8,
    right: i8,
    up: i8,
}

impl Pressed {
    /// The keys of a user command's `forwardmove`, `rightmove` and `upmove`.
    pub(crate) fn from_command([forward, right, up]: [i8; 3]) -> Self {
        Self {
            forward: forward.signum(),
            right: right.signum(),
            up: up.signum(),
        }
    }

    /// The keys of a movement direction (`ps.movementDir`, 0 forward and on round to
    /// 7, forward right), as JoF's switch maps them; `None` (standing still) holds none.
    fn from_direction(direction: Option<i8>) -> Self {
        let (forward, right) = match direction {
            Some(0) => (1, 0),
            Some(1) => (1, -1),
            Some(2) => (0, -1),
            Some(3) => (-1, -1),
            Some(4) => (-1, 0),
            Some(5) => (-1, 1),
            Some(6) => (0, 1),
            Some(7) => (1, 1),
            _ => (0, 0),
        };
        Self {
            forward,
            right,
            up: 0,
        }
    }
}

/// Whether `player` is the player being followed rather than the viewer's own.
pub(crate) fn following(player: &PlayerState) -> bool {
    player.movement_flags() & PMF_FOLLOW != 0
}

/// The movement keys' settings, read once a frame.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Settings {
    on: bool,
    /// Top left corner in the 640 by 480 screen, JoF's numbered HUD shift included.
    position: [f32; 2],
    size: f32,
}

impl Settings {
    fn read(console: Option<&ViewerConsole>, gametype: i32) -> Self {
        let on = console.and_then(|c| c.integer_cvar(CVAR)).unwrap_or(0) != 0;
        // JoF reads the place as `.integer`, cut to whole units.
        let place = |name, default| crate::cgame_options::scalar(console, name, default).trunc();
        let mut position = [place(X_CVAR, DEFAULT_X), place(Y_CVAR, DEFAULT_Y)];
        if let Some(files) = numbered_hud(console) {
            let score = console
                .and_then(|c| c.integer_cvar("cg_drawScore"))
                .unwrap_or(2);
            let shift = jof_hud_shift(files, score, gametype);
            position = [position[0] + shift[0], position[1] + shift[1]];
        }
        Self {
            on,
            position,
            size: crate::cgame_options::scalar(console, SIZE_CVAR, DEFAULT_SIZE),
        }
    }
}

/// JoF's numbered HUD in use (`cgs.newHud`): only the game-data HUD style with a
/// number in `cg_hudFiles`; a menu file path such as the retail `ui/jahud.txt` is not
/// one.
fn numbered_hud(console: Option<&ViewerConsole>) -> Option<i64> {
    if crate::menu_hud::HudStyle::read(console) != crate::menu_hud::HudStyle::Game {
        return None;
    }
    console
        .and_then(|c| c.text_value(crate::menu_hud::FILES_CVAR))
        .and_then(|value| value.trim().parse::<i64>().ok())
}

/// How far JoF moves the keys for numbered HUD `files` (units right, units down), with
/// `cg_drawScore` `score` in game type `gametype`. jaPRO's race mode, which moves them
/// as no score does, is not covered.
fn jof_hud_shift(files: i64, score: i64, gametype: i32) -> [f32; 2] {
    let mut shift = JOF_HUD_OFFSETS
        .iter()
        .find_map(|&(known, offset)| (files == known).then_some(offset))
        .unwrap_or([0.0, 0.0]);
    if score == 0 || gametype == GT_POWERDUEL {
        shift[1] += JOF_NO_SCORE_DOWN;
    } else if score > 1 && gametype >= GT_TEAM && gametype != GT_SIEGE {
        let (first, others, up) = JOF_TEAM_SCORE_SHIFT;
        shift[0] -= if files == 1 { first } else { others };
        shift[1] -= up;
    }
    shift
}

/// The keys' pictures in the UI atlas, where they loaded; all twelve or none, so the
/// block never mixes pictures with SJK's own keys.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Pictures(Option<[TextureId; 12]>);

/// Upload the keys' pictures once per installed world.
pub(crate) fn load(
    vfs: &VirtualFileSystem,
    shaders: &ShaderCatalog,
    mut upload: impl FnMut(TextureId, &[u8]),
) -> Pictures {
    let mut decoded = Vec::with_capacity(PICTURES.len());
    for name in PICTURES {
        let Some(pixels) = decode(vfs, shaders, name) else {
            return Pictures(None);
        };
        decoded.push(pixels);
    }
    let first = crate::ui_renderer::MOVEMENT_KEY_ICON_FIRST;
    let mut textures = [TextureId(first); 12];
    for (index, pixels) in decoded.iter().enumerate() {
        let texture = TextureId(first + index as u32);
        upload(texture, pixels.as_raw());
        textures[index] = texture;
    }
    Pictures(Some(textures))
}

/// The movement keys' frame state.
#[derive(Debug, Default)]
pub(crate) struct State {
    settings: Option<Settings>,
    /// The player can have a HUD: alive, playing and on foot.
    playing: bool,
    keys: Pressed,
    /// Upward speed at the last guess, for "still rising" (JoF's `lastZSpeed`).
    last_z_speed: f32,
    pictures: Pictures,
}

impl State {
    /// Use these pictures from now on.
    pub(crate) fn set_pictures(&mut self, pictures: Pictures) {
        self.pictures = pictures;
    }

    /// Read the settings and this frame's keys. `own` is the command being made this
    /// frame when `player` is the viewer's own (not followed, not a demo); otherwise the
    /// keys are guessed, and `ground_distance` gives how far `player`'s origin stands
    /// above the floor (JoF's `PM_GroundDistance2`, before taking the view height off).
    pub(crate) fn update(
        &mut self,
        player: &PlayerState,
        game: Option<&GameState>,
        own: Option<[i8; 3]>,
        console: Option<&ViewerConsole>,
        ground_distance: impl FnOnce([f32; 3]) -> f32,
    ) {
        let gametype = super::identification::info_number(
            game.and_then(|game| game.config_string(0)),
            "g_gametype",
        );
        let settings = Settings::read(console, gametype);
        let status = console
            .and_then(|c| c.bool_cvar("cg_drawStatus"))
            .unwrap_or(true);
        self.settings = (settings.on && status).then_some(settings);
        // `CG_Draw2D` draws the HUD for a living player not on the spectator team, and
        // `CG_DrawStats` leaves it to a vehicle's own HUD while riding.
        self.playing = player.team() != TEAM_SPECTATOR
            && player.health() > 0
            && player.vehicle_entity_num() == 0;
        if self.settings.is_none() || !self.playing {
            return;
        }
        self.keys = match own {
            Some(command) => Pressed::from_command(command),
            None => self.guess(player, ground_distance),
        };
    }

    /// JoF's guess at the keys of a player whose commands are not ours.
    fn guess(
        &mut self,
        player: &PlayerState,
        ground_distance: impl FnOnce([f32; 3]) -> f32,
    ) -> Pressed {
        let [x, y, z] = player.velocity();
        let flags = player.movement_flags();
        let gripped = player.raw_field(PS_FORCE_GRIP_CRIPPLE).unwrap_or(0) != 0;
        let rising = z > 8.0 && z > self.last_z_speed && !gripped;
        // The trace is only needed while rising.
        let off_ground =
            rising && ground_distance(player.origin()) - player.view_height() as f32 > 1.0;
        let up = if off_ground || flags & PMF_JUMP_HELD != 0 {
            1
        } else if flags & PMF_DUCKED != 0 || ROLLS.contains(&player.leg_animation()) {
            -1
        } else {
            0
        };
        self.last_z_speed = z;
        let moving = x.hypot(y) >= STILL_SPEED;
        Pressed {
            up,
            ..Pressed::from_direction(moving.then(|| player.movement_direction()))
        }
    }

    /// Draw the keys in a `viewport` of physical pixels.
    pub(crate) fn emit(&self, draw: &mut DrawList, viewport: [f32; 2]) {
        let Some(settings) = self.settings.filter(|_| self.playing) else {
            return;
        };
        let [width, height] = viewport;
        if width <= 0.0 || height <= 0.0 {
            return;
        }
        // EternalJK's `widthRatioCoef`, never above 1.
        let coef = ((640.0 * height) / (480.0 * width)).min(1.0);
        let unit = [width / 640.0, height / 480.0];
        let x = 640.0 - (640.0 - settings.position[0]) * coef;
        let side = [SIDE * settings.size * coef, SIDE * settings.size];
        for key in Key::ALL {
            let (column, row) = key.cell();
            let rect = Rect::new(
                (x + side[0] * column) * unit[0],
                (settings.position[1] + side[1] * row) * unit[1],
                side[0] * unit[0],
                side[1] * unit[1],
            );
            let held = key.held(self.keys);
            match self.pictures.0 {
                Some(textures) => {
                    let _ = draw.push(DrawCommand::TexturedQuad {
                        rect,
                        texture: textures[key as usize * 2 + usize::from(held)],
                        color: Color::new(1.0, 1.0, 1.0, 1.0),
                    });
                }
                None => own_key(draw, rect, key, held),
            }
        }
    }
}

/// The most shapes an arrow is made of.
const GLYPH_PARTS: usize = 3;
/// A shape of an arrow in a key's unit square, pointing up; the rest of the points
/// are unused.
type Part = [[f32; 2]; 4];
/// An up arrow: its head and its stem.
const ARROW: [Part; 2] = [
    [[0.5, 0.2], [0.78, 0.5], [0.22, 0.5], [0.22, 0.5]],
    [[0.41, 0.5], [0.59, 0.5], [0.59, 0.8], [0.41, 0.8]],
];
/// Jump: a shorter up arrow over a floor line.
const JUMP: [Part; 3] = [
    [[0.5, 0.18], [0.76, 0.46], [0.24, 0.46], [0.24, 0.46]],
    [[0.42, 0.46], [0.58, 0.46], [0.58, 0.66], [0.42, 0.66]],
    [[0.24, 0.73], [0.76, 0.73], [0.76, 0.82], [0.24, 0.82]],
];
/// Crouch: a ceiling line over a shorter down arrow.
const CROUCH: [Part; 3] = [
    [[0.24, 0.18], [0.76, 0.18], [0.76, 0.27], [0.24, 0.27]],
    [[0.42, 0.34], [0.58, 0.34], [0.58, 0.54], [0.42, 0.54]],
    [[0.24, 0.54], [0.76, 0.54], [0.5, 0.82], [0.5, 0.82]],
];

/// SJK's own key where jaPRO's pictures are not installed: a rounded cap with an
/// arrow, dim while released and bright while held, as the pictures are.
fn own_key(draw: &mut DrawList, rect: Rect, key: Key, held: bool) {
    let inset = rect.width.min(rect.height) * 0.06;
    let cap = Rect::new(
        rect.x + inset,
        rect.y + inset,
        rect.width - 2.0 * inset,
        rect.height - 2.0 * inset,
    );
    let radius = cap.width.min(cap.height) * 0.2;
    let (fill, rim, glyph) = if held {
        (
            Color::new(0.93, 0.93, 0.93, 0.95),
            Color::new(0.6, 0.6, 0.6, 1.0),
            Color::new(0.2, 0.2, 0.2, 1.0),
        )
    } else {
        (
            Color::new(0.2, 0.2, 0.2, 0.7),
            Color::new(0.38, 0.38, 0.38, 0.85),
            Color::new(0.42, 0.42, 0.42, 0.9),
        )
    };
    let _ = draw.push(DrawCommand::RoundedRect {
        rect: cap,
        radius,
        color: fill,
    });
    let _ = draw.push(DrawCommand::Border {
        rect: cap,
        radius,
        width: (cap.width * 0.06).max(1.0),
        color: rim,
    });
    // Up, right, down or left: the arrow's quarter turns clockwise.
    let (parts, turns): (&[Part], u8) = match key {
        Key::Jump => (&JUMP, 0),
        Key::Crouch => (&CROUCH, 0),
        Key::Forward => (&ARROW, 0),
        Key::Right => (&ARROW, 1),
        Key::Back => (&ARROW, 2),
        Key::Left => (&ARROW, 3),
    };
    for part in parts {
        let mut points = [[0.0; 2]; POLYGON_POINTS];
        for (point, &[u, v]) in points.iter_mut().zip(part) {
            let [u, v] = match turns {
                1 => [1.0 - v, u],
                2 => [1.0 - u, 1.0 - v],
                3 => [v, 1.0 - u],
                _ => [u, v],
            };
            *point = [cap.x + u * cap.width, cap.y + v * cap.height];
        }
        // A triangle repeats its last corner to fill the four.
        let count = if part[2] == part[3] { 3 } else { 4 };
        let _ = draw.push(DrawCommand::Polygon {
            points,
            count,
            color: glyph,
            texture: None,
        });
    }
}

/// Register the keys' options, with JoF EJK's names and defaults.
pub(crate) fn register(cvars: &mut CvarRegistry) -> Result<(), sjk_shell::CvarError> {
    cvars.register(CvarDefinition::new(
        CVAR,
        0_i64,
        CvarFlags::ARCHIVE,
        "Show the movement keys on screen (after JoF EJK)",
    ))?;
    for (name, value, help) in [
        (
            X_CVAR,
            DEFAULT_X,
            "Movement keys' left edge, 640-wide screen",
        ),
        (
            Y_CVAR,
            DEFAULT_Y,
            "Movement keys' top edge, 480-high screen",
        ),
        (SIZE_CVAR, DEFAULT_SIZE, "Movement keys' scale"),
    ] {
        cvars.register(CvarDefinition::new(name, value, CvarFlags::ARCHIVE, help))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn console() -> (tempfile::TempDir, ViewerConsole) {
        let directory = tempfile::tempdir().unwrap();
        let console = ViewerConsole::new(directory.path().join("config.cfg")).unwrap();
        (directory, console)
    }

    fn alive() -> PlayerState {
        let mut player = PlayerState::default();
        player.stats[0] = 100;
        player
    }

    /// A state with the keys on and these keys held, as a frame leaves it.
    fn showing(keys: Pressed, pictures: Pictures) -> State {
        State {
            settings: Some(Settings {
                on: true,
                position: [DEFAULT_X as f32, DEFAULT_Y as f32],
                size: 1.0,
            }),
            playing: true,
            keys,
            last_z_speed: 0.0,
            pictures,
        }
    }

    fn rects(draw: &DrawList) -> Vec<Rect> {
        draw.commands()
            .iter()
            .filter_map(|command| match *command {
                DrawCommand::TexturedQuad { rect, .. } => Some(rect),
                _ => None,
            })
            .collect()
    }

    fn pictures() -> Pictures {
        let first = crate::ui_renderer::MOVEMENT_KEY_ICON_FIRST;
        Pictures(Some(std::array::from_fn(|i| TextureId(first + i as u32))))
    }

    #[test]
    fn the_keys_stand_where_jof_puts_them_on_a_4_3_screen() {
        let state = showing(Pressed::default(), pictures());
        let mut draw = DrawList::new(COMMANDS);
        state.emit(&mut draw, [640.0, 480.0]);
        let rects = rects(&draw);
        let at = |rect: Rect| (rect.x, rect.y, rect.width, rect.height);
        // Jump, forward, crouch over left, back, right, 16 units each from (465, 432).
        assert_eq!(at(rects[0]), (465.0, 432.0, 16.0, 16.0));
        assert_eq!(at(rects[1]), (481.0, 432.0, 16.0, 16.0));
        assert_eq!(at(rects[2]), (497.0, 432.0, 16.0, 16.0));
        assert_eq!(at(rects[3]), (465.0, 448.0, 16.0, 16.0));
        assert_eq!(at(rects[4]), (481.0, 448.0, 16.0, 16.0));
        assert_eq!(at(rects[5]), (497.0, 448.0, 16.0, 16.0));
    }

    #[test]
    fn on_a_wide_screen_the_keys_stay_square_and_keep_to_the_right_edge() {
        let state = showing(Pressed::default(), pictures());
        let mut draw = DrawList::new(COMMANDS);
        state.emit(&mut draw, [1920.0, 1080.0]);
        let jump = rects(&draw)[0];
        // widthRatioCoef 0.75: 175 units from the right edge become 131.25, 3 pixels each.
        assert!((jump.x - (640.0 - 175.0 * 0.75) * 3.0).abs() < 1e-3);
        assert!((jump.y - 432.0 * 2.25).abs() < 1e-3);
        assert!((jump.width - 36.0).abs() < 1e-3);
        assert!((jump.height - 36.0).abs() < 1e-3);
    }

    #[test]
    fn a_held_key_shows_its_pressed_picture() {
        let keys = Pressed::from_command([127, -127, 127]);
        let state = showing(keys, pictures());
        let mut draw = DrawList::new(COMMANDS);
        state.emit(&mut draw, [640.0, 480.0]);
        let first = crate::ui_renderer::MOVEMENT_KEY_ICON_FIRST;
        let textures: Vec<u32> = draw
            .commands()
            .iter()
            .filter_map(|command| match *command {
                DrawCommand::TexturedQuad { texture, .. } => Some(texture.0 - first),
                _ => None,
            })
            .collect();
        // Jump on, forward on, crouch off, left on, back off, right off.
        assert_eq!(textures, [1, 3, 4, 7, 8, 10]);
    }

    #[test]
    fn without_japro_s_pictures_sjk_draws_its_own_keys() {
        let mut draw = DrawList::new(COMMANDS);
        showing(Pressed::default(), Pictures(None)).emit(&mut draw, [640.0, 480.0]);
        let caps = draw
            .commands()
            .iter()
            .filter(|command| matches!(command, DrawCommand::RoundedRect { .. }))
            .count();
        assert_eq!(caps, 6);
        assert!(draw.len() <= COMMANDS);
        assert!(rects(&draw).is_empty());
        // Held keys are brighter than released ones.
        let mut held = DrawList::new(COMMANDS);
        showing(Pressed::from_command([1, 0, 0]), Pictures(None)).emit(&mut held, [640.0, 480.0]);
        let fills = |draw: &DrawList| -> Vec<Color> {
            draw.commands()
                .iter()
                .filter_map(|command| match *command {
                    DrawCommand::RoundedRect { color, .. } => Some(color),
                    _ => None,
                })
                .collect()
        };
        assert!(fills(&held)[1].r > fills(&draw)[1].r);
        assert_eq!(fills(&held)[0], fills(&draw)[0]);
    }

    #[test]
    fn analog_moves_light_the_key_of_their_direction() {
        assert_eq!(
            Pressed::from_command([40, 0, -90]),
            Pressed {
                forward: 1,
                right: 0,
                up: -1
            }
        );
    }

    #[test]
    fn a_followed_player_s_keys_are_guessed_from_how_they_move() {
        let mut state = State::default();
        let mut player = alive();
        player.set_velocity([300.0, 0.0, 0.0]);
        player.set_raw_field(30, 1); // movementDir: forward left
        let keys = state.guess(&player, |_| 0.0);
        assert_eq!((keys.forward, keys.right, keys.up), (1, -1, 0));
        // Barely moving holds no direction.
        player.set_velocity([5.0, 5.0, 0.0]);
        assert_eq!(state.guess(&player, |_| 0.0), Pressed::default());
        // Ducked or rolling is crouch.
        player.set_movement_flags(PMF_DUCKED);
        assert_eq!(state.guess(&player, |_| 0.0).up, -1);
        player.set_movement_flags(0);
        player.set_raw_field(13, 1_169); // legsAnim: BOTH_ROLL_L
        assert_eq!(state.guess(&player, |_| 0.0).up, -1);
        // The jump held is jump.
        player.set_raw_field(13, 0);
        player.set_movement_flags(PMF_JUMP_HELD);
        assert_eq!(state.guess(&player, |_| 0.0).up, 1);
    }

    #[test]
    fn a_guessed_jump_needs_rising_off_the_ground() {
        let mut state = State::default();
        let mut player = alive();
        player.set_view_height(36);
        player.set_velocity([0.0, 0.0, 200.0]);
        // Still at standing height (24 units up, the view height 36 over it): no jump.
        assert_eq!(state.guess(&player, |_| 24.0).up, 0);
        // Rising faster than last frame and well off the ground: jump.
        state.last_z_speed = 100.0;
        assert_eq!(state.guess(&player, |_| 80.0).up, 1);
        // Slowing on the way up is not a jump any more.
        player.set_velocity([0.0, 0.0, 150.0]);
        assert_eq!(state.guess(&player, |_| 80.0).up, 0);
        // Nor is being pulled up by a grip.
        state.last_z_speed = 0.0;
        player.set_raw_field(PS_FORCE_GRIP_CRIPPLE, 1);
        assert_eq!(state.guess(&player, |_| 80.0).up, 0);
    }

    #[test]
    fn the_own_command_wins_over_a_guess() {
        let (_directory, mut console) = console();
        assert!(console.set_cvar(CVAR, "1"));
        let mut state = State::default();
        let mut player = alive();
        player.set_velocity([300.0, 0.0, 0.0]);
        state.update(&player, None, Some([0, 127, 0]), Some(&console), |_| 0.0);
        assert_eq!((state.keys.forward, state.keys.right), (0, 1));
    }

    #[test]
    fn nothing_shows_when_off_dead_spectating_or_riding() {
        let (_directory, mut console) = console();
        let mut state = State::default();
        let mut player = alive();
        let drawn = |state: &mut State, player: &PlayerState, console: &ViewerConsole| {
            state.update(player, None, Some([0; 3]), Some(console), |_| 0.0);
            let mut draw = DrawList::new(COMMANDS);
            state.emit(&mut draw, [640.0, 480.0]);
            !draw.is_empty()
        };
        assert!(!drawn(&mut state, &player, &console), "off by default");
        assert!(console.set_cvar(CVAR, "1"));
        assert!(drawn(&mut state, &player, &console));
        assert!(console.set_cvar("cg_drawStatus", "0"));
        assert!(!drawn(&mut state, &player, &console), "cg_drawStatus 0");
        assert!(console.set_cvar("cg_drawStatus", "1"));
        player.stats[0] = 0;
        assert!(!drawn(&mut state, &player, &console), "dead");
        player.stats[0] = 100;
        player.persistent[3] = u32::from(TEAM_SPECTATOR);
        assert!(!drawn(&mut state, &player, &console), "spectator team");
        player.persistent[3] = 0;
        player.set_raw_field(84, 40); // m_iVehicleNum
        assert!(!drawn(&mut state, &player, &console), "riding");
    }

    #[test]
    fn the_place_and_size_come_from_their_settings() {
        let (_directory, mut console) = console();
        assert!(console.set_cvar(X_CVAR, "100.9"));
        assert!(console.set_cvar(Y_CVAR, "200"));
        assert!(console.set_cvar(SIZE_CVAR, "2"));
        let settings = Settings::read(Some(&console), 0);
        // `.integer` cuts the place to whole units; the size keeps its fraction.
        assert_eq!(settings.position, [100.0, 200.0]);
        assert_eq!(settings.size, 2.0);
    }

    #[test]
    fn jof_s_numbered_huds_move_the_keys_beside_them() {
        // HUD 1 with the personal score in free for all: 51 units right.
        assert_eq!(jof_hud_shift(1, 1, 0), [51.0, 0.0]);
        assert_eq!(jof_hud_shift(2, 1, 0), [26.0, -3.0]);
        assert_eq!(jof_hud_shift(3, 1, 0), [-18.0, 0.0]);
        // No score line, or Power Duel: 12 units down.
        assert_eq!(jof_hud_shift(1, 0, 0), [51.0, 12.0]);
        assert_eq!(jof_hud_shift(0, 2, GT_POWERDUEL), [0.0, 12.0]);
        // Team scores beside the HUD: up 14 and left 23 for HUD 1, 12 for the others.
        assert_eq!(jof_hud_shift(1, 2, GT_TEAM), [28.0, -14.0]);
        assert_eq!(jof_hud_shift(2, 2, 8), [14.0, -17.0]);
        // Siege keeps its place.
        assert_eq!(jof_hud_shift(2, 2, GT_SIEGE), [26.0, -3.0]);
    }

    #[test]
    fn only_a_numbered_game_hud_moves_the_keys() {
        let (_directory, mut console) = console();
        assert!(console.set_cvar("cg_hudStyle", "game"));
        assert!(console.set_cvar("cg_hudFiles", "1"));
        assert_eq!(Settings::read(Some(&console), 0).position, [516.0, 432.0]);
        assert!(console.set_cvar("cg_hudFiles", "ui/jahud.txt"));
        assert_eq!(Settings::read(Some(&console), 0).position, [465.0, 432.0]);
        assert!(console.set_cvar("cg_hudFiles", "1"));
        assert!(console.set_cvar("cg_hudStyle", "classic"));
        assert_eq!(Settings::read(Some(&console), 0).position, [465.0, 432.0]);
    }
}
