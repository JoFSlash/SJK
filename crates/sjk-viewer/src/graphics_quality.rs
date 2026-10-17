//! Graphics presets: FPS boost, Vanilla style, SJK (recommended) and
//! Experimental, each one choice that sets the renderer's costly settings
//! together (Sol's three, 16/10/2026, in place of the Renderer row and the SJK
//! renderer's four levels; FPS boost added 17/10/2026 for integrated GPUs).
//!
//! FPS boost is Vanilla style with the remaining extras a weak, integrated GPU
//! pays for also off: impact marks, player shadows, weather, Force Speed
//! afterimages, footprints and dust, and saber steam in rain
//! ([`FPS_BOOST_OFF`]). The other presets leave those to the player.
//! Vanilla style is the original game's look at the least cost, every SJK extra
//! off (the EJK level until 12/10/2026, then Vanilla). SJK (recommended) is
//! Sol's own setup: SJK's image (HDR, bloom, glow, FXAA, ambient occlusion, lit
//! models, emission maps) over the maps' baked lighting, without the sun,
//! shadows, reflections, material maps or weather extras still in alpha. A new
//! profile starts on it ([`apply_new_profile`]). Experimental turns everything
//! the SJK renderer draws on; it is the renderer cvars' own defaults, the look
//! they were tuned for.
//! The preset is not stored: it is whichever preset the settings match, and
//! Custom once one of them is changed on its own. Only settings with a row in
//! Settings are touched, so every change a preset makes can be seen and undone
//! there; the FPS cap, vertical sync, resolution, supersampling and anything
//! that changes gameplay or a map's content (its fog) are left alone.

use crate::console::ViewerConsole;

/// Console command that names or sets the preset (its name is from when the
/// presets were quality levels).
pub(crate) const COMMAND: &str = "graphicsquality";
/// Help text for completion and `cmdlist`.
pub(crate) const HELP: &str =
    "Show or set the graphics preset: fpsboost, vanilla, sjk (recommended) or experimental";
/// The Graphics preset row's stand-in for a cvar name: the command that sets it.
pub(crate) const ROW_NAME: &str = COMMAND;
/// Console command that names the renderer or switches Vanilla style on and
/// off, keeping the player's settings ([`toggle_vanilla`]).
pub(crate) const RENDERER_COMMAND: &str = "renderer";
/// Help text of [`RENDERER_COMMAND`] for completion and `cmdlist`.
pub(crate) const RENDERER_HELP: &str = "Show or set the renderer: vanilla (the original look, \
     your settings kept) or sjk (SJK's renderer, your settings back)";
/// What turning Vanilla on put aside, `cvar=value` pairs joined by `;`, so
/// turning the SJK renderer on again restores the player's own settings. Its
/// name is from when Vanilla was Ultra low.
pub(crate) const RESTORE_CVAR: &str = "r_ultraLowRestore";
/// The renderer command's two values.
pub(crate) const VANILLA: &str = "vanilla";
pub(crate) const SJK: &str = "sjk";

/// A graphics preset, cheapest first.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Level {
    /// The most FPS, for integrated GPUs: Vanilla style with impact marks,
    /// player shadows, weather and the small cosmetic effects off too.
    FpsBoost,
    /// The original game's look at the least cost (EJK, Ultra low and Vanilla,
    /// before): every SJK extra off, maps show their baked lightmaps.
    Vanilla,
    /// SJK's image over the maps' own lighting: Sol's setup, the default.
    Sjk,
    /// Everything the SJK renderer draws: sun and sky, shadows, reflections,
    /// material maps, light shafts and the richest weather; still in alpha.
    Experimental,
}

