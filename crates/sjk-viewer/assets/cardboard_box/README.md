# The cardboard box

The cardboard box toy (SJK's own, [client.md](../../../../docs/client.md#cardboard-box))
puts a shipping carton over the player. Everything here is drawn by
[cardboard_box_assets.py](../../../../scripts/cardboard_box_assets.py) from noise and
shapes (nothing taken from another game), so none is edited by hand except
`cardboard_box.shader`.

| File | Game path | What |
| --- | --- | --- |
| `toy_box.png` | `gfx/sjk/toy_box.png` | The toy's icon (quick wheel, Collection, key bindings), 128x128, like `gfx/mp/f_icon_*` |
| `cardboard_box.jpg` | `models/sjk/cardboard_box.jpg` | 1024x512: a side on the left, the taped top on the right |
| `cardboard_box.md3` | `models/sjk/cardboard_box.md3` | An open-bottomed box, 46 x 46 x 44 units, its origin in the middle of the bottom |
| `cardboard_box.shader` | `shaders/sjk_cardboard_box.shader` | `models/sjk/cardboard_box`: lit board, both sides drawn |

The client mounts them in memory below all game data
([cardboard_box.rs](../../src/cardboard_box.rs)), so a PK3 that has the same paths
replaces them. The toy's sounds are not here, and SJK ships none: they are the game
paths `sound/sjk/cardboard_box/music`, `equip`, `spotted`, `drop` and `death`.
