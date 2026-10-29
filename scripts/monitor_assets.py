"""Make the Monitor toy's game files (`toy_monitor`, SJK's own).

The Monitor is a meme toy: players throw a big old CRT monitor at each other, and one
that hits sends its target flying as a ragdoll. Nothing here comes from another game:
the monitor is drawn from shapes and noise, and the sounds are synthesized. This script
writes, into the output folder:

- `monitor.jpg`: the monitor's picture, 1024x1024: its face (the beige bezel, the dark
  curved glass with a prompt glowing on it, the power button and its light) top left,
  its back (vents and a label) top right, the plain plastic of its other faces below;
- `monitor.md3`: the bezel and the tube's housing tapering back to a smaller back, its
  screen facing +X, its left +Y, its centre (the centre of the eight corners the
  client's flight simulates, `crates/sjk-viewer/src/monitor/flight.rs`, `SHAPE`) at the
  origin: 20 units wide, 17 tall, 18 deep;
- `toy_monitor.png`: the toy's icon, 128x128 like `gfx/mp/f_icon_*`: the monitor flying,
  speed lines behind it, on transparency;
- `throw.wav`: the whoosh of a throw; `bonk.wav`: a monitor striking a body, a hollow
  plastic thunk and the ring of its glass; `knock.wav`: a monitor knocking on the
  floor; `charge.wav`: a throw held to charge, a tube's hum rising to a ready ping;
  `throw_hard.wav`: a charged throw, a thump and a deep whoosh; `hit.wav`: the tick the
  thrower hears when their monitor strikes someone; `saver_on.wav`: the Screen Saver
  rising, a CRT's degauss boing under a gold shimmer; `saver_off.wav`: the Screen
  Saver ending, a CRT switched off, its whine falling to a pop; all 22050 Hz mono
  16-bit.

    python scripts/monitor_assets.py crates/sjk-viewer/assets/monitor

The output is deterministic (fixed seeds). Needs Pillow and numpy
(pip install pillow numpy). The shader (`monitor.shader`) is written by hand.
"""

import math
import struct
import sys
import wave
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFilter

# The corners, as flight.rs's SHAPE: the face at x = 9, the back at x = -9, a flat bottom.
FRONT_X = 9.0
BEZEL_X = 6.0
BACK_X = -9.0
FACE_HALF_WIDTH = 10.0
FACE_TOP = 10.25
BOTTOM = -6.75
BACK_HALF_WIDTH = 6.0
BACK_TOP = 3.25
SHADER = "models/sjk/monitor"
ATLAS = 1024
# 32 pixels a unit on the face: 20 x 17 units.
FACE = (0, 0, 640, 544)
BACK = (640, 0, 1024, 320)
PLASTIC = (0, 544, 1024, 1024)
BEIGE = (214, 204, 178)
ICON_SIZE = 128
JPEG_QUALITY = 90
RATE = 22050


def noise(rng, size, scale: float) -> np.ndarray:
    """Smooth value noise of `size` (h, w), about `scale` pixels per feature, in 0..1."""
    h, w = size
    gh, gw = int(h / scale) + 2, int(w / scale) + 2
    grid = Image.fromarray((rng.random((gh, gw)) * 255).astype(np.uint8))
    big = grid.resize((int(gw * scale), int(gh * scale)), Image.BICUBIC)
    return np.asarray(big, dtype=float)[:h, :w] / 255.0


def plastic(rng, size, base=BEIGE) -> Image.Image:
    """Old beige plastic: a fine grain and the faint blotches of years of yellowing."""
    grain = noise(rng, size, 2.5) * 0.45 + noise(rng, size, 60) * 0.55
    rgb = np.array(base, float)[None, None, :] * (0.93 + 0.12 * grain)[:, :, None]
    rgb[:, :, 2] *= 0.97 + 0.03 * noise(rng, size, 90)
    return Image.fromarray(np.clip(rgb, 0, 255).astype(np.uint8))


