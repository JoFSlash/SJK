#!/usr/bin/env python3
"""Build sjk_sith1, a dark Sith temple with a lava moat and hooded statues.

Usage:
    python scripts/sith_map.py [--gamedata DIR] [--q3map2 EXE]
        [--out DIR] [--install] [--no-compile] [--preview PNG]

Writes, in --out (default S:/Tools/sith):

- home/base/maps/sjk_sith1.map: the source map. A hall 2048 units square and
  768 high with chamfered corners; in the middle an octagonal duelling
  platform with an emblem dais, ringed by a lava moat that four bridges
  cross; a throne on a stepped dais in the north; six octagonal pillars,
  ceiling beams, six braziers, glowing rune bands on every wall, banners, and
  six hooded statues on pedestals. Two `info_player_duel` and eight
  `info_player_deathmatch` spawn points.
- home/base/models/map_objects/sjk_sith/statue.md3: the statue, a robed and
  hooded figure resting its hands on a saber hilt, made by sith_models.py. q3map2
  bakes each `misc_model` into the BSP with lightmaps (spawnflags 4), so the
  game never loads the model itself. An invisible octagonal player clip
  stands in for its shape: q3map2's own model clip (spawnflags 2) made over
  a thousand brushes per statue.
- sjk_sith1.pk3: the compiled map with its textures (all drawn here, none
  taken from the game) and shaders/sjk_sith.shader. --install copies it to
  GameData/base (`devmap sjk_sith1`).
- --preview: a plan of the brushes (shaded by height, lava in orange),
  statues and spawn points.

Light comes from emissive surfaces only (lava, braziers, runes, the emblem's
inlay, dim lamps under the beam crossings): q3map2 strips `light` entities from the BSP, and SJK's real-time
lighting finds its lamps in emissive materials (see docs/rendering.md, Arena
test map).

Needs Python 3 with Pillow and NumPy, and q3map2 from NetRadiant-custom
(`-game ja`) unless --no-compile.
"""
import argparse
import math
import random
import shutil
import sys
import zipfile
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFilter

from arena_map import compile_map
from sith_models import STATUES as FIGURES, statue_mesh, write_md3

DEFAULT_GAMEDATA = r"C:\Program Files (x86)\Steam\steamapps\common\Jedi Academy\GameData"
DEFAULT_Q3MAP2 = r"S:\Tools\netradiant-custom\q3map2.exe"
DEFAULT_OUT = r"S:\Tools\sith"

MAP = "sjk_sith1"
PREFIX = "sjk_sith"  # textures/<PREFIX>/, shaders/<PREFIX>.shader
STATUE = "models/map_objects/%s/statue" % PREFIX

H = 1024  # the hall's interior spans -H..H on x and y
HEIGHT = 768
WALL = 16
PIT = -112  # moat floor; the lava's surface is LAVA_TOP
LAVA_TOP = -48
CHAMFER = 384
MOAT_IN, MOAT_OUT = 448, 640  # apothems of the platform and the moat's outer edge
DAIS = 192
PLATFORM_TOP, DAIS_TOP = 16, 32
THRONE_TOP = 64
RUNE_Z = (192, 208)
STATUE_SCALE = 3.2
CONTENTS_LAVA = 8
CONTENTS_DETAIL = 0x8000000  # in a brush side's content flags


# ---------------------------------------------------------------- brushes

class Face:
    """A side's texture; scale and shift as in a .map line (texels per unit)."""

    def __init__(self, tex, scale=(0.5, 0.5), shift=(0.0, 0.0)):
        self.tex, self.scale, self.shift = tex, scale, shift


def as_face(f):
    return f if isinstance(f, Face) else Face(f)


