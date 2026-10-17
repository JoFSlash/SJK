## FPS boost graphics preset

17/10/2026, `claude/inspiring-davinci-s81e0l`: a fourth graphics preset, FPS boost,
first in the list, for weak PCs with an integrated GPU: Vanilla style with impact
marks, player shadows, weather, Force Speed afterimages, footprints and dust, and
saber steam in rain off too ([client.md](client.md#graphics-quality)). The other
presets leave those small effects as the player set them; leaving FPS boost turns them
back on. It counts as the Vanilla renderer, so the low-FPS help does not offer Vanilla
on it. Checked by unit tests. JoFSlash's Ryzen 7 PRO 4750U (Radeon Vega 7) laptop:
about 100 FPS on FPS boost against about 50 before (one report, settings and map not
recorded; EJK 1.6.2 gave 100-120 there).

## NPC sabers and voices

17/10/2026, `personal/npc-sabers-sounds`, based on `Sol-Vulpes/SJK` `cb03b251`
(hub report 68, a JoF server on `JoFCrucible`): implemented, after JoF EJK
`codemp`, NPCs' thrown sabers (owners past the client slots, `npcSaber1` hilt),
NPC and vehicle custom sounds from their own `csSounds_Std`/`_Combat`/`_Jedi`
folders instead of player slot 31's voice, `.sab` blade colours for NPCs without a
`boltToPlayer` override, and no primary saber while `saberEntityNum` is zero. See
[client](client.md#npc-sabers-and-voices).

Workspace fmt/build/clippy and the touched crates' tests passed on Windows
x86_64. Unit tests pin the thrown-saber owner range and NPC hilt/colour, the
per-set NPC voice (own set, silence without one, temporary entities naming an
NPC, players unchanged, slot reload), the colour overrides and the missing-saber
rule. Not exercised in a live match or against the reported server; the `_Extra`
NPC sounds have no event in the client's table and stay unloaded.
## JoF EJK jetpack sound choices

17/10/2026, `feat/jof-jetpack-sounds-upstream`, based on `Sol-Vulpes/SJK`
`94b1d1ce`: implemented JoF EJK #43's archived ignition/hover selectors, both
default 1, with the original integer branches and sound paths. Shutdown is
fixed; the flaming fire loop is independent. Both menu styles expose the
choices. Start/stop now use #87's `CHAN_AUTO`, retaining the wearer's actual
source and position. See [client](client.md#jetpack-sounds) and
[rendering](rendering.md).

Workspace fmt/build/test/clippy checks passed on Windows x86_64, Rust 1.99.0.
Four focused regressions pin integer/default choices, fixed shutdown,
source/channel/position, live local/remote hover selection, independent fire,
retained loop storage and archived preferences without restarting active jets;
the existing jetpack tests pass too. Source evidence is EJK `codemp/cgame` at
#43 merge `364c5ff1` and #87 merge `c3e2331f`. Live listening and jetpack
reproduction were not exercised; game assets are not configured for the
ignored installed-assets/GPU check. These checks do not establish audible or
gameplay parity, or populated-match performance.
## Tavion and Desann idle poses

17/10/2026, `feat/jof-idle-stances`, based on `Sol-Vulpes/SJK` `94b1d1ce`:
implemented the distinct idle clips from JoF EJK #47 (`1fbe4678`,
`codemp/game/bg_pmove.c::PM_GetSaberStance`) as default presentation, with no
toggle. A two-entry style/clip table replaces Tavion's fast idle with
`BOTH_P1_S1_T_` and Desann's medium idle with `BOTH_STAND9`. Snapshot entities,
local player snapshots and predicted actors share the mapping, independent of
menu style. Simulation inputs and attack timers are unchanged. Other clips,
unlit/thrown sabers, flips and forced frames are retained. See
[client](client.md#tavion-and-desann-idle-poses).

Five focused Rust regressions passed on Windows x86_64, Rust 1.99.0. They pin
the source-selected ordinals, exclusions and snapshot/prediction agreement at
command times advanced by 8/7/4/3 ms; they do not exercise movement simulation
or establish gameplay parity. Source inspection also checked `bg_misc.c`'s
`s->fireflag = ps->fd.saberAnimLevel` style transfer. Workspace formatting,
build, tests and Clippy passed (existing warnings remain).
Installed-animation/GPU appearance and live-server behavior remain
unverified; no game assets are configured for an in-game comparison.
## JA+ black sabers

17/10/2026, `feat/japlus-black-sabers`, based on `Sol-Vulpes/SJK` `94b1d1ce`:
implemented JoF EJK #50's black-blade rendering and corrected plugin-bit
semantics. Colour 11 uses an independent alpha-blended material and trail for
both hands and thrown sabers, without blade light or dynamic glow. A set
`cp_pluginDisable` bit 3 produces orange; the existing mask default 1536 allows
black. Both settings interfaces include the default-on `cg_blackSabers`
presentation override. Automatic support follows the session's `CompatProfile`
(JA+, TaystJK/jaPRO). Review fixes (17/10/2026): no second detection table;
colour 11 keeps purple/RGB off those servers (EJK shows orange there); the scan
is cached until configstring 0 or a `CS_PLAYERS` slot changes and covers the 32
client slots only. Normal RGB and SJK blade skins retain their paths.
See [client](client.md#ja-black-sabers) and
[rendering](rendering.md#ja-black-sabers).

Reference evidence: EJK merge `4ebc7e7e`,
`codemp/cgame/cg_players.c::ClampSaberColor`, black material selection,
`doLight = qfalse`, and the two alpha stages in
`assets/japro/shaders/sbRGB.shader`. Installed community textures are optional;
generated alpha profiles provide a local fallback.

Workspace fmt/build/test/clippy passed on Windows x86_64, Rust 1.99.0 (Clippy
warnings remain). Eight focused regressions cover bit polarity, colour and
server transitions, both hands, cosmetic suffixes, muted players, material
separation, no blade light, synthetic installed/generated textures, settings,
trail timing and WGSL validation. An explicitly run offscreen GPU pixel test
passed on NVIDIA GeForce RTX 5070 Ti / Vulkan: generated blade centre
`[0, 0, 0, 255]`, white background preserved. It checks the blade alpha pass,
not the trail draw or a complete in-game scene. A release measurement on
AMD Ryzen 7 5800X3D averaged 4.586 us for a 64-slot policy scan over 20,000
synthetic frames; this excludes trails, submission, rendering and GPU work
and does not establish a game FPS rate. Live server behavior, installed-texture
appearance, crowded-scene performance and EJK pixel parity remain unverified;
simulation and codecs are unchanged.

## One Russian letter in chat

17/10/2026, `personal/cyrillic-one` (Foshin's report, builds 2026.1012.x-1013.x, JoF / JA+: a
message of one Russian letter "bugs out"). Cause: under `cl_charset auto` a single-byte
encoding is used only when `looks_cyrillic` reads it back, and the word rule needs two
letters with a lower-case one or four letters, so `я`, `Ж` or `ДА` were sent as UTF-8
(`D1 8F`), which JoF, JA+ and EternalJK players draw as two garbage characters; received
as the byte they send (0xFF), `я` showed as `ÿ`. Now a message (after a `"` in a typed
command, after the `: ` of a received line, or a localized chat's body) with no ASCII
letter and a high letter reads as Windows-1251 (`looks_cyrillic`,
`decode_legacy_message`); names keep the word rule (`ÆØ` stays Latin-1). Unit tests send
and read back every `А`-`я`, `Ё`, `ё` and short mixed messages (`chat::escape_tests`,
`legacy_text::tests`). Not tried on a server. Limits: a letter next to ASCII words
(`я lol`) still goes as UTF-8, and a lone Latin `à` message now reads as Cyrillic
(`cl_charset western` pins it); see [networking.md](networking.md#player-text).

## Player card in sight only; grey bars for every non-sharer

17/10/2026, `personal/inspect-sight` (Sol's request). The player card, dwelled or
pinned with `inspect`, needs its player within 1200 units of the camera with no wall
between the camera and their head or hips (the nameplates' trace); a pinned card fades
out of sight and unpins after 3 seconds ([client.md](client.md#player-card)). Nameplates
give grey bars to every player who does not share, bots and players without SJK
included (the SJK-only check, `player_identity::sjk_slots`, is gone), so
`cg_nameplateEstimate` ("Bars on everyone") had nothing left to show and was removed
(Sol's choice); the client always sends the hub `estimates: false`, and cards still read
the flag from older clients. Checked by unit tests (the new `Sight` grace); not tried in
the game.

## Chroma blades on pale custom colours

17/10/2026, `personal/rgb-saber` (Kushina's report, build 2026.1016.2: moving the Green
slider five steps turned a Spectral blade and its thumbnails from violet to cyan). A
chroma shader blade turns its colours to the wearer's hue (`saber_skins::hue_of`), but
`CHROMA_GREY` 0.2 took every colour under 20% saturation for grey and left the skin at its
own hue: RGB 207 185 234 (0.21) turned, 207 190 234 (0.19) did not. The threshold is now
0.03. Unit tests: `a_custom_tint_never_jumps_hue_between_neighbouring_values` and, for
the plain path (all fine, kept as regression tests), a custom tint's round trip through
the menu, userinfo and configstring (`saber::tests`, `presentation_equipment::tests`).

## Linux clipboard in process

17/10/2026, `personal/linux-paste` (hub report 58, Nib, build 2026.1015.1): Ctrl+V did
nothing on Linux because the clipboard only went through `wl-paste` or `xclip`, which
many desktops do not install. `clipboard.rs` now tries `arboard` first on Linux (X11 and
Wayland data-control, one `Clipboard` kept for the run so X11 can still serve a copy),
then the tools as before. Every text field shares `clipboard::copy` and `paste`, so all
get the fix; the key detection (`text_entry`, `sjk_chat_field::is_paste`, the console's
`shortcut_of`) already covers Linux's U+0016 text. Not run on Linux: the cross
`cargo check` needs ALSA/pkg-config sysroots; untested in a Linux session.

## Capture the flag HUD

17/10/2026, `personal/ctf-flags` (Slash's report from JoF and JA+ servers: no flag on the
scoreboard, no sign of carrying one, no team scores on the HUD). The scoreboards already
drew a carrier's flag from the `scores` command's powerups; the HUD drew nothing but the
upper-right powerup icon, which `cg_drawUpperRight` and `cg_drawPowerupIcons` can hide and
which took Capture the Flag for the ysalamiri variant. Added in every HUD style: the flag
icons at the left (`CG_DrawFlagStatus`, from `CS_FLAGSTATUS`), a gold plate and "YOU HAVE
THE FLAG" while the local player carries one, and the team scores at the top centre
(`cg_drawFlagStatus`, `cg_drawTeamScores`, Settings > HUD). The scoreboards also mark a
carrier in view from the snapshot, for servers whose `scores` list no powerups
([client.md](client.md#capture-the-flag-hud)). Checked by unit tests and world shots
`world_shot::ctf_flags_shots::ctf_flags_on_a_demo` (three HUD styles, two states, over
Sol's demo); not tried on a live server.
## Clipboard tools follow the desktop session

17/10/2026, SJK pull request #115 (LukasMoudry), on top of the in-process Linux clipboard
already in `main`: the fallback tools now follow the session, `wl-copy`/`wl-paste` only
with a Wayland display and `xclip` only with an X11 display, and a selection owner gets
50 ms to report an immediate startup failure before it is accepted. Console and browser
copy notices claim success only after the clipboard accepted the text, and a failed cut
keeps its selection. Checked by unit tests; the real X11/Wayland clipboard and the
in-game interaction have not been tried.

## Two-key chord binds from JoF EJK

17/10/2026, `feat/key-combo-binds`: the console accepts arbitrary two-key binds
such as `bind K+L kill`. Either press order works; the first key keeps its normal
press, the chord replaces the second key's normal bind, and a chord `+command`
gets its release. Query, `unbind`, `unbindall`, `bindlist`, generated configs and
the config importer understand chords. The implementation follows JoF EJK alpha
commit `b6623f51` (`codemp/client/cl_keys.cpp`), including its 256-chord limit,
last-plus parsing (`++K`) and first-match table order. Unit tests cover name
normalization, both press orders, release behavior, focus-state clearing,
console commands, config round-trip and import. On Linux x86_64 with Rust 1.99.1,
`cargo fmt --all --check`, the locked workspace build and tests, workspace
all-target clippy, and the release viewer/dedicated build pass (existing clippy
warnings remain). It has not yet been tried in the game; Sol will test the local
build before anything is pushed.
## JoF EJK camera settings

17/10/2026, `feat/ejk-camera-settings`: Advanced > Gameplay > Camera and
prediction and Classic+ Setup > Game options expose JoF EJK's functional camera
menu controls: first-person weapon FOV, its three damping timing presets,
third-person distance, angle, pitch, height and horizontal offset, and camera and
target damping. The ranges, defaults and preset names follow the official JoF EJK
alpha commit `9822a9d6` (`1.6.1-alpha.347`), whose
`assets/japro/ui/jamp/ingame_camera.menu` is unchanged since its initial
`52e068d0` import. The underlying archived
cvars and camera behavior already existed in SJK; this change adds their shared
SJK/classic settings rows and help. On Linux x86_64 with Rust 1.99.1, the focused
settings tests (101 passed), formatting, locked workspace build and tests,
workspace all-target Clippy, and release viewer/dedicated build pass (existing
Clippy warnings remain). The settings have not been tried in the game; the
GPU/retail-data snapshots were not run.

## SJK chat messages up to 300 characters

16/10/2026, `personal/chat-300` (Sol's request), with the hub's `chat-300` (deployed
16/10): `sjk_identity::chat::TEXT_MAX` 300 (was 150), the client's paste and `sjksay`
cut follow it; a client from before shows a longer message's first 150
([hub-chat.md](hub-chat.md)). Checked by unit tests; not tried in the game.

## Grey bars for SJK players who do not share

16/10/2026, `personal/nameplate-withheld`, Sol's rule: an SJK player (the hub lists them
in that slot under the shown name) who does not share their bars gets grey, empty bars
on every viewer's nameplate, whatever that viewer turned on: `cg_nameplateEstimate`
and a teammate's `tinfo` no longer show their values ([client.md](client.md#sharing-and-estimates)).
Players without SJK keep the 15/10 behaviour. Client only, no hub change. Checked by
the unit tests and the nameplate snapshot (`menu_snapshot::nameplate_snapshot`, a grey
plate added); not tried in the game.

## Name emoji

16/10/2026, `personal/name-emoji` (Sol's request): a player picks one emoji on the
Profile page (Name emoji: a grid of 60, or any single emoji typed or pasted) and everyone
sees it at the right of their name: SJK and game chat lines, the dock, the SJK chat page
and its people, sender cards, the profile card and the SJK scoreboard
([identity.md](identity.md#name-emoji)). Rule shared with the hub
(`sjk_identity::name_emoji`). Needs the hub's `name-emoji` branch (Sol-Vulpes/SJK-hub,
on top of `trust`): an older hub refuses the change and sends no emoji. Checked by unit
tests (the rule, the service, the Profile page's panel) and world shots (Profile page,
chat box, scoreboard). Not tried in the game or against a running hub.

## Order 66 toy

17/10/2026, `personal/order66` (Sol's idea): a meme toy ([client.md](client.md#order-66)).
`toy_order66` (bind, quick wheel Toys page, Collection's Toys tab) turns it on, and the
look's `toys` tells the other SJK players. `order66` gives the order through the hub's
existing toy events (`execute_<moment>`, no hub change): every client with the toy on
counts down to the same game server moment, 3 seconds on, and sends `kill` unless its
player is spared (spectating, dead, in a duel, carrying a flag); one order counts every
30 seconds per server. Holders who die in the order's window blow up on every SJK
client: the thermal detonator's explosion and six pieces (head, arms, legs, trunk) cut
with the dismemberment code (`GpuState::blow_apart`) that bounce off the world and sink
away, or confetti with `cg_dismember 0`. Limb meshes are pooled during the countdown.
Pooled limb meshes are no longer taken as the fallback mesh of entities without one.
Checked by unit tests (event parsing, hearing and cooldown, countdown and moment,
blasts once in the window, the piece's bounce and rest, the throw, ids, the bundled
files and the confetti's effect and shaders, the hidden body and its trunk, the
Collection's three toys and Order 66's switch, the wheel's Toys page) and an off-screen
render of a demo player (`jedi_hf`) blown apart, the pieces at four moments and the
confetti (`world_shot::order66_shots`); that shot found `jedi_hf`'s head under
`head_off`, so Order 66's cuts also take a surface's `_off` name. Not tried in a game,
with two clients or on a server.

## Cardboard box toy

16/10/2026, `personal/cardboard-box` (Sol's idea) with hub branch `cardboard-box`: a
Metal Gear style carton to hide in ([client.md](client.md#cardboard-box)). `toy_box`
(bind, quick wheel Toys page, Collection's Toys tab) puts it on: the commands hold
crouch, its music loops on the player, and other SJK players see the carton through the
look's new `toys` list. `+use` facing a boxed player lifts it (spotted sound, hub toy
event `POST /v1/toy`, [hub-chat.md](hub-chat.md#toy-events)); dying in it plays a death
sound for every SJK client that sees it. Putting it on plays a sound; taken off or
lifted, it tips over on its side with a drop sound and lies there 5 seconds. The carton
is drawn by `scripts/cardboard_box_assets.py`. The five sounds
(`sound/sjk/cardboard_box/music`, `equip`, `spotted`, `drop`, `death`) are not in the
repository: Sol's are Metal Gear clips, installed only in Sol's GameData
(`base/zz_sjk_box_sounds_local.pk3`) for tests (AGENTS.md, other games' assets).
Checked by unit tests (the model, crouch, `+use` presses and the target pick, spotted
and death logic, the equip sound and the fall, looks' toys both ways, the identity feed
queue and send, the look body, the Collection's two toys and their layout at four
sizes, the wheel's pages) and an off-screen render of two cartons on duel6, one tipping
and one lying (`world_shot::cardboard_box`). Merged into main on Sol's word. The hub
branch has its own unit and API tests and is not deployed: until it is, the live hub
refuses the look of a player wearing the box (its look body takes no unknown field).
Not tried in a game or with two clients.

## Emote help card

16/10/2026, `personal/emote-help` (Sol's request): `help_emotes` or `sjkemote help`
opens an SJK pop-up card listing the `sjkemote` commands (play, stop, sync, stop
others, mute) and how to put emotes on the quick wheel
([hub-chat.md](hub-chat.md#emotes-in-the-client)). Checked by unit tests and the
`emote_help_card` world shot; not tried in the game.

## Emote stop and mute

16/10/2026, `personal/emote-mute` (Sol's request): `sjkemote stop all` or `sjkemote stop
<player|target>` ends other players' emotes on this screen once (their next one plays);
`sjkemote mute <player|target>` hides one SJK player's emotes by their key until
`sjkemote unmute <player|all>` or the game closes, `mute` alone lists them
([hub-chat.md](hub-chat.md#emotes-in-the-client)). Local only, nothing sent to the hub.
Checked by the emote unit tests and clippy; not tried in the game.

## Unverified chat limits

16/10/2026, `personal/trust` (Sol's request): a key that is neither verified nor staff
sends one SJK chat message every 30 seconds and no links or GIFs, refused by the client
first with how long to wait, then by the hub (branch `trust`)
([hub-chat.md](hub-chat.md#limits-for-unverified-players)). Checked by unit tests
(`has_link`, the worker's wait and link refusal). Not tried in the game.

## Trust marks

16/10/2026, `personal/trust` (Sol's request): a player the hub neither verifies nor made
staff wears the hub's trust where the verified tick goes: an empty ring tick (new), a
green disc tick (trusted: 30 days and 10 hours of play since registering or the last
chat ban) or a red disc cross (a chat ban holds them)
([hub-chat.md](hub-chat.md#trust-marks)). On SJK and game chat lines, the docks, the
chat page, sender cards, nameplates, the scoreboard emblem's foot, the player card's
word (NEW, TRUSTED, CHAT BANNED), the profile card's picture and the Profile page's
status line. Ban 24 h is not offered on a trusted sender to a player who is not staff.
Built against the hub's `trust` field (Sol-Vulpes/SJK-hub branch `trust`, on top of
`chat-ban`). Checked by unit tests (`trust_badge`, the chat line's marks, `offered` on a
trusted sender). Not tried in the game or against a running hub.
## JoF EJK client lists

16/10/2026, `feat/clientlist-commands`: `clientlist` lists client numbers, teams,
names and bots, with the existing `showplayerid` retained as an alias;
`clientlistinfo` adds JoF EJK's human-player model, skin, gender, saber, colour,
team-model group and available body-RGB details
([client.md](client.md#chat-player-actions)). Ported against JoF EJK
`cg_consolecmds.c` revision `bd5e202dfedd49500f1b7aadd655619bac152167`; SJK prints the real `c3`/`c4`
values where that revision's RGB format call omits its arguments. On Linux
7.3.0-rc5 x86_64 with Rust 1.99.1, its four focused unit tests, `cargo fmt --all
--check`, the locked workspace build and test suite, all-targets Clippy, and the
optimized viewer/dedicated build pass (the repository's existing warnings remain).
Not yet tried against a running game server. SJK has no copied
`clientInfo_t::deferred` model-load flag, so the EJK-only `(deferred)` diagnostic is
not shown.
## Menu and chat text sizes

16/10/2026, `feat/chat-text-size-setting` based on review revision `1d2922c3`
(Sol's request): the
existing `cg_chatBoxFontSize` is now **Chat text size** in the classic+ HUD page,
SJK Advanced > Chat and SJK Simple > HUD, with its full supported 0.25–3 range.
**Menu text size** (`ui_textScale`) now reaches 1.4 instead of 1.2 (the pull request asked for 1.5; at 1.5 the home page's key hints were cut, so it was capped at 1.4 at the merge). Classic+
settings grow their rows and show fewer through their existing scrolling; the
long SJK Collection tab row closes its gaps, and the dual-saber search leaves its
heading room at the largest size ([client.md](client.md#text-size-and-spacing)).
Simple > HUD keeps Team logos beside the new Chat text size (Sol's choice at the
merge, 17/10/2026), so its row limit is now thirteen.
Group and layout tests pin those placements. On x86_64 Linux with rustc 1.99.1,
`cargo fmt --all --check`, `cargo build --locked --workspace`,
`cargo test --locked --workspace` and
`cargo clippy --locked --workspace --all-targets` passed (the repository's
existing warnings remain). The optimized viewer and dedicated-server build also
passed. The review follow-up removed a 1.5 cap that had made chat beside the SJK
scoreboard ignore the upper half of the advertised 0.25-3 range; its focused
regression test passed.
GPU/world-shot tests requiring `JKA_GAME_DATA` were
ignored by the suite; the settings and their persistence have not yet been
checked interactively in the game.

## Chat bans: Ban 24 h

16/10/2026, `personal/chat-ban` (Sol's request): a verified or staff player bans an
unverified one from SJK chat for 24 hours from the sender card or the SJK chat page's
chosen message, armed by a first click and sent by a second within 4 seconds; staff lift
bans; a banned player's refused message says until when, in local time
([hub-chat.md](hub-chat.md#chat-bans)). Built against the hub's `POST /v1/chat/ban`,
`chat_banned` and `/v1/staff/chat-unban` (Sol-Vulpes/SJK-hub branch `chat-ban`, built at
the same time). Checked by unit tests (`ban`, `chat_ban`, the worker's
`chat_bans_go_only_from_a_verified_or_staff_key`, the refusal's `until`, the card and
the page offering it only to a trusted player on an unverified sender). Not tried in the
game or against a running hub.

## Game chat lines look like SJK chat lines; names the server cut

16/10/2026, `personal/chat-marks` (Sol's request): a game chat line with a known sender
flows as an SJK chat line does, with SJK's emblem for a sender the hub knows in that
slot, the clan emblem, the verified tick and the channel, the message in its own
colours instead of the SJK chat's gold; JA+ lines, which name no sender, are read from
the name that starts them. An SJK player claims their slot with the name as typed when
the server only dropped characters of it (an emoji), and other SJK players' chat shows
that name ([hub-chat.md](hub-chat.md#game-chat-lines-look-the-same)). Checked by unit
tests (`chat::marks`, `chat::view::sjk_line`, `chat::reply`) and an off-screen render
of the chat box over duel6 at 1080p and 4K (`game_lines_with_marks`). Not tried in the
game: no claim with a restored name has reached the hub yet.

## Alpha notice at the first start

16/10/2026, `personal/alpha-notice` (Sol's request): the first start shows an alpha
notice card ([sjk-ui.md](sjk-ui.md#alpha-notice)): alpha build of a free and open-source
project, bugs expected, report on the SJK Discord (a permanent invite to #welcome,
discord.gg/dGBdMygvJw, made by the bot) or with Report a bug, no warranty, nobody owed
anything. Continue needs the "I understand" tick; Quit leaves. `ui_alphaNotice` keeps
the accepted version. Checked by the card's unit tests and an off-screen render over
duel6 before and after the tick (`alpha_notice_card_shot`). Not tried in the game.

## Graphics presets: Vanilla style, SJK (recommended), Experimental

16/10/2026, `personal/graphics-presets` (Sol's request): the Renderer row (Vanilla or
SJK (alpha)) and SJK quality's four levels are one Graphics preset dropdown of three
presets ([client.md](client.md#graphics-quality)): Vanilla style (the former Vanilla),
SJK (recommended) (Sol's config.cfg of that day: HDR, bloom, glow, FXAA, ambient
occlusion, model lighting and emission maps on; sun, shadows, reflections, material
maps, light shafts and weather extras off), the default of a new profile and of
Backspace, and Experimental (the cvars' defaults, the former High). Rows of a renderer
setting read the preset in use as their default. Checked by the `graphics_quality`,
settings and reload card unit tests and an off-screen render of Simple > Graphics with
the list open (`sjk_settings_snapshot`). Not tried in the game; no preset's frame rate
measured.

## JoF content: JoF's repository, opt in

16/10/2026, `personal/jof-assets` (Sol's request, after Kypdurron pointed Sol to JoF's
launcher): `jof.downloads` and Settings > Mods > JoF content download JoF's maps,
skins, hilts and packs from JoF's repository into SJK's own JoF folder, checked by MD5,
and load them from the next map while `jof.content` is on; off by default
([jof-content.md](jof-content.md)). Checked by the `sjk-jof-content` unit tests (a
local HTTP server for downloads), live against both of JoF's hosts (the manifest and a
file, ignored test `live`), against Sol's `GameData` (the launcher's files recognised)
and by an off-screen render of the card. Not tried in the game: no full download of a
group, no JoF map loaded from the folder on a JoF server yet.

## Synced emotes: one song, sample-aligned

16/10/2026, `personal/emote-sync-audio` (Sol's report: the music of synced emotes
sometimes sounded weird): each follower's copy started from the clock, a few
milliseconds off the leader's, and the copies added up, an echo and a louder song. A
follower's music now starts at the leader's voice's own sample (`Mixer::play_aligned`)
and the mixer plays a group's identical voices once, each ear as loud as the loudest
([hub-chat.md](hub-chat.md#emotes-in-the-client)). Checked by `sjk-audio` tests
(`sync_mix_tests`) and the emote tests. Not heard in the game.

## Synced emotes: the follower's music plays too

15/10/2026, `personal/emote-sync-sound` (Sol's report: a synced emote played its animation
without sound): a follower now starts its own music where the leader's is, in step,
instead of staying silent behind the leader's. Checked by the emote unit tests. Not tried
in the game.

## Synced emotes

15/10/2026, `personal/emote-sync` (Sol's request) with hub branch `emote-sync`:
`sjkemote sync [player]` and the quick wheel's Sync emote join another player's emote in
step. The hub relays the leader's slot with the emote (`sync` in `POST /v1/emote`), and
every client starts the follower at the leader's start as it saw it, so they are in phase
on each screen whatever the delays; the pair shares the leader's music, the follower's
picking up part way through (the mixer's new `play_from`) when the leader stops first
([hub-chat.md](hub-chat.md#emotes-in-the-client)). Against a hub without the change the
client resends the emote without `sync`. Checked by unit tests (client, identity, hub)
and the identity crate's HTTP test against a local hub of the branch. Not tried in the
game; the hub is not deployed.

## Emote sounds survive a full mixer

15/10/2026, `personal/emote-sound-cap` (Sol's report: emote sounds sometimes stopped
with many sounds playing): the mixer had retail's 32 voices and, when full, replaced
the oldest one, which a long emote track always was. It now has 128 voices, and a full
table gives up voices on the emote channel last
([rig-models.md](rig-models.md)). Checked by `sjk-audio` unit tests (the first fails
without the protection). Not tried in the game.

## Debug collectibles: staff try everything without the hub

15/10/2026, `personal/debug-collectibles` (Sol's request): `debug_collectibles 1` shows a
staff key every shader, medal, achievement and card as held, on that PC only, so the
team can try them without granting and taking them back at the hub
([unlockables.md](unlockables.md#settings-and-commands)). The look sent to the hub and
the pop-ups keep reading the real profile. Checked by unit tests and the world shot
`duel6_sjk_collection_debug` (Medals 4/4, Achievements 21/21, Shaders 51/51 with the
mode's line, the card in its three finishes). Not tried in the game.

## Player card: sees your model

15/10/2026, `personal/model-check` (Sol's request) with hub branch `model-check`: SJK
clients send the hub the player model folders their game has (`cl_shareModels`, default
on) and another SJK player's card says "Sees your model" or, in amber, "Can't see your
model" ([identity.md](identity.md#installed-models)). Checked by unit tests (listing
folders with a mesh or rig manifest, the model value's folder, the identity worker
sending a list once and again when it changes and storing answers), the hub's API test
(registration, bad lists, answers by slot, unknown list, replacement, empty list
forgetting, bad slot and model) and an off-screen render of the pinned card with the
line. Not yet seen in the game; the hub branch is not deployed, so until it is the hub
refuses the list (once an hour) and the card shows no line.
## Duels: shown as JoF EJK shows them, with paired duel badges

15/10/2026, `feat/duel-visibility` (Creyon, after JoF players' reports): players outside a
duel could hardly tell who was duelling, as SJK drew stock's dim blue sight bubble on a
fully lit body. JoF shows a duellist as a see-through blue figure (EternalJK's
`gfx/effects/duelShader`, `cg_stylePlayer`'s `JAPRO_STYLE_VFXDUELERS`), now SJK's default
with a lighter blue so it stays see-through ([duel_ghost.rs](../crates/sjk-viewer/src/duel_ghost.rs),
`cg_duelGhost`, Settings > Quick > Force shells > Duel figures (JoF)). Off, a duellist is
dark under the bubble at JoF EJK's 100 100 255. While you duel, everyone else is drawn at a
fifth of their colour and you and your opponent look as you are (EternalJK's default)
([duel_look.rs](../crates/sjk-client/src/duel_look.rs)). Nameplates put a crossed-sabers
duel badge ([duel_badge.rs](../crates/sjk-viewer/src/ui_renderer/duel_badge.rs)) before a
duellist's name (`cg_nameplateDuel` turns it off), the two of one duel in the same colour
([duel_pairs.rs](../crates/sjk-viewer/src/hud/duel_pairs.rs): the server's duel-start
broadcast, stock's or jaPRO/TaystJK's, or your own duel; an unknown opponent gets a neutral
grey badge, never a guess). SJK-made glows (a repeated bubble, a chrome shell, a blue wash)
were tried in playtest and dropped. Played on JoF (Windows 11): the figures, the badges and
their colours for duels started in view, no glow on the pair.
## Weapon selection across view transitions

16/10/2026, `fix/weapon-selection-view-transitions` based on `7026c735`:
accepted snapshots reset the retained weapon choice/display time when the
viewed client or follow/spectator mode changes, or server time rewinds. This
adopts JoF EJK #39's multiplayer `CG_TransitionPlayerState` intent before
prediction and command submission, including same-client native transitions.
The auto-switch event baseline is rebased without discarding the current map's
item catalogue; old predicted acknowledgements cannot suppress the new player's
events. Ordinary snapshots preserve pending weapon input. Map/session
presentation resets clear selection/timer state. Live and demo snapshot paths
share the guard; both menu/HUD styles consume the same selection.

Verification on Windows, Rust 1.99.0: formatting, locked workspace build/tests
and Clippy with all targets passed. Five new tests cover target changes,
returning to the own saber, same-client follow/spectator transitions, pending
input across ordinary/duplicate snapshots and rewind at 8/7/4/3 ms steps, event
baseline rebasing, and rejection of another client's queued no-ammo event.
The original icon bug has not been reproduced in-game; live spectator-to-play
and GPU icon appearance remain unverified. No gameplay simulation or protocol
codecs changed.
## Native NPC definition filename capacity

16/10/2026, `fix/native-npc-definition-list` based on `7026c735`: native NPC
loading now reads every file listed by the VFS, rather than losing filenames
beyond the stock 2,048-byte list before reaching its unbounded text loader.
The stock profile still receives its original filename prefix and text-size
bound; both preserve VFS order. The regression fixture reproduced the loss at
`npc_070` before the fix and loads all 100 synthetic definitions after it. A
second fixture verifies the stock text-size error fallback and native loading
of the same file. Formatting, locked workspace build/tests and Clippy with all
targets passed on Windows, Rust 1.99.0, with existing warnings. Fixtures use no
retail data; GPU/data-dependent workspace tests remain ignored.
Live map loading and NPC spawning with a large community pack remain unverified.
This changes definition availability, not gameplay timing, live NPC capacity or
the wire format. See [NPC definition packs](server.md#npc-definition-packs).
## Distinct native colours for all seven saber styles

16/10/2026, `fix/native-saber-style-colours` based on `7026c735`: Desann now
uses violet and Tavion cyan in the native HUD's stance labels and radial meter,
and in nameplates' saber halos. The existing five style colours and unknown-style
fallback remain. The shared palette applies in either menu style; custom menu
HUD pictures retain their existing seven-style support. This adopts the missing
native colour distinction from [EJK #26](https://github.com/JediofFreedom/JoF_EJK/pull/26).
The existing radial draw-command test now checks all seven styles, pairwise
distinct colours, separation from the unknown fallback, full-width strokes and
matching label colours. Live gameplay and GPU appearance remain unverified.
Formatting, locked workspace build/tests and Clippy with all targets passed on
Windows, Rust 1.99.0, with existing warnings. The GPU snapshot check was not run:
its required `JKA_GAME_DATA` asset path is not configured in this environment.
See [radial HUD](rendering.md#game-data-hud) and [nameplates](client.md#nameplates).
## Team powers in outgoing Force profiles

16/10/2026, `fix/team-force-profiles`, based on `7026c735`: Team Heal and Team
Energize remain in the client's sent profile in non-team modes, within the
server's rank/point ceiling and the chosen side. This lets a server supporting
them in FFA read the selected levels; authoritative stock legalization still
removes them. Both Force menus mark their availability as "Server decides"
outside team games ([client](client.md#force-profile-on-a-server)). Reference:
OpenJK `260c59c` `codemp/game/bg_misc.c:513-517` strips team powers without
invalidating the profile; JoF EJK #10 keeps the allowed side's team power.
On Windows, Rust 1.99.0, nine Force-profile tests and 80 player-menu tests passed,
including side/point limits, retained FFA selections, stock removal without an
illegal-profile result, server disable rules, and both Force-menu layouts.
Formatting, workspace build/tests and all-target clippy passed with existing
warnings, using a target on a separate volume with incremental compilation off.
Live FFA server grants and GPU/UI verification have not been performed.
## Optional private-duel nameplate filter

16/10/2026, `feat/hide-dueller-nameplates` based on `7026c735`: the archived,
default-off `cg_nameplateHideDuelers` option hides whole plates for server-marked
private-duel participants, including the local player's optional third-person
plate. Both settings interfaces expose it under Nameplates. The reference is
[EJK #12](https://github.com/JediofFreedom/JoF_EJK/pull/12), whose player-label
filter uses `entityState_t::bolt1`; the local player uses `duelInProgress`.
A synthetic-snapshot test on Windows exercises the production collection pass:
default visibility, immediate suppression and restoration, duel completion,
Power Duel side independence and NPC field reuse. Formatting, locked workspace
build/tests and Clippy with all targets passed on Windows with Rust 1.99.0
(existing warnings; GPU/data-dependent tests remain ignored). Missing player entities have no
duel marker, so sealed pilots' plates remain visible. Live server duel transitions
and GPU presentation have not been exercised. See [Nameplates](client.md#nameplates).
## Game chat substitutions

16/10/2026, `feat/chat-substitutions`, based on `7026c735`: the game-chat send
path expands EJK's eleven health, shield, Force, weapon, ammo, ping, speed and
local-time tokens for composer messages, console/binds, team chat, tells and
replies in both menu styles ([client](client.md#chat-substitutions)). Expansion
uses the existing serializer and byte budget, before optional chat colouring.
Five focused unit tests cover all tokens, horizontal speed/shared ammo pools,
clock formats, literal text, quoting and the post-expansion byte limit. Formatting,
workspace build/tests and clippy passed on Windows with Rust 1.99.0, using a fresh
target on another volume and `CARGO_INCREMENTAL=0` after the original volume ran
out of space during debug-symbol/cache writes. Existing warnings and ignored
GPU/asset tests remain. Live server delivery and visual checks have not been
performed. This changes no movement rules or wire codec.
## Inventory list and selector visibility

16/10/2026, `fix/inventory-list-default` based on `7026c735`: the archived
`cg_drawInventory` now defaults to 0 for the permanent holdable list, matching
JoF EJK #40. The transient inventory selector is independent and remains visible
when cycling. Both settings interfaces expose **Inventory list** in their HUD
category. Existing saved preferences remain; trip-mine weapons were already
absent from the holdable list. Input, item use and gameplay are unchanged.

Verification on Windows, Rust 1.99.0: formatting, locked workspace build/tests
and Clippy with all targets passed. The retained-HUD regression checks the
registered default and both list values across Game, Classic and Radial layouts:
the permanent labels follow the toggle while the inventory selector remains.
Shared settings layout/category checks passed. Live/GPU appearance remains
unverified; `JKA_GAME_DATA` is not configured.
## Grip victim effects

16/10/2026, `feat/grip-victim-effects` based on `7026c735`: red neck puffs
follow the `BOTH_CHOKE3` leg-animation marker, using JoF EJK #33's puff
parameters and #90's later victim placement and camera guard. The archived
`cg_gripVictimFX` setting defaults on in classic Game options and SJK Advanced
Gameplay. Local prediction and remote snapshot markers are supported; the
viewed player's puffs are hidden in first person or zoom, while other players
and detached cameras retain them. Cached neck/head positions reuse the existing
pose, with missing joints suppressing the effect. Emission uses the existing
8 ms effect cadence. Gameplay and protocol encoding are unchanged.

Verification on Windows with Rust 1.99.0: `cargo fmt --all --check`, locked
workspace build/tests, and Clippy with all targets passed. Four focused tests
cover local/remote animation markers and prediction, camera/setting guards,
scaled and rotated joint placement, missing joints, and reference puff colour,
lifetime, motion and density at 8/7/4/3 ms frame steps. These are synthetic
checks, not live visual parity evidence.

Live Grip play and actual GPU appearance remain unverified. `JKA_GAME_DATA`
is not configured; no release frame-time measurements have been made.
## Bundled JoF Force presets

16/10/2026, `feat/jof-force-presets` based on `7026c735`: the client bundles
JoF EJK #23's 22 community Force templates (13 light, 9 dark) unchanged, with
upstream attribution. They are content mounted below installed packs, selectable
through the normal Force draft; loading still applies rank and server rules and
waits for Apply. The classic list groups retail templates first under Jedi Academy,
then Community. The SJK Force page now has selectors for the same two groups.
Retail membership comes from the `base/assets0.pk3` through `assets3.pk3` view,
and the catalogue exposes its retail count. Saved templates still override
same-named packaged templates. All 22 files match upstream revision
`9750845d194ced3d4ef86fcb80ea482339f629d1` byte for byte.
Synthetic-content tests verify side assignment, archive-based retail grouping,
installed and saved overrides, and SJK pointer/keyboard selection changing only
the draft. No retail assets or player-saved files are used by these fixtures.
Formatting, locked workspace build/tests and Clippy with all targets passed on
Windows with Rust 1.99.0 (existing warnings; GPU/data-dependent tests ignored).
Live server use and GPU presentation remain unverified. See
[the client](client.md#menu-style) and [SJK UI](sjk-ui.md#character).
## JA+ flamethrower client presentation

16/10/2026, `feat/japlus-flamethrower-presentation` based on `7026c735`: a named
compatibility table adopts JoF EJK #31's firing/hit flags, flame graphs, sample
and hold pose. The archived `cg_flameThrower` override (Auto/Off/JA+) is exposed
in both settings interfaces. Local prediction and remote actors select flames
instead of Lightning only while firing; Drain and vehicles retain their paths.
Flame streams use the existing 8 ms effect cadence; impact bursts and sound
repeats use snapshot server time (100 ms and 2.9 s intervals). The upper-body
hold blends over 100 ms and leaves leg animation and gameplay state unchanged.
Sound stops on firing end, mute, disappearance or disconnect and resets across
map changes and time rewinds. Installed flame graphs/audio are required and are
not bundled. Live JA+ play, GPU appearance and audible playback remain unverified;
`JKA_GAME_DATA` is not configured for the asset/GPU checks. Review fixes
(17/10/2026): the contract accepts JA+ itself only (not abyssmod or UA Galaxy),
keeps Lightning when the flame graph is not installed, and the sound re-detects
when the server info changes, as the effects do. See
[client behaviour](client.md#ja-flamethrower-presentation).

Verification on Windows with Rust 1.99.0: workspace formatting, locked build,
locked tests and Clippy (all targets) passed. Focused tests pin local/remote
flag selection, the reference pose ordinal, capability overrides, hit/sound
cadence, stop/restart and rewind handling, and upper-body blending at integer
8/7/4/3 ms steps. These synthetic checks do not establish live visual or audio
parity, or release-build frame-time performance.
## Holstered saber hilts

16/10/2026, `feat/holstered-saber-hilts`, based on `7026c735`: implemented
JoF EJK #42/#81/#95/#267/#300's holstered hilt family with shared native meshes,
independent hip attachments, conditional staff back placement, frame-latched
handoff and registered hilt/skin sound synchronization. Both settings styles
expose the controls. Auto support detects the named JA+ contract; the existing
`cp_pluginDisable 1536` default is preserved. See
[client](client.md#holstered-saber-hilts) and
[rendering](rendering.md#holstered-saber-hilts).

Windows x86_64, Rust 1.99.0: workspace fmt/build/test/clippy checks passed.
Fourteen focused tests cover crowded dual-hilt capacity, cached bolt lookup, missing attachments, placement
axes, visibility, archived settings, registered sound layers, bounded ignition
fallback and forward/reverse handoff at 8/7/4/3 ms steps. Source comparison used
EJK `codemp/cgame/cg_players.c` at `f21eee8`, plus the family's safety/lifecycle
patches. A release CPU measurement on Ryzen 7 5800X3D averaged about 11 us per
64-actor frame over 20,000 synthetic frames for attachment placement alone;
this excludes pose evaluation, visibility/audio work, submission and GPU cost.
No installed game/JA+ assets, GPU scene comparison, audible timing check or
populated-server/cache-lifetime stress was exercised. The measurement and
tests do not establish gameplay parity or 500+ FPS performance. Review fixes
(17/10/2026): no per-actor work while the feature is off, the server contract
cached per configstring 0 change, no bolt evaluation for a saber in hand,
deferred staff sounds played rather than dropped on a slot release, and hilt
groups sized for 64 carriers (about 16 KB each instead of 256 KB). The staff
test (`saber_type` staff or more than one blade) is kept: it is EJK's own
`CG_StaffHolsteredOnBack` rule (`cg_players.c:10812`).
## Classic scoreboard JoF behavior

16/10/2026, `fix/classic-scoreboard-jof-parity` based on `7026c735`, compares
classic against JoF EJK `codemp/cgame` at `bd5e202d` (the reference already
pinned in the client wiki), including #35's crowded-board behavior. Classic
now opens automatically on death and follows JoF's warmup gate and selective
HUD suppression. Status and selection rows hide for the playing client while
scores are held or health is zero; telemetry and votes remain, and spectators
retain crosshairs. The SJK board retains its existing opening/HUD policy.

Row placement distinguishes connected clients from received scores, uses the
reference 19/32-row budgets, advances empty sections, preserves team-leading
spacing, and appends the local row at the reference cursor. A full team board
can retain its final spectator instead of losing it to the former bottom clip.
Classic uses white ping text, omits alternating row stripes, and restores the
reference flag column and header-label position. Native identity/clan marks
and renderer-native header art remain; pixel-identical rendering is unverified.

Verification on Windows, Rust 1.99.0: formatting, locked workspace build/tests
and Clippy with all targets passed. Synthetic scoreboard checks pin 20/21/32
clients, partial scores, empty sections, team spacing, final spectators, local
rows, selective HUD visibility and death/respawn fading at 8/7/4/3 ms steps.
Live JoF comparison and updated GPU screenshots remain unverified;
`JKA_GAME_DATA` is not configured. No release frame-time measurements were made.
No wire codecs or gameplay rules changed.

## Emotes on stock models: limbs out of wider bodies

15/10/2026, `feat/emote-body-fit` (Creyon, on `feat/emotes-on-classic-models`): a dance made
on a slim rig played on a wide stock model no longer puts its hands and elbows inside the
wider body: both bodies are measured from their meshes (ellipsoids round the trunk and
thighs, limb thickness), and each limb keeps the rig's clearance up to 2 units, never inside,
a touched body still touched ([rig-models.md](rig-models.md#body-fit-limbs-out-of-wider-bodies)).
Checked by unit tests (measuring a body without its arms, clearance, the push, a two-bone
fit keeping lengths) and the ignored measurement on four community models: summed limb
depth into the body down 80 to 95% on the three wide ones, unchanged on a thin robot. Not
yet seen in the game.

## Emotes on stock models

15/10/2026, `feat/emotes-on-classic-models` (Creyon, at Sol's request): a stock GLM model
(Kyle, the retail characters, community models on `_humanoid`) plays the emotes made on rig
models, on its own bones: each bone the source rig's bone map names takes its joint's turn
from the aligned bind pose, the pelvis follows the clip's on the driver's ground
([rig-models.md](rig-models.md#emotes-on-stock-models)). Checked by three unit tests (bind
pose gives the base pose, a turned head turns the cranium about its own base, a fade blends
half way, a crouching pelvis carries its children, bone order) and the workspace checks on
Linux. Creyon's first game test (Windows 11, local build) showed broken limbs on some
models: the fingers and hang tags hang on the forearm in JKA's skeleton and stayed with it
while the dancing hand turned. Fixed the same day (positions carried by the rig's own
hierarchy, unmapped bones moving with the bone they are most rigid with); a retail
round-trip check now brings every skinned bone back within 3.4 units (fingers were 5.2
off). The fix is not yet seen in the game.

## Model tags no longer drawn (purple arrows)

15/10/2026, `fix/hide-model-tag-surfaces` (Creyon, after Nib's report): community player
models showed small magenta arrows on the chest, belt and face (the Nautolan's "face
mask"). They were the GLM's tag surfaces (`*lchest_l`, `*hip_l`, `*head_front`...), flagged
as bolts and shaded `[nomaterial]`, which the client drew as a missing image; rd-vanilla
draws a surface only while all its flags are clear. Tags now have no shader in the skinned
surfaces, so no draw path shows them; they stay in the list, as bolts and surface
indices count them. Checked by a unit test, by reading the surface tables of the two
reported models (Nib, `jedi_nautolan`: their tags are `[nomaterial]` and sit where the
arrows were) and the workspace checks; not yet seen in the game.

## Nameplate bars only for players who share

15/10/2026, `nameplate-share` (hub `nameplate-share`), Sol's rule: other players' health,
shield and Force bars show only for SJK players who share them (`cg_nameplateShare`,
off), through the hub (`POST /v1/vitals`, twice a second, in memory 5 s). Estimated bars
on everyone (`cg_nameplateEstimate`, off, replacing `cg_nameplatePredict`, which was on)
share the player's own and say so on their player card ("Sees your health, shield and
Force"). A sharer's bars are the estimates re-based on each shared value at its server
time, so a value a second old still corrects them (`estimate::History`). Teammates
(`tinfo`) and NPCs keep their server-sent bars. Checked by unit tests (client, hub,
estimates) and the player card's snapshot; not tried in the game, and the hub's load at
twice a second per player is not measured. Hub not deployed yet.

## The Blade Lab's last three: solid blades

15/10/2026, `blade-lab-3` (hub `blade-lab-3`): Comic and Quicksilver (Legendary) and
Obsidian Edge (Mythical), all droppable, a tier above Creyon's ranking, completing his
Blade Lab page (sixteen shaders). A seventh set of blade-skin sections, `ink` and
`chrome` (`blade_skin_solids.rs`, `.bladeextra5`), drawn in a new alpha-blended pass
after the blades (`fragment_solid`), so a blade can be dark, outlined or a mirror; the
Comic's halftone is printed in the glow. Checked by unit tests and the world shots
`duel6_lab_blades_solids` and `duel6_lab_swatches`; tuned there (the obsidian body made
fully opaque, as 96 % let the bright glow through as grey seams, and narrower so its
crackle shows). Not heard, not tried in the game; the Comic's words and speed lines
and a true reflection of the map are not built.

## The Blade Lab's next seven shaders

15/10/2026, `blade-lab-2` (hub `blade-lab-2`): Sketch (Rare), Hourglass and Fiber Optic
(Legendary), Serpent, Monarch, Phoenix and Quantum (Mythical), all droppable, a tier above
Creyon's ranking; a sixth set of blade-skin sections (figures: sketch, hourglass,
serpent, strands, butterflies, wings, copies; `blade_skin_figures.rs`, `saber.wgsl`,
`.bladeextra4`), their swatches, and synthesized sounds (one builder, a texture each).
Checked by unit tests and the world shots `duel6_lab_blades_figures` and
`duel6_lab_swatches` (now also the held Hourglass, Fiber Optic, Monarch, Quantum and
Sketch); tuned there (the Hourglass's and Fiber Optic's core and glow dimmed so the sand
and fibres show, the copies kept inside the glow's quad, which had cut them off). Not
heard, not tried in the game. Swing-driven parts and the Phoenix's ignition from ash are
not built (no blade motion or ignition time in the shader). Comic, Obsidian Edge and
Quicksilver are left: they need a dark or mirror pass.

## The Blade Lab's first six shaders

15/10/2026, `blade-lab` (hub `blade-lab`): six shaders after Creyon's Blade Lab page,
each a tier above his ranking (Sol: his were too low): Neon and Peppermint (Uncommon),
Soap Bubble and Lava Lamp (Rare), Reactor and Bleeding Kyber (Mythical), all droppable,
so Mythical holocrons can now bring a Mythical shader. They are drawn with a fifth set of
blade-skin sections (bands, segments, film, blobs, front: `blade_skin_lab.rs`,
`saber.wgsl`) read from a third additions file (`.bladeextra3`) that older clients never
open, and the Collection's swatches draw them too. Sounds are synthesized
(`blade_skin_sounds_third.py`). Checked by unit tests (the files, the lanes, the ranges,
the front's cycle) and the world shots `duel6_lab_blades` and `duel6_lab_swatches` (the
Collection's swatches and the held blade) with the hub's pack built
locally; tuned there (the wax blobs' falloff squared so they stay apart, the Reactor's
rings deep blue so they read on its white core). Not heard yet; not tried in the game.
The other ten Blade Lab ideas need new kinds of drawing (a solid or outlined blade,
strands, creatures, copies of the blade), not built.

## Saber smiths

14/10/2026, `saber-smith`: every shader names who gave its idea, "Saber smith: Creyon"
for the 21 plain ones after Creyon's Saber ideas page, "Saber smith: Sol" for the others: at
the right of the Shaders big card and on the hover card, and the Shaders search finds it
([unlockables.md](unlockables.md#saber-smiths)).

## Rig models: glTF players with their own skeleton

14/10/2026: a model folder with `sjk_rig.cfg` is drawn from a glTF (any skeleton,
fingers, face) and posed every frame from the stock `_humanoid` animations through a
bone map, the stock driver (Kyle) still doing bolts, traces and the server's animations
([rig-models.md](rig-models.md)). Checked by unit tests (glTF reading, manifest, pose
maths, CPU and GPU paths agreeing) and the world shot `rig_model_on_a_demo`: a
177-joint mannequin character on a demo player, drawn and skinned on the GPU, holding
the saber, on one GPU, the player idle. Sol tried it in the game the same day: drawn
and animated, but the far side showed through (glTF winding) and the saber missed the
hand (bolts on the driver's bones); both fixed (game winding at load, bolts and Force
points moved onto the rig's joints) and checked again in the world shot. Not yet
seen: dying, other players wearing one. The Blender add-on `sjk_rig_export` writes a
model folder; its export matched a hand export and rendered the same. Before this,
release-build measurement on the same demo (19 players): pose, skinning and upload
took 0.064 ms a frame, vertex skinning already on the GPU, so the rig path is about
looks, not speed. Emotes followed the same day ([emotes](rig-models.md#emotes-the-rigs-own-clips)):
the glTF's animations play on the rig's own skeleton through `sjkemote` and the
quick wheel, local by default; the add-on writes them and packs everything into one
`rig_<name>.pk3`. Checked by unit tests (animation reading, clip posing, fades,
bolts following a clip, the emote file's new fields, the wheel editor) and a world
shot of a 3.4 s, 84-bone dance on a demo player (feet on the floor, saber in the
hand). Sol's game test: it plays, twice too fast (a 60 fps Blender scene for a 30 fps
animation); the add-on then gained a frame rate, a sound per emote and "keep playing
while moving", emotes became visible to other SJK players by default (the hub relays
the id; each needs the same files), and `sjkemote stop` ends one. Sound playback,
moving emotes and two clients seeing each other's emote are not yet tried in the
game. Sol saw the feet sink: measured 2.6 to 3.2 units below Kyle's soles standing and
running; the feet are now planted on the driver's every frame (within a unit in the
measured poses), with a `lift` line (the add-on's Height adjustment) for the rest.
Then icons: a rig model with an `icon_default` picture is listed in the character grid,
and an emote's `icon` shows on the quick wheel; the add-on (1.4) takes both, a display
name per emote, a pack name shared by several models, and meshes on another armature
that follows this one (a tail). Checked by unit tests, a world shot of a second model
(312 joints and a tail) playing a looping emote, and a wheel shot with two emote
pictures; the character grid with a rig model is not yet seen. Emotes are no longer
tied to one model: an emote file names the model its clip comes from (`rig`) and every
rig model with the same joint names borrows it at load, turned onto its own skeleton
(unit tests; world shots of each test model playing the other's emote).

## Shaders: a filter for a long list

14/10/2026: the Collection's Shaders tab filters by tier (chips, T), by shaders owned (O)
and by a search (/): the list shown is the panel's own (`shader_list`), so keys, wheel,
clicks and the stage follow it. Unit tests and a world shot of the chips; typing in the
search field not tried in the game.

## Shaders: duplicates in the Collection

14/10/2026: a shader held more than once reads "Yours x2" in the Collection's Shaders tab
(rack, grid, big card) and ", 2 held" in its owned line and hover card. Unit tests only.
The search, tier filter and paging for a long list are still to build.

## The hub's odds, new shaders, the Saber tab's blade row

14/10/2026, `loot-odds` (hub `loot-wire`): the hub sends every odds table with
`GET /v1/holocrons` (`loot`) and the client's loot table, card odds, holocron lines, tier
odds and crate spin read them (`loot_odds::current`), its own copy only before the first
answer or when the hub's tables do not fit. A shader the hub drops that this client does
not know still shows at opening, by the hub's name ("A saber shader new to this version of
SJK"), the spin ending on a "?" tile. The Saber tab's Blade row is browsed with the wheel
and its ‹ › arrows a row at a time without wearing anything; a pick follows the chosen
blade again.

## Holocron opening: the crate spin

14/10/2026, step 4 of the shader catalogue plan: before each holocron's burst a strip of
what its tier can drop (the cards its odds allow and the droppable shaders, weighted by
their chances, by the hub's odds: `loot_odds.rs`, see below) runs past a gold
marker, clicking a tile at a time, and slows to a stop inside the real prize's tile; the
strip is seeded by the holocron's number. A shader prize has its own entrance (the burst,
its blade framed in its tier's colour, its words and how many are held). Enter skips the
spin, then the entrance. `debug_open <tier> shader` rehearses it offline; world shot
`duel6_sjk_crate_spin`, reviewed by eye on one GPU (the blades grey, as the hub's packs
were not loaded). Not tried in the game.

## Holocrons may drop shaders

14/10/2026, step 3 of the shader catalogue plan: the hub (branch `loot-kinds`, merged in its
`main`, not deployed) holds unlocks as counts and rolls an opening as a card or a shader by the
holocron's tier; the client reads the new answer (`Unlock::count`, `Opened::unlock`, `card` now
optional). No ceremony for a shader yet (a line in the pop-up says what was found); the crate
animation is next. Checked by unit tests only. See [unlockables.md](unlockables.md#holocron-shader-drops).

## Shaders: the Common tier and nine plain blades

14/10/2026, seasons: `editions.rs` (Alpha Edition) and an `edition` on every shader, medal and
achievement, shown on the Shaders and Medals big cards and every hover card; the hub already
stamps card copies ([unlockables.md](unlockables.md#editions)).

14/10/2026, more-shaders: twelve more plain blade skins (Common Slate, Amber, Lilac, Mint, Coral,
Ivory; Uncommon Glacier, Dusk, Ruby, Amethyst, Jade, Moonlight), 35 shaders in all (hub branch
`more-shaders`, same script and pack).

14/10/2026, step 2 of the shader catalogue plan: `unlockables::Rarity` gains Common (white,
SJK Symbols' empty octagon; no Common holocron exists), and nine plain blade skins join the
catalogue after Creyon's "Saber ideas" page: Tidal, Circuit, Aurora, Echo (Common), Nebula,
Cinder, Venom, Sakura, Firefly (Uncommon). Their files are written by the hub's
`scripts/common_skins.py` (hub branch `common-shaders`, with the catalogue entries, the rebuilt
`sjk_skins` pack and stock saber sounds allowed in the pack test); they only reach players once
that hub is deployed and the keys are granted. The Collection's shader tokens moved to 7000 and
the Saber tab's to 7300 (the old ranges held 19 skins), and the Staff page's unlockables list
has a pager. Checked by the unit tests; not seen in the game.

## Blade skins: 256 slots

14/10/2026: the skin array (`saber.wgsl` `skins`) is a read-only storage buffer of 256
`Skin` structs (253952 bytes), no longer a 16-skin uniform capped by 16 KiB; `MAX_SKINS` is 256
and the material slots follow. Step 1 of the shader catalogue plan (about 200 shaders, a Common
quality, holocron drops). Checked by the `saber` tests (101 pass, shader text pinned); not run
on a GPU or in the game. Still per-skin texture pairs (generated), loaded eagerly for catalogue
ids with a pack file; lazy loading and a texture array come when the catalogue grows.

# Status and priorities

## Chat: the euro sign and a Retry button

Branch `personal/euro-retry` (13/10/2026):

- The euro sign `€` is in the bio alphabet (`PUNCTUATION` in `bio.rs`), so the SJK chat,
  bios and notes take it; before, the check answered "a message keeps to letters, digits,
  common punctuation and colour codes". The hub has the same character on its branch
  `euro-sign` (`dae0aee`, not deployed: until it is, the hub refuses a message with `€`).
  Messages to game servers already sent `€` as the Windows-1252 byte 0x80.
- The SJK chat page keeps the last message it handed to the hub; when the hub did not
  take it ("Not sent: too many requests; slow down" and the like) a Retry button under
  Send sends it again. The hub's `Any` allowance is 30 requests a minute per address,
  shared by everyone behind one address, and `Chat` (messages, emotes, pings, looks)
  60 a minute.

## World notes: decals in fog, sky through a ceiling, security glass

Branch `personal/render-notes` (13/10/2026, based on `1b02a8c8`, Windows 11), three of
Sol's world notes:

- Fixed, the decal "like there is no fog" (`mp/ctf4`, `textures/decals/mp_imperial_symbol`,
  07/10/2026): decals that write no depth are drawn before the opaque world's fog pass,
  whose `Equal` draw on the wall then fogs the decal with it; their own fog pass, which
  met nothing, is gone ([rendering.md](rendering.md), world decals).
  World shots of the note with `r_drawfog 1` and `2`, before and after: the symbol stood
  out black in the fog, now it fades with its wall; without fog the decal is unchanged.
- Fixed, "something is flickering around here" (`jofmemorial`, 12/10/2026): the ceiling
  there lies two units under its sky brush, 2300 units up, and the sky faces, drawn
  depth-tested after the world from absolute coordinates, rounded in front of it in
  places; the clouds (`r_clouds 1`), drawn a fixed depth step nearer than the sky, covered
  it with white sky beyond about 2000 units, the edge of it moving with the camera. The
  sky faces, their mask, the sky portal's composite and the clouds now push their depth
  back by 1/32 of the way to the far plane
  ([rendering.md](rendering.md#sky-faces-depth)). World shots: the ceiling white with
  clouds before, dark after, over eight camera steps; open sky keeps its clouds. A unit
  test reproduces the rounding with the client's projection (17 of 2000 points lost
  unpushed, none pushed). The 3% push lets geometry just behind a sky face show through.
- Not changed, the security glass that "looks too white" (`vjun3`,
  `textures/common/glass_security_square`, 08/10/2026): SJK draws its two stages as
  rd-vanilla does (environment map added, then `sglass5`, black at 20% alpha: the glass
  is 0.8 x (scene behind + reflection)); world shots with and without the glass agree with
  that to a few levels. It reads white because with `r_dayNight 1` the hangar behind it,
  lit in the lightmap by its lamps, is almost black, so the fixed reflection is all that
  shows; with `r_dayNight 0` the hangar and its crates show through as tinted glass. The
  cause is the real-time lighting of that interior, not the glass. Sol's EJK also runs
  `r_gamma` 1.31, which lifts dark rooms against the reflection.

The world shots of notes pick notes by map, shader or time too (`SJK_NOTES=09:41:20`),
since removing done notes renumbers them, and `world_note_flicker` holds a note's view
still and steps it a few units at a time to find what flickers. Not tried in a game.
Decals flickering outside fog (below) stays open: eight 0.37-unit steps at the `mp/ctf4`
decal showed none.
## Relief method, Korriban glossy walls, obsidian shine

Branch `personal/relief-client` (13/10/2026), two of Sol's world notes. Korriban's
glossy walls (`mp/ffa2`: `korriban/door`, `entrance_top`, `os_outsidebased`,
`os_basic_pillarb`; "make this 3d", 07/10/2026) are drawn as two lightmap collapses, the
lit shine then the lit paint blended over it with its alpha, the same colour as the three
authored passes; the paint pass takes material maps and the generator selects it, not
polished ([rendering.md](rendering.md#material-maps)). The yellow glow Sol asked for is an
override (`emission=`). The obsidian wall JoF remaps over `vjun/newfloor_vjun` on
`mp/ffa1` ("way too reflective and disgusting"): SJK drew its shine at the authored
strength (in baked lighting it adds 0.055 in display values where retail adds 0.061), but
real-time lighting relit the paint six times darker than its bake while the unlit shine
kept its strength, 80% of the wall's brightness; an unlit environment shine now follows the
live light relative to the bake, at most its authored strength
([rendering.md](rendering.md#server-shader-remaps)). Remapped materials still compile without
material maps, so Sol's override for `yavin/obsidian_templebasic` (relief, roughness) does
not reach that wall. Verified: unit tests (the collapse and its refusals, the map
selection on both sides, the shine rule), world shots of notes 4 to 8 and 15 before and
after with a generated FFA1/FFA2 pack, and paired shots of the obsidian wall with and
without its shine stage (test shaders). Not tried in a game.

## Material notes: overrides found again, crates and concrete

Branch `personal/material-notes` (13/10/2026), from Sol's world notes about relief,
shine and lights. The pack installed on 12/10 (`GameData/SJK/generated`) had been built
without Sol's overrides: `sjk-materialgen` read them only next to the output pk3, and the
file had stayed in `%APPDATA%\jkr\generated` when the default output moved. The tool now
also reads the file in the client's `generated` folder, copies one left in the JKR-era
folder there, prints which file it read (or `NONE FOUND` with every place it looked, and
a warning), and records it in the manifest
([overrides_file.rs](../crates/sjk-materialgen/src/overrides_file.rs)). Generation 7 also
classes crates as painted metal (they were wood), keeps `concrete`/`cement` before part
words (`desert/concrete_wall_tall_vent` was polished metal), ignores keywords inside
look-alike words (`background`, `rocket`, `rustic`), and takes a per-texture
`strength=` override for the relief; the 12/10 and 13/10 notes became override lines
in Sol's file (`GameData/SJK/generated/sjk-materialgen-overrides.txt`). Findings that
need no generator change: every declared light fixture Sol noted (`mp/s_ylight_*`,
`s_vjun_*`, `s_tracklight4`, `s_deathconlight`) has had an emission map since the 12/10
pack, which was installed after those notes; lamps with an authored `glow` overlay
(`byss/lights`, `rooftop/groundlight`, `taspir/light_ceiling`) take none by design and
already light rooms in real-time lighting through fixture inference, so "stronger
light" there is renderer tuning, not done; Korriban's glossy walls do not collapse and
vertex-coloured flares light nothing (both renderer work, not done; see
[rendering.md](rendering.md#emission-maps)). Sol's config has `r_specularMapping 1`
and `r_normalMapStrength 4` (the maximum), so the shine notes are not a disabled-maps
artifact. The notes' world shots (`world_shot::notes::world_notes`) now mount the
player's generated packs as the game does (`SJK_NOTES_GENERATED` names another client
folder to compare): their throwaway profile had none, so until now they showed the
generation-5 pack left in `GameData/base` (07/10) instead of the installed one, and
material shots taken with them since 12/10 compared the wrong pack. Verified: unit
tests (the lookup order and copy, `strength=`, the class
words), dry runs listing the class changes over Sol's installed maps (90 textures: the
crates, concrete and the look-alike words), the regenerated pack's manifest, and world
shots of the notes before and after (`world_shot::notes::world_notes`, Sol's renderer
settings). Not tried in a game.

## World notes: Rancor tables, sand steps and footprints

Branch `personal/world-notes-fixes` (13/10/2026), two of Sol's world notes.

- Fixed, "tables are missing on this map" (`t2_rancor`, 08/10/2026, on a JoF server): the
  bar tables are `func_usable` clip brushes whose only drawn part is their `model2` md3,
  which SJK never drew. Movers now draw their `modelindex2` model as `CG_Mover` does, and
  the native server sends usables at their origin with their `model2`, solid, and fires
  only `PLAYER_USE` ones ([client.md](client.md#mover-models)). The note's companion
  "this should be a light" (`flares/solid_blue`, same place) is not addressed.
- Done, sand on `mp/siege_desert` (07/10/2026): steps on mud, dirt, sand and snow kick
  up dust and leave footprints as retail's `cg_footsteps 3` does, on untagged sand
  through the texture-name guess; jumps and forward landings step with both feet (SJK);
  three switches, `cg_footprints`, `cg_jumpFootsteps`, `cg_footstepGuess`, in Settings >
  Sound ([client.md](client.md#dust-and-footprints)). The 08/10 sand sounds already
  covered walking and the side, back and Force landings; plain jumps and the forward
  landing had no step in retail's table.

Verified: unit tests (the mover model rule, the server's usable state and use rule, the
jump and landing cues, prints by material, the mark list's lifetime and eviction, the
mirror transform), workspace build, tests and clippy (no new warnings), and two world
shots looked at: `rancor_tables` (the bar without and with the usables as the native
server sends them: the table under the note's light appears) and
`siege_desert_footprints` (ten steps on `siege2sand` leave faint alternating prints;
none with the guess off). Not run in a game: the prints and dust on players' feet, the
jump and landing timing, the native server's usable collision and the tables on a live
server are untested.

## Donations and the Supporter medal

Shelved on 14/10/2026 (Sol's request, branch `personal/shelve-donations`): the site's
section, the main page's card and its setting are hidden, and the Collection shows the
Supporter medal only to holders ([sjk.md](sjk.md#donations)).

14/10/2026: the medal has its final art (supporter-art, Sol's pick of a generated sheet; crimson ribbon).

Branch `personal/donate-goal` (13/10/2026), Sol's request, with the hub's
`donate-supporter`: donations on Ko-fi (https://ko-fi.com/soljk) with a live monthly
goal of 200 euros, on the site's Support SJK section and top left of the SJK UI's main
page (`ui_donationGoal`), and the Supporter medal (`supporter`, given once, no blade) in
the medal catalogue on both sides. Ko-fi's webhook feeds the hub, which sums payments
for `GET /v1/donations/goal` and gives the medal at once when the Ko-fi message holds a
registered key id; the rest waits for the SJK team ([sjk.md](sjk.md#donations)). The
medal's art is a drawn placeholder (a gold medallion with a heart on a violet ribbon)
until Sol picks real art. Verified: unit tests (client, identity crate and hub, the hub's
in-process webhook and goal tests), a local hub answering the webhook and the goal, the
site rendered in headless Chrome reading that hub, and the main page's world shot
(`duel6-menu`). Not tried in a game; the hub not deployed, so until it is, the site
shows its fallback numbers and the client's card has none.

## World notes: longer, with remaps and models

Branch `personal/worldnote-plus` (13/10/2026), Sol's request: longer notes, the shader
remaps in play, and models. A note takes 4000 characters (500 before; the hub's and the
client's rule together, its body up to 48 KiB), in a note card of ten lines. Every note
keeps the remaps active when it was taken (`remaps`, as `listRemaps` resolves them) and
the hub gets the noted shader's remap (`remapped`). A map entity drawing an MD3 is
selected when the view ray meets its triangles before the world
([world_notes_models.rs](../crates/sjk-viewer/src/world_notes_models.rs)); the note keeps
`model` (path, entity, placement, shaders with their remaps), Clear model drops it, and a
click on the scene round the card selects there. The hub takes the new fields on branch
`worldnote-plus` of the hub, not deployed: a hub from before refuses a note carrying
them, so the hub must be deployed before a client with this ships. The world shots of
notes make each note's recorded server and console remaps again. Verified: unit tests
(the limit on both sides, the hub client's body, a turned and scaled model picked on its
triangles, a note about a model, Clear model, a click's ray, the card's model row, keys
and pointer, the remap listing), the hub's API tests, and the GPU world shots
`duel6_note_model` (a duel6 rock selected through `worldnote`, outlined, its card,
then cleared) and `duel6_sjk_report` looked at. Not tried in a game, nor against the
deployed hub.

## Team outlines and logos

Branch `personal/team-outline` (13/10/2026), Sol's request: the player card's ring at
half its width (2 pixels of a 1080-line screen, 4 at 2160, from 4 and 8), the same ring
in team games around every enemy in red and every teammate in green (`cg_teamOutline`),
the team logos over allies' heads as JoF EJK floats them, readable and hidden behind
walls (`cg_drawFriend`, a small coloured square before), and no team overlay at the top
right (`cg_drawTeamOverlay` and its position settings unregistered). The outline mask
holds each body's group and the ring is drawn only in merged boxes around the outlined
players. See [client.md](client.md#team-outlines-and-logos). Verified: unit tests (the
groups' mask values against the ring's thresholds, scissor merging, the ring's width,
who is outlined in which game type, the logos' rules and sizes, the settings tests),
naga validation of the ring program, and the ignored world shots
`card_outline_on_a_demo` and `team_outlines_on_a_demo` (a CTF game staged over Sol's
FFA demo: green and red rings, the Imperial logo under the nameplate, the card's ring
winning on its player) looked at. Not tried in a game; the frame cost on a full server
is unmeasured.

## World pings

Branch `personal/world-ping` (13/10/2026), Sol's request: a modern game's ping that every
SJK player on the server sees in games without teams and only teammates in team games.
`+ping` (middle mouse where free) or `worldping` marks the point under the crosshair: a
gold ring through walls with the name and distance, a short sound, 7 seconds
([client.md](client.md#world-pings)). Transport: the hub, as emotes (`POST /v1/ping`, the
feed's `pings`; hub branch `world-ping`, not deployed), chosen over hidden server chat,
which other clients would show and server mods would rate-limit
([hub-chat.md](hub-chat.md#pings)). Team rule checked by each reader from its game's
configstrings; the hub cannot enforce it. Limits on sending (1 a second, 4 in 10 s), at the
hub (5 in 10 s) and on receipt (per slot); `cg_worldPings`, `cg_worldPingSound`, muted
players hidden. No minimap exists to show them on. Verified: unit tests (scopes and who
sees them, limits, the border placement, the draw list, the binds, the feed queue per
server, the service, the wire), the hub's unit and API tests, world shots
`world_shot::pings::duel6_world_pings` at 1080p and 4K looked at. Not tried with two
clients, a live hub or in a game; the sound not heard.

## Cards tilt towards the pointer, cleaner Holo

Branch `personal/card-tilt` (13/10/2026), Sol's report and request: Holo's bands showed
their square ends inside the card and over its edge, the sweep restarted while bands were
still on the card, and the card should move a little in 3D under the pointer. The light now
stays inside the rarity's edge, its strokes reach far off the card, and the sweep crosses,
then waits off the card. `sjk-ui` gained `PushTransform`/`PopTransform` (a projective
transform applied to shape vertices and text glyphs), and a Cards tab card under the
pointer tilts towards it, its light following the pointer, with a glare. See
[cards.md](cards.md#in-the-client). Verified: unit tests (the perspective, the sweep, the
strokes' ends) and the world shots of the tilted card looked at. Not tried in a game.

## Profile in game: the model shows the Force power

Branch `feat/force-power-preview` (12/10/2026, rebased 13/10/2026 onto `b2e8c750`,
Linux), Creyon's idea: opened from a game, the SJK UI's Force page described each power
in its box but the model beside it only stood. Now, while the pointer is on a power, the
model shows it, and stops when the pointer leaves: the game's gesture where multiplayer
has one (Drain the JoF server's `BOTH_FORCE_DRAIN_HOLD`), single player's poses the
models carry for Heal, the team powers, Rage, Protect and Absorb; the power's shell as
`CG_Player` tints it; its effect from the game's own files, in their stock look, in a
small silent effect world of the preview's own (Lightning's and Drain's beams, wide at
level 3, the push and grip puffs, JoF EJK's `force/rage2` and `force/heal2` chest
sprays, now preloaded, Mind Trick's confusion over the head, Speed's two see-through
copies, each drawn whole and faded, behind the runner as the picture shows him); a
standing jump as the game plays it (the body takes off, keeps the crouch in the air
and lands, arms and all), the model seen to rise and land back on its spot; the
saber thrown out spinning and caught; three looped Saber Offense swings flowing into one
another and Saber Defense's parries in the strong (red) style with a single saber, a
little slower than the game's. The preview is the same on every map: the model is lit as
a menu model with no world and the effects play in an empty world. The legs and the
torso play their own animations, blended between frames and from one animation into the
next from the pose on screen, each joint going straight from pose to pose and the bone turning
about it the short way at an even pace rather than its matrix blended (a swing folded
the model; turning the bone without moving the joint with it, Creyon found in game on
13/10/2026, took the joints off their parents between poses far apart, in every move:
arms stretched out and swung through the body) ([sjk-ui.md](sjk-ui.md)). Each saber
swing hands over to the next on reaching its last frame, taking over as the game does
(100 ms, even pace) or straight on where the two meet (a pause, then a slowed start and
a hurried middle between Offense's swings, Creyon, 13/10/2026); one swing at a time,
each back to the strong style's stance (`BOTH_SABERSLOW_STANCE`) for 0.3 s before the next start eases in
(waiting where the return ended looked frozen); takeovers between animations blended
bone by bone in the model, not in the parent's frame (the game's swing starts hold the
upper arm a half turn about itself from the stance's, and blended in the parent's frame
the saber swung a full turn round: the blade tip travelled about 315 units for a
13-unit move, measured on the game's `_humanoid.gla`; now every takeover the skills, the
throw and the parries make keeps the tip within 1.2 times the straight line); the
second swing the A+S one (`BOTH_A3_BL_TR`); every preview waiting 1.5 s once it has
played (gestures, their effects, the skills, the throw), Mind Trick's confusion, Rage's
spray and the shells of Protect, Absorb, Rage and the team powers on 2.5 s after the
gesture and then off, Sense's shell on
3 s and off 1.5 s, Speed fast with its copies for 3 s then slower without them, Push and
Pull straight back into the stance (their last frame kept stopped the hand), the blocks
from the strong style's stance, and Heal played whole (down, heal, up); the throwing arm held out until the saber is back; each parry is
raised over 0.25 s, kept 0.3 s and let go as slowly (the blocks were a snap); every preview
at the game's own speed (the skills at 0.9 felt slowed down); Lightning, Drain and Grip held 2.5 s
then let go and again, Rage's shell and spray on 1 s after its gesture; Speed's run at the game's 1.7 times; the throw out longer,
slowing to its far end; the sabers off in a jump's air (the game's jump holds the blade
through the head), lit again on landing; the jump on the whole body, arms and all, as
in the game, its landing as long as `BOTH_LAND1` lasts (held longer it stood crouched). Creyon's tests (12/10/2026) shaped it: hover only,
smooth movement, see-through and visible speed copies (their middles and jaws once went
missing, and straight behind they hid behind a model facing the camera), the jump (a
lifted jump crashed: it wrote the model's instance buffer, made without `COPY_DST`; the
camera moves instead, and no longer lets the model sink below its spot), a real throw,
the team shells' colours, Heal's effect, no dark rim round Sense's shell, Drain's pose,
Grip held rather than repeated, white push puffs on a map that recolours them, Absorb's
shell as the game draws it, the saber skills' red-style swings and parries. Verified:
tests (every power shows something in every style, the saber skills' animations and
returns, gestures, shells, tints, effects, beams widening at level 3, the jump's timing
and height, the rise seen never below the spot, moves in turn with their holds, the
throw out and back, the trail's copies beside the body, the empty effect world, the 120
ms sprays, a one-shot gesture replayed, a far turn blended the short way at an even
pace whichever sign its quaternion is stored with, a bone turned far about a joint off
the model's middle staying on it and keeping its length, a move handing over on its last
frame, a chained move not eased in, the game's even 100 ms takeover, a parry's keep and
eased takeovers, the blades off and on over a jump, the throw slowing out); the
workspace checks. In game on Windows 11 by Creyon (13/10/2026, the final build): every
power's preview, the saber skills, the jump and the throw looked at and approved. Then
branch `fix/force-preview-stance-landing` (14/10/2026, after PR #100's merge): Offense
rests in the strong style's stance between swings, as Defense does (it was the medium
idle), and the jump's landing lasts as long as `BOTH_LAND1` (held 0.5 s it stood
crouched); tried in game on Windows 11 by Creyon. Not covered: OpenJK side-by-side image comparison; the staff's and dual sabers' skills were
not singled out in those tests.

## SJK Symbols

Branch `personal/sjk-symbols` (13/10/2026), Sol's request: coloured special characters to
mark qualities, a font of SJK's own that keeps growing. Kanji and Aurebesh were set aside
(the setting, and Lucasfilm's), three families drafted, Sol picked the holocron one.
`scripts/sjk_symbols.py` draws the symbols from strokes and builds
`assets/fonts/SJKSymbols.ttf` (Private Use Area, U+E000 rarities, U+E010 finishes); the
client lends it to every face through the wide-glyph atlas; `symbols.rs` names them; they
mark cards, the loot table and the Shaders tab. See [sjk-ui.md](sjk-ui.md#symbols).
Verified: unit tests (every named symbol in the font, the ranks), `--check` of the font,
world shots of cards, the loot table and the Collection looked at. Not tried in a game.

## Card corners and the first card's words

Branch `personal/card-corners` (12/10/2026), Sol's report: on a Holo card the sheen ran
over the card's rounded corners. Clips were rectangles only; `sjk-ui` gained
`PushRoundedClip` (strokes cut to the rounded shape, each corner four chords of its arc,
everything else to the rectangle) and cards draw their finish under it, the washes with
the card's radius. Sol also renamed the first card "Toxiee and Sol on JoF Memorial"
(Toxiee is on the left) and its art reads "Sol (AI-assisted)": re-imported into the hub's
`sjk_cards` pack (hub branch `feat/card-001-words`). Verified: unit tests (the rounded
clip keeps a stroke off the corners), the world shots looked at, corners zoomed in.

## Chinese, Korean and emoji

Branch `personal/cjk-emoji` (13/10/2026), Sol's request after the Russian and Japanese
work, with Sol's picks: colour Twemoji, the common Chinese and Korean sets, and the
hub taking them in chat and bios. Implemented: Noto Sans SC (the 2,215 GB 2312 hanzi
Noto Sans JP lacks) and Noto Sans KR (Hangul) in the wide-glyph fallback; a Twemoji
17.0.3 pack (`scripts/twemoji_pack.py`), emoji sequences matched as one glyph in the
layout walk, and an RGBA emoji atlas at text binding 3
([rendering.md](rendering.md#emoji)); chat wrapping, dialog widths and the caret by
glyph; the shared bio and chat rules taking Korean, Chinese punctuation and emoji
(`sjk-identity` `emoji.rs`, generated, and the hub's copy). Verified: unit tests (the
sequence matcher, atlas decoding, metrics, caret steps, the rules on both sides) and
the GPU world shots `chat_in_russian_and_japanese` (now with Chinese, Korean and emoji
lines, a typed draft, at 1080p, 4K and with the game fonts) looked at. Not tried in a
game.

## Russian and Japanese text

Branch `personal/unicode-text` (12/10/2026), Sol's request after Russian friends saw
their chat as `?`: every font atlas had 256 Windows-1252 slots and drew `?` for
anything else. Implemented: a shared wide-glyph atlas filled on first use (Cyrillic
from each face's own family or the closest, kana and kanji from a bundled Noto Sans JP
subset; [rendering.md](rendering.md#characters-beyond-the-code-page)); Russian sent and
read in Windows-1251 as EternalJK sends it, guessed from the text with `cl_charset` to
override ([networking.md](networking.md#player-text)); the chat field counting the
bytes a line is sent with. Verified: unit tests (both code pages, the guess and its
round trip, glyph scale and baseline, atlas packing and mips, the console cell, the
chat budget, `text.wgsl` validated by naga) and the GPU world shots
`chat_in_russian_and_japanese` (an EternalJK Windows-1251 line, UTF-8 Russian and
Japanese, a typed draft, at 1080p, 4K and with the game fonts) looked at. Not tried in
a game, nor against an EternalJK client with `se_language russian`. The hub's SJK
channel and bios take Japanese too since 12/10/2026 (`personal/hub-japanese`: kana,
kanji and Japanese punctuation in `sjk-identity` `bio.rs`, word for word in the hub's
copy, deployed): unit tests on both sides; not tried from a game.

## Card faces and the first card

Branch `personal/card-face` (12/10/2026), Sol's request: Sol made the first card
("Toxiee and Sol on JoF Memorial", Sol lore 001, Legendary) in `card_import/`
(git-ignored) and asked for everything it needs. Hub branch `feat/card-catalogue` in
Sol-Vulpes/SJK-hub: the catalogue (`cards.json`), a card picked at opening, `POST
/v1/cards/reveal`, `scripts/import_cards.py`, and the `sjk_cards` pack with the first
card. Client: faces read from the packs and their pictures uploaded, the face drawn as
a trading card on the Cards tab (bigger up close) and in the ceremony, Reveal (R), the
loot table's note. See [cards.md](cards.md#faces-and-the-first-card). Verified: unit
tests on both sides, the world shots `duel6_sjk_card_faces` with the real pack looked
at. Not tried: in a game against the deployed hub.

## Settings: Simple and Advanced, Vanilla and the SJK renderer, settings usage

Branch `personal/settings-simple` (12/10/2026), Sol's request: a clearer Settings.
Implemented: Simple (default) and Advanced modes in the SJK UI's Settings
(`ui_settingsMode`, top bar segments, F2), Simple's eight short categories, Advanced's
Display and Network regrouped under sub-headings and Graphics led by the renderer; the
EJK level renamed Vanilla, a Renderer row (Vanilla | SJK (alpha)) in place of the EJK
graphics switch, an alpha warning on renderer settings, and a new profile started on
Vanilla (existing profiles keep their settings); number-coded choices given names;
F3 and Ctrl+F for the search; anonymous settings usage (`cl_shareUsage`,
[client.md](client.md#settings-usage)). Verified: unit tests (modes and rails, the
renderer and quality rows and their defaults, the usage queue and what a change or a
search records, F3/Ctrl+F/F2) and the off-screen settings snapshots
(`sjk_settings_snapshot`, Simple Display and Graphics on Vanilla and on the SJK
renderer) looked at. Not tried in game. The hub's endpoint is built on a hub branch and
not deployed, so no usage reaches it yet.

## Hover cards in the Collection

Branch `personal/hover-cards` (12/10/2026), Sol's request: every achievement and thing
in the Collection says on mouse hover how it is obtained. A shared SJK UI hover card
([sjk-ui.md](sjk-ui.md#hover-cards)) on medals, achievements, shaders, the toy, cards and
the Holocrons tab's tiers; the page's text under it is left out so it does not show
through. Verified: unit tests (timing, placement, wrapping, every medal, achievement and
shader has its words, a card's odds) and the world shots `duel6_sjk_hover_cards` looked
at. Not tried in a game.

## Holding push or pull repeats it

Branch `personal/force-hold-repeat` (12/10/2026), Sol's request: a held Force key
should use its power again once it is ready, as in JoF EJK. EJK does it with the
system's key repeats, which re-run a plain bind; SJK drops those, so a held key that
binds `force_throw` or `force_pull` now puts its `generic_cmd` in every user command
until let go (`cl_forceRepeat`, default 1, Settings > Controls), and the server fires
it on the first command after the cooldown. Only push and pull: codemp takes them on
every command, while the toggles would switch off and the other powers would spend
the pool. See [client.md](client.md) (`cl_forceRepeat`). Verified: unit tests (the
repeated table, last key wins, release taken by a menu, window lost, setting off, a
bind with other commands). Not tried on a server.

## Firing rockets while moving: no stutter

Branch `fix/own-rocket-prediction` (12/10/2026, based on `c6ce9485`, Linux): Creyon's
videos on JoF showed the player shaking forward and back, in first and third person, as
soon as he fired rockets while moving; standing still, and in JoF EJK, there was none.
Rockets are solid (`r.contents = MASK_SHOT`, to be shot down), so they came into the
prediction's solid list, built from the newest snapshot as `CG_BuildSolidList` builds
it. A rocket fired in that snapshot was placed before its launch, where its trajectory
runs back through the shooter; and Creyon's trace of the first fix showed the launched
rocket still blocking him (predicted 20 to 50 units/s where the server kept 250). The
server's traces pass through what the mover owns, and another's missile explodes on
touching a player rather than stopping him: missiles are now left out of the
prediction's solid list, and any other moving box until its trajectory has started
([networking.md](networking.md#own-rockets)). Verified: tests (a rocket blocks the
runner neither before nor after its launch); the workspace checks; Creyon's trace.

## Rocket launcher: the lock-on ring and its sounds

Branch `feat/rocket-lock-on` (12/10/2026, based on `c6ce9485`, Linux): Creyon's video
showed the rocket launcher's alternate attack locking on with nothing on screen and no
sound, where JoF EJK draws a ring round the target and ticks. The game already sent and
predicted the lock (`rocketLockIndex`, `rocketLockTime`); nothing drew it. The HUD now
draws `CG_DrawRocketLocking`: a red `gfx/2d/wedge` slice per 75 ms (150 in Siege), each
turned another 45 degrees, up to eight, then `gfx/2d/lock` pulsing, sized by the target's
distance; each new slice plays `tick.wav` at the target and the eighth `lock.wav`. The
HUD gains a turned-picture draw command (`TexturedQuadTurned`) and two atlas cells
([rendering.md](rendering.md)). Live and in demos; the ring is square on wide screens,
a teammate's lock is not drawn in team games, vehicle weapons' lock times are not covered.
Verified: tests (slices and sounds over time, turns and opacity, placement and size by
distance, no ring without a lock, for a teammate or behind the view, a held lock, the
additive picture, the turned corners); the workspace checks. Not tried in game.

## Force powers apply when the Force page is left

Branch `personal/force-apply-on-close` (12/10/2026), Sol's report: on the JoF JA+ server
Force changes made in game did not reach Full Force duels. The client log showed one
`forcechanged` in a whole session of tries: the Force page kept edits as a draft that
only its Apply button wrote, and Escape, Back, another page or closing the screen
dropped it silently. Leaving the page now applies it, as retail's Force menu did on
close, and, at Sol's request, sends the profile again with `forcechanged` even when
nothing changed (not twice after an Apply in the same visit); an unchanged profile is
sent as the player wrote it, not fitted to the server's rank
(`PlayerMenu::settle_force_page`, once a frame, so every way out is covered). Sol
confirmed JA+ reads a profile changed before a Full Force duel at its start, as at a
respawn (the server sends `nfr` at each duel's start and end). The console's
"Force profile: ... sent" lines lost a run of spaces. Verified: unit tests (a changed
draft applied on leaving, the own profile resent, nothing more after Apply,
`forcechanged` queued on leaving by page or by closing). Not tried in a game.

## Opening holocrons: sealed cards and credits

Branch `personal/holocron-opening` (12/10/2026), Sol's request: holocrons open, each
giving a TCG-style card and credits; the cards are placeholders (sealed: rarity, finish and
edition rolled now, the face chosen when the first set is out) and credits will buy
droppable cosmetics as a fix for bad luck (planned). The design and what is built are in
[cards.md](cards.md). Client: wire and worker (`Service::open_holocrons`), Open and Open
all with a ceremony on the Holocrons tab, the Collection's Cards tab, the credits' sign
drawn from shapes, `debug_open`; the UI renderer now cuts strokes to the clip rectangle as
it cut rectangles. Hub: branch `feat/holocron-opening` in Sol-Vulpes/SJK-hub, merged
and deployed 12/10/2026. Then (branch `personal/loot-table`, Sol's request) the Holocrons
tab's loot table: every chance a holocron brings, per tier and per holocron found
([holocrons.md](holocrons.md#loot-table)). Verified: the workspace checks, the hub's tests
and the world shots `cards_shots` looked at. Not tried: against the running hub, in a
game. Then (branch `personal/opening-staff`, Sol's request, 12/10/2026) opening is for
staff only until the hub's operator opens it to everyone (hub `--opening everyone`; hub
branch `feat/opening-staff`, deployed 12/10/2026): the tab hides Open for the others and
says opening comes soon ([cards.md](cards.md#opening)).

## Server browser: servers missing at random

Branch `fix/browser-missing-servers` (12/10/2026, based on `9dbbdf19`, Linux): Creyon saw
the number of servers change from one refresh to the next and favourites come and go. On
Windows a dead server's "port unreachable" comes back as a reset on the socket's next
read, and `query_server_infos` took it for an error and ended the refresh, so every
server that had not answered yet was missing; the engine ignores it (`NET_GetPacket`).
It is now passed over; a server that has not answered is asked twice more (a lost
packet or a busy server's `getinfo` rate limit left it out); the answers already in are
read after every 32 requests; and the list takes up to 1,000 servers (it cut a master's
list at 256) ([client.md](client.md)). Verified: a test that the reset and the other
waits go on, a test of a server answering only the second request, a test with 101
loopback servers answering and one silent, the workspace checks; in game on Windows 11
by Creyon (the count steady from one refresh to the next, favourites staying).
`{JoF}Temple Server` shows only as a favourite: none of `master.jkhub.org`,
`master.ouned.de` and `masterjk3.ravensoft.com` (which lists nothing) lists it.

## Profile: turn the model in game, pick hats and capes from chips

Branch `feat/character-model-picker` (12/10/2026, based on `3ab63816`, Linux): Sol asked
(10/10/2026) for the model picker to "rotate model with mouse click, better cosmetic
picker". Opened from a game, on the SJK UI's Character, Saber and Force pages, dragging
over the model turns it (the live preview's camera goes round it; the turn holds from page
to page), and under its feet "Drag" in a key cap and "turn the model" say so, drawn as the
screen's keys are. On the menu map's stage it does not turn: Creyon's tests found a raised
blade turned end on to the stage's camera looked far too long. Enter or a click on the Hat
or Cape row opens a picker in place of the model grid: Hats and Capes, None and every
installed piece as named chips, the worn one ringed gold; a click wears it, and in game
the model shows its back for capes ([sjk-ui.md](sjk-ui.md#character)). Verified: tests of
the turn (a drag on each page in game, none on the menu map, easing the short way, whole
turns dropped, the preview's angle, the hint) and of the picker (its chips, wearing by
click and by key, Tab to capes turning the model, Escape); the workspace checks; in game
on Windows 11 by Creyon, the drag in the in-game menu.

## Text fields: Ctrl+A, paste, copy and a held Backspace everywhere

Ctrl+A followed (12/10/2026, branch `personal/select-all`): Sol added Ctrl+A to the rule.
A one-line field has no caret movement, so [text_entry.rs](../crates/sjk-viewer/src/text_entry.rs)
keeps one selection, all of one field's text (its address and length, so it never outlives a
change of text or field): Backspace or Delete clears it, typing or a paste replaces it,
Ctrl+C copies it and any other key drops it. The menu pages' fields get it through
`ClientMenu::handle_key` (the field is erased with Backspaces), the prompts, bios, staff
panel, console browser and SJK chat fields through `before_edit`, the report dialog through
its line editor's own selection; the console and the game chat had it. The SJK UI's
`kit::field` and the classic+ `choice_field` draw a highlight over a selected field's text;
the fields that draw their own text with a trailing `_` show no highlight. Verified: unit
tests of the selection and the workspace checks. Not tried in game.

Branch `personal/text-input` (12/10/2026): Sol could not paste an IP into the new
"Add a favourite" prompt, and a held Backspace erased one character; he made it a rule
that every text field pastes, copies and repeats Backspace ([AGENTS.md](../AGENTS.md)).
An audit of the typing handlers found the menu's `if event.repeat` return ahead of every
field behind `ClientMenu::handle_key`, and Ctrl+V handled in only some panels. Now the
menu lets Backspace and typed keys through while a field is open (`menu.rs`), the address,
password and browser search use the new one-line helper
([text_entry.rs](../crates/sjk-viewer/src/text_entry.rs): paste, copy, Backspace, typing),
the settings, character, create game and key binding fields get a paste as the typed text
of the clipboard's first line and Ctrl+C of the field, and the bio, report dialog and
console browser fields accept Ctrl+V as the letter Windows reports and Shift+Insert.
Verified: unit tests of the helper and the workspace checks. Not covered: the F8
screenshot panel's numbers and the First setup FPS box (digits only), the wheel editor's
Ctrl+C. Not tried in game.

## Creyon's PRs #78-#94 merged

Branch `personal/creyon-78-94` (12/10/2026): Sol asked for the review and merge of Creyon's
seventeen open pull requests (CI green on each, unpushed and not played here). Three pairs
overlapped and were merged by hand: #87 and #89 both changed `discover()` (every master is
asked, then the favourites join the list), #89 and #90 both changed the browser's prompts,
and #82 and #92 both added the frame rate to `FramePacer` (#92's version stands, #82's
scoreboard header reads it). Sol's decisions on the review: the match clock joins the upper
right only with the new `cg_drawTimerUpperRight` (off by default; #92 had removed the
layouts' clock for everyone), the second master `cl_master2` (`master.ouned.de:29060`) and the
plain-text remembered passwords stay as built, and #89's host name lookup runs off the menu
thread so a slow DNS answer cannot freeze it.

## Standing on a ledge: no leg stutter

Branch `fix/slope-stand-prediction` (11/10/2026, based on `9dbbdf19`, Linux): Creyon saw his
legs stutter standing still, worst with one foot over a ledge. His trace showed the server
sending a slope pose (`LEGS_S1_LUP5`, the foot hanging) while the client, predicting without
the model's feet, stepped it back to `BOTH_STAND1` within a few commands, every snapshot.
Prediction now keeps the server's slope pose while the player stands still
([networking.md](networking.md#standing-on-uneven-ground)). Verified: a predictor test at
8, 7, 4 and 3 ms command steps (the pose holds; before, it stepped back at the first
command), the workspace checks, and in game on Windows 11 by Creyon (no stutter standing
still or on a ledge, the foot hanging).

## A dropped saber stays visible

Branch `fix/dropped-saber` (12/10/2026, based on `9dbbdf19`, Linux): Creyon saw his thrown
saber vanish once it fell to the ground, until it came back to his hand. A saber that is
knocked down becomes an `ET_MISSILE` with `WP_SABER`, and the projectile drawing had no
model for that weapon. It now draws the saber's hilt from `CS_MODELS`, as `CG_Missile` does
([rendering.md](rendering.md)). Verified: unit tests (the hilt from its model index, the
default saber without one, nothing while `EF_NODRAW`), the workspace checks, and in game on
Windows 11 by Creyon (the hilt stays on the ground until pulled back).

## Door sounds

Branch `fix/door-sounds` (12/10/2026, based on `9dbbdf19`, Linux): Creyon heard doors open
silently on some maps where JoF EternalJK plays their sound. `G_PlayDoorSound` raises
`EV_PLAYDOORSOUND`, event 71, but SJK's sound events answered only 73 (`EV_BMODEL_SOUND`),
which a test had taken for the door event. 71 now plays the mover's soundset sound for the
stage as `CG_PlayDoorSound` does, and nothing without a soundset
([client.md](client.md#door-and-mover-sounds)). Verified: the door test with the right event
number, a test for 73 and a door without a soundset, the workspace checks, and in game on
Windows 11 by Creyon (the silent doors sound).

## Guns sound once

Branch `fix/muzzle-sound-double` (12/10/2026, based on `9dbbdf19`, Linux): Creyon heard
every gun fire twice. His sound trace showed the fire sound played by the predicted
`EV_FIRE_WEAPON`, then the same sound three more times about a round trip later: JoF's HD
muzzle effects carry the fire sound, and SJK started the local player's muzzle flash only
when the snapshot confirmed the shot, where `CG_FireWeapon` starts it with the predicted
event. The local flash now starts with the predicted shot, and the confirmation does not
start it again ([rendering.md](rendering.md)). Verified: tests (a predicted shot flashes
once; neither its replay nor its confirmation flashes again; a followed or unpredicted
player's shot flashes from the snapshot), the workspace checks, and in game on Windows 11 by
Creyon (each shot heard once).

## SJK scoreboard: the frame rate on its header

Branch `feat/scoreboard-fps` (11/10/2026, based on `9dbbdf19`, Linux): Sol asked for the
frame rate on the SJK scoreboard's header line, left of the clock, in the same type, on by
default ([sjk-ui.md](sjk-ui.md#scoreboard)). The number is the one the `cg_drawFPS` line
measures over half a second (`FramePacer::fps`); `cg_scoreboardFps` (archived, default 1,
Settings > Scoreboard) turns it off. Verified: a test at eight window sizes that "240" and
"fps" stand in the clock's type left of the clock and right of the mode, and that nothing
shows without a reading; an off-screen CPU render at 1920x1080 (reviewed); the workspace
checks; in game on Windows 11 by Creyon.

## Clan emblems: a table, the JoFA tag, JoF in teal, tags as players write them

Branch `feat/jofa-tag` (11/10/2026, based on `9dbbdf19`, Linux): Sol asked for JoF
Academy's tag to be read like JoF's, with the academy's logo, and Creyon for JoF's emblem
in the clan's teal and for the tags players write to count. The clans are now the entries
of a documented table (`jof_tag::KNOWN`: tag, emblem, tint, ink width, how a stuck tag
counts) read by generic code, with `cg_clanEmblems` to turn every emblem off
([identity.md](identity.md#clan-tags)). A name tagged `JoFA` gets the academy's HD logo; JoF's
emblem is tinted `#02B99D`; look-alike letters (`jøf'p Evi`, `J0F`, `Jöƒ`, `jøfå`) and tags
stuck to the name (`JoFSanjiJ`, `jofsanji`, `JoFAKen`; not `Joffrey`) count. Verified: tag
tests (JoFA names, look-alikes, stuck tags, near misses, the wider room, the table's order,
emblems and letters, the setting), previews at 20 to 48 pixels and the workspace checks. In
game on Windows 11 Creyon saw the HD JoFA emblem, the teal JoF emblem, look-alike tags and
the setting at work.

## SJK UI in-game menu: the chat under the match card

Branch `feat/ingame-menu-chat-layout` (11/10/2026, based on `9dbbdf19`, Linux): Sol asked
for the match card to move up and the docked SJK chat to sit under it, in its column, rather
than in the middle under the match ([sjk-ui.md](sjk-ui.md#in-game-menu)). The card starts at
y 218 (was 318), the vote over it at 66 and the side's line at 600 (100 higher each); the
dock is at x 1360, the card's width, from y 770. A sender's card now shows left of the dock
and above the keys (`chat_dock::Place::card_floor`), since above it lies the match card's
text, and stays 0.6 seconds after the pointer leaves the name and the card, so the pointer
can cross the dock to it. The main page's dock does the same: its sender card went beside
the name, on its right where it fitted, over the message and the dock's lines under it
(Creyon, 12/10/2026). Verified: tests that the dock lies under the card's lowest control
with and without a vote, inside the window and right of the keys, at five window sizes; that
a sender's card lies left of the dock, above the keys and off the list and profile card, and
stays until its grace has run out; that on the main page it lies left of the dock, above
the keys, off the profile card and over no text at six window sizes; the existing tests that no control, icon or chat area
overlaps another; off-screen CPU renders at 1920x1080 with and without a vote and with a
sender card (reviewed); the workspace checks. In game (Windows 11, Creyon) the layout, the
card's grace and the main page's card placement work.

## SJK chat page: the sender card covers its text

Branch `fix/sjk-chat-card-overlap` (11/10/2026, based on `9dbbdf19`, Linux): Sol saw the
times and messages of the SJK chat page show through a sender's card. Text draws over every
shape, so the page now leaves out the text runs whose measured ink reaches the card while it
shows ([hub-chat.md](hub-chat.md#in-the-menus)); runs beside it stay. `sjk_ui` gains
`DrawList::retain` and `Rect::overlaps`. Verified: a unit test (it fails without the change)
that no history text or time lies under a message sender's card while the history away from
it still shows, a test of `Rect::overlaps`, an off-screen CPU render of the page at
1920x1080 with a card over the history (reviewed) and the workspace checks; in game on Windows 11 by
Creyon.

## Character page: no Model row

Branch `fix/character-model-row` (11/10/2026, based on `9dbbdf19`, Linux): Sol asked to
remove the Character page's Model row (‹ olol/dark ›), which only stepped to the next model
like a next button; the grid chooses the model ([sjk-ui.md](sjk-ui.md#character)). The
classic profile's own model controls are unchanged. Verified: the workspace checks; in game on
Windows 11 by Creyon.

## Server browser: a second master server

Branch `fix/server-browser-masters` (11/10/2026, based on `9dbbdf19`, Linux): Sol saw
servers missing from the server list. SJK asked one master (`master.jkhub.org`); stock and
EternalJK clients know `master.ouned.de` too, which JKHub recommends beside its own. Both are
now asked at once (`cl_master`, new `cl_master2`) and their lists merged, each server once;
a master that fails is passed over ([client.md](client.md#menu-style)). Verified: unit tests
of the list's splitting and of the merge (duplicates, one master down, both down) and the
workspace checks; in game on Windows 11 by Creyon, the browser filling in from both
masters.

## Server browser: names in the game's charset

Branch `fix/browser-name-symbols` (11/10/2026, based on `9dbbdf19`, Linux): Creyon saw
`?jof?entinel?rollem2` in the browser for `jof§entinel·rollem2`, and servers missing from
the list. Server replies carry names as Windows-1252 bytes; the browser refused a server
info that was not UTF-8 (the server vanished from the list) and replaced such bytes in
players' names with `?`. Infos, statuses and printed replies are now read as the game reads
chat (`sjk_protocol::decode_legacy_text`: UTF-8 when valid, else byte by byte)
([client.md](client.md#menu-style)). Verified: tests that a host name with `»«§` keeps its
server and that a status's player names keep `·§`, and the workspace checks; in game on
Windows 11 by Creyon, the names showing right and those servers listed.

## Server browser: add a favourite by its address

Branch `feat/browser-favourite-by-address` (11/10/2026, based on `9dbbdf19`, Linux): Sol
asked for favourites added by hand. The SJK UI browser's left column gains Add a favourite
(and the key A): a prompt like Join by address that keeps the address as a favourite, then
shows the favourites and refreshes ([sjk-ui.md](sjk-ui.md#servers)). Every refresh now asks
the favourites for their info directly as well as the master's list, so a server no master
lists still answers; when the master fails the favourites are still asked. Verified: a test
of the prompt (Add, a bad address and a duplicate refused, the address kept and saved, the
favourites shown, Join by address unchanged), a test that unlisted favourites are asked
once, a test that the browser's controls have distinct tokens, the screen test with the new
button and the workspace checks; in game on Windows 11 by Creyon.

## Server browser: remembered passwords

Branch `feat/browser-saved-passwords` (11/10/2026, based on `9dbbdf19`, Linux): Sol asked
for a password option for locked servers such as Temple. The password prompt opens with the
password remembered for that server already typed and has a "Remember on this PC" switch,
on by default; joining keeps or forgets it ([sjk-ui.md](sjk-ui.md#servers)). Verified: tests
of the store (kept on disk, forgotten, empty never kept, a bad file read as none) and of the
prompt (typed next time, the switch above the buttons, switching off forgets); an off-screen
render of the card (reviewed); the workspace checks; in game on Windows 11 by Creyon.

## Server browser: the players scroll

Branch `feat/browser-players-scroll` (12/10/2026, based on `9dbbdf19`, Linux): Creyon asked
for the SJK browser's "Playing now" list to scroll instead of ending in "and 16 more". Past
six players the wheel over the list scrolls it, two rows a notch, with a thin bar showing
where the rows in view are; it starts at the top for each server ([sjk-ui.md](sjk-ui.md)).
Verified: a test wheeling through ten players (down, clamped at the last row, up, the server
table unmoved, everything fitting) and the workspace checks; in game on Windows 11 by
Creyon.

## Upper right: the frame rate and the match clock as in JoF EJK

Branch `feat/upper-right-fps-timer` (11/10/2026, based on `9dbbdf19`, Linux): `cg_drawFPS`
showed one long line ("125 FPS host 2.44 ms p99 3.83 max 4.87 1%low 115", wrapping at the
edge) and `cg_drawTimer` nothing with the game-data HUD. The upper right now stacks them as
JoF EJK does: "125fps" in big console characters, the frame times smaller under it, then
the match clock, right-aligned, in every HUD style. Sol kept the layouts' top-centre clock
by default (12/10/2026): the clock joins the stack only with `cg_drawTimerUpperRight`
(Settings > HUD, off by default), and then the layouts' own clock steps aside
([client.md](client.md#hud-style)). Verified: a test of the stack's order and height, an
off-screen render of the three lines in the console font (reviewed) and the workspace
checks; in game on Windows 11 by Creyon.

## showplayerid

Branch `feat/showplayerid` (11/10/2026, based on `9dbbdf19`, Linux): Creyon asked for JoF
EternalJK's `showPlayerId`. The console command `showplayerid` prints each player's client
number, coloured team letter and name (` (bot)` after bots) from the player configstrings,
then `Listed N clients`, in EternalJK's format ([client.md](client.md)). Verified: unit tests
for the lines and an empty server, and the workspace checks; in game on Windows 11 by Creyon.

## amcolor kept across reconnects

Branch `fix/amcolors-saved` (11/10/2026, based on `9dbbdf19`, Linux): Creyon reported that a
tint set with `amcolor` was gone after reconnecting. SJK had EternalJK's `amColor` only as
the JA+ mod's `japlus.color`, so `amcolor` went to the server, which applied it for that
connection only. The console command `amcolor <r> <g> <b>` now sets the archived
`char_color_*` userinfo cvars, as EternalJK does ([client.md](client.md)). Verified: unit
tests (set, saved and read back by a new console; bad values change nothing) and the
workspace checks; in game on Windows 11 by Creyon (kept across a reconnect).
## Electrified players crackle (the JoF amlogin sound)

Branch `fix/electrify-crackle` (12/10/2026, based on `9dbbdf19`, Linux): Creyon heard no
sound when logging in with `amlogin` on JoF, where JoF EJK buzzes like a DEMP2 alt hit.
His demo shows the server sends a print and sets `ps.electrifyTime` about four seconds
ahead; stock's `CG_Player` then crackles (`sound/effects/energy_crackle.wav`) on one
frame in ten of the electric shell, which SJK drew without the sound. It now plays it
for every electrified player, the local one in first person too, on the 8 ms effect
cadence rather than every rendered frame, and not for a muted or mind-tricking player
([client.md](client.md#electrified-players-crackle)). Verified: tests of the crackle's
gates, of its rate over the demo's four seconds, of the same rate at 125 to 1000 frames a
second and of its draws; the workspace checks; in game on Windows 11 by Creyon (the
crackle on `amlogin` on JoF).

## Downloads that work, and what they may bring

Branch `personal/allow-download` (12/10/2026, based on `c2016fbf`, Windows 11). Sol
asked for `cl_allowDownload` to work and for downloads to be stripped of anything
harmful, such as shaders and scripts. Found: JoF's March 2026 reference list had a
pak name with a space (`base/{JoF}Sentinel Nexus`), which shifted every later name
onto the next checksum, and names with braces and apostrophes failed the filename
check; any one refused or bad pak ended the join. Now names are rejoined, the
filename rule takes community names, requests are quoted, a failed pak is skipped,
a map nothing installed has is taken from an earlier download when the server lists
no paks, and a download only adds content: programs, configs, ICARUS scripts, menus,
nested archives and replacements of installed files are hidden at mount, and its
shader scripts may not redefine installed shaders or texture-named ones
([client.md](client.md#what-a-download-may-bring)). The archives stay as downloaded,
so their checksums (and pure proofs) are unchanged.

Not a fix for: JoF's servers today send `sv_allowdownload 0` and an empty
`sv_referencedPakNames` (Sol's 11/10 logs, both JoF addresses and 135.125.145.49), so
no client can download from them; EternalJK and JoF EJK download over UDP only, as SJK
does. Verified: unit tests (name rejoining on JoF's real list, the filtered mount, the
policy's kinds and replacements, the shader screening), the workspace's tests. Not
verified: a download from a server, the skip-and-continue path against a stock server,
the map fallback in a game, and FFA3 Night under the policy.

## Log requests

Branch `personal/log-requests` (11/10/2026, based on `491ee67c`, Windows 11), with the
hub's `feat/log-requests`. Sol asked to request logs from friends who have problems
through the hub, and to see crash reports on the Staff page. Staff press Request logs on
a player (Staff page, Logs tab); the player's game shows a card on the menu asking
whether to send; Send logs uploads the newest lines of `logs/session.log` (new: the
chat-free crash trail written to disk, the previous session kept) and of the session
before, and the last crash report. Staff open the logs and every player's crash reports
(Crashes tab) as text files ([identity.md](identity.md#log-requests)).

Verified: unit tests (the fitting under the hub's limits, the wire's paths and fields,
the service's polling, answering once, retrying an answer the hub never heard, staff
only, the opened text's safe file name, the card's keys, words and announcements, the
session log's rotation, restart at 16 MB and tails, the Staff page's actions and every
view fitting the canvas); `hub_e2e`'s `a_staff_key_asks_for_logs_and_reads_them_and_crash_reports`
against the hub branch built and run locally (request, listing, a fitted ~400 KB
multi-byte upload taken whole, decline, crash groups, reports and one read back); world
shots `log_request_shots` (the card, the Logs and Crashes tabs) and `duel6_sjk_staff`.
Not verified: in a game (the card on a real request, the editor opening, a real
session log across a crash); the hub branch is not deployed, so live hubs answer 404
and clients ask them again hourly.

## Gameplay FPS on Auto, and rate 90000

Branch `personal/auto-ticks` (11/10/2026). Sol asked for the command clock to follow
"the server's max" by default, only JoF's server capping it, with an easy cvar that
does nothing at 0; asked what that max is elsewhere, Sol chose the FPS cap. The cvar is
now `gameplay_fps` (was `cl_gameplayFps`, never released), default 0 (Auto): the FPS
cap's step, a `pmove_fixed` server's step first; the card and the setting gained Auto,
an import picks Auto (or 125 for JoF EJK's `cl_cmdratecap`), and the command is
`help_gameplay_fps`. `rate` defaults to 90000, a saved 25000 moved once. Verified: unit
tests (Auto at caps 333, 125, 0 and 10, the server's step over Auto, the card's choices
and import picks, the rate move and a rate chosen after it), and a world shot of the
step after an import. Not verified in a game.

## Gameplay FPS and importing from other clients

Branch `personal/gameplay-fps` (11/10/2026, based on `491ee67c`, Windows 11). Sol asked
for First setup to start with importing a config from another client (base, EJK,
TaystJK) and, above all, for the frame rate's split from movement to be explained,
since players keep asking, with an easy choice of the movement they are used to and an
advanced mode for any value. User commands now run at the gameplay FPS
(`cl_gameplayFps`, 125 by default as before; a `pmove_fixed` server's step wins)
instead of a fixed 8 ms; the Import page lists the configs found from installed
clients, imports the FPS cap, and asks for the gameplay FPS with the old config's
picked; the `gameplayfps` command and a Video setting open the same card
([client.md](client.md#gameplay-fps),
[networking.md](networking.md#user-commands-and-move-packets)). Prediction keeps 512
pending commands instead of 128.

Verified: unit tests (steps of 8, 7, 4 and 3 ms for the presets, 2 ms for 400, the
clamp, the server's step winning, a 3 ms grid filling a 60 FPS frame's five slots, a
changed step never resending a slot, the old config's frame rate picked or 125 under
`cl_cmdratecap`, digits into Other, Left/Right through the presets, the import of
`com_maxfps`, configs found newest first with their client, name and date and SJK's
own left out), the settings catalogue tests, and world shots of First setup in both
styles, the Import page on Sol's installed configs, the gameplay FPS step after an
import and on its own with Other typed. Not verified: movement in a game at 142, 250
or 333 (jump heights and strafes against an EJK client at that FPS on a server without
`pmove_fixed`), prediction at those rates with high ping, and the file dialog.

## Mimic

Since branch `personal/experimental-showcase` (11/10/2026) `ghost` and `mimic` are
experimental, off by default behind Settings > Gameplay > "Experimental: showcase
tools" (`cl_experimentalShowcase`), at Sol's request: they drive the player on live
servers and are a little glitchy, fine for filming.

Branch `personal/mimic` (11/10/2026, based on `71acafb0`, Windows 11). Sol asked for a
mimic for saber showcases: stand facing another player and copy their mouse movement,
movement, crouches and saber attacks, with a configurable delay. `mimic <player>
[delay] [mirror|copy]` ([client.md](client.md#mimic)) rebuilds commands from what
snapshots show of the leader. The real-Pmove tests found and fixed four things a plain
copy gets wrong: crouching while running rolls, so a crouch waits for walking pace;
holding an attack's keys through the move walks too far, so they are pressed only where
JKA chooses the next move; chasing the offset in the air bends the flight, so the air
follows the leader's acceleration; and any backward key component slows the player, so
a slight one is dropped.

Verified: unit tests (the command's arguments, the facing and mirror frames, sample
interpolation, a key stopping the copy, every directional attack's keys, transitions,
the mirror's swaps, specials and reactions) and the real Pmove for leader and mimic on a
flat floor with snapshots 24 ms apart arriving 60 ms late: at delays of 0, 0.3 and 1
second the mirror copies the saber chain mirrored, each attack 4-24 ms late, jumps,
crouches, mirrors a 30-degree turn and ends within 2 units; it tracks within 10 units
with a delay and 31 without. `walk` (branch `personal/mimic-walk`, Sol's request for a
walking choreography) walks always, and any walk keeps its keys within 64, past which
Pmove clears the walk; with a walking leader the tests hold the walk on every command,
track within 8 units with no delay and 5 with 0.3 seconds, and copy the attacks.
Not verified: in a game on a server (real snapshot rates,
specials needing Force or a nearby enemy, JA+ moves, collisions between the two
players).

## Ghost replay

Branch `personal/ghost-replay` (11/10/2026, based on `2dae1fa1`, Windows 11). Sol asked
for a way to record a run (jumping across pillars and back) and have the client redo it
exactly on a server with other players, while another player films, adjusting itself
if lag or anything else knocks it off. `ghost record`, `ghost stop` and `ghost play`
([client.md](client.md#ghost-replay)) record the commands with the state after each and
resend them later with fresh stamps, walking to the start first; drift is corrected
through the movement keys on the ground only, with a deadband, because Pmove rounds the
velocity every command and small corrections otherwise leave a lasting sideways speed
(found by the real-Pmove test below; the first controller made a 0.4-unit start offset
into 11 units by the end).

Verified: unit tests (the file format, names, the key turn keeping the speed, the
steering's deadband and caps, the walk's push, presses carried over skipped commands,
stops on a key, drift, a far start and another map) and the real Pmove on a synthetic
pillar course: replayed from its start the run ends where the recording did; walked to
from 30 units off with the second takeoff's command and one in the air lost, it lands
every pillar and ends 0.4 units off; a 60 units a second shove on the platform ends 7
units off against 46 unsteered. Not verified: in a game on a live server (prediction
against a JA+ or JoF server's own movement, a real map's movers, the filming player's
view of it); race modes are not detected.

## The Void's black holes and more effects

Branch `personal/void-holes` (11/10/2026, based on `ac84cbc0`, Windows 11), with the
hub's `feat/void-holes`. Sol asked for more effects on the Void and, like the Sun's
glints, two small black holes orbiting it. The format's fourth set gains `holes`, drawn
in the glints' places (`orbit` places them), in the later additions file
([unlockables.md](unlockables.md#blade-skin-files),
[rendering.md](rendering.md#saber-blade-skins)): a photon ring and swirl in the glow, a
lens bending the scene with the haze, and a black horizon drawn after the glow (two new
pipelines). The Void's first additions file adds warped space (haze), a gravity pulse
and afterimages, which 2026.1011.x clients draw too; its later one the holes. The skin
uniform grows to 62 lanes (15872 bytes of the 16 KiB).

Verified: unit tests (the holes' fields, ranges, star and holes exclusive, the orbit
needing one, the uniform lanes, the option leaving the instances out) and `saber.wgsl`
validated by naga; world shots with the hub branch's built pack: `duel6_mythical_parts`
(the Void all parts on beside all off, close up, upright) and `duel6_orbits`
(`duel6-void-orbit.gif`: the horizons black, the ring violet, the pedestal behind bent).
The hub (`feat/void-holes`, merged `49f2e69`) was deployed on 11/10/2026 and serves the
new pack. Not verified: in a live match, the lens's and horizon's cost on a GPU.

## The Sun's orbiting glints

Branch `personal/sun-orbit` (11/10/2026, based on `3654f5b2`, Windows 11), with the hub's
`feat/sun-orbit`. Sol asked for the Sun's tip glint lower on the blade, circling it with
a second glint. The blade-skin format gains a fourth set, `orbit` (glints `at` units
below the tip, `count` of them, `radius` out, `rate` turns a second, `tilt`), read from
a later additions file, `<id>.bladeextra2`, that clients of the third set never open
(they keep the glint at the tip) ([unlockables.md](unlockables.md#blade-skin-files),
[rendering.md](rendering.md#saber-blade-skins)). The Sun's: two glints, the second
smaller, 9 units below the tip, 4.5 out, 0.45 turns a second; they dim behind the blade.
The skin uniform grows to 60 lanes (15360 bytes of the 16 KiB).

Verified: unit tests (the orbit's fields and ranges, the later file laid over the first,
the uniform lanes, one instance per glint with its number and bounds) and `saber.wgsl`
validated by naga; world shots with the hub branch's built pack: `duel6_mythical_parts`
and `duel6_sun_orbit` (an upright Sun up close over one orbit, `duel6-sun-orbit.gif`).
The hub (`feat/sun-orbit`, merged `8568413`) was deployed on 11/10/2026 and serves the
new pack. Not verified: in a live match.

## Saber steam in rain

Branch `feat/saber-rain-steam` (11/10/2026, based on `3654f5b2`, Windows 11 Pro,
Rust 1.99.0) ports [JoF EJK PR #257](https://github.com/JediofFreedom/JoF_EJK/pull/257).
Exposed rendered blades play the stock `saber/fizz` steam/sizzle effect at independent
300–600 ms intervals. `cg_saberRainSteam` is archived: `1` everyone (default), `2`
only your blades, `0` off; both settings interfaces expose it. Dry-point and sky
traces include moving inline solids and permanent BSP instances. See
[rendering.md](rendering.md#weather).

Verified: formatting, locked workspace build/tests and all-target clippy pass
(existing warnings remain). Eight focused unit tests cover mode ownership, archived
defaults, timer bounds at 8/7/4/3 ms, duplicate submissions and demo rewinds, roofs,
indoor/liquid volumes, a translated inline door, rain commands, clear/die/freeze,
forced rain/snow and disabling weather. The release run also passes an off-screen
GPU test on `mp/duel6` at 960×540 with forced rain, fog/clouds off, and installed
retail assets: Off creates no particles, On creates the stock fizz particles, the
graph contains its sound component, and the Off/On images were reviewed. GPU:
RTX 5070 Ti, driver 617.14; CPU: Ryzen 7 5800X3D. One blade's added submission cost
averaged 0.199 µs over 1,000 release iterations at 8 ms presentation intervals, with no
snapshot solids or audio; this excludes rendering and does not measure a crowd.

Unverified: a live match's held, first-person, dual/multi-blade and thrown saber
views, actually hearing the sizzle, moving-door visuals, other maps/GPUs, and
crowd/frame-time cost. The GPU check uses a tagged blade, not a connected player.

## Lagometer scale

Branch `feat/lagometer-scale` (11/10/2026, based on `3654f5b2`, Windows 11 Pro,
Rust 1.99.0) adds archived `cg_lagometerScale`, default `1`, range 0.25–4, in both
settings interfaces. It scales the graph around its existing bottom-right corner
and preserves the displayed sample count and millisecond ranges; X/Y positioning
still applies. See [client.md](client.md#lagometer-size).

Verified: formatting, locked workspace build/tests and all-target clippy pass
(existing warnings remain). Unit checks cover default size, scaling and anchoring
with custom offsets at 640×480, 1920×1080 and 2560×1080, range limits, live reads and
archived restoration. A GPU check starts `mp/duel6` at 960×540 and draws synthetic
frame/ping samples through the retained UI pipeline at 0.5/1/2 in the classic,
game and radial HUDs. Sample counts stay fixed, bar widths scale, the GPU draws
green ping bars, and the comparison images were reviewed (RTX 5070 Ti).
Unverified: live-network use, custom HUD files and other GPUs. The UI comparison
uses its own render target because a snapshot-less world suppresses gameplay UI.

## Classic scoreboard interactions

Branch `feat/classic-scoreboard-behavior` (11/10/2026, based on `3654f5b2`,
Windows 11 Pro), compared against JoF EJK `codemp/cgame` revision
`bd5e202dfedd49500f1b7aadd655619bac152167`. Classic uses JoF's bottom-left chat,
held-board chat suppression above 25 connected players, compact automatic
intermission list with a full-list hint, instant opening/reordering, score-request
gate and duel intermission headers. The current classic-parity change
adopts JoF's automatic death board and selective HUD visibility too. The SJK
board keeps its held/intermission policy, whole-HUD suppression, side-column
chat and animated presentation ([client.md](client.md#scoreboard-styles)).

Verified with Rust 1.99.0: `cargo fmt --all --check`,
`cargo build --locked --workspace`, `cargo test --locked --workspace` and
`cargo clippy --locked --workspace --all-targets` pass (existing warnings remain).
Focused tests cover roster changes without score rows, 20/21 and 25/26-player
boundaries, compact/full lists, losing-team room, empty score caches, two-second
requests and rewinds, duel-header precedence, manual death's retained killer and
classic motion. GPU startup and three 1920x1080 `mp/duel6` frames with synthetic
rosters/chat passed on RTX 5070 Ti, driver 32.0.16.1714, Ryzen 7 5800X3D; inspected
ordinary chat, compact intermission/chat and full held list without chat.

Not verified in a live match, against a running JoF client, on another GPU or for
performance. Existing plugin death-handshake, Siege class/winner presentation and
recorded demo score-row gaps remain; the UI does not fabricate missing server
facts. The source comparison and synthetic frames establish these interaction
decisions, not full JoF compatibility.

## Equipped jetpack attachment and prediction

Branch `fix/jetpack-prediction` (11/10/2026, based on `3654f5b`, Windows 11 Pro,
Rust 1.99.0),
for the missing equipped model and jitter reported on JoF's JA+ server. The
pack now has a resting chest attachment on the shared fallback body and resolves
the current bolt on the equip frame. `PM_JETPACK` predicts reduced gravity,
height-dependent per-slice thrust, idle hovering/descent, directional legs,
air control and airborne ground tracing. The jump-pad adapter now recognizes
its actual movement type, 1, rather than spectator-intermission type 8.

Evidence: source comparison against OpenJK `codemp` at `4d0dfaf1`, with the
unmodified gravity/thrust blocks extracted and compiled using MSVC 14.44;
all six height/velocity/flag boundary cases matched the Rust tests. Focused
command tests cover 8/7/4/3 ms, animation directions, gravity restoration and
jump-pad eligibility. A release measurement on Ryzen 7 5800X3D gave 0.242 us
per 8 ms jetpack command over 100,000 commands on a synthetic floor, versus
0.340 us for normal movement; it excludes BSP tracing and rendering and is
not a live-game performance claim. The ignored `jetpack::tests::equipped_pack_on_duel6`
started the actual GPU (RTX 5070 Ti, driver 617.14), loaded retail assets only,
and compared unequipped/equipped frames of a synthetic Kyle fallback body on
`mp/duel6`. Its resting chest bolt was available immediately, both stock flame
bolts resolved, and the pack reached the model pass and visibly sat on the back.
This asset/attachment/model-pass check supplies the pack instance directly;
it does not exercise a received server snapshot or flight. Workspace formatting,
build, tests and clippy passed; existing clippy warnings remain.

Unverified: the reported disappearance and corrections on the live JoF JA+
server, other player models and replacement packs, actual sound/flame playback,
fuel/toggle timing, stock spinning-saber random impulses and jaPRO's separate
new/race flight. See [networking.md](networking.md#equipped-jetpack-prediction)
and [rendering.md](rendering.md#entity-render-effects).

## Mythical shader parts

Branch `personal/shader-options` (11/10/2026, based on `5dadea06`, Windows 11), with the
hub's `feat/look-options` (the look's `saber_off`) and `feat/mythical-parts` (the Sun and
Spectral files), Sol's request: more effects on the Mythical shaders, some of them
switchable. The blade-skin format gains a third set read from an additions file older
clients never open (a tip glint, wisps, heat haze as a pass of its own, an echo) and
`options`, the parts a wearer may switch off, sent with the look so every SJK player
draws them the same ([unlockables.md](unlockables.md#options),
[rendering.md](rendering.md#saber-blade-skins)); the Collection's Shaders tab ticks
them, `saberskin <id> <part> on|off` too.

Verified: `cargo fmt --all`, `cargo test --workspace --release` (all pass) and `cargo
clippy --workspace --all-targets` with no new warning in the files changed;
`saber.wgsl` validated by naga; unit tests of the file's third set and options (ranges,
sections, bit collisions, the additions file laid over the skin), the option mask and
room, the CPU's parts left out, the glint's and echo's instances, the cvar, the look's
`saber_off` both ways and the command. World shots on one GPU with the hub branch's
built pack: `duel6_mythical_parts` (each skin all parts on beside all off, close up and
upright; the haze measured to bend about 87,000 pixels behind the Sun and nothing of the
glow) and `duel6_sjk_collection` (the Parts grid of both skins). Not verified: a live
match, another player's view through a deployed hub (neither hub branch deployed), the
haze and wisps' cost on a GPU, the wisps against a bright sky (additive, they fade there).
## Effects and sabers under the weather, no squares

Branch `personal/effect-transparency` (11/10/2026, based on `58c3f20d`, Windows 11). Sol
saw the weather over weapon effects and some saber shaders and the square of gun smoke
textures. Three causes, found in the code and the installed game data:

- The weather (rain, snow, the volumetric haze and ground fog) was drawn into the effect
  layer after every effect. Effects write no depth, so the haze, measured to the wall
  behind, greyed blades and smoke as if they stood there, and rain lay across them. The
  weather is now drawn after the impact marks and before the other effects.
- Effect stages that multiply the scene took the effect's fading colour although their
  shader says `rgbGen identity`: the repeater's alt-fire burst (`concussion.efx`, ten
  fading `gfx/effects/shock_ball` puffs, `GL_DST_COLOR GL_SRC_COLOR`) turned dark squares.
- `clear` (`GL_ZERO GL_ONE`), which servers remap effects onto (Sol's log: "unsupported
  effect blend falls back to alpha: clear"), drew its white image as an opaque square.

Verified by unit tests (the `clear` stage, which stages take the colour, the tint) and
the viewer suite; no world shot (Sol's game was running) and not tried in a game. Not
changed: rd-vanilla also ignores the colour of additive and blended stages without
`rgbGen vertex` (`fire_radial`, `wookie1` keep fading in SJK); effects far away in fog
are no longer hazed.

## Balanced without light shafts

Branch `personal/balanced-shafts` (11/10/2026, based on `82c3fa31`). Sol asked to take
the two settings that cost the most frames in the 09/10/2026 frame test out of the
Performance and Balanced levels. Recomputed from `logs/perftest-20261009-2134-high.log`
(JoFTemple, 4K, RTX 5080, step against the base, run-to-run noise about 0.4 ms): light
shafts off saved 0.79 ms, floor mirrors off 0.64 ms, the next were bounce light and the
near sun cascade (about 0.5 ms each, debug bits with no setting of their own).
Performance already had both off and Balanced had mirrors off; Balanced now has
`r_volumetrics 0` (was 1) and `r_dustMotes 0` (dust shows only in shafts). Verified by
the graphics quality tests; not measured in game.

## Crash reports

Branch `personal/crash-reports` (11/10/2026, based on `ea95fcdd`, Windows 11). Sol asked
for a crash handler that sends crash data to the hub: a friend crashes often and
`qconsole.log` (off by default, emptied at each start) never holds the panic. A panic
hook, the fatal error path and a per-session marker for silent exits write reports to
`crashes/` with the last console and diagnostic lines (no chat); the identity service
sends them at the next start (`cl_crashReports`, default 1). Hub side: SJK-hub branch
`feat/crash-reports` (`POST /v1/crash`, operator `crash-groups`/`crashes`), not deployed
yet; until it is, reports wait (404) and go once it is. See
[identity.md](identity.md#crash-reports). Verified by unit tests (format, cutting, the
folder, a sending round, unclean markers) and the workspace suite; no crash tried in a
game.

## Ctrl+V without a PowerShell

Branch `personal/fast-paste` (10/10/2026, based on `76068d21`, Windows 11). Sol's
`hitch: 248.5 ms ... between-frames 242.2` came with a Ctrl+V: on Windows `clipboard::paste`
started PowerShell (`Get-Clipboard`) and waited for it. Windows now reads and sets the
clipboard directly through the `clipboard-win` crate (Windows-only dependency, safe API;
the workspace forbids `unsafe`); copies fall back to the tools if the direct call fails,
and other systems keep their tools. Verified by the clipboard unit tests and Clippy; the
real-clipboard round trip (ignored test) and a game not tried.

## Player model mip chains built off the render thread

Branch `personal/model-mips` (10/10/2026, based on `96786a69`, Windows 11). With the
pipelines on a worker, Sol still had `config-refresh` 56 ms (shinzo) and 182 ms
(kushina/default_sum) the frame a model was uploaded: `Forge::texture` built each
texture's mip chain on the CPU there (`mips::chain`: 6.5 ms per 1024x1024, 25 ms per
2048x2048, timed in a release build). The player-load worker now builds the chains
(`warm_entity_materials`, `PreparedTexture`) and `Runtime::upload_prepared_textures`
uploads them under their keys before the materials are appended. Verified by the
viewer suite (release) and Clippy; not tried in a game.

## Menu world rebuild loop on a map change

Branch `personal/menu-rebuild-loop` (10/10/2026, based on `10d953ed`, Windows 11). Sol
and a player hit it on a JoF server's map change: Sol could not join again, the other
player's client closed. In 2026.1010.2 the server stopped sending snapshots for 8 s, the
client reconnected, and `Parked::restore_if_idle` (menu_world.rs) took the shell for
"off a server" because it did not look at `join_task`; it started rebuilding the menu
world, `poll_rebuild` cancelled that under the join, and the next frame started it
again: 13,865 starts in the log, each a worker reading duel6 that the cancel did not
stop. `shell_is_off_server` now requires no join in flight, and a rebuild starts at most
once every 5 s (`RETRY`). Verified by a unit test of the retry rule and the viewer suite;
not tried on a server's map change.

## Player model pipelines compiled off the render thread

Branch `personal/model-pipelines` (10/10/2026, based on `10d953ed`, Windows 11). Sol's
`hitch:` lines showed `config-refresh` 30 to 85 ms the frame some players' models went
on (Jaina, CosmicReborn, yesphantom, sungal, jaina_rogue), not others: models whose
stages need pipeline keys the map had not used compiled 2 entity, glow and fog
variants per key there (3 to 6 ms each from the load profile). Now the async player
load appends the materials without compiling (`append_entity_materials_deferred`),
compiles what `Runtime::pipeline_jobs` lists on a `sjk-model-pipelines` worker, and
places the model once `install_pipelines` has filled the slots (a generation number
drops results made against a program that changed). Fog pipelines became lazy slots
like the entity and glow ones. Verified by the viewer suite (release), fmt and Clippy;
not tried in a game. Log line: `client N: compiling K pipelines for <model> off the
render thread`.

## Join hitch: saber definitions read once

Branch `personal/join-hitch` (10/10/2026, based on `5b0bc8ef`, Windows 11). Sol's
`hitch:` lines from a JoF server showed `snapshot+reconcile` 52 ms when a player joined.
Cause: `SoundEvents::refresh_clients` and `LoopSounds::refresh_clients` each called
`legacy_saber_definitions`, which lists every file of every mount (`vfs.paths()`) and
parses every `.sab`: 30 ms each over Sol's `base` (354 PK3s, 73,494 files), measured in
a release build. `LocalPrediction::refresh_sabers` made two more calls when the local
player's clientinfo changed. `shared_saber_definitions` now keeps the catalog per set
of mounts (`VirtualFileSystem::mount_identities`, the last four sets), filled at world
load; later calls take under a microsecond. Verified by a unit test (a clone shares the
catalog, a new mount reads again), the timing above, the sjk-client and sjk-vfs suites
and Clippy. Not tried in a game. See [rendering](rendering.md#gameplay-hitch-lines).

## Update check while running

Branch `personal/update-recheck` (10/10/2026, based on `726a346a`, Windows 11), Sol's
request after releasing 2026.1010.2 that the auto update tell players to update: the
client only asked GitHub at start, so players in a match heard of a release on their next
start. With `cl_autoUpdate` on it now asks again every 30 minutes while the last check found
nothing new or failed (`update::recheck_if_due`), and the Update card comes as at start.
Verified by a unit test of the rule; not tried in a game. The first release it helps is the
next one.

## Config saved only on real changes

Branch `personal/config-save`, based on `a4ea2316` (10/10/2026, Windows).
Found: every bound key press rewrote `config.cfg` on the main thread. Each
command frame ended in a save (`execute_buffered_frame`, `execute_bound_command`,
`apply_cvar` on any cvar, archived or not), and a `wait` script kept the buffer
busy so it saved every frame: 13 KB of text, `create_dir_all`, a temporary file
and a rename, 0.8 ms median on an NVMe SSD, can be far more with antivirus/HDD.
Now `CvarRegistry::archive_revision` (restamped only when an archived cvar is
added, removed, or changes value or flags) and `BindTable::revision` (bind,
unbind, clear) say whether the file is out of date. `persist()` and a call at the
end of each frame's console upkeep run `Shell::autosave`: two comparisons when
nothing changed; once the state has held still for 2 s (`CONFIG_SAVE_DELAY`), the
text is built on the main thread and written by one background thread
(`sjk-shell` `config_saver.rs`), in queue order. Writes happen at once, after the
writer drains, on exit (the console's `Drop`), `writeconfig`, and `exec
config.cfg`. Verified by: new unit tests (key presses do not write, an archived
change writes once after the delay, an unarchived one never, the exit save
writes), the existing viewer and shell suites, fmt and Clippy. Not tried in a
game.

## Gameplay hitch log

Branch `personal/hitch-log`, based on `a4ea2316` (10/10/2026, Windows). Request:
log random stutters players report, always on. Every gameplay frame's interval
is compared with the median of the last 128; over 33 ms and three medians it
logs one `hitch:` line with the three largest host phases of the long frame
(and time between frames), at most one a second, plus a `hitches:` summary per
60 s of gameplay that had any. Loads, transitions, unfocused windows and the
2 s after a map change, resize, focus change or graphics reload are skipped.
Lines go to stderr and the console scrollback (`qconsole.log` with
`logfile 1`). See [rendering](rendering.md#gameplay-hitch-lines).
Verified by unit tests (median, threshold, rate limit, summary, settling,
phase ranking) and `cargo test --release -p sjk-viewer`; not tried in a game.

## Quieter body damage flash

Branch `fix/subtle-damage-shader`, based on `0c76ff03` (2026-10-10,
Windows): the form-fitting shield-hit flash defaults to one shader pass instead
of four. Old profiles with brightness `4` migrate once to `1`, while other
strengths and subsequent choices remain saved. No shader, hit duration, damage
calculation or shield-sphere behavior changed. A focused profile test covers
fresh defaults, migration and preserving later choices. Real-match appearance
remains unverified.
Formatting, workspace build/tests and Clippy passed on Windows with Rust
1.99.0 (warnings remain), after clearing the viewer's shared dev artifacts
to ensure the tests used this worktree's source.
The Release GPU comparison on an RTX 5070 Ti loaded retail duel6 at 640x480,
HDR and scene HDR/material maps off, CPU model skinning for a fixed Kyle pose.
The isolated personal-shield pass produced summed positive RGB gain above the
body-only image of 48771 at one pass versus 288114 at four. The captured images
were inspected: the one-pass shell was visibly fainter. This is shader-strength
evidence, not a live-match check or frame-time measurement.

## Damage hitmarker toggle

Branch `feat/hitmarker-toggle`, based on `0c76ff03` (2026-10-10, Windows):
the red damage-direction indicator is controlled by archived `cg_hitMarker`,
off by default, also exposed in HUD settings. Only the marker's frame sample
is gated; damage tracking and camera kick retain their existing behavior.
A synthetic test covers default-off, live toggling, profile persistence,
expiry and unchanged view kick. In-match mouse/keyboard use remains unverified.
Formatting, workspace build/tests and Clippy passed with Rust 1.99.0 (warnings
remain). The Release GPU comparison on an RTX 5070 Ti loaded duel6 at 640x480,
HDR/material maps off, then drew the HUD shader with synthetic damage feedback:
0 red pixels off versus 2288 on. The enabled ring image was inspected. This
isolated pass does not verify a live server's damage events.

## Installed PK3 choices

Branch `feat/asset-browser-disable`, based on `0c76ff03` (2026-10-10,
Windows): `assetbrowser` lists installed packs with persistent enable/disable
choices for the next client start. Required retail base archives cannot be
disabled. The startup mount policy and supplemental EternalJK asset probes
honor the saved choices. Tests cover portable case-insensitive names, spaces,
required packs, VFS fallback to a lower-priority archive and re-enabling.
Workspace formatting, build, tests and Clippy passed on Windows with Rust
1.99.0 (warnings remain). A Release offscreen GPU check on an RTX 5070 Ti,
duel6, 1280x960, HDR/material maps off rendered the browser; the captured
card was inspected for fit and legibility. This does not establish input
behavior in a live match.
Server-selected downloads and hub unlockables are outside this policy.
Interactive in-match use and pure-server compatibility remain unverified.

## First-use server models

Branch `perf/on-demand-models`, based on `0c76ff03` (2026-10-10, Windows):
unused `CS_MODELS` entries are deferred instead of building all 511 usable
slots at map load. Newly referenced rigid entity appearances are parsed on a
worker and uploaded into the owning world. Map props, stock weapons/items,
effect models and current player bodies retain their existing preload policy.
Workspace fmt, build, test and clippy passed with warnings. The release
external-data GPU test passed on an RTX 5070 Ti (Rust 1.99.0, driver
32.0.16.1714): duel6 at 640x480 started with 87 rigid models; a late antenna
model was installed once across 100 repeated requests, then the map rendered.
Startup took 16.5 s, and first demand through the first rendered frame took
1.23 s (including font/first-frame work). There is no before/after startup
comparison. Pop-in, texture-upload stalls and populated-match performance remain
unverified; this is not a complete asset streaming system or file hot reload.

## Duplicate local pistol fire

Branch `fix/duplicate-sound-events`, based on `0c76ff03` (2026-10-10,
Windows): the local view client's ordinary snapshot entity no longer plays
events a second time alongside player-state events. This follows OpenJK
`codemp/cgame/cg_ents.c`'s `CG_AddPacketEntities` exclusion of the view client;
temporary event entities and remote players retain their own event latches.
Synthetic tests reproduced duplicate unpredicted pistol fire and external
events before the fix, and cover repeated snapshots at 8/7/4/3 ms intervals.
Audible verification of the reported intermittent DL-44 gunfire and
other potential duplicate-sound causes remains open.

## Held blaster-pistol model

Branch `fix/dl44-held-model`, based on `0c76ff03` (2026-10-10, Windows):
weapon slot 4 no longer shares slot 16's old Bryar world model. It follows
OpenJK `codemp/game/bg_misc.c`'s `weapon_blaster_pistol` model path; slot 16
keeps the `weapon_bryar_pistol` path. A focused test pins both mappings.
Visual verification with a community DL-44 replacement pack remains open.

Reviewed 2026-10-04 against GitHub baseline `b394022` and the owner-approved
client, server, rendering and loading changes described below.

SJK currently contains a native client and standard dedicated server in a
20-crate Rust workspace. This page records scope and verification, rather than
claiming complete parity from the presence of an implementation.

## Unlock cards wait for the window; an update card

Branch `personal/away-cards` (10/10/2026, based on `78f0510f`, Windows 11), Sol's
request: the unlock cards show only while the player is in the game, not alt-tabbed out,
and a newer release gets a card like them. The cards wait while the window is unfocused
or minimised (`ViewerConsole::window_away`, from the window's focus and minimise events),
as under the console, and a card's clock now runs only while it is drawn (at most 0.1 s a
frame), so a card hidden half way resumes and a minimised window or a map load no longer
spends it. The new medal and holocron pop-ups' entrances and fanfare wait the same way.
The update check's result gives one card a session per release ("Update available",
"SJK <version>", "Main menu > Update", SJK's emblem in holo blue, no sound); the Update
page takes it back, and the frame sees a change through a counter (`update::generation`)
without locking the state. `debug_update [version] [manual]` shows the card for a made-up
release without checking anything. Verified: unit tests for the hidden clock (a minute away
leaves the card where it was, ten seconds without frames count as one step, nothing
starts hidden), the update card (read only on a change, once per release, none over the
Update page, which takes it back, no chime), the version kept inline, and every card's
words and canvas, including the update's; the update card's world shot over duel6
(`duel6_unlock_toast`) was reviewed. Not tried in the game window: focus changes and the
update card over a real check.

## Medal and holocron cards in a match

Branch `personal/unlock-toasts` (10/10/2026, based on `fd3ebc38`, Windows 11), Sol's
request: a medal or holocron that arrives during a match is announced like an achievement,
while the large pop-up still opens with the game menu. The achievement pop-up became the
unlock pop-up ([unlock_toast.rs](../crates/sjk-viewer/src/unlock_toast.rs)): the same card
and animation for an achievement, a medal ("New medal", the medallion's picture, the name
with its count, what it is for) or a holocron ("New holocron", the tier's icon or gem, the
burst and name in the tier's colour, where it came from and its odds), with "Open the game
menu to see it" in the category's place for the last two and the achievements' chime. It
replaces the centre print; one arrival brings at most three cards, the large pop-up takes
back the cards of what it shows as it opens, and a holocron's card no longer waits for a
medal still unseen. Settings > Sound > Achievement sound is now Unlock sound (same
`cg_achievementSound`). Verified: unit tests for the queue with the three kinds (a medal
given again is a new card, the same holocron is not), the large pop-up taking its cards
back, the cards' words, every unlock's words fitting the card in the families and Inter
and the card fitting the canvas and the screen at five window shapes, and the pop-ups
announcing each new medal (or count) and holocron once, at most three of one arrival;
off-screen world shots (`duel6_unlock_toast`) of the medal's and each tier's card, in its
burst and held, over duel6 at 1920x1080 and 1440x1080 (the tiers' icons loaded) were
reviewed, and the achievement's moments look as before. Not tried in a game: the cards
over a live match, the chime, and Esc taking the card back.

## SJK chat paste

Branch `personal/chat-paste` (10/10/2026, based on `a70d6ec0`, Windows 11), Sol's
request: Ctrl+V in the SJK chat page's field and the docks' (main page, in-game menu),
which had none (the page passed Ctrl+V's text on unfiltered, the docks dropped it).
Ctrl+V and Shift+Insert now paste through the console's clipboard tools, kept as the
hub takes a message: line breaks and tabs made spaces, characters outside the chat's
alphabet dropped, colour codes kept, cut at 150 characters
([hub-chat.md](hub-chat.md#in-the-menus)). Verified by unit tests of the filter and the
key detection only; not tried in the game window or with a real clipboard.

## SJK chat GIFs

Branch `personal/chat-gifs` (10/10/2026, based on `a70d6ec0`, Windows 11), Sol's decision
that GIPHY links display the GIF in chat, not through the hub: a GIPHY link in an SJK
chat message reads `GIF` and its GIF plays under it on the SJK chat page and the docks,
fetched by each reader's client from `media.giphy.com` (the `200.gif` rendition, the
original only after a 404), from an address rebuilt from the id, with a 4 MiB cap, no
redirect and a 10 s timeout, decoded off the frame thread and kept in memory by id
([hub-chat.md](hub-chat.md#gifs-from-giphy)). `cl_sjkChatGifs` (default 1, Settings >
Network > SJK chat GIFs) switches it; muted senders' GIFs are never fetched. Verified by
unit tests, one real GIF fetched from GIPHY by an ignored test and off-screen world shots
with test GIFs at 1080p and 4K. Not seen in the game window or end to end through the
hub. GIPHY share links with their tracking query are often over the chat's 150
characters and are refused by the hub's rules (unchanged).

Branch `personal/ingame-gifs` (10/10/2026, based on `a10730dc`, Windows 11), after Sol
saw only the link in the chat box during play: the in-play chat box (`chat/view.rs`, the
only path drawing it, in play, beside the scoreboard and over the composer's history) now
shows an SJK line's GIF under it, three rows high (54 pixels at 1080p), at its line's
place and alpha, with the same cache, caps, mute rule and switch
([hub-chat.md](hub-chat.md#gifs-from-giphy)); game server chat keeps links as text.
Verified by unit tests (room under the line at 1080p and 4K, fading and moving with the
line, muted and switched-off lines, the history while typing, a game line) and off-screen
world shots of the box with test GIFs, in play and typing, at 1080p and 4K
(`duel6_sjk_chat_gifs_feed`). Not tried in a match.

Branch `personal/gif-picker` (12/10/2026, based on `fa9f4295`, Windows 11), Sol's request
for a GIF picker with favourites in SJK chat: GIF beside Send on the SJK chat page opens
a picker over the messages with Favourites (kept in `gif-favourites.txt` beside
`config.cfg`), Search (GIPHY's search API from this PC with `cl_giphyKey`, debounced,
paged, cached) and Seen in chat, as animated tiles of GIPHY's preview file; picking
writes the GIF's link in the message ([hub-chat.md](hub-chat.md#the-gif-picker)).
Verified by unit tests (favourites file, search addresses and paging, the cache keeping
tiles apart, the grid's keys, picking into the draft) and off-screen world shots of the
picker at 1080p and 4K with test GIFs (`duel6_sjk_chat_gif_picker`). No real search was
made against GIPHY's API (no key); not tried in the game window.

Branch `personal/gif-own-key` (12/10/2026, based on `46953c53`, Windows 11), Sol's
decision that players use only their own GIPHY key: SJK's shared key is gone, and Search
without a key shows the key card (why, three steps, Open developers.giphy.com, a paste
field, Save key, Not now), which sets `cl_giphyKey`
([hub-chat.md](hub-chat.md#the-gif-picker)). The chat page's field and the picker's
search field copy with Ctrl+C now. Verified by unit tests (only the player's key is
used, a new key forgets refusals, the card shows until put off, Save key refuses what is
not a key) and off-screen world shots of the card at 1080p and 4K (`key-card`,
`key-card-pasted`). GIPHY's dashboard steps were read from its documentation (Create an
API Key), not walked through; not tried in the game window.

Branch `personal/sjk-chat-binds` (12/10/2026, on `personal/gif-own-key`, Windows 11),
Sol's request for SJK chat binds: `sjksay`, `sjkgif` (a favourite's number, an id or a
link, with text), `+gifwheel` (the quick wheel on the favourite GIFs, their previews
moving) and Keys rows for `sjkchat` and the GIF wheel
([hub-chat.md](hub-chat.md#from-a-bind), [client.md](client.md#gif-wheel)). Verified
by unit tests (the words as the chat takes them, which GIF `sjkgif` names, the GIF
wheel's pages and its choice's command, the last page kept) and off-screen world shots
of the GIF wheel at 1080p and 4K (`duel6_gif_wheel`). Nothing was sent to the hub, and
it was not tried in the game window.

## Stutters on joins

Branch `personal/player-async` (10/10/2026, based on `a4ea2316`, Windows 11, RTX 5080),
players' reports of random stutters in game. Read from the code, the likeliest was a
player joining or changing model: the model's files read and parsed, its textures
decoded, materials and pipelines compiled and the shared geometry regrown by copying
all of it, inside one frame on the render thread. Now a worker reads the model, builds
its mesh and decodes its textures (`player_loads.rs`), and the render thread uploads it,
one a frame; the shared buffers keep room for about a dozen models (no copy per model),
and glow pipelines are compiled with their materials, at map load and when a model is
installed, instead of on the first glowing frame
([rendering.md](rendering.md#geometry-and-pipelines-added-during-a-match),
[client.md](client.md)). Still on the render thread: a model's material and pipeline
creation at install, hilts (`load_hilt`), NPCs (one a frame), corpses, cosmetics and
models named by `CS_MODELS` (whose read is on a worker since PR #67).

Verified by unit tests (the load queue: a newer request replaces an older one, a
cancelled load never lands, a stopped worker is reported, nothing starts without a
world). Not measured: the frame times of a join before and after, which the hitch log
(`personal/hitch-log`) is for; not tested in the game window.

## Low FPS help

Branch `personal/help-fps` (10/10/2026, based on `a10730dc`, Windows 11, RTX 5080),
Sol's request: detect a map whose lighting is heavy at load and suggest turning it off,
with a "don't ask again" tick, a `help_fps` command to tell players, and Ultra low
renamed EJK. A few seconds after a server's map is in, the GPU times of eight sampled
frames decide; a card over the game menu offers EJK graphics (the Settings switch,
then `vid_restart`) when the frame misses the FPS target (cap or monitor rate, 60 to
144) with the lighting at least 40% of it ([client.md](client.md#low-fps-help)).
The phase profiler now exists whenever the GPU has timestamps and samples only while
the check or the card watches, or `SJK_GPU_PHASES` asks.

Verified by unit tests (the target, the 40% rule on JoFTemple's numbers before and
after its fix, the median, the card's keys and words, the rename and `ultralow` still
naming EJK). Not tested in the game window: the check on a real map, the card in a
match and its game menu, the reload it starts, other GPUs and maps for the thresholds.

## Memory on servers

Branch `personal/menu-world-drop` (10/10/2026, based on `0c76ff03`, Windows 11, RTX
5080), Sol's report of SJK at 11 GB in Task Manager and FPS going from 200 to 30 on
the same map: the running client was at 13.4 GB after six maps in 17 minutes. Off
screen, duel6's menu world alone was 4.8 GB (2.7 GB of GPU memory, about 1 GB of CPU
memory), kept parked during matches, and the GPU allocator's 128 to 256 MiB blocks
stayed partly used after each map change. The menu world is now dropped on a server
and built again after it, and the device uses 4 to 16 MiB blocks
([rendering.md](rendering.md#gpu-memory)): 4K map changes went from 11.0 to 12.5 GB
to 7.2 to 7.4 GB; at 1920x1080 on a local `devmap mp/ffa3`, from 8.0 to 6.0 GB.

Verified by unit tests and off-screen in release builds: the graphics reload test on a
local `sjk-server` (`graphics_reload_on_server`: join, Ultra low, `vid_restart`,
`disconnect`) passes, the menu world coming back on the reloaded context in 3.0 to
3.8 s and shot over duel6. Not measured: frame times with the smaller blocks (the GPU
was busy with the game during the runs), the FPS swings, the 1 GB of CPU memory each
world keeps, and the menu in the game window (the last server's map under the menus
for a few seconds after leaving).

## Wall-jump view prediction

Branch `fix/wall-jump-prediction` (2026-10-11, based on `3654f5b2`, Windows,
Rust 1.99): wall-move prediction now reports when it held the view, and the
camera presents those held angles even on frames that reuse the same predicted
command time. Delta changes alone could leave fresh mouse movement visible on
such frames, also turning the local character's root. JA+'s free-look rebound
keeps its existing behavior. See [wall-move view holds](networking.md#wall-move-view-holds).

Reference comparison: OpenJK `codemp` revision `4d0dfaf1`, `bg_pmove.c`
(`PM_SetPMViewAngle`, the wall handlers and their ordering in `PmoveSingle`),
and `cg_view.c`'s use of `cg.predictedPlayerState.viewangles`. Focused synthetic
tests cover rebounds, wall runs on both sides, runs up walls, ledge flips,
release, JA+ free look and repeated presentation with unchanged deltas at
8/7/4/3 ms command steps. No gameplay velocity, animation timing or wire
field changes. Workspace fmt, locked build, locked tests and all-targets
clippy passed (existing warnings; the viewer tests have 75 existing ignored
cases). In-game reproduction of the reported turning, remote-server
behavior and full wall-acrobatics parity remain unverified.

## Live console height

Branch `fix/console-height-setting` (2026-10-11, based on `3654f5b2`,
Windows, Rust 1.99): a normal console opening now follows the current
`con_height` in both styles instead of caching its value at opening. Changing
the setting while the console is open therefore changes its target height;
`toggleconsole` also follows the current setting when reusing a normal opening.
Ctrl's full-screen and Shift's quarter-screen overrides remain explicit and
are cleared by a normal opening or Shift+Escape. See [client console keys](client.md#console-styles).
The updated key-height unit test covers normal mode and modifier overrides.
Workspace fmt, locked build, locked tests and all-targets clippy passed with
existing warnings.
The reported resize has not been checked in the game window.

## Graphics reload

Branch `personal/graphics-reload` (10/10/2026, based on `5faf4944`, Windows 11, RTX
5080), Sol's report that Ultra low left the sun and SJK's lighting on: the log of that
session shows it turned on in a match and play going on without a restart, while HDR,
FXAA, sun and sky, sun shadows and material maps were read only when the graphics
context was made. Those settings now apply with a graphics reload: the world on show is
built again on a new context with them read anew
([rendering.md](rendering.md#graphics-reload)), offered by an SJK pop-up card on the
main page or the game menu and run by `vid_restart`
([client.md](client.md#graphics-reload)).

Verified by unit tests (what a reload would change, a comparison that latches nothing,
the card's offer, Later and words, the timing marks) and off-screen in release builds:
on duel6's menu world, Ultra low then Reload rebuilt the map in about 3 s on a context
with HDR, FXAA, sun and sky and normal maps off (`Lamps: none`), the card before and
during; on a local `sjk-server` (`devmap mp/ffa3`), Ultra low then `vid_restart` built
ffa3 again on the new context in 5.6 to 7.2 s over four runs, the same session all along
and still connected 2 s later, and after `disconnect` the parked menu world was rebuilt
on that context in 4.2 s (since `personal/menu-world-drop` it is dropped during the
match and built again then). Seen once and then reproduced with an 8 s stall: ffa3's blue
force-field door shown closed for about 12 frames after a long reload, because the
server ignores a loading client's neutral commands and the player stops touching the
door's trigger; not seen with Ultra low from the start. Not tested in the game window:
the card's mouse and keys, its place over the game menu and the classic menus,
JoFTemple's reload time and a remote server.

## SJK chat sound

Branch `personal/chat-sound` (10/10/2026, based on `c513abee`, Windows 11), Sol's
request: a sound when an SJK chat message arrives; none played before. A new message
from another player, newer than what the feed had seen, plays a variant of JKA's chat
sound made from the game's own `sound/player/talk` when SJK starts (quicker and higher,
cut, faded, a short echo; nothing of JKA's audio ships), in the menus and in games; not
for the player's own messages, muted players, holocron drops or the backlog at start, at
most once a second ([hub-chat.md](hub-chat.md#a-sound-for-a-new-message)).
`cl_sjkChatSound` (default 1, Settings > Network > SJK chat sound) switches it, under the
master `cg_chatSounds`. Verified by unit tests (the sound's length, pitch, fade, echo taps
and level; when it plays; the settings row) and against the sound Sol chose, made from the
retail file: correlation 0.9993, RMS 0.03321 against 0.03319, peak 0.527 against 0.527.
Not heard in the client, in the menus or in a game.

## Create game in the SJK UI's look

Branch `personal/create-game-sjk` (10/10/2026, based on `c513abee`, Windows 11), Sol's
request that the hero look disappear and the SJK UI be the look of every screen without
a classic+ version. Create game and its map list are redrawn with the SJK UI's kit
(`menu/create_game_view.rs`, `menu/map_picker_view.rs`) and shown in every menu style:
`ClientMenu::sjk_screen` covers the Create game phase whatever `ui_menuStyle` says, so
the map shows under it, its text goes to the UI's families and `game_font::prepare`
loads them for it with the classic menus too. The screen's state, keys, rows, pointer
tokens (rows `0..9`, the way back 900, the list's rows `0..14`) and the hosting flow
are unchanged; the hero widgets are no longer used by it (`text_backing` and
`form_cycler_fmt` lost their last user and carry a `dead_code` allowance until the
hero look is removed). Verified by a unit test (both pages lay out within the canvas at
1080p, 4K and 4:3, the cycler's first target is its control), `cargo test -p
sjk-viewer --bins`, clippy and fmt, and the world shot
`world_shot::create_game::duel6_sjk_create_game` (Start chosen, the Map row, the name
being typed, the list, filtered and filtered to nothing; 4K and 4:3 at `ui_textScale
1.2`; the classic menus), looked at. Not tried in a game.

## Staff: verify, merge players, linked keys

Branch `personal/staff-merge` (10/10/2026, based on `6b13a8cd`, Windows 11), with the
hub's matching change built in parallel, Sol's request: "the staff tools page should be
able to verify people and also merge two persons into one if someone resets their key
(by accident for example); we should be able to have multiple keys per person". A
player is now one main key and any linked keys (`Profile::keys`, empty from older
hubs); the Staff page has Verify/Unverify, a Keys and merge tab with Unlink per linked
key (second press) and the merge of another key into the chosen player behind a
confirmation card ([identity.md](identity.md#players-with-several-keys)).
`sjk-identity` gains `StaffRequest::Verify`, `Merge` and `Unlink` (`/v1/staff/verify`,
`/merge`, `/unlink`); a merge's answer replaces the kept player and drops the merged-away
one from the staff list. The client's own id follows the person (`Snapshot::key_id`
takes the own profile's id; `Snapshot::local_key_id` keeps the key file's for the
Identity page and the pop-ups' seen lists; the feed is told through `FeedShared`), and an
unlinked own key registers again. The `staff` command has no subcommands, so none were
added. Verified by unit tests (the wire fields, the requests' paths and bodies, the
worker's list updates and own-id following, the page's verify, unlink, merge and pick
logic, every state fitting the canvas), the workspace checks and the world shot
`duel6_sjk_staff` (new `duel6-staff-keys` and `duel6-staff-merge`, looked at); an
env-gated `hub_e2e` case covers verify, merge, the linked key's profile, unlink and the
hub's refusals and passed against a local build of the hub's change
(10/10/2026). Not tried in a game or
against the deployed hub.

## Console feed off by default: `con_drawNotify`

Branch `personal/console-feed` (10/10/2026, based on `c378cabb`, Windows 11), Sol's
request: "an option to enable/disable the console feed on the top left while in game;
the console feed would be off by default". The closed console's notify lines
(`Con_DrawNotify`, the classic and SJK UI consoles alike) now draw only with
`con_drawNotify 1`, archived, default 0, so every profile starts without them (none
saved the new cvar). Settings > Interface > Console feed (the TEXT tab in the tabbed
screen) is its row, so it is in settings search too. Chat was already kept out of the
notify lines (the `*` echo, `push_chat_line`) and has its own box; what the feed hides
is server prints, the one-line obituaries and the client's own console messages.
Centre prints, the kill feed and the vote and team overlays are HUD elements and stay.
`con_notifylines` and `con_notifytime` still shape the feed when it is on. Verified by
a unit test (off in a new profile, on once set, kept across a restart), the settings
tests (catalogue, groups, help text) and the world shot `duel6_sjk_menu` (Interface
shows the Console feed switch, off), re-rendered and looked at; not tried in a game:
the top left staying empty with the switch off and filling with it on.

## SJK chat page: faces in the who-is-online window

Branch `personal/chat-faces` (10/10/2026, based on `a74f5cf4`, Windows 11), Sol's
feedback on the window below: "we can show who's online with a little circle with their
profile picture in it, maybe just that actually (and the letter if no picture), with
mouse hover to see the profile card". The window's rows of names became faces: the hub
picture from the picture cache or the name's first letter on the player's stand-in
colour, a gold dot for a player in a match, online faces wrapping nine to a row (two rows,
the last face "+N" beyond 18), recent ones smaller and dimmed in one row (up to eight, the
fallback's last senders too), muted ones greyed; when they were seen moved to the sender
card's new note line. The window shrinks to its faces and Chosen message follows it. The
fallback's faces look up their picture versions after the frame, as the card does. The
canvas knows the target under a resting pointer by its place in the frame's list, so when
the faces change the page looks again for what the pointer rests on (without it, the
field's place lit a face and showed its card). Verified by unit tests (the grid's columns,
centres, heights and "+N" split; the letter; faces inside the window, not touching, no time
written; the window shrinking and Chosen message moving up; fit at 1080p, 4K, 4:3, 21:9,
1440p and 1024x768 with whole-pixel discs at 4K; hover at a face's centre, none in the gap,
the card's note, the gold ring, the muted grey, click; picture lookups for senders; the
resting pointer) and the world shot `duel6_sjk_chat_page` (listed, a face's card, a recent
face's card, chosen, a crowd with "+14", unlisted; 1080p and 4K with `ui_textScale 1.2`; four
made-up pictures put in the cache), looked at. Not tried in a game, with real pictures from
the hub or against a hub that sends `people`.

## SJK chat page: who is online

Branch `personal/chat-online` (10/10/2026, based on `c378cabb`, Windows 11), Sol's
request: "add a small window on the SJK chat (only when it is big) to quickly see who is
online/active, and the most recent active players that are offline". The SJK chat page
(the big chat; the docks are unchanged) has a window over Chosen message, which moved down
under it: Online (playing first) and Recently active, or, from a hub without the list,
the count and Recently in chat ([hub-chat.md](hub-chat.md#who-is-online)). The hub gives
only the online count today, so the client reads a proposed optional `people` field of the
feed (`sjk_identity::People`, kept in `ChatState::people`) and falls back on the chat's
senders without it; the hub side is not built. Verified by unit tests (the derivation:
order, dedup, the "more" row, recent leaving out who is online, the fallback skipping
drops; the feed keeping and clearing the list; parsing; the window's rows, cut names,
card, click and fit at 1080p, 4K, 4:3 and 21:9), the workspace checks and the world shots
`duel6_sjk_chat_page` (listed, a card from the window, a chosen message, unlisted; 1080p
and 4K with `ui_textScale 1.2`), looked at. Not tried in a game or against a hub that
sends `people`.

## Illuminate is a toy: `toy_illuminate`

Branch `personal/toy-illuminate` (10/10/2026, based on `75c455c2`, Windows 11), Sol's
request: "change the Illuminate toy to not be a force power, but just a toy, and change
also the name of the command; all toys would probably start with toy_". Illuminate left
the Force wheel for good: the pseudo-slot 21, the known bit the client set for it, its
`+useforce` remap, `Selection::set_illuminate`, the bar's and radial list's entry and
name, and `forceselect 21` are removed with their tests (`force_wheel::MAX_SLOTS` is 21
again; the flamethrower's icon and name moved back down one). The command is
`toy_illuminate` ([client.md](client.md#illuminate)); every toy is a `toy_<name>` command
([sjk.md](sjk.md#names)). `force_illuminate` is a hidden alias: the input dispatcher
recognises it but it is registered nowhere, so completion and `cmdlist` never list it,
and a saved bind is renamed on load. `cg_illuminate` (the Force wheel switch) is
removed instead of becoming the lit state: the holocron stays out at every start as
before, so a saved 0 or 1 has no meaning; a load drops the saved cvar, which leaves
nothing lit or unusable and needs no version marker (it changes nothing once run).
The Collection's Toys tab has a "Holocron lit" switch (the console asks, the viewer
applies, the console mirrors the state each frame); the key-binding row is
Interaction > Toy: Illuminate; the icon is `gfx/sjk/toy_illuminate.png`. Verified by
unit tests (the Force bar and selector never offer Illuminate with any bits, the
command and its alias, the hidden registration, the profile migration, the Toys tab's
click and keys, the quick wheel choice), the workspace checks, and the world shots
`duel6_sjk_collection` (with a new Toys-lit shot: the switch, "put out" in the keys
line, the icon from the quick wheel's atlas cells) and `duel6_quick_wheel*` (the Force
page with 9 powers and Force 2 with JoF's icons, Toys, the Settings editor), re-rendered
and looked at; not tried in a game: the holocron on a key, in the wheel and on the
tab, and others seeing it through the hub.

## Quick wheel: Force first, a Toys page; a Melee bind

Branch `personal/quick-wheel-toys` (10/10/2026, based on `aa95e8cb`, Windows 11), Sol's
requests. The quick wheel's default order is now Force, General, Toys, Weather, so a new
run's bare `+wheel` opens on the Force page ("make the force wheel the first wheel to
appear by default"). A Toys page (`+wheel toys`; "a toy wheel too for the force
illuminate and future toy") is an ordinary page of a new catalogue group, Toys, one entry
per toy: Illuminate (`toy_illuminate`, then `force_illuminate`; the gold dot following the
holocron's lit state, which no cvar holds) moved there from the Force page, which now lists
Force powers only (the Force bar's own Illuminate entry went in the next section). `wheel.json` is
version 3: a file whose pages are exactly the version 2 defaults is removed (the new
defaults apply), any other keeps its pages and gains the Toys page once after its Force
page (else last), a removed Toys page stays removed and a full wheel is left alone
(unit-tested in `pages.rs`, as the Force page's migration was). The new bindable action
Melee (fists), `weapmelee`, selects `WP_MELEE` by name (unbound by default; the rule is
`legacy_melee_weapon`, unit-tested: from the saber, from a gun, already melee, no melee
held, following, emplaced); `weapon 1` stays the saber. The wheel has no Melee choice: a
disc for it would need a new picture, and every action has one. Also two unused imports
and a test's `unused_mut` warning removed. Verified by unit tests, the world shots
`duel6_quick_wheel*` (re-rendered and looked at: Force first, Toys with the holocron lit,
Force 2, the editor on Force and Toys) and the workspace checks; not tried in a game: that
the holocron toggles from the wheel, and `weapmelee` against a real server.

## Medal shaders

Branch `personal/medal-blades` (10/10/2026, based on `8018e58e`, Windows 11), with the
hub's `feat/medal-blades`: Sol decided that medals bring saber shaders while held (Bug
Hunter the Glitch, Early Tester the Hologram, Early Contributor the Runic blade), taken
back with the medal; achievements grant none yet and holocron drops of blades are
parked. Details in [unlockables.md](unlockables.md#medal-shaders). Verified by unit
tests in both repositories; the hub not deployed when built, not tried in the game.

## Nine more saber shaders

Branch `personal/saber-shaders` (10/10/2026, based on `fa97d354`, Windows 11), with the
hub's `feat/saber-shaders`, Sol's request: the Unstable, Molten, Spectral, Glitch,
Hologram, Runic, Chameleon, Banner and Heartbeat blades
([unlockables.md](unlockables.md#catalogue)). The generic blade-skin shading gains ten
optional sections (sputter, glitch, scan, pulse, ghosts, embers, veins, team, ambient,
glyphs; [rendering.md](rendering.md#saber-blade-skins)), instances carry who wears them
(team, name, the light where they are), up to 16 skins load at once, and the
Collection's rack and the Saber tab's blade row scroll. Unlocking is unchanged (staff or
operator by hand).

Verified: `cargo fmt --all --check`, `cargo build --locked --workspace`, `cargo test
--locked --workspace` and `cargo clippy --locked --workspace --all-targets`; `saber.wgsl`
validated by naga; unit tests of the new sections' parsing and ranges, the uniform's
52-lane layout matched by name against the shader, names spelt as glyph codes, every
letter's glyph distinct (a CPU copy pinned to the shader's text), afterimage poses, the
heartbeat's CPU copy for the light, the rack's and the blade row's scrolling. The world
shot `duel6_second_blades` with the hub's built pack (all 14 skins load through the strict
parser) was reviewed by eye on one GPU. Not verified: a live match, other players' looks
through a deployed hub, the sounds by ear, the cost of the new sections on a GPU.

## The Collection: medals, achievements, shaders, toys and nameplates

Branch `personal/collection` (10/10/2026, based on `bca3ac2e`, Windows 11), Sol's
request, its design picked from a canvas of mock-ups ("the vault"). The achievements,
medals and unlockables leave the Profile screen (now Character, Saber, Force, SJK
Profile) for a screen of their own, the Collection, under Profile on the main page's
arc (six entries, 12.5 degrees apart) and in the in-game menu (seven entries), with
tabs Medals (hanging from a rail), Achievements (a wall of medallions by category, the
one chosen up close with "Next up"), Shaders (the blade skins, renamed for players: the
stock blade and every skin in a rack, Body "soon"), Toys (Illuminate, with its switch
to light it) and Nameplates (none yet: the three places ornaments will go, and the
player's nameplate over their model). Behind Shaders, Toys and Nameplates the player's
model stands on the menu map's stage (the camera goes to the player's shot), or shows
in the page's live preview over a match and in the classic menus; it holds the chosen
shader, a locked one too as a local preview, the holocron floats by it on Toys and its
nameplate stands over its head on Nameplates. Commands: `collection` (new),
`achievements` and `unlockables` open its tabs; the Profile page's See the board and
See the collection open it. Docs: [sjk-ui.md](sjk-ui.md#collection),
[unlockables.md](unlockables.md#shaders-tab).

Verified: `cargo test -p sjk-viewer` with unit tests of the two screens' rows (each
screen's tabs wrapping within it, the Collection's counts, rows within the frame at the
largest text style, the areas answering the pointer), the Collection's keys, pointer,
equipping, the locked preview never equipped, the toy's switch, the stage wish, and
every tab within its canvas over the keys at 1080p, 4K, 4:3 and 21:9; the world shots
`duel6_sjk_collection` (every tab from the main page on the stage, the Storm previewed,
4:3, the in-game preview, the classic menus' page), `duel6_sjk_profile_screen`,
`duel6_sjk_profile`, `duel6_sjk_menu`, `duel6_sjk_ingame` and
`duel6_sjk_profile_tabs_styled` were looked at. The nameplate's height over the head was
measured on a world shot for the stage's stance. Not tried in the game: the camera's
move to the player's shot and back when the Collection opens and closes on the main
page, the preview over a real match, sounds.

## Holocron drops: the client

Branch `personal/holocrons` (10/10/2026, based on `bca3ac2e`, Windows 11), Sol's feature,
decided with the hub's protocol the same day ([holocrons.md](holocrons.md)). The hub rolls
and stores holocrons earned by 30 minutes of active play (four tiers, daily caps, staff
gifts); the client sends `active` with every claim (a live, non-local game, the player's
own view, mid-match, out of the menus, input in the last two minutes: `holocrons/activity.rs`),
reads the progress, shows each new holocron once in a modal pop-up on the main menu or the
game menu (a centre print in a match, `holocrons_seen.txt`, the newest 20 on a first read),
words a drop as an SJK chat line in the tier's colour with a gem in the game feed, the dock
and the page, and gives the Staff page Give holocron and Remove. `debug_holocron` rehearses
a drop offline. [holocrons.rs](../crates/sjk-viewer/src/holocrons.rs) is the list the
Profile screen's Holocrons tab reads.

The Holocrons tab is built (10/10/2026, [holocrons.md](holocrons.md#the-holocrons-tab)): the
Profile screen's eighth tab and the `holocrons` command, with a 3D holocron floating in the
world behind it (not on the Character tab's stage: a normal object of the world pass before a
backdrop shot of its own) in the chosen tier's look and light, dimmed for a tier held none
of; the four tiers with their counts and odds, the progress, the newest ten, and the states
when the identity is off. A tier needed a model of its own (`holocron_<tier>.md3`), since a
surface's shader is named by its model file; `scripts/holocron_assets.py` writes them and
the locked look. The classic Profile page has See holocrons. Verified: unit tests and every
state's fit in four window shapes, and world shots rendered with the GPU here and looked at
(tiers, locked, identity off, 4:3, 21:9, 4K, text scale 1.2, classic style). Not verified:
in a game (the holocron over a match), other maps than duel6, real hub data, motion.

Verified: `cargo test --locked --workspace` with unit tests for each piece (the wire, the
worker against a scripted hub, the feed, the catalogue and its colours, the pop-up, the
chat views, the staff page, the activity rules); `cargo fmt --all --check`,
`cargo build --locked --workspace` and workspace clippy (no warning from the new code).
Not verified: anything against a running hub (an ignored end-to-end test in
`crates/sjk-identity/tests/hub_e2e.rs` needs one), in a game, or with the tiers' art (the
pop-up draws a gem until `gfx/sjk/holocron_<tier>.png` is mounted); the Staff page now lists
10 players, not 14, to make room for the holocrons; the Holocrons tab's world shots
are above; the pop-up and the Staff page were not shot.

## The SJK chat dock wraps, and docks in the in-game menu

Branch `personal/chat-dock` (10/10/2026, Windows 11), Sol's request. The SJK UI main
page's docked SJK chat lays each message out over as many rows as it needs, measured in
the body family with the text style (`sjk_chat_look::flow_each`), in a box of fixed
height that shows the newest messages that fit; a message taller than the box shows its
first rows with its name and an ellipsis. The dock is one piece of code
(`menu::sjk::chat_dock`: `Dock`, `DockCache`, `draw`, `sender_card`), drawn by the main
page and by the SJK UI's in-game menu, where it sits under the match between the row of
icons and the card, joins the Tab cycle (list, icons, card, chat) and takes every key
while typing, ahead of the console key and the bindings
([sjk-ui.md](sjk-ui.md#in-game-menu), [hub-chat.md](hub-chat.md#in-the-menus)).

Verified: `cargo test -p sjk-viewer` with unit tests of the wrapping (whole text kept,
rows under rows, older messages pushed out, the cut message at `ui_textScale 1.2`),
typing and the pointer, the focus model's chat moves, the in-game dock typing, sending
without an identity and going away with its page or `cl_sjkChat 0`, its sender card
above the dock in five window shapes, and every in-game main page with the chat within
its canvas; the world shot `duel6_sjk_chat_dock` (main page and in-game menu at 1080p,
4:3 and 4K, plain and `ui_textScale 1.2`, typing, a sender card, no canvas
overflowing) and `duel6_sjk_ingame`. Not tried in the game: the key routing while
typing in a match (`GpuState::sjk_chat_typing`) has no test of its own.

## Quick wheel: a Force page of the player's powers

Branch `personal/force-wheel-page` (10/10/2026, based on `915475f9`, Windows 11),
Sol's request. The quick wheel's defaults were General, Force and Weather (now Force,
General, Toys, Weather: see the first section); a
`wheel.json` of version 1 gets the Force page once, after General, and is written
back (unit-tested, with the removed page staying removed and a full wheel
left alone). The page is live: the powers the Force bar's known bits allow, in a
fixed order (neutral, light or dark in F-key order, JoF's), the
selected one marked; instant powers run `forceselect <n>; force_*` (the retail
`genCmds_t` binds, JoF's server commands), held ones (Grip,
Lightning, Drain, Stasis) only `forceselect <n>`, a new local command
(`Selection::select`, unit-tested). Up to 12 on the ring, drawn smaller; more go
on "Force 2". Settings > Quick wheel shows a note instead of choices, an example
preview, and Add the Force page once it is removed. World shots
`duel6_quick_wheel_force` (light, dark, every power and Force 2, none, no game, 4K
at `ui_textScale 1.2`) and `duel6_quick_wheel_settings` were looked at. Not tried
in a game: whether `force_*` from the wheel and the selection reach a real server
as from the F-keys is unverified.

## Saber tab: hilt search and the blade skins owned

Branch `personal/saber-search` (10/10/2026, Windows 11), Sol's request ("make saber
searchable and let us select unlocked saber shaders"). The SJK UI Profile screen's
Saber tab has a hilt search, a field at the right of the hilt lists' heading drawn as
the Character page's Search, filtering by any part of a hilt's name or file name
(Escape clears it; a kept search clears with Escape before the screen closes; Dual's
two lists show four lines each to leave room), and a Blade row offering the stock blade
and every blade skin the player's hub profile lists as the Collection's swatches
shrunk; picking one sets `cg_saberSkin` through the Collection's code
([sjk-ui.md](sjk-ui.md#character),
[unlockables.md](unlockables.md#saber-tabs-blade-choice)). The player screen's canvas
holds 1,280 draw commands (up to about 650 measured with five skins carrying every
effect). Verified: `cargo test -p sjk-viewer` with tests of the search (typing from the
field, matches, Left and Right, a click, nothing found, Escape, Dual and Staff) and the
blade choice (offered skins, picking, stepping, a pending choice, the canvas), and the
world shot `duel6_sjk_saber_page` (1080p, `ui_textScale` 1.2, 4K with it) with the
hub's pack. Not tried in the game.

## Profile tabs at a larger text size; a shorter Profile line

Branch `personal/tab-fit` (10/10/2026, Windows 11), Sol's report: at `ui_textScale
1.2` the Profile screen's "Achievements" tab was cut, the row being laid out at the
neutral size and drawn larger inside it. The text style is now published each frame
(`text::style::current`) and the row sets each name at its styled width (unit test:
the largest style keeps the seven within the frame; world shot
`duel6_sjk_profile_tabs_styled`). The main page's Profile line reads "Character, Force
and SJK profile", clear of the server column. Not tried in the game.

## The player's own key id off screen; the SJK page without Profile

Branch `personal/key-quiet` (09/10/2026, Windows 11), Sol's answers on the Profile
screen: the player's own key id is no longer printed on the SJK Profile tab, their own
row of the in-game Players cards (SJK and classic), their own sender card or the SJK
chat page's side panel for their own message (`player_identity::printable_key_id`,
unit-tested); other players' key ids still show. The main page's SJK page loses its
Profile entry (What's new, Update, Credits, Back), the main Profile button and the
profile card opening the screen. Not tried in the game.

## Profile screen: one row of tabs, the key in Settings, Browse for the picture

Branch `personal/profile-tabs` (09/10/2026, based on `9d8b9da5`, Windows 11), Sol's
request. The SJK UI's Profile screen has one row of seven tabs where the player
screen's three stood: Character, Saber, Force, SJK Profile, Achievements, Medals (new:
each medal's picture, what it is for, when it was given and the team's note, the ones
not given dimmed) and Collection (the Unlockables page; the name is one constant). The
Identity tab and the separate strip at the top right are gone; the in-game menu's list
is Resume, Profile, Players, Settings, Servers, Leave, and the main page's Character
reads Profile. In the SJK UI the `profile`, `achievements` and `unlockables` commands
open their tabs. The identity's page opens from Settings > Network > SJK identity key,
its key id and file hidden until Show key each time it opens. The picture panel has
Browse..., the system's file dialog (`rfd` 0.17.2, on a worker thread), and the client
refuses empty files, files that are no PNG, JPEG or TGA whatever their name, damaged
or cut-short PNGs and JPEGs, pictures under 64 pixels a side, more than four times as
long as wide or with nothing visible, each with its reason
([sjk-ui.md](sjk-ui.md#profile-screen), [identity.md](identity.md#pictures)).

Verified: `cargo test -p sjk-viewer` (1,549 passed) with tests of the row (drawn and
hit-tested alike in four window shapes, under the title, in the player screen's
place), the Profile page's tabs, buttons and Escape, Browse with a stand-in dialog
(one at a time, the file read as a dropped one), each picture refusal, the hidden key,
and every state of the pages fitting their canvases and staying clear of the row; the
world shots `duel6_sjk_profile_screen` (all seven tabs from the game menu and round,
the tab remembered, the picture panel, 4:3, Character and Collection from the main
page, no canvas overflowing), `duel6_sjk_menu` (the main page's Profile, Settings'
Network), `duel6_sjk_ingame`, `duel6_sjk_profile`, `duel6_sjk_unlockables` and the
`medals_snapshot` Identity pages, hidden and shown. `rfd` was checked to build for
Linux with only pure-Rust dependencies (`cargo check -p rfd --target
x86_64-unknown-linux-gnu`). Not verified: the file dialog was never opened (tests use
a stand-in), so how it stacks over a fullscreen game window (it has no parent window)
is unchecked; nothing was tried in the game.

## Settings' changed dot placed from the drawn name

Branch `personal/changed-dot` (09/10/2026, Windows 11): Sol reported the gold
"changed from default" dot in the SJK UI's Settings sitting on the letters or
well after them. Its place was guessed from the name's character count (0.4 em a
character). It is now set from the name's width as the body font draws it
(`Measure::ink_width`, the player's text size and spacing included), 8 frame
pixels after the last letter, or after the column's end for a name cut short;
Key bindings' rows share the code. A unit test draws names through the renderer
at 1080p, 4K and a styled size and checks the gap; world shots
(`duel6_settings_changed_dots`) show Gameplay and Key bindings before and after.
Not tried in the game.

## Console `forcepowers` changes reach the server

Branch `personal/force-console` (09/10/2026, based on `bc8ac6b`, Windows 11):
a player on a JA+/JoF server (build 2026.1007.3) set Force Jump to level 1,
killed themselves to respawn and kept Jump 3.

Two causes. Builds up to 2026.1008.2 sent the profile fitted at join in every
later userinfo and sent no `forcechanged`, so no profile applied in play reached
the server; PR #47 (`ee1d9c9`, merged in `99b7485`) fixed that for the Force
page's Apply and is in no release yet. A `forcepowers` typed in the console or
set by a config still sent its userinfo but no `forcechanged`, and a server
(`Cmd_ForceChanged_f`) re-reads a changed profile only when told to. Now a
change of the `forcepowers` cvar on a server queues `forcechanged` behind the
userinfo that carries it (`ForceProfileNegotiator::profile_applied` says whether
it queued, found a reply already waiting, or had no server rules), and the
console logs which, with the reason when nothing is sent. A change before a
connection is left to the join, which sends the profile and `forcechanged`.

Unit tests cover the three outcomes, a console change queued once, and Apply not
sending it twice. Not verified: nothing was run against a server or a game, so
whether JA+ or JoF re-reads the profile at the respawn after `forcechanged` (the
server's `forceDoInit`, stock OpenJK) is still unchecked there.

## Saber change without a respawn on JA+ servers

Branch `personal/saber-command` (09/10/2026): a player reported that a saber
changed in the menu applies only after respawning. The cvars `saber1`/`saber2`
are read by stock servers at spawn. JA+ and jaPRO servers also accept a `saber
<saber1> [<saber2>]` client command (jaPRO `Cmd_Saber_f`; JoF EJK's
`CG_Saber_f` sends it when `serverMod >= SVMOD_JAPLUS`). `SaberChangeNotifier`
(`sjk-client/src/saber_command.rs`) now watches the two cvars each frame, so the
menu and a console `set` share one path, and sends the command once the userinfo
has gone out, once per changed selection, at most once a second, only for the
`JaPlus` and `TaystJk` compat profiles. The argument format (one name for a
single saber, two for a pair, `saber2` `none` or empty meaning single) is taken
from those two reference sources; it has not been tried on a live server.
Unit tests cover the command string, the profile gate, the connect baseline,
dedupe, the userinfo and interval holds and the reset on a new server.

## Report dialog: caret and editing

Branch `personal/report-dialog` (09/10/2026, based on `bc8ac6ba`, Windows 11): players
reported that the caret of Report a bug is misplaced, that the letters cannot be gone
back over, that the arrows do nothing and that the buttons cannot be clicked. Read in
code: the text dialog kept only a string (Backspace popped its last character, Left,
Right, Home, End and Delete did nothing), and drew the caret at the end of the last
wrapped line with the font's unstyled width, while the renderer applies `ui_textScale`
and `ui_letterSpacing`, so the caret drifted from the glyphs and a scaled text wrapped
wider than its box (the classic look appended a `|` to the last line). Now the dialog
has an insertion point (`console::line_edit::LineEdit`, the console's editor) and a
layout of the wrapped lines ([text_dialog_field.rs](../crates/sjk-viewer/src/text_dialog_field.rs))
measured with the renderer's own style; Left/Right/Up/Down/Home/End/Delete/Backspace,
Ctrl words, insertion and paste at the caret within the same limits, click to place,
and the arrows walking Send/Cancel are in both looks, which draw a caret bar between
the glyphs. A refused Send was a small gold line under the box; it is a red band by the
buttons, shown after a refused Send and already while Send has the keyboard or the
pointer (Send is dimmed until the text would pass, which read as a dead button).

Verified: unit tests (`text_dialog`, `text_dialog::field`, `text_dialog::sjk`): editing at
the caret, Ctrl words, limits and the character filter at the caret, the arrows over
the buttons, the caret's x equal to the renderer's measure of the line before it for
five text styles and two spacings on every wrapped line, Up/Down/Home/End over wrapped
lines, click placement, the caret bar of the SJK card at 1080 lines and 4K in three
styles and of the classic look, the pointer reaching the classic look's field, Send
and Cancel, and the refusal band before and after the press.

Not verified: no game was started (keys, the OS cursor and a real pointer were not tried);
no click defect was found in code: the pointer routing, the hit regions and the tokens of
both looks are covered by tests, so "cannot be clicked" is attributed to the dimmed Send
with its easy-to-miss reason; if a click really is lost in a running client it needs a
log of that session.

## Vehicle nameplates (09/10/2026)

Branch `personal/vehicle-fixes`: a pilot in a `hideRider` vehicle had no plate, as the
server never sends his entity (`SVF_NOCLIENT`). The plate now follows the vehicle's
`owner` like stock's crosshair name, over the vehicle ([Nameplates](client.md#nameplates)).
Verified by unit tests of the selection and anchor logic only; not seen in a game, and
several riders of one vehicle (passengers) are not named, as stock tells only the pilot.

## Vehicle fire sounds (09/10/2026)

Branch `personal/vehicle-fixes`: a vehicle's `EV_FIRE_WEAPON`/`EV_ALT_FIRE` played an
ordinary weapon's flash sound; stock plays nothing for them (`cg_event.c:2751-2760`,
`:2779-2784`) and the client now matches. Known gap: the `muzzleFX` effect of
`EV_VEH_FIRE` (`CG_VehMuzzleFireFX`), which carries the vehicle weapons' sound in
retail, is not played, so vehicle fire is silent. The retail `.vwp` data was not
available to confirm what each effect holds. Unit-tested only, not heard in a game.

## Vehicle HUD

Branch `personal/vehicle-hud` (09/10/2026, based on `bc8ac6ba`), after the report
"in a vehicle we have no vehicle HUD". The pilot now gets stock's vehicle meters (hull,
shield, speed, ammunition, turbo recharge, weapons-linked mark) and crosshair, and a
vehicle that hides its rider replaces the player's status and weapon HUD, as
`CG_DrawVehicleHud` does; layout and sources are in
[rendering.md](rendering.md#game-data-hud). `Snapshot.vehicle_player` supplies the values (its
`stats[STAT_HEALTH]` is the hull, `stats[STAT_ARMOR]` the shield, `ammo[0..2]`), the
`.veh` file the maxima and `crosshairShader`, and the ride prediction the speed and
turbo time. Unit tests cover the tic and turbo mapping, clamping, visibility, hiding,
the layout bounds at 1080p, 4K (also with `cg_hudScale 1.5`) and ultra-wide, text
resolution, the crosshair rules and the `.veh` parse. An off-screen CPU raster of the
draw list at 1920x1080 showed the layout (not committed). Not run in the client or
against a live vehicle server: that `STAT_HEALTH` and `STAT_ARMOR` of the vehicle's
player state carry what stock reads, the look at 4K, and the third-person `cg_groundHud`
(which still draws while riding) are unverified. Not done: the damage icons, the
no-ammo flash and the weapons-linked sound.

## JoFTemple frame rate and Ultra low

Branch `personal/ultra-low` (09/10/2026, based on `8e2ac60`, Windows 11, RTX 5080,
Vulkan): Sol and another tester lose many frames on `JoFTemple` (`{JoF}TempleV1.pk3`)
and Sol asked why, and for a simple switch, also in First setup, back to the
original game's look.

Measured with a scratch release world shot (not committed) at 3840x2160, the
spawn camera, `SJK_GPU_PHASES`, 200 frames after 400 of warm-up:

| Settings | GPU frame | Wall p50 | Load to first frame |
| --- | --- | --- | --- |
| High, movers still | ~8 ms | 8.4 ms | 23 s |
| High, the 15 `func_bobbing` moving | ~13 ms | 12.3 ms | 23 s |
| Performance, bobbing | 1.15 ms | 1.25 ms | 16 s |
| Ultra low, bobbing | 0.33 ms | 0.77 ms | 9 s |
| `mp/ffa5`, `mp/ffa3`, High | ~3.1 ms | 3.2-3.4 ms | |

What the map asks of real-time lighting: 58,124 area lights (stock maps have a few
hundred to 10,000), 407 inline movers (143 `func_door`, 129 `func_usable`, 15
`func_bobbing`, 11,843 door tiles), 89 floor mirror planes and 174 lightmaps. The
light pass is 4.5-6.6 ms against 0.7 ms on stock maps: hiding every mover takes
about 1.5-2.5 ms off it, since mover surfaces are lit directly rather than from
the lamp cache. Moving bobbers add about 3 ms of floor mirrors and 1 ms of light
pass, as their door tiles are traced again and the cache re-baked. 44,065 of the
lamps are the map's two statues (`models/map_objects/joftemple/shree_statue`,
`aldro_statue`): their shaders draw an opaque base stage over the `$lightmap`
stage, so they show fullbright as in retail and the [self-lit fixture
rule](rendering.md#inferring-fixture-light-from-legacy-materials) makes every
patch of them a lamp. Leaving them out (a scratch test) took the load from 23 to
16 s but the frame only from 12.3 to 11.8 ms, so the lamp count is mostly a load
cost; the per-frame cost is the directly lit receivers and the mirrors.

Built ([client.md](client.md#graphics-quality)): a fifth graphics quality level,
Ultra low, below Performance, which also turns off normal and specular maps and
ambient occlusion (`r_ssao`), each now with a Settings row; the **Ultra low**
switch under Graphics quality in VIDEO and First setup, on while the settings are
Ultra low's, keeping the player's values and putting them back when turned off;
and no lamp extraction at load when sun and sky, light shafts and sun shadows are
all off ([rendering](rendering.md#default-visual-profile)).

Verified: `cargo fmt --all --check`, `cargo build --locked --workspace`, `cargo
test --locked --workspace` and `cargo clippy --locked --workspace --all-targets`;
new unit tests for the level's values, the switch's keep and restore across a
profile restart, the command's `ultralow` and the First setup order; world shots
of First setup in both menu styles show the switch. Not verified: no client was
started on a server, so the switch was not flipped in game and the restart path,
populated matches, other GPUs and the look under Ultra low are Sol's to test.
Follow-ups the same day (4K, RTX 5080, High, JoFTemple's spawn view with the bobbing
platforms moving): statues are no longer self-lit lamps (58,124 -> 14,059 lamps, load
23 -> 16 s); `func_bobbing`, `func_rotating` and `func_pendulum` block no lamp (frame
12.1 -> 7.4 ms); a material over 4,096 patches is cooked from a coarser mask
(`JKLevel1` load 91 -> 28 s); see [rendering](rendering.md#movers-in-lamp-shadows)
and [load-time preparation](rendering.md#load-time-texture-and-light-preparation).
Lighting movers at rest from the lamp cache was built and measured, then dropped: the
light pass stayed at 4.4 ms, and with the lamp cache off altogether it is 4.3 ms, so
the movers were not what this view's light pass spends. Sol's in-game run of a
`perftest` script (`perfmark` steps, `SJK_GPU_PHASES`, RTX 5080 at 4K) showed a 5.7 ms
light pass with every real-time term off, and the pass split into parts put 4.2 ms in
the direct-lamp list: the great hall floor, lightmapped 3.6 times coarser than the lamp
cache's spacing, was lit lamp by lamp. Caching surfaces up to 4 times coarser took the
frame from 7.5 to 3.6 ms, level with `mp/ffa5` and `mp/ffa3`
([rendering](rendering.md#movers-in-lamp-shadows)). Not yet checked in game.

## In-game menu rework and the Profile screen

SJK-only branch `personal/ingame-menu-2` (09/10/2026, Rust stable), Sol's request of
09/10/2026: fewer buttons in the SJK UI's in-game menu. The list is Resume, Profile,
Achievements, Players, Settings, Servers, Leave; Team and Vote moved onto the match
card (the vote on with Yes and No, the player's side with Join red, Join blue and
Spectate, or Join and Spectate, or Siege's Class and side, and Call a vote at its foot);
Camera control, What's new, Credits, Report a bug, SJK chat and, for staff keys, Staff
tools are a row of icons under the emblem; the SJK page is gone. Tab moves the keyboard
list, row, card; Right enters the card. Profile is the new Profile screen: the player
screen, the Profile page and the Identity page as tabs (Ctrl+Tab, or a click on the
strip), also from the main page's Character and SJK > Profile; Achievements opens the
board alone (since 09/10/2026 one row of seven tabs and no Achievements entry: see the
top of this page). Camera control is a quick wheel action and the `cameracontrol` command.
The classic in-game bar is unchanged ([sjk-ui.md](sjk-ui.md#in-game-menu),
[Profile screen](sjk-ui.md#profile-screen)).

Verified: `cargo test --release -p sjk-viewer` (1491 passed; new tests for the entry
order, the card's controls and the commands they send, Tab and arrow moves across the
list, row and card, Staff only for staff, the keyboard settling when a control goes
away, the Profile page's modes, the strip on every Character page and its hit areas,
canvas overflow at 1080p, 4K, 21:9 and 4:3), `cargo clippy --locked --workspace
--all-targets` (no warning on a changed line), `cargo fmt --all --check`, and the
off-screen shots `duel6_sjk_ingame` (no vote, vote on, row focused, side buttons
focused, Staff shown, a spectator in an FFA, Siege, Team, call-vote maps, Leave, 4:3,
Settings), `duel6_sjk_profile_screen` (Character, Profile and Identity tabs switched by
the shot's Ctrl+Tab path, the Achievements board, 4:3, the Character tab from the main
page), `duel6_camera_control`, `duel6_sjk_profile` and `duel6_sjk_character_in_game`,
all looked at. Not verified: anything in a running client or on a server (the side
buttons and votes sending to a real server, Siege's Class and side, Ctrl+Tab and clicks
on the strip through the window's real keyboard and mouse, the main page's camera
moving between the Character tab's stage and the Profile tab, the quick wheel's Camera
control and `cameracontrol` in a match), the classic menus beyond the camera shot's
classic bar, and the tests of the other crates (unchanged; clippy built every target).

## Review fixes for SJK PRs #61, #62 and #64

SJK-only branch `personal/review-fixes` (09/10/2026, Rust stable), from the review
before those three merged. #61: a `.wav` the game data only holds as `.mp3` (retail's
`enemy_saber_on`) was read and registered again on every player config change;
`RegisteredLegacySound` now remembers the path asked for and `intern_sound` matches it.
#62: after the hub URL or identity changed, the pictures that failed to load were
dropped and the later slots moved to other atlas cells without being uploaded again, so a
player could show another's picture (`Avatars::drop_missing` re-uploads them); a
picture the hub serves is decoded under a 256 pixel, 4 MB limit, since a small 8,192
pixel PNG decoded to 256 MB. #64: a reconnect kept the old applied mask, so a muted
player was not rebuilt as the stand-in (`MutedPlayers::clear` resets it); an
`EV_SABER_BLOCK` with no owner (stock sends 0) no longer counts as client 0's, which
silenced every block when slot 0 was muted; a mute list that is not UTF-8 is read
lossily instead of failing and being saved over. Not fixed: a muted slot's old chat
lines come back when another player takes the slot (the lines are judged by slot
number), and each player config change parses the `.sab` files once for the saber
switch sounds. Verified: `cargo fmt --all --check`, `cargo test --locked --workspace`.

## Force page: buttons clear of This server, right click removes a level

SJK-only branch `personal/force-page-layout` (09/10/2026, Rust stable), from Sol's
report that the Force page's buttons sat on the "This server" text. The three
buttons ran across the whole page, over the right column where the server panel's
lines grow (a long off list, the notes). Apply now spans the left column under the
Lightsaber group with Start over and Discard side by side below it, and the panel's
lines wrap at 42 characters, not 36. A click on a level only buys up to it; a right
click on a level removes it (the power stands one below it, nothing happens on a
mark above the power's level), replacing the click on the power's own level that
dropped one. Checked with the off-screen shots: `duel6_player_sjk` now also writes
`duel6-player-sjk-force-server.png` (the page on a server with eight powers off).
Verified: `cargo fmt --all --check`, `cargo test --locked --workspace`, the shots. Not
tried in a running client.
## New medal pop-up: SJK UI look, ceremony, fanfare and `debug_medal`

Branch `feat/medal-award` (08/10/2026, based on `a6230f9`, Linux): Sol asked for the
new medal pop-up's Next button in the SJK UI's style, for medals to arrive animated,
a bit like the achievements, with a sound, and for a `debug_` command to fake
receiving them. Built ([identity.md](identity.md#medals),
[sjk-ui.md](sjk-ui.md#new-medal)): the pop-up was drawn on the retired modern canvas,
its Next button that style's last user (`ButtonStyle` and `MenuCanvas::button_styled`
are removed). With the SJK UI's menus it is now the SJK UI's: no card, the medal
floating over the scene darkened by the UI's navy, the words in its families, the
kit's gold button (Next, Close on the last) and the key bottom right; with the
classic menus, the classic+ pop-up box sized to what it holds, NEXT or CLOSE in
retail's gold with its glow and the description line. Each medal arrives in a
ceremony both looks share (`medal_popup/award.rs`, draw-list shapes of a fixed
number): it comes down growing into place with a little overshoot, light bursts from
the medallion (glow, two gold rings, a flash, twenty falling sparks), a gold arc
sweeps a ring with turning ticks, a band of light crosses the medal and the words
fade up one group after another, all in 1.8 s; waiting, it breathes, glints softly
every 6 s and a few sparkles twinkle. A press in the entrance finishes it; then it
takes the button and the medal lifts away in 0.32 s. Each arrival plays the
multiplayer game's Jedi Master fanfare, `music/goodsmall.mp3`, as an interface cue
(silent without the file); a key on the button plays the menus' click, as a mouse
click does. `debug_medal <id> [x<count>] [note]`, `debug_medal all [note]` and
`debug_medal` alone (the ids) queue made-up medals through the same queue, centre
print, pop-up, ceremony and sound; nothing is sent and `medals_seen.txt` is never
written for them. The client has no reduced-motion setting to follow.

Verified: unit tests of the timeline (the landing's overshoot, standing still at
1.8 s, the words in order, the burst, sparks and band of light over before then, the
breath and soft glints bounded and periodic, the exit), a bounded draw count at every
moment, the band of light lying on the medal brighter than white, the queue with keys
and pointer (a press in the entrance finishes it silently, the next takes the button
with a click, one fanfare per medal, the wait for its picture), rehearsals never
written as seen and replaced by a real medal, the command's parsing and listing. For
both looks, every medal with no note and the longest notes (running text, the widest
letters), alone and first of several, at nine moments, fits the canvas's draw and text
budgets at 1080p, 4K, 4:3, 21:9 and 720 lines in the families and Inter, the button
inside the window (above its key; inside its box in classic+); the SJK UI's button is
the kit's gold pill, its pointer area exactly the pill, and a click there or elsewhere
acts. CPU renders of both looks at several moments over a plain backdrop (the snapshot
rasterizer, through a test not committed) were looked at; they led to the stronger
band of light, the ticks outside the gold ring, a smoother glow and the classic box
fitted to its content. On Linux with Rust 1.97: formatting, the locked workspace
build, the locked workspace tests and workspace Clippy (no warning in the changed
code) pass.

Not seen or heard: there is no GPU, display, audio device, game data or hub here, so
the ceremony's motion at speed, the band of light on the GPU (it relies on the UI
shader leaving a tint past white unclamped, as it does), `music/goodsmall.mp3` in a
player's install and its loudness, and the flow in a running client are unchecked;
`menu_snapshot::medals_snapshot` and `world_shot::tests::duel6_medal_popup` were
updated for both looks and the moments but not run. To try it: on the main menu,
`debug_medal all Thank you from the SJK team` in the console (it closes, and the four
medals show one after another), or `debug_medal bug_hunter x3` in a match, then open
the game menu.
## More blade skins, and a round tip for them

Branch `feat/more-blade-skins` (09/10/2026, based on `personal/blade-skins`, Linux), with
the hub's `feat/more-blade-skins`: Sol found the Sun blade's tip square and asked for it
to be fixed, then for more blade skins, an electric one first
([unlockables.md](unlockables.md)).

- The square tip: the core line is a flat quad from behind the hilt to the tip
  (`saber.wgsl` `vertex_main`, as `RB_SurfaceLine`), and the generated core's fringe
  is still about a fifth of its brightness at the quad's end and sides, so its end and
  corners showed as a bright square: plainest with the Sun's wide, bright fringe over a
  tip glow half as bright as the shaft's (stock blades end the same way, their white
  core hidden in their glow). Past the tip, too, the corona's grading and flame tongues
  were taken from the distance across only (`out = abs(x)`), so they ran on straight as
  a column, and a widening corona widened only sideways. Fixed in the generic shading,
  for skins only: the core narrows on a quarter circle over its last `core.tip`
  half-widths (default 1.5) to a rounded point, cut softly at that edge; past the tip
  the corona widens round the tip, its distance out is taken from the tip and its
  tongues run on round it. Along the shaft nothing changed; retail and RGB blades are
  untouched.
- New generic, optional sections of the blade-skin file, all drawing nothing when
  absent (so the Sun draws as before but for its tip): lightning `arcs` (up to 4: struck
  at random places, bulging off the blade or leaping from the tip, jagged, re-shaped,
  crawling and fading), drifting `motes` (sparks, specks, shards in a field of cells
  round the blade, drifting along, outward or inward, twinkling) and a turning `hue`
  (by time, along and out; the light turns with it). The uniform grows from 21 to 31
  `vec4`s a skin (3968 bytes for the 8 skins), still written only when skins load; the
  loops are bounded (4 arcs, one mote cell a fragment).
- New skins (the hub's catalogue and pack, and this client's catalogue):
  `saber_storm` (Storm: white-blue core, lightning arcs, sparks at the tip, electric
  buzz), `saber_void` (Void: hollow black core in a violet rim, star specks drifting in,
  low drone), `saber_frost` (Frost: ice core with a pointed tip, crystalline veins,
  trailing shards, chiming hum) and `saber_prism` (Prism: a rainbow running along the
  blade and turning, a chord hum). Their looks and sounds are in the hub's private
  repository only.
- The Unlockables page shows two cards at a time and scrolls (to the card chosen, and
  with the mouse wheel) with a scroll bar and a line saying which show; the Staff page's
  unlockable rows are 42 pixels so all five fit; the swatches draw arcs, motes and hue.

Verified (Ubuntu 24.04, Rust 1.97, no GPU): `cargo fmt --all --check`,
`cargo build --locked --workspace`, `cargo test --locked --workspace` (1668 passed,
viewer 1263, 60 ignored, none failed) and
`cargo clippy --locked --workspace --all-targets` (no warning on a changed line), each
with the shared target directory's per-package profile settings so other worktrees'
builds could not stand in; unit tests: the format's new fields, ranges, refusals and
absent sections, the uniform's lanes and its 31-`vec4` layout matched by name against
`saber.wgsl`, a CPU copy of the tip's maths (shaft unchanged, a quarter circle, the
square corners cut, the glow round past the tip) and of the hue turn, both checked to be
what the shader holds, `saber.wgsl` validated by naga, the swatch's arcs, motes and hue
inside its frame, the page's scrolling, every Staff row, and the catalogue pinned to the
same list as the hub's; the hub's built pack, given through `SJK_TEST_PACKS`, read by the
strict parser with every sound found (an ignored test). The looks were judged only from
images of a CPU copy of the skin shading (side on, no world), the sounds only by their
levels and spectra. Not seen on a GPU (no world shot: this machine has none), not heard,
not measured for frame time, not tried with a real hub. Deploy and merge order: the
hub's commit first (its pack and catalogue are safe for clients without this change:
the Sun's file is unchanged, and such a client leaves out the four new files, logging a
line for each, and does not know their ids), then PR #45, then this branch.
## Saber ignition on a weapon switch

Branch `fix/saber-switch-sounds` (08/10/2026, based on `a6230f9`, Linux): Sol
reported that a lightsaber turned on made no ignition sound. The server's own
ignition sounds (the toggle's sound event, and `EV_SABER_UNHOLSTER` when an attack
ignites the saber) already played. What SJK never played was cgame's own sound when
a player draws the saber from another weapon or puts a lit one away: OpenJK's
`CG_CheckPlayerG2Weapons` and `CG_Player` play each hilt's `soundOn` or `soundOff`
on that switch, and the server sends nothing for it (`EV_CHANGE_WEAPON` has no
saber select sound). SJK now tracks the weapon bolted to each player as cgame does
and plays those sounds with cgame's quirks, and `EV_SABER_UNHOLSTER` plays the
player's own hilts instead of the stock `saberon`
([client.md](client.md#saber-ignition-and-retraction-sounds)). Merged with the
blade skins (`ba17416`): a client wearing one hears its skin's single ignition or
switching-off sound on these switches and on `EV_SABER_UNHOLSTER`; each hilt's
`soundOn`/`soundOff`, then their defaults, play for every client. Since 09/10/2026
(`personal/saber-skin-layers`) a skin's sounds (ignition, switching off, swings, hum,
a thrown saber's hum) play over the stock ones instead of replacing them: the skin's
sound is added on a channel of its own, so the stock sound is not cut.

Verified on Linux: unit tests pin the hilt sounds (`notInMP`, a removed second
saber, one dropped beside a two-handed staff), the local switch from fists or a gun
(each hilt once, `CHAN_AUTO`, the listener's own entity), retraction of a lit saber
only, remote dual and staff switches from the player's position, and silence for a
player coming into view, a caught or knocked-away saber, the dead, and the server's
toggle and unholster events. They also pin one ignition per hilt when
`EV_CHANGE_WEAPON` arrives, the reset on a new map and on a reused client slot, a
new hilt, and a followed player voiced once. Breaking three of the rules on purpose
fails four of these tests. Two more pin the blade skin first: one sound per switch
and per unholster for a wearer (dual sabers included), the hilts again once it is
taken off, and a server toggle the skin replaces with nothing added by the switch
path; dropping the skin from the switch path fails them and two of the skins' own
tests. Workspace formatting, the locked build, tests and Clippy
pass, with no new warnings. Not heard: nothing was run in a game or with retail
assets. Whether retail ships `enemy_saber_on`, the second ignition sound of a single
saber, and the timing of the predicted local switch against a live server are
unchecked.
## Muting a player from a name in chat

Branch `feat/sjk-chat-mute` (08/10/2026, based on `feat/sjk-chat-look`, Linux): Sol
asked that resting the pointer on a player's name in chat show their profile with Mute
or Unmute, and that a mute hide their chat (SJK chat and, when they can be matched to a
slot, game chat), draw them as `kyle/default` with the default saber and colour while
they are on the server, and silence every sound they cause, on this PC only. A sender
card ([sender_card.rs](../crates/sjk-viewer/src/sender_card.rs)) now shows beside a
name under the pointer in the game's chat while the composer is open, on the main
page's dock and on the SJK chat page; the Players page's card is that page's right
column and could not be reused. The page's session-only mute became one list kept in
`chat-mutes.txt` beside `config.cfg` (key, or `-`, and the last name seen), which the
card, the page and the dock share. Muted slots are matched as the scoreboard's badges
are, by the hub's claim under the name shown, else by name with colour codes and
symbols ignored (the card says "matched by name"); a game line the server does not
attribute (JA+) is hidden when it starts with a muted slot's name. The slots are a bit
mask worked out only when the list, the hub's claims or the players change; the
renderer swaps the model, sabers, blade colour and hat or cape, and the sound filter
tests the source entity and, for saber hits and blocks, voice commands and chat beeps,
the cause the sound events now name (`LegacySoundDecision::cause`)
([hub-chat.md](hub-chat.md#muting-a-player)). Merged with main (`bde59f7`, through
`feat/sjk-chat-look`), a muted player's look goes too: no blade skin, so none of its
sounds, and no lit Illuminate holocron (`Looks::set_muted`). Stacked on
`feat/sjk-profile-card` too (#62), the hover card is the sender card (`sender_card.rs`,
#62's `profile_card.rs` being the player's own card), shows the sender's picture
through #62's picture cache (the version from the hub's players on the server, else
their profile, fetched once), and its dock targets moved off #62's card (names 60 to
64, card 70, Mute 71).

Verified on Linux with Rust 1.97: `cargo fmt --all --check`, `cargo build --locked
--workspace`, `cargo test --locked --workspace` (1664 passed, 51 ignored) and `cargo
clippy --locked --workspace --all-targets` (exit 0, no warning in the changed code).
New unit tests: the mute list (by key and by name, colours ignored; a newer name kept;
its file written and read back, bad lines left out); slot matching (a claim under the
shown name, another key's claim never matched by name, the own slot never muted,
locating a player by claim then by name); the slots worked out only when their inputs
change; the model (Kyle, a team's red or blue kept), sabers (`single_1` in each hand)
and blade colour of a muted player; the sound filter (their entity, a sound naming them
as its cause, a saber they threw, nobody else) and the causes the sound events name
(saber hit, block, voice command, a chat beep's sender); a muted slot's game lines
hidden and shown again, a JA+ line by its leading name; hovering a name in the game's
feed, on the dock and on the page shows the card, which stays while the pointer is on
it, and Mute asks for that player once, then offers Unmute. Merged with main, the same
four checks pass (1836 passed, 59 ignored), with a test that a muted player's blade skin
and holocron are not drawn while anyone else's and the own slot's are. Stacked on
`feat/sjk-profile-card` too, they pass again (1871 passed, 61 ignored), with tests that
the sender card keeps clear of the profile card in every window, its picture disc sits
before the name and its picture's version is asked until known. Not verified: no client was
started (no GPU, display, hub or game data here), so the card, Kyle and the silence
were not seen or heard in a game, nor with players matched through a real hub.

## SJK chat in its own gold, on one line, with the tick alone

Branch `feat/sjk-chat-look` (08/10/2026, based on `a6230f9`, Linux): Sol asked for
SJK chat text in a special gold that is none of the game's colour codes, for an SJK
chat line not to break straight after the name and its badge, and for a verified
player to show only the verified tick by the name. The message is now drawn in
`#F5C756` (`sjk_chat_look::GOLD`, [sjk_chat_look.rs](../crates/sjk-viewer/src/sjk_chat_look.rs))
in the game's chat, on the main page's dock and on the SJK chat page, its own colour
codes dropped; names keep theirs. The line broke at once because the game's chat
feed drew a sender's name and tag on a row of their own with the message under it
(`NAME_ADVANCE`, `chat/view.rs`), unlike the servers' chat lines, which carry the
name in their text and flow; an SJK line now starts its first row with the SJK tag,
the name, the tick and a colon and goes on there (`Wrapped::update_indented`,
`chat/view/sjk_line.rs`), wrapping only when it is too long, and the page lays its
lines out the same way. "SJK VERIFIED", "Verified" and the dock's gold dot gave way
to the nameplates' verified seal after the name
([hub-chat.md](hub-chat.md#how-a-line-looks)).

Verified on Linux with Rust 1.97: `cargo fmt --all --check`, `cargo build --locked
--workspace`, `cargo test --locked --workspace` (1643 passed, 51 ignored) and `cargo
clippy --locked --workspace --all-targets` (exit 0, no warning in the changed code).
New unit tests: the gold is far from every colour code in both palettes and from the
SJK UI's accent; a message's codes are dropped; only SJK messages are drawn gold (game
lines and the name are not); a short SJK line is one row with tag, name, tick, colon
and message left to right on it; a long one wraps into as many rows as the same text
in a game line, its later rows from the edge; a first word too wide for the name's row
starts the next; a verified sender has the seal and no "verified" text; the dock's row
and the page's line flow the same way. Not verified: no client was started (no GPU,
display, hub or game data here), so nothing was seen on screen, in the retail fonts
(`ui_gameFont`) or over a map, and no hub message was received.

## SJK profile card and players' pictures

Branch `feat/sjk-profile-card` (08/10/2026, based on `a6230f9`, Linux). Sol asked
for the name and circle at the bottom left of the main menu to show the player's SJK
profile, for a small square picture players upload as theirs, for the same in the
in-game Escape menu, and for a click on it to open the profile and achievements.

- The SJK UI's main page and in-game menu show a profile card bottom left
  ([sjk-ui.md](sjk-ui.md#profile-card)) in place of the name and circle: the
  player's picture (or their initial on a colour from their key), the name in its
  colours, the verified badge, a line such as "Verified · 2 medals · 12/21
  achievements" (or why the hub is not answering) and their model and blade. A click
  (on the main page also Left, then Enter) opens the Profile page. The classic menus
  have no such corner and keep their layout; their SJK page opens the Profile page.
- Pictures ([identity.md](identity.md#pictures)): the Profile page's picture opens a
  picture panel. A picture file dropped on the window, or `sjkavatar <file>`, is read,
  cropped to a square from its middle and scaled to 128 pixels on a worker thread,
  previewed, and sent with Use this picture as a signed `PUT /v1/avatar`;
  `sjkavatar clear` or Remove picture takes it down. Other players' pictures show on
  the in-game Players page's card and the Staff page (where staff have Take picture
  down); they are fetched by version on a worker, decoded there, kept in memory (31 in
  the UI atlas) and in `avatars/` in the settings folder (256 files, 8 MB at most).
- sjk-identity gained the `avatar` module, `avatar` in Profile and Presence, the
  upload, removal and download requests, `Service::set_avatar`/`remove_avatar` with
  `Snapshot::avatar`, and the staff requests to take a picture down or stop a key's
  uploads.
- Merged with `main` at `bde59f7` (unlockables and looks, #45): the card's last
  line names the blade skin worn while the profile holds it ("Kyle, Sun blade"), the
  Profile page's Tab order runs picture, Identity settings, bio or picture panel, See
  the board, See unlockables, and the Staff page keeps Take picture down in its header
  above the medals and unlockables.

It needs the hub's branch `feat/avatars` (Sol-Vulpes/SJK-hub) deployed first. Against
a hub without it, profiles carry no picture (the stand-ins show) and an upload says
the hub does not take pictures yet.

Verified on Linux with Rust 1.97: workspace formatting, the locked build, the locked
tests and Clippy (no warning on a changed line). Unit tests pin the crop, scale and
PNG encoding (wide, tall, JPEG, TGA, transparency, refusals with their reasons, a
header claiming a huge picture), the round cut, the served picture's checks, the
memory cache (asked once, versions, the renderer that holds it, the bound that keeps
what is on screen, retries), the folder (names, versions, file and byte bounds), the
card's line and stand-in colours, the card fitting its corner clear of the page's
other text at 1080 lines, 4K, 4:3 and 21:9 on the main page and on every
in-game page with room for it, its pointer and keys, the picture panel's layout and
its drop, preview, send, refusal, removal and offline flows, where a dropped file
goes, the command's words, the staff page's Take picture down, and the service
sending a picture only once registered. Against a local `sjk-hub` built from
`feat/avatars` (a fresh database, a few tests per start because of its registration
limit), the ignored end-to-end tests passed: a picture sent, read back by its version,
refused when not a PNG or too small, and taken down (`hub_e2e`); the viewer's worker
downloading a JPEG made ready as the Profile page does and keeping it in its folder;
and the other hub tests but the staff one (it needs a staff key's seed). The service's
chat test passes only once the hub's chat holds a message: a fresh hub holds the first
poll 25 seconds, longer than the test waits, as before this change.

Not verified: nothing was seen on screen (no GPU or display here), so the card, the
picture panel and the atlas upload of pictures were never looked at; dropping a file
on a real window and the production hub were not tried.

## Force profile and kill feed follow-ups

SJK-only branch `personal/force-profile-fixes` (09/10/2026, Rust stable), from the
review of SJK PRs #47 and #58. A `CS_SERVERINFO` change no longer drops the side an
`nfr` notice held the player to under `g_forceBasedTeams`
(`ForceProfileNegotiator::refresh_server_rules`): the next userinfo would have sent
the other side's profile and the server parked the player again. The nameplate's
own powers are now what the server grants the profile that is sent, not the player's
own: a profile over the server's points only through disabled powers has other
powers trimmed in the sent one (`own_force_allocation`). The kill feed tells Force
Grip from Force Lightning (`MOD_FORCE_DARK` covers both) by the killer's
`forcePowersActive` bits in the snapshot of the kill (`ObituaryEvent::attacker_force`):
Grip's holocron when only grip was active, else Lightning's. Verified: `cargo fmt --all
--check`, `cargo test --locked --workspace` (new tests: a serverinfo refresh keeping
the side, the nameplate following the sent profile, the grip holocron). Not seen in
a game: grip kills in the feed, and `g_forceBasedTeams` servers. Unknown: whether the
attacker's grip bit is still set in the snapshot that carries the obituary when the
grip ends with the kill.

## Unlockables: the Sun blade, and looks through the hub

SJK-only branch `personal/saber-skins` (08/10/2026, stacked on `personal/sjk-chat`,
Ubuntu 24.04, Rust 1.99) with the hub's `feat/unlocks-looks` (stacked on
`feat/chat-emotes`): Sol asked for unlockable saber blade skins, a golden/orange sun
blade with flares and its own sounds first, seen by everyone, and for others to see
Illuminate too ([unlockables.md](unlockables.md)). Built: the hub keeps unlocks per key
(operator and staff grant them) and a look (blade skin, Illuminate lit) on each live
claim, lists it in presence and sends changes through the feed; the client sends its
own look, gated by its unlocks, keeps every slot's look under the badges' name rule,
draws the Sun blade (an animated material in `saber.wgsl`, amber trail, warm
flickering light) and plays its synthesized sounds (ignition, off, hum, three swings,
also for a thrown saber) for every player wearing it, puts a holocron by every other
lit player's shoulder (light for the 4 nearest), and has the Unlockables page
(Profile, `unlockables`), `saberskin` and Unlock/Relock on the Staff page. The feed now
reads on a game server with `cl_sjkChat 0` (chat stays hidden).

The same day Sol asked for the skin's art to stay SJK's own: the client's renderer is
now generic and data-driven (a `skins/blades/<id>.bladeskin` JSON file per skin holds
every shading parameter, the trail, light, flicker and sound paths; up to 8 skins in a
uniform array, rebuilt only when skins load), and the Sun's file and sounds moved to a
pack in the hub's private repository (`assets/packs/sjk_skins`), served by the hub
(`GET /v1/assets`, `/v1/assets/<name>`) under "all rights reserved". The identity
service downloads changed packs into `assets/` beside `identity.key` (after
registering, then every 6 hours; size and SHA-256 checked, written in one rename,
backing off from a minute to 6 hours); the viewer mounts every cached pack at start
(identity on or off) and a new one at once, loading its skins and registering their
sounds mid-session. Nothing of the Sun's look or sounds remains in this repository
(the WAVs, `scripts/saber_skin_sounds.py` and the shader's constants are gone; the
Unlockables swatch draws from the loaded file, or says the look downloads from the
hub). Verified: `cargo fmt --all --check`, `cargo build --locked --workspace`,
`cargo test --locked --workspace` (1659 passed, viewer 1254, 59 ignored, none failed)
and `cargo clippy --locked --workspace --all-targets` (no warning on a changed line);
unit tests against a made-up test skin (format, ranges and errors, the
uniform's layout, loading from a pack, a runtime mount, flicker, mid-session sound
registration, the neutral pair's bytes unchanged), `saber.wgsl` validated by naga, the
identity service's pack tests with a fake hub (only changed packs downloaded, a
mismatch refused and not written, the size cap, atomic write, backoff, nothing while
off) and its `hub_e2e` assets tests against the hub's assets work built on this PC;
the `duel6_sun_blade` and Unlockables world shots with a pack zipped from the hub's
sources (`SJK_TEST_PACKS`), compared with shots of the built-in Sun rendered from the
previous commit in the same session: the blades are pixel-identical (the only
differences are isolated single pixels in the air and on the floor, such as also differ
between two runs of the old build; none on a blade), and the swatch keeps its colours (its flare period now follows the skin's
flare rate, 2.63 s instead of 2.6). Not verified: a download from the deployed hub,
a pack arriving during a real match, Windows (replacing a pack the viewer holds open).

Verified: `cargo fmt --all --check`, `cargo build --locked --workspace`,
`cargo test --locked --workspace` (viewer 1241 passed, 44 ignored; every crate passed)
and `cargo clippy --locked --workspace --all-targets` (no warning on a changed line);
unit tests for the wire, the look worker (coalescing, resend on a new claim, refusals),
the feed with chat off, the looks table, the gating, the per-client sound overrides,
the skin material and its instance layout, the bundled sounds (format, loop point) and
the page; `hub_e2e` against the new hub built and run on this PC (looks, chat off,
staff unlock and relock); the hub's own suite (101 unit, 69 API, 3 command line). World
shots looked at: `duel6_sun_blade` (beside stock orange and blue, close-ups over time,
held on the Character page), `other_players_holocrons_on_duel6`, the Unlockables page
(owned, locked, identity off, 4:3), Profile and Staff. Not verified: nothing ran
against the deployed hub (it has neither chat nor looks yet) or with other players in a
match; the sounds were not listened to; follow mode with real players is untested (a
review fixed the followed player wearing the spectator's skin, looks lost to a 429 or
a failed claim, looks read across a server change, and a relocked skin lingering on
its owner's screen, each with a test); giving a saber on/off sound to the nearest
skinned player within 64 units is a guess, as the game sends it with no owner.
## Clicks on the SJK UI's Character screen

Branch `fix/sjk-character-clicks` (08/10/2026, based on `a6230f9`, Linux). Sol
reported that clicks on the Character screen (saber colours and other controls)
landed beside where they were aimed. The SJK UI's rows give their token the
control's own rectangle, but its ‹ › controls and blade chips were hit-tested as
on the older form, by a value zone in the right 48% of the control: a click just
right of a ‹ ›'s middle stepped back, a chip click took the chip to its left (and
any click on the left half the first chip), and a click on a Force power's name
stepped the power instead of only choosing its row. They now follow the control
as drawn ([sjk-ui.md](sjk-ui.md)).

Verified on Linux: unit tests of the halves and chips, and pointer clicks on the
drawn SJK UI (every blade chip at its centre and edges, a power's holocron and
name after buying a level), which fail without the fix; formatting, the locked
workspace build and tests pass, and workspace Clippy finishes without errors and
with no warning on a changed line. Not seen in a running client or on screen.
## A sound device that goes away no longer freezes the game

Branch `fix/alt-tab-hang` (08/10/2026, based on `a6230f9`, Linux): Sol reported that
after alt-tabbing away for a long while (AFK a few times), alt-tabbing back did
nothing: the window never came back and the game had to be killed; short alt-tabs
were fine. The likely cause is the sound output, not the window. The render thread
pushed every sound command into the mixer's queue and, while it was full, waited for
the audio callback to make room, without limit (`AudioOutput::send`). Windows ends an
output stream for good when its device goes away (cpal's WASAPI thread returns on
`AUDCLNT_E_DEVICE_INVALIDATED` and the callback is never called again): a headset
or Bluetooth speaker switching itself off after minutes of silence, which a window
muted in the background by `snd_mute_losefocus` plays, or a monitor's speakers while
the display sleeps. The 8,192-command queue then fills within seconds (gains,
listener and loops each frame, a position for each entity of each snapshot) and the
game stops answering, which shows only at the Alt+Tab back. Time away matters
because the device has to go away first. A full queue now waits at most 250 ms, once;
then commands are dropped (each frame sends its state again) until the queue drains,
and the log says `audio output stopped taking sound`. The decode worker keeps sounds
in order and still waits for room, but stops when the output is dropped, so
`snd_restart` (which opens the output again) and quitting do not hang on it either
([client.md](client.md#configuration-and-content)).

Verified on Linux: unit tests of a queue nothing drains (one bounded wait, then
every push dropped at once even with an hour's patience; room again ends the stall;
a slow callback is still waited for; the decoder waits until the output closes), of
an output whose callback stopped receiving four queues of commands, and of dropping
an output whose decoder waits; the two output tests time out with the old waits.
Formatting, the locked build, tests and Clippy (no warning in the changed code)
pass. Not verified: nothing was reproduced on Windows, and that Sol's device went
away while he was AFK is inferred, not seen in a log. To check it, in a match switch
off or unplug the headset, or disable the playback device (Settings > System >
Sound), focused or not: the previous release should freeze within seconds; this one
should go on silently, log the line, and `snd_restart` should bring the sound back
once a device is there. If the freeze
remains with sound working, the window side is next and unchanged here: where Vulkan
reports a minimised window's swapchain out of date, every frame reconfigures it at
the old size (`Resized` to 0×0 is ignored), waiting for the GPU each time, and a
redraw request that never arrives leaves the event loop polling instead of sleeping.
## Monitor refresh-rate detection off by default

Branch `feat/monitor-rate-cvar` (08/10/2026, based on `a6230f9`, Linux): Sol asked
for a cvar to turn off the detection of the monitor's refresh rate, off by default.
`com_maxfps -1` (AUTO, the default) capped frames at the refresh rate of the
monitor holding the window, re-read once a second, or at 125 when the monitor
reported none ([runtime_settings.rs](../crates/sjk-viewer/src/runtime_settings.rs)).
`com_maxfpsMonitor` (archived, default 0; Settings > Video > Detect refresh rate)
now decides: at 0 the monitor's rate is never read and AUTO caps at 125, as for a
monitor that reports none; at 1 AUTO follows the monitor as before. The frame loop
reads it from a change-callback cache, without a name lookup. A `com_maxfps` the
player set is left alone, and nothing is migrated: the cvar is new, so no profile
has saved it and the default reaches existing profiles
([client.md](client.md#configuration-and-content)).

Verified on Linux with Rust 1.97: unit tests pin the default (0, AUTO at 125
without reading the monitor, also without a console), the monitor's rate in whole
hertz and the 125 fallback when on, the setting saved and loaded, and a cap the
player set (0 to 1000, and 144 from a profile saved before the cvar) kept either
way; formatting, the locked workspace build and tests pass, and workspace Clippy
reports no warning on a changed line. Not verified: no client was run (no GPU,
display or game data), so neither the cap in either state nor the new Settings row
was seen on screen.
## Low frame rate for a while after a map load

Branch `perf/after-load-stalls` (08/10/2026, based on `a6230f9`, Linux). Sol reported
that after a map load the frame rate sometimes sits around 10 FPS for a while before
recovering, and asked what else could hog frame time. Read from the code (no GPU or
game data here), the first seconds of a map stack several one-off costs on the render
thread and the GPU, in this order of likely weight:

1. **Pipelines compiled mid-frame.** In real-time lighting (on devices with binding
   arrays) the static world draws through the stage table, whose program and pipelines
   were compiled on first draw (`stage_table.rs` `Table::pipeline`), as were the lamp
   cache's receiver pipelines (`light_receivers.rs` `cached`) and depth priming: the
   first frame compiled one pipeline per visible key, later frames more as surfaces
   came into view. A cold driver cache (after an SJK or driver update) costs about half
   a second per program. Still compiled in a frame: glow variants (`world_glow.rs`), the
   lamp cache's bake pipelines, clouds and weather (`weather.rs`), and models loaded
   mid-match.
2. **Players' models on the render thread** (`config_string_refresh.rs`,
   `clientinfo_refresh.rs`): every changed `CS_PLAYERS` or `CS_MODELS` string of a frame
   loads its model there (files, textures, materials, pipelines), and each load
   regrows the shared geometry buffers by copying all of them and waits for the frame
   in flight (`shared_geometry.rs`, `FrameQueue::submit`). Team, model and saber
   changes after a map change depend on the server, one reason it happens sometimes.
3. **Lamp cache bake** (`lamp_cache.rs` `bake_once`): every layer of the cache (up to
   512 MiB), each texel evaluating its lamps, in the first lit frame: a one-frame GPU
   burst, after its bake pipelines are compiled in that frame.
4. **Movers in lamp shadows** (`mover_occlusion_gpu.rs`): the first snapshot queues
   every door tile, so their cache regions are baked again over the next frames (four
   layers every 100 ms, then every frame), each with full-layer steep and rim passes.
5. **Reflection probes** (maps with specular maps, `reflection_capture.rs`): a whole
   probe, six scene renders with their light passes, per frame for up to 64 frames.
6. **GI probes** (`gi_probes.rs`): three passes over every probe are queued at
   installation, then the first frame with the far cascade relights 8,192 probes a
   frame (32 times the steady 256) until all are done.

In steady play: floor mirrors (up to six scene renders), every map video decoded each
frame, lamp cache re-bakes every 100 ms while doors and lifts near lamps move, and GI
and reflection refreshes while the day clock runs.

Changed: map installation, on its own thread, now also compiles the stage table's
program and pipelines (both lighting variants of every stage it holds, static-world or
entity as drawn), the depth-priming pipelines and, on maps with a lamp cache, the
cached receiver pipelines ([rendering](rendering.md#pipelines-compiled-at-load)). The
pipelines are the ones the frames would have created, so the picture is unchanged; the
load takes longer by those compiles instead. Nothing else changed: items 2 to 6 alter
what the first frames show or need restructuring, and want a GPU measurement first.

Verified on Linux: unit tests of which table pipelines are compiled (static and model
stages, off-table stages, movers and flares, shared keys); workspace formatting, the
locked build, tests and Clippy (no warnings on changed lines). Not measured: no GPU,
game or timing run, so neither the stall nor the gain has been seen. To measure, in a
release build on the same map and server before and after: `SJK_FRAME_BUDGET=1`
prints a `frame-budget` line every half second (mean, p99, max, worst frame's phases)
and, with timestamp support, `gpu-phases` every 32 frames; the load log's `[+ms]`
lines (`compiled N pipeline keys`, `Stage table: compiled N pipelines at load`,
`Light pass: ... compiled at load`, `Lamp light cache: bake encoded`, `GI probes
converged`, `Reflection probes`, `Mover occlusion`, `client N now wears`) place each
cost. Comparisons: `SJK_LAMP_CACHE=0`, `SJK_MOVER_OCCLUSION=0`, `SJK_STAGE_TABLE=0`,
`r_cubeMapping 0` (restart) and `r_clouds 0`.
## Door start sounds

Branch `fix/start-sounds` (08/10/2026, based on `a6230f9`, Linux): Sol reported that
a door's first sound is not heard when it opens, and the same for a lightsaber's
ignition. A door is a brush model whose entity origin is the world origin unless the
map gave it an origin brush. In codemp its start and end sounds (`EV_PLAYDOORSOUND`,
`S_StartSound` without an origin) come from the middle of its model
(`CG_SetEntitySoundPosition`); SJK played them at the entity origin, on most maps
too far away to be heard, while the door's loop already came from its middle, and the
mixer moved the door's sounds back to that origin at every snapshot. The sound
adapter now places a brush entity's sounds at its origin plus its inline model's
midpoint, taken from the map when the sound tables are built, and the per-snapshot
source positions use the same point ([client.md](client.md#door-and-mover-sounds)).

The lightsaber was not found to share the cause. The ignition sounds a server sends,
the toggle's `EV_GENERAL_SOUND` at the player and `EV_SABER_UNHOLSTER` when an attack
ignites the saber (also predicted locally), are resolved and played at the player:
traced in the code, and a throwaway sound-adapter test produced the toggle's sound
there. The one ignition SJK does not play is cgame's own, when a player switches to
the saber from another weapon such as melee (`CG_CheckPlayerG2Weapons`, `CG_Player`:
the saber's `soundOn`, and `soundOff` when switching away); it is left for a separate
change, as nothing shows yet that it is the case Sol heard.

Verified on Linux: new unit tests place a door's start and end sounds at the middle of
its model (both came from the world origin before the change) and add a model's
midpoint only for brush models. Formatting, the locked workspace build and tests pass,
and workspace Clippy reports no warning in the changed code. Not verified: no client
was run, so neither door nor saber was heard in game.
## Kill feed with icons

Branch `feat/kill-feed` (08/10/2026, based on `fix/obituary-names`, Linux): Sol
asked for an optional kill feed at the top right with icons: `Name [saber icon]
Name` for a saber kill, a skull for a suicide or a death to the world, and weapon
icons for other weapons. `cg_killfeed` (Settings > HUD > "Kill feed") now shows
the last five kills, newest at the top, each `killer [icon] victim` or `[skull]
victim` with the names in their colours, for five seconds and a one-second fade
([client.md](client.md#kill-feed), [kill_feed.rs](../crates/sjk-viewer/src/hud/kill_feed.rs)).
Icons are the HUD's cause-of-death pictures: the weapon's `w_icon_*` (an icon
pack's `hud/mod/*` when installed), Force Lightning's or Push's holocron for dark
Force kills and Force tosses, a drawn skull for solo deaths and causes without a
weapon, a short word for a picture that did not load. The feed stands under the
top right's FPS, team overlay, duel portrait, snapshot, inventory and powerups,
keeps clear of the top centre, and hides with the HUD. It replaces the
off-by-default one-line obituary of the same cvar: `cg_killfeed` defaults to 1, a
saved 0 is moved to 1 once (`cg_killfeedDefaultVersion`, as other defaults moved;
[sjk.md](sjk.md#defaults) puts new profiles on Sol's choices and says nothing
against a new HUD widget being on), and that line's alignment and reverse cvars
are gone. The console's kill line and the feed share one name reader.

Verified on Linux with Rust 1.97: unit tests of the means-of-death marks (picture,
holocron, word, skull), each weapon's `MOD_*` falling back to that weapon's
picture as in OpenJK, suicides and world deaths without a killer, the viewed
player's entries, names kept from the moment of the kill, the five-entry ring,
hold and fade, kills from an earlier timeline, the area under the top right's
stack and right of the centre, and a full feed of long names and every mark kind
drawing inside its area and command budget at 1080p, 1280x1024 and 4K; the HUD's
draw list gained room for it. Formatting, the locked workspace build and tests
pass; workspace Clippy finishes without errors and with no warning on a changed
line. Not seen on screen: there is no GPU or game data here, so the feed's look,
the skull, the icon sizes and the stacking under the team overlay and FPS were
not checked in a running client.

## Player names in kill messages

Branch `fix/obituary-names` (08/10/2026, based on `a6230f9`, Linux): Sol reported
that a player killed or killing sometimes showed in the console as `noname`, as in
`noname was sabered by {JoF}emiah{I}`. Cause: the console's kill message read the
name after checking that the player's whole `CS_PLAYERS` configstring was UTF-8,
and printed `noname` when it was not. Servers keep Latin-1 letters in names
(`é` is byte 0xE9), so every player with an accented letter or a Windows-1252
symbol in their name was `noname`; the same check gave such a player's gendered
suicide message the male form. OpenJK prints the name's bytes. Fix: the name and
gender are read from the string's bytes and the name decoded as the scoreboard
and crosshair already do (`LegacyClientInfo::name`), and each name ends with `^7`
as in `CG_Obituary`, so a colour left open in a name no longer tints the rest of
the line ([networking.md](networking.md#player-text)). `noname` remains only
for a slot with no name at all, where OpenJK prints an empty name.

Verified on Linux with Rust 1.97: a unit test of the console line with Sol's
example, the victim named `Rémi` in Latin-1 bytes, printed `noname^7 was sabered by
{JoF}emiah{I}^7` before the fix and the name after it; further tests cover the
`^7` after each name, the placeholder for an empty slot, the name accessor and a
female player's falling death with a Latin-1 name. Formatting, the locked
workspace build and tests pass; workspace Clippy finishes without errors, and its
warnings are all in code this change does not touch. Not verified: not seen in
game; Sol's victim's exact name is unknown, so a name that is `noname` for another
reason (none found in the code) would remain.
## Teleport and spawn beam

Branch `fix/teleport-spawn-beam` (08/10/2026, based on `a6230f9`, Linux): Sol reported
that the green beam shown when a player teleports or spawns appeared in a weird
position. Where the event plays it already matched `cg_event.c` (the player's box
dropped onto the floor, forward axis straight up). The beam itself is made of
`org2fromTrace` lines, which `CFxScheduler::CreateEffect` ends where a trace from the
line's origin along that axis meets a solid (`FxScheduler.cpp:1392-1418`): it stands
from the floor to the ceiling or sky. SJK's lines took only their authored `origin2` as
an offset (`effect_runtime.rs`), which these lines leave at zero, so each beam line drew
as a camera-facing square around the player's feet instead of a column. Every
`org2fromTrace` line is now traced once, before it is first drawn, as electricity bolts
already were ([rendering](rendering.md#entity-render-effects)). The other traced lines
(`env/beam`, `mp/jedispawn`) are stretched the same way; `EV_BECOME_JEDIMASTER`, whose
`cg_event.c` block is the teleport's with `mp/jedispawn`, now drops and points its
effect as the teleport events do (it played at the player's origin along their angles,
which would have laid its beam sideways); and the floor drop uses `MASK_SOLID`, terrain
included. Trip mine beams keep their cached traces. The retail `mp/spawn.efx` was not
available here; the public copies of `mp/jedispawn` and `env/beam`, its siblings, are
two such lines and an emitter. JoF EJK's source was not found publicly; JoF EJK's
`cg_event.c` places the effect as OpenJK does, adding only `cg_noTeleFX` and a duel
filter.

Verified on Linux: unit tests in a made-up room (a beam-shaped test effect runs from
the dropped box's origin, and from 20 units below it inside the floor slab, up to the
ceiling, traced once; the box lands 16 units above the floor and on terrain, and plays
nothing over a void; the offset end turns with the effect; the three events point
straight up whatever the player faces). The beam and terrain tests fail on the old
code. Workspace formatting, the locked build, the locked tests and workspace Clippy (no
warning in the changed code) pass. Not seen on screen (no GPU or game data here, and no
world shot plays effects): in game, respawn (`kill`) under a roof and in the open, watch
another player spawn, and teleport (a map teleporter, or `setviewpos` on an SJK server
with cheats); the beam should rise from the floor where the player stands to the
ceiling or sky, as in EternalJK. A Jedi Master pickup should show the same.
Known: stock's floor trace also stops on solid entities such as a lift (SJK's uses the
world only), and `traceImpactFx` on a traced line is not played.
## Narrower, centred compact SJK scoreboard

Branch `feat/compact-scoreboard-width` (08/10/2026, based on `a6230f9`, Linux): Sol
found the compact SJK scoreboard far too wide, with much wasted space, and asked
for the names much closer to the score and the board centred. The compact board
filled 680 to 1824 frame pixels (1144) whatever it held, its name column taking
what the numbers left (630 to 750 pixels in free for all, 316 a team). It is now
sized from its content (`scoreboard::sjk::Board`): the name column as wide as the
longest name with its emblem, medal bars and "Ready", measured in the families that
draw it (140 to 320, a longer name ending in an ellipsis), the numbers 48 after it
in columns as wide as their labels, the ping as before. It is centred on the
screen, or stands just clear of the chat column where centring would cover it,
never past 1824 or wider than before; the dim follows it, "Watching" starts at its
edge, and a duel's cards and queue share its middle
([sjk-ui.md](sjk-ui.md#scoreboard)). At 1080 lines in the UI's families, on the
world shot's made-up names: free for all of 8, 1144 to 440 (740 to 1180, centred;
500 with deaths counted); of 32, 1144 to 411 (471); Team FFA of 6 a side, 1144 to
835 (from 680, beside the chat column); capture the flag of 6 a side, 1144 to 1142,
where four number columns and a player's three medal bars leave little to gain. The
full board (compact off), the classic board, colours, badges, header and the HUD
hiding are unchanged.

Verified on Linux: unit tests pin, at 16:9 at 1080 lines and 4K, 5:4, 4:3, 21:9
(2560x1080, 3440x1440) and 32:9 (3840x1080, 5120x1440), the compact board centred
or 40 clear of the chat column, never past 1824, 400 to 1144 wide, a duel's cards
and queue on one middle; the full board's unchanged place; the numbers packed right
after the name column; the name column following its names within its bounds, a long
name stopping short of the score, a three-medal player's bars; every made-up match
fitting the canvas both ways at all those sizes; the widths above with the bundled
families. Workspace formatting, the locked build, tests and Clippy (no warning on a
changed line). Not seen on screen (no GPU or game data here): the world shot
`duel6_sjk_scoreboard`, run with `JKA_GAME_DATA` set (`cargo test --release -p
sjk-viewer duel6_sjk_scoreboard -- --ignored`), writes the compact board to
`target/world-shots/duel6-scoreboard-ffa.png`, `-full`, `-ctf`, `-duel`,
`-power-duel` and `-4x3`, and the full one to `duel6-scoreboard-full-split`; nor
seen over a real match, with a live chat beside it, or on an ultrawide screen.
## Parallax reaches farther and holds still up close

Branch `feat/parallax-range` (08/10/2026, based on `a6230f9`, Linux): Sol asked for
parallax to show farther away, and for something to be done when the camera is too
close. Its far limits were a fade below 20° above the surface and from 1.5 to 4 texels a
pixel along the longer side of the pixel's footprint, which a grazing view stretches: with
a 1024-texel map a floor went flat 90 to 160 units ahead of a standing player. Up close
nothing limited it: the parallax on screen grows as 1/distance from the surface, so the
third-person camera pressed against a wall saw several times the shift, swimming and
stretching the texture over relief edges, where the offset's jumps also picked blurred mip
levels. Now ([rendering](rendering.md#parallax)) the depth fades where it would move the
texture by less than half a pixel, below 8.6° and at mip levels 2 to 4 (at 1080p a floor
keeps it to about 410 units and loses it by 920, a wall seen at 45° to 570 and 1,920);
within `r_parallaxNearDistance` (default 24 units, live, console only) of a surface's plane
it stops growing on screen; the march takes 4 to 24 linear steps by the texels it crosses
(at most 31 reads of the height a pixel, against 25); and the maps and the diffuse image
are read with the coordinates' own derivatives. `r_materialMapsDebug 7` shows the reach.

Verified on Linux with Rust 1.97: formatting, the locked workspace build and tests (1639
passed, 51 ignored) pass, and workspace Clippy finishes without errors and without a
warning in the changed files. Unit tests cover the new cvar's bits (the default leaves the
word empty, clamping, steps of 4, apart from every other field, the shader decoding them),
view 7, the offset reads with explicit derivatives in both lighting modes, the material
programs validating with naga, and a CPU model of the shader's limits: floors and walls
keep parallax farther than before, the fade has no ring or jump, wherever it keeps less
than the old limits the old ones moved the texture by less than half a pixel, the near
limit holds the parallax on screen constant inside 24 units, the steps stay within 4 to
24, and the shader still holds the modelled arithmetic.

Not seen on screen: no GPU or game data was available, so nothing was rendered; the
ranges come from the model and the cost is estimated. To check in game, on a map with a
generated pack (sand on `mp/siege_desert`, stone floors on `mp/ffa3`):
`r_parallaxStrength 1` exaggerates the relief and `r_materialMapsDebug 7` shows how far
it reaches (yellow the whole depth, green faded, red held back near the camera); walk a
long floor, back the third-person camera into a relief wall and compare
`r_parallaxNearDistance 0` with 24, and compare frame times with `SJK_FRAME_BUDGET=1`.
Offline, the world shot `world_shot::notes::world_notes` with
`SJK_NOTES_CVARS=r_materialMapsDebug=7,r_parallaxStrength=1` on a note at a parallax
floor shows the far limit, and on a note written against a relief wall, with and without
`r_parallaxNearDistance=0`, the near one. Known: grazing views now keep parallax down to
8.6° (it was 20°), so the open note of sand too deep at grazing angles may return at high
strengths.
## Dynamic lights stop at walls

Branch `fix/dynamic-light-leaks` (08/10/2026, based on `a6230f9`, Linux): Sol reported
that dynamic lights leak through walls and round corners, which breaks immersion. A
surface took a saber's, bolt's or explosion's light by distance and facing alone
(`dynamic_light_modulation` and `emitted_light` in `point_lights.wgsl`, the material-map
highlights and the per-pixel model light), so the floor of the room behind a wall, or of
the corridor round a corner, was lit like open ground. Each frame every dynamic light now
gets a small octahedral tile of the static world around it, traced on the GPU against the
lamps' triangles, and every surface and per-pixel model checks it before taking the light
([rendering](rendering.md#dynamic-lights-and-walls)); `r_dynamicLightShadows 0` restores
the old look. Real-time lighting only: baked lightmaps (`r_dayNight 0`) have no triangles
to trace, and movers (doors) do not stop dynamic light.

Verified on Linux: unit tests replay the trace and the receivers' test in Rust on small
scenes (lit floors, walls, creases and stairs stay fully lit; a wall leaves the room behind
it dark, at most 3% of a large light's strength near its foot; a corner stops the light
wrapping round; the edge is soft and ordered; a light a hair inside the face it hit still
lights its side), pin the block layout against the programs and the uniform limit, the
tile mapping, the normal codes and the cvar, and validate the tracer and every changed
program with naga; workspace formatting, the locked build, tests and Clippy (no new
warnings). Not seen on screen and not timed: no GPU or game data were available. The owner
should ignite a saber beside a wall and at a corner on a real-time-lit map (`mp/ffa3`) and
look at the far side with `r_dynamicLightShadows` 1 and 0, check stairs and wall feet near
the saber for dark bands, and time `dlight-shadows` with `SJK_GPU_PHASES`; the world shot
`world_shot::notes::world_notes` with notes written on the floor beside a corner or
doorway, run again with `SJK_NOTES_CVARS=r_dynamicLightShadows=0`, compares the two.
## Force-profile rejoin retries

Branch `fix/force-rejoin-retries` (08/10/2026, based on the Force profile branch
below, Linux). When a server parks the player in spectator over their Force
profile (`nfr <rank> 1 <team>`), the client answers with `forcechanged "<TEAM>"`
and was meant to ask for the team again up to three times, 5.5 s apart, while
the player stayed parked. The negotiator was only polled after a userinfo
flush, and a retry falls due seconds after that flush, so the retries never
went out. It is now polled every frame (`ViewerConsole::flush_userinfo`; no
allocation when idle). Retries are left out in duel and power duel, where
spectating is the queue: OpenJK's `SetTeam` keeps a queued player spectating
but announces and respawns them on every `team` request, and `Cmd_Team_f`
refuses any change in power duel ([client.md](client.md#force-profile-on-a-server)).

Verified on Linux: unit tests for the retries (three, then the notice) and for
none in duel and power duel (fails without the change); formatting, the locked
workspace build and tests, and workspace Clippy (no warning on a changed line).
Not verified against a server: stock servers no longer park a player for an SJK
profile (it is fitted to their rules), so this path needs a mod that sends `nfr`
on its own. Known: a `team` typed in the console does not reach the negotiator,
so a player who typed `team spectator` within the 16 seconds after such a park
would be sent back once per remaining retry.

## Force profile on a server

Branch `claude/sleepy-noether-xv9qk9` (08/10/2026, based on `a6230f9`, Linux).
Sol reported that on a server with other Force rules no profile they picked
became usable, and asked to see which powers the server accepts while still
picking any (for full Force duels).

The value fitted to the server at join replaced `forcepowers` in every later
userinfo of the connection, so a profile applied in play never reached the
server, and nothing sent `forcechanged`, so the server would not have re-read it
anyway. The page also never knew the server's rules: it read `ui_rankChange`
(stale across servers, never set by a join) and `g_gametype` and `ui_freesaber`
cvars that do not exist. Now each userinfo sends the player's profile fitted to
the server's rules, keeping its `g_forcePowerDisable` powers (stock
legalization drops them without parking the player), Apply on a server sends
`forcechanged` after the userinfo, the page takes the server's rank and free
saber skills, and the server's limits are marked but not enforced, with a This
server panel in the SJK UI ([client.md](client.md#force-profile-on-a-server)).
An `nfr` reply now always waits for a userinfo flush, which an unchanged profile
did not start.

Evidence: OpenJK `codemp` (`WP_InitForcePowers`, `BG_LegalizedForcePowers`,
`Cmd_ForceChanged_f`, `ClientSpawn`'s `forceDoInit`, `UI_UpdateClientForcePowers`)
and jaPRO's `g_forcePowerDisableFFA` for duel powers. Unit tests cover the sent
profile (disabled powers kept, fitted to the rank's points, legal for the
server), a later profile being the one sent, `forcechanged` after Apply only on
a server and after the userinfo, the `nfr` rejoin retries, the page's server
rank, free saber and limits, and the Force page drawn with every rule to show.
On Linux, formatting, the locked workspace build and the locked workspace tests
pass; workspace Clippy finishes without errors and none of its warnings is on a
changed line.

Not verified: nothing was run against a server, in a game or on screen (no game
data or GPU here), so the server's re-read at respawn, JA+ and jaPRO full Force
duels and the new panels' looks are unchecked.

## Percent signs and quotes in chat

Branch `fix/chat-percent` (08/10/2026, based on `15cf7a9`, Linux): a `%` typed in
chat arrived as `.`, because the engine turns `%` into `.` in every command it
reads (`MSG_ReadString`), and a `"` became a space. SJK now sends them as
EternalJK does, `%` as `°/.` (byte 0xB0) and `"` as `''`, and its chat box shows
those back as `%` and `"`, so both clients show each other's. The composer
counts the escapes in its length limit, so a long message is never cut when sent
([client.md](client.md#percent-signs-and-quotes-in-chat)). Unit tests cover the
sent command and its bytes, an escape never cut by the byte budget, the
composer's limit, showing the escapes (colour codes, three apostrophes, a cut-off
escape, SJK's own message), a received chat line, and every printable ASCII,
Latin-1 and Windows-1252 character coming back as typed after the server's
`%` rewrite. On Linux with Rust 1.97, formatting, the locked
workspace build and tests pass; workspace Clippy finishes without errors, and its
warnings are all in code this change does not touch. Checked in game on Windows 11
on a JoF server: a `%` sent from SJK shows as `%`.

## Remapped vertex-lit targets and blocked remap maps

Branch `feat/map-remap-blocklist` (08/10/2026, based on `15cf7a9`, Linux). Sol
reported that `mp/ffa4` on a JA+ server, whose remaps include
`textures/rift/thick_trim -> textures/yavin/stonewall2_vertex` (and `flag2`,
`rockdoor` to other `yavin/*_vertex` shaders), looked too bright and blind to light.
Those targets are `q3map_onlyvertexlighting` shaders with one `rgbGen vertex`
stage, drawn on lightmapped surfaces. The surfaces' BSP vertex colours match their
lightmaps (`thick_trim`: mean 62.0 for both), so the colour was right, but only
vertex-lit (`LIGHTMAP_BY_VERTEX`) surfaces were marked as baked light at load, and
real-time lighting left the remapped stage's static bake in place. A remap onto a
lightmapped slot now marks such stages too
([rendering](rendering.md#server-shader-remaps)). `cg_remapsBlockedMaps`, with
`blockRemaps` and `unblockRemaps`, ignores server remaps on listed maps
([client](client.md#shader-remap-controls)).

Verified: workspace formatting, the locked build, clippy (no new warnings) and the
locked tests passed on Linux, with new unit tests of the vertex-light decision, map
name matching, list editing and the cache. The ignored world shot
`world_shot::notes::world_notes` on `mp/ffa4`, with the server's five remaps applied
as local remaps, at the largest `thick_trim`, `flag2` and `rockdoor` surfaces showed
them flat and pale, unaffected by the nearby purple lamp, before the change, and lit
like the unremapped map after it; the harness exits with `free(): invalid pointer`
after the test passes. Not checked on a live server or demo, in classic lighting
(where the bake shows as before), or in game for the blocked-map cvar and commands.

Review at the merge (08/10/2026): a map name with a multi-byte character across its
fifth byte (accented letters right after `mp/`) no longer panics `remap_blocked_maps::map_name`, which the
archived `cg_remapsBlockedMaps` would have repeated at every start; the docs say a list
with semicolons needs quotes in the console. Known: only the server's remaps are
blocked, not the map's own worldspawn or local `remapShader` ones, and a map's own
`rgbGen vertex` shader on a lightmapped surface still keeps its baked light.

## Compact SJK scoreboard; the HUD hides under the scoreboard

Branch `feat/compact-scoreboard` (08/10/2026, based on `15cf7a9`, Linux): Sol
asked for a much more compact SJK UI scoreboard, on by default, holding every
player in one column, and for the HUD to hide while the scoreboard is held.
`cg_compactScoreboard` (default 1, Settings > Scoreboard) gives the SJK look rows
of 20 to 32 frame pixels, splitting a list only below 20, so 32 players in free
for all or on one team stand in one column ([sjk-ui.md](sjk-ui.md#scoreboard)).
`scoreboard::hides_hud` joins the quick wheel and intermission in
`ground_hud::frame`, in every scoreboard style
([client.md](client.md#scoreboard-styles)).

Verified on Linux: unit tests pin one column for 32 players and an uncut team of
32 when compact, the old split and cut without it, and every made-up match
fitting the canvas both ways at 1080 lines, 4K and 5:4; workspace formatting,
build, tests and clippy (no new warnings). The world shot
`duel6_sjk_scoreboard` rendered the 30-player board in one column and, compact
off, in two (`duel6-scoreboard-full-split`); the harness process aborts with
`free(): invalid pointer` after the test passes. Not verified: the HUD hiding
was not seen in a running client (the world shots draw no HUD), and the compact
board was not seen over a real match.

## Movers in lamp shadows

Branch `feat/mover-light-occlusion` (08/10/2026, based on `15cf7a9`, Linux): Sol asked
for doors and every other moving object to block light when closed and let it through
when open. Lamp shadows came from a one-time trace of the static world, so a closed
door let every lamp through, and the far sun cascade kept movers where they stood when
it was drawn. Lamps near movers now get door tiles in the visibility atlas, traced
against the movers at their current pose, the lamp cache is baked again where their
shadows change, and the far cascade follows movers
([rendering](rendering.md#movers-in-lamp-shadows)).

Verified: workspace formatting, the locked build, clippy and the locked tests passed on
Linux; unit tests cover mover reach from spawn keys, lamp selection (reach, world
visibility, capacity), shadow cones, cache regions, poses, the trace queue, atlas
capacity and the CPU segment test. The ignored world shot on `mp/siege_hoth` (release,
Radeon RX 9060 XT), compared with `SJK_MOVER_OCCLUSION=0` on the same views: the light
a lamp sent through the closed hangar door onto the floor before it is gone (about
27,000 pixels darker), and with every mover hidden both runs match (no pixel differs by
6 levels or more at two of the three movers; 156 pixels by up to 21 at the third).
Timing is in [rendering](rendering.md#movers-in-lamp-shadows). Not checked in a live
game, on a server whose doors open and close, or on Windows; GI bounce follows doors
only as its probes refresh.

Review at the merge (08/10/2026): a still mover turned 90 or 270 degrees no longer
re-traces every frame (rotations are compared by distance, not by a dot product that
rounds to 1.0 in f32), slow motion adds up to a trace, movers the server removes or
hides (`SVF_NOCLIENT`) stop blocking lamps once the snapshot should have held them (by
its PVS and area test), the far sun cascade follows movers on maps without door tiles,
and the lamp cache's bake targets (20 bytes a cache texel) are kept only while movers
re-bake it. The detached-prop change the PR made in `world_props.rs` is dropped: that
file went with the join gate. Verified: formatting, the locked build, tests and Clippy
on the branch and merged on main; no game, GPU or timing run, so none of it has been
seen on screen. Known: a snapshot that hits the 256-entity limit and drops a mover in
view would briefly unblock it.

## SJK chat and emotes through the hub

SJK-only branch `personal/sjk-chat` (08/10/2026, based on `15cf7a9`, Ubuntu 24.04, Rust
1.99), with `feat/chat-emotes` in Sol-Vulpes/SJK-hub: Sol asked for a chat that goes
through the SJK hub rather than the game server, seen in the menus too and as a new
message mode in games, and for the groundwork of emotes synced between SJK players
(a friend makes the emotes themselves). Built ([hub-chat.md](hub-chat.md)): the hub's
`/v1/chat`, `/v1/emote` and long-polled `/v1/feed` (memory only, a `chat_muted` flag,
staff delete and mute), the client's feed thread, the SJK channel in the game's chat
(`messagemode5`, I), the main page's dock, the SJK chat page (`sjkchat`, the in-game
SJK menu), `cl_sjkChat`, and emotes' catalogue format, `sjkemote` and per-slot active
emotes for the animation work to read.

Verified: the hub's unit and in-memory API tests (the long poll under a paused clock),
`cargo test` and `cargo clippy --all-targets` there; workspace formatting, the locked
build, the locked workspace tests and workspace Clippy (no warning in the changed code)
here; the new unit tests listed in [hub-chat.md](hub-chat.md#verification); `hub_e2e`
against a hub built and run on this PC (the long poll delivered a message in about two
seconds; the service read its own message); the dock and the page rendered off screen
over a plain backdrop and looked at. Not verified: no client was started (nothing seen in
a game or over the map), nothing through Cloudflare or the deployed hub (not deployed),
Windows, many players at once.

Review at the merge (08/10/2026): an emote id from the hub that no catalogue entry
names is checked (`emotes::valid_id`) before the console shows it; the docs say that
only a new profile, or a whole config table imported at First setup, gets I, because
the missing-defaults migration only runs for profiles at `cl_bindDefaultsVersion` 0
(an existing profile binds it under Key bindings). Known and left as built: Tab from
Team goes to SJK, `cl_sjkChat` defaults to 1 (the feed is polled from the menus too,
about every 27 s when quiet), the client does not rate-limit sending (the hub's
quotas do), and a hub without the chat routes leaves the dock on "Not connected"
without logging.

## Graphics quality levels

Branch `feat/graphics-preset` (08/10/2026, based on `15cf7a9`, Ubuntu 24.04, Rust
1.99): Sol asked for a graphics performance choice at the top of First setup, from
a level that puts frame rate first to one that makes everything look its best.
Built ([client.md](client.md#graphics-quality)): Graphics quality, the first row of
First setup (under a Graphics heading) and of VIDEO, with Performance, Balanced,
High and Ultra, each setting 22 costly rendering cvars together; High is the
default visual profile, the row shows Custom once one of them is changed on its
own, and `graphicsquality [level]` names or sets it from the console. The FPS
cap, vsync, resolution, supersampling, taste settings and gameplay are not touched.

Verified on Linux: `cargo fmt --all --check`, `cargo build --locked --workspace`,
`cargo test --locked --workspace` (1516 passed, 49 ignored) and `cargo clippy
--locked --workspace --all-targets` (exit 0, no warning in the changed files).
New unit tests: a fresh profile is High (every High value is its cvar's default);
each level applies, reads back and survives a restart of the profile; no level is
cheaper than the one below it; every value is one its Settings row can show; a
setting changed on its own is Custom and steps from its nearest level; steps stop
at Performance and Ultra; the command; and the classic+ row's list, default and
Backspace. Not verified: no client was started (no game data or GPU run on this
machine), so the row was not seen in the SJK UI pop-up or the classic panel, and
no level's frame rate or look was measured or compared; the levels are chosen from
what each setting draws.

## Menu pictures follow the renderer

Branch `fix/menu-icons-after-world-change` (08/10/2026, based on `15cf7a9`, Linux): the
model squares on the character screen sometimes showed solid black, in the classic
menus and the SJK UI alike. Every world builds its own `ShapeRenderer`, with an empty
icon atlas, map preview and HUD preview, while the menu is handed from world to world
and its image caches kept counting their cells as uploaded. The menu now notices a new
renderer (`ShapeRenderer::id`, `ClientMenu::follow_renderer`) and each cache forgets
its uploads: the model icons, the Force page, the character-creation parts, the
key-binding pictures, the map preview and the HUD picker preview
([client](client.md)).

Verified: workspace formatting, the locked build, clippy (the same warning count) and
the locked tests passed on Linux, including
`player_menu::model_icons::tests::another_atlas_loads_the_icons_again`, which does not
compile without the fix. Not verified: the client was not run (join a server, change
map or disconnect, then open the character screen); the other caches share the reset
path but have no tests of their own.

## Modern UI removed

Branch `refactor/remove-modern-ui` (08/10/2026, based on `15cf7a9`, Ubuntu 24.04,
Rust 1.99): the native "modern" style is removed, so the menus are the SJK UI (the
default) or the classic ones ([client.md](client.md#menu-style)). Gone with it:
`ui_menuStyle modern` and its main page, server browser, connect notices, in-game
menu, player screen view and the modern looks of the changelog, Identity and
credits pages, the text dialog and the console's command browser, with the menu
wordmark banner; `cg_scoreboardStyle modern` (the floating table); `con_style
modern` (the Inter console) with `con_lineSpacing`, `con_maxLines` and
`con_datetime`, which only it read; `cg_hudStyle modern` (the default layout stays
as the `game` style's base layer); `ui_accent`, which only coloured the modern
canvas; and the join's gate flight on mp/ffa3 (the gate prop, its dust, the glide
and the portal pass drawing the destination through the doorway), which no other
style showed. The destination world is still prepared during a join and handed
over as before; the gate's face is drawn as ordinary world geometry again. At
start a saved `modern` (or `0`) in those four settings goes back to its default
and the four retired cvars are dropped (`retire_modern_ui`). Create game and its
map picker, the tabbed settings, the key-binding editor opened on its own, the
Update page and Camera control with the classic menus have no classic or SJK UI
version yet and keep the hero look; earlier sections of this page that test the
modern style describe code that no longer exists.

Verified on Linux: `cargo fmt --all --check`, `cargo build --locked --workspace`,
`cargo test --locked --workspace` (1501 passed, 48 ignored) and `cargo clippy
--locked --workspace --all-targets` (no denied lint; no warning kind more often
than on `main`). A new unit test starts a profile saved with every modern style and
the retired cvars; the style, HUD picker, in-game menu, First setup and text dialog
tests were updated to the two styles. Each removed branch was checked by reading to
be reachable only under the modern style. Not verified: no client was started, no
world shot or menu snapshot was rendered (no game data or GPU run on this machine),
nothing on Windows, and mp/ffa3's gate surface was not looked at on screen.

## Remapped surfaces keep the map's light

Branch `fix/remap-source-lightmap` (08/10/2026, based on `6d2eb4b`, Linux): Sol
saw that surfaces a server remaps seem not to react to light. A replacement was
compiled as rd-vanilla `R_RemapShader` registers it: with `LIGHTMAP_NONE` when the
world never used the target (a flat grey lightmap, or the fixed fallback entity
light for an unscripted texture), or with another surface's lightmap page. It now
keeps the replaced surface's own lightmap, vertex light or entity light
([rendering](rendering.md#server-shader-remaps)). This explains the "not affected by
light", uniformly pale surfaces in Sol's world notes from a JA+ server (below).

Verified: workspace formatting, the locked build, clippy (no new warnings) and the
locked tests passed on Linux. The ignored world shot `world_shot::notes::world_notes`
on `mp/duel6`, with yavin floors and walls remapped to `textures/imperial/*` images,
showed them flat and uniformly lit before the change and with the map's light and
shadow after it, matching the unremapped map; the harness's process exits with a
SIGSEGV after the test passes, with and without the change. Not checked on a live
server or demo, nor with a remap target shader that has a `$lightmap` stage.

## Achievement pop-up

SJK-only branch `personal/achievement-toast` (08/10/2026, based on `cf77cf8`,
Windows 11): Sol asked for a World of Warcraft style pop-up when an achievement
unlocks, with the single-player secret-area sound. Built
([identity.md](identity.md#achievements)): the unlock's centre print is replaced by
a card at the top centre of the screen with the board's medallion (now shared,
`achievements/medallion.rs`), "Achievement unlocked", the name, category and
description, in the SJK UI's families (Inter when they are not loaded). It takes no
input and pauses nothing, shows over play and the menus, queues several unlocks and
waits while the console is open or the medal pop-up shows. It slides down and grows
into place (0.45 s) with a gold ring sweeping round the medallion, a glow, a ring of
light, sparks, an edge flare and a glint, holds 5 s and fades out (0.65 s). Each
pop-up plays `sound/interface/secret_area.mp3` once through the interface cues; the
file is in `assets0.pk3`, and the single-player game module (`jagamex86.dll`) names
it beside `@SP_INGAME_SECRET_AREA`. `cg_achievementSound 0` (Settings > Sound)
leaves the sound out. Verified: unit tests for the queue (one at a time, no
duplicates, the pause between two), the phases and their opacity, lift and scale,
nothing drawn while idle, the sound posted once per pop-up and not with the setting
off, and every achievement's words fitting their column in the families and Inter
with the card on screen and clear of the crosshair at 1080p, 4K, 4:3, 21:9 and
800x600; off-screen world shots (`duel6_achievement_toast`) of seven moments over
duel6 at 1920x1080 and 1440x1080 and over the SJK UI's main page were reviewed.
Formatting, the locked workspace build, the viewer's tests and workspace Clippy
(no warnings in the changed code) pass. Not verified: the sound actually playing
and its loudness, the animation's motion at speed, and an unlock in a live match,
none of which a world shot shows.

## Staff tools

SJK-only branch `personal/staff-page` (08/10/2026, based on `cf77cf8`, Windows 11):
Sol asked for an in-game page, for Sol only for now, to give and take back medals
and clear achievements (his own, to test them). Sol chose a staff flag on keys,
set only by the hub operator, over a shared secret. Built
([identity.md](identity.md#staff), [SJK UI](sjk-ui.md#sjks-pages)): `staff` in
Profile, signed `StaffRequest`s in `sjk-identity` (search, award, unaward, clear
achievements) with their answers in `Service::staff_state`, the Staff page (Profile's
Staff tools, `staff`), and the client forgetting its own counts when a staff member
clears their own achievements. The hub's side is in the hub's repository, committed
there and not deployed.

Verified: unit tests (the service refusing a key that is not staff before the hub
hears it, answers replacing the player found and the player's own profile; the
page's target, Give and Take back as the catalogue allows, Clear all's second press,
Tab and typing; every focus within its canvas at 1080p, 4K, 4:3 and 21:9; forgetting
a count just below the goal and the higher goals on the same counter);
`cargo test --release -p sjk-viewer -p sjk-identity`; an end-to-end test against the
new hub built and run on this PC (`hub_e2e`, a key that is not staff refused, a key the test hub made staff
finding a player by name and key, giving and taking back a medal
with Decorated unlocked, clearing one achievement and all); the world shot
`duel6_sjk_staff`, looked at. Not verified: no game was started (the keys and pointer
on the page, a cleared own achievement unlocking again in a match); nothing reached
the deployed hub, which does not have staff yet.

## Wrapped chat rows keep their colour

Branch `personal/chat-wrap-colour` (08/10/2026, based on `cf77cf8`, Windows 11, Rust
1.96.0): Sol reported that the second line of a multi-line chat message turned white
though the message was green. The chat feed wrapped a message into up to four rows
(`Wrapped::update`, `chat/layout.rs`) but drew every row as its own text
(`chat/view.rs`, `build_feed`), and a text starts in its base colour, so a `^2` on
the first row never reached the next. `Wrapped` now keeps, for each row, the colour
code in force where it starts (`text::Carry`, built on the server browser's
`last_colour`), and the row is drawn after it, emoji rows included: a code takes no
room, so no wrap changes, and a wrap never cuts a code. A message with no codes is in
the base colour on every row. The console's scrollback and notify lines already
carried the colour into the next row (`console::classic::wrap`); they are unchanged,
and a test now pins chat lines there. Unit tests cover a long green message, a
coloured name cut across rows then the `^7: ^2` separator and message, a colour
change mid-message, no codes, a code at a break or before the space a break eats,
every width against a code between each pair of letters, a broken long word, rows
with emoji pictures, the feed's drawn glyph colours (every glyph of a long green
message is pure green on every row), and the console's wrap of a chat line; the
tests that need rows to carry fail without the change. On Windows 11 with Rust 1.96.0,
`cargo fmt --all`, `cargo test --release -p sjk-viewer` and
`cargo clippy --locked --workspace --all-targets` pass, with no clippy warning in
the changed code. Not checked in game (the client was not started): how it looks on
a JoF server, and a screenshot. The typed draft is one scrolling row, not wrapped,
and still starts in the base colour where it has scrolled past a code; this change
leaves it as it was.

## Crosshair-target private chat on U

`fix/chat-u-default`, based on `cfbc789` (2026-10-07, Windows, Rust 1.99):
U defaults to `messagemode3`, which opens a tell to the crosshair player;
global chat remains on Y. Migration preserves occupied keys and custom
crosshair-chat bindings and repairs the previous version-2 global-U default.
Focused tests cover fresh defaults, legacy migration, repair and custom keys.
Workspace build, tests and clippy passed before this correction; correction
checks are pending. Workspace formatting fails on existing material-generator
formatting. Physical-key targeting behavior has not been checked in game.

## Quick wheel hides the HUD

SJK-only branch `personal/wheel-hides-hud` (08/10/2026, Windows 11): Sol asked for
the HUD to be hidden while the quick wheel is open. `QuickWheel::hides_hud` makes
the frame's HUD visibility the hidden one (`ground_hud::frame`, as intermission
and the menus do) and the game-data HUD's readout none (`menu_hud::readout`), so
what `cg_drawHud 0` hides goes away (status, weapon, timers, vote and kill lines,
crosshair, the third-person ground readout); chat and nameplates stay. A unit test
covers the switch following the wheel (open, let go, cancel); the HUD going away
over a match was not seen in a running client or a world shot.

## Chat emojis

Branch `feat/chat-emojis` (08/10/2026, based on `f91e8ac`, Linux and Windows 11):
JoF EternalJK's chat emojis. With `cg_chatBoxEmojis 1` (default 0; also in Settings
> HUD), every `gfx/emoji/*.png` is drawn in place of its name in arriving chat
messages, its name made from the file name as JoF EternalJK's `CG_LoadEmojis` makes
it (`cg_main.c:2767-2827` at bd5e202), and `listEmojis` lists the names. JoF's 175
pictures in `EternalJK/japro-assets.pk3` are mounted on their own, without the rest
of that pack, and a row's text after a picture keeps its colour through the server
browser's helper for a text's last colour code, now in `text.rs` for both. The
pictures share four atlas rows, four to a cell. Where it differs from JoF EternalJK
(all of JoF's 175 load, the option is not switched off at map load, sender names are
not matched, an empty name or a missing picture affects only itself) is listed in
[client.md](client.md#chat-emojis). Unit tests cover the names, replacement in
messages (colour codes, first match, 32 per message, a name without a picture),
clean chat keeping messages that differ only in their emojis, the listing, mounting
the pictures alone from the `EternalJK` folder, the atlas packing, and where a row's
text and pictures are drawn with the option on and off. Mounting and loading all 175
pictures from a Windows install's `EternalJK/japro-assets.pk3`, and from JoF
EternalJK's `assets/japro` folder, was checked with test runs that are not part of
the change. On Linux with Rust 1.97, formatting, the locked workspace build and
tests pass; workspace Clippy finishes without errors, and its warnings are all in
code this change does not touch. Checked in game on Windows 11 on a JoF server:
`listEmojis` lists the pictures and chat messages show them.

## Console input colours

Branch `fix/console-input-colour` (08/10/2026, based on `f91e8ac`, Linux and Windows
11): the console's input row, classic and the SJK UI's deck alike, drew everything
typed in one colour, so `^1` did not turn what follows red. It now draws the input
as JoF EternalJK does, with its colour codes shown and applied: a code switches the
colour and is itself drawn in the new one (`Con_DrawInput`,
`cl_console.cpp:847-848`; `Field_VariableSizeDraw`, `cl_keys.cpp:455`;
`SCR_DrawSmallStringExt`, `cl_scrn.cpp:388-414`, at bd5e202), in the palette the
console draws its rows in: the game's, or the SJK UI deck's legible one. Columns,
scrolling, the cursor and selection are unchanged. Unit tests cover each character's
colour in the game's palette and in the deck's. On Linux with Rust 1.97, formatting,
the locked workspace build and tests pass; workspace Clippy finishes without errors,
and its warnings are all in code this change does not touch. Checked in game on
Windows 11: typing `^1a^2b^3c^4d^5e^6f^7g^8h^9i^0j` in the classic console shows
each part in its colour, and on a JoF server the SJK UI's console shows typed
codes in their colours.

## Profile page, bio rules and achievements

SJK-only branch `personal/profile` (08/10/2026, based on `c08e6cb`, Windows 11):
Sol asked for a real SJK profile with the medals and an editable bio that only
takes sane characters, and an achievements board synced with the hub. Built
([identity.md](identity.md#profile), [identity.md](identity.md#achievements),
[SJK UI](sjk-ui.md#sjks-pages)): the Profile page (SJK UI look in every menu style,
Profile and Achievements tabs; it replaces Identity on the SJK UI's SJK page, whose
arc holds five entries, and opens Identity from a button); the bio's rules in
`sjk_identity::bio`, shared word for word with the hub, applied to typing, before
sending and to anything shown; 21 achievements, 17 counted by the client from live
matches (obituaries, the player's own captures, duels, maps, servers, time) into
`achievements.json` and sent to the hub (`PUT /v1/achievements`), 4 counted by the
hub. The hub's side is in the hub's repository (bio rules, achievements table,
hourly allowances, backfill), committed there and not deployed.

Verified: unit tests for the bio rules (plain bios, tidying, 24 kinds of refused
character, limits, display of anything a hub sends), the service (counts sent at
most once a minute, held-back counts again after an hour, a bad bio refused before
the hub), the catalogue, the record (unlocks once, minutes, sets bounded, the hub's
counts restoring a reinstall, the file round trip and damage), the tracker (kills
by means of death, streaks and suicides, teammates, private and tournament duels,
captures and their resets, maps, servers and time while playing, following someone
and local games counting nothing), the page (typing filter, the hub's copy and a
draft, saving and the answers, no hub, Tab order, every tab, focus and state within
its canvas at 1080p, 4K, 4:3 and 21:9 in the families and Inter, the bio wrapped
inside its box, the board and the tabs' pointer); `cargo test --release -p
sjk-viewer` and `-p sjk-identity` pass. World shots `duel6_sjk_profile` (made-up
profile, typing, board, identity off, 4:3) were looked at.

Not verified: no game was started, so no real kill, duel, capture or unlock, the
centre print, the keys and pointer in a running client, and the shared save of
`achievements.json` are untested; nothing reached the hub (the shots' identity is
off). Until the hub update is deployed, the old hub answers achievements with 404
(the client waits an hour and tries again) and keeps its old bio rule.

## Quick wheel: every action's icon, and sounds

SJK-only branch `personal/wheel-icons-sounds` (08/10/2026, based on `01c29aa`,
Windows 11). Sol drew a second icon board (28 discs) for the quick wheel and
asked (08/10/2026) for "a sound effect while changing pages, moving selection
and actually validating selection". `scripts/wheel_icons.py` learnt the board
(`second`, a list of names per board) and cut its 28 icons
([assets/wheel](../crates/sjk-viewer/assets/wheel/README.md)): every catalogue
action now has its icon, every custom command shows the `{•}` disc, and Settings'
Quick wheel category wears the ring-of-discs icon on its rail instead of the
settings board's sliders. The UI atlas gives the wheel three rows (48 cells)
instead of one, which makes it 5504 texels tall instead of 5248 (2048 wide,
within the 8192 the device asks for). The wheel posts three cues through the
menus' mailbox ([client.md](client.md#sounds)): `sub_select` on a page change,
`menuroam` when the highlight moves to another choice, `button1` when the
chosen choice runs, quieter than the menus; nothing on letting go on nothing,
Escape or a close. `cg_wheelSounds` (archived, default 1) turns them off, from
Settings > Quick wheel's new Sound row or Interface's "Quick wheel sounds" row.

Verified: the first board, cut again by the changed script, gives its sixteen
icons back byte for byte; the second board's icons were compared with them on a
contact sheet on dark and light grounds (same disc size, the alpha edge crossing
half at 61.75 pixels in all 44, no dark fringe at the rim). Unit tests: every
action has an icon and every icon a use, the atlas fits the device's texture
limit, each cue has its own bit and the wheel's are quieter than the menus'; the
wheel posts the move cue only when another choice is highlighted (from none
too; not for moves within a choice or back to the middle), the page cue alone
on a scroll, a click or a second wheel key, the run cue only when a choice runs,
nothing for letting go on nothing, Escape or a single page, and nothing at all
with `cg_wheelSounds 0`; the editor's Sound row switches the cvar from Enter,
Space and a click. The world shots `duel6_quick_wheel` (now also
`duel6-wheel-icons-1` to `-3`, three full pages of the new icons beside three of
the first board's) and `duel6_quick_wheel_settings` (now also
`duel6-wheel-settings-sounds`, the switch off; the rail's icon in every shot; the
classic+ and modern editors with the row) were looked at.
`cargo test --release -p sjk-viewer` passed (1095), clippy reports nothing in
the files changed.

Not verified: no game was started, so no sound was heard. The cues' choice and
volumes (0.4, 0.5, 0.5 against the menus' 0.6 to 0.9) were picked from the
retail menus' use of each file and from their measured length and loudness
(`sub_select` 0.10 s, `menuroam` 0.16 s, `button1` 0.6 s, or 0.26 s for the
version JoF's cosmetic mod puts in its place); whether they are heard over a
fight, too loud, or whether the retail `button1`'s slow rise reads as a
confirmation, is for a listen. The second board's discs are a little brighter
than the first's, and its Saber style icon (arcs crossed by a saber) can read as
a "Wi-Fi off" symbol.

## The menus say SJK

SJK-only branch `personal/sjk-name` (08/10/2026, based on `01c29aa`, Windows 11):
Sol asked for the menus to say SJK instead of Sol JK. The main page's entry and
page, the in-game menu's entry and page, Quit and its hint, the version line,
Update's card, the hints that named the client, the window title, the console's
ready line, the startup notice, the GameData error and both programs' version
information now say SJK. The credits keep "Sol JK" for Sol's own section, as
the name's origin, and so do CREDITS.md, the site and the release titles.
Verified: `cargo test --release -p sjk-viewer` (1090 passed), workspace build,
no clippy warning in the files changed. Not verified: no game was started.

## SJK UI Credits and medals on the cards

SJK-only branch `personal/sjk-ui-credits` (08/10/2026, based on `5daf065`,
Windows 11): Sol asked to restyle Credits while keeping its sun animation, to
put the medals of Creyon and Lumaya on their cards, and to write Lumaya with a
capital L. With the SJK UI the page now has that UI's look
([credits_sjk.rs](../crates/sjk-viewer/src/credits_sjk.rs),
[SJK UI](sjk-ui.md#sjks-pages)): the emblem on the left is the sun (the
classic page's two sunbursts and two sets of god rays, from the same light
pictures, and its sparks), the sections down a lit rail under it and the people
in a reading column on the right. The page's state, folds, links, scrolling and
keys are shared with the other looks, which keep their drawing; Tab, Shift+Tab,
`[` and `]` step through the sections and E opens or closes every fold in every
look. credits.txt has a `medal:` key (the hub's ids, checked by
`credits_data.rs`); the cards show their medals in every look, and Creyon's and
Lumaya's carry Early Contributor. The card `[lumaya]` became `[Lumaya]` (handle
`lumayaa` kept), with `scripts/credits_history.py` mapping both names to it and
the history regenerated (one line changed), and the name capitalised in
CREDITS.md, the changelog, the debug panel, the site and the docs.

Verified: unit tests for the key (catalogue order, a repeated Bug Hunter
counting up, unknown, misspelt, empty and repeated ids rejected with their line,
the built-in cards' medals), and for the SJK look: every scroll position of the
page, folded and with every fold open, fits its canvas (no dropped draw, text or
pointer area) at 1920x1080, 3840x2160, 1024x768 and 2560x1080; the layout stays
in the column and does not change with the window; Creyon's medal picture and
name are drawn; Tab, Shift+Tab and E move as described and stop at both ends.
The world shots `duel6_sjk_credits` (1080p over duel6: the top, Creyon's and
Lumaya's panels unfolded, Creyon's work with a commit list open, the end of the
page) and the modern and classic `menu_snapshot` credits pages with the medal
chips were looked at. `cargo test --release -p sjk-viewer` passed (1054),
clippy reports nothing in the files changed.

Not verified: no game was started, so the pointer (hover bands, the rail's and
Expand all's clicks, links opening GitHub, dragging the scrollbar), the keys on
a real keyboard and the opening and unfolding animations in motion are untested;
the shots are 1080p only (other sizes only by the unit tests), and the page in
Inter before the families load was not shot. In a commit row at 16 pixels the
body family's underscores did not show (the run's rectangle was not the cause;
probably the atlas's minification); commit subjects are now 17 pixels, where
they show, but other small SJK UI text was not checked for it.

## SJK UI: Report a bug and its dialogs

SJK-only branch `personal/sjk-ui-report` (08/10/2026, based on `5daf065`,
Windows 11): Sol asked to restyle Report a bug in the SJK UI. With the SJK UI's
menus the text dialog, which serves Report a bug, a player report's few words
and the world note, is the UI's pop-up card
([text_dialog_sjk.rs](../crates/sjk-viewer/src/text_dialog_sjk.rs),
[SJK UI](sjk-ui.md#report-a-bug-and-its-dialogs)); the classic+ and modern looks
are unchanged. A report's card stays after Send: "Sending...", then the hub's
number, or why it was not sent (the identity off, the hub out of reach, its
refusal) with Edit back to the kept text. The centre print carries the outcome
only when no card waits for it (the other looks, or the card closed first). A
note still closes on Send, as its screenshot is of the next frame. Fixed with
it: the 2D pass now runs while the dialog is open without a session or menu (a
note on a map explored alone was not drawn); each layer of the 2D pass starts
alpha blended (a layer ending on the emblem's additive light, as the SJK UI's
main page does, drew the next layer's shapes additively, so a card's glass and
scrim over it added nothing); the pointer reaches the dialog before the menus,
as the keys did; the client menu is not drawn under the card.

Verified: unit tests (`text_dialog::sjk`: Tab and Shift+Tab, Enter and Space on
each control, typing and Backspace; the card waiting only for its own report's
answer, a second answer refused, Edit keeping the text, Escape while sending
leaving the answer to the centre print; a note and the classic and modern looks
closing on Send, and a card waiting dropped when the look changes; the pointer
reaching the field, Send, Cancel, Close, Done and Edit by their tokens; every
kind, state and focus within the canvas at 1080 lines, 4K and 4:3, in the
families and in Inter; the text wrapping inside the field; a long headline cut;
`ui_renderer::art`: a layer never inheriting the light blend), the full
`cargo test --release -p sjk-viewer` (1059 passed), and the world shots
(`duel6_sjk_report`, looked at: the empty report, a long text, a refusal,
sending, sent, not sent through the client's own path with the identity off,
a player report, a note, a 4:3 window, and over the main page, where the layer
fix was seen to work). `cargo clippy --locked --workspace --all-targets` adds no
warning in the files touched.

Not verified: no game was started, so the keys, the pointer, Ctrl+V, the
caret's blink and the cursor were not tried in a running client; nothing was
sent to the hub (the shots' identity is off; Sent is a made-up answer), so the
real hub's answers and how long Sending shows are untested; the note card over a
real selection's highlight; the layer fix on screens other than these shots.

## Nameplates: your drain measured

SJK-only branch `personal/drain-fix` (08/10/2026, based on `6d2eb4b`, Windows 11):
Sol drained players on the JoF JA+ server and their Force bars dropped only a
little. A test feeding the whole nameplate pipeline synthetic snapshots of a level-3
drain took the stock 4 a shot, so the rebuilt shots work as written; in the real
game either the rebuilt shot missed (leaving only the drained event's one shot every
0.45 s) or JA+, built on the original game, shoots every server frame rather than
every 50 ms. The local player's own drain is now measured from what the server
sends it (5 Force a shot, the heal equal to what was taken): its victims lose what
it healed, found in reach, by the event, or most in front; its pace and strength
are learnt for everyone's drain and lightning; and each drain writes a
`nameplate drain:` line to the log to settle which it was. Verified: unit tests
(the heal taken, the maximum, the event and front fallbacks, the learnt pace and
strength, the end-to-end pipeline). Not verified: in game; how JA+ really shoots is
still unknown until a log line comes back.

## Nameplates: the rest of the Force and health rules

SJK-only branch `personal/nameplate-sources` (08/10/2026, based on `5daf065`,
Windows 11): after drain, Sol asked for every prediction we had forgotten. Against
OpenJK's `w_force.c`, `g_combat.c`, `bg_pmove.c` and `bg_saber.c` (and SJK's ports
in `sjk-game-jka`), the estimates gained, by what each needs to be seen:

- shots ([force_streams.rs](../crates/sjk-viewer/src/hud/force_streams.rs), which
  replaces `drain_estimate.rs`): lightning (1-2 a 50 ms shot, doubled two-handed at
  level 3, 300-unit arc), confirmed by a renewed electrification; grip (its victim
  from the 256-unit line at its start, 2 a second past the shield); drain now also
  heals the drainer by what it takes (it was 0.01-0.04 a millisecond guessed);
- health: rage's 2 every 150/300/450 ms and its halving of blows and falls,
  protect's 40/60/80% kept off (it was "anything"), team heal from `EV_TEAM_POWER`'s
  named teammates (exact);
- Force: the force jump as the server charges it (nothing to start, then by the
  rise's speed every 200/300 ms; it was half the price at the start), the saber
  specials' flat prices (kata 50, cartwheel 10, the 25s), the wall moves' 6, a grip's
  start of 30, heal and team powers at their price (none show as active), an
  energize's gift, protect paying for blows, absorb giving back against a push or
  pull, `EV_NOAMMO` 0 proving under 50, the Jedi Master's full pool, and a disarm no
  longer charged as a throw. Drain and lightning were proven to start from their
  price instead of 25.

The vitals estimator now reads a snapshot's events first (`read`), lets the shots be
rebuilt, then applies both (`apply`); it hands the Force estimate what it saw
(`ForceFacts`). Verified: unit tests for each rule (143 HUD tests). Not verified:
no game was started; other players' levels are guessed at 3; JA+ (closed source) may
differ from OpenJK; held saber blocks in mods, hurt triggers and movers are not
seen.

## Camera control

SJK-only branch `personal/camera-control` (08/10/2026, based on `5daf065`,
Windows 11): Sol asked to rename Shot controls to Camera control and to restyle
it. Everything a player reads says Camera control: the SJK UI's and the modern
game menu's entry, the panel's title in both looks, its Sun page's line, the docs
and the debug panel ([client.md](client.md#camera-control)). Nothing it reads or
binds had "shot" in its name: `demo_camera`, `demo_sun`, F8 and the cvars it
sets are unchanged, and the code keeps its identifiers (`Page::Shot`,
`ingame_menu::shot`). With the SJK UI's menus the panel has the SJK UI's look
([SJK UI](sjk-ui.md#camera-control)): a column down the window's right edge over
a fade, the scene clear left of it with viewfinder corner and thirds marks, the
kit's tabs, sliders, buttons and switches, and the keys of what has the keyboard;
the classic and modern menus keep the modern panel. Up, Down and Tab now stop
once on each slider in both looks (its number and track were two stops), and the
SJK look shows numbers to a tenth.

Verified: unit tests for the rename (every style's main page has Camera control
on the row that opens the panel and no "shot"; both looks' text), the SJK look
(every state, Camera, Sun, no sun, live preview off and a number typed, inside
its canvas with every control's pointer area at 1080p, 4K, 21:9, 4:3 and
1024x768; nothing but the viewfinder's hairlines left of the window's middle; the
current tab gold, Hide panel gold, the navy fade), its pointer (a click on a
track sets the value where it lands, on the number opens typing, on a tab, a
switch and Hide panel acts), the keys' order in both looks, and the numbers.
`cargo test --release -p sjk-viewer` (1055 passed), `cargo fmt`, workspace
clippy (no new warnings in the files changed). World shots
(`world_shot::tests::duel6_camera_control`) rendered the SJK UI's game menu with
the entry chosen and the panel on its Camera page, the Sun page, a number being
typed and the Sun page where the sun cannot be set over duel6 at 1080p and 4:3,
and the modern look's entry and panel. Not verified: no game was started, so
the panel in a real match or demo (the camera and sun moving, orbit, live
preview, Move and hide, the HUD switch, F8), dragging with a real mouse and
typing with a real keyboard are untested; the panel's text widths are estimated
from the fonts, checked only in the shots.

## Quick wheel: Q opens the page used last

SJK-only branch `personal/wheel-q-last` (08/10/2026, based on `5daf065`, Windows
11): Sol asked for Q to open the quick wheel on the page used last. Q's default
is now a bare `+wheel` (Key bindings > Other > Quick wheel); profiles that saved
the old default `+wheel general` on Q are moved once (`cl_wheelBindVersion`,
in the console's start, as the console key's default moved); a page bound later
on Q, or on another key, stays, and R keeps `+wheel weather`. A unit test covers
a new profile, the move, other keys left alone and a later choice kept. Not tried
in a running client.

## Nameplates: drain and the verified badge

SJK-only branch `personal/nameplate-drain` (08/10/2026, based on `27696e5`,
Windows 11): Sol asked that the nameplates' Force estimate count what drain takes
from its victims, theirs or anyone's, and said the verified tick was misaligned
with the name. Drain is now rebuilt shot by shot
(now [force_streams.rs](../crates/sjk-viewer/src/hud/force_streams.rs), see
[client.md](client.md) "Force" and "Force streams"): every 50 ms, level 3's 512-unit arc
or levels 1-2's line from each drainer's origin and view, with the map's walls
traced; 2/3/4 a shot (absorb reduces it and gives a point back), the victim's
refill held 800 ms, the drainer paying 5 and holding its own refill 500 ms. The
old flat guess per `EV_FORCE_DRAINED` is gone; the event now only guarantees a
shot's worth. The local player's level comes from its Force profile. The drainer
used to pay nothing for drain. The badge was centred in a line box taller than
the text, while the HUD font hangs its glyphs low; it now centres on the middle of
the capitals from the font's own metrics, and `nameplate_snapshot` renders the
plates with the HUD font they use in game. Verified: unit tests (shot rate,
arc, line, walls, teams, duels, absorb, the event's floor, the refill holds, the
drainer's floor of 20, the badge level with the capitals) and the snapshot looked
at (1080p, the tick's middle on the capitals' middle; it was about 5 px high).
Not verified: no game was started; the drain levels of other players are guessed
(level 3), movers and players in the way are not traced, and JA+ (closed source)
may differ from OpenJK's drain.

## Quick wheel pages

SJK-only branch `personal/wheel-pages` (08/10/2026, based on `27696e5`, Windows
11): Sol asked for a better `+wheel`: one wheel with pages, the two existing
wheels its default pages, changed with the mouse wheel or the buttons while it
is held, pages edited in Settings, and a better look
([client.md](client.md#quick-wheels), [SJK UI](sjk-ui.md#quick-wheel)). The
wheel's data moved into `quick_wheel/` (`catalog.rs`: 42 actions in seven
groups, `pages.rs`: the pages and `wheel.json`, `ring.rs`: the SJK UI's ring);
the pages live on the console (`ViewerConsole::wheel_pages`), which the wheel
copies when it opens and Settings edits. Scroll down or right click is the next
page, scroll up or left click the previous, wrapping; neither fires nor changes
weapon while the wheel is open (`pointer_input.rs`). `+wheel general` and
`+wheel weather` (Q, R) open on their pages; a bare `+wheel` on the page shown
last. Settings has a Quick wheel category in the SJK UI and an Interface row
("Quick wheel pages") that opens the same editor over the classic+ and modern
settings. Limits: 8 pages, 10 choices a page.

Verified: unit tests for page changes (scroll notches, a trackpad's steps
adding up, the buttons with their releases kept from the game while a release
pressed before the wheel opened goes on, wrapping, a second wheel key, a single
page), release and cancel, the layout and text of both default pages, a full
wheel of custom choices inside its canvas at 4K; the pages store (defaults
without a file, edits saved and read back, limits, ids kept across a rename and
kept unique, a hand-written or broken file); the catalogue; the editor's keys
(a page added, named, filled with an action and a custom command, moved; a
choice changed, moved, removed; a page removed and the defaults restored, each
asked twice; Tab and Escape in the category) and pointer (hover focuses, clicks
act, the catalogue, the remove control, the way back), and every editor state of
a full wheel inside its canvas at 1080p, 4:3 and 4K. `cargo test --release -p
sjk-viewer` (1038 passed), `cargo fmt`, workspace clippy (no new warnings in the
files changed). World shots (`world_shot::tests::duel6_quick_wheel`,
`duel6_quick_wheel_settings`) rendered the ring over duel6 at 1080p and 4:3, in
the SJK UI's families and in Inter, half-way through a change of page, and the
Settings category in each state and the editor over the classic+ and modern
settings. Not verified: no game was started, so the mouse wheel and buttons
changing page in a match (and that nothing reaches the game meanwhile), the
change of page's feel, the hidden crosshair, typing in the editor's fields and
Shift or Ctrl moves with a real keyboard, the new actions in play (Day and
Night, Spectate, Respawn, the others) and a custom command run from the wheel
are untested.

## SJK UI: First setup and Update as pop-ups

SJK-only branch `personal/first-setup-popup` (08/10/2026, based on `b9db8c0`,
Windows 11): Sol asked for First setup as a pop-up with a clear, always visible
tick to stop it showing at start at its foot, and for the Update page as the
same kind of pop-up. In the SJK UI, First setup at start and `firstsetup` now
open a card over the map (`settings/sjk_popup.rs`): its rows are Settings'
(`sjk_view.rs` draws them in a moved frame, `Frame::shifted`; its row column,
open list and keys take their size and bounds as arguments), eleven lines
scrolling inside the card, the focused row's help under them, and a foot with
the "Don't show at start" tick, All settings and Done. The tick is the group's
own `ui_hideFirstSetup` row, pinned out of the scrolling lines
(`ClassicRows::pinned`), so the keyboard, clicks and the default reset work as
on any row. Tab and All settings open the Settings screen on First setup
(`SettingsResult::AllSettings`); the pop-up has no search. Picking the SJK UI on
the classic or modern First setup's Menu style row now comes back to the pop-up
(`set_menu_style`). Update in the SJK UI is a card too (`update_panel_sjk.rs`),
as tall as its text, with Release notes and Check again on the left and Close
and the gold action on the right. The browser's prompts share the card
(`kit::card`); `kit::scrim` and `kit::tick` are new.

Verified: unit tests for the pinned row (out of the lines, reached by Up from
the first row and Down back, flipped and reset), the pop-up's keys, All
settings and Done, and the hand-over from the classic and modern First setup;
workspace fmt, build, tests and clippy. `world_shot::tests::duel6_first_setup`
rendered the pop-up over duel6 (Styles, the rows, the menu style's help, the
foot) and `duel6_sjk_pages` the Update card. Not verified: no game was started,
so the pointer on the tick and buttons, scrolling inside the card and the
pickers (resolution, HUD) opened from it are untested in play. Classic and
modern First setup are unchanged.

## SJK UI Character in a game

Branch `personal/sjk-ui-ingame-character` (08/10/2026, based on
`0e4c914`, Windows 11): Sol asked for the SJK UI while in game. The player
screen opened from the in-game menu (Character) now shows its SJK UI pages
instead of the classic ones: `set_menu_art` makes it SJK in a game too, and the
model, with no menu-map stage in a match, stands in the classic pages' live
preview (`menu_stage::preview`) right of the form. `ModelPreview` carries a
`PreviewArea` (classic canvas or SJK frame), a `room` round the body (1.35 here,
so a raised blade stays in view) and a still `angle` (none: turning, as
classic). `ClientMenu::stage_model` never stages a screen opened from a game.
The match is dimmed by half behind the screen; the back key reads "Game menu".
Unit tests cover the preview request in and out of a game, the preview quad on
every page, and the camera's room keeping a held-out blade in view;
`duel6_sjk_character_in_game` rendered the three pages over duel6 with no server
(the in-game path forced by the return target). Not verified in a real match:
the preview's lighting where the player stands, and handing back to the game
menu.

## Medals

Branch `personal/medals` (08/10/2026, based on `8a45662`, Windows 11): Sol
asked for medals, recognition the SJK team gives players by hand that grants nothing
([identity.md](identity.md#medals)). The hub's profile and presence entries gain a
`medals` list (`sjk_identity::Medal`, empty from older hubs); the service reads the
player's own profile again every ten minutes. The viewer's catalogue (`medals.rs`)
knows four medals, each with a ribbon and two bundled pictures (the small ones in new
icon atlas cells, the whole ones decoded on a worker and uploaded the first time a
screen draws one). They show as ribbon bars after the scoreboard's SJK emblem (every
style), medallions on the player card and the SJK UI's Players card, a Medals panel on
the Identity page (all three looks) and a pop-up the first time the client sees a medal
(main menu or game menu; a centre print during play), remembered in `medals_seen.txt`.
The scoreboard's draw list grew to 1536 commands for the bars.

Verified: unit tests (old-hub JSON without medals, unknown ids left out, counts only on
the repeatable medal, the ribbon layout and how many bars fit, the card's lines, the
seen file per key and per count, the pop-up's queue and centre print, the own-profile
refresh with an explicit clock, the pictures' sizes and mip chains). The CPU snapshots
(`menu_snapshot::medals_snapshot`) drew the card glanced at and pinned, the Identity
page in the modern, classic+ and SJK UI looks with four medals and with none (16:9 and
4:3), and the pop-up; the world shots `duel6_scoreboard_medals` (classic, modern and
SJK UI boards; a duel's card), `duel6_ingame_players` (the Players card) and
`duel6_medal_popup` (the pop-up over the SJK UI's main page, its whole picture decoded
and uploaded on first use) drew them on made-up data; all reviewed. Not verified: no game
was started and no hub was contacted, so the live roster, the refresh, the pop-up's
timing, keys and pointer in a running client are unchecked. The hub side is in its own
repository.

## Illuminate (as a Force wheel entry; since 10/10/2026 a toy, see the first section)

Branch `personal/force-illuminate` (08/10/2026, based on `b9db8c0`,
Windows 11): Sol asked for a free power every player has, a light to see better
in dark maps, toggled off in the settings. [Illuminate](client.md#illuminate) is
a client-only Force-wheel pseudo-slot (21, after JoF's three) whose `+useforce`
or `force_illuminate` turns on a holocron floating by the left shoulder, with a
warm point light; `cg_illuminate` (Settings > Game) puts it on the wheel. Its
cube (a hand-written MD3), pictures, shader and icon are made from Sol's two
generated images by `scripts/holocron_assets.py` and mounted from memory below
the game data. Shared changes: `force_wheel::MAX_SLOTS` is 22 (the JA+
flamethrower's icon and name slots moved up one), `Selection::known` gives the
wheel's known bits, and the controls list has the bind with its picture. Unit
tests cover the wheel order and stepping with Illuminate, its press toggling once
and swallowing `+useforce`, the setting's effect on the selection, the inventory
fallback without Force, the bundled cube's geometry and winding, and the
holocron's fade, follow, lag, snap and placement; `cargo test --release
--workspace` passed (viewer 1000) and workspace clippy reported no new warning.
`world_shot::holocron::holocron_at_the_first_spawn` rendered it on duel6, ffa4,
siege_korriban and duel2 (off, lit, close, from a step back); the menu snapshot
drew the wheel with Illuminate selected. On review the metal read black with the
sun behind it, so the shader adds a faint self-light, and the emblem's glow was
lowered. No game was started: the holocron in a match, its place in third
person with each camera style, the toggle on a key and the light's cost are
unverified.

## SJK UI Character: classic+ Force level marks, coloured and lit

Branch `personal/sjk-ui-force-marks` (08/10/2026, based on `1eb2d77`,
Windows 11): Sol found the square level cells foreign and asked for classic+'s
style, a Force effect and colours by type. The levels are now classic+'s round
marks (the JoF HD `forcecircle`/`forcestar` art's shape, drawn by the UI):
a ring with the cost until bought, a disc after, on a channel lit to the level,
in the group's colour (Neutral silver, light blue, dark red, Lightsaber green;
the sub-headings and the power box too). Bought discs glow; a hovered level
charges its channel with a running spark and breathing marks; a bought level
sends out a ring (`note_level_bought`, from a click or Right). Unit tests
pass; `duel6_player_sjk` rendered the page and the hover (Heal's second level,
the spark caught mid-way). Not seen moving in a running client: the spark,
breath and ring timings are from the code only.

## SJK UI Character: hilt list, style buttons, priced Force levels

Branch `personal/sjk-ui-character-2` (08/10/2026, based on `dd38afc`,
Windows 11): Sol asked for a better character editor in the SJK UI. Saber page:
the style is three buttons and the hilts a list (two side by side for Dual),
in the order JoF EJK lists them; the catalogue now orders `saber_hilts` by
the game's load order (`legacy_saber_load_order`: `list_files` over
`ext_data/sabers`, then each file's definitions, as `WP_SaberLoadParms` and
`WP_SaberGetHiltInfo` meet them), so the classic+ and modern lists follow it
too. No hilt icons exist in JA or JoF's packs (JoF's lists are text), so the
list is names. Force page: classic+'s groups (Neutral, the side's, Lightsaber),
each level a cell with its price, a points bar previewing a hovered level, and
a box at the bottom right with the power under the pointer (holocron, name,
group, level, a short description written for SJK, prices). Up and Down follow
the pages' visual order. Unit tests cover the load order (two mounts, a file
hidden by a higher-priority one), the tokens, the groups and the other side
left out, buying and stepping down a level by pointer, the style buttons, Dual's
key order and the canvas fitting; the sjk-viewer tests passed (980) and
`duel6_player_sjk` rendered the three pages and Dual. No game was started: the
pointer's hover preview, the wheel on the lists and the order against JoF EJK's
own menu on Sol's install are unverified (the research simulated JoF's order on
the installed pk3s and it matched the shot's).

## EJK camera style by default

Branch `personal/camera-ejk-default` (08/10/2026, based on `dd38afc`,
Windows 11): Sol asked for the locked JoF EJK camera as the default and for the
camera style to be a style chosen near the top of First setup. `cg_cameraStyle`
now defaults to `ejk` and an unknown value falls back to it (`sjk` still selects
the eased camera); the offered order is EJK, SJK. Every profile had saved the old
default `sjk`, so the console's start moves a saved `sjk` once to `ejk`
(`cg_cameraStyleDefaultVersion`, as the menu style default moved); an `sjk`
picked after that stays ([client.md](client.md#camera-style)). First setup's
first heading is now Styles, over the menu style and the camera style; the
camera row left its own Camera heading between Aim and Sound.

Verified: unit tests for the parse and default (an unknown value is `ejk`), the
one-time move (a saved `sjk` moved, a later `sjk` kept across starts), the
cvar's saving, and Styles over First setup's first two rows; workspace fmt,
build, tests and clippy (no new warnings). `world_shot::tests::duel6_first_setup`
rendered a new profile's First setup in the SJK UI and classic menus: Styles,
Menu style, then Camera style with EJK lit. Not verified: no game was started,
so the move from Sol's real `config.cfg` and the locked camera as a default in
play are untested.

## SJK UI as the default menu style

Branch `personal/sjk-default` (08/10/2026, based on `7a85516`, Windows
11): Sol asked for the SJK UI as the default, also for players updating, with
the choice offered in First setup, and for `quicksetup` to go so that `q`
completes to `quit` alone. `ui_menuStyle` now defaults to `sjk` and an unknown
value falls back to it (`classic`/`1` and `modern`/`0` still select those); the
offered order is SJK, Classic, Modern. Every profile had saved the old default
`classic`, so the console's start moves a saved `classic` once to `sjk`
(`ui_menuStyleDefaultVersion`, as the scoreboard and console key defaults moved);
a style picked after that stays, and a saved `modern` is left alone
([client.md](client.md#menu-style)). First setup's first row is the menu style,
under a Menus heading; picking a style there keeps First setup on show (the
classic panel, the SJK UI category or the modern FIRST SETUP tab, which
`SettingsMenu::continue_modern` now prefers to Interface). The `quicksetup` alias
is removed outright, with no hidden one; `menu/quick_setup.rs` became
`menu/first_setup.rs`.

Verified: unit tests for the parse and default, the one-time move (a saved
`classic` moved, a later `classic` and an earlier `modern` kept across starts),
the scoreboard's `auto` following the new default, First setup's first row and
heading, and the style hand-over from First setup in each direction; workspace
fmt, build, tests and clippy. `world_shot::tests::duel6_first_setup` rendered a
new profile's first start over duel6 (the SJK UI with First setup open by itself
on the Menu style row), then classic and modern picked from there. Not verified:
no game was started, so an update from a real old `config.cfg`, the boot map
following the moved style on the next start, and the SJK UI's unfinished
screens (dialogs, Credits, Create a game, the in-game player screen) now meeting
every new player are untested in play.

## Wall grab facing

Branch `personal/wall-grab` (07/10/2026, based on `7a85516`, Windows 11):
Sol found that on a JA+ server a player holding a grabbed wall could turn their
body with the mouse, floating off the wall. A JA+ server leaves the view free
during the hold while `CG_G2PlayerAngles` turns the body with the view; the model
of a player whose legs play a wall rebound or its hold now faces the wall as a
stock server turns it ([client.md](client.md#wall-grab-facing)), for the local
player and others. `PM_AdjustAngleForWallJump`'s rebound side and wall check are
shared by the move and the new `pmove::wall_hold_yaw` (a refactor, no change to
the move); prediction, the camera and the kick off are unchanged. Unit tests pin
the facing for each rebound side, a view turned on the wall (square and slanted
walls), no facing out of reach, on a slope or outside a rebound, and the latch
across a missed check. No game was started: the pose on a real JA+ wall, the
swing back to the view after the kick off and other players' holds are unverified
in a running client.

## Players page and player reports

Branch `personal/player-report` (07/10/2026, based on `7a85516`, Windows 11):
Sol asked for a small scoreboard in the in-game menu and a way to report a player to
the hub, for verified players only, with antispam. The game menu gains a Players page
(`ingame_menu::players`: the roster read from the session once a second, scores asked
for every two seconds) and a Report page (seven reasons); a reason opens the text
dialog and Send goes through the identity service as `POST /v1/player-report`
(`sjk_identity::PlayerReport`, sent only when the hub's profile says the key is
verified) ([identity.md](identity.md#player-reports)). The SJK UI has it as a main
entry, Players, drawn as a table with the chosen player's card
([sjk-ui.md](sjk-ui.md#in-game-menu)); the classic and modern menus reach it from the
SJK pop-up's Report a player. The hub side (endpoint, quotas, operator routes) is in
the hub repository, branch `player-report`, not deployed. Verified: unit tests (the
roster's order and rows, paging through 32 players, yourself and bots refused, the
gates and their reasons, the dialog's limit, the service refusing an unverified key and
sending a verified one's report with the worn name, the request body, the SJK UI pages
drawn within the canvas at five window sizes with every row's pointer area), the
`hub_e2e` player report test against a local hub built from that branch (refused while
unverified, stored once verified with the hub naming the target's key from its claim,
a repeat refused, listed by the operator), and `world_shot::tests::duel6_ingame_players`,
which rendered both pages in the SJK UI, classic and modern looks, verified and not,
and the dialog, on a made-up roster, reviewed. No game was started and no server joined:
the roster, scores and hub marks on a real server, Escape, keys and pointer on these
pages in a running client, and the outcome's centre print are unverified.

## SJK UI: console design and selectable browser text

Branch `personal/console-style` (08/10/2026, based on `7a85516`,
Windows 11): Sol asked for the console in the SJK UI's look, its text
selectable with the mouse and copied with Ctrl+C, the same in the F3 command
browser, and several designs to choose from. `con_style` gained `sjk` (deck),
`horizon` and `dock`, the classic console's grid and keys in the SJK UI's
colours and families ([client.md](client.md#sjk-ui-console)), and `auto`, the
new default (the deck with the SJK UI's menus, else classic; a saved `classic`
moves once to `auto`, `con_styleDefaultVersion`). The console's layer gained
fades and the SJK UI's family text runs; the scrollback's mouse selection,
double click and Ctrl+C were already there and now draw in gold, with "Copied"
in the header. The browser (F3) has an SJK UI look whose detail column's text
is selectable (`text_select.rs`), and Ctrl+C in every look copies the selection
or the chosen entry as a console line. Unit tests cover the cvar's values and
move, the grid, caret and ghost completion, and the selection's drag, double
click, joins and reset; the release sjk-viewer tests passed (973).
`world_shot::tests::duel6_console_styles` and `duel6_console_browser_sjk`
rendered every look over mp/duel6 with made-up scrollback, a drag selection and
a ghost completion, at 1080 lines and the deck at 4K. Not verified: no game was
started, so real typing, dragging, the clipboard, the opening slide, `con_scale`
and `con_opacity` with these looks, narrow windows and the families' first load
are untested in a running client.

Sol chose the deck, and branch `personal/console-deck` (08/10/2026,
based on `b9db8c0`) removed `horizon` and `dock`: a profile that saved either
now gets `auto`'s console, the deck with the SJK UI's menus. The cvar's unit
tests cover that fallback; the deck's code and world shots are unchanged, and
it is still untested in a running client.

## Sol's world notes of 07/10/2026

Branch `personal/inspect-notes` (08/10/2026, based on `7a85516`, Windows 11):
the 45 world notes Sol wrote on 07/10/2026 (`notes.jsonl`; the nine sent to the hub
are copies of local ones) on `mp/ctf4`, `mp/ffa1`-`ffa4`, `mp/siege_desert`,
`ffa_bespin`, `t2_rogue` and `vjun3`, three of them tests. A new ignored world shot,
`world_shot::notes::world_notes`, renders each note's view again with the player's
own renderer settings, with and without a saber-sized red point light, optionally
with remaps (`SJK_NOTES`, `SJK_NOTES_CVARS`, `SJK_NOTES_REMAP`; see its rustdoc).

- Fixed, `videoMap` ("Where video ???", `mp/ffa1` `vjun/hangar_console` drawn as the
  magenta checker on a server): RoQ videos play on video stages
  ([Video stages](rendering.md#video-stages-videomap)). A world shot with the console
  remapped locally to `textures/video/raven` showed the Raven logo playing.
- Fixed, light fixtures that should emit (ten notes): declared fixtures whose overlay
  does not glow take emission maps, generation 6 of the generator
  ([Emission maps](rendering.md#emission-maps)). World shots showed seven of them
  glowing with a generation-6 pack.
- Fixed, sand steps on `mp/siege_desert`: untagged ground takes its footstep bank
  from its shader's name ([client.md](client.md#animation-sounds-and-voice-variants)).
  Not heard in a game; footprints followed on 13/10/2026 (top of this page).
- Changed, notes name a remap's target: several notes from a JA+ server describe
  surfaces ("not affected by light", uniformly pale) that the same views draw normally
  offline, even with Sol's settings and a point light; the server's remaps are the
  likeliest difference, and a note now names the shader a remap draws
  ([client.md](client.md#player-card)). Not resolved until notes with remaps arrive.
- Material pack, not committed (retail-derived): Sol's per-texture requests ("make it
  3d", "too shiny", "reversed", "metallic", flagstones and a landing pad classed as
  cloth and metal) became lines in his local overrides file, and a generation-6 pack
  for the MP maps, the test maps, `t2_rogue`, `vjun3` and `ffa_bespin` (1,326
  textures, 70 emission maps) was written to the per-user folder's `generated\gen6`
  for Sol to install. Most "too shiny" notes were made with `r_specularMapping 0`, where the
  pack's roughness and metalness do not apply.
- Fixed on 13/10/2026 (branch `personal/render-notes`): the decal of `mp/ctf4` that
  "appears like there is no fog", which took about 40% of the fog its wall took: as in
  rd-vanilla, its `Equal` fog pass with the decal's bias met no depth. Decals are now
  drawn before the opaque world's fog.
- Open: Korriban's environment-mapped `glossyBase` shaders (`door`, `entrance_top`,
  `os_outsidebased`, `os_basic_pillarb`)
  still take no material maps (their stages do not collapse).

## SJK UI: coloured names and the browser's sort mark

Branch `personal/sjk-ui-browser-colours` (07/10/2026, Windows 11): Sol
asked for the browser's server names in their colours and found its sort caret
odd. The list, the chosen server's title (a wrapped second line carries the
first line's last colour) and its players now draw their `^<digit>` codes; the
search and the password prompt still read names without them. Every text in the
SJK UI's families draws the codes in `text::CodePalette::Legible` (black as grey;
red, green, blue and magenta lifted, same hue), since pure blue and black do not
read on the navy; the game's own palette is unchanged everywhere else. The sort
mark is three stacked bars (`kit::sort_mark`). A unit test checks the palette's
lightness and hues; the world shot was re-rendered and checked zoomed. Not seen
in a running client.

## SJK UI: loading screen

Branch `personal/sjk-ui-loading` (07/10/2026, based on `076806e`, Windows 11):
the SJK UI's own connect and loading screen replaces the classic one in
`ui_menuStyle sjk` ([sjk-ui.md](sjk-ui.md#loading)). `menu::sjk::loading`
draws `ClassicLoading`'s state: before the map is known a block at the bottom
left over the touring menu map (server name, address, a gold step line and
the step in words); once it is known the destination's levelshot over the
window with the map's name, the server, its rules, mod and message of the day
and a gold load bar; a failure in the same layout. Shared changes:
`ClassicLoading` also keeps the session's world stage and whether the session
is in hand (fed by `sync_classic_loading`), a join count, whether the load
named its map and the gamestate's facts; `levelshot::cover_uv` (the browser's
crop, now shared; the browser uses it) and `levelshot::screen_fit` (a picture
wider than the window keeps its sides, so the HD packs' titles are whole);
`ClientMenu::sjk_screen` includes the loading phases and
`classic_hides_world` asks the SJK screen whether to leave the world out (on a
server's world always, on the menu map once the levelshot covers it). The
classic and modern screens are unchanged. Unit tests cover the steps and their
words, the line never running back within a join, the levelshot's fade, the
gamestate's facts (free for all, duel, siege), the names' fallbacks, every
state at four window sizes fitting the canvas with its Esc target, the next-map
case and when the world is left out; `cargo test --release -p sjk-viewer`
passed (931). `world_shot::tests::duel6_sjk_loading` rendered five frames over
duel6 on a made-up join of the JoF server (gamestate and progress faked, the
real mp/ffa3 levelshot from the installed JoF HD pack). No game was started and
no server was contacted: a real join's steps and timing, the fade in a running
client, downloads, a hosted game's start, a server's change of map, a kick and
the hand-over from the screen to the game are unverified. On review the lead
put the map's own name large over its file name and fitted wide levelshots
whole; the frames were rendered again.

## SJK UI: Scoreboard

Branch `personal/sjk-ui-scoreboard` (07/10/2026, based on `076806e`,
Windows 11): the scoreboard in the SJK UI ([sjk-ui.md](sjk-ui.md#scoreboard)).
`scoreboard::sjk` draws the board's rows and match facts as columns floating
over the darkened game, right of the chat column: a header line (map, mode and
limits, time left, your place in gold), one list in free-for-all (two side by
side past 22 players), the teams side by side under their scores, the duelists
as facing cards over the players waiting, spectators on one line.
`cg_scoreboardStyle` gained `sjk` and `auto`, the new default: the SJK UI's
board with `ui_menuStyle sjk`, else the classic one. Every profile had saved
the old default `classic`; on review the lead added a one-time move of a saved
`classic` to `auto` (`cg_scoreboardStyleDefaultVersion`, as the console key's
default moved), so the SJK UI brings its board; a look chosen after that stays
([client.md](client.md#scoreboard-styles)). The board reads the match's
limits, `CS_LEVEL_START_TIME`, `CS_CLIENT_DUELISTS` and
`CS_CLIENT_DUELHEALTHS`; its canvas holds 320 text runs and 1024 draw commands
(was 208 and 640); the UI's families load for a scoreboard chosen on its own;
the browser's signal bars are shared (`pub(crate)`).

Verified: unit tests pin the cvar's semantics (`auto` under each menu style, a
saved old `classic` moved once to `auto` and a later `classic` kept across
starts, mistyped values),
32-player boards in every mode fitting the canvas at 1080 lines, 4K and 5:4,
a cut team list keeping your row, shared places for ties, the duelists' cards
and health, the clock and duelists read from config strings, the header's words
and the columns not overlapping; the release sjk-viewer tests passed (932) and
workspace clippy shows no warning in the files touched.
`world_shot::tests::duel6_sjk_scoreboard` rendered the board over duel6 on
made-up matches (free for all with 14 and 30 players, capture the flag with
the classic menus and `cg_scoreboardStyle sjk`, a duel, a power duel, a 4:3
window), with the families loaded. Not verified: no game was started, so the
board over a real match (the server's scores and their order, the clock
against a real level start, the duel config strings, flag carriers, the
intermission's Ready marks), its fades, the chat column beside it, the HUD
under its dim and the Settings row's list of four are untested in play.

## SJK UI: in-game menu

Branch `personal/sjk-ui-ingame` (07/10/2026, based on `076806e`, Windows 11):
the SJK UI gets its own in-game menu ([sjk-ui.md](sjk-ui.md#in-game-menu));
until now `ui_menuStyle sjk` showed the classic bar there. `ingame_menu::sjk_view`
draws the in-game menu's pages as a compact arc over a fade on the match's left,
a match card on the right (read from the session by `refresh_game_menu_card`)
and the keys at the bottom; `ingame_menu::sjk_actions` acts on its own rows
(the main page, Vote, Leave) and sends Escape back to the entry that opened a
page, the shared actions keeping Team's sides, Siege's classes, the call-vote
lists and Sol JK's page. Shared changes: `InGameMenu::is_classic` is false for
the SJK UI; Settings opened from a game names "Game menu" as its way back
(`settings::Rail::back`, all screens of the SJK UI's Settings); a screen opened
from the SJK UI's game menu returns on its entry; the frame's 2D pass also runs
while the game menu is open without a session (`frame_overlays.rs`; by reading
the code, the game menu of a map explored alone, which has no session, was
undrawn in every style for the same reason, not checked in a running client).
Verified: unit tests (rows and hints of each page,
row counts per style, the keys passing over rows that cannot be taken, Escape's
parents and the call-vote rows they return to, the arc fitting 1 to 18 entries
between the page name and the keys, the main page, Leave and an 18-row map list
drawn at 1080p, 4K, 21:9 and two 4:3 sizes, and each page (Main, both Teams,
Vote, Call a vote, a 40-map list, game types, Sol JK, Leave) at 1080p, 4K and
1024x768, each with every row's pointer area and no canvas overflow, the card
read from a made-up game state, Settings opened from a game returning to it), and
`world_shot::tests::duel6_sjk_ingame`, which rendered the menu over duel6 on a
made-up match (main page, Team in a CTF, a vote on, the installed maps, Leave, a
spectator, Settings from the menu) and was reviewed. No game was started and no
server joined: the card on a real server (its score, place, clock, team scores
and counts), Escape and the pointer in a running client, the hand-over to
Settings, Servers and Character and back, voting and calling a vote, and Siege's
class list in this look are unverified. On review the lead made the Leave page
open on Stay (its rows act at once, with no confirmation) and darkened the
match card's side; the frames were rendered again.

## SJK UI: Servers

Branch `personal/sjk-ui-browser` (07/10/2026, based on `b0470fc`, Windows 11):
Sol asked for the server browser in the SJK UI next. `menu::sjk::browser` draws
the browser's state and tokens on the UI's frame ([sjk-ui.md](sjk-ui.md#servers)):
sources and Show filters on the left, the sortable list, the chosen server's
levelshot, numbers and players on the right, and SJK UI password and address
prompts. Its keys go first to `sjk_browser_key` (the search, the left column),
the rest to the shared browser keys, now `ClientMenu::browser_key`. Shared
changes: the browser's search matches names without colour codes (all styles);
`ServerBrowser::clear_filter` and `favorites_listed`; the game type steps both
ways (`step_browser_mode`); the levelshot cache reports the shown image's size;
the Settings top bar became `menu::sjk::top_bar`; a menu canvas keeps 224 text
runs (was 160). Unit tests cover the tokens, the layout, the keys (left column,
search, a filter switch), a 40-server list at 4K fitting the canvas and the
password prompt; `world_shot::tests::duel6_sjk_browser` rendered the screen
over duel6 on made-up servers. No game was started: a real master server's
list, the status query, the levelshot loading in a running client and joining
from it are unverified.

## Third-person camera style

Branch `personal/camera-style` (07/10/2026, based on `7741498`, Windows 11):
Sol found SJK's third-person camera like single player's and asked for a setting
to have it as in JoF EJK. Reading JoF EJK's `cg_view.c` against `camera.rs` and
`camera_motion.rs` found the same camera: EternalJK's `CG_OffsetThirdPersonView`
with `cg_cameraFPS 125`, range 80, height 16 and damping 0.3 and 0.5 by default
in both. What differs is that EternalJK drops both dampings while a strafe helper
style is drawn, and Sol's JoF EJK profile (`GameData/EternalJK/eternaljk.cfg`,
03/10/2026) has one on (`cg_strafeHelper 2242`), so its camera never trails; that
profile also sets `cg_thirdPersonRange 100`. `cg_cameraStyle` (archived; `sjk`,
the default then, or `ejk`, the default since 08/10/2026 as described above)
offers that locked camera in Game options and First setup
([client.md](client.md#camera-style)); distance and height stay with their cvars.
Unit tests pin each style's damping, the `ejk` camera at its ideal place after a
move and turn while `sjk` trails, and the cvar's registration and saving; the
sjk-viewer tests passed. No game was started: how `ejk` feels in play and whether
it matches Sol's JoF EJK are unverified. Left as they were: SJK clamps the pitch
to 89 degrees on a reset frame (EternalJK 80), and `sjk` does not apply
EternalJK's strafe helper rule (the lines and CGaz styles, bits 1 and 2, would
turn damping off in EternalJK).

## Strafe lines (JoF EJK's strafe helper)

Branch `personal/strafe-lines` (11/10/2026, based on `059f347f`, Windows 11): the
strafe helper's lines style (`cg_strafeHelper` option 1), JoF EJK's two lines, in
first and third person, airborne and on the ground; the guides are no longer
limited to first person. A `Line` draw command (anti-aliased stroke) was added to
the UI renderer and the menu snapshot rasterizer. Unit tests pin the lines'
geometry and colours for Sol's JoF value 2242, the speed gate and the Settings
switch's bits; the workspace checks passed. No game was started: how the lines
look and whether they match JoF EJK in play are unverified.

## SJK UI: text centred in its controls

Branch `personal/sjk-ui-centre` (07/10/2026, based on `7741498`, Windows 11):
Sol saw the SJK UI's text sit off centre in its buttons. The renderer sets a
run's line box from its rectangle's top and the capitals' middle lies 0.48
(Rajdhani) or 0.54 (Exo 2) of the size below it, so text drew 2 to 4 pixels high
at 1080 lines. `menu::sjk::text` now centres each run's letters on its
rectangle's middle from the families' measured metrics; the key hints lost
their own 2-pixel shifts. A unit test checks the constants against the bundled
fonts and the placement; the world shots were re-rendered and checked zoomed
(buttons, segments, key caps, the badge). The main page's arc entries now sit
on their arc points (they drew about a tenth of their size high).

## Reports and notes carry the worn name

Branch `personal/hub-names` (07/10/2026, based on `6678967`, Windows 11): Sol
asked for the player's name with every note and its history kept. `BugReport` and
`WorldNote` gain `name`; the identity service fills it from the in-game name it already
sends the hub, and the hub stores it with the row and in the key's worn names
([identity.md](identity.md#world-notes)). The service tests check the name goes out; the
`hub_e2e` note test passed against a local hub. No game was started.

## World notes sent to the hub

Branch `personal/hub-notes` (07/10/2026, based on `57fcc62`, Windows 11):
Sol asked for world notes to reach the hub so players can send them too. The note
dialog keeps the bug reports' alphabet; Send checks the hub's note rules
(`sjk_identity::report::note_text`). Saving a note still writes `notes.jsonl`,
`notes.txt` and the full screenshot, and with the identity on also sends a
`WorldNote` (`POST /v1/note`) through the identity service; the screenshot writer
makes a smaller JPEG (`capture::preview_jpeg`) that follows with
`PUT /v1/note/<id>/image` ([identity.md](identity.md#world-notes)). The hub side
(notes table, pictures, quotas, operator routes) is in the hub repository. Unit tests
cover the note rules, the service sending a note and only its own picture, the
preview's size and the dialog; the sjk-viewer tests passed, and the `hub_e2e` note
test passed against a local hub. No game was started: the whole flow in a running
client (dialog, centre print, picture upload) is unverified.

## SJK UI: What's new, Update and Identity

Branch `personal/sjk-ui-pages` (07/10/2026, based on `57fcc62`, Windows 11):
Sol asked for the What's new, Update and Identity pages in the SJK UI. Each
console page gets a third look ([sjk-ui.md](sjk-ui.md), Sol JK's pages), picked
with the menu style (`ViewerConsole::set_sjk_pages`) and drawn in the UI's
families: the console overlay routes an open SJK page's text through
`TextTarget` (`console_sjk_pages.rs`). Their state, keys and pointer are
unchanged. The SJK UI changelog entry was split into one bullet per screen.
`update::pretend_available` (tests only) sets an available release for the
world shots, which drew the three pages over duel6 (Identity switched off: the
shots' profile never registers). The sjk-viewer tests passed. No game was
started: the pages' pointer targets and the Identity page switched on are
unverified in a running client.

## SJK UI: Key bindings in Settings

Branch `personal/sjk-ui-keys` (07/10/2026, based on `5ad57a0`, Windows 11):
Sol asked for the key-binding menu in the SJK UI. Key bindings is now a category
of the SJK UI's Settings, drawn in the same frame ([sjk-ui.md](sjk-ui.md),
Settings): the key-binding editor's classic+ list with a new view
(`keybind_editor/sjk_view.rs`), each action's two keys as caps (key names in
words: "Mouse 1", "Wheel up", "Up"), the awaited one gold, the detail column
with its keys, default, command and shared keys. The rail and Tab now move
through it like any category; Escape returns to the main page; a menu style
change on it hands over to the classic Controls panel. The Settings frame's
parts (top bar, rail, layout) are shared from `settings/sjk_view.rs`. Also: the
SJK UI Character's slider number now takes clicks (it was registered under its
row, so a click set the slider to its top). Unit tests cover the key labels and
moving into, out of and back to the category; the sjk-viewer tests passed; the
world shots drew the category and an action awaiting its key. No game was
started: capture and clearing are unverified in a running client.

## SJK UI: Character on duel6's stage

Branch `personal/sjk-ui-character` (07/10/2026, based on `af22dd7`, Windows
11): Sol asked for the character creator in the SJK UI "on the map we have, ...
but our own way" ([sjk-ui.md](sjk-ui.md), Character). duel6 gets a
player stage on the south-west path below the tower and a saber shot (routes
placed with the world shots), reached by cuts on the toured map; the player
screen draws the modern screen's state in a new SJK view: the name as title,
tabs, the form's rows and model grid in a column on the left, the model on the
stage at the right with a gallery caption. The kit gained a cycler, colour
chips, buttons and rank pips. A click on a row outside its control only chooses
the row; slider rows map the pointer over their track. From a game, where there
is no stage, the screen keeps its classic pages. Unit tests cover the layout's
fit, power names in sentence case and the slider's pointer mapping; the
sjk-viewer tests passed; the world shots rendered the three pages in the SJK UI
and the modern screen on the stage. No game was started: the pointer on the new
view, the cut's timing and the model at other window sizes are unverified in a
running client.

## SJK UI: the camera tour of mp/duel6, and off-screen world shots

Branch `personal/menu-camera` (07/10/2026, based on `39323fe`, Windows 11):
Sol asked for more camera angles from the map behind the menu. The backdrop plays
a tour on maps that have one ([sjk-ui.md](sjk-ui.md), The map behind): duel6 has
ten 15-second glides with fades between, opening on the intermission view, the
rest shuffled every start. A screen whose shot has a route is reached by a cut
through dark on such a map. To place the shots without starting the game,
`world_shot` builds the client windowless and renders full frames into an image
it reads back to PNG (`GpuState::headless_frame`, only set by those tests); its
ignored tests drew duel6's plan from the BSP, views from every spawn, candidate
shots, the tour's starts and ends (each glide traced clear of brushes) and the
SJK UI over the touring map. The first run had the throwaway profile's identity
on (its default), so it may have registered one throwaway key with the hub; the
harness now switches the identity and the update check off. Unit tests cover the
tour's order (the map's view first, every shot once a round, no repeat across
rounds) and a shot's glide and fades; the sjk-viewer tests passed. No game was
started: the tour's pace and fades are unverified in a running client.

## SJK UI: Settings

Branch `personal/sjk-ui-settings` (07/10/2026, based on `be3b45d`, Windows
11): the SJK UI's second screen, Settings ([sjk-ui.md](sjk-ui.md)), as the
mock-up draws it: the categories down a lit rail with their icons (First setup,
Display, Graphics, Sound, Mouse, Key bindings, Gameplay, Interface, HUD,
Scoreboard, Network), the rows under sub-headings with the kit's switches,
sliders, segments, fields and lists, the focused setting explained on the right
and the search at the top, over the map. It is the classic+ panels' state with
its own view, so search, lists, defaults and typed numbers are theirs. Graphics
gathers the four renderer tabs (`Group::Graphics`); Key bindings opens the
classic+ ones and comes back. The main page's Settings opens it, and First setup
at start opens it on First setup. The menu snapshots' rasterizer now rounds
outlines' corners as the renderer does. Unit tests cover the Graphics group, the
rail against the classic Setup groups, Tab past Key bindings, opening, switching
and closing, the way back from the key bindings and First setup in the SJK UI;
the sjk-viewer tests passed. Off-screen snapshots drew Interface with a changed
slider, Display with a list open, a search, Graphics scrolled, and 4:3. No game
was started: the pointer, the pickers over it and the map behind are unverified
in game.

## SJK UI: the ring main page and recent servers

Branch `personal/sjk-ui-ring` (07/10/2026, based on `f780bd8`, Windows 11):
Sol went back to direction A for the SJK UI's main page ([sjk-ui.md](sjk-ui.md)):
the emblem in its turning ring at the left, the menu on an arc round it (Play, Sol
JK and Quit open pages of their own on the ring), and the servers joined last in a
column on the right, kept in `recent_servers.json` and recorded when a join reaches
the game; the horizon line and its saber are gone. The page is laid out on a 16:9
frame that scales down on narrower windows. Unit tests cover the frame at six window
sizes, the arc's clearance of the ring and column, pages opening and closing,
keyboard and pointer paths to the servers, the recent list (order, cap, merge of
names, reading back, a broken file) and the relative times; the sjk-viewer tests
passed. Off-screen snapshots drew the main, Play and Sol JK pages, a server focused,
the first-start suggestion and 4:3. No game was started: recording a join, the
live map behind the page and joining from the column are unverified in game.

## SJK UI: main page

Branch `personal/sjk-ui` (07/10/2026, based on `afd3625`, Windows 11): a
third menu style, `ui_menuStyle sjk`, SJK's own menus redesigned from the ground up
([sjk-ui.md](sjk-ui.md)), with its main page: SJK's emblem in a turning holo ring
over the live map (mp/duel6 when the client starts in it), the menu on a horizon line
whose chosen part ignites in the player's saber colour, the ring's gold arc pointing
at it, and the chosen item's actions under the line (Join JoF, Rejoin the last server,
Browse servers, Create a game under Play). Its type is SJK's site's, Rajdhani and
Exo 2, bundled and rasterized into their own atlases. Every other screen opens in its
classic+ version. Unit tests cover the style's parsing and fallbacks, the page's
layout at six window sizes, its keyboard and pointer paths, the last-server rule, the
arc's easing and the blade's ignition, the blade colour from the profile, the ring
picture and the key hints; the sjk-viewer tests passed and workspace clippy reports
nothing in the changed files. Off-screen snapshots drew the page on Play, Settings
(an action focused) and Quit at 16:9 and on Play at 4:3, over the JoF HD wide
levelshot of mp/duel6. No game was started: the live map behind the page, the
fonts' first load (synchronous, on the first frame in the style), joining from the
page and the switch between styles are unverified in game.

## Classic+ settings controls

Branch `personal/settings-modern` (07/10/2026, based on `fe9face`, Windows
11): the classic option panels draw their own controls instead of retail's YES/NO
text and slider art: switches for on/off settings, slim sliders with their value in
a frame, segments for choices of two or three values, fields with a caret for longer
lists, sentence-case labels with brighter values, a soft band on the focused row,
a gold dot on changed rows and a reset arrow on the focused one, sub-headings in the
Game options, Interface, HUD and First setup groups, and the group's icon in the
detail box (the key bindings' category icon too). A switch flips and a segment sets
on the click; only longer lists open a dropdown. See
[client.md](client.md#menu-style) and [classic-plus.md](classic-plus.md). Unit tests
cover the control tokens, flipping and segment setting without a dropdown, the reset
button, the group headings and every group's icon; the sjk-viewer tests passed.
Off-screen snapshots drew the Interface, HUD (in game), First setup, Image and Game
options panels in Inter and in SJK Menu (`ui_gameFont 1`). No game was started:
hover and click feel, the switch and segment clicks and the reset arrow are
unverified in game; the modern settings screen is unchanged.

## Saber wall cutoff

SJK pull request #24 by Lumaya, branch `fix/saber-wall-cutoff` (based on
`cfbc789`, 2026-10-07, Windows): blade contact clips the glow and core as well
as the trail, including with trails or wall marks disabled. Reference: local OpenJK
`4d0dfaf1`, `codemp/cgame/cg_players.c`, `CG_AddSaberBlade`'s `saberLen =
VectorLength(v)` after its `MASK_SOLID` trace. Unit coverage checks the render
pair's shortened length and radius and restored extension at 8/7/4/3 ms steps.
In-game appearance and GPU startup are unverified; solid brush entities remain
outside the client contact trace. Workspace build, tests and clippy pass on
Windows with Rust 1.99 (clippy warnings remain). `cargo fmt --all --check`
fails on existing formatting in `sjk-materialgen`'s `classes.rs` and
`generate.rs`; the changed Rust files pass rustfmt.

## Kicks and saber attacks predicted in a joined game

Branch `fix/predicted-animation-lengths`. Prediction had an animation length table
only when the client was started with a player model argument; a game joined from
the menu or the command line had none, so the melee kicks (`g_debugMelee`) and saber
attacks were not predicted and started a round trip late. A joined game now loads
`models/players/_humanoid/animation.cfg` for prediction, as EternalJK hands Pmove
the local player's animation set (`cg_predict.c:1311` at a40e793); reusing the
loaded map for a new gamestate keeps the table the prediction had, instead of taking
the first loaded model's, which could be another player's, an NPC's or a vehicle's.

On a JA+ server, melee's W+A and W+D kicks are replaced by JA+'s own spin and back
kicks (`BOTH_MELEE_SPINKICK`, `BOTH_MELEE_BACKKICK`), which JA+ plays server-side.
JoF EternalJK holds the kicker still while those and JA+'s other own animations play
(`bg_pmove.c:12470-12498`, `SVMOD_JAPLUS`, at bd5e202); the client kept predicting
the held movement, so every snapshot pulled the player back through the kick.
Prediction now holds still for them, and locks the view for a kiss, a ledge, a
get-up, a stab or a backflip kick taken, as JoF EternalJK does. Unit tests cover the
table for a joined game, its hand-over when the map is reused, and the JA+ holds.
Formatting, the locked workspace build and tests pass; workspace Clippy finishes
without errors, and its warnings are all in code this change does not touch.
Checked in game on Windows 11 against a local JA+ server (`openjkded` with the JA+
module, `g_debugMelee 1`): melee's W+A and W+D kicks play smoothly. Other server
types are not checked.

## Actor instance buffer capacity

The shared actor instance buffer holds 4,096 instances, up from 1,024 (the old
size plus the pickup and effect overrides could not hold a busy frame). Packing
stops at the capacity, so a draw range never reaches past the buffer; before,
a frame with 1,026 instances aborted the client with a wgpu validation error
(reported on a JoF server after about 31 minutes). Which map and entities
produced the 1,026 instances was not captured. Covered by unit tests of the
packing only; not run in game.

## EFX keys without a value

Branch `fix/efx-bare-key-brace`: in an EFX block, a key with no value no longer
consumes the block's closing brace. JoF's HD `effects/concussion/shot.efx` has a
bare `linear` inside its Light's `size { }`; the parser skipped the next token as
its value, took the `}` with it and rejected the effect ("unterminated effect
component"), so the concussion rifle's main shot drew no trail. Retail's
line-based parser ignores such a key. With the fix all 139 effect files of
`JoF_HDWeaponEffects.pk3` parse. A unit test parses the HD block. Checked in game
on Windows 11: the concussion shot shows in flight.

## Image extension order

Branch `fix/image-extension-order`: when an image name has no file of its own
(no extension, or the named file is missing), it is tried as `.jpg`, `.png` and
then `.tga`, the order rd-common registers its loaders in
`R_ImageLoader_Init` (`tr_image_load.cpp:93-95`), instead of `.tga` first. As in
`R_LoadImage` (`tr_image_load.cpp:104-139`), the extension named in the script is
still tried first, so a script that names `base.tga` keeps loading the `.tga`
when it exists. HD packs that ship a `.png` or `.jpg` beside the base `.tga` take
effect for extensionless names (for example the disruptor scope in
`JoF_HDWeaponScopeTrue.pk3`). Unit tests in `sjk-shader` cover the order and the
named-extension-first rule. Not verified in game.

## Detached free camera

Detached free camera (`feat/free-camera`): `/freecam [on|off]`
uses the existing JoF EJK fake-noclip predictor and frozen server commands,
with the body left at its entity and the view weapon hidden. Within 32 units
of its eye the body is hidden from the main view to avoid head clipping. The existing
flight tests cover server command suppression and return-to-server behavior.
Death/spectator/vehicle policy is shared with fake noclip. Combined revision
`7b76890` was checked on 2026-10-07 (Windows, RTX 5070 Ti, release, retail
`mp/duel1`, isolated native server, 1280x720): forward/back flight moves the
camera while the body stays behind with the talk icon; turning off returns
to the server position. The initial view is clear, the body is visible after
moving away, and enabling while dead is refused. Turbo/vertical input,
vehicle/spectator/map transitions and all command-step rates remain unverified.

Review follow-up against `c096c64`: free camera blocks generic actions,
weapon switching and Force-wheel reliable commands, clears on all session exits,
and can be cancelled from menus. Body sounds, shadows and nameplates use the
authoritative entity. Command-policy tests cover 8/7/4/3 ms steps; these are
input-policy checks, not movement-parity certification. Windows workspace fmt,
build, tests and clippy pass with existing warnings. Combined revision `44ff6e5` was checked on 2026-10-07 (Windows, RTX 5070 Ti,
release, isolated native server, retail `mp/duel1`, 1280x720): backward
flight separates the view from the body and off returns to it. Disconnect
clears `cg_freeCamera`; off succeeds from the menu and reconnect starts
with the cvar at 0. Reliable-wheel suppression, kick/timeout/map/vehicle
transitions and precise body-effect placement remain unverified in game.

## Mouse wheel in the key-binding form

Branch `fix/bind-mouse-wheel`: the mouse wheel binds in Settings > Key bindings
(for example `flipkick` under Movement). A notch while a slot waited for a key
scrolled the list instead; it now binds `MWHEELUP` or `MWHEELDOWN`, as the retail
controls menu binds any key event. Wheel deltas under half a notch (small trackpad
pixel deltas) do not bind. Unit tests cover both directions, that the wheel does
nothing to the form when no slot waits, and the threshold. Checked in game on
Windows 11 before the threshold was added; the threshold is covered by unit tests only.

## Timed player peek

Refreshed peek (`feat/refreshed-peek`): timed player camera
from JoF EJKSol `f57d678`, `CG_Peek_f` and `CG_CalcViewValues`, with explicit
cancel, crosshair selection, unique name fragments and target-loss handling.
Only received players can be watched. Name/duration policy has unit coverage;
combined revision `7b76890` was checked on 2026-10-07 (Windows, RTX 5070 Ti,
release, retail `mp/duel1`, isolated native server and Alora bot, 1280x720).
Monitoring by name shows the target from behind and the local body at its
entity; explicit cancellation and a one-second timeout return to the player.
Starting peek during free flight is refused. Wall-collision edge cases,
crosshair selection and slot/map transitions remain unverified.

Review follow-up against `c096c64`: missing snapshot data pauses monitoring
until the original timer expires; local flight cancels it. Detached rendering
is scoped to this view and restores the preceding camera state, without
reading the free-camera cvar in the main render path. Empty/colour-only names
are rejected. Windows workspace fmt, build, tests and clippy pass with
existing warnings; temporary target loss/return has unit coverage. Combined revision `44ff6e5` was checked on 2026-10-07 (Windows, RTX 5070 Ti,
release, isolated native server, retail `mp/duel1`, 1280x720): starting
freecam during a player-name peek returns the view to the local camera.
PVS/portal pause and resumption remain unverified in game.

## Centre-print line breaks

Branch `fix/center-print-space-break`: centre-print rows break only at a space, as
`BG_IsWhiteSpace` counts only the space. A vertical tab (0x0B) in
`{JoF}\vToxiee\v{C}.ak` broke the row inside the name, where EternalJK keeps the
name whole. A unit test covers that name. Checked in game on Windows 11: the name
stays whole on its own row.

## First-person saber body

First-person saber body on `fix/first-person-saber`:
the local posed body and arms render while head variants and pilot hoses are
masked (JoF EJKSol's `CG_ForceFPLSPlayerModel`, local reference). Visibility
policy is unit-tested; Windows Rust 1.99 workspace build/tests/clippy pass
with existing warnings. At the initial revision, formatting failed on unrelated
materialgen files; current main has fixed them. Combined revision `7b76890` was smoke
checked on 2026-10-07 (Windows, RTX 5070 Ti, release, retail `mp/duel1`,
1280x720): first-person blade rendering has no head obstruction and the head
returns in third person. Arm framing, head-bolt camera placement, community
models and mirrors remain unverified; mirrors share the head mask.

Review follow-up against `c096c64`: death, spectator and follow states hide
the first-person saber body; shared fallback models stay hidden. Body and
head masking use one filtered equipment decision, including weapon loss.
The mask covers the whole head subtree. Policy and hierarchy fixtures cover
these cases without retail assets. Windows workspace fmt, build, tests and
clippy pass with existing warnings. Combined revision `44ff6e5` was checked on 2026-10-07 (Windows, RTX 5070 Ti,
release, isolated native server, retail `mp/duel1`, 1280x720): death and
spectator follow views have no headless saber body at the camera. Community
models, mirrors and exact arm framing remain unverified.

## Saber clash flare

Branch `fix/saber-clash-flare`: every saber clash flashed the whole screen
yellow-white. The flare asked the effect atlas for `gfx/effects/saberFlare`, but the
atlas keys shaders in lower case, so the lookup missed and drew the fallback spark
picture at the flare's size (up to 2.35 times 600 virtual units). Atlas lookups are
case-insensitive now, without allocating. The flare's width is also scaled by
EternalJK's `widthRatioCoef`, so it stays round on wide screens (`CG_SaberClashFlare`,
`cg_draw.c:7149-7208`). Checked in game on Windows 11 on a JoF server: a clash shows a
short glow where the sabers meet.

## Scoreboard rows on full servers

Branch `fix/scoreboard-rows`: on servers with more than 20 players, many scoreboard
rows showed one player's name (client 0) with score, ping and time 0. The `scores`
count is every connected client, but the game sends at most 20 rows
(`MAX_CLIENT_SCORE_SEND`, `DeathmatchScoreboardMessage`), so the check that picked
jaPRO's 15-field rows (with deaths) failed, and they were read as 14 fields, shifting
every later row. The row width now comes from the fields actually sent; when both
widths divide them, the one whose rows read as players wins. See
[networking.md](networking.md). Unit tests cover full stock and jaPRO servers and a
length both widths divide. Checked in game on Windows 11 on JoF's full server.

## EternalJK camera damping

Branch `feat/camera-fps`: the third-person camera damps as EternalJK's does. Stock
damping eases the camera once per 50 ms; EternalJK eases once per frame of
`cg_cameraFPS` (default 125), compensating for the ideal point's own movement, so
the result does not depend on the frame rate. SJK registers `cg_cameraFPS` (a
float) and follows EternalJK above 15; `cg_cameraFPS 0` (below 15) keeps the stock
damping. The damping is timed by the predicted command time, the clock the focus
moves on; timing it by the presentation clock made the camera stutter. JoF EJK's
look also needs `cg_fov 90` and `cg_thirdPersonRange 80`. Unit tests cover one
125 fps step against EternalJK's per-frame formula, frame-rate independence with a
still and a moving ideal point, the stock path at 0 and no elapsed time. Checked in
game on Windows 11 on a JoF server next to EternalJK, with the default 125;
`cg_cameraFPS 0` was not tried in game.

## Windows-1252 symbols on screen

Branch `fix/windows-1252-display`:

- Glyphs are chosen by Windows-1252 byte and the modern font's slots 0x80..=0x9F
  hold the Windows-1252 characters. Typed `€`, `’`, `‘`, `™` or `—` drew `?` (the
  glyph was chosen by Unicode value below 256), and the same symbols received from
  other clients drew blank.
- Chat keeps those symbols and other control bytes in names for display; it
  dropped them as control characters. The chat roster keys each player by the name
  with bytes 0x80..=0x9F mapped to their characters and other controls removed, so
  Friend and `tell <name>` work for those players.
- A slot with no glyph (a vertical tab in a name, an unassigned byte) draws `.` in
  the modern font and the retail `.fontdat` fonts, as OpenJK `RE_Font_DrawString`
  does: `{JoF}\vToxiee\v{C}.ak` reads `{JoF}.Toxiee.{C}.ak` as in EternalJK.
- The classic console leaves typographic characters (0x80..=0x9E), which the retail
  console character set lacks, out of the row as EternalJK's console does, while
  the text keeps them.
- JoF cosmetic wildcard keys compare bytes, so a model name with a multi-byte
  character across the prefix length no longer panics. An audit of client-side
  slicing of player and server text found no other site that can split a character.

Unit tests cover every byte's round trip, common name symbols typed and received,
roster keys and `tell` lookups, and the cosmetic match. Checked in game on Windows
11 on a JoF server, before the roster keys were added; those are covered by the
unit tests only.

## Game fonts: SJK Menu, SJK Chat and SJK HUD

Branch `personal/vector-game-fonts` (07/10/2026, Sol's request; SJK draws no bitmap
fonts, docs/sjk.md "Fonts"). The retail `ergoec`, `ocr_a` and `arialnb` bitmaps and
the logo glyph are replaced by three bundled TrueType fonts that keep the retail
layout: SJK Menu and SJK HUD traced from the JoF HD pack's atlases, SJK Chat OCR-A
set to the retail widths with Latin-1 composed. `ui_gameFont` and
`cg_classicHudFont` no longer read fonts from the game data. Unit tests cover the
line metrics, whole-pixel advances, the baseline, Latin-1 coverage, the `.`
fallback and the logo; the fonts were compared with the retail and HD atlases in
offline renders (Python, Pillow). Workspace build and tests pass on Windows; the
workspace clippy warnings are the existing ones, none in the changed code. Not run
in a game.

Branch `personal/chat-font-spacing` (07/10/2026, Sol reported uneven chat
spacing): the first build had left most glyphs of all three fonts flush left
(`removeOverlaps` moved outlines to left bearings that were still 0, see
docs/sjk.md "Fonts"), and SJK Chat now centres each OCR-A glyph in its retail
advance instead of on the unevenly padded retail atlas cell. Advances, line
metrics and the baseline are unchanged. A unit test checks that chat letters and
digits sit centred in their advance (it fails on the first build); compared in
offline renders before and after. Not run in a game.

## Console font: JetBrains Mono

The console character set's surfaces (the console, notify lines, FPS, vote, team
overlay, kill feed) draw with JetBrains Mono, bundled (OFL) and rasterized once at
startup like Inter, instead of the `charsgrid_med` bitmap, which looked heavy and
blocky at 4K (07/10/2026, Sol's request; SJK draws no bitmap fonts, docs/sjk.md
"Fonts"). The font keeps the 8 by 16 unit cell, so layout, wrapping, selection and
`con_scale` are unchanged; the insert and overstrike cursors are solid shapes in the
retail cursor proportions. This replaces EternalJK's character set below: the
`japro-assets.pk3` probe is gone. Unit tests cover the cell metrics, the atlas and
the cursor shapes, and the atlas was rendered offline to check placement; not run
in a game.

## EternalJK's character set (replaced 07/10/2026)

Replaced by the console font above. Branch `feat/eternaljk-charset`: when `GameData/EternalJK` holds a PK3 with
`gfx/2d/charsgrid_med` (jaPRO's `japro-assets.pk3`), that one image is mounted above
the game's `base`, and below the `fs_basegame` and `fs_game` directories, as EternalJK
mounts its folder above `base` and below the mod. The PK3 probe runs once per process. It has `¬`, `¥`, `²`, `½`
and the rest of Latin-1, which the retail set lacks. It replaces the set for all text
that uses it (the console and the character-set text of `game_font.rs`), as in
EternalJK. Unit tests cover that it overrides the base set, that a mod's own set still
overrides it, and that nothing else is mounted from the pack. Checked in game on Windows 11: the console shows `¬¬¬` and `¥²½` as in
EternalJK.

## EternalJK player animation fixes

Implemented:

- Players' animations are remapped before they are shown, as JoF EJK's
  `CG_Player` does (`cg_players.c:10665-10730` at EternalJK a40e793): without a
  saber in hand, two-handed, dual and staff runs and walks play as the ordinary
  ones; a thrown saber's standing torso follows the legs; the old Bryar's
  `BOTH_STAND1` shows as `BOTH_ATTACK2`; the concussion rifle's `BOTH_ATTACK2` as
  `BOTH_ATTACK3`. NPCs are left as sent. The own player's pose reads the remapped
  animations too.
- Prediction fires with the attack table the server runs: OpenJK's
  `BG_FixWeaponAttackAnim` (`codemp/game/bg_misc.c:297-346` at OpenJK 260c59c)
  changes four entries (concussion, old Bryar, emplaced gun, turret) only with
  `CS_LEGACY_FIXES` bit 1 (`g_fixWeaponAttackAnim`). SJK always predicted the fixed
  table, so against a server without the fix the torso restarted on every snapshot.
  The fixed entries for the emplaced gun and turret (915, 113) are OpenJK's and
  EternalJK's (`bg_misc.c:433-458`). SJK's own server publishes `CS_LEGACY_FIXES` 7
  (all fixes) and runs the fixed table.

Verified: unit tests cover the remaps, both attack tables and the weapon-animation
prediction at command steps of 8, 7, 4 and 3 ms (old Bryar and concussion, fixed and
unfixed table, two seconds of held fire: the shot count and the shot's torso
animation). The table values were checked against OpenJK 260c59c (read on GitHub) and
EternalJK's source tree. `cargo fmt`, build, test and clippy pass (06/10/2026,
Windows 11).

Not verified: no run in game and no run against a stock, JoF or SJK server by this
revision, so which servers publish which `CS_LEGACY_FIXES` value is taken from the
sources, not from a capture; the remaps are checked only by unit tests against
`CG_Player`'s logic. Windows CI and the parity evidence in `docs/development.md`
were not run. The emplaced gun's fire is `pmove_emplaced`'s and the turret is not
predicted, so their table entries are not exercised by prediction.

## Effect atlas mipmaps and impact marks

Branch `fix/effect-atlas-mipmaps`:

- The effect atlas has five mip levels, as rd-vanilla's images have mipmaps. Without
  them a small or far mark sampled a few texels of its 128-pixel picture and showed
  as a hard black dot on walls. Each picture sits in a 160-pixel cell whose 16-pixel
  border repeats its edge texels, so filtering down to the 8-pixel level never
  reaches the next picture, and each level averages its 2x2 parents weighted by
  alpha, so transparent texels do not darken the visible ones.
- Impact marks are drawn before every other effect, as rd-vanilla sorts mark
  shaders (`sort decal`) ahead of blended effects, so an explosion's fire and smoke
  cover its own scorch mark.
- Effect atlas stages honour `alphaGen const` and a grey `rgbGen const`; JoF's HD
  scorch marks are 15% grey at 80% opacity and were drawn fully opaque.

Unit tests cover the constant colours, the alpha-weighted levels and that two
neighbouring pictures never mix at any level. On Windows 11 the mipmaps removed the
black dots; the gutter, the alpha weighting, the draw order and the constant
colours were not checked in game yet.

## Dismemberment and disintegration

Branch `feat/dismember-disintegrate`: cut-off limbs (`cg_dismember`, JoF EJK's
`CG_General` limb case) and bodies burning away (`EF_DISINTEGRATION`,
`CG_Disintegration`), as described in
[client.md](client.md#dismemberment-and-disintegration). `cg_dismember` defaults to
0, as in EternalJK; disintegration is always on, with no cvar, as in EternalJK and
OpenJK. At `cg_dismember 0` with nothing cut the limb update returns at once; it
borrows the presented snapshot and never copies it. Disintegration colours are final:
the real-time sun and fullbright leave them as they are. Surface state is copied from
a player to a body, or to a pooled limb, only between meshes with the same surfaces
and draws. Unit tests cover the surface rules (caps, stump, limb root, root-surface
variants, reattaching, matching layouts), the frozen disintegration pose and the burn
radius; the composed stage shader passes naga validation. Checked in game on Windows
11 on a local server with `g_dismember 100` and `cg_dismember 3`, before the review
changes (snapshot borrow, fullbright colours, layout check), which are covered by the
unit tests only.

## Clipboard symbols on Windows

Implemented: on Windows the clipboard is reached through PowerShell with UTF-8 in
both directions, so `€`, `’`, `×` and other symbols keep their characters
(`a×¥’€…` pasted as `a??'???` through the console's OEM code page before).
Copy tries, in order, `Set-Clipboard` fed from an environment variable (works in
PowerShell Constrained Language Mode; texts up to 30000 bytes without NUL),
`Set-Clipboard` fed raw UTF-8 bytes on standard input (Full Language Mode only),
then `clip`. Paste tries the text's UTF-8 bytes on standard output, then, when
Constrained Language Mode refuses that, the text's UTF-16 code units printed as
numbers and decoded. A tool that exits (PowerShell, `clip`, `pbcopy`) is waited for
up to 1.5 s in total on the calling thread, and a non-zero exit moves on to the next
tool, so a failed copy reports failure. A copy still running after that is left
pending: the next copy stops it (copies never finish out of order) and a paste waits
up to 1 s for it. `wl-copy` and `xclip` keep serving the selection and are not
waited for. Unit tests cover the UTF-8, byte-order-mark and code-unit decoding and
which tool can carry which text.

Verified on Windows 11 (06/10/2026, Windows PowerShell 5.1), by hand and with
`cargo test -p sjk-viewer clipboard -- --ignored` (a real-clipboard round trip of
`a×¥’€…`, CRLF, LF and an emoji, and two quick copies followed by a paste): the
environment copy, the stdin-bytes copy, both pastes and `clip` kept the symbols;
the paste writes UTF-8 with no byte-order mark (the decoder strips one if a tool
adds it); with the language mode set to `ConstrainedLanguage` the stdin-bytes copy
and the byte paste exit 1 (clipboard untouched) while the environment copy and the
code-unit paste round-trip correctly. A PowerShell copy takes about 0.25 s.
Unverified: a machine whose Constrained Language Mode comes from AppLocker/WDAC
policy (it was simulated with `$ExecutionContext.SessionState.LanguageMode`),
`pwsh` as the only PowerShell, and Linux and macOS tools.

## Classic crosshair pictures

Branch `feat/classic-crosshair`: the crosshair is retail's picture,
`gfx/2d/crosshaira`..`j`, drawn as EternalJK's `CG_DrawCrosshair` draws it
(`cg_draw.c:6665-7060`), instead of the procedural cross.

- `cg_drawCrosshair` is an integer clamped to 0..=10: 0 hides the crosshair, the
  picture is the value `% 10`, with `crosshaira` and `crosshairj` swapped as
  EternalJK swaps them with `R_RemapShader`, and 10 is EternalJK's white dot of
  `cg_crosshairSize` pixels. The settings entry is a 0-10 picker.
- `cg_crosshairSize` (default 24, as `cg_xcvar.h:234`) is in 480-line virtual
  units with `cg_crosshairSizeScale 1` (default 1), so 24 * height / 480 pixels,
  kept square on wide screens as `widthRatioCoef` keeps it; it is in pixels
  without scaling and for 10.
- The picture follows the dynamic crosshair and `cg_crosshairX/Y` and takes the
  existing target colours. With the default `cg_crosshairColor` "0 0 0 255" and no
  target it is drawn white, in its own colours, as EternalJK's `R_SetColor(NULL)`.
- The default crosshair colour changed from the old bluish tint (0.964, 0.991, 1.0)
  to white (1, 1, 1): `crosshair_color` returns white for the all-black default and
  for an unparsable `cg_crosshairColor`, and the targeting state starts white. The
  procedural fallback cross takes the same colour, so it is white too.
- EternalJK's own `crosshaira` and `crosshairj` from `EternalJK/japro-assets.pk3`
  replace the base ones, and nothing else from that pack.
- When a picture is missing, or the HUD draw list is full, the procedural cross is
  drawn instead.

The vehicle's doubled size and its own `crosshairShader` picture are done (see
[Vehicle HUD](#vehicle-hud)). Not done: the item-pickup pulse of
`cg_dynamicCrosshair 3`. Unit tests cover the picture
order, the clamp, sizes, the missing-picture and full-list fallbacks and the
EternalJK pack override. Checked in game on Windows 11 before the later
changes, which were not re-tested in game and rest on unit tests and the workspace
checks: the `cg_drawCrosshair` clamp, the full-list fallback, and the merge with
`main` (the crosshair cells now follow `LOGO_ICON` in the icon atlas).

## First-person weapon field of view

Implemented: the first-person weapon follows EternalJK's `cg_fovViewmodel`
(default 80), `cg_fovViewmodelAdjust` (default 1) and `cg_fovAspectAdjust`
(`CG_AddViewWeapon`, `cg_weapons.c:951-1042` in JoF EternalJK). It is drawn with
the world's projection, its forward axis scaled by
`tan(viewmodel/2) / tan(fov_x/2)` (the view-model FOV widened to the screen when
aspect adjustment is on), and it drops by 0.2 units per degree of view-model FOV
over 90. `cg_fovViewmodel 0` draws it with `cg_fov`.

The default 80 is EternalJK's and changes the first-person look for everyone
compared with retail and OpenJK, which have no such cvar and draw the weapon
as with `cg_fovViewmodel 0` (about 0.84 of the depth at the default settings).
Set `cg_fovViewmodel 0` to get the retail look back.

Gun and muzzle: EternalJK scales only the hand's `axis[0]`, and the gun, barrels
and flash inherit that through the tag matrices (`CG_PositionEntityOnTag`,
`cg_ents.c:48-66`), so the whole view model is squashed along the hand's forward
axis. The tag origin is scaled exactly. A rendered instance can only scale its
own axes, so the gun and barrels scale the axis of their frame closest to the
hand's forward axis, and the muzzle socket goes through the same transform
as the gun instance, so socket and barrel stay together. This is exact for the
retail hand rigs whose `tag_weapon` only rolls about the forward axis (bowcaster,
repeater, concussion, disruptor and others). The blaster and pistol rigs tilt
their `tag_weapon` by about 13 degrees, so at the default settings their flash
ends up about 0.45 units from where EternalJK places it (12 units out), against
about 1.9 for scaling the gun's own x. Removing that remainder needs a
per-instance matrix in the actor shaders.

The hand rig's frames are chosen from the posed torso animation together with its
frame, in the same change because the view weapon reads both for each frame:
`CG_AddViewWeapon` reads `lower_lumbar`'s frame with the predicted `torsoAnim`,
and pairing the frame with the snapshot's older torso animation put the hand on
idle frames while the predicted shot already played. This is a separate fix from
the FOV scale and could be its own change.

Verified: unit tests for the FOV terms (21:9 screen, equal FOVs at 4:3, the `0`
fallback, the drop) and for the tag composition (unit scale equals the plain
composition; the roll tag matches an independent row-matrix model of the engine
within 0.05 units; the blaster tag stays within 0.5 units; the muzzle socket
equals the rendered gun's flash point for both tags and several scales).
`cargo fmt --all --check`, `cargo build --locked --workspace`,
`cargo test --locked --workspace` and
`cargo clippy --locked --workspace --all-targets` on Windows 11 (06/10/2026).

Not verified: the author reports a side-by-side check in game against JoF
EternalJK at the same settings; that was not repeated for this review. The
muzzle-flash effect and crosshair were not compared in game at `cg_fovViewmodel`
values other than 80.

## Zoom, scope and effect camera shakes

Implemented, described in [rendering.md](rendering.md#zoom-scope-and-camera-shakes):

- Effect camera shakes are measured from the rendered view, as `CG_DoCameraShake`
  measures from `cg.refdef.vieworg`. `cg_screenShake` (default 2) gates effect-file
  shakes: 0 off, nonzero on. Not implemented: EternalJK's level 2 shakes of a
  charging or just-fired weapon (`cg_weapons.c:793,801,2466`) and the damage view
  kick being switched off by 0 (`cg_view.c:1145`).
- The viewer's own muzzle flash never shakes the camera; other players' flashes and
  explosions do, by distance, as in the reference.
- A zoom puts the view in first person (`CG_DrawActiveFrame`), in EternalJK's order:
  a living player on an emplaced gun, or riding a vehicle with a weapon other than
  saber or melee, is not forced. The effective `third_person` is derived every
  frame from the player's camera choice, so nothing is rewritten or left behind
  when the zoom ends. A demo's spectate, look-at, orbit and free cameras ignore
  the recorded player's zoom. EternalJK also forces third person for the
  knockdown, grapple and fall states; SJK does not.
- The scope mask is the image `gfx/2d/cropcircle2` in the engine's image order
  instead of its shader, whose other stage drew a full-screen white picture.
  **Depends on #17** (image order): without it `.tga` is tried first and JoF's HD
  `cropcircle2.png` does not show. Merge #17 first.
- Included, unrelated to the zoom: the first-person duck smoothing follows the
  predicted view height (`CG_TransitionPlayerState` after prediction,
  `cg_playerstate.c:539-543`); it also followed the later snapshot and lifted the
  view again after each crouch.

Verified: unit tests cover the zoom decision (disruptor, saber/melee, emplaced
gun, vehicle, dead player, the choice surviving a zoom, the demo cameras) and the
predicted duck smoothing; the workspace checks passed on Windows 11
(06/10/2026). The contributor reported in-game checks of the first version on a
JoF server (no shake when firing, the disruptor scope in third person, the HD
scope picture, no crouch bump); the derived camera choice and the own-flash
narrowing were not run in a game, and the HD scope needs #17.

## Weapon selection row in every HUD

Branch `feat/hud-weapon-row`: retail's weapon selection row shows after a weapon
change in every HUD style (game-data, classic, modern and radial), as in
EternalJK, not only with the game-data HUD. While it shows, SJK's own layouts
hide their weapon-name widget and the held-weapon icon at the bottom centre; the
ammunition count, the radial ammunition arc and pill, the classic ammunition line
and the Force power timer icons stay visible. The name transient lasts as long as
the row (1.4 s), so it does not appear after the row ends. SJK's layouts no
longer draw an ammo picture at the bottom centre. See
[rendering.md](rendering.md).

Verified (06/10/2026, Windows 11): `cargo fmt --all --check`, `cargo build
--locked --workspace`, `cargo test --locked --workspace` and `cargo clippy
--locked --workspace --all-targets`. Not verified: the row and the layouts in the
running client (no game was started), including the row's placement against the
radial layout's widgets, and the conditions that hide it (spectating, emplaced
guns).

## Weapon world effects

Branch `feat/weapon-world-effects` (06/10/2026, based on `99bd31d`):

- Stuck trip mines and det packs are turned a quarter about their own Z before
  their facing, so they lie flat against the surface they stick to, facing out.
  The quarter turn matches EternalJK's look; no source for it was found
  (EternalJK's cgame and Ghoul2 renderer add no rotation), so it is unverified
  against the model data.
- An armed trip mine plays its beam (`CG_General`, `cg_ents.c:1789-1814`):
  `tripMine/laserMP`, or `tripMine/glowbit` in proximity mode, from 6.6 units
  out along its facing. The beam's line is an `org2fromTrace` primitive and is
  stretched to the solid its facing hits. A stuck mine does not move, so each
  mine's trace is kept while its position and facing stay the same. At most
  four new traces run per cgame tick; a mine beyond that budget shows its beam
  on a later tick. A beam whose traced segment is more than 8192 units from
  the camera is not played.
- Charging weapons glow at the muzzle as `CG_AddPlayerWeapon` draws it: the
  Bryar pistols' alt fire (`bryarFrontFlash`), the bowcaster
  (`greenFrontFlash`) and the DEMP2's alt fire (`lightningFlash`, 1.75 times),
  growing over a second.
- The concussion rifle's alt fire draws its beam (`EV_CONC_ALT_IMPACT`,
  `cg_event.c:2860-2881`): rings every 64 units, `FX_ConcAltShot`'s `blueLine`
  and `whiteline2`, the wall hit and the disruptor's alt miss. The rings are
  one run in the impact plan, not one visual each. A shot longer than
  EternalJK's `shotRange` of 16384 (`g_weapon.c`, `WP_FireConcussionAlt`) is
  clamped to it, so at most 256 rings are drawn; a non-finite shot vector,
  start or ring direction draws nothing. Stock OpenJK servers fire 8192 at
  most.

Unit tests cover which mines draw a beam, the kept traces and the budget, the
beam cull, the placed charges' facing, the charge glow's sizes, the concussion
ring count, clamp and rejected shots, and the ordinary impact plans' visuals.

Verification: the beam, placed charges, charge glow and concussion beam were
checked in game on Windows 11 on a JoF server by the PR author before the
trace budget, the ring cap, the clamp and the beam cull were added. Those
four, and the plan restructuring, are covered by the unit tests and
`cargo fmt`, `build`, `test` and `clippy` on Windows 11 (06/10/2026) only;
they have not been checked in game.

## Effect diagnostics

Implemented: `fx_debug` and `cg_debugMissiles`, two console diagnostics that
are 0 by default and not archived ([rendering.md](rendering.md#entity-render-effects),
[effect_debug.rs](../crates/sjk-viewer/src/effect_debug.rs)). Both follow their
cvar every frame, including with no snapshot, and log nothing while off.
Unit tests cover the flag following the cvars and resetting without a console.
The workspace checks passed on Windows 11 (06/10/2026). Unverified: the log
output in a running client.

## Classic Settings hub

Branch `personal/settings-hub` (06/10/2026, based on `5c66ccd`): the
classic menus' Controls and Setup are one Settings screen with KEY BINDINGS and
OPTIONS tabs on the panel's title band (main page, navigation rows, profile
pages and the in-game bar, whose Controls and Setup buttons are one Settings).
The main page's SJK button opens a page with Changelog, Credits and Update,
replacing the three corner buttons. KEY BINDINGS shows every binding in one list
under category headings; each panel's first row is a search field (every option
of OPTIONS, renderer included, by name, console name, description or group,
results under group headings; bindings by name, command, category or key);
choices open a dropdown and change only when applied (since `personal/settings-modern`,
only lists of more than three values and the display mode; switches flip and short
choices are segments). Retail's Mods and
Defaults entries are gone and Mouse moved to OPTIONS. See
[client.md](client.md#menu-style). Unit tests cover the page tables, tab
mapping, the bindings list, search and Escape order, the option search's
coverage and grouping, and the panel geometry; the sjk-viewer tests and
workspace clippy (no new warnings) passed. Off-screen snapshots drew the main
and SJK pages, the option search, dropdowns on both frames, the bindings list
and search, and the in-game bar. No game was started: typing in the search
field, the dropdown's pointer handling and the in-game pop-up are unverified.

## Centre prints with non-ASCII names

Branch `fix/center-print-utf8` (05/10/2026, based on `86ad1be`): a centre print
with a row over 50 bytes crashed the client when byte 50 fell inside a non-ASCII
character, for example a duel challenge (`cp`, `PLDUELCHALLENGE`) from a player
named with `×`. Rows now wrap at 50 characters, as OpenJK `CG_DrawCenterString`
wraps 50 code-page bytes. Unit tests cover the wrap at a space, a two-byte
character across the old byte limit (it panicked before the change) and 50 two-byte
characters on one row. `print` and `cp` text was also read as UTF-8, so the same
name showed `?` for each `×`; it is now decoded as legacy text like chat and the
scoreboard (unit-tested). Formatting, the locked workspace build, tests and clippy
passed on Linux and Windows 11. On Windows 11 the old build crashed when the
contributor challenged a player named with `×` on a live server; with the wrapping
fix the challenge showed without a crash, but with `?` for each `×`; with both
fixes it read `You have challenged ×jof.jk.belyash×`. Merged into SJK `main` from
SJK pull request #2 (Creyon94, 06/10/2026).

## Worldspawn shader remaps and remap order

Local change against `af65396` (2026-10-05, Windows 11): a map's worldspawn
`remapshader` keys now apply when its world loads, as rd-vanilla `R_LoadEntities`
does, and the latest of a shader's map, server and local remaps wins, as in
rd-vanilla. A local restore no longer blocks later server remaps, each shader-state
update re-applies its entries, and a server remap that a live `cg_remaps` change
excludes reveals the remap it had replaced. Server time offsets are parsed like C
`atof`; `listRemaps` shows map remaps and marks overridden entries. See
[rendering](rendering.md#server-shader-remaps).

Unit tests cover worldspawn key parsing (prefix, first `;`, the C scan stops),
ordering across sources, self-restore, `cg_remaps` gating and `atof`. Workspace
formatting, the locked build of all targets and the locked tests passed on
Windows 11. Not run in the client: no map with worldspawn remaps, server or demo
has exercised this change, and `vertexremapshader` keys stay unsupported.

## Effect shader remaps

Local change on `af65396` (2026-10-05, Windows 11): effects drawn from the effect
atlas (EFX particles, missile trails, muzzle flashes, beams, impact marks and blob
shadows) follow server shader remaps and local `remapShader` overrides, with the
world materials' precedence. A remapped entry samples a copy of its target's
original stages and time offset, loading a missing target into the atlas first;
removal, a self-remap or `cg_remaps 0` restores it. This runs after the world
applies a remap change, never per frame; sampling adds one subtraction per stage.
No wire code changed. See [rendering scope and limitations](rendering.md#server-shader-remaps).

Workspace formatting, the locked build of all targets and locked tests passed.
New unit tests cover remap planning through the shared world lookup (aliases,
extensions, destination clocks, local overrides, self-remaps, `cg_remaps 0`) and
copying/restoring atlas stages. Unverified: nothing was run in the client, so no
server, demo or visual check exercised remapped effects, atlas growth or the time
offset, and the rebuild time of a grown atlas was not measured. HUD pictures,
saber blades and trails, surface sprites and menu previews still ignore remaps.

## Shader remap restore

Change on `af65396` (2026-10-05, Windows 11): undoing a remap (self-remap,
`cg_remaps 0`, gamestate reset, cleared local override) recompiled the slot through
the replacement path, which treats every slot as a map material. Late entity
materials came back with world gloss, view bounds and light-pass classification,
and sky or colourless shaders lost their loaded stages. The first replacement now
keeps the slot's stages, sort and draw/fog classification aside, with their bind
groups and pipeline indices; restoring swaps them back, followed by the existing
order, fog, range, stage-table, SSAO and caster rebuilds. Native users of a
destination at offset zero are no longer recompiled; a nonzero offset still is.
Replacements and stamps are unchanged, and slots never replaced keep no copy.

Formatting, the locked workspace build (all targets) and tests passed on Windows 11,
including a unit test of the keep/restore/replace decision. Not run in the client:
restores on a live server and their visual result remain unverified.

## Shader remap clearing and setting

Local change against `af65396` (2026-10-05, Windows 11) adds JoF EJK's
`clearRemaps` console command and a Settings > GAME row for `cg_remaps`
(0 off / 1 map / 2 all; the default stays 1). EternalJK's `R_ClearRemaps_f`
(`codemp/rd-vanilla/tr_init.cpp`) resets every renderer shader's remap and keeps
destination time offsets. The client clears the live or demo session's server remaps and
local overrides the same way, sends nothing to the server, and lets a later
shader-state change, reliable `remapShader` command or new gamestate apply again.
See [shader remap controls](client.md#shader-remap-controls).

Workspace formatting, locked build of all targets and tests passed on Windows 11,
including a unit test for the clear logic. Not run in the client: material
restoration after `clearRemaps` and the Settings row were not checked in game.

## Leading-slash asset paths

Change on `af65396` (2026-10-05, Windows 11): VFS reads (`read`, `contains`,
`read_from_mount`, cache identity) drop one leading `/` or `\` before the lookup,
as OpenJK `FS_FOpenFileRead` (`codemp/qcommon/files.cpp`) does. A JoF map's
`/models/items/a_pwr_converter.md3` item model failed to load with "virtual path
must be relative". Mounted names, PK3 entries and directory listings keep refusing
a leading slash, and a second one still fails as before. Unit tests cover the path
rule and a read through a mounted source; formatting, the locked workspace build
and tests passed. Not run in the client: the item on that map is unverified.

## Server shader remaps

Local changes on `dc36792`, verified on Linux/RADV on 2026-10-05, add initial and
live multiplayer shader-state consumption, reliable `remapShader` commands,
demo playback support, destination animation offsets and Tayst-style controls.
`cg_remaps` defaults to 1 (exclude player-texture configstring remaps); 0 disables
server remaps and 2 includes player textures. The setting applies live.
`listRemaps` and temporary local `remapShader` overrides are available.
See [rendering scope and limitations](rendering.md#server-shader-remaps).

External evidence used the unmodified OpenJK multiplayer
`CG_ShaderStateChanged` function: 6,000 valid entries matched the compatibility
parser's result, including extension/case variants, repeated sources, shared
clocks and truncated tails. Separate checks covered one-hop aliases, self-reset,
empty configstrings, nonfinite offsets, policy filtering and gamestate reset.
A 400-snapshot synthetic demo derived from a local recording exercised a direct
remap command followed by reapplication of an unchanged shader-state string.
Both its decoded state and offscreen blue-to-green rendering passed.

An isolated local Tayst server and an original small BSP verified join-time green
replacement, live pulsing material, self-reset to red, enable/disable, translucent
local replacement, and the default/player-inclusive policies (24 player material
slots changed when enabled). Automated checks used an ALSA null sink and isolated
zero-volume settings. Simple remap updates measured 0.5–0.8 ms. One settled
2,048-frame sample at 1280×720 measured 0.419 ms mean and 0.609 ms p99 CPU frame
work; this is a small fixture, not a populated-server benchmark, and excludes cold
pipeline compilation. No wire, movement or combat rules changed. Workspace
formatting, locked build/tests and the optimized Linux viewer build passed.

Shader replacement does not imply geometry editing. Existing server entity and
sub-BSP paths were inspected but not changed. Arbitrary HUD and generated sprite
remaps, full lighting reconstruction and broad custom-map parity remain outside
this implementation (effects: see above); these limitations are recorded on the
rendering page.

## Third-person prediction-error focus

Local change on `af65396` (2026-10-05, Windows 11): the third-person camera adds
the decaying prediction error to its focus before the collision and damping traces,
following `CG_CalcViewValues` (OpenJK `codemp/cgame/cg_view.c:1597-1608`), instead
of adding it to the finished camera position. First-person, intermission and free
views are unchanged. This completes the earlier camera and vehicle change, which
had left it out. Workspace formatting, the locked build and tests passed. No client
was run here and no harness compared it with the C camera; SJK's builds have used
it since the camera and vehicle change was merged.

## Third-person camera collision and vehicle framing

Local change against `dc36792` (2026-10-04): restore the multiplayer camera's
collapsed-target fallback, pitch bounds/offset sign, rapid-turn damping,
vehicle-authored framing and mount/teleport resets. Camera collision now includes
presented inline doors/platforms and excludes BODY from the stock camera mask.
Vehicle definitions are read at appearance loading, with no per-frame file reads.
See [client camera behavior](client.md#third-person-camera).

An external harness compiled the six original OpenJK `codemp/cgame/cg_view.c`
camera functions and compared 10,000 camera frames at 8/7/4/3 ms. Cases include
open space, confined rooms, fully collapsed traces, rapid yaw, animal offsets,
vehicle overrides, pitch-dependent offsets, fighter strafing, unrestricted pitch
and sideways offsets. Maximum component error was 0.000132; this is a floating
point tolerance comparison with shared synthetic trace conditions, not full
client parity certification.

A separately compiled original BSP exercised the production collision adapter:
player-clip obstruction, a translating inline door and ignored packed vehicle
bodies passed. All 8,640 low-ceiling/corner camera frames remained finite. The
optimized camera/collision microbenchmark took 336 ns/frame on this small fixture;
it does not establish populated-map frame performance. Workspace formatting,
build and tests passed, and an optimized Linux viewer was built. Offscreen Vulkan
validation exercised the confined fixture and a mounted stock swoop on an
isolated loopback server. Broader vehicle/mod playtesting remains open; vehicle
rider animation is outside this camera correction. The follow-up below addresses
local vehicle model prediction.

### Local mount presentation follow-up

The local pilot's vehicle model and rider seat now consume its committed/per-frame
vehicle prediction, instead of interpolating old snapshot transforms behind the
predicted camera. This follows OpenJK `codemp/cgame/cg_ents.c`:
`CG_AddPacketEntities` publishes `cg.predictedVehicleState`, and
`CG_CalcEntityLerpPositions` bypasses snapshot interpolation for that vehicle.
No movement, command quantization, animation timing or wire code changed.

An external harness exercised the production vehicle placement/seat module for
8,000 frames at 8/7/4/3 ms, with 50 ms snapshots and 100 ms simulated delay.
The old vehicle-root lag reached 38.735 units; the new root exactly matched the
predicted input. Rider bolt placement, remote/demo fallback and unrelated-vehicle
isolation passed. These are presentation fixtures, not a network or physics
parity certification. Workspace format/build/tests and the Linux release build
passed. An isolated offscreen Vulkan session mounted, moved and turned a stock
tauntaun; a recorded demo confirmed mounted state throughout all 53 snapshots.
Owner testing found continued severe jitter. The follow-up investigation found
that the vehicle's snapshot body remained in its own prediction collision list:
movement started all-solid, then corrected at the next server snapshot. The
adapter now applies vehicle skip/ownership exclusions. Rider-based error decay
also incorrectly treated gait motion as a prediction miss; it now measures the
vehicle root, as stock does. Presentation samples the animated driver seat every
frame instead of holding the snapshot-time offset.

The unmodified OpenJK `CG_VehicleClipCheck` confirmed pilot/own-vehicle exclusion
and foreign-vehicle collision. A production collision-adapter fixture reproduced
all-solid before the fix and clear motion afterward across 4,000 hull sweeps at
8/7/4/3 ms; foreign ownership and dismount restore collision. Another 4,000 frames
using the installed tauntaun's real skeleton reduced the maximum seat step from
5.8814 units (snapshot-held) to 0.9411 (frame-sampled), with no gait-induced
vehicle prediction correction. Seat evaluation measured about 2 microseconds per
frame in the optimized harness; this is not a populated-server performance claim.
In isolated Linux Vulkan play, the same eight-second riding/turning input sequence
went from 19 logged corrections above eight units (maximum 27.78) to none after
the self-collision fix. The final demo confirmed mounting in all 183 snapshots.
Workspace format/build/tests passed. This closes the reproduced self-collision
fault; owner playtesting and broader vehicle/mod coverage remain open.

The exhausted-boost follow-up found another reset: snapshot reseeding copied the
fresh vehicle template over client-only runtime state, losing turbo expiry and
recharge. The same ride now retains that state, as multiplayer cgame retains its
`Vehicle_t`; a changed entity, pilot or definition resets it. An external harness
compiled OpenJK's unmodified `AnimalNPC.c` `ProcessMoveCommands` and compared
27,237 production prediction steps with 50 ms reseeds at 8/7/4/3 ms. Held boost
through expiry/recharge, with alternating primary attack, matched all speed
samples within 0.001 units/second. Separate identity checks verified retention
for the same ride and resets for changed vehicle, pilot and definition.

A silent, headless Linux Vulkan client/server check at 1280x720 and 125 FPS held
boost while turning and attacking for 32 seconds. Logged corrections above eight
units fell from 75 (maximum 17.23) to one (17.10, at a boost transition). Demos
confirmed mounted state in all 696 baseline and 697 corrected snapshots, with
180 boosted snapshots in each. This fixes the repeated exhausted-boost mismatch;
it does not establish zero correction at boost boundaries or full vehicle parity.
Workspace format/build/tests and the owner release build passed. Automated checks
used isolated zero-volume settings and an ALSA null sink from startup.

## Vehicle assets, boarding and native sand-creature AI

Local work on `dc36792`, verified on Linux on 2026-10-05:

- The appearance loader now uses the shared `.veh` parser. A commented example
  in retail `template.veh` previously selected an X-wing for `tie-fighter`.
  Direct lookup and native Vulkan play now load `models/players/tie_fighter`.
- Vehicle weapon indices now resolve `.vwp` EFX and rigid projectile models.
  The local AT-ST firing check produced 243 primary and 86 alternate missile
  samples, selecting `atst/shot_red` and `atst/side_alt_shot` respectively.
  Replaying that capture at 8/7/4/3 ms exercised 6,345 presentation frames with
  correct authored model selection and no default rocket model on laser shots.
  Optimized effect dispatch averaged 66 ns/frame on that small capture; this
  does not establish populated-map performance. Vehicle flight-loop audio is
  not changed in this pass.
- Landing/standing boarding follows multiplayer conditions. The original C
  landing branch matched 20,090 eligibility cases. Production prediction at
  8/7/4/3 ms emitted one authoritative request per landing and none for client
  prediction. A loopback capture confirmed the player boarded the tauntaun by
  landing without a use-key press. Broader mod vehicles remain unverified.
- The reported missing glider/minemonster meshes were absent community content
  in the newer local installation. Mounting the existing pack restored their
  own models. Its malformed glider animation remains a content limitation;
  other actors retain the existing error isolation.
- Native sand-creature AI is enabled only outside stock-rules mode, as described
  in [server.md](server.md#vehicle-boarding-and-sand-creatures). An isolated native
  capture confirmed hidden pursuit, `BOTH_WALK2` breach, both attack animations,
  a normal player death and a successful visible respawn. A second server with
  stock rules enabled retained visible generic NPC behavior throughout all 154
  post-spawn snapshots, with no native ambush. This is a separate
  server extension inspired by SP, not a change to multiplayer class IDs or a
  claim of full single-player AI parity.

Automated native checks used headless Gamescope/Vulkan, isolated settings, all
volumes zero and an ALSA null sink. No public server, owner profile or running
owner game was used for these checks. Formatting, locked workspace build/tests
and optimized client/server builds passed. The workspace has no bundled gameplay
tests; the external checks above provide the focused evidence. Wire encode/decode
paths are unchanged.

## Actor animation error isolation

Local fix based on `3a70c22` (2026-10-04): a custom glider's run clip ends at
frame 235 in a 180-frame skeleton. Its evaluation error previously returned
before the shared joint upload, freezing otherwise healthy actors until the NPC
disappeared. Preparation and upload now contain errors per actor, retain its last
uploaded pose, suppress failed-frame animation audio and log once per failure
episode. Valid animation selection, timing, movement and wire code are unchanged.
The malformed custom clip is still rejected; no content files are modified.

An external headless Vulkan check on the RX 9060 XT loaded the installed glider
and Kyle, exercised failure, recovery and removal, and compared both healthy
actors' uploaded palettes against independently evaluated controls. All 576
comparisons matched exactly at 8/7/4/3 ms with one/four evaluation lanes. The bad
actor kept its previous palette and emitted no active animation-audio request;
only one diagnostic was emitted per failure episode. Native playtesting remains
open. See [rendering](rendering.md#actor-animation-failures).

## Distributable builds

Windows x64 and Linux x64 release ZIPs for merged source `3a70c22` were built
and checked on 2026-10-04 by the
[GameData packages workflow](../.github/workflows/packages.yml).
Both native jobs extracted their archives, started/stopped an isolated loopback
dedicated server, and verified client discovery and portable settings with
synthetic assets. The source snapshots match on both platforms; checkout and
archiving preserve embedded shader line endings.

Final inspection checked archive CRCs, binary hashes, source revision and
dependency notices. Linux binaries require at most glibc 2.35; Windows imports
contain only system DLLs, with no separate VC++ or MinGW runtime DLL requirement.
No retail data, personal settings or debug symbols are packaged. These checks do
not cover Windows graphical gameplay. See [packages.md](packages.md) for layout,
runtime requirements and the repeatable build procedure.

### Simplified release archives

The `dc36792` Windows/Linux playtest archives were repacked on 2026-10-04
with exactly four files: the two executables, `README.txt` and `LICENSES.txt`.
Binary bytes and executable permissions are preserved. All 317 Linux and 306
Windows original license/attribution sections are retained in the consolidated
text. Build manifests are separate release assets, covered by the updated
checksums; the source archive is unchanged. GitHub asset digests match the local
archives and manifests.

The extracted Linux archive passed synthetic adjacent-asset discovery, portable
configuration and isolated loopback dedicated-server startup/shutdown. Archive
CRCs, four-file contents, binary hashes and notice preservation passed for both
platforms. The updated dependency collector also matched all 315 entries emitted
by the previous Linux collector. Windows execution was not repeated for this
packaging-only update; its executables are identical to the previous release.

## Drop-in client installation

Local change based on `da8adc9` (2026-10-04): the client discovers game data
beside its executable (or in its `GameData` subdirectory), independently of the
working directory. This takes priority over saved paths; explicit positional
paths and `JKA_GAME_DATA` remain overrides. Direct `--connect HOST:PORT` also
supports discovery. See [client launch](client.md#launch) for the full order.

Linux verification: 19 external startup checks passed with synthetic asset-file
markers, covering unrelated working directories, spaces/non-ASCII paths,
discovery precedence, invalid/incomplete locations, known-install fallbacks and
both direct-connect syntaxes. The unmodified optimized binary, placed beside
synthetic `base/` assets and launched from elsewhere with isolated configuration,
found and attempted to read those archives; it then exited on the intentionally
invalid content before creating a window. No retail files were copied. Formatting,
locked workspace build/tests and the optimized client build passed. Windows
double-click/shortcut behavior and full rendering from a drop-in install have not
been exercised by these checks.

The same local work defaults generated client files to a folder in `GameData`
(now `GameData/SJK/`). Storage is selected before the console, browser or HUD is
created, so settings, marks, screenshots, recordings, favorites, friends and
identity share one root. Unwritable installations use the per-user profile. Downloaded content keeps its separate cache.

Eleven external Linux storage scenarios passed using synthetic profiles:
first-run creation, supported-file import and byte preservation, identity-file
permissions, PK3/link exclusion, existing destination conflicts, repeated launches,
real permission-denied fallback, both roots unwritable, obstructing files, failed
import/retry, linked destinations, saved installation hints and same-root aliases.
The checks execute the production storage module in separate processes; they do
not migrate the owner's profile. The unmodified release binary also passed
first-launch, repeat-launch and read-only-installation checks with isolated
synthetic content: imported settings were loaded, autosave used the selected
root, marks/key bytes survived, and originals remained unchanged during portable
launches. Intentionally invalid PK3s stopped these runs before window creation.
Formatting, locked workspace build/tests and the optimized client build passed.
Windows ACLs and native Windows launch remain unverified.

## Console editing and command browser

The console includes a command/cvar browser and caret/output-selection
controls. Browser Apply/Cancel pointer actions
match the keyboard, the footer Filter control works, and underlying menu shapes,
text and FPS output are suppressed while browsing. Printable opening shortcuts
become text when the console is open. Dead-key `^` inserts a literal colour-code
prefix; toggling the console clears pending accent composition so the next
command letter is not changed or swallowed. Escape and non-text toggle bindings
still close it. Gameplay and wire code are unchanged.

Linux verification (2026-10-04, based on `7155455`): 22 temporary checks passed
for caret motion, selection/copying, UTF-8 byte limits, glyph alignment and browser
footer pointer actions at 960×540, 1920×1080 and 3840×2160. Test infrastructure
remains outside the repository. A release X11/Vulkan desktop run exercised
browser opening, search and edit mode. Native keyboard probes reproduced and
corrected the pending-accent problem; the owner confirmed that fix and accepted
the console preview. A separate-profile release run typed a dead-circumflex
followed by `1` and a name and saved exactly `^1` and the name, with the console
still open.
Formatting, locked workspace build/tests and the release build passed. Clipboard
round trips, drag behavior and platform/layout combinations are not exhaustively
verified.

## Classic console

SJK's default console (`con_style classic`) follows EternalJK's: the `console`
shader's background with its stage motion, `con_ratioFix`, the bar, a monospaced
character grid with per-row local timestamps (`con_timestamps 2`), word wrap, the
green-clock input row with overstrike, the version line and corner clock, the
scrollback arrows, EternalJK's heights and keys, and notify lines; it draws on a
layer after all other 2D, so text under it is hidden. See
[client.md](client.md#classic-console). The modern console is unchanged
(`con_style modern`).

Verification (2026-10-04, Windows 11): formatting, the locked workspace build and
tests, including unit tests for the grid, background extent, `con_ratioFix`,
heights, page steps, word wrap with stamps, colour carry-over, overstrike, clock
formatting, the retail and JoF `console` shaders' stage programs, texture motion
and colour waves, and `con_style` parsing. The layer was not run on a GPU or in a
game by the change's author; side-by-side comparison with EternalJK is pending.

The console key closes the console again (05/10/2026, Sol's request): as in
EternalJK, a `cl_consoleKeys` character (or the scan-code key) toggles either
console style and never types; holding it toggles once. Unit tests cover the
open-console decision and the key list; not tried in a game.

The key under Escape opens the console on every layout (07/10/2026, a Hungarian
tester could not open it): `cl_consoleUseScanCode` defaults to 1 as in EternalJK,
with its exception that a layout typing `^` there needs Shift, and saved profiles
are moved from the old 0 once. Unit tests cover the Shift rule for the `0`, `` ` ``,
`²`, `§` and `^` layouts; not tried in a game or with a Hungarian layout.

With the classic console the F3 command browser is classic+ (05/10/2026): the
in-game pop-up frame, retail buttons and list box, and a detail box; the menus'
retail font under `ui_gameFont`. Layout tests check that every part lies inside
the box without overlapping; menu snapshots (`console-browser-classic`,
`console-browser-modern`) were looked at. Not tried in a game: the retail font's
fit in the rows and pointer use are unverified.

## UI texture-switch limit

Branch `personal/ui-runs` (06/10/2026, based on `16f1b20`): the UI shape renderer
allowed 48 texture switches (bind-group runs) per frame across every layer and
dropped art quads past them without a trace. The classic profile's Force page
switches between the icon atlas (holocrons) and the retail star art on every
row, 59 times for its page alone, so Dark Rage's and Team Energize's holocrons
and numbered stars were dropped, and a hover glow pushed a Lightning star out.
The limit is now 512 switches and 16,384 shape vertices (was 4,096), and a frame
that still runs out logs "UI shapes dropped" once. A unit test counts the page's
switches as the renderer does (`ui_renderer::texture_switches`) and keeps them
under a third of the limit. Workspace tests and clippy passed; not checked in
game.

## Credits page

Branch `personal/credits` (06/10/2026, based on `5f14439`) adds the
animated credits page and its file
([credits.txt](../crates/sjk-viewer/assets/credits.txt): Sol, Bishop, Creyon,
then Claude and the reference clients), opened from the main menu, the in-game
SJK pop-up or `credits`; see [client.md](client.md#credits-page). Unit tests
parse the built-in file and its errors, keep scrolling in range and check the
sparks spread across the width; the sjk-viewer tests and clippy passed.
Off-screen snapshots drew the page in both palettes, scrolled and at 21:9; the
snapshot rasterizer has no emblem texture and draws rounded shapes square, so
the halo and card corners are unverified, as is the motion. No game was started.

Branch `personal/credits-history-page` (07/10/2026, based on `82bd816`)
gives everyone with a history a panel with their name large and their whole
history folded under it (features, pull requests and commits from
`credits_history.txt`, written by `scripts/credits_history.py`), and replaces the
drifting beams with turning god rays and a sunburst after the site; see
[client.md](client.md#credits-page). Unit tests parse the built-in history and
check that every person in it has a card, its order and links, the folds, and
the ray pictures and their turning; the sjk-viewer tests passed and clippy has
nothing new. The snapshot rasterizer now draws the emblem and ray layers
(nearest texel, added as light); snapshots drew the page folded and unfolded in
both palettes and at 21:9. The motion, the fade-in of unfolded rows, the
scrollbar and the links were not tried in game.

## In-game SJK menu and classic+ changelog

Branch `personal/sjk-menu` (06/10/2026, based on `75bec2e`): the
changelog page takes the classic+ pop-up look with the classic menus, and the
in-game menu gains an SJK button left of About on the classic bar (an SJK row in
the modern menu) whose pop-up opens the changelog; more SJK screens are to be
added there ([ingame_menu/sjk.rs](../crates/sjk-viewer/src/ingame_menu/sjk.rs)).
See [client.md](client.md#changelog-page). Unit tests cover the bar's order and
fit, the pop-up's rows and position, and the classic page's geometry; the
sjk-viewer tests and clippy passed. Off-screen snapshots drew the classic+ page
and the bar with the SJK pop-up. No game was started: both are unverified in the
running client.

## Changelog page

Branch `personal/changelog` (06/10/2026, based on `86ad1be`) adds
[CHANGELOG.md](../CHANGELOG.md) (every release, each change with its credit) and
the client's changelog page: main menu > Changelog on both menu styles, or the
`changelog` command; see [client.md](client.md#changelog-page) and
[SJK conventions](sjk.md#changelog). Unit tests parse the built-in file (every
change credited, EU dates, ASCII) and cover word wrapping and the page's
selection and scrolling; the sjk-viewer tests and workspace clippy passed.
Off-screen snapshots (`menu_snapshot`) drew the page and both main menus with the
new entry. No game was started: the page in the running client is unverified.

## Self-update

Branch `personal/self-update` (06/10/2026, based on `f61230b`) adds an
update check at start-up (`cl_autoUpdate`, default on) and an Update page (main
menu > Update, or the `update` command) that downloads the latest release ZIP,
verifies its SHA-256 and swaps the programs, starting the new one when the client
exits; see [client.md](client.md#updates). New dependencies: `ureq` 3 (rustls)
and `sha2`. Unit tests cover version ordering, the release answer, checksum
lookup, ZIP names, a swap that keeps `.old` files and one that rejects a folder
path; the sjk-viewer tests and workspace formatting passed and the new files add
no clippy warnings. Not verified: a real download and install against a published
release (none newer than the build exists yet), the page in the running client, and
a Linux swap. No game was started.

## Player identity

Branch `personal/identity` (06/10/2026, based on `5f14439`) adds the
`sjk-identity` crate (Ed25519 key file, signed hub requests, an HTTPS hub client and
a background service), `cl_identity` and `cl_hubUrl`, an `SJK`/`VERIFIED` mark on both
scoreboard styles, an Identity page (in-game SJK menu, `identity` command) and
`identity name|bio|key|who`; see [identity.md](identity.md) and
[client.md](client.md#identity). The hub is a separate repository
(Sol-Vulpes/SJK-hub, private). New dependencies: `ed25519-dalek`,
`getrandom` and `base64`.

Verified: the crate's unit tests (key file, signing, address rules, a fake-hub
service run with an explicit clock) and the viewer's tests; the hub's 32 tests; a
signed-request test vector that the hub and client both produce byte for byte; and
an ignored end-to-end suite (`crates/sjk-identity/tests/hub_e2e.rs`) in which the
real client and the real service registered, named, claimed, read the roster,
matched a badge by slot and name, and withdrew the claim on shutdown against a hub
running on this machine. Workspace formatting, `--locked` build and tests passed (855
tests, 7 ignored); clippy adds no warnings in the new files, and the workspace's
existing warnings in other crates (`sjk-nav`, `sjk-icarus`, `sjk-model`, the viewer's
`surface_tables.rs` and others) are unchanged.

Not verified: the scoreboard mark, the Identity page and the SJK menu entry in the
running client (no game was started; their layout is unchecked); a hub on a real
host behind HTTPS; behaviour when the hub's clock and the client's differ by more
than a minute outside the retry; a Linux build. Not built: a main-menu entry, the
confirmed badge from SJK's own server, assets, music, video and chat.

## Player card and scoreboard emblem

Branch `personal/player-card` (06/10/2026, based on `663083b`): looking at
a player with a steady view for `cg_playerCardDelay` seconds shows a card beside
their head (name, model, saber hilts and blade colours, duel record or bot skill,
and for hub players SJK's emblem, hub name and VERIFIED), and the scoreboard marks
hub players with SJK's emblem instead of text; see [client.md](client.md#player-card).
`personal/card-inspect` adds the model's head icon and the worn hat and cape, an
`inspect` key that pins the card to the player under the crosshair, and anchors the card
beside the top of the player's box instead of 70 units above the origin.
`feat/player-inspect-polish` (08/10/2026) anchors it at the hips (the origin), lets
Escape unpin it, drops the duel wins and losses, prints the medal names small and
aligns the saber swatches, the emblem and VERIFIED with their text. Verified: unit tests
(Escape unpins once, only a pinned card; no duel line; medal line breaks) and the
`menu_snapshot`/`medals_snapshot` drawings, measured at 1080p. Not verified in the
running client or on a live server.
The emblem is one more cell of the UI icon atlas, uploaded at start.

Verified: unit tests for the dwell rules (steady look, a turn restarting the wait,
a forgiven 0.3 s gap, a clock that goes back), the card's content from real
player-string shapes (cosmetic text after the colour digit, dual sabers, duels,
bots, hub fields), placement near the screen edges and the emblem's position on a
scoreboard row; the sjk-viewer tests pass. The off-screen snapshots
(`menu_snapshot`, `player-card-*`) drew the card for a verified, a registered, a
plain and an edge-of-screen player; that drawing is an approximation of the
renderer (flat corners, one font). Not verified: the card and the emblem in the
running client (no game was started), their scaling at 4K, the target tracking on
a live server and demos. Not built: the hub bio on the card.

## Default hub address

Branch `personal/hub-default` (06/10/2026): `cl_hubUrl` now defaults to
`https://sjk.dfox.app`, the hub running on the prod box, so players set nothing and a
default install tells that hub where it plays (see [identity.md](identity.md#privacy)).
The Identity page states what is sent and how to stop it. Verified: the sjk-viewer tests;
the hub answered the real client's end-to-end tests over HTTPS through Cloudflare
(06/10/2026). Not verified: the client from outside France (the zone's WAF rule was
widened for the host the same day), a default install end to end.

## Identity page you can edit

Branch `personal/identity-menu` (06/10/2026): the Identity page is now an interface,
not a read-only list: an on/off switch for `cl_identity`, name and bio fields with Save,
"Copy my key id", and the key file's location; it opens from the classic main menu's SJK page
(new IDENTITY entry), the in-game SJK menu or the `identity` command; see
[client.md](client.md#identity). The modern main menu has no entry (its eight rows already fill
the window).

Verified: the sjk-viewer tests (focus order, typing limits in characters, what Save sends,
the fields following the hub's copy until the player types, the classic SJK page's rows); the
off-screen snapshots (`menu_snapshot`, `identity-*`) drew the page switched off, online with a
profile and known players, and for a player with no name yet; that drawing is an approximation
of the renderer (text sits at the top of its box). Not verified: the page in the running
client (typing, Tab, the pointer, the clipboard button), the new classic entry's position.

## Identity page in classic+

Branch `personal/identity-classic` (06/10/2026): with the classic menus the Identity
page takes the classic+ look (retail pop-up, option rows, list-box fields, gold buttons; see
[client.md](client.md#identity)), and a "Use the official hub" button sets `cl_hubUrl` back
to `https://sjk.dfox.app` while another address is saved (a saved value wins over the new
default, which is what a local test hub left behind). The failed-hub status names the address
it tried.

Verified: the sjk-viewer tests (the page's rows fit the box and do not overlap with the most
status lines and players, the focus order with the hub button, when the button is offered);
the off-screen snapshots (`menu_snapshot`, `identity-*-classic`) drew the classic+ page in four
states. Not verified: the classic+ page in the running client (retail font and art, typing,
the pointer).

## Automatic identity, verified badge and own nameplate

Branch `personal/identity-auto` (07/10/2026): a player no longer chooses a hub
name. The client registers its key with the in-game name it wears and registers again
when the name changes; the hub (Sol-Vulpes/SJK-hub `19ddcdd`) keeps each key's worn
names (also from claims) and serves the latest as the profile's name unless the
operator set another; verification stays the operator's. The Identity page and the
`identity` command lose the name (the page shows the worn name and up to three earlier
ones; the bio stays, optional). Nameplates put a gold verified badge (a seal with a
tick, computed into an icon cell) after a verified player's name, and
`cg_nameplateSelf` draws the local player's own plate in third person with their real
bars. See [identity.md](identity.md) and [client.md](client.md#nameplates).

Verified: the sjk-identity tests (the worn name sent with the registration and again on
a change, retried after a failure; bio-only saves), the sjk-viewer tests, the badge's
pixels, and the client's end-to-end suite against the new hub run on this machine. The
off-screen snapshot `nameplate_snapshot` drew verified, unsure and own plates over a
match. Not verified: the badge and the own plate in the running client; the new hub is
committed but not deployed, so until it is, the deployed hub ignores the worn name
(new players show as "Registered") and refuses a bio-only save.

## Nameplate bars and duel rules

Branch `personal/nameplate-bars` (07/10/2026): the shield bar sits on top and
is always drawn (grey dashes when known to be empty); health and shield take the HUD's
`health_ratio` and `armor_ratio` colours (red and green on the Radial HUD), health no
longer ramps through yellow; values over the maximum show as a deeper inner band; a
health or shield range 60 or wider dims under a yellow "?". The estimate reads JA+
private duels (100/100 at the start, 100/25 for the winner; stock heals the winner to
the maximum; start values learnt from the local player's own duels) and the pain
sounds a server plays when it hides the pain value (`*pain25` to `*pain100`). See
[client.md](client.md#nameplates).

`personal/chat-protect` adds JA+'s chat protection: a player with the chat balloon up
takes no damage unless mid-action (saber swing, kick, punch, grab), so on JA+ a hit seen
without a pain or a shield flash, and a fall, change nothing for them (Sol's account of
JA+, which is closed source).

Verified: the sjk-viewer tests (duel start, a duel called off, a JA+ and a stock win,
a chatting player hit with and without a pain, what counts as an action,
the pain-sound quarters, the overflow shade, the broken bar's dashes, the stack order);
the off-screen snapshot `nameplate_snapshot` (overheal, an empty shield, the "?", a
199 shield); its pixels show the overflow green apart from the base green. Not
verified: any of it in the running client or on a JA+ server; JA+'s duel values are
Sol's account, not read from code (JA+ is closed source).

## Alt codes

Branch `personal/alt-codes` (2026-10-05, based on `2696590`) types
Windows Alt codes (Alt + numeric keypad) in the console, chat and menu fields,
which winit 0.30 drops; see [client typing](client.md#useful-console-commands).
Unit tests cover the code page 1252 and 437 tables, modulo 256, control codes,
withheld keypad digits, play keeping the keypad, AltGr and Ctrl+Alt, cancelling
keys, Shift, key repeat and focus loss; the locked workspace build and tests
passed. No game was started: typing a code in the running client is unverified.
Codes without a leading 0 always use code page 437, not the system OEM page.

## Flip kick bind

Branch `personal/flipkick` (06/10/2026, based on `86ad1be`) ports JoF
EJK's `flipkick` command and its `cg_fkDuration`, `cg_fkFirstJumpDuration` and
`cg_fkSecondJumpDelay` cvars: one press starts a run of jump taps, stepped once
per user command (EJK steps per frame; both run at 125 a second since
`personal/cmd-rate`), forbidden by serverinfo `restricts` bit 7.
It is bindable in Controls > Movement. See [client.md](client.md) (`flipkick`).
Unit tests cover the run (alternation, first-jump hold, second-jump delay,
restart); the sjk-viewer tests and workspace clippy passed. No game was started:
a flip kick on a live JA+ server is unverified.

## Crouch without rolling

Branch `feat/duck-no-roll` (08/10/2026, based on `15cf7a9`) ports JoF EJK's
`+duck`: one walking command with jump released, then a crouch, so moving into
a crouch does not roll. It is bindable in Controls > Movement. See
[client.md](client.md) (`+duck`). Unit tests cover the user commands (the walk,
then the crouch, jump let go, `cl_run 0`, a held `+movedown`); the workspace
checks passed. No game was started: that the server keeps the player from
rolling is unverified.

Review at the merge (08/10/2026): the crouch value now goes through the same `axis`
as every pair, so `cl_idrive` (merged first) still resolves jump and crouch with
`+duck` as the crouch key; before, `+duck` and `+movedown` bypassed it and crouch
while jump read 0. Tests cover that, the walk-then-crouch sequence at 8, 7, 4 and 3 ms
command steps, and a jump pressed while ducking. `-duck` releases only itself, unlike
EJK's, which also lets go of every `+movedown` and `+speed` key. Not run on a server.

## Last key wins input

Branch `feat/cl-idrive` (08/10/2026, based on `15cf7a9`) ports JoF EJK's
`cl_idrive` (the last-pressed key of a movement pair wins; 2 limits it to
jump/crouch) and adds `cl_idriveDelay`, a neutral gap in milliseconds before the
newer key takes over and again after it is let go, for servers that penalise instant
reversals. See [client.md](client.md) (`cl_idrive`). Review fixes at the merge
(08/10/2026): the cvars are integers, so reading them as floats left the feature off
(now read from the console in a test); the gap also follows the newer key's release;
the delay is counted in each command's own time; the per-frame read no longer
allocates. Unit tests cover the resolution, mode 2, same-millisecond presses, the
gap at 8, 7, 4 and 3 ms command steps (press and release), the command time, the
delay and its early end, the cvar reads, and a forward/back reversal through the
user-command path. No game was started: what a given server's penalty actually
detects, and whether a delay avoids it, is unverified.

## Weather

Branch `personal/weather` (06/10/2026, based on `2e348b0`) draws the
rain, snow, dust and mist that maps' weather entities ask for, which the client
ignored before, kept under open sky by a surveyed rain cover, with splashes and a
far rain layer; `r_we` runs weather commands from the console. See
[Weather](rendering.md#weather). Unit tests cover the commands and their original
parameters, wind gusts, the cover on synthetic maps (open ground, a roof, water, a
non-sky ceiling, inside brushes), the cover tiles and texture slots, the uniform
layout, and the shader's translation to SPIR-V and HLSL for every entry point. A
local, uncommitted probe surveyed retail `t1_rail` and `hoth2` (columns and
timings in the rendering page). No game was started: how weather looks, its
GPU cost and the cover at real eaves are unverified.

`personal/weather-2` (06/10/2026, based on `888714e`), after Sol tried it: splashes
are anchored in the world (they followed the player), rain is blended as a faint
blue-grey instead of added grey (it looked white), and it adds quality levels
(`r_weatherQuality`), forced weather (`r_weatherForce`), volumetric rain haze and
ground fog replacing the fog sprites (`r_weatherFog`) and volumetric clouds over
every map with sky (`r_clouds`). Unit tests cover the settings and quality levels,
forced weather, the storm and fog parameters, the noise volume's range and tiling,
the cloud uniform and drift, and both shaders' translation to SPIR-V and HLSL. No
game was started: how the fog and clouds look and what they cost are unverified.

`personal/fog-floor` (07/10/2026, based on `6678967`), after Sol saw the ground fog
change height with where they stood or jumped on `T2_Rogue`: beyond the cover
window the fog was floored at the column under the camera, so stepping from a roof
over a street moved all distant fog by 500 units. A coarse far cover of the whole
map, surveyed once on a second worker, now gives the fog its floors there and in
columns past the walls ([Weather](rendering.md#weather)). Unit tests cover the far
grid's placement, the level fill past the walls and void columns on a synthetic
map. Off-screen world shots of `T2_Rogue` either side of a roof edge showed a band
of fog flipping before the change and none after; a local, uncommitted probe timed
the far survey on four retail maps. Not yet tried in a game.

`personal/wet-anchor` (08/10/2026, based on `2630c7a`), after Sol saw the running
water on walls flicker and move with the camera: its streaks now lie on each wall's
plane in world coordinates and scroll at one speed from a wrapped offset, thinner than
before ([Weather](rendering.md#weather)). Off-screen world shots of the streak mask on a
`T2_Rogue` wall from two cameras 6 units and 2° apart, with weather time frozen, showed
a different, speckled pattern from each before the change and the same streaks in the
same places after. Not yet tried in a game.

`personal/rain-wet` (08/10/2026, based on `8c1ac09`), Sol's request: rain wets what it
falls on, chosen with the weather quality (1 wet surfaces, 2 water running down slopes
and walls, 3 puddles with rain rings), in a pass over the world before the players
are drawn, so players stay dry; no drying, as weather rarely changes during a map
([Weather](rendering.md#weather)). Unit tests cover the quality levels, how wet each
rain makes things and its slant, the mirrored sky, the uniform layout and the new
shader's translation to SPIR-V and HLSL. Off-screen world shots of `T2_Rogue` in a
forced storm, with the pass on and off and as a debug mask, showed walls and roofs
wet, ledges' undersides and the pad under a parked ship dry, streaks on walls,
puddle patches and rings; the GPU cost was measured at 4K (rendering page). Not yet
tried in a game; how the streaks and rings look in motion is unverified.

## Shader review

Sol is reviewing every retail world shader on the test maps of
[Shader test maps](rendering.md#shader-test-maps) (06/10/2026, first 30 notes on
`sjk_shaders_a`). What the notes showed:

- Most "needs depth / metal / reflection" notes were pads without material maps:
  the installed generated pack covered only `mp/ffa3` and `mp/duel1` and was
  generation 1. A generation-3 pack for every retail MP map and the test maps
  (1,059 textures, `--max-size 1024`, Sol's notes as overrides) replaced it
  locally; not yet looked at in game.
- Shaders with an authored glow layer already take normal and specular maps on
  their lightmapped pair; they take no emission map, by design.
- 07/10/2026, after Sol saw metal wobble and read pushed in: generation 4 of the
  generator turns relief the right way up from the painted light (52 of 1,054
  retail MP textures, checked by eye on samples: right on most, `kejim/mp_barrel`
  pinned back by an override), gives metal no parallax and less grain, and
  classifies by texture set (generic 560 -> 65 textures); the client limits
  parallax at grazing angles and distance, adds specular anti-aliasing and halves
  the bump tilt of probe reflections. Not yet looked at in game.
- 07/10/2026: parallax is off by default (Sol: focus on normal, specular and
  emission maps). Vertex-lit paint (opaque `rgbGen vertex`: terrain, `_phong` sand
  and rock) takes material maps, its vertex colour standing in for the lightmap;
  generation 5 of the generator covers it (1,199 textures on the retail MP maps and
  the test maps, 145 of them vertex-lit). Open: 180 vertex-lit shaders whose base is
  not opaque vertex paint, chiefly blended terrain layers, stay unmapped.
- 07/10/2026, later: parallax is on again at a tenth of its depth. `r_parallaxStrength`
  (default 0.1, 0 flat, 1 the pack's full depth) scales it live, and Settings >
  Graphics > Image has a Parallax depth slider beside a Parallax mapping switch
  (restart): a profile saved while the default was off keeps `r_parallaxMapping 0`.
  Tests only; not yet looked at in game.
- Controls' painted indicator lights are emission evidence now, but on retail data it
  finds nothing new: nearly every switch and door control already glows through its
  shader, and the three that do not have no lights painted. 31 textures emit.
  Not yet looked at in game.
- Open: decals flicker on every map (Sol). Their bias matches rd-vanilla; only the
  fog pass lacked it (fixed, and since 13/10/2026 decals take their wall's fog). The
  cause outside fog needs a reproduction (map, decal, distance): `world_note_flicker`
  at the `mp/ctf4` decal showed none, and the sky flicker found on `jofmemorial` was the
  sky, not a decal.
- Open: sand footprints (untagged sand takes sand footsteps since 08/10/2026), glass with

  fog pass lacked it (fixed). The cause outside fog needs a reproduction (map,
  decal, distance).
- Open: glass with
  depth (interior mapping), see-through backgrounds on opaque animated fields
  (`byss/static_field`), and parallax that is too deep on sand at grazing angles.
- Open, found on the way: shaders whose `wave` has fewer than four numbers are
  dropped whole (JoF holosigns); rd-vanilla only warns.

## Settings grouped into Graphics and Gameplay

Branch `personal/settings-groups` (06/10/2026, based on `9145a86`), Sol's
regrouping of the classic Setup page: OPTIONS lists First setup (Quick setup
renamed, the `firstsetup` command, `quicksetup` still accepted), Graphics (Video,
the renderer's Image, Lighting and Shadows, and a new Weather group, also a WEATHER
tab of the modern renderer screen), Sound and Gameplay (Mouse, Game options,
Interface, HUD, Scoreboard, Network); Graphics and Gameplay are pages with Back to
OPTIONS, as the renderer page was. See [client.md](client.md). The modern screen's
flat tabs are unchanged apart from FIRST SETUP and WEATHER. The sjk-viewer tests
cover the pages, their groups, Back and Escape; no game was started.

## Quick wheels

Branch `personal/quick-wheel` (07/10/2026, based on `3a875f2`): hold
`+wheel general` (Q) or `+wheel weather` (R), point the mouse at a choice and let go;
see [client.md](client.md#quick-wheels). Unit tests cover the pointer-to-choice
mapping, release and cancel, the pointer's reach, and every wheel's layout; the
off-screen `menu_snapshot` renders both wheels. No game was started: the feel of the
pointer (its dead zone and reach in raw mouse counts) is unverified.

## Force wheel

Branch `personal/force-wheel` (05/10/2026, based on `3f57938`) ports
JoF EJK's Force wheel: the retail Force selection bar for the game and classic
HUD styles, JoF JA+'s Stasis, Repulse and Dash pseudo-slots with their
`+useforce` handling and binds, and JA+ merc mode's flamethrower; SJK now also
mounts `EternalJK/jofclient-assets.pk3` for its pictures. See
[Force wheel](client.md#force-wheel). Unit tests cover the wheel order
(`CG_BuildForceWheel`), stepping, the use remap (Stasis button, Repulse and Dash
once per press, revoked slots), the flamethrower latch, selection through
pseudo-slots, the bar's retail geometry and side counts, and pack discovery;
the workspace build, tests and clippy (0 errors) passed. An off-screen snapshot
(`menu_snapshot`) drew the bar with the installed pictures. No game was started:
the bar in a match, and Stasis, Repulse and Dash on a live JoF JA+ server, are
unverified.

## Classic profile Force page and cosmetics

The classic profile gains a Force page (retail's `ingame_playerforce`, on both
frames, with holocrons, cost-numbered level stars, side cards, a points meter
and a detail panel), a Force summary and button on the profile page, and JoF
EJK's hats and capes: the Cosmetics window, the `cosmetics` command,
`cg_cosmetics`, the name carried after the colour in `color1`/`color2`, and the
pieces drawn on players' `*head_top` and `*back` bolts. Another player's blade
colour is now read with `atoi`, so JoF's `c1 "8santahat"` no longer draws
blue. Clicking a cell of the profile pages' lists picks that cell; before, the
list's own pointer region, registered after its cells, took the click and
stepped to the next entry. See [client.md](client.md#menu-style) and
[Hats and capes](client.md#hats-and-capes).

Follow-ups on the same branch: retail's Force templates (`forcecfg`, the
player's own saved beside `config.cfg`; `sjk-vfs` gains `original_name` for
the capitalised names packs ship), part icons and tint-base swatches in
character creation (64 more UI atlas cells), Hat and Cape rows on the modern
player screen and the pieces on its stage model, and a live model preview
on character creation and in the cosmetics window: the stage actor drawn
offscreen with its own camera into a scene-format target and encoded for
the UI ([rendering](rendering.md#classic-model-preview)), and on lightsaber
creation holding the lit sabers (blades through the game's blade renderer
into a texture of their own). jaPRO's race-unlock hats are left out on
purpose: every hat and cape is open to everyone. Unit tests cover the
template listing, naming and loading, the original names, the part-icon
cells, the preview's target sizes and camera framing. The preview pass is
new GPU code that has not run on any GPU: its validation, colours, lighting
and framing are untested.

Verification (2026-10-05, Windows 11): formatting, the locked workspace build
and tests, including unit tests for the level costs against spending, the
draft's next-level status and one-step level setting, the Force page's
entries, geometry and star tokens, the region order of the menu canvas, the
colour and name split and join, userinfo with a worn name, the saber colour
of a JoF clientinfo, the installed-piece scan (new and old folders), the
`.cosmetic` offset rules, the bolt placement, the drawing rules and the
`cosmetics` command. The behaviour was compared with JoF EJK's source
(`4eb081be`) by reading it. Nothing was run in the client or on a GPU by the
change's author: the pages' look, a piece's fit on a model and other clients
seeing it are untested.

### Lightsaber creation: sabers alone and custom colour

Branch `personal/saber-page` (05/10/2026, based on `af2656f`), from
Sol's testing: the classic lightsaber creation preview shows only the sabers,
lit, laid on their side and turning about their length as retail's `isSaber`
items did (`Item_Model_Paint`), instead of the model holding them; and the page
gains JoF EJK's red, green and blue sliders per saber, writing `color1`/`color2`
6 and `cp_sbRGB1`/`cp_sbRGB2` as JoF EJK does. The preview band moves to the
lower box's left (full page) or under the sliders (in game). Unit tests cover
the showcase (span over hilt and blades, centring at any turn, dual stacking,
the camera taking in the whole saber with its blade to the right, the turn's
start) and the sliders (tokens, channels, pointer values, row room); the
locked workspace build, tests and clippy passed. Offscreen menu snapshots of
the page (single, dual with a custom colour, staff, in game) were looked at;
they show the drawn stand-in saber, since the 3D preview needs the renderer:
the showcase on a GPU is unverified, as is another client seeing the colour.

## Classic profile Force page readability and bars

Branch `personal/profile-force-fixes` (05/10/2026, based on `af2656f`).
Sol reported Dark Rage and Team Energize without holocrons or visible costs on the
Force page. With Sol's profile (`7-2-031330310000030333`: dark side, all 100 points
spent, both powers at level 0, a free-for-all) the menu snapshot showed both rows
drawn but faded: an unbought holocron at 60% (dark red on the dark window), and Team
Energize, a team power outside team games, at 25% with retail's 0.2-grey stars. No
draw, text or pointer cap is reached (138 of 512 draws and 81 of 96 pointer widgets
with retail's 14 templates listed). A power that can be bought now shows its holocron
whole, bought or not; one that cannot keeps its holocron at 55% and its stars a 0.55
grey. The profile's Character Model, The Force and Saber bars sit flush on their boxes
with centred titles (retail's overhung them), and the in-game APPLY is centred on its
band. Unit tests draw the Force page on both frames and sides for that profile and
check every row's holocron opacity, all 39 stars readable and pointable, the detail
panel's holocron for the two powers, and draw-list headroom; the menu snapshot now
draws the profile pages and the Force page. The Force page's storage does not grow
with the template list: with 60 of the player's own templates, scrolled anywhere, it
uses 81 of 96 pointer areas (76 in game), 67 of 160 text runs and 151 of 512 draw
commands, since only the 14 visible rows register; past a limit the menu canvas now
panics in debug builds (logs once in release) instead of dropping areas or labels
silently, and the tests cover both. Not seen in the client: whether the
in-game renderer showed the same faded rows Sol saw is inferred, not reproduced.

## Model grid icons and search

Branch `personal/model-grid` (2026-10-05, based on `0fc6e24`): the
character grids' icons are a cache over the 207 atlas cells instead of the
first 207 catalogue entries, so every model shows its icon; tiles answer to the
pointer by their place on screen (in the classic profile, a click on a head past
the 200th used to pick a part, tint or hilt). Both profile styles gain a model
search. See [menu style](client.md#menu-style).

Verification (2026-10-05, Windows 11): formatting, the locked workspace build
and tests, including unit tests for the icon cache (four times as many models
as cells, scrolled through a screenful at a time; tiles on screen keep their
cells; a new catalogue empties it) and the search (words, case, the team
filter). A temporary check against the owner's installation (not bundled)
listed 844 characters and 72 species whose icons all decode, so the black tiles
were only the cell limit. No client window was opened: the grid on screen and
the cache while scrolling are untested.

## Accepted client improvements

The owner approved publishing the current playtest improvements on 2026-10-04.
These changes retain the accepted rendering defaults. Each topic is verified
and published separately; platform/content coverage limits below still apply.

## Current transition policy

Local gameplay continuation during match-end intermission and server map changes
is suspended at the owner's request. Intermission uses the real server's camera,
scores, chat and ready controls; map changes show a loading notice with gameplay
paused. The native continuation adapter and the viewer's dedicated-server
dependency have been removed. Background loading, shared GPU context, archive
inventory optimization, gate-world adoption and matching same-map reuse remain.
Earlier continuation results below describe the historical implementation, not
current enabled behavior. Fast joining and map loading are the current priority;
no universal loading-time target has been verified.

Verification of this policy (local change based on `7155455`): formatting,
locked workspace build/tests and the optimized Linux client build passed. An
external release/Vulkan run against isolated loopback TaystJK exercised a natural
FFA3 timelimit exit into FFA1 and a same-map restart. It observed 1,666 normal
intermission frames and 2,023 loading frames, asserted that no local simulation
started, checked that attempted movement/mouse input could not move the loading
camera, and verified return to the remote session and disconnect to the menu.
Captures confirmed the scoreboard, loading notice and absence of the local body
at the intermission camera. The visibility rule follows codemp `CG_Player`;
scripted NPC and vehicle intermission scenes were not separately exercised.

On RX 9060 XT at 960×540, this final run took about 7.9 seconds from connection
request to playable FFA3 (excluding initial menu construction) and 9.3 seconds
for FFA1 map preparation/adoption, of which 0.52 seconds was CPU map preparation.
These are individual observations, not a controlled speedup or cold-cache result.
Native-window owner playtesting and Windows runtime checks remain pending.

### Loading optimization verification

Local loading changes based on `7155455` plus the suspended-continuation policy
reduce emission-mask preparation, lamp patch searches and serial mip generation.
An interleaved optimized/baseline/optimized Linux release run on RX 9060 XT,
Vulkan, 960×540 and an isolated loopback TaystJK server measured:

| Operation | Baseline | Optimized runs |
| --- | --- | --- |
| Connection request to playable FFA3, including gate animation | 6.82 s | 4.75 / 4.76 s |
| Natural FFA3 → FFA1 map preparation/adoption | 7.88 s | 3.82 / 3.64 s |

Initial menu construction is excluded. These are warm-machine observations from
one host, not cold-cache guarantees or internet-server latency measurements. All
three runs checked ordinary intermission, paused loading, same-map restart,
remote-session adoption and disconnect to the menu.

External reference checks matched all generated lamp-source float bits and
ordering on FFA3 (978 sources), FFA1 (3,562) and `t2_rancor` (4,333). Mip pixels
matched the original algorithm in 12 dimension/layer cases; changed content,
concurrent reuse and byte/entry eviction checks passed. Emission reduction
matched every float bit in 54 rectangular, power-of-two and NPOT cases.
Before/after 1280×720 Vulkan captures retained the scene appearance; animated
materials and temporal rendering mean whole screenshots are not bit-identical.
The isolated checks live outside the source tree and do not add a regression
suite. Gameplay rules, command quantization and protocol encoding are unchanged.
Formatting, locked workspace build/tests and the optimized Linux client build
passed. The owner accepted the faster loading in native playtesting.

## Server content references

Local fix based on `7155455` (2026-10-03): downloading and world content selection
now accept the common prefix of unequal pak-name/checksum lists, matching OpenJK
codemp `FS_PureServerSetReferencedPaks`. Previously both rejected such lists and
prevented joining some servers. External checks compared 441 list-length cases
with the actual OpenJK `1a6a643` C function (whitespace tokenization stubs), and
exercised both production consumers for equal, unequal and absent lists,
installed/duplicate content, malformed checksums, retail-pack exclusions,
unsafe download paths and the reference-count limit. Formatting, locked workspace
build/tests and the optimized Linux client build passed. No wire codec changed.
The owner's EFF retry exposed a second assumption: references were treated as
mandatory archives even with server downloads disabled. The follow-up now skips
UDP transfers when disabled by either side, skips unrequestable/unsafe/retail
download names, and permits absent optional references during mounting. External
checks using the full production storage/selection modules loaded installed FFA1
with missing and malformed references; covered server on/off/absent flags,
client downloads off, non-UTF-8 hostname bytes and a valid community request;
and retained missing-map and wrong-map-checksum rejection. The matching BSP
checksum passed. Workspace checks and the optimized build passed again.
EFF's read-only status advertised stock FFA1 and downloads disabled. Its complete
join with this follow-up remains unverified; no public server was joined for
these checks.

## Chat player menu preview

Local preview `chat5` (2026-10-04) adds a compact square-edged dropdown left of
chat with only `whisper`, `ignore`, `friend`, and `copy`. It follows the clicked
name, stays above typing controls, and leaves chat positions unchanged. At a
narrow left margin it falls back inside the right edge of the chat lane, clear
of the scoreboard. Active ignore/friend toggles are highlighted. There are no
headers, descriptions, standing hints, or success notices; failures still show.
Friends have a small five-point star before the name. Name hover fits the glyph
bounds; each dropdown highlight matches its button rectangle without the wider
menu-row sweep. The dropdown starts without a selected action and switches
cleanly between mouse hover and keyboard focus, so whisper is not permanently
highlighted. See [chat player actions](client.md#chat-player-actions).

Whispers preserve drafts and send only on Enter. Ignores hide messages for the
current map without muting gameplay sounds; friends are saved as local name
bookmarks. Copy preserves name colour codes. Draft editing shares the console's
caret and glyph metrics, including clipboard shortcuts, word motion/deletion,
Shift/mouse selection, double-click token selection and literal dead-key `^`.

Earlier focused checks covered identity reuse, persistence failures, selection,
Unicode, draft limits and pointer actions. An offline X11 probe with an isolated
clipboard adapter verified a `^1Alice^7` clipboard round trip and word selection/
cut. Those checks predate the compact layout; no windows or game instances are
launched to verify this layout revision, per owner preference. Visual playtesting
remains with the owner. Formatting, locked workspace build/tests and the release
build passed.

## Classic scoreboard preview

Local change based on `dc36792` (2026-10-04): `cg_scoreboardStyle classic` draws
the retail scoreboard layout after JoF EternalJK's `cg_scoreboard.c`, with the
client ID column, head icons, flag icons, compact rows, fades and gliding rows
([client.md](client.md#scoreboard-styles)). Score rows now also keep the stock
powerups, defend, assist and capture fields. Verified on Windows 11 with
formatting, the locked workspace build and tests, including layout tests for the
retail columns, row sizes, group order, team bands and the many-clients layout.
A temporary CPU render of the draw list (free-for-all at 1920x1080, team game at
3440x1440, 26 clients) checked placement and was then deleted. Not run in the
client; head icons, flag icons and the game-font option's retail fonts on this
layout are unverified on screen.

## Leader HUD placement preview

Local preview `leader1` moves the portrait and leader/opponent name/score from the
old minimum 230-unit vertical offset to a 32-unit top margin. The existing right
margin and sizes remain. Optional inventory/snapshot readouts and the default
team-overlay placement follow below the visible block; explicit team coordinates
are preserved. Server selection, scores, visibility and asset resolution are unchanged.
No windows or game instances are launched for this layout-only revision; visual
playtesting remains with the owner. Formatting, locked workspace build/tests
and the optimized build passed.

## Game-data HUD preview

Local change based on `dc36792` (2026-10-04): `cg_hudStyle game` draws the
status HUD from the game's menu files (`cg_hudFiles`), giving the retail HUD
and custom HUD packs; a nonzero integer `cg_hudFiles` gives the stock text HUD
([rendering](rendering.md#game-data-hud)). Windows 11 checks, no client window:
a temporary test read the real files and composited one HUD frame (health 60,
armor 40, Force 80, blaster ammo 120 and a medium-style saber) to PNG at
1920×1080 for retail `assets1.pk3`'s `ui/hud.menu` (13 menus, 31 pictures), the
TheRisqe Radial HUD PK3 (33 pictures, centred on the crosshair) and JoF's
`ui/elegance_hud.txt` (22 pictures); every picture resolved and the placement,
tic fades and digits matched the reference logic. Unit tests cover the reader,
item resolution, tic/number/blink/ammo-colour logic, `cg_hudFiles` values,
widescreen placement and atlas packing. The checks are not bundled. Not run in
the client; siege HUD menus and the out-of-Force flash are not drawn (the vehicle HUD
is [SJK's own](#vehicle-hud)).
Formatting, locked workspace build/tests passed.

## Manual slider entry preview

Local preview `sliders1` (2026-10-04, based on `7155455`) adds direct numeric
entry to every Settings slider, both sabers' RGB sliders, and all Shot sliders
(Camera control since 08/10/2026).
Click the value or press Enter on its row; Enter applies, Escape cancels.
Bounds are enforced without drag-step quantization. Drafts stay attached to
their original row, and invalid values leave the previous setting intact.

Eleven temporary offline checks passed on Linux, covering actual pointer routing,
all numeric settings/cvar types, all six saber channels, Shot preview actions,
sub-step values, cancellation, bounds, malformed/non-finite input, caret editing,
and value targets at 1280×720, 1920×1080 and 3440×1440. The temporary checks are
not bundled with the source. No windows, game instances or servers were opened;
visual playtesting remains with the owner. Formatting, locked workspace build/tests
and the optimized build passed.

## Dynamic glow

Branch `personal/dynamic-glow` (2026-10-04, based on `024c22a`) draws
stock's dynamic glow: `glow` shader stages get a blurred halo, with rd-vulkan's
blur by default and rd-vanilla's as `r_dynamicGlowStyle 0`. See
[Dynamic glow](rendering.md#dynamic-glow). Unit tests (glow flags through the
multitexture collapse, saber blade/core split, cvars, kernels) and naga validation
of the changed programs passed with the locked workspace build and tests. No game
or window was started: appearance, GPU cost and the first-use pipeline compile
remain to be checked on screen. Secondary views and fog do not affect glow yet.

## Player model tolerance

Branch `personal/model-tolerance` (2026-10-05, based on `bcb0b76`) loads
player models, skins and animation tables as rd-vanilla and the retail cgame do
instead of drawing Kyle; see [player models](client.md#player-models). The
headless `player_model_scan` ran on a local Windows 11 install (782 model
directories in `base`, 5064 model/skin rows including one uninstalled skin per
model): SJK failures fell from 887 to 2, and from 796 rows (700 directories)
where rd-vanilla keeps the model to none. Fixed classes: skins without commas or
outside UTF-8 (the young* Jedi packs, aldrokoon), missing skins and parts falling
back to `model_default.skin`, weights past one (sad_scout), slash-prefixed
skeleton names and nameless or zero-frame animation lines (vehicle and creature
packs). With `EternalJK` and `japlus` mounted, every row loads. The two remaining
rows are the dianoga creature, whose animations lie past its skeleton's frames;
retail refuses it as a player model too. Unit tests cover each class on synthetic
files. No game or window was started: how the fixed models look and animate in
play, and the GPU skinning of the rescued meshes, remain to be checked.

## Player model fallback during a match

Branch `personal/model-fallback` (2026-10-05, based on `2696590`): a
player whose clientinfo names a model this client cannot load now shows Kyle, as
`CG_LoadClientInfo` falls back to `DEFAULT_MODEL`, instead of keeping the slot's
previous model; body copies of that player use the same stand-in. It is the only
mechanism found for a `/model` change others see while the local player keeps
the old model (logged as `cs <1131+n>: clientinfo failed`); see
[player models](client.md#player-models). The locked workspace build and tests
passed; there is no unit test of the GPU-side rebuild, and no game was started.

## Native radial HUD

Branch `personal/native-radial-hud` (2026-10-06, based on `de7380b`): the
`radial` HUD style, a cleaned-up TheRisqe Radial HUD drawn by the engine with no PK3
([rendering](rendering.md#hud-style)). Verified: unit tests for the arc geometry,
distance function, ammunition ratio, picker order and style names; the WGSL passes
naga validation; four sample layouts were rendered to PNG through the CPU copy of the
shader (`menu_snapshot`) at 1440x1080. Not verified: the GPU shader and the layout in a
running client (no game window was started), other aspect ratios and `cg_hudScale`
values, and the placement against the in-game crosshair.

Branch `personal/radial-layout` (2026-10-06, based on `30134b4`) moves the rings to
0.69 of the screen height, where TheRisqe's bars sit (the first version centred them on
the crosshair), and puts the numbers in a pill beside each pair of bars instead of one
pill below. Layout widgets gained `offset_fraction` (a fraction of the screen added to
the pixel `offset`) for the drop. Unit tests place the HUD at nine window sizes, an 8K
one past the UI scale's clamp and `cg_hudScale` 0.5, 1 and 1.5 and check the ring centre
and the position of the numbers, pills and weapon name against the bars; sample layouts
were rendered to PNG through `menu_snapshot`. Not verified: the layout in a running
client, and whether 0.69 and the pill placement suit the player's taste.

Branch `personal/radial-outline` (2026-10-06, based on `6ad2a71`) follows up on the
first look at the renders: every bar segment gets a dark shadow outline, the numbers ride
a pill through the bars (health outside and armor inside on the left, ammunition inside and
Force outside on the right), the weapon name moves above the pills, and a saber turns the
ammunition bars into one full line in the style's colour. Unit tests cover the outline
under every segment, the number positions at nine window sizes and the style line for five
styles; renders went through `menu_snapshot`. Not verified: the layout in a running client.

Branch `personal/radial-shadow` (2026-10-06, based on `787cdbb`) fixes what that look showed:
the pills were painted over the bars (widgets paint in document order) and now come first,
and the per-segment outlines became one rounded shadow band per meter that follows its curve
(`arc_span` gives the span). `menu_snapshot` now draws rounded rectangles with their corner
radius, so its renders show the pills as the renderer draws them. Branch
`personal/radial-thick` then widens the band to a 7 px margin round the 7 px bars (it was 4),
a clear outline, and moves the numbers and pills outwards to keep their clearance from it.
Branch `personal/radial-union` (2026-10-06, based on `7b837c3`) makes each pill and the
shadow bands crossing it one shape (a knockout stripe in the arc shader, so the translucent
layers no longer stack into a darker one) and draws the look's text with the classic HUD
font. The unit tests check each shadow's knockout against its side's pill; `menu_snapshot`
renders with the classic font (decoding its distance field on the CPU) and applies the
knockout. Branch `personal/radial-tight` (2026-10-06, based on `d3d4328`) returns to Inter
(the classic font's baked-in drop shadow looked cut at the bottom, and it did not centre),
forced for this look with the digits centred on the pill, aligns each number toward its bars
5 px from their shadow (a new text `align` in layouts), thins the shadow to a 5 px margin, and
blends the shadow with the pill exactly (`knockout_remainder`) so their edge leaves no seam.
Not verified: the shader on a GPU (no window was opened), and the layout in a running client.

## HUD picker

Branch `personal/hud-picker` (2026-10-05, based on `0fc6e24`): the
game-data HUD (`cg_hudStyle game`, the retail HUD unless a HUD pack is
installed) is SJK's default for new configs. `cg_hudPack` uses one PK3's
`ui/hud.menu` and pictures as if the HUD packs mounted after it were not
installed. Settings > HUD > "HUD look" steps through every usable HUD, and
Enter opens a picker with a picture of each game-data HUD drawn for a sample
player. See [HUD style](client.md#hud-style) and
[game-data HUD](rendering.md#game-data-hud).

Verification (2026-10-05, Windows 11): formatting, the locked workspace build
and tests, including unit tests for the HUD choices (install order, labels, the
cvars finding their choice, a pack read without the later ones), the preview
(each pack drawn from its own pictures, mirroring, the text HUD's runs), the
picker (row stepping, opening on the HUD in use, one upload per preview, Escape
and Enter) and `without_mounts`. A temporary check against the owner's
installation (not bundled) listed Jedi Academy, JoF AssetsExtra, TheRisqe
Radial HUD, Text only and SJK's two layouts, rendered each game-data preview in
24-54 ms (release) and wrote them to PNG: the retail and JoF frames in the
bottom corners, the Radial HUD around the crosshair, over the retail `ffa3`
levelshot. No client window was opened: the picker on screen, the preview
texture upload and the live HUD switching are untested.

## Weapon selection row

Branch `personal/weapon-select-style` (05/10/2026, based on `3f57938`):
with the game-data HUD, a weapon change shows retail's `CG_DrawWeaponSelect` row
as JoF EternalJK draws it (square icons, 3/5/7 per side by aspect ratio, `_na`
and staff/dual saber icons, gold `SP_INGAME` name in `FONT_SMALL`, 1.4 s)
instead of SJK's former name line (`^3`, 0.78 of the height down, 42.6 px at
1080 lines, 2 s, no icons). See [game-data HUD](rendering.md#game-data-hud). Unit
tests cover the row order (outward walk, wrap, Concussion between Flechette and
Rocket, empty thermals and mines), the `_na` ammo test, icons per side at 4:3,
16:10, 16:9 and 21:9, and the pixel geometry at 1080 and 2160 lines with
`cg_hudScale`; offscreen snapshots (`menu_snapshot`, the bundled font) drew the
row at 4:3 and 16:9 over a levelshot. The locked workspace build, tests and
clippy passed. No game was started: the row in play, the `ocr_a` name and the
row against a JoF EternalJK screenshot are unverified.

## Classic+ option panels and renderer page

Branch `personal/classic-plus` (2026-10-05, based on `7643ca4`) writes
down how SJK modernises classic pages ([Classic+ menus](classic-plus.md)) and
applies it to the option panels: a detail box under the Setup and Controls rows
describes the focused setting or binding, rows changed from their default and
settings that apply later are marked, Backspace or the right button restores a
setting's default, and Setup's RENDERER opens a classic renderer page with
IMAGE, LIGHTING and SHADOWS groups instead of the modern form. See
[menu style](client.md#menu-style).

Verification (2026-10-05, Windows 11): formatting, the locked workspace build
and tests, including unit tests for the panel geometry (rows, detail box,
retail bounds), the help text (every setting described in two lines, label
notes), the detail box's facts and defaults, reset to default, the binding
detail and its shared-key line, and the renderer page's groups and way back.
The new [menu snapshots](classic-plus.md#seeing-a-page-without-the-game) drew
Setup, Controls and the renderer page on both frames from the owner's
installation; looking at them led to shorter row labels, wider description
lines and the scrollbar moving to the panel's edge (in the in-game pop-up it
covered the values). No client window was opened: the panels' pointer and
keyboard behaviour on screen are untested.

A second pass (same day) merges what retail split for room: one Video group,
one Force Powers group, and Interface, HUD and Scoreboard groups gathering
the modern GAME, HUD, HUD+ and TEXT rows by subject (switching to the modern style
from a group continues on the tab holding its row). Key bindings carry the
retail picture of the weapon, item or Force power they select, in a column and
in the detail box, and the weapon rows are named. Join's team rows in the
in-game bar show their flag and player count, and the profile's Force strip
shrinks its holocrons instead of dropping the last known power. Settings whose
label overflowed the in-game label column have a short row name (the detail box
keeps the full one), and slider numbers read `0.9` rather than a
single-precision default's `0.8999999761581421`. Unit tests
cover the groups (every listed cvar is a setting, each GAME, HUD, HUD+ and TEXT
row in exactly one group), the binding pictures (cells, sharing, every weapon
and power pictured), the strip's fit, every row label fitting its column and
the slider numbers; the snapshots, now with atlas icons and
the in-game bar's pop-ups over a levelshot, showed the in-game panels, bindings
and Join pop-up. `cargo clippy --no-deps` on the viewer added no finding in the
changed code; clippy on the workspace then stopped on four older `sjk-game-jka`
lint errors, fixed since. No client window was opened.

Merged into SJK main with `personal/hud-picker` and `personal/model-grid`
(2026-10-05): the HUD picker's "HUD look" row sits in the classic HUD group,
where LEFT and RIGHT step through the HUDs and ENTER opens the picker; it has
no default mark, since two cvars select the HUD together. The locked workspace
build and tests passed on the merged tree.

## Client devmap preview

Local `devmap1` preview (2026-10-04, based on `7155455`) exposes `devmap <map>`
in the client console and completion catalogue. It launches a fresh, owned,
loopback-only FFA server with `--cheats`, no bots and no match limits, then uses
the existing automatic join path. Invalid names/missing mounted maps are rejected
before session replacement. Create game keeps cheats disabled. See
[development maps](client.md#development-maps) for current server-command limits.

Four temporary offline/headless checks passed: map-name parsing and usage,
console action handoff, private launch arguments/cheat opt-in, and loading
`mp/ffa3`, joining over loopback and observing `give health 77` in a snapshot.
The reference for devmap cheat policy was OpenJK multiplayer `SV_Map_f`.
No gameplay or protocol codec changes were made. The test child stopped cleanly;
no windows were opened and the owner's running game was untouched. Visual
transition playtesting remains open. Formatting, locked workspace build/tests and
the optimized build passed.

## Noclip and talk balloons preview

Local `playfeatures1` preview (2026-10-04, based on `7155455`) integrates
the server noclip command (`60533c8`) and the talk balloon's upright texture
coordinates (`379aaa2`) into the current client/server sources,
retaining the later upright billboard correction and current UI changes.
Native `noclip` requires cheats and a living player; spawning clears it.
Talk/connection icons use stock priority, placement and visibility rules. See
[development maps](client.md#development-maps), [talk balloons](client.md#talk-balloons)
and [server noclip](server.md#noclip).

On Linux, eleven temporary contributed checks passed for movement, talk flags,
sprite selection and billboard orientation. An external harness compared 640
noclip states against unmodified OpenJK multiplayer movement at 8/7/4/3 ms:
origin, velocity and talk flags matched exactly, including vertical-only input.
Headless loopback checks passed for flying, toggling, respawn reset, normal-server
cheat rejection, and a second client receiving the talk flag and selecting the
balloon. No chat messages were sent. Test servers stopped cleanly; the owner's
running game was untouched. Temporary checks are not bundled in the repository.
Visual owner acceptance and populated-match performance remain unverified.
Formatting, locked workspace build/tests and both optimized binaries passed.


Owner follow-up `bubbleopacity1` fixes status/item icons fading like smoke near
geometry when soft particles are enabled. The new icon instance kind bypasses
only that depth fade; authored texture alpha, depth testing and bounded draw
regions remain intact. An offscreen Vulkan check on the RX 9060 XT compared the
production vertex/fragment shaders at 1/8/32-unit depth gaps with texture alpha
0/0.4/1. All nine icon outputs matched the plain shader byte-for-byte; ordinary
particles still faded at close gaps. No windows were opened. Native visual
confirmation remains with the owner. Formatting, locked workspace build/tests
and the optimized client build passed.


## Rocket trails in a barrage

Branch `personal/effect-pool` (05/10/2026, based on `771cb43`): rocket trails
vanished when many rockets flew. JoF's HD `rocket/shot` (it overrides retail in
`JoF_HDWeaponEffects.pk3`) keeps about 1,200 particles alive per rocket: six fire puffs
for half a second and two or three physics smoke puffs for two to three seconds, played
every 8 ms. Two caps cut them. The effect pool (2,048) refused new particles from the
second rocket on, and the instance buffer (1,024 instances, shared with opaque
entities) drew particles oldest first and left out the newest, the trail head, even
with one rocket. Stock has the same problem in another form (`MAX_EFFECTS` 1,800; a full
list reuses `effectList[0]`, so each new puff replaces the last, `FxUtil.cpp`).

The pool is now 4,096, the instance buffer holds every slot at eight shader stages
(35,840 instances, 3.1 MB), and `particle_room.rs` frees room each frame by ending the
particles closest to the end of their lives (linear selection over preallocated
scratch; per-frame billboards are never removed). An ignored offline test
(`particle_room_barrage.rs`) flies rockets around an `mp/ffa3` spawn with the installed
effect and real physics at 2 ms frames, in release:

| Rockets | Trail heads, 2,048, no room | Heads, 4,096 + room | Update + sort per frame |
| --- | --- | --- | --- |
| 1 | 100% | 100% | 0.09 + 0.015 ms |
| 3 | 91% | 100% | 0.29 + 0.05 ms |
| 4 | 67% | 100% | 0.33 + 0.05 ms |
| 8 | 33% | 100% | 0.44 + 0.05 ms |

At 2,048 the same scene cost about 0.2 + 0.025 ms per frame; making room costs about
20 us. Unit tests cover the eviction order, billboards, the headroom and the capacity.
The billboard count above ignores the 1,024-instance limit, which hid trail heads
earlier still. No game was started: the barrage on screen, GPU time for 4,000
billboards and other heavy effects (map smoke, explosions) are unverified.

## Talk balloons in busy scenes

Branch `personal/talk-balloon` (05/10/2026, based on `3f57938`): talk
balloons sometimes vanished over every player at once. Player sprites, pickup icons
and hook ropes are per-frame billboards in the effect particle pool: cleared and
appended again every frame, they lost the slots they had freed to effects spawned in
between (map effects before them, other players' muzzle flashes during actor
submission), and a pool that effects held at its 2,048 cap dropped them all. The pool
now keeps 256 slots that only those billboards use (`particle_types.rs`); effects
stop at their own cap (4,096 since the rocket-trail change below). Unit tests replay that frame. No game was started: the busy
scene Sol saw is not reproduced on screen.

The same branch raises your balloon while the window is unfocused or minimised, as
EternalJK's `cl_unfocusedChatbox` / `cl_minimizedChatbox` do in `CL_CmdButtons`
(both 1). A unit test covers the settings; whether commands keep flowing while
Windows has the window minimised (which the minimised case needs) is unverified.

## Force power presentation

Change based on `024c22a` (2026-10-04), fixing gaps in the Force effects.
Evidence is EternalJK's stock `codemp` code (`cg_players.c`, `cg_ents.c`,
`w_force.c`, `FxTemplate.cpp`, `FxScheduler.cpp`) and the retail EFX files.

- Own Force effects: the local player's Lightning and Drain beams, Push/Pull and
  Grip puffs and body push blur now come from its player state (they were looked
  up in the snapshot's entity list, which never holds the local player). A trickster's
  beam and hand puffs stay visible to its victim, as in stock. See
  [rendering](rendering.md#entity-render-effects).
- Drain bolt shape: EFX `bounce` now sets an electricity bolt's jaggedness, as
  `intensity` did; both also set physics, and the bounce default is retail's
  0.1. This changes the 18 retail electricity primitives using `bounce` (Drain,
  crystal and scepter respawns, DEMP2 alt detonation, environment sparks, ship
  damage) and three expensive-physics emitters without a bounce key (`env/beam`,
  `mp/spawn`, `mp/jedispawn`), which now rebound weakly instead of stopping.
- Mind Trick: a trickster fades out for its victims and is then hidden, fading
  back in when the trick ends; the trickster sees the confusion effect over its
  victims' heads; active Force Sight sees through it. Tricksters were drawn fully
  before.

Not done yet, with what each needs:

- Dodge afterimage (`PW_SPEEDBURST`, `cg_players.c:12612-12661`): stock
  duplicates the Ghoul2 instance frozen at its current frame and draws it at the
  player's current origin for 254 ms, alpha 254 down to 1. A copy in the live
  pose, as the speed trail draws, would coincide with the body; it needs a
  frozen pose, i.e. a second joint palette and vertex range per actor (as
  corpse-pool bodies have) staged once at the burst, drawn with forced alpha.
- Force Sight lighting (`cg_players.c:12258-12264`, `tr_light.cpp:351-357`):
  other players get `RF_MINLIGHT` with `shaderRGBA` 255,255,0, which adds that to
  their ambient light. Actors are lit per fragment from the light grid or, in
  real-time mode, the light buffer, not from the instance light, so this needs a
  per-instance flag through `stage_runtime.wgsl` and a rule for real-time mode.
  The Sight shell overlay is drawn.
- `surfaceparm forcesight` surfaces (`tr_main.cpp:1103-1106`,
  `cg_draw.c:10741-10742`) should draw only while the viewer's Sight is on. The
  shader parser ignores the parm, so they always draw; it needs a shader flag and
  a per-frame world draw toggle. No retail MP map uses it (only SP `rift.shader`).
- Push/Pull refraction (`cg_renderToTextureFX 1`, `CG_ForcePushBlur`
  `cg_players.c:5264-5395`, `tr_backend.cpp:1085-1130`): stock copies a square
  of the frame around the hand and draws `models/weaphits/testboom.md3` textured
  from it with `effects/refraction`, scale 1 to 0.2 (Pull 0.2 to 1) over 500 ms,
  alpha 244 to 10, fixed in place after 200 ms. SJK keeps the
  `cg_renderToTextureFX 0` puffs. It needs the scene pass split after opaque
  entities on frames with a push, a frame-sized copy target made at resize and a
  distortion pipeline.

Unit tests cover the effect selection (own beam levels 2-6, own Grip in first
and third person, own Push, a remote caster, a trickster, Force Sight), parse
the retail Drain EFX, and step the trick fade (fade-out, hiding, fade-in,
truncation, reset after a second's absence, the Sight exception). None of it
has been checked in a game window yet. Formatting and the locked workspace
build and tests passed; clippy finds nothing in the changed code apart from
the four known `sjk-game-jka` deny errors, which stop it otherwise.

## g_debugMelee prediction

Local change based on `da8adc9` (2026-10-04): client prediction reads
`CS_SERVERINFO` `g_debugMelee` as OpenJK `codemp` does (`cg_servercmds.c:137`)
and predicts its melee kicks, the grapple's early return and the wall hold
(`bg_pmove.c:1621-1641,7464-7581`). Melee's alternate attack is now predicted
with the cvar off too (stock punches). On the JA+ dialect the levels follow JA+:
1 is the melee attacks only and 2 adds the wall hold; a grabbed wall leaves the
view to the player, and alternate attack standing still is a front kick. The
TaystJK dialect takes the JA+ levels without the view or standing kick (from
jaPRO's server code; not observed). SJK's dedicated server does not simulate the cvar,
whose default there is 0. See [networking](networking.md#server-dialect-movement-rules).

Evidence (Windows 11): a scratch harness outside the repository joined a local,
windowless JA+ 2.4 Build 7 server (EternalJK x86 dedicated, `mp/ffa3`, loopback),
sent scripted commands at 8/7/4/3 ms, and replayed them offline through
`sjk-game-jka` exactly as the viewer reseeds from each snapshot, comparing
origin, velocity, view, animations, timers, flags, weapon time/state, holster and
ground per snapshot (about 640 per run). Scenarios: punches, four ground kicks,
standing alternate attack, the grapple, an air kick after a Force jump, a kick out
of a run; and a wall grab approached squarely and 30 degrees off square, holding
jump while sweeping the pitch.

| `g_debugMelee` | Melee runs, clean intervals | Wall runs, clean intervals |
| --- | --- | --- |
| 0 | all clean at every step (489 of 639 before) | free look clean in every grab |
| 1 | all but 2 (the grapple) at every step | all clean at every step (about 20 view misses per grab before) |
| 2 | all but 2 (the grapple) at every step | all clean at every step (439-554 of about 640 before) |

The grapple (`TryGrapple`) is server-only; stock clients do not predict it
either. Remaining misses in other runs came from a moving platform and a corner
slide that the world-only harness collision does not model, one wall run-up
onset and one 1-unit Force-jump velocity difference (one interval each in about
30 wall runs), both outside the changed code. Saber staff kicks
standing still on JA+ are not changed. Not run in the client. Formatting, locked
workspace build/tests passed.

## JA+ private-duel pass-through

Local change based on `da8adc9` (2026-10-04): on JA+ and TaystJK/jaPRO profiles,
prediction lets a bystander pass through duelling players and a dueller pass
through every player and NPC but its opponent
([networking](networking.md)). Windows 11 check, no client window: a local JA+
Mod v2.4 Build 7 server (EternalJK x86 dedicated, loopback, `devmap mp/ffa3`,
scratch home) with three windowless clients, two in a private duel. The server
let a bystander walk through a dueller (closest approach 0.2–2.4 units) and a
dueller walk through the bystander. For a client sending no plugin identity the
dueller arrived with `solid 0` and bystanders were never sent to duellers, so
prediction already matched. With the JA+ plugin identity the
dueller arrived as a solid box with `bolt1`: replaying the bystander's commands
through the predictor with player boxes, the stock rule mispredicted 30–33 of
about 88 intervals around the crossing at 8/7/4/3 ms steps, the duel rule none.
Bystanders were not sent to duellers with "Duel see others" on or off, so no
drawing change was needed on this server. jaPRO was not exercised live, and
the harness models world collision and player boxes only. Not run in the
client. Formatting, locked workspace build/tests passed.

## JA+ grapple prediction

Local change based on `3a70c22` (2026-10-04): the JA+ hook pull and rope hang
are predicted (see [networking.md](networking.md#ja-grapple-hook)). Evidence: a
windowless JA+ 2.4 B7 server (EternalJK x86 dedicated, loopback, `jp_altDim 1`
so players start in the dimension that allows the hook) and a scratch replay
harness that joined, fired,
held, released, re-pulled and used off the hook twice on `mp/ffa3`, then replayed
each snapshot interval through the predictor. With the plugin identity, intervals
matching the server went from 549/774, 643/772, 536/776 and 544/775 at 8/7/4/3 ms
to 762, 763, 763 and 763; pull intervals mismatched 8/159, 4/94, 9/174 and 8/166
(all before: the hook's game-side edges) and rope-hang intervals 0 of 62, 30, 62
and 61 (all before). Without the plugin identity, 685 → 762 of 774 at 8 ms.
The hook's rope is drawn as EternalJK draws it
([rendering.md](rendering.md#billboard-icons)), checked by unit tests only.
Nothing was run in the client. Crouched and in-water pulls and TaystJK/jaPRO's
own grapple were not exercised.

## JA+ movement rules prediction

Local change based on `3a70c22` (2026-10-04): on a JA+ server, prediction follows
JA+'s movement and saber rules ([networking](networking.md#ja-movement-rules)):
the flip kick off players, the head-slide setting, the improved yellow DFA,
wall runs from the Force jump's flips, grip speed, melee with the holdable
button, free taunts and the staff's standing front kick. JA+ is closed source;
EternalJK's JA+ plugin reimplementation is the reference, and the server decides
where they differ (yellow DFA launch 60, not EternalJK's 50; no flip-kick branch
blocking wall runs; wall runs from flips, which EternalJK lacks). Other servers,
SJK's dedicated server included, keep the stock rules.

Evidence (Windows 11): the windowless replay harness outside the repository (as
for `g_debugMelee`), extended with an idle second client, `setviewpos`
placement on a devmap server and other players' boxes in the replayed
collision, against a local JA+ 2.4 Build 7 server on `mp/ffa3` with
`g_debugMelee 0` and the default `jp_cinfo` 196819. Clean intervals at
8/7/4/3 ms, with the JA+ rules and with them off (the previous prediction):

| Scenario | With JA+ rules | Rules off |
| --- | --- | --- |
| Yellow DFA, looking around in the flip | 501-502 of 502-503 (7/4/3 ms) | 412-413 |
| Staff alternate attack standing and moving | all 536-538 | 533-535 |
| Melee attacks with the holdable button | all 286-288 | 238-241 |
| Bow, flourish, gloat, meditate while moving and turning | 802-806 of 806-810 | 611-613 |
| Grip while walking | 238-240 of 239-240 | 179-187 |
| Walking off a player's head (103-104 snapshots on it) | 209/208 at 8/7 ms, 204 of 210 at 4/3 ms | 197-203 |
| Wall flip off a player beside | all 170-171 | 169-171 |
| Jump at a player, jump again close to it | 159-162 of 161-162 | 158-162 |
| Wall run-ups: plain jump, running Force jump, Force jump flips | all but one interval in 12 runs | 514-719, misses at every run-up from a flip |

A second server with flip kick off, the head slide on and the yellow DFA off
(`jp_cinfo` 196834) matched with the rules on in all of these: no flips off
players, frictionless heads, stock DFA, and wall runs from flips still allowed.

Remaining misses are the frames where the server applies a style change or a
taunt from a `generic_cmd` (server-only), slope stance animations the harness
cannot pose (no model feet), and a few one-unit velocity differences in contact
with the other player's box. One wall-run interval at 7 ms (a rebound where
prediction ended a run up the wall) is unexplained. Not predicted: the options
EternalJK never reads, the Jedi Outcast red DFA, a changed `jp_gripSpeedScale`
and holds for JA+'s extra animations. Not run in the client, and not checked
against a public JA+ server or another JA+ version.

## Eye adaptation

SJK's exposure follows the view (`r_autoExposure`, on by default, -0.5 to +1 EV
around `r_hdrExposure`, only brightening with `r_sceneHdr 0`).
Headless Vulkan and DX12 probes on 2026-10-04 (Windows 11, RTX 5080) showed the
resolve and effect layer byte-identical to the fixed exposure at exposure 1 and
checked metering, snapping and smoothing on synthetic scenes; both passes took
about 0.01 ms at 1080p and 0.02–0.03 ms at 4K. No client was run: the look in
play, the default key on real maps and the cost in a full frame are unverified.
See [rendering](rendering.md#eye-adaptation).

## Emission maps

Branch `personal/emission-maps` (2026-10-05, based on `bcb0b76`): `_e`
emission maps on lightmapped world surfaces, added unlit with a dynamic-glow halo,
and, in real-time lighting, lamps for surfaces with no light of their own (capped at
1,024 per map); `sjk-materialgen` generation 3 writes them from shader, glow-image,
name and texel evidence. See [Emission maps](rendering.md#emission-maps). Workspace
build, tests and naga validation of the changed programs passed. A read-only survey of
the owner's installation (215 readable maps, 5,356 candidate textures) wrote 118
emission maps, refused 31 keyword or surface-light textures without luminous texels
and left 619 textures alone whose shaders already glow; on the 23 retail MP maps alone
it wrote 8 (neon signs, Bespin windows). No client was run: appearance, halo, the
light added in real-time lighting, load time and frame cost are unverified.

## Shader remap implementation

A change on 2026-10-05 (based on `7969f0b`) replaces SJK's first shader remap
implementation (`personal/shader-remaps`) with the server shader remaps at
`af65396` and their follow-ups, merged in that order: worldspawn remaps with the
latest remap winning, effects following remaps, undoing a remap restoring the
material's own state, and `clearRemaps` with the Settings row. The follow-ups
overlap: the effects' shared lookup and `clearRemaps` were adapted to the map
remaps (latest remap wins), and `clearRemaps` also drops the map's worldspawn
remaps, as EternalJK's renderer command does. The default stays `cg_remaps 2`
(EternalJK's default). The new remap path did not know about dynamic
glow: a remap that changed a material's stages left the glow pass with a stale
stage index, and joining a JoF server crashed (`world_glow.rs`, 05/10/2026). The
remap now rebuilds the glow order and the material's glow flag, and the glow pass
skips a stage that no longer exists. The locked workspace build of all targets and
the workspace tests passed. No game was started: remaps on screen, from a JoF
server or a map with worldspawn remaps, are unverified in this combination.

## Signed player-state arrays

Branch `personal/score-display` (05/10/2026, based on `3f57938`): the
score showed about 65000 after it went below zero. The client decoded the
snapshot's 16-bit `stats`, `persistant` and `ammo` entries unsigned, where codemp's
`MSG_ReadShort` sign-extends them; they are now signed, as in codemp. This also
makes negative health count as dead for the death camera and ammo's -1 sentinel
read as -1. Unit tests round-trip negative and positive entries and an unsigned
weapon bitset through the player-state writer and reader; the locked workspace
build and tests passed. No game was started: a negative score on a live server
is unverified on screen.

## First-person melee shows no baton

Branch `personal/melee-viewmodel` (05/10/2026, based on `3f57938`): with
melee selected, the first-person view drew the stun baton on the baton's hand rig.
Stock registers no hand rig for `WP_MELEE`, which leaves the baton behind the eye,
so melee now has no view model. A unit test checks melee has none and the stun
baton keeps its model and three barrels; the locked workspace build and tests
passed. No game was started: the first-person view is unverified on screen.

## Server BSP instances in edited maps

Branch `personal/misc-bsp-maps` (05/10/2026, based on `3f57938`). JoF's server
places retail map pieces (`maps/mp/duel1.bsp`, `maps/academy2.bsp`) in a map's void
as `misc_bsp` entities (codemp `SP_misc_bsp`): `EF_PERMANENT` `ET_MOVER`s with the
sub-BSP's world as their inline model, sent only in the baselines. Prediction
already clipped against them, but the third-person camera and the crosshair name
trace read only snapshot entities, so the camera went through an instance's walls.
Both now use the snapshot plus the visible permanent baselines, as stock `CG_Trace`
does through `CG_BuildSolidList`. Unit tests build a main map and an appended
sub-BSP placed at x 8000 and check the camera stops at the instance wall and a
predicted player stands on its floor (`bsp_instance_tests`). Not checked in a game:
no server with these instances was joined.

Props a server spawns or places (JA+ admin models, `ET_GENERAL` entities with
`iModelScale`) were drawn at scale 1: the client built every entity's transform
unscaled. They now take `iModelScale / 100` on all three axes, as `CG_General`
does; players, NPCs and bodies keep their own scaling and brush models stay
unscaled (`general_models_take_the_servers_model_scale`). Map-placed
`misc_model_static` props already matched `SP_misc_model_static`. Lighting at the
instances was not changed: their surfaces use their own lightmaps, and models on
them sample the main map's light grid outside its bounds, clamped as rd-vanilla
does; SJK's real-time light caches cover only the main map.

## Lightsaber creation: one Apply

The classic lightsaber creation page drew two "Apply" buttons; the middle one did
nothing. Retail's `ui/jamp/saber.menu` keeps that button (`apply`, 255 444) inside a
commented-out block, so retail shows only EXIT and one Apply, which writes the saber
and returns to the main menu. SJK now draws the same (05/10/2026); a unit test keeps
one Apply on the page. Not checked in the running client.

## Outgoing text encoding

Branch `personal/legacy-text` (2026-10-05, based on `2696590`) sends
names, chat, forwarded commands and `rcon` text in Windows-1252 when every
character fits, as retail and EternalJK do, instead of UTF-8; other text stays
UTF-8. See [player text](networking.md#player-text). Unit tests cover the
encoder (ASCII borrowed, Latin-1, the Windows-1252 typography bytes, C1
round-trip, non-Windows-1252 text left as UTF-8), the reliable-command path, a
name decoded from the wire going back byte-exact, and the compressed connect
packet and `rcon` datagram bytes; the locked workspace build and tests passed. No
game was started: how a retail or EternalJK client shows SJK's chat and names on
a live server is unverified.

## SJK emblem

Branch `personal/sjk-logo` (2026-10-05, based on `bcb0b76`) puts Sol's
emblem in the classic main menu's ring, where the `ja01` logo video played (the
video is no longer read), and above the modern main menu's title, with a pulsing
core and shimmering blade lights drawn as additive layers. `sjk.exe` and
`sjk-server.exe` carry it as their Windows icon, the client sets it as its window
icon, and the README, release notes and site use it. See
[client.md](client.md#menu-style) and [assets/branding](../assets/branding/README.md).

Verification (2026-10-05, Windows 11, MSVC): formatting, the locked workspace
build and tests, including unit tests for the glow curves, the emblem's place in
the ring at six window sizes and above the modern title, its texture ids, the
alpha-weighted mips, the additive runs and the decoding of every bundled picture
and window icon; the optimized build, whose executables were checked to contain
the 10-size icon group and the version strings. The menu was composited offline
from the retail art at two scales and three animation phases. No client window
was opened: the look on screen, the additive pipeline on a GPU, the window and
taskbar icons and the X11 icon are unverified, as is the GNU toolchain's
`windres` path.

## Version label and dates

Branch `personal/version-overlay` (05/10/2026, based on `3f57938`): the
client draws `SJK <version> · <dd/mm/yyyy HH:MM> · <commit>` at the top centre of
every frame (`cg_drawVersion`, default on) and logs it at startup; the version
comes from one build script for both programs (`SJK_VERSION` in releases,
`dev` otherwise; releases are numbered by date, see [SJK conventions](sjk.md#version)), and the classic console's corner clock reads
`Sun 04/10/2026 22:52:10` instead of EternalJK's 12-hour `asctime`. See
[version label](client.md#version-label) and [SJK conventions](sjk.md#version).
Unit tests cover the date formatting, the label's parts and the console clock;
the locked workspace build, tests and workspace clippy passed. No client was
started: the label's place over each HUD, menu and screen size, and a release
build's version from the workflow, are unverified.

## Visual defaults

Branch `personal/sol-visual-defaults` (2026-10-05, based on `78b8bf7`)
makes Sol's own settings the defaults: noon sun (`r_dayHour 12`), bloom
(`r_sceneBloom 1`), sunbeam dust (`r_dustMotes 1`) and material maps
(`r_normalMapping`, `r_specularMapping`, `r_parallaxMapping` 1, which act only
where a pack supplies maps). Saved `config.cfg` values still win; nothing is
migrated. See [Default visual profile](rendering.md#default-visual-profile).
Formatting, the locked workspace build, tests and Clippy passed. No client was
run: the look and frame cost of the new defaults are unverified.

## Gameplay and interface defaults

Branch `personal/defaults` (2026-10-06, based on `5c22d2d`) turns the
settings Sol had chosen into defaults for a new profile: the classic scoreboard
(`cg_scoreboardStyle`, parsed like `ui_menuStyle`: only `modern` or `0` selects
the modern one) and the retail fonts (`ui_gameFont`) to go with the classic menus
and console; the match timer and team overlay on (`cg_drawTimer`,
`cg_drawTeamOverlay`; the overlay draws only for a player on the red or blue
team, so free-for-all is unchanged; the overlay was removed on 13/10/2026, see
[Team outlines and logos](#team-outlines-and-logos)); cut-off limbs shown (`cg_dismember 2`);
`snaps` 120, with the Settings slider raised from 60 to 125 so the default lies
on it; and a 30 FPS cap while the window has no focus (`com_maxfpsUnfocused`).
Saved `config.cfg` values still win; nothing is migrated. The values were chosen
by comparing Sol's `config.cfg` with a fresh registry's defaults. A `snaps` request
above the server's rate is clamped to `min(sv_fps, sv_maxSnaps)` (read in this
repository's dedicated server,
[schedule.rs](../crates/sjk-network/src/server_session/schedule.rs), which follows
the reference); other servers' handling was not checked. Not measured: the
per-frame cost of the limb scan now that `cg_dismember` is not 0 by default (one
pass over the snapshot's entities). No client was run; Sol tests through `play`.

## Classic+ text dialog and Report a bug button

Branch `personal/report-classic-plus` (07/10/2026, based on `61eecc2`): with
the classic menus the text dialog (bug reports and world notes) and the Report a bug
button take the classic+ look; see [classic-plus.md](classic-plus.md#pages). The text
now wraps by the drawn font's measured width in both looks. Layout tests and the menu
snapshots (`report-launcher`, `report-dialog`, `report-dialog-refused`, `note-dialog`,
each also `-classic`) checked it; no game was started. The hub's `POST /v1/report`
and worn-name history were deployed to sjk.dfox.app the same day (an unsigned report
gets 401 instead of the earlier 404).

## Legacy config text

Branch `personal/legacy-cfg-text` (07/10/2026, based on `8fea9b8`): `exec`
refused any file that was not UTF-8 ("is not valid UTF-8 command text"), and a
legacy `config.cfg` with a Latin-1 name, a byte-order mark, `unbindall` or a key
SJK has no name for failed to load, which also stops every save. Such files are now
read as Latin-1 and those lines accepted; see
[client.md](client.md#configuration-and-content). sjk-shell unit tests cover the
decoding and a legacy saved config; no game was started.

## First setup at start and config import

Branch `personal/first-setup-import` (07/10/2026, based on `9ca3ec4`): First
setup opens at every start until its "Don't show at start" row is ticked
(`ui_hideFirstSetup`, replacing the once-only `ui_quickSetup`), and a `.cfg` from
another client dropped on the window (or given to `firstsetup import <path>`)
opens an Import page offering its name, model (with tint), field of view and key
bindings; see [client.md](client.md#importing-from-another-client). Unit tests cover
the parser (saved and partial configs, comments, unknown keys, Latin-1), the page's
ticks and the copy into the profile, including a whole bind table keeping locked
keys. No game was started: the drop itself (winit's drag and drop on Windows) and
the page's look are unverified.

## 125 Hz user commands and cl_maxpackets

Branch `personal/cmd-rate` (06/10/2026, based on `3b620bb`) replaces
the earlier fixed one user command per packet every 25 ms (40 a second) with JoF
EJK's `cl_cmdratecap`: a command every 8 ms on an 8 ms grid of server time,
batched into packets paced by `cl_maxpackets` (now working, SJK default 125,
was 63 and unused), with `cl_packetdup` repeating earlier packets rather than
earlier commands; see [networking.md](networking.md#user-commands-and-move-packets).
Prediction keeps 128 pending commands instead of 64, and the flip kick's frame
cvars count one command per EJK frame again. Unit tests cover the grid (one
command per slot, catch-up after a slow frame capped at four, a restarted
timeline, unanchored stamps of 0), packet pacing and its clamp, and batching
(commands since the last packet, packet-based duplicates, the 32-command limit,
stale stamps). Formatting, workspace clippy, and the sjk-client, sjk-network and
sjk-viewer tests passed. No client was run: movement, prediction and the input
loss on a JoF `pmove_fixed` server are unverified in game, as is the replay cost
of three times as many pending commands at high ping.

## Mute in background

Branch `personal/focus-mute` (06/10/2026, based on `2e348b0`) adds
EternalJK's `snd_mute_losefocus` (default 1): all sound is silent while the window
is unfocused or minimized, by zero effects and music gains rather than a paused
device; see [client.md](client.md#configuration-and-content). It is a row on the
Sound tab. The settings tests cover its row label and help text. No client was
run: the mute on alt-tab and minimize, and the instant gain step (no fade; a
click is possible), are unverified in game.

## Implemented scope

- PK3/loose-file content, BSP maps/collision, legacy models and shader scripts.
- Protocol-26 client/server sessions, shared JKA game rules and client prediction.
- Graphical client with browser, menus/settings, HUD, console, audio, screenshots,
  demos and a Create game flow.
- wgpu BSP renderer with optional modern lighting and post processing.
- Offline generator of local rend2-convention material maps and SJK emission maps
  (`sjk-materialgen`).
- Dedicated-server game integration, console/configuration, stock game-type
  options, map entities, bots/NPCs and script integration.

Entry points and ownership are linked from [architecture.md](architecture.md).
The lists above describe source coverage; they do not close the validation gaps below.

## Verification recorded for this baseline

| Check | Result and scope |
| --- | --- |
| Linux workspace build, Cargo tests and formatting | Passed; no bundled regression tests currently run |
| Optimized client and dedicated-server build | Passed with Rust 1.96.1 on Linux |
| External OpenJK movement reference checks | 560 cases / 72,275 commands at 8/7/4/3 ms; movement, animation, events and compared wire fields matched |
| External player-angle checks | 125 samples matched the OpenJK reference |
| Local OpenJK client against the dedicated server | Joined `mp/ffa3`, walked, jumped and turned; no prediction misses observed in that run |
| Native Vulkan rendering | The local client and dedicated server completed joined-map rendering on `mp/ffa3` with defaults and with day/night + lighting tier 2 + HDR; each continued for 15 seconds without panic or GPU validation error |

The reference checkout used for the external checks was OpenJK
`1a6a643427aa347553e9073dac5570b33337c4d9`, multiplayer `codemp`.
The external harnesses and raw reports are not part of this repository, so these
are recorded maintainer results, not checks reproducible by running `cargo test`
alone. The GPU runs used debug builds and establish startup/integration only;
they do not establish visual parity or release performance.

Sun-shadow correction (2026-10-02, based on `4a8fe31`): external release GPU
captures reproduced and removed ground self-shadow bands in the reported
`mp/ffa3` view, including a nearby camera position. An `mp/ffa1` comparison
showed no obvious regression. See [rendering](rendering.md) for settings,
timings and limits. The updated production release client also rendered
`mp/ffa3` on Linux/Vulkan (Radeon RX 9060 XT) with day/night and HDR enabled
without a panic or GPU validation error during a short startup check.
The owner also playtested the release build and confirmed that the reported
view looked clean.

Sun-shadow edge refinement (2026-10-02, based on `980e693`): the owner
accepted the release playtest with smoother structure shadows and more stable
player-shadow overlaps. Cascades share a reconstruction footprint and wider
transition bands. World and moving-caster depths are filtered independently;
a world-space blocker search and Gaussian reconstruction smooth broad edges
without letting a player change the building's separation estimate.

External release GPU evidence covers the three marked `mp/ffa3` views, camera
approaches and seven positions of an actor in the dynamic-caster pass. Adding
the actor did not brighten unchanged receivers in that probe. Tier 2, day/night
disabled and actor-only GPU smoke checks passed, as did formatting, workspace
build/tests and the release client build. The tested 4K view adds about 0.49 ms
of GPU work over the preceding playtest filter. See [rendering](rendering.md)
for settings, the overlap approximation, memory cost and remaining limits.

Performance work based on `b4debe4` (2026-10-02) preserves the accepted shadow
filter while skipping provably constant footprints, unused baked-lightmap reads
and hidden opaque shading. External 31-player replays show 4K GPU means falling
from 5.30 to 4.57 ms on `mp/ffa3` and 6.71 to 5.81 ms on `mp/ffa1`. At
2560×1080, `ffa3` GPU work is 1.85 ms but total frame time remains 2.47 ms:
the 2 ms target is still open. Shader reference comparisons, alternate-mode
captures, workspace checks and a native release smoke check passed; see
[rendering](rendering.md) for settings, evidence and limitations.
An additional PVS/area candidate cache reduced CPU world-pass encoding by
0.052 ms on the smaller `ffa3` replay and matched 3,178,666 direct-traversal
results, including forced visibility transitions. Its total-frame gain was
0.042 ms there; the reflection-heavy `ffa1` route was essentially unchanged.
Extending opaque depth priming to floor reflections then saved about 0.036 ms
of GPU work in paired 4K `ffa1` runs; lower-resolution timing and captures on
both maps also passed. Reflection resolution and shadow filtering are unchanged.
Lazy particle-stage sampling removes about 0.025 ms of measured effect preparation
on the `ffa3` replay; 3,360 sampled stage values matched the previous implementation
bit for bit. Its total-frame effect was within run variation.
Caching immutable material sort keys saves another 0.01–0.014 ms in instance
preparation on the two routes. The original comparator matched 1,362,200 ordered
draw entries; total-frame improvement remains below run variation.
Reusing each draw's stage-major classification removes a further 0.011–0.016 ms
of CPU world-pass encoding in paired runs, with unchanged draw storage size
and reference-checked classification.
Fixed AO sample tables and equivalent depth-ray arithmetic save another
0.023–0.026 ms of total GPU time in paired 4K runs, with unchanged AO settings
and checked captures on both routes. The owner playtested and accepted the
combined performance preview before publication.


Submission and deferred-lighting work based on `f3f3db2` (2026-10-02) records
frame uploads for a bounded submission worker, restricts HUD shading, shades
uncached lamp receivers in compute, avoids redundant clears/copies and uses
conservative shadow-bound mip levels. On Linux with Ryzen 5 5500 / RX 9060 XT,
external 31-player replays at 2560×1080 reduced mean total frame time from
2.464 to 1.822 ms on `ffa3` and 3.042 to 2.157 ms on `ffa1`. The latter's p99
increased from 3.757 to 4.263 ms, so improved tail latency is not established.
Finite image comparisons and workspace/release checks passed; see
[rendering](rendering.md#submission-and-lighting-work-reduction) for evidence
and limits. Gameplay and protocol code are unchanged.

Optional dust (`r_dustMotes`, default on) is restricted to local godray scattering,
with colour and visibility sampled at each mote's depth. It requires active
volumetrics and follows their shadows and clarity. Linux workspace/release
checks, GPU sampling probes and native HDR/SDR captures passed on 2026-10-02
(Ryzen 5 5500 / RX 9060 XT). The earlier everywhere-dust preview was superseded
after owner feedback. See [rendering](rendering.md) for
current verification and remaining visual/readability limits.

Resident world transitions (2026-10-03, local changes based on `8f692ac`):
menu joining and live map changes retain a rendered, locally playable world.
External release checks against an isolated loopback TaystJK server exercised
FFA3 entry, FFA3 → FFA1, same-map restart, and two consecutive map changes.
Cancellation after entering the destination restored the menu, and the transition
sequence also passed with a local bot present. Artificially delaying delivery of
the completed session from the connection worker allowed more than 1,600 locally controlled
frames before the verified session attached without rebuilding the map. That
check exercises delayed handoff, not a real slow-network handshake. The
same-map restart reattached about 50 ms after its transition event. Old-map
movement and rendering continued while the replacement was built, without a
loading overlay or old remote actors in the resident runtime world.

The installed-archive checksum comparison matched ordered CRC sequences for all
73 previously readable PK3s. Six small external ZIP cases covered empty archives,
empty files, Unicode names, duplicate names, an executable prefix and ZIP64
sizes; malformed central-directory data was rejected. A catalogue with an invalid
unused local payload header is now inventory-readable, matching OpenJK's
central-directory inventory behavior; loading an affected asset still validates
its header/data. One installed-set comparison measured 4,697 ms for the old
payload-header inventory and 134 ms for the directory reader. These checks do not
constitute a new pure-server/wire certification; protocol codecs were unchanged.

Checks used Linux/RADV on a Radeon RX 9060 XT, 960×540, owner graphics settings,
and external ignored harnesses. Normal joins measured about 7–18 seconds in these
runs, depending on machine/cache pressure; the early pre-optimization run took
41 seconds. Typical FFA1 background preparation was about 8–12 seconds. These
are observations, not a controlled cold-cache speedup or a populated-match frame
budget result. Simultaneous build pressure worsened some runs substantially.
One default-backend headless run emitted an EGL destruction panic on the submit
thread after all assertions passed and while exiting. The explicit Vulkan rerun
completed without that diagnostic; native-window/backend shutdown coverage is
still needed. Cold loading, missing-content downloads, platform coverage and visual acceptance
remain open. The movement adapter calls the existing predictor; the external
OpenJK on-foot/force-jump comparison still passed 560 cases / 72,275 commands,
including 8/7/4/3 ms caps. See [client transitions](client.md#joining-and-changing-maps)
for local-authority and exploration limits.

Local gameplay continuation (2026-10-03, same unmerged baseline): departed live
worlds now run the native dedicated gameplay behind a socket-free client session.
Release GPU checks on isolated loopback TaystJK exercised FFA3 → FFA1, same-map
restart, rapid map changes and cancellation, both with and without a bot. They
asserted that only the local player remained, local saber moves advanced, and
the real connection reattached. Screenshots confirmed the third-person model and
saber remained visible. A further run granted test-only weapons and observed
pistol projectiles after releasing the saber attack and switching weapons. It
injected an intermission movement type at handoff to check recovery from the
cached playable state; natural match-end timing remains unverified. A stock
FFA3 door import check preserved its open position, area portal and closing timer.
Native import/reuse checks covered slots 0/17/31 at
8/7/4/3 ms; the 560-case movement/animation/event fixtures and compiled codemp
weapon zoom/charge fixture passed. These are focused checks, not complete native
combat, mod or vehicle compatibility certification. First gate entry is still
movement-only until a server player is available. Its original connection notice
and pointer Cancel action are restored for menu joins; in-server map changes
keep the local gameplay presentation without that overlay. The notice restoration
passed workspace build/test/format checks; native visual acceptance is pending.

In the final 960×540 Vulkan run, local render calls after the first ten local
frames measured 0.98 ms median, 2.72 ms p99 and 38.35 ms maximum, excluding the
harness sleep and capture work. This is a single-player continuation while
background loading, not a 31-player benchmark. Initial model, effect and shader
work can still hitch; moving native skeleton loading into background preparation
removed the observed roughly 0.6-second first-saber-command stall in that run.
It does not establish hitch-free transitions or complete content coverage.

Transition/input polish (2026-10-03, unmerged changes based on `8f692ac`):
a queued Alt bind was reproduced surviving focus loss; focus handling now drops
that frame's gameplay input and ignores synthetic keyboard events. Isolated
TaystJK and native client runs observed a genuine saber throw return after focus
loss. The retained actor keeps its animation tracks and displayed prediction,
with local presentation paced by the same wall-clock origin as local commands.
Remote world adoption and backwards server time retire stale runtime samples;
a forced provisional-clock rollback restored every entity to the current epoch.

Native server lifecycle checks also found bots waiting for an impossible network
acknowledgement after a map change, loss of bot identity during `map_restart`,
and old saber entity handles surviving a rebuilt entity pool. Bots now begin
immediately on the new map, and transient player state is reset while preserving
session/bot ownership, following multiplayer `SV_SpawnServer`/`ClientConnect`.
External integration checks cover active bot slots, current-pool saber handles
and advancing bot commands after both kinds of transition. No packet codec or
movement/combat rules changed. The external 560-case on-foot reference checks
passed again at 8/7/4/3 ms. Native visual acceptance and wider mod/vehicle coverage
remain open.
The final 960×540 Vulkan native-server run covered eight bots, FFA3 → FFA1,
same-map restart, rapid map changes and cancellation. All 8,650 presented remote
actor endpoints matched their received snapshot positions. The saber returned
about 1.21 seconds after the test press, following focus loss. Local render calls
measured 0.56 ms median and 1.93 ms p99, with a 287.81 ms maximum during background
loading; this does not establish hitch-free transitions or 31-player performance.
Workspace build/tests, formatting, standalone clock checks and release builds
passed. The owner playtested the updated transitions and accepted the combined
preview for publication. Wider mod, vehicle and platform coverage remains open.

Natural intermission and chat (2026-10-03, local changes based on `a993436`):
the previous forced-map checks missed the frozen scoreboard phase at normal
match end. The viewer now starts full local continuation before presenting that
snapshot, while the real remote session retains scores and communication.
A Linux/RADV 960×540 release check against an isolated loopback TaystJK server
expired its timelimit, displayed authoritative scores over the local character,
used the stock ready button and followed `nextmap` from FFA3 to FFA1. The final
run observed 1,404 scoreboard frames and 2,450 loading frames with exactly one
local actor and continuing movement, then adopted the destination. Local render
calls measured 0.92 ms median, 2.37 ms p99 and 309.76 ms maximum; an earlier run
under concurrent build pressure reached 3.11 seconds. This is not hitch-free or
a populated-match performance certification.

Socket-free mock endpoints verified global/team/private composer dispatch and
console chat routing without sending test messages to any server. Incoming
server announcements continued during intermission. They now go to the console
exclusively: the owner requested a general separation of console prints from
chat after observing TaystJK's `PrintStats` table in the conversation overlay.
Global/team/private chat and separate center-print HUD notices are preserved;
no server-specific table filter is used. An external offline check confirmed
that 101 console prints, including a stats table, could not enter or displace
chat history or close its composer; chat/team messages and a center notice still
reached their intended presentation. Workspace build/tests, formatting and the
updated production release build passed. This routing check sent no messages
to a server. Captures cover the composer
beside real scores and a synthetic 32-player team board; 576 geometry cases cover
1–32 rows, FFA/team layouts and nine viewports from 640×480 through 4K, including
ultrawide and portrait. These checks do not establish human-to-human delivery or
all mod/platform behavior. Client changes leave gameplay rules and wire codecs
untouched. External compiled OpenJK on-foot/force-jump checks passed again at
8/7/4/3 ms (560 cases / 72,275 commands). Workspace build/tests, formatting and
the production release build passed. The owner accepted the combined preview
for publication; broader mod and platform coverage remains open.

Default visual profile (local change based on `a993436`): fresh profiles now
use the selected day/night, volumetric, shadow, HDR, AO and filtering defaults.
An external release/Vulkan check on Linux/RADV RX 9060 XT verified the graphics
values for empty, explicit and existing override configs and rendered FFA5 and
FFA1. The fresh and explicit FFA5 captures matched at 99.94% of pixels, with
mean absolute RGB difference 0.00022 levels and maximum 2/255. Existing saved
values remained authoritative. Formatting, workspace build/tests and the
production release build passed on the combined local changes. Personal configuration is
excluded; see [default visual profile](rendering.md#default-visual-profile).
This is startup/rendering evidence, not a new populated-match performance claim.

## Open validation and limitations

Volumetric silhouette correction (local changes based on `a993436`): excluded
depth samples no longer dilute the visible-air lighting estimate. External
Linux/RADV release checks reproduced and removed the sampled FFA5 player fringe
without increasing grid resolution. Paired 720p/4K timings showed no material
change in the tested scene; captures also cover FFA3, a Rancor interior and 24
camera turns. See [volumetric coverage](rendering.md#volumetric-silhouette-coverage)
for measurements and reproduction limits. Formatting, workspace build/tests and
the production release build passed. Owner acceptance and broad content coverage
remain open.

- Complete server/gameplay parity remains unverified. Audit concrete scenarios
  across game types, combat, vehicles, NPCs, scripting and map transitions before
  marking individual capabilities complete.
- Model animation sounds now follow `animevents.cfg` frames. Local release checks
  against OpenJK `3e465e7c`'s extracted `CG_PlayerAnimEvents` predicate matched
  238,328 frame-crossing cases. Walk/run and blue-style gesture cues were stable
  at 8/7/4/3 ms steps; a six-second gesture produced its ten authored spin cues
  at every cap, and held frames did not replay them. Include overrides, material
  selection and missing voice-family fallbacks passed external checks. A
  32-actor cursor-only microbenchmark averaged 0.44 microseconds per iteration
  with zero measured heap allocations; this excludes bone queries, collision
  traces, mixing and rendering.
  An isolated loopback TaystJK `5802c99` run produced stone/metal running steps
  and blue-taunt spin cues through a real decoder and null-output mixer, with
  zero decode failures or missing handles. Authored custom saber sound fields
  also passed an external parsing check. Formatting, locked workspace build/tests
  and the optimized Linux client build passed. Native owner listening, broader
  custom-model coverage remain open; footstep dust and prints followed on 13/10/2026.
- Snapshot entities draw a model from `modelindex` only for the entity types
  whose codemp cgame function does so; the per-type rules and their reference
  are in [entity_models.rs](../crates/sjk-client/src/entity_models.rs). An
  `ET_GENERAL` model takes the server's `iModelScale` (a percentage) as
  `CG_General` does, so a prop a server spawns with `modelscale` has its size
  (SJK, `presentation.rs`). A mover's secondary `modelindex2` model is drawn since
  13/10/2026 ([client.md](client.md#mover-models)). Models codemp draws that the client
  still does not: force holocrons, non-brush `ET_MOVER` models, and the portable shield
  (`ET_SPECIAL`) and `ET_BEAM` effects.
- Mod compatibility is scoped by explicit profiles; broad BaseJKA/JA+/TaystJK
  feature parity is not established by profile detection.
- Community PK3 compatibility needs broader map/model coverage. One retail map
  cannot establish every shader, animation or content combination.
- Windows runtime behavior is largely unverified. Do not infer platform support
  from source conditionals alone. Windows reserves 1 MiB for the main thread and
  the client overflowed it after loading `mp/ffa3`; [the viewer build
  script](../crates/sjk-viewer/build.rs) now links Windows binaries with the
  8 MiB Linux size. On Windows 11 (Rust 1.96, MSVC), release and debug clients
  then loaded `mp/ffa3`, and a release client joined a local dedicated server and
  completed its map load. Longer play, other maps and the GNU toolchain are unchecked.
- DX12 rendering is unverified. The client does not select DX12 itself (Vulkan is preferred
  where present); with `WGPU_BACKEND=dx12` and no `dxcompiler.dll`, wgpu compiles
  shaders with FXC. A headless pipeline build of every entry point of the 39 viewer
  shader modules on DX12/FXC (Windows 11, RTX 5080, 2026-10-02) fails only for the
  HUD fragment program (error X3507: the function ends in `discard` without a
  return) and the GI probe update (error X4026: a workgroup barrier after
  storage-dependent early returns). The stage programs' skinning loop no longer
  fails with X3511.
- The 500+ FPS / roughly 2 ms frame target is not certified. Measure representative
  release workloads, including populated matches and chosen graphics settings.
- SJK material-map work (2026-10-04, Windows 11, change based on `024c22a`): the
  lamp/bounce direction target, reflection probes, material-mapped floor mirrors,
  `r_normalMapStrength` and materialgen generation 2 pass the workspace build and
  tests, and naga validates every new or changed program; a dry run of the generator
  on `mp/ffa3`/`mp/duel1` picked 19 metal textures and one polished texture. None
  of it has run on a GPU: appearance, wgpu validation at run time, capture load time
  and frame cost in a match are unverified (estimates are in
  [rendering](rendering.md#reflection-probes)).
- The repository does not bundle a regression suite. Required reference evidence
  must be supplied externally until an in-repository verification approach is agreed.

Sky/terrain correction (2026-10-03, local preview based on `8f692ac`): the three
`t1_danger` marks exposed sky draw-buffer reuse between views and smoothed normals
flipping across visible hills. Both terrain bands are removed in fixed GPU views;
a 24-turn capture reproduced sky disappearance in 12 old-reset frames and verified
the corrected path against direct draws. Workspace checks and owner release build
passed. Native owner playtesting remains pending; see
[rendering](rendering.md#sky-scenery-and-hillside-orientation) for scope and timings.

## Current priorities

1. Evaluate dark-area readability while preserving the accepted lighting style
   (owner priority, 2026-10-03). A local, default-preserving indirect gain and fill
   occlusion experiment passed workspace and external GPU checks; see
   [rendering](rendering.md#indirect-lighting-and-dark-area-readability). Owner
   acceptance and populated-match measurements are pending. The marked custom-map
   ceiling lights were recognized but underpowered; an explicit-glow inference
   refinement passed source-policy and GPU checks on that map plus two stock maps
   ([fixture inference](rendering.md#inferring-fixture-light-from-legacy-materials)).
   That refinement also awaits owner acceptance. Static model fixtures were also
   missing from source extraction: both `t2_rancor` marks now gain local illumination,
   with unchanged stock-map appearance in the sampled `ffa1`/`ffa3` views. The
   Rancor checks add approximately 0.017–0.144 ms of GPU work; see
   [static model fixtures](rendering.md#static-model-fixtures) for evidence and
   remaining visibility limits. Owner acceptance is pending. A subsequent GPU audit
   confirmed a sign error in GI voxel traversal. The corrected forward distances
   and range checks pass 8,302 external GPU/reference cases and workspace checks.
   Both Rancor captures remain byte-identical, so the correction has not solved
   their low visibility. Readback confirms nonzero live probe lighting; receiver
   coverage and effective bounce strength remain to investigate. See
   [GI traversal correction](rendering.md#gi-traversal-correction).
   Dust remains parked.
2. Improve populated-match release frame times while preserving the owner-accepted
   appearance (owner priority, 2026-10-02). Target below 2 ms with 31 players;
   reaching that target is not a reason to stop investigating useful savings.
3. Stabilize normal client and dedicated-server use with reproducible local reports.
4. Audit compatibility gaps by subsystem and scenario; preserve exact combat and wire behavior.
5. Broaden community-content and platform validation.

These priorities guide requested work; they do not authorize an assistant to
start an unrelated task. Update this page when evidence or agreed priorities
change. For a new result, record the source revision, environment, scenario,
reference and limits. Keep resolved change history in Git rather than appending
session-by-session notes here.