def face(rng) -> Image.Image:
    w, h = FACE[2] - FACE[0], FACE[3] - FACE[1]
    img = plastic(rng, (h, w))
    draw = ImageDraw.Draw(img)
    # A slight bevel round the edge.
    draw.rectangle((0, 0, w - 1, h - 1), outline=(186, 176, 150), width=6)
    draw.rectangle((6, 6, w - 7, h - 7), outline=(228, 220, 198), width=3)
    # The screen's recess and its glass.
    left, top, right, bottom = 58, 46, w - 58, h - 118
    draw.rounded_rectangle((left - 14, top - 14, right + 14, bottom + 14), radius=46,
                           fill=(150, 142, 120))
    draw.rounded_rectangle((left - 6, top - 6, right + 6, bottom + 6), radius=40,
                           fill=(70, 68, 60))
    sw, sh = right - left, bottom - top
    y, x = np.mgrid[0:sh, 0:sw]
    # Curved glass: lighter in the middle, darker to the corners.
    r = np.hypot((x - sw / 2) / (sw / 2), (y - sh / 2) / (sh / 2))
    glow = np.clip(1.0 - r * 0.55, 0, 1)
    glass = np.zeros((sh, sw, 3))
    glass[:, :, 0] = 18 + 16 * glow
    glass[:, :, 1] = 26 + 26 * glow
    glass[:, :, 2] = 24 + 18 * glow
    # Scan lines.
    glass *= (0.86 + 0.14 * (y % 4 < 2))[:, :, None]
    screen = Image.fromarray(np.clip(glass, 0, 255).astype(np.uint8))
    text = ImageDraw.Draw(screen)
    phosphor = (110, 255, 150)
    lines = ["SJK BIOS v1.0", "Memory: 640K OK", "", "C:\\>throw monitor", "C:\\>_"]
    for i, line in enumerate(lines):
        # Big blocky letters: the default font drawn at 1x then scaled up 3x.
        if not line:
            continue
        small = Image.new("L", (len(line) * 6 + 2, 12), 0)
        ImageDraw.Draw(small).text((1, 0), line, fill=255)
        big = small.resize((small.width * 3, small.height * 3), Image.NEAREST)
        glow_mask = big.filter(ImageFilter.GaussianBlur(3))
        colour = Image.new("RGB", big.size, phosphor)
        screen.paste(Image.new("RGB", big.size, (40, 110, 60)), (34, 30 + i * 46), glow_mask)
        screen.paste(colour, (34, 30 + i * 46), big)
    # The glass's sheen, top left.
    sheen = Image.new("L", (sw, sh), 0)
    ImageDraw.Draw(sheen).ellipse((-sw * 0.2, -sh * 0.6, sw * 0.55, sh * 0.35), fill=40)
    sheen = sheen.filter(ImageFilter.GaussianBlur(30))
    screen.paste(Image.new("RGB", (sw, sh), (255, 255, 255)), (0, 0), sheen)
    mask = Image.new("L", (sw, sh), 0)
    ImageDraw.Draw(mask).rounded_rectangle((0, 0, sw - 1, sh - 1), radius=36, fill=255)
    img.paste(screen, (left, top), mask)
    # The control strip: the maker's mark, two knobs, the power button and its light.
    strip = bottom + 34
    draw.text((left + 6, strip + 18), "S J K", fill=(120, 112, 92))
    draw.rounded_rectangle((left + 2, strip + 8, left + 92, strip + 46), radius=6,
                           outline=(176, 166, 140), width=3)
    for i in range(2):
        cx = right - 210 + i * 46
        draw.ellipse((cx - 14, strip + 14, cx + 14, strip + 42), fill=(160, 152, 128),
                     outline=(120, 112, 92), width=2)
    draw.rounded_rectangle((right - 104, strip + 10, right - 44, strip + 46), radius=8,
                           fill=(190, 182, 156), outline=(130, 122, 100), width=3)
    draw.ellipse((right - 30, strip + 20, right - 14, strip + 36), fill=(60, 230, 90))
    return img


