//! The Collection screen (`docs/sjk-ui.md`, Collection): what an SJK player collects,
//! under one row of tabs. Medals (given by the SJK team, hanging from a rail),
//! Achievements (a wall of medallions by category), Shaders (the saber's looks: the
//! stock blade and every blade skin, owned or locked), Toys (the Illuminate holocron)
//! Nameplates (ornaments for the plate over the player's head; none yet) and Cards (the
//! sealed cards opening holocrons brought, and the credits), and since 21/10/2026 the
//! Forge between them (the shop, crafting and breaking shaders down, [`forge`]). Every
//! tab shows its things on the left, each in its place from the start (the ones the
//! player lacks dimmed), and the one chosen up close on the right; on Shaders, Toys and
//! Nameplates that is the player's own model, on the menu map's stage or in a live
//! preview over a match ([`Backstage`]), wearing the shader chosen, a locked one too
//! (only on this screen: nothing is sent).
//!
//! In the SJK UI it is the main page's and the game menu's Collection, a screen of its
//! own ([`crate::profile_hub`]); in the classic menus the `collection`, `achievements`
//! and `unlockables` commands open it on its own. Like the Profile page it lives in the
//! console and has the SJK UI's look in every menu style ([`view`]).
//!
//! Keys: the arrows choose (on Achievements Up and Down move between the categories),
//! Tab and Shift+Tab walk the things, Enter or Space equips or unequips the shader
//! chosen or lights and puts out the Illuminate holocron, Ctrl+Tab changes tab (the
//! switch's, [`crate::profile_hub`]), Escape goes back. The pointer chooses by
//! hovering; a click on a button acts, on a tab shows it.

use crate::achievements::Standing;
use crate::menu_widgets::{BACK_TOKEN, MenuCanvas};
use crate::player_menu::ReturnTarget;
use crate::profile_hub::Tab;
use crate::unlockables::{self, Holdings};
use sjk_identity::Snapshot;
use sjk_ui::{InputEvent, UiEventKind};
use std::time::Instant;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

#[path = "collection_achievements.rs"]
mod achievements;
#[path = "collection_cards.rs"]
mod cards;
#[path = "collection_forge.rs"]
pub(crate) mod forge;
#[path = "collection_forge_view.rs"]
mod forge_view;
#[path = "collection_hover.rs"]
mod hover;
#[path = "collection_medals.rs"]
mod medals;
#[path = "collection_nameplates.rs"]
mod nameplates;
#[path = "collection_shaders.rs"]
mod shaders;
#[path = "collection_spin.rs"]
mod spin;
#[path = "collection_swatch.rs"]
pub(crate) mod swatch;
#[path = "collection_toys.rs"]
mod toys;
#[path = "collection_view.rs"]
mod view;

pub(crate) use spin::SaberTurn;
pub(crate) use view::{MODEL_AREA, SABER_AREA, row_labels};

/// The things' tokens: the medals, the achievements, the shaders' rows, the shader's
/// Equip or Unequip, the toy and its switch.
/// Draw commands the page holds a frame: twice the most a world shot found (`world_shot_forge.rs`,
/// `duel6_sjk_forge_drawn_swatches_fit`).
pub(crate) const DRAW_COMMANDS: usize = 4_096;
const MEDAL_BASE: u16 = 1_200;
const ACHIEVEMENT_BASE: u16 = 1_240;
/// The shaders' rows have a range of their own, 7000 to 7299, for a catalogue of up
/// to 299 skins (the stock blade is row 0).
const SHADER_BASE: u16 = 7_000;
const SHADER_LIMIT: u16 = 7_300;
/// The click every page plays for a press.
fn ui_cues_click() {
    crate::audio::ui_cues::post(crate::audio::ui_cues::Cue::Click);
}

/// Two clicks on a shader row this close together wear it (as the main menu's server
/// browser joins on one).
const DOUBLE_CLICK: std::time::Duration = std::time::Duration::from_millis(400);
const WEAR_TOKEN: u16 = 1_300;
const TOY_TOKEN: u16 = 1_310;
const TOY_SWITCH_TOKEN: u16 = 1_311;
/// The cardboard box's niche on Toys (Illuminate's is [`TOY_TOKEN`]).
const BOX_TOKEN: u16 = 1_314;
/// Order 66's niche on Toys.
const ORDER_TOKEN: u16 = 1_315;
/// The Hax's niche on Toys.
const HAX_TOKEN: u16 = 1_318;
/// The Ragdoll's niche on Toys.
const RAGDOLL_TOKEN: u16 = 1_309;
/// The Drone's niche on Toys.
const DRONE_TOKEN: u16 = 1_308;
/// The Shaders tab's lone saber, where the pointer takes hold of it to turn it.
const SABER_TOKEN: u16 = 1_316;
/// The shader card's way to the Forge: Craft in the Forge (one not held) or Break down
/// in the Forge (a copy of the player's own).
const FORGE_LINK_TOKEN: u16 = 1_317;
/// The Nameplates tab's See and share nameplates switch.
const SHARE_TOKEN: u16 = 1_319;
/// The toys the Toys tab shows: Illuminate, the cardboard box, Order 66, the Hax, the
/// Ragdoll, then the Drone.
const TOY_COUNT: usize = 6;
/// Each toy's niche, in the order of [`Panel::toy`].
const TOY_TOKENS: [u16; TOY_COUNT] = [
    TOY_TOKEN,
    BOX_TOKEN,
    ORDER_TOKEN,
    HAX_TOKEN,
    RAGDOLL_TOKEN,
    DRONE_TOKEN,
];
/// The Shaders tab's view switches: the rack (a list) and the grid of cards.
const LIST_VIEW_TOKEN: u16 = 1_312;
const GRID_VIEW_TOKEN: u16 = 1_313;
/// The chosen shader's options (its parts the wearer may switch off), one each.
const OPTION_BASE: u16 = 1_320;
/// The Cards tab's Reveal button.
const REVEAL_TOKEN: u16 = 1_350;
const _: () = assert!(OPTION_BASE as usize + OPTION_TOKENS <= REVEAL_TOKEN as usize);
/// The Cards tab's tiles (19/10/2026, the binder): one token each in the list the filter
/// shows, up to [`CARD_TILES_MAX`]; then its filter's chips (All, Owned and Missing,
/// then the rarity, the kind and the finish, each cycling).
const CARD_BASE: u16 = 7_400;
pub(super) const CARD_TILES_MAX: usize = 600;
const CARD_SHOW_BASE: u16 = 8_000;
const CARD_RARITY_TOKEN: u16 = 8_003;
const CARD_KIND_TOKEN: u16 = 8_004;
const CARD_FINISH_TOKEN: u16 = 8_005;
/// The card looked at closely: the darkened window under it (a click closes it), and
/// the card up close, where a click opens it (20/10/2026).
const CARD_INSPECT_TOKEN: u16 = 8_006;
const CARD_CLOSE_UP_TOKEN: u16 = 8_007;
/// Under a blade card held, up close: the way to its shader, on the Forge's Craft while
/// the player lacks it (21/10/2026, where its copy crafts the shader for less), else on
/// the Shaders tab.
const CARD_SHADER_TOKEN: u16 = 8_008;
const _: () = assert!(CARD_BASE as usize + CARD_TILES_MAX <= CARD_SHOW_BASE as usize);
const OPTION_TOKENS: usize = crate::blade_skin_file::extras::MAX_OPTIONS;
/// The Shaders tab's rows: the stock blade, then every blade skin, the rarest first
/// ([`unlockables::blade_skins_by_tier`]).
const SHADER_ROWS: usize = 1 + unlockables::ALL.len();
const _: () = assert!(SHADER_BASE as usize + SHADER_ROWS <= SHADER_LIMIT as usize);
/// The Shaders tab's filter (14/10/2026): its tier chips (All, then the five tiers), the
/// owned-only chip and the search field.
const FILTER_TIER_BASE: u16 = 7_300;
const FILTER_OWNED_TOKEN: u16 = 7_310;
const SEARCH_TOKEN: u16 = 7_311;
/// The most characters a search holds.
const SEARCH_MAX: usize = 24;
/// The tiers the chips filter by, after All, the commonest first.
const FILTER_TIERS: [unlockables::Rarity; 5] = [
    unlockables::Rarity::Common,
    unlockables::Rarity::Uncommon,
    unlockables::Rarity::Rare,
    unlockables::Rarity::Legendary,
    unlockables::Rarity::Mythical,
];

/// What the Shaders tab lists: a tier, the shaders held only, and words of a name (or a
/// tier) or a blade's number the search names. The stock blade is always the first row.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct ShaderFilter {
    pub(super) tier: Option<unlockables::Rarity>,
    pub(super) owned: bool,
    pub(super) query: String,
}

impl ShaderFilter {
    /// Whether anything is filtered out.
    pub(super) fn active(&self) -> bool {
        self.tier.is_some() || self.owned || !self.query.trim().is_empty()
    }

    /// Whether `skin` (held or not) stays in the list.
    pub(super) fn passes(&self, skin: &unlockables::Unlockable, owned: bool) -> bool {
        if self.tier.is_some_and(|tier| tier != skin.tier) || (self.owned && !owned) {
            return false;
        }
        let query = self.query.trim().to_lowercase();
        // A number finds that blade alone ("42", "042" or "#42").
        let digits = query.strip_prefix('#').unwrap_or(&query);
        if !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()) {
            return digits.parse() == Ok(skin.number);
        }
        query.is_empty()
            || skin.name.to_lowercase().contains(&query)
            || skin.tier.label().to_lowercase().contains(&query)
            || crate::editions::name(skin.edition)
                .to_lowercase()
                .contains(&query)
            || skin.smith.to_lowercase().contains(&query)
    }
}

/// Shader rows the rack shows at once; it scrolls to the rest.
const RACK_SHOWN: usize = 6;
/// The grid's cards a line, and the lines it shows at once; it scrolls a line at a time.
const GRID_COLUMNS: usize = 5;
const GRID_LINES_SHOWN: usize = 3;
/// Archived: how the Shaders tab lists the shaders, 0 the rack, 1 the grid of cards.
pub(crate) const SHADER_VIEW_CVAR: &str = "ui_shaderView";

/// The blade skin of shader row `row` with no filter: `None` for the stock blade (row 0) or
/// past the end. The page itself reads the filtered list ([`Panel::skin_at`]).
#[cfg(test)]
fn shader_skin(row: usize) -> Option<&'static unlockables::Unlockable> {
    row.checked_sub(1)
        .and_then(|index| unlockables::blade_skins_by_tier().get(index))
        .copied()
}

