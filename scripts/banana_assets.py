"""Make the Banana peel toy's game files (`toy_banana`, SJK's own).

The Banana peel is a meme toy (Kushina's idea, 29/10/2026): players drop banana peels
on the floor, and a player who has the toy on and walks onto one slips and falls as a
ragdoll. Nothing here comes from another game: the peel is modelled from a few
parametric strips, its picture drawn from noise, and the sounds are synthesized. This
script writes, into the output folder:

- `banana.jpg`: the peel's picture, 256x256: the yellow outer skin with brown speckles
  (top), the pale inner flesh side with its fibres (bottom left), the brown stem (bottom
  right);
- `banana.md3`: a peel lying on the floor, its origin where it rests on it (+Z up): a
  low mound, the peel's closed end, with the stem's stub poking up and back (-X), and
  four floppy strips fanning out from it ahead (+X) and to the sides like fingers, two
  of them curling up at their tips to show the pale inside; about 19 units long, 12
  wide and 6 tall, centred on its origin;
- `toy_banana.png`: the toy's icon, 128x128 like `gfx/mp/f_icon_*`: the model itself,
  drawn from the side and a little above by the small renderer below, on transparency;
- `slip.wav`: a cartoon slip, a squeaky skid and a slide whistle zipping up then
  falling; `drop.wav`: the soft wet flop of a peel landing; both 22050 Hz mono 16-bit.

    python scripts/banana_assets.py crates/sjk-viewer/assets/banana

With `--preview <folder>` it also writes the model drawn from a few sides
(`banana_preview.png`), to look at it without the game. The output is deterministic
(fixed seeds). Needs Pillow and numpy (pip install pillow numpy). The shader
(`banana.shader`) is written by hand.
"""

import math
import struct
import sys
import wave
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFilter

SHADER = "models/sjk/banana"
ATLAS = 256
# The picture's regions: the skin (strips run down it, the mound's end at its top), the
# flesh side and the stem.
OUTER = (0, 0, 256, 128)
INNER = (0, 128, 192, 256)
STEM = (192, 128, 256, 256)
YELLOW = (247, 212, 58)
CREAM = (246, 238, 198)
BROWN = (118, 96, 46)
ICON_SIZE = 128
JPEG_QUALITY = 92
RATE = 22050

# The model, in game units. The mound where the strips join: its radius and height.
MOUND_RADIUS = 2.2
MOUND_TOP = 3.0
# Each strip: the way it points (degrees from +X: a fan ahead, the stem behind), its
# length from the middle, whether its tip curls up, and the phase of its sideways wave.
STRIPS = [
    (-100.0, 8.0, True, 0.0),
    (-35.0, 9.2, False, 1.3),
    (30.0, 9.0, True, 2.1),
    (98.0, 7.8, False, 0.7),
]
# The whole peel is drawn this much bigger than the sizes here (a real peel's would be
# lost at a player's distance).
SCALE = 1.25
# Where a strip leaves the mound, how high it starts and how high it lies; how high a
# curling tip rises; how thick a strip is.
STRIP_START = 1.5
STRIP_HIGH = 2.5
STRIP_LOW = 0.32
CURL = 2.6
THICKNESS = 0.24
# A strip's width along it (at its start, widest, three quarters on, tip), and how much
# its edges rise over its middle (the peel's hollow) at its widest.
WIDTHS = [(0.0, 2.3), (0.3, 3.1), (0.75, 2.4), (1.0, 0.2)]
HOLLOW = 0.5
# Rings along a strip, and the stem: its radius, length and lean back (-X).
RINGS = 8
STEM_RADIUS = 0.5
STEM_LENGTH = 2.3
STEM_LEAN = 0.55


def noise(rng, size, scale: float) -> np.ndarray:
    """Smooth value noise of `size` (h, w), about `scale` pixels per feature, in 0..1."""
    h, w = size
    gh, gw = int(h / scale) + 2, int(w / scale) + 2
    grid = Image.fromarray((rng.random((gh, gw)) * 255).astype(np.uint8))
    big = grid.resize((int(gw * scale), int(gh * scale)), Image.BICUBIC)
    return np.asarray(big, dtype=float)[:h, :w] / 255.0


