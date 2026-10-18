//! World shots of the holocron economy (`docs/cards.md`, "Shards and the shop") over the
//! live duel6 in the SJK UI: the Holocrons tab with the kyber shards beside the credits
//! and its Shop button; the loot table with its shards column; the shop with the day's
//! four offers for kyber shards (one too dear, one owned, one waiting for its Confirm) and
//! the holocrons for credits (one waiting for its Confirm, one too dear), while it reads
//! the hub, after a purchase and after a holocron bought; a pile of shards coming out of a holocron (the strip at
//! rest, the crystals and the count, the words) and among twelve opened at once; the
//! Cards tab with both balances; and the shop in a 4:3, a 21:9 and a 4K window. Made-up
//! holdings; off-screen renders (no game window), ignored like the other world shots:
//! they need a GPU adapter and the installed game data named by `JKA_GAME_DATA` (and the
//! hub's packs in `SJK_TEST_PACKS` for the offers' blades).
//!
//! Run: `JKA_GAME_DATA=<GameData> SJK_TEST_PACKS=<hub>/assets/dist cargo test --release -p
//! sjk-viewer economy_shots -- --ignored --test-threads=1 --nocapture`; the pictures are
//! in `target/world-shots`.

use super::*;
use crate::console::holocrons_panel::Data;
use crate::console::holocrons_panel::opening::{Pile, Prize, Shown};
use crate::profile_hub::Tab;

/// The Holocrons tab's made-up holdings: holocrons to open, credits, kyber shards, the
/// shop's day read `left` seconds before it turns.
fn holdings(left: i64) -> Data {
    Data {
        sealed: [vec![11, 12, 15], vec![13], Vec::new(), vec![14]],
        credits: Some(300),
        shards: Some(160),
        shop: Some(crate::console::holocrons_panel::shop::sample_day(left)),
        ..super::holocrons_tab::holdings()
    }
}

fn sjk_cvars() -> [(&'static str, &'static str); 3] {
    [
        ("ui_menuStyle", "sjk"),
        (crate::settings::quick::HIDE_CVAR, "1"),
        ("name", "^1Sol^7Vulpes"),
    ]
}

/// The main menu over duel6 in `size`, the Collection open on its Holocrons tab with
/// `data`.
fn open_holocrons(size: [u32; 2], data: Data) -> Option<GpuState> {
    super::saber_skins::mount_test_packs();
    let menu = menu::ClientMenu::new(true, String::new());
    let (mut gpu, profile) = open("maps/mp/duel6.bsp", size, Some(menu), &sjk_cvars())?;
    // The window's profile folder lives as long as the test.
    std::mem::forget(profile);
    let _ = frame(&mut gpu, 10);
    gpu.holocron_stage_seconds = Some(2.4);
    if let Some(console) = gpu.console.as_mut() {
        console.preview_holocrons(data, 0);
    }
    gpu.open_profile_hub(Tab::Holocrons);
    assert_eq!(gpu.profile_hub_tab(), Some(Tab::Holocrons));
    let _ = frame(&mut gpu, 150);
    Some(gpu)
}

fn overflowed(gpu: &GpuState) -> bool {
    gpu.console.as_ref().is_some_and(|console| {
        console.holocrons_page_overflowed() || console.profile_pages_overflowed()
    })
}

/// A pile of `shards` from a holocron of tier `tier`.
fn pile(tier: usize, holocron: u64, shards: u64) -> Prize {
    Prize::Shards(Pile {
        holocron,
        tier,
        shards,
    })
}

/// Twelve opened at once, three of them piles of shards.
fn twelve() -> Vec<Prize> {
    let card = |holocron: u64, tier: usize, rarity: usize, finish: usize| {
        Prize::Card(Shown {
            holocron,
            tier,
            rarity: &crate::cards::RARITIES[rarity],
            finish: &crate::cards::FINISHES[finish],
            edition: "Alpha Edition".to_owned(),
            credits: 10 + holocron % 30,
            shards: 1 + holocron % 4,
            card: None,
        })
    };
    let shader = |holocron: u64, tier: usize, id: &str| {
        let skin = crate::unlockables::find(id);
        Prize::Shader(crate::console::holocrons_panel::opening::Found {
            holocron,
            tier,
            skin,
            id: id.to_owned(),
            name: skin.map_or("A blade", |skin| skin.name).to_owned(),
            count: 1,
        })
    };
    vec![
        card(40, 0, 0, 0),
        pile(0, 41, 12),
        card(42, 1, 1, 0),
        shader(43, 0, "saber_tidal"),
        card(44, 2, 2, 0),
        pile(1, 45, 19),
        card(46, 0, 0, 1),
        shader(47, 3, "saber_phoenix"),
        card(48, 1, 0, 0),
        pile(3, 49, 164),
        card(50, 2, 1, 0),
        card(51, 0, 0, 0),
    ]
}

