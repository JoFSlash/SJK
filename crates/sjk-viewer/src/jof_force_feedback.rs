//! Feature-gated host bridge for the JoF mod's server-confirmed sounds.
use crate::console::ViewerConsole;
use sjk_mod_jof::feedback;

pub(crate) fn enabled(console: Option<&ViewerConsole>) -> bool {
    console.is_some_and(|c| {
        c.mods.loaded(sjk_mod_jof::ID)
            && c.bool_cvar("mod_jof").unwrap_or(false)
            && c.bool_cvar("jof.ejkoptin").unwrap_or(false)
            && c.bool_cvar("jof.abilitysounds").unwrap_or(false)
    })
}

pub(crate) fn dispatch(
    args: &[Vec<u8>],
    console: Option<&ViewerConsole>,
    audio: Option<&mut crate::GameAudio>,
    snapshot: &sjk_protocol::Snapshot,
) -> bool {
    if !enabled(console) {
        return false;
    }
    let Some(index) = args.first().and_then(|name| feedback::command(name)) else {
        return false;
    };
    if let Some(audio) = audio {
        audio.force_feedback(index, snapshot);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn confirmations_are_ignored_without_loaded_jof_and_both_preferences() {
        let dir = tempfile::tempdir().unwrap();
        let mut c = ViewerConsole::new(dir.path().join("config.cfg")).unwrap();
        assert!(!enabled(Some(&c)));
        for (kind, info, expected) in [
            (sjk_mod::ServerKind::BaseJka, "", false),
            (sjk_mod::ServerKind::JaPlus, r"\V\2.4B7", false),
            (sjk_mod::ServerKind::JaPro, r"\V\2.5B0", false),
            (sjk_mod::ServerKind::JaPlus, r"\V\2.5B0", true),
        ] {
            c.mods.set_server(Some(sjk_mod::Server {
                kind,
                address: "fixture".into(),
                info: info.into(),
            }));
            c.execute_buffered_frame(None, |_, _| false);
            assert_eq!(enabled(Some(&c)), expected);
        }
        for name in [
            sjk_mod_jof::ABILITY_SOUNDS,
            sjk_mod_jof::EJK_OPT_IN,
            "mod_jof",
        ] {
            c.set_cvar(name, "0");
            assert!(!enabled(Some(&c)));
            c.set_cvar(name, "1");
            c.execute_buffered_frame(None, |_, _| false);
        }
        c.mods.set_server(None);
        c.execute_buffered_frame(None, |_, _| false);
        assert!(!enabled(Some(&c)));
    }
}