def back(rng) -> Image.Image:
    w, h = BACK[2] - BACK[0], BACK[3] - BACK[1]
    img = plastic(rng, (h, w), (204, 194, 168))
    draw = ImageDraw.Draw(img)
    # Vent slots.
    for row in range(7):
        y = 40 + row * 22
        for col in range(5):
            x = 34 + col * 64
            draw.rounded_rectangle((x, y, x + 50, y + 8), radius=4, fill=(70, 64, 52))
    # The label, and the cable's socket.
    draw.rectangle((110, 214, 274, 290), fill=(236, 232, 220), outline=(150, 140, 120), width=2)
    for i in range(4):
        draw.line((122, 230 + i * 14, 262 - i * 20, 230 + i * 14), fill=(110, 110, 110), width=3)
    draw.rounded_rectangle((300, 236, 344, 270), radius=6, fill=(40, 38, 34))
    return img


def texture(output: Path) -> Image.Image:
    rng = np.random.default_rng(1989)
    atlas = Image.new("RGB", (ATLAS, ATLAS))
    front = face(rng)
    atlas.paste(front, FACE[:2])
    atlas.paste(back(rng), BACK[:2])
    atlas.paste(plastic(rng, (FACE[3] - BACK[3], ATLAS - BACK[0])), (BACK[0], BACK[3]))
    atlas.paste(plastic(rng, (PLASTIC[3] - PLASTIC[1], PLASTIC[2] - PLASTIC[0])), PLASTIC[:2])
    atlas.save(output / "monitor.jpg", quality=JPEG_QUALITY)
    return front


def encode_normal(n) -> int:
    # The inverse of sjk-model's `decode_normal` (stock's MD3 normal), as in
    # holocron_assets.py.
    around = round(math.atan2(n[1], n[0]) * 255 / (2 * math.pi)) & 0xFF
    from_up = round(math.acos(max(-1.0, min(1.0, n[2]))) * 255 / (2 * math.pi)) & 0xFF
    return (around << 8) | from_up


def region_uv(region) -> tuple[float, float, float, float]:
    x0, y0, x1, y1 = region
    return x0 / ATLAS, y0 / ATLAS, x1 / ATLAS, y1 / ATLAS


def corners():
    """The model's corners: the face (F), the back of the bezel (E) and the back (K),
    each top +Y, top -Y, bottom -Y, bottom +Y."""
    def ring(x, half, top):
        return [np.array(p, float) for p in
                ((x, half, top), (x, -half, top), (x, -half, BOTTOM), (x, half, BOTTOM))]
    return (ring(FRONT_X, FACE_HALF_WIDTH, FACE_TOP), ring(BEZEL_X, FACE_HALF_WIDTH, FACE_TOP),
            ring(BACK_X, BACK_HALF_WIDTH, BACK_TOP))