class Brush:
    """A convex prism: a convex polygon in XY between z0 and z1.

    faces maps 'top', 'bottom' and 'side' (or '*') to a texture or Face.
    """

    def __init__(self, poly, z0, z1, faces, lava=False, detail=False):
        """`detail`: a decoration q3map2 leaves out of the BSP tree and visibility."""
        assert z0 < z1
        self.detail = detail
        poly = [(int(round(x)), int(round(y))) for x, y in poly]
        if signed_area(poly) < 0:  # keep counter-clockwise
            poly.reverse()
        self.poly, self.z0, self.z1, self.faces, self.lava = poly, z0, z1, faces, lava
        n = len(poly)
        for i in range(n):  # convex: every turn is to the left
            a, b, c = poly[i], poly[(i + 1) % n], poly[(i + 2) % n]
            assert (b[0] - a[0]) * (c[1] - b[1]) - (b[1] - a[1]) * (c[0] - b[0]) > 0, poly

    def face(self, key):
        return as_face(self.faces.get(key, self.faces.get("*")))

    def planes(self):
        """(three points with q3map2's outward winding, Face) per side."""
        out = []
        z0, z1 = self.z0, self.z1
        n = len(self.poly)
        for i in range(n):
            (ax, ay), (bx, by) = self.poly[i], self.poly[(i + 1) % n]
            # outward normal of a CCW edge is to its right
            out.append((((ax, ay, z0), (ax, ay, z1), (bx, by, z0)), self.face("side")))
        x0, y0 = self.poly[0]
        out.append((((x0, y0, z1), (x0, y0 + 64, z1), (x0 + 64, y0, z1)), self.face("top")))
        out.append((((x0, y0, z0), (x0 + 64, y0, z0), (x0, y0 + 64, z0)), self.face("bottom")))
        return out

    def text(self):
        lines = ["{"]
        for (a, b, c), f in self.planes():
            n = cross(sub(c, a), sub(b, a))  # q3map2's PlaneFromPoints
            assert n != (0, 0, 0)
            # a bare name is one of this map's textures; one with a folder is as given
            tex = f.tex if "/" in f.tex else "%s/%s" % (PREFIX, f.tex)
            lines.append("( %d %d %d ) ( %d %d %d ) ( %d %d %d ) %s %.3f %.3f 0 %.6f %.6f %d 0 0"
                         % (*a, *b, *c, tex, f.shift[0], f.shift[1], f.scale[0], f.scale[1],
                            CONTENTS_DETAIL if self.detail else 0))
        lines.append("}")
        return "\n".join(lines)

    def half_spaces(self):
        """Outward unit normals and distances."""
        out = []
        for (a, b, c), _ in self.planes():
            n = cross(sub(c, a), sub(b, a))
            length = math.sqrt(sum(v * v for v in n))
            n = tuple(v / length for v in n)
            out.append((n, sum(n[k] * a[k] for k in range(3))))
        return out

    def contains(self, p):
        return all(sum(n[k] * p[k] for k in range(3)) <= d + 0.5 for n, d in self.half_spaces())

    def touches_box(self, lo, hi):
        """Whether the box overlaps the brush (separating planes: the brush's and the box's)."""
        bx0 = min(x for x, _ in self.poly), max(x for x, _ in self.poly)
        by0 = min(y for _, y in self.poly), max(y for _, y in self.poly)
        if not (bx0[0] < hi[0] and lo[0] < bx0[1] and by0[0] < hi[1] and lo[1] < by0[1]
                and self.z0 < hi[2] and lo[2] < self.z1):
            return False
        for n, d in self.half_spaces():
            nearest = sum(n[k] * (lo[k] if n[k] > 0 else hi[k]) for k in range(3))
            if nearest >= d:
                return False
        return True


def signed_area(poly):
    return sum(poly[i][0] * poly[(i + 1) % len(poly)][1] - poly[(i + 1) % len(poly)][0] * poly[i][1]
               for i in range(len(poly))) / 2


def cross(a, b):
    return (a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0])


def sub(a, b):
    return tuple(x - y for x, y in zip(a, b))


def box(mins, maxs, faces, **kw):
    (x0, y0, z0), (x1, y1, z1) = mins, maxs
    return Brush([(x0, y0), (x1, y0), (x1, y1), (x0, y1)], z0, z1, faces, **kw)


def octagon(cx, cy, apothem):
    """Edges facing the axes and the diagonals."""
    t = round(apothem * math.tan(math.pi / 8))
    a = apothem
    return [(cx + x, cy + y) for x, y in
            ((a, -t), (a, t), (t, a), (-t, a), (-a, t), (-a, -t), (-t, -a), (t, -a))]


def mirrored(polys):
    """Each polygon in all four quadrants."""
    out = []
    for sx in (1, -1):
        for sy in (1, -1):
            out += [[(sx * x, sy * y) for x, y in p] for p in polys]
    return out


def entity(fields, brushes=()):
    out = ["{"] + ['"%s" "%s"' % kv for kv in fields.items()]
    out += [b.text() for b in brushes]
    out.append("}")
    return "\n".join(out)


# ---------------------------------------------------------------- layout

FLOOR = Face("floor")
TRIM = Face("trim", (0.25, 0.25))
WALLF = Face("wall", (1.0, 1.0))
ROCK = Face("lavarock", (0.5, 0.5))
STONE = Face("stone", (0.5, 0.5))


def shell():
    o = H + WALL
    return [
        box((-o, -o, PIT - 32), (o, o, PIT), {"*": TRIM, "top": ROCK}),
        box((-o, -o, HEIGHT), (o, o, HEIGHT + WALL), {"*": TRIM, "bottom": Face("ceiling")}),
        box((-o, -o, PIT), (-H, o, HEIGHT), {"*": WALLF}),
        box((H, -o, PIT), (o, o, HEIGHT), {"*": WALLF}),
        box((-H, -o, PIT), (H, -H, HEIGHT), {"*": WALLF}),
        box((-H, H, PIT), (H, o, HEIGHT), {"*": WALLF}),
    ]


def floors():
    """The hall's floor around the moat, the platform, the dais and the bridges."""
    ot = round(MOAT_OUT * math.tan(math.pi / 8))
    quadrant = [
        [(0, MOAT_OUT), (ot, MOAT_OUT), (H, H), (0, H)],
        [(ot, MOAT_OUT), (MOAT_OUT, ot), (H, H)],
        [(MOAT_OUT, ot), (MOAT_OUT, 0), (H, 0), (H, H)],
    ]
    out = [Brush(p, PIT, 0, {"*": ROCK, "top": FLOOR}) for p in mirrored(quadrant)]
    out.append(Brush(octagon(0, 0, MOAT_IN), PIT, PLATFORM_TOP, {"*": ROCK, "top": FLOOR}))
    emblem = Face("emblem", (2 * DAIS / 1024,) * 2, (512, 512))
    out.append(Brush(octagon(0, 0, DAIS), PLATFORM_TOP, DAIS_TOP, {"*": TRIM, "top": emblem}))
    for k in range(4):  # bridges, flush with the floor, with low curbs
        bridge = [((-96, MOAT_IN), (96, MOAT_IN), (96, MOAT_OUT), (-96, MOAT_OUT)),
                  ((-112, MOAT_IN), (-96, MOAT_IN), (-96, MOAT_OUT), (-112, MOAT_OUT)),
                  ((96, MOAT_IN), (112, MOAT_IN), (112, MOAT_OUT), (96, MOAT_OUT))]
        for j, p in enumerate(bridge):
            p = [rotate(v, k) for v in p]
            top = 0 if j == 0 else 16
            out.append(Brush(p, PIT, top, {"*": TRIM, "top": FLOOR if j == 0 else TRIM}))
    return out