/// Each setting a preset sets, with its value at FPS boost, Vanilla style, SJK
/// (recommended) and Experimental. Experimental is each setting's default
/// (pinned by a test); SJK (recommended) is Sol's config of 16/10/2026.
const VALUES: &[(&str, [&str; 4])] = &[
    // Image.
    ("r_sceneHdr", ["0", "0", "1", "1"]),
    ("r_sceneBloom", ["0", "0", "1", "1"]),
    ("r_DynamicGlow", ["0", "0", "1", "1"]),
    ("r_fxaa", ["0", "0", "1", "1"]),
    ("r_softParticles", ["0", "0", "0", "1"]),
    ("r_ssao", ["0", "0", "1", "1"]),
    // Dust shows only in light shafts.
    (crate::dust_motes::CVAR, ["0", "0", "0", "1"]),
    ("r_modelPixelLight", ["0", "0", "1", "1"]),
    ("r_cubeMapping", ["0", "0", "0", "1"]),
    ("r_floorReflections", ["0", "0", "0", "1"]),
    ("r_normalMapping", ["0", "0", "0", "1"]),
    ("r_specularMapping", ["0", "0", "0", "1"]),
    ("r_parallaxMapping", ["0", "0", "0", "1"]),
    ("r_emissiveMaps", ["0", "0", "1", "1"]),
    ("r_emissiveGlow", ["0", "0", "1", "1"]),
    // Lighting.
    ("r_dayNight", ["0", "0", "0", "1"]),
    ("r_emissiveLights", ["0", "0", "0", "1"]),
    ("r_volumetrics", ["0", "0", "0", "3"]),
    // Shadows.
    ("r_worldSunShadows", ["0", "0", "0", "1"]),
    ("r_actorSunShadows", ["0", "0", "0", "1"]),
    ("r_sunShadowResolution", ["1024", "1024", "1024", "2048"]),
    ("r_sunShadowTaps", ["8", "8", "8", "16"]),
    // Weather.
    (crate::weather::DENSITY_CVAR, ["1", "1", "1", "2"]),
    (crate::weather::QUALITY_CVAR, ["0", "0", "0", "2"]),
    (crate::weather::CLOUDS_CVAR, ["0", "0", "0", "1"]),
];

/// The small effects FPS boost turns off (to 0) besides its [`VALUES`]. The other
/// presets leave them as the player set them, so a player's own choice (no impact
/// marks under SJK (recommended), say) still matches its preset; leaving FPS boost
/// for another preset turns them back on (their defaults).
const FPS_BOOST_OFF: &[&str] = &[
    "cg_marks",
    "cg_shadows",
    "cg_speedTrail",
    crate::footprints::PRINTS_CVAR,
    crate::effect_aux::saber_rain_steam::CVAR,
    crate::weather::CVAR,
];

impl Level {
    pub(crate) const ALL: [Self; 4] =
        [Self::FpsBoost, Self::Vanilla, Self::Sjk, Self::Experimental];
    /// A new profile's preset, which the row's Backspace returns to.
    pub(crate) const DEFAULT: Self = Self::Sjk;

    /// The preset's name in Settings.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::FpsBoost => "FPS boost",
            Self::Vanilla => "Vanilla style",
            Self::Sjk => "SJK (recommended)",
            Self::Experimental => "Experimental",
        }
    }

    fn index(self) -> usize {
        self as usize
    }

    /// The preset's name in the command.
    fn command_name(self) -> &'static str {
        match self {
            Self::FpsBoost => "fpsboost",
            Self::Vanilla => VANILLA,
            Self::Sjk => SJK,
            Self::Experimental => "experimental",
        }
    }

    /// The preset `name` names, in any case: its command name or label, with
    /// or without spaces; EJK and Ultra low are Vanilla style's old names.
    fn parse(name: &str) -> Option<Self> {
        let name: String = name.split_whitespace().collect();
        if ["ultralow", "ejk"]
            .iter()
            .any(|old| name.eq_ignore_ascii_case(old))
        {
            return Some(Self::Vanilla);
        }
        Self::ALL.into_iter().find(|level| {
            name.eq_ignore_ascii_case(level.command_name())
                || name.eq_ignore_ascii_case(&level.label().replace(' ', ""))
        })
    }

    /// The preset a step of `direction` from this one reaches, stopping at the
    /// ends rather than wrapping.
    pub(crate) fn step(self, direction: i32) -> Self {
        let last = Self::ALL.len() as i32 - 1;
        Self::ALL[(self.index() as i32 + direction.signum()).clamp(0, last) as usize]
    }

    /// The preset whose values `console` holds, if every one matches.
    pub(crate) fn current(console: &ViewerConsole) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|level| level.matches(console) == level.settings())
    }

    /// How many settings the preset sets.
    fn settings(self) -> usize {
        VALUES.len()
            + if self == Self::FpsBoost {
                FPS_BOOST_OFF.len()
            } else {
                0
            }
    }

    /// The current preset, or for custom settings the preset they are nearest,
    /// sharing the most values (the higher on a tie: Vanilla style with its small
    /// effects on is Vanilla style, not FPS boost). Steps and lists start here.
    pub(crate) fn nearest(console: &ViewerConsole) -> Self {
        Self::ALL
            .into_iter()
            .max_by_key(|level| level.matches(console))
            .unwrap_or(Self::DEFAULT)
    }

    /// How many of the preset's values `console` holds.
    fn matches(self, console: &ViewerConsole) -> usize {
        let small_effects = if self == Self::FpsBoost {
            FPS_BOOST_OFF
                .iter()
                .filter(|cvar| holds(console, cvar, "0"))
                .count()
        } else {
            0
        };
        VALUES
            .iter()
            .filter(|(cvar, values)| holds(console, cvar, values[self.index()]))
            .count()
            + small_effects
    }

    /// Set the preset's values, saving the profile once. Leaving FPS boost turns
    /// its small effects back on.
    pub(crate) fn apply(self, console: &mut ViewerConsole) {
        if self != Self::FpsBoost && Self::current(console) == Some(Self::FpsBoost) {
            small_effects_on(console);
        }
        let small_effects_off = FPS_BOOST_OFF
            .iter()
            .filter(|_| self == Self::FpsBoost)
            .map(|cvar| (*cvar, "0"));
        console.set_cvars(
            VALUES
                .iter()
                .map(|(cvar, values)| (*cvar, values[self.index()]))
                .chain(small_effects_off),
        );
    }
}

