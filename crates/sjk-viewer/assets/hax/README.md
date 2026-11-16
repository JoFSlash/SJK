# Hax

The Hax toy (SJK's own, [client.md](../../../../docs/client.md#hax)) lets a player who has
it on hack another who has it on and steer them for a few seconds. The pictures here are
drawn by [hax_assets.py](../../../../scripts/hax_assets.py) from shapes and SJK's bundled
JetBrains Mono and Noto Sans JP fonts (nothing taken from another game or a film), so
none is edited by hand except `hax_glyphs.efx` and `hax.shader`.

| File | Game path | What |
| --- | --- | --- |
| `toy_hax.png` | `gfx/sjk/toy_hax.png` | The toy's icon (quick wheel, Collection, key bindings), 128x128: a terminal with a green ">_" |
| `glyph_<n>.png` | `gfx/sjk/hax/glyph_<n>.png` | The digital rain's glyphs (0, 1 and six half-width katakana), 32x32, white with a glow |
| `hax_glyphs.efx` | `effects/sjk/hax_glyphs.efx` | One puff of the rain: glyphs on a ring round the player, drifting down and fading |
| `hax.shader` | `shaders/sjk_hax.shader` | `gfx/sjk/hax/glyph_<n>`: the glyphs, additive, tinted by the effect |

The client mounts them in memory below all game data ([hax.rs](../../src/hax.rs)), so a
PK3 that has the same paths replaces them. The hack bar's icon and the sounds are the
retail game's (`gfx/mp/c_icon_tech`, `sound/player/hacking.wav`, the mind trick's).
