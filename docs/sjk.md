# SJK conventions

Rules that belong to SJK as a project: names, defaults, versions, releases,
credits. [AGENTS.md](../AGENTS.md) and [development.md](development.md) hold the
general contributor rules.

## Branches

- `main` of [Sol-Vulpes/SJK](https://github.com/Sol-Vulpes/SJK) is SJK. It is
  updated by merging, never rebased.
- A change is named `personal/<topic>`, starts from `main` and is merged back
  without a pull request. Contributors' changes arrive as pull requests on SJK
  ([development.md](development.md#branches-commits-and-pull-requests)).

## Names

- Everything a player sees says SJK: the programs `sjk` and `sjk-server`, the
  release ZIPs, the `GameData/SJK/` folder and messages.
- New settings get neutral engine names (`r_*`, `cg_*`, `cl_*`, ...), never a
  `sjk_` prefix, and must not collide with EternalJK or rend2 settings.
- A client mod's commands and settings start with its id and a dot
  (`japlus.guntele`, `jof.commands`), and its switch is `mod_<id>`
  ([mods.md](mods.md)).
- Every toy is a `toy_<name>` command (`toy_illuminate`, `toy_box`, `toy_order66`); a
  toy has no Force wheel entry, and one with a setting names it `cg_toy<Name>...`.
  Illuminate, the cardboard box and Order 66 have none: they are always available and
  on only for the client's run. A toy others see has an id in the look's `toys`
  (`cardboard_box`, `order66`). A toy may have a command of its own besides
  (`order66`, which gives the order).
- The crates are SJK's too: `crates/sjk-*`, packages `sjk-*` (`cargo build -p
  sjk-viewer -p sjk-dedicated`) and Rust paths `sjk_*`. Environment variables for
  diagnostics are `SJK_*`; the game-data and server paths keep the neutral
  `JKA_GAME_DATA` and `JKA_DEDICATED`.
- SJK's emblem (Sol's) is its picture: the SJK UI's and classic main menus, the
  programs' and window's icons, the README, release notes and site. Every image
  of it is generated from one original by the scripts in
  [assets/branding](../assets/branding/README.md); regenerate them rather than
  editing one by hand.
- Public text ([README](../README.md), [CREDITS.md](../CREDITS.md), the site)
  presents SJK as Sol's: Sol develops it and sets its direction, and Sol comes
  first (Sol's rule, 06/10/2026). SJK stands on its own (Sol's decision,
  08/10/2026): JKR, the engine SJK began from, and Bishop, its creator, appear only
  in the credits (CREDITS.md's "Origins", the credits page's Origins card), the
  copyright notice and one footnote in the README and on the site. Code, wiki
  pages and in-game text describe SJK without comparing it to JKR. Keep CREDITS.md
  current when SJK gains notable work.

## Defaults

A new profile starts with Sol's own choices. Defaults apply only to settings a
`config.cfg` has not saved: an existing profile keeps its values and nothing is
migrated, except where the table says so. Where each is documented:

| Setting | Default | See |
| --- | --- | --- |
| `ui_menuStyle` | `sjk` (the SJK UI), saved `classic` moved to `sjk` once, saved `modern` reset | [Menu style](client.md#menu-style) |
| `ui_gameFont` | on | [UI ownership](rendering.md#ui-ownership) |
| `cg_scoreboardStyle` | `auto` (SJK UI's with its menus, else `classic`), saved `modern` reset | [Scoreboard styles](client.md#scoreboard-styles) |
| `cg_compactScoreboard` | on (SJK UI's scoreboard in one column, as wide as its names, centred) | [SJK UI scoreboard](sjk-ui.md#scoreboard) |
| `cg_drawTimer` | on | [status.md](status.md#gameplay-and-interface-defaults) |
| `cg_teamOutline` | on (red rings on enemies, green on teammates, in team games) | [Team outlines and logos](client.md#team-outlines-and-logos) |
| `cg_killfeed` | on, saved 0 moved to 1 once | [Kill feed](client.md#kill-feed) |
| `cg_hitMarker` | off (red damage-direction indicator) | [Client](client.md#launch) |
| `cg_shieldBrightness` | 2 passes, old defaults 1 and 4 moved to 2 once | [Client](client.md#launch) |
| `cg_duelGhost` | on: duellists as JoF EJK's blue figures (off: dark under the bubble) | [Rendering](rendering.md#entity-render-effects) |
| `cg_dismember` | 2 | [Dismemberment](client.md#dismemberment-and-disintegration) |
| `snaps` | 120 (slider to 125) | [status.md](status.md#gameplay-and-interface-defaults) |
| `com_maxfpsUnfocused` | 30 | [Configuration](client.md#configuration-and-content) |
| `com_maxfpsMonitor` | off (`com_maxfps` AUTO caps at 125, the monitor's rate unread) | [Configuration](client.md#configuration-and-content) |
| `cl_maxpackets` | 125 | [User commands and move packets](networking.md#user-commands-and-move-packets) |
| `gameplay_fps` | 0, Auto: movement at the FPS cap, a `pmove_fixed` server's step wins | [Gameplay FPS](client.md#gameplay-fps) |
| `gameplay_fps_nocap` | 0: the known server caps (JoF's 125) hold | [User commands and move packets](networking.md#user-commands-and-move-packets) |
| `cjp_client` | empty: `1.4 SJK` on JA+, `1.4JAPRO` on jaPRO (its client-support switch) | [Networking](networking.md) |
| `rate` | 90000 (stock 25000), saved 25000 moved to 90000 once | [User commands and move packets](networking.md#user-commands-and-move-packets) |
| rendering profile | noon, bloom, dust, material maps | [Default visual profile](rendering.md#default-visual-profile) |
| `cl_consoleUseScanCode` | 1, saved 0 moved to 1 once | [Useful console commands](client.md#useful-console-commands) |
| `mod_japlus`, `mod_jof` | on (each mod loads on its servers only), saved 0 moved to 1 once | [Client mods](mods.md) |
| `con_drawNotify` | off (no console feed at the top left; chat keeps its box) | [Classic console](client.md#classic-console) |
| `cl_experimentalShowcase` | off (the experimental `ghost` and `mimic` commands) | [Ghost replay](client.md#ghost-replay) |
| `ui_donationGoal` | on, but shelved since 14/10/2026: the card never shows ([Donations](#donations)) | [Main page](sjk-ui.md#main-page) |

The HUD look (`cg_hudStyle game`) and the console (`con_style auto`: the SJK UI's
console with its menus, else the classic one; a saved `classic` moved once to
`auto`) are SJK defaults too, documented with their pages; a saved `modern` in
either is reset at start, as the modern style is retired
([Menu style](client.md#menu-style)).

## Fonts

SJK draws no bitmap fonts (Sol's rule, 07/10/2026): text uses vector fonts,
bundled under an open licence (OFL or Apache) and rasterized once at startup, as
Inter is ([text.rs](../crates/sjk-viewer/src/text.rs)). Retail bitmap fonts are
replaced, by an open-licence look-alike or by converting them to vector outlines
where that is possible, never added. A bundled font ships with its licence next to
it in `crates/sjk-viewer/assets/fonts` and is credited in CREDITS.md and
credits.txt.

| Retail bitmap | Used for | Replacement |
| --- | --- | --- |
| `gfx/2d/charsgrid_med` | Console, notify lines, cgame small/big strings | JetBrains Mono ([console_font.rs](../crates/sjk-viewer/src/text/console_font.rs)) |
| `ocr_a` | Chat, selection names, scoreboard numbers (`ui_gameFont`) | SJK Chat: OCR-A (public domain) set to the retail widths ([retail_font.rs](../crates/sjk-viewer/src/text/retail_font.rs)) |
| `ergoec` | Menus, centre prints, scoreboard names (`ui_gameFont`) | SJK Menu: traced from the JoF HD pack's atlas |
| `arialnb` | Classic status HUD (`cg_classicHudFont`) | SJK HUD: traced from the JoF HD pack's atlas |
| `logo` glyph (0xAC) | The "WSI fonts" logo in Inter's atlas | SJK Menu's vector logo ([logo_glyph.rs](../crates/sjk-viewer/src/text/logo_glyph.rs)) |

Characters outside the 256 code-page slots (Cyrillic, Japanese, Chinese, Korean)
come from the same vector fonts, Cyrillic from Inter, Exo 2 or JetBrains Mono, kana
and kanji from Noto Sans JP (OFL), subset to kana, kanji, CJK punctuation and
full-width forms (3 MB), the GB 2312 hanzi Noto Sans JP lacks from Noto Sans SC
(0.5 MB) and Hangul from Noto Sans KR (1.1 MB), all three subset with fontTools.
Emoji are Twemoji's colour pictures (CC-BY 4.0), not a font
([rendering.md](rendering.md#characters-beyond-the-code-page)).

SJK Menu, SJK Chat and SJK HUD keep the retail layout (advances, glyph positions,
line height and baseline), so every surface lays out as it did with the bitmaps.
[game_fonts.py](../scripts/game_fonts.py) rebuilds them from a player's install
(the retail `.fontdat` metrics in `assets1.pk3`, the HD atlases of
`JoF_HDFonts&Icons.pk3`) and Sauter's `OCRA.ttf`, none of which are committed:
`ergoec` and `arialnb` are traced with light smoothing; OCR-A gets the retail
widths, each glyph centred in its advance (the retail atlas cells are padded
unevenly, so following them spaced the even OCR-A letters unevenly), and the Latin-1 letters and symbols it lacks are composed from its own
glyphs (accented capitals squashed under the accent, as its own `Ñ` is), except
seven (`ß þ Þ ð § ¶ ¤`) traced from the HD `ocr_a`. Sol chose the traced menu and
HUD fonts and OCR-A Regular from side-by-side comparisons (07/10/2026). The JoF
pack's author is unknown and it states no licence: Sol chose to bundle the
tracings on the assumption that it is shared freely, crediting the pack and
removing them at the author's request.

The first build (07/10/2026) placed most glyphs wrongly: fontTools'
`removeOverlaps` draws through the glyph set, which moves each outline to its
`hmtx` left bearing, and the bearings were still 0, so every glyph with
overlapping contours (most of OCR-A, about half of the traced fonts) landed flush
left while the rest kept their place. `game_fonts.py` now sets the bearings from
the outlines before that step.
[SJK-fonts.txt](../crates/sjk-viewer/assets/fonts/SJK-fonts.txt) records this
next to the fonts.

## Copyright notice

[NOTICE](../NOTICE) is SJK's copyright notice: Sol-Vulpes, Bishop-R and the JKR
contributors, under GPL-2.0-only. Bishop-R and the JKR contributors hold the
copyright of the code SJK began from, so the line keeps naming them. The client
([notice.rs](../crates/sjk-viewer/src/notice.rs), printed to its log and console
by `app_launch.rs`) and the dedicated server (`NOTICE` in its `main.rs`) announce
the copyright and the absence of warranty at startup, so GPLv2 section 2(c)
requires modified versions to keep announcing them. Keep the two copies of the
lines in step. SJK packages put `NOTICE` first in `LICENSES-SJK.txt`. The SJK
names and emblem are not licensed with the code (see `NOTICE`).

## Version

One file decides the programs' version: [build_version.rs](../scripts/build_version.rs),
which both programs' `build.rs` include. Releases are numbered by date and a
counter, `YYYY.MMDD.N`: the first release of 05/10/2026 is `2026.1005.1`, a second
one that day `2026.1005.2` (Sol's choice, 05/10/2026). The form sorts, is a valid
Cargo and Windows version, and is tagged `sjk-v2026.1005.1`; people see the date
itself day first in the label below. A release build gets the version of its tag
through `SJK_VERSION`, which the release workflow sets; any other build says `dev`,
so a local build never passes for a release. The Cargo package version (`0.1.0`)
is not the release number. The source commit's short hash and committer
time come from git when the source is a checkout, else from `SJK_COMMIT` and
`SJK_COMMIT_TIME`, else they are left out. A build is redone when `HEAD` moves;
uncommitted edits keep the last commit's label. The client shows
`SJK <version> · <dd/mm/yyyy HH:MM> · <commit>` at the top of the screen
([client.md](client.md#version-label)) and in its log, the menus and console show
`SJK <version>`, and both startup notices carry the version.

## Dates and times

Dates that players see, and dates in SJK's own text (release notes, Discord posts,
SJK's own pages), are written day first, `dd/mm/yyyy`, and times on the 24-hour
clock, `HH:MM`, with no AM or PM (Sol's rule, 05/10/2026). The version label and the
classic console's clock follow it. Machine formats are exempt: git tags, ISO 8601
timestamps in logs and JSON, and file names meant to sort.

## Automation

| Workflow | Runs on | Does |
| --- | --- | --- |
| [CI](../.github/workflows/ci.yml) | Pull requests, pushes to `main` | Formatting, workspace build, tests and clippy |
| [Pages](../.github/workflows/pages.yml) | Pushes to `main` changing `site/` or the workflow | Publishes `site/` to https://sol-vulpes.github.io/SJK/ |
| [SJK release](../.github/workflows/release.yml) | Tags `sjk-v<version>` | Builds and publishes the release ZIPs |

The site is static: `index.html`, `assets/style.css` and `assets/site.js` (release
list, download entry, screenshot gallery, donation card). It wears the SJK UI's look
([sjk-ui.md](sjk-ui.md), Sol's request of 09/10/2026): the home page is the
client's main page, SJK's emblem in its turning ring with a plain menu beside it
(a gold Download button, then the sections in one column; Sol found the first
version's arc, whose entries grew and moved under the pointer, hard to use), the
latest release where the client lists recent servers, a profile card for Sol with
Sol's GitHub picture (the initial shows if it does not load),
over shots of the client's tour of Yavin Training Grounds that cross-fade every 15
seconds; What SJK brings is laid out as the client's Settings (a rail of categories,
rows, a detail column with a screenshot). Below 900 px wide (or in a tall window) the
home page stacks into a list. Its pictures are the client's own off-screen renders
([screenshots/README.md](../site/screenshots/README.md)). The sun behind the
emblem (a warm glow, uneven god rays and an 18-ray sunburst turning against each
other) is CSS (`.sun` in `style.css`), so every browser shows it; it once was a
WebGPU layer, which headless Chrome cannot capture in a screenshot.
`assets/effects.js` adds drifting dust and a saber trail on the pointer with the
MIT-licensed
[Shaders](https://github.com/shader-effects-inc/shaders) library, vendored as
`assets/vendor/shaders/shaders-4.0.0.js` with its license so no other server is
contacted; its telemetry is turned off. When the visitor asks for reduced motion
(Windows' "Animation effects" off does), the scene is drawn once and held still,
without the saber trail; the backdrop stays on its first shot and the sun stops turning. Without WebGPU
the script adds nothing. To
update the library, replace the vendored file with a release's
`dist/js/bundle.js` and check that `disableTelemetry` still stops its telemetry.

Release tags exist only on SJK and are created on Sol's request. The release
title carries the stage, "Sol JK 2026.1005.1 (Alpha)" (`STAGE` in the workflow).
Alphas are not marked as pre-releases, so the site's download link
(`releases/latest`) finds them; their name and notes say "Alpha" instead.

## Donations

**Shelved since 14/10/2026 (Sol's decision).** Nothing asks for donations for now: the
site's Support SJK section and its two menu links carry `hidden` and `site.js` skips the
hub (`DONATION.shelved`); the client's card never shows and its setting is out of the
settings screens (`donation_goal::SHELVED`), and the Collection lists the Supporter
medal only to players who hold it. The hub's webhook, goal endpoint and medal stay as
they are. To bring it back: `DONATION.shelved = false` and the three `hidden`
attributes in `index.html`, then `SHELVED = false` and the "Donation goal" row back in
`settings/catalog.rs` (GAME), `settings/groups.rs` (Interface) and its help line in
`settings/help.rs`, all three in git history before 14/10/2026. What follows is how
it works once back.

SJK takes donations on Ko-fi, https://ko-fi.com/soljk (Sol's page, 13/10/2026). The
site's Support SJK section and the SJK UI's main page ([sjk-ui.md](sjk-ui.md#main-page))
say what they are for: a full Claude plan used only for SJK, so more of the tasks Sol
has ready can run as parallel agents. A Claude plan is billed monthly, so the goal is
monthly ("80 € of 200 € this month") and starts again on the 1st.

The goal is live: Ko-fi calls the SJK hub's webhook for every payment and the hub sums
them (the hub's `PROTOCOL.md`, "Donations"). Ko-fi has no API to read a goal, and its
public page does not carry it, so the goal set on Ko-fi is repeated in the hub's
configuration, which the hub operator keeps. Both the site and the client read
`GET https://sjk.dfox.app/v1/donations/goal` (no key, readable from any origin):

| Reader | Where | Without an answer |
| --- | --- | --- |
| Site | `DONATION.hub` at the top of [site.js](../site/assets/site.js), fetched once per visit (6 s at most) | `DONATION`'s own `url`, `goal`, `raised` and `monthly`; without JavaScript, [index.html](../site/index.html)'s "200 € a month" and its Ko-fi link |
| Client | [donation_goal.rs](../crates/sjk-viewer/src/donation_goal.rs) through `cl_hubUrl`, every ten minutes while the main page shows | The card without numbers, Donate still opening Ko-fi |

The Ko-fi address is written in three places, all for when the hub cannot answer: the
site's `DONATION.url` and its link in `index.html`, and the client's
`sjk_identity::donations::DEFAULT_URL`. Changing the page means changing the three and
the hub's configuration. `ui_donationGoal 0` hides the client's card and asks nothing.

A donation of more than 1 euro earns the Supporter medal
([identity.md](identity.md#medals)): with the donor's SJK key id in the Ko-fi message
(Settings, Network, SJK identity key, Copy key id), the hub gives it as soon as Ko-fi
calls; without one, or in another currency, the SJK team matches it by hand. The site
shows the medallion from `site/assets/medal-supporter.png`, a copy of the client's
`supporter_small.png`: replace both when the art changes. The page promises nothing
else: no feature is sold and a donation brings no advantage in the game.

## Changelog

[CHANGELOG.md](../CHANGELOG.md) lists every release, newest first, each change
closed by its credit: who made it (Sol, a contributor) and "after <client>" when it
follows another client's behaviour (EternalJK, JoF EJK). The client builds the
file in and shows it on its changelog page (main menu > Changelog, or the
`changelog` command; [client.md](client.md#changelog-page)), and its tests reject
a change without a credit, a date that is not `dd/mm/yyyy` or a non-ASCII
character. The merge that brings a player-visible change into `main` adds its line
under "Unreleased"; tagging a release renames that section to
`<version> | <dd/mm/yyyy>` and its lines become the release notes' "New since"
list. The Discord changelog posts are made from the same file.

Every release carries a short recap (Sol's rule, 12/10/2026): three to six lines
`> <Heading>: <one sentence>` between the release's title and its changes, in player
words, each heading at most 28 characters and each sentence at most 160, grouped by
what a player notices (servers, gameplay, tools, cards) rather than commit by commit.
Put a command or cvar the player can type in backticks (`` `whatsnew` ``), in a recap
or a change: the card and the changelog page draw it in a chip the player can click.
The client's what's-new card shows them to a player the first time they open the game
after updating ([sjk-ui.md](sjk-ui.md#whats-new-card)). Write the recap when the
release is cut, from the "Unreleased" lines, in the same commit that renames the
section; the tests reject a dated release from 12/10/2026 on without one, and the
release notes' introduction may repeat it. A player who skips releases sees the recaps
of each, so a recap must read on its own.

## Credits

[credits.txt](../crates/sjk-viewer/assets/credits.txt) feeds the client's credits
page ([client.md](client.md#credits-page)); [CREDITS.md](../CREDITS.md) stays
the full written record. Sol develops SJK alone since October 2026; Bishop is
credited for JKR, the engine SJK began from, in the Origins section of both.

Credits are kept for everything (Sol's rule, 06/10/2026): every merge into
`main` that brings someone's work updates credits.txt, CREDITS.md and the
changelog line's credit in the same change. A person gets a card in the merge
that brings their first change (Creyon with SJK pull request #2), and their card
gains lines, with the pull request numbers, as they keep contributing; Sol's own
card gains a line when a notable feature lands. Cards carry the person's GitHub
handle (`github:`, opens their profile when clicked) and may carry `link:`
addresses (https only, optionally `Label | https://...`) and `medal:` lines, the
medals the SJK team gave them, by the hub's ids (`early_contributor`;
[identity.md](identity.md#medals)), shown on the card with their pictures and
names. Sections are free: a later "Supporters" section is a `== Supporters`
heading and its cards, with no code. The tests reject a card without a role, an
unknown key, a link that is not https, an unknown medal id, a medal listed twice
(but Bug Hunter, which counts up) or a non-ASCII character.

Under each card the page folds that person's whole history: every feature and
pull request merged into `main`, with its commits. It comes from
[credits_history.txt](../crates/sjk-viewer/assets/credits_history.txt), which
[credits_history.py](../scripts/credits_history.py) writes from git and, through
`gh`, GitHub's pull request titles and authors; the file is never edited by
hand. Every merge into `main` regenerates it: merge, run
`python scripts/credits_history.py` in the merged checkout, and amend the merge
commit with the file (`git commit --amend`), so the merge carries its own
history and no extra commit appears in it; `--check` says whether the file is
current. A new contributor goes into the script's `PEOPLE` (their git name and
GitHub login) along with their card; the tests reject a person in the history
without a card.

## Debug panel

The `debug_panel` console command lists SJK's changes and how to test them, from
[debug_panel.txt](../crates/sjk-viewer/assets/debug_panel.txt). It is Sol's
personal test list. Update it in the merge that brings a change into SJK's `main`.

Since 10/10/2026 the panel is a full-frame SJK UI page in every menu style, as the
Profile and SJK chat pages are ([sjk-ui.md](sjk-ui.md#sjks-pages)): the way back and
"Test list" at the top left, the tabs To test, Tested and All under them, how many are
tested at the top right; the entries down the left (tick box, title, area), the chosen
one on a band; its details in a reading column on the right (area, title, its tick,
What changed, To test numbered in gold, Note on a gold band), the last tick's outcome
bottom left and the keys bottom right. Up and Down (Page Up, Page Down, Home, End)
choose, Space or Enter (or a click on a tick box or the Space key hint) ticks, Tab and
Shift+Tab change the tab, the wheel scrolls the list wherever the pointer is, Escape
closes. Ticks stay saved by id in `debug_panel_tested.txt`. It still draws over the
command browser when both are open. World shot:
`world_shot::tests::duel6_debug_panel`.
