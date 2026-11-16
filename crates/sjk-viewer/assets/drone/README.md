# Drone

The Drone toy (SJK's own, [client.md](../../../../docs/client.md#drone)) flies the free
camera and Peek as a training remote other SJK players see. Its icon is made by
[drone_assets.py](../../../../scripts/drone_assets.py) from shapes (nothing taken from
another game), so it is not edited by hand.

| File | Game path | What |
| --- | --- | --- |
| `toy_drone.png` | `gfx/sjk/toy_drone.png` | The toy's icon (quick wheel, Collection, key bindings), 128x128: a training remote hovering |

The client mounts it in memory below all game data ([drone.rs](../../src/drone.rs)), so a
PK3 that has the same path replaces it. The drone itself is the game's own seeker ball
(`models/items/remote.md3`), and it comes out with the game's remote hiss
(`sound/chars/remote/misc/hiss.wav`).
