//! Tags, SJK's toy (`toy_tag`; Sol's idea, 27/10/2026): a player sticks a premade
//! picture on the wall or floor they look at, as TF2's sprays, and every SJK player on
//! the same map sees it there (`docs/client.md`, Tag).
//!
//! - **The tags** are unlocks of the hub (`PROTOCOL.md`, "Unlocks"): [`CATALOGUE`] here,
//!   the same list there. Three are free to everyone ([`Tag::free`]); the others drop
//!   from holocrons, each with a tier as shaders have. Sol picks every one: there are
//!   no uploads (his decision, 27/10/2026); new tags come from rounds on Discord.
//! - **Choosing:** [`CVAR`] holds the chosen tag (the Collection's Tags tab sets it);
//!   one the player does not hold is not stuck.
//! - **Sticking:** [`COMMAND`] traces the crosshair [`REACH`] units into the world and
//!   sticks the chosen tag where it lands, upright on a wall and facing the player on a
//!   floor ([`spin_for`]); `toy_tag clear` takes it off. One tag a player: a new one
//!   replaces the last. At most one every [`GAP`].
//! - **Others:** the tag travels in the player's look (`looks.rs`, `PROTOCOL.md`,
//!   "Looks"), so players who come later see it too. It stays until the player sticks
//!   another, clears it, changes map or server, or leaves. [`SHOW_CVAR`] 0 hides other
//!   players' tags on this PC; a muted or blocked player's never show.
//!
//! The pictures are art in the hub's `sjk_tags` pack (`gfx/tags/<id>.png`, with their
//! decal shaders in `shaders/sjk_tags.shader`); the client draws a tag as a lasting decal
//! of its own (`decal_store.rs`, [`crate::decal_store::DecalStore::set_tag`]).

use crate::unlockables::Rarity;
use glam::Vec3;
use std::time::{Duration, Instant};

/// The toy's console command: stick the chosen tag, or `toy_tag clear` take it off.
pub(crate) const COMMAND: &str = "toy_tag";
/// Archived: the chosen tag's id.
pub(crate) const CVAR: &str = "cg_tag";
/// The tag chosen before the player chose one.
pub(crate) const DEFAULT: &str = "tag_sjk";
/// Archived: other players' tags are drawn (0 hides them on this PC; the own still
/// shows and is still sent).
pub(crate) const SHOW_CVAR: &str = "cg_toyTags";
/// The toy's icon (quick wheel, Collection, key bindings).
pub(crate) const ICON: &str = "gfx/sjk/toy_tag.png";
/// How far the crosshair reaches for a wall, in units.
pub(crate) const REACH: f32 = 128.0;
/// Half a tag's width, in units: a tag is a bit less than a player is tall.
pub(crate) const RADIUS: f32 = 28.0;
/// Shortest time between two tags stuck.
pub(crate) const GAP: Duration = Duration::from_secs(10);

/// One tag of the catalogue.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Tag {
    /// Its unlock id at the hub and in looks.
    pub(crate) id: &'static str,
    pub(crate) name: &'static str,
    pub(crate) tier: Rarity,
    /// Every player holds it, without a grant (the starter set).
    pub(crate) free: bool,
}

impl Tag {
    /// Its picture and decal shader (`gfx/tags/<id>`).
    pub(crate) fn shader(&self) -> String {
        format!("gfx/tags/{}", self.id)
    }

    /// How a player gets it, in a line.
    pub(crate) fn how_to_get(&self) -> &'static str {
        if self.free {
            "Everyone's, from the start."
        } else {
            crate::unlockables::FROM_HOLOCRONS
        }
    }
}

const fn tag(id: &'static str, name: &'static str, tier: Rarity, free: bool) -> Tag {
    Tag {
        id,
        name,
        tier,
        free,
    }
}

/// Every tag, in the hub's order (`src/unlocks.rs`). Ids never change meaning; a new
/// tag (one Sol picked from a Discord round) takes a new id here and at the hub, and its
/// picture goes in the pack.
pub(crate) const CATALOGUE: &[Tag] = &[
    tag("tag_sjk", "SJK tag", Rarity::Common, true),
    tag("tag_gg", "GG tag", Rarity::Common, true),
    tag("tag_fox", "Fox tag", Rarity::Common, true),
    tag(
        "tag_crossed_sabers",
        "Crossed Sabers tag",
        Rarity::Common,
        false,
    ),
    tag("tag_was_here", "Was Here tag", Rarity::Common, false),
    tag("tag_this_way", "This Way tag", Rarity::Common, false),
    tag("tag_kyber", "Kyber tag", Rarity::Uncommon, false),
    tag(
        "tag_droid_doodle",
        "Droid Doodle tag",
        Rarity::Uncommon,
        false,
    ),
    tag("tag_rage_quit", "Rage Quit tag", Rarity::Uncommon, false),
    tag("tag_gonk", "Gonk tag", Rarity::Uncommon, false),
    tag("tag_holocron", "Holocron tag", Rarity::Rare, false),
    tag("tag_ez", "EZ tag", Rarity::Rare, false),
    tag("tag_saber_skull", "Saber Skull tag", Rarity::Rare, false),
    tag("tag_geyon", "Geyon tag", Rarity::Rare, false),
    tag("tag_force_push", "Force Push tag", Rarity::Legendary, false),
    tag("tag_dark_mask", "Dark Mask tag", Rarity::Legendary, false),
    tag("tag_solar_fox", "Solar Fox tag", Rarity::Mythical, false),
];

