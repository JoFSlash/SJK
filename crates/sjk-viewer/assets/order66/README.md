# Order 66

The Order 66 toy (SJK's own, [client.md](../../../../docs/client.md#order-66)) blows the
players who have it on apart when one of them executes the order. The pictures here are
drawn by [order66_assets.py](../../../../scripts/order66_assets.py) from shapes and SJK's
bundled Exo 2 font (nothing taken from another game or a film), so none is edited by hand
except `order66_confetti.efx` and `order66.shader`.

| File | Game path | What |
| --- | --- | --- |
| `toy_order66.png` | `gfx/sjk/toy_order66.png` | The toy's icon (quick wheel, Collection, key bindings), 128x128: a red "66" badge |
| `order66_execute.png` | `gfx/sjk/order66_execute.png` | The order's icon (quick wheel, key bindings), 128x128: "66" on a blast |
| `confetti_<colour>.png` | `gfx/sjk/order66/confetti_<colour>.png` | One paper scrap each of red, yellow, green, cyan, blue and pink, 16x16 |
| `order66_confetti.efx` | `effects/sjk/order66_confetti.efx` | The confetti that takes the place of the pieces with `cg_dismember 0` |
| `order66.shader` | `shaders/sjk_order66.shader` | `gfx/sjk/order66/confetti_<colour>`: the scraps, both sides drawn |

The client mounts them in memory below all game data
([order66.rs](../../src/order66.rs)), so a PK3 that has the same paths replaces them. The
order's sound is not here, and SJK ships none: it is the game path
`sound/sjk/order66/order.wav`. The explosion and the countdown's beep are the game's own
(`thermal/explosion`, `sound/weapons/thermal/warning`).
