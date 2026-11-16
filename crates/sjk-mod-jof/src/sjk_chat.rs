//! SJK chat on JoF servers (`docs/mods.md`, "SJK chat on JoF servers"). JoF asked
//! for it so the SJK chat does not split their players away from All chat: while
//! JoF tools are loaded, what the player says on the SJK chat during a match is said in
//! the server's All chat too, once the hub took it, as JoF EternalJK does. The copy
//! starts with [`sjk_client::SJK_COPY_TAG`] and ends with the hub's id for the message
//! ([`sjk_client::sjk_copy_marker`]), which the core client reads to show the line
//! once. This module owns the sending and the words that tell the player; the viewer
//! only bridges them while the mod is loaded.

use sjk_client::{ChatDestination, SJK_COPY_TAG, chat_command, chat_input_fits, sjk_copy_marker};

/// Archived; on until the notice card was shown once on a JoF server. Turning it back
/// on shows the card again on the next JoF server.
pub const NOTICE: &str = "jof.sjkChatNotice";

/// The notice card's kicker and headline.
pub const NOTICE_KICKER: &str = "JOF SERVERS";
pub const NOTICE_TITLE: &str = "SJK chat is said in All chat here";
/// What the card says, one paragraph each.
pub const NOTICE_PARAGRAPHS: [&str; 3] = [
    "On JoF servers, what you say in SJK chat during a match is also said in the \
     server's All chat, so every player on the server reads it, whatever client they \
     use.",
    "JoF asked for this so that SJK chat does not split their players away from All \
     chat. Messages with a GIF stay in SJK chat only. Elsewhere, and in the menus, SJK \
     chat stays as it was.",
    "Lines said in both show once, marked ALL/SJK; lines only in SJK chat are marked \
     SJK. While you type in SJK chat here, a red note above the field reminds you.",
];
/// The red note above the in-game chat field while the SJK channel is picked.
pub const COMPOSER_NOTE: &str = "Also said in All chat: everyone on this server reads it";
/// The SJK chat page's button that sends one message to SJK chat only, for every
/// player, and its hover card: why it should stay rare.
pub const ONLY_SJK: &str = "Send only in SJK";
pub const ONLY_SJK_TITLE: &str = "Only in SJK chat";
pub const ONLY_SJK_WHY: [&str; 3] = [
    "Sends this message to SJK chat only: the server's All chat does not get it.",
    "JoF asked for SJK chat to be said in All chat so the community does not split. \
     Keep messages only in SJK for when that is justified.",
    "Staff: Shift+Enter sends only in SJK from any chat field.",
];

/// The `say` commands that put an SJK chat message the hub took (its `id`) in the game
/// server's chat, each after [`SJK_COPY_TAG`] and ended with its marker. A server keeps
/// [`sjk_client::CHAT_INPUT_BYTES`] of a line, so a longer message goes as several, cut
/// between words (inside a word only when it does not fit alone); the reliable channel
/// sends them a second apart. Empty for an id too long for a marker: such a copy could
/// not be told from a plain line, so it is not said.
pub fn copy_commands(text: &str, id: u64) -> Vec<String> {
    let Some(marker) = sjk_copy_marker(id) else {
        return Vec::new();
    };
    let fits = |piece: &str| chat_input_fits(&format!("{SJK_COPY_TAG}{piece}{marker}"));
    let mut pieces = Vec::new();
    let mut piece = String::new();
    for word in text.split_whitespace() {
        let joined = if piece.is_empty() {
            word.to_owned()
        } else {
            format!("{piece} {word}")
        };
        if fits(&joined) {
            piece = joined;
            continue;
        }
        if !piece.is_empty() {
            pieces.push(std::mem::take(&mut piece));
        }
        for c in word.chars() {
            piece.push(c);
            if !fits(&piece) {
                piece.pop();
                pieces.push(std::mem::take(&mut piece));
                piece.push(c);
            }
        }
    }
    if !piece.is_empty() {
        pieces.push(piece);
    }
    pieces
        .iter()
        .filter_map(|piece| {
            chat_command(
                ChatDestination::Global,
                &format!("{SJK_COPY_TAG}{piece}{marker}"),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjk_client::CHAT_INPUT_BYTES;

    #[test]
    fn a_message_goes_to_the_server_in_pieces_that_fit_each_with_its_id() {
        assert_eq!(
            copy_commands("  hello   there ", 12),
            ["say \"^3SJK^2 hello there^9^2^1^2\""]
        );
        assert!(copy_commands("   ", 12).is_empty());
        assert!(
            copy_commands("hello", 1_000_000_000).is_empty(),
            "no marker, no copy"
        );
        let text = "word ".repeat(60);
        let commands = copy_commands(&text, 12_345);
        assert_eq!(commands.len(), 3);
        let mut words = 0;
        for command in &commands {
            let payload = command
                .strip_prefix("say \"")
                .and_then(|rest| rest.strip_suffix('"'))
                .unwrap();
            assert!(payload.len() <= CHAT_INPUT_BYTES, "{command}");
            let body = payload
                .strip_prefix(SJK_COPY_TAG)
                .and_then(|rest| rest.strip_suffix("^9^5^1^2^3^4^5"))
                .unwrap();
            assert!(!body.starts_with(' ') && !body.ends_with(' '), "{command}");
            words += body.split(' ').count();
        }
        assert_eq!(words, 60, "no word lost");
        // A word too long for a piece alone is cut inside it.
        let long = "x".repeat(200);
        let commands = copy_commands(&long, 1);
        assert_eq!(commands.len(), 2);
        assert!(
            commands
                .iter()
                .all(|command| command.starts_with("say \"^3SJK^2 ")
                    && command.ends_with("^9^1^1\""))
        );
    }

    #[test]
    fn the_notice_says_who_asked_and_where_it_applies() {
        let words = NOTICE_PARAGRAPHS.join(" ");
        for needed in ["All chat", "JoF asked", "Elsewhere", "ALL/SJK"] {
            assert!(words.contains(needed), "{needed}");
        }
    }
}
