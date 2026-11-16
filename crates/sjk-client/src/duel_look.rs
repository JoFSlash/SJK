//! How `CG_Player` colours players around a private duel
//! (`codemp/cgame/cg_players.c:10561-10633`, OpenJK).
//!
//! - The viewer is not in a duel: a living duellist other than the viewer is drawn
//!   flat dark grey (`shaderRGBA` 50 50 50 with `RF_RGB_TINT`), and the blue sight
//!   bubble later goes over it ([`crate::legacy_force_overlays`]). The dark body is
//!   what makes the bubble stand out.
//! - The viewer duels: everyone but the two duellists is drawn very dark (their
//!   colour a fifth, `RF_RGB_TINT`).
//!
//! Stock also lights the two duellists, while the viewer duels, with the Force shell
//! (`powerups/forceshell`, brighter the nearer they stand) and draws them flat in
//! their colour darkened by distance. JoF EJK (EternalJK `CG_Player`) does that only
//! with `cg_stylePlayer`'s shell bit, off by default, so by default the two look as
//! they are; SJK follows that default.

/// The viewer's side of a duel, from the snapshot's player state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DuelViewer {
    /// `ps.clientNum`.
    pub client: u16,
    /// `ps.duelInProgress`.
    pub duelling: bool,
    /// `ps.duelIndex`.
    pub opponent: u16,
    /// JoF EJK's ghost look (`cg_stylePlayer`'s `JAPRO_STYLE_VFXDUELERS`): a duellist
    /// is drawn as a blue figure in place of its body, so it is neither darkened nor
    /// bubbled.
    pub ghosts: bool,
}

/// One player as `CG_Player` sees it here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DuelActor {
    /// Entity (client) number.
    pub number: u16,
    /// `bolt1`: the player is in a duel.
    pub duelling: bool,
    /// `EF_DEAD`.
    pub dead: bool,
    /// The colour the body carries before the duel rules (`legs.shaderRGBA`).
    pub color: [u8; 3],
}

/// `RF_RGB_TINT` from the duel rules: the body of `actor`, seen by `viewer`, drawn
/// flat in this colour, if at all.
pub fn legacy_duel_tint(actor: DuelActor, viewer: DuelViewer) -> Option<[u8; 3]> {
    if viewer.duelling {
        // `legs.shaderRGBA[i] /= 5.0f`, stored back into a byte; the pair is left alone.
        return (actor.number != viewer.opponent && actor.number != viewer.client)
            .then(|| actor.color.map(|channel| (f32::from(channel) / 5.0) as u8));
    }
    (!viewer.ghosts && actor.duelling && !actor.dead && actor.number != viewer.client)
        .then_some([50; 3])
}

/// Whether `actor` is drawn as JoF EJK's blue duel figure for `viewer`, in place of
/// its body (EternalJK `CG_Player`, "Draw Duelers/Racers Style": a duellist seen by a
/// player outside every duel, the viewer never).
pub fn legacy_duel_ghost(actor: DuelActor, viewer: DuelViewer) -> bool {
    viewer.ghosts
        && !viewer.duelling
        && actor.duelling
        && !actor.dead
        && actor.number != viewer.client
}

#[cfg(test)]
mod tests {
    use super::*;

    fn actor(number: u16, duelling: bool) -> DuelActor {
        DuelActor {
            number,
            duelling,
            dead: false,
            color: [255; 3],
        }
    }

    fn viewer(opponent: Option<u16>) -> DuelViewer {
        DuelViewer {
            client: 0,
            duelling: opponent.is_some(),
            opponent: opponent.unwrap_or(0),
            ghosts: false,
        }
    }

    #[test]
    fn a_bystander_sees_a_duellist_dark_and_everyone_else_as_is() {
        let bystander = viewer(None);
        assert_eq!(legacy_duel_tint(actor(3, true), bystander), Some([50; 3]));
        assert_eq!(legacy_duel_tint(actor(4, false), bystander), None);
        let dead = DuelActor {
            dead: true,
            ..actor(3, true)
        };
        assert_eq!(legacy_duel_tint(dead, bystander), None);
        // The viewer's own body, seen in third person.
        assert_eq!(legacy_duel_tint(actor(0, false), bystander), None);
    }

    #[test]
    fn a_duellist_sees_bystanders_very_dark_and_the_pair_as_they_are() {
        let duelling = viewer(Some(5));
        assert_eq!(legacy_duel_tint(actor(7, false), duelling), Some([51; 3]));
        assert_eq!(legacy_duel_tint(actor(0, true), duelling), None);
        assert_eq!(legacy_duel_tint(actor(5, true), duelling), None);
    }

    #[test]
    fn with_ghosts_a_bystander_sees_a_duellist_as_a_figure_only() {
        let bystander = DuelViewer {
            ghosts: true,
            ..viewer(None)
        };
        assert!(legacy_duel_ghost(actor(3, true), bystander));
        assert_eq!(legacy_duel_tint(actor(3, true), bystander), None);
        assert!(!legacy_duel_ghost(actor(4, false), bystander));
        assert!(!legacy_duel_ghost(actor(0, true), bystander));
        // A viewer in a duel sees no ghosts; bystanders still go dark.
        let duelling = DuelViewer {
            ghosts: true,
            ..viewer(Some(5))
        };
        assert!(!legacy_duel_ghost(actor(5, true), duelling));
        assert_eq!(legacy_duel_tint(actor(7, false), duelling), Some([51; 3]));
        // Off, the stock look.
        assert!(!legacy_duel_ghost(actor(3, true), viewer(None)));
    }
}
