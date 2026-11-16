# Ragdoll

The Ragdoll toy (SJK's own, [client.md](../../../../docs/client.md#ragdoll)) makes the
player's body go limp and fall, sandbox style. Both files here are made by
[ragdoll_assets.py](../../../../scripts/ragdoll_assets.py), from shapes and synthesized
tones (nothing taken from another game), so none is edited by hand.

| File | Game path | What |
| --- | --- | --- |
| `toy_ragdoll.png` | `gfx/sjk/toy_ragdoll.png` | The toy's icon (quick wheel, Collection, key bindings), 128x128: a limp figure slumped against a wall |
| `flatline.wav` | `sound/sjk/ragdoll/flatline.wav` | Played as the body goes limp: two heart monitor beeps, then a flatline fading out, 22050 Hz mono |

The client mounts them in memory below all game data
([ragdoll.rs](../../src/ragdoll.rs)), so a PK3 that has the same paths replaces them. The
body's knocks are the game's own (`sound/player/bodyfall_human1.mp3` to `3`,
`sound/effects/body_slam1.mp3` and `2`).