def skin(rng) -> Image.Image:
    """The outer skin: ripe yellow, a little green toward the stem end (the top), its
    ridges shaded, brown speckles, and darker, browner toward the tips (the bottom)."""
    x0, y0, x1, y1 = OUTER
    w, h = x1 - x0, y1 - y0
    v = np.linspace(0, 1, h)[:, None] * np.ones((1, w))
    u = np.ones((h, 1)) * np.linspace(0, 1, w)[None, :]
    rgb = np.array(YELLOW, float)[None, None, :] * np.ones((h, w, 1))
    # Green-yellow near the stem end.
    green = np.clip(1 - v / 0.22, 0, 1)[:, :, None]
    rgb = rgb * (1 - 0.35 * green) + np.array((176, 196, 64), float) * 0.35 * green
    # The ridges along the skin: darker at the edges and two faint lines between.
    ridge = 1 - 0.16 * np.abs(np.cos(u * np.pi * 3)) ** 6 - 0.2 * (np.abs(u - 0.5) * 2) ** 3
    rgb *= ridge[:, :, None]
    # Blotches and fine grain.
    rgb *= (0.92 + 0.1 * noise(rng, (h, w), 18) + 0.04 * noise(rng, (h, w), 2))[:, :, None]
    # Browner toward the tips.
    tip = np.clip((v - 0.7) / 0.3, 0, 1)[:, :, None] ** 1.5
    rgb = rgb * (1 - 0.45 * tip) + np.array((150, 104, 40), float) * 0.45 * tip
    # The dark end of each strip.
    end = np.clip((v - 0.9) / 0.06, 0, 1)[:, :, None]
    rgb = rgb * (1 - 0.7 * end) + np.array((74, 52, 22), float) * 0.7 * end
    img = Image.fromarray(np.clip(rgb, 0, 255).astype(np.uint8))
    draw = ImageDraw.Draw(img)
    # Speckles: small brown dots, a few larger ones.
    for _ in range(170):
        sx, sy = rng.random() * w, rng.random() * h
        r = 0.6 + rng.random() * (1.2 if rng.random() < 0.85 else 2.6)
        shade = int(70 + rng.random() * 40)
        draw.ellipse((sx - r, sy - r * 0.8, sx + r, sy + r * 0.8),
                     fill=(shade + 30, shade, 24))
    return img.filter(ImageFilter.GaussianBlur(0.5))


def flesh(rng) -> Image.Image:
    """The inner side: pale cream, its fibres running along it, yellower at the edges."""
    x0, y0, x1, y1 = INNER
    w, h = x1 - x0, y1 - y0
    u = np.ones((h, 1)) * np.linspace(0, 1, w)[None, :]
    rgb = np.array(CREAM, float)[None, None, :] * np.ones((h, w, 1))
    edge = (np.abs(u - 0.5) * 2) ** 4
    rgb = rgb * (1 - 0.3 * edge[:, :, None]) + np.array(YELLOW, float) * 0.3 * edge[:, :, None]
    # Fibres: noise stretched along the strip.
    stretched = Image.fromarray((rng.random((8, w)) * 255).astype(np.uint8)).resize(
        (w, h), Image.BICUBIC)
    fibres = np.asarray(stretched, float) / 255.0
    rgb *= (0.94 + 0.08 * fibres + 0.03 * noise(rng, (h, w), 6))[:, :, None]
    return Image.fromarray(np.clip(rgb, 0, 255).astype(np.uint8))