/// What the console does after the page handled an event.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum PanelAction {
    None,
    Close,
    /// Set `cg_saberSkin` to this id (`""` for the stock blade).
    Wear(&'static str),
    /// Light the Illuminate holocron (`true`) or put it out: the same as `toy_illuminate`,
    /// set rather than toggled.
    Illuminate(bool),
    /// Get in the cardboard box (`true`) or out: the same as `toy_box`, set rather than
    /// toggled.
    CardboardBox(bool),
    /// Turn Order 66 on (`true`) or off: the same as `toy_order66`, set rather than
    /// toggled.
    Order66(bool),
    /// Turn the Hax on (`true`) or off: the same as `toy_hax`, set rather than toggled.
    Hax(bool),
    /// Go limp (`true`) or get up: the same as `toy_ragdoll`, set rather than toggled.
    Ragdoll(bool),
    /// Turn the Drone on (`true`) or off: the same as `toy_drone`, set rather than toggled.
    Drone(bool),
    /// Show a tab another page draws: the Holocrons page.
    Hub(Tab),
    /// Switch blade skin `skin`'s option `option` on or off (`cg_saberSkinOptions`).
    SkinOption {
        skin: &'static str,
        option: String,
        on: bool,
    },
    /// Set [`SHADER_VIEW_CVAR`]: the grid of cards (`true`) or the rack.
    ShaderView(bool),
    /// Break one duplicate of this shader down into kyber shards (`crate::economy`).
    Shatter(&'static str),
    /// Break this shader down, the last copy too (a hub that says `shatter_last`).
    ShatterLast(&'static str),
    /// Break every duplicate down, one copy of each kept.
    ShatterAll,
    /// Craft this shader with kyber shards.
    Craft(&'static str),
    /// Ask the hub to sell the player this offer of the day.
    Buy(String),
    /// Ask the hub to sell the player a holocron of this tier for credits.
    BuyHolocron(String),
    /// Ask the hub to reveal the player's sealed cards.
    RevealCards,
    /// See and share nameplates on (`true`) or off: `cg_nameplate`
    /// ([`crate::hud::nameplate::see_and_share`]).
    SeeAndShare(bool),
}

/// Where the player's model stands behind Shaders, Toys and Nameplates this frame, as
/// the renderer says ([`crate::menu_stage`]).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) enum Backstage {
    /// Nowhere: the page draws the thing itself.
    #[default]
    None,
    /// On the menu map's stage, right of the page; `head` is where the top of its head
    /// is in the window, when known.
    World { head: Option<[f32; 2]> },
    /// In the live preview the page shows at [`MODEL_AREA`], once a frame of it is drawn.
    Preview { ready: bool },
}

/// The blade the model on the stage holds while the page shows it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PreviewSkin {
    /// What the player wears ([`crate::GpuState::local_saber_skin`]).
    Worn,
    /// The stock blade.
    Stock,
    /// This blade skin, owned or not: only drawn here, never sent.
    Skin(&'static str),
}

/// What the page wants the stage to show.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct StageWish {
    pub(crate) skin: PreviewSkin,
    /// The saber on its own, turned so, in place of the model holding it (Shaders).
    pub(crate) saber: Option<SaberTurn>,
    /// The Illuminate holocron floats by the model, lit.
    pub(crate) holocron: bool,
}

/// What the page knows this frame.
pub(crate) struct Inputs<'a> {
    /// `cl_identity`.
    pub(crate) enabled: bool,
    pub(crate) snapshot: Option<&'a Snapshot>,
    /// The unlocks the player's own hub profile lists, or why they are not known.
    pub(crate) holdings: Holdings<'a>,
    /// `cg_saberSkin`.
    pub(crate) setting: &'a str,
    /// Every achievement with the player's progress.
    pub(crate) standings: &'a [Standing],
    /// The Illuminate holocron is lit now (the client's own state, kept in the console
    /// by the viewer each frame).
    pub(crate) illuminate: bool,
    /// The player is in the cardboard box now (kept as [`Self::illuminate`] is).
    pub(crate) cardboard_box: bool,
    /// The Order 66 toy is on now (kept as [`Self::illuminate`] is).
    pub(crate) order66: bool,
    /// The Hax toy is on now (kept as [`Self::illuminate`] is).
    pub(crate) hax: bool,
    /// The player is limp now (the Ragdoll, kept as [`Self::illuminate`] is).
    pub(crate) ragdoll: bool,
    /// The Drone toy is on now (kept as [`Self::illuminate`] is).
    pub(crate) drone: bool,
    /// The `name` cvar, for the nameplate.
    pub(crate) name: &'a str,
    /// The stock blade's colour (`color1`), for its swatch.
    pub(crate) stock: sjk_ui::Color,
    /// [`SHADER_VIEW_CVAR`]: the Shaders tab shows the grid of cards.
    pub(crate) grid: bool,
    /// `cg_saberSkinOptions`: the skins' parts switched off.
    pub(crate) options: &'a str,
    /// See and share nameplates (`cg_nameplate`): the player sees nameplates and shares
    /// their health, shield and Force.
    pub(crate) share: bool,
}

/// What the last frame showed of a shader's row, for keys and clicks.
#[derive(Clone, Copy, Debug, Default)]
struct ShaderRow {
    /// What Enter does on it: `None` for a locked one or one already worn by the stock
    /// row.
    wear: Option<&'static str>,
    /// Worn now: a double click on it changes nothing.
    worn: bool,
}