/// Put FPS boost's small effects back to their defaults.
fn small_effects_on(console: &mut ViewerConsole) {
    let defaults: Vec<(&str, String)> = FPS_BOOST_OFF
        .iter()
        .filter_map(|cvar| Some((*cvar, console.cvar_default(cvar)?.as_text())))
        .collect();
    console.set_cvars(defaults.iter().map(|(cvar, value)| (*cvar, value.as_str())));
}

/// Whether the renderer is Vanilla: the settings are Vanilla style's or FPS
/// boost's, which has every SJK extra off too.
pub(crate) fn vanilla(console: &ViewerConsole) -> bool {
    matches!(
        Level::current(console),
        Some(Level::FpsBoost | Level::Vanilla)
    )
}

/// Set the renderer to `value` ([`VANILLA`] or [`SJK`], any case), as the
/// renderer command does; another value changes nothing.
pub(crate) fn set_renderer(console: &mut ViewerConsole, value: &str) {
    let value = value.trim();
    let to_vanilla = value.eq_ignore_ascii_case(VANILLA);
    if (to_vanilla || value.eq_ignore_ascii_case(SJK)) && to_vanilla != vanilla(console) {
        toggle_vanilla(console);
    }
}

/// Switch between Vanilla style and the SJK renderer. To Vanilla, it keeps the
/// player's value of every setting a preset sets, then applies Vanilla style;
/// back, it puts the kept values back (SJK (recommended) when none were kept,
/// as when Vanilla style was picked as a preset).
pub(crate) fn toggle_vanilla(console: &mut ViewerConsole) {
    if vanilla(console) {
        if Level::current(console) == Some(Level::FpsBoost) {
            small_effects_on(console);
        }
        let kept = console
            .cvar(RESTORE_CVAR)
            .map(sjk_shell::CvarValue::as_text)
            .unwrap_or_default();
        let restored: Vec<(&str, &str)> = kept
            .split(';')
            .filter_map(|pair| pair.split_once('='))
            .filter(|(cvar, _)| VALUES.iter().any(|(known, _)| known == cvar))
            .chain([(RESTORE_CVAR, "")])
            .collect();
        if restored.len() == 1 {
            Level::DEFAULT.apply(console);
        }
        console.set_cvars(restored);
    } else {
        let kept = VALUES
            .iter()
            .filter_map(|(cvar, _)| Some(format!("{cvar}={}", console.cvar(cvar)?.as_text())))
            .collect::<Vec<_>>()
            .join(";");
        console.set_cvar(RESTORE_CVAR, &kept);
        Level::Vanilla.apply(console);
    }
}

/// Start a new profile on SJK (recommended).
pub(crate) fn apply_new_profile(console: &mut ViewerConsole) {
    Level::DEFAULT.apply(console);
}

/// What the Graphics preset row shows: the preset, or Custom.
pub(crate) fn shown(console: &ViewerConsole) -> &'static str {
    Level::current(console).map_or("Custom", Level::label)
}

