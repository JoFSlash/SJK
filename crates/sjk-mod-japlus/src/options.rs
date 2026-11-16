//! `japlus.serverconfig`: the JA+ client plugin's `serverconfig`.
//!
//! On JA+ the plugin answers `serverconfig` itself from the server's `jp_cinfo`
//! (EternalJK `codemp/cgame/cg_consolecmds.c:505-530`); jaPRO answers it on the
//! server, so it is forwarded there. The plugin's options (`cp_pluginDisable`) are
//! [`crate::plugin`]'s.

use sjk_mod::{Host, ServerKind};
use sjk_protocol::JaPlusCapabilities;

/// `jp_cinfo` options in EternalJK's listing order, with the macro-scan pair
/// and the three roll modes reported separately.
const SERVER_OPTIONS: [(u32, &str); 12] = [
    (JaPlusCapabilities::FLIP_KICK, "Flip kick"),
    (JaPlusCapabilities::YELLOW_DFA, "Improved yellow DFA"),
    (JaPlusCapabilities::HEAD_SLIDE, "Head slide"),
    (JaPlusCapabilities::SINGLE_PLAYER_ATTACKS, "SP attacks"),
    (JaPlusCapabilities::NEW_DFA, "New close-range DFA"),
    (JaPlusCapabilities::MODEL_SCALE, "Model scale"),
    (
        JaPlusCapabilities::DAMAGE_SPEED_SCALE,
        "Model scale damage speed scale",
    ),
    (JaPlusCapabilities::JK2_DFA, "JK2 red DFA"),
    (JaPlusCapabilities::NO_KATA, "Remove kata"),
    (JaPlusCapabilities::NO_AUTO_REPLIER, "Remove auto replier"),
    (JaPlusCapabilities::GLA_ANIMATIONS, "New GLA animations"),
    (JaPlusCapabilities::LEDGE_GRAB, "Ledge grab"),
];

fn yes_no(on: bool) -> &'static str {
    if on { "^2Yes" } else { "^1No" }
}

/// The JA+ server options `jp_cinfo` advertises, one line each.
fn server_option_lines(capabilities: JaPlusCapabilities) -> Vec<String> {
    let roll = if capabilities.contains(JaPlusCapabilities::FIX_ROLL_3) {
        "^23 (long, breakable)"
    } else if capabilities.contains(JaPlusCapabilities::FIX_ROLL_2) {
        "^22 (grip, chainable)"
    } else if capabilities.contains(JaPlusCapabilities::FIX_ROLL_1) {
        "^21 (grip while rolling)"
    } else {
        "^10 (stock)"
    };
    let mut lines = vec![
        format!("^5JA+ server options^3:^7 jp_cinfo {}", capabilities.0),
        format!(
            "^5Flip kick^3:^7 {}",
            yes_no(capabilities.contains(JaPlusCapabilities::FLIP_KICK))
        ),
        format!("^5Roll fix^3:^7 {roll}"),
    ];
    lines.extend(
        SERVER_OPTIONS[1..].iter().map(|(flag, label)| {
            format!("^5{label}^3:^7 {}", yes_no(capabilities.contains(*flag)))
        }),
    );
    let macro_scan = capabilities.contains(JaPlusCapabilities::MACRO_SCAN_1)
        || capabilities.contains(JaPlusCapabilities::MACRO_SCAN_2);
    lines.push(format!("^5Macro scan^3:^7 {}", yes_no(macro_scan)));
    lines.push(format!(
        "^5Alternate dimension^3:^7 {}",
        yes_no(capabilities.contains(JaPlusCapabilities::ALTERNATE_DIMENSION))
    ));
    lines
}

/// `japlus.serverconfig`: list JA+ options locally, ask jaPRO.
pub(crate) fn server_config(host: &mut dyn Host) -> Result<Vec<String>, String> {
    let server = host.server().ok_or("Not connected to a server.")?;
    match server.kind {
        ServerKind::JaPlus => {
            let bits = server
                .info("jp_cinfo")
                .and_then(|value| value.trim().parse::<i64>().ok())
                .unwrap_or(0);
            Ok(server_option_lines(JaPlusCapabilities(bits as u32)))
        }
        ServerKind::JaPro => {
            host.send("serverconfig")?;
            Ok(Vec::new())
        }
        ServerKind::BaseJka | ServerKind::Other => {
            Ok(vec!["^5Server is not running JA+ or jaPRO.".into()])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_the_local_ja_plus_server_options() {
        // JA+ Mod v2.4 B7 as configured on the test server: 0x300d3.
        let lines = server_option_lines(JaPlusCapabilities(196_819));
        assert_eq!(lines[0], "^5JA+ server options^3:^7 jp_cinfo 196819");
        assert_eq!(lines[1], "^5Flip kick^3:^7 ^2Yes");
        assert_eq!(lines[2], "^5Roll fix^3:^7 ^21 (grip while rolling)");
        let find = |label: &str| {
            lines
                .iter()
                .find(|line| line.starts_with(&format!("^5{label}^3")))
                .unwrap()
                .clone()
        };
        assert!(find("Improved yellow DFA").ends_with("^2Yes"));
        assert!(find("Head slide").ends_with("^1No"));
        assert!(find("SP attacks").ends_with("^2Yes"));
        assert!(find("New close-range DFA").ends_with("^2Yes"));
        assert!(find("Remove kata").ends_with("^1No"));
        assert!(find("Ledge grab").ends_with("^2Yes"));
        assert!(find("Macro scan").ends_with("^1No"));
        assert!(find("Alternate dimension").ends_with("^2Yes"));
        // One line per option plus the header.
        assert_eq!(lines.len(), 1 + 2 + 11 + 2);
    }

    #[test]
    fn highest_roll_mode_wins() {
        let lines = server_option_lines(JaPlusCapabilities(
            JaPlusCapabilities::FIX_ROLL_1 | JaPlusCapabilities::FIX_ROLL_3,
        ));
        assert_eq!(lines[2], "^5Roll fix^3:^7 ^23 (long, breakable)");
        assert_eq!(
            server_option_lines(JaPlusCapabilities(0))[2],
            "^5Roll fix^3:^7 ^10 (stock)"
        );
    }
}