#[test]
#[ignore = "renders with the GPU and the installed game data named by JKA_GAME_DATA"]
fn duel6_sjk_economy_shots() {
    on_big_stack(|| {
        let Some(mut gpu) = open_holocrons([1920, 1080], holdings(5 * 3_600 + 12 * 60)) else {
            return;
        };
        let mut moments = Vec::new();
        let mut keep = |gpu: &mut GpuState, frames: u32, name: &str| {
            let path = shoot(gpu, frames, &format!("duel6-economy-{name}"));
            assert!(!overflowed(gpu), "{name}");
            println!("{}", path.display());
            moments.push(image::open(path).expect("the shot").to_rgba8());
        };
        // The tab: shards beside the credits, Shop beside Open all.
        keep(&mut gpu, 2, "tab");
        if let Some(console) = gpu.console.as_mut() {
            console.preview_loot(true);
        }
        keep(&mut gpu, 2, "loot-table");
        if let Some(console) = gpu.console.as_mut() {
            console.preview_loot(false);
            console.preview_shop(true, Some(0));
        }
        keep(&mut gpu, 4, "shop");
        // While the hub has not answered.
        let mut reading = holdings(0);
        reading.shop = None;
        if let Some(console) = gpu.console.as_mut() {
            console.preview_holocrons(reading, 0);
        }
        keep(&mut gpu, 2, "shop-reading");
        // After a purchase: the offer bought, the line saying so.
        let mut bought = holdings(3_600);
        if let Some(shop) = bought.shop.as_mut() {
            shop.day.offers[1].bought = true;
            shop.day.offers[1].owned = true;
        }
        bought.shards = Some(85);
        bought.trade = Some(sjk_identity::TradeOutcome {
            serial: 1,
            trade: sjk_identity::Trade::Buy,
            id: "saber_sakura".to_owned(),
            amount: 75,
            message: String::new(),
        });
        if let Some(console) = gpu.console.as_mut() {
            console.preview_holocrons(bought, 0);
            console.preview_shop(false, None);
            console.preview_shop(true, None);
            console.preview_shop_answer();
        }
        keep(&mut gpu, 2, "shop-bought");
        // The Uncommon holocron for credits waiting for its Confirm, then bought.
        if let Some(console) = gpu.console.as_mut() {
            console.preview_shop(false, None);
            console.preview_holocrons(holdings(3_600), 0);
            console.preview_shop(true, Some(4));
        }
        keep(&mut gpu, 2, "shop-holocron-confirm");
        let mut holocron = holdings(3_600);
        holocron.credits = Some(150);
        holocron.trade = Some(sjk_identity::TradeOutcome {
            serial: 1,
            trade: sjk_identity::Trade::Holocron,
            id: "uncommon".to_owned(),
            amount: 150,
            message: String::new(),
        });
        if let Some(console) = gpu.console.as_mut() {
            console.preview_holocrons(holocron, 0);
            console.preview_shop(false, None);
            console.preview_shop(true, None);
            console.preview_shop_answer();
        }
        keep(&mut gpu, 2, "shop-holocron-bought");
        if let Some(console) = gpu.console.as_mut() {
            console.preview_shop(false, None);
            console.preview_holocrons(holdings(3_600), 0);
        }
        // A pile of shards out of a Legendary holocron: at rest on the strip, coming in,
        // and its words.
        for (elapsed, name) in [
            (-0.2, "pile-rest"),
            (1.4, "pile-entrance"),
            (3.0, "pile-words"),
        ] {
            if let Some(console) = gpu.console.as_mut() {
                console.preview_opening(Some(vec![pile(2, 31, 37)]), elapsed, 3.1);
            }
            keep(&mut gpu, 40, name);
        }
        // Twelve at once: a round of strips at rest, then everything laid out.
        for (elapsed, name) in [
            (1.7 + 5.5, "batch-rest"),
            (1.7 + 5.9 * 3.0 + 3.0, "batch-found"),
        ] {
            if let Some(console) = gpu.console.as_mut() {
                console.preview_opening(Some(twelve()), elapsed, 3.1);
            }
            keep(&mut gpu, 40, name);
        }
        if let Some(console) = gpu.console.as_mut() {
            console.preview_opening(None, 0.0, 0.0);
            console.preview_cards(Vec::new(), 2_000, 45);
        }
        gpu.open_profile_hub(Tab::Cards);
        keep(&mut gpu, 30, "cards-tab");
        println!(
            "{}",
            sheet(&moments, 3, 640, "duel6-economy-moments").display()
        );
    });
}

