//! Default presentation of the two extra single-saber styles.
//!
//! JoF EJK #47 (`1fbe4678`, `codemp/game/bg_pmove.c::PM_GetSaberStance`)
//! selects Tavion's high parry and Desann's own standing pose. Apply that
//! choice only to the stock idle clips, after prediction, so a remote server's
//! movement and attack timers retain their original animation inputs.

struct IdleStance {
    style: u8,
    stock: u16,
    presented: u16,
}

/// One entry per changed style, using protocol-26 animation ordinals.
/// Tavion (5): `BOTH_SABERFAST_STANCE` -> `BOTH_P1_S1_T_`.
/// Desann (4): `BOTH_STAND2` -> `BOTH_STAND9`.
const IDLE_STANCES: [IdleStance; 2] = [
    IdleStance {
        style: 5,
        stock: 850,
        presented: 665,
    },
    IdleStance {
        style: 4,
        stock: 917,
        presented: 942,
    },
];

pub(crate) fn present(
    legs: u16,
    torso: u16,
    weapon: u8,
    in_flight: bool,
    holstered: u8,
    style: u8,
) -> (u16, u16) {
    const WP_SABER: u8 = 3;
    if weapon != WP_SABER || in_flight || holstered != 0 {
        return (legs, torso);
    }
    let Some(stance) = IDLE_STANCES.iter().find(|stance| stance.style == style) else {
        return (legs, torso);
    };
    let map = |clip| {
        if clip == stance.stock {
            stance.presented
        } else {
            clip
        }
    };
    (map(legs), map(torso))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_ordinals_and_default_poses() {
        for (style, stock, expected, name) in
            [(5, 850, 665, "BOTH_P1_S1_T_"), (4, 917, 942, "BOTH_STAND9")]
        {
            assert_eq!(crate::legacy_animation_name(expected), Some(name));
            assert_eq!(
                present(stock, stock, 3, false, 0, style),
                (expected as u16, expected as u16)
            );
            // A server already sending the EJK poses needs no further remap.
            assert_eq!(
                present(expected as u16, expected as u16, 3, false, 0, style),
                (expected as u16, expected as u16)
            );
        }
    }

    #[test]
    fn only_the_matching_idle_track_changes() {
        // Walking legs, attacking torso and a custom ready pose stay intact.
        assert_eq!(present(1102, 850, 3, false, 0, 5), (1102, 665));
        assert_eq!(present(917, 114, 3, false, 0, 4), (942, 114));
        assert_eq!(present(915, 915, 3, false, 0, 4), (915, 915));
        for style in [0, 1, 2, 3, 6, 7, 255] {
            assert_eq!(present(850, 917, 3, false, 0, style), (850, 917));
        }
    }

    #[test]
    fn guns_thrown_and_unlit_sabers_keep_their_pose() {
        for style in [4, 5] {
            for weapon in [0, 2, 5, 16] {
                assert_eq!(present(850, 917, weapon, false, 0, style), (850, 917));
            }
            assert_eq!(present(850, 917, 3, true, 0, style), (850, 917));
            for holstered in [1, 2] {
                assert_eq!(present(850, 917, 3, false, holstered, style), (850, 917));
            }
        }
    }
}