def stem(rng) -> Image.Image:
    """The stem: dry brown, darker at its cut end (the bottom)."""
    x0, y0, x1, y1 = STEM
    w, h = x1 - x0, y1 - y0
    v = np.linspace(0, 1, h)[:, None] * np.ones((1, w))
    rgb = np.array(BROWN, float)[None, None, :] * np.ones((h, w, 1))
    rgb *= (0.85 + 0.25 * noise(rng, (h, w), 5))[:, :, None]
    rgb *= (1 - 0.45 * np.clip((v - 0.8) / 0.2, 0, 1))[:, :, None]
    return Image.fromarray(np.clip(rgb, 0, 255).astype(np.uint8))


def texture(output: Path) -> Image.Image:
    rng = np.random.default_rng(2910)
    atlas = Image.new("RGB", (ATLAS, ATLAS))
    atlas.paste(skin(rng), OUTER[:2])
    atlas.paste(flesh(rng), INNER[:2])
    atlas.paste(stem(rng), STEM[:2])
    atlas.save(output / "banana.jpg", quality=JPEG_QUALITY)
    return atlas


def region_uv(region, s: float, t: float) -> tuple[float, float]:
    """The atlas coordinates of (`s`, `t`), each 0..1, within `region`, inset half a
    pixel so the regions do not bleed into each other."""
    x0, y0, x1, y1 = region
    inset = 1.5
    return ((x0 + inset + (x1 - x0 - 2 * inset) * s) / ATLAS,
            (y0 + inset + (y1 - y0 - 2 * inset) * t) / ATLAS)


def smooth(edge0: float, edge1: float, x: float) -> float:
    t = min(1.0, max(0.0, (x - edge0) / (edge1 - edge0)))
    return t * t * (3 - 2 * t)


def width_at(s: float) -> float:
    for (s0, w0), (s1, w1) in zip(WIDTHS, WIDTHS[1:]):
        if s <= s1:
            t = (s - s0) / (s1 - s0)
            return w0 + (w1 - w0) * smooth(0, 1, t)
    return WIDTHS[-1][1]


class Mesh:
    """Triangles with a corner each of position, normal and texture coordinates."""

    def __init__(self):
        self.positions, self.normals, self.uvs, self.triangles = [], [], [], []

    def vertex(self, position, normal, uv) -> int:
        # A copy: corners made from the same point must not move together.
        self.positions.append(np.array(position, float))
        n = np.asarray(normal, float)
        self.normals.append(n / (np.linalg.norm(n) or 1.0))
        self.uvs.append(uv)
        return len(self.positions) - 1

    def triangle(self, a: int, b: int, c: int, outward) -> None:
        """A triangle facing `outward`, wound clockwise seen from that side (stock's
        front side), as monitor_assets.py does."""
        p = self.positions
        if np.cross(p[b] - p[a], p[c] - p[a]) @ np.asarray(outward, float) > 0:
            b, c = c, b
        self.triangles.append((a, b, c))

    def quad(self, a, b, c, d, outward) -> None:
        self.triangle(a, b, c, outward)
        self.triangle(a, c, d, outward)


def strip_grid(angle: float, length: float, curls: bool, phase: float):
    """A strip's upper surface: rings along it of three points across (left, middle,
    right), with each ring's along-way (s) and the strip's sideways."""
    forward = np.array((math.cos(math.radians(angle)), math.sin(math.radians(angle)), 0.0))
    side = np.array((-forward[1], forward[0], 0.0))
    rings = []
    for i in range(RINGS + 1):
        s = i / RINGS
        r = STRIP_START + (length - STRIP_START) * s
        z = STRIP_LOW + (STRIP_HIGH - STRIP_LOW) * (1 - smooth(0.0, 0.42, s))
        if curls:
            z += CURL * smooth(0.62, 1.0, s) ** 2
            # A curling tip folds back a little over itself.
            r -= 0.9 * smooth(0.8, 1.0, s)
        wave = 0.45 * math.sin(math.pi * 1.4 * s + phase) * smooth(0.1, 0.5, s)
        centre = forward * r + side * wave + np.array((0.0, 0.0, z))
        half = width_at(s) / 2
        lift = HOLLOW * width_at(s) / 3.1 * (1 - 0.6 * smooth(0.6, 1.0, s))
        ring = [centre - side * half + np.array((0, 0, lift)), centre,
                centre + side * half + np.array((0, 0, lift))]
        rings.append((s, ring))
    return rings, side


