//! The main menu's first page in either style, and the version line the
//! menus and console show.

use super::ClientMenu;
use crate::{TextVertex, UiFont};

/// Build version ([`crate::build_info::VERSION`]), bottom right of the main
/// menus; the console shows it too.
pub(crate) const VERSION_LINE: &str = concat!("SJK ", env!("SJK_BUILD_VERSION"));

/// The version line the classic menu shows: the version, and what the update line
/// says ([`crate::update::notice`]); a click on it acts where the notice is clickable
/// ([`ClientMenu::follow_update_notice`]).
pub(crate) fn version_line(
    notice: Option<crate::update::Notice>,
) -> std::borrow::Cow<'static, str> {
    use crate::update::Notice;
    match notice {
        Some(Notice::Ready(version)) => format!(
            "{VERSION_LINE}   /   update {} available: click to install",
            version.as_str()
        )
        .into(),
        Some(Notice::Manual(version)) => format!(
            "{VERSION_LINE}   /   update {} available: click for its page",
            version.as_str()
        )
        .into(),
        Some(Notice::Downloading(Some(percent))) => {
            format!("{VERSION_LINE}   /   downloading the update {percent}%").into()
        }
        Some(Notice::Downloading(None)) => {
            format!("{VERSION_LINE}   /   downloading the update").into()
        }
        Some(Notice::Installing) => format!("{VERSION_LINE}   /   installing the update").into(),
        Some(Notice::Installed) => {
            format!("{VERSION_LINE}   /   update installed: click to restart").into()
        }
        Some(Notice::Failed) => {
            format!("{VERSION_LINE}   /   the update did not work: click for details").into()
        }
        None => VERSION_LINE.into(),
    }
}

impl ClientMenu {
    /// A click on the main menu's update line, in either style: it starts the
    /// download of the release waiting (the line then shows its progress), opens the
    /// page of one this folder cannot install, restarts into an installed one, or
    /// opens the Update page after a failed install.
    pub(super) fn follow_update_notice(
        &mut self,
        console: &mut crate::console::ViewerConsole,
    ) -> super::MenuAction {
        match crate::update::click_notice() {
            crate::update::NoticeClick::Done => super::MenuAction::None,
            crate::update::NoticeClick::Restart => super::MenuAction::Quit,
            crate::update::NoticeClick::OpenPage => {
                self.open_main_destination(super::MainDestination::Update, console)
            }
        }
    }

    pub(super) fn append_main(
        &mut self,
        vertices: &mut Vec<TextVertex>,
        font: &UiFont,
        viewport: [f32; 2],
    ) {
        match self.menu_style {
            super::MenuStyle::Classic => {
                let reveal = self.screen_reveal();
                super::classic::view::build(
                    &mut self.ui,
                    viewport,
                    &self.classic,
                    reveal,
                    self.art,
                );
                self.ui.append_text(vertices, font, viewport);
            }
            // Drawn with the player's details by `append_sjk_home`; here, from
            // a caller without the console, in Inter.
            super::MenuStyle::Sjk => {
                let target = super::sjk::TextTarget::Inter(vertices, font);
                self.append_sjk_home(target, None, viewport);
            }
        }
    }
}
