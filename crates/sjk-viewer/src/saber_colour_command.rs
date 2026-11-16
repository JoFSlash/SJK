//! JoF EJK's `saberColor` console command (`CG_Sabercolor_f`,
//! `cg_consolecmds.c:1154-1211`): a custom RGB blade colour for one blade or both.
//!
//! Each blade it sets becomes colour 6 (`SABER_RGB`) and takes its packed colour in
//! `cp_sbRGB1`/`cp_sbRGB2`; like JoF's `CG_SetSaberColorCvar` it keeps the hat or cape
//! carried after `color1`'s and `color2`'s digits.

use crate::console::ViewerConsole;
use sjk_client::{CosmeticSlot, SaberColor};

/// Completion and help entry.
pub(crate) const COMMANDS: &[(&str, &str)] = &[(
    "saberColor",
    "Custom RGB blade colour: saberColor <r> <g> <b>, saberColor <blade> <r> <g> <b>, or both blades' six values",
)];

/// Lowercase name the client's command dispatch matches.
pub(crate) const COMMAND: &str = "sabercolor";

const USAGE: [&str; 2] = [
    "Usage: ^3saberColor <red> <green> <blue>^7 (both blades), ^3saberColor <blade> <red> <green> <blue>^7,",
    "or ^3saberColor <red 1> <green 1> <blue 1> <red 2> <green 2> <blue 2>^7. Example: ^3saberColor 255 255 0^7",
];

/// Run `saberColor` with `args` (the words after the command).
pub(crate) fn run(console: &mut ViewerConsole, args: &[String]) -> Result<Vec<String>, String> {
    // `atoi` on every word, as JoF reads them.
    let value = |index: usize| atoi(&args[index]);
    let blades: Vec<(usize, [i64; 3])> = match args.len() {
        0..=2 => return Ok(USAGE.map(str::to_owned).to_vec()),
        3 => {
            let rgb = [value(0), value(1), value(2)];
            vec![(1, rgb), (2, rgb)]
        }
        4 => vec![(
            value(0).clamp(1, 2) as usize,
            [value(1), value(2), value(3)],
        )],
        _ => vec![
            (1, [value(0), value(1), value(2)]),
            (2, [value(3), value(4), value(5)]),
        ],
    };
    let mut lines = Vec::new();
    for (blade, rgb) in blades {
        let slot = CosmeticSlot::ALL[blade - 1];
        let piece = console
            .text_value(slot.cvar())
            .and_then(|current| sjk_client::split_color_value(current).1)
            .map(str::to_owned);
        console.set_cvar(
            slot.cvar(),
            &sjk_client::join_color_value(i64::from(SaberColor::Rgb.index()), piece.as_deref()),
        );
        console.set_cvar(&format!("cp_sbRGB{blade}"), &pack(rgb).to_string());
        lines.push(format!(
            "Blade {blade}: custom colour {} {} {}",
            rgb[0], rgb[1], rgb[2]
        ));
    }
    Ok(lines)
}

/// `red | ((green | (blue << 8)) << 8)` on C ints, as JoF packs it.
fn pack([red, green, blue]: [i64; 3]) -> i32 {
    let [red, green, blue] = [red, green, blue].map(|channel| channel as i32);
    red | ((green | (blue.wrapping_shl(8))).wrapping_shl(8))
}

/// C `atoi`: optional spaces and sign, then digits; anything else stops it.
fn atoi(text: &str) -> i64 {
    let text = text.trim_start();
    let (sign, digits) = match text.as_bytes().first() {
        Some(b'-') => (-1, &text[1..]),
        Some(b'+') => (1, &text[1..]),
        _ => (1, text),
    };
    let number = digits
        .bytes()
        .take_while(u8::is_ascii_digit)
        .fold(0_i64, |total, digit| {
            total
                .saturating_mul(10)
                .saturating_add(i64::from(digit - b'0'))
        });
    sign * number
}

impl crate::GpuState {
    /// `saberColor ...`.
    pub(crate) fn saber_colour_command(&mut self, args: &[String]) -> Result<Vec<String>, String> {
        let console = self.console.as_mut().ok_or("Console unavailable")?;
        run(console, args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_owned).collect()
    }

    #[test]
    fn both_blades_go_rgb_and_keep_the_hat_and_cape() {
        let directory = tempfile::tempdir().unwrap();
        let mut console = ViewerConsole::new(directory.path().join("config.cfg")).unwrap();
        console.set_cvar("color1", "4santahat");
        console.set_cvar("color2", "2");
        run(&mut console, &args("255 255 0")).unwrap();
        assert_eq!(console.text_value("color1"), Some("6santahat"));
        assert_eq!(console.text_value("color2"), Some("6"));
        assert_eq!(console.integer_cvar("cp_sbRGB1"), Some(0xFF_FF));
        assert_eq!(console.integer_cvar("cp_sbRGB2"), Some(0xFF_FF));
        // One blade, then each its own.
        run(&mut console, &args("2 255 90 120")).unwrap();
        assert_eq!(
            console.integer_cvar("cp_sbRGB2"),
            Some(255 | (90 << 8) | (120 << 16))
        );
        run(&mut console, &args("60 110 200 160 120 255")).unwrap();
        assert_eq!(
            console.integer_cvar("cp_sbRGB1"),
            Some(60 | (110 << 8) | (200 << 16))
        );
        assert_eq!(
            console.integer_cvar("cp_sbRGB2"),
            Some(160 | (120 << 8) | (255 << 16))
        );
        assert_eq!(console.text_value("color1"), Some("6santahat"));
        // Too few words print the usage and change nothing.
        assert_eq!(run(&mut console, &args("1 2")).unwrap().len(), 2);
    }

    #[test]
    fn words_read_as_c_atoi() {
        assert_eq!(atoi("255"), 255);
        assert_eq!(atoi(" -12x"), -12);
        assert_eq!(atoi("red"), 0);
    }
}
