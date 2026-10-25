//! Card faces (`docs/cards.md`, "Faces"): what a revealed card shows, read from the SJK
//! packs the hub serves (`sjk_cards`, [`crate::sjk_packs`]). Each card is a file
//! `cards/<id>.card` (JSON: its name, set, number, kind, rarity, lore, artist and the game
//! path of its picture) beside its picture, a JPEG or PNG framed for the card's art
//! window ([`ART_ASPECT`]). The hub keeps which card each copy is; the pack says what it
//! looks like, so a client draws a card it has the face of and shows "Unknown card"
//! (its id) for one it has not.
//!
//! The faces are read whenever the packs are mounted ([`install`]); a picture is decoded
//! on a worker thread when a screen first draws it ([`Faces::request`]), squeezed into a
//! square mip chain the UI renderer uploads (`ui_renderer/card_art.rs`) and takes back
//! ([`Faces::take_art`]), then drawn back into the 6:5 window, which undoes the squeeze.
//! Until a picture is decoded the window stays dark. All 339 pictures were decoded at
//! every start and kept, 5.3 MiB each in memory and again on the graphics card: about
//! 2 GB of each on Slash's built-in Radeon (`vram`, 25/10/2026), a card never opened.

use super::Rarity;
use crate::menu::emblem::MipChain;
use serde::Deserialize;
use sjk_ui::TextureId;
use sjk_vfs::VirtualFileSystem;
use std::sync::{Arc, Mutex, MutexGuard, Weak, mpsc};

/// Where the card files are in a pack, and their extension.
pub(crate) const FOLDER: &str = "cards";
pub(crate) const EXTENSION: &str = ".card";
/// The art window's width against its height: 6:5 (pictures are 1020 x 850).
pub(crate) const ART_ASPECT: f32 = 1.2;
/// The square texture a picture is squeezed into.
const TEXTURE_EDGE: u32 = 1_024;
/// Most faces the client draws pictures for; one texture each.
pub(crate) const MAX_FACES: usize = 512;
/// Card pictures' texture ids: this, plus the face's position in [`Faces::list`].
const TEXTURE_BASE: u32 = 0x6100_0000;
/// The longest name, set name and artist, and lore a face may have (as the hub's pack
/// build checks).
const NAME_MAX: usize = 40;
const LORE_MAX: usize = 400;

/// What a card is about. A face of a kind (or rarity) this client does not know is left
/// out on its own, named in the log; the pack's other faces still read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Kind {
    Community,
    Place,
    Personality,
    Game,
    /// The people the project owes (19/10/2026): card 1 of SJK Alpha is Bishop.
    Credits,
    /// A meme of the community (19/10/2026).
    Meme,
    /// A droppable saber shader (20/10/2026, the SJK Blades collection): its Solar copies
    /// drop that blade too ([`Face::blade`]).
    Blade,
}

impl Kind {
    /// Every kind the client knows, in the order the Cards tab's filter walks them.
    pub(crate) const ALL: [Self; 7] = [
        Self::Community,
        Self::Place,
        Self::Personality,
        Self::Game,
        Self::Credits,
        Self::Meme,
        Self::Blade,
    ];

    fn from_id(id: &str) -> Option<Self> {
        match id {
            "community" => Some(Self::Community),
            "place" => Some(Self::Place),
            "personality" => Some(Self::Personality),
            "game" => Some(Self::Game),
            "credits" => Some(Self::Credits),
            "meme" => Some(Self::Meme),
            "blade" => Some(Self::Blade),
            _ => None,
        }
    }

    /// What players read ("Community").
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Community => "Community",
            Self::Place => "Place",
            Self::Personality => "Personality",
            Self::Game => "Jedi Academy",
            Self::Credits => "Credits",
            Self::Meme => "Meme",
            Self::Blade => "Blade",
        }
    }
}

