#!/usr/bin/env python3
"""Build sjk_arena1, a small duel arena made only of SJK's own textures.

Usage:
    python scripts/arena_map.py [--gamedata DIR] [--q3map2 EXE]
        [--out DIR] [--install] [--no-compile] [--preview PNG]

Writes, in --out (default S:/Tools/arena):

- home/base/maps/sjk_arena1.map: the source map, a closed hall with a two-step
  dais in the middle, four pillars, a balcony on the north and south walls
  with stairs at their west and east ends, light entities, two
  `info_player_duel` and six `info_player_deathmatch` spawn points.
- sjk_arena1.pk3: the compiled map, its textures (drawn here, not taken from
  the game) and shaders/sjk_arena.shader. --install copies it to GameData/base,
  where both SJK and EternalJK load it (`devmap sjk_arena1`).
- --preview: a plan of the brushes (shaded by height), lights and spawn
  points, for checking a layout without compiling it.

Every brush is an axis-aligned box. Interior units: the hall is 1280 square
and 512 high; steps are 16 high (pmove climbs 18) and 24 deep.

Needs Python 3 with Pillow, and q3map2 from NetRadiant-custom (`-game ja`)
unless --no-compile.
"""
import argparse
import random
import shutil
import subprocess
import sys
import zipfile
from pathlib import Path

from PIL import Image, ImageDraw

DEFAULT_GAMEDATA = r"C:\Program Files (x86)\Steam\steamapps\common\Jedi Academy\GameData"
DEFAULT_Q3MAP2 = r"S:\Tools\netradiant-custom\q3map2.exe"
DEFAULT_OUT = r"S:\Tools\arena"

MAP = "sjk_arena1"
PREFIX = "sjk_arena"  # textures/<PREFIX>/ and shaders/<PREFIX>.shader

HALF = 640  # the hall's interior spans -HALF..HALF on x and y
HEIGHT = 512
WALL = 16
STEP_H, STEP_D = 16, 24
BALCONY_D, BALCONY_Z = 128, 160
STAIR_W = 128

TEXTURES = ("floor", "wall", "trim", "pillar", "ceiling")


# ---------------------------------------------------------------- brushes

def cross(a, b):
    return (a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0])


def sub(a, b):
    return tuple(x - y for x, y in zip(a, b))


class Brush:
    """An axis-aligned box with one texture per side ('-x' .. '+z', or '*')."""

    def __init__(self, mins, maxs, faces):
        assert all(lo < hi for lo, hi in zip(mins, maxs)), (mins, maxs)
        self.mins, self.maxs, self.faces = tuple(mins), tuple(maxs), faces

    def text(self):
        lines = ["{"]
        for axis in range(3):
            for sign in (-1, 1):
                key = ("-" if sign < 0 else "+") + "xyz"[axis]
                tex = self.faces.get(key, self.faces.get("*"))
                coord = self.maxs[axis] if sign > 0 else self.mins[axis]
                u, v = [k for k in range(3) if k != axis]
                pts = []
                for du, dv in ((0, 0), (1, 0), (0, 1)):
                    p = [0, 0, 0]
                    p[axis] = coord
                    p[u] = self.mins[u] + du * 64
                    p[v] = self.mins[v] + dv * 64
                    pts.append(tuple(p))
                a, b, c = pts
                n = cross(sub(c, a), sub(b, a))  # q3map2's PlaneFromPoints
                if n[axis] * sign < 0:
                    b, c = c, b
                lines.append("( %d %d %d ) ( %d %d %d ) ( %d %d %d ) %s/%s 0 0 0 0.5 0.5 0 0 0"
                             % (*a, *b, *c, PREFIX, tex))
        lines.append("}")
        return "\n".join(lines)


def entity(fields, brushes=()):
    out = ["{"] + ['"%s" "%s"' % kv for kv in fields.items()]
    out += [b.text() for b in brushes]
    out.append("}")
    return "\n".join(out)


def mirror_x(b):
    return Brush((-b.maxs[0], b.mins[1], b.mins[2]), (-b.mins[0], b.maxs[1], b.maxs[2]), b.faces)


def mirror_y(b):
    return Brush((b.mins[0], -b.maxs[1], b.mins[2]), (b.maxs[0], -b.mins[1], b.maxs[2]), b.faces)


# ---------------------------------------------------------------- layout

def shell():
    """Floor, ceiling and four walls, overlapping at the corners so the hall seals."""
    o = HALF + WALL
    wall = {"*": "wall"}
    return [
        Brush((-o, -o, -WALL), (o, o, 0), {"*": "trim", "+z": "floor"}),
        Brush((-o, -o, HEIGHT), (o, o, HEIGHT + WALL), {"*": "trim", "-z": "ceiling"}),
        Brush((-o, -o, 0), (-HALF, o, HEIGHT), wall),
        Brush((HALF, -o, 0), (o, o, HEIGHT), wall),
        Brush((-HALF, -o, 0), (HALF, -HALF, HEIGHT), wall),
        Brush((-HALF, HALF, 0), (HALF, o, HEIGHT), wall),
    ]