def grid_normals(rings):
    """Each point's normal on the upper surface: across the strip by along it, up."""
    normals = []
    for i, (_, ring) in enumerate(rings):
        before = rings[max(i - 1, 0)][1]
        after = rings[min(i + 1, len(rings) - 1)][1]
        row = []
        for j in range(3):
            along = after[j] - before[j]
            across = ring[min(j + 1, 2)] - ring[max(j - 1, 0)]
            n = np.cross(along, across)
            if n[2] < 0:
                n = -n
            row.append(n / (np.linalg.norm(n) or 1.0))
        normals.append(row)
    return normals


def add_strip(mesh: Mesh, angle: float, length: float, curls: bool, phase: float) -> None:
    rings, side = strip_grid(angle, length, curls, phase)
    normals = grid_normals(rings)
    # The upper side is the skin (yellow), the lower the flesh side (cream), a thickness
    # under it; their rims join them along both edges.
    upper, lower = [], []
    for (s, ring), row in zip(rings, normals):
        upper.append([mesh.vertex(p, n, region_uv(OUTER, j / 2, 0.08 + 0.92 * s))
                      for j, (p, n) in enumerate(zip(ring, row))])
        lower.append([mesh.vertex(p - n * THICKNESS, -n, region_uv(INNER, j / 2, s))
                      for j, (p, n) in enumerate(zip(ring, row))])
    for i in range(RINGS):
        for j in range(2):
            up = normals[i][j] + normals[i + 1][j + 1]
            mesh.quad(upper[i][j], upper[i][j + 1], upper[i + 1][j + 1], upper[i + 1][j], up)
            mesh.quad(lower[i][j], lower[i][j + 1], lower[i + 1][j + 1], lower[i + 1][j], -up)
    for j, way in ((0, -side), (2, side)):
        rim = []
        for (s, ring), row in zip(rings, normals):
            p, n = ring[j], row[j]
            out = way - n * (way @ n)
            rim.append((mesh.vertex(p, out, region_uv(INNER, 0.95 if j else 0.05, s)),
                        mesh.vertex(p - n * THICKNESS, out,
                                    region_uv(INNER, 0.9 if j else 0.1, s))))
        for i in range(RINGS):
            mesh.quad(rim[i][0], rim[i + 1][0], rim[i + 1][1], rim[i][1], way)


def add_mound(mesh: Mesh) -> None:
    """The mound where the strips join: a low dome of skin."""
    sides = 10
    top = mesh.vertex((0.2, 0.0, MOUND_TOP), (0, 0, 1), region_uv(OUTER, 0.5, 0.0))
    rings = []
    for height, radius, t in ((0.82, 0.62, 0.03), (0.45, 0.92, 0.07), (0.0, 1.0, 0.1)):
        ring = []
        for k in range(sides + 1):
            a = 2 * math.pi * k / sides
            x, y = math.cos(a) * MOUND_RADIUS * radius, math.sin(a) * MOUND_RADIUS * radius
            z = STRIP_LOW + (MOUND_TOP - STRIP_LOW) * height
            n = np.array((x, y, (1.2 - height) * 1.5 + 0.3))
            ring.append(mesh.vertex((x, y, z), n, region_uv(OUTER, k / sides, t)))
        rings.append(ring)
    for k in range(sides):
        mesh.triangle(top, rings[0][k], rings[0][k + 1], (0, 0, 1))
    for upper, lower in zip(rings, rings[1:]):
        for k in range(sides):
            a = 2 * math.pi * (k + 0.5) / sides
            out = (math.cos(a), math.sin(a), 0.6)
            mesh.quad(upper[k], upper[k + 1], lower[k + 1], lower[k], out)