def monitor_md3(output: Path) -> None:
    f, e, k = corners()
    centre = (sum(f) + sum(k)) / 8  # the flight's eight corners
    assert np.allclose(centre, 0.0), centre
    # Each face: its corners top-left, top-right, bottom-right, bottom-left as seen from
    # outside (the face and the back exactly; the others in any turn, fixed below).
    faces = [
        ([f[1], f[0], f[3], f[2]], FACE),
        ([k[0], k[1], k[2], k[3]], BACK),
        # The bezel's sides.
        ([f[0], f[1], e[1], e[0]], PLASTIC),
        ([f[3], f[2], e[2], e[3]], PLASTIC),
        ([f[0], e[0], e[3], f[3]], PLASTIC),
        ([f[1], f[2], e[2], e[1]], PLASTIC),
        # The housing, tapering back.
        ([e[0], e[1], k[1], k[0]], PLASTIC),
        ([e[3], e[2], k[2], k[3]], PLASTIC),
        ([e[0], k[0], k[3], e[3]], PLASTIC),
        ([e[1], e[2], k[2], k[1]], PLASTIC),
    ]
    positions, normals, uvs, triangles = [], [], [], []
    for quad, region in faces:
        middle = sum(quad) / 4
        normal = np.cross(quad[1] - quad[0], quad[2] - quad[0])
        normal /= np.linalg.norm(normal)
        if normal @ (middle - np.array([0.0, 0.0, -1.0])) < 0:
            normal = -normal
        st = [(0, 0), (1, 0), (1, 1), (0, 1)]
        # Clockwise seen from outside (stock's front side): the corners' turn points in.
        if np.cross(quad[1] - quad[0], quad[2] - quad[0]) @ normal > 0:
            quad = quad[::-1]
            st = st[::-1]
        u0, v0, u1, v1 = region_uv(region)
        base = len(positions)
        for corner, (s, t) in zip(quad, st):
            positions.append(corner)
            normals.append(normal)
            uvs.append((u0 + (u1 - u0) * s, v0 + (v1 - v0) * t))
        triangles += [(base, base + 1, base + 2), (base, base + 2, base + 3)]

    def name(text: str, size: int) -> bytes:
        return text.encode("ascii").ljust(size, b"\0")

    shaders = name(SHADER, 64) + struct.pack("<i", 0)
    tris = b"".join(struct.pack("<3i", *t) for t in triangles)
    sts = b"".join(struct.pack("<2f", *uv) for uv in uvs)
    verts = b"".join(
        struct.pack("<3hH", *(round(c * 64) for c in p), encode_normal(n))
        for p, n in zip(positions, normals)
    )
    surface_header = 108
    ofs_tris = surface_header + len(shaders)
    ofs_st = ofs_tris + len(tris)
    ofs_verts = ofs_st + len(sts)
    surface_end = ofs_verts + len(verts)
    surface = (
        b"IDP3"
        + name("monitor", 64)
        + struct.pack(
            "<10i", 0, 1, 1, len(positions), len(triangles),
            ofs_tris, surface_header, ofs_st, ofs_verts, surface_end,
        )
        + shaders + tris + sts + verts
    )
    low = (BACK_X, -FACE_HALF_WIDTH, BOTTOM)
    high = (FRONT_X, FACE_HALF_WIDTH, FACE_TOP)
    radius = max(np.linalg.norm(p) for p in positions)
    frame = struct.pack("<10f", *low, *high, 0, 0, 0, radius) + name("monitor", 16)
    header_size = 108
    ofs_frames = header_size
    ofs_surfaces = ofs_frames + len(frame)
    end = ofs_surfaces + len(surface)
    header = (
        b"IDP3"
        + struct.pack("<i", 15)
        + name("models/sjk/monitor.md3", 64)
        + struct.pack("<9i", 0, 1, 0, 1, 0, ofs_frames, ofs_surfaces, ofs_surfaces, end)
    )
    (output / "monitor.md3").write_bytes(header + frame + surface)


def icon(output: Path, front: Image.Image) -> None:
    """The monitor flying to the right, seen a little from the side: its face, its
    housing behind it, speed lines trailing."""
    scale = 4
    size = ICON_SIZE * scale
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(canvas)
    fw = int(size * 0.62)
    fh = int(fw * 17 / 20)
    fx, fy = int(size * 0.33), int(size * 0.2)
    # The housing: tapering back and to the left of the face, its top sloping down.
    depth = int(size * 0.17)
    housing = [(fx + 4, fy), (fx - depth, fy + fh * 0.2), (fx - depth, fy + fh * 0.86),
               (fx + 4, fy + fh)]
    draw.polygon(housing, fill=(170, 160, 134, 255), outline=(96, 88, 70, 255), width=6)
    canvas.paste(front.resize((fw, fh), Image.LANCZOS).convert("RGBA"), (fx, fy))
    draw.rectangle((fx, fy, fx + fw, fy + fh), outline=(96, 88, 70, 255), width=6)
    # Speed lines.
    for length, y in ((0.26, 0.32), (0.2, 0.47), (0.24, 0.62)):
        x1 = fx - depth - size * 0.035
        draw.rounded_rectangle(
            (x1 - size * length, size * y, x1, size * y + size * 0.045),
            radius=size * 0.022, fill=(255, 238, 214, 255),
        )
    canvas = canvas.rotate(-10, resample=Image.BICUBIC, center=(size / 2, size / 2))
    alpha = canvas.split()[3]
    halo = alpha.filter(ImageFilter.MaxFilter(9)).filter(ImageFilter.GaussianBlur(3))
    ground = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    ground.putalpha(halo.point(lambda v: int(v * 0.7)))
    ground.alpha_composite(canvas)
    ground.resize((ICON_SIZE, ICON_SIZE), Image.LANCZOS).save(output / "toy_monitor.png")