def rotate(v, quarter):
    x, y = v
    for _ in range(quarter):
        x, y = -y, x
    return (x, y)


def lava():
    """The moat: eight trapezoids, the four an axis faces split around their bridge."""
    inner, outer = octagon(0, 0, MOAT_IN), octagon(0, 0, MOAT_OUT)
    faces = {"*": Face("lava", (0.5, 0.5))}
    out = []
    for i in range(8):  # octagon() edges alternate: even ones face an axis
        p = [inner[i], inner[(i + 1) % 8], outer[(i + 1) % 8], outer[i]]
        pieces = split_around_bridge(p, inner[i][0] == inner[(i + 1) % 8][0]) if i % 2 == 0 else [p]
        out += [Brush(q, PIT, LAVA_TOP, faces, lava=True) for q in pieces]
    return out


def split_around_bridge(p, axis_x):
    """Cut an axis-facing moat trapezoid where its bridge (|across| < 112) crosses."""
    # work in coordinates where the edge faces +y: (u, w) with w the distance out
    def to_uw(v):
        return (v[1], v[0]) if axis_x else (v[0], v[1])

    def from_uw(u, w):
        return (w, u) if axis_x else (u, w)

    sign = 1 if to_uw(p[0])[1] > 0 else -1
    uw = [to_uw(v) for v in p]
    wi, wo = abs(uw[0][1]), abs(uw[2][1])
    ti = max(abs(u) for u, _ in uw[:2])
    to = max(abs(u) for u, _ in uw[2:])
    pieces = []
    for s in (-1, 1):
        piece = [(s * 112, wi), (s * ti, wi), (s * to, wo), (s * 112, wo)]
        pieces.append([from_uw(u, sign * w) for u, w in piece])
    return pieces


def chamfers():
    out = []
    for sx in (1, -1):
        for sy in (1, -1):
            tri = [(sx * (H - CHAMFER), sy * H), (sx * H, sy * H), (sx * H, sy * (H - CHAMFER))]
            out.append(Brush(tri, 0, HEIGHT, {"*": WALLF}))
    return out


def rune_bands():
    """Glowing rune strips on the walls and chamfers."""
    runes = Face("runes", (0.25, 0.25))
    z0, z1 = RUNE_Z
    out = []
    edge = H - CHAMFER
    for k in range(4):
        p = [(-edge, H - 6), (edge, H - 6), (edge, H), (-edge, H)]
        out.append(Brush([rotate(v, k) for v in p], z0, z1, {"*": TRIM, "side": runes}))
    for sx in (1, -1):
        for sy in (1, -1):
            p = [(sx * edge, sy * H), (sx * H, sy * edge),
                 (sx * (H - 4), sy * (edge - 4)), (sx * (edge - 4), sy * (H - 4))]
            out.append(Brush(p, z0, z1, {"*": TRIM, "side": runes}))
    return out


def banner(axis, at, along, width, z0, z1):
    """A banner hanging on the wall at x (axis 'x') or y (axis 'y') = at."""
    su, sv = width / 256, (z1 - z0) / 1024
    a0 = along - width / 2
    face = Face("banner", (su, sv), ((-a0 / su) % 256, (z1 / sv) % 1024))
    inward = -1 if at > 0 else 1
    t0, t1 = sorted((at, at + inward * 4))
    if axis == "x":
        return box((t0, a0, z0), (t1, a0 + width, z1), {"*": TRIM, "side": face}, detail=True)
    return box((a0, t0, z0), (a0 + width, t1, z1), {"*": TRIM, "side": face}, detail=True)


def banners():
    out = []
    for at in (H, -H):
        for y in (-560, 0, 560):
            out.append(banner("x", at, y, 96, 256, 640))
    for x in (-560, 0, 560):
        out.append(banner("y", -H, x, 96, 256, 640))
    out.append(banner("y", H, 0, 192, 224, 704))  # behind the throne
    return out


def throne():
    out = [box((-256, 760, 0), (256, H, THRONE_TOP), {"*": TRIM, "top": FLOOR})]
    for i in (1, 2, 3):  # steps of 16 up to the dais
        y1 = 760 - 24 * (i - 1)
        out.append(box((-256, y1 - 24, 0), (256, y1, THRONE_TOP - 16 * i), {"*": TRIM, "top": FLOOR}))
    z = THRONE_TOP
    out += [
        box((-48, 880, z), (48, 944, z + 32), {"*": STONE}),
        box((-64, 944, z), (64, 972, z + 192), {"*": STONE}),
        box((-64, 880, z), (-48, 944, z + 56), {"*": TRIM}),
        box((48, 880, z), (64, 944, z + 56), {"*": TRIM}),
        box((-64, 944, z + 192), (-52, 972, z + 248), {"*": TRIM}),
        box((52, 944, z + 192), (64, 972, z + 248), {"*": TRIM}),
    ]
    return out