/// One card's face.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Face {
    /// The card's id ("sjk_alpha_010"), the hub's.
    pub(crate) id: String,
    pub(crate) name: String,
    /// The set's id ("sjk_alpha") and the name players read ("SJK Alpha").
    pub(crate) set: String,
    pub(crate) set_name: String,
    /// Its place in its set, from 1.
    pub(crate) number: u16,
    pub(crate) kind: Kind,
    pub(crate) rarity: &'static Rarity,
    pub(crate) lore: String,
    pub(crate) artist: String,
    /// The game path of its picture in the pack.
    pub(crate) art: String,
    /// The unlockable a copy of it brings while held (`saber_sun`; 20/10/2026), as the
    /// hub grants it ([`crate::unlockables::CARD_BLADES`]); `None` for most cards.
    pub(crate) grants: Option<String>,
    /// The saber shader a blade card is of (`saber_sun`; 20/10/2026): the hub drops it
    /// to whoever finds or reveals a Solar copy. `None` for other cards.
    pub(crate) blade: Option<String>,
}

impl Face {
    /// What it brings, as its hover card and the Cards tab say it ("Brings the Sun blade
    /// while you hold it."), when it brings an unlockable this client knows.
    pub(crate) fn brings(&self) -> Option<String> {
        let unlockable = crate::unlockables::find(self.grants.as_deref()?)?;
        Some(format!("Brings the {} while you hold it.", unlockable.name))
    }

    /// What its Solar copies drop, as its hover card and the Cards tab say it ("Its Solar
    /// copies also drop the Sun blade."), when it is a blade card of an unlockable this
    /// client knows.
    pub(crate) fn solar_drop(&self) -> Option<String> {
        let unlockable = crate::unlockables::find(self.blade.as_deref()?)?;
        Some(format!(
            "Its Solar copies also drop the {}.",
            unlockable.name
        ))
    }

    /// What any copy of a blade card gives when crafting, as its hover card says it ("Any
    /// copy crafts the Sun blade for less."; 21/10/2026), for a shader this client knows.
    pub(crate) fn crafts_for_less(&self) -> Option<String> {
        let unlockable = crate::unlockables::find(self.blade.as_deref()?)?;
        Some(format!("Any copy crafts the {} for less.", unlockable.name))
    }

    /// What its copies bring ([`PERKS`], in that order; 21/10/2026, Sol's request: a mark
    /// on the card for each), as the tiles, the close-up and the hover card show them.
    pub(crate) fn perks(&self) -> Vec<Perk> {
        PERKS
            .iter()
            .filter_map(|(mark, words)| {
                Some(Perk {
                    mark: *mark,
                    words: words(self)?,
                })
            })
            .collect()
    }
}

/// A perk of a card: its SJK Symbols mark and its words ("Brings the Sun blade while you
/// hold it.").
pub(crate) struct Perk {
    pub(crate) mark: char,
    pub(crate) words: String,
}

/// Every perk a card can have and its mark, in the order they show: a new perk is a row
/// here. A lightsaber brings a blade while held, the Solar finish's sun drops one, the
/// smith's hammer crafts one for less.
const PERKS: [(char, fn(&Face) -> Option<String>); 3] = [
    (crate::symbols::SABER, Face::brings),
    (crate::symbols::FINISH[2], Face::solar_drop),
    (crate::symbols::SMITH, Face::crafts_for_less),
];

/// A card file as written.
#[derive(Deserialize)]
struct File {
    name: String,
    set: String,
    set_name: String,
    number: u16,
    kind: String,
    rarity: String,
    #[serde(default)]
    lore: String,
    art: String,
    #[serde(default)]
    artist: String,
    #[serde(default)]
    grants: Option<String>,
    #[serde(default)]
    blade: Option<String>,
}

