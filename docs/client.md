# Client

Build instructions are in [development.md](development.md). SJK needs an installed
Jedi Academy `GameData` directory with the retail `base/assets*.pk3` files.

## Launch

The body shield flash uses two shader passes by default (`cg_shieldBrightness 2`,
Settings > Game > Shield hit brightness); the default was four, then one. Profiles
on an old default (`4` or `1`) move to `2` once (`cg_shieldBrightnessDefaultVersion`
2); other strengths are preserved, and any later choice, including `1`, stays saved.
The shield sphere option and damage-event timing are unchanged.

`cg_hitMarker` (Settings > HUD > Damage hitmarker) controls the red
damage-direction indicator near the crosshair when you take damage. It defaults
to `0` (off); `cg_hitMarker 1` enables it immediately and saves the choice.
It does not change damage camera kick, hit sounds or damage-event timing.

Settings > Mods > Asset browser or `assetbrowser` opens SJK's installed PK3 browser (also discoverable through F3
in the console's command browser). Select a pack with the arrow keys or wheel;
Enter, Space or a click toggles it. Choices save immediately and apply at the
next client start. The page lists configured game directories and supplemental
`EternalJK` packs, including disabled packs, and protects retail
`base/assets0.pk3` through `assets3.pk3` from disabling.

The archived `fs_disabledPaks` setting stores a JSON list of portable
`directory/filename.pk3` names. Names are case-insensitive, and identical names
in an optional content home follow the same choice. Disabling skips the entire
installed archive, including supplemental crosshairs, emoji and cosmetics;
enabled packs retain their original search priority. Loose files, built-in
content, hub unlockable packs and server-selected download-cache archives are
outside this installed-pack policy. Files are never renamed or removed.

Server `CS_MODELS` entries that no entity currently uses are not parsed or
uploaded when the map starts. A newly referenced rigid model is prepared on
one background worker and installed into that world's GPU buffers when ready;
until then its draw is absent. Failed models are tried once per world, and a
map change discards outstanding results. Map props, stock weapons/items and
effect dependencies still preload. Player models already load for the current
roster rather than the entire installed model catalogue; this change does not
defer player bodies or reload files edited on disk.

Local weapon and movement event sounds come from prediction or player-state
events. The view client's ordinary snapshot entity is excluded from the
entity-sound path, following OpenJK multiplayer, so a shot received before
local prediction plays once. Remote and temporary entity events retain their
independent latches, including during demo playback and spectating.

Held multiplayer pistols use distinct world models: weapon slot 4 (the blaster
pistol, commonly replaced by a DL-44 pack) uses
`models/weapons2/blaster_pistol/blaster_pistol_w.glm`; slot 16 (the old Bryar)
uses `models/weapons2/briar_pistol/briar_pistol_w.glm`. These match OpenJK
`codemp/game/bg_misc.c` item definitions, independently of the first-person
view models. Mounted replacement packs keep their normal priority.

The client program is `sjk` and the dedicated server `sjk-server` (`.exe` on
Windows). Developer and diagnostic environment variables are named `SJK_*`
(`SJK_TRACE_*`, `SJK_LAMP_*`, `SJK_GPU_*` and the like). On Windows both programs
carry SJK's icon and call themselves "SJK" and "SJK dedicated server" in
their version information; the client also sets the icon on its window (title bar
and taskbar on Windows, the window icon on X11; Wayland has none). See
[assets/branding](../assets/branding/README.md).

Put `sjk` (`sjk.exe` on Windows) inside the installed game's
`GameData` folder, beside `base/`, then launch it to open the main menu. A shortcut
can use any working directory. No path settings are required. Keep
`sjk-server` (`sjk-server.exe` on Windows) beside the client for Create game
and local `devmap`.

From that folder:

```sh
./sjk
```

Join a server directly:

```sh
./sjk --connect 127.0.0.1:29071
```

Without an explicit positional GameData argument, discovery checks these locations
in order and uses the first containing `base/assets0.pk3` and `base/assets3.pk3`:

1. `JKA_GAME_DATA`, if set and nonempty.
2. The executable's directory, then its `GameData` subdirectory.
3. The saved `fs_gameData` setting.
4. The working directory, its `GameData` subdirectory, then its
   `Star Wars Jedi Knight - Jedi Academy/GameData` subdirectory.
5. The existing Linux Steam and `~/Games/Jedi Academy/GameData` locations.

Thus a drop-in installation wins over a saved location from another installation,
while an explicit environment or positional path still overrides it. Invalid
discovery candidates are skipped; an invalid explicit positional path is an error.
Discovery does not change the working directory or move any game data.

For a binary kept separately, `JKA_GAME_DATA=/path/to/GameData ./sjk` opens
the main menu. Positional launch also accepts a map path and optional player-model
directory (`sjk /path/to/GameData maps/mp/ffa3.bsp`); it remains a direct
world/viewer launch, whereas no arguments opens the main menu. Demo playback
retains its explicit GameData argument.
See [launch.rs](../crates/sjk-viewer/src/launch.rs) and
[app_launch.rs](../crates/sjk-viewer/src/app_launch.rs).

The client has a server browser, player and settings screens, in-game menus,
console, HUD, screenshots, demo recording/playback and a Create game flow.
Presence here describes implemented surfaces; validation limits are in
[status.md](status.md).

Create game starts a child `sjk-server`, normally found beside the client.
Set `JKA_DEDICATED` to its executable path if installed elsewhere. The child
lifetime is managed by the client and defaults to local access; see
[local_server.rs](../crates/sjk-viewer/src/local_server.rs).

Map previews (`levelshots/<map>.jpg`, `.tga` or `.png`) keep the resolution they
ship in, so an HD levelshot pack stays sharp in a large preview and on the
classic loading screen. They are decoded with their mip chain on a worker thread
([levelshot.rs](../crates/sjk-viewer/src/menu/levelshot.rs)) and drawn from
their own texture ([ui_renderer/levelshot.rs](../crates/sjk-viewer/src/ui_renderer/levelshot.rs)),
stretched to the 4:3 frame as the stock UI draws them. Images longer than 4096
pixels are reduced to that; decoded previews are cached up to 64 MiB.

## Menu style

`ui_menuStyle` (Settings, GAME tab, "Menu style") picks the layout of the main
and in-game menus: `sjk` (default), the SJK UI; or `classic` (or `1`), which is
close to the retail multiplayer menus in layout and flow without porting their
`.menu` scripts. An unknown value falls back to `sjk`. The retail 640x480 layout
is fitted to the window height and centred.

SJK's own classic pages keep the retail look and add modern help inside it; the
rules are in [Classic+ menus](classic-plus.md).