def add_stem(mesh: Mesh) -> None:
    """The stem's stub: a six-sided post from the mound's top, leaning back (-X), its
    cut end flat."""
    sides = 6
    base = np.array((0.0, 0.0, MOUND_TOP - 0.4))
    axis = np.array((-STEM_LEAN, 0.0, 1.0))
    axis /= np.linalg.norm(axis)
    end = base + axis * STEM_LENGTH
    a1 = np.cross(axis, (0.0, 1.0, 0.0))
    a1 /= np.linalg.norm(a1)
    a2 = np.cross(axis, a1)
    ring_base, ring_end = [], []
    for k in range(sides + 1):
        a = 2 * math.pi * k / sides
        out = a1 * math.cos(a) + a2 * math.sin(a)
        # A stem narrows a little to its end.
        ring_base.append(mesh.vertex(base + out * STEM_RADIUS * 1.15, out,
                                     region_uv(STEM, k / sides, 0.0)))
        ring_end.append(mesh.vertex(end + out * STEM_RADIUS, out,
                                    region_uv(STEM, k / sides, 0.85)))
    for k in range(sides):
        a = 2 * math.pi * (k + 0.5) / sides
        out = a1 * math.cos(a) + a2 * math.sin(a)
        mesh.quad(ring_base[k], ring_base[k + 1], ring_end[k + 1], ring_end[k], out)
    middle = mesh.vertex(end, axis, region_uv(STEM, 0.5, 1.0))
    cap = [mesh.vertex(end + (a1 * math.cos(2 * math.pi * k / sides)
                              + a2 * math.sin(2 * math.pi * k / sides)) * STEM_RADIUS,
                       axis, region_uv(STEM, 0.5 + 0.4 * math.cos(2 * math.pi * k / sides),
                                       0.97))
           for k in range(sides + 1)]
    for k in range(sides):
        mesh.triangle(middle, cap[k], cap[k + 1], axis)


def build() -> Mesh:
    mesh = Mesh()
    for angle, length, curls, phase in STRIPS:
        add_strip(mesh, angle, length, curls, phase)
    add_mound(mesh)
    add_stem(mesh)
    positions = np.array(mesh.positions)
    low, high = positions.min(axis=0), positions.max(axis=0)
    # It rests on the floor at its origin, centred over it: nothing below it.
    shift = np.array(((low[0] + high[0]) / 2, (low[1] + high[1]) / 2, low[2] - 0.02))
    for p in mesh.positions:
        p[:] = (p - shift) * SCALE
    return mesh


def encode_normal(n) -> int:
    # The inverse of sjk-model's `decode_normal` (stock's MD3 normal), as in
    # monitor_assets.py.
    around = round(math.atan2(n[1], n[0]) * 255 / (2 * math.pi)) & 0xFF
    from_up = round(math.acos(max(-1.0, min(1.0, n[2]))) * 255 / (2 * math.pi)) & 0xFF
    return (around << 8) | from_up


def banana_md3(output: Path, mesh: Mesh) -> None:
    def name(text: str, size: int) -> bytes:
        return text.encode("ascii").ljust(size, b"\0")

    positions, normals = mesh.positions, mesh.normals
    shaders = name(SHADER, 64) + struct.pack("<i", 0)
    tris = b"".join(struct.pack("<3i", *t) for t in mesh.triangles)
    sts = b"".join(struct.pack("<2f", *uv) for uv in mesh.uvs)
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
        + name("banana", 64)
        + struct.pack(
            "<10i", 0, 1, 1, len(positions), len(mesh.triangles),
            ofs_tris, surface_header, ofs_st, ofs_verts, surface_end,
        )
        + shaders + tris + sts + verts
    )
    low = np.min(positions, axis=0)
    high = np.max(positions, axis=0)
    radius = max(np.linalg.norm(p) for p in positions)
    frame = struct.pack("<10f", *low, *high, 0, 0, 0, radius) + name("banana", 16)
    header_size = 108
    ofs_frames = header_size
    ofs_surfaces = ofs_frames + len(frame)
    end = ofs_surfaces + len(surface)
    header = (
        b"IDP3"
        + struct.pack("<i", 15)
        + name("models/sjk/banana.md3", 64)
        + struct.pack("<9i", 0, 1, 0, 1, 0, ofs_frames, ofs_surfaces, ofs_surfaces, end)
    )
    (output / "banana.md3").write_bytes(header + frame + surface)
    print(f"banana.md3: {len(positions)} corners, {len(mesh.triangles)} triangles, "
          f"{high[0] - low[0]:.1f} x {high[1] - low[1]:.1f} x {high[2] - low[2]:.1f} units")