/// The card file `text` of card `id`, read.
fn parse(id: &str, text: &str) -> Result<Face, String> {
    let file: File = serde_json::from_str(text).map_err(|error| error.to_string())?;
    let within = |what: &str, value: &str, max: usize| {
        if value.trim().is_empty() || value.chars().count() > max {
            Err(format!("{what} must be 1 to {max} characters"))
        } else {
            Ok(())
        }
    };
    within("name", &file.name, NAME_MAX)?;
    within("set_name", &file.set_name, NAME_MAX)?;
    if file.lore.chars().count() > LORE_MAX {
        return Err(format!("lore is longer than {LORE_MAX} characters"));
    }
    // 0 is allowed: SJK Alpha's card 000 is Sol's (19/10/2026).
    if file.number > 999 {
        return Err("number must be 0 to 999".to_owned());
    }
    let kind = Kind::from_id(&file.kind).ok_or_else(|| format!("unknown kind {}", file.kind))?;
    let rarity =
        Rarity::from_id(&file.rarity).ok_or_else(|| format!("unknown rarity {}", file.rarity))?;
    Ok(Face {
        id: id.to_owned(),
        name: file.name,
        set: file.set,
        set_name: file.set_name,
        number: file.number,
        kind,
        rarity,
        lore: file.lore,
        artist: if file.artist.trim().is_empty() {
            "Sol".to_owned()
        } else {
            file.artist
        },
        art: file.art,
        grants: file.grants.filter(|id| !id.trim().is_empty()),
        blade: file.blade.filter(|id| !id.trim().is_empty()),
    })
}

/// Every card face in `vfs`, by set and number; a file that cannot be read is named in
/// the log and left out.
pub(crate) fn load(vfs: &VirtualFileSystem) -> Vec<Face> {
    let mut faces = Vec::new();
    for listed in vfs.list_files(FOLDER, EXTENSION) {
        let path = format!("{FOLDER}/{listed}");
        let id = listed.trim_end_matches(EXTENSION);
        let read = vfs
            .read(&path)
            .map_err(|error| error.to_string())
            .and_then(|asset| asset.ok_or_else(|| "missing".to_owned()))
            .and_then(|asset| String::from_utf8(asset.bytes).map_err(|_| "not UTF-8".to_owned()))
            .and_then(|text| parse(id, &text));
        match read {
            Ok(face) => faces.push(face),
            Err(why) => crate::log::progress(format_args!("cards: {path}: {why}")),
        }
    }
    faces.sort_by(|a, b| (&a.set, a.number, &a.id).cmp(&(&b.set, b.number, &b.id)));
    faces.dedup_by(|a, b| a.id == b.id);
    faces.truncate(MAX_FACES);
    faces
}

/// Where one face's picture is.
enum Art {
    /// Not asked for, or handed to the renderer.
    Idle,
    /// On the worker.
    Decoding,
    /// Decoded, waiting for the renderer.
    Ready(MipChain),
    /// Missing or not a picture: not tried again under this mount.
    Failed,
}

/// The faces of the last mount and their pictures, as they are asked for.
pub(crate) struct Faces {
    pub(crate) list: Vec<Face>,
    art: Vec<Mutex<Art>>,
    /// The packs the pictures are read from.
    packs: Option<VirtualFileSystem>,
    /// The worker decoding the pictures asked for, started by the first request.
    worker: Mutex<Option<mpsc::Sender<usize>>>,
    /// Counts mounts; the renderer uploads the pictures again when it changes.
    pub(crate) generation: u64,
}

impl Faces {
    fn new(list: Vec<Face>, packs: Option<VirtualFileSystem>, generation: u64) -> Self {
        Self {
            art: list.iter().map(|_| Mutex::new(Art::Idle)).collect(),
            list,
            packs,
            worker: Mutex::new(None),
            generation,
        }
    }

    fn empty() -> Self {
        Self::new(Vec::new(), None, 0)
    }

    /// The face of card `id`, with its position in [`Self::list`].
    pub(crate) fn find(&self, id: &str) -> Option<(usize, &Face)> {
        self.list.iter().enumerate().find(|(_, face)| face.id == id)
    }

    /// Whether some face has rarity `rarity`: sealed copies of it can be revealed.
    pub(crate) fn has_rarity(&self, rarity: &Rarity) -> bool {
        self.list.iter().any(|face| face.rarity.id == rarity.id)
    }

    /// Faces `list` without pictures, for tests (not the current ones).
    #[cfg(test)]
    pub(crate) fn of_list(list: Vec<Face>) -> Self {
        Self::new(list, None, 1)
    }

