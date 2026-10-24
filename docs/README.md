# SJK project wiki

This directory is the canonical project wiki for contributors and AI assistants.
It lives beside the source so implementation and documentation can be reviewed
and changed together. Begin with the status page, then the architecture.

| Page | Contents |
| --- | --- |
| [Status and priorities](status.md) | Current scope, verification and open work |
| [Architecture](architecture.md) | Crate ownership and compatibility boundaries |
| [Development](development.md) | Build, validation and contribution workflow |
| [Distributable packages](packages.md) | Windows/Linux ZIP layout, builds and startup checks |
| [Client](client.md) | Launch, configuration, content and common commands |
| [Dedicated server](server.md) | Hosting, configuration and local testing |
| [Rendering](rendering.md) | BSP rendering, lighting, UI and measurement |
| [Networking and gameplay](networking.md) | Protocol 26, prediction and server authority |
| [SJK conventions](sjk.md) | Branches, names, defaults, versions, releases, changelog, credits and debug panel |
| [Classic+ menus](classic-plus.md) | SJK's modernised classic pages: rules, layout and code recipe |
| [SJK UI](sjk-ui.md) | SJK's own menus (`ui_menuStyle sjk`): design, tokens, screens and plan |
| [Player identity](identity.md) | SJK's identity key, the hub, scoreboard badges, medals, clan tags: design, limits and privacy |
| [SJK chat and emotes](hub-chat.md) | The chat every SJK player shares through the hub, and the emotes path |
| [Holocron drops](holocrons.md) | Loot earned by playing: tiers, odds, caps, the active flag, the pop-up, chat lines, the staff tools and the list other screens read |
| [Cards and credits](cards.md) | Opening holocrons: sealed cards (rarity, finish, edition), credits and their sign, the ceremony, the Cards tab, odds; the first set and the shop planned |
| [Client mods](mods.md) | Mods kept out of the core client (`japlus.*`, `jof.*`): switches, names, what they may do |
| [JoF content](jof-content.md) | JoF's maps, skins, hilts and packs from JoF's repository (`jof.downloads`), opt in |
| [Rig models](rig-models.md) | glTF player models with their own skeleton, driven by the game's animations: manifest, bone maps, limits |
| [Unlockables and looks](unlockables.md) | Blade skins and other unlockables, and looks every SJK player sees through the hub |

[AGENTS.md](../AGENTS.md) defines the documentation maintenance rules. Update the
page that owns a fact rather than adding a second account elsewhere. New facts
should link to code or identify their verification; proposals must remain labeled
as planned until implemented and checked.

For installation and quick launch commands, see the [project README](../README.md).
