# Cards and credits: what a holocron holds

**Status (12/10/2026): opening is built, cards are sealed.** Opening a holocron
gives one *sealed* card and some credits. A sealed card has its rarity, finish and
edition but no face yet: the faces come with the first set of cards, and every
sealed card is revealed into that set then. Credits are kept and shown. Since
17/10/2026 the economy is prepared, not deployed ([Shards and the shop](#shards-and-the-shop)):
kyber shards beside credits, shattering duplicates, crafting, a daily shop that spends
shards, and holocrons bought with credits (Sol's decisions of 17/10/2026). [Holocron drops](holocrons.md) give the holocrons. The wire is the hub's
`PROTOCOL.md` ("Holocrons", "Opening", "Shards and the shop") in Sol-Vulpes/SJK-hub.

Collecting cards grants nothing: no power, setting or right, as with medals
([identity.md](identity.md#medals)). There is no trading and never any real money
in the system. Hats and capes stay free for everyone and are never sold or dropped
([client.md](client.md#hats-and-capes)).

## Decisions

| Question | Decision |
| --- | --- |
| What opening gives | One card and credits per holocron (Sol, 12/10/2026). |
| Cards before the art exists | The card is sealed: rarity, finish and edition are rolled at opening; the face is chosen when the first set is out (Sol, 12/10/2026). |
| Where a sealed card is revealed | Into the first set only, so Alpha cards stay launch-set cards (my pick, Sol's go-ahead). |
| What the holocron's tier does | A higher tier raises the odds of rarer cards, better finishes and more credits ([Odds](#odds)). |
| Finishes | Normal, Holo, Solar (the rarest: an animated sunlit shimmer). |
| Editions | A stamp apart from the finish: **Alpha Edition** while the alpha lasts. Every other collectible carries one too ([Editions](unlockables.md#editions)). |
| Credits' sign | The galactic credit's, the slanted seven crossed by two bars (Sol's pick, from Wikipedia); drawn from shapes. |
| What credits will buy | An Uncommon (150) or a Rare (400) holocron, outside the daily caps; Legendary and Mythical ones are only ever earned (Sol, 17/10/2026, [Holocrons for credits](#holocrons-for-credits)); prepared, not deployed. Until then the shop sold for credits. |
| Duplicates | Cards keep a count ("x3"). A droppable shader held twice or more can be shattered into kyber shards, one copy at a time, never the last (Sol, 17/10/2026; it replaced selling duplicates for credits). |
| Kyber shards | A second currency (Sol, 17/10/2026): from shattering duplicates, a holocron's pile of shards and a card's pack; spent to craft the droppable shader the player chooses and in the shop's four daily offers ([Shards and the shop](#shards-and-the-shop)). |
| Drop rates | Start stingy; buffing later is easy on the hub and nerfing is not. |
| Money, trading | None. Free, non-commercial, no trading. |

## Opening

All decided by the hub, in one transaction:

1. `POST /v1/holocrons/open` with `{"ids":[..]}` (1 to 20 of the player's own
   holocrons not opened yet, signed). For each the hub rolls the card's rarity and
   finish and the credits, stores the sealed copy (with the current edition, the
   holocron and its tier) and a line in the credits ledger, and marks the holocron
   opened. All or nothing: 404 `not_found` (not the player's), 409
   `holocron_opened` (opened already, from another PC of the person's), 400
   `bad_ids`.
2. It answers what each brought (`opened`: holocron, tier, card's id, rarity,
   finish, edition, credits) and the person's profile after.
3. The profile carries `credits` (the balance) and `cards` (the copies, one entry per
   card, rarity, finish and edition, with a count, the first and last time; `card` is
   null while sealed). A holocron's entry carries `opened`; `holocron_counts` counts
   only the unopened ones, so a client shows what can still be opened.

Opening counts toward no cap, and an opened holocron still counts toward the day's
drop caps. Removing an opened holocron (staff) leaves its card and credits.

**Staff only for now.** Since 12/10/2026 (Sol's request, while opening is worked on)
the hub lets only staff open holocrons unless its operator opens it to everyone
(`--opening everyone`, or `SJK_HUB_OPENING=everyone`, on the hub; no client release is
needed). Anyone else gets 403 `opening_staff_only` and keeps their holocrons. The
progress answer (`GET /v1/holocrons`) says `can_open`; the Holocrons tab hides Open and
Open all, and Enter and A do nothing, when it is false (or not read yet), and says
"Opening comes soon: keep your holocrons, they will wait." in their place.

## Odds

The hub's tables (`src/cards.rs` there); the holocron's tier picks the row. Players see
them, with the chances per holocron found, on the Holocrons tab's loot table
([holocrons.md](holocrons.md#loot-table)); the client's copy (`cards::ODDS`,
`cards::CREDITS`) is pinned to these by a test.

Rarity of the card:

| Holocron | Common | Uncommon | Rare | Legendary |
| --- | --- | --- | --- | --- |
| Uncommon | 70 % | 24 % | 5.5 % | 0.5 % |
| Rare | 50 % | 33 % | 15 % | 2 % |
| Legendary | 30 % | 38 % | 26 % | 6 % |
| Mythical | 10 % | 30 % | 45 % | 15 % |

Finish of the copy:

| Holocron | Normal | Holo | Solar |
| --- | --- | --- | --- |
| Uncommon | 97 % | 3 % | 0 % |
| Rare | 90 % | 10 % | 0 % |
| Legendary | 70 % | 29 % | 1 % |
| Mythical | 40 % | 55 % | 5 % |

Credits, any whole number in the range, equally likely:

| Holocron | Credits | On average |
| --- | --- | --- |
| Uncommon | 10 to 20 | 15 |
| Rare | 30 to 50 | 40 |
| Legendary | 80 to 120 | 100 |
| Mythical | 250 to 350 | 300 |

About 35 credits a card opening on average; counting the openings that bring a shader or a
pile of kyber shards instead (no credits), about 17.5 credits a holocron found, so about 140
a day under the holocrons' daily cap of 8.

Since 17/10/2026 (prepared, not deployed) one opening in ten, in every tier, brings a pile of
kyber shards instead of the card and its credits, taken from the card's share (the shader's
odds stay): any whole number in the range, equally likely.

| Holocron | Shard pile | Shards | On average a holocron |
| --- | --- | --- | --- |
| Uncommon | 10 % | 5 to 15 | 1 |
| Rare | 10 % | 10 to 25 | 1.75 |
| Legendary | 10 % | 25 to 50 | 3.75 |
| Mythical | 10 % | 100 to 200 | 15 |

The piles are a consolation: about once a day of capped play, a pile is about one
duplicate of the holocron's own level shattered, a Mythical holocron's as much as a
Mythical shader's shattering.

**The card pack** (Sol's request, 17/10/2026, prepared on hub branch `feat/card-pack`, not
deployed): a card is the most common and least wanted prize, so it comes as a pack, a few
kyber shards beside its credits (about a fifth of the tier's pile), rolled after the card,
any whole number in the range, only while the economy is on:

| Holocron | Shards with a card | On average a holocron |
| --- | --- | --- |
| Uncommon | 1 to 3 | 1.2 |
| Rare | 2 to 5 | 1.75 |
| Legendary | 5 to 10 | 3.4 |
| Mythical | 20 to 40 | 12 |

The hub sends the range as the loot tables' `card_shards`; the opening answer's `shards`
then rides with the `card` (a pile has no card). Provisional, the hub's `src/loot.rs`. The client shows the ranges
([cards.rs](../crates/sjk-viewer/src/cards.rs) `CREDITS`, the hub's copy, tested);
the hub decides. Rolls use the operating system's entropy, as drops do.

## Credits

Credits are a ledger at the hub (`credit_ledger`: each change with its reason,
`holocron`, `holocron_buy`, `staff`, and `shop` from before the shop sold for shards, and
what it refers to); the balance is its sum and never
goes below 0. Staff correct a balance with `POST /v1/staff/credits` (logged), the
operator with `give-credits`; the Staff page's Holocrons and credits tab gives and takes
credits and kyber shards ([identity.md](identity.md#staff)).

## Shards and the shop

**Prepared 17/10/2026, not deployed** (hub branch `feat/economy`, client branch
`personal/economy`; Sol's decisions of 17/10/2026, after his direction of 12/10/2026 for a
shop that fixes bad luck). Every number here is provisional, in one table each at the hub
(`src/economy.rs`, `src/loot.rs`), sent to clients with the progress (`economy` and the
loot tables' `shard_pile`, `shards` and `card_shards`), the client keeping a copy for before
(`economy::BUILT_IN`, `loot_odds::BUILT_IN`, pinned by tests).

- **The trade.** Only saber shaders holocrons drop take part, less any a medal brings
  (Glitch, Hologram and Runic today). Both sides read it from their catalogues (the
  droppable flag, the medals' blades), never from a list, so a shader made droppable
  joins by itself. Achievements grant nothing today; one that brings a shader must be
  left out the same way. Cards are luck only: never sold, crafted or shattered. Hats and
  capes are free for everyone and never take part.
- **Kyber shards** are one currency for every shader (my choice for simplicity; Sol's
  question had offered "shards of one shader", see the open points), a ledger at the hub
  like credits (`shard_ledger`, reasons `shatter`, `holocron`, `craft`, `shop`, `staff`),
  never below 0, on the profile as `shards`.
- **Shattering** breaks one duplicate (a shader held twice or more by drops or grants; a
  medal's copy does not count) into shards, one copy a request, never the last.
- **Crafting** makes a shader of the trade the player does not hold, for its cost.
- **Who may:** like opening, everything follows the hub's opening setting: while opening
  is for staff, others get `opening_staff_only` and the Shop button is not shown.

| Shader tier | Shattering gives | Crafting costs | Shop price |
| --- | --- | --- | --- |
| Common | 5 shards | 40 shards | 30 shards |
| Uncommon | 10 | 100 | 75 |
| Rare | 25 | 300 | 225 |
| Legendary | 60 | 800 | 600 |
| Mythical | 150 | 2 000 | 1 500 |

Why these numbers:

- **Crafting against shattering:** eight to thirteen duplicates of a tier craft one of it,
  so shattering never mills one shader into another of the same tier. Shards come in at
  about 1.7 a holocron from the piles, and up to about 3.9 more once every shader dropped is
  a duplicate: a Common craft is then about 7 holocrons, a Mythical one about 360 (some 45
  days of capped play), against about 3 800 holocrons to drop one given Mythical shader.
- **The shop's prices** (Sol, 17/10/2026: the shop sells for shards): three quarters of the
  craft cost, so the day's offers are the cheaper road and crafting the dearer one to the
  shader the player chooses, and buying one to shatter never gains. Before, in credits
  (250, 600, 1 500, 4 000 and 10 000), for the reason that follows.
- **The credit prices before:** the rule of 12/10/2026 (the credits earned by the time 80 % of
  players would have dropped the item: 35 credits times `1.6 / p` holocrons) does not give
  sane numbers with today's catalogue: per holocron found a given Common drops at about
  1.95 %, an Uncommon at 0.75 %, a Rare at 1.38 % (only three Rares share their tier), a
  Legendary at 0.27 % and a Mythical at 0.026 %, so it prices them about 2 900, 7 500,
  4 100, 20 600 and 212 000 credits: a Rare cheaper than an Uncommon, and a Mythical some 750
  days of capped play. So the prices are per tier instead, rising with it: at about 17.5
  credits a holocron, 14, 34, 86, 229 and 571 holocrons (2, 4, 11, 29 and 71 days of capped
  play).

### The shop

- **Four offers a day**, the same for every player: one Common, one Uncommon, one Rare and
  a Legendary three days in four, a Mythical the fourth (Sol, 17/10/2026: with four
  Mythical shaders an even share brought each back about every eight days), each a
  different shader of the trade; a tier with none left takes the nearest that has one.
- **The day** runs from midnight to midnight in Paris (Europe/Paris, the EU's summer-time
  rule). The hub deals it from the day and a secret it made once in its database (SHA-256),
  so nothing is stored per day, a restart keeps the offers, and nobody, even with the hub's
  code, can tell a later day's.
- **Once a day, never a duplicate** (my decision): a player buys each offer at most once a
  day (`already_bought`), and the hub refuses a shader they hold (`already_owned`): shards
  never buy a duplicate, the shop being a fix for missing shaders. Crafting refuses one held
  the same way, so the only road to a duplicate is luck, and shattering is there for it
  (Sol asked whether bought shaders should be unbreakable instead; with prices above the
  shatter yield there is nothing to block, so no).
- A purchase is a line of the shard ledger (reason `shop`, the shader's id as its note),
  which is also how the hub tells what was bought today.

### Holocrons for credits

- **Credits buy holocrons** (Sol, 17/10/2026, once the shop sold for shards): an Uncommon
  for 150 credits and a Rare for 400; Legendary and Mythical holocrons are only ever earned.
  At about 17.5 credits a holocron opened, that is about one more holocron for every nine
  opened.
- A bought holocron is the player's at once (source `bought`), opens like any other, and
  never counts against the daily caps (8 a key, 16 an address, 1 Mythical).
- `POST /v1/holocrons/buy` `{"tier"}`; `not_for_sale` for another tier, `not_enough_credits`.
  The hub sends the prices with the progress (`economy.holocrons`), the client keeping a copy
  for before (`economy::BUILT_IN`); a hub that sends none sells none, and the shop shows no
  holocrons.

### Open points for Sol

Answered by Sol on 17/10/2026: one generic currency; the piles, shatter and craft tables
kept for the launch; the shop in shards; buying or crafting a held shader refused; the
Mythical offer one day in four; credits buy Uncommon and Rare holocrons. Left: tuning
after a week of real openings, and a trade-up (spare copies of a tier for one of the next,
proposed, not built).

No player text promises any of this until it is deployed.

## Faces and the first card

Since 12/10/2026 (Sol's first card, "Toxiee and Sol on JoF Memorial", Sol lore 001,
Legendary, its art by "Sol (AI-assisted)": the names in the title follow the picture,
Toxiee on the left) a sealed copy can become a real card. A card's `artist` line says
how its art was made; the footer reads "Art: <artist>".

- **Where cards are made:** Sol keeps each card's picture and words in `card_import/`
  at the repository's root, which git ignores (`card_import/README.txt` says the format):
  `cards/card1.png` (1020 x 850, the 6:5 art window, framed as it will show) and
  `cards/card1.txt` (`number`, `collection`, `kind`, `rarity`, `title`, `art`,
  `description`, optional `artist`), the full paintings in `sources/`, a white canvas
  and a frame overlay to frame new ones in `templates/`.
- **How they reach players:** the hub's `scripts/import_cards.py <card_import>` writes
  them into the hub's `sjk_cards` pack sources (`cards/<id>.card` JSON and
  `cards/<id>.jpg`, id `<set>_<NNN>` such as `sol_lore_001`), and its
  `scripts/build_packs.py` builds `sjk_cards.pk3` and the hub's catalogue
  `cards.json` (id, set, number, rarity). The hub serves the pack like the blade skins'
  (`GET /v1/assets`), so every SJK client downloads it into `assets/` beside
  `identity.key` and mounts it; adding a card is a new pack and a hub restart, no client
  release.
- **The hub picks:** opening a holocron rolls the copy's rarity and finish as before,
  then picks a card of that rarity from the catalogue (evenly); a rarity with no card
  yet stays sealed. `POST /v1/cards/reveal` gives the person's sealed copies of a
  rarity that has cards a card each (oldest first). Both follow who may open
  holocrons (staff only for now).
- **Each card has one rarity** (Sol lore 001 is Legendary): the first set needs cards
  of every rarity before every copy can be revealed.

## In the client

| Piece | Where |
| --- | --- |
| Wire types (`credits`, `cards`, `opened`, the opening's answer), the request, the outcome | [wire.rs](../crates/sjk-identity/src/wire.rs), [hub.rs](../crates/sjk-identity/src/hub.rs), [service.rs](../crates/sjk-identity/src/service.rs) (`Service::open_holocrons`, `Snapshot::opening`) |
| Rarities, finishes, editions, the credits' ranges, a profile's stacks | [cards.rs](../crates/sjk-viewer/src/cards.rs) |
| A card drawn from shapes, the finishes' light | [cards/look.rs](../crates/sjk-viewer/src/cards/look.rs) |
| The credits' mark | [cards/mark.rs](../crates/sjk-viewer/src/cards/mark.rs) |
| Open, Open all and the ceremony on the Holocrons tab | [holocrons_panel_opening.rs](../crates/sjk-viewer/src/holocrons_panel_opening.rs), [holocrons_panel.rs](../crates/sjk-viewer/src/holocrons_panel.rs), [holocrons_panel_view.rs](../crates/sjk-viewer/src/holocrons_panel_view.rs) |
| Kyber shards, shattering, crafting, the tables and the Shaders tab's hooks | [economy.rs](../crates/sjk-viewer/src/economy.rs); wire and requests in [wire.rs](../crates/sjk-identity/src/wire.rs), [hub.rs](../crates/sjk-identity/src/hub.rs), [service.rs](../crates/sjk-identity/src/service.rs) (`Service::shatter`, `craft`, `read_shop`, `buy`, `Snapshot::shop`, `Snapshot::trade`) |
| The kyber shard's mark | [cards/shard.rs](../crates/sjk-viewer/src/cards/shard.rs) |
| The shop | [holocrons_panel_shop.rs](../crates/sjk-viewer/src/holocrons_panel_shop.rs) |
| The Collection's Cards tab | [collection_cards.rs](../crates/sjk-viewer/src/collection_cards.rs) |
| `debug_open` | [cards/rehearsal.rs](../crates/sjk-viewer/src/cards/rehearsal.rs) |
| The faces from the packs, their pictures decoded on a worker | [cards/faces.rs](../crates/sjk-viewer/src/cards/faces.rs), mounted by [sjk_packs.rs](../crates/sjk-viewer/src/sjk_packs.rs) |
| The pictures' textures | [ui_renderer/card_art.rs](../crates/sjk-viewer/src/ui_renderer/card_art.rs) |
| Reveal (`Service::reveal_cards`, `Snapshot::reveal`) | [service.rs](../crates/sjk-identity/src/service.rs), [player_identity.rs](../crates/sjk-viewer/src/player_identity.rs) |

- **Rarities' colours:** Common silver `#B4C0D4`, then the holocron tiers' green, blue
  and purple for Uncommon, Rare and Legendary.
- **A card** is drawn, not a picture: a dark plate edged in its rarity's colour, the
  holocron's gem at its heart in a ring, the rarity's name above, "Sealed" under the
  gem and the edition stamped at the foot. Holo lays slanted bands of the rainbow
  drifting across it with a glint; Solar a warm wash, a corona of arcs turning round
  the gem, a band of sunlight passing over, embers at the foot and a breathing gold
  edge. The light is cut to the card, rounded corners included: the finish draws
  under a rounded clip (`DrawCommand::PushRoundedClip`), and the UI renderer cuts
  strokes to it, each corner as four chords of its arc
  ([ui_renderer.rs](../crates/sjk-viewer/src/ui_renderer.rs) `clip_polygon`); the
  finish's washes take the card's corner radius themselves. The light stays inside the
  rarity's edge, its slanting strokes reach far past the card so their square ends never
  show, and Holo's sweep crosses the card in about four seconds, eased, then waits off
  it for about three before the next pass (Solar's sun the same, slower).
- **Tilt (13/10/2026, Sol's request):** while the pointer rests on a card of the Cards
  tab (the one up close or one of the grid), the card turns towards it in perspective,
  up to about 9 degrees at its edges, the side under the pointer receding as if pressed;
  its light follows the pointer across it (Holo's bands, Solar's sun) and a soft glare
  sits under the pointer. It eases in and out in about a tenth of a second. The card is
  drawn as usual under `DrawCommand::PushTransform` (a projective transform the UI
  renderer applies to every shape vertex and text glyph drawn inside it, after clipping;
  `look::tilt_matrix`).
- **A revealed card** shows its face as a trading card: its name and its rarity's
  symbol ([sjk-ui.md](sjk-ui.md#symbols)) on top, its picture in a gold-edged 6:5 window (dark until the
  picture is decoded and uploaded), a strip with its kind and set ("Community ·
  Sol lore") and its edition and finish, its lore in a box (cut into lines at spaces,
  seven at most), and at the foot its number in its set ("001 / 001", counting the
  set's cards the packs hold), its artist and its rarity. The plate, the rarity's edge
  and the finishes' light are the sealed card's. A card narrower than 200 frame pixels
  shows the picture without words, and below 300 only the name. A copy whose face is
  not in the packs reads "Unknown card <id>" and keeps its sealed back.
- **The pictures** are decoded on a worker thread when the packs are mounted, cut to
  6:5 about their middle if they are not, squeezed into a 1024 square with its mip
  chain and drawn back into the 6:5 window (which undoes the squeeze); the renderer
  uploads two a frame. 512 faces at most.
- **The kyber shard's mark** (17/10/2026) is a long crystal with a point at each end: a
  square turned on its point (one thick stroke) stretched upright by a transform, its left
  facet the same square again in the full kyber cyan (`shard::KYBER`) under a clip, its
  right one deeper, a light ridge between them and a glint near the top. Eight commands.
  The SJK Symbols font has it too (U+E030, [sjk-ui.md](sjk-ui.md#symbols)).
- **The credits' mark** is the sign's outline in shapes: the crossbar and four short
  bars as rectangles, the slanting leg one stroke cut level at the bar's top and the
  foot. Eight commands, sharp at every size.
- **The Holocrons tab:** the player's kyber shards and credits stand at the panel's top
  right, each mark and sum with its word under it, the chosen tier says what opening one
  gives ("Opening it gives a sealed card and 30 to 50 credits, or a saber shader (40%) or
  kyber shards (10%)."), and under the holocron's caption **Open** (the oldest of the tier
  chosen, Enter), **Open all (n)** (up to twenty of all tiers, the oldest first, A) and,
  while the hub lets the player open, **Shop** (S). Opened holocrons stay in the recent
  list, dimmed, marked "Opened".
- **The shop** (17/10/2026, prepared): over the darkened tab as the loot table, "Shop", what
  it is and its rules, the balances at the top right, "Offers of 17/10/2026" and "New offers
  in 5 h 12 min, at midnight in Paris" (counting down; once it is due the page reads the
  hub again, once a day), and the four offers as cards: the tier's symbol and name, the
  tier's frame, wash and effects, the blade, its name, its price beside the kyber shards'
  mark (the credits' on a hub before), why it cannot be bought ("You hold it already.",
  "You bought it today.", "65 kyber shards more needed.") and Buy. Buy asks twice: it turns
  into "Confirm: 30 kyber shards" for five seconds, a second press sends it ("Buying..."
  until the hub answers); the line under the cards then says "Sakura blade bought for 75
  kyber shards. It is yours." or why not. Under them, **Holocrons for credits**: a button
  for each holocron the hub sells ("Uncommon Holocron 150 credits", in its tier's ring),
  dimmed with "100 more credits needed." under it when short, asking twice the same way
  ("Confirm: 150 credits"); the line then says "Uncommon Holocron bought for 150 credits.
  It waits in your holocrons." and the holocron joins the list. Left and Right choose (the
  offers, then the holocrons), Enter buys, a click chooses or buys, Escape, S or Back go
  back. Opening the shop reads the hub's day; "Reading the shop..." until it answers.
- **The ceremony** is drawn by the tab over its own page, which is not drawn under it
  (text draws over every shape). Since 17/10/2026 it opens the holocron itself, as a
  loot crate opens
  ([holocrons_panel_unseal.rs](../crates/sjk-viewer/src/holocrons_panel_unseal.rs)):
  - **While the hub answers** (15 seconds at most, then "The SJK hub did not answer")
    the tab's 3D holocron ([holocrons.md](holocrons.md#the-holocrons-tab)), now the one
    asked for in its own look, glides in 0.6 seconds from its place on the page to the
    middle of the window, humming round and breathing its light, with "Opening..." under
    it. The world shows round it through a vignette: four straight fades from the
    window's edges to a clear box about it (the UI renderer's arcs are strokes, not
    fills, and take no clip, so rings could not make it).
  - **The unseal** (1.7 seconds): it spins up (to about 42 radians a second more),
    swells, shakes a few pixels at the end and its light grows to three and a half
    times the tier's; rays of the tier's colour break out of its seams, more, longer and
    faster as it charges, its name above it; then it pops and is gone in a flash of the
    tier's colour with a white heart, which fades over the strip in 0.45 seconds. It
    builds on the Force jump's charge (`jumpbuild`) and cracks open on the game's own
    holocron pick-up sound (`sound/player/holocron.wav`). The motion
    (`holocrons::stage::Motion`: where, turn, swell, light, shown) is worked out from
    the ceremony's clock alone and read by the world each frame; gone, the holocron
    grows back from nothing in its place when the ceremony ends.
  - **The crate spin** (6 seconds; since 14/10/2026): a strip of what its tier can drop,
    cards and droppable shaders, runs past a gold marker with a soft tick a tile
    (`menuroam`) and slows over 88 % of it to a stop on what the hub rolled, the last
    tile taking about two seconds; the strip is made up round the real prize and the
    stop lands anywhere up to 45 % of a tile from its middle, a close call near its
    edge but always inside it (the strip never shows a near miss that was not). Each
    tile stands over a band of its colour, as a loot crate's, and at speed its colour
    streaks across it and trails it. At rest (`sub_select`) the prize lifts and lights
    up in its colour, breathing, the others dim to a third, and a grand prize (a
    Legendary or Solar card, a Legendary or Mythical shader) turns slow rays behind it.
  - **A card shows as its credits** on the strip (Sol's request, 18/10/2026): a card is
    the commonest and least wanted prize, so its tile is a stack of three credit coins
    with the amount ("+42 credits"), washed from above in the holocron's colour, and the
    card tucked face down in its top right corner, turned a little, its back in the
    holocron's colour with the holocron's gem (edged in gold for Solar, a rainbow across it
    for Holo), so a strip of cards is never grey. The prize's amount is what the hub gave;
    a neighbour's is a whole number in the tier's credits range (Uncommon 10 to 20, Rare 30
    to 50, Legendary 80 to 120, Mythical 250 to 350, from the loot tables), the same each
    time for the same holocron, drawn apart from which neighbours come up. The tile's edge
    and its band stay in the card's rarity colour (gold for Solar), and a grand card keeps
    its rays. No shards on the tile: they show on the pack once the strip stops.
  - **A pile of kyber shards** (17/10/2026) is a tile of its own on the strip: "Kyber
    shards", three crystals in a kyber glow and the tier's range ("25 to 50"), or for the
    prize how many ("+37"); stacked, the crystals alone. Its entrance is the burst, then
    three crystals growing in in a kyber glow and the shards counting up beside the shard
    mark, then "37 kyber shards", "From a Legendary Holocron", what shards craft and how many
    the player holds now. Among many opened, the finds show it ("37 kyber shards", "For
    crafting"), the shards counted beside the credits at the top, a pack's shards with them
    (a card's note "+24 credits, +3 shards"). A client from before the
    piles skips such an opening (nothing it knows: an opening of piles alone says "Nothing
    this client knows came out."); the profile carries the shards either way.
  - **The entrance:** the burst in its tier's light (rings, turning rays, the gem
    flaring away), then the card's pack (18/10/2026, 3.1 seconds): the credits' tile, as
    the strip showed it, rises where the strip stopped with the card tucked in its corner
    and its credits counting up; it slides aside to the left while the card slides out of
    its corner to the middle, growing and straightening, and flips over (it narrows
    edge-on and opens on its front, its rarity and finish or its face); a Holo, Solar or
    Legendary card flips into a flourish, a glow, a ring running out and rays turning
    (gold for Solar, the rainbow for Holo, its rarity's colour for a plain Legendary);
    then the kyber shards' tile slides out on the right and counts up (none while the
    hub's economy is off). Then its words: "Rare card, Holo", "Alpha Edition. From a Rare
    Holocron", what the finish is, that it is sealed, "1 of 3", Next or Done. A shader instead of a card shows its
    blade framed in its tier's colour, its name, "Common saber shader. From a Rare
    Holocron", what it looks like and how many the player holds. A Solar finish or a
    Legendary card plays the medals' fanfare, another card the unlocks' chime.
  - **Opening more than three**
    ([holocrons_panel_batch.rs](../crates/sjk-viewer/src/holocrons_panel_batch.rs)):
    the first holocron cracks open ("12 holocrons"), then their strips run five at a
    time, stacked, each named by its holocron's tier at its left and marked in its
    colour, each stopping 0.35 seconds after the one above (3.6 seconds for the top);
    the next five follow on their own ("Holocrons 6 to 10 of 12"). The stacked tiles are
    light: a plate washed from below in the prize's colour, a card's credits (two coins,
    the amount and the card tucked in its corner, its plate alone, in the holocron's
    colour, with three rainbow bands for Holo; 18/10/2026) or a shader's blade. A grand
    prize plays the fanfare as its strip stops. Then everything found is laid out, the
    finest first, ten to a row, each coming in on its beat, with its name and its
    credits or how many the player holds, the credits they brought together, and Done.
  - **Grand finds** are told to everyone: a hub that sends the feed's `finds` (hub branch
    `feat/grand-finds`, not deployed yet) relays a Legendary or Solar card, or a
    Legendary or Mythical shader, to every SJK player as a chat line, "Sol opened a
    Mythical Holocron: Phoenix blade!" ([holocrons.md](holocrons.md#chat-lines)).
  - Enter, Space, Right or a click skip to the burst (to the strips' stops, then the
    next five, then the finds, for many), finish the entrance, then go on; Escape ends
    it (what was opened is the hub's already). A refusal says why ("That holocron was
    already opened."; a hub from before opening: "This SJK hub cannot open holocrons
    yet.") and the profile is read again soon.
- **The Cards tab** (the Collection's seventh): "N cards" and a line about sealed cards
  while there are some, the credits at the right of the headline and the kyber shards left
  of them, up to twelve kinds
  (six by two, the best first, each with "x3" and its finish; more are counted under
  them), and the kind chosen up close on the right, 470 frame pixels wide so a face's
  lore reads at 1080p (290 before faces), with its name, rarity, finish, edition, how
  many and when the first was found, and its set and number (a sealed one: what its
  finish is and that it is sealed). **Reveal N sealed cards** (or R) stands under the
  grid when the hub lets the player open holocrons and the packs have faces of a sealed
  copy's rarity; it says "Revealing..." until the hub answers, then a line says how
  many were revealed or why not. Left and Right choose. The tab's count in the row is
  the cards held.
- **The ceremony** shows a card the hub picked with its face, bigger (400 wide) and
  higher, its name as the title, "Legendary, Holo. Alpha Edition. From a Mythical
  Holocron" and its set and number in place of the sealed line.
- **The player card's** rarest holocron gem counts opened holocrons too, as the
  counts now hold only the unopened ones.
- `debug_open <tier|all> [x<count>] [normal|holo|solar|shader|shards]` plays the ceremony
  with made-up cards (or shaders, or piles of kyber shards) on the Holocrons tab; nothing is
  sent or kept. A card comes as a pack, with made-up shards from the tier's card shards. A rarity the packs have
  faces of shows one of them, as the hub would pick.

## Planned, not built

1. The rest of the first set: cards of every rarity (only Sol lore 001, Legendary,
   exists), so every sealed copy can be revealed; a reveal ceremony (the copies turn
   over one by one, as an opening's cards turn in).
2. The Cards tab as a binder by set with progress ("Places 7 of 12"), missing cards
   as dark outlines.
3. Deploying the economy ([Shards and the shop](#shards-and-the-shop)) and the Shaders
   tab's Shatter and Craft. The Staff page's credits and shards control is built
   (18/10/2026); shards wait for the hub's `/v1/staff/shards`.
4. Set completion feeding achievements the hub counts.

Suggested first sets, about 20 to 30 cards: Places (planets, the Jedi Academy, the
game's maps), Personalities (figures of the lore), Jedi Academy (Force powers,
saber forms, hilts, moments of the game), Community (players and servers, with their
consent).

## The art and the rights

Sol makes the pictures, with AI tools, in one shared style. Cards of the lore draw
on Lucasfilm and Disney properties and Jedi Academy itself, and the credits' sign is
Lucasfilm's, so the system stays strictly non-commercial. AI-made art is generally
not protected by copyright, so card packs cannot carry the "all rights reserved"
claim the blade skins' art does; the credits say the pictures are AI-made. Community
cards use a player's name and picture, or a stylised emblem, only with their consent,
never an AI likeness of a real person's face, and staff can take one down at the
player's request.

## Privacy

A person's cards and credits are public like their medals: anyone can read a
profile. The hub stores which copies a person holds and when each was obtained, and
every change to their credits with its reason.

## Checked and not checked

Verified (12/10/2026, Windows 11): the hub's tests on branch `feat/holocron-opening`
(odds tables, rolls, the API's opening of one and twenty, refusals that change
nothing, merges moving cards and credits, linked keys, staff credits); the client's
`cargo test --locked --workspace` (the wire, the worker's opening and refusals, the
lists, stacks and credits' words, the mark's and a card's fixed commands, the
rehearsal's words, the tab's keys, the ceremony's flow, timeout and room on the
canvas in three windows, the Cards tab's states and room, the renderer's cut of a
stroke). World shots `world_shot::cards_shots` (off-screen renders on duel6, made-up
data) were looked at: the tab with Open and the credits, a Holo card's entrance
moment by moment, each finish, the wait, a refusal, the Cards tab at 1080p and 4:3
and with none. Not tried: against the real hub (it is not deployed), in a game, the
keys and clicks in a running client, the motion (the shots are stills).

The economy (17/10/2026, Windows 11, prepared): the hub's tests (the tables, the trade
read from the catalogues, Paris's day across summer time, the offers stable and different
by day and secret, shattering never the last copy, crafting and buying once with enough,
two crafts with shards for one, refusals that change nothing, the gate, a pile from an
opening, staff and operator changes with their undo, the shop's secret kept across a
reopening, the API's shapes and errors); the client's unit tests (the wire with and without
the new fields, the service's trades and shop read, the tables' copy and the hub's, the
hooks' choices, the strip's shard share, the odds' figures, the shop's keys, confirm,
countdown and room in four windows). World shots `duel6_sjk_economy_shots` and
`duel6_sjk_economy_shop_windows` (off-screen, duel6, made-up data, the hub's packs mounted)
were looked at: the tab with both balances and Shop, the loot table's shards column, the
shop with a Confirm, while reading and after a purchase, in 4:3, 21:9 and 4K, a pile at
rest on the strip, coming in and its words, twelve opened with three piles at rest and laid
out, the Cards tab's balances; `duel6_sjk_card_pack`: a sealed Rare pack sliding out,
counting and with its words, one with a face, and the pack in 4:3. Not tried: against a running hub, in a running client, a
purchase's or a turned day's flow live.

A card's credits on the strip (18/10/2026, Windows 11, `personal/credits-roll`): unit tests
for the strip's credits (the prize's kept, every neighbour's in its tier's range and not
all alike, in every tier) and every moment of a single opening and of twenty at once,
twenty Holo cards among them, fitting the canvas in three windows. World shots
`duel6_sjk_card_pack` (the hub's packs mounted) were looked at: an Uncommon strip running
and at rest, a Rare and a Mythical one at rest (a Legendary Solar card's rays), the Rare
Holo pack rising, sliding aside as the card slides out, edge-on in its flip, flipped, its
rainbow flourish, a Solar one's gold flourish, the shards counting, its words, one with a
face and the words in 4:3; `duel6_sjk_crate_open` and `duel6_sjk_economy_shots` for the
stacked strips' light tiles at rest (their peak, 1 569 of the canvas's 2 400 shapes). Not
tried: in a running client (no motion seen; the flip plays no sound of its own).

The crate opening (17/10/2026, Windows 11): unit tests for the 3D holocron's motion
(glide, hum carried into the unseal, spin-up, swell, light, shake bounds, gone in the
flash), its handover (the one asked for, then the one opened, back in its place after),
the strip's stop always inside the prize's tile in both strip sizes and its speed curve,
the finds' order and grand ones, the batch's moments and skips, Open all's flow, and every
moment of a single opening (every tenth of a second) and of twenty at once (every
quarter) fitting the canvas in three windows. World shot `duel6_sjk_crate_open`
(off-screen, duel6, made-up prizes, no blade-skin packs so shader tiles show plain
blades) was looked at: the wait, the unseal early, charged and flashing, the strip out
of the flash, fast, crawling and at rest, a Solar Legendary card and a Mythical shader at
rest, and a batch of twelve cracking open, running, stopping, at rest, its last round and
its finds coming in and laid out. Not tried: in a running client (no motion seen, no
sounds heard: the cue files are checked to exist in the retail `assets0.pk3` only), with
the hub's packs, at 4K or 4:3 in a shot (the canvas test covers them).
