//! The system's file dialog behind the Browse buttons (the Import page's `.cfg`, the
//! Profile picture). It blocks until the dialog closes, so callers run it on a worker
//! thread.
//!
//! On Linux `rfd` asks the desktop portal, which never answers on desktops whose
//! portal has no file chooser (wlroots compositors, gamescope): Browse then waited for
//! good. So `zenity`, then `kdialog`, are asked first when installed; the portal comes
//! after them (and `rfd` tries `zenity` itself when no portal runs).

use std::path::PathBuf;

/// The file the player picked with the system's dialog, titled `title` and listing
/// the `extensions` (without dots) under `filter`; `None` when it was closed without
/// one or no dialog could open.
pub(crate) fn pick_file(title: &str, filter: &str, extensions: &[&str]) -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    if let Some(picked) = linux::pick_file(title, filter, extensions) {
        return picked;
    }
    rfd::FileDialog::new()
        .set_title(title)
        .add_filter(filter, extensions)
        .pick_file()
}

#[cfg(target_os = "linux")]
mod linux {
    use std::path::PathBuf;
    use std::process::{Command, Stdio};

    /// What `zenity` or `kdialog` answered; `None` when neither could show a dialog.
    pub(super) fn pick_file(
        title: &str,
        filter: &str,
        extensions: &[&str],
    ) -> Option<Option<PathBuf>> {
        let patterns = patterns(extensions);
        let mut zenity = Command::new("zenity");
        zenity
            .arg("--file-selection")
            .arg(format!("--title={title}"))
            .arg(format!("--file-filter={filter} | {patterns}"));
        let mut kdialog = Command::new("kdialog");
        kdialog
            .arg("--title")
            .arg(title)
            .arg("--getopenfilename")
            .arg(std::env::var_os("HOME").unwrap_or_else(|| ".".into()))
            .arg(format!("{patterns}|{filter}"));
        [zenity, kdialog].into_iter().find_map(run)
    }

    /// Run one dialog program: the file (exit 0), nothing (1, closed), or `None`
    /// when it is not installed or failed to show, so the next one is tried.
    fn run(mut command: Command) -> Option<Option<PathBuf>> {
        let output = command
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .ok()?;
        match output.status.code() {
            Some(0) => Some(answer(&output.stdout)),
            Some(1) => Some(None),
            _ => None,
        }
    }

    /// The path a dialog printed, without its line break.
    pub(super) fn answer(stdout: &[u8]) -> Option<PathBuf> {
        use std::os::unix::ffi::OsStrExt;
        let path = stdout.strip_suffix(b"\n").unwrap_or(stdout);
        (!path.is_empty()).then(|| PathBuf::from(std::ffi::OsStr::from_bytes(path)))
    }

    /// `*.cfg *.CFG`: Linux file names are case-sensitive and JKA's are not.
    pub(super) fn patterns(extensions: &[&str]) -> String {
        extensions
            .iter()
            .flat_map(|extension| {
                [
                    format!("*.{}", extension.to_ascii_lowercase()),
                    format!("*.{}", extension.to_ascii_uppercase()),
                ]
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn both_cases_are_listed_and_the_answer_loses_its_line_break() {
            assert_eq!(patterns(&["cfg", "Tga"]), "*.cfg *.CFG *.tga *.TGA");
            assert_eq!(
                answer(b"/home/sol/jampconfig.cfg\n"),
                Some(PathBuf::from("/home/sol/jampconfig.cfg"))
            );
            assert_eq!(answer(b""), None);
        }
    }
}
