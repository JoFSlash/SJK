# Fishing

The Fishing toy (SJK's own, [client.md](../../../../docs/client.md#fishing)) props a rod
beside the player, sets a bucket at their feet and floats a line on the water. Everything
here is drawn by [fishing_assets.py](../../../../scripts/fishing_assets.py) from cylinders
and flat colours (nothing taken from another game), so none is edited by hand except
`fishing.shader`.

| File | Game path | What |
| --- | --- | --- |
| `toy_fishing.png` | `gfx/sjk/toy_fishing.png` | The toy's icon (quick wheel, key bindings), 128x128 |
| `fishing.png` | `models/sjk/fishing.png` | The colour atlas, 64x64, a flat colour a 16x16 cell |
| `fishing_rod.md3` | `models/sjk/fishing_rod.md3` | The rod along +X, 72 units from its butt (the origin) to its tip |
| `fishing_bucket.md3` | `models/sjk/fishing_bucket.md3` | An open bucket with water and a handle, its origin in the middle of its bottom |
| `fishing_float.md3` | `models/sjk/fishing_float.md3` | The float, red over white, its origin at the waterline |
| `fishing_line.md3` | `models/sjk/fishing_line.md3` | A thin strand along +X, 64 units long, scaled to the length drawn |
| `fishing_fish.md3` | `models/sjk/fishing_fish.md3` | A small fish along +X, its origin in its middle |
| `fishing.shader` | `shaders/sjk_fishing.shader` | `models/sjk/fishing`: lit flat colours, both sides drawn |

The client mounts them in memory below all game data ([fishing.rs](../../src/fishing.rs)),
so a PK3 that has the same paths replaces them. The toy's sounds are retail game paths.
