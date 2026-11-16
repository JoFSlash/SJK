//! Model-voice families from JoF EJK #287/#374, independent of resolved asset names.

/// Independently throttled player model voice family; all taunt variants share one slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum LegacyVoiceLine {
    /// Taunt, anger, victory, deflect, respect, meditate and gloat.
    Taunt,
    /// Leaving deep water.
    Gasp,
    /// Jump grunt.
    Jump,
    /// Roll grunt (not the body's impact).
    Roll,
    /// Landing grunt (not the fall impact).
    Land,
}

impl LegacyVoiceLine {
    /// Classify a server's custom-sound token, never an ordinary sound path.
    pub fn custom_token(token: &str) -> Option<Self> {
        let token = token.strip_prefix('*')?;
        const PREFIXES: [(&str, LegacyVoiceLine); 11] = [
            ("taunt", LegacyVoiceLine::Taunt),
            ("anger", LegacyVoiceLine::Taunt),
            ("gloat", LegacyVoiceLine::Taunt),
            ("victory", LegacyVoiceLine::Taunt),
            ("deflect", LegacyVoiceLine::Taunt),
            ("respect", LegacyVoiceLine::Taunt),
            ("meditate", LegacyVoiceLine::Taunt),
            ("gasp", LegacyVoiceLine::Gasp),
            ("jump", LegacyVoiceLine::Jump),
            ("roll", LegacyVoiceLine::Roll),
            ("land", LegacyVoiceLine::Land),
        ];
        PREFIXES.iter().find_map(|(prefix, family)| {
            token
                .get(..prefix.len())
                .filter(|head| head.eq_ignore_ascii_case(prefix))
                .map(|_| *family)
        })
    }

    /// Reference hold in milliseconds; taunts replace this fallback with decoded duration.
    pub const fn fallback_ms(self) -> u32 {
        match self {
            Self::Taunt => 2500,
            Self::Gasp => 2000,
            _ => 1000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tokens_share_taunt_family_without_classifying_regular_paths_or_pain() {
        for token in [
            "*taunt.wav",
            "*ANGER2.mp3",
            "*respect.wav",
            "*meditate.wav",
            "*gloat3.wav",
        ] {
            assert_eq!(
                LegacyVoiceLine::custom_token(token),
                Some(LegacyVoiceLine::Taunt)
            );
        }
        for token in [
            "sound/weapons/taunt.wav",
            "taunt.wav",
            "*pain25.wav",
            "*death1.wav",
            "",
        ] {
            assert_eq!(LegacyVoiceLine::custom_token(token), None);
        }
    }
}
