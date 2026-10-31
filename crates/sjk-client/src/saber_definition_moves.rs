//! Movement fields from codemp `WP_SaberParseParms` (OpenJK 4d0dfaf1).
use sjk_game_jka::saber_info::SaberInfo;

#[path = "saber_definition_move_names.rs"]
mod names;

/// `saberFlags` authored by the corresponding nonzero `.sab` keywords.
const FLAGS: &[(&str, u32)] = &[
    ("singlebladethrowable", 1 << 5),
    ("nopullattack", 1 << 10),
    ("nobackattack", 1 << 11),
    ("nostabdown", 1 << 12),
    ("nowallruns", 1 << 13),
    ("nowallflips", 1 << 14),
    ("nowallgrab", 1 << 15),
    ("norolls", 1 << 16),
    ("noflips", 1 << 17),
    ("nocartwheels", 1 << 18),
    ("nokicks", 1 << 19),
    ("nomirrorattacks", 1 << 20),
    ("norollstab", 1 << 21),
];

pub(super) fn parse(info: &mut SaberInfo, key: &str, value: &str) {
    if key.eq_ignore_ascii_case("throwable") {
        if value.parse::<i32>() == Ok(0) {
            info.flags |= 1 << 1;
        }
        return;
    }
    if let Some((_, flag)) = FLAGS
        .iter()
        .find(|(name, _)| key.eq_ignore_ascii_case(name))
    {
        if value.parse::<i32>().unwrap_or(0) != 0 {
            info.flags |= flag;
        }
        return;
    }
    let movement = || {
        names::MOVES
            .iter()
            .find(|(name, _)| value.eq_ignore_ascii_case(name))
            .map_or(-1, |(_, id)| *id)
    };
    let animation = || {
        sjk_game_jka::legacy_animation::NAMES
            .iter()
            .position(|name| value.eq_ignore_ascii_case(name))
            .map(|id| id as i32)
    };
    match key.to_ascii_lowercase().as_str() {
        "katamove" => info.kata_move = movement(),
        "lungeatkmove" => info.lunge_move = movement(),
        "jumpatkupmove" => info.jump_up_move = movement(),
        "jumpatkfwdmove" => info.jump_forward_move = movement(),
        "jumpatkbackmove" => info.jump_back_move = movement(),
        "jumpatkrightmove" => info.jump_right_move = movement(),
        "jumpatkleftmove" => info.jump_left_move = movement(),
        "readyanim" => {
            if let Some(id) = animation() {
                info.ready_anim = id;
            }
        }
        "drawanim" => {
            if let Some(id) = animation() {
                info.draw_anim = id;
            }
        }
        "putawayanim" => {
            if let Some(id) = animation() {
                info.putaway_anim = id;
            }
        }
        "lockbonus" => {
            if let Ok(bonus) = value.parse() {
                info.lock_bonus = bonus;
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authored_move_tokens_and_animations_match_codemp() {
        let mut info = SaberInfo::STOCK;
        parse(&mut info, "NoWallRuns", "1");
        parse(&mut info, "noFlips", "1");
        parse(&mut info, "noFlips", "0"); // Set-only parser flags.
        parse(&mut info, "kataMove", "LS_NONE");
        parse(&mut info, "lungeAtkMove", "ls_a_lunge");
        parse(&mut info, "jumpAtkFwdMove", "LS_HILT_BASH");
        parse(&mut info, "jumpAtkBackMove", "unrecognised");
        parse(&mut info, "readyAnim", "BOTH_STAND1");
        parse(&mut info, "drawAnim", "BOTH_STAND2");
        parse(&mut info, "drawAnim", "unrecognised");
        assert_eq!(info.flags, (1 << 13) | (1 << 17));
        assert_eq!(
            (
                info.kata_move,
                info.lunge_move,
                info.jump_forward_move,
                info.jump_back_move
            ),
            (0, 15, 61, -1)
        );
        assert_eq!((info.ready_anim, info.draw_anim), (915, 917));
        parse(&mut info, "throwable", "invalid");
        assert_eq!(info.flags & (1 << 1), 0);
        parse(&mut info, "throwable", "0");
        parse(&mut info, "throwable", "1");
        assert_ne!(info.flags & (1 << 1), 0);
        assert!(!names::MOVES.iter().any(|(name, _)| *name == "LS_READY"));
    }
}