/// A card as a pack (17/10/2026; its credits on the strip, 18/10/2026): the strip running
/// and at rest on a card's credits, the card tucked face down in the tile's corner, for an
/// Uncommon, a Rare Holo and a Mythical holocron's Legendary card; then the Rare Holo pack's
/// credits rising, sliding aside as the card slides out, the card flipping, its flourish,
/// the shards counting and its words; one with a face, and the pack in a 4:3 window.
#[test]
#[ignore = "renders with the GPU and the installed game data named by JKA_GAME_DATA"]
fn duel6_sjk_card_pack() {
    on_big_stack(|| {
        let pack = |holocron: u64, tier: usize, [rarity, finish]: [usize; 2], credits: u64| {
            move |card: Option<String>| {
                Prize::Card(Shown {
                    holocron,
                    tier,
                    rarity: &crate::cards::RARITIES[rarity],
                    finish: &crate::cards::FINISHES[finish],
                    edition: "Alpha Edition".to_owned(),
                    credits,
                    shards: 4,
                    card,
                })
            }
        };
        let rare = pack(21, 1, [1, 1], 42);
        let uncommon = pack(23, 0, [0, 0], 14);
        let mythical = pack(27, 3, [3, 2], 312);
        // Sol's first card, as the card shots show it (from the hub's packs).
        let face = Some("sol_lore_001".to_owned());
        let mut moments = Vec::new();
        for (size, name) in [([1920, 1080], ""), ([1440, 1080], "-4x3")] {
            let Some(mut gpu) = open_holocrons(size, holdings(3_600)) else {
                return;
            };
            let mut keep = |gpu: &mut GpuState, frames: u32, name: &str| {
                let path = shoot(gpu, frames, &format!("duel6-card-pack-{name}"));
                assert!(!overflowed(gpu), "{name}");
                println!("{}", path.display());
                moments.push(image::open(path).expect("the shot").to_rgba8());
            };
            type Pack<'a> = &'a dyn Fn(Option<String>) -> Prize;
            let moments_here: &[(Pack<'_>, f32, &str, bool)] = if name.is_empty() {
                &[
                    (&uncommon, -1.6, "running-uncommon", false),
                    (&uncommon, -0.2, "rest-uncommon", false),
                    (&rare, -0.2, "rest-rare", false),
                    (&mythical, -0.2, "rest-mythical", false),
                    (&rare, 0.7, "rising", false),
                    (&rare, 1.3, "sliding", false),
                    (&rare, 1.68, "flipping", false),
                    (&rare, 1.85, "flipped", false),
                    (&rare, 2.15, "flourish", false),
                    (&mythical, 2.15, "flourish-solar", false),
                    (&rare, 2.45, "counting", false),
                    (&rare, 3.1, "words", false),
                    (&rare, 3.1, "face", true),
                ]
            } else {
                &[(&rare, 3.1, "words", false)]
            };
            for &(prize, elapsed, moment, with_face) in moments_here {
                let card = if with_face { face.clone() } else { None };
                if let Some(console) = gpu.console.as_mut() {
                    console.preview_opening(Some(vec![prize(card)]), elapsed, 3.1);
                }
                keep(&mut gpu, 40, &format!("{moment}{name}"));
            }
        }
        println!("{}", sheet(&moments, 3, 640, "duel6-card-pack").display());
    });
}

/// The shop in other windows: 4:3, 21:9 and 4K.
#[test]
#[ignore = "renders with the GPU and the installed game data named by JKA_GAME_DATA"]
fn duel6_sjk_economy_shop_windows() {
    on_big_stack(|| {
        for (size, name) in [
            ([1440, 1080], "4x3"),
            ([2560, 1080], "21x9"),
            ([3840, 2160], "4k"),
        ] {
            let Some(mut gpu) = open_holocrons(size, holdings(42 * 60)) else {
                return;
            };
            if let Some(console) = gpu.console.as_mut() {
                console.preview_shop(true, None);
            }
            let path = shoot(&mut gpu, 4, &format!("duel6-economy-shop-{name}"));
            assert!(!overflowed(&gpu), "{name}");
            println!("{}", path.display());
        }
    });
}