def write_wav(path: Path, samples: np.ndarray) -> None:
    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.9
    data = (np.clip(samples, -1, 1) * 32767).astype("<i2").tobytes()
    with wave.open(str(path), "wb") as out:
        out.setnchannels(1)
        out.setsampwidth(2)
        out.setframerate(RATE)
        out.writeframes(data)


def lowpass(signal: np.ndarray, cutoff) -> np.ndarray:
    """A one-pole low pass; `cutoff` in Hz, one value or one per sample."""
    cutoff = np.broadcast_to(np.asarray(cutoff, float), signal.shape)
    out = np.empty_like(signal)
    y = 0.0
    for i, x in enumerate(signal):
        a = 1.0 - math.exp(-2 * math.pi * cutoff[i] / RATE)
        y += a * (x - y)
        out[i] = y
    return out


def sounds(output: Path) -> None:
    rng = np.random.default_rng(7)
    # The throw: noise swept up then down through a band, swelling and fading.
    n = int(RATE * 0.42)
    t = np.arange(n) / RATE
    swell = np.sin(np.pi * t / t[-1]) ** 2
    sweep = 500 + 2600 * np.sin(np.pi * t / t[-1])
    white = rng.standard_normal(n)
    band = lowpass(white, sweep) - lowpass(white, sweep * 0.25)
    write_wav(output / "throw.wav", band * swell)
    # The bonk: a hollow plastic body's thunk, a click, and the glass ringing.
    n = int(RATE * 0.7)
    t = np.arange(n) / RATE
    thunk = (np.sin(2 * np.pi * 135 * t) * np.exp(-t * 18)
             + 0.6 * np.sin(2 * np.pi * 212 * t) * np.exp(-t * 26)
             + 0.35 * np.sin(2 * np.pi * 420 * t) * np.exp(-t * 40))
    click = lowpass(rng.standard_normal(n), 3500) * np.exp(-t * 120) * 1.4
    ring = (np.sin(2 * np.pi * 2310 * t) + 0.7 * np.sin(2 * np.pi * 3170 * t)
            + 0.4 * np.sin(2 * np.pi * 4480 * t)) * np.exp(-t * 7) * 0.12
    write_wav(output / "bonk.wav", thunk + click + ring)
    # The knock: a short heavy clunk and the rattle of what is loose inside.
    n = int(RATE * 0.45)
    t = np.arange(n) / RATE
    clunk = (np.sin(2 * np.pi * 92 * t) * np.exp(-t * 22)
             + 0.5 * np.sin(2 * np.pi * 180 * t) * np.exp(-t * 35))
    rattle = np.zeros(n)
    for start in (0.0, 0.035, 0.07, 0.11, 0.16):
        at = int(start * RATE)
        burst = lowpass(rng.standard_normal(n - at), 2500) * np.exp(-np.arange(n - at) / RATE * 90)
        rattle[at:] += burst * (0.5 - start * 2.2)
    write_wav(output / "knock.wav", clunk + lowpass(rattle, 4000) * 0.8)
    charged_sounds(output)


