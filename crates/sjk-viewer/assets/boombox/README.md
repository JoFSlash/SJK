# Boombox

The Boombox toy (SJK's own, [client.md](../../../../docs/client.md#boombox)) puts an 80s
stereo on the player's shoulder. Everything here is drawn by
[boombox_assets.py](../../../../scripts/boombox_assets.py) from shapes and noise (nothing
taken from another game), so none is edited by hand except `boombox.shader`.

| File | Game path | What |
| --- | --- | --- |
| `toy_boombox.png` | `gfx/sjk/toy_boombox.png` | The toy's icon (quick wheel, Collection, key bindings), 128x128, like `gfx/mp/f_icon_*` |
| `boombox.jpg` | `models/sjk/boombox.jpg` | 1024x1024: the front (speakers, cassette deck, tuner, keys) on top, plastic and chrome below |
| `boombox.md3` | `models/sjk/boombox.md3` | The body, 20 x 5 x 10 units, and its handle; its origin in the middle of the bottom, its front facing +X |
| `boombox.shader` | `shaders/sjk_boombox.shader` | `models/sjk/boombox`: lit as a player model is |

The client mounts them in memory below all game data
([boombox.rs](../../src/boombox.rs)), so a PK3 that has the same paths replaces them. The
music is not here, and SJK ships none: each player's tracks are their own files,
`sound/boombox/<id>.mp3` (or `.wav`) in a pk3.
