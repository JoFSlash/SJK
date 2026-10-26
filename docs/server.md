# Dedicated server

`sjk-server` is SJK's native headless server (the `sjk-dedicated` crate). It does
not launch another engine. Build it alongside the client as described in [development.md](development.md).

## Local game

Use an unused loopback port for development:

```sh
./target/release/sjk-server \
  --game-data /path/to/GameData \
  --map mp/ffa3 \
  --bind 127.0.0.1:29071 \
  --hostname "SJK local" \
  --gametype ffa \
  --team-auto-join \
  +set dedicated 1
```

Then connect the client to `127.0.0.1:29071`. `dedicated 1` selects LAN operation
without master-server advertising. Keep automated checks local and free of human
players. For remote hosting, explicitly choose an externally reachable bind
address and open the chosen UDP port.

Always supply `--game-data` for a playable map. Without it the process can
advertise a map name but does not load that map's content.

## Siege class catalogues

Installed Siege content follows JoF EJK #253's expanded catalogue capacities:
the first 1024 `.scl` files and 256 `.team` files in load order are considered.
A theme has up to 16 sequential `classN` slots (`class1` through `class16`),
stopping at the first missing key; missing class references retain their slot.
Oversized/refused files still occupy a global listing slot. The named limits
live in the `sjk-game-jka` Siege compatibility parser; owned registries remain
vectors. The client class selector uses the same capacities in both UI styles,
so it does not exceed those catalogue capacities. These content limits
change no protocol-26 field, class loadout, movement or combat timing.

## Options and configuration

The authoritative option parser and usage text are in
[main.rs](../crates/sjk-dedicated/src/main.rs).

| Option | Meaning |
| --- | --- |
| `--map NAME` | Initial map, such as `mp/ffa3` |
| `--maps NAME,NAME` | Rotating list of subsequent maps |
| `--gametype NAME` | `ffa`, `holocron`, `jm`, `duel`, `powerduel`, `team`, `siege`, `ctf`, `cty` |
| `--fraglimit N`, `--timelimit MINUTES` | Match limits |
| `--wire-clients N` | Protocol-26 client slots, 0–32 |
| `--peers N` | Native player capacity, separate from wire slots |
| `--team-auto-join` | Place newcomers in play without the initial spectator selection |
| `--home DIRECTORY` | Writable server config home |
| `--set NAME VALUE` | Set a console variable |
| `--quit-on-eof` | Stop when a supervising parent's stdin pipe closes |
| `--cheats` | Enable the native server's development commands |

These are implemented option surfaces, not a guarantee of complete parity for
every game type or map. See [status.md](status.md).

`+set` and other `+COMMAND` arguments are supported. Startup applies `+set`
values before loading `mpdefault.cfg`, `sjk_server.cfg` and `autoexec.cfg`, then
runs the command-line `+` commands in order. With `--home`, `exec` checks its
`base` directory first and archived variables are saved in `base/sjk_server.cfg`.
Without a home, this archive is not written.

Type commands such as `status`, `map mp/ffa3` and `quit` on stdin. Remote console
commands require an explicitly configured `rconpassword`; never commit that
password. The console implementation lives in
[bridge_console.rs](../crates/sjk-dedicated/src/bridge_console.rs) and the server's
[command buffer](../crates/sjk-dedicated/src/command_buffer.rs).

`g_forceRegenTime` (default 200) is the milliseconds a player's point of Force
takes to come back, read when it changes. Before 26/10/2026 the server ignored it
and always used 200 (a full pool in 20 seconds). NPCs still use 200
([npc_force_update.rs](../crates/sjk-game-jka/src/npc_force_update.rs)), and a
client's `devmap` passes no such setting to the server it starts.

Bots retain their slots and bot identity across map changes and `map_restart`.
They enter the new map immediately because they have no network handshake.
Player session data survives, while transient state and entity handles are reset
for the new world; saber entities are allocated from the new map's pool.

The [networking page](networking.md) explains the boundary between native entity
ownership, game behavior and the legacy endpoint.

## Cheat commands

Cheats follow `sv_cheats`, which only the server sets: `--cheats` turns it on at
startup, the server console's `devmap <map>` loads a map with it on and `map`
loads one with it off. The client's [devmap](client.md#development-maps) starts
an owned local game with cheats on.

Players type these in the client console. Without cheats the server answers
with the stock "cheats are not enabled" message (`@@@NOCHEATS`). `give`, `t_use`
and `noclip` also need a living player in the game (`@@@MUSTBEALIVE`), as in
OpenJK's `ClientCommand`; `setviewpos` also works for spectators.