def draw_model(mesh: Mesh, atlas: Image.Image, size: int, yaw: float, pitch: float,
               fill: float = 0.82) -> Image.Image:
    """The model drawn by a small painter's renderer: each triangle in the picture's
    colour at its middle, lit from above and a side, farthest first; seen from `yaw`
    degrees round and `pitch` degrees above. On transparency, `size` pixels square."""
    pixels = np.asarray(atlas, float)
    cy, sy = math.cos(math.radians(yaw)), math.sin(math.radians(yaw))
    cp, sp = math.cos(math.radians(pitch)), math.sin(math.radians(pitch))
    # Camera axes: right, up, toward the viewer.
    forward = np.array((-cy * cp, -sy * cp, -sp))
    right = np.array((-sy, cy, 0.0))
    up = np.cross(right, forward)
    positions = np.array(mesh.positions)
    centre = (positions.min(axis=0) + positions.max(axis=0)) / 2
    xs = (positions - centre) @ right
    ys = (positions - centre) @ up
    depth = (positions - centre) @ forward
    extent = max(np.abs(xs).max(), np.abs(ys).max())
    scale = size * fill / 2 / extent
    light = np.array((0.35, -0.45, 0.82))
    light /= np.linalg.norm(light)
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(canvas)
    order = sorted(range(len(mesh.triangles)),
                   key=lambda t: -max(depth[i] for i in mesh.triangles[t]))
    for t in order:
        a, b, c = mesh.triangles[t]
        uv = (np.array(mesh.uvs[a]) + mesh.uvs[b] + mesh.uvs[c]) / 3
        colour = pixels[min(int(uv[1] * ATLAS), ATLAS - 1), min(int(uv[0] * ATLAS), ATLAS - 1)]
        n = mesh.normals[a] + mesh.normals[b] + mesh.normals[c]
        n /= np.linalg.norm(n) or 1.0
        # Two-sided, as the shader draws it: the side toward the viewer is lit.
        if n @ forward > 0:
            n = -n
        shade = 0.45 + 0.65 * max(0.0, n @ light)
        rgb = tuple(int(min(255, v * shade)) for v in colour)
        points = [(size / 2 + xs[i] * scale, size / 2 - ys[i] * scale) for i in (a, b, c)]
        draw.polygon(points, fill=rgb + (255,))
    return canvas


def icon(output: Path, mesh: Mesh, atlas: Image.Image) -> None:
    """The model seen from the side and a little above, a dark outline round it."""
    scale = 4
    size = ICON_SIZE * scale
    peel = draw_model(mesh, atlas, size, yaw=60.0, pitch=34.0, fill=0.94)
    alpha = peel.split()[3]
    outline = alpha.filter(ImageFilter.MaxFilter(13))
    ground = Image.new("RGBA", (size, size), (52, 34, 10, 0))
    ground.putalpha(outline)
    halo = outline.filter(ImageFilter.MaxFilter(9)).filter(ImageFilter.GaussianBlur(4))
    shadow = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    shadow.putalpha(halo.point(lambda v: int(v * 0.6)))
    shadow.alpha_composite(ground)
    shadow.alpha_composite(peel)
    shadow.resize((ICON_SIZE, ICON_SIZE), Image.LANCZOS).save(output / "toy_banana.png")


