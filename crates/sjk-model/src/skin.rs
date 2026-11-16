//! Ghoul2 `.skin` files: which shader draws each named mesh surface.
//!
//! Skins are read the way rd-vanilla's `RE_RegisterIndividualSkin` reads them
//! (`codemp/rd-vanilla/tr_skin.cpp`): a stream of `CommaParse` tokens taken in
//! surface/shader pairs, so line breaks, missing commas, comments and stray
//! text change nothing about what loads. No file is ever refused; a file with
//! no usable pair gives an empty skin, which a caller treats as rd-vanilla
//! treats skin handle 0.
//!
//! A surface named more than once takes its shader from its first entry, as
//! `RenderSurfaces` (`tr_ghoul2.cpp`) stops at the first match, but whether it
//! is shown from its last: `G2_SetSurfaceOnOffFromSkin` (`G2_surfaces.cpp`)
//! walks every entry in order, an `*off` one switching the surface off and any
//! other switching it back on ([`Skin::hides`]).

use std::collections::{HashMap, HashSet};

/// Skin shader name that hides a surface instead of texturing it.
pub const SKIN_SHADER_OFF: &str = "*off";

/// Surfaces one skin can name, counted over all parts (`skin_t::surfaces[128]`).
const MAX_SKIN_SURFACES: usize = 128;

/// Longest surface name `RE_RegisterIndividualSkin` keeps (`MAX_QPATH` - 1).
const MAX_SURFACE_NAME: usize = 63;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Skin {
    /// Lower-case surface name without `_off` to lower-case shader name.
    mappings: HashMap<String, String>,
    /// Surface entries read, duplicates included, as `skin_t::numSurfaces` counts them.
    entries: usize,
    /// Lower-case surfaces without `_off` whose last entry is `*off`.
    hidden: HashSet<String>,
}

impl Skin {
    /// Read one `.skin` file. It never fails: whatever the tokenizer finds is
    /// what rd-vanilla would register.
    pub fn parse(bytes: &[u8]) -> Self {
        let mut skin = Self::default();
        skin.append(bytes);
        skin
    }

    /// Add one more file to this skin, as `RE_RegisterSkin` adds the torso and
    /// lower parts of a `head|torso|lower` skin to the head's. An entry for a
    /// surface already named keeps the earlier shader: the renderer takes the
    /// first match (`RenderSurfaces`, `tr_ghoul2.cpp`). It still switches the
    /// surface off (`*off`) or back on (any other shader), as
    /// `G2_SetSurfaceOnOffFromSkin` applies every entry in turn.
    pub fn append(&mut self, bytes: &[u8]) {
        let mut tokens = CommaParser { bytes, position: 0 };
        loop {
            let token = tokens.next_token();
            if token.is_empty() {
                break;
            }
            if tokens.peek() == b',' {
                tokens.position += 1;
            }
            // id-style tag entries; the token after one is read as a surface.
            if token.starts_with(b"tag_") {
                continue;
            }
            let mut surface = text(&token[..token.len().min(MAX_SURFACE_NAME)]);
            surface.make_ascii_lowercase();
            let shader = tokens.next_token();
            if let Some(stripped) = surface.strip_suffix("_off") {
                if shader == SKIN_SHADER_OFF.as_bytes() {
                    continue;
                }
                surface.truncate(stripped.len());
            }
            if self.entries >= MAX_SKIN_SURFACES {
                break;
            }
            self.entries += 1;
            let shader = text(&shader).replace('\\', "/").to_ascii_lowercase();
            if shader == SKIN_SHADER_OFF {
                self.hidden.insert(surface.clone());
            } else {
                self.hidden.remove(&surface);
            }
            self.mappings.entry(surface).or_insert(shader);
        }
    }

    /// The shader for a mesh surface, matched without case and without a
    /// trailing `_off`, which `R_LoadMDXM` strips from surface names: its first
    /// entry's. That is `*off` itself for a surface first switched off and then
    /// on again, which retail draws with the shader `R_FindShader("*off")` makes:
    /// no script and no image, so the default image.
    pub fn shader(&self, surface: &str) -> Option<&str> {
        self.mappings.get(&key(surface)).map(String::as_str)
    }

