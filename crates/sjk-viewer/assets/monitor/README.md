# Monitor

The Monitor toy (SJK's own, [client.md](../../../../docs/client.md#monitor)) throws a big
old CRT monitor that sends the players it hits flying as ragdolls. Everything here is
made by [monitor_assets.py](../../../../scripts/monitor_assets.py) from shapes, noise and
synthesized sound (nothing taken from another game), so none is edited by hand except
`monitor.shader`.

| File | Game path | What |
| --- | --- | --- |
| `toy_monitor.png` | `gfx/sjk/toy_monitor.png` | The toy's icon (quick wheel, Collection, key bindings), 128x128: the monitor flying, speed lines behind it |
| `monitor.jpg` | `models/sjk/monitor.jpg` | 1024x1024: the face (bezel, glass with a glowing prompt, power button) top left, the back (vents, label) top right, plain plastic below |
| `monitor.md3` | `models/sjk/monitor.md3` | The bezel and the housing tapering back, 20 x 17 x 18 units; its screen facing +X, its centre (the flight's eight corners', `src/monitor/flight.rs`) at the origin |
| `monitor.shader` | `shaders/sjk_monitor.shader` | `models/sjk/monitor`: lit as a player model is |
| `throw.wav` | `sound/sjk/monitor/throw.wav` | A throw's whoosh, 22050 Hz mono |
| `bonk.wav` | `sound/sjk/monitor/bonk.wav` | A monitor striking a body: a hollow plastic thunk and its glass ringing |
| `knock.wav` | `sound/sjk/monitor/knock.wav` | A monitor knocking on the world: a clunk and a rattle |
| `charge.wav` | `sound/sjk/monitor/charge.wav` | A throw held to charge (the thrower's): a tube's hum rising over a second to a ready ping |
| `throw_hard.wav` | `sound/sjk/monitor/throw_hard.wav` | A charged throw leaving the hand: a thump and a deep whoosh |
| `hit.wav` | `sound/sjk/monitor/hit.wav` | The thrower's hit sound when their monitor strikes someone: a bright tick |
| `saver_on.wav` | `sound/sjk/monitor/saver_on.wav` | A Screen Saver rising: a CRT's degauss boing under a gold shimmer |
| `saver_off.wav` | `sound/sjk/monitor/saver_off.wav` | A Screen Saver ending: a CRT switched off, its whine falling to a pop |

The client mounts them in memory below all game data
([monitor.rs](../../src/monitor.rs)), so a PK3 that has the same paths replaces them.