    fn state(&self, index: usize) -> Option<MutexGuard<'_, Art>> {
        self.art
            .get(index)
            .map(|art| art.lock().unwrap_or_else(|poisoned| poisoned.into_inner()))
    }

    /// Decode face `index`'s picture on the worker, unless it is decoding, waiting or
    /// could not be read.
    pub(crate) fn request(self: &Arc<Self>, index: usize) {
        let Some(mut state) = self.state(index) else {
            return;
        };
        if !matches!(*state, Art::Idle) || self.packs.is_none() {
            return;
        }
        let mut worker = self
            .worker
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if worker.is_none() {
            let (sender, receiver) = mpsc::channel();
            let faces = Arc::downgrade(self);
            let spawned = std::thread::Builder::new()
                .name("sjk-card-art".into())
                .spawn(move || decode_requests(&faces, &receiver));
            match spawned {
                Ok(_) => *worker = Some(sender),
                Err(error) => {
                    crate::log::progress(format_args!(
                        "warning: card pictures not loaded, drawing without them: {error}"
                    ));
                    *state = Art::Failed;
                    return;
                }
            }
        }
        if worker
            .as_ref()
            .is_some_and(|sender| sender.send(index).is_ok())
        {
            *state = Art::Decoding;
        }
    }

    /// Face `index`'s decoded picture, handed over: the renderer keeps it on the card,
    /// and asks again ([`Self::request`]) if it lets it go.
    pub(crate) fn take_art(&self, index: usize) -> Option<MipChain> {
        let mut state = self.state(index)?;
        match std::mem::replace(&mut *state, Art::Idle) {
            Art::Ready(chain) => Some(chain),
            other => {
                *state = other;
                None
            }
        }
    }

    /// Whether face `index`'s picture is decoded and waiting, for tests.
    #[cfg(test)]
    pub(crate) fn art_ready(&self, index: usize) -> bool {
        self.state(index)
            .is_some_and(|state| matches!(*state, Art::Ready(_)))
    }

    /// Face `index`'s picture decoded now, on this thread, for a world shot.
    #[cfg(test)]
    fn decode_now(&self, index: usize) -> Option<MipChain> {
        read_picture(self.packs.as_ref()?, &self.list.get(index)?.art)
    }
}

/// The worker: decode each face asked for while its faces are the ones in use.
fn decode_requests(faces: &Weak<Faces>, requests: &mpsc::Receiver<usize>) {
    while let Ok(index) = requests.recv() {
        let Some(faces) = faces.upgrade() else {
            return;
        };
        let (Some(packs), Some(face)) = (faces.packs.as_ref(), faces.list.get(index)) else {
            continue;
        };
        let decoded = read_picture(packs, &face.art);
        if let Some(mut state) = faces.state(index) {
            *state = decoded.map_or(Art::Failed, Art::Ready);
        }
    }
}

/// The picture at `path` in `packs` as the renderer's mip chain, or `None` (logged).
fn read_picture(packs: &VirtualFileSystem, path: &str) -> Option<MipChain> {
    match packs.read(path).ok().flatten() {
        Some(asset) => decode(&asset.bytes)
            .inspect_err(|why| crate::log::progress(format_args!("cards: {path}: {why}")))
            .ok(),
        None => {
            crate::log::progress(format_args!("cards: {path}: missing"));
            None
        }
    }
}

static FACES: Mutex<Option<Arc<Faces>>> = Mutex::new(None);

fn lock() -> MutexGuard<'static, Option<Arc<Faces>>> {
    FACES
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The faces of the last mount (none before one).
pub(crate) fn current() -> Arc<Faces> {
    lock().clone().unwrap_or_else(|| Arc::new(Faces::empty()))
}