    /// Whether the skin switches the surface off: its last entry is `*off`
    /// (`G2_SetSurfaceOnOffFromSkin`). A surface the model itself has off (an
    /// `_off` cap) stays off whatever the skin says; that is the model's flag.
    pub fn hides(&self, surface: &str) -> bool {
        self.hidden.contains(&key(surface))
    }

    /// Distinct surfaces the skin names.
    pub fn len(&self) -> usize {
        self.mappings.len()
    }

    /// True when the skin names no surface; rd-vanilla then returns skin
    /// handle 0 ("never let a skin have 0 shaders").
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }
}

/// A surface name as the skin stores it: lower case, without a trailing `_off`.
fn key(surface: &str) -> String {
    let mut surface = surface.to_ascii_lowercase();
    if surface.ends_with("_off") {
        surface.truncate(surface.len() - 4);
    }
    surface
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// `CommaParse` (`tr_skin.cpp`): whitespace, `//` and `/* */` comments separate
/// tokens; a token is a quoted string or runs to whitespace or a comma.
struct CommaParser<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl CommaParser<'_> {
    fn at(&self, index: usize) -> u8 {
        // The C string ends at its first NUL as well as at the buffer's end.
        self.bytes.get(index).copied().unwrap_or(0)
    }

    fn peek(&self) -> u8 {
        self.at(self.position)
    }

    fn next_token(&mut self) -> Vec<u8> {
        loop {
            while self.peek() != 0 && self.peek() <= b' ' {
                self.position += 1;
            }
            if self.peek() == b'/' && self.at(self.position + 1) == b'/' {
                while self.peek() != 0 && self.peek() != b'\n' {
                    self.position += 1;
                }
            } else if self.peek() == b'/' && self.at(self.position + 1) == b'*' {
                while self.peek() != 0
                    && !(self.peek() == b'*' && self.at(self.position + 1) == b'/')
                {
                    self.position += 1;
                }
                if self.peek() != 0 {
                    self.position += 2;
                }
            } else {
                break;
            }
        }
        let mut token = Vec::new();
        if self.peek() == 0 {
            return token;
        }
        if self.peek() == b'"' {
            self.position += 1;
            loop {
                let character = self.peek();
                if character != 0 {
                    self.position += 1;
                }
                if character == b'"' || character == 0 {
                    return token;
                }
                token.push(character);
            }
        }
        loop {
            token.push(self.peek());
            self.position += 1;
            // `c > 32` on a signed char: a byte from 0x80 up also ends the word.
            let next = self.peek();
            if next <= b' ' || next == b',' || next >= 0x80 {
                return token;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SKIN_SHADER_OFF, Skin};

    #[test]
    fn plain_lines_map_surfaces_to_shaders() {
        let skin =
            Skin::parse(b"head,models/players/a/head\r\nTorso,Models\\Players\\A\\Torso.tga\r\n");
        assert_eq!(skin.shader("head"), Some("models/players/a/head"));
        assert_eq!(skin.shader("TORSO"), Some("models/players/a/torso.tga"));
        assert_eq!(skin.len(), 2);
    }

    #[test]
    fn comments_blank_lines_and_spacing_are_ignored() {
        let skin = Skin::parse(
            b"// header\n\n  head,  shader/a // trailing\n/* block\n torso,nope */ torso,shader/b\n",
        );
        assert_eq!(skin.shader("head"), Some("shader/a"));
        assert_eq!(skin.shader("torso"), Some("shader/b"));
        assert_eq!(skin.len(), 2);
    }

    #[test]
    fn a_comma_after_a_space_is_read_as_the_shader() {
        // CommaParse only skips a comma right after the surface name, so
        // rd-vanilla gives `head` the shader "," and pairs the rest anew.
        let skin = Skin::parse(b"head , shader/a\ntorso,shader/b\n");
        assert_eq!(skin.shader("head"), Some(","));
        assert_eq!(skin.shader("shader/a"), Some("torso"));
    }

    #[test]
    fn a_missing_comma_still_pairs_tokens() {
        let skin = Skin::parse(b"head shader/a\ntorso\tshader/b");
        assert_eq!(skin.shader("head"), Some("shader/a"));
        assert_eq!(skin.shader("torso"), Some("shader/b"));
    }

    #[test]
    fn stray_text_is_read_as_pairs_instead_of_refused() {
        // A three-part pack ships this as its lower_a1.skin.
        let skin = Skin::parse(b"There's nothing here!");
        assert_eq!(skin.shader("there's"), Some("nothing"));
        assert_eq!(skin.shader("here!"), Some(""));
        assert!(skin.shader("hips").is_none());
    }

    #[test]
    fn quoted_tokens_keep_their_spaces() {
        let skin = Skin::parse(b"\"head\",\"models/my skins/head\"");
        assert_eq!(skin.shader("head"), Some("models/my skins/head"));
    }

    #[test]
    fn tag_entries_are_skipped() {
        let skin = Skin::parse(b"tag_head,\ntag_weapon,\nhead,shader/a\n");
        assert_eq!(skin.shader("head"), Some("shader/a"));
        assert_eq!(skin.len(), 1);
    }

    #[test]
    fn off_suffixes_match_surfaces_like_the_renderer() {
        let skin = Skin::parse(b"head_cap_torso_off,caps\nhips_cap_l_leg_off,*off\n");
        // R_LoadMDXM strips _off from surface names; the skin entry is stored without it.
        assert_eq!(skin.shader("head_cap_torso_off"), Some("caps"));
        assert_eq!(skin.shader("head_cap_torso"), Some("caps"));
        // An `_off` surface switched off again is dropped as a double off.
        assert!(skin.shader("hips_cap_l_leg_off").is_none());
        assert_eq!(skin.len(), 1);
    }

    #[test]
    fn star_off_hides_ordinary_surfaces() {
        let skin = Skin::parse(b"helmet,*off");
        assert_eq!(skin.shader("helmet"), Some(SKIN_SHADER_OFF));
        assert!(skin.hides("helmet"));
        assert!(skin.hides("HELMET_off"));
        assert!(!skin.hides("head"));
    }

    #[test]
    fn the_last_entry_switches_a_surface_on_or_off() {
        // A community skin (Yuna's) names every surface with its texture, then
        // switches the outfit's spare parts off further down.
        let skin = Skin::parse(b"hips_skirt,skirt\ntorso_sum,x_torso\nhips_skirt,*off\n");
        assert!(skin.hides("hips_skirt"));
        assert_eq!(skin.shader("hips_skirt"), Some("skirt"));
        assert!(!skin.hides("torso_sum"));
        // Switched off, then on again by its `_off` name (Zuvio's neck cap): shown,
        // with the first entry's shader, `*off`, which retail draws as its default image.
        let skin = Skin::parse(b"torso_cap_head,*off\ntorso_cap_head_off,caps\n");
        assert!(!skin.hides("torso_cap_head"));
        assert_eq!(skin.shader("torso_cap_head"), Some(SKIN_SHADER_OFF));
        // Across the parts of a three-part skin as well.
        let mut skin = Skin::parse(b"cloak,cloak\n");
        skin.append(b"cloak,*off\n");
        assert!(skin.hides("cloak"));
    }

    #[test]
    fn the_first_entry_for_a_surface_wins() {
        let mut skin = Skin::parse(b"head,first\nhead,second\n");
        assert_eq!(skin.shader("head"), Some("first"));
        skin.append(b"head,third\ntorso,body\n");
        assert_eq!(skin.shader("head"), Some("first"));
        assert_eq!(skin.shader("torso"), Some("body"));
    }

    #[test]
    fn bytes_outside_utf8_do_not_refuse_the_file() {
        let skin = Skin::parse(b"// caf\xe9\nhead,shader/a\n");
        assert_eq!(skin.shader("head"), Some("shader/a"));
        // CommaParse compares a signed char: a high byte ends a word.
        let skin = Skin::parse(b"head,ab\xe9cd\n");
        assert_eq!(skin.shader("head"), Some("ab"));
    }

    #[test]
    fn an_empty_or_comment_only_file_names_nothing() {
        assert!(Skin::parse(b"").is_empty());
        assert!(Skin::parse(b"// nothing\r\n\r\n").is_empty());
    }

    #[test]
    fn a_skin_stops_at_128_entries_over_all_parts() {
        let lines = |prefix: &str, count: usize| {
            (0..count)
                .map(|index| format!("{prefix}{index},shader\n"))
                .collect::<String>()
        };
        let mut skin = Skin::parse(lines("head", 100).as_bytes());
        skin.append(lines("torso", 40).as_bytes());
        assert_eq!(skin.len(), 128);
        assert!(skin.shader("torso27").is_some());
        assert!(skin.shader("torso28").is_none());
    }
}