/// The default a renderer setting's row shows and returns to: its value in the
/// preset in use, or for custom settings in the preset they are nearest, so a
/// preset's rows do not read as changed and Backspace leads back to it; `None`
/// for a setting no preset sets, whose default is its cvar's.
pub(crate) fn row_default(console: &ViewerConsole, cvar: &str) -> Option<&'static str> {
    if FPS_BOOST_OFF
        .iter()
        .any(|name| name.eq_ignore_ascii_case(cvar))
    {
        return (Level::nearest(console) == Level::FpsBoost).then_some("0");
    }
    let (_, values) = VALUES
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(cvar))?;
    Some(values[Level::nearest(console).index()])
}

/// `renderer [vanilla|sjk]`: name the renderer, or set it.
pub(crate) fn renderer_command(
    console: &mut ViewerConsole,
    args: &[String],
) -> Result<Vec<String>, String> {
    let named = |console: &ViewerConsole| {
        if vanilla(console) {
            "Renderer: Vanilla (the original look)".to_owned()
        } else {
            format!(
                "Renderer: SJK, {} graphics; it can still show glitches, which the SJK menu's \
                 Report a bug sends to the team",
                shown(console)
            )
        }
    };
    match args {
        [] => Ok(vec![named(console)]),
        [value]
            if [VANILLA, SJK, "ejk"]
                .iter()
                .any(|name| value.eq_ignore_ascii_case(name)) =>
        {
            let value = if value.eq_ignore_ascii_case(SJK) {
                SJK
            } else {
                VANILLA
            };
            set_renderer(console, value);
            Ok(vec![
                named(console),
                "It applies after a graphics reload (vid_restart)".to_owned(),
            ])
        }
        _ => Err(format!("usage: {RENDERER_COMMAND} [{VANILLA}|{SJK}]")),
    }
}

/// Whether `cvar` holds `value`: numbers compare as numbers ("1.0" holds "1").
fn holds(console: &ViewerConsole, cvar: &str, value: &str) -> bool {
    let Some(current) = console.cvar(cvar).map(sjk_shell::CvarValue::as_text) else {
        return false;
    };
    match (current.trim().parse::<f64>(), value.parse::<f64>()) {
        (Ok(current), Ok(value)) => (current - value).abs() < 1e-6,
        _ => current.trim().eq_ignore_ascii_case(value),
    }
}