/// Make `list` the current faces, face `index`'s picture a copy of the current face
/// `pictures(index)`'s (decoded here): a made-up catalogue over the packs' pictures, for
/// a world shot.
#[cfg(test)]
pub(crate) fn install_for_shot(list: Vec<Face>, pictures: impl Fn(usize) -> usize) {
    let before = current();
    let faces = Faces::new(list, before.packs.clone(), before.generation + 1);
    for index in 0..faces.list.len() {
        if let (Some(chain), Some(mut state)) =
            (before.decode_now(pictures(index)), faces.state(index))
        {
            *state = Art::Ready(chain);
        }
    }
    *lock() = Some(Arc::new(faces));
}

/// The name of card `id` as players read it: its face's name, or its id when the packs
/// have no face for it.
pub(crate) fn name_of(id: &str) -> String {
    current()
        .find(id)
        .map_or_else(|| id.to_owned(), |(_, face)| face.name.clone())
}

/// The texture of face `index`'s picture.
pub(crate) fn texture(index: usize) -> TextureId {
    TextureId(TEXTURE_BASE + index.min(MAX_FACES - 1) as u32)
}

/// The face position a texture id stands for, if it is a card picture's.
pub(crate) fn index_of(texture: TextureId) -> Option<usize> {
    let index = texture.0.checked_sub(TEXTURE_BASE)? as usize;
    (index < MAX_FACES).then_some(index)
}