/// The tag called `id`, if this client knows it.
pub(crate) fn get(id: &str) -> Option<&'static Tag> {
    CATALOGUE.iter().find(|tag| tag.id == id)
}

/// Whether the player may stick tag `id`: a free one, or one `owned` says the hub
/// profile lists.
pub(crate) fn may_stick(id: &str, owned: impl Fn(&str) -> bool) -> bool {
    get(id).is_some_and(|tag| tag.free || owned(tag.id))
}

/// The turn about `normal`, in degrees from 0 up to 360, that stands a tag's top along
/// `up` (a direction in its plane): on a wall the world's up, on a floor or a ceiling
/// the way the player looks ([`up_for`]). Every client lays the picture out from the
/// normal and this turn alone (`decal_marks::texture_axes`), so all see it the same.
pub(crate) fn spin_for(normal: Vec3, up: Vec3) -> f32 {
    let normal = normal.normalize_or(Vec3::Z);
    let seed = crate::decal_marks::texture_axes(normal, 0.0)[2];
    // The picture's rows run down its texture's `t`, the third axis: its top is that
    // axis turned round.
    let want = -(up - normal * up.dot(normal)).normalize_or(seed);
    let angle = normal
        .dot(seed.cross(want))
        .atan2(seed.dot(want))
        .to_degrees();
    let spin = angle.rem_euclid(360.0);
    if spin >= 360.0 { 0.0 } else { spin }
}

/// The way a tag's top points on a surface facing `normal`, for a player looking along
/// `forward`: the world's up on a wall, the look's way on a floor or a ceiling.
pub(crate) fn up_for(normal: Vec3, forward: Vec3) -> Vec3 {
    if normal.z.abs() < 0.7 {
        Vec3::Z
    } else {
        forward
    }
}

/// The player's own tag and when the last one went up.
#[derive(Default)]
pub(crate) struct Tags {
    /// The tag stuck, with the server it is on.
    own: Option<(std::net::SocketAddr, sjk_identity::PlacedTag)>,
    last: Option<Instant>,
    /// The own tag changed since the look last went: send it now.
    changed: bool,
    /// The decal store was handed tags in a game: leaving it clears them once.
    pub(crate) in_game: bool,
}

impl Tags {
    /// Whether one may go up at `now`; `Err` the time still to wait.
    pub(crate) fn may_stick(&self, now: Instant) -> Result<(), Duration> {
        match self.last {
            Some(last) if now.saturating_duration_since(last) < GAP => {
                Err(GAP - now.saturating_duration_since(last))
            }
            _ => Ok(()),
        }
    }

    /// Stick `tag` on `server`, replacing the last.
    pub(crate) fn stick(
        &mut self,
        server: std::net::SocketAddr,
        tag: sjk_identity::PlacedTag,
        now: Instant,
    ) {
        self.own = Some((server, tag));
        self.last = Some(now);
        self.changed = true;
    }

    /// Take the own tag off; true when there was one.
    pub(crate) fn clear(&mut self) -> bool {
        let had = self.own.take().is_some();
        self.changed |= had;
        had
    }

    /// Whether the own tag changed since last asked.
    pub(crate) fn take_changed(&mut self) -> bool {
        std::mem::take(&mut self.changed)
    }

    /// The own tag while it stands on `server` and `map` and the player still may stick
    /// it (`owned`): any other, it goes (a tag taken back, another server or map).
    pub(crate) fn own(
        &mut self,
        server: Option<std::net::SocketAddr>,
        map: Option<&str>,
        owned: impl Fn(&str) -> bool,
    ) -> Option<&sjk_identity::PlacedTag> {
        let stands = self.own.as_ref().is_some_and(|(on, tag)| {
            Some(*on) == server && Some(tag.map.as_str()) == map && may_stick(&tag.id, &owned)
        });
        if !stands {
            self.own = None;
        }
        self.own.as_ref().map(|(_, tag)| tag)
    }
}

/// The tags' decal shaders, one to each tag of [`CATALOGUE`] (made by
/// `scripts/tag_assets.py`, which reads the catalogue): the pack's picture blended over
/// the wall. Bundled, so a pack of pictures alone makes a tag.
const SHADERS: &[u8] = include_bytes!("../assets/tags/tags.shader");

/// Mount the toy's icon and the tags' decal shaders.
pub(crate) fn mount(vfs: &mut sjk_vfs::VirtualFileSystem) -> Result<(), sjk_vfs::VfsError> {
    vfs.mount_memory(
        "SJK Tag",
        [
            (
                ICON,
                include_bytes!("../assets/tags/toy_tag.png").as_slice(),
            ),
            ("shaders/sjk_tags.shader", SHADERS),
        ],
    )?;
    Ok(())
}