def dais():
    return [
        Brush((-256, -256, 0), (256, 256, STEP_H), {"*": "trim", "+z": "floor"}),
        Brush((-160, -160, STEP_H), (160, 160, 2 * STEP_H), {"*": "trim", "+z": "floor"}),
    ]


def pillars():
    out = []
    for sx in (-1, 1):
        for sy in (-1, 1):
            x, y = 384 * sx, 384 * sy
            out.append(Brush((x - 32, y - 32, 0), (x + 32, y + 32, HEIGHT), {"*": "pillar"}))
            out.append(Brush((x - 40, y - 40, 0), (x + 40, y + 40, 16), {"*": "trim"}))
    return out


def balconies():
    """North balcony with its rail and two stairs, mirrored to the south."""
    y0 = HALF - BALCONY_D
    north = [
        Brush((-HALF, y0, BALCONY_Z - 16), (HALF, HALF, BALCONY_Z),
              {"*": "trim", "+z": "floor"}),
        # rail, open over the stairs
        Brush((-HALF + STAIR_W, y0, BALCONY_Z), (HALF - STAIR_W, y0 + 8, BALCONY_Z + 40),
              {"*": "trim"}),
    ]
    steps = BALCONY_Z // STEP_H
    for i in range(steps - 1):  # the balcony itself is the last step
        # one box per step, side by side: overlapping boxes leave slivers in -meta
        top = (i + 1) * STEP_H
        y = y0 - (steps - 1 - i) * STEP_D
        stair = Brush((-HALF, y, 0), (-HALF + STAIR_W, y + STEP_D, top),
                      {"*": "trim", "+z": "floor"})
        north += [stair, mirror_x(stair)]
    return north + [mirror_y(b) for b in north]


def lights():
    out = []
    for x in (-480, 0, 480):  # off the pillars at +-384
        for y in (-480, 0, 480):
            out.append(((x, y, HEIGHT - 64), 450, "1 0.92 0.8"))
    for x in (-384, 0, 384):
        for y in (HALF - 64, -HALF + 64):
            out.append(((x, y, BALCONY_Z - 48), 200, "0.6 0.75 1"))
            out.append(((x, y, BALCONY_Z + 128), 250, "1 0.92 0.8"))
    return out


def spawns():
    """(classname, origin, angle); origins 24 above the floor they stand on."""
    z0 = 24
    return [
        ("info_player_duel", (0, -200, STEP_H + z0), 90),
        ("info_player_duel", (0, 200, STEP_H + z0), 270),
        ("info_player_deathmatch", (-480, 0, z0), 0),
        ("info_player_deathmatch", (480, 0, z0), 180),
        ("info_player_deathmatch", (0, -HALF + 64, BALCONY_Z + z0), 90),
        ("info_player_deathmatch", (0, HALF - 64, BALCONY_Z + z0), 270),
        ("info_player_deathmatch", (-480, -480, z0), 45),
        ("info_player_deathmatch", (480, 480, z0), 225),
    ]


def build():
    brushes = shell() + dais() + pillars() + balconies()
    ents = [entity({"classname": "worldspawn", "message": "SJK Arena",
                    "_ambient": "8", "gridsize": "64 64 64"}, brushes)]
    for origin, level, color in lights():
        ents.append(entity({"classname": "light", "origin": "%d %d %d" % origin,
                            "light": str(level), "_color": color}))
    for cls, origin, angle in spawns():
        ents.append(entity({"classname": cls, "origin": "%d %d %d" % origin,
                            "angle": str(angle)}))
    return brushes, "\n".join(ents) + "\n"


def check(brushes):
    """Spawn points need room for a player's box, and no light may be inside a brush."""
    boxes = [(cls, (x - 16, y - 16, z - 23), (x + 16, y + 16, z + 40))
             for cls, (x, y, z), _ in spawns()]
    boxes += [("light", o, o) for o, _, _ in lights()]
    for name, lo, hi in boxes:
        for b in brushes:
            if all(b.mins[k] <= hi[k] and lo[k] <= b.maxs[k] for k in range(3)):
                sys.exit("%s at %s touches the brush %s..%s" % (name, lo, b.mins, b.maxs))


# ---------------------------------------------------------------- assets

def noise_tile(size, base, spread, seed):
    rnd = random.Random(seed)
    img = Image.new("RGB", (size, size))
    px = img.load()
    for y in range(size):
        for x in range(size):
            d = rnd.randint(-spread, spread)
            px[x, y] = tuple(max(0, min(255, c + d)) for c in base)
    return img