PILLARS = [(880, 400), (880, -400), (-880, 400), (-880, -400), (400, -880), (-400, -880)]
BRAZIERS = [(560, 560), (-560, 560), (560, -560), (-560, -560), (300, 700), (-300, 700)]
STATUES = [((720, 720), 225), ((-720, 720), 315), ((720, -720), 135), ((-720, -720), 45),
           ((420, 900), 270), ((-420, 900), 270)]


def pillars():
    out = []
    for x, y in PILLARS:
        out.append(Brush(octagon(x, y, 48), 0, HEIGHT, {"*": STONE}))
        out.append(Brush(octagon(x, y, 64), 0, 32, {"*": TRIM}))
        out.append(Brush(octagon(x, y, 64), 672, 704, {"*": TRIM}))
    return out


def beams():
    out = []
    z0, z1 = 720, HEIGHT
    for y in (-512, 0, 512):
        out.append(box((-H, y - 16, z0), (H, y + 16, z1), {"*": TRIM}))
    edges = [-H, -528, -496, -16, 16, 496, 528, H]
    for x in (-512, 0, 512):
        for a, b in zip(edges[::2], edges[1::2]):
            out.append(box((x - 16, a, z0), (x + 16, b, z1), {"*": TRIM}))
    return out


def lamps():
    """Warm panels under the beam crossings, so a duel can be seen."""
    lit = {"*": TRIM, "bottom": Face("lamp", (1.0, 1.0))}
    return [box((x - 32, y - 32, 704), (x + 32, y + 32, 720), lit)
            for x in (-512, 0, 512) for y in (-512, 0, 512)]


def braziers():
    out = []
    for x, y in BRAZIERS:
        out.append(Brush(octagon(x, y, 16), 0, 48, {"*": TRIM}))
        out.append(Brush(octagon(x, y, 32), 48, 64, {"*": TRIM}))
        out.append(Brush(octagon(x, y, 28), 64, 68, {"*": TRIM, "top": Face("embers")}))
    return out


def pedestals():
    out = []
    for (x, y), _ in STATUES:
        out.append(Brush(octagon(x, y, 96), 0, 48, {"*": STONE, "side": TRIM}))
        out.append(Brush(octagon(x, y, 64), 48, 48 + 104 * STATUE_SCALE, {"*": "clip"}))
    return out


def spawns():
    """(classname, origin, angle); origins 24 above their floor."""
    return [
        ("info_player_duel", (0, -300, PLATFORM_TOP + 24), 90),
        ("info_player_duel", (0, 300, PLATFORM_TOP + 24), 270),
        ("info_player_deathmatch", (880, 0, 24), 180),
        ("info_player_deathmatch", (-880, 0, 24), 0),
        ("info_player_deathmatch", (0, -880, 24), 90),
        ("info_player_deathmatch", (160, 820, THRONE_TOP + 24), 270),
        ("info_player_deathmatch", (-160, 820, THRONE_TOP + 24), 270),
        ("info_player_deathmatch", (760, -560, 24), 144),
        ("info_player_deathmatch", (-760, -560, 24), 36),
        ("info_player_deathmatch", (0, -660, 24), 90),
    ]


def build():
    brushes = (shell() + floors() + lava() + chamfers() + rune_bands() + banners()
               + throne() + pillars() + beams() + lamps() + braziers() + pedestals())
    ents = [entity({"classname": "worldspawn", "message": "SJK Sith Temple",
                    "_ambient": "10", "gridsize": "64 64 64"}, brushes)]
    figures = ["acolyte", "lord", "lord", "acolyte", "caster", "caster"]
    for ((x, y), angle), figure in zip(STATUES, figures):
        ents.append(entity({"classname": "misc_model", "model": "%s_%s.md3" % (STATUE, figure),
                            "origin": "%d %d 48" % (x, y), "angle": str(angle),
                            "modelscale": str(STATUE_SCALE), "spawnflags": "4"}))
    for cls, origin, angle in spawns():
        ents.append(entity({"classname": cls, "origin": "%d %d %d" % origin,
                            "angle": str(angle)}))
    return brushes, "\n".join(ents) + "\n"


def check(brushes):
    """Each spawn point has room for a player and solid, lava-free ground under it."""
    solid = [b for b in brushes if not b.lava]
    for cls, (x, y, z), _ in spawns():
        lo, hi = (x - 16, y - 16, z - 23), (x + 16, y + 16, z + 40)
        for b in brushes:
            if b.touches_box(lo, hi):
                sys.exit("%s at %s touches the brush %s" % (cls, (x, y, z), b.poly))
        if not any(b.contains((x, y, z - 26)) for b in solid):
            sys.exit("%s at %s has no floor under it" % (cls, (x, y, z)))
    for (x, y), _ in STATUES:
        if not any(b.contains((x, y, 40)) for b in solid):
            sys.exit("statue at %s has no pedestal" % ((x, y),))


# ---------------------------------------------------------------- textures

def fbm(size, beta, seed):
    """Tileable fractal noise in 0..1 (random phases under a 1/f^beta spectrum)."""
    rng = np.random.default_rng(seed)
    f = np.fft.fftfreq(size)
    fx, fy = np.meshgrid(f, f)
    amp = np.hypot(fx, fy)
    amp[0, 0] = 1
    amp = amp ** -beta
    amp[0, 0] = 0
    n = np.real(np.fft.ifft2(amp * np.exp(2j * np.pi * rng.random((size, size)))))
    return (n - n.min()) / (n.max() - n.min())