/// The decal a placed tag draws, when this client knows the tag.
pub(crate) fn decal(tag: &sjk_identity::PlacedTag) -> Option<crate::decal_marks::DecalRequest> {
    let known = get(&tag.id)?;
    Some(crate::decal_marks::DecalRequest {
        origin: Vec3::from_array(tag.at),
        direction: Vec3::from_array(tag.normal),
        orientation: tag.spin,
        color: [1.0; 4],
        radius: RADIUS,
        shader: known.shader().into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_tag_words_and_three_are_free() {
        for (index, tag) in CATALOGUE.iter().enumerate() {
            assert!(tag.id.starts_with("tag_"), "{}", tag.id);
            assert!(tag.id.len() <= 24, "{}", tag.id);
            assert_eq!(get(tag.id), Some(tag));
            assert!(
                CATALOGUE[..index].iter().all(|other| other.id != tag.id),
                "{}",
                tag.id
            );
        }
        let free: Vec<_> = CATALOGUE
            .iter()
            .filter(|tag| tag.free)
            .map(|tag| tag.id)
            .collect();
        assert_eq!(free, ["tag_sjk", "tag_gg", "tag_fox"]);
        assert!(get(DEFAULT).is_some_and(|tag| tag.free));
    }

    /// `scripts/tag_assets.py` was run since the catalogue last changed.
    #[test]
    fn every_tag_has_its_bundled_decal_shader() {
        let shaders = std::str::from_utf8(SHADERS).unwrap();
        let named: Vec<&str> = shaders
            .lines()
            .filter(|line| line.starts_with("gfx/tags/"))
            .collect();
        let wanted: Vec<String> = CATALOGUE.iter().map(Tag::shader).collect();
        assert_eq!(named, wanted);
    }

    #[test]
    fn a_free_tag_needs_no_grant_and_another_does() {
        assert!(may_stick("tag_gg", |_| false));
        assert!(!may_stick("tag_ez", |_| false));
        assert!(may_stick("tag_ez", |id| id == "tag_ez"));
        assert!(!may_stick("tag_nope", |_| true));
    }

    /// The tag's top, laid out from the normal and the turn alone, points where it
    /// should: up on walls whichever way they face, ahead on a floor.
    #[test]
    fn a_tag_stands_upright_on_walls_and_faces_the_player_on_floors() {
        let top = |normal: Vec3, spin: f32| -crate::decal_marks::texture_axes(normal, spin)[2];
        for normal in [
            Vec3::X,
            -Vec3::X,
            Vec3::Y,
            -Vec3::Y,
            Vec3::new(1.0, 1.0, 0.3).normalize(),
        ] {
            let spin = spin_for(normal, up_for(normal, Vec3::X));
            assert!((0.0..360.0).contains(&spin));
            let up = top(normal, spin);
            let want = (Vec3::Z - normal * normal.z).normalize();
            assert!(up.dot(want) > 0.999, "{normal} {up}");
            // Read from the front, the picture's left-to-right runs to the viewer's right.
            let right = (-normal).cross(want);
            assert!(crate::decal_marks::texture_axes(normal, spin)[1].dot(right) > 0.999);
        }
        for forward in [Vec3::X, Vec3::Y, Vec3::new(-1.0, 0.5, -0.4).normalize()] {
            let spin = spin_for(Vec3::Z, up_for(Vec3::Z, forward));
            let want = Vec3::new(forward.x, forward.y, 0.0).normalize();
            assert!(top(Vec3::Z, spin).dot(want) > 0.999, "{forward}");
        }
    }

    #[test]
    fn the_own_tag_goes_with_its_server_map_or_grant_and_waits_its_gap() {
        let server: std::net::SocketAddr = "1.2.3.4:29070".parse().unwrap();
        let other: std::net::SocketAddr = "1.2.3.4:29071".parse().unwrap();
        let placed = |id: &str| sjk_identity::PlacedTag {
            id: id.to_owned(),
            map: "mp/ffa3".to_owned(),
            at: [0.0; 3],
            normal: [0.0, 0.0, 1.0],
            spin: 0.0,
        };
        let now = Instant::now();
        let mut tags = Tags::default();
        assert!(tags.may_stick(now).is_ok());
        tags.stick(server, placed("tag_gg"), now);
        assert!(tags.may_stick(now + GAP / 2).is_err());
        assert!(tags.may_stick(now + GAP).is_ok());
        assert!(tags.own(Some(server), Some("mp/ffa3"), |_| false).is_some());
        assert!(
            tags.own(Some(server), Some("mp/duel6"), |_| false)
                .is_none()
        );
        tags.stick(server, placed("tag_ez"), now);
        assert!(tags.own(Some(other), Some("mp/ffa3"), |_| true).is_none());
        tags.stick(server, placed("tag_ez"), now);
        assert!(
            tags.own(Some(server), Some("mp/ffa3"), |_| false).is_none(),
            "taken back"
        );
        tags.stick(server, placed("tag_ez"), now);
        assert!(tags.clear());
        assert!(!tags.clear());
    }
}
