# Banana peel

The Banana peel toy (SJK's own, after Kushina's idea,
[client.md](../../../../docs/client.md#banana-peel)) drops banana peels that players who
have the toy on slip on. Everything here is made by
[banana_assets.py](../../../../scripts/banana_assets.py) from a few parametric strips,
noise and synthesized sound (nothing taken from another game), so none is edited by hand
except `banana.shader`.

| File | Game path | What |
| --- | --- | --- |
| `toy_banana.png` | `gfx/sjk/toy_banana.png` | The toy's icon (quick wheel, Collection, key bindings), 128x128: the model itself, from the side and a little above |
| `banana.jpg` | `models/sjk/banana.jpg` | 256x256: the yellow skin with brown speckles and dark tips (top), the pale flesh side (bottom left), the stem (bottom right) |
| `banana.md3` | `models/sjk/banana.md3` | A peel lying on the floor, about 19 units long, 12 wide and 6 tall: a mound (its closed end) with the stem's stub leaning back (-X), four strips fanning out ahead and to the sides, two curling up at their tips; centred on its origin, where it rests on the floor, +Z up |
| `banana.shader` | `shaders/sjk_banana.shader` | `models/sjk/banana`: lit as a player model is, both sides drawn |
| `slip.wav` | `sound/sjk/banana/slip.wav` | A cartoon slip: a squeaky skid, then a slide whistle zipping up and dropping; 22050 Hz mono |
| `drop.wav` | `sound/sjk/banana/drop.wav` | A peel landing: a soft, wet flop |

`python scripts/banana_assets.py crates/sjk-viewer/assets/banana --preview <folder>` also
draws the model from five sides into `<folder>/banana_preview.png`, to look at it without
the game. The client mounts the files in memory below all game data
([banana.rs](../../src/banana.rs)), so a PK3 that has the same paths replaces them.