def draw_textures(folder):
    folder.mkdir(parents=True, exist_ok=True)
    floor = noise_tile(256, (92, 88, 82), 6, 1)
    d = ImageDraw.Draw(floor)
    for i in range(0, 256, 64):  # 128-unit tiles at scale 0.5
        d.line([(i, 0), (i, 255)], fill=(60, 58, 55), width=2)
        d.line([(0, i), (255, i)], fill=(60, 58, 55), width=2)
    floor.save(folder / "floor.tga")
    wall = noise_tile(256, (120, 112, 100), 8, 2)
    d = ImageDraw.Draw(wall)
    for row, y in enumerate(range(0, 256, 32)):  # stone courses
        d.line([(0, y), (255, y)], fill=(80, 74, 66), width=2)
        for x in range(64 * (row % 2) // 2, 256, 64):
            d.line([(x, y), (x, y + 32)], fill=(80, 74, 66), width=2)
    wall.save(folder / "wall.tga")
    trim = noise_tile(64, (70, 74, 82), 4, 3)
    ImageDraw.Draw(trim).rectangle([0, 0, 63, 63], outline=(110, 116, 126), width=2)
    trim.save(folder / "trim.tga")
    pillar = noise_tile(128, (150, 140, 120), 6, 4)
    d = ImageDraw.Draw(pillar)
    for x in range(8, 128, 32):  # flutes
        d.rectangle([x, 0, x + 8, 127], fill=(120, 112, 96))
    pillar.save(folder / "pillar.tga")
    noise_tile(128, (55, 55, 62), 4, 5).save(folder / "ceiling.tga")


def shader_text():
    out = []
    for name in TEXTURES:
        full = "textures/%s/%s" % (PREFIX, name)
        out.append("%s\n{\n\tqer_editorimage %s.tga\n\t{\n\t\tmap $lightmap\n\t\trgbGen identity\n"
                   "\t}\n\t{\n\t\tmap %s\n\t\tblendFunc GL_DST_COLOR GL_ZERO\n\t\trgbGen identity\n"
                   "\t}\n}\n" % (full, full, full))
    return "\n".join(out)


def write_assets(stage):
    draw_textures(stage / "textures" / PREFIX)
    shaders = stage / "shaders"
    shaders.mkdir(exist_ok=True)
    (shaders / (PREFIX + ".shader")).write_text(shader_text(), newline="\n")
    # q3map2 reads only the shader files a shaderlist.txt names. For the
    # compile only; not in the pk3.
    (shaders / "shaderlist.txt").write_text(PREFIX + "\n", newline="\n")


# ---------------------------------------------------------------- preview

def preview(brushes, path):
    """Top-down plan: brushes shaded by their top height, lights and spawns."""
    o = HALF + WALL
    scale = 0.5
    size = int(2 * o * scale) + 1

    def px(x, y):  # +y (north) is up in the picture
        return ((x + o) * scale, (o - y) * scale)

    img = Image.new("RGB", (size, size), (20, 20, 24))
    d = ImageDraw.Draw(img)
    floor = [b for b in brushes if b.maxs[2] <= HEIGHT]
    for b in sorted(floor, key=lambda b: b.maxs[2]):
        t = max(0.0, min(1.0, b.maxs[2] / BALCONY_Z))
        color = (60, 60, 70) if b.maxs[2] >= HEIGHT else (
            int(70 + 130 * t), int(90 + 80 * t), int(110 - 40 * t))
        (x0, y0), (x1, y1) = px(b.mins[0], b.maxs[1]), px(b.maxs[0], b.mins[1])
        d.rectangle([x0, y0, x1, y1], fill=color, outline=(15, 15, 18))
    for (x, y, _), _, _ in lights():
        cx, cy = px(x, y)
        d.ellipse([cx - 4, cy - 4, cx + 4, cy + 4], fill=(255, 230, 120))
    for cls, (x, y, _), angle in spawns():
        cx, cy = px(x, y)
        color = (230, 70, 60) if cls == "info_player_duel" else (90, 200, 110)
        d.ellipse([cx - 8, cy - 8, cx + 8, cy + 8], fill=color)
        dx = {0: (1, 0), 45: (1, -1), 90: (0, -1), 180: (-1, 0), 225: (-1, 1), 270: (0, 1)}[angle]
        d.line([cx, cy, cx + 16 * dx[0], cy + 16 * dx[1]], fill=(255, 255, 255), width=2)
    img.save(path)


# ---------------------------------------------------------------- compile

def compile_map(q3map2, gamedata, home, mapfile):
    common = [q3map2, "-game", "ja", "-fs_basepath", str(gamedata), "-fs_homepath", str(home)]
    for stage in (["-meta"], ["-vis", "-saveprt"], ["-light", "-fast", "-samples", "2"]):
        target = mapfile if stage == ["-meta"] else mapfile.with_suffix(".bsp")
        args = common + (["-bsp"] if stage == ["-meta"] else []) + stage + [str(target)]
        res = subprocess.run(args, capture_output=True, text=True, errors="replace")
        log = mapfile.with_suffix(".log")
        with log.open("a", encoding="utf-8") as f:
            f.write(" ".join(args) + "\n" + res.stdout + res.stderr + "\n")
        if res.returncode != 0 or "leaked" in res.stdout.lower():
            sys.exit("q3map2 %s failed on %s, see %s" % (stage[0], mapfile.name, log))


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
    print("wrote %s: %d brushes" % (mapfile, len(brushes)))
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
    print("wrote", pk3)
    if args.install:
        shutil.copy2(pk3, gamedata / "base" / pk3.name)
        print("installed", gamedata / "base" / pk3.name)


if __name__ == "__main__":
    main()