/// Read the faces of the packs `vfs` (just mounted, under `generation`) and make them the
/// current ones; their pictures are read when asked for ([`Faces::request`]).
pub(crate) fn install(vfs: &VirtualFileSystem, generation: u64) {
    let list = load(vfs);
    if !list.is_empty() {
        crate::log::progress(format_args!(
            "SJK packs: card faces: {}",
            list.iter()
                .map(|face| face.id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    *lock() = Some(Arc::new(Faces::new(list, Some(vfs.clone()), generation)));
}

/// A picture's bytes as the square mip chain the renderer uploads: cut to the window's
/// 6:5 about its middle when it is not 6:5, then squeezed to [`TEXTURE_EDGE`] square.
pub(crate) fn decode(bytes: &[u8]) -> Result<MipChain, String> {
    let image = image::load_from_memory(bytes)
        .map_err(|error| error.to_string())?
        .into_rgba8();
    let (width, height) = image.dimensions();
    if width == 0 || height == 0 {
        return Err("the picture is empty".to_owned());
    }
    let wide = width as f32 / height as f32;
    let (cut_w, cut_h) = if wide > ART_ASPECT {
        ((height as f32 * ART_ASPECT).round() as u32, height)
    } else {
        (width, (width as f32 / ART_ASPECT).round() as u32)
    };
    let cut = image::imageops::crop_imm(
        &image,
        (width - cut_w.min(width)) / 2,
        (height - cut_h.min(height)) / 2,
        cut_w.min(width),
        cut_h.min(height),
    )
    .to_image();
    let square = image::imageops::resize(
        &cut,
        TEXTURE_EDGE,
        TEXTURE_EDGE,
        image::imageops::FilterType::Triangle,
    );
    Ok(MipChain::new(square))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: &str = r#"{
  "name": "Toxiee and Sol on JoF Memorial",
  "set": "sjk_alpha",
  "set_name": "SJK Alpha",
  "number": 10,
  "kind": "meme",
  "rarity": "legendary",
  "lore": "Tox was simply chilling.",
  "art": "cards/sjk_alpha_010.jpg",
  "artist": "Sol"
}
"#;

    #[test]
    fn a_card_file_reads_and_bad_ones_are_refused() {
        let face = parse("sjk_alpha_010", FILE).unwrap();
        assert_eq!(face.name, "Toxiee and Sol on JoF Memorial");
        assert_eq!((face.set.as_str(), face.number), ("sjk_alpha", 10));
        assert_eq!(face.kind, Kind::Meme);
        assert_eq!(face.rarity.id, "legendary");
        assert_eq!(face.art, "cards/sjk_alpha_010.jpg");
        let bishop = FILE
            .replace("\"meme\"", "\"credits\"")
            .replace("\"legendary\"", "\"mythical\"");
        let bishop = parse("sjk_alpha_001", &bishop).unwrap();
        assert_eq!(
            (bishop.kind.name(), bishop.rarity.name),
            ("Credits", "Mythical")
        );
        assert_eq!(Kind::Meme.name(), "Meme");
        for (from, to) in [
            ("\"legendary\"", "\"mythic\""),
            ("\"meme\"", "\"memes\""),
            ("\"number\": 10", "\"number\": 1000"),
            ("Toxiee and Sol on JoF Memorial", ""),
        ] {
            assert!(parse("x", &FILE.replace(from, to)).is_err(), "{to}");
        }
        // Card 000 is a card like any other.
        let zero = FILE.replace("\"number\": 10", "\"number\": 0");
        assert_eq!(parse("x", &zero).unwrap().number, 0);
        // A card that brings a blade says which; one this client does not know brings
        // nothing it can name.
        assert_eq!(face.grants, None);
        let sun = FILE.replace("\"artist\"", "\"grants\": \"saber_sun\", \"artist\"");
        let sun = parse("sjk_alpha_000", &sun).unwrap();
        assert_eq!(sun.grants.as_deref(), Some("saber_sun"));
        assert_eq!(
            sun.brings().as_deref(),
            Some("Brings the Sun blade while you hold it.")
        );
        let later = FILE.replace("\"artist\"", "\"grants\": \"saber_later\", \"artist\"");
        assert_eq!(parse("x", &later).unwrap().brings(), None);
        assert_eq!(face.brings(), None);
        // A blade card names its shader; its Solar copies drop it.
        let blade = FILE
            .replace("\"meme\"", "\"blade\"")
            .replace("\"artist\"", "\"blade\": \"saber_sun\", \"artist\"");
        let blade = parse("sjk_blades_001", &blade).unwrap();
        assert_eq!((blade.kind, blade.kind.name()), (Kind::Blade, "Blade"));
        assert_eq!(blade.blade.as_deref(), Some("saber_sun"));
        assert_eq!(
            blade.solar_drop().as_deref(),
            Some("Its Solar copies also drop the Sun blade.")
        );
        let blank = FILE.replace("\"artist\"", "\"blade\": \" \", \"artist\"");
        assert_eq!(parse("x", &blank).unwrap().blade, None);
        let later = FILE.replace("\"artist\"", "\"blade\": \"saber_later\", \"artist\"");
        assert_eq!(parse("x", &later).unwrap().solar_drop(), None);
        assert_eq!(
            blade.crafts_for_less().as_deref(),
            Some("Any copy crafts the Sun blade for less.")
        );
        assert_eq!(face.crafts_for_less(), None);
        assert_eq!(face.solar_drop(), None);
        // Its perks, a mark each, in the table's order; a plain card has none.
        let marks: Vec<char> = blade.perks().iter().map(|perk| perk.mark).collect();
        assert_eq!(
            marks,
            [crate::symbols::FINISH[2], crate::symbols::SMITH],
            "the Solar drop, then the craft"
        );
        assert!(face.perks().is_empty());
        assert_eq!(sun.perks()[0].mark, crate::symbols::SABER);
        // A field the client does not know yet is read past.
        let later = FILE.replace("\"artist\"", "\"frame\": \"gold\", \"artist\"");
        assert!(parse("x", &later).is_ok());
    }

    #[test]
    fn a_picture_is_squeezed_into_a_square_chain() {
        let mut png = Vec::new();
        let picture = image::RgbaImage::from_pixel(1_020, 850, image::Rgba([10, 20, 30, 255]));
        image::DynamicImage::ImageRgba8(picture)
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        let chain = decode(&png).unwrap();
        assert_eq!(chain.size, TEXTURE_EDGE);
        assert_eq!(chain.levels.len(), 11, "1024 down to 1");
        assert!(decode(b"not a picture").is_err());
    }

    #[test]
    fn texture_ids_round_trip() {
        assert_eq!(index_of(texture(3)), Some(3));
        assert_eq!(index_of(TextureId(TEXTURE_BASE - 1)), None);
        assert_eq!(index_of(TextureId(TEXTURE_BASE + MAX_FACES as u32)), None);
    }
}