def ridges(size, seed, width=0.03, beta=1.6):
    """Thin tileable crack lines, 1 on the line."""
    n = fbm(size, beta, seed)
    return np.clip(1 - np.abs(n - 0.5) / width, 0, 1)


def rgb(arr):
    return Image.fromarray(np.clip(arr, 0, 255).astype(np.uint8), "RGB")


def tint(gray, low, high):
    g = gray[..., None]
    return np.array(low) * (1 - g) + np.array(high) * g


def ramp(x, stops):
    """Map 0..1 through colour stops [(t, (r, g, b)), ...]."""
    out = np.zeros(x.shape + (3,))
    for (t0, c0), (t1, c1) in zip(stops, stops[1:]):
        m = (x >= t0) & (x <= t1)
        k = ((x - t0) / (t1 - t0))[..., None]
        out[m] = (np.array(c0) * (1 - k) + np.array(c1) * k)[m]
    return out


def lava_crust(size, seed, cells=14):
    """Dark plates of cooled crust with glowing cracks between them (a multiplier:
    white where the lava shows): tileable Voronoi edges."""
    rng = np.random.default_rng(seed)
    points = rng.random((cells, 2)) * size
    yy, xx = np.mgrid[0:size, 0:size].astype(float)
    nearest = np.full((size, size), 1e9)
    second = np.full((size, size), 1e9)
    for px, py in points:
        for ox in (-size, 0, size):
            for oy in (-size, 0, size):
                d = np.hypot(xx - px - ox, yy - py - oy)
                second = np.minimum(second, np.maximum(nearest, d))
                nearest = np.minimum(nearest, d)
    edge = np.clip(1 - (second - nearest) / 7, 0, 1) ** 1.5  # 1 on a crack
    plate = 0.22 + 0.12 * fbm(size, 1.4, seed + 1)
    v = plate * (1 - edge) + edge
    return np.stack([v * 255, v * 235, v * 215], axis=-1)