def charged_sounds(output: Path) -> None:
    """The charged throw's sounds and the thrower's hit sound (their own seed, so the
    sounds above stay as they were)."""
    rng = np.random.default_rng(11)
    # The charge: a tube warming up, a hum rising from 110 to 880 Hz with its harmonics,
    # buzzing faster and crackling more as it rises, then a bright two-note ready ping.
    rise = 1.0
    n = int(RATE * (rise + 0.3))
    t = np.arange(n) / RATE
    up = np.clip(t / rise, 0, 1)
    pitch = 110 * 8 ** (up ** 1.4)
    phase = 2 * np.pi * np.cumsum(pitch) / RATE
    hum = np.sin(phase) + 0.45 * np.sin(2 * phase) + 0.25 * np.sin(3 * phase)
    buzz = 0.75 + 0.25 * np.sin(2 * np.pi * np.cumsum(8 + 30 * up) / RATE)
    crackle = lowpass(rng.standard_normal(n), 6000) * (rng.random(n) < 0.004 + 0.02 * up)
    hold = np.where(t < rise, 0.25 + 0.75 * up, np.exp(-(t - rise) * 14))
    ping_t = np.clip(t - rise, 0, None)
    ping = ((np.sin(2 * np.pi * 1320 * ping_t) + 0.6 * np.sin(2 * np.pi * 1980 * ping_t))
            * np.exp(-ping_t * 12) * (t >= rise) * 0.7)
    write_wav(output / "charge.wav", (hum * buzz + crackle * 2.5) * hold * 0.6 + ping)
    # The charged throw: a heavy thump, then a deep long whoosh with a low roar under it.
    n = int(RATE * 0.7)
    t = np.arange(n) / RATE
    swell = np.sin(np.pi * np.clip(t / 0.6, 0, 1)) ** 1.5
    sweep = 300 + 3200 * np.sin(np.pi * np.clip(t / 0.6, 0, 1))
    white = rng.standard_normal(n)
    band = lowpass(white, sweep) - lowpass(white, sweep * 0.2)
    roar = lowpass(rng.standard_normal(n), 220) * swell * 4.0
    thump = np.sin(2 * np.pi * 70 * t) * np.exp(-t * 16) * 1.6
    write_wav(output / "throw_hard.wav", band * swell + roar + thump)
    # The hit: a short bright tick the thrower hears when their monitor strikes someone.
    n = int(RATE * 0.22)
    t = np.arange(n) / RATE
    tick = ((np.sin(2 * np.pi * 1870 * t) + 0.5 * np.sin(2 * np.pi * 2810 * t))
            * np.exp(-t * 28))
    click = lowpass(rng.standard_normal(n), 7000) * np.exp(-t * 400)
    write_wav(output / "hit.wav", tick + click * 0.8)
    saver_sounds(output)


def saver_sounds(output: Path) -> None:
    """The Screen Saver's sounds (their own seed, so the sounds above stay as they
    were)."""
    rng = np.random.default_rng(13)
    # Rising: a CRT's degauss, a heavy 60 Hz boing whose wobble dies away, a click, and
    # a shimmer of bright notes climbing over it.
    n = int(RATE * 1.3)
    t = np.arange(n) / RATE
    wobble = 1 + 0.08 * np.sin(2 * np.pi * 9 * t) * np.exp(-t * 3)
    phase = 2 * np.pi * np.cumsum(60 * wobble) / RATE
    boing = (np.sin(phase) + 0.5 * np.sin(2 * phase) + 0.3 * np.sin(3 * phase)) * np.exp(-t * 3.2)
    click = lowpass(rng.standard_normal(n), 5000) * np.exp(-t * 300) * 1.2
    shimmer = np.zeros(n)
    for index, note in enumerate((880, 1109, 1319, 1760, 2217)):
        start = 0.12 + index * 0.07
        at = np.clip(t - start, 0, None)
        shimmer += (np.sin(2 * np.pi * note * at) * np.exp(-at * 4.5)
                    * (t >= start) * (0.35 - index * 0.03))
    write_wav(output / "saver_on.wav", boing * 1.3 + click + shimmer)
    # Ending: a CRT switched off, its high whine falling fast to a low hum and a pop.
    n = int(RATE * 0.7)
    t = np.arange(n) / RATE
    pitch = 200 + 6000 * np.exp(-t * 9)
    phase = 2 * np.pi * np.cumsum(pitch) / RATE
    whine = np.sin(phase) * np.exp(-t * 4) * 0.5
    hum = np.sin(2 * np.pi * 60 * t) * np.exp(-t * 6) * 0.6
    pop = lowpass(rng.standard_normal(n), 1500) * np.exp(-np.clip(t - 0.04, 0, None) * 80) * (t >= 0.04)
    write_wav(output / "saver_off.wav", whine + hum + pop * 0.8)


def main() -> None:
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    output = Path(sys.argv[1])
    output.mkdir(parents=True, exist_ok=True)
    front = texture(output)
    monitor_md3(output)
    icon(output, front)
    sounds(output)


if __name__ == "__main__":
    main()