/// `graphicsquality [preset]`: name the preset, or set it.
pub(crate) fn command(console: &mut ViewerConsole, args: &[String]) -> Result<Vec<String>, String> {
    let names = || Level::ALL.map(Level::command_name).join(", ");
    match args {
        [] => Ok(vec![format!(
            "Graphics preset: {} ({})",
            shown(console),
            names()
        )]),
        [name] => {
            let level =
                Level::parse(name).ok_or_else(|| format!("usage: {COMMAND} [{}]", names()))?;
            level.apply(console);
            Ok(vec![format!(
                "Graphics preset: {}; shadows, light shafts, HDR, FXAA, sun and sky, \
                 reflection probes and material maps apply after a graphics reload \
                 (vid_restart), emission-map lights on the next map",
                level.label()
            )])
        }
        _ => Err(format!("usage: {COMMAND} [{}]", names())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn console() -> (tempfile::TempDir, ViewerConsole) {
        let directory = tempfile::tempdir().unwrap();
        let console = ViewerConsole::new(directory.path().join("config.cfg")).unwrap();
        (directory, console)
    }

    #[test]
    fn the_cvars_defaults_are_experimental() {
        let (_directory, console) = console();
        for (cvar, values) in VALUES {
            let default = console
                .cvar_default(cvar)
                .unwrap_or_else(|| panic!("{cvar} is not registered"))
                .as_text();
            assert_eq!(
                default.parse::<f64>().ok(),
                values[Level::Experimental.index()].parse::<f64>().ok(),
                "{cvar}: Experimental is not its default"
            );
        }
        assert_eq!(Level::current(&console), Some(Level::Experimental));
        assert_eq!(shown(&console), "Experimental");
    }

    #[test]
    fn each_preset_sets_its_values_and_reads_back() {
        let (directory, mut console) = console();
        for level in Level::ALL.into_iter().rev() {
            level.apply(&mut console);
            assert_eq!(Level::current(&console), Some(level));
            assert_eq!(shown(&console), level.label());
        }
        // The profile was saved: a new start reads the last preset.
        drop(console);
        let console = ViewerConsole::new(directory.path().join("config.cfg")).unwrap();
        assert_eq!(Level::current(&console), Some(Level::FpsBoost));
    }

    #[test]
    fn no_preset_costs_less_than_the_one_below_it() {
        for (cvar, values) in VALUES {
            let numbers: Vec<f64> = values.iter().map(|v| v.parse().unwrap()).collect();
            assert!(
                numbers.windows(2).all(|pair| pair[0] <= pair[1]),
                "{cvar}: {values:?}"
            );
            assert!(
                numbers[0] < numbers[numbers.len() - 1],
                "{cvar} is the same in every preset"
            );
        }
    }

    #[test]
    fn every_value_is_one_its_settings_row_can_show() {
        let (_directory, mut console) = console();
        for level in Level::ALL {
            for (cvar, values) in VALUES {
                let value = values[level.index()];
                assert!(
                    crate::settings::row_takes(cvar, value),
                    "{cvar} {value}: no row in Settings shows it"
                );
                assert!(console.set_cvar(cvar, value), "{cvar} {value}");
                assert!(holds(&console, cvar, value), "{cvar} refused {value}");
            }
        }
    }

    #[test]
    fn a_setting_changed_on_its_own_is_custom_near_its_preset() {
        let (_directory, mut console) = console();
        Level::Experimental.apply(&mut console);
        console.set_cvar("r_sunShadowTaps", "12");
        assert_eq!(Level::current(&console), None);
        assert_eq!(shown(&console), "Custom");
        assert_eq!(Level::nearest(&console), Level::Experimental);
        Level::Sjk.apply(&mut console);
        console.set_cvar("r_normalMapping", "1");
        assert_eq!(Level::nearest(&console), Level::Sjk);
        // Its rows' defaults are the nearest preset's: the changed one reads
        // as changed, the others do not.
        assert_eq!(row_default(&console, "r_normalMapping"), Some("0"));
        assert_eq!(row_default(&console, "r_sceneHdr"), Some("1"));
        assert_eq!(row_default(&console, "cg_fov"), None);
    }

    #[test]
    fn steps_stop_at_the_ends() {
        assert_eq!(Level::FpsBoost.step(-1), Level::FpsBoost);
        assert_eq!(Level::FpsBoost.step(1), Level::Vanilla);
        assert_eq!(Level::Vanilla.step(-1), Level::FpsBoost);
        assert_eq!(Level::Vanilla.step(1), Level::Sjk);
        assert_eq!(Level::Sjk.step(-1), Level::Vanilla);
        assert_eq!(Level::Sjk.step(1), Level::Experimental);
        assert_eq!(Level::Experimental.step(1), Level::Experimental);
    }

    #[test]
    fn the_command_names_and_sets_the_preset() {
        let (_directory, mut console) = console();
        let lines = command(&mut console, &[]).unwrap();
        assert!(
            lines[0].starts_with("Graphics preset: Experimental"),
            "{lines:?}"
        );
        command(&mut console, &["SJK".to_owned()]).unwrap();
        assert_eq!(Level::current(&console), Some(Level::Sjk));
        assert!(command(&mut console, &["ultra".to_owned()]).is_err());
        assert_eq!(Level::current(&console), Some(Level::Sjk));
        command(&mut console, &["ultralow".to_owned()]).unwrap();
        assert_eq!(Level::current(&console), Some(Level::Vanilla));
        command(&mut console, &["Experimental".to_owned()]).unwrap();
        assert_eq!(Level::current(&console), Some(Level::Experimental));
        command(&mut console, &["ejk".to_owned()]).unwrap();
        assert!(vanilla(&console));
        command(&mut console, &["SJK (recommended)".to_owned()]).unwrap();
        assert_eq!(Level::current(&console), Some(Level::Sjk));
        command(&mut console, &["Vanilla style".to_owned()]).unwrap();
        assert!(vanilla(&console));
    }

    #[test]
    fn vanilla_turns_sjk_lighting_and_material_maps_off() {
        let (_directory, mut console) = console();
        Level::Vanilla.apply(&mut console);
        for cvar in [
            "r_dayNight",
            "r_worldSunShadows",
            "r_volumetrics",
            "r_cubeMapping",
            "r_floorReflections",
            "r_normalMapping",
            "r_specularMapping",
            "r_parallaxMapping",
            "r_emissiveMaps",
            "r_ssao",
            "r_sceneHdr",
        ] {
            assert!(holds(&console, cvar, "0"), "{cvar}");
        }
    }

    #[test]
    fn fps_boost_is_vanilla_with_the_small_effects_off_too() {
        let (_directory, mut console) = console();
        command(&mut console, &["FPS boost".to_owned()]).unwrap();
        assert_eq!(Level::current(&console), Some(Level::FpsBoost));
        assert_eq!(shown(&console), "FPS boost");
        assert!(vanilla(&console));
        for (cvar, values) in VALUES {
            assert_eq!(
                values[Level::FpsBoost.index()],
                values[Level::Vanilla.index()],
                "{cvar}: FPS boost is Vanilla style's renderer"
            );
        }
        for cvar in FPS_BOOST_OFF {
            assert!(holds(&console, cvar, "0"), "{cvar}");
            assert!(crate::settings::row_takes(cvar, "0"), "{cvar}");
            assert_eq!(row_default(&console, cvar), Some("0"));
            // Its default is on, which leaving FPS boost puts back.
            let default = console.cvar_default(cvar).unwrap().as_text();
            assert_eq!(default.parse::<f64>().ok(), Some(1.0), "{cvar}");
        }
        // Another preset puts the small effects back.
        command(&mut console, &["vanilla".to_owned()]).unwrap();
        assert_eq!(Level::current(&console), Some(Level::Vanilla));
        assert!(holds(&console, "cg_marks", "1"));
        assert_eq!(row_default(&console, "cg_marks"), None);
        command(&mut console, &["fpsboost".to_owned()]).unwrap();
        command(&mut console, &["sjk".to_owned()]).unwrap();
        assert!(!vanilla(&console));
        assert!(holds(&console, crate::weather::CVAR, "1"));
        // So does the renderer switch back to SJK.
        command(&mut console, &["fpsboost".to_owned()]).unwrap();
        set_renderer(&mut console, SJK);
        assert!(!vanilla(&console));
        assert!(holds(&console, "cg_shadows", "1"));
    }

    #[test]
    fn the_other_presets_keep_the_players_small_effects() {
        let (_directory, mut console) = console();
        Level::Sjk.apply(&mut console);
        console.set_cvar("cg_marks", "0");
        assert_eq!(Level::current(&console), Some(Level::Sjk));
        Level::Experimental.apply(&mut console);
        assert!(holds(&console, "cg_marks", "0"));
        Level::Vanilla.apply(&mut console);
        assert_eq!(Level::current(&console), Some(Level::Vanilla));
        assert!(holds(&console, "cg_marks", "0"));
    }

    #[test]
    fn vanilla_and_back_restores_the_players_settings() {
        let (directory, mut console) = console();
        Level::Experimental.apply(&mut console);
        console.set_cvar("r_sunShadowTaps", "12");
        assert!(!vanilla(&console));
        set_renderer(&mut console, VANILLA);
        assert!(vanilla(&console));
        // Kept across a restart.
        drop(console);
        let mut console = ViewerConsole::new(directory.path().join("config.cfg")).unwrap();
        assert!(vanilla(&console));
        set_renderer(&mut console, VANILLA);
        assert!(vanilla(&console), "Vanilla again changes nothing");
        set_renderer(&mut console, SJK);
        assert!(!vanilla(&console));
        assert!(holds(&console, "r_sunShadowTaps", "12"));
        console.set_cvar("r_sunShadowTaps", "16");
        assert_eq!(Level::current(&console), Some(Level::Experimental));
        assert!(holds(&console, RESTORE_CVAR, ""));
        // Vanilla picked as a preset has nothing kept: back is SJK (recommended).
        Level::Vanilla.apply(&mut console);
        toggle_vanilla(&mut console);
        assert_eq!(Level::current(&console), Some(Level::Sjk));
    }

    #[test]
    fn a_new_profile_is_sjk_recommended_and_its_rows_read_unchanged() {
        let (_directory, mut console) = console();
        apply_new_profile(&mut console);
        assert_eq!(Level::current(&console), Some(Level::Sjk));
        assert_eq!(shown(&console), "SJK (recommended)");
        assert_eq!(row_default(&console, "r_sceneHdr"), Some("1"));
        assert_eq!(row_default(&console, "r_dayNight"), Some("0"));
        assert_eq!(row_default(&console, "cg_fov"), None);
        let lines = renderer_command(&mut console, &[]).unwrap();
        assert!(lines[0].contains("SJK (recommended)"), "{lines:?}");
        renderer_command(&mut console, &["vanilla".to_owned()]).unwrap();
        assert!(vanilla(&console));
        assert_eq!(row_default(&console, "r_sceneHdr"), Some("0"));
        renderer_command(&mut console, &["SJK".to_owned()]).unwrap();
        assert_eq!(Level::current(&console), Some(Level::Sjk));
        assert!(renderer_command(&mut console, &["rtx".to_owned()]).is_err());
    }
}