def preview(folder: Path, mesh: Mesh, atlas: Image.Image) -> None:
    """The model from five sides on a grey ground, to look at without the game."""
    views = [(60, 34), (250, 25), (90, 15), (0, 89), (160, 8)]
    cell = 384
    sheet = Image.new("RGB", (cell * len(views), cell), (92, 96, 104))
    for i, (yaw, pitch) in enumerate(views):
        sheet.paste(draw_model(mesh, atlas, cell, yaw, pitch), (i * cell, 0),
                    draw_model(mesh, atlas, cell, yaw, pitch))
    sheet.paste(atlas.resize((cell // 2, cell // 2)), (0, 0))
    folder.mkdir(parents=True, exist_ok=True)
    sheet.save(folder / "banana_preview.png")


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


def tone(pitch: np.ndarray) -> np.ndarray:
    """A whistle's tone following `pitch` (Hz per sample): a sine, a touch of its
    second and third harmonics."""
    phase = 2 * np.pi * np.cumsum(pitch) / RATE
    return np.sin(phase) + 0.18 * np.sin(2 * phase) + 0.06 * np.sin(3 * phase)


def sounds(output: Path) -> None:
    rng = np.random.default_rng(29)
    # The slip: a squeaky skid of the shoe, then a slide whistle zipping up (the feet
    # flying) and dropping (the fall), wobbling a little.
    n = int(RATE * 0.62)
    t = np.arange(n) / RATE
    skid_env = np.clip(t / 0.008, 0, 1) * np.exp(-np.clip(t - 0.02, 0, None) * 30)
    skid_pitch = 1500 + 500 * np.sin(2 * np.pi * 38 * t)
    skid = (tone(skid_pitch) * 0.6
            + (lowpass(rng.standard_normal(n), 4200) - lowpass(rng.standard_normal(n), 900))
            * 0.9) * skid_env
    rise_start, rise, fall = 0.05, 0.27, 0.24
    up = np.clip((t - rise_start) / rise, 0, 1)
    down = np.clip((t - rise_start - rise) / fall, 0, 1)
    pitch = np.where(t < rise_start + rise, 380 * (2300 / 380) ** (up ** 0.8),
                     2300 * (620 / 2300) ** (down ** 1.3))
    pitch *= 1 + 0.025 * np.sin(2 * np.pi * 11 * t)
    whistle_env = (np.clip((t - rise_start) / 0.02, 0, 1)
                   * np.where(t < rise_start + rise, 1.0, 0.75)
                   * np.clip((rise_start + rise + fall - t) / 0.06, 0, 1))
    breath = lowpass(rng.standard_normal(n), 3000) * 0.05
    write_wav(output / "slip.wav", skid + (tone(pitch) + breath) * whistle_env * 0.8)
    # The drop: a soft, wet flop: a low thud and a short splat of noise.
    n = int(RATE * 0.26)
    t = np.arange(n) / RATE
    thud = np.sin(2 * np.pi * (150 - 60 * np.clip(t / 0.08, 0, 1)) * t) * np.exp(-t * 32)
    splat = lowpass(rng.standard_normal(n), 1800) * np.exp(-t * 55) * 0.9
    smack = lowpass(rng.standard_normal(n), 5000) * np.exp(-t * 240) * 0.4
    write_wav(output / "drop.wav", thud + splat + smack)


def main() -> None:
    args = sys.argv[1:]
    preview_folder = None
    if "--preview" in args:
        at = args.index("--preview")
        preview_folder = Path(args[at + 1])
        del args[at:at + 2]
    if len(args) != 1:
        sys.exit(__doc__)
    output = Path(args[0])
    output.mkdir(parents=True, exist_ok=True)
    atlas = texture(output)
    mesh = build()
    banana_md3(output, mesh)
    icon(output, mesh, atlas)
    sounds(output)
    if preview_folder is not None:
        preview(preview_folder, mesh, atlas)


if __name__ == "__main__":
    main()