def draw_textures(folder):
    folder.mkdir(parents=True, exist_ok=True)
    save = lambda img, n: img.save(folder / (n + ".tga"))  # noqa: E731
    rnd = random.Random(7)

    # walls: big obsidian blocks with faint red veins
    s = 512
    n = fbm(s, 1.4, 1)
    wall = tint(n, (44, 40, 50), (88, 80, 96))
    yy, xx = np.mgrid[0:s, 0:s]
    for by in range(0, s, 64):
        row = by // 64
        off = 64 * (row % 2)
        for bx in range(-128, s, 128):
            x0 = bx + off
            k = rnd.uniform(0.8, 1.15)
            m = (yy >= by) & (yy < by + 64) & (((xx - x0) % s) < 128)
            wall[m] *= k
    mortar = ((yy % 64) < 3) | ((((xx - 64 * ((yy // 64) % 2)) % 128) < 3))
    wall[mortar] = (18, 16, 20)
    veins = ridges(s, 2, 0.018)
    wall = wall * (1 - veins[..., None] * 0.7) + np.array((120, 12, 10)) * veins[..., None] * 0.7
    save(rgb(wall), "wall")

    # floor: polished dark stone, marble veins, a red inlay on the tile joints
    n = fbm(s, 1.8, 3)
    floor = tint(n, (52, 48, 58), (96, 90, 104))
    floor += ridges(s, 4, 0.02)[..., None] * np.array((40, 38, 44))
    joint = ((yy % 256) < 4) | ((xx % 256) < 4)
    inlay = ((yy % 256 >= 1) & (yy % 256 < 3)) | ((xx % 256 >= 1) & (xx % 256 < 3))
    floor[joint] = (12, 10, 12)
    floor[inlay] = (110, 14, 12)
    save(rgb(floor), "floor")

    # trim: black metal, a lighter bevel and rivets
    t = 256
    n = fbm(t, 1.2, 5)
    trim = tint(n, (38, 38, 44), (64, 64, 72))
    img = rgb(trim)
    d = ImageDraw.Draw(img)
    d.rectangle([0, 0, t - 1, t - 1], outline=(70, 66, 72), width=4)
    d.rectangle([6, 6, t - 7, t - 7], outline=(8, 8, 10), width=2)
    for x in (24, t - 24):
        for y in (24, t - 24):
            d.ellipse([x - 6, y - 6, x + 6, y + 6], fill=(60, 58, 64), outline=(10, 10, 12))
    save(img, "trim")

    # stone for pillars, statues and the throne
    n = fbm(s, 1.5, 6)
    stone = tint(n, (78, 74, 82), (140, 134, 144)) * (1 - ridges(s, 8, 0.015)[..., None] * 0.5)
    save(rgb(stone), "stone")
    save(rgb(np.full((32, 32, 3), 4)), "face")
    hilt = rgb(tint(fbm(64, 1.0, 9), (40, 40, 44), (120, 118, 124)))
    save(hilt, "hilt")
    dark = tint(fbm(s, 1.5, 19), (30, 28, 34), (70, 66, 76)) * (1 - ridges(s, 20, 0.015)[..., None] * 0.4)
    dark += ridges(s, 21, 0.008)[..., None] * np.array((90, 20, 16))  # faint red veins
    save(rgb(dark), "statue_dark")
    patina = fbm(s, 1.3, 22)
    bronze = tint(fbm(s, 1.6, 23), (90, 60, 30), (170, 120, 64))
    bronze = bronze * (1 - patina[..., None] * 0.35) + np.array((40, 90, 70)) * patina[..., None] * 0.35
    save(rgb(bronze), "statue_bronze")
    save(rgb(np.full((16, 16, 3), (255, 50, 30))), "eyes")
    save(rgb(np.full((16, 16, 3), (255, 40, 24))), "blade")
    save(rgb(np.full((16, 16, 3), (255, 230, 220))), "blade_core")

    # ceiling: dark rough rock
    save(rgb(tint(fbm(s, 1.3, 10), (24, 22, 28), (56, 50, 60))), "ceiling")

    # rock around the lava, with glowing cracks (glow mask in lavarock_glow)
    n = fbm(s, 1.6, 11)
    rock = tint(n, (34, 28, 28), (80, 64, 58))
    cracks = ridges(s, 12, 0.025) * np.clip(1.4 - yy / s, 0, 1)
    rock = rock * (1 - cracks[..., None]) + np.array((150, 40, 8)) * cracks[..., None]
    save(rgb(rock), "lavarock")
    save(rgb(np.array((255, 90, 20)) * (cracks ** 1.5)[..., None]), "lavarock_glow")

    # lava: two layers of molten noise
    stops = [(0, (40, 2, 0)), (0.35, (150, 20, 0)), (0.6, (240, 90, 10)),
             (0.85, (255, 190, 60)), (1, (255, 240, 170))]
    n = fbm(256, 1.6, 13)
    save(rgb(ramp(np.clip(n * 1.25 - 0.1, 0, 1), stops)), "lava")
    n = fbm(256, 1.2, 14)
    save(rgb(ramp(np.clip(n * 1.4 - 0.55, 0, 1), stops) * 0.6), "lava2")
    save(rgb(lava_crust(256, 24)), "lava_crust")
    n = fbm(128, 1.1, 15)
    save(rgb(ramp(np.clip(n * 1.3 - 0.05, 0, 1), stops)), "embers")

    # lamp: a warm, dim panel in a black iron frame
    lamp = Image.new("RGB", (64, 64), (255, 196, 150))
    ImageDraw.Draw(lamp).rectangle([0, 0, 63, 63], outline=(20, 18, 20), width=6)
    save(lamp, "lamp")

    # runes: angular glyphs, dull red on black; runes_glow is their light
    w, hgt = 512, 64
    mask = Image.new("L", (w, hgt), 0)
    d = ImageDraw.Draw(mask)
    for cx in range(16, w, 32):
        pts = [(cx + rnd.choice((-8, 0, 8)), 12 + rnd.choice((0, 20, 40))) for _ in range(4)]
        d.line(pts, fill=255, width=3)
        if rnd.random() < 0.5:
            d.line([(cx - 8, 32), (cx + 8, 32)], fill=255, width=3)
    d.line([(0, 3), (w, 3)], fill=160, width=2)
    d.line([(0, hgt - 4), (w, hgt - 4)], fill=160, width=2)
    m = np.asarray(mask.filter(ImageFilter.GaussianBlur(0.8)), dtype=float) / 255
    base = tint(fbm(512, 1.2, 16)[:hgt], (10, 9, 11), (26, 24, 28))
    save(rgb(base * (1 - m[..., None]) + np.array((140, 16, 10)) * m[..., None]), "runes")
    glow = np.asarray(mask.filter(ImageFilter.GaussianBlur(2.0)), dtype=float) / 255
    save(rgb(np.array((255, 40, 16)) * np.clip(m + glow * 0.6, 0, 1)[..., None]), "runes_glow")

    # banner: red cloth, black borders, an angular emblem
    bw, bh = 256, 1024
    by_, bx_ = np.mgrid[0:bh, 0:bw]
    folds = 0.75 + 0.25 * np.sin(bx_ / bw * 2 * np.pi * 5) ** 2
    cloth = tint(fbm(1024, 1.2, 17)[:, :bw], (70, 4, 6), (130, 10, 12)) * folds[..., None]
    img = rgb(cloth)
    d = ImageDraw.Draw(img)
    d.rectangle([0, 0, bw - 1, 40], fill=(12, 10, 12))
    d.rectangle([0, 0, 14, bh - 1], fill=(12, 10, 12))
    d.rectangle([bw - 15, 0, bw - 1, bh - 1], fill=(12, 10, 12))
    d.rectangle([0, bh - 60, bw - 1, bh - 1], fill=(12, 10, 12))
    cx, cy = bw // 2, 300
    star = []
    for i in range(16):
        a = math.pi * i / 8 - math.pi / 2
        r = 100 if i % 2 == 0 else 38
        star.append((cx + r * math.cos(a) * 0.9, cy + r * math.sin(a)))
    d.polygon(star, fill=(14, 10, 12), outline=(150, 110, 50))
    d.ellipse([cx - 26, cy - 26, cx + 26, cy + 26], fill=(130, 10, 12), outline=(150, 110, 50), width=3)
    d.polygon([(cx, cy - 18), (cx + 12, cy), (cx, cy + 18), (cx - 12, cy)], fill=(14, 10, 12))
    for y in (500, 520):
        d.line([(30, y), (bw - 30, y)], fill=(150, 110, 50), width=3)
    save(img, "banner")

    # emblem on the duelling dais: rings and an eight-pointed star in red inlay
    e = 1024
    n = fbm(e, 1.8, 18)
    img = rgb(tint(n, (16, 14, 18), (40, 36, 44)))
    lines = Image.new("L", (e, e), 0)
    d2 = ImageDraw.Draw(lines)
    c = e // 2
    for r, wdt in ((470, 10), (430, 4), (180, 8), (60, 6)):
        d2.ellipse([c - r, c - r, c + r, c + r], outline=255, width=wdt)
    pts = []
    for i in range(16):
        a = math.pi * i / 8
        r = 420 if i % 2 == 0 else 190
        pts.append((c + r * math.cos(a), c + r * math.sin(a)))
    d2.polygon(pts, outline=255, width=6)
    for i in range(8):
        a = math.pi * i / 4 + math.pi / 8
        d2.line([(c + 190 * math.cos(a), c + 190 * math.sin(a)),
                 (c + 430 * math.cos(a), c + 430 * math.sin(a))], fill=180, width=3)
    m = np.asarray(lines, dtype=float)[..., None] / 255
    arr = np.asarray(img, dtype=float) * (1 - m) + np.array((150, 18, 12)) * m
    save(rgb(arr), "emblem")
    halo = np.asarray(lines.filter(ImageFilter.GaussianBlur(3)), dtype=float)[..., None] / 255
    save(rgb(np.array((255, 40, 16)) * np.clip(m + halo, 0, 1)), "emblem_glow")


# ---------------------------------------------------------------- shaders

def lit(name, glow=None, power=0, extra=""):
    full = "textures/%s/%s" % (PREFIX, name)
    out = "%s\n{\n\tqer_editorimage %s.tga\n" % (full, full)
    if power:
        out += "\tq3map_surfacelight %d\n\tq3map_lightimage textures/%s/%s.tga\n" % (
            power, PREFIX, glow or name)
    out += extra
    out += ("\t{\n\t\tmap $lightmap\n\t\trgbGen identity\n\t}\n"
            "\t{\n\t\tmap %s.tga\n\t\tblendFunc GL_DST_COLOR GL_ZERO\n\t\trgbGen identity\n\t}\n"
            % full)
    if glow:
        out += ("\t{\n\t\tmap textures/%s/%s.tga\n\t\tblendFunc GL_ONE GL_ONE\n\t\tglow\n\t}\n"
                % (PREFIX, glow))
    return out + "}\n"


def shader_text():
    t = "textures/" + PREFIX
    parts = [lit(n) for n in ("wall", "floor", "trim", "stone", "ceiling", "banner")]
    parts.append(lit("lavarock", "lavarock_glow", 200))
    parts.append("%s/clip\n{\n\tqer_trans 0.4\n\tsurfaceparm nodraw\n\tsurfaceparm nolightmap\n"
                 "\tsurfaceparm nonsolid\n\tsurfaceparm trans\n\tsurfaceparm nomarks\n"
                 "\tsurfaceparm playerclip\n}\n" % t)
    parts.append(lit("runes", "runes_glow", 900, "\tsurfaceparm nomarks\n"))
    parts.append(lit("emblem", "emblem_glow", 400))
    parts.append(lit("lamp", "lamp", 4000, "\tsurfaceparm nomarks\n"))
    parts.append("""%(t)s/embers
{
\tqer_editorimage %(t)s/embers.tga
\tq3map_surfacelight 6000
\tq3map_lightimage %(t)s/embers.tga
\tsurfaceparm nomarks
\t{
\t\tmap %(t)s/embers.tga
\t\ttcMod turb 0 0.03 0 0.2
\t\tglow
\t}
}
""" % {"t": t})
    parts.append("""%(t)s/lava
{
\tqer_editorimage %(t)s/lava.tga
\tq3map_surfacelight 700
\tq3map_lightimage %(t)s/lava.tga
\tq3map_lightsubdivide 128
\tq3map_globaltexture
\ttessSize 128
\tdeformVertexes wave 160 sin 0 2.5 0 0.12
\tsurfaceparm nonsolid
\tsurfaceparm lava
\tsurfaceparm trans
\tsurfaceparm noimpact
\tsurfaceparm nomarks
\t{
\t\tmap %(t)s/lava.tga
\t\ttcMod turb 0 0.06 0 0.05
\t\ttcMod scroll 0.015 0.008
\t\tglow
\t}
\t{
\t\tmap %(t)s/lava_crust.tga
\t\tblendFunc GL_DST_COLOR GL_ZERO
\t\ttcMod scale 0.5 0.5
\t\ttcMod scroll 0.006 0.003
\t}
\t{
\t\tmap %(t)s/lava2.tga
\t\tblendFunc GL_ONE GL_ONE
\t\trgbGen wave sin 0.55 0.35 0 0.25
\t\ttcMod scroll -0.01 0.015
\t\tglow
\t}
}
""" % {"t": t})
    models = STATUE.rsplit("/", 1)[0]
    for n in ("statue", "statue_dark", "statue_bronze", "face", "hilt"):
        tex = {"statue": "stone"}.get(n, n)
        parts.append("%s/%s\n{\n\tqer_editorimage %s/%s.tga\n"
                     "\t{\n\t\tmap $lightmap\n\t\trgbGen identity\n\t}\n"
                     "\t{\n\t\tmap %s/%s.tga\n\t\tblendFunc GL_DST_COLOR GL_ZERO\n"
                     "\t\trgbGen identity\n\t}\n}\n" % (models, n, t, tex, t, tex))
    # glowing parts: helm eyes and lit blades, which light what is near them
    for n, power in (("eyes", 200), ("blade", 2500), ("blade_core", 1500)):
        parts.append("%(m)s/%(n)s\n{\n\tqer_editorimage %(t)s/%(n)s.tga\n"
                     "\tq3map_surfacelight %(p)d\n\tq3map_lightimage %(t)s/%(n)s.tga\n"
                     "\tsurfaceparm nomarks\n\tsurfaceparm nolightmap\n"
                     "\t{\n\t\tmap %(t)s/%(n)s.tga\n\t\tglow\n\t}\n}\n"
                     % {"m": models, "n": n, "t": t, "p": power})
    return "\n".join(parts)


def write_statues(stage):
    """statue.md3 (the acolyte this map places) and statue_<name>.md3 for each figure."""
    model_dir = STATUE.rsplit("/", 1)[0]
    for name in FIGURES:
        surfaces = statue_mesh(name)
        shaders = {k: "%s/%s" % (model_dir, k) for k in surfaces}
        write_md3(stage / ("%s_%s.md3" % (STATUE, name)), surfaces, shaders)
        if name == "acolyte":
            write_md3(stage / (STATUE + ".md3"), surfaces, shaders)


def write_assets(stage):
    draw_textures(stage / "textures" / PREFIX)
    write_statues(stage)
    shaders = stage / "shaders"
    shaders.mkdir(exist_ok=True)
    (shaders / (PREFIX + ".shader")).write_text(shader_text(), newline="\n")
    # q3map2 reads only the shader files a shaderlist.txt names (compile only)
    (shaders / "shaderlist.txt").write_text(PREFIX + "\n", newline="\n")


# ---------------------------------------------------------------- preview

def preview(brushes, path):
    o = H + WALL
    scale = 0.4
    size = int(2 * o * scale) + 1

    def px(x, y):  # north up
        return ((x + o) * scale, (o - y) * scale)

    img = Image.new("RGB", (size, size), (16, 14, 18))
    d = ImageDraw.Draw(img)
    for b in sorted(brushes, key=lambda b: b.z1):
        if b.z1 > HEIGHT or b.z0 >= 640:  # ceiling, beams, capitals
            continue
        if b.lava:
            color = (230, 90, 20)
        elif b.z1 >= HEIGHT:
            color = (50, 46, 56)
        else:
            t = max(0.0, min(1.0, (b.z1 - PIT) / (THRONE_TOP - PIT)))
            color = (int(40 + 90 * t), int(40 + 70 * t), int(60 + 60 * t))
        d.polygon([px(*v) for v in b.poly], fill=color, outline=(10, 10, 12))
    for (x, y), angle in STATUES:
        cx, cy = px(x, y)
        d.ellipse([cx - 14, cy - 14, cx + 14, cy + 14], fill=(170, 165, 175))
        a = math.radians(angle)
        d.line([cx, cy, cx + 24 * math.cos(a), cy - 24 * math.sin(a)], fill=(20, 20, 20), width=3)
    for cls, (x, y, _), angle in spawns():
        cx, cy = px(x, y)
        color = (230, 60, 60) if cls == "info_player_duel" else (90, 200, 110)
        d.ellipse([cx - 7, cy - 7, cx + 7, cy + 7], fill=color)
        a = math.radians(angle)
        d.line([cx, cy, cx + 16 * math.cos(a), cy - 16 * math.sin(a)], fill=(255, 255, 255), width=2)
    img.save(path)


# ---------------------------------------------------------------- main

def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--gamedata", default=DEFAULT_GAMEDATA)
    ap.add_argument("--q3map2", default=DEFAULT_Q3MAP2)
    ap.add_argument("--out", default=DEFAULT_OUT)
    ap.add_argument("--install", action="store_true")
    ap.add_argument("--no-compile", action="store_true")
    ap.add_argument("--preview", metavar="PNG")
    args = ap.parse_args()
    gamedata, out = Path(args.gamedata), Path(args.out)
    home = out / "home"
    stage = home / "base"
    if out.exists():
        shutil.rmtree(out)
    (stage / "maps").mkdir(parents=True)

    brushes, text = build()
    check(brushes)
    mapfile = stage / "maps" / (MAP + ".map")
    mapfile.write_text(text, newline="\n")
    write_assets(stage)
    print("wrote %s: %d brushes, %d statues" % (mapfile, len(brushes), len(STATUES)))
    if args.preview:
        preview(brushes, args.preview)
        print("wrote", args.preview)
    if args.no_compile:
        return
    print("compiling", MAP, flush=True)
    compile_map(args.q3map2, gamedata, home, mapfile)
    pk3 = out / (MAP + ".pk3")
    with zipfile.ZipFile(pk3, "w", zipfile.ZIP_DEFLATED) as z:
        z.write(mapfile.with_suffix(".bsp"), "maps/%s.bsp" % MAP)
        z.write(stage / "shaders" / (PREFIX + ".shader"), "shaders/%s.shader" % PREFIX)
        for tga in sorted((stage / "textures" / PREFIX).glob("*.tga")):
            z.write(tga, "textures/%s/%s" % (PREFIX, tga.name))
        for md3 in sorted((stage / STATUE).parent.glob("*.md3")):
            z.write(md3, md3.relative_to(stage).as_posix())
    print("wrote", pk3)
    if args.install:
        shutil.copy2(pk3, gamedata / "base" / pk3.name)
        print("installed", gamedata / "base" / pk3.name)


if __name__ == "__main__":
    main()
