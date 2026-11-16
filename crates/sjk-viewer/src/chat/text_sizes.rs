//! One-time move of saved chat and centre-print sizes to EternalJK's scale.
//!
//! On 19/10/2026 `cg_chatBoxFontSize` 1 and `cg_centerSize` 1 became EternalJK's
//! absolute sizes ([`super::layout::FONT_1080`], [`super::view::CENTER_FONT_1080`]);
//! before, they drew a smaller SJK size. A profile saved before that keeps the size
//! it showed: a value other than the old default 1 is divided by how much larger
//! the new 1 is, and clamped to the new range. A saved 1 was the default, and takes
//! the new default like a new profile.
//!
//! Later that day the chat's default became [`super::options::CHAT_SIZE_DEFAULT`]
//! (Sol found EternalJK's size too large); a profile saved in between with the chat
//! at 1, the default it had then, takes the new one (marker 2).

use sjk_shell::{CvarRegistry, CvarValue};

/// Archived marker: 0 until the saved sizes were moved to the new scale, 1 while
/// the chat's default was EternalJK's 1, 2 since.
pub(crate) const VERSION_CVAR: &str = "cg_textSizeEjkVersion";
const VERSION: &str = "2";

/// How much larger the new chat text is at the same value: the old line box was
/// 18 px at 1080 lines.
pub(crate) const CHAT_FACTOR: f32 = super::layout::FONT_1080 / 18.0;
/// How much larger the new centre-print text is at the same value: the old line
/// box was 22 px at 1080 lines.
pub(crate) const CENTER_FACTOR: f32 = super::view::CENTER_FONT_1080 / 22.0;

const CHAT: &str = "cg_chatBoxFontSize";
const CENTER: &str = "cg_centerSize";

/// Move a profile's saved sizes to the current scale and default once, then set
/// the marker. `new_profile` (no config file yet) only sets the marker.
pub(crate) fn migrate(cvars: &mut CvarRegistry, new_profile: bool) {
    let version = match cvars.get(VERSION_CVAR).map(|cvar| &cvar.value) {
        Some(CvarValue::Integer(version @ (0 | 1))) => *version,
        _ => return,
    };
    if !new_profile {
        let saved = |cvars: &CvarRegistry, name| {
            crate::console::saved_number(cvars, name).map(|value| value as f32)
        };
        // The default 1 of either earlier scale takes today's default.
        let chat_was_default = saved(cvars, CHAT) == Some(1.0);
        if chat_was_default {
            let _ = cvars.reset(CHAT);
        }
        if version == 0 {
            let (chat_min, chat_max) = super::options::CHAT_SIZE;
            for (name, factor, min, max) in [
                (CHAT, CHAT_FACTOR, chat_min as f32, chat_max as f32),
                (CENTER, CENTER_FACTOR, 0.0, super::options::CENTER_SIZE_MAX),
            ] {
                let Some(value) = saved(cvars, name) else {
                    continue;
                };
                if value == 1.0 || !value.is_finite() || (name == CHAT && chat_was_default) {
                    continue;
                }
                let moved = (value / factor).clamp(min, max);
                let _ = cvars.set_text(name, &format!("{moved:.3}"));
            }
        }
    }
    let _ = cvars.set_text(VERSION_CVAR, VERSION);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::ViewerConsole;

    #[test]
    fn the_new_sizes_are_eternaljks() {
        // ocr_a's 21-pixel line at 0.65, ergoec's 22 at 1, 2.25 1080p pixels each.
        assert!((super::super::layout::FONT_1080 - 30.7125).abs() < 1e-4);
        assert!((super::super::view::CENTER_FONT_1080 - 49.5).abs() < 1e-4);
        assert!((super::super::view::center_row(1.0) - 58.5).abs() < 1e-4);
        assert!((CHAT_FACTOR - 1.70625).abs() < 1e-5);
        assert!((CENTER_FACTOR - 2.25).abs() < 1e-5);
    }

    /// A saved size other than the old default keeps its look, once; the old
    /// default and a new profile get today's default.
    #[test]
    fn saved_text_sizes_move_to_the_eternaljk_scale_once() {
        let directory = tempfile::tempdir().unwrap();
        let fresh = ViewerConsole::new(directory.path().join("new.cfg")).unwrap();
        assert_eq!(fresh.float_cvar("cg_chatBoxFontSize"), Some(0.85));
        assert_eq!(fresh.integer_cvar(VERSION_CVAR), Some(2));
        drop(fresh);
        for (chat, center, chat_after, center_after) in [
            ("1", "1", 0.85, 1.0),
            ("1.25", "2", 0.733, 0.889),
            ("3", "4", 1.758, 1.778),
            ("0.25", "0.5", 0.15, 0.222),
        ] {
            let path = directory.path().join(format!("old-{chat}-{center}.cfg"));
            std::fs::write(
                &path,
                format!("seta cg_chatBoxFontSize \"{chat}\"\nseta cg_centerSize \"{center}\"\n"),
            )
            .unwrap();
            let mut console = ViewerConsole::new(path.clone()).unwrap();
            let read = |console: &ViewerConsole, name| console.float_cvar(name).unwrap();
            assert!((read(&console, "cg_chatBoxFontSize") - chat_after).abs() < 1e-9);
            assert!((read(&console, "cg_centerSize") - center_after).abs() < 1e-9);
            assert!(console.set_cvar("cg_chatBoxFontSize", "1.5"));
            drop(console);
            let console = ViewerConsole::new(path).unwrap();
            assert_eq!(console.float_cvar("cg_chatBoxFontSize"), Some(1.5));
            assert!((read(&console, "cg_centerSize") - center_after).abs() < 1e-9);
        }
    }

    /// A profile saved while the chat's default was EternalJK's 1 takes the
    /// smaller default; a size chosen then stays.
    #[test]
    fn the_eternaljk_default_moves_to_the_new_default_once() {
        let directory = tempfile::tempdir().unwrap();
        for (chat, chat_after) in [("1.0", 0.85), ("1.2", 1.2)] {
            let path = directory.path().join(format!("v1-{chat}.cfg"));
            std::fs::write(
                &path,
                format!(
                    "seta cg_chatBoxFontSize \"{chat}\"
seta cg_centerSize \"1.5\"
                     seta {VERSION_CVAR} \"1\"
"
                ),
            )
            .unwrap();
            let console = ViewerConsole::new(path).unwrap();
            assert_eq!(console.float_cvar("cg_chatBoxFontSize"), Some(chat_after));
            assert_eq!(console.float_cvar("cg_centerSize"), Some(1.5));
            assert_eq!(console.integer_cvar(VERSION_CVAR), Some(2));
        }
    }
}