pub(crate) struct Panel {
    open: bool,
    owns_console: bool,
    /// Shown as the SJK UI's Collection screen: its way back names where it returns.
    hub: bool,
    back: ReturnTarget,
    tab: Tab,
    ui: MenuCanvas,
    /// The thing chosen on each tab: a medal in the order shown, an achievement in the
    /// catalogue's order, a shader's row.
    medal: usize,
    achievement: usize,
    shader: usize,
    /// The toy chosen on Toys: 0 Illuminate, 1 the cardboard box, 2 Order 66, 3 the Hax,
    /// 4 the Ragdoll, 5 the Drone.
    toy: usize,
    /// The player was in the cardboard box, as the last frame's inputs said.
    box_on: bool,
    /// Order 66 was on, as the last frame's inputs said.
    order66_on: bool,
    /// The Hax was on, as the last frame's inputs said.
    hax_on: bool,
    /// The player was limp, as the last frame's inputs said.
    ragdoll_on: bool,
    /// The Drone was on, as the last frame's inputs said.
    drone_on: bool,
    /// See and share nameplates was on, as the last frame's inputs said.
    share_on: bool,
    /// The tile chosen on Cards (in the list the filter shows), how many tiles the last
    /// frame's list held, and the first line of them shown (the grid scrolls a line at a
    /// time).
    card: usize,
    cards_shown: usize,
    card_first: usize,
    /// Whether the chosen tile is looked at closely, large over the darkened tab
    /// (`collection_cards.rs`, `card_inspect_view`).
    card_inspect: bool,
    /// The Cards tab's filter, and the tiles it showed last frame (for the hover cards).
    card_filter: crate::cards::binder::Filter,
    card_tiles: Vec<crate::cards::binder::Tile>,
    /// The shader of the blade card up close last frame, when one is held (its button
    /// goes there), whether the Forge crafts it for the player (the button goes to the
    /// Forge's Craft then), and the shader the Shaders tab turns to on its next frame.
    card_blade: Option<&'static unlockables::Unlockable>,
    card_blade_craft: bool,
    shader_focus: Option<&'static str>,
    /// How many sealed copies the last frame offered to reveal (0: no Reveal button).
    revealable: u32,
    /// The identity's last reveal when Reveal was pressed: the line under the button
    /// says what the next one brought.
    reveal_asked: Option<u64>,
    /// The first shader row the rack shows (it scrolls, [`RACK_SHOWN`] at a time).
    shader_first: usize,
    /// The Shaders tab shows the grid of cards, as the last frame's inputs said (or a
    /// switch since).
    grid: bool,
    /// The first line of cards the grid shows (it scrolls, [`GRID_LINES_SHOWN`] at a
    /// time).
    grid_first: usize,
    /// The shader row last clicked and when, for a double click's Equip.
    last_click: Option<(u16, Instant)>,
    /// How many medals the last frame showed.
    medals_shown: usize,
    shader_rows: [ShaderRow; SHADER_ROWS],
    /// The shaders the tab lists now (the filter's), the stock blade's row 0 before them,
    /// and how many rows that makes; read again each frame.
    shader_list: Vec<&'static unlockables::Unlockable>,
    shader_count: usize,
    shader_filter: ShaderFilter,
    /// The search field takes the typing.
    search_focus: bool,
    /// The chosen shader's options the last frame showed (its skin, the option's id and
    /// whether it is on), for clicks and the number keys.
    shader_options: Vec<(&'static str, String, bool)>,
    /// Where the model stands this frame.
    backstage: Backstage,
    /// The thing the pointer rests on, for its hover card.
    hover: crate::menu::sjk::hover::Hover,
    /// Where the pointer is (window pixels), while it is over the window.
    pointer: Option<[f32; 2]>,
    /// The cards answering the pointer on Cards, each eased in and out.
    tilt: crate::tile_shine::HoverEase,
    /// The Shaders tab's lone saber's turn, and the window pixels to a frame pixel the
    /// last frame was drawn at (a drag turns it by frame pixels).
    spin: spin::Spin,
    scale: f32,
    /// The Forge tab: its section, lists, filters and trades ([`forge`]).
    forge: forge::Forge,
    /// Where the chosen shader's way to the Forge leads, as the last frame drew it.
    forge_link: Option<(forge::Section, &'static str)>,
    /// Whether the hub's economy is known (the Shaders tab's way to the Forge shows then).
    trade_known: bool,
    /// How much wider than the display family the page's text face is drawn.
    widen: f32,
    epoch: Instant,
    /// The blade skins the hub's packs brought, the swatches' looks (`sjk_packs.rs`);
    /// taken again only when they change ([`Panel::follow_skins`]).
    pub(super) skins: std::sync::Arc<crate::saber_skins::LoadedSkins>,
    /// What a world shot shows in place of the own profile's unlocks.
    #[cfg(test)]
    pub(crate) preview: Option<Vec<sjk_identity::Unlock>>,
    /// The time a world shot draws the swatches at, instead of the page's.
    #[cfg(test)]
    pub(crate) shot_seconds: Option<f32>,
    /// What a world shot shows in place of the own profile's cards and credits.
    #[cfg(test)]
    pub(crate) preview_cards: Option<(Vec<sjk_identity::CardStack>, u64, u64)>,
    /// What a world shot shows on the Forge in place of the live identity's.
    #[cfg(test)]
    pub(crate) preview_forge: Option<forge::ForgeData>,
    /// Whether a world shot shows the Shaders tab's way to the Forge.
    #[cfg(test)]
    trade_pin: Option<bool>,
}

impl Default for Panel {
    fn default() -> Self {
        Self::new()
    }
}

impl Panel {
    pub(crate) fn new() -> Self {
        Self {
            open: false,
            owns_console: false,
            hub: false,
            back: ReturnTarget::MainMenu,
            tab: Tab::Medals,
            // #117 (Nib): the Forge's Craft grid on its Legendary and Mythical lines, its
            // fifteen swatches drawn (the menu pictures on, or before their live pictures are
            // ready), took up to about 2 100 commands; 2 048 dropped the last ones on some
            // frames, which flickered.
            ui: MenuCanvas::with_capacities(160, 220, DRAW_COMMANDS),
            medal: 0,
            // The first frame chooses the one nearest to unlocking.
            achievement: usize::MAX,
            shader: 0,
            toy: 0,
            box_on: false,
            order66_on: false,
            hax_on: false,
            ragdoll_on: false,
            drone_on: false,
            share_on: false,
            card: 0,
            cards_shown: 0,
            card_first: 0,
            card_inspect: false,
            card_filter: Default::default(),
            card_tiles: Vec::new(),
            card_blade: None,
            card_blade_craft: false,
            shader_focus: None,
            revealable: 0,
            reveal_asked: None,
            shader_first: 0,
            grid: false,
            grid_first: 0,
            last_click: None,
            medals_shown: 0,
            shader_rows: [ShaderRow::default(); SHADER_ROWS],
            shader_list: unlockables::blade_skins_by_tier().to_vec(),
            shader_count: SHADER_ROWS,
            shader_filter: ShaderFilter::default(),
            search_focus: false,
            shader_options: Vec::new(),
            backstage: Backstage::None,
            hover: Default::default(),
            pointer: None,
            tilt: crate::tile_shine::HoverEase::default(),
            spin: spin::Spin::default(),
            scale: 1.0,
            forge: forge::Forge::default(),
            forge_link: None,
            trade_known: false,
            widen: 1.0,
            epoch: Instant::now(),
            skins: Default::default(),
            #[cfg(test)]
            preview: None,
            #[cfg(test)]
            shot_seconds: None,
            #[cfg(test)]
            preview_cards: None,
            #[cfg(test)]
            preview_forge: None,
            #[cfg(test)]
            trade_pin: None,
        }
    }

    pub(crate) fn is_open(&self) -> bool {
        self.open
    }

    /// Take the loaded blade skins when a pack brought new ones (one atomic read a
    /// frame otherwise).
    pub(crate) fn follow_skins(&mut self) {
        if self.skins.generation() != crate::sjk_packs::generation() {
            self.skins = crate::sjk_packs::skins();
        }
    }

    /// Show the page on `tab` (one of the Collection's), on its own (`hub` false: the
    /// classic menus' commands) or as the SJK UI's Collection screen returning to
    /// `back`; `owns_console` when the console was closed before it. The things chosen
    /// on each tab stay as they were.
    pub(crate) fn open(&mut self, tab: Tab, owns_console: bool, hub: bool, back: ReturnTarget) {
        if !self.open {
            // Shaders opens on what the player wears.
            self.shader = usize::MAX;
        }
        self.open = true;
        self.owns_console = owns_console;
        self.hub = hub;
        self.back = back;
        self.show(tab);
    }

    /// Turn to `tab`, if it is one of the Collection's this page draws (Holocrons is
    /// the Holocrons page).
    pub(crate) fn show(&mut self, tab: Tab) {
        if tab.screen() == crate::profile_hub::Screen::Collection && tab != Tab::Holocrons {
            if tab != self.tab {
                self.leave_search();
                self.forge.leave_search();
                self.forge.ask = None;
                self.card_inspect = false;
            }
            self.tab = tab;
        }
    }

    /// The tab on show (or shown last).
    pub(crate) fn tab(&self) -> Tab {
        self.tab
    }

    /// Whether the page is the SJK UI's Collection screen.
    pub(crate) fn is_hub(&self) -> bool {
        self.hub
    }

    /// Where the Collection screen returns.
    pub(crate) fn back(&self) -> ReturnTarget {
        self.back
    }

    /// Hide the page; returns whether it had opened the console.
    pub(crate) fn close(&mut self) -> bool {
        let owned = self.open && self.owns_console;
        self.open = false;
        self.owns_console = false;
        self.backstage = Backstage::None;
        owned
    }

    /// Where the model stands this frame.
    pub(crate) fn set_backstage(&mut self, backstage: Backstage) {
        self.backstage = backstage;
    }

    /// Whether the hub's economy is known this frame: the Shaders tab's way to the Forge
    /// shows then.
    pub(crate) fn set_trade_known(&mut self, known: bool) {
        #[cfg(test)]
        let known = self.trade_pin.unwrap_or(known);
        self.trade_known = known;
    }

    /// Show the Shaders tab's way to the Forge (`known`) whatever the hub says, for a world
    /// shot.
    #[cfg(test)]
    pub(crate) fn trade_known_for_shot(&mut self, known: bool) {
        self.trade_pin = Some(known);
        self.trade_known = known;
    }

    /// What the stage should show while the page is open on Shaders, Toys or
    /// Nameplates.
    pub(crate) fn stage_wish(&self) -> Option<StageWish> {
        if !self.open {
            return None;
        }
        match self.tab {
            Tab::Shaders => Some(StageWish {
                skin: match self.shader {
                    0 => PreviewSkin::Stock,
                    row => self
                        .skin_at(row)
                        .map_or(PreviewSkin::Worn, |skin| PreviewSkin::Skin(skin.id)),
                },
                saber: Some(self.spin.turn()),
                holocron: false,
            }),
            Tab::Toys => Some(StageWish {
                skin: PreviewSkin::Worn,
                saber: None,
                holocron: self.toy == 0,
            }),
            Tab::Nameplates => Some(StageWish {
                skin: PreviewSkin::Worn,
                saber: None,
                holocron: false,
            }),
            _ => None,
        }
    }

    pub(crate) fn draw_list(&self) -> &sjk_ui::DrawList {
        self.ui.draw_list()
    }

    /// Whether the last frame ran out of room on the canvas.
    #[cfg(test)]
    pub(crate) fn overflowed(&self) -> bool {
        self.ui.overflowed()
    }

    /// What Enter (or a click on the button) does on the tab on show.
    fn activate(&mut self, illuminate: bool) -> PanelAction {
        match self.tab {
            Tab::Shaders => self
                .shader_rows
                .get(self.shader)
                .and_then(|row| row.wear)
                .map_or(PanelAction::None, PanelAction::Wear),
            Tab::Toys if self.toy == 1 => PanelAction::CardboardBox(!self.box_on),
            Tab::Toys if self.toy == 2 => PanelAction::Order66(!self.order66_on),
            Tab::Toys if self.toy == 3 => PanelAction::Hax(!self.hax_on),
            Tab::Toys if self.toy == 4 => PanelAction::Ragdoll(!self.ragdoll_on),
            Tab::Toys if self.toy == 5 => PanelAction::Drone(!self.drone_on),
            Tab::Toys => PanelAction::Illuminate(!illuminate),
            Tab::Nameplates => PanelAction::SeeAndShare(!self.share_on),
            Tab::Cards => {
                self.card_inspect = self.cards_shown > 0;
                PanelAction::None
            }
            _ => PanelAction::None,
        }
    }

    /// How many things the tab on show offers to choose.
    fn things(&self) -> usize {
        match self.tab {
            Tab::Medals => self.medals_shown,
            Tab::Achievements => crate::achievements::ALL.len(),
            Tab::Shaders => self.shader_count,
            Tab::Cards => self.cards_shown,
            Tab::Toys => TOY_COUNT,
            _ => 0,
        }
    }

    /// The chosen thing of the tab on show, by its index.
    fn chosen_mut(&mut self) -> Option<&mut usize> {
        match self.tab {
            Tab::Medals => Some(&mut self.medal),
            Tab::Achievements => Some(&mut self.achievement),
            Tab::Shaders => Some(&mut self.shader),
            Tab::Cards => Some(&mut self.card),
            Tab::Toys => Some(&mut self.toy),
            _ => None,
        }
    }

    /// Move the choice `by` things: wrapping round for Tab, stopping at the ends for
    /// the arrows.
    fn step(&mut self, by: isize, wrap: bool) {
        let count = self.things() as isize;
        let Some(chosen) = self.chosen_mut() else {
            return;
        };
        if count == 0 {
            return;
        }
        let at = (*chosen as isize).min(count - 1);
        let next = if wrap {
            (at + by).rem_euclid(count)
        } else {
            (at + by).clamp(0, count - 1)
        };
        *chosen = next as usize;
    }

    /// A key; `illuminate` is whether the holocron is lit.
    pub(crate) fn handle_key(
        &mut self,
        event: &KeyEvent,
        shift: bool,
        control: bool,
        illuminate: bool,
    ) -> PanelAction {
        if event.state != ElementState::Pressed {
            return PanelAction::None;
        }
        let PhysicalKey::Code(key) = event.physical_key else {
            return PanelAction::None;
        };
        self.hover.keyed();
        if self.tab == Tab::Forge {
            let action = self
                .forge
                .key(event.text.as_deref(), key, shift, control, self.grid);
            if let PanelAction::ShaderView(grid) = action {
                return self.set_grid(grid);
            }
            return action;
        }
        if self.tab == Tab::Cards && self.card_inspect {
            return self.key(key, shift, illuminate);
        }
        if self.tab == Tab::Cards && !control {
            let faces = crate::cards::faces::current();
            let filter = &mut self.card_filter;
            match key {
                KeyCode::KeyO => filter.cycle_show(shift),
                KeyCode::KeyT => filter.cycle_rarity(shift),
                KeyCode::KeyK => filter.cycle_kind(&faces, shift),
                KeyCode::KeyF => filter.cycle_finish(shift),
                KeyCode::KeyI => return self.activate(illuminate),
                _ => return self.key(key, shift, illuminate),
            }
            return self.card_filter_changed();
        }
        if self.tab == Tab::Shaders {
            if self.search_focus {
                return self.search_key(event, key, shift, control);
            }
            match key {
                KeyCode::Slash => return self.focus_search(),
                KeyCode::KeyF if control => return self.focus_search(),
                KeyCode::KeyT => return self.cycle_tier(shift),
                KeyCode::KeyO => return self.toggle_owned(),
                _ => {}
            }
        }
        self.key(key, shift, illuminate)
    }

    /// The skin of shader row `row` in the list the tab shows: `None` for the stock blade
    /// (row 0) or past the end.
    fn skin_at(&self, row: usize) -> Option<&'static unlockables::Unlockable> {
        row.checked_sub(1)
            .and_then(|index| self.shader_list.get(index))
            .copied()
    }

    /// The list changed: back to the top, and the rows read again next frame.
    fn filter_changed(&mut self) -> PanelAction {
        self.shader = 0;
        self.shader_first = 0;
        self.grid_first = 0;
        PanelAction::None
    }

    /// The Cards tab's filter changed: back to its first tile.
    /// Go `by` tiles on from the card looked at closely, stopping at the ends; the grid
    /// follows it when it is put back.
    fn inspect_step(&mut self, by: isize) {
        let last = self.cards_shown.saturating_sub(1) as isize;
        self.card = (self.card as isize + by).clamp(0, last) as usize;
    }

    fn card_filter_changed(&mut self) -> PanelAction {
        self.card = 0;
        self.card_first = 0;
        PanelAction::None
    }

    /// The search field lets go of the typing (what it holds stays).
    fn leave_search(&mut self) {
        if self.search_focus {
            self.search_focus = false;
            crate::text_entry::deselect();
        }
    }

    /// Put the typing in the search field (`/` or Ctrl+F).
    fn focus_search(&mut self) -> PanelAction {
        self.search_focus = true;
        crate::text_entry::select_all(&self.shader_filter.query);
        PanelAction::None
    }

    /// Show one tier of shaders: the next chip (the previous with Shift), All after the
    /// last.
    fn cycle_tier(&mut self, back: bool) -> PanelAction {
        let at = self
            .shader_filter
            .tier
            .and_then(|tier| FILTER_TIERS.iter().position(|&t| t == tier))
            .map_or(0, |index| index + 1);
        let count = FILTER_TIERS.len() + 1;
        let next = if back {
            (at + count - 1) % count
        } else {
            (at + 1) % count
        };
        self.shader_filter.tier = next.checked_sub(1).map(|index| FILTER_TIERS[index]);
        self.filter_changed()
    }

    /// Show only the shaders held, or all again.
    fn toggle_owned(&mut self) -> PanelAction {
        self.shader_filter.owned = !self.shader_filter.owned;
        self.filter_changed()
    }

    /// A key while the search field has the typing: Escape or Enter leave it (Escape
    /// first clears what was typed, as the other fields do), the rest edit it.
    fn search_key(
        &mut self,
        event: &KeyEvent,
        key: KeyCode,
        shift: bool,
        control: bool,
    ) -> PanelAction {
        match key {
            KeyCode::Escape => {
                self.leave_search();
                if !self.shader_filter.query.is_empty() {
                    self.shader_filter.query.clear();
                    return self.filter_changed();
                }
            }
            KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Tab => self.leave_search(),
            KeyCode::ArrowUp | KeyCode::ArrowDown => {
                return self.key(key, shift, false);
            }
            _ => {
                if crate::text_entry::edit(
                    &mut self.shader_filter.query,
                    key,
                    event.text.as_deref(),
                    shift,
                    control,
                    SEARCH_MAX,
                    true,
                ) {
                    return self.filter_changed();
                }
            }
        }
        PanelAction::None
    }

    /// Switch the Shaders tab to the grid of cards (`true`) or the rack.
    fn set_grid(&mut self, grid: bool) -> PanelAction {
        self.grid = grid;
        PanelAction::ShaderView(grid)
    }

    /// A pressed key, by its code.
    fn key(&mut self, key: KeyCode, shift: bool, illuminate: bool) -> PanelAction {
        let vertical = self.tab == Tab::Shaders;
        if self.tab == Tab::Cards && self.card_inspect {
            // Looking at a card closely: the arrows go to the previous or next tile,
            // Escape, Enter, Space and I put it back.
            match key {
                KeyCode::ArrowLeft | KeyCode::ArrowUp | KeyCode::PageUp => self.inspect_step(-1),
                KeyCode::ArrowRight | KeyCode::ArrowDown | KeyCode::PageDown => {
                    self.inspect_step(1);
                }
                KeyCode::Home => self.card = 0,
                KeyCode::End => self.card = self.cards_shown.saturating_sub(1),
                KeyCode::Escape
                | KeyCode::Enter
                | KeyCode::NumpadEnter
                | KeyCode::Space
                | KeyCode::KeyI => self.card_inspect = false,
                _ => {}
            }
            return PanelAction::None;
        }
        if self.tab == Tab::Cards && self.cards_shown > 0 {
            // The binder: Left and Right step a tile, Up and Down a line, stopping at the
            // ends (a short last line takes Down to its last tile); Page Up and Page Down
            // a screen of lines, Home and End to the first and last tile.
            let columns = cards::COLUMNS;
            let page = columns * cards::LINES;
            let last = self.cards_shown - 1;
            let at = self.card.min(last);
            match key {
                KeyCode::ArrowLeft => self.card = at.saturating_sub(1),
                KeyCode::ArrowRight => self.card = (at + 1).min(last),
                KeyCode::ArrowUp => self.card = at.checked_sub(columns).unwrap_or(at),
                KeyCode::ArrowDown if at / columns < last / columns => {
                    self.card = (at + columns).min(last);
                }
                KeyCode::ArrowDown => {}
                KeyCode::PageUp => self.card = at.saturating_sub(page.min(at / columns * columns)),
                KeyCode::PageDown if at / columns < last / columns => {
                    self.card =
                        (at + page.min((last / columns - at / columns) * columns)).min(last);
                }
                KeyCode::PageDown => {}
                KeyCode::Home => self.card = 0,
                KeyCode::End => self.card = last,
                _ => return self.key_any(key, shift, illuminate, vertical),
            }
            return PanelAction::None;
        }
        if self.tab == Tab::Shaders && self.grid {
            // The grid: Left and Right step a card, Up and Down a line, stopping at
            // the ends (a short last line takes Down to its last card).
            let last = self.shader_count - 1;
            match key {
                KeyCode::ArrowLeft => self.shader = self.shader.min(last).saturating_sub(1),
                KeyCode::ArrowRight => self.shader = (self.shader + 1).min(last),
                KeyCode::ArrowUp => {
                    self.shader = self.shader.checked_sub(GRID_COLUMNS).unwrap_or(self.shader);
                }
                KeyCode::ArrowDown if self.shader / GRID_COLUMNS < last / GRID_COLUMNS => {
                    self.shader = (self.shader + GRID_COLUMNS).min(last);
                }
                _ => {}
            }
            if matches!(
                key,
                KeyCode::ArrowLeft | KeyCode::ArrowRight | KeyCode::ArrowUp | KeyCode::ArrowDown
            ) {
                return PanelAction::None;
            }
        }
        self.key_any(key, shift, illuminate, vertical)
    }

    /// A pressed key the tab's own grid did not take.
    fn key_any(
        &mut self,
        key: KeyCode,
        shift: bool,
        illuminate: bool,
        vertical: bool,
    ) -> PanelAction {
        match key {
            KeyCode::Escape => return PanelAction::Close,
            KeyCode::KeyV if self.tab == Tab::Shaders => return self.set_grid(!self.grid),
            KeyCode::KeyR if self.tab == Tab::Cards => return self.reveal(),
            KeyCode::Tab => self.step(if shift { -1 } else { 1 }, true),
            KeyCode::ArrowLeft if !vertical => self.step(-1, false),
            KeyCode::ArrowRight if !vertical => self.step(1, false),
            KeyCode::ArrowUp | KeyCode::ArrowDown if self.tab == Tab::Achievements => {
                self.achievement =
                    achievements::vertical(self.achievement, key == KeyCode::ArrowDown);
            }
            KeyCode::ArrowUp | KeyCode::ArrowLeft => self.step(-1, false),
            KeyCode::ArrowDown | KeyCode::ArrowRight => self.step(1, false),
            KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space => {
                return self.activate(illuminate);
            }
            // 1 to 6 switch the chosen shader's options.
            KeyCode::Digit1
            | KeyCode::Digit2
            | KeyCode::Digit3
            | KeyCode::Digit4
            | KeyCode::Digit5
            | KeyCode::Digit6
                if self.tab == Tab::Shaders =>
            {
                let index = match key {
                    KeyCode::Digit1 => 0,
                    KeyCode::Digit2 => 1,
                    KeyCode::Digit3 => 2,
                    KeyCode::Digit4 => 3,
                    KeyCode::Digit5 => 4,
                    _ => 5,
                };
                return self.switch_option(index);
            }
            _ => {}
        }
        PanelAction::None
    }

    /// Switch the chosen shader's option `index` (as the last frame showed them) the
    /// other way.
    fn switch_option(&self, index: usize) -> PanelAction {
        self.shader_options
            .get(index)
            .map_or(PanelAction::None, |(skin, option, on)| {
                PanelAction::SkinOption {
                    skin,
                    option: option.clone(),
                    on: !on,
                }
            })
    }

    /// A pointer event; `illuminate` is whether the holocron is lit.
    pub(crate) fn handle_pointer(&mut self, event: InputEvent, illuminate: bool) -> PanelAction {
        self.pointer_at(event, illuminate, Instant::now())
    }

    /// A pointer event at `now`. A second click on a shader row within
    /// [`DOUBLE_CLICK`] wears it, as Equip does (an owned one only; a locked one stays a
    /// preview, the worn one stays worn).
    fn pointer_at(&mut self, event: InputEvent, illuminate: bool, now: Instant) -> PanelAction {
        self.hover.moved();
        match event {
            InputEvent::PointerMove(at)
            | InputEvent::PointerPress { position: at, .. }
            | InputEvent::PointerRelease { position: at, .. }
            | InputEvent::PointerWheel { position: at, .. } => self.pointer = Some([at.x, at.y]),
            InputEvent::PointerLeave => self.pointer = None,
            InputEvent::Action(_) => {}
        }
        // The lone saber turns with a drag that starts on it, wherever it goes after.
        if self.tab == Tab::Shaders {
            match event {
                InputEvent::PointerPress { position: at, .. }
                    if self.backstage != Backstage::None
                        && self
                            .ui
                            .rect_for(SABER_TOKEN)
                            .is_some_and(|rect| rect.contains(at)) =>
                {
                    self.spin.grab([at.x, at.y]);
                    return PanelAction::None;
                }
                InputEvent::PointerMove(at) if self.spin.held() => {
                    self.spin.drag([at.x, at.y], self.scale);
                    return PanelAction::None;
                }
                InputEvent::PointerRelease { .. } | InputEvent::PointerLeave
                    if self.spin.held() =>
                {
                    self.spin.release(self.epoch.elapsed().as_secs_f32());
                    return PanelAction::None;
                }
                _ => {}
            }
        }
        if let InputEvent::PointerWheel { delta, .. } = event
            && self.tab == Tab::Forge
        {
            if delta.y != 0.0 && self.forge.ask.is_none() {
                self.forge.wheel(delta.y > 0.0, self.grid);
            }
            return PanelAction::None;
        }
        if let InputEvent::PointerWheel { delta, .. } = event
            && self.tab == Tab::Cards
        {
            if self.card_inspect {
                // The wheel goes to the previous (up) or next (down) card.
                if delta.y != 0.0 {
                    self.inspect_step(if delta.y > 0.0 { -1 } else { 1 });
                }
                return PanelAction::None;
            }
            self.scroll_cards(delta.y);
            return PanelAction::None;
        }
        if let InputEvent::PointerWheel { delta, .. } = event
            && self.tab == Tab::Shaders
        {
            if self.grid {
                // The wheel scrolls the grid a line a notch; the chosen card stays in
                // view.
                let lines = self.shader_count.div_ceil(GRID_COLUMNS);
                let last = lines.saturating_sub(GRID_LINES_SHOWN);
                self.grid_first = if delta.y > 0.0 {
                    self.grid_first.saturating_sub(1)
                } else if delta.y < 0.0 {
                    (self.grid_first + 1).min(last)
                } else {
                    self.grid_first
                };
                let line = (self.shader / GRID_COLUMNS)
                    .clamp(self.grid_first, self.grid_first + GRID_LINES_SHOWN - 1);
                self.shader =
                    (line * GRID_COLUMNS + self.shader % GRID_COLUMNS).min(self.shader_count - 1);
                return PanelAction::None;
            }
            // The wheel scrolls the rack a row a notch; the chosen row stays in view.
            let last = self.shader_count.saturating_sub(RACK_SHOWN);
            self.shader_first = if delta.y > 0.0 {
                self.shader_first.saturating_sub(1)
            } else if delta.y < 0.0 {
                (self.shader_first + 1).min(last)
            } else {
                self.shader_first
            };
            self.shader = self
                .shader
                .clamp(self.shader_first, self.shader_first + RACK_SHOWN - 1);
            return PanelAction::None;
        }
        let Some(event) = self.ui.pointer(event) else {
            return PanelAction::None;
        };
        if event.kind == UiEventKind::Activate && event.token != Some(SEARCH_TOKEN) {
            self.leave_search();
        }
        let Some(token) = event.token else {
            return PanelAction::None;
        };
        if self.tab == Tab::Forge && (forge::forge_token(token) || self.forge.ask.is_some()) {
            let activate = event.kind == UiEventKind::Activate;
            let resting = matches!(event.kind, UiEventKind::HoverEnter | UiEventKind::Hover);
            if !(activate || resting) {
                return PanelAction::None;
            }
            return match self.forge.token(token, activate, now) {
                Some(PanelAction::ShaderView(grid)) => self.set_grid(grid),
                Some(action) => action,
                None => PanelAction::None,
            };
        }
        if event.kind == UiEventKind::Activate
            && let Some(index) = token
                .checked_sub(SHADER_BASE)
                .map(usize::from)
                .filter(|index| *index < self.shader_count)
        {
            let double = self
                .last_click
                .is_some_and(|(last, at)| last == token && now.duration_since(at) <= DOUBLE_CLICK);
            self.shader = index;
            if double {
                self.last_click = None;
                return self
                    .shader_rows
                    .get(index)
                    .filter(|row| !row.worn)
                    .and_then(|row| row.wear)
                    .map_or(PanelAction::None, PanelAction::Wear);
            }
            self.last_click = Some((token, now));
            return PanelAction::None;
        }
        if event.kind == UiEventKind::Activate {
            if let Some(index) = token
                .checked_sub(FILTER_TIER_BASE)
                .map(usize::from)
                .filter(|index| *index <= FILTER_TIERS.len())
            {
                self.shader_filter.tier = index.checked_sub(1).map(|index| FILTER_TIERS[index]);
                return self.filter_changed();
            }
            if token == FILTER_OWNED_TOKEN {
                return self.toggle_owned();
            }
            if let Some(show) = token
                .checked_sub(CARD_SHOW_BASE)
                .and_then(|index| crate::cards::binder::Show::ALL.get(usize::from(index)))
            {
                self.card_filter.show = *show;
                return self.card_filter_changed();
            }
            let faces = crate::cards::faces::current();
            match token {
                CARD_RARITY_TOKEN => self.card_filter.cycle_rarity(false),
                CARD_KIND_TOKEN => self.card_filter.cycle_kind(&faces, false),
                CARD_FINISH_TOKEN => self.card_filter.cycle_finish(false),
                _ => {}
            }
            if matches!(
                token,
                CARD_RARITY_TOKEN | CARD_KIND_TOKEN | CARD_FINISH_TOKEN
            ) {
                return self.card_filter_changed();
            }
            if token == SEARCH_TOKEN {
                return self.focus_search();
            }
            // Looking at a card closely: a click anywhere puts it back; a click on the
            // card up close or on a tile looks at it.
            if token == CARD_INSPECT_TOKEN {
                self.card_inspect = false;
                return PanelAction::None;
            }
            if token == CARD_CLOSE_UP_TOKEN {
                return self.activate(illuminate);
            }
            if token == CARD_SHADER_TOKEN {
                return self.go_to_card_shader();
            }
            if self.tab == Tab::Cards && self.choose_token(token) {
                return self.activate(illuminate);
            }
        }
        let choose = matches!(
            event.kind,
            UiEventKind::HoverEnter | UiEventKind::Hover | UiEventKind::Activate
        );
        if choose && self.choose_token(token) {
            return PanelAction::None;
        }
        if event.kind != UiEventKind::Activate {
            return PanelAction::None;
        }
        if let Some(index) = token
            .checked_sub(OPTION_BASE)
            .map(usize::from)
            .filter(|index| *index < OPTION_TOKENS)
        {
            return self.switch_option(index);
        }
        match token {
            BACK_TOKEN => PanelAction::Close,
            LIST_VIEW_TOKEN => self.set_grid(false),
            GRID_VIEW_TOKEN => self.set_grid(true),
            WEAR_TOKEN | TOY_SWITCH_TOKEN | SHARE_TOKEN => self.activate(illuminate),
            FORGE_LINK_TOKEN => self.open_forge_link(),
            // A toy's niche chooses it; the chosen one's niche runs it, as its switch does.
            TOY_TOKEN | BOX_TOKEN | ORDER_TOKEN | HAX_TOKEN | RAGDOLL_TOKEN | DRONE_TOKEN => {
                let toy = TOY_TOKENS
                    .iter()
                    .position(|&niche| niche == token)
                    .unwrap_or(0);
                if self.toy == toy {
                    self.activate(illuminate)
                } else {
                    self.toy = toy;
                    PanelAction::None
                }
            }
            REVEAL_TOKEN => self.reveal(),
            token => match crate::profile_hub::collection_tab_of(token) {
                Some(Tab::Holocrons) => PanelAction::Hub(Tab::Holocrons),
                Some(tab) => {
                    self.tab = tab;
                    PanelAction::None
                }
                None => PanelAction::None,
            },
        }
    }

    /// The blade card up close's button: the Forge's Craft on its shader while it is one
    /// of the trade the player lacks (there its copy crafts it for less), else the Shaders
    /// tab on it; any filter there is cleared so it is in the list.
    fn go_to_card_shader(&mut self) -> PanelAction {
        let Some(skin) = self.card_blade else {
            return PanelAction::None;
        };
        if self.card_blade_craft {
            self.leave_search();
            self.forge_on(forge::Section::Craft, skin.id);
            return PanelAction::None;
        }
        self.show(Tab::Shaders);
        self.shader_filter = ShaderFilter::default();
        self.shader_focus = Some(skin.id);
        PanelAction::None
    }

    /// The Shaders tab's way to the Forge: Craft for a shader of the trade not held, Break
    /// down for one of the player's own copies, the shader chosen there.
    fn open_forge_link(&mut self) -> PanelAction {
        if let Some((section, id)) = self.forge_link {
            self.leave_search();
            self.forge_on(section, id);
            ui_cues_click();
        }
        PanelAction::None
    }

    /// Reveal the sealed cards, when the last frame offered it: the line under the button
    /// waits for the identity's answer.
    fn reveal(&mut self) -> PanelAction {
        if self.revealable == 0 {
            return PanelAction::None;
        }
        self.reveal_asked =
            Some(crate::player_identity::reveal_outcome().map_or(0, |outcome| outcome.serial));
        PanelAction::RevealCards
    }

    /// Choose the thing `token` stands for; whether it is one.
    fn choose_token(&mut self, token: u16) -> bool {
        let within = |base: u16, count: usize| {
            token
                .checked_sub(base)
                .map(usize::from)
                .filter(|index| *index < count)
        };
        if let Some(index) = within(MEDAL_BASE, self.medals_shown) {
            self.medal = index;
        } else if let Some(index) = within(ACHIEVEMENT_BASE, crate::achievements::ALL.len()) {
            self.achievement = index;
        } else if let Some(index) = within(SHADER_BASE, self.shader_count) {
            self.shader = index;
        } else if let Some(index) = within(CARD_BASE, self.cards_shown) {
            self.card = index;
        } else {
            return false;
        }
        true
    }

    /// Choose the Cards tab's tile `index` (in the list the filter shows), for a world
    /// shot.
    #[cfg(test)]
    pub(crate) fn choose_card_for_shot(&mut self, index: usize) {
        self.card = index;
    }

    /// Look at the Cards tab's chosen tile closely (or put it back), for a world shot.
    #[cfg(test)]
    pub(crate) fn inspect_card_for_shot(&mut self, on: bool) {
        self.card_inspect = on;
    }

    /// Set the Cards tab's filter, for a world shot.
    #[cfg(test)]
    pub(crate) fn card_filter_for_shot(&mut self, filter: crate::cards::binder::Filter) {
        self.card_filter = filter;
        let _ = self.card_filter_changed();
    }

    /// Tilt card `key` (0 up close, 1 and up the grid's) as though the pointer had rested
    /// at `at` on it for a while, for a world shot.
    #[cfg(test)]
    pub(crate) fn tilt_card_for_shot(&mut self, key: usize, at: [f32; 2]) {
        self.tilt.rest(key, at, self.shot_seconds.unwrap_or(0.0));
    }

    /// Show the hover card of the tab's thing `index` (the toy for Toys; `None` for none),
    /// for a world shot.
    #[cfg(test)]
    pub(crate) fn hover_for_shot(&mut self, index: Option<usize>) {
        let base = match self.tab {
            Tab::Medals => MEDAL_BASE,
            Tab::Achievements => ACHIEVEMENT_BASE,
            Tab::Shaders => SHADER_BASE,
            Tab::Cards => CARD_BASE,
            // The Forge's shots name the token itself.
            Tab::Forge => 0,
            _ => TOY_TOKEN,
        };
        self.hover.pin(index.map(|index| base + index as u16));
    }

    /// Hold the lone saber turned so, still, as a drag would (`None` lets go), for a
    /// world shot.
    #[cfg(test)]
    pub(crate) fn hold_saber_for_shot(&mut self, turn: Option<SaberTurn>) {
        match turn {
            Some(turn) => self.spin.hold(turn),
            None => self.spin.release(f32::NEG_INFINITY),
        }
    }

    /// Choose shader row `row` (0 the stock blade), for a world shot.
    #[cfg(test)]
    pub(crate) fn choose_shader_for_shot(&mut self, row: usize) {
        self.shader = row.min(SHADER_ROWS - 1);
    }

    /// The token of the thing chosen on the tab on show, for the canvas's focus.
    fn focus_token(&self) -> u16 {
        match self.tab {
            Tab::Medals => MEDAL_BASE + self.medal as u16,
            Tab::Achievements => {
                ACHIEVEMENT_BASE + self.achievement.min(crate::achievements::ALL.len() - 1) as u16
            }
            Tab::Shaders => SHADER_BASE + self.shader as u16,
            Tab::Toys => TOY_TOKENS[self.toy.min(TOY_COUNT - 1)],
            Tab::Cards if self.cards_shown > 0 => CARD_BASE + self.card as u16,
            Tab::Nameplates => SHARE_TOKEN,
            Tab::Forge => match self.forge.ask {
                Some(ask) => {
                    if matches!(
                        ask,
                        forge::Ask::Last { yes: true, .. } | forge::Ask::All { yes: true }
                    ) {
                        forge::ASK_YES_TOKEN
                    } else {
                        forge::ASK_NO_TOKEN
                    }
                }
                None => BACK_TOKEN,
            },
            _ => BACK_TOKEN,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sjk_identity::Unlock;

    fn sun() -> Unlock {
        Unlock {
            id: "saber_sun".into(),
            granted: 1_791_336_225,
            note: "Thanks for testing the blade".into(),
            count: 1,
            medal: None,
            card: None,
        }
    }

    pub(super) fn inputs<'a>(holdings: Holdings<'a>, setting: &'a str) -> Inputs<'a> {
        Inputs {
            enabled: true,
            snapshot: None,
            holdings,
            setting,
            standings: &[],
            illuminate: true,
            cardboard_box: false,
            order66: false,
            hax: false,
            ragdoll: false,
            drone: false,
            name: "^1Sol^7Vulpes",
            stock: sjk_ui::Color::new(0.16, 0.48, 1.0, 1.0),
            grid: false,
            options: "",
            share: true,
        }
    }

    /// Hub report #108 (Nib): the Toys tab counts every toy, all of them held.
    #[test]
    fn the_toys_tab_counts_every_toy() {
        let counts = view::counts(&inputs(Holdings::Known(&[]), ""));
        let toys = &counts[3];
        assert_eq!((toys.held, toys.total), (Some(TOY_COUNT), TOY_COUNT));
        assert_eq!(TOY_COUNT, TOY_TOKENS.len());
    }

    /// The shader row of blade skin `id`.
    pub(super) fn row_of(id: &str) -> usize {
        (1..SHADER_ROWS)
            .find(|&row| shader_skin(row).is_some_and(|skin| skin.id == id))
            .unwrap()
    }

    /// The panel open on `tab` after one frame of `inputs`, so its things are known.
    pub(super) fn drawn(tab: Tab, inputs: &Inputs<'_>) -> Panel {
        let mut panel = Panel::new();
        panel.open(tab, true, true, ReturnTarget::MainMenu);
        let fonts = crate::text::load_modern(1.0, None).expect("Inter");
        panel.build(inputs, &fonts.font, [1920.0, 1080.0]);
        panel
    }

    /// Redraw the Shaders tab of `panel` with `inputs`.
    fn redraw(panel: &mut Panel, inputs: &Inputs<'_>) {
        let fonts = crate::text::load_modern(1.0, None).expect("Inter");
        panel.build(inputs, &fonts.font, [1920.0, 1080.0]);
        assert!(!panel.overflowed(), "the filter fits the canvas");
    }

    fn ids(panel: &Panel) -> Vec<&'static str> {
        panel.shader_list.iter().map(|skin| skin.id).collect()
    }

    /// A blade's number in the search lists that blade alone.
    #[test]
    fn a_number_searches_for_that_blade_alone() {
        let filter = ShaderFilter {
            query: "#2".to_owned(),
            ..ShaderFilter::default()
        };
        let found: Vec<_> = unlockables::blade_skins()
            .filter(|skin| filter.passes(skin, false))
            .map(|skin| skin.number)
            .collect();
        assert_eq!(found, [2]);
    }

    /// A tier chip lists that tier's shaders alone; cycling comes round to All again.
    #[test]
    fn the_tier_filter_lists_one_tier_and_cycling_comes_round() {
        let held = [sun()];
        let inputs = inputs(Holdings::Known(&held), "");
        let mut panel = drawn(Tab::Shaders, &inputs);
        let all = ids(&panel);
        assert_eq!(panel.shader_count, 1 + unlockables::ALL.len());
        panel.shader = 9;
        let _ = panel.cycle_tier(false);
        assert_eq!(panel.shader, 0, "back to the top");
        redraw(&mut panel, &inputs);
        assert_eq!(ids(&panel).len(), 43);
        assert_eq!(ids(&panel)[0], "saber_frost");
        assert_eq!(panel.shader_count, 44, "the stock blade stays first");
        for _ in 0..5 {
            let _ = panel.cycle_tier(false);
        }
        redraw(&mut panel, &inputs);
        assert_eq!(ids(&panel), all, "All again after the last tier");
        let _ = panel.cycle_tier(true);
        redraw(&mut panel, &inputs);
        assert_eq!(
            panel.shader_filter.tier,
            Some(unlockables::Rarity::Mythical)
        );
    }

    /// A hover card under a filter is the card of the row shown, not of the catalogue's
    /// row at that place; a click elsewhere or another tab lets go of the search.
    #[test]
    fn the_hover_card_follows_the_filter_and_the_search_lets_go() {
        let held = [sun()];
        let inputs = inputs(Holdings::Known(&held), "");
        let mut panel = drawn(Tab::Shaders, &inputs);
        panel.shader_filter.query = "kamino".to_owned();
        redraw(&mut panel, &inputs);
        let card = panel.card_of(SHADER_BASE + 1, &inputs).expect("a card");
        assert_eq!(card.title, "Kamino blade");
        assert!(
            panel.card_of(SHADER_BASE + 2, &inputs).is_none(),
            "past the list"
        );
        let _ = panel.focus_search();
        panel.show(Tab::Medals);
        assert!(!panel.search_focus);
        panel.show(Tab::Shaders);
        let _ = panel.focus_search();
        panel.leave_search();
        assert!(!panel.search_focus);
        assert_eq!(panel.shader_filter.query, "kamino", "what it holds stays");
    }

    /// Owned shows the shaders held; a search matches names and tier words.
    #[test]
    fn owned_and_search_narrow_the_list() {
        let held = [sun()];
        let inputs = inputs(Holdings::Known(&held), "");
        let mut panel = drawn(Tab::Shaders, &inputs);
        let _ = panel.toggle_owned();
        redraw(&mut panel, &inputs);
        assert_eq!(ids(&panel), ["saber_sun"]);
        let _ = panel.toggle_owned();
        panel.shader_filter.query = " KAMINO ".to_owned();
        redraw(&mut panel, &inputs);
        assert_eq!(ids(&panel), ["saber_tidal"]);
        panel.shader_filter.query = "creyon".to_owned();
        redraw(&mut panel, &inputs);
        assert_eq!(ids(&panel).len(), 151, "the saber smith is searched");
        panel.shader_filter.query = "mythical".to_owned();
        redraw(&mut panel, &inputs);
        assert_eq!(
            ids(&panel),
            [
                "saber_sun",
                "saber_void",
                "saber_comic",
                "saber_obsidian",
                "saber_abyssal_eye",
                "saber_carbonite",
                "saber_eclipse",
                "saber_event_horizon",
                "saber_exogorth",
                "saber_world_shift",
                "saber_exegol_rift",
                "saber_varykino_blossom",
            ]
        );
        panel.shader_filter.query = "nothing".to_owned();
        redraw(&mut panel, &inputs);
        assert!(ids(&panel).is_empty());
        assert_eq!(panel.shader_count, 1, "only the stock blade is left");
        assert!(panel.shader_filter.active());
        panel.shader_filter = ShaderFilter::default();
        assert!(!panel.shader_filter.active());
    }

    /// R on Cards (or Reveal) asks the hub only when the last frame offered something to
    /// reveal, and then waits for the answer.
    #[test]
    fn reveal_asks_only_when_there_is_something_to_reveal() {
        let mut panel = drawn(Tab::Cards, &inputs(Holdings::Known(&[]), ""));
        assert_eq!(panel.revealable, 0, "no identity, nothing to reveal");
        assert_eq!(panel.key(KeyCode::KeyR, false, true), PanelAction::None);
        assert_eq!(panel.reveal_asked, None);
        panel.revealable = 3;
        assert_eq!(
            panel.key(KeyCode::KeyR, false, true),
            PanelAction::RevealCards
        );
        assert_eq!(panel.reveal_asked, Some(0), "no reveal before this one");
        // Not on another tab.
        panel.tab = Tab::Medals;
        assert_eq!(panel.key(KeyCode::KeyR, false, true), PanelAction::None);
    }

    #[test]
    fn shaders_open_on_what_is_worn_and_enter_equips_and_unequips() {
        let owned = [sun()];
        let mut panel = drawn(Tab::Shaders, &inputs(Holdings::Known(&owned), "saber_sun"));
        let sun_row = row_of("saber_sun");
        assert_eq!(panel.shader, sun_row, "the worn one is chosen");
        assert_eq!(
            panel.stage_wish().map(|wish| wish.skin),
            Some(PreviewSkin::Skin("saber_sun"))
        );
        assert_eq!(
            panel.key(KeyCode::Enter, false, true),
            PanelAction::Wear("")
        );
        // The stock row wears the stock blade; the Sun row, once not worn, the Sun.
        let fonts = crate::text::load_modern(1.0, None).expect("Inter");
        panel.build(
            &inputs(Holdings::Known(&owned), ""),
            &fonts.font,
            [1920.0, 1080.0],
        );
        assert_eq!(
            panel.key(KeyCode::Enter, false, true),
            PanelAction::Wear("saber_sun")
        );
        let _ = panel.key(KeyCode::ArrowUp, false, true);
        assert_eq!(panel.shader, 0);
        assert_eq!(
            panel.stage_wish().map(|wish| wish.skin),
            Some(PreviewSkin::Stock)
        );
        assert_eq!(
            panel.key(KeyCode::Enter, false, true),
            PanelAction::None,
            "worn already"
        );
        assert_eq!(panel.key(KeyCode::Escape, false, true), PanelAction::Close);
    }

    #[test]
    fn a_locked_shader_is_previewed_but_never_equipped() {
        for holdings in [
            Holdings::Known(&[]),
            Holdings::IdentityOff,
            Holdings::Waiting,
        ] {
            let mut panel = drawn(Tab::Shaders, &inputs(holdings, ""));
            let _ = panel.key(KeyCode::ArrowDown, false, true);
            let id = shader_skin(1).unwrap().id;
            assert_eq!(
                panel.stage_wish().map(|wish| wish.skin),
                Some(PreviewSkin::Skin(id)),
                "{holdings:?}"
            );
            assert_eq!(panel.key(KeyCode::Enter, false, true), PanelAction::None);
            assert!(panel.ui.rect_for(WEAR_TOKEN).is_none(), "no Equip");
        }
    }

    #[test]
    fn the_toy_lights_and_puts_out_illuminate_and_only_the_kit_tabs_want_the_stage() {
        let mut panel = drawn(Tab::Toys, &inputs(Holdings::Known(&[]), ""));
        assert_eq!(
            panel.key(KeyCode::Enter, false, true),
            PanelAction::Illuminate(false)
        );
        assert_eq!(
            panel.key(KeyCode::Space, false, false),
            PanelAction::Illuminate(true)
        );
        assert!(panel.stage_wish().is_some_and(|wish| wish.holocron));
        for (tab, wants) in [
            (Tab::Medals, false),
            (Tab::Achievements, false),
            (Tab::Shaders, true),
            (Tab::Toys, true),
            (Tab::Nameplates, true),
        ] {
            panel.show(tab);
            assert_eq!(panel.stage_wish().is_some(), wants, "{tab:?}");
        }
        // A tab of the Profile screen is not the page's.
        panel.show(Tab::Saber);
        assert_eq!(panel.tab(), Tab::Nameplates);
        let _ = panel.close();
        assert_eq!(panel.stage_wish(), None);
    }

    #[test]
    fn the_keys_stay_on_the_things_and_tab_comes_round() {
        let mut panel = drawn(Tab::Shaders, &inputs(Holdings::Known(&[]), ""));
        assert_eq!(panel.shader, 0, "the stock blade, worn");
        let _ = panel.key(KeyCode::ArrowUp, false, true);
        assert_eq!(panel.shader, 0, "the arrows stop at the ends");
        for _ in 0..SHADER_ROWS + 2 {
            let _ = panel.key(KeyCode::ArrowDown, false, true);
        }
        assert_eq!(panel.shader, SHADER_ROWS - 1);
        let _ = panel.key(KeyCode::Tab, false, true);
        assert_eq!(panel.shader, 0, "Tab comes round");
        let _ = panel.key(KeyCode::Tab, true, true);
        assert_eq!(panel.shader, SHADER_ROWS - 1);
    }

    /// The Toys tab's switch and plinth light the holocron when it is out and put it out
    /// when it is lit, whichever state the page was told, and its words name the new
    /// command, not the Force wheel.
    #[test]
    fn the_toys_switch_lights_or_puts_out_by_the_state_it_is_told() {
        let click = |panel: &mut Panel, token: u16, lit: bool| {
            let rect = panel.ui.rect_for(token).expect("a pointer area");
            let at = sjk_ui::Vec2::new(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5);
            let mut action = PanelAction::None;
            for event in [
                InputEvent::PointerMove(at),
                InputEvent::PointerPress {
                    position: at,
                    button: sjk_ui::PointerButton::Primary,
                },
                InputEvent::PointerRelease {
                    position: at,
                    button: sjk_ui::PointerButton::Primary,
                },
            ] {
                let next = panel.handle_pointer(event, lit);
                if next != PanelAction::None {
                    action = next;
                }
            }
            action
        };
        for (token, lit, wanted) in [
            (TOY_SWITCH_TOKEN, false, true),
            (TOY_SWITCH_TOKEN, true, false),
            (TOY_TOKEN, false, true),
        ] {
            let mut shown = inputs(Holdings::Known(&[]), "");
            shown.illuminate = lit;
            let mut panel = drawn(Tab::Toys, &shown);
            assert_eq!(
                click(&mut panel, token, lit),
                PanelAction::Illuminate(wanted),
                "token {token}, lit {lit}"
            );
            let all = panel.ui.text_runs().collect::<Vec<_>>().join(
                "
",
            );
            assert!(all.contains("toy_illuminate"), "{all}");
            assert!(all.contains("Holocron lit"), "{all}");
            assert!(
                !all.contains("Force wheel") && !all.contains("force_illuminate"),
                "{all}"
            );
        }
    }

    /// The box's niche chooses it, then runs it; Enter and the switch get in or out by
    /// the state the page was told, and the stage holds no holocron for it.
    #[test]
    fn the_cardboard_box_is_chosen_then_put_on_or_taken_off() {
        let mut shown = inputs(Holdings::Known(&[]), "");
        let mut panel = drawn(Tab::Toys, &shown);
        let _ = panel.key(KeyCode::ArrowRight, false, false);
        assert_eq!(panel.toy, 1);
        assert_eq!(
            panel.key(KeyCode::Enter, false, false),
            PanelAction::CardboardBox(true)
        );
        assert!(panel.stage_wish().is_some_and(|wish| !wish.holocron));
        shown.cardboard_box = true;
        let mut panel = drawn(Tab::Toys, &shown);
        let _ = panel.key(KeyCode::ArrowRight, false, false);
        redraw(&mut panel, &shown);
        assert_eq!(
            panel.key(KeyCode::Enter, false, false),
            PanelAction::CardboardBox(false)
        );
        let all = panel.ui.text_runs().collect::<Vec<_>>().join("\n");
        assert!(
            all.contains("toy_box") && all.contains("In the box"),
            "{all}"
        );
        let _ = panel.key(KeyCode::ArrowLeft, false, false);
        assert_eq!(
            panel.key(KeyCode::Enter, false, true),
            PanelAction::Illuminate(false)
        );
    }

    /// Order 66 is the third toy: chosen, then turned on or off by the state the page
    /// was told, its niche running it once chosen.
    #[test]
    fn order_66_is_chosen_then_turned_on_or_off() {
        let mut shown = inputs(Holdings::Known(&[]), "");
        let mut panel = drawn(Tab::Toys, &shown);
        let _ = panel.key(KeyCode::ArrowRight, false, false);
        let _ = panel.key(KeyCode::ArrowRight, false, false);
        assert_eq!(panel.toy, 2);
        assert_eq!(
            panel.key(KeyCode::Enter, false, false),
            PanelAction::Order66(true)
        );
        assert!(panel.stage_wish().is_some_and(|wish| !wish.holocron));
        shown.order66 = true;
        let click = |panel: &mut Panel, token: u16| {
            let rect = panel.ui.rect_for(token).expect("a pointer area");
            let at = sjk_ui::Vec2::new(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5);
            let button = sjk_ui::PointerButton::Primary;
            [
                InputEvent::PointerMove(at),
                InputEvent::PointerPress {
                    position: at,
                    button,
                },
                InputEvent::PointerRelease {
                    position: at,
                    button,
                },
            ]
            .into_iter()
            .map(|event| panel.pointer_at(event, true, Instant::now()))
            .find(|action| *action != PanelAction::None)
            .unwrap_or(PanelAction::None)
        };
        let mut panel = drawn(Tab::Toys, &shown);
        assert_eq!(click(&mut panel, ORDER_TOKEN), PanelAction::None, "chosen");
        redraw(&mut panel, &shown);
        assert_eq!(click(&mut panel, ORDER_TOKEN), PanelAction::Order66(false));
        let all = panel.ui.text_runs().collect::<Vec<_>>().join("\n");
        assert!(
            all.contains("toy_order66") && all.contains("In the order"),
            "{all}"
        );
    }

    /// The Nameplates tab's See and share nameplates switch, clicked or with Enter, flips the
    /// state the page was told, and says what off means.
    #[test]
    fn the_share_switch_flips_sharing() {
        let mut shown = inputs(Holdings::Known(&[]), "");
        let mut panel = drawn(Tab::Nameplates, &shown);
        let rect = panel.ui.rect_for(SHARE_TOKEN).expect("the switch");
        let at = sjk_ui::Vec2::new(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5);
        let button = sjk_ui::PointerButton::Primary;
        let action = [
            InputEvent::PointerMove(at),
            InputEvent::PointerPress {
                position: at,
                button,
            },
            InputEvent::PointerRelease {
                position: at,
                button,
            },
        ]
        .into_iter()
        .map(|event| panel.pointer_at(event, true, Instant::now()))
        .find(|action| *action != PanelAction::None);
        assert_eq!(action, Some(PanelAction::SeeAndShare(false)));
        shown.share = false;
        redraw(&mut panel, &shown);
        assert_eq!(
            panel.key(KeyCode::Enter, false, false),
            PanelAction::SeeAndShare(true)
        );
        let all = panel.ui.text_runs().collect::<Vec<_>>().join("\n");
        assert!(
            all.contains("See & share nameplates") && all.contains("you see no nameplates"),
            "{all}"
        );
    }

    #[test]
    fn a_double_click_wears_an_owned_shader_only() {
        let owned = [sun()];
        let click = |panel: &mut Panel, token: u16, now: Instant| {
            let rect = panel.ui.rect_for(token).expect("a pointer area");
            let at = sjk_ui::Vec2::new(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5);
            let mut action = PanelAction::None;
            for event in [
                InputEvent::PointerMove(at),
                InputEvent::PointerPress {
                    position: at,
                    button: sjk_ui::PointerButton::Primary,
                },
                InputEvent::PointerRelease {
                    position: at,
                    button: sjk_ui::PointerButton::Primary,
                },
            ] {
                let next = panel.pointer_at(event, true, now);
                if next != PanelAction::None {
                    action = next;
                }
            }
            action
        };
        let row = |id: &str| SHADER_BASE + row_of(id) as u16;
        let start = Instant::now();
        let later = |millis: u64| start + std::time::Duration::from_millis(millis);
        let mut panel = drawn(Tab::Shaders, &inputs(Holdings::Known(&owned), ""));
        // One click chooses; a second soon after wears it.
        assert_eq!(
            click(&mut panel, row("saber_sun"), later(0)),
            PanelAction::None
        );
        assert_eq!(
            click(&mut panel, row("saber_sun"), later(250)),
            PanelAction::Wear("saber_sun")
        );
        // Too slow, or on another row, is two single clicks.
        assert_eq!(
            click(&mut panel, row("saber_sun"), later(1_000)),
            PanelAction::None
        );
        assert_eq!(
            click(&mut panel, row("saber_sun"), later(1_600)),
            PanelAction::None
        );
        assert_eq!(
            click(&mut panel, SHADER_BASE, later(1_700)),
            PanelAction::None
        );
        assert_eq!(
            click(&mut panel, row("saber_sun"), later(1_800)),
            PanelAction::None
        );
        // A locked one is only previewed.
        assert_eq!(
            click(&mut panel, row("saber_void"), later(3_000)),
            PanelAction::None
        );
        assert_eq!(
            click(&mut panel, row("saber_void"), later(3_200)),
            PanelAction::None
        );
        assert_eq!(panel.shader, usize::from(row("saber_void") - SHADER_BASE));
        // The worn one stays worn.
        let mut panel = drawn(Tab::Shaders, &inputs(Holdings::Known(&owned), "saber_sun"));
        assert_eq!(
            click(&mut panel, row("saber_sun"), later(0)),
            PanelAction::None
        );
        assert_eq!(
            click(&mut panel, row("saber_sun"), later(200)),
            PanelAction::None
        );
        // The stock blade, worn by a double click while a shader is on.
        assert_eq!(
            click(&mut panel, SHADER_BASE, later(1_000)),
            PanelAction::None
        );
        assert_eq!(
            click(&mut panel, SHADER_BASE, later(1_200)),
            PanelAction::Wear("")
        );
    }

    #[test]
    fn a_click_chooses_acts_and_turns_the_tab() {
        let owned = [sun()];
        let mut panel = drawn(Tab::Shaders, &inputs(Holdings::Known(&owned), ""));
        let click = |panel: &mut Panel, token: u16| {
            let rect = panel.ui.rect_for(token).expect("a pointer area");
            let at = sjk_ui::Vec2::new(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5);
            let mut action = PanelAction::None;
            for event in [
                InputEvent::PointerMove(at),
                InputEvent::PointerPress {
                    position: at,
                    button: sjk_ui::PointerButton::Primary,
                },
                InputEvent::PointerRelease {
                    position: at,
                    button: sjk_ui::PointerButton::Primary,
                },
            ] {
                let next = panel.handle_pointer(event, true);
                if next != PanelAction::None {
                    action = next;
                }
            }
            action
        };
        let sun_row = row_of("saber_sun") as u16;
        assert_eq!(click(&mut panel, SHADER_BASE + sun_row), PanelAction::None);
        assert_eq!(panel.shader, usize::from(sun_row));
        let fonts = crate::text::load_modern(1.0, None).expect("Inter");
        panel.build(
            &inputs(Holdings::Known(&owned), ""),
            &fonts.font,
            [1920.0, 1080.0],
        );
        assert_eq!(
            click(&mut panel, WEAR_TOKEN),
            PanelAction::Wear("saber_sun")
        );
        assert_eq!(
            click(&mut panel, crate::profile_hub::TOKEN + 1),
            PanelAction::None
        );
        assert_eq!(panel.tab(), Tab::Achievements);
        panel.build(
            &inputs(Holdings::Known(&owned), ""),
            &fonts.font,
            [1920.0, 1080.0],
        );
        assert_eq!(click(&mut panel, BACK_TOKEN), PanelAction::Close);
    }

    /// The Shaders tab's card points to the Forge once the hub's economy is known: Break
    /// down for a shader of the trade held (a medal's never), Craft for one not held; a
    /// click shows the Forge on that section with the shader chosen.
    #[test]
    fn the_shader_card_points_to_the_forge() {
        let owned = [sjk_identity::Unlock {
            id: "saber_tidal".to_owned(),
            count: 3,
            ..sun()
        }];
        let fonts = crate::text::load_modern(1.0, None).expect("Inter");
        let mut panel = Panel::new();
        panel.open(Tab::Shaders, true, true, ReturnTarget::MainMenu);
        let build = |panel: &mut Panel| {
            panel.build(
                &inputs(Holdings::Known(&owned), ""),
                &fonts.font,
                [1920.0, 1080.0],
            );
        };
        build(&mut panel);
        panel.choose_shader_for_shot(row_of("saber_tidal"));
        build(&mut panel);
        assert!(
            panel.ui.rect_for(FORGE_LINK_TOKEN).is_none(),
            "no economy yet"
        );
        panel.set_trade_known(true);
        build(&mut panel);
        assert!(panel.ui.rect_for(FORGE_LINK_TOKEN).is_some());
        let runs: Vec<_> = panel.ui.text_runs().collect();
        assert!(
            runs.iter()
                .any(|run| run.contains("Break down in the Forge")),
            "{runs:?}"
        );
        assert_eq!(panel.open_forge_link(), PanelAction::None);
        assert_eq!(panel.tab(), Tab::Forge);
        assert_eq!(panel.forge_section(), forge::Section::Break);
        assert_eq!(panel.forge.want.as_deref(), Some("saber_tidal"));
        // A locked shader of the trade points to Craft; a medal's blade nowhere.
        panel.show(Tab::Shaders);
        let locked = (1..SHADER_ROWS)
            .find(|&row| {
                shader_skin(row)
                    .is_some_and(|skin| skin.id != "saber_tidal" && crate::economy::in_trade(skin))
            })
            .unwrap();
        panel.choose_shader_for_shot(locked);
        build(&mut panel);
        let runs: Vec<_> = panel.ui.text_runs().collect();
        assert!(runs.iter().any(|run| run.contains("Craft in the Forge")));
        panel.choose_shader_for_shot(row_of("saber_glitch"));
        build(&mut panel);
        assert!(
            panel.ui.rect_for(FORGE_LINK_TOKEN).is_none(),
            "a medal's blade"
        );
    }

    #[test]
    fn the_shaders_list_the_rarest_first_after_the_stock_blade() {
        assert_eq!(shader_skin(0), None, "the stock blade first");
        let tiers: Vec<_> = (1..SHADER_ROWS)
            .map(|row| shader_skin(row).unwrap().tier)
            .collect();
        assert!(tiers.windows(2).all(|pair| pair[0] >= pair[1]), "{tiers:?}");
        assert_eq!(shader_skin(1).unwrap().id, "saber_sun");
        assert_eq!(shader_skin(SHADER_ROWS), None);
        let panel = drawn(Tab::Shaders, &inputs(Holdings::Known(&[]), ""));
        let names: Vec<_> = panel.ui.text_runs().collect();
        let at = |name: &str| names.iter().position(|run| *run == name).unwrap();
        assert!(at("Sun blade") < at("Void blade"));
        let mythical = format!("{} Mythical", unlockables::Rarity::Mythical.symbol());
        let legendary = format!("{} Legendary", unlockables::Rarity::Legendary.symbol());
        // Below the Mythical ones, if the rack reaches that far.
        let legendary = names.iter().position(|run| *run == legendary);
        assert!(legendary.is_none_or(|l| at(&mythical) < l), "{names:?}");
    }

    /// The panel on Shaders in the grid after one frame.
    fn grid(holdings: Holdings<'_>, setting: &str, viewport: [f32; 2]) -> Panel {
        let mut panel = Panel::new();
        panel.open(Tab::Shaders, true, true, ReturnTarget::MainMenu);
        let fonts = crate::text::load_modern(1.0, None).expect("Inter");
        let shown = Inputs {
            grid: true,
            ..inputs(holdings, setting)
        };
        panel.build(&shown, &fonts.font, viewport);
        panel
    }

    #[test]
    fn v_and_the_switches_change_the_view_and_say_so() {
        let mut panel = drawn(Tab::Shaders, &inputs(Holdings::Known(&[]), ""));
        assert!(!panel.grid);
        assert_eq!(
            panel.key(KeyCode::KeyV, false, true),
            PanelAction::ShaderView(true)
        );
        assert!(panel.grid);
        assert_eq!(
            panel.key(KeyCode::KeyV, false, true),
            PanelAction::ShaderView(false)
        );
        // V is the Shaders tab's only.
        panel.show(Tab::Medals);
        assert_eq!(panel.key(KeyCode::KeyV, false, true), PanelAction::None);
        // The switches answer a click.
        let mut panel = drawn(Tab::Shaders, &inputs(Holdings::Known(&[]), ""));
        for (token, grid) in [(GRID_VIEW_TOKEN, true), (LIST_VIEW_TOKEN, false)] {
            let rect = panel.ui.rect_for(token).expect("a switch");
            let at = sjk_ui::Vec2::new(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5);
            let mut action = PanelAction::None;
            for event in [
                InputEvent::PointerMove(at),
                InputEvent::PointerPress {
                    position: at,
                    button: sjk_ui::PointerButton::Primary,
                },
                InputEvent::PointerRelease {
                    position: at,
                    button: sjk_ui::PointerButton::Primary,
                },
            ] {
                let next = panel.handle_pointer(event, true);
                if next != PanelAction::None {
                    action = next;
                }
            }
            assert_eq!(action, PanelAction::ShaderView(grid));
        }
        // The page follows the setting it is given each frame.
        let panel = grid(Holdings::Known(&[]), "", [1920.0, 1080.0]);
        assert!(panel.grid);
    }

    #[test]
    fn the_arrows_move_in_two_directions_on_the_grid() {
        let mut panel = grid(Holdings::Known(&[]), "", [1920.0, 1080.0]);
        assert_eq!(panel.shader, 0);
        let _ = panel.key(KeyCode::ArrowLeft, false, true);
        assert_eq!(panel.shader, 0, "stops at the start");
        let _ = panel.key(KeyCode::ArrowRight, false, true);
        assert_eq!(panel.shader, 1);
        let _ = panel.key(KeyCode::ArrowDown, false, true);
        assert_eq!(panel.shader, 1 + GRID_COLUMNS);
        let _ = panel.key(KeyCode::ArrowUp, false, true);
        assert_eq!(panel.shader, 1);
        let _ = panel.key(KeyCode::ArrowUp, false, true);
        assert_eq!(panel.shader, 1, "stops at the first line");
        for _ in 0..SHADER_ROWS {
            let _ = panel.key(KeyCode::ArrowDown, false, true);
        }
        let last_line = (SHADER_ROWS - 1) / GRID_COLUMNS;
        assert_eq!(panel.shader / GRID_COLUMNS, last_line);
        for _ in 0..SHADER_ROWS {
            let _ = panel.key(KeyCode::ArrowRight, false, true);
        }
        assert_eq!(panel.shader, SHADER_ROWS - 1, "stops at the last card");
    }

    #[test]
    fn the_wheel_scrolls_the_grid_a_line_and_keeps_the_card_shown() {
        let mut panel = grid(Holdings::Known(&[]), "", [1920.0, 1080.0]);
        let lines = SHADER_ROWS.div_ceil(GRID_COLUMNS);
        let wheel = |panel: &mut Panel, y: f32| {
            let _ = panel.handle_pointer(
                InputEvent::PointerWheel {
                    position: sjk_ui::Vec2::new(500.0, 500.0),
                    delta: sjk_ui::Vec2::new(0.0, y),
                },
                false,
            );
        };
        for _ in 0..lines + 2 {
            wheel(&mut panel, -1.0);
        }
        assert_eq!(panel.grid_first, lines.saturating_sub(GRID_LINES_SHOWN));
        let line = panel.shader / GRID_COLUMNS;
        assert!((panel.grid_first..panel.grid_first + GRID_LINES_SHOWN).contains(&line));
        for _ in 0..lines + 2 {
            wheel(&mut panel, 1.0);
        }
        assert_eq!(panel.grid_first, 0);
    }

    #[test]
    fn a_double_click_on_a_card_wears_an_owned_shader() {
        let owned = [sun()];
        let mut panel = grid(Holdings::Known(&owned), "", [1920.0, 1080.0]);
        let token = SHADER_BASE + row_of("saber_sun") as u16;
        let rect = panel.ui.rect_for(token).expect("the Sun's card");
        let at = sjk_ui::Vec2::new(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5);
        let start = Instant::now();
        let mut click = |millis: u64| {
            let now = start + std::time::Duration::from_millis(millis);
            let mut action = PanelAction::None;
            for event in [
                InputEvent::PointerMove(at),
                InputEvent::PointerPress {
                    position: at,
                    button: sjk_ui::PointerButton::Primary,
                },
                InputEvent::PointerRelease {
                    position: at,
                    button: sjk_ui::PointerButton::Primary,
                },
            ] {
                let next = panel.pointer_at(event, true, now);
                if next != PanelAction::None {
                    action = next;
                }
            }
            action
        };
        assert_eq!(click(0), PanelAction::None);
        assert_eq!(click(200), PanelAction::Wear("saber_sun"));
    }

    /// Every card in view, its words and the switches fit the canvas above the keys,
    /// inside the cards' column, at 1080 lines, 4K, 4:3 and 21:9, wherever it scrolls.
    #[test]
    fn the_grid_fits_over_the_keys_at_every_shape() {
        let sun = [sun()];
        let font = crate::text::load_modern(1.0, None).expect("Inter").font;
        for viewport in [
            [1_920.0, 1_080.0],
            [3_840.0, 2_160.0],
            [1_440.0, 1_080.0],
            [2_560.0, 1_080.0],
        ] {
            for chosen in [0, SHADER_ROWS - 1] {
                let mut panel = Panel::new();
                panel.open(Tab::Shaders, true, true, ReturnTarget::MainMenu);
                panel.shader = chosen;
                let shown = Inputs {
                    grid: true,
                    ..inputs(Holdings::Known(&sun), "saber_sun")
                };
                panel.build(&shown, &font, viewport);
                assert!(!panel.ui.overflowed(), "{viewport:?}");
                let frame = crate::menu::sjk::Frame::new(viewport);
                let keys = frame.point(0.0, view::KEYS_Y)[1];
                let first = panel.grid_first * GRID_COLUMNS;
                let shown_cards = first..(first + GRID_COLUMNS * GRID_LINES_SHOWN).min(SHADER_ROWS);
                assert!(shown_cards.contains(&chosen), "{chosen}");
                let switches = [
                    panel.ui.rect_for(LIST_VIEW_TOKEN).unwrap(),
                    panel.ui.rect_for(GRID_VIEW_TOKEN).unwrap(),
                ];
                for index in 0..SHADER_ROWS {
                    let area = panel.ui.rect_for(SHADER_BASE + index as u16);
                    assert_eq!(area.is_some(), shown_cards.contains(&index), "{index}");
                    if let Some(area) = area {
                        assert!(area.bottom() <= keys, "{index} {viewport:?}");
                        assert!(area.right() <= viewport[0], "{index} {viewport:?}");
                        for switch in switches {
                            assert!(switch.bottom() <= area.y, "{viewport:?}");
                        }
                    }
                }
                // The switches clear the kinds' pills.
                let pills_left = frame.rect(view::LEFT_X + 560.0, 0.0, 0.0, 0.0).x;
                assert!(switches[1].right() < pills_left, "{viewport:?}");
            }
        }
    }
}