Before 08/10/2026 SJK's default was `classic`, and `config.cfg` saves every
archived cvar, so every existing profile carries it. The console's start moves a
saved `classic` once to `sjk` and marks the profile (`ui_menuStyleDefaultVersion`
1, archived); a style chosen after that, `classic` included, stays
([console_session.rs](../crates/sjk-viewer/src/console_session.rs)). The style is
also the second row of [First setup](#first-setup), under Styles with the camera
style.

SJK began with a third, native style, `modern` (or `0`): a column of entries over
the live map, with its own browser, settings tabs, in-game menu, scoreboard
(`cg_scoreboardStyle modern`), console (`con_style modern`) and HUD layout
(`cg_hudStyle modern`), and a gate on mp/ffa3 that opened onto the server being
joined. It is gone. The console's start puts a saved `modern` (or `0`) in any of
those settings back to its default (the SJK UI, `auto`, `auto` and `game`) and
drops the settings only it read (`ui_accent`, `con_lineSpacing`, `con_maxLines`,
`con_datetime`, and since 10/10/2026 `ui_menuContrast`) from the profile
(`retire_modern_ui` in
[console_session.rs](../crates/sjk-viewer/src/console_session.rs)). The modern
style's full-screen form look, which some screens kept after it, is gone too
(10/10/2026, Sol's request that no trace of it stay): every screen or page with
no classic+ version of its own has the SJK UI's look in every menu style. That is
Create game and its map picker ([SJK UI](sjk-ui.md#create-game)), the Update and
Import pages, Camera control, the test list, the resolution list (a pop-up card)
and, for a way to settings without a classic+ panel (a main-menu destination, the
classic renderer route, First setup's fallback), the SJK UI's Settings or Key
bindings. The SJK UI's fonts load in every menu style for them.

The SJK UI is SJK's own menus drawn over the live map in SJK's type
([SJK UI](sjk-ui.md)). It has its main page: the emblem in a
turning holo ring with the menu on an arc round it (Play, Character, Settings,
SJK and Quit, the first, fourth and last opening pages of their own on the
ring), and the servers joined last in a column on the right, joined with one
click; and its Settings: every category down a lit rail, the rows with switches,
sliders and segments under sub-headings, the focused setting explained on the
right and the search at the top, the key bindings one of its categories; and
its Character, the character standing on the map's path below the tower beside
the form; What's new, Update and Identity in the same look; and its Servers, the
server browser with where the servers come from and which show on the left, the
sortable list in the middle and the chosen server's map, numbers and players on
the right; and its in-game menu, a compact arc of six entries (Resume, Profile,
Players, Settings, Servers, Leave) over a dark fade on the match's left,
a row of small icons under the emblem (Camera control, What's new, Credits, Report a
bug, SJK chat, and Staff tools for a staff key), and a card of the match (server, map,
mode and limits, score and place, clock, players) on the right with the vote on (Yes,
No), the player's side and its buttons (Join red, Join blue, Spectate) and Call a
vote; and its Profile screen, one row of tabs: Character, Saber, Force, SJK Profile,
Achievements, Medals and Collection (Ctrl+Tab).
Tab moves the keyboard between the list, the row and the card
([sjk-ui.md](sjk-ui.md#in-game-menu)). Every other screen opens
in its classic+ version. A client started in the SJK UI stands on mp/duel6
instead of mp/ffa3.

The classic main menu has the retail pages, entries and order, with SJK's
Settings and SJK in place of retail's Controls and Setup:

- Main: Play and Profile on the left, Settings and SJK on the right, Exit below.
  Exit and Escape ask before quitting.
- Play: Solo Game, Join Server, Create Server, Play Demo and Rules. Solo Game
  and Create Server both open Create game, which hosts a local match with bots.
- Settings is retail's Controls and Setup as one screen of option panels with two
  tabs, KEY BINDINGS and OPTIONS (described below). It opens on OPTIONS.
- SJK lists SJK's own screens in the Play page's layout: Changelog, Credits and
  Update.
- Every sub-page repeats the navigation row (Play, Profile, Settings, SJK) and
  has Back and Exit.
- Profile opens the retail profile pages (`player`, `player2`, `saber`), which
  edit the same drafts as the SJK UI's Character and write them at once:
  - Profile: name, team colour and the head grid (six 64-unit cells per row),
    Custom to character creation, APPLY on to lightsaber creation, Exit. SJK
    sets Custom beside a Force button (retail's in-game `configforce` art) to
    the right of the grid, with the Force profile at a glance under them
    (mastery, side, points left and the holocrons of the powers with a level,
    a pip per level; a Jedi with more powers than fit gets smaller holocrons,
    never a missing one), then JoF EJK's Cosmetics button and what is worn. SJK
    also adds a model search right of Team Color: Enter or a click starts
    typing, and the grid then lists only the models of the team whose
    `model/skin` name (a species' model name) contains every typed word,
    ignoring case, with the count at the field's end; Enter or a click
    elsewhere keeps the search, Escape clears it, and it is cleared when the
    screen opens. The description line names the model under the pointer.
    Retail's Character Model, The Force and Saber bars overhung their boxes by
    two units (the in-game one also stood above its box with its title off
    centre); SJK sets each flush on its box with the title centred, and the
    in-game APPLY on the bottom band it sits on.
  - Character creation: species, skin tint swatches, the Head, Torso and Legs
    lists, Back and APPLY. Entering it from an ordinary character puts on the
    first species, as retail's Custom did.
  - Lightsaber creation: saber type, the hilt list (two for Dual Sabers; in
    every menu style the hilts come in the game's load order, as EternalJK and
    JoF EJK list them: the highest-priority pk3 first, each file's hilts in
    order, not sorted), the
    six blade colour swatches (two rows for Dual), Apply, and Apply back to the
    main menu. SJK adds JoF EJK's custom colour: a red, green and blue slider
    per saber (`ingame_saber.menu`'s "RGB Color Creation"), right of the live
    sabers in the lower box under a "Custom color" heading with a chip of the
    colour (framed white while it is the saber's own), or under each saber's
    swatches in the in-game window. The sliders show the blade as it is drawn,
    a stock colour's own tint included; a click or drag on a bar, the wheel
    over it or Left/Right (steps of 5) makes the colour the saber's own:
    `color1`/`color2` 6 (`SABER_RGB`) and the tint in `cp_sbRGB1`/`cp_sbRGB2`
    (`r | g << 8 | b << 16`), as JoF EJK's `UI_UpdateSaberColor` writes them.
    A swatch brings a stock colour back.
    A changed saber (hilt pick here or `set saber1 ...` in the console) is
    applied at once on JA+ and jaPRO-lineage servers: after the userinfo flush
    the client sends `saber <saber1>` for a single saber or `saber <saber1>
    <saber2>` for a pair, as JoF EJK's `CG_Saber_f` does, at most once a
    second and once per changed selection. Stock and unknown servers read
    `saber1`/`saber2` only at spawn, so there the change waits for the next
    respawn. The server may refuse the command (jaPRO: standing still, FFA
    only, not in a duel) and says so in a `print`; it is not retried.
  - Force: retail's in-game `ingame_playerforce` window, here on both
    frames, editing the same draft as the SJK UI's Force tab. It keeps retail's
    frame, title band, gold mastery line, blue and red side bars and level
    stars, each numbered with what that level costs (`UI_DrawForceStars`:
    `forcestarN` once bought, `forcecircleN` before), and retail's templates
    down the left: the side's `forcecfg/light` or `forcecfg/dark` `.fcf`
    files from the game data (retail's Blademaster, Knight and others, and
    any pack's, under their own capitalised names) and the player's own.
    Retail templates come first under **Jedi Academy**, followed by **Community**;
    saved templates are tagged and sort first within their group. SJK bundles
    JoF EJK #23's 22 community presets (13 light, 9 dark), preserving their names,
    colours and values; installed packs and saved files can override them.
    A click or Left/Right loads a template
    into the draft, legalized at the draft's rank as `UI_ForceConfigHandle`
    does, and the row stays filled until the draft is edited. In the SJK UI,
    the Force page's **Force templates** panel has Jedi Academy and Community
    selectors; Left/Right or clicking either half steps through that group.
    Templates edit the draft; Apply sends it. Name and Save
    Template write the draft to `forcecfg/<side>/<name>.fcf` in the client's
    user folder (beside `config.cfg`). The window is as wide as the in-game
    profile. SJK lays the powers out in two columns, the neutral powers over the saber skills and the chosen
    side's five beside them, each with its holocron (`gfx/mp/f_icon_*`), whole
    whenever the power can be bought (Saber Defend or Throw without Attack
    keeps its holocron at 55% and its stars a readable grey, where retail's
    near-black hid their costs), and
    adds Light and Dark cards with the side emblems, a points meter that shows
    a hovered star's price (red when the points left cannot pay it) and a
    panel for the hovered or focused power: its holocron, level, the next
    level's cost or why it cannot be bought (other side, Saber Attack 1
    needed), every level's cost and what it has taken, or what the server does
    with it. On a server, the names of the powers it turns off, holds at a
    level or does not use are amber, the mastery line says when the rank is
    the server's highest, and the panel with nothing focused says how many
    powers the server turns off (see [Force profile on a
    server](#force-profile-on-a-server)). A click
    on a power raises it a level and the right button lowers it, as retail's
    did; a click on a star sets that level (on the power's top star, one
    below). Left and Right step the focused power. Reset, Discard and Apply
    Powers act on the draft; the points line says "not applied" until it is
    written, and leaving the Force window (Escape, Back, another page, the
    screen closing) applies it, as retail's did on close.
    The main menu shows the window over the backdrop with the navigation row
    and Back.
  - Cosmetics (JoF EJK's `ingame_cosmetics`): the installed hats and
    capes side by side and the live model wearing them in a column on the
    right, the worn row filled.
    A click wears a piece and a second takes it off; the wheel and Left/Right
    move through a list and Enter wears. Show Cosmetics cycles `cg_cosmetics`
    (On, Only Me, Off), Remove All takes both off and Apply returns to the
    profile. An empty list says where cosmetics come from; when JoF's
    catalogue has pieces that are not installed the list ends with its note.
    See [Hats and capes](#hats-and-capes).

  Character creation shows the live model where retail drew its
  `ITEM_TYPE_MODEL`: the player's model, wearing their hat and cape, walking
  in place (`BOTH_WALK1`) while the view turns round it at retail's
  `model_rotation 50` (20 degrees a second), drawn without sabers as retail's
  was; the cosmetics window shows it standing (`BOTH_STAND1`) in a column
  beside the lists, where JoF EJK drew its model. The model's portrait shows
  until the first preview frame is drawn. Lightsaber creation shows the
  sabers alone in the band under its boxes, as retail spun the bare hilt
  model (`isSaber` items, `Item_Model_Paint`): lit, laid on their side and
  turning about their length at retail's `model_rotation 20` (50 degrees a
  second, from `model_angle 180`), each centred on its whole length so a
  staff fills the band as a single saber does, the second of a pair below
  the first; a drawn hilt and blade stand in until the first frame. See
  [model preview](rendering.md#classic-model-preview). The part
  lists show each variant's icon (`models/players/<species>/icon_<part>`,
  `.jpg`, `.png` or `.tga`) as retail did, its name when there is none, and
  the swatches are the species' tint base (`gfx/menus/players/<species>/`
  `*tintbase`) multiplied by each `playerchoice.txt` colour, as the swatch
  shaders draw them (a flat colour without the image). The species being
  edited is loaded into 64 cells of the UI icon atlas of its own (four rows,
  about 4 MiB), again when the species changes. Escape returns to the
  profile page, then to the menu. A click
  on a cell of the head grid, the part, tint and hilt lists or the blade
  swatches picks that cell; each list registers its own pointer region before
  its cells, since the menu canvas gives the pointer to the region registered
  last.

Both browsers ask two master servers at once and list every server either knows,
each once ([discover.rs](../crates/sjk-viewer/src/server_browser/discover.rs)):
`cl_master` (`master.jkhub.org:29060`) and `cl_master2` (`master.ouned.de:29060`,
the second master JKHub recommends; empty asks only the first), both in Settings >
Network. A master that fails is passed over while the other answers. With
`master.jkhub.org` alone, servers that list themselves only with the other master
were missing (Sol's report, 11/10/2026).

Join Server opens retail's join-server screen (`ui/jamp/joinserver.menu`) on
the same browser as the SJK UI's, so the list, favourites, filters and sorting
carry over between styles. Labels and buttons are in capitals:

- GET NEW LIST and REFRESH LIST both fetch the master list again, as retail's
  `RefreshServers` behind both did.
- The selectors box: SOURCE (INTERNET or FAVORITES; Tab also switches),
  FILTER (retail's mod filter row is the client's text filter over names and maps;
  `/` or a click starts typing, Escape ends), TYPE (game-type filter), and the
  VIEW EMPTY, VIEW FULL and VIEW LOCKED toggles (the archived
  `ui_browserShow*` cvars; VIEW LOCKED stands where retail's data rate was).
- The list: SERVER NAME, MAP NAME, PLYRS, TYPE and PING columns over retail's
  row bands and column frames, ten 26-unit rows, the sorted column filled and
  its header white with a ^ or v for the direction. Clicking a header sorts by
  it, again reverses it. TYPE adds JA+, JAPRO or MOD where the client detects the
  server's mod, and `*P` for a locked server; favourites carry a gold `*`.
  The wheel and the scrollbar along the right edge scroll the list.
- The secondary row: CONNECT IP (where retail had NEW FAVORITE; the client's
  direct connect), ADD FAVORITE or DEL. FAVORITE for the selected server, and SERVER
  INFO, a pop-up of the server's published settings and players (Escape or a
  click closes it). PASSWORD and FIND PLAYER are dimmed: the client asks for the
  password when a locked server is joined, in a retail-style prompt, and has
  no player search yet.
- BACK returns to the Play page, EXIT to the quit page (not shown when the
  browser was opened from the game menu), JOIN joins the selected server (a
  double-click on a row too).

Server names, rules and players' names arrive in the game's byte charset, not UTF-8:
the browser reads them byte by byte as the game reads chat (UTF-8 when the bytes are
UTF-8), so `»`, `«`, `§` or `·` show as themselves. Until 11/10/2026 a server whose host
name had such a byte was dropped from the list (its info was refused as not UTF-8), and
players' names showed `?` for them.

The status line under the list shows the fetch progress where retail showed
the refresh time, and the description line shows the hovered item's
description. The screen is in
[menu/classic/browser.rs](../crates/sjk-viewer/src/menu/classic/browser.rs).

Both browsers list what one refresh finds: the master's list, up to 1,000 servers
(256 cut off the end of a long list, whose servers never showed; stock keeps 2,048),
each asked for its info at once, the answers already in read after every 32 requests
so they do not pile up in the socket, and every answer within 0.9 seconds of the last
request listed ([lib.rs](../crates/sjk-network/src/lib.rs) `query_server_infos`). A
server that has not answered by then is asked twice more, 0.6 seconds apart: a lost
packet, or a busy server's `getinfo` rate limit (`SVC_Info`), left it out of that
refresh, so the count changed from one refresh to the next and a favourite came and
went. A dead server's "port unreachable", which Windows reports as a reset on the next
read, no longer ends the refresh: it used to, so on Windows every server that had not
answered by then was missing, a different set each time (the engine ignores it too,
`NET_GetPacket`).

Retail entries SJK has no screen for yet (Play Demo, Rules) are shown dimmed,
and their description line says so; retail's Mods and Defaults are left out of
Settings (Backspace restores one setting's default).

Settings' two tabs sit on the panel's title band, KEY BINDINGS on the left and
OPTIONS on the right; a click switches, and Tab walks through the groups of both
tabs in turn. Each tab keeps its group list down the left and shows the chosen
group's items in the panel beside it, opening on First setup (OPTIONS) and Movement
(KEY BINDINGS). The items are the same settings and key bindings as the tabbed
settings screen's, laid out the retail way (labels set against a column at retail
`textalignx`, the value after them, the open group's entry in white) with
classic+'s own controls, below. A group with more items than the
panel holds scrolls: the wheel over its items moves the list one item per notch
(three on a key-binding group), a thin bar along the panel's right edge shows the
position and can be dragged, and Up and Down keep the selected item in view.

SJK makes these panels [classic+](classic-plus.md): a search field and the items
fill the panel's upper part (up to 10 rows on the main menu and 8 in game, where
retail's taller panel held 15 and 12; larger menu text can show fewer) and a
detail box under them describes the focused item. For a setting
it shows the full name and value, what the setting does
([help.rs](../crates/sjk-viewer/src/settings/help.rs), one or two lines for
every setting), its default, its range (or how many choices it has), "applies
after a restart", "applies after a graphics reload" or "applies on the next map",
and its console name. For a key
binding it shows the keys, the console command, which other actions those keys
also do, and the default key. Row labels drop their bracketed notes and their
"(restart)", "(reload)" or "(next map)", which the detail box says instead; a setting that
applies later has a gold `*` after its label, and a row changed from its default
(a binding off its default key) has a gold dot after its label. Backspace, the
right button or the focused row's gold reset arrow (at the row's right end)
returns the setting to its default (resolution and display mode excepted). The
description line names the keys of the focused row, and the detail box's title
carries the icon of the setting's group (or of a binding's category).

Classic+ draws the items with its own controls on retail's blue and gold
(Sol's request, 07/10/2026; [classic-plus.md](classic-plus.md)), instead of
retail's YES/NO text and `menu/new` slider art:

- Labels are in sentence case ("Menu text size"), in retail's option blue; values
  are brighter. The focused row's label turns white over a soft band with a gold
  bar at its left end. Headings, tabs and group names keep retail's capitals.
- An on/off setting is a switch: a pill, gold with its knob right when on, dark
  with its knob left when off, and On or Off after it. A click on its row, Enter,
  Space, Left or Right flips it.
- A number is a slim rail filled gold up to a round knob (ringed while the row is
  focused), with its value centred in a small frame after it. Clicking or dragging
  along the row sets it, Left and Right step it, typing or Enter edits it exactly.
- A choice of two or three values (menu style, menu contrast, console style,
  scoreboard style) shows them all side by side, the one in use filled gold; a
  click on one sets it, Left and Right step.
- A longer list (the display mode, the resolution, the HUD) shows its value in
  a field with a small caret; a click or Enter opens the list.
  A text setting shows its text in a field, typed in place.
- The longer groups are divided by sub-headings: Game options (items and models,
  effects, camera and prediction), Interface (menus, console), HUD (layout,
  crosshair, names and cards, nameplates, readouts) and First setup (graphics,
  styles, display, aim, sound, HUD, nameplates, Force shells, SJK).

Retail split video and the Force binds over two pages because a page held few
items; classic+ panels scroll and explain the focused item, so SJK shows each as
one group and regroups the settings screen's GAME, HUD, HUD+ and TEXT tabs by subject
([settings/groups.rs](../crates/sjk-viewer/src/settings/groups.rs)):

- OPTIONS lists First setup, Graphics, Sound and Gameplay (Sol's grouping,
  06/10/2026). First setup is the FIRST SETUP tab's rows as a group (see
  [First setup](#first-setup)); Sound is AUDIO. Graphics and Gameplay open pages
  of their own in the same layout, their groups down the left, the OPTIONS tab
  still marked, and Back (or Escape) returning to OPTIONS; in game they are the
  same pop-up.
  - Graphics: Video is the whole VIDEO tab (graphics quality, resolution,
    display, frame rate, field of view, marks, shadows, gamma); Image, Lighting and Shadows are the
    renderer settings' tabs; Weather is the renderer settings' WEATHER tab
    (weather, density, quality, forced weather, ground fog, clouds; see
    [Weather](rendering.md#weather)).
  - Gameplay: Mouse is the CONTROLS settings (retail's Mouse/Joystick, moved
    here from the key bindings). Game Options holds the gameplay rows (simple
    items, forced models, saber and speed trails, aura shell, shader remaps,
    third-person camera style and damping, prediction smoothing). Interface
    gathers the menus' and console's look (menu style, contrast, game fonts, menu
    text size and spacing, console style and text size); HUD the HUD style, files
    and scale, status, weapon bar, crosshair and its size, names, nameplates,
    timer, speedometer, lagometer and its scale, chat and ground readout;
    Scoreboard its style, the SJK one's compact rows, client numbers, head
    icons and small rows. Network
    follows.
- KEY BINDINGS: every binding is in one list, under the headings Movement,
  Interaction, Weapons, Force powers (retail's two Force pages as one) and Other.
  The categories down the left jump to their heading, and the one holding the
  selected action is marked as the list scrolls. The weapon rows name their
  weapon (Saber / melee, Blaster pistol, ... Explosives) rather than `weapon N`;
  Melee (fists), unbound until a key is picked, is `weapmelee`
  ([Useful console commands](#useful-console-commands)).

Every group down the left (both tabs, Graphics and Gameplay, main menu and in-game
pop-up) has an icon at the left end of its row, opposite the label: SJK's own grey
metal discs in the quick wheels' style (holocron faces from 13/10/2026 until Sol
brought the discs back on 15/10/2026), generated by Sol
([assets/settings](../crates/sjk-viewer/assets/settings/README.md),
`Entry::icon`). The open or hovered group's icon is at full strength, the others
dimmed.

The search field on the panel's first row finds across the whole tab. `/`, Up
from the first row, or a click gives it the keyboard; typing filters as it goes,
with the number found at the row's end; Enter, Down or Tab go to the results and
Escape clears the text, then leaves the field (an Escape in the panel clears a
search before it leaves). On OPTIONS it searches every option of every group,
the renderer's included, by name, console name, description and group
([settings/search.rs](../crates/sjk-viewer/src/settings/search.rs)), and shows
the results under their groups' headings, the detail box adding "In <group>";
opening a group clears the search. On KEY BINDINGS it finds actions by name,
console command, category or bound key (`space` finds Jump).

The two tabs' searches reach each other: while text is typed, the description line
says how many entries of the other tab match it ("3 key bindings match too: click the
KEY BINDINGS tab"), and a search with no result here names them in the list. Clicking
the other tab (or Tab, or `[` and `]`, across the tab boundary) carries the search text
to it, so one query walks every setting and binding
([menu/classic.rs](../crates/sjk-viewer/src/menu/classic.rs), `sync_cross_search`).

A list (a choice of more than three values, the display mode) does not change
on a click or Enter: they open a dropdown under the value, a rounded list over a
soft shadow with the value in use in gold and marked with a dot, and nothing
changes until a choice is applied with Enter or a click. Up and Down move in it;
Escape, Backspace or a click elsewhere close it unchanged. Left and Right still
step the value directly. The rows the dropdown covers leave their values out
while it is open, since text draws over every shape.

Key bindings carry the retail HUD picture of what they select or use
([keybind_editor/icons.rs](../crates/sjk-viewer/src/keybind_editor/icons.rs)):
each weapon's `gfx/hud/w_icon_*`, each Force power's holocron
(`gfx/mp/f_icon_*`), the items' `i_icon_*` (bacta, seeker, sentry, force field,
binoculars), the saber toggle and style, and the team menu's flag. They sit in
a column between the labels and the keys and, larger, in the detail box. They
are read from the player's game data on a worker thread when the key bindings
first open, into two atlas rows of their own; a picture the game data lacks is
left out.

Up and Down move through the items, Left and Right change a value, Enter flips a
switch or opens a list's dropdown, and typing or Enter on a number edits it exactly; clicking
a slider sets it. Tab moves to the next group (on KEY BINDINGS, Left and Right
do too). A key binding reads "A OR B" (retail's `KEYBIND_OR`,
raised to capitals with the key names as retail's `BindingFromName` does) or `???`
when unbound; Enter or a click waits for the new key, shown in red with
retail's "Enter new key, or ESC to cancel, BACKSPACE to clear.", and Backspace
clears every key of the action. Escape closes the page to the main page (the
Graphics and Gameplay pages to OPTIONS).

The classic in-game bar's Settings opens the same panels as retail's
`ingame_setup` and `ingame_controls` pop-ups, with the two tabs on the pop-up's
title band: a box under the bar with the group list and panel at their in-game
positions and no navigation row, closing back to the bar.

The classic in-game menu (Escape during a match) is the retail top bar: About,
Join, Profile, Add Bot, Settings (retail's Controls and Setup), Vote, Call Vote
and Exit, after SJK's own narrower SJK button at its left end. Each opens a pop-up under it or the matching screen. SJK's pop-up holds
SJK's own screens, for now the [changelog page](#changelog-page)
([ingame_menu/sjk.rs](../crates/sjk-viewer/src/ingame_menu/sjk.rs) lists them);
the SJK UI's in-game menu has them in its row of icons instead
([sjk-ui.md](sjk-ui.md#in-game-menu)), its Join and Vote on the match card.
About shows the server info. Join picks
a team, or opens the class list in Siege; in a team game Team Red and Team Blue
carry their flag (`gfx/hud/mpi_rflag`, `mpi_bflag`) and the team's player count. Vote is Yes/No. Call Vote opens the
call-vote lists. Exit offers Main Menu, Restart Match and Quit Program, each
with a Yes/No confirmation. Profile opens the retail in-game profile window
(`ingame_player`: name, team colour, head grid, Custom, Saber and the Force
summary, then `ingame_player2` and `ingame_saber`); its Apply returns to the
match. The Force box shows the side's emblem beside retail's mastery, side and
points lines and the known powers' holocrons, and its `configforce` button
opens the Force window; the Cosmetics button sits under Custom, as in JoF EJK.
Its Join Red, Join Blue and Spectate buttons are left to the Join tab. Settings opens
the option panels described above. Siege swaps in Objectives and V Chat as retail does. Add Bot,
Objectives, V Chat and Restart Match are dimmed with a note, because the client
cannot add bots or restart a match it does not host. Left and Right move along
the bar; Escape closes a pop-up, then the menu. The Server browser entry and the
[Camera control](#camera-control) icon, which retail did not have, are in the
SJK UI only (F8 and `cameracontrol` open Camera control in either style).

With the player's retail game data mounted, the classic menus draw its own
artwork: the backdrop, side glyph columns, ring, windows, logo, sub-page frames,
button glow, list glow (`menu_buttonback2`), slider bar and thumb
(`menu/new`), in-game bar and pop-up boxes from `gfx/menus`. The art is decoded
once on a worker thread the first time the classic style is used, and the UI
renderer uploads it into one texture per image, separate from the shared UI icon
atlas. Its bind group changes only between draw runs that need a different
texture, so layer order is kept. Additively blended retail images (glow, title
band, bar) are converted to alpha at decode time. A missing image falls back to
the client's own shapes. Retail assets are never bundled.

The art moves as retail's shaders move it (`shaders/ui.shader`); the `.menu`
scripts themselves only swap pages at once and show or hide the glows. Retail's
main page plays `video/ja01` (`gfx/menus/videologo`) in its ring; SJK shows its
own emblem there instead and does not read the video: the gold
starburst with the JK blade, centred in the centre window's opening and drawn
over the frames, 176 of the 640x480 canvas's units across (about 400 pixels at
1080 lines, 790 at 2160), with or without the retail art. Its orange core and
ring breathe (glow strength 0.08 to 0.45 every 4.2 seconds) and the blade's cyan
lights shimmer (0.30 ± 0.15 from two sines of 0.9 and 0.37 seconds), drawn as
additive glow layers over the still emblem. The emblem is bundled
([assets/branding](../assets/branding/README.md), drawn by
[menu/emblem.rs](../crates/sjk-viewer/src/menu/emblem.rs)) and decoded with its
mip chain on a worker thread when the first menu shows. The SJK UI's main page
shows it too, in its turning holo ring ([SJK UI](sjk-ui.md)). The ring
turns 5 degrees a second (`tcMod rotate 5`), the side glyph columns climb over
their `menu_side_text_b` backdrop (`tcMod scroll 0 0.025`), and a quarter of
`env_logo` drifts through the logo's translucent letters between an opaque and a
blended pass of the logo, as its three shader stages do. The button and list
glows, the title band and the in-game bar flicker: retail multiplies the screen
under them by four scrolling layers of `gfx/hud/static_menu`, which the renderer
reproduces by recomposing those small images with the noise on the CPU each frame
they are drawn, over the piece alone because the UI blends with alpha. The
motion clock and curves, the emblem's included, are in
[motion.rs](../crates/sjk-viewer/src/menu/art/motion.rs). The focused entry's
text pulses between white and 80% of it, as `Item_TextColor` does
(`PULSE_DIVISOR` 75 ms); the open group's or page's entry stays steady white.
Labels, buttons, titles and option values are in retail's capitals; descriptions,
typed text, vote-list names and the about values keep their case.

Outside a match the classic style draws no world. The main pages are opaque
over the retail background (the main page's emblem fills the centre gap,
the sub-pages' gap stays dark), and the screens they open (the tabbed settings,
key bindings, Player) get the retail backdrop beneath them; the
classic server browser draws its own. The frame
then clears instead of rendering the map, its secondary views and flares; the
boot map is still loaded, because the menu world is what joins build on, and
switching to the SJK UI shows it again. Create game is the SJK UI's screen in
this style too, over the map. Not loading it at all in the classic
style is a possible follow-up. Over a live match the in-game menu and the
screens it opens leave the game visible, as retail's do.

Joins and server map changes show retail's loading screens. Until the gamestate
arrives it is the connect screen
(`ui/jamp/connect.menu`, `UI_DrawConnectScreen`): `menu/art/unknownmap_mp`,
"Connecting to <address>" (or "Starting up..." when the client hosts the game)
and "Awaiting connection...", "Awaiting challenge..." or "Awaiting
gamestate...", following the join worker's phases. Then it is cgame's
information screen (`CG_DrawInformation`, `CG_LoadBar`): the map's
`levelshots/<map>` over the window (cropped top and bottom on a wide one, the
unknown-map art without a levelshot), "Loading... <what>" or "Awaiting
snapshot...", and the server's lines in retail order: host name, Pure Server,
message of the day, game name, the map's long name, cheats, game type, limits,
force rules and the game type's rules, worded from the player's `MP_INGAME`
strings. The LED bar along the bottom (`gfx/hud/mp_levelload`, `load_tick`,
`load_tick_cap`) has retail's nine ticks; the client lights them from its own load
(gamestate, map parse, world build, world ready, session) rather than cgame's
registration steps. Colour codes in the host name are dropped. The destination
world is adopted only once it is built from the session's own gamestate with the
session in hand, and the player never walks a
preview world: the screen stays until the map is live. Escape or a click
cancels, as before. A failed join shows the connect screen with the reason; a
retail-style error page (`error.menu`) is not drawn yet. The loading screen is
in [loading.rs](../crates/sjk-viewer/src/menu/classic/loading.rs).

The profile pages are in
[player_menu/classic.rs](../crates/sjk-viewer/src/player_menu/classic.rs), with
entries and retail geometry in
[layout.rs](../crates/sjk-viewer/src/player_menu/classic/layout.rs), drawing in
[view.rs](../crates/sjk-viewer/src/player_menu/classic/view.rs) (the Force page
in [force_page.rs](../crates/sjk-viewer/src/player_menu/classic/force_page.rs),
the cosmetics window in
[cosmetics_page.rs](../crates/sjk-viewer/src/player_menu/classic/cosmetics_page.rs))
and pointer routing in
[pointer.rs](../crates/sjk-viewer/src/player_menu/classic/pointer.rs).
The main menu code is in [menu/classic.rs](../crates/sjk-viewer/src/menu/classic.rs): the
page tables are in [pages.rs](../crates/sjk-viewer/src/menu/classic/pages.rs),
types and geometry in [layout.rs](../crates/sjk-viewer/src/menu/classic/layout.rs)
and drawing in [view.rs](../crates/sjk-viewer/src/menu/classic/view.rs). The
option panels' frame is [panel.rs](../crates/sjk-viewer/src/menu/classic/panel.rs);
their items are drawn by
[settings/classic_view.rs](../crates/sjk-viewer/src/settings/classic_view.rs) and
[keybind_editor/classic_view.rs](../crates/sjk-viewer/src/keybind_editor/classic_view.rs).
The
in-game version is in
[ingame_menu/classic.rs](../crates/sjk-viewer/src/ingame_menu/classic.rs), with
[classic_view.rs](../crates/sjk-viewer/src/ingame_menu/classic_view.rs) and
[classic_actions.rs](../crates/sjk-viewer/src/ingame_menu/classic_actions.rs).
The artwork is loaded in [menu/art.rs](../crates/sjk-viewer/src/menu/art.rs) and
bound in [ui_renderer/art.rs](../crates/sjk-viewer/src/ui_renderer/art.rs). The
style is read in [style.rs](../crates/sjk-viewer/src/menu/style.rs). Shared exits
are in [destination.rs](../crates/sjk-viewer/src/menu/destination.rs).

Planned follow-ups, each a new page or screen module, following the retail
`ui/jamp` menus:

- Classic versions of screens the classic pages open in the SJK UI's look, or
  do not have yet: Join Server's `findplayer` and `createfavorite` pop-ups, Create
  Server (`createserver`, `advancedcreateserver`),
  Solo Game (`quickgame`).
- Retail option items SJK has no setting for (video quality presets, colour
  depth, geometric and texture detail, EAX, languages) are left out of the
  panels, and the video restart confirmation is not needed.
- On the profile pages: portraits for every model (the atlas holds 207, so
  species after the characters show none).
- The screens with no SJK equivalent yet: Play Demo (`demo`), Rules
  (`rules*`), Mods, Defaults, Add Bot (`ingame_addbot`), Siege objectives and
  voice chat, and the error page (`error`).
- The retail fonts (`ui_gameFont`, a separate change), and the main page's
  hover captions (`*_undertext`, drawn in the `aurabesh` font).

The player screen's Character and Saber pages write their cvars as soon as a
value changes. The Force page edits a draft instead: Apply writes `forcepowers`
once, in the stock format and legalized as before, and Discard returns to the
applied profile. Leaving the Force page, however it is left (another page or
tab, Back, Escape, the screen closing for the game), applies it as retail's
Force menu did on close: the draft when it changed, else the profile
`forcepowers` already holds, so a server is told to read it (`forcechanged`)
either way, unless Apply sent it during that visit
(`PlayerMenu::settle_force_page`, run once a frame). The unchanged profile is
sent as the player wrote it, not as the page fitted it to a server's lower
rank. Before 12/10/2026 leaving dropped an unapplied draft, and players who
edited and closed the screen kept their old powers. While a draft is pending,
the page says it is not applied yet (the SJK UI's points line, the classic Force
window's status). Power icons (`gfx/mp/f_icon_*`) and side emblems
(`gfx/hud/mpi_jlight`, `gfx/hud/mpi_dklight`) come from the installed game data;
without them the page shows text only. They take icon-atlas cells of their own
after the HUD's, so the character grid keeps all 207 of its icon cells. See
[force.rs](../crates/sjk-viewer/src/player_menu/force.rs).

The Character page's grid (and the classic head grid) can list far more models
than those 207 cells: an installation with community packs lists over 900. The
cells are therefore a cache ([model_icons.rs](../crates/sjk-viewer/src/player_menu/model_icons.rs)):
a tile asks for its icon when it is drawn, a worker thread decodes it (without
the shared image cache, so the whole catalogue is never held at full size) into
a free cell or the one drawn least recently, and tiles on screen keep theirs.
Before SJK did this, every model past the 207th was a black tile. A model whose
icon file cannot be decoded shows its name in the tile instead. Every world (the
menu map, each server map) has a UI renderer and icon atlas of its own while the
menu moves between them, so when the menu meets a renderer it has not uploaded
into (`ShapeRenderer::id`), its image caches (model, Force, part and key-binding
icons, the map and HUD previews) forget what they uploaded and load it again
(`ClientMenu::follow_renderer` in [menu.rs](../crates/sjk-viewer/src/menu.rs)).
Before that, tiles cached in an earlier world drew black after joining, changing
map or disconnecting. Tiles answer to
the pointer by their place on screen, so a long list never runs into other
controls' pointer tokens. The Search row under Team colour (Enter to type)
filters the grid as the classic profile's search does (see above).

### Force profile on a server

`forcepowers` is the player's own profile. On a server, every userinfo sends it
fitted to the server's rules from `CS_SERVERINFO` (`g_maxForceRank`, free saber
skills when every other weapon is off, the game type), as stock
`BG_LegalizedForcePowers` would fit it, but keeping team powers and the powers
`g_forcePowerDisable` turns off: the server drops unavailable powers from its own
copy without calling the profile illegal, so sending them never parks the player in
spectator, and a mod that gives them back in a full Force duel finds them
(jaPRO lets the powers its `g_forcePowerDisableFFA` holds back work in duels).
Apply on a server sends the new profile and then `forcechanged`, as stock
`UI_UpdateClientForcePowers` does: the server reads it at once while you
spectate and at your next respawn in play (it prints that it will). A
`forcepowers` typed in the console or set by a config on a server is sent fitted
the same way and followed by `forcechanged` too, and the console says so (or why
nothing was sent: the server's rules are not known yet, or a Force-profile reply
is already waiting and covers it); a change made before joining is sent by the
join. Before this a console change reached the server's copy only at the next
join or menu Apply. When a server still parks you in spectator over your profile
(`nfr <rank> 1 <team>`, which a mod can send for its own reasons), SJK answers
as the stock Force menu would, `forcechanged` with the team you last chose from
the game menu, and if you are still spectating it asks for that team again up
to three times, 5.5 seconds apart (a server refuses a second team change within
five seconds), then says to use the join menu. Not in duel or power duel, where
waiting as a spectator is the queue.

Opened on a server, the Force page takes the server's highest rank and free
saber skills, which decide whether the server keeps the profile it is sent. What
the server only will not let be used stays buyable and is marked: its disabled
powers ("Off on this server"; Jump "fixed" at 1 and Saber Offense and Defense
at 3, as the server holds them), and team powers outside team games ("Server
decides", since modified servers can grant them there). They cost points like
any other, so the profile still fits where they work. Team powers are kept in
the sent profile even in FFA, so a server
that permits them can read the purchased levels; stock servers still remove
them in non-team games without treating the profile as illegal. Team Heal
remains light-side and Team Energize dark-side. The SJK UI's Force page lists
the server's rules under the side's powers (This server: the highest rank and
its points, free saber skills, the
powers it turns off or, past eight, how many, and the server-dependent team-power
notice); off a server it says a server's rules show there once you join, and team powers can
be bought. A mod's own limits that are not in `CS_SERVERINFO` (jaPRO's
`g_forcePowerDisableFFA`) cannot be shown. The rules are in
[force_profile_negotiation.rs](../crates/sjk-client/src/force_profile_negotiation.rs)
and the page's handling in
[force.rs](../crates/sjk-viewer/src/player_menu/force.rs).

## Player models

A player's or NPC's appearance (`models/players/<model>/<skin>`) loads its
`model.glm`, the skeleton the mesh names (`<name>.gla`), that skeleton's
`animation.cfg` and a skin; if the model cannot be loaded or built, the client
draws Kyle for that player instead
([player_assets.rs](../crates/sjk-viewer/src/player_assets.rs),
[actor_load.rs](../crates/sjk-viewer/src/actor_load.rs)). A folder holding
`sjk_rig.cfg` is a [rig model](rig-models.md): a glTF with its own skeleton, drawn in
place of a stock driver model whose skeleton still carries the animations. That includes a model
changed during the match (`/model`, or a new player in a used slot), as
`CG_LoadClientInfo` falls back to `DEFAULT_MODEL`: the player is not left in
their previous model, and the log says `client N Kyle in place of <model>`. A
model that failed is not loaded again until the next map
([clientinfo_refresh.rs](../crates/sjk-viewer/src/clientinfo_refresh.rs)). A model
that changes during the match (a join, `/model`, a forced-model or mute change) is
read on a worker, at most three at once: its files, its mesh and its textures
decoded there, then uploaded by the render thread, one model a frame
([player_loads.rs](../crates/sjk-viewer/src/player_loads.rs)); until then the player
keeps the model they had, and a new player shows the stand-in actor. Loaded inside
one frame before 10/10/2026, it was a stutter players felt on every join. Files are read as
rd-vanilla and the retail cgame read them, so a model EternalJK draws and
animates is not swapped for Kyle:

- Skins are read with rd-vanilla's `CommaParse` loop (`RE_RegisterIndividualSkin`,
  `tr_skin.cpp`): tokens in surface/shader pairs, comments, missing commas, stray
  text and bytes outside UTF-8 change nothing about what loads, `tag_` entries
  are skipped, `_off` is stripped from surface names, the first entry for a
  surface wins and a skin keeps at most 128 entries
  ([skin.rs](../crates/sjk-model/src/skin.rs)).
- A skin never costs the model (`CG_RegisterClientModelname`, `cg_players.c`):
  when the requested skin is missing, has a missing part or names no surface,
  the model wears `model_default.skin`, and without one its surfaces' own
  shaders. A name is a three-part skin only when it has `|` and says `head`,
  `torso` and `lower`; the console notes each fallback
  ([player_skin.rs](../crates/sjk-viewer/src/player_skin.rs)).
- A surface is drawn only while all its GLM flags are clear (`RenderSurfaces`,
  rd-vanilla `tr_ghoul2.cpp`): a tag (`*r_hand`, `*hip_l`, `*head_front`: the bolt
  flag) is never drawn, whatever the skin names for it. Community models often give
  their tags the shader `[nomaterial]`, which drew as small magenta arrows on the
  chest, the belt and the face until 15/10/2026 (Nib's report)
  ([lib.rs](../crates/sjk-model/src/lib.rs) `skin_with_matrices`).
- One leading slash on the mesh's skeleton name is dropped, as the filesystem
  drops it (`FS_FOpenFileRead`); vertex weights adding up past one are used as
  written (`G2_GetVertBoneWeight`); a mesh whose header bone count differs from
  its skeleton's loads when its bone references name skeleton bones.
- `animation.cfg` lines that name no animation, and sequences with no frames,
  are left out as `BG_ParseAnimationFile` leaves them; a table that then names
  nothing holds frame 0, as `G2_TransformBone` does.

Still drawn as Kyle: a model whose `model.glm` or skeleton is not installed or is
not version 6, a mesh referencing a bone past its skeleton, an `animation.cfg`
line naming an animation with other than five fields, and a model whose standing
animation lies past its skeleton's frames (rd-vanilla clamps such frames to 0).
SJK is more lenient than retail in two places: it keeps non-humanoid skeletons
and models without the hand, head or lumbar bolts the retail cgame checks for a
player, and a surface a skin does not name draws the mesh's own shader where
rd-vanilla draws its default shader.

SJK mounts `base` and, when set, `fs_basegame` and `fs_game`. EternalJK mounts its
own `EternalJK` folder by default (`fs_basegame EternalJK`), so skins shipped
there, such as `jedi/model_rgb.skin` in `japro-assets.pk3`, exist for EternalJK
and fall back to the default skin in SJK. Setting `fs_basegame EternalJK` and
restarting mounts it in SJK as well, together with that folder's menu, HUD and
string files.

The ignored test `player_model_scan` loads every installed model and skin through
this path without opening a window and compares each with what rd-vanilla and the
retail cgame would do:

```sh
JKA_GAME_DATA="/path/to/GameData" cargo test --release -p sjk-viewer \
    player_model_scan -- --ignored --nocapture
```

`JKA_MODEL_SCAN_GAMES=EternalJK` (comma-separated) mounts further folders above
`base` and reports only the models with files there. The table goes to
`target/parity-reports/player-models/`; see
[player_model_scan.rs](../crates/sjk-viewer/src/player_model_scan.rs).

## Hats and capes

SJK wears JoF EJK's free-choice cosmetics
([cosmetics.rs](../crates/sjk-viewer/src/cosmetics.rs)): every player wears
whichever hat and cape they like. A hat is any `.md3`
in `models/cosmetics/hats/`, a cape any in `models/cosmetics/capes/`
(`models/players/hats/` and `capes/` when the new folders are empty, as older
packs used them); names of JoF's catalogue are found in either. A name is at
most 13 letters, digits, `_` or `-` and does not start with a digit. JoF's
launcher installs its pack (`zzz_jof_cosmetics.pk3`) in the `EternalJK` folder:
SJK mounts every PK3 there that has models in `models/cosmetics/hats/` or
`capes/`, or JoF's client pictures in `gfx/jof/` (`jofclient-assets.pk3`, which
also holds the sounds JoF servers play, see [Force wheel](#force-wheel)),
below all other content, without mounting the rest of that folder
(its menus, HUD and strings come only with `fs_basegame EternalJK`, see
[Player models](#player-models)); the log names each pack. A pack in `base`
works too ([asset_search_paths.rs](../crates/sjk-viewer/src/asset_search_paths.rs)).

What a player wears travels in the saber colour keys, as JoF EJK sends it:
`color1 "4santahat"` is blue blade 4 wearing the hat `santahat`, and `color2`
carries the cape. SJK's `color1`/`color2` are therefore text cvars read with
`atoi`; the profile writes them as `<colour><name>`, the saber page keeps the
name when it changes the colour, and the userinfo carries the name after the
digits only when it follows the rule above
([jof_cosmetics.rs](../crates/sjk-client/src/jof_cosmetics.rs)). Servers copy
the keys to the `c1`/`c2` clientinfo untouched (cut to 15 bytes), and other
clients' `atoi` reads only the colour. SJK reads another player's blade
colour with `atoi` too: before, `c1 "8santahat"` failed to parse and drew blue.

Each player's pieces are resolved when their actor is built or their
clientinfo changes: the names, the models (loaded mid-match like a
configstring model, on first use) and the fitting offset from
`settings/cosmetics/<hats|capes>/<name>.cosmetic`, JoF's and TaystJK's JSON of
per-model and per-skin `xOffset`/`yOffset`/`zOffset` (exact keys, else the
longest `prefix*` key; the skin's entry wins, the model's applies with
`"modelFallback": true`). Each evaluated pose then reads the `*head_top` and
`*back` bolts of the worn slots only. A piece is drawn with the body as
`CG_DrawCosmeticOnPlayer` places it: the bolt's axes, two units down its up
axis, plus the offset along the world axes, never on the dead, the
mind-tricked or a scaled model, and on the local first-person player only in
mirrors and portals. A piece this client does not have is not drawn.
`cg_cosmetics` (archived, 1) draws everyone's (1), only yours (2) or none (0).

`cosmetics hats` and `cosmetics capes` list the installed pieces, with a
number or a name they wear one (the same again takes it off), `cosmetics
clear` takes both off and `cosmetics visibility [off|on|onlyme]` shows or sets
`cg_cosmetics` ([command.rs](../crates/sjk-viewer/src/cosmetics/command.rs)).
A colour number typed alone (`color1 2` in the console, a bind or a script, `set` and
`seta` too) keeps the worn piece after it, as JoF's own colour commands do; a value
with a name after its digits (`2tophat`, `4none`) is set as it is
([typed_colour.rs](../crates/sjk-viewer/src/cosmetics/typed_colour.rs)). JoF's
`saberColor <r> <g> <b>` (or `<blade> <r> <g> <b>`, or six values for the two blades)
makes the blades custom RGB, colour 6 with `cp_sbRGB1`/`cp_sbRGB2`, keeping the pieces
too ([saber_colour_command.rs](../crates/sjk-viewer/src/saber_colour_command.rs)).
The classic profile's Cosmetics window does the same, and the SJK UI's
Character page has Hat and Cape rows (None, then each installed piece; Enter or a
click opens a picker of named chips, [sjk-ui.md](sjk-ui.md#character)). The menu stage
model wears what `color1` and `color2`
name, placed on the bolts of the pose it is skinned with like its hilts
([menu_stage/cosmetics.rs](../crates/sjk-viewer/src/menu_stage/cosmetics.rs)),
unless `cg_cosmetics` is 0.

Every installed hat and cape is open to everyone: nothing is unlocked or
granted by a server. jaPRO's race-unlock hats (the `c5` clientinfo,
`cp_cosmetics`, `cosmetics unlocks`) are left out on purpose.

## Team saber colours

`cg_saberTeamColors` (archived, default 1; Settings > Advanced > Gameplay and classic+
Game options, **Team saber colours**) is JoF EJK's rule (`cg_players.c:8096-8111`,
`cg_xcvar.h:411`): in team gametypes (team FFA, CTF, CTY; not siege, not jedi-vs-merc,
never NPCs) a red player's blades are drawn in the stock red and a blue player's in
the stock blue, whatever `c1`..`c4` say: both sabers, held and thrown, a custom RGB
or a JA+ black blade included, and the player card's swatches. At 0 a blade keeps its
own colour unless it is the other team's (blue on red, red on blue) or past purple
(RGB, black and the other extended colours), as in EJK. Free players, spectators and
other game types keep each player's own colours. Fixes hub report 70 (random saber
colours in team FFA). An SJK blade shader stays worn and takes the team colour as its
base, as it takes the player's own colour elsewhere (a chroma turns to red or blue,
[unlockables.md](unlockables.md#chromas); integrator's choice, 17/10/2026, Sol may
revisit). See [team_sabers.rs](../crates/sjk-viewer/src/team_sabers.rs) and
`sjk_client::legacy_saber_team_colors`.

`cg_ctfTeamSabers` (archived, default 0; same pages, **CTF team saber colours**) goes
further in capture the flag and capture the ysalamiri: every red or blue player's blades
in their team's stock colour with no blade shader over them, muted players' default
saber included, whatever `cg_saberTeamColors` says. From SJK pull request #113 (Slash),
made an option at the merge (Sol's choice, 17/10/2026).

Verified by unit tests (team FFA, CTF and CTY force red and blue, free players and
spectators untouched, FFA, duel, siege and jedi-vs-merc keep the choice, the `0`
semantics per saber, black blades replaced per hand, the card's swatches); not tried
in the game.

## JA+ black sabers

On JA+/jaPRO and TaystJK servers, saber colour number 11 is shown as a black
blade with an alpha-blended trail and no blade light or bloom. It works for
both hands and thrown sabers. The archived `cg_blackSabers` setting is on by
default, under classic Settings > Game Options and SJK Advanced > Gameplay.
Turning it off uses orange. The existing `cp_pluginDisable` bit 3 also disables
black blades when set: `japlus.plugin 3` toggles it, and a clear bit means allowed.
The normal plugin-mask default (1536) allows black blades.

Other servers show colour 11 as before: purple, or the player's RGB tint when
one is set. (JoF EJK shows orange there; SJK keeps its earlier colour.) Ordinary RGB colours, including
an RGB tint of zero, retain their existing behavior; SJK blade skins retain
priority over the server's blade colour. Installed community black glow/core/
trail images are used when available; otherwise SJK generates black alpha
profiles locally. No extra content download is required. See
[rendering](rendering.md#ja-black-sabers) and
[verification](status.md#ja-black-sabers).

## EternalJK animation fixes

Players' animations are remapped before they are shown, as EternalJK's `CG_Player`
does: without a saber in hand, two-handed, dual and staff runs and walks play as the
ordinary ones; the old Bryar's `BOTH_STAND1` (servers without `g_fixWeaponAttackAnim`,
such as JoF's, fire it that way, arm hanging between shots) shows as `BOTH_ATTACK2`
with the arm raised; the concussion rifle's `BOTH_ATTACK2` shows as `BOTH_ATTACK3`.
NPCs are left as the server sends them.

## Tavion and Desann idle poses

Tavion and Desann saber styles use their distinct JoF EJK standing poses by
default, with no setting: Tavion uses `BOTH_P1_S1_T_`, Desann `BOTH_STAND9`.
This applies to snapshot and predicted actors in every UI style. The client
replaces only each style's ordinary idle clip; other custom ready animations,
attacks and movement clips stay as supplied. Unlit or thrown sabers and other
weapons retain their ordinary presentation. Already distinct server-sent idle
poses pass through unchanged.

The change is visual: simulation, prediction inputs, attack timers and protocol
26 remain unchanged. Source choices follow [JoF EJK #47](https://github.com/JediofFreedom/JoF_EJK/pull/47).
Live appearance with installed animations remains unverified; see
[status](status.md#tavion-and-desann-idle-poses).

## Wall grab facing

A player grabbing a wall to jump off it (Force jump 3, pushing into a wall in the
air: `BOTH_FORCEWALLREBOUND_*`, then `BOTH_FORCEWALLHOLD_*` while the hold lasts)
is drawn facing the wall the way the animation needs, whatever their view does
([wall_hold_pose.rs](../crates/sjk-viewer/src/wall_hold_pose.rs)). `CG_G2PlayerAngles`
turns the body with the view yaw. A stock server turns that yaw to the wall during
the hold (`PM_AdjustAngleForWallJump`), so the camera locks too, but a JA+ server
leaves the view free there ([networking](networking.md#server-dialect-movement-rules)),
and looking around turned the body off the wall, as if it floated. While the
legs play a rebound the model's yaw is the stock server's facing instead
(`vectoyaw(wall normal) + yawAdjust`, from the same 128-unit check along the
view, [`wall_hold_yaw`](../crates/sjk-game-jka/src/pmove_wall_moves.rs)); a frame
whose check misses keeps the rebound's last facing, and the kick off returns the
body to the view. It applies to the local player (from the predicted state) and to
other players alike. Only the model changes: the camera stays free on JA+ and
locked on a stock server, as each server computes it, and the kick off still goes
straight back from the side the view picks (`checkDir`), so on JA+ it follows
where the player looks, not the wall. The check sees the world's brushes only: a
wall on a mover keeps the body on the view.

## Dismemberment and disintegration

Both follow EternalJK's cgame (`cg_ents.c`, `cg_players.c`) and rd-vanilla's renderer.

- **Cut-off limbs.** A server with `g_dismember` above 0 sends each cut limb as its
  own entity. With `cg_dismember` 1 (no heads or waists) or 2 and up (everything) the
  client copies the owner's model for the limb, draws only the limb and its cap, and
  turns the limb off on the owner with the stump's cap on; a cut right arm, right hand
  or waist takes the weapon with it. Both cuts smoke, and a flying limb trails smoke.
  A body left behind keeps the missing limbs; the player is whole again once alive.
  `cg_dismember` defaults to 2 (EternalJK: 0), so a server that enables
  dismemberment shows every cut limb (0 shows none, and a config that saved 0
  keeps it). `g_dismember` (0 to 100) set in the
  console is handed to games this client hosts (Create game and `devmap`).
- **Disintegration** (`EF_DISINTEGRATION`): a disruptor kill, or a corpse shot or cut
  until it gives way, freezes the pose and burns the body away from the hit point:
  the model is eaten away with a blackened edge and `gfx/effects/burn` glows on what
  remains, with `disruptor/death_smoke` for the first second. A player is gone after
  1.5 s; a body draws until the server removes it. The stage program does rd-vanilla's
  per-vertex work (`RB_CalcDisintegrateColors`, `RB_CalcDisintegrateVertDeform`).
  It is always on, with no cvar, as in EternalJK and OpenJK (`CG_Player` checks only
  `EF_DISINTEGRATION`). Its colours are final: real-time sun light and fullbright
  leave them as they are, as the stage's colour generators do (`entity_control.x` 3).

Model cap surfaces (`*_cap_*`) are carried in every actor mesh, hidden until a cut.

## Third-person camera

Third-person framing follows OpenJK multiplayer `CG_OffsetThirdPersonView`.
The camera uses a four-unit collision hull against solid/terrain/player-clip
surfaces and presented inline models, including moving doors and platforms and
the permanent ones the server sends only in its baselines, such as the `misc_bsp`
map pieces a server places in a map's void: stock `CG_BuildSolidList` adds them
from `cg_permanents` (`cg_predict.c`), so the camera stops at their walls. The
crosshair name trace uses the same list, so such a wall hides a player behind it.
Player and vehicle bodies do not obstruct this camera trace. When geometry
collapses the camera onto its target, the view uses the intended forward direction
instead of constructing an undefined look-at matrix.

Pitch limits, pitch-offset direction and turn-dependent damping follow the
multiplayer reference. How fast the camera closes the gap follows JoF EJK's
`cg_cameraFPS` (default 125, as EternalJK): the damping cvars apply per frame of
that rate whatever the real frame rate, and the ideal point's own movement is
compensated (EternalJK `CG_DampPosition`), so the camera stays close behind a
fast-moving player. As in EternalJK the damping is timed by the predicted
player's command time, the clock the focus moves on; timed by the presentation
clock it stuttered. Stock multiplayer applies them once per 50 ms step, against
8 ms per step at the default 125, so the same damping value closes the gap in
steps 6.25 times shorter; `cg_cameraFPS` below 15 restores the stock behaviour.
View changes, teleports, followed-player changes and mounting/dismounting reset
the presentation history. The ordinary range, height, angle and damping cvars
remain available; `cg_thirdPersonHorzOffset` controls the stock sideways offset.
`cg_cameraStyle` chooses whether the damping applies ([Camera style](#camera-style)).

The decaying prediction error (`cg_errorDecay`) moves the third-person camera's
focus before its collision and damping traces, as `CG_CalcViewValues`
(`cg_view.c`) adds it to the view origin before `CG_OffsetThirdPersonView`. A
correction next to a wall is therefore traced like any other focus movement
instead of shifting the finished camera into the wall. First-person and other
views still add it to the finished view origin.

Vehicle appearances cache their `.veh` camera settings when loaded. Mounted
views use the authored range, height, pitch and sideways offsets, including the
pitch-dependent and fighter-strafe adjustments. Vehicle targets follow immediately
and camera damping follows the stock vehicle policy. These are presentation
changes; movement, vehicle physics and protocol serialization are unchanged.

The locally piloted vehicle is drawn at the same predicted origin and angles as
its movement, following `CG_AddPacketEntities` and `CG_CalcEntityLerpPositions`.
Its rider seat uses that same root with the current animated driver bolt. This
avoids the camera advancing ahead while the mount and rider trail behind on
snapshot interpolation. Remote vehicles, passengers without local vehicle
prediction, and demo playback retain their snapshot presentation. The camera's
animated seat is evaluated every rendered frame. Prediction-error smoothing uses
the vehicle root while piloting, avoiding false corrections from rider animation.
The movement collision adapter excludes the piloted vehicle's own snapshot body
and its owned objects, matching the stock skip/ownership rules; dismounted and
foreign vehicles remain solid.
Snapshot replay also retains the same ride's local vehicle timers, including turbo
expiry and recharge, while refreshing its networked state. A different vehicle,
pilot or definition starts with fresh local state. This prevents exhausted boost
input from predicting a new burst after every snapshot.

The decaying prediction error moves the camera's focus before the
collision traces, as `CG_CalcViewValues` adds it to the view origin before
`CG_OffsetThirdPersonView`, rather than shifting the finished camera, which could
put it inside a wall. `cg_thirdPersonAlpha` is not implemented; stock multiplayer
has no automatic fade when the camera nears the player. See
[camera.rs](../crates/sjk-viewer/src/camera.rs) and
[camera_motion.rs](../crates/sjk-viewer/src/camera_motion.rs).

### Camera style

`cg_cameraStyle` (archived; Settings > Gameplay > Game options, "Camera and
prediction", and First setup's third row, under Styles) picks how the camera
follows the player ([camera_style.rs](../crates/sjk-viewer/src/camera_style.rs)):

| Value | Camera |
| --- | --- |
| `ejk` (default) | Sits at its ideal place every frame: no camera or target damping, whatever the damping cvars say. |
| `sjk` | Eases towards its ideal place with `cg_thirdPersonCameraDamp` and `cg_thirdPersonTargetDamp`, as described above, so it trails a moving or turning player a little. |

An unknown value selects `ejk`. `ejk` became the default on 08/10/2026 (Sol's
choice; `sjk` was the default before). Every profile had saved `sjk`, so the
console's start moves a saved `sjk` once to `ejk` and marks the profile
(`cg_cameraStyleDefaultVersion`); an `sjk` picked after that stays.

`ejk` is JoF EJK's camera as Sol plays it (07/10/2026). The camera code is the same
in both clients: EternalJK's damping with `cg_cameraFPS 125`, range 80, height 16
and damping 0.3 and 0.5 are SJK's defaults too. But EternalJK switches both
dampings off while a strafe helper style is drawn (`cg_strafeHelper` bits 0-3 or
13; `CG_UpdateThirdPersonTargetDamp` and `CG_UpdateThirdPersonCameraDamp` in
`cg_view.c`), and Sol's JoF EJK profile has one on (`cg_strafeHelper 2242`), so
there the camera never trails. Range, height, angles, collision, vehicle framing
and the first/third person choice are the same in both styles; Sol's JoF EJK
profile also sets `cg_thirdPersonRange 100`, which SJK leaves to that cvar. SJK's
`sjk` style does not apply EternalJK's strafe helper rule.

### Camera settings

Settings > Advanced > Gameplay > Camera and prediction, and Classic+ Setup >
Game options, expose the camera controls from JoF EJK's
`assets/japro/ui/jamp/ingame_camera.menu` at official alpha commit `9822a9d6`
(`1.6.1-alpha.347`; the file is unchanged since its initial `52e068d0` import). The
shared settings rows cover the first-person weapon FOV; EJK's damping timing
presets (Default 125, Smooth (SP) 30 and BaseJKA); third-person distance, angle,
pitch, height and horizontal offset; and camera/target damping. Their ranges and
defaults match that menu. The ordinary world FOV stays under Display, the HUD
switch under HUD, and first/third-person switching remains the Camera key rather
than a persistent option. The damping rows (timing preset, camera and target damping)
only act under `cg_cameraStyle sjk`; the default `ejk` style keeps its own camera.

### Camera control

Camera control (named Shot controls until 08/10/2026) is a panel for framing
screenshots and recordings in a match or a demo
([ingame_menu/shot.rs](../crates/sjk-viewer/src/ingame_menu/shot.rs)). The game
menu's camera icon opens it (the SJK UI's row of icons; the classic bar has no
entry), as do F8 while F8 has no binding, the `cameracontrol` command (bindable
under Controls > Other) and the quick wheel's Camera control. It drives the
console's presentation director, the same one as the `demo_camera` and
`demo_sun` commands, which keep their names: nothing it does reaches the
server, prediction or the player's input.

- **Camera page:** View from Back, Front, Left or Right; Angle (-180 to 180),
  Pitch (-80 to 80), Distance (16 to 512) and Height (-64 to 160) of the
  third-person camera, starting from where it is (`cg_thirdPersonAngle`,
  `cg_thirdPersonPitchOffset`, `cg_thirdPersonRange`,
  `cg_thirdPersonVertOffset`). A change turns third person on.
- **Sun page:** the sun's direction (0 to 360) and elevation (-20 to 90), when
  the view has real-time lighting (`r_dayNight 1` at launch) and open sky, what
  holds the sun now (the day settings, Camera control, or returning), and Reset
  sun, which hands it back to the day settings at once.
- **Motion:** Move duration (0.2 to 20 seconds) and Orbit speed (-30 to 30
  degrees a second); Orbit turns the camera (or the sun) round, Stop holds it,
  Reset (the Sun page's Ease back) eases back to the settings over the move's
  duration.
- **Live preview** (on by default) shows each change at once; off, changes wait
  for Move and hide, which moves there over the move's duration. **HUD**
  switches `cg_draw2D`. **Hide panel** (Move and hide) closes the panel and
  applies; **Close** closes it without moving.

Keys: Up, Down and Tab move from control to control (a slider's number and track
are one stop), Left and Right step a slider, a digit types its value (Enter
applies, Escape cancels), Enter or Space acts, F8 or Escape hides the panel. The
pointer drags a slider's track, a click on its number types it, and a click acts
on a button, a tab or a switch. The panel has the SJK UI's look in every menu
style ([SJK UI](sjk-ui.md#camera-control)); the classic menus' right-edge panel
was removed on 10/10/2026.

## Ghost replay

`ghost` records the player's own run and plays it back later on the same server, to
film a showcase: the player records a run once, then replays it while another player
films ([ghost_replay.rs](../crates/sjk-viewer/src/ghost_replay.rs)).

`ghost` and [`mimic`](#mimic) are experimental and off by default: Settings > Mods >
"Experimental: showcase tools (ghost, mimic)" (`cl_experimentalShowcase`, archived;
"Experimental showcase" in the classic menus) turns both on. Off, either command says
so and where to turn it on; switching it off ends a replay or a copy, and a recording
is saved. They have no other menu entry.

| Command | Does |
| --- | --- |
| `ghost record <name>` | Records every command from the next one: view, movement keys, buttons, weapon, Force selection and generic actions, with the position and speed after each |
| `ghost stop` | Ends a recording and saves it, or ends a replay |
| `ghost play <name>` | Walks to the recorded start, then plays the run |
| `ghost list` | Lists the saved runs with their maps and lengths |
| `ghost info <name>` | A run's map, length, start (position and facing, to teleport there) and how much of it the server confirmed |
| `ghost` | Help and what it is doing |

Runs are text files in `ghosts/` beside `config.cfg` (`<name>.ghost`, letters,
digits, `-` and `_`): a header with the map and start, then one line per command
(time, view, keys, buttons, selections, then the origin, velocity and ground after
it). While recording, each command keeps the predicted state after it until a
snapshot acknowledges that command; the server's state then replaces it. A recording
ends on its own at ten minutes, on death or spectating, on flight, or on leaving or
changing map, and is saved.

**Replaying.** Movement is deterministic: the same start and the same commands with
the same durations give the same run whatever the ping, because the server steps each
command by its own time (`ClientThink_real`). A replay sends the recorded commands with
fresh stamps; the view the server derives (`cmd.angles + delta_angles`) is the
recorded one, so a different `delta_angles` (after a respawn or a teleport) does not
turn it. The camera follows the replayed view. Commands the clock passes over (a
slower frame) give their presses to the next one, so a one-command jump or Force press
is not lost. Force-wheel reliable commands (Stasis, Repulse, Dash) are not recorded.

**The start.** `ghost play` refuses a run recorded on another map or a start more
than 192 units away (it prints the start, to go or teleport there). Closer, it walks
there at walking speed in the current view, switches to the recorded weapon, faces
the recorded view and stands still for a few commands, then begins. A start reached
within half a unit and standing still replays the run untouched. Start recordings
standing still too.

**Drift.** The server owns the position, so a replay corrects drift (a command
lost twice merges into the next one, a shove) through the movement keys alone, and
only on the ground:

- The keys turn, up to 20 degrees, keeping the largest key's size so the speed
  stays (`PM_CmdScale`); the view stays the recorded one. They first take out a
  sideways speed, then close the offset at a unit a second per unit of drift (at
  most 20). Drift under 2 units with a sideways speed under 1.5 is left alone:
  Pmove rounds the velocity to whole units after every command and friction cannot
  shrink a small sideways speed past that rounding, so a tiny correction would leave
  a sideways speed for the whole run.
- In the air nothing steers: air control (`pm_airaccelerate` 1) adds less than the
  rounding takes back. What a jump carries is the speed it left the ground with,
  which the ground steering has already matched.
- Behind or ahead along the path by more than 2 units on steady ground (the same
  keys as the next command), the recorded clock waits a command or skips one, up to
  a second in all.

A replay stops, giving control back, at the end of the run, when the player presses
any movement key or button, takes damage, dies, spectates, starts flying
(`freecam`, `fakenoclip`) or a duel begins, or when it drifts more than 48 units
sideways or 64 vertically ("lost the run"). It refuses to start in a duel or within
five seconds of taking damage. The last line says how many corrections it made and
the worst drift.

The tests run the real Pmove on a synthetic course (a platform, four pillars over a
pit, a landing) with a scripted pillar-jumping run: replayed from its start it ends
where the recording did; walked to from 30 units away with the server losing the
second takeoff's command and a command in the air, it lands every pillar and ends
0.4 units from the recording; shoved sideways at 60 units a second on the platform,
it ends 7 units off where an unsteered replay ends 46 off. Not yet tried on a live
server.

## Mimic

`mimic <player> [delay] [mirror|copy] [walk]` copies another player's moves live: their view,
movement, jumps, crouches, saber attacks, stance and holster
([mimic.rs](../crates/sjk-viewer/src/mimic.rs)). It is meant for a saber showcase with
two players facing each other. The player is a slot number or part of a name (as
`tell` takes it); the delay goes from 0 to 10 seconds (`0.5`, `2s`); `mirror`, the
default, copies like a reflection, and `copy` keeps the leader's sides; `walk` walks
always, for a choreography done at walking pace, instead of walking only while the
leader moves slower than 140. `mimic off` stops it and `mimic` alone says what it does.
Like `ghost`, it is experimental and off until Settings > Mods > "Experimental:
showcase tools" turns it on ([Ghost replay](#ghost-replay)).

The client never sees another player's keys, only what they lead to: each snapshot
carries their origin and velocity (`trBase`, `trDelta`), view angles (`apos.trBase`),
ground entity, linked box (`solid`'s top tells a crouch, 16, from standing, 40), saber
move, stance (`fireflag`), holster and weapon. The mimic keeps these samples, plays them
back on its own timeline after the delay (interpolated, extrapolated at most 100 ms past
the newest) and rebuilds commands:

- **Facing each other.** Where both stood and faced when it began are the anchors. The
  leader's motion since then, in their own frame, becomes the mimic's in its frame: the
  leader stepping toward the mimic makes the mimic step toward them. `mirror` swaps
  left and right, so a step to the leader's left is a step to the mimic's right and both
  stay face to face; their yaw turns mirror too. Pitch is copied.
- **Movement.** On the ground the keys aim for the leader's velocity, read 80 ms ahead
  where the delay leaves samples there (so a turn starts when theirs did, not an
  acceleration later), plus a return toward their mapped position at 4 units a second
  per unit beyond 2, with part of the gap to the current velocity added to take a
  sideways speed out. A slight backward component is dropped, since any backward key
  slows the player (`BG_AdjustClientSpeed`: 0.75, 0.45-0.75 in an attack). Walking
  pace (under 140), or always with `walk`, sets `BUTTON_WALKING` for the walk
  animation, and every key is then scaled within 64: Pmove clears the walk past half a
  key, "to avoid proxy no-footsteps cheats" (`bg_pmove.c:10490-10494`), so a mimic
  catching up with a larger key would run. A saber attack is chosen by the keys' signs,
  so walking attacks keep their direction. In the air the keys
  follow the leader's horizontal acceleration (air control pushes along the keys), not
  the offset.
- **Jumps and crouches.** A takeoff faster than 100 upward is a jump, held while the
  leader rises (a Force jump's height follows), let go for a command first if jump is
  still held. A crouched box crouches once the mimic is under walking pace: crouching
  while running rolls (`PM_Footsteps`), so a crouch waits for the mimic to slow down,
  unless the leader moves faster than 200 and is rolling too.
- **Saber.** The leader's saber move names the controls that ask for it, the inverse of
  `PM_SaberAttackForMovement`: a directional attack, its start or a transition into it
  holds attack with the keys of the quadrant it starts from (right and forward for the
  top-left start, right alone for the left, forward for the overhead); `mirror` swaps
  the quadrants and the specials' left and right forms. Those keys are pressed only where
  a move is chosen, starting from the ready or a return and in the last 24 ms of an
  attack (`PM_WeaponLightsaber`), where the leader 32 ms ahead says what comes next; in
  between the keys follow. Specials hold their controls throughout: the backstabs (back),
  the lunge and spins (crouch and forward), the DFA, flip over and jump attacks (jump
  and forward), the cartwheels and butterflies (jump and a side), the backflip, the
  kicks (alternate attack and a direction), the katas (both buttons); the others hold
  attack. A bounce keeps the buttons; parries, knockaways and locks, the fight's and not
  inputs, let go. A different stance presses `saberAttackCycle` (every 400 ms, eight
  tries for a stance this player lacks) and a different holster `sv_saberswitch`.

With no delay the copy runs as soon as the leader is seen, which is after their ping,
the snapshot interval and this client's ping, and nothing ahead is known. It stops on any
key or button the player presses, death, spectating, flight, a duel, leaving, a map
change or the leader out of sight for three seconds; it refuses to start in a duel, on
oneself or during a ghost replay. Taking damage does not stop it: blades meet in a
mirrored showcase.

The tests run the real Pmove for both players on a flat floor: a scripted leader turns,
walks, strafes, jumps, crouch-walks and attacks (two chained attacks, then an overhead),
with snapshots every 24 ms reaching the mimic 60 ms late, the mimic facing them 128 units
away. With delays of 0, 0.3 and 1 second the mirror does every saber move mirrored (top
right, from the left, then the overhead), each attack starting 4 to 24 ms after it
should, jumps and crouches, turns its view 30 degrees the other way and ends within 2
units of the mirrored spot; during the run it stays within 10 units with a delay and 31
with none (a full-speed leader cannot be caught up). `copy` keeps the sides. With `walk`
and a leader doing the same choreography at walking pace (keys of 46, walk held,
attacks included) every command holds the walk with no key past 64, the attacks come
mirrored 4-24 ms late, it crouches, tracks within 8 units with no delay and 5 with 0.3
seconds, and ends within 2. Not yet tried in a game.

## Jetpack sounds

Settings > Audio (classic) and Settings > Advanced > Sound (SJK UI) offer
**Jetpack ignition sound** and **Jetpack hover sound**, matching JoF EJK #43.
Both default to `1`; `2` selects the alternate. The archived console cvars
are `cg_jetpackOnSound` and `cg_jetpackHoverSound`. As in EJK, values <= 1
use the default and values > 1 use the alternate; zero does not mute them.

| Choice | Ignition | Hover |
| --- | --- | --- |
| `1` (default) | `sound/chars/boba/bf_land.mp3` | `sound/chars/boba/jethover.wav` |
| `2` | `sound/chars/boba/jeton.wav` | `sound/chars/boba/bf_jetpack_lp.wav` |

Shutdown always uses `sound/chars/boba/bf_blast-off.mp3`; EJK has no shutdown
selector. The additional `sound/effects/fire_lp.wav` loop while flaming is
independent of the hover choice. A hover change applies on the next audio
frame; changing ignition during flight does not restart it. Choices apply to
every audible jetpack, including your own.

Start/stop sounds use the wearer's actual source and position on `CHAN_AUTO`,
including EJK #87's later attribution/channel correction. All variants are
preloaded/registered before frame playback; choosing them does not read or
decode sound assets during a frame. See [rendering](rendering.md) for the
existing pack, flame and visibility behavior, and
[status](status.md#jof-ejk-jetpack-sound-choices) for verification limits.

## Saber ignition and retraction sounds

Two sources voice a saber turning on or off, as in OpenJK `codemp`. The server
plays the hilt's own sound when a held saber is toggled and sends
`EV_SABER_UNHOLSTER` when an attack ignites a holstered one. Drawing the saber
from another weapon (fists, a gun), or putting a lit one away, is voiced by cgame
alone: `CG_CheckPlayerG2Weapons` for the local view and `CG_Player` for everyone
else compare the weapon whose model the player holds with the current one
(`cg_weapons.c:2498-2580`, `cg_players.c:8932-9001`). SJK mirrors that once per
rendered frame
([sound_saber_switch.rs](../crates/sjk-client/src/sound_saber_switch.rs)): the
local view from the predicted state in live play, from the snapshot's playerstate
in demos and when following, and other players from the presented snapshot. Each
hilt's `soundOn` or `soundOff` from `ext_data/sabers/*.sab` (`enemy_saber_on` and
`enemy_saber_off` when a definition gives none) plays on `CHAN_AUTO` from the
player's entity; a second saber adds its own, and a two-handed hilt drops a second
one. `EV_CHANGE_WEAPON` has no select sound for the saber, so nothing is doubled.
`EV_SABER_UNHOLSTER` plays the player's own hilts too; an NPC's keeps the stock
`saberon`, and so does a player whose clientinfo names no hilt (no server sends
one). A player wearing a [blade skin](#blade-skins) also has its skin's single
ignition or switching-off sound play once for the switch or the unholster, over
both hilts' sounds, which keep playing; each hilt's `soundOn`/`soundOff`, then
the defaults above. The server's toggle plays the stock sound and the skin's over
it, which the switch path never adds to, since the held weapon does not change.

cgame's quirks are kept. A single saber's ignition also plays `enemy_saber_on`,
the default sound of its removed second slot, as the server's toggle does. A
caught saber is silent, and so is drawing it again from fists after it was knocked
away. A player who comes into view or respawns holding a saber is silent, but one
who left view with a lit saber and returns with a gun is heard retracting it. The
local player hears the ignition on a map's first frame, on joining from
spectator, on respawning with a saber after dying with another weapon and on
choosing another hilt with the saber out. A severed arm, which takes the weapon
model off in cgame, is not modelled.

`fd.saberAnimLevelBase` is no wire field, so the client works it out for every
command it predicts, as `CG_PredictPlayerState` does before `Pmove`
(`cg_predict.c:1335-1347` in EternalJK and JoF EJK): the current style, unless one
blade or saber is off (`saberHolstered` 1), when it is the staff base for a first
saber with blades and the dual base for a second saber alone
([pmove_saber_base.rs](../crates/sjk-game-jka/src/pmove_saber_base.rs); the hands
come from clientinfo and the `.sab` files, `legacy_saber_hands`). Without it a half
staff or one saber of a pair counted as "sabers off": the swing became a draw, the
server's `saberHolstered` 1 undid it and `EV_SABER_UNHOLSTER` played again. This
follows the reference's rule from reading it; it has not been run against a live
dual or staff in the fast style.

## Animation sounds and voice variants

Footsteps and authored swing/spin sounds follow the evaluated lower/upper Ghoul2
frames and the model's `animevents.cfg`, including the shared skeleton table and
`include` directives. The client reads supported sound assets during appearance
loading and queues decoding on the audio worker; ordinary frame playback performs
no file reads. A world build shares the skeleton and event assets across matching
appearances. Active actors release their prefetched encoded bytes after registration.

Ground-contact events trace beneath the animated foot and select the authored
walk/run sound bank for the surface material. Ground the map gives no material
(no `MATERIAL_*` bits, which codemp plays as stone) takes its bank from the hit
shader's file name in SJK (`footsteps::material_from_name`: sand but not sandstone,
snow, grass, gravel, mud, dirt, carpet, wood or plank): retail leaves much of its
sand untagged, such as `mp/siege_desert`'s `siege/siege2sand`, where Sol asked for
sand steps (world note, 07/10/2026). The collision trace reports the brush side's
or patch's shader for this (`CollisionTrace::shader`); `cg_footstepGuess 0` turns the guess
off (stone, as retail), and it applies to dust and prints too. As in `_PlayerFootStep`, a
foot in water (`MASK_WATER`) makes no step. `cg_footsteps 0` mutes them. Frame
latches prevent repeated playback while an animation frame is held; absent actors,
teleports, backwards seeks and paused map changes reset the cursor. First-person
local actors use the same evaluated timing. This restores the blue-stance taunt's
spin sounds and authored melee/kick swing cues. Custom saber `spinSound` and
`swingSound1`–`3` override the standard animation samples.

Taunt, flourish and gloat voice choices advance per accepted event, with fallbacks
based on samples that actually resolved. The expanded taunt bank is used in FFA
as in TaystJK, rather than restricting ordinary FFA taunts to `taunt.wav`. Selection
is replay-stable but is not the legacy global random stream; repeats remain
possible. Animation selection, movement, saber timing and network events are
unchanged. Gameplay event actions remain outside this audio adapter.

### Dust and footprints

A step on mud, dirt, sand or snow also kicks up dust and leaves a footprint, as
`_PlayerFootStep` does with retail's default `cg_footsteps 3` (`cg_players.c:2889-3086`,
[footprints.rs](../crates/sjk-viewer/src/footprints.rs)): the material's effect
(`materials/mud`, `materials/sand` for dirt and sand, `materials/snow`) along the ground's
normal, and the print (`footstep_r`, `footstep_l`, `footstep_heavy_r` or `footstep_heavy_l`
for a run, radius 6) turned with the legs. Gravel kicks up dust and leaves no print. A
print is a lasting cgame mark (`CG_ImpactMark` with `temporary` false): it is drawn for
ten seconds, darkening over the last one, at most 256 mark polys with the oldest mark
given up for a new one (`CG_AddMarks`, `CG_AllocMark`; the third list of
[decal_store.rs](../crates/sjk-viewer/src/decal_store.rs)), and `cg_marks 0` drops it.
The left foot's shader mirrors the right's picture with `tcMod transform -1 0 0 1 1 0`,
which effect stages now read as a mirror.

SJK saved `cg_footsteps` as an on/off `1`, which retail reads as "sounds only", so the
dust and prints have their own switch, `cg_footprints` (on; Settings > Sound,
"Footprints and dust"); `cg_footsteps 0` still turns off all three. Retail's untagged
sand (`mp/siege_desert`'s `siege/siege2sand`) only gets prints through the texture-name
guess above.

### Jump and landing steps

Retail's `_humanoid/animevents.cfg` steps in its side, back and Force landings but not
in the jumps or the forward landing (`BOTH_LAND1`), so a plain jump on sand makes no
sand sound. SJK adds both feet at the second frame of `BOTH_JUMP1`, `BOTH_JUMPBACK1`,
`BOTH_JUMPLEFT1`, `BOTH_JUMPRIGHT1`, the four `BOTH_FORCEJUMP*1` and `BOTH_LAND1`, as run
steps with their sound, dust and print (`JUMP_LAND_STEPS` in
[footsteps.rs](../crates/sjk-client/src/animation_events/footsteps.rs)), unless the
model's own table already steps somewhere in that animation. `cg_jumpFootsteps 0`
(Settings > Sound, "Jump and landing steps") keeps retail's steps alone.

## Mover models

A brush entity a map gives a `model2` key draws that model too: the game registers it as
the mover's `modelindex2` (`InitMover`, `SP_func_usable`, `g_mover.c`) and `CG_Mover`
draws it at the mover's origin and angles, scaled by `iModelScale` when set
(`cg_ents.c:3469-3479`). `t2_rancor`'s bar tables are such movers: `func_usable` clip
brushes, with no drawn face, carrying `models/map_objects/nar_shaddar/table01.md3`, so
until 13/10/2026 SJK showed no tables at all (Sol's world note, 08/10/2026). The rule is
`legacy_mover_model2` in [entity_models.rs](../crates/sjk-client/src/entity_models.rs);
[mover_models.rs](../crates/sjk-viewer/src/mover_models.rs) loads the models (the
baselines' at map load, later ones on first use) and draws them with the rigid entity
models, for the snapshot's movers and the permanent ones. `EF_NODRAW` hides the model as
it hides the brush. Multiplayer ignores a `func_usable`'s `modelAngles` (single player's
key) and leaves out a `.glm` `model2`; so does SJK. Tables that start off
(`spawnflags 1`) stay hidden unless a script brings them back.

The native server sends a `func_usable` the same way: at its origin (`s.pos.trBase`),
with its `model2` registered, solid to players and traces where it stands (it used to be
neither placed nor solid), and the use key fires one only when the map made it
`PLAYER_USE` (spawnflag 64) and not `INACTIVE` (128), as `ValidUseTarget` checks. One that
starts off is not sent (`SVF_NOCLIENT`, `EF_NODRAW`); it used to carry `EF_PERMANENT`
(bit 7) in the place of `EF_NODRAW` (bit 8).

## Door and mover sounds

A brush entity (a door, lift, button or breakable) has its entity origin at the
world origin unless the map gave it an origin brush, so codemp plays its sounds
from the middle of its model (`CG_SetEntitySoundPosition`, `cg_ents.c:118-132`).
SJK does the same: an event sound on a brush entity, such as a door's start and
end sounds (`EV_PLAYDOORSOUND`), and the position the mixer moves the entity's
sounds to on every snapshot are its origin plus its inline model's midpoint
(`LegacySoundAdapter::sound_origin` in
[sound_events.rs](../crates/sjk-client/src/sound_events.rs), with the midpoints
taken from the map when its sound tables are built). A mover's loop already came
from the same point. That origin is the entity's `pos.trBase`, where codemp
evaluates its trajectory, so a door that turns back halfway sounds from where
its move began.

The start and end sounds come from the mover's soundset (`CS_AMBIENT_SET`): `G_PlayDoorSound`
raises `EV_PLAYDOORSOUND` (71) on a door, plat or button with one, and scripts and
breakables raise `EV_BMODEL_SOUND` (73); both play the soundset's sound for the stage, and
`EV_PLAYDOORSOUND` nothing without a soundset (`CG_PlayDoorSound`). Until 12/10/2026 SJK
answered only 73, taking it for the door event, so doors with a soundset were silent on
every server (Creyon's report: some maps' doors).

A vehicle's own `EV_FIRE_WEAPON` and `EV_ALT_FIRE` (an `ET_NPC` of `CLASS_VEHICLE`)
play nothing, as in `CG_EntityEvent` (`cg_event.c:2751-2760`, `:2779-2784`): they used
to play the flash sound of whichever ordinary gun the vehicle's `weapon` field named.
A rider's own events are unchanged. Stock gives a vehicle's guns their sound through
the `muzzleFX` effect that `EV_VEH_FIRE` plays bolted to the vehicle
(`CG_VehMuzzleFireFX`); SJK does not play that effect yet, so vehicle fire is silent
until it does.

## Electrified players' crackle

A player whose `ps.electrifyTime` (`emplacedOwner` on other players) is ahead of the
clock wears the electric shell and crackles: on each frame the shell is drawn, one draw
in ten starts `sound/effects/energy_crackle.wav` on the player (`CG_Player`,
`cg_players.c:11060-11098`). A DEMP2 hit sets that time, and a JoF server sets it for
about four seconds when an admin logs in with `amlogin` (its only login sound: the rest
is a print). SJK had the shell but not the sound. It now plays it, your own in first
person too, but rolls on the 8 ms effect cadence instead of every rendered frame, so it
crackles about seven times a second at any frame rate, as stock does at
`com_maxfps 125`. A muted player and one hidden from you by a mind trick make no crackle
([electrify.rs](../crates/sjk-viewer/src/electrify.rs)). JoF EJK's skip while a saber
deflects lightning (`CG_LightningDeflectionActive`) is not followed, as the shell does
not follow it either.

## Renderer settings

SJK's own cvars have engine names: rendering ones are `r_*` (`r_sceneHdr`,
`r_toneCurve`, `r_sceneBloom`, `r_superSample`, `r_actorSunShadows`,
`r_sunShadow*`, `r_dayNight`, `r_liveLighting`, ...), the ground HUD is
`cg_groundHud` and the dedicated server's are `g_npcNav` and `g_stockRules`.
The ground HUD (health, shield and Force at the player's feet in third person)
stands in for the status HUD; a zoom that shows the view in first person (the
disruptor's scope, the binoculars) keeps the status hidden rather than bringing it
back for the zoom (`Presentation::Covered` in `ground_hud/readout.rs`).
Names rend2 or EternalJK use with another meaning are avoided.

These rendering cvars have their own four groups. The SJK UI's Settings
gathers them in its Graphics category; with the classic menu style they are
panels of the Setup page's GRAPHICS page (in the main menu and the in-game
pop-up), after Video. A renderer entry without a panel of its own opens the SJK
UI's Settings on Graphics, and backing out shows the classic panel again (in
game too); the old tabbed renderer page reached from a "Renderer" row at the end
of VIDEO is gone (10/10/2026). The groups are:

| Tab | Settings |
| --- | --- |
| IMAGE | HDR scene and exposure, eye adaptation (`r_autoExposure`) and its range in EV, filmic tone curve, bloom, dynamic glow (`r_DynamicGlow` 0-3) and its blur style (`r_dynamicGlowStyle`), FXAA, supersampling (`r_superSample`), soft particles, ambient occlusion (`r_ssao`), sunbeam dust (`r_dustMotes`), per-pixel model lighting, reflection probes (`r_cubeMapping`), floor mirrors (`r_floorReflections`), normal and specular maps (`r_normalMapping`, `r_specularMapping`), parallax mapping (`r_parallaxMapping`) and its depth (`r_parallaxStrength`), emission maps (`r_emissiveMaps`), their strength (`r_emissionStrength`) and glow halo (`r_emissiveGlow`) |
| LIGHTING | Sun and sky (`r_dayNight`), live lighting tier, time of day, day length, sunlight brightness, ambient fill and its corner shading, indirect boost, emission-map lights (`r_emissiveLights`), light shafts (`r_volumetrics`) and their clarity |
| SHADOWS | World and character sun shadows, shadow resolution, sharp and close cascade distances, filter taps, slit closing, contact shadows |
| WEATHER | Weather (`r_weather`), its density (`r_weatherDensity`), quality (`r_weatherQuality`), forced weather (`r_weatherForce`), ground fog (`r_weatherFog`) and volumetric clouds (`r_clouds`); see [Weather](rendering.md#weather) |

Rows marked "(reload)" are read only when the graphics start and apply after a
[graphics reload](#graphics-reload); "(next map)" applies when a map loads; the
rest apply immediately.
Changing a value saves it like any other setting. Switches over numeric cvars
show ON/OFF and write 1/0. Defaults are unchanged (see
[Default visual profile](rendering.md#default-visual-profile)). Diagnostics such
as `r_dayDebug` stay console-only, as do the speeds and key of
[eye adaptation](rendering.md#eye-adaptation) and the distance inside which parallax
stops growing near the camera (`r_parallaxNearDistance`, [Parallax](rendering.md#parallax));
the ground HUD stays on the HUD
tab, and exclusive fullscreen (`r_exclusiveFullscreen`) stays on VIDEO's
display-mode row. Eye adaptation holds still while this page is open, so
exposure changes made here show at once instead of being eased.
See [catalog.rs](../crates/sjk-viewer/src/settings/catalog.rs).

### Graphics quality

The first row of VIDEO, of [First setup](#first-setup) (under Graphics), of the SJK
UI's Graphics (Advanced, under Quality) and of Simple's Graphics (under Preset) is
the graphics preset ([graphics_quality.rs](../crates/sjk-viewer/src/graphics_quality.rs)),
a dropdown of four presets (Sol's request, 16/10/2026, FPS boost added 17/10/2026;
until then a Renderer row, Vanilla or SJK (alpha), and an SJK quality row of four
levels, from 12/10/2026):

- **FPS boost**: the most frames per second, for weak PCs with an integrated GPU:
  Vanilla style with the small effects it keeps also off (impact marks, player
  shadows, weather, Force Speed afterimages, footprints and dust, saber steam in
  rain). The other presets leave those as the player set them and do not count
  them; leaving FPS boost (for a preset, or `renderer sjk`) turns them back on.
  It counts as the Vanilla renderer (`renderer`, the low-FPS help). Resolution,
  supersampling and the FPS cap stay the player's.
- **Vanilla style**: the original game's look at the most frames per second, every
  SJK extra off (the level called EJK from 10/10/2026, Ultra low before, then
  Vanilla).
- **SJK (recommended)**: Sol's own setup (as Sol had it on 16/10/2026): SJK's image
  (HDR, bloom, dynamic glow, FXAA, ambient occlusion, per-pixel model lighting,
  emission maps and their halo) over the maps' baked lighting, without the sun,
  shadows, reflections, material maps, light shafts or weather extras. A new
  profile (no `config.cfg` when SJK starts, applied in
  [app_launch.rs](../crates/sjk-viewer/src/app_launch.rs)) starts on it, and
  Backspace on the row returns to it.
- **Experimental**: everything the SJK renderer draws (sun and sky, sun shadows,
  reflection probes, floor mirrors, normal, specular and parallax maps, light
  shafts, the richest weather), still in alpha. Its values are the renderer cvars'
  registered defaults, the [default visual profile](rendering.md#default-visual-profile)
  the renderer was tuned for (the former High).

While the settings are not Vanilla style's, the detail column of every renderer
setting carries the alpha warning in gold ([sjk-ui.md](sjk-ui.md#settings)).

The preset is not saved on its own: the row shows whichever preset the settings
match, and Custom once one of them has been changed on its own; stepping (Left,
Right) from Custom starts at the preset the settings are nearest, and a step stops
at either end. A renderer setting's row reads its value in the preset in use (or,
for Custom, the nearest) as its default (`graphics_quality::row_default`), so a
preset's rows do not carry the changed dot, a row changed on its own does, and its
Backspace leads back to the preset. The `graphicsquality` command names the preset,
or sets it with `graphicsquality fpsboost|vanilla|sjk|experimental` (a preset's name works
too; `ejk` and `ultralow` still name Vanilla style). Profiles on one of the former
levels read Experimental (High) or Custom (Performance, Balanced, Ultra).

| Setting | FPS boost | Vanilla style | SJK (recommended) | Experimental |
| --- | --- | --- | --- | --- |
| HDR scene, bloom, FXAA, per-pixel model lighting | off | off | on | on |
| Soft particles | off | off | off | on |
| Ambient occlusion (`r_ssao`) | off | off | on | on |
| Dynamic glow (`r_DynamicGlow`) | off | off | on | on |
| Sunbeam dust (`r_dustMotes`) | 0 | 0 | 0 | 1 |
| Reflection probes, floor mirrors | off | off | off | on |
| Normal, specular and parallax maps | off | off | off | on |
| Emission maps and their halo | off | off | on | on |
| Sun and sky (`r_dayNight`) | off | off | off | on |
| Emission-map lights (`r_emissiveLights`) | 0 | 0 | 0 | 1 |
| Light shafts (`r_volumetrics`) | 0 | 0 | 0 | 3 |
| World and character sun shadows | off | off | off | on |
| Shadow resolution | 1024 | 1024 | 1024 | 2048 |
| Shadow filter taps | 8 | 8 | 8 | 16 |
| Weather density | 1 | 1 | 1 | 2 |
| Weather quality | 0 | 0 | 0 | 2 |
| Volumetric clouds | off | off | off | on |
| Impact marks, player shadows (`cg_marks`, `cg_shadows`) | off | player's | player's | player's |
| Weather (`r_weather`) | off | player's | player's | player's |
| Force Speed afterimages, footprints, saber steam in rain | off | player's | player's | player's |

Under FPS boost, Vanilla style and SJK (recommended), with no sun, light shafts or sun shadows,
a map loads without building its lamps (see [rendering](rendering.md#default-visual-profile)), which no pass reads there.

The `renderer` command keeps the former Renderer row's switch: `renderer vanilla`
keeps the player's values of every setting in the table (`r_ultraLowRestore`,
archived, its name from before the renames) before applying Vanilla style;
`renderer sjk` puts them back, custom values included, or SJK (recommended) when
Vanilla style was picked as a preset and nothing was kept. The [low-FPS
help](#low-fps-help) offers the same switch. Like the preset, it needs a [graphics
reload](#graphics-reload) for the lighting to change. Everything else is left as the
player set it: the FPS cap, vertical sync, resolution and supersampling (which
multiplies the pixels drawn, so no preset sets it), the taste settings (exposure,
time of day, tone curve, glow style, parallax depth, emission strength), contact
shadows (still being worked out) and whether weather and fog show at all. Rows
marked "(reload)" apply after a graphics reload, so a new preset's shadows, light
shafts, HDR, FXAA, sun and sky, reflection probes and material maps show once the
graphics reload, and emission-map lights on the next map (or the reload, which loads
the map again); the rest change at once. A preset is saved with one write of the
profile. How many frames each preset gains on a given GPU is not yet measured.

### Graphics reload

Some renderer settings are read only when the graphics start: HDR scene, FXAA,
supersampling, sun and sky, sun shadows and their quality, light shafts, normal,
specular, parallax and emission maps, reflection probes and texture filtering,
so the graphics presets too. They apply with a
graphics reload rather than a restart of SJK
([graphics_reload.rs](../crates/sjk-viewer/src/graphics_reload.rs), [how it
works](rendering.md#graphics-reload)):

- `vid_restart` reloads the graphics when one of them changed, as it restarted
  the renderer in the original game (it still reapplies the display settings
  first).
- When one of them changed and the player is back on the main menu or the game
  menu (not in Settings, so changing several rows is not interrupted), a pop-up
  card in the SJK UI's look, in every menu style, says what changed ("Vanilla
  graphics apply after a graphics reload", or the settings by name) with **Reload now**
  (Enter) and **Later** (Escape). Later puts it away until another such setting
  changes; it waits for the medal and holocron pop-ups and the console.
- On a server (or in a local game) the map loads again under the loading screen,
  as on a map change to the same map, in a few seconds (ffa3 took 5.6 to 7.2 s
  off-screen, see [status](status.md)); the connection stays and the player stands still meanwhile,
  so the server does what it does when a client goes quiet for that long (a door
  the player held open may close until the player is back).
- On the main menu the menu map loads again behind the card ("Reloading the
  graphics...", about 3 s for duel6), and the menu carries on over the new world.
  After a match the menu map is built again quietly, with the graphics of the
  moment, when the menus come back to it.
- Not during a demo, a map being loaded or joined, or a map explored without a
  server: there the graphics reload with the next map.

The console notices say so: `HDR setting changed: reload the graphics to apply
(vid_restart)`.

### Low FPS help

SJK's real-time lighting (lamps, sun, sky light, shadows, light shafts) is most of
what a heavy map costs: on `JoFTemple` at 4K it was 5.6 of the frame's 8.1 ms before
its floor fix, and turning its parts off one at a time barely changed it
([status](status.md#joftemple-frame-rate-and-ultra-low)). Vanilla graphics (EJK
graphics until 12/10/2026) turn it off as
a whole. The low-FPS help
([fps_help.rs](../crates/sjk-viewer/src/fps_help.rs)) offers them when it matters:

- Six seconds after a server's map is in (loading's last work is over by then), the
  GPU times of eight sampled frames, one every 32 frames
  ([gpu_phases.rs](../crates/sjk-viewer/src/gpu_phases.rs), timestamps when the GPU
  has them), give the median frame and the part of it spent in the `light-...`
  sections. The check is done once per map (each new world), not under Vanilla,
  and gives up 30 s after the map came in.
- It offers Vanilla graphics when the frame misses the player's FPS target and the
  lighting is at least 40% of it. The target is the FPS cap (`com_maxfps`, or the
  monitor's rate for auto), kept between 60 and 144: an uncapped player is offered
  them below 144 FPS of GPU time, not because the frame could be faster still. The
  log says what it found: `fps help: frame 8.07 ms, SJK lighting 5.64 ms, target 144
  FPS: Vanilla graphics offered` (or `fine`).
- The offer is an SJK pop-up card over the game menu, which it opens and closes
  again with it ([sjk-ui.md](sjk-ui.md#low-fps-help)): this map's numbers, what
  Vanilla graphics turn off, **Use Vanilla graphics** (Enter: the renderer choice,
  then `vid_restart`;
  the map loads again and the player stays on the server) or **Not now** (Escape),
  and a **Don't suggest this again** tick (D, `cg_fpsHelp 0`).
- `help_fps` opens the same card at any time, on the menus or in a match, with the
  numbers once the frames it times while open come in. With Vanilla graphics on, its
  button turns them off and the player's settings come back.

The 40% share and the 60 to 144 target were chosen from JoFTemple's numbers before
and after its fix (offered at 8.1 ms with 5.6 of lighting, fine at 3.6 ms); they are
not yet checked on other GPUs or maps.

## First setup

The tabbed settings screen's last tab, FIRST SETUP (called Quick setup before
06/10/2026)
([quick.rs](../crates/sjk-viewer/src/settings/quick.rs)), gathers the settings worth
choosing on a first start: first, under a "From another client" heading, Import from
a client ([Importing from another client](#importing-from-another-client)) and the
Gameplay FPS ([Gameplay FPS](#gameplay-fps)), which players coming from another
client ask about most (Sol, 11/10/2026); under a Graphics heading, the graphics
preset ([Graphics quality](#graphics-quality)); under a Styles heading, the menu style
(SJK or Classic) and the camera style (EJK or SJK, [Camera
style](#camera-style));
then resolution, display mode, vsync, field of view, mouse
sensitivity and inversion, always run, effects and music volume, the HUD look and
scale, the crosshair, the nameplates and their bars, Force bar and power icons, the
Force aura and the combined Protect+Absorb shell, and the update and identity
opt-ins, then its own "Don't show at start" row (`ui_hideFirstSetup`), ending in a
Key bindings row. Its other rows are the catalogue's own, looked up by cvar, so a
change there is the same change the other tabs make; it is the last tab so the
other tabs keep their numbers. The SJK UI pop-up's line under its title points at
the first row.

With the classic menus the same rows are the first group of the
Setup page, FIRST SETUP (`Group::Quick`, [groups.rs](../crates/sjk-viewer/src/settings/groups.rs)),
drawn as a classic+ option panel like the others, with search, descriptions and
defaults, and opens the FIRST SETUP tab only if that panel cannot open. The SJK UI shows
them as a pop-up card over the map (since 08/10/2026,
[sjk_popup.rs](../crates/sjk-viewer/src/settings/sjk_popup.rs),
[sjk-ui.md](sjk-ui.md#first-setup)): the rows scroll inside it, and "Don't show
at start" is a tick box pinned at its foot, always in view, beside All settings
(the Settings screen on its First setup category, which keeps the rows too) and
Done. Picking another menu style on its Menu style row keeps First setup on show in
the new style: the classic Setup page's FIRST SETUP group or the SJK UI's
pop-up.

At every start, the first time the main menu is up and the menu style is known, it
opens in the active style, until the player ticks "Don't show at start"
(`ui_hideFirstSetup` 1, archived); Escape leaves it for that run. (Before
07/10/2026 it opened once, on the first start, marked by `ui_quickSetup`, which is
no longer read.) The `firstsetup` console command opens it too, over the main
menu or from a running game, in the active style
([first_setup.rs](../crates/sjk-viewer/src/menu/first_setup.rs)). Its old name
`quicksetup` was removed on 08/10/2026, with no hidden alias, so `quit` is the only
command a `q` completes to. Not yet run in a game window.

### Importing from another client

First setup's first row, Import from a client (its `firstsetup import` row, in
every menu style), opens the Import page on the configs the clients installed beside
SJK wrote, newest first, at most four: retail Jedi Academy and JA+
(`jampconfig.cfg`), OpenJK (`openjk.cfg`), EternalJK and JoF EJK (`eternaljk.cfg`)
and TaystJK (`taystjk.cfg`), in any folder of GameData (portable installs) or of
`Documents/My Games/OpenJK`, `EternalJK` or `TaystJK`, SJK's own folder left out
([config_import.rs](../crates/sjk-viewer/src/config_import.rs) `detect`). Each row
names the client (and the mod folder when it is not the client's own), the player
name the config holds and the day it was saved; Enter or a click reads it. Configs
holding nothing to import are not offered. With none found, the row opens the
system's file dialog as before.

A `.cfg` file dropped on the window (winit's `DroppedFile`), at any time, opens the
Import page over whatever is on show (a picture file, PNG, JPEG or TGA, opens the
Profile page's picture panel instead, as does any other file dropped while that page
shows: [Identity](#identity))
([config_import.rs](../crates/sjk-viewer/src/config_import.rs),
[config_import_panel.rs](../crates/sjk-viewer/src/config_import_panel.rs)). The
page's Browse (B, or its footer button) opens the system's file dialog, listing
`.cfg` files ([file_dialog.rs](../crates/sjk-viewer/src/file_dialog.rs), on a worker
thread, as the profile picture's Browse), and the file chosen is read as a dropped one.
On Linux it asks `zenity`, then `kdialog`, before `rfd`'s desktop portal, which never
answers on desktops whose portal has no file chooser; while the dialog is open the
button reads Stop waiting and gives up on it, so a dialog that never showed no longer
leaves Browse greyed out. The screen's text says so. `firstsetup import <path>` does
the same from the console (a path's words may be left unquoted, or quoted as
Explorer's Copy as path gives them: the path is taken as typed, without the
console's backslash escapes, and one pair of surrounding quotes is dropped), and
`firstsetup import` alone opens the page without a file. The file is read, never run: only `seta`/`set`/`sets`/`setu`, a
bare `name`, `model` or `cg_fov`, `bind`, `unbind` and `unbindall` count (of the
cvars, `name`, `model`, the `char_color_*` tint, `cg_fov`, `com_maxfps` and JoF EJK's
`cl_cmdratecap`), `//`
comments are skipped, and a file that is not UTF-8 is read as Latin-1, as the
legacy game wrote names. Files over 1 MB, other extensions, and configs holding
none of the parts are refused with the reason.

The page lists the parts found, each ticked at first: Name (`name`), Model
(`model`, with `char_color_red`/`_green`/`_blue` when the file sets them), Field
of view (`cg_fov`), FPS cap (`com_maxfps`, the frames drawn) and Key bindings.
Space, Left/Right or a click ticks a row; Enter copies the ticked parts into the
profile (also logged to the console), then asks for the gameplay FPS with what the
file's frame rate meant for its movement picked ([Gameplay FPS](#gameplay-fps)):
Auto, which follows the FPS cap just imported (333 moves like 333), or 125 when it
had `cl_cmdratecap 1`, as JoF EJK moved at 125 whatever it drew. Use sets it, Skip
(Escape) leaves it; either way the page ends on everything that changed. A file that clears every binding first (`unbindall`, as a saved
`jampconfig.cfg` does) replaces the player's table: the keys the binding editor
locks keep their binding, and SJK's own actions get their default key back when
the file left both the action and its key free. A file without `unbindall` only
adds its keys. Bindings on keys SJK does not know are left out and counted. The
page is the SJK UI's pop-up card in every menu style, as Update's
([config_import_panel_sjk.rs](../crates/sjk-viewer/src/config_import_panel_sjk.rs),
[SJK UI](sjk-ui.md)). With no file yet or after a refused one, Enter browses.

### Gameplay FPS

In the original game the frame rate is also the movement: every rendered frame makes
one user command, and the movement code's rounding makes jumps, strafes and
acceleration differ at 125, 142, 250 or 333 FPS, so players lock `com_maxfps` at the
one they are used to. SJK makes commands on their own clock
([networking.md](networking.md#user-commands-and-move-packets)). The gameplay FPS
(`gameplay_fps`, archived) sets it: on Auto (0, the default) the clock follows the
FPS cap, so the movement is what the other clients give at that cap, and a server that
fixes the step (`pmove_fixed`) uses its own, and the servers known to cap commands
(JoF's, at 125 a second; the list in
[server_caps.rs](../crates/sjk-viewer/src/server_caps.rs)) keep their cap unless
Settings > Display > Ignore server caps (`gameplay_fps_nocap 1`) lifts it; a number from
20 to 1000 keeps that
movement whatever the FPS cap. Players kept asking Sol how the frame rate and the
movement relate (11/10/2026), so it is asked where they arrive: First setup's second
row, the step after an import, Settings > Display > Gameplay FPS beside the FPS cap
(Simple's Gameplay has it too)
(Left/Right step through Auto, 125, 142, 250 and 333, Enter opens the card), and the
`help_gameplay_fps` command ([gameplay_fps.rs](../crates/sjk-viewer/src/gameplay_fps.rs)).

The card ([config_import_panel_fps.rs](../crates/sjk-viewer/src/config_import_panel_fps.rs),
the Import page's step, an SJK UI pop-up in every menu style) explains it in three
lines, says what the imported config had and why that is picked, and offers Auto ("your
FPS cap" under it), the four presets (JoF EJK under 125, "your old one" or "in use"
under the matching one) and Other: typing digits (top row or keypad) fills Other with any value from 20
to 1000 and says what it moves like (400 moves in 2 ms steps, 500 a second, as stock
rounds `1000 / fps`). Left/Right choose, Enter uses, Escape cancels (Skip after an
import). Its notes: a `pmove_fixed` server moves everyone in its own step (named when
the current server does), and the frames drawn are the FPS cap, shown with its value.
When a server does not take the inputs at a gameplay FPS above 125 (snapshots keep
saying it ran none of our commands, or keep correcting the prediction;
[networking.md](networking.md#user-commands-and-move-packets)), the card opens by
itself, once per server and value, with a line in orange saying the inputs are not
going through at that rate and 125 picked.
`help_gameplay_fps` prints the explanation and the value in use to the console and
opens the card; `help_gameplay_fps <fps>` or `gameplay_fps <fps>` sets it (0 for Auto).
The settings row shows Auto with the cap it follows (`Auto (125)`), and the server's
step beside the value while a `pmove_fixed` server overrides it.

## Slider values

Every slider in Settings (including the classic Setup panels), the saber RGB
controls (including the second saber), and Camera control supports direct
numeric entry. Click its displayed value, select the row and press Enter, or
just type a number on the selected row: the draft starts with what was typed.
Enter applies it; Escape cancels. Left/Right, Home/End, Backspace and Delete
edit the draft. Space still steps a selected slider instead of opening entry.
Hovering does not move an edit to another setting. Text settings such as the
master server keep their edit on the row that opened it, and Enter writes that
setting.

Pressing anything else while a numeric draft is open applies the draft when it
is a valid number and discards an invalid one, and integer sliders round a typed
fraction (142.6 becomes 143).

Manual values respect the slider bounds but do not snap to its drag increment:
for example, the FPS cap accepts 142 and FOV accepts 97.5. A draft takes
digits, one decimal point (a comma counts as one), a minus sign only first and
only on sliders that go below zero (the FPS cap's `-1` is AUTO), and at most ten
characters; a held key does not repeat typed characters. Invalid or empty input
stays open with a red underline and does not change the setting. Dragging and
arrow adjustment outside editing retain their existing behavior. Values they
write are rounded to the step's decimals, so a 0.05-step slider stores 0.35
rather than 0.35000000000000003.

## Key bindings

Settings > Key bindings binds two keys per action: click a slot (or select it
and press Enter), then press a key or mouse button, or turn the mouse wheel to
bind `MWHEELUP` or `MWHEELDOWN` (a notch only scrolls the list when no slot is
waiting; trackpad deltas under half a notch do not bind). A key already bound
elsewhere moves to the new action. Escape, or a click with the left or right
mouse button, cancels a pending capture without binding anything; the click is
consumed, so it activates nothing under the pointer. Retail
(`Item_Bind_HandleKey`) bound any key pressed while waiting, so a stray click
could take `+attack` off `MOUSE1`.

`MOUSE1`, `MOUSE2` and `ESCAPE` are locked in the editor: they cannot be
captured, a slot holding one cannot be rebound or cleared there, and locked
keys are drawn muted. Reset defaults restores them. The console's `bind` and
`unbind` commands are unrestricted. See
[keybind_editor.rs](../crates/sjk-viewer/src/keybind_editor.rs).

The console can also bind an arbitrary pair of keys as one chord, using the
JoF EJK syntax `bind K+L kill`. Hold either key and press the other to run the
chord. The first key's ordinary bind has already run; the chord replaces the
ordinary bind of the second key. For a chord bound to a `+command`, releasing
that second key sends its matching `-command`. The order in the command does
not matter (`K+L` and `L+K` name the same chord); `unbind K+L`, `bind K+L`,
`bindlist`, `unbindall` and `config.cfg` handle it like any other bind. There
are at most 256 chords. A literal plus key can be the first key as `++K`.

The settings editor shows an existing chord as one occupied slot and can clear
or replace it, but captures one physical key at a time. Create a chord in the
console. This follows JoF EJK commit `b6623f51` (`codemp/client/cl_keys.cpp`),
including first-match table order when several held chords could fire.

## Development maps

Run `devmap mp/ffa3` in the client console to start and join an owned local
FFA server with cheats enabled, no bots and no match limits. Other installed
maps work too, including `devmap t2_rancor`; `maps/` and `.bsp` are optional.
The command appears in console completion/help. It uses the same `sjk-dedicated`
binary lookup as Create game (`JKA_DEDICATED` overrides the adjacent binary).

This starts a fresh game on loopback, without master-server advertising. Once
launched, it replaces the current connection; it never asks a remote server to
change maps or allow cheats. Missing map names are reported before leaving the
current game. Disconnecting, cancelling the join, or exiting stops the owned
server. Ordinary Create game launches still leave cheats disabled.

The native server implements `noclip`, `give`, `setviewpos`, and `t_use` for
development. Run `noclip` again to return to ordinary movement; spawning again
clears it. Normal servers still require their own cheat permission. `god` remains
unimplemented on the native server.

## Invisible brushes

`r_showBrushes` draws the brushes a map is built with but never shows (clip,
trigger, ladder, nodrop…) see-through over the world, the way Radiant shows them:
each face in its kind's colour with a framed tile carrying its name, 64 units to
a tile, projected on the face's main axis as Radiant's default projection is, and
each face outlined. **0** is off (the default), **1** shows the kinds that change
play, **2** every kind. Settings > Game options > Map tools > Invisible brushes sets
it in both menu styles. `r_showBrushesTypes` replaces the set with the kinds it
lists (spaces or commas between names), for instance `r_showBrushesTypes "clip
trigger"`. It works on any server: the client loads every brush for its own
collision already.

| Name | Shows | Set |
| --- | --- | --- |
| `clip` | Player and NPC clip (`textures/system/clip`) | 1 |
| `playerclip` | Player clip only (`blockplayer`) | 1 |
| `npcclip` | NPC clip only (`blocknpc`) | 1 |
| `shotclip` | Stops shots, not players (`shotonlyclip`) | 1 |
| `solid` | Solid with no drawn face: invisible walls (`physics_clip`, `nodraw_solid`) | 1 |
| `trigger`, `teleport`, `push`, `hurt` | `trigger_*` entities, by classname; other trigger-textured brushes are `trigger` | 1 |
| `ladder` | Ladders | 1 |
| `nodrop` | Death pits and other nodrop volumes | 1 |
| `slick` | Slick floors | 1 |
| `cushion` | No falling damage (`cushion`) | 1 |
| `door` | `func_door` and `func_door_rotating` brushes | 1 |
| `lift` | `func_plat` | 1 |
| `button` | `func_button` | 1 |
| `train` | `func_train` | 1 |
| `endless` | `func_rotating`, `func_pendulum` and `func_bobbing`, which never stop | 1 |
| `weather` | `outside` / `inside` weather volumes | 2 |
| `botclip` | Bot navigation hints (`do_not_enter`) | 2 |
| `caulk` | Caulk faces | 2 |
| `nodraw` | Every other face that draws nothing (hint, skip, areaportal, origin…) | 2 |
| `breakable` | `func_breakable` and `func_glass` | 2 |
| `travel` | The box a door, lift or button moves through, from `reach()` in [mover_occlusion.rs](../crates/sjk-viewer/src/mover_occlusion.rs) | 2 |

A face counts when its shader has `surfaceparm nodraw`; its kind comes from its
brush's contents and its own surface flags as q3map2 compiled them
([kinds.rs](../crates/sjk-viewer/src/brush_view/kinds.rs)). Caulk and an invisible
wall share their flags, so a shader whose name contains `caulk` is caulk. Fog, water,
lava, slime and terrain volumes are left out, as they show anyway. Retail does not
ship Radiant's `textures/system` editor images, so the tiles are SJK's own, written
with the bundled Rajdhani Bold.

The faces are rebuilt from each brush's planes
([brush_winding.rs](../crates/sjk-bsp/src/brush_winding.rs)) the first time the view
is switched on for a map, never before, and drawn in the overlay pass after the
world: tested against the scene depth without writing it, lifted 0.35 units off
their plane toward the camera so a clip brush flush with a wall still shows, from
both sides ([brush_view.rs](../crates/sjk-viewer/src/brush_view.rs),
[brush_view.wgsl](../crates/sjk-viewer/src/brush_view.wgsl)). Brushes of entities
of other entities are left out (the map draws them). Movers are drawn once, where the
map file puts them, not where they are now: a door open at that moment still shows
closed, and its `travel` box shows how far it goes. A brush model with an origin brush
(rotating doors, trains) is stored relative to its `origin` key, so its faces are shifted
back by it; a `func_train` shows where the map file puts it, not at its first path
corner. A brush behind a wall stays hidden behind it. The view is off on the menu map.

Checked 11/10/2026 with world shots (`world_shot_brushes.rs`, `SJK_BRUSH_MAPS`,
`SJK_BRUSH_MODE`) on `mp/ffa3` (clips, a trigger, solid) and `mp/ffa5` (its death pit
as `hurt`, clips); not yet in a game. `mp/ffa5` wraps much of its detail in
`physics_clip`, so `solid` shows white over those walls at 1: that is where players
collide.

## Map entities

`r_showEntities` marks the point entities a map file places, through walls, with
their class names (and target names; a light adds its intensity). **0** is off (the
default), **1** shows lights, spawns, origins, items and targets, **2** every kind.
`r_showEntitiesTypes` replaces the set with the kinds it lists (spaces or commas), for
instance `r_showEntitiesTypes "light spawn"`. Settings > Game options > Map entities
sets the mode. It works on any server: the entity lump is in the BSP the client loads.

| Name | Shows | Set |
| --- | --- | --- |
| `light` | `light` and `light_*` | 1 |
| `spawn` | `info_player_*`, `team_CTF_*`, `info_jedimaster_start` | 1 |
| `origin` | `info_null`, `info_notnull`, `target_position`, `target_location`, `path_corner`, `ref_tag` | 1 |
| `item` | `item_*`, `weapon_*`, `ammo_*` | 1 |
| `target` | every other `target_*` | 1 |
| `sound` | `target_speaker`, `ambient_*` | 2 |
| `effect` | `fx_*` | 2 |
| `misc` | `misc_*` (props, models) | 2 |
| `other` | any other entity with an origin and no brush model | 2 |

The kind comes from the class name prefix ([entity_view.rs](../crates/sjk-viewer/src/entity_view.rs));
brush entities (`*N` models) are `r_showBrushes`'. A marker is a coloured square on the
entity's origin; the 60 nearest within 1500 units carry their label, and at most 220
within 6000 units are drawn. The markers are drawn in the HUD layer, so they are not
depth tested and show through walls. The list is read from the entity lump the first
time the view is on for a map.

A world note picks a marker: the one the view ray passes within about 1.4 degrees of
beats the wall or model behind it. The note records the entity (number, class name,
target name, origin) and the outline is a box round its origin.

## Talk balloons

Opening chat, the console or a menu sends the stock talk button and disables
other movement input while that keyboard catcher is active. As in EternalJK, so
does a window you alt-tab away from or minimise, so others see you are away:
`cl_unfocusedChatbox` and `cl_minimizedChatbox` (archived, both 1) turn that off. Players carrying
the talk flag have a chatbubble over their heads; the connection-trouble icon
takes priority when the server marks a lost connection. These are upright frame
billboards using the existing [sprite orientation](rendering.md#billboard-icons).
Their texture opacity is preserved near walls even with soft particles enabled.
Your own bubble is visible in third person, not the first-person view. Mind-tricked
players, NPC talk flags and intermission do not show talk balloons. The balloons,
pickup icons and hook ropes have their own share of the effect particle pool, so a
scene full of effects no longer hides every balloon at once. Siege voice command
icons remain unimplemented. See
[player_sprites.rs](../crates/sjk-viewer/src/player_sprites.rs) and
[pmove_talk.rs](../crates/sjk-game-jka/src/pmove_talk.rs).

## Joining and changing maps

A join from the menu prepares the destination world while the connection runs:
map preparation and connection are independent, and the loading screen (the
classic one or the SJK UI's) shows how far both are. The world is adopted once it
is built from the joined session's own gamestate with the session in hand, so
the player never walks a preview; a matching verified world is reused when the
server session becomes ready, and joining does not build it twice. Escape or a
click on the loading screen cancels.

On a server map change, gameplay pauses and a loading notice is shown over the
previous view until the destination and a fresh active snapshot are ready. The
client then adopts the server's spawn state. Match-end intermission uses the
server's normal camera, scoreboard and ready-to-exit controls. Player bodies and
vehicles are hidden there, matching codemp; scripted non-vehicle NPCs remain visible. Local gameplay
continuation on the old map is suspended while that feature is developed further.
The viewer no longer prepares an in-process native game for every loaded world.

CPU map preparation and GPU resource installation remain on background workers,
using the existing GPU context. Archive checksum inventory reads ZIP directories
without decompressing every asset. Matching same-map restarts can still reuse
the prepared world, and the join adopts its already-built destination. The
waiting connection sends neutral commands during map loading and is kept separate
from the displayed old world, so new-map entities cannot appear in the wrong BSP.
Texture mip preparation and lamp extraction use bounded CPU workers; repeated
texture loads can reuse a bounded process-local mip cache. See
[load-time rendering preparation](rendering.md#load-time-texture-and-light-preparation)
for cache limits and unchanged output semantics. Server readiness still
contributes to the time before play begins.

First entry into the destination before a server player exists retains
movement-only exploration with frozen brush collision.

Server console output (`print`) goes exclusively to the console, including match
statistics, command replies and server announcements. It never enters chat history.
Each line of a print becomes its own console row, without the empty row a
server's closing newline would leave. Global, team and private chat retain their
conversation overlay, and are also kept in the console scrollback but not among
its notify lines, as stock cgame echoes chat with the `*` print prefix that
`CL_ConsolePrint` keeps out of the notify area. Center-print
gameplay notices keep their separate HUD presentation. As `CG_DrawCenterString`
does, a centre-print row longer than 50 characters wraps at its last space; retail
counts bytes of its one-byte code page, so the limit counts characters, and a name
with `×` or `é` (two bytes in UTF-8) in a duel challenge wraps like any other. The scoreboard continues
to use structured server scores rather than parsing printed statistics tables.

Chat remains connected to the real server during intermission, including
global/team/private composer messages and console/bound chat commands. Messages
received during background loading are retained; its loading notice hides the HUD. The scoreboard reserves a separate left column for messages and the
composer, temporarily overriding chat position/width while scores are visible.
Chat visibility/lifetime settings still apply. Typing captures gameplay input as
usual, and the ordinary chat layout returns after the scoreboard closes.

Escape opens the normal menu during exploration. Disconnect/cancel abandons the
pending connection and builds the main-menu world again (the last world stays under
the menus for the few seconds that takes; see
[rendering.md](rendering.md#gpu-memory)). Connection and asset
errors still show an error message. Downloads retain their existing policy and
limits; progress is available in the console. This does not eliminate disk,
shader, network or download latency, and direct command-line startup is separate
from the already-open menu's transition path.

Implementation: [resident worlds](../crates/sjk-viewer/src/resident_world.rs),
[early exploration](../crates/sjk-viewer/src/resident_walk.rs),
[world handoff](../crates/sjk-viewer/src/session_transition.rs) and
[destination preparation](../crates/sjk-viewer/src/portal.rs).

Losing window focus releases held gameplay controls and discards gameplay actions
already queued for that frame. Synthetic key events on refocus cannot re-press a
held modifier such as Alt. This allows a saber throw already sent to the server
to finish normally after Alt+Tab.

### Timed player peek

`/peek <client ID|name> [seconds]` temporarily follows a received player from
behind, with wall collision, as JoF EJKSol does. The default is 5 seconds,
maximum 60. Names ignore colours/case; an unambiguous fragment also works.
Quote names containing spaces, for example `/peek "Reborn Master" 5`.
Empty or colour-only queries are rejected. As in JoF, scope and crosshair
overlays still use the local player aim. Monitoring only received entities
respects server PVS, but may show surroundings outside your line of sight.
`/peek` alone uses the crosshair player; `/peek off` or a duration of 0 cancels.
Local flight (`fakenoclip` or `freecam`) ends a peek; stop flight before peeking.
The local body stays at its player position and the view gun is suppressed.
The camera cancels when the timer ends, the target slot/name changes, or the
map changes. Missing snapshot data pauses the view; it resumes if data returns
before expiry. Players outside the
received snapshot cannot be watched. Player input and server authority remain
unchanged. The command is bindable under Settings > Key bindings > Other.

### Chat in other alphabets

Chat, names and every other text show Russian, Japanese, Chinese, Korean and colour
emoji (and any character a bundled font has) instead of `?` (Sol's requests,
12/10/2026; drawn from the wide-glyph and emoji atlases,
[rendering.md](rendering.md#characters-beyond-the-code-page)). An emoji is typed as
any character (Windows' emoji panel, Win+.); skin tones, families and flags draw as
one picture, and the caret and Backspace take them whole.
Russian travels as EternalJK sends it, in Windows-1251, so SJK and EternalJK players
with `se_language russian` read each other; `cl_charset` (Settings > Chat > Other
clients' text: Auto, Western, Cyrillic) says how text from other clients is read
when the guess is wrong ([player text](networking.md#player-text)). Japanese,
Chinese, Korean and emoji go as UTF-8: SJK players read them, retail and EternalJK
players see other characters. The chat field stops where the server would cut the
line (149 bytes as sent: 149 Russian letters, about 49 Japanese, Chinese or Korean
characters). The hub's SJK channel and bios take them too since 12/10/2026 (Cyrillic
they already took; the letter rules they share,
[identity.md](identity.md#the-bios-rules)); every SJK player reads them.

### Chat player actions

U defaults to crosshair-target private chat (`messagemode3`): aim at a player,
press U, then compose a tell addressed to that player. Global chat remains on
Y (`messagemode`), and team chat on T (`messagemode2`). I opens the SJK chat
(`messagemode5`, [hub-chat.md](hub-chat.md)); a new profile, or a whole config table
imported at First setup, gets I when it is free and nothing is bound to
`messagemode5`; an existing profile binds it in Settings > Key bindings > Other.
Existing profiles gain
U when it is free and no custom crosshair-chat key exists. Version-2 profiles
from the earlier incorrect global-U default repair U to `messagemode3` and
restore Y global chat if needed and free; other occupied keys stay unchanged.
This migration advances `cl_bindDefaultsVersion` to 3. Use
`bind u messagemode3` to select U explicitly. No message is sent until Enter.

`amcolor <red> <green> <blue>` (EternalJK's `amColor`) sets the player tint, the archived
`char_color_red`, `_green` and `_blue` userinfo cvars, each 0 to 255, so the server gets
it at once and it is kept for the next connect; bare, it prints the current tint
([am_color.rs](../crates/sjk-viewer/src/am_color.rs)). Before, `amcolor` went to the
server as an unknown command and the tint was lost on reconnect.

`reply <message>` (JoF EternalJK's) whispers to the last player who whispered to
you; a bare `reply`, bindable as "Reply to last whisper" in Settings > Key bindings >
Other (unbound by default), opens the composer on that whisper instead. Tab in the
composer reaches the same whisper after SJK (All, Team, SJK, the last whisperer).
Your own whispers, which the server echoes back, do not count; when nobody has
whispered, or the player left or renamed, `reply` says so and sends nothing
([chat/reply.rs](../crates/sjk-viewer/src/chat/reply.rs)).

`clientlist` (also `showplayerid`, JoF EJK's `showPlayerId` spelling) lists each player
in the console: the client number, a team letter (F free, R red, B blue, S spectator),
the name and ` (bot)` after bots, then `Listed N clients`. Commands that take a client
number use it. `clientlistinfo` gives JoF EJK's human-only detailed view: model and
skin, gender when advertised, saber names and colours, team model group in team games,
and non-default body RGB when that player's current snapshot state is available
([player_ids.rs](../crates/sjk-viewer/src/player_ids.rs)).

### Chat substitutions

Game-chat messages expand JoF EJK's substitutions when sent, from the console,
binds, the chat composer in either menu style, team chat, tells and `reply`.
Write each token as a separate word; matching ignores case. For example,
`say_team "Health %H% shield %S% Force %F%"` sends the current values.

| Token | Value |
| --- | --- |
| `%H%` | Health |
| `%S%` | Shield/armor |
| `%F%` | Force reserve |
| `%W%` | Current weapon's EJK display name |
| `%A%` | Current weapon's ammo pool, including its negative sentinel |
| `%P%` | SJK lagometer's latest snapshot latency estimate in milliseconds |
| `%U%` | Horizontal speed in units per second, rounded to an integer |
| `%UKM%` | Horizontal speed using EJK's km/h conversion, one decimal |
| `%UM%` | Horizontal speed using EJK's mph conversion, one decimal |
| `%T%` | Local time, 12-hour clock with AM/PM |
| `%T2%` | Local time, 24-hour clock |

Unknown tokens, embedded tokens such as `health=%H%`, and ordinary percent signs
stay literal. Expansion precedes chat colouring, quoting and the outgoing byte
limit. It runs only for messages the player sends to the game server; received
messages and the separate SJK hub chat are unchanged. Player values follow the
viewed snapshot, with predicted health, Force, weapon, ammo and velocity when
available; model/map transitions do not borrow the previous player's prediction.
See [chat/substitutions.rs](../crates/sjk-viewer/src/chat/substitutions.rs).

`cg_chatRememberChannel` (default 1, Settings > HUD > Remember chat channel)
makes the chat key (`messagemode`) open the composer on the channel of the last
message sent from it: All, Team, SJK, or a whisper to a player still in the game
(All once they leave). It lasts while SJK runs. The team, crosshair, last-attacker
and SJK keys always open on their own channel; with 0 the chat key always opens on
All.

Open the chat composer with your chat binding, then click a
sender's name. The cursor is free while composing. The player menu offers:

- **whisper:** keeps the current draft and addresses the selected player using
  the stock `tell` command. Nothing is sent until Enter.
- **ignore:** hides that player's existing and incoming
  messages locally for the current map. Opening chat shows a hidden-message row
  whose name can be clicked to undo the ignore. It does not change server policy
  or suppress footsteps, saber effects, or other gameplay sounds.
- **friend:** saves a local name bookmark and adds a
  small five-point star to the left of that player's name. Bookmarks survive restarts in
  `chat-friends.txt`, beside `config.cfg`. Names ignore colour codes but otherwise
  match exactly; these are name bookmarks, not authenticated accounts.
- **copy:** copies the complete name, including its colour escapes.

Resting the pointer on a name (a sender's, or an SJK chat sender's) shows their
sender card (picture, name, hub, medals) beside it, with Mute or Unmute: unlike ignore, a mute is kept on this PC
(`chat-mutes.txt`) and, besides their chat, draws them as Kyle with the default saber
and silences every sound they cause ([hub-chat.md](hub-chat.md#muting-a-player)).

The dropdown opens without a highlighted action. Hover follows the pointer;
keyboard navigation highlights only its current row until the pointer moves.
Arrow keys/Tab navigate the player menu; Enter selects and Escape dismisses it
without discarding the draft. The compact square-edged dropdown sits to the left
of chat, aligned with the clicked name and kept above the composer. It has only
four labels, no title or description, and never moves the conversation. If the
left margin is too narrow, it uses the right edge inside the chat lane to avoid
the scoreboard. Highlighted `ignore`/`friend` rows indicate active toggles; clicking
again undoes them. Name hover fits the visible username glyph bounds, excluding the star. Dropdown
row highlights use exactly the same rectangle as their clickable button. There is no
standing player-options hint or success notice. Opening the composer exposes history even when
passive chat is hidden with `cg_chatbox 0`.

The draft shares the console's UTF-8 caret and selection rules: arrows/Home/End,
Ctrl+arrows and Ctrl+Backspace/Delete, Shift-selection, Ctrl+A/C/X/V,
Ctrl+Insert to copy and Shift+Insert to paste. Click places the caret, drag selects,
and double-click selects a token. Selected text is highlighted; typing/pasting
replaces it. Clipboard text keeps colour escapes, strips controls and obeys the
existing chat byte limit. Pasting never sends a message. Dead keys type as in
the console (see below): `^` then a digit is a colour code, `^` then `e` is `ê`.

Actions use server-provided sender slots and current roster generations. Old
messages cannot address a replacement after an observed departure/name change;
an invalid whisper recipient leaves the draft open. A line the server does not
attribute (JA+) is its sender's when it starts with exactly one connected player's
name as the game shows it, then `: ` (or `(name): `, `[name]: `), since 16/10/2026
([hub-chat.md](hub-chat.md#game-chat-lines-look-the-same)); other unattributed lines
stay unclickable. `reply` also reads an unattributed `[name]: ` whisper whose name
(colour codes aside) is exactly a connected player's, as JoF EternalJK does.
Legacy servers provide no authenticated account identity; unobserved same-name
slot reuse cannot be distinguished. No transport or protocol encoding changed.

The leader/opponent portrait and its name/score occupy the top-right corner,
with a 32-unit top margin and the existing 40-unit right margin at 1080p (scaled
with the HUD). Optional snapshot diagnostics and inventory flow below that block.
Visibility and server-selected leader/opponent rules are unchanged. There is no team
overlay ([Team outlines and logos](#team-outlines-and-logos)).

`cg_drawInventory` (archived, default **0**) controls the permanent list of
carried holdable items below that block, following JoF EJK #40's quieter default.
Enable **Inventory list** in classic Settings > HUD or SJK Settings > Advanced >
HUD to show it. The brief inventory selector still appears when cycling items,
whether this permanent list is on or off. Trip-mine weapons are not part of the
holdable list. Existing saved values remain in effect.

## Lagometer size

`cg_lagometerScale` changes the lagometer's width and height: `1` (default) is
normal size, `0.5` half size, `2` double size. Settings > HUD > **Lagometer scale**
offers 0.25–4 in both menu interfaces; values outside that range are clamped when
drawn. The setting is archived and takes effect immediately. Enable the graph with
`cg_lagometer 1`.

Scaling keeps the graph's existing bottom-right corner fixed, including custom
`cg_lagometerX` and `cg_lagometerY` offsets. It applies to every built-in HUD and
custom layouts that use the `lagometer` binding, on top of their layout dimensions.
It preserves the number and order of visible samples and the graph's millisecond
ranges, so a larger graph does not show a longer history.

## Strafe helper

`cg_strafeHelper` draws guides for strafe-jumping with JKA's movement physics, as
JAPRO-derived clients do; its options are bits, listed and toggled by the
`strafehelper` command (`strafehelper 1` switches the lines style on). Two styles
are drawn, one at a time (the command switches between them as JoF EJK's does):

- option 1, the lines (JoF EJK's "updated style", `DrawStrafeLine` and
  `CG_StrafeHelper` in JoF's `cg_draw.c`): one line per enabled key, from the bottom
  centre of the screen up to just above the crosshair (virtual height 230), aimed at
  where the best heading for that key projects from the camera at 256 units
  (`cg_strafeHelperPrecision`). Keys: W (option 5, two lines), WA (6), WD (7), A (8),
  D (9) and the rear A/D lines (10); the centre line (11) is drawn only for other
  physics, so not here. The line of the keys held takes `cg_strafeHelperActiveColor`;
  the others keep JoF's tints at `cg_strafeHelperInactiveAlpha`. They show in first
  and third person, airborne and on the ground (JKA's ground acceleration 10 and
  friction 6), once the horizontal speed reaches the player's speed;
  `cg_strafeHelperCutoff` hides that many virtual pixels from the bottom (at most up
  to 20 below the line top), `cg_strafeHelperLineWidth` sets the thickness;
- option 2, CGAZ markers (TaystJK's `hud_strafehelper.c`): short ticks across the
  crosshair at the same headings, airborne only.

Settings > HUD+ > "Strafe lines" switches option 1 (a derived switch: on replaces
the other style and adds WA and WD when no key is picked, off clears it). The
default value, 3008, picks keys but no style, so nothing is drawn; JoF EJK's
profile value `2242` (lines, WA, WD, centre) draws the same two lines here. Nothing
is drawn on JAPRO servers (their serverinfo names `japro`), whose movement styles
change the physics, while in a vehicle, zoomed or in a special movement type. The
helper's sound, accel bar, Weze, Warsow and original styles are not drawn.

## Followed player's name

While following (spectating) someone, and throughout demo playback, the HUD names
the player whose view it shows in the top left, as JoF EternalJK's `CG_DrawFollow`
does (stock Jedi Academy centred "following" and the name at the top): the name with
its colour codes, white otherwise, its line top at (4, 27) of the 640x480 screen and
`FONT_MEDIUM` at 0.85 (SJK Menu with "Classic game fonts", the HUD font otherwise), placed
in the HUD's 1080-line frame like the other HUD text
([follow.rs](../crates/sjk-viewer/src/hud/follow.rs)). While it shows, the warmup
line ("WAITING FOR PLAYERS", "STARTS IN") is not drawn, as `CG_Draw2D` skips
`CG_DrawWarmup` then. It follows `cg_drawHud` and `cg_draw2D` and steps aside with
the rest of the HUD while the scoreboard is held (hub report 69).

## Scoreboard styles

`cg_scoreboardStyle` (Settings > HUD > Scoreboard > "Scoreboard style") picks the
scoreboard layout. While either scoreboard is held, the HUD steps
aside as for the quick wheel (status, weapon, timers, vote and kill lines,
crosshair, lagometer, speedometer, warmup line, Force and item
selectors, movement guides, the red damage-direction flash, the ground readout
and the game-data HUD; chat stays, `scoreboard::hides_hud`), unless
`cg_drawScores 0` leaves the key drawing nothing:

- `auto`, the default of a new profile: the SJK UI's scoreboard while
  `ui_menuStyle` is `sjk`, the classic one with the classic menus (what SJK
  showed before the choice existed);
- `sjk`, the SJK UI's scoreboard whatever the menus
  ([SJK UI](sjk-ui.md#scoreboard)): columns floating over the darkened game in
  the UI's type, the teams side by side, the duelists as facing cards. Its
  compact rows (`cg_compactScoreboard`, on by default) keep a full server in
  one column, in a board as wide as its names, centred clear of the chat; its
  header shows the frame rate left of the clock (`cg_scoreboardFps`, on by
  default; Settings > HUD > Scoreboard > "Scoreboard frame rate");
- `classic`, the retail scoreboard as EternalJK-derived clients such as JoF EJK
  draw it (below).

Without the key the board opens only at intermission (`PM_INTERMISSION`, 7): not on
death (SJK's choice), and not while a JA+ emote holds the player in `PM_FREEZE` (6),
which once read as intermission and kept the board up for the emote's whole length
(`scoreboard/behavior.rs`).

A mistyped value gives the classic board. `config.cfg` saves every archived
setting, so every profile from before `auto` existed had the old default
`classic` saved: the console's start moves a saved `classic` once to `auto`
(`cg_scoreboardStyleDefaultVersion`), which draws the same classic board unless
the menus are the SJK UI. A look chosen after that stays. The classic board
([classic.rs](../crates/sjk-viewer/src/scoreboard/classic.rs), after
`CG_DrawOldScoreboard`/`CG_DrawClientScore` in `cg_scoreboard.c`):

- The header shows "Killed by" while you are dead, otherwise the player count
  (`cg_drawScoreboardPlayerCount`: 1 host name and counts, 2 counts only, 0 off;
  team games show "N vs. M", your team first), and below it your place ("2nd
  place (of 7) with 18", place in its retail colour) or the team lead.
- Columns are Name, Score, Ping, Time and, with `cg_showClientIDs` (on by
  default), the client ID; CTF shows Score, C, A, D, Ping and Time, and duels with
  a frag limit show wins/losses. Score shows score/deaths when `cg_scoreDeaths`
  provides deaths. Bots show `BOT` for their ping; clients still connecting show
  `-`, and connected clients the scores do not list yet show `N/A`.
- Team games list the leading team first over a translucent team band, then the
  spectators; free-for-all lists the players, then the spectators. Your row is
  highlighted in your rank's colour (1st blue, 2nd red, 3rd yellow, else grey) and
  is added at the bottom if the list does not reach it. Flag carriers show the
  flag icon before their row; at intermission, ready players are marked `READY`.
- Rows are 25 units of the 480-line screen, 15 units once more than 12 scores
  arrive (always with `cg_smallScoreboard` and in CTF), and 12 units in that small
  format above 20 connected clients. The connected roster controls the raised
  header independently of received scores. Normal small lists have a 19-row
  budget; crowded lists have 32. Section spacing, empty sections, losing-team
  reservations and the appended local row follow the pinned JoF reference,
  rather than an additional screen-bottom clip. `cg_drawScoreboardIcons`
  (on by default) shows
  each player's head icon (`models/players/<model>/icon_<skin>`), decoded when a
  model changes, two per frame at most.
- Positions follow the 640x480 screen fitted to the window height, spreading up
  to 1.25x horizontally on wide windows while text keeps its proportions. Names
  and headings are sized like retail `ergoec` text and numbers like `ocr_a`.

The classic board's interaction follows
[JoF EJK revision bd5e202d](https://github.com/JediofFreedom/JoF_EJK/blob/bd5e202dfedd49500f1b7aadd655619bac152167/codemp/cgame/cg_scoreboard.c)
(`CG_DrawOldScoreboard`) and its
[chat/HUD rules](https://github.com/JediofFreedom/JoF_EJK/blob/bd5e202dfedd49500f1b7aadd655619bac152167/codemp/cgame/cg_draw.c).
It opens instantly when held, on death and at intermission. Releasing the key or
respawning retains the board for a 200 ms fade if neither death nor intermission
keeps it open. Warmup suppresses the board unless the score key is held, matching
JoF's warmup gate. Menus, console, pause and
`cg_drawScores 0` suppress it.
Rows change places instantly. The SJK style keeps its animated opening and rows.

Classic moves chat to virtual x=8, bottom y=475 on the 640x480 screen, retaining
the configured font, width, lines and message expiry. Above 25 connected players,
holding scores hides the chat feed; release restores it, even when the automatic
death or intermission board remains open. That automatic crowded board limits the
list to 24 rows and adds a **HOLD TAB TO VIEW FULL SCOREBOARD** hint. Its local-row
append follows the reference too: a missed playing client is appended except
on the blue team. Holding the key shows the full list. Center prints stay
hidden through the scoreboard fade. SJK's chat composer stays inside the window
and can still be used independently of the feed.

Classic follows JoF's selective HUD visibility: holding scores or dying hides
the playing client's status, weapon/Force/inventory selection and crosshair;
votes, telemetry and independent power indicators remain visible. Spectators
retain their crosshair while holding scores. Intermission hides the gameplay
HUD. The SJK scoreboard keeps its whole-HUD suppression while held and its
held/intermission-only opening policy. Team rows reserve space for the losing
side, with connecting
clients listed after scored clients. An intermission duel winner takes precedence
over the next duel's participants and the killer/player-count header.

[Score requests](https://github.com/JediofFreedom/JoF_EJK/blob/bd5e202dfedd49500f1b7aadd655619bac152167/codemp/cgame/cg_consolecmds.c)
share a two-second gate with intermission refreshes; an explicit stale classic
request clears cached rows until the server responds. Demo key presses neither
request nor clear scores. The roster cache also tracks connected clients absent
from a score response, so joins, departures and team changes update the layout.

SJK retains its ping colours, faint alternating stripes and identity marks, and
its existing meaning of `cg_drawScores` as the bound-board enable switch (JoF
uses it for the small HUD scores). Server-plugin deaths without a verified plugin
handshake, Siege class/winner presentation and recorded demo score rows remain
existing compatibility gaps; this change does not synthesize those server facts.
The retained text and draw storage is reserved for 32 clients.

## HUD style

`cg_hudStyle` chooses `game`, the status HUD of the game's own menu files (the
original Jedi Academy HUD, or a custom HUD pack that replaces `ui/hud.menu`),
or SJK's own `classic` or `radial` layout. The default is `game`, the classic
HUD, and a saved `cg_hudStyle` is kept (but the retired `modern`, see
[Menu style](#menu-style)). `cg_hudFiles`
names the menu list, `ui/jahud.txt` by default; `1` gives the text-only HUD and
EternalJK's `3`/`4` name its elegance and JoF HUD lists when those files are
installed. `cg_hudPack` names the PK3 whose HUD to use when several replace
`ui/hud.menu` (`assets1.pk3` is the original); empty, the one installed last
wins, as in the game. See [game-data HUD](rendering.md#game-data-hud) for what
is drawn.

The upper right stacks the readouts as JoF EJK's `CG_DrawUpperRight` does, in every
HUD style ([console_overlay.rs](../crates/sjk-viewer/src/console_overlay.rs)): with
`cg_drawFPS` the frame rate ("125fps", JoF EJK's big console characters, measured over
half a second) and, smaller under it, the frame times ("host 2.44 ms · p99 3.83 · max
4.87 · 1% low 115"; Sol's request, 11/10/2026: the long single line, tidier); then with
`cg_drawTimer` with `cg_drawTimerUpperRight` (off by default) the match clock ("0:04.419"
with `cg_drawTimerMsec`, counting down with `cg_drawTimerCountdown`), right-aligned 5 of
480 from the edge, under `cg_drawUpperRight`. Without that option the layouts keep their own
clock at the top centre (Sol's choice, 12/10/2026); with it that clock steps aside. The
game-data HUD draws no clock of its own, so `cg_drawTimer` showed nothing there until the
option existed (Creyon's report, 11/10/2026). The kill feed stands under the stack.

In SJK's own layouts, health and armour over the maximum (125 health at spawn, up
to 199 armour from a large shield picked up at 99) show on their meter: the meter
holds one maximum, and what lies over it is a thinner band down the middle in a
deeper shade of the meter's colour, from the start, as on the nameplates. The
game-data HUD draws the original pictures and does not.

Settings > HUD > "HUD look" shows the HUD in use; Left and Right step through
every HUD that can be used and Enter (or a click) opens the HUD picker
([hud_picker.rs](../crates/sjk-viewer/src/settings/hud_picker.rs)): the list on
the left, a picture of the highlighted HUD on the right, drawn for a sample
player over a retail levelshot, and its PK3 or list file under it, the HUD in
use marked. The list holds each installed PK3 that ships `ui/hud.menu` in
install order (the game's own archives as "Jedi Academy", others by file name
without a `zz_` load-order prefix), every other `ui/*hud*.txt` list that
describes a HUD, the text-only HUD, and SJK's radial and classic layouts, which
have a note instead of a picture. Arrows, the wheel or the pointer highlight;
Enter or a click uses the HUD and closes; Escape closes. The picker takes
retail's colours and highlight art on the classic menus and the theme's with
the SJK UI. "Game HUD files" stays on the tab for lists the picker does not
find.

## Capture the flag HUD

In Capture the Flag and Capture the Ysalamiri every HUD style shows the flags' state
([flag_status.rs](../crates/sjk-viewer/src/hud/flag_status.rs)), reading what
the original cgame reads:

- Flag icons at the left (`cg_drawFlagStatus`, on by default, Settings > HUD), as
  `CG_DrawFlagStatus` places them (32 units at x 2, y 328 and one icon up): the enemy flag
  (`gfx/hud/mpi_bflag` or `mpi_rflag`, the `_ys` pictures in CTY) while your team holds it,
  taken or dropped, and your own flag's `mpi_*flag_x` picture while the enemy holds it. The
  state is `CS_FLAGSTATUS` (config string 23: red then blue, `0` at base, `1` taken, `2`
  dropped); the caption beside each says TAKEN or DROPPED.
- While the local player carries a flag (`PW_REDFLAG`, `PW_BLUEFLAG` or `PW_NEUTRALFLAG`
  in the player state, whatever the status string says) the enemy flag's icon stands on a
  gold plate with "YOU HAVE THE FLAG". The upper-right powerup icon of `CG_DrawPowerupIcons`
  still follows `cg_drawPowerupIcons` and `cg_drawUpperRight`; it now uses the ysalamiri
  pictures in CTY only (it took CTF for CTY).
- The team scores in every team game but Siege (`cg_drawTeamScores`, on by default): "RED 3"
  and "BLUE 5" plates under the clock at the top centre, from `CS_SCORES1` and `CS_SCORES2`
  (`-` while the server has not sent one). With them on, the layouts' score line drops its
  trailing team scores (`cg_drawScore 2`), which they replace.

The scoreboards (classic and SJK UI) draw a flag icon before a carrier's name from the
`scores` command's powerups; they also use the powerups of a player in view in the
snapshot (and the local player state), for servers whose `scores` list none.

## Kill feed

`cg_killfeed` (on by default, Settings > HUD > "Kill feed") lists the last kills
at the top right of the HUD, in every HUD style
([kill_feed.rs](../crates/sjk-viewer/src/hud/kill_feed.rs)). Each entry is
`killer [icon] victim`, or `[skull] victim` for a suicide or a death to the world
(a fall, lava, a trigger), on a dark plate; an entry where the viewed player
killed or died has a stronger plate and a white outline. Names are read from the
players' info when the kill arrives, as the console's kill message is, and keep
their colour codes; they are not tinted by team, as the console, chat and
scoreboard draw them. A name too long for its room ends in an ellipsis.

The icon is the HUD's picture for the means of death: the weapon's own
`gfx/hud/w_icon_*` (the saber's `w_icon_lightsaber`), mapped from each `MOD_*` to
the weapon whose shots carry it in OpenJK's `g_weapon.c` (the sentry gun has its
item picture), or an installed icon pack's `hud/mod/*` picture, which takes its
place. A dark Force kill (`MOD_FORCE_DARK`, lightning or grip) without such a
picture shows Force Grip's holocron when only the killer's grip was active in the
snapshot of the kill (`ObituaryEvent::attacker_force`), else Force Lightning's, and a player
knocked to their death (`KILLED_FORCETOSS`) Force Push's. Causes no weapon deals
(water, lava, crushing, telefrags, unknown causes) show the skull, which is drawn
from shapes, not a game picture. A weapon whose picture did not load shows a
short word instead (`SABER`, `ROCKET`, `FORCE`).

Up to five entries show, the newest at the top so it is always right under the
readouts above; each holds five seconds of server time and fades out over one.
The feed stands under whatever the HUD already
draws at the top right — the FPS readout, duel portrait, snapshot,
inventory and powerup column — and never reaches left of the vote panel's right
edge, so it does not cover the top centre's vote and timer. It is part of the
HUD's draw list, so it hides whenever the HUD does: scoreboard held,
`cg_drawHud 0` or `cg_draw2D 0`, intermission, menus and the console. Names draw
in the HUD's font (Inter, or SJK HUD with `cg_classicHudFont`), which the feed
measures them in.

| Cvar | Default | Effect |
| --- | --- | --- |
| `cg_killfeed` | 1 | Show the feed |
| `cg_killfeedX`, `cg_killfeedY` | 0 | Move it left and down, in 640x480 units |
| `cg_killfeedTextSize` | 0.8 | Name size; 0.8 is the normal size, 0 uses it |
| `cg_killfeedIconSize` | 12 | Icon size in 640x480 units; 0 uses 18 |
| `cg_killfeedColors` | 0 | Tint icons by cause of death (TaystJK's colours) |

The feed replaces an earlier one-line obituary at the same cvar, which was off by
default; a profile that saved that 0 has it moved once to 1
(`cg_killfeedDefaultVersion`), and that feed's `cg_killfeedAlignment` and
`cg_killfeedReverse` are gone.

## Nameplates

`cg_nameplate` (on by default) draws MMO-style nameplates over players
([nameplate.rs](../crates/sjk-viewer/src/hud/nameplate.rs); layout in
[nameplate_math.rs](../crates/sjk-viewer/src/hud/nameplate_math.rs)): 2D HUD shapes
and text at the projected point 8 units above the head of the player's box (decoded
from `entityState_t::solid`, so a crouching player's plate drops). Names come from the
chat roster with their colour codes and are drawn in the classic HUD font (Inter when
that font is not loaded), whatever `cg_classicHudFont` says.

- **Far:** only the name, small and dim. Plates shrink with distance (to 60% at the
  range) and fade over the last quarter of `cg_nameplateRange` (3000 units).
- **Near:** inside `cg_nameplateNear` (1000 units) the name rises and a plate fades
  in under it, framed in red or blue in team games: the shield bar on top, then
  health, then Force. The bars take the HUD's own colours (the layout's
  `armor_ratio`, `health_ratio` and `force_ratio` widgets: the Radial HUD's green
  shield, red health and blue Force; the retail green, red and light blue for the
  game-data HUD, which draws pictures). Health is always that red, whatever is left.
- **Empty shield:** a player's shield bar is always there; known to be empty it is
  drawn broken, grey dashes on the dark track.
- **Overheal and overshield:** health and shield go to twice the maximum (125 at
  spawn; 199 shield from a large shield picked up at 99). The bar shows 0 to the
  maximum; what lies over it is a second, inner band in a deeper shade of the same
  colour, from the left (125 health: a full red bar with a deep red first quarter).
  Both decay a point a second back to the maximum, as the server does.
- **Unsure:** a health or shield estimate whose range spans 60 or more (a player
  first seen, or back from long out of view) dims and gets a yellow "?" over it until
  a pain, a shield hit or another clue narrows it.
- **Uncertainty:** an estimated bar is a range ([estimate.rs](../crates/sjk-viewer/src/hud/estimate.rs)):
  the lowest and highest the value can be and the best guess. The bar fills to the
  guess, and a grey haze, thickest at the guess and fading out towards either bound,
  covers the uncertain stretch, so a loose estimate reads as a blurred edge and an
  exact one as a sharp edge.
- `cg_nameplateWeapon` (on): the held weapon's icon (the weapon selection row's art,
  the staff and dual saber icons for those styles) left of the plate, or left of
  the name when there is no plate. A saber's is ringed and haloed in its stance's
  colour (`fireflag`, which carries `fd.saberAnimLevel`; the Radial HUD's colours:
  fast blue, medium yellow, strong red, Desann violet, Tavion cyan, dual green,
  staff magenta), dimmed while
  the blade is put away.
- **Verified badge:** a player the SJK hub's operator vouches for (see
  [Identity](#identity)) gets a gold seal with a white tick after the name, drawn at
  start into an icon cell ([verified_badge.rs](../crates/sjk-viewer/src/ui_renderer/verified_badge.rs)),
  centred on the middle of the name's capitals as the plate's font lays them out
  (`UiFont::capital_middle`; the HUD font hangs its glyphs low in the line).
  Which slots are verified is read from the identity service once a second.
- **Duel badge:** a player in a private duel (`bolt1`, your own `duelInProgress`
  on your own plate) gets two crossed lightsabers, white blades in a coloured glow on a
  disc filled and ringed in the same colour (so the colours tell apart at a nameplate's
  size), before the name. **The two duellists of one duel
  share a colour** (blue, orange, green, magenta, yellow, cyan, red, in the order duels
  start), so in a crowd the matching badges show who duels whom. The pairs come
  from [duel_pairs.rs](../crates/sjk-viewer/src/hud/duel_pairs.rs):
  - the server's broadcast when a duel starts, `"<challenged> @@@PLDUELACCEPT
    <challenger>!"` to every client (`Cmd_EngageDuel_f`, `g_cmds.c`; jaPRO and TaystJK,
    as JoF runs, add `^7` after each name and the duel type after the `!`, such as
    `(Saber) [Ranked]`), its names matched
    against `CS_PLAYERS` (decoded as the print is, exactly, else without colour codes; a
    name two players share pairs nobody);
  - your own duel, from your player state.

  Nothing is guessed: a duellist whose opponent SJK was never told (a duel that began
  before you joined, or a mod that words the message otherwise) wears a neutral grey
  badge rather than a colour that might pair the wrong two.

  A pair keeps its colour until the snapshot shows one of the two no longer duelling (a
  pair named while out of view waits for them; one named but shown not duelling lapses
  after 3 s), and every pair is forgotten with a new gamestate (map or server) or a demo
  seek. Each colour is its own icon
  cell, drawn at start ([duel_badge.rs](../crates/sjk-viewer/src/ui_renderer/duel_badge.rs)).
  The badge sits level with the name's capitals like the verified badge, and stays at
  least 90% opaque on a far plate whose name dims. `cg_nameplateDuel` (on; Settings >
  Quick > Nameplates > Duel badge) turns it off, and it shows with the name, so
  `cg_nameplate 0` hides it too. The duellist's body is drawn as JoF EJK draws it (see
  [Entity render effects](rendering.md#entity-render-effects)).
- **Pilots in a vehicle:** a pilot sealed in a `hideRider` vehicle (a fighter, a
  walker) is `Ghost`ed with `SVF_NOCLIENT` (`g_vehicles.c:1924-1940`), so no snapshot
  carries his entity and the world never presents his body. Like stock's crosshair
  target name (`CG_DrawCrosshair`), the plate follows the vehicle's `owner` while it
  is below `MAX_CLIENTS`: the name and team come from his `CS_PLAYERS` string and the
  plate hangs over the vehicle's presented position and box top. Only the name and
  team frame show (no bars, icons or weapon: the client has no state of him). A rider
  whose entity is sent keeps his own plate, anchored to the vehicle if his body is
  hidden (`nameplate_math::anchor`, `hidden_pilot`).
- `cg_nameplateSelf` (off): your own plate over your head in third person, with your
  real health, shield and Force (the server sends you those), your weapon and your
  badge when your key is verified. In first person it is not drawn: it would sit in
  the camera.
- **Modes (V):** the `nameplates` command, bound to V by default (Controls > Other >
  "Nameplate mode"; an existing config gets it when V is free), cycles off, names
  only, target, everyone, and round again, showing the mode's name near the top of the
  screen for a moment; `nameplates off|names|target|all` picks one. Target shows bars
  (and the weapon and power icons) only on the player you aim at, kept for three
  seconds after the crosshair leaves them, and on your duel opponent. Names only is
  just the names. The modes set `cg_nameplate` and `cg_nameplateBars` (0 names, 3
  target, 2 everyone); 1 (allies only) stays reachable in the settings.
- `cg_nameplateBars`: 0 none, 1 allies only, 2 everyone (default), 3 target and duel
  opponent. `cg_nameplateScale`
  sets the text size, `cg_nameplateForce` the Force bar (see [Sharing](#sharing-and-estimates)
  for whose), `cg_nameplateWalls` shows
  players behind walls at 35% opacity instead of fading them (the same BSP trace as
  before, from the rendered eye; changes ease over 120 ms), and `cg_nameplateNpcs`
  adds NPC plates (class name, health; at most 16; vehicles skipped).
- `cg_nameplateHideDuelers` (off; "Hide duellers" under Nameplates in SJK Settings >
  Advanced > HUD and classic Settings > HUD): hide the entire plate (name, bars and
  icons) on players the server marks as participating in a private duel. This also
  hides your own third-person plate during a private duel. It uses the player's
  snapshot duel marker, not the Duel/Power Duel game type or team. Players whose
  entities are omitted from snapshots, such as sealed vehicle pilots, have no
  available private-duel marker and keep their plates.
- `cg_nameplateIcons` (on): up close, a row of the holocron icons of the Force powers a
  player has on sits over the name, at most four, dark side first (lightning, grip,
  drain, rage, then protect, absorb, speed, heal, team force, mind trick, sight). They
  come from `forcePowersActive`, so every player's powers are known without a request;
  jump, push, pull and the saber powers are left out because they last a moment. The
  pictures are the Force bar's, loaded with the map.
- `cg_nameplateDebug` logs, every two seconds, what the server sends about each other
  player (health, `tinfo`, active powers, weapon and stance) and the estimates (health,
  armour and Force as `guess[low..high]`) to `logs\last-client.log`, with the regen
  pace in use and where it came from, the server's Force-related info keys, whether
  its pain events carry real health, and your own real Force next to the estimate
  for yourself.
- While a menu is open the plates hide: their text is in the classic stream, which
  draws over the menus' text.

### Sharing and estimates

Another player's health, shield and Force bars show only when that player shares
them (decided 15/10/2026; since 17/10/2026 with no estimates on anyone else), and
seeing any of them means sharing your own, through the hub (Sol's rule, 15/10/2026):

- `cg_nameplateShare` (Settings > Interface "Share my bars", and First setup):
  your client sends your real health, shield and Force (from the snapshot the server
  sends you, with its server time) to the SJK hub, which passes them to the other
  SJK players of your server ([hub-chat.md](hub-chat.md#vitals)). It needs
  `cl_identity` and a live claim, so it starts a few seconds after joining. It is
  held on (the client sets it back) while nameplates show bars (`cg_nameplate` on,
  `cg_nameplateBars` not 0); it can only be off with bars off.
- **Tied to the hub:** shared bars on other players show only while the
  hub's last answer (under 6 seconds old) lists you sharing in your slot. With
  `cl_identity` off, no hub, no claim yet or the hub down, other players get no such
  bars; the server's own (teammates' `tinfo`, NPCs) stay. The hub itself gives the
  values only to players who share: a post without `share` gets who shares, not
  their numbers.
- **Players who do not share** (Sol's rule, 16/10/2026 for SJK players, 17/10/2026
  for everyone, bots and players without SJK too): grey bars, full length and empty,
  for every viewer: health, shield, and Force with `cg_nameplateForce`. Nothing
  overrides it, not even a teammate's `tinfo`. Their name, weapon and power icons
  stay. NPCs keep their bars.
- **Sharers:** their bars are the estimates below, corrected with every value they
  share ([estimate.rs](../crates/sjk-viewer/src/hud/estimate.rs) `History`). A
  shared value is up to about a second old by the time it arrives, so it re-bases the
  estimate as it stood at that server time: what was seen since (a hit, a pain, a
  heal) is kept on top, and the doubt the estimate had then is dropped. With nothing
  seen since, the bar is exact; it never waits for the next value to move.
- **Gold contour:** the plate of a player who shares (`cg_nameplateShare`) is framed in the verified badge's
  gold instead of grey; in team games a thin line of the
  team's colour stays inside it. Your own plate (`cg_nameplateSelf`) gets it too
  while the hub lists you sharing. NPC plates (`cg_nameplateNpcs`) are a paler gold, drawn thinner.
- `cg_nameplateEstimate` ("Bars on everyone", estimates on every player without
  SJK) was removed on 17/10/2026, when they turned grey. A client from before may
  still tell the hub it sees everyone's bars: their player card (`inspect`) says, in
  amber, "Sees your health, shield and Force". A sharer's card says "Shares their
  health, shield and Force".
- The hub cannot check what a client says: a modified client could share false
  values or estimate without saying so. The protocol keeps honest clients honest.

Where the numbers come from, and what is not known:

- **Teammates** in team games: health and shield from the server's team overlay data
  (the `tinfo` command), exact. The client asks for it (`teamoverlay` userinfo)
  whenever nameplate bars are on; SJK draws no team overlay of its own.
- **NPCs** (and any entity whose `entityState_t::health`/`maxhealth` the server
  sets): exact. The SJK server sets them for NPCs, breakables and emplaced guns, not
  for clients; stock JKA does not either.
- **Sharers' health and shield** are estimated
  ([vitals_estimate.rs](../crates/sjk-viewer/src/hud/vitals_estimate.rs)) once per
  snapshot (every one, so no event is missed) from what the server sends everyone,
  with its own rules:
  - spawn at a quarter over the maximum health with a quarter of it as shield (125
    and 25; 100 and none in Duel), learnt from your own spawns for mods that change
    it; health and shield over the maximum lose a point a second;
  - `EV_PAIN` carries the health left after a hit of ten or more (at most every
    700 ms): exact. `EV_SHIELD_HIT` carries the shield a hit took: exact.
    `EV_SABER_HIT` sizes the damage (under 5, under 20, more); a missile striking
    (`EV_MISSILE_HIT`) is a hit of unknown size, except JA+'s grapple hook (the only
    missile fired as `WP_STUN_BATON`), which does exactly one point. With no pain after a hit outside the
    700 ms, it was under ten. A hit with no shield flash met no shield;
  - when you hit someone, the server tells you their health and shield before the
    hit (`PERS_ATTACKEE_ARMOR`), matched to them when only one player fits (your
    saber's victim, your duel opponent, or the only estimate that allows the values);
  - falls (`EV_FALL`/`EV_ROLL`, exact), medpack and shield pickups
    (`EV_ITEM_PICKUP`), Force heal (its sound at the player: 5 to 25), team heal
    (`EV_TEAM_POWER` names everyone it reached: 50, 33 or 25 each by how many),
    deaths (`EF_DEAD`, `EV_OBITUARY`) and the respawn after one; a spawn with no
    death seen toggles `EF_TELEPORT_BIT`, as a teleport does, so it only raises the
    high bound;
  - drain, lightning and grip, rebuilt shot by shot (see **Force streams** below):
    drain heals the drainer by what it takes, lightning does 1 or 2 a shot (twice
    two-handed at level 3) to the shield first, whose flash counts it, then the
    health, and a bolt seen landing with no flash means no shield was left; grip
    does 2 a second past the shield (its crushing blow is a pain);
  - rage costs 2 health every 150, 300 or 450 ms by level (the guess level 3), down
    to 1, and halves every blow and fall before the shield; protect keeps 40, 60 or
    80% of a blow off the health (the guess 80%) and pays for it with Force, point
    for point, a half or a quarter (halved with the boon), which the Force estimate
    takes;
  - private duels (`EV_PRIVATE_DUEL`, 1 at the start, 0 at the end): on JA+ both
    duellists are set to 100 health and 100 shield when it starts, and the winner to
    100 and 25 when it ends, whatever they had left; on stock JKA the start changes
    nothing and the winner is healed to the maximum. A duel ending within a second of
    a death was won; one ending otherwise (called off, too far apart) changes
    nothing. The start values are learnt from your own duels once you have had one,
    for servers that set others (jaPRO's `g_duelStartHealth`);
  - JA+'s chat protection: a player with the chat balloon up (`EF_TALK`, whatever
    opened it) takes no damage unless in the middle of an action (a saber swing,
    its transitions or a special, a kick, a punch or a melee grab, read from the
    saber move and the torso and legs animations). On JA+ a hit seen on such a player
    without a pain or a shield flash takes nothing, and a fall takes nothing; a pain
    or a shield flash still counts, since the server only sends one when damage landed;
  - a player first seen, or back after time out of view, gets a wide range that the
    next pain closes.

  Servers can hide the pain values: JAPro's `g_stopHealthESP` (off by default) sends a
  fixed 50 or no pain event at all, and scrambles `PERS_ATTACKEE_ARMOR`. At 2 it plays
  the pain sound instead (`EV_ENTITY_SOUND` of `*pain25` to `*pain100`, chosen by the
  health left), which bounds the health to its quarter: 25 or less, 50 or less, 75 or
  less, or more. Your own pain
  events are checked against your real health: until one has been, pain values are
  trusted but their absence proves nothing; if they do not match (or every pain says
  50), they are ignored and a pain only says ten or more landed. JA+ is closed source
  and not verified.
- **Force** is never sent for other players, so it is estimated
  ([force_estimate.rs](../crates/sjk-viewer/src/hud/force_estimate.rs)) from their
  entity state with the server's own rules: a full pool at spawn, a point per
  regen pace (measured from your own pool while you idle, which the server does send,
  else `g_forceRegenTime` from the server's info string, else 200 ms; six times as
  fast with the boon) while no power but drain is on and no saber is thrown or in a
  special move, and these costs:
  - the price of each power when it switches on (a grip a flat 30; drain and
    lightning need 25 to start), and the running costs of protect, absorb, grip and
    lightning;
  - a force jump nothing to start, then while it lasts every 300 ms (200 at level 1)
    20, 16, 12, 8, 6 or 4 by how fast it rises (`pos.trDelta[2]` over 250, 200, 150,
    100, 50, 0), over the jump level;
  - push and pull (by their animation) and a saber throw at their price; a disarm
    flies the saber too but its own entity is not in flight, and costs nothing;
  - saber specials when their `saberMove` starts, proving the pool held the price:
    a kata 50, a cartwheel or butterfly 10, a lunge, spin, flip attack, death from
    above, stab down, roll stab or dual/staff jump attack 25; a wall run-up's flip
    and a jump off a wall grab 6, a level-2 flip back off a wall half the rise's
    band;
  - heal (its sound) and team heal or energize (`EV_TEAM_POWER`, the caster stands
    at it), which never show as active, at their price; an energize gives its 50,
    33 or 25 to each named teammate;
  - protect's share of the blows it softens; drain shots and absorb's point back a
    drain or lightning shot (see below); an absorb's hit sound while somebody pushes
    or pulls gives 6 to 18 back;
  - `EV_NOAMMO` 0 (a special refused for want of Force) proves the pool under 50;
    becoming the Jedi Master fills it.

  Power levels are not sent, so the low bound pays each power at its dearest level
  and the high bound at its cheapest (the guess at level 3); until the pace is
  measured the bounds refill a little slower and faster than the guess. A power
  starting proves the pool held its price, which raises the low bound. It misses
  saber blocks in some mods and anything that changes costs. It refills while a
  player idles, so the range closes within about twenty seconds.
- **Force streams** ([force_streams.rs](../crates/sjk-viewer/src/hud/force_streams.rs))
  rebuild drain, lightning and grip shot by shot, as `ForceShootDrain`,
  `ForceShootLightning` and `DoGripAction` do it. Drain and lightning shoot every
  50 ms: level 3 reaches everyone within 512 units (lightning 300) of the caster's
  origin, in a 60-degree cone along its view and in clear sight of the map's walls,
  levels 1 and 2 the first player on a 2048-unit line. A drain shot takes 2, 3 or 4
  Force by level, heals the drainer as much, stops the victim's refill for 800 ms,
  and costs the drainer 5 (at levels 1 and 2 only when the line found a player) and
  its own refill for 500 ms; while drain stays on the drainer holds at least 20. A
  lightning shot does 1 or 2. Against an absorb that is up, what the levels'
  difference leaves gets through and a point of Force comes back a shot. A grip holds
  the first player on a 256-unit line from the gripper's eyes when it starts and
  hurts them 2 then and every second. Other players' levels are not sent: the guess
  is level 3, the bounds the dearest and cheapest level that reach. Your own levels
  are your Force profile's, so your drain and lightning on others are exact.
  `EV_FORCE_DRAINED` (at most every 400 ms a victim) says a victim lost at least a
  drain shot, and a renewed electrification (`emplacedOwner`, which lightning sets
  800 ms ahead) while somebody casts lightning that a bolt landed, even where the
  geometry missed it. Doors, movers and other players in the arc's line of sight are
  not checked. Your own pace measurement skips the time drain holds your refill
  back.

  Your own drain is measured, not rebuilt: the server sends you your pool, which
  each shot takes 5 from, and your health, which each shot raises by what it took
  (below your maximum). Your victims lose what you gained (at the maximum, the shots
  you paid for times what a shot takes), shared among those in reach, else those
  the drained event names, else the player most in front. Your drains also teach
  the server's pace (milliseconds a shot) and a shot's strength, which then apply to
  everyone's drain and lightning: servers built on the original game (JA+) shoot
  every server frame rather than every 50 ms. Each of your drains writes one line
  to the log (`nameplate drain:` with its shots, heal, victims, how they were found
  and the pace).

`cg_drawPlayerNames` keeps TaystJK's plain overhead names (0 off, 1 names, 2 adds a
health strip, text only, off by default); they are hidden while nameplates are on.
Both stand above an ally's team logo
([Team outlines and logos](#team-outlines-and-logos)). At most 32 players and 16 NPCs
are tagged and nothing is allocated per frame. Not tested in game yet; the estimates
are unit-tested against synthetic snapshots only.

## World pings

A world ping marks a point in the map for other SJK players on the same server, as
modern games ping (Sol's request, 13/10/2026): a gold ring at the point, seen through
walls, with the pinger's name above it and the distance under it, a short sound, and
gone after 7 seconds (it fades over the last 1.5). Players on other clients and game
servers see nothing: pings travel through the SJK hub ([hub-chat.md](hub-chat.md#pings)).
Code: [world_ping.rs](../crates/sjk-viewer/src/world_ping.rs) (rules, limits, drawing)
and [world_ping_frame.rs](../crates/sjk-viewer/src/world_ping_frame.rs) (the press, what
arrives, the HUD pass).

- **Pinging:** `+ping` (Settings > Key bindings > Interaction > World ping; the middle
  mouse button, `MOUSE3`, in a new profile and in one where it is free, as retail binds it
  to the saber style, which SJK binds to L) pings on the press; `worldping` does the same
  from the console or a [quick wheel](#quick-wheels) choice (`ping` is the engine's server
  query). The point is where the crosshair's ray meets the map: the dynamic crosshair's
  (`cg_dynamicCrosshair`, the player's aim from the muzzle, which a third-person camera
  with its own pitch or tilt does not look along), else a ray from the camera through the
  screen's centre (static crosshair, free camera), to 16,384 units against solid brushes (not clips; the sky and nothing in reach say
  "Nothing to ping there"). It shows at once for the pinger, a paler gold, and goes to
  the hub in whole units; in a local game nothing is sent.
- **Who sees it** ([`Scope`](../crates/sjk-viewer/src/world_ping.rs)): in a game without
  teams (FFA, Holocron, Jedi Master, Duel) every SJK player on the server; in a team game
  (Team FFA, Siege, CTF, CTY) the pinger's team only; in Power Duel the pinger's side
  (the lone duelist, or the pair). Spectators see the players' pings in a game without
  teams, never a team's, and spectators' pings reach only other spectators, so a
  spectator cannot guide a player. The sender names its scope from its own team; each
  reader shows a ping only when the game shows, in the ping's slot, the name the
  sender's hub claim was made under (as badges and emotes check) and the team that scope
  belongs to, and when its own team sees that scope. A ping sent before a team change and
  read after it is dropped.
- **Limits:** a player sends at most one ping a second and four in ten seconds ("Too many
  pings: wait a moment"), under the hub's five in ten. On arrival each sender is held to
  one every half second and five in ten seconds whatever the hub let through, at most two
  of a sender's pings show (a third replaces their oldest) and 24 in all. The sound plays
  at most every 0.3 seconds. When the hub refuses a ping (no identity, not claimed on the
  server yet), the chat says why in an `SJK ping:` line, once in 30 seconds for the same
  words.
- **Hiding and muting:** `cg_worldPings` (archived, default 1; Settings > HUD > World
  pings > Others' world pings) 0 hides other players' pings; one's own still show and
  are still sent. A player muted on this PC ([hub-chat.md](hub-chat.md#muting-a-player))
  shows no pings. `cg_worldPingSound` (archived, default 1; Settings > HUD, and in the
  SJK UI's Advanced Sound under Notifications) plays `sound/interface/update` from the
  game's own data, the player's own pings included.
- **On screen:** the ring, its centre dot and, for the first 0.8 seconds, a ring that
  grows out of it; a point off the screen or behind the camera sits on the screen's edge
  (44 pixels in at 1080p) toward it, smaller, with only its distance. The distance is from
  the camera, in metres of 39.37 units (a unit an inch). Pings hide with the HUD's
  information: in the menus, the console, the scoreboard, the intermission and with
  `cg_draw2D 0`. Another server, or leaving it, clears them.
- **Not built:** SJK has no minimap or radar to show them on; a ping cannot name a player
  or an item (only a point); the team rule is enforced by honest clients only, since the
  hub knows no teams ([hub-chat.md](hub-chat.md#pings)).

World shot: `world_shot::pings::duel6_world_pings` (1080p and 4K).

## Version label

The client draws its build label small at the top centre of every frame, in
menus and in play, so a screenshot says which build it shows:
`SJK <version> · <dd/mm/yyyy HH:MM> · <commit>`, the release version (a date
version such as `2026.1005.1`, or `dev` for a local build), the source commit's date and time on the 24-hour
clock, and its short hash ([version_overlay.rs](../crates/sjk-viewer/src/version_overlay.rs),
[build_info.rs](../crates/sjk-viewer/src/build_info.rs)). The top centre is free in
the stock and game-data HUDs (gauges at the bottom corners, the FPS counter and
timer at the top right, notify and vote lines at the top left). It is left out
while the console is open, which shows the version in its own corner.
`cg_drawVersion 0` (Settings > HUD > "Version and date") hides it. The log names
the same build at startup (`build: SJK ...`). How the version is decided is in
[SJK conventions](sjk.md#version).

## Changelog page

The main menu's Changelog entry (SJK > What's new in the SJK UI; SJK >
CHANGELOG on the classic page), the in-game SJK pop-up's Changelog and the
`changelog` console command show every SJK release from [CHANGELOG.md](../CHANGELOG.md), built into the client
([changelog.rs](../crates/sjk-viewer/src/changelog.rs), parsed by
[changelog_data.rs](../crates/sjk-viewer/src/changelog_data.rs)). Releases are
listed newest first on the left, "Unreleased" (the changes on `main` since the
last release) on top; the right pane shows the selected release's introduction
and its changes, wrapped to the pane, each followed by its credit in capitals
(the SJK UI's look draws what a change puts in backticks as a chip; a click opens
Settings on that setting or types it on the console's input line).
Up and Down (or a click) pick a release; Page Up, Page Down, the wheel over the
pane and its scrollbar scroll it; Escape closes. Like the `debug_panel` list the
page lives in the console and is drawn in its place, so it opens over the menus
and in a match, and closes the console with itself when it opened it. With
`ui_menuStyle classic` it takes the classic+ look of the command browser's
pop-up ([changelog_classic.rs](../crates/sjk-viewer/src/changelog_classic.rs)):
the in-game pop-up box and title band, a retail list box of releases, a detail
box with the release and its credits in gold, a Close button and the
description line, in the menus' retail font. How the file is kept is in
[SJK conventions](sjk.md#changelog). A release's `> Heading: sentence` recap lines are
not drawn here: they feed the [what's-new card](sjk-ui.md#whats-new-card), the pop-up
shown once after an update (`cl_whatsNewSeen` remembers the version; `whatsnew` previews
the newest recap).

## Updates

The client looks for a newer SJK release when it starts and from the Update page
([update.rs](../crates/sjk-viewer/src/update.rs) does the work,
[update_panel.rs](../crates/sjk-viewer/src/update_panel.rs) draws the page).

- `cl_autoUpdate` (default 1; Settings > Network > Check for updates)
  asks `api.github.com/repos/Sol-Vulpes/SJK/releases/latest` at start-up on
  a worker thread, and again every 30 minutes while SJK runs as long as nothing new
  was found (or the check failed), so a release made during a match reaches the
  players in it as its card without a restart (since 10/10/2026). A newer version shows on the main menus' version line
  ("update 2026.1010.1 available"). The check sends the request GitHub needs and
  nothing else; turn it off to send none.
- When the check finds a newer release, a card says so once a session at the top of
  the screen, over the menus or a match, the achievements' card
  ([identity.md](identity.md#achievements); since 10/10/2026, Sol's request): "Update
  available", "SJK 2026.1011.1", "Main menu > Update" and "Install it there; it starts
  when SJK restarts" (or, for a folder that cannot update itself, "This folder cannot
  update itself: see its page"), with SJK's emblem in the SJK UI's holo blue and no
  sound. It waits while the window is away, and the Update page opening takes it back
  (a release found while the page shows gets no card). `debug_update [version] [manual]`
  shows the card for a made-up release (2026.1231.1 without one; `manual` for the
  folder that cannot update itself) and checks and installs nothing.
- Main menu > Update (classic: SJK > UPDATE) or the
  `update` command opens the page (with the SJK UI's menus, a pop-up card over
  the map: [sjk-ui.md](sjk-ui.md#sjks-pages)). It shows the state and offers Install (Enter),
  Check again (C) and Release notes (N); Escape closes. Opening it with nothing
  checked yet checks at once.
- Install downloads `SJK-<version>-<platform>.zip`, refuses it unless its SHA-256
  is the one in the release's `SJK-<version>-SHA256SUMS-<platform>.txt`, writes the
  ZIP's files beside the running program as `<name>.new` and renames each running
  file to `<name>.old` before the new one takes its name (a Windows program can be
  renamed while it runs, not overwritten). A failed step puts the old files back.
  Only plain file names are taken from the ZIP. The `.old` files are deleted at
  the next start. Settings in `GameData/SJK/` are not touched.
- The new version starts when the client exits, after it saved its settings:
  "Restart now" on the page quits, or the player keeps playing and restarts later.
  The program's path is read once at start-up: on Linux `current_exe` follows the
  running file, so after its rename it named `<name>.old` and the restart started
  the old version again (fixed 16/10/2026).
- Nothing is installed without Install. When the program's folder is not writable
  (a Program Files install) the page offers the release page instead.
- A local build says `dev`, which has no release number to compare, so it reports
  that and never offers an update. `cl_updateAs 2026.1001.1` makes it check as if
  it were that release, to try the page without a release build.

## Credits page

The main menu's Credits entry (SJK > Credits in the SJK UI; SJK > CREDITS on the classic page), the in-game SJK pop-up's Credits and the
`credits` console command show who makes SJK, from
[credits.txt](../crates/sjk-viewer/assets/credits.txt), built into the client
([credits.rs](../crates/sjk-viewer/src/credits.rs), parsed by
[credits_data.rs](../crates/sjk-viewer/src/credits_data.rs)), and everyone's
work from [credits_history.txt](../crates/sjk-viewer/assets/credits_history.txt),
which `scripts/credits_history.py` writes from git and GitHub. Each section of
the file (Sol JK, Contributors, Origins, ...) is a heading with its people.

- Everyone with a history (Sol, Bishop, Creyon, Lumaya) has a panel across the
  column: the name large (the first section's larger still), the role, chips
  counting their changes, pull requests and commits, their links, and two folds.
  HIGHLIGHTS holds the card's own lines; ALL WORK (ALL PULL REQUESTS when all of
  it is pull requests) lists every feature and pull request newest first under
  the day it was merged into main, each row with its pull request's number
  (JKR #48, SJK #23) and how many commits it has. A row of several commits
  unfolds into them: hash, subject, and who made it when that is not the owner.
- Every fold is closed when the page opens. A fold's rows fade in one after the
  other as it opens; EXPAND ALL in the footer opens every fold at once and then
  reads COLLAPSE ALL.
- Clicking a pull request's number opens it on GitHub, clicking a commit (or a
  one-commit row) opens the commit on SJK's GitHub, a GitHub handle opens that
  profile and a link its address (`update::open_page`, https only). They brighten
  under the pointer and take clicks only while the page shows them whole, up to
  84 targets a frame, under the menu canvas's limit.
- The rest (Claude, the fonts, the reference clients) sit on cards in rows of up
  to three with their lines shown.
- A card's medals (credits.txt `medal:` lines, the hub's ids:
  [identity.md](identity.md#medals)) show on it: a chip with the medallion and the
  name after the counts (under the role on a card without a history). Creyon and
  Lumaya wear Early Contributor.

The page is animated from the clock alone. Golden god rays turn slowly down from
above the top of the screen, two sets against each other so the shafts shimmer
where they cross, sparks rise through them, and SJK's emblem breathes in front of
two sunbursts turning opposite ways, above a title a glint crosses: the look of
SJK's site (`site/assets/effects.js`). The rays are two light pictures the emblem
module draws on its worker at start (`EmblemLayer::Sunburst` and `Godrays` in
[emblem.rs](../crates/sjk-viewer/src/menu/emblem.rs)), added as light by the UI
renderer's additive pipeline and turned by their texture coordinates
(`emblem::rays`). Panels rise into place as the page opens and their edges glow
in turn. With the classic menus it takes retail's gold and blue and the menus'
retail font; with the SJK UI it has that UI's look, the emblem as a sun on the
left with the sections under it and the people in a column on the right
([sjk-ui.md](sjk-ui.md#sjks-pages)). Arrow keys, Page Up and Page Down, Space,
the wheel and the scrollbar at the right edge scroll it, gliding to the place
asked; Tab (or `]`) scrolls to the next section and Shift+Tab (or `[`) to the one
before, round to the first past the last; E opens every fold or closes them;
Escape, Enter or CLOSE closes it. Like the changelog it lives in the console, so it opens over
the menus and in a match. How the files are kept is in
[SJK conventions](sjk.md#credits).

## Settings usage

Since 12/10/2026 (Sol's request) the client tells the SJK hub, anonymously, which
settings players change and what they search for in Settings, so the team can see
which settings matter and what players look for
([settings_usage.rs](../crates/sjk-viewer/src/settings_usage.rs), recorded by
[settings/usage.rs](../crates/sjk-viewer/src/settings/usage.rs), the wire format in
[sjk-identity's usage.rs](../crates/sjk-identity/src/usage.rs)).

- `cl_shareUsage` (archived, default 1; Settings > Network > Share settings usage,
  and Simple's General) turns it on. At 0 nothing is sent and nothing is kept.
- A change records the setting's console name (or its row's stand-in, such as
  `renderer`), the category it was shown under (a search's group for a result),
  the mode on show (simple or advanced; a classic+ panel counts as advanced), its
  value before and after as the row shows them (On, Off, a choice's name, a
  number) and the search showing, if one was. A text setting (an address, the
  GIPHY key) sends `(text)`, never its value. Only the row the player acted on is
  recorded, though a quality level changes many rows at once. A change of the same
  setting within 4 s updates the last event (a slider dragged, a key held), and one
  brought back where it started is dropped.
- A search is recorded once it settles: the longest text typed in one go (a
  shorter correction is not recorded again) and how many settings it finds, when
  the field is left, a result is changed or Settings closes.
- What was recorded goes to the hub (`POST /v1/usage/settings`, unsigned) when
  Settings closes, also from Key bindings, on a thread of its own: batches of up to
  100 events with the build and a random id drawn once per run, so one run's events
  read together without linking runs. No identity key, signature, name or server is
  sent. Up to 400 events wait (the oldest go first); what the hub cannot take is
  dropped, not retried, as these are statistics. `cl_hubUrl` empty sends nothing.

The hub's side (the endpoint, its storage without the sender's address, and the
operator's summary of the most changed settings and searches) is built on a branch
of the hub and not deployed yet, so nothing reaches it until it is; nothing of this
was tested in game.

## Identity

SJK keeps an identity key and can mark other SJK players on a server with SJK's
emblem on the scoreboard and in a card beside them; the design, limits and privacy
are in [identity.md](identity.md).

- `cl_identity` (default 1; Settings > Network > SJK identity) makes the key
  (`identity.key` beside `config.cfg`) the first time it is on and lets the client
  talk to the hub. Off sends nothing and makes no key.
- `cl_hubUrl` (default `https://sjk.dfox.app`; Settings > Network > SJK hub) is the
  hub's `https://` address. Empty means no hub, so nothing is sent.
- The Identity page is where a player sets all of this up without a command:
  Settings > Network > SJK identity key, main menu > SJK > IDENTITY (classic menus),
  the in-game SJK menu, or the `identity` command
  ([identity_panel.rs](../crates/sjk-viewer/src/identity_panel.rs)). It shows the key
  id and the file to back up (hidden behind bullets until Show key, each time the page
  opens, so it can be opened on a stream), whether the hub's operator vouches for the
  player, the hub's status and the players it knows on the current server, and has
  - a switch for `cl_identity` ("Share my identity with the SJK hub", ON or OFF);
  - an optional bio field (500 characters at most), saved to the hub with Save or Enter.
    There is no name field: the hub takes the name the player plays under (`name`), and
    the page shows it with up to three earlier names. A new player has nothing to do;
  - "Copy my key id" (for the operator to verify the player);
  - "Show key" (then "Hide key"): shows the key id and file;
  - "Use the official hub", shown only while `cl_hubUrl` is another address (a hub tried
    on this PC, say): it sets `cl_hubUrl` back to `https://sjk.dfox.app`. A saved
    `cl_hubUrl` wins over the default, so an old address stays until it is changed.

  With the classic menus (`ui_menuStyle classic`) the page takes the classic+ look
  ([identity_panel_classic.rs](../crates/sjk-viewer/src/identity_panel_classic.rs)): the
  retail pop-up and title band, option rows with the highlight band, text fields in list
  boxes, gold buttons that glow, and a description line under the box.

  Tab, Shift+Tab and the arrow keys move between the controls, Enter or Space works the
  switch and the buttons, letters type into the focused field (Ctrl+V pastes) and Escape
  closes; the pointer works too. The bio field follows the hub's copy until the player
  types in it. The commands remain: `identity bio <text>` changes
  the bio (`identity name` says the name is the one played under), `identity key` prints
  the key id and file,
  `identity who [slot]` lists known players (with a slot, that player's bio).
- The Profile page (in the SJK UI the Profile screen's SJK Profile tab, opened by its
  main page's Profile and profile card and the in-game menu's Profile; the classic
  menu's SJK page and in-game SJK menu; or the `profile` command) shows the player's
  profile as others read it on the hub: name, verified, member since, other names,
  medals (on its own only), the bio (written there, up to 6 lines under the hub's
  rules), their record from the achievement counts and how many achievements and
  shaders they hold. It
  has the SJK UI's look in every menu style; See the board and See the collection
  open the Collection (`collection`, `achievements`, `unlockables`;
  [sjk-ui.md](sjk-ui.md#collection)): medals, achievements, saber shaders
  ([unlockables.md](unlockables.md#shaders-tab)), toys, nameplates and holocrons
  (`holocrons`, [holocrons.md](holocrons.md#the-holocrons-tab)), a screen of its own in
  the SJK UI ([identity.md](identity.md#profile)); See holocrons opens its Holocrons.
  The SJK UI's main page and in-game menu show a profile card bottom left (picture,
  name, verified, medals and achievements unlocked) that opens it with a click
  ([sjk-ui.md](sjk-ui.md#profile-card)).
- A picture: choose a PNG, JPEG or TGA with Browse... on the Profile page's picture
  panel (the system's file dialog), drop one on the window or type `sjkavatar <file>`
  (a path's words may be left unquoted); it is checked (64 to 8192 pixels a side, at
  most four times as long as wide, something visible, a whole file of one of those
  formats whatever its name; every refusal says why), shown cropped to a square and
  made 128 pixels across, and Use this picture sends it to the hub, where everyone
  sees it. `sjkavatar clear` (or Remove picture there, pressed twice) takes it
  down and `sjkavatar` alone opens the picture panel. Other players' pictures are
  downloaded when first shown and kept in `avatars/` beside `identity.key` (at most
  256 pictures, 8 MB) ([identity.md](identity.md#pictures)).
- Blade skins: `cg_saberSkin` and `saberskin` choose the one worn, shown only while the
  hub profile owns it ([Blade skins](#blade-skins)).
- Achievements are counted in matches on servers and kept in `achievements.json`
  beside `identity.key`, sent to the hub with the identity on; an unlock says so in the
  console and with a card at the top of the screen, the same card that announces a new
  medal or holocron during a match ([identity.md](identity.md#achievements)).
  Settings > Sound > Unlock sound (`cg_achievementSound`) turns its chime off.
- The scoreboard (every style) draws SJK's emblem at the end of the name of a
  player the hub knows (the SJK UI's right after the name), in gold when the
  hub's operator vouches for them, then up to three ribbon bars for the medals the
  SJK team gave them ([identity.md](identity.md#medals)); fewer bars show where the
  name would run short of room. It trusts
  a claim only when the claimed name matches the name the game shows in that slot.
- The Identity page lists the player's own medals: in the SJK UI a column right of
  its own, each with its
  picture, name, description, the date it was given (`dd/mm/yyyy`) and the team's note;
  the classic+ box names them in its status with their medallions. With none, a line
  says how medals come. A medal new to the client shows once in a pop-up on the main
  menu or when the game menu opens (during a match a card at the top of the screen says
  it came first), arriving in a short ceremony with the game's
  Jedi Master fanfare (`music/goodsmall.mp3`, silent without game data); Enter, Space,
  Right, Escape or a click finish the entrance, then take Next or Close. It has the
  SJK UI's look with its menus and the classic+ look with the classic ones
  ([identity.md](identity.md#medals), [sjk-ui.md](sjk-ui.md#new-medal)).
- `debug_medal <id> [x<count>] [note]` (`debug_medal all [note]` for every medal,
  alone it lists the ids: `early_tester`, `early_contributor`, `bug_hunter`, which
  takes a count) shows made-up medals as if the SJK team had just given them, through
  the same queue, card (in a match), pop-up, ceremony and sound, to try them without the
  hub. It sends nothing and never writes `medals_seen.txt`; the console
  closes so the pop-up shows at once on the main menu (in a match, open the game
  menu).
- `debug_holocron <tier|all> [x<count>]` (alone it lists the tiers `uncommon`, `rare`,
  `legendary` and `mythical` with their odds) shows made-up holocron drops as if the hub
  had just dropped them, through the same queue, card (in a match), pop-up, ceremony and
  sound, and puts the first one's chat line in the game's feed, to try them without the
  hub. It
  sends nothing and never writes `holocrons_seen.txt`
  ([holocrons.md](holocrons.md#debug_holocron)). A holocron the hub drops while you play
  shows the same way, once, and appears as a chat line in its tier's colour
  ([holocrons.md](holocrons.md)).
- SJK chat: one conversation for every SJK player, through the hub, in games and in
  the menus ([hub-chat.md](hub-chat.md)). In a game, I (`messagemode5`) opens the
  composer on the SJK channel (Tab cycles All, Team and SJK); hub messages show in the
  chat feed after SJK's emblem, each one flowing line with its text in the SJK chat's own gold
  and the verified tick after a verified sender's name
  ([hub-chat.md](hub-chat.md#how-a-line-looks)). The SJK UI's main page docks it under Recent servers;
  `sjkchat`, the dock's Open chat and the in-game SJK menu open its page. Resting the
  pointer on a name in the chat (composer open), on the dock or on the page shows the
  player's sender card with Mute: a muted player's chat is hidden and, on the server,
  they are drawn as Kyle with the default saber and nothing they cause is heard, on
  this PC only ([hub-chat.md](hub-chat.md#muting-a-player)). `cl_sjkChat 0` (Settings > Network > SJK
  chat) hides it and stops the reading. `sjkemote <id>` sends an emote the SJK players
  on the server see (`sjkemote` alone lists the installed ones; none ship yet).
- Back up `identity.key`: losing it loses the identity.

### Player card

Look at a player, keeping the view steady, and a card appears beside their hips
([player_card.rs](../crates/sjk-viewer/src/hud/player_card.rs)): a leader runs from a
dot at the player's hips (10 units above the origin, near the humanoid skeleton's
pelvis; a crouch lowers it by half the box's drop) to the card, centred on it. While the
card is up a ring in the card's accent colour (red or blue by team, else SJK's light
blue), fading with it, runs around the outside of the player's body and only the
outside: no line where an arm crosses the chest
([card_outline.rs](../crates/sjk-viewer/src/card_outline.rs)). After the scene the body
is drawn again, with its usual vertex path, into a one-channel mask against the scene's
depth, so only the part in view counts and nothing shows through walls; the 2D layer
then draws every pixel outside the mask within 2 pixels of a 1080-line screen (scaled
with the height, 4 at 2160; it was 4 and 8 until Sol halved it on 13/10/2026) in a
scissor around the player, after anti-aliasing so the colour is the card's
(11/10/2026). The team outlines share the ring
([Team outlines and logos](#team-outlines-and-logos)); on the card's player the
card's colour wins. Hats, capes and sabers are not outlined. The ignored
world shot `card_outline_on_a_demo`
([world_shot_card_outline.rs](../crates/sjk-viewer/src/world_shot_card_outline.rs))
plays the demo `SJK_TEST_DEMO` names off screen, pins the card on another player and
renders the ring from an orbit around them and from behind the recorder. It shows what
the server already publishes to every client (name with its colour codes, model and its
head icon, saber hilts with their blade colours, the hat and cape worn, bot skill; not
the duel wins and losses), the saber shader they wear (a live swatch, as the Collection
draws it, and its name; only when it draws on their blade on this PC: worn by their
look, its pack loaded) and, when the hub knows the player, their profile picture in
place of the head icon (the icon then sits small on its lower left corner), SJK's
emblem, their hub name, a gold VERIFIED, a teal SJK TEAM pill for staff, a gem of the
rarest holocron tier they hold with its name and the medallions of the medals the SJK
team gave them ([identity.md](identity.md#medals)). A pinned card adds what takes
reading: the medals' names in small print (a repeatable one with its count),
"Achievements N / 21" with the three rarest unlocked as small medallions (the highest
step of their ladder first, the latest first among equals), the first line of their bio
and "In SJK since dd/mm/yyyy". The profile is asked of the hub when the card is built
for a player, at most every ten minutes a player and never per frame; until it comes,
and from a hub that does not send a field, that line is left out (10/10/2026). It adds
nothing a glance at the scoreboard would not: no health, Force or position.

- `cg_playerCard` (default 1; Settings > HUD+ > Player card) turns it on.
- `cg_playerCardDelay` (default 1.5; Settings > HUD+ > Card delay) is the seconds
  the crosshair must stay on the player before the card fades in. Turning the view
  more than 6 degrees, or losing the player for over 0.3 seconds, starts the wait
  again and the card fades out.
- `cg_playerCardOutline` (default 1; Settings > HUD+ > Card outline) draws the outline.
- On another SJK player's card, under what they share: "Sees your model", or in amber
  "Can't see your model", from the model lists SJK clients send the hub
  (`cl_shareModels`, [identity.md](identity.md#installed-models)); no line when the hub
  has no list of theirs or you wear a retail model.
- `inspect` (Settings > Key bindings > Interaction > Inspect player; bound to X by default,
  or `bind <key> inspect`) pins the card to the player under the crosshair at once, with
  no wait. It stays when you look away or the player moves, follows them (kept inside the
  screen), and a second press or Escape hides it (Escape then does not open the game
  menu); it also drops when the player leaves or the map changes. After hiding it, the
  card does not return until the crosshair leaves that player. Behind you, the card is not drawn but stays pinned.
- **In sight only** (17/10/2026, Sol's request): no card, by looking or by `inspect`,
  for a player farther than 1200 units from the camera or with a wall between the
  camera and both their head and their hips (the nameplates' trace, solid and terrain
  brushes). A pinned card fades out while its player is out of sight that way and
  comes back if they return within 3 seconds; after that the pin drops
  (`player_card::SIGHT_RANGE`, `LOST_MS`).
- World notes: `worldnote` (Settings > Key bindings > Interaction > World note; bound
  to O by default, or `bind <key> worldnote`; until 10/10/2026 `inspect` did it when no
  player was under the crosshair) selects the world surface under the crosshair, or the mover whose bounds the view ray
  meets first, and names it in a centre print (shader, BSP surface, lightmap or vertex
  lighting, distance). When a remap (the map's, the server's or a local `remapShader`)
  draws another shader over the surface, the print, `notes.txt` and `notes.jsonl`
  (`surface.remapped_to`) name it too (`<shader> remapped to <target>`): on a server
  what the player saw is often the remap's target (08/10/2026). Since 13/10/2026 the hub
  gets both (`shader` and `remapped`), and every note keeps the remaps active when it
  was taken, as `listRemaps` lists them without those a later remap hides
  (`remaps`: source `map`, `server` or `local`, `from`, `to`). A model drawn from the
  map's entities comes first when the view ray meets one of its triangles before the
  world (13/10/2026): any entity naming an MD3 (`misc_model_static` props, which the
  client draws from the map, and breakables or other server entities where the map
  places them), read once per map on a press and tested on its first frame, turned and
  scaled as the entity says. The print names its class, path and first shader (with
  its remap); the note keeps `model` (path, class, entity number, targetname, origin,
  angles, scale, skin key, and every surface's shader with `remapped_to`), names the
  model's entity in `entity` and leaves `surface` out. A GLM model or a server entity
  that moved is not picked. While selected, and while its note is written, the surface's
  triangle edges (a mover's bounds; a model's folds, or its turned bounds when it has
  more than 512 triangles) are traced over the view in flickering green dots
  (at most 600 a frame) over a green scanline shade of its triangles (horizontal strips
  with a brighter wave running down them, at most 800 a frame; cut where the surface
  passes behind the eye, so a floor underfoot keeps its visible part). Escape before the second press drops the selection (it does not
  open the game menu then). A second press within 20 seconds opens the text dialog
  ([text_dialog.rs](../crates/sjk-viewer/src/text_dialog.rs)): a panel in the middle of
  the screen with the subject, a text box of wrapped lines (ten in the SJK UI's card,
  scrolling to keep the caret in view; up to 4000 characters since 13/10/2026, 500
  before), a character count, Send and Cancel; Tab moves between them. A note about a
  model shows the model with Clear model (Tab reaches it last), which notes what is
  behind the model instead; a click on the scene round the dialog selects what is under
  the pointer (a model too) for the note, the text kept. With the
  classic menus it takes the classic+ look of the in-game pop-ups, eight lines in a
  retail list box with Clear left of Send ([identity.md](identity.md#bug-reports)); with
  the SJK UI it is its
  pop-up card over the scene, darkened by half so the selection still shows round it
  ([sjk-ui.md](sjk-ui.md#report-a-bug-and-its-dialogs)). The selection's outline also
  shows over a map explored without a server. As it is typed the text
  keeps the bug reports' alphabet (letters, digits, spaces and `. , ! ? ' - : ( )`), and
  Send needs 3 characters with 2 letters. Enter or
  Send appends the note to `notes.jsonl` (one JSON object: map, UTC time,
  `setviewpos`, camera pose, hit point and normal, draw surface, collision flags,
  nearest map entity, and since 13/10/2026 `model` and `remaps`, screenshot path; older
  lines lack the new keys) and a readable line to `notes.txt` in the config
  directory, and takes a silent JPEG of the same view (`note_<unix seconds>.jpg` in the
  screenshots folder; JPEG screenshots drop the alpha channel, which the encoder
  refused before, so `screenshotjpeg` works too). With `cl_identity` on, the note and a
  smaller copy of the screenshot also go to the SJK hub for the SJK team, and a centre
  print says whether it took them ([identity.md](identity.md#world-notes)). Escape or
  Cancel drops it. [world_notes.rs](../crates/sjk-viewer/src/world_notes.rs).
- The crosshair scan that names players under the crosshair finds the target. The card
  is anchored in the world beside the top of the player's box (the box height the server
  sends, as the nameplates use), a body width clear of them in screen pixels, so it
  neither floats above nor overlaps the player at any range. It goes to the player's left
  near the right edge of the screen and hides under the scoreboard, menus, the console
  and intermission.
- The head icon is the scoreboard's (`models/players/<model>/icon_<skin>`, one atlas cell
  per client slot), resolved when a card is shown even with the scoreboard closed. Hats and
  capes are models without pictures, so they are named in text (`c1`, `c2` after the colour
  digits).
- A hub bio is not shown yet.

### Team outlines and logos

In team games (Team FFA, Siege, Capture the Flag, Capture the Ysalamiri) every enemy
gets a red ring around the outside of their body and every teammate a green one, the
player card's ring at its width (Sol's request, 13/10/2026;
[team_outline.rs](../crates/sjk-viewer/src/hud/team_outline.rs),
[card_outline.rs](../crates/sjk-viewer/src/card_outline.rs)). Like the card's, a ring
follows only the part of a body that is in view: the bodies are drawn into the mask
against the scene's depth, so nobody is outlined through a wall and the rings show
nothing the screen does not. Each body writes its group (card, enemy, teammate) into
the one-channel mask through the blend constant, and the ring takes the colour of the
nearest body; the ring is drawn only inside a box around each outlined player, boxes
that overlap merged so no pixel is drawn twice, with about one mask lookup per pixel of
its circumference (8 to 16 a step). Nobody is outlined in free-for-all, duel or power
duel, while spectating freely (a followed player's team is the viewer's), nor a dead,
cloaked or mind-tricking player, nor anyone off screen. Hats, capes and sabers are not
outlined. The rings hide with the overhead labels (menus, the scoreboard,
intermission, `cg_drawHud 0`, `cg_draw2D 0`).

- `cg_teamOutline` (default 1; Settings > HUD+ > Team outlines, and Simple > HUD)
  turns them on.

Over each teammate's head stands their team's logo, as JoF EJK floats it
(`CG_PlayerFloatSprite` in `CG_Player`): the Imperial logo (`sprites/team_red`) over a
red teammate, the Rebel one (`sprites/team_blue`) over a blue one, every player's while
spectating freely, the duel partner's icon (`gfx/mp/pduel_icon_double`) in power duel
and the master's foes in Jedi Master, as stock does; Siege's per-map team icons are not
read, so Siege shows the red and blue logos, stock's fallback
([team_logo.rs](../crates/sjk-viewer/src/hud/team_logo.rs)). EJK's sprite is 20 units
across in the world and hidden by the depth; SJK's logo is a picture on the screen as
large as a 24-unit sprite, held between 20 and 40 pixels of a 1080-line screen so it
stays readable far away and never outgrows a name up close, its foot just over the head
(over the talk balloon or connection icon when one floats there), and a trace from the
eye to that point hides it behind walls. The nameplate, or the plain overhead name,
stands above it. It hides with the nameplates (menus, the scoreboard, intermission,
`cg_drawHud 0`).

- `cg_drawFriend` (default 1; Settings > HUD+ > Team logos) turns
  them on. It was a small coloured square until 13/10/2026.

The team overlay at the top right (`cg_drawTeamOverlay`, teammates' names, health and
location) was removed the same day at Sol's request: the outlines and logos show who
is who. The setting and its position settings (`cg_drawTeamOverlayX`, `Y`, `Scale`,
`Weapons`) are no longer registered (a saved line becomes an unused cvar), a layout's
`team_rows` widget draws nothing, and the client asks the server for the team data
only for the nameplates' bars.

## Force wheel

With the `game` and `classic` HUD styles, `forcenext`/`forceprev` show JoF
EternalJK's Force wheel: retail's Force selection bar (`CG_DrawForceSelect`),
the selected power large in the middle above the HUD with up to three
neighbours on each side and its name under it, for 1.4 seconds as in retail.
It scales with `cg_hudScale`; the `radial` style keeps its list of names. On a
JoF JA+ server that grants them, the wheel also holds Stasis and Repulse (after
Sense) and Dash (before Speed), from spare `forcePowersKnown` bits, as in JoF EJK
([force_wheel.rs](../crates/sjk-client/src/force_wheel.rs)). They are never sent
as the selected power: with one selected, `+useforce` engages Stasis (the
usercmd button the server reads) or sends `force_repulse` or `force_dash` once
per press, and the selection stays until the server takes the ability away. The
binds `force_dash`, `+force_stasis` and `force_repulse` (Settings > Key
bindings > Force powers) work without the wheel; `+force_stasis` does nothing
where Stasis is not granted. On JA+, merc mode shows Lightning as the
flamethrower until the player is seen using real lightning. Repulse, Dash and
the flamethrower use JoF EJK's pictures from `EternalJK/jofclient-assets.pk3`;
Stasis has none and shows Jump's. Power names are retail's (Dark Rage, Sense).
SJK's own [Illuminate](#illuminate) is a toy, not a Force power: it is not on the wheel.
See [hud/force_wheel.rs](../crates/sjk-viewer/src/hud/force_wheel.rs).

### JA+ flamethrower presentation

On a JA+ server (gamename `ja+`, `japlus` or `ja_plus`; abyssmod and UA Galaxy,
which share JA+'s dialect, are left out until their flag meanings are verified), firing the flamethrower shows its flame stream and
hit effects, uses the Lightning-hold upper-body pose, and plays its positional
flame sound until firing stops. Local predicted firing and remote players use
the existing snapshot flags; ordinary Lightning and Drain retain their effects.
The server owns damage, fuel and all weapon rules.

`cg_flameThrower` (archived; Settings > Game options in classic, Settings >
Advanced > Gameplay in SJK, **Flamethrower effects**) selects Auto (`-1`, default),
Off (`0`) or JA+ (`1`, force support on a server using the same flags). The
documented capability table is in
[japlus_flame.rs](../crates/sjk-viewer/src/japlus_flame.rs); it detects the server
independently of the optional JA+ command mod. The flame graphs and sound must
be present in installed game data, such as the JoF EJK client-assets pack.
SJK does not bundle them. Without the flame stream graph the player keeps
showing stock Lightning; other missing assets leave that effect or sound absent.
The flame sound follows the same detection as the effects, refreshed when the
server info changes.

`forceselect <n>` (SJK's) selects wheel entry `n` at once, by its
`forcePowers_t` number (18 Stasis, 19 Repulse, 20 Dash), as if
`forcenext` had stopped on it: a real power becomes the selection the user
command sends, a pseudo-slot the one `+useforce` uses, and the bar shows it. An
entry not on the player's wheel, or spectating, selects nothing
([`Selection::select`](../crates/sjk-client/src/selection.rs)). The quick wheel's
[Force page](#force-page) runs it.

## Illuminate

Illuminate is a toy, SJK's own and free for every player (Sol's request, 10/10/2026:
"not a force power, but just a toy"): a holocron that floats by the player's left
shoulder, turning slowly and bobbing, with a warm point light (300 units) that
lights the way in dark maps. No game server knows of it: its lit state travels
through the SJK hub as part of the player's look ([unlockables.md](unlockables.md)),
so other SJK players on the same server see it by that player (below); players on
stock clients do not. It is not a Force power and is on no Force wheel, bar or
list: the `toy_illuminate` command lights it or puts it out, from a bind
(Settings > Key bindings > Interaction > Toy: Illuminate, unbound until a key is
picked), from the quick wheel's [Toys page](#toys-page), or from the switch on the
Collection's Toys tab ([SJK UI](sjk-ui.md#collection)). Every toy is a
`toy_<name>` command ([sjk.md](sjk.md#names)).

It is always there, with no setting that enables it, and it is out at every start:
whether it is lit is the client's run only, not saved. The old Force wheel setting
`cg_illuminate` is gone; a profile that still holds it (0, off the wheel, or 1, on
it) has it dropped when it loads, so neither leaves the holocron lit or unusable.
The command was `force_illuminate` while it was a Force wheel entry; that name
still runs the toy, so a key bind or quick wheel choice saved under it works (the
bind is renamed to `toy_illuminate` when the profile loads, so the key-binding
editor lists it), but it is not listed in the console's completion or help.

The holocron appears and goes out over 0.3 seconds, trails a moving player by at
most 20 units and jumps with a teleport. It goes out while the player is dead,
spectating, following someone or at the intermission, and comes back at the player.
In first person only its light shows (the cube is drawn in mirrors, like the body).
Its light is added first each frame, so a full light list never drops it, but
`r_dynamiclight 0` puts it out with the others.

Other SJK players' holocrons show when their look says lit (with `cl_identity` on and
a hub; their claimed name must match the name the game shows in their slot, as for
badges). Each floats by that player's left shoulder, placed from their entity's
interpolated origin, eye height (crouching lowers it) and view yaw, with the same fade,
bob, turn and trailing as one's own (each slot's bob and turn a little out of step).
The cube always shows for another player, in first and third person, while the game
draws them; dead, hidden, cloaked or out of the snapshot, it goes out where it was.
All the cubes show, but only the four nearest the camera add their light, so the
frame's 32 lights stay for the weapons and sabers.

The cube, its two pictures (lit metal, and the emblem alone for the glowing
stage), its shader and its icon are bundled and mounted below all game
data, so a PK3 with the same paths replaces them (the picture is `gfx/sjk/toy_illuminate.png`). The same cube also has four
loot-box tiers (uncommon green, rare blue, legendary purple, mythical gold and
shiny): other pictures and shaders on the one model, with their icons and point-light
colours, bundled the same way. Sol generated the art; see
[assets/holocron](../crates/sjk-viewer/assets/holocron/README.md) and
[illuminate.rs](../crates/sjk-viewer/src/illuminate.rs).

## Cardboard box

The cardboard box is a toy, SJK's own and free for every player (Sol's idea,
16/10/2026): a shipping carton to hide in, Metal Gear style. `toy_box` puts it on or
takes it off, from a bind (Settings > Key bindings > Interaction > Toy: Cardboard box,
unbound until a key is picked), from the quick wheel's [Toys page](#toys-page) or from
the Collection's Toys tab ([SJK UI](sjk-ui.md#collection)). It is off at every start.
Putting it on plays its sound (one's own at once, in both ears; another's where they
stand when their look gains the box, but not for the boxes already in the looks in the
first 5 seconds on a server).

- **In the box** the player crouches: while it is on and the player is alive and in the
  match, every user command holds crouch (`upmove` -127, as a held `+movedown`, over a
  jump), so the server sees an ordinary crouching player and movement stays stock. Its
  music loops on the player, in both ears for one's own and placed on the player for
  others (on channel 14, its own, so it cuts no game sound).
- **Others see it** through the SJK hub: the box is a toy in the player's look
  (`toys: ["cardboard_box"]`, [unlockables.md](unlockables.md); `looks::TOYS`), so every
  SJK player on the server draws the carton over that player (their entity's
  interpolated origin, floor and view yaw) and hears the music, while the game draws
  them: dead, hidden, cloaked or out of the snapshot, neither shows. Players on stock
  clients see a crouching player. In first person one's own carton is drawn only in
  mirrors, like the body.
- **Spotted:** an SJK player who presses `+use` facing a boxed player (within 96
  units, 60 degrees either side of where they face; the nearest) lifts the box: the
  spotted sound plays where they stand and the box goes at once on the lifter's screen,
  and the hub relays a toy event (`POST /v1/toy`, `cardboard_box` `spotted`, the
  target's slot, [hub-chat.md](hub-chat.md#toy-events)). The boxed player's client
  takes the box off and plays the sound; every other SJK client hides it and plays the
  sound where the player stands. A lift seen here holds until that player's look drops
  the box, at most 15 seconds. The `+use` itself still reaches the server as always.
- **Taken off or lifted**, while its player is still drawn, the box tips over on its
  side (0.45 s, pivoting on its bottom edge, faster as it goes): away from the lifter
  when spotted, else the way the player faced. It lands with the drop sound, lies there
  5 seconds and shrinks away; at most 8 lie about at once. A box that goes with its
  player (dead, cloaked, out of sight) does not fall. One's own in first person shows
  only in mirrors until it has tipped 0.5 radians, out of the view inside it.
- **Dying in the box** plays its death sound: one's own in both ears, another's where
  they fell, on every SJK client that sees the death (the player's entity turning
  `EF_DEAD` while their look has the box). The box stays on until the respawn, so the
  others still know whose death it was, and is lost with the life.

The carton (an open-bottomed box of 46 x 46 x 44 units, drawn from both sides), its
picture, shader and icon are bundled and mounted below all game data, so a PK3 with
the same paths replaces them; they are drawn by
[cardboard_box_assets.py](../scripts/cardboard_box_assets.py), nothing taken from
another game ([assets/cardboard_box](../crates/sjk-viewer/assets/cardboard_box/README.md)).
The sounds are game paths that SJK does not ship, `sound/sjk/cardboard_box/music`,
`equip`, `spotted`, `drop` and `death` (`.wav`, else `.mp3`); where one is missing,
nothing plays. See
[cardboard_box.rs](../crates/sjk-viewer/src/cardboard_box.rs).

## Order 66

Order 66 is a meme toy, SJK's own and free for every player (Sol's idea, 17/10/2026):
every player who has it on dies at once, blown apart, when one of them executes the
order. `toy_order66` turns it on or off, from a bind (Settings > Key bindings >
Interaction > Toy: Order 66, unbound until a key is picked), from the quick wheel's
[Toys page](#toys-page) or from the Collection's Toys tab ([SJK UI](sjk-ui.md#collection)).
It is off at every start and stays on across deaths. `order66` (Interaction > Execute
Order 66, or the Toys page) gives the order; only a player whose own toy is on, alive in
the match on a server, can give it.

- **Who takes part:** the players whose toy is on. It is a toy in their look
  (`toys: ["order66"]`, [unlockables.md](unlockables.md); `looks::TOYS`), so every SJK
  player on the server knows who is in. Players on stock clients, and SJK players with
  the toy off, are never touched.
- **The order** travels through the SJK hub as a [toy event](hub-chat.md#toy-events):
  toy `order66`, event `execute_<moment>`, the moment it falls due in the game server's
  time (milliseconds), 3 seconds after it was given. Every client on a server shares
  that clock through its snapshots, so each counts down to the same moment however long
  its way to the hub took. A client drops an order heard more than half a second after
  its moment, one more than 10 seconds ahead, one whose sender's claim does not match
  the name the game shows, and one less than 30 seconds after the last order it took on
  that server: one order counts every 30 seconds, whoever sends it. The sender's own
  client refuses sooner ones ("Order 66 again in N s"). Without the hub
  (`cl_identity 0`) the order reaches only the sender, and the console says so.
- **The countdown:** for a player in the order, the centre of the screen says
  "EXECUTE ORDER 66", who gave it and the seconds left (3, 2, 1), with the thermal
  detonator's warning beep each second; the order itself plays
  `sound/sjk/order66/order.wav`, a game path SJK does not ship (nothing plays where it is
  missing).
- **The moment:** each client sends the stock `kill` command for its own player, so the
  server sees ordinary suicides (a point lost in FFA, as with `kill`). It does not when
  its toy is off by then, when the player is spectating, following someone, at the
  intermission or already dead, in a duel (a private one, or a duel or power duel
  game), or carrying a flag ("Spared: you are in a duel", "you carry the flag"). These
  are game rules, not server cases. The order is called off when its sender's look no
  longer has the toy.
- **The blast:** a player whose toy is on and who dies from 0.1 seconds before the
  moment to 2 seconds after blows up on every SJK client that sees it (their entity
  turning `EF_DEAD`; the own player and the one followed from the player state). The
  thermal detonator's explosion (`thermal/explosion`, its sound, flash and shake) goes
  off where they stand, and the body flies apart: the head, both arms, both legs and
  the trunk each become a piece, cut as [dismemberment](#dismemberment-and-disintegration)
  cuts a limb (the limb's cap on the piece, the stump's on what is left). The pieces
  are thrown out from the body and up (the head highest), spinning, fall under gravity,
  bounce off the world's walls and floors a little slower each time, trail smoke while
  they fly, lie still on a floor, and after 6 seconds sink away. Nothing is left
  standing: the body is hidden, weapon and hat too, until the player is alive again,
  and the corpse the server leaves at the respawn copies it, so it stays hidden as well.
- **Confetti:** where dismemberment is off (`cg_dismember 0`) the body bursts into
  confetti instead of pieces: six colours of paper scraps thrown up and falling
  (`effects/sjk/order66_confetti.efx`).
- **Limits:** pieces are made only for players within 3000 units of the camera and up
  to 48 at once (eight players); past that a player still blows up, without pieces. A
  piece is a limb mesh of the player's model: while the order counts down, the client
  puts up to six per nearby player's model in the limb pool, one upload a frame, so the
  blast itself uploads nothing (it may upload up to six a frame when the pool falls
  short). A [rig model](rig-models.md) has no surfaces to cut: it gets the explosion and
  stays whole. The cuts take a surface by its name or, failing that, its name and
  `_off`, as some models name what they leave off by default (`jedi_hf`'s head variants
  sit under `head_off`, its caps are `..._cap_torso_off`); a server's cut keeps stock's
  names.

The icons (the toy's red "66" badge, the order's blast) and the confetti scraps are
drawn by [order66_assets.py](../scripts/order66_assets.py), nothing taken from another
game or a film; the effect and its shader are written by hand
([assets/order66](../crates/sjk-viewer/assets/order66/README.md)). They are bundled and
mounted below all game data. See [order66.rs](../crates/sjk-viewer/src/order66.rs) and
`GpuState::blow_apart` in [dismember.rs](../crates/sjk-viewer/src/dismember.rs).

## Holstered saber hilts

Settings > Game offers **Holstered saber hilts**, **Holster server support**,
**Staff handoff point** and **Staff handoff sounds** in both SJK and classic
menus. `cg_drawHolsteredSaber` defaults on. Support (`cg_holsteredSaberSupport`)
defaults to `1`, automatic JA+ detection; `0` disables support and `2` forces
the documented JA+ ownership/plugin contract for a compatible server. Forcing
it on an unrelated server can misinterpret that server's ownership flags.

Stowed single hilts sit at the right hip; dual hilts use both hips. A single
staff goes across the back when JA+ plugin 9 is enabled, with a hip fallback
for models without a back tag. `cp_pluginDisable` is a disable mask: bit 9
(`512`) disables the staff back reach, and bit 13 (`8192`) disables holstered
hilts altogether. SJK preserves its existing default mask `1536`, so the
default staff placement remains at the hip. This feature does not enable
JA+ animation plugins or install their animation assets.

With plugin 9 enabled and usable JA+ reach animations installed, the staff
changes between back and hand at fraction `cg_holsteredStaffSwap` (default
`0.45`) of the actual torso frame range. It changes once per playing, including
the reverse put-away. Blades stay off before the drawing hand reaches the
hilt and retract during put-away. `cg_holsteredStaffSound` (default on) aligns
the registered hilt and blade-skin ignition/shutdown sounds with that reach;
missing or interrupted reaches have a bounded ignition fallback. Without the
reach clip, the normal weapon change controls the attachment.

Holsters are hidden on dead, disintegrating, mounted, mind-tricked or undrawn
players and on a first-person player's own/followed body. A cloaked player's
hilts require the viewer's Force Sight; your own cloaked hilts stay hidden.
A thrown primary hilt is never duplicated on the belt. Missing attachment
points affect their own hilt independently. The reference's Jedi NPC class
list is supported too.

Console placement overrides are archived, case-insensitive three-number
vectors, with finite values bounded to +/-3600; invalid vectors use defaults:

| Cvar | Default | Meaning |
| --- | --- | --- |
| `cg_holsteredSaberPos` | `0 3.5 0` | Hip offset: bolt right, forward, up; opposite side for the second hilt |
| `cg_holsteredSaberAng1` | `0 0 270` | Primary hip pitch, yaw, roll |
| `cg_holsteredSaberAng2` | `0 0 270` | Secondary hip pitch, yaw, roll |
| `cg_holsteredStaffPos` | `0 0 0` | Back offset: bolt forward, right, up |
| `cg_holsteredStaffAng` | `0 0 150` | Back pitch, yaw, roll |

These are client presentation options; they change no gameplay, wire format
or animation timing. See [rendering](rendering.md#holstered-saber-hilts) for
attachment ownership and [status](status.md#holstered-saber-hilts) for limits.

## Blade skins

A blade skin is an unlockable saber look ([unlockables.md](unlockables.md)): the Sun
(`saber_sun`), Storm (`saber_storm`), Void (`saber_void`), Frost (`saber_frost`) and
Prism (`saber_prism`) blades. Its look and sounds are a blade-skin file and
sound files in a pack the SJK hub delivers ([unlockables.md](unlockables.md#packs)),
drawn by the generic renderer described in
[rendering.md](rendering.md#saber-blade-skins); with no pack the skin is the stock
blade. `cg_saberSkin` (archived, default
empty) holds the skin the player wears by unlock id; empty or an unknown id is the
stock blade. It shows only while the player's own hub profile lists that unlock
(`Looks::own_saber_skin`, read twice a second) and its pack is loaded: with the
identity off, no hub, no answer yet, the unlock missing or no pack, the stock blade
shows, in the hand, in first person, thrown and on the Character page. Other players'
skins are their looks from the hub ([unlockables.md](unlockables.md#receiving)), copied
into the per-client table (`GpuState::sync_saber_skins`) only when the looks or the
loaded skins change.

`saberskin` lists the blade skins, owned (since when) or locked (how to get it), and
which is worn; `saberskin <id>` or `saberskin none` sets `cg_saberSkin` (a locked one
is kept and shows once unlocked). `unlockables` opens the Collection on its Shaders
tab (also the Profile page's See the collection), where owned skins are equipped and
unequipped and any is previewed on the player's own saber
([sjk-ui.md](sjk-ui.md#collection)).

A skin brings its own sounds, heard for each player wearing it, over the stock ones,
which keep playing (the skin's sound goes on a channel of its own, so that it does not
cut the stock one): its ignition for
`EV_SABER_UNHOLSTER`, its ignition or switching off once when the player draws the
saber from another weapon or puts a lit one away
([saber ignition sounds](#saber-ignition-and-retraction-sounds)), and for a general sound that is a stock `saberon*`/`saberoff*`
or any `.sab` `soundOn`/`soundOff` (the game's `G_Sound` gives these no owner, so
the player whose origin is nearest the sound, within 64 units, is taken), its hum
once beside the sabers' hum loops, and three swings for `EV_SABER_ATTACK` and the
animation `saberhup` cues, over a hilt's own `swingSound`s
([saber_sound_overrides.rs](../crates/sjk-client/src/saber_sound_overrides.rs)).
The viewer registers the skins' sounds with the gamestate's sound tables and passes
the per-client table every frame. With no skin worn every path is the stock one. A
thrown saber hums its owner's skin too: over the flying saber entity's `loopSound` (the
stock hum or the hilt's `soundLoop`), while the owner it names
(`genericenemyindex`) wears one. A skin's sounds are the game paths its file names, read from its pack below all game
data (`VirtualFileSystem::with_lower`), so a PK3 with the same paths replaces them. They
are registered when the gamestate's tables are built and again, at once, when a pack
arrives mid-session (`GameAudio::follow_blade_skins`, which also decodes a skin's
sounds afresh in case the new pack changed them); the per-client table then names the
new sets. The Sun's sounds have not been checked by ear in a game.

## Quick wheels

SJK's quick wheel ([quick_wheel.rs](../crates/sjk-viewer/src/quick_wheel.rs)):
hold the key, a ring of choices opens in the middle of the screen, move the mouse
towards one and let go to run it. The wheel has pages (Force, General, Toys and
Weather unless the player changed them); while it is open, the mouse wheel and buttons change
page. Letting go with the mouse still near the middle, or Escape, runs nothing;
opening the console, chat or a menu closes the wheel. While it is open the mouse
moves its pointer instead of the view and the HUD steps aside as `cg_drawHud 0`
would have it (status, weapon, timers, vote and kill lines, crosshair and the
game-data HUD; chat and nameplates stay, `QuickWheel::hides_hud`); the movement
keys keep working. A choice is the console command it stands for, so it does exactly
what typing it would.

### Binds and pages

| Bind | Opens |
| --- | --- |
| `+wheel`, Q by default | On the page shown last in this run (the first page, Force by default, in a new run) |
| `+wheel weather`, R by default | On Weather |
| `+wheel <page>` | On that page, named by its id or its name (case and spaces ignored), e.g. `+wheel general` or `+wheel toys` |

Q was `+wheel general` until 08/10/2026; a profile that saved that default has
it moved once to `+wheel` (`cl_wheelBindVersion`), and a page bound on Q after
that, or on another key, stays.

Q and R are in Settings > Key bindings > Other (Quick wheel, and Quick wheel,
Weather); their defaults are bound only where a profile has nothing on Q or R. A
page keeps its id (`force`, `general`, `toys`, `weather`, or what a new page's
first name made of it) when it is renamed, so a bind keeps working; a bind naming a page since
removed opens the page shown last. A second wheel key pressed while the wheel is
open changes it to that key's page.

While the wheel is open:

- **Scroll down or right click:** the next page; **scroll up or left click:** the
  previous one, wrapping round. Right is on and left is back, as the next and
  previous pages' names stand right and left of the ring; down is on, as down a
  list. A trackpad's small scroll steps add up to a notch (40 counts).
- The pointer stays where it is, so the same move picks on the new page; the
  new page turns in (0.17 s), its choices from a little way round the ring.
- The scroll and the left and right buttons do nothing else: no weapon change, no
  attack. Their releases are the wheel's too, while an attack held before the
  wheel opened is released as usual.

### Force page

The Force page (Sol's request, 10/10/2026;
[force_page.rs](../crates/sjk-viewer/src/quick_wheel/force_page.rs)) is a default
page, and since 10/10/2026 (Sol: "make the force wheel the first wheel to appear by
default") the first one, so a new run's bare `+wheel` opens on it. It is live: as
the wheel opens it holds the Force powers the player can use right now, read from
the latest snapshot as the [Force bar](#force-wheel) reads them
(`forcePowersKnown`, set for a power with a level above 0), with JoF JA+'s Dash,
Stasis and Repulse where the server grants them. Jump and the saber powers are
passive and left out, and so is [Illuminate](#illuminate), which is not a Force
power: it is on the [Toys page](#toys-page). Each choice shows the Force bar's picture and
name (Stasis borrows Jump's, merc mode's Lightning is the Flamethrower), and the
power selected now wears the gold dot.

| Power | Choosing it runs |
| --- | --- |
| Push, Pull, Speed, Sense | `forceselect <n>; force_throw` (`force_pull`, `force_speed`, `force_seeing`) |
| Heal, Protect, Absorb, Mind Trick, Team Heal | `forceselect <n>; force_heal` (`force_protect`, `force_absorb`, `force_distract`, `force_healother`) |
| Dark Rage, Team Energize | `forceselect <n>; force_rage` (`force_forcepowerother`) |
| Grip, Lightning, Drain, Stasis | `forceselect <n>` only |
| Dash, Repulse | `forceselect <n>; force_dash` (`force_repulse`) |

An instant power is used at once by its own command (the `genCmds_t` usercmd
values retail's F1-F12 binds send, JoF's server commands, SJK's toggle) and
selected, as the Force bar would leave it, so the Use Force key uses it next. A
held power cannot be held from a wheel that runs on letting go, so choosing Grip,
Lightning, Drain or Stasis only selects it for `+useforce`.

The order is fixed, so a power keeps its place round the ring while the build
does: the neutral powers (Push, Pull, Speed, Sense), the light side's (Heal,
Protect, Absorb, Mind Trick, Team Heal) or the dark side's (Grip, Lightning, Dark
Rage, Drain, Team Energize), each in the retail default keys' order (F1 to F12),
then Dash, Stasis and Repulse. A usual build is 9 powers; the page holds 12, its
pictures drawn at ten-twelfths of their size so the gap between them stays. Past
12 (an admin's every-power build on JA+, up to 17) the rest continue on a second page right after it, "Force 2", reached by the
same scroll or click; a bare `+wheel` that last showed Force 2 opens on Force when
the second page is gone. Shrinking 17 onto one ring would make every picture
half-size for a build almost nobody has, and an inner ring would put two choices
in one direction from the middle, which the wheel's pointer cannot tell apart.

Out of a game (or in a demo, where the choices could not act), spectating or
following, and with no usable power, the page stays in its place and says "No
Force powers here", so the pages keep their order and their dots.

### Toys page

The Toys page (Sol's request, 10/10/2026: "make a toy wheel too for the force
illuminate and future toy") is a default page named Toys, `+wheel toys`, for
[Illuminate](#illuminate) and future toys. It is an ordinary, editable page, not a
live one: its choices come from the [catalogue](#choices)'s Toys group, one entry
per toy, so a toy deleted from the page is added again in Settings > Quick wheel
(Add a choice, Toys), and a future toy is one catalogue entry. Its default choice
is Illuminate, which runs `toy_illuminate` (the toggle the bind uses) with the
Illuminate holocron's picture. Its gold dot marks a lit holocron (the client's own
state; no cvar holds it). Since 16/10/2026 the default page also has the
[Cardboard box](#cardboard-box) (`toy_box`, marked while the player is in it), and
since 17/10/2026 [Order 66](#order-66) (`toy_order66`, marked while it is on, then
Execute Order 66, `order66`); a wheel saved before keeps its Toys page as it was, and
gains them from Add a choice, Toys.

### GIF wheel

The GIF wheel (Sol's request, 12/10/2026: "maybe a wheel even of gifs / favorite
gifs") is the quick wheel opened by `+gifwheel` (Settings > Keys > Other > GIF wheel,
unbound by default) on the player's favourite GIFs from the SJK chat's
[GIF picker](hub-chat.md#the-gif-picker), newest first: ten a page, up to eight pages
("Favourite GIFs", then "Favourite GIFs 2" and on), each choice the GIF's preview
moving, drawn whole on a dark disc in its place on the ring, and "GIF 1", "GIF 2"... in
the middle (the number `sjkgif` takes). Letting go on one runs `sjkgif <id>`: the GIF
goes to the SJK chat ([From a bind](hub-chat.md#from-a-bind)). Its pages are made as it
opens ([`gif_pages`](../crates/sjk-viewer/src/quick_wheel.rs)), are not in
`wheel.json` and are not editable; a bare `+wheel` still opens on the page shown last
before it. Without favourites it says "No favourite GIFs yet". A GIF not loaded yet (or
with `cl_sjkChatGifs 0`) shows a disc with its name; it is sent all the same.

### Look

The ring is drawn in the SJK UI's look ([ring.rs](../crates/sjk-viewer/src/quick_wheel/ring.rs),
[SJK UI](sjk-ui.md)): the choices on a navy band (deep enough to read over any
scene) edged with holo line-work and a tick between choices; the highlighted
choice grown inside a gold ring with a gold glow behind it and a gold arc on the
band's rim; a gold arc on the middle disc pointing where the mouse points. In the
middle: the page's name, the highlighted choice's name in gold, and a dot for
each page (the page on show a gold bar). With more than one page, the previous
and next pages' names stand left and right of the ring on navy pills, with the
button that goes there ("Left click", "Right click"), and "Scroll to change page"
under it. A gold dot outside the band marks choices in effect. Text is in the
SJK UI's families (Rajdhani, Exo 2), which load in every menu style (Inter
until they have). Sizes follow the window's height.

The choices are round icons in the style of the game's neutral Force icons
(SJK's own, generated by Sol on two boards; [assets/wheel](../crates/sjk-viewer/assets/wheel/README.md)).
Every action has its own since 08/10/2026, and every custom command shares one,
braces round a dot (`{•}`), so several custom choices on a page look alike: the
middle names the highlighted one. A choice whose picture is missing shows its
name on a navy disc with a holo rim (two lines at most). A page holds at most 10
choices (ten icons fill the ring with a gap between each, the highlighted one
grown; the Force page holds 12, drawn smaller); an empty page says "Nothing here
yet".

### Sounds

The wheel plays the game's own menu sounds
([ui_cues.rs](../crates/sjk-viewer/src/audio/ui_cues.rs)), quieter than the
menus play them, since the wheel is used during matches:

| When | Sound | Volume | Where retail plays it |
| --- | --- | --- | --- |
| The page changes (scroll, a click, a second wheel key) | `sound/interface/sub_select` | 0.4 | The controls menu's sub-tabs |
| The highlight moves to another choice, from none too | `sound/interface/menuroam` | 0.5 | Every menu's focus sound; the Force power screen as powers are picked |
| The chosen choice runs, on letting go | `sound/interface/button1` | 0.5 | Every menu button (the SJK menus' click plays it at 0.9) |

Moving within a choice, or back to the middle, plays nothing; a page change
plays only its own sound, though the pointer then points at another page's
choice. Letting go on nothing, Escape and the wheel closing for the console, chat
or a menu play nothing: nothing ran. The sounds go through the effects volume
(`s_volume`, and silence when the window is muted while unfocused), each on its
own interface channel, so a quick run of moves cuts the last tick short instead
of piling up. A PK3 replacing one of these files (JoF's cosmetic mod replaces
`button1`) replaces it in the wheel and the menus alike.

`cg_wheelSounds` (archived, default 1) turns them off: Settings > Quick wheel's
Sound row, which the classic+ menus' editor has too, or Interface's "Quick
wheel sounds" row. The wheel reads it as it opens.

### Choices

The catalogue ([catalog.rs](../crates/sjk-viewer/src/quick_wheel/catalog.rs)),
by its groups in Settings:

| Group | Actions (console command) |
| --- | --- |
| Camera | Third person (`togglecamera`), Free camera (`freecam`), Camera control (`cameracontrol`) |
| Interface | Nameplates (`nameplates`, cycles), HUD on/off, Screenshot (`screenshotJPEG`), Game menu (`togglemenu`), First setup (`firstsetup`), Match timer, FPS counter, Speedometer, Lagometer (`toggle cg_...`), Console (`toggleconsole`), What's new (`changelog`) |
| Player | Cosmetics, AFK, Team menu (`teammenu`), Spectate (`team spectator`), Respawn (`kill`), Inspect player, Engage duel, Saber style (`saberAttackCycle`), Saber on/off (`sv_saberswitch`) |
| Emotes | Taunt, Bow, Meditate, Flourish, Gloat, Sync emote (`sjkemote sync`), Stop emote (`sjkemote stop`), then the installed emotes (`sjkemote <id>`, [hub-chat.md](hub-chat.md#emotes-in-the-client)) |
| Toys | Illuminate (`toy_illuminate`, marked while the holocron is lit), Cardboard box (`toy_box`, marked while the player is in it), Order 66 (`toy_order66`, marked while it is on), Execute Order 66 (`order66`) |
| Weather | The map's own weather, drizzle, rain, storm, snow (`r_weatherForce`), ground fog everywhere (`r_weatherFog` 1/2), clouds, weather on/off |
| Time of day | Day (`r_dayHour 12`), Night (`r_dayHour 0`), with the spare sun and moon icons |
| Votes, demos and server | Vote yes, Vote no, Record demo (`record`), Stop recording (`stoprecord`), Reconnect |

A custom choice is a name (up to 24 characters) and any console command (up to
200), several separated by `;`.

### Settings

In the SJK UI, Settings has a Quick wheel category (after HUD; see
[SJK UI](sjk-ui.md#quick-wheel)); Interface's "Quick wheel pages" row goes to it.
With the classic+ menus that row (Interface, and the tabbed settings' GAME tab)
opens the same editor on its own, in the SJK UI's look over the settings, Escape
returning to the row ([wheel_editor.rs](../crates/sjk-viewer/src/settings/wheel_editor.rs)).

- **Pages** (left): each page with how many choices it has ("Your powers" for
  the Force page), the one whose choices show in gold; Add a page (at most 8),
  Add the Force page (only while it has been removed: it goes back after General,
  else last), then Restore the default pages (Force, General, Toys, Weather; Enter
  twice, or two clicks); under them, Sound's "Wheel sounds" switch
  (`cg_wheelSounds`: Enter, Space or a click). Up and Down choose (the choices
  follow), Enter or Right goes to the page's choices, F2 or Rename names it,
  Shift (or Ctrl) with Up or Down moves it, Delete twice removes it (not the
  last one). A new page is named at once ("Page 4" if left blank); names are 20
  characters at most. The Force page is renamed, moved and removed the same way,
  but has no choices to go to: the middle column says they follow your Force
  powers and how they act, and the preview is an example (a light-side build).
- **Choices** (middle): the page's choices in ring order (the first at the top,
  then clockwise), each with its picture and group, then Add a choice. Enter
  changes a choice from the catalogue (a custom one opens its form), Shift with
  Up or Down moves it round the ring, Delete removes it, Left returns to the pages.
- **Right column:** a live preview of the page's ring, the focused choice
  highlighted, then the page's key and bind (`+wheel <id>`) or what the choice
  runs. Picking a choice shows the catalogue there instead (Up and Down, Enter,
  Escape; actions already on the page say so), Custom command last; a custom
  choice's form has its name and command fields, Save and Cancel.
- The pointer: hovering a row focuses it (a page row without showing that page,
  which a click does), the focused row's small controls act (Rename, up, down,
  remove), a click on a choice or Add a choice opens the catalogue, a click on a
  line of it picks.

Every change is saved at once.

### Kept in wheel.json

The pages are kept in `wheel.json` in the profile folder
([pages.rs](../crates/sjk-viewer/src/quick_wheel/pages.rs)), written (whole, then
moved over the old file) when the player changes them; without the file the
wheel has Force, General, Toys and Weather, and Restore the default pages removes it so
a later version's defaults reach the profile. A JSON file rather than cvars: a page
is a list, a custom choice a free console command (quotes, semicolons) that a
`config.cfg` line would have to escape and a cvar's length would cap.

```json
{ "version": 3, "pages": [
  { "id": "force", "name": "Force", "kind": "force" },
  { "id": "general", "name": "General", "choices": ["third_person", "nameplates", "hud"] },
  { "id": "toys", "name": "Toys", "choices": ["illuminate"] },
  { "id": "duels", "name": "Duels", "choices": ["duel", {"name": "Ready", "command": "ready"}] }
] }
```

A choice is an action's id or a `{name, command}` object; the Force page is
`"kind": "force"`, without choices (one at most: a second is left out). Reading
keeps what it can: unknown action ids, unnamed pages and incomplete custom choices
are left out, pages past 8 and choices past 10 cut, a repeated id numbered; a
file that is not JSON, or has no page, reads as the defaults (and is kept until
the next change). Version 2 brought the Force page: a file of version 1 (or none)
gets it once as it is read, after the page whose id is `general` (else last), and
is written back, so a player who removes the page afterwards keeps it removed; a
wheel of 8 pages is left as it is (Settings' Add the Force page, after removing a
page, or Restore, brings it).

Version 3 (10/10/2026) made the Force page the first and added the Toys page, an
ordinary page of the catalogue's Toys group. A file older than 3 whose pages are
exactly what version 2 had as its defaults (General, Force and Weather, nothing
renamed or changed in them) was never customised: it is removed and the wheel has
the new defaults, in the new order. Any other file keeps its pages and their order
and gains the Toys page once, after its Force page (else last), and is written back
as version 3; a Toys page removed afterwards stays removed, a wheel of 8 pages is
left alone (the file is not rewritten) and a page of the player's own called Toys
keeps its id, the new one taking `toys2`.

## Configuration and content

The default writable client folder is `GameData/SJK/`, under the selected game
installation (see [storage.rs](../crates/sjk-viewer/src/platform/storage.rs)). It
is independent of the executable's location and working directory. The client creates it automatically. Important files include:

| File or folder | Contents |
| --- | --- |
| `config.cfg` | Settings and key bindings |
| `marks.txt` | Marked map positions, views and notes |
| `favorites.json`, `chat-friends.txt` | Favorite servers and friend names |
| `jakey` | Persistent client identity key |
| `hud.json` | Optional custom HUD |
| `wheel.json` | The quick wheel's pages, once changed ([Quick wheels](#kept-in-wheeljson)) |
| `screenshots/`, `demos/` | Screenshots and recordings |
| `chatlogs/`, `qconsole.log` | Logs when enabled |

`config.cfg` and the scripts `exec` runs are read as UTF-8, or, when a file is not
valid UTF-8, one character per byte (Latin-1), as Jedi Academy and EternalJK save
names and chat binds; a leading byte-order mark (Notepad) is dropped
([config.rs](../crates/sjk-shell/src/config.rs) `decode_config_text`). So another
client's config runs with `exec`, and a `jampconfig.cfg` copied over `config.cfg`
loads: its `unbindall` and `unbind` lines are accepted and bindings on keys SJK has
no name for are dropped. Any other line still stops the load (and saving, so the
file is not overwritten). The [Import page](#importing-from-another-client) is the
way that takes only the name, model, field of view and bindings.

If `GameData/SJK/` cannot be written, the client uses its per-user folder:
`%APPDATA%\SJK\` on Windows, `~/Library/Application Support/SJK/` on macOS, or
`$XDG_CONFIG_HOME/SJK/` (otherwise `~/.config/SJK/`) elsewhere. The chosen folder
is shared by all profile consumers for the entire session. Startup reports it;
the console's `path` command also lists it. The fallback uses the profile in
that per-user folder; the per-user and portable profiles are not synced.
See [platform.rs](../crates/sjk-viewer/src/platform.rs) and
[storage.rs](../crates/sjk-viewer/src/platform/storage.rs).
Edit settings through the client, or edit the file while the client is stopped
so autosaving cannot overwrite your changes.

Autosaving writes `config.cfg` only when an archived cvar or a binding changed:
key presses, `+`/`-` buttons and unarchived cvars never touch the file. A change
is saved once the archived state has held still for 2 seconds
(`CONFIG_SAVE_DELAY`), so a dragged slider or a script setting many cvars writes
once. The text is built on the main thread and written, through a temporary file
and a rename, by one background thread that takes the saves in order. The file
is written at once, on the main thread, when the client exits (every exit drops
the console), on `writeconfig`, and before `exec config.cfg` reads it back. Until
10/10/2026 every command frame, so every bound key press, rewrote the whole file
on the main thread. See [config_saver.rs](../crates/sjk-shell/src/config_saver.rs)
and `Shell::autosave` in [shell.rs](../crates/sjk-shell/src/shell.rs).

`writeconfig <name>` (or `write <name>`) saves the archived cvars and bindings to
`<name>.cfg` beside `config.cfg`, where `exec <name>` finds it, as OpenJK's
`Com_WriteConfig_f` does: `.cfg` is added to a name without an extension, any other
extension is refused, and the name stays inside that folder (no absolute path, no
`..`). Until 16/10/2026 a name was a path from the process's working folder, without
`.cfg`: on Linux the file went to the home folder and `exec` could not read it back
(a player's bug report). See `config_file_path` in
[shell_builtins.rs](../crates/sjk-shell/src/shell_builtins.rs).

`com_maxfps` defaults to `-1` (AUTO in Settings > Display); `0` is uncapped. The
old default, 1000, saved in every existing profile, is reset to AUTO once on
first launch (marker `com_maxfpsDefaultVersion`); a cap chosen afterwards is
kept. The default is not saved to the configuration. On the slider
AUTO is the rail's left end: arrows step AUTO, 0, 25, 50 and so on, and typing
`-1` selects it. An uncapped
client saturates the GPU; screen recorders and streamers sharing it then skip
frames (OBS reported 83% skipped for encoding lag against an uncapped client at
4K). See [runtime_settings.rs](../crates/sjk-viewer/src/runtime_settings.rs).

AUTO caps frames at stock's 125 unless `com_maxfpsMonitor` (archived, default 0;
Settings > Display > Detect refresh rate) is 1: then AUTO is the refresh rate of
the monitor holding the window, rounded to whole hertz and re-read once a second,
or 125 when the monitor reports none. At 0 the monitor's rate is never read, so
AUTO caps as it does for a monitor that reports none. Sol asked for the detection
to be off by default (08/10/2026). The setting changes only what AUTO means: a
`com_maxfps` the player set stays as set. It is new, so no profile has saved it
and its default reaches existing profiles without a migration.

A window without focus has caps of its own: `com_maxfpsUnfocused` (SJK's default
30; EternalJK's 0 keeps the normal cap) and `com_maxfpsMinimized` (50), where 0
uses `com_maxfps`. They replace the normal cap while they apply, minimized first,
so an alt-tabbed client does not render a 4K scene at full rate.

`snd_mute_losefocus` (archived, 1, EternalJK's name and default; Settings > Sound
> Mute in background) silences every sound, music included, while the window is
unfocused or minimized. EternalJK pauses its sound device; SJK sets the effects
and music gains to zero and keeps mixing, so music and loops carry on silently and
come back where they are. The change is applied from the focus and minimize
events themselves, since a minimized window may draw no frame; see
[console_window_options.rs](../crates/sjk-viewer/src/console_window_options.rs).

The game never waits for good on a sound device that stopped. Windows ends an
output stream when its device goes away: a headset or Bluetooth speaker that
switches itself off after a while of silence (which a muted background window
plays), a monitor's speakers while the display sleeps, a change of the device's
format. The stream's callback then no longer empties the mixer's queue of 8,192
commands, and the render thread used to wait for room in it without end, so the
game stopped answering seconds later and Alt+Tab never brought it back. A full
queue now waits at most 250 ms, once; then commands are dropped (every frame sends
its state again) until the queue drains, and the log says `audio output stopped
taking sound`. The game goes on silently; `snd_restart` opens the output again, on
the device that is then the default. The decode worker keeps sounds in order and
waits for room as before, but stops when the output is dropped, so `snd_restart` and
quitting do not wait on it. See [audio_feed.rs](../crates/sjk-viewer/src/audio_feed.rs).

The Video tab's Display mode row offers Windowed, Borderless fullscreen and,
where the windowing system supports it, Exclusive fullscreen (Wayland does not).
Stock `r_fullscreen` keeps its meaning, fullscreen on or off, and Alt+Enter still
toggles it. `r_exclusiveFullscreen` chooses the kind: 0 (default) is a borderless
window at the desktop size, 1 switches the monitor to the `r_resolution` video
mode. Stock JA's fullscreen is always the exclusive kind. Choosing Windowed leaves `r_exclusiveFullscreen` alone, so Alt+Enter
returns to the last fullscreen kind. Exclusive fullscreen without a monitor mode
of that size falls back to borderless.

Exclusive fullscreen is paused while the window lacks focus: winit makes an
exclusive window topmost and keeps the monitor's video mode, which left the game
over the desktop after Alt+Tab or Win+D. On focus loss the window goes windowed
(restoring the desktop mode and the window's earlier placement) and `r_resolution`
is not re-requested; on focus gain, such as restoring from the taskbar, it enters
the exclusive mode again. A loss within 0.75 s of a display change is ignored so
a focus flicker caused by the switch cannot loop. Borderless and windowed are
unaffected. The logic is in `apply_display` and `display_focus_changed`
([runtime_settings.rs](../crates/sjk-viewer/src/runtime_settings.rs)) and
`suspended_after_focus` ([display.rs](../crates/sjk-viewer/src/settings/display.rs));
the unit tests cover the decision, not Windows' behaviour, which is unverified
in the real client (a player reported Alt+Tab and Win+D failing in exclusive mode).

Enter or a click on the Resolution row opens a list of the monitor's video-mode
sizes, grouped by aspect ratio with the monitor's own first, then the other
groups by how close their shape is to it (on a 16:9 monitor 16:10, then 3:2,
4:3, 5:4; since 10/10/2026, before which they ran narrow to wide), largest first
in each, and the size in use highlighted. Windowed and borderless also list the classic presets that fit the
monitor and a custom `r_resolution`; borderless fullscreen always fills the
desktop and uses the size only when windowed. Left and Right step the row within
its aspect-ratio group. See [display.rs](../crates/sjk-viewer/src/settings/display.rs)
and [resolution.rs](../crates/sjk-viewer/src/settings/resolution.rs).

`fs_game`, `fs_basegame` and `fs_homepath` configure content search paths; restart
the client after changing them. Search precedence and shader protection are owned
by [asset_search_paths.rs](../crates/sjk-viewer/src/asset_search_paths.rs).

Downloaded content is stored separately from the retail installation and config.
The default is `%LOCALAPPDATA%\SJK\downloads\base` on Windows,
`~/Library/Application Support/SJK/downloads/base` on macOS, and elsewhere
`$XDG_DATA_HOME/SJK/downloads/base`, falling back to
`~/.local/share/SJK/downloads/base`. `SJK_DOWNLOAD_HOME` overrides the download
root (the implementation appends `base`). See
[download_store.rs](../crates/sjk-viewer/src/download_store.rs).

Server reference lists use OpenJK's positional common-prefix rule: extra pak
names or checksums without a counterpart are ignored, including when one list
is empty. A name token without a `/` is the rest of the name before it (stock
servers write every name as `game/basename` and separate them with spaces), so
`base/{JoF}Sentinel Nexus` keeps its own checksum instead of shifting every later
name onto its neighbour's, as JoF's lists did in March 2026 (21 names for 20
checksums). Download comparison and session cache selection share
[the compatibility parser](../crates/sjk-client/src/referenced_paks.rs), so a
connection cannot pass one check only to fail the other on list length.
Paired checksums, download paths and file contents remain validated; advertised
BSP checksums are still enforced. This does not change pure-server proofs.

References are not a mandatory client install manifest. When the server sets
`sv_allowDownload 0`, or the client sets `cl_allowDownload 0`, UDP transfers are
skipped and joining proceeds with available content. Missing directory names,
unsafe download names and retail packs never produce a download request.
Pak names may hold spaces, braces, brackets, apostrophes and `!,&~` (JoF's
`{JoF}'s-GWing`); quotes, separators, `;`, `%`, `$`, wildcards and control bytes
are refused. The request is sent quoted (`download "base/{JoF}Sentinel Nexus.pk3"`),
since a stock server reads the name as `Cmd_Argv(1)`. A pak the server refuses
(it answers with a reason) or that fails verification (size caps, checksum, a
broken archive) is skipped with a `session content: skipped download` log line and
the join goes on with the others; before 12/10/2026 any one failure ended the
connection. 30 s without a block still ends it, as the server has stopped
answering ([download.rs](../crates/sjk-client/src/download.rs)).
Unavailable referenced archives do not abort world mounting; only locally
available checksum matches are selected from the cache. The actual map must
still exist and match its advertised BSP checksum. HTTP downloading remains
unsupported, and this policy does not disable pure-server admission checks.

A server that lists no paks still names its map. When no installed content has
`maps/<mapname>.bsp`, the newest download holding it is mounted (with the policy
below), so a map downloaded once stays playable on servers that stopped listing
their paks; JoF's servers send `sv_allowdownload 0` and an empty list since at least
October 2026. A map that is installed is never replaced this way.

### What a download may bring

A downloaded pak only adds content, and only to the sessions whose server
references it (or whose map it alone has). The archive is kept exactly as
downloaded, because its checksums are what servers compare (pure servers
included); what the policy refuses is hidden when it is mounted
(`VirtualFileSystem::mount_pk3_filtered`), so nothing reads, lists or `exec`s it:

| Hidden | Examples |
| --- | --- |
| Programs and host scripts | `cgamex86.dll`, `.so`, `.exe`, `.bat`, `.ps1`, `.js`, `.qvm` |
| Console scripts | every `.cfg` (`autoexec.cfg`, `mpdefault.cfg`) but the model tables `models/**/animation.cfg`, `animevents.cfg` and `sounds.cfg`; ICARUS `.ibi` |
| Menus | everything under `ui/` and any `.menu` (menus run commands) |
| Archives | a `.pk3`, `.zip`, `.7z` or `.rar` inside the pak |
| Replacements | any file the game data or an installed pack already has (sounds, textures, models, levelshots, sabers) |

Two kinds are exceptions to the replacement rule. A map (`.bsp`) may replace an
installed one of the same name, as the server runs it (JoF's FFA3 Night ships as
`maps/mp/ffa3.bsp`); the session still checks it against the server's map
checksum when one is sent. Shader scripts always mount, and their definitions are
screened one by one: the shader catalogue is loaded from the game and installed
packs as before, then a download's scripts may add new shaders but never redefine
one those define, nor one named after an installed texture (an implicit shader
every map using that texture would take up). Each download logs what it lost
(`session content: download ... stripped of 1 scripts, 2 replacements (...)` and
`shader authority: download ... may not redefine N shaders`). See
[download_policy.rs](../crates/sjk-viewer/src/download_policy.rs),
[session_content.rs](../crates/sjk-viewer/src/session_content.rs) and
[shader_authority.rs](../crates/sjk-viewer/src/shader_authority.rs).

A map that changes retail textures for its own look loses those changes: the
installed texture is used. Installed packs (in `GameData`) are the player's own
choice and are not screened.

## Key names and binds

`bind`, the controls editor and the config name keys as stock JA does on Windows:
by what the active keyboard layout prints on them. On a French AZERTY keyboard
`bind w +forward` is the key labelled W, and the key left of W is named `<`.
Character keys take the layout's unshifted character, including non-ASCII ones
such as `é`, `ù` or `²`; a dead key is named by its accent, so AZERTY's `^` key is
`^`. The digit row keeps `0`-`9` on every layout, keys without a character
(arrows, F-keys, keypad, modifiers) keep their stock names, and letter keys of
non-Latin layouts such as Cyrillic keep their US letter, so the default binds
still reach a key there. A release runs the binds of the name its press had.
Menu navigation keys (W/A/S/D beside the arrows) stay positional. See
[keys.rs](../crates/sjk-viewer/src/input/keys.rs) and
[key_names.rs](../crates/sjk-shell/src/key_names.rs).

Key names are shown in capitals, as retail's controls menu shows them
(`BindingFromName` upper-cases with `Q_strupr`): the key binding editor, the
vote prompt and the console's `bind`, `unbind` and `bindlist` output read `W`,
`SPACE`, `MOUSE1`. Only ASCII letters change, so layout names such as `é` keep
their character. `config.cfg` keeps the saved spelling (`bind "w" ...`), and
`bind` accepts names in any case. See
[key_names.rs](../crates/sjk-shell/src/key_names.rs).

## Colour codes

Text draws `^0` to `^9` as OpenJK's ten-entry colour table does: `^0`–`^7` are
the retail colours, `^8` is orange and `^9` grey (retail wrapped them onto black
and red). The table is `quake_color` in [text.rs](../crates/sjk-viewer/src/text.rs).

## Console styles

`con_style` (Settings, TEXT tab, "Console style") picks how the console looks:

| Value | Look |
| --- | --- |
| `auto` (default) | `sjk` while the menus are the SJK UI (`ui_menuStyle sjk`), else `classic` |
| `sjk` | The SJK UI's deck ([SJK UI console](#sjk-ui-console)) |
| `classic` | EternalJK's console (`cl_console.cpp`, `cl_keys.cpp`) |

A mistyped value acts as `auto`, as do `horizon` and `dock`, two SJK UI designs
removed on 08/10/2026 once Sol chose the deck, and the retired `modern` (or `0`),
which a profile's start also resets ([Menu style](#menu-style)). Every profile had saved the old default
`classic`, so a saved `classic` moves once to `auto` (`con_styleDefaultVersion`,
as `cg_scoreboardStyle` did): it still gives the classic console unless the
menus are the SJK UI; a `classic` chosen afterwards stays. Input editing, mouse
selection, completion, the `]cmd` echo, history (Up and Down) and the F3 browser
work the same in both looks, which share EternalJK's keys below. See
[console_classic.rs](../crates/sjk-viewer/src/console_classic.rs),
[console_sjk.rs](../crates/sjk-viewer/src/console_sjk.rs),
[console_backdrop.rs](../crates/sjk-viewer/src/console_backdrop.rs) and
[console_options.rs](../crates/sjk-viewer/src/console_options.rs).

### Classic console

Text sits on a grid of character cells drawn with the console font, without the
glyph shadow other UI text has. The console font is JetBrains Mono, a bundled
vector font rasterized once at startup like Inter
([console_font.rs](../crates/sjk-viewer/src/text/console_font.rs)), in place of
retail's `gfx/2d/charsgrid_med` bitmap, whose 8 by 16 texel glyphs looked heavy
and blocky magnified at 1440p and 4K (SJK draws no bitmap fonts:
[sjk.md](sjk.md#fonts)). It keeps the character set's cell: every character
advances half the line height, and the font's ascent and descent are centred in
the cell. The retail character set had no Windows-1252 typographic characters
(`€`, `’`, `‘`, `…`, bytes 0x80..=0x9E); the classic console still gives them no
cell, like colour codes, so rows close up as in EternalJK, while the text keeps
them. The classic console loads the font whether `ui_gameFont` is on or not; if it
cannot be loaded the cells use Inter.
A cell is 8 by 16 pixels at 1080 lines and `con_scale 1`, grows with the window
height like the rest of the UI (with the console's 0.75 floor) and is rounded to
whole pixels. EternalJK's cells are 8 by 16 screen pixels times `con_scale`, so
`con_scale 0.5` at 2160 lines matches EternalJK at 4K with its default scale. A
row holds the screen width in cells minus two, and column `c` is drawn `c + 1`
cells from the left edge, as `Con_DrawSolidConsole` does.

The background is the game's `console` shader, read from the shader scripts and
the PK3s like any other, so a pack that overrides it wins as it does in
EternalJK: the retail one scrolls a star field (alpha blended) and adds the
pulsing Jedi Academy logo; a cosmetic pack such as JoF's draws an opaque picture
with scrolling stars over it. Each stage keeps its `blendFunc`, its `tcMod`
(`scroll`, `scale`, `rotate`, `transform`, `stretch`) and its `rgbGen` and
`alphaGen` (`wave`, `const`, `vertex`); other generators draw at full strength,
and an `animMap` shows its first frame. Below full height `alphaGen vertex` is
`con_opacity`, so the retail stars fade with it while an opaque first stage stays
opaque; at full height it is 1. Without the shader the console is a dark navy
panel. The background reaches `480 × fraction − 2` units of the 640×480 virtual
screen, with a bar two units tall in EternalJK's `console_color` (0.509, 0.609,
0.847) under it. `con_ratioFix` (1, archived, EternalJK's name and meaning) shows
the middle of the picture (`t` from `1 − k` to `k`, `k` = 4:3 over the screen's
aspect) when the console is half the screen or less on a wide screen, instead of
squashing the whole picture; set 0 for custom backgrounds made for a squashed
fit.

The classic console has its own 2D layer, drawn after every other 2D element,
text included, and its own text last, so menu, chat, HUD and frame-rate text
never shows through an opaque console.

| Key | Classic console |
| --- | --- |
| Console key (the key under Escape, or `cl_consoleKeys` with `cl_consoleUseScanCode 0`) | Opens to `con_height` (0.5) |
| Ctrl + console key | Opens full screen |
| Shift + console key | Opens a quarter of the screen |
| Shift+Escape | Opens to `con_height`; Escape closes |
| Page Up, Page Down, mouse wheel | Scroll back or forward two rows; ten with Ctrl |
| Ctrl+Home, Ctrl+End | Oldest row, newest row |
| Up, Down, keypad 8 and 2 (Num Lock off), Shift+wheel, Ctrl+P, Ctrl+N | Command history (32 commands) |
| Ctrl+L | Clear the scrollback |
| Insert | Toggle overstrike: typing replaces the character after the cursor |

A normal console opening follows `con_height` live, so changing it while the
console is open resizes it in both styles. Ctrl and Shift openings keep their
full-screen and quarter-screen overrides; a normal opening or Shift+Escape
clears that override. A `toggleconsole` bind reuses the last opening mode,
following the current `con_height` when that mode was normal. The console
slides at `scr_conspeed` screens per second and is drawn from its first pixel.
A disconnected client whose menu
is closed shows the console full screen, as `Con_DrawConsole` does; the map
viewer without a menu does not.

Scrollback rows are drawn from three cells above the console's bottom edge up to
the top of the screen. Lines start white (`^7`),
error lines light red, and a colour code carries on into the rows a line wraps
onto. Words wrap as `CL_ConsolePrint` wraps them: a word that fits on a row but
not in what is left of the current one, or would end exactly at its edge, starts
the next row; a word longer than a row breaks at the edge (EternalJK breaks such
a word early, at an odd point); colour codes take no room. With `con_timestamps 2`
(EternalJK's layout) every row, wrapped ones included, starts with the local time
its line was written in grey, `HH:MM:SS` and a space, and the text wraps in the
columns after it; `1` also stamps the notify lines and `0` leaves no stamp column.
Scrolled back, a row of `^` every four columns in the bar colour sits under the
rows, and new output does not move the view.

The input row is two cells above the bottom edge: the local time in green in
columns 1 to 8, `]` in column 10, then the input as typed, colour codes shown and
applied as EternalJK draws them (`^1a` shows `^1a` in red; it was all white before),
and a cursor that blinks every 256 ms: a bar on the cell's
bottom rows or, in overstrike mode, a block, both solid shapes in the retail
character set's cursor proportions (Inter cells use `_` and a box). The
input scrolls sideways to keep the cursor on screen. In the bottom-right corner
the version line ends one cell from the edge, two and a half rows up, and the
local day, date and time sit under it at the edge, both in the bar colour. Where
EternalJK prints `asctime` on a 12-hour clock (`Sun Oct  4 10:52:10 PM`), SJK writes
the date day first on the 24-hour clock (`Sun 04/10/2026 22:52:10`; see
[SJK conventions](sjk.md#dates-and-times)).

Closed, the console can draw notify lines, the console feed, while a game, a demo
or a map walk runs and no menu has focus: of the last `con_notifylines` rows, those
written within `con_notifytime` seconds and not quiet (chat), from the top edge of
the screen, one cell plus `cl_conXOffset` pixels from the left. The feed is off by
default and `con_drawNotify 1` (archived; Settings > Interface > Console feed, the
TEXT tab in the tabbed screen) turns it on (Sol's request, 10/10/2026). What it
shows: server `print`s (joins, team changes, votes, command replies, server
announcements), the one-line obituary of each kill, the client's own messages
(errors, demo and download notices, cvar replies). Chat never goes through it: it
keeps its chat box (`cg_drawChat`), and centre prints, the kill feed
(`cg_killfeed`) and the vote panel are drawn by the HUD, so all of them
stay with the feed off. The open console still shows every line. Dragging the mouse selects
scrollback text and Ctrl+C copies it; the selection does not include the time
column.

### SJK UI console

`sjk`, the deck, is the classic console with another skin
([console_sjk.rs](../crates/sjk-viewer/src/console_sjk.rs)): the same grid of
JetBrains Mono cells, word wrap, scrollback, history, keys, overstrike, mouse
selection and notify lines, drawn in the SJK UI's colours
([sjk-ui.md](sjk-ui.md#colours)) without the `console` shader. Rows are a fifth
further apart than the classic console's (19 pixels at 1080 lines), the cells
centred in them; text starts in the SJK UI's text colour, error lines in a soft
red, time stamps quiet, and colour codes use its legible palette. The input row
starts with a gold `›`, the caret is a thin gold bar (a gold block in overstrike
mode), and while the caret ends a unique command or cvar name being typed the
rest of the name shows faintly after it, as Tab would complete it. Scrolled back,
the bottom row says how many newer rows there are; a thin bar right of the rows
shows the part on screen, gold while scrolled back. Labels are in the SJK UI's
families (Rajdhani and Exo 2), drawn on the console's own layer so nothing shows
through; they load with the look, and the console font stands in until they
have. After Ctrl+C the header says how many characters were copied. Below full
height the panel follows `con_opacity`. The notify lines stay the classic ones.

The deck is a full-width navy panel (the world shows faintly through): a header
with "Console" underlined gold, the version, the date and the time; the input
on a band with the SJK UI's gold bar; key hints (Tab, Up and Down, Page Up and
Down, F3, Ctrl+C); a gold rail fading into a holo line along its bottom edge.

A click on the F3 hint opens the command browser, which is drawn in the SJK
UI's look with the deck (see below). Verified by unit tests (the looks' names,
`auto` and the retired names, the saved value's move, the grid, caret and ghost
completion) and by off-screen frames of every look over mp/duel6 at 1080 lines
and 4K (`world_shot::tests::duel6_console_styles`); not yet used in a running
client.

Local time comes from the operating system's time zone rules, daylight saving
included ([local_time.rs](../crates/sjk-shell/src/local_time.rs)); the scrollback's
stamps and the log file's `[HH:MM:SS]` are local time too. Differences from EternalJK besides those above: the input never runs past
the screen's right edge (EternalJK's can).

## Console socket

`cl_consoleSocket <port>` lets a program on the same computer read the console
and run commands, as JoF EJK's setting of the same name does; apps written for
it (Sol's Archive, a chat quote bot) work unchanged. It is off (`0`) by default.
`cl_consoleSocketPassword` is required: the socket does not open without one and
starts as soon as one is set, so a web page that writes to a loopback port cannot
run console commands (JoF EJK allowed an empty password). Both cvars are archived,
the password in plain text like `password`. The `consolesocket` command explains
the feature and shows its state.

```
cl_consoleSocketPassword mySecret123
cl_consoleSocket 29071
```

The protocol is lines of text on `127.0.0.1:<port>`, which only this computer can
reach. The app sends the password as its first line (anything else closes the
connection and nothing runs) and is answered `cl_consoleSocket: authenticated`.
Then every console line the app has not seen is streamed to it as it prints, ended
by `
`, with `^n` colour codes; lines printed before it authenticated are not
sent. Each line the app sends runs as a console command, as if typed (`say hi`
chats). Text uses Windows-1252 bytes, like chat on the wire
([legacy_text.rs](../crates/sjk-protocol/src/legacy_text.rs)), or Windows-1251
for Russian ([player text](networking.md#player-text)); other text is sent as
UTF-8.

Chat lines carry the stock `0x19` separator before the colon (`Name^7<0x19>: hi`),
which is how an app tells chat from other output; the console shows the line
without it, so the socket is given the server's raw text instead
(`push_chat_line` in [console_socket.rs](../crates/sjk-viewer/src/console_socket.rs)).

Up to 4 apps connect at once, a command line is at most 1023 characters (a longer
one is discarded whole), and an app that stops reading is dropped after 256 KB of
unread output; the game never waits for it. The listener, framing and limits are
[console_socket.rs](../crates/sjk-shell/src/console_socket.rs) in `sjk-shell`; the
viewer feeds it once per frame from `run_console_command_buffer`. Covered by unit
tests over real loopback connections (authentication, limits, chat separator,
command execution); not yet run against a game session or Sol's Archive itself.

## Useful console commands

The console key opens and closes either console style and never types its
character, as in EternalJK (`IN_TranslateSDLToJKKey` and `IN_IsConsoleKey` turn
it into `A_CONSOLE`). By default (`cl_consoleUseScanCode 1`, EternalJK's) it is
the physical key under Escape on every layout, so Hungarian, where that key types
`0`, has one; on layouts where it types `^` (German) it needs Shift, leaving `^`
for colour codes, and `cl_consoleShiftRequirement` can require Shift elsewhere.
Profiles that saved the earlier default 0 are moved to 1 once
(`cl_consoleKeyDefaultVersion`). With `cl_consoleUseScanCode 0` the console key is
a `cl_consoleKeys` character such as `~` or `²` instead. Holding it toggles once. Escape and Shift+Escape close
the console too, and so does a key bound to `toggleconsole` that prints nothing;
a printable key bound to it types while the console is open
([console_keyboard.rs](../crates/sjk-viewer/src/console_keyboard.rs)). The chat
composer still types `~` when the console is closed. On layouts with dead keys
(`^` on French AZERTY and German QWERTZ, `'` on US International), the console
line and the chat draft show a dead key at the caret at once and replace it with
what the platform composes on the next key, so typing reads as on a layout without
dead keys: `^` then `1` gives the colour code `^1`, `^` then `e` gives `ê`, and `^`
then Space gives `^`. Backspace removes only the shown `^`; Enter sends it. A dead
`^` that xkb composes into a superscript digit (`¹`) becomes `^1` again, since a
caret before a digit is a colour code. See
[dead_key.rs](../crates/sjk-viewer/src/input/dead_key.rs). The browser's search
field takes a dead `^` as a literal colour prefix. Opening or closing the console
clears the window's pending accent composition, so a dead toggle key such as `^`
on a German layout does not combine with the next letter.

Alt codes type as in other Windows programs: hold Alt, type a number on the
numeric keypad and release Alt. A number starting with 0 is a Windows-1252 code
(`Alt+0248` gives `ø`), any other a code page 437 one (`Alt+21` gives `§`,
`Alt+130` gives `é`), counted modulo 256; a code naming a control character types
nothing. They work wherever text is typed (the console line and browser search,
the chat draft, menu fields) and not during play, where the keypad keeps its
bindings; the keypad digits of a code reach no field, so the classic console's
keypad 8/2 do not walk the history. AltGr and Ctrl+Alt do not compose. Windows
composes these itself, but winit drops the resulting character, so the client
composes them with the same rules
([alt_code.rs](../crates/sjk-viewer/src/input/alt_code.rs)). Codes without a
leading 0 always use code page 437, the US OEM page, even where Windows uses
another OEM page (850 in much of Western Europe).

`r_we <command>` runs one weather command on the current map, as the original
renderer's `r_we` does: `r_we heavyrain`, `r_we snow`, `r_we heavyrainfog`,
`r_we constantwind ( 0 -800 0 )`, `r_we clear` (an unknown command lists them).
It adds to the map's own weather until the next map; see
[Weather](rendering.md#weather).

`connect host:port`, `disconnect` and `reconnect` control the session.
`record`, `stoprecord`, `demo` and `playdemo` control demos.
`ghost record`, `ghost stop` and `ghost play` record and replay the player's own run
([Ghost replay](#ghost-replay)); `mimic <player> [delay] [mirror|copy] [walk]` copies another
player's moves live, `walk` to walk always ([Mimic](#mimic)).
`modelviewer` opens a model viewer (no menu entry): every installed model in a list the
search narrows (each word must be in the name, case ignored): each player and NPC body
with every skin it has, then every rigid `.md3` and `.glm` under `models/` (items,
weapons, map objects, vehicles, hats), the retail game's first. A player stands on the
stage beside the list, or in the live preview over a match, as the Collection's Shaders
tab shows the player's model; a rigid model replaces it, scaled to a player's height
and turning slowly. Type to search, Up/Down, Page Up/Down, Home/End or the wheel
choose, Escape closes. The list is read from the mounted files when the page opens
([model_viewer_scan.rs](../crates/sjk-viewer/src/model_viewer_scan.rs)); a model that
cannot be drawn is logged and leaves the stage empty. It changes no setting and draws in
the SJK UI's look in every menu style
([model_viewer.rs](../crates/sjk-viewer/src/model_viewer.rs),
[prop.rs](../crates/sjk-viewer/src/menu_stage/prop.rs)).
`screenshot` and `screenshotJPEG` request captures; `condump filename` saves
console output. A capture copies the final swapchain image into a buffer that is
mapped over the next frames; frames keep running meanwhile, and the readback stays
in [screenshot.rs](../crates/sjk-viewer/src/screenshot.rs)'s state until its own map
callback lands (dropping it earlier aborts the map). See [console registration](../crates/sjk-viewer/src/console_session.rs)
and [file commands](../crates/sjk-viewer/src/console_files.rs) for argument handling.

`tell <player> <message>` takes a slot number or, as in EternalJK, a name or a
unique part of one, ignoring case and colour codes; an exact name wins over longer
names containing it. The client sends the stock `tell <slot>` command; when no
player or several players match, it lists them and sends nothing. See
[console_tell.rs](../crates/sjk-viewer/src/console_tell.rs).

Held actions such as `+button12` work from binds, cfg files and the console. A
hold typed at the console lasts until its `-` command, as in the stock client:
opening the console, a menu or chat, or losing window focus, releases held keys
but not typed holds; `in_restart` and session changes release both. `+grapple` is
EternalJK's name for `+button12`, the JA+/JaPRO grapple hook; on a JA+ server
releasing it also taps `+use`, as EternalJK does. See
[input.rs](../crates/sjk-viewer/src/input.rs).

`flipkick` is JoF EJK's flip-kick bind: one press starts a run of jump taps, jump
held in one user command and released in the next, overriding a held jump key and
letting go of it when the run ends. EJK counts frames; SJK counts user commands,
which it makes every 8 ms, EJK's 125 fps frame
([networking.md](networking.md#user-commands-and-move-packets)), so one frame is
one command and `cg_fkDuration` (default 50) lasts 0.4 s.
`cg_fkFirstJumpDuration` holds the first jump for that many frames and
`cg_fkSecondJumpDelay` starts the second jump at that frame (both 0). A server
forbids it with bit 7 (`RESTRICT_FLIPKICKBIND`) of serverinfo `restricts`. See
[flip_kick.rs](../crates/sjk-viewer/src/input/flip_kick.rs).

`weapmelee` (SJK's; Settings > Key bindings > Weapons > Melee (fists), unbound by
default, Sol's request of 10/10/2026) selects the melee weapon, the fists
(`WP_MELEE`), by name. The numbered binds cannot: `weapon 1` is the saber while the
player holds one (`CG_Weapon_f`), and with the saber already selected it switches
the saber on or off instead; the fists otherwise come only from cycling the weapon
keys. `weapmelee` never selects the saber and never toggles: it does nothing when
the fists are already selected, when the player does not hold them, while
following someone or on an emplaced gun. The stun baton, `WP_STUN_BATON`, is another
weapon and is not chosen. Like `weapnext` it only changes the client's selection
(the user command sends it); the rule is
[`legacy_melee_weapon`](../crates/sjk-client/src/weapon_selection.rs).

Weapon selection belongs to the viewed player. When an accepted snapshot changes
that player, enters/leaves follow or spectator mode, or rewinds the timeline,
SJK resets the retained weapon choice to the snapshot's equipped weapon and
refreshes the selection display time before the next user command. This follows
JoF EJK #39's `CG_TransitionPlayerState` reset and also covers native transitions
that keep the same client slot. Old pickup/no-ammo event baselines and predicted
acknowledgements do not carry into the new view. Ordinary snapshots preserve a
pending local weapon choice; map/session presentation resets clear the retained
choice and display timer. Both HUD/menu styles share this behavior. See
[weapon_selection_view.rs](../crates/sjk-viewer/src/weapon_selection_view.rs).

`cl_idrive` is JoF EJK's "last key wins" input (archived, default 0): while both
keys of a movement pair are held, the one pressed last moves the player instead of
the two cancelling out. `1` resolves every pair (forward/back, strafe left/right
including `+strafe` with the turn keys, jump/crouch), `2` only jump/crouch; keys
pressed in the same millisecond still cancel, as in EJK. `cl_idriveDelay` is SJK's
own (archived, default 0, at most 1000): for that many milliseconds after the newer
key's press, and again after it is let go, while the older key is still held, the
pair stays neutral, so a reversal is never instant on servers that penalise one. The
delay is counted per command, so a frame that makes several commands (below 125 FPS)
spaces them out. Both apply from the next frame and are in Settings > Controls. See
[idrive.rs](../crates/sjk-viewer/src/input/idrive.rs).

`cl_forceRepeat` (archived, default 1, Settings > Controls) makes a held push or pull
key use its power again as soon as it can, as JoF EJK does: there, as in every OpenJK
client, the system's key repeats run a plain bind command again (`CL_ParseBinding` on
every press), while SJK drops them for binds. While a key or mouse button whose bind
runs `force_throw` or `force_pull` is held, every user command no press filled carries
that power's `generic_cmd`; the last such key pressed wins. The server decides when it
fires: codemp takes push and pull on every command (`lastGenCmd` spares only those
two) and `ForceThrow` refuses while the hand is still out or the power is not usable,
so it fires on the first command after the cooldown. The other powers' keys do not
repeat: speed, rage, protect, absorb and seeing are toggles a second command switches
off, and heal, mind trick and the team powers would spend the pool unasked. A key's
release stops it even when a menu or the chat took the release, losing the window
stops every key, and nothing is sent while the console, chat or a menu has the
keyboard. `+useforce` does not repeat (the server waits for its release). An alias
that runs push is not followed. The table of repeated commands, with why each is safe,
is in [force_repeat.rs](../crates/sjk-viewer/src/input/force_repeat.rs).

`+duck` is JoF EJK's crouch without a roll (`CG_NorollDown_f`). Its press lets go
of jump and walks for one user command, then it crouches like `+movedown`, sharing
its key state. Crouching while moving rolls only from a running legs animation
(`bg_pmove.c` `PM_Footsteps`); the walking command replaces it with the walk, and
once crouched the legs crouch instead. EJK walks with `+speed`, which runs under
`cl_run 0`; SJK walks whatever `cl_run` is. A roll on landing while crouched
(`PM_CrashLand`) is not prevented, in EJK either. Under `cl_idrive` `+duck` is the
crouch key of the jump/crouch pair, as `+movedown` is. Unlike EJK, whose `-duck`
also lets go of every `+movedown` and `+speed` key, `-duck` releases only itself. See
[input.rs](../crates/sjk-viewer/src/input.rs) (`up_move`).

`cg_gripVictimFX` (archived, default **on**) shows red puffs near choking players'
necks, following JoF EJK's Grip victim effect. Turn **Grip victim effects** off in
classic Settings > Game options or SJK Settings > Advanced > Gameplay to hide
them. Your viewed player's puffs stay hidden in first person and while zoomed;
other players and detached cameras retain theirs. A missing neck uses the posed
head as fallback; models with neither show no puff. The choking animation is a
visual marker, so the effect can also accompany other uses of that animation.
It changes no Grip damage, timing, targeting or movement. See
[rendering](rendering.md) for the reference and effect details.

`cp_pluginDisable` is the archived userinfo cvar a JA+ server reads to switch its
client-plugin features off for this client (a set bit disables one). Its default,
1536, disables the holstered saber and ledge grab, which need JA+ animations SJK
does not have. JA+ and TaystJK/jaPRO servers receive it from the connect packet
on, and a change sends a userinfo update ([networking.md](networking.md)). The
JA+ tools mod lists and toggles its bits (`japlus.plugin`) and lists a JA+
server's options (`japlus.serverconfig`); see [mods.md](mods.md).

The console input line has a caret, drawn as stock's underscore: Left and Right
move it, Ctrl+Left and Ctrl+Right by word, Home and End to either end, and Shift
with any of them selects. Backspace and Delete remove a character, or a word with
Ctrl; words end at anything but a letter or digit, so `cg_drawFPS` and
`127.0.0.1` are edited a piece at a time. Ctrl+A selects the line, Ctrl+X cuts,
and Ctrl+V or Shift+Insert pastes, replacing a selection. Typing inserts at the
caret, a long line scrolls sideways to keep it in view, and Enter runs the whole
line. Dragging the mouse over console output selects it, a double click selects
one whitespace-separated word (a whole `host:port`), Shift+click extends a
selection and a click elsewhere clears it; in the input line the mouse places the
caret and selects the same way. Ctrl+C (or Ctrl+Insert) copies selected output
without colour codes, else the selected input, else the whole input line, or the
last `viewpos` or `mark` answer when the line is empty. Up and Down stay history.
On Windows the clipboard is reached through PowerShell with UTF-8 in both
directions ([clipboard.rs](../crates/sjk-viewer/src/clipboard.rs)), so pasted or
copied symbols such as `€`, `’` or `×` keep their characters, including under
PowerShell's Constrained Language Mode (with `clip` as a last resort for copy). A
copy waits up to 1.5 s for the tool to finish and reports failure if it fails; a
paste waits for a copy still being set.
On Linux, SJK first reads and owns the X11 or Wayland clipboard directly. X11 and
Wayland compositors exposing the data-control protocol therefore need neither
`xclip` nor `wl-clipboard`; XWayland covers common remaining Wayland sessions. Those
desktop tools remain fallbacks: `wl-copy`/`wl-paste` are used only in a Wayland
session and `xclip` only with an X11 display. An installed tool for the other desktop
type is skipped, and an immediate startup failure falls through to the next clipboard
backend. Copy notices are shown only after a backend accepts the text.
See [console_editing.rs](../crates/sjk-viewer/src/console_editing.rs) and
[console_selection.rs](../crates/sjk-viewer/src/console_selection.rs). The input
line and output rows are drawn and measured on the console's character grid
([console_classic.rs](../crates/sjk-viewer/src/console_classic.rs)), so the caret,
highlights and mouse hits follow the drawn cells at any `con_scale`.

Tab completes the command or cvar name being typed, after a leading `/` or `\` and
after the last `;`. A unique name completes with a trailing space; otherwise the
input extends to the longest shared prefix and the matching commands and cvars,
with cvar values, are listed as EternalJK prints them (`PrintMatches`,
`PrintCvarMatches`): a grey `Cmd` or `Cvar` label, the white name, a cvar's value
in grey quotes and the description in green. Up to 16 matches also show their
descriptions; longer listings end with the match count instead. Typed lines, and
the line a listing completes, are echoed into the scrollback as `]cmd`, the
prompt character straight before the text as in stock. Both are shell behaviour
and the same in either console style. Enter strips one leading `/` or
`\` from the line, then applies the same completion while `cl_allowEnterCompletion`
is set, without listing when the input is already a full name. Nothing strips a
slash on a command after `;`, so completing that command drops it. Command
boundaries follow the shell's quote and escape rules, and no completion happens
inside an open quote. Tab also completes the file name after `exec`, `execq`,
`writeconfig` and `write` (`Cmd_CompleteCfgName`): the `.cfg` files at the top of
the profile's folder, then those at the root of the search paths, matched without
case and each name once. A unique file completes with a trailing space; several
extend the typed text to their shared start, keeping its letter case, and are
listed as grey `File` rows (`PrintFileMatches`). Enter does not complete
arguments, and no other argument is completed. See
[shell_completion.rs](../crates/sjk-shell/src/shell_completion.rs) and
[console_scripting.rs](../crates/sjk-viewer/src/console_scripting.rs).

F3 in the open console, or the bindable `consolebrowser` command, opens a browser of
every command and cvar with its description, and each cvar's value and default. Typing
searches names, then descriptions; Tab cycles All, Commands, Cvars and Changed (cvars
away from their default). Enter edits the selected cvar in place and applies it, or
starts a console line with the selected command; Delete restores a cvar's default;
Escape cancels an active edit first; otherwise Escape or F3 returns to the console.
The clickable Apply, Cancel and Filter controls follow the same actions as the
keyboard. Read-only cvars are listed but not edited. The
browser covers the whole frame: underlying menu shapes/text, chat and the FPS
counter are suppressed, including every font batch. See
[console_browser.rs](../crates/sjk-viewer/src/console_browser.rs).

With the classic console (`con_style classic`) the browser is drawn classic+
([classic-plus.md](classic-plus.md)): the in-game pop-up's retail box and title
band over the dimmed screen, the four filters and Edit (Insert, Apply), Default,
Filter and Close as retail gold buttons with the `menu_buttonback` glow, a retail
list box whose selected row sits on `menu_blendbox2` (a changed cvar's value in
gold with its default beside it), a detail box with the selected entry's kind,
value, default and whole description, and the description line under the box.
It uses the menus' retail font when `ui_gameFont` is on, and works without the
retail art. Keys, pointer and wheel act as in the SJK UI's look
([console_browser_classic.rs](../crates/sjk-viewer/src/console_browser_classic.rs)).

With the SJK UI's console (`con_style sjk`, or `auto` with the SJK UI's
menus) the browser has the SJK UI's look, laid out as its
Settings over the darkened frame
([console_browser_sjk.rs](../crates/sjk-viewer/src/console_browser_sjk.rs)): Esc
"Back", "Commands and cvars" and the search pill at the top; the four filters
(All, Commands, Cvars, Changed) down a lit rail with how many entries each shows
for the search; the entries (name, description, the value at the right and a
gold dot when it differs from the default; a command's kind); and the chosen
entry's detail column: its kind, name, value, default and whole description,
then Edit value (Insert for a command, Apply while typing), Default and Copy.
The detail's name, value, default and description can be selected with the
mouse: a drag selects across lines, a double click selects a word
([text_select.rs](../crates/sjk-viewer/src/text_select.rs)); the selection lasts
while the same entry is shown. In every look Ctrl+C (or Ctrl+Insert) copies the
selected text, or else the chosen entry as a console line (a command's name, a
cvar's `name value`, the value quoted when empty or spaced), and the footer says
what was copied. Mouse selection is the SJK UI's look's only; the classic+ look
copies the chosen entry.

For graphics controls and diagnostics, see [rendering.md](rendering.md).

## Text size and spacing

The Settings screen's TEXT tab holds three archived cvars, beside the console
style and the console feed (`con_drawNotify`, [above](#classic-console)). Menu text
draws as before at the defaults.

| Cvar | Default | Range | Effect |
| --- | --- | --- | --- |
| `ui_textScale` | 1 | 0.8 to 1.4 | Text size on menu screens and the in-game menu |
| `ui_letterSpacing` | 0 | -0.05 to 0.15 | Extra space after each letter of menu text, as a fraction of the text size; the console's character grid keeps its cells |
| `con_scale` | 1 | above 0 (menu: 0.5 to 2) | Size of the whole console: its character cells, margins and rows |

Menu text grows or shrinks about the centre of its line. SJK Settings' option
rows have room for the largest size; long Profile and Collection tab rows close
their gaps as the text grows. Classic+ option panels grow their rows above size
1 and show fewer at once through their existing scrolling, so their labels,
controls and pointer targets do not overlap. Other layout positions stay
unchanged. The style is applied where retained text commands become glyph quads, see
[text/style.rs](../crates/sjk-viewer/src/text/style.rs). These settings do not
affect chat, the scoreboard or the HUD. Letter spacing stays 0: Inter's average
advance relative to its x-height (0.89) is already close to stock's cells (0.8),
Inter's own size-specific tracking at this size is +0.002 em, and tighter text
would run digits and `il1` together. A profile's `con_lineSpacing` and
`con_maxLines`, which only the retired modern console read, are dropped at its
start ([Menu style](#menu-style)).

The chat box's wrapped body rows are one line box apart (18 px at 1080p and
`cg_chatBoxFontSize` 1; they were 27 px). Inter's capitals are 0.60 of that box,
the stock chat box's ratio (`ocr_a` capitals at scale 0.65 in rows 13 virtual
pixels apart, `CG_ChatBox_DrawStrings`), and descenders clear the next row by
0.18 of the box. A sender's name line advances 20 px to the body (was 26) and
messages are 8 px apart (were 10 after a named message, 14 otherwise); see
[chat/layout.rs](../crates/sjk-viewer/src/chat/layout.rs).

`cg_chatBoxFontSize` (default 1, archived; SJK Advanced > Chat > **Chat text size**,
0.25 to 3 in steps of 0.05) scales the chat's text, row spacing and inline GIFs.
The wrapping width stays in its configured screen position, so larger text wraps
sooner. The setting is also on Simple > HUD and the classic+ HUD page.

`cg_chatBoxLetterSpacing` (default -0.5, archived; also Settings > Chat > Chat letter
spacing, -2 to 8 in steps of 0.5) adds that many 1080p pixels after each character of
a message body, scaled with the viewport and `cg_chatBoxFontSize`; below zero packs
the letters closer; the default -0.5 tightens the fixed-width game chat font a
little, and 0 is each font's own spacing. Wrapping measures the same spacing (an emoji picture keeps its
own width), so a wider setting wraps sooner. Sender names, tags and the composer
keep their own spacing.

A message too long for one row wraps onto up to four, each drawn as its own text,
and every row after the first starts in the colour code in force where the row
before it ended, as if the message were one line: a green `say` (the server puts
`^2` in front of its text) stays green on every row, and a code set on one row
reaches the next. A row's carried code is kept with its range when the message is
wrapped (`Wrapped::carry`, [chat/layout.rs](../crates/sjk-viewer/src/chat/layout.rs)),
written in front of the row as `^<digit>` (`text::Carry`), which takes no room, so
no wrap moves; a code is never cut between its `^` and its digit. A message with no
codes is in the chat's base colour on every row, and the rows after an emoji picture
keep it too. The console's scrollback and notify lines carry the colour in the same
way (`wrap` in [console_classic.rs](../crates/sjk-viewer/src/console_classic.rs)).

### Percent signs and quotes in chat

The engine turns every `%` of a command into `.` on its way to the server
(`MSG_ReadString`), and a `"` would end the quoted message. Messages from the chat
composer and `tell` are therefore sent as EternalJK sends them (`Message_Key`,
`cl_keys.cpp:896-930` at JoF EternalJK bd5e202): `%` as `°/.` (the degree sign is
byte 0xB0) and `"` as `''`. The chat box shows those back as `%` and `"`, as
EternalJK's `CG_ChatBox_AddString` does (`cg_draw.c:9897-9916`), so SJK and
EternalJK players see each other's `%` and quotes; a client without this shows
`°/.` and `''`. As in EternalJK, the console's copy of a chat line keeps the
escapes, and two apostrophes typed together show as a quote. The composer counts
a draft's length as it will be sent (`%` takes four bytes, `"` two), so it stops
accepting characters where the sent message would otherwise be cut
([chat.rs](../crates/sjk-client/src/chat.rs) `chat_command`, `chat_unescape`,
`chat_input_cost`).

### Chat emojis

`cg_chatBoxEmojis 1` (default 0, also Settings > Chat > Chat emojis) shows JoF
EternalJK's chat emojis: each `gfx/emoji/*.png` is drawn in place of its name in
messages that arrive while the option is on, and shows while it stays on. Names are
made as JoF EternalJK makes them (`CG_LoadEmojis`, `cg_main.c:2767-2827` at
bd5e202): the file name without `.png`, a backtick for `:`, `~` for `>` and `!`
before a letter for upper case, so `` `poop`.png `` is `:poop:` and `#~`!D.png` is
`#>:D`. File names over 26 characters are skipped with a warning and at most 256
load. A message replaces names outside colour codes, the first listed emoji that
matches winning, up to 32 per message (`CG_ChatBox_AddString`,
`cg_draw.c:10078-10204`); a picture is 17/20 of a row high and text after it keeps
the row's colour. A name whose picture does not decode leaves a blank. `listEmojis`
prints the names. JoF's 175 pictures ship in `japro-assets.pk3` in the `EternalJK`
folder, which SJK does not mount whole (its menus and HUD would replace SJK's); the
`gfx/emoji/*.png` of that folder's PK3s are mounted on their own, above `base`, as
EternalJK mounts its folder
([asset_search_paths.rs](../crates/sjk-viewer/src/asset_search_paths.rs)).

Differences from JoF EternalJK: its 2048-byte folder list holds about 150 of JoF's
175 names, depending on the archive's order, so the rest never show there, while all
load here; its loader switches the option off when it is on as a map loads
(`!fileCnt < 1`, a slip for `fileCnt < 1`), while here the list always loads; it
also replaces names in the sender's name, which here is drawn as text on its own
line; an empty name (a file called `!.png`) ends its matching for every later emoji,
while here that file is skipped; a picture that does not load blanks it and every
later picture of its message, while here only its own place is blank; and its folder
sits below an `fs_game` mod, whose own emoji picture of the same name wins, while
here the `EternalJK` pictures sit above every game folder, as the crosshair pictures
do. See [chat/emoji.rs](../crates/sjk-viewer/src/chat/emoji.rs).

## Vehicle and creature assets

Vehicle appearances use the shared multiplayer `.veh` parser, including its
comment handling, model/skin selection and camera fields. Example definitions
inside block comments cannot replace the real vehicle model.

Vehicle projectile trails resolve their vehicle-weapon index through `.vwp`
definitions in model-registration order. Their authored EFX and optional rigid
models are preloaded when the vehicle is registered; an effect-only laser does
not acquire the ordinary rocket model. Custom vehicle flight-loop sounds remain
a separate audio gap.

Community NPCs need their model PK3 mounted by both the server and client. An NPC
definition alone cannot supply a missing mesh. SJK does not distribute those
packs. The animation-error isolation described in [rendering.md](rendering.md#actor-animation-failures)
protects other actors from malformed custom clips, but does not repair the clip.

## NPC sabers and voices

NPCs (`ET_NPC`, entity numbers 32 and up) have no clientinfo, so their sabers and
voices come from their own entity rather than a player slot (hub report 68, a JoF
server on `JoFCrucible`). Sources are JoF EJK `codemp/cgame`.

- **Thrown sabers.** The server tags a flying saber with `genericenemyindex = owner
  + 1024` for a player and an NPC alike (`w_saber.c:6939`). The client accepts any
  owner entity number, not only the 32 client slots, and draws an NPC's thrown
  saber with its `npcSaber1` hilt and blade colour
  ([thrown_sabers.rs](../crates/sjk-client/src/thrown_sabers.rs)); before
  17/10/2026 an NPC's thrown saber was invisible while its hand was empty.
- **Blade colours.** With `boltToPlayer` zero, each NPC blade wears its `.sab`
  `saberColor` (`CG_AddSaberBlade`, `cg_players.c:8049-8064`; `saberColor random`
  is one of orange to purple, steady per NPC); before 17/10/2026 it was always red.
  With `boltToPlayer` non-zero both sabers read their packed colour, and one whose
  three bits are zero gets stock's `-1`, drawn blue (`CG_DoSaber`'s `default`).
- **No saber entity.** An NPC whose `saberEntityNum` is zero has its primary saber
  neither in hand nor lit; only a second saber is drawn (`cg_players.c:14210-14212`),
  as for a saber knocked away. Players are not checked this way yet
  ([npc_identity.rs](../crates/sjk-client/src/npc_identity.rs),
  [actor_world_submission.rs](../crates/sjk-viewer/src/actor_world_submission.rs)).
- **Voices.** `*pain25`, `*death1`, `*jump1` and the other custom sounds of an NPC
  (or a vehicle, which is an NPC) come from its own sound folders: the server names
  each as `*$<dir>` in `CS_SOUNDS` and points the NPC's `csSounds_Std`, `_Combat`
  and `_Jedi` at it (`NPC_stats.c`, `CG_HandleNPCSounds` in `cg_servercmds.c:718`).
  As `CG_CustomSound` (`cg_players.c:213-322`), death, jump, pain, choke, gasp, land
  and taunt come from the standard set, anger, victory and pushed from the combat
  set, and taunt1-3, deflect, gloat and pushfail from the Jedi set, each
  `sound/chars/<dir>/misc/<name>`. An NPC with no set for a sound, or whose file is
  missing, is silent: unlike a player it has no `mp_generic` fallback. Sets load
  when their `CS_SOUNDS` slot changes, not per event
  ([sound_npc_voices.rs](../crates/sjk-client/src/sound_npc_voices.rs)). Before
  17/10/2026 every NPC spoke with the voice of player slot 31. The `_Extra` set
  (`*chase1`, `*cover1`, ...) has no event in the sound table and is not loaded.

## Shader remap controls

Server map recolors and material replacements are enabled by default. SJK
defaults to `cg_remaps 2`, EternalJK's default, which includes player-texture
configstring remaps; `cg_remaps 1` is TaystJK's default policy
excluding them, and `cg_remaps 0` disables server remaps.
Settings > GAME > "Shader remaps" sets the same cvar and applies at once.
`cg_remapsBlockedMaps` lists maps where server remaps are ignored, as with
`cg_remaps 0` on those maps only: map names separated by spaces, commas or
semicolons (`mp/ffa4 mp/duel6`; `maps/` and `.bsp` are optional, capitals do not
matter; typed in the console, a list with semicolons needs quotes). `blockRemaps [map]` adds a map and `unblockRemaps [map]` removes it, the
loaded one when no map is named; the change applies at once, and `listRemaps` says
when the loaded map is blocked. The list is saved with the other settings.
A map's own worldspawn remaps always apply. `listRemaps` lists the map's, the
enabled server and the temporary local remaps in the order they were applied, each
labelled `map:`, `server:` or `local:`, and marks those a later remap overrides.
`remapShader <old> <new>` replaces a shader locally for the loaded map; remapping
it to itself restores the original. The latest remap of a shader wins, so a later
server remap replaces a local one. `clearRemaps`, EternalJK's renderer command,
drops every active remap whatever its origin, the map's included, without telling
the server; the world draws its own shaders until the server sends new remaps (a
shader-state change or a `remapShader` command) or a new map loads. As in
EternalJK, destination time offsets are kept. See [rendering](rendering.md#server-shader-remaps)
for scope and remaining limitations. These controls do not edit map geometry.
