//! JoF EJK's duel look: for players outside every duel, a duellist is drawn only as
//! a see-through blue figure, `gfx/effects/duelShader`, in place of its body
//! (EternalJK `CG_Player`, `cg_stylePlayer` bit `JAPRO_STYLE_VFXDUELERS`, as JoF
//! plays it). The dark body and the sight bubble of the stock look are not drawn
//! then ([`sjk_client::legacy_duel_tint`], [`sjk_client::legacy_force_overlays`]).
//!
//! The shader, `sjk/duelFigure`, is EternalJK's `gfx/effects/duelShader`
//! (`assets/japro/shaders/japro_players.shader`) under SJK's own name, so a JoF or
//! EternalJK pack's copy never replaces it:
//!
//! - the Force shell's texture through `tcGen environment`, added in a dark blue;
//! - a faint red wash, lit like the body, that writes depth so the figure hides
//!   what is behind its own far side;
//! - a blue added over all of it, pulsing gently. EternalJK adds 0.9 of full blue;
//!   in SJK that read as a solid figure (playtest on JoF), so SJK adds 0.6, letting
//!   more of what is behind show through.
//!
//! Its `textures/colors/red` and `textures/colors/blue` are plain colours; SJK
//! mounts its own below game data in case an installation lacks them.
//!
//! `cg_duelGhost` ([`Look`]) chooses it (1, the default); 0 keeps the stock look, and
//! 2 hides the duellists altogether, with their sabers, shots, events and sounds
//! (headshotsmcgee117's idea, [`sjk_client::DuelHideScope::Duellists`]). Each holds
//! only while the viewer (or the player it follows) is in no duel itself.

use sjk_vfs::{VfsError, VirtualFileSystem};

/// The figure's shader: EternalJK's `gfx/effects/duelShader` (`cgs.media.duelShader`),
/// with a lighter blue.
pub(crate) const SHADER: &str = "sjk/duelFigure";
/// How players in others' duels look: [`Look`].
pub(crate) const CVAR: &str = "cg_duelGhost";

/// `cg_duelGhost`: how a viewer in no duel sees the players in a private duel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Look {
    /// 0: dark under the blue sight bubble, as stock JKA draws them.
    Dark,
    /// 1, the default: JoF EJK's see-through blue figures.
    Figures,
    /// 2: not drawn or heard at all.
    Hidden,
}

impl Look {
    /// The look a cvar value names; values out of range are the default.
    pub(crate) fn from_cvar(value: i64) -> Self {
        match value {
            0 => Self::Dark,
            2 => Self::Hidden,
            _ => Self::Figures,
        }
    }
}

const SHADER_FILE: &str = "// SJK: JoF EJK's duel look (duel_ghost.rs), after EternalJK's
// assets/japro/shaders/japro_players.shader (gfx/effects/duelShader), its last blue
// 0.6 rather than 0.9 so the figure is more see-through.
sjk/duelFigure
{
\t{
\t\tmap gfx/mp/forceshell
\t\tblendFunc GL_ONE GL_ONE
\t\trgbGen const ( 0.1 0.1 0.4 )
\t\ttcGen environment
\t\ttcMod rotate 25
\t\ttcMod turb 0.4 0.2 0 0.4
\t\ttcMod scale 0.2 0.2
\t}
\t{
\t\tmap textures/colors/red
\t\tblendFunc GL_SRC_ALPHA GL_ONE_MINUS_SRC_ALPHA
\t\tdepthWrite
\t\trgbGen lightingDiffuse
\t\talphaGen wave sin 0.1 0.1 0.1 0.1
\t}
\t{
\t\tmap textures/colors/blue
\t\tblendFunc GL_ONE GL_ONE
\t\trgbGen wave sin 0.6 0.07 0.1 0.1
\t}
}
";

/// An 8x8 uncompressed true-colour TGA of one colour.
fn plain_tga(rgb: [u8; 3]) -> Vec<u8> {
    let mut file = vec![0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 8, 0, 24, 0];
    for _ in 0..64 {
        // TGA stores blue, green, red.
        file.extend([rgb[2], rgb[1], rgb[0]]);
    }
    file
}

/// Mount the shader and its plain colours, below game data like SJK's other content.
pub(crate) fn mount(vfs: &mut VirtualFileSystem) -> Result<(), VfsError> {
    vfs.mount_memory(
        "SJK duel ghost",
        [
            (
                "shaders/sjk_duel_ghost.shader",
                SHADER_FILE.as_bytes().to_vec(),
            ),
            ("textures/colors/red.tga", plain_tga([255, 0, 0])),
            ("textures/colors/blue.tga", plain_tga([0, 0, 255])),
        ],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ghost_shader_is_eternaljks_three_stages() {
        let shaders =
            sjk_shader::parse_shader_script(SHADER_FILE.as_bytes(), "duel_ghost").unwrap();
        assert_eq!(shaders.len(), 1);
        assert!(shaders[0].name.eq_ignore_ascii_case(SHADER));
        assert_eq!(shaders[0].stages.len(), 3);
        assert_eq!(shaders[0].stages[0].rgb_constant, Some([0.1, 0.1, 0.4]));
    }

    #[test]
    fn the_look_follows_the_cvar_and_hidden_reaches_the_duel_hiding() {
        assert_eq!(Look::from_cvar(0), Look::Dark);
        assert_eq!(Look::from_cvar(1), Look::Figures);
        assert_eq!(Look::from_cvar(2), Look::Hidden);
        assert_eq!(Look::from_cvar(9), Look::Figures);
        let directory = tempfile::tempdir().unwrap();
        let mut console =
            crate::console::ViewerConsole::new(directory.path().join("config.cfg")).unwrap();
        assert_eq!(console.integer_cvar(CVAR), Some(1));
        assert!(!console.duel_hide().hide_duellists);
        assert!(console.set_cvar(CVAR, "2"));
        assert!(console.duel_hide().hide_duellists);
        assert!(console.set_cvar(CVAR, "0"));
        assert!(!console.duel_hide().hide_duellists);
    }

    #[test]
    fn the_plain_colours_are_whole_tga_files() {
        let blue = plain_tga([0, 0, 255]);
        assert_eq!(blue.len(), 18 + 64 * 3);
        assert_eq!(&blue[18..21], &[255, 0, 0]);
        let mut vfs = VirtualFileSystem::default();
        mount(&mut vfs).unwrap();
        assert!(vfs.contains("textures/colors/red.tga").unwrap());
        assert!(vfs.contains("shaders/sjk_duel_ghost.shader").unwrap());
    }
}
