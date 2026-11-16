//! Bundled community Force presets, offered through the normal `.fcf` loader.
//! JoF EJK #23 supplies the data; `assets/forcecfg/README.md` records its source.
//! Mounted below installed content so a pack can replace any preset.

use sjk_vfs::{VfsError, VirtualFileSystem};

macro_rules! preset {
    ($path:literal) => {
        (
            $path,
            include_bytes!(concat!("../assets/", $path)).as_slice(),
        )
    };
}

/// Each bundled preset's virtual path and original contents.
pub(crate) const FILES: &[(&str, &[u8])] = &[
    preset!("forcecfg/dark/^5{^2JoF^5}^2Academy Trial.fcf"),
    preset!("forcecfg/dark/^5{^2JoF^5}^2Coop - Devastating.fcf"),
    preset!("forcecfg/dark/^5{^2JoF^5}^2Coop - Hard.fcf"),
    preset!("forcecfg/dark/^5{^2JoF^5}^2Coop - Normal.fcf"),
    preset!("forcecfg/dark/^5{^2JoF^5}^2Fullforce CTF.fcf"),
    preset!("forcecfg/dark/^5{^2JoF^5}^2Fullforce Duels.fcf"),
    preset!("forcecfg/dark/^5{^2JoF^5}^2Fullforce FFA.fcf"),
    preset!("forcecfg/dark/^5{^2JoF^5}^2Fullforce TFFA.fcf"),
    preset!("forcecfg/dark/^5{^2JoF^5}^2Trial Mission.fcf"),
    preset!("forcecfg/light/^5{^2JoF^5}^2Coop - Devastating.fcf"),
    preset!("forcecfg/light/^5{^2JoF^5}^2Coop - Hard.fcf"),
    preset!("forcecfg/light/^5{^2JoF^5}^2Coop - Normal.fcf"),
    preset!("forcecfg/light/^5{^2JoF^5}^2Crucible Origins.fcf"),
    preset!("forcecfg/light/^5{^2JoF^5}^2Crucible.fcf"),
    preset!("forcecfg/light/^5{^2JoF^5}^2Fullforce CTF.fcf"),
    preset!("forcecfg/light/^5{^2JoF^5}^2Fullforce Duels.fcf"),
    preset!("forcecfg/light/^5{^2JoF^5}^2Fullforce FFA.fcf"),
    preset!("forcecfg/light/^5{^2JoF^5}^2Fullforce TFFA.fcf"),
    preset!("forcecfg/light/^5{^2JoF^5}^2Gauntlet Duo.fcf"),
    preset!("forcecfg/light/^5{^2JoF^5}^2Gauntlet Solo.fcf"),
    preset!("forcecfg/light/^5{^2JoF^5}^2Side Mission.fcf"),
    preset!("forcecfg/light/^5{^2JoF^5}^2Trial Mission.fcf"),
];

/// Mount the bundled presets below subsequently installed game data.
pub(crate) fn mount(vfs: &mut VirtualFileSystem) -> Result<(), VfsError> {
    vfs.mount_memory("SJK community Force presets", FILES.iter().copied())?;
    Ok(())
}