| Command | Effect |
| --- | --- |
| `give all` | Every holdable item, full health, armour and Force power, every usable weapon and 999 of each ammo type |
| `give health [N]`, `give armor [N]` (or `shield`) | Health (1 to the maximum) or armour (0 to the maximum health), full without `N` |
| `give force [N]` | Force power, 0–100, full without `N` |
| `give weapons`, `give weaponnum N` | Every usable weapon, or the weapon with number `N` |
| `give ammo [N]` | `N` (0–999, 999 without `N`) of every ammo type |
| `give excellent` (also `impressive`, `gauntletaward`, `defend`, `assist`) | One more of that award |
| `t_use NAME` | Fire every target named `NAME` as the player (`Cmd_TargetUse_f`), as mappers test triggers and scripts |
| `setviewpos X Y Z YAW` | Teleport to the position, facing `YAW` |
| `noclip` | Toggle flying through the world (see [Noclip](#noclip)) |

`give` follows OpenJK's `G_Give`; giving an item by its name (which the reference
spawns on the player) is not ported and does nothing. See
[give.rs](../crates/sjk-game-jka/src/give.rs),
[bridge_commands.rs](../crates/sjk-dedicated/src/bridge_commands.rs),
[bridge_cheats.rs](../crates/sjk-dedicated/src/bridge_cheats.rs) and
[bridge_teleport.rs](../crates/sjk-dedicated/src/bridge_teleport.rs).

## Noclip

With cheats enabled (`--cheats` or server-console `devmap`), a living player can
run `noclip` in the client console to toggle flying through the world. The client
also offers [devmap](client.md#development-maps) for starting an owned local game.
The server refuses noclip during intermission, for spectators/dead players, or
with cheats disabled. Spawning again clears it.

Noclip uses the existing OpenJK-compatible movement and snapshot prediction.
It skips item/trigger contacts, drowning and ordinary damage, retaining the
reference's earlier DEMP2 shock handling. OpenJK's command scaling quirk remains:
vertical input alone does not accelerate without forward/sideways input.
See [bridge_cheats.rs](../crates/sjk-dedicated/src/bridge_cheats.rs) and
[noclip.rs](../crates/sjk-game-jka/src/noclip.rs).

## NPC definition packs

With `g_stockRules 0` (the default), native NPC loading reads every listed
`ext_data/npcs/*.npc` file without the stock filename-list or aggregate-text
limits. This allows larger collections of community definitions; it does not
raise the number of NPCs alive at once or change protocol 26 limits.

`g_stockRules 1` keeps the reference's 2,048-byte filename listing and 256 KiB
aggregate-text bound, including the existing empty-definition fallback on a
text-size load error. Both profiles preserve VFS listing order and installed
content precedence. Reload the map after changing the installed definitions.
See [map.rs](../crates/sjk-dedicated/src/map.rs).

## Vehicle boarding and sand creatures

Landing on an empty animal or speeder requests boarding using the multiplayer
rider, vehicle-type and team checks. Empty vehicles marked `SUSPENDED` also allow
boarding while standing on them, including fighters and walkers. These are
server decisions; a client connected to another server uses that server's rules.

With `g_stockRules 0` (the default), `npc spawn sand_creature` and
`npc spawn sand_creature_fast` use native burrowing AI. Set the rule before
loading the map; the NPC roster keeps that selection for the map's lifetime.
`g_stockRules 1` preserves multiplayer's existing generic NPC behavior.
Move clear of the spawn location so the creature can finish spawning.

This extension takes its movement sensing, hunting timeout, breach/attack
animations and sand effects from the single-player sand creature. It hunts moving,
grounded players and NPCs, passes beneath bodies while still colliding with map
solids, breaches, and ambushes close targets. Standing still or leaving the ground
breaks its movement tracking. A successful catch uses ordinary multiplayer damage
and respawn handling, rather than the single-player swallowed-player state.

It is not a campaign-AI parity claim: scripted alerts/turf constraints, thermal
bait, victim attachment/swallowing, and single-player camera effects are not
implemented. Pursuit uses local steering and ordinary NPC movement; complex
obstacles still need broader map playtesting. No new protocol fields or SP entity
flags are sent, and no new assets are required for the retail sand creature.

## Siege doors

The use key reaches a `PLAYER_USE` door (`ValidUseTarget`, `GlobalUse`). In siege,
`TryUse`'s `alliedTeam` rule keeps the door's `teamallow` team from using it unless
`g_ff_objectives` is set. Before the round begins, the use key does nothing.

A door that spawns locked carries `EF_SHADER_ANIM`. Its `s.frame` is 0 while locked and 1
once unlocked (`LockDoors`, `UnLockDoors`), so the client draws the lock lights by state.

`trigger_multiple` and `trigger_once` take their team from `team`, or else from the generic
`teamuser`/`alliedteam` spawn fields.
