#!/usr/bin/env python3
"""Build the shader test maps: every retail world shader on a labelled pad.

Usage:
    python scripts/shader_testmap.py [--gamedata DIR] [--q3map2 EXE]
        [--out DIR] [--install] [--no-compile]

Reads the `textures/...` shaders of GameData/base/assets*.pk3 and writes, in
--out (default S:/Tools/shadertest):

- sjk_shaders_a.map .. sjk_shaders_d.map: galleries. Each shader file is a
  section that starts with a yellow header plaque; each shader is a panel
  (brush with the shader on every face) over a label naming its ID, its name
  and its kind. Mirrors get a misc_portal_surface, water/lava/slime and
  surface-sprite shaders are floor tiles and fog shaders are fog cubes (at most
  28 per map: q3map2 and the renderer's sort key take about 30 fogs per map).
  The wall behind the spawn point repeats the section plaques with the
  `setviewpos` that leads to each section.
- sjk_shaders_s.map: the sky shaders, each the ceiling of its own room off a
  corridor, labelled at the door and inside.
- sjk_shadertest.pk3: the compiled maps, the label atlases and
  shaders/sjk_shadertest.shader. --install copies it to GameData/base, where
  both SJK and EternalJK load it (`devmap sjk_shaders_a`).
- index.txt and index.json: every ID with its shader, file and kind, the
  skipped shaders (nodraw and BSP tool shaders) and each section's setviewpos.

Duplicated shader names keep the definition of the last shader file in name
order, as ioq3's ScanAndLoadShaderFiles does; the index lists the others.

Needs Python 3 with Pillow, Consolas (Windows) for the labels, and q3map2 from
NetRadiant-custom (`-game ja`) unless --no-compile.
"""
import argparse
import json
import re
import shutil
import struct
import subprocess
import sys
import zipfile
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

DEFAULT_GAMEDATA = r"C:\Program Files (x86)\Steam\steamapps\common\Jedi Academy\GameData"
DEFAULT_Q3MAP2 = r"S:\Tools\netradiant-custom\q3map2.exe"
DEFAULT_OUT = r"S:\Tools\shadertest"

PREFIX = "sjk_shadertest"  # textures/<PREFIX>/ and shaders/<PREFIX>.shader
DETAIL = 0x8000000  # CONTENTS_DETAIL in a brush face's content flags

# Gallery layout, in game units. A slot is a 160-wide panel standing on its
# label; rows face -Y and the walkway is in front of each row.
COLS = 20
PITCH_X = 224
PITCH_Y = 320
PAD = 160
LABEL_W, LABEL_H = 160, 40
HALL_HEIGHT = 384
MAP_CAPACITY = 480  # slots per gallery map, headers included
FOGS_PER_MAP = 28

# Label atlas: 2048 square, cells of 384x96 texels (2.4 texels per unit).
ATLAS = 2048
CELL_W, CELL_H = 384, 96
CELLS_X, CELLS_Y = ATLAS // CELL_W, ATLAS // CELL_H

TILE = 64  # sky map grid
SKY_CELL = 6  # room side, in tiles
SKY_ROOMS_PER_SIDE = 10

KIND_TAG = {"mirror": "MIRROR", "liquid": "LIQUID", "sprites": "SPRITES",
            "fog": "FOG", "sky": "SKY"}
TOOL_PARMS = {"nodraw", "origin", "areaportal", "hint", "skip"}


# ---------------------------------------------------------------- shaders

def tokens(text):
    text = re.sub(r"//[^\n]*", "", text)
    return re.findall(r'\{|\}|"[^"]*"|[^\s{}]+', text)


def read_shaders(base):
    """Return {name: {file, body, also}} in definition order, last file wins."""
    files = []
    for pk3 in sorted(base.glob("assets*.pk3")):
        with zipfile.ZipFile(pk3) as z:
            for n in z.namelist():
                if n.lower().startswith("shaders/") and n.lower().endswith(".shader"):
                    files.append((n.split("/")[-1].lower(), z.read(n).decode("latin1")))
    files.sort(key=lambda f: f[0])
    shaders = {}
    for fname, text in files:
        toks = tokens(text)
        i = 0
        while i < len(toks):
            name = toks[i].lower().strip('"')
            i += 1
            if i >= len(toks) or toks[i] != "{":
                continue
            depth, body = 0, []
            while i < len(toks):
                t = toks[i]
                i += 1
                if t == "{":
                    depth += 1
                elif t == "}":
                    depth -= 1
                    if depth == 0:
                        break
                body.append((depth, t.lower()))
            old = shaders.pop(name, None)
            also = (old["also"] + [old["file"]]) if old else []
            shaders[name] = {"file": fname, "body": body, "also": also}
    return shaders


def classify(body):
    top = [t for d, t in body if d == 1]
    words = [t for _, t in body]
    parms = {top[i + 1] for i, t in enumerate(top[:-1]) if t == "surfaceparm"}
    if "skyparms" in top or "sky" in parms:
        return "sky"
    if parms & TOOL_PARMS:
        return "skip:" + ",".join(sorted(parms & TOOL_PARMS))
    if "fogparms" in top:
        return "fog"
    if "portal" in top:
        return "mirror"
    if parms & {"water", "lava", "slime"}:
        return "liquid"
    if "surfacesprites" in words:
        return "sprites"
    return "panel"


# ---------------------------------------------------------------- .map output

def cross(a, b):
    return (a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0])


def sub(a, b):
    return tuple(x - y for x, y in zip(a, b))


class Face:
    def __init__(self, shader, shift=(0, 0), scale=(0.5, 0.5)):
        self.shader, self.shift, self.scale = shader, shift, scale


def box(mins, maxs, faces, contents=0):
    """An axis-aligned brush; faces maps '-x' .. '+z' (or '*') to a Face."""
    lines = ["{"]
    for axis in range(3):
        for sign in (-1, 1):
            key = ("-" if sign < 0 else "+") + "xyz"[axis]
            face = faces.get(key, faces.get("*"))
            coord = maxs[axis] if sign > 0 else mins[axis]
            u, v = [k for k in range(3) if k != axis]
            pts = []
            for du, dv in ((0, 0), (1, 0), (0, 1)):
                p = [0, 0, 0]
                p[axis] = coord
                p[u] = mins[u] + du * 64
                p[v] = mins[v] + dv * 64
                pts.append(tuple(p))
            a, b, c = pts
            n = cross(sub(c, a), sub(b, a))  # q3map2's PlaneFromPoints
            if n[axis] * sign < 0:
                b, c = c, b
            shader = face.shader[len("textures/"):]
            lines.append("( %d %d %d ) ( %d %d %d ) ( %d %d %d ) %s %.4f %.4f 0 %.6f %.6f %d 0 0"
                         % (*a, *b, *c, shader, face.shift[0], face.shift[1],
                            face.scale[0], face.scale[1], contents))
    lines.append("}")
    return "\n".join(lines)


def entity(fields, brushes=()):
    out = ["{"] + ['"%s" "%s"' % kv for kv in fields.items()]
    out += list(brushes)
    out.append("}")
    return "\n".join(out)


# ---------------------------------------------------------------- labels

class Atlas:
    """Label cells packed into 2048 atlases; one shader per atlas."""

    def __init__(self, tag):
        self.tag, self.images, self.used = tag, [], 0
        self.fonts = {
            "id": ImageFont.truetype(r"C:\Windows\Fonts\consolab.ttf", 34),
            "tag": ImageFont.truetype(r"C:\Windows\Fonts\consolab.ttf", 20),
            "text": ImageFont.truetype(r"C:\Windows\Fonts\consola.ttf", 19),
        }

    def shader(self, index):
        return "textures/%s/labels_%s%d" % (PREFIX, self.tag, index)

    def add(self, title, tag, lines, header=False):
        """Draw a cell; return (shader, cell x, cell y) in texels."""
        index, slot = divmod(self.used, CELLS_X * CELLS_Y)
        self.used += 1
        if index == len(self.images):
            self.images.append(Image.new("RGB", (ATLAS, ATLAS), (0, 0, 0)))
        cx, cy = (slot % CELLS_X) * CELL_W, (slot // CELLS_X) * CELL_H
        bg, fg, edge = ((232, 196, 40), (0, 0, 0), (120, 90, 0)) if header else \
            ((22, 30, 42), (235, 235, 235), (90, 110, 140))
        d = ImageDraw.Draw(self.images[index])
        d.rectangle([cx, cy, cx + CELL_W - 1, cy + CELL_H - 1], fill=bg)
        d.rectangle([cx + 2, cy + 2, cx + CELL_W - 3, cy + CELL_H - 3], outline=edge, width=2)
        d.text((cx + 10, cy + 6), title, font=self.fonts["id"], fill=fg)
        if tag:
            w = d.textlength(tag, font=self.fonts["tag"])
            d.text((cx + CELL_W - 12 - w, cy + 12), tag, font=self.fonts["tag"],
                   fill=(255, 170, 60) if not header else fg)
        for n, line in enumerate(lines[:2]):
            d.text((cx + 10, cy + 46 + n * 22), line, font=self.fonts["text"], fill=fg)
        return self.shader(index), cx, cy

    def save(self, textures):
        for i, img in enumerate(self.images):
            img.save(textures / ("labels_%s%d.tga" % (self.tag, i)))

    def shader_text(self):
        out = []
        for i in range(len(self.images)):
            name = self.shader(i)
            out.append("%s\n{\n\tqer_editorimage %s.tga\n\tsurfaceparm nolightmap\n"
                       "\tsurfaceparm nomarks\n\tnopicmip\n\t{\n\t\tmap %s\n"
                       "\t\trgbGen identity\n\t}\n}\n" % (name, name, name))
        return "\n".join(out)


def wrap(name, width=33):
    name = name[len("textures/"):]
    return [name[i:i + width] for i in range(0, len(name), width)] or [""]


def label_brush(cell, x0, z0, plane_y, facing):
    """An 8-deep label plate whose front face is the plane y = plane_y, facing
    -y (facing < 0) or +y; the plate extends behind its front face."""
    shader, cx, cy = cell
    s = LABEL_W / CELL_W
    if facing < 0:  # front face is -y, read looking +y: text runs along +x
        mins, maxs = (x0, plane_y, z0), (x0 + LABEL_W, plane_y + 8, z0 + LABEL_H)
        front = Face(shader, (cx - x0 / s, cy + (z0 + LABEL_H) / s), (s, s))
        key = "-y"
    else:  # front face is +y, read looking -y: text runs along -x
        mins, maxs = (x0, plane_y - 8, z0), (x0 + LABEL_W, plane_y, z0 + LABEL_H)
        front = Face(shader, (cx + (x0 + LABEL_W) / s, cy + (z0 + LABEL_H) / s), (-s, s))
        key = "+y"
    return box(mins, maxs, {"*": Face("textures/%s/dark" % PREFIX), key: front}, DETAIL)


def label_lines(info):
    lines = wrap(info["name"])
    if info["also"] and len(lines) == 1:
        lines.append("also in " + ", ".join(info["also"]))
    return lines


# ---------------------------------------------------------------- gallery maps

def plan_galleries(shaders):
    sections, fogs = {}, []
    for name, info in shaders.items():
        if not name.startswith("textures/") or info["kind"] in ("sky",) or \
                info["kind"].startswith("skip"):
            continue
        if info["kind"] == "fog":
            fogs.append(name)
        else:
            sections.setdefault(info["file"], []).append(name)
    maps, current, used = [], [], 0
    for fname, names in sections.items():
        need = len(names) + 1
        if current and used + need > MAP_CAPACITY:
            maps.append(current)
            current, used = [], 0
        current.append((fname, names))
        used += need
    maps.append(current)
    while len(fogs) > FOGS_PER_MAP * len(maps):
        maps.append([])
    per = -(-len(fogs) // len(maps))
    for i, m in enumerate(maps):
        chunk = fogs[i * per:(i + 1) * per]
        if chunk:
            m.append(("fog volumes", chunk))
    return maps


def build_gallery(letter, sections, shaders, index):
    atlas = Atlas(letter.lower())
    brushes, ents, board = [], [], []
    slot, number = 0, 0
    detail = DETAIL
    for fname, names in sections:
        col, row = slot % COLS, slot // COLS
        x0, ry = col * PITCH_X + 32, row * PITCH_Y
        first = "%s%03d" % (letter, number + 1)
        last = "%s%03d" % (letter, number + len(names))
        goto = "setviewpos %d %d 40 90" % (x0 + PAD // 2, ry - 240)
        cell = atlas.add(fname, "%d" % len(names), ["%s - %s" % (first, last), goto], header=True)
        brushes.append(label_brush(cell, x0, 100, ry, -1))
        board.append(cell)
        index["sections"].append({"map": "sjk_shaders_" + letter.lower(), "section": fname,
                                  "first": first, "last": last, "goto": goto})
        slot += 1
        for name in names:
            info = shaders[name]
            number += 1
            ident = "%s%03d" % (letter, number)
            col, row = slot % COLS, slot // COLS
            x0, ry = col * PITCH_X + 32, row * PITCH_Y
            kind = info["kind"]
            tag = KIND_TAG.get(kind, "") + (" DUP" if info["also"] else "")
            cell = atlas.add(ident, tag.strip(), label_lines(info))
            face = {"*": Face(name)}
            if kind in ("liquid", "sprites"):
                brushes.append(box((x0, ry - 176, 0), (x0 + PAD, ry - 16, 16), face, detail))
                brushes.append(label_brush(cell, x0, 24, ry, -1))
            elif kind == "fog":
                brushes.append(box((x0, ry - 176, 0), (x0 + PAD, ry - 16, PAD), face, detail))
                brushes.append(label_brush(cell, x0, 4, ry, -1))
            else:
                brushes.append(box((x0, ry, 48), (x0 + PAD, ry + 8, 48 + PAD), face, detail))
                brushes.append(label_brush(cell, x0, 4, ry, -1))
                if kind == "mirror":
                    ents.append(entity({"classname": "misc_portal_surface",
                                        "origin": "%d %d %d" % (x0 + PAD // 2, ry - 16, 128)}))
            index["ids"][ident] = {"shader": name, "file": info["file"], "kind": kind,
                                   "also": info["also"], "map": "sjk_shaders_" + letter.lower()}
            slot += 1
    rows = slot // COLS + 1
    xmin, xmax = -64, COLS * PITCH_X + 32
    ymin, ymax = -384, rows * PITCH_Y
    grid = {"*": Face("textures/%s/grid" % PREFIX)}
    brushes += hall(xmin, xmax, ymin, ymax, grid)
    for i, cell in enumerate(board):  # contents board on the wall behind spawn
        per_row = (xmax - xmin - 64) // 192
        x = xmax - 64 - LABEL_W - (i % per_row) * 192
        brushes.append(label_brush(cell, x, 60 + (i // per_row) * 56, ymin + 8, 1))
    for y in range(-150, ymax, PITCH_Y):  # over each walkway
        for x in range(xmin + 224, xmax, 448):
            ents.append(entity({"classname": "light", "origin": "%d %d 330" % (x, y),
                                "light": "900"}))
    for cls in ("info_player_deathmatch", "info_player_start"):
        ents.append(entity({"classname": cls, "origin": "112 -260 40", "angle": "90"}))
    world = entity({"classname": "worldspawn", "message": "SJK shader gallery " + letter,
                    "_ambient": "40", "_blocksize": "1024 1024 1024"}, brushes)
    return "\n".join([world] + ents) + "\n", atlas


def hall(xmin, xmax, ymin, ymax, face, height=HALL_HEIGHT, t=16):
    return [
        box((xmin - t, ymin - t, -t), (xmax + t, ymax + t, 0), face),
        box((xmin - t, ymin - t, height), (xmax + t, ymax + t, height + t), face),
        box((xmin - t, ymin - t, 0), (xmin, ymax + t, height), face),
        box((xmax, ymin - t, 0), (xmax + t, ymax + t, height), face),
        box((xmin, ymin - t, 0), (xmax, ymin, height), face),
        box((xmin, ymax, 0), (xmax, ymax + t, height), face),
    ]


# ---------------------------------------------------------------- sky map

def merge_rects(tiles, w, h):
    """Merge equal tile values into rectangles: [(x0, y0, x1, y1, value)]."""
    rects, open_runs = [], {}
    for y in range(h + 1):
        runs = {}
        if y < h:
            x = 0
            while x < w:
                v = tiles[y][x]
                x1 = x
                while x1 + 1 < w and tiles[y][x1 + 1] == v:
                    x1 += 1
                if v is not None:
                    runs[(x, x1, v)] = None
                x = x1 + 1
        nxt = {}
        for key, y0 in open_runs.items():
            if key in runs:
                nxt[key] = y0
            else:
                rects.append((key[0], y0, key[1] + 1, y, key[2]))
        for key in runs:
            if key not in nxt:
                nxt[key] = y
        open_runs = nxt
    return rects


def build_sky(names, shaders, index):
    letter = "S"
    atlas = Atlas("s")
    bands = -(-len(names) // (2 * SKY_ROOMS_PER_SIDE))
    w = 4 + SKY_ROOMS_PER_SIDE * (SKY_CELL + 1) + 1
    h = bands * 18 + 1
    grid_shader = "textures/%s/grid" % PREFIX
    solid = [[True] * w for _ in range(h)]
    ceil = [[grid_shader] * w for _ in range(h)]
    for y in range(1, h - 1):
        for x in range(1, 4):
            solid[y][x] = False  # cross corridor
    brushes, ents = [], []
    n = 0
    for b in range(bands):
        y0 = b * 18
        for y in range(y0 + 8, y0 + 11):
            for x in range(1, w - 1):
                solid[y][x] = False  # band corridor
        ents.append(entity({"classname": "light", "origin": "%d %d 330" % (2 * TILE, (y0 + 9.5) * TILE),
                            "light": "500"}))
        for side, (cy0, door_row) in enumerate(((y0 + 1, y0 + 7), (y0 + 12, y0 + 11))):
            for k in range(SKY_ROOMS_PER_SIDE):
                if n >= len(names):
                    break
                name = names[n]
                n += 1
                ident = "%s%03d" % (letter, n)
                cx0 = 5 + k * (SKY_CELL + 1)
                for y in range(cy0, cy0 + SKY_CELL):
                    for x in range(cx0, cx0 + SKY_CELL):
                        solid[y][x] = False
                        ceil[y][x] = name
                for x in (cx0 + 2, cx0 + 3):
                    solid[door_row][x] = False
                info = shaders[name]
                tag = "SKY" + (" DUP" if info["also"] else "")
                cell = atlas.add(ident, tag, label_lines(info))
                # A lintel over the door carries a plate on each face of the
                # wall row: the +y face is the corridor's for side 0 (room
                # below) and the room's for side 1.
                brushes.append(box(((cx0 + 2) * TILE, door_row * TILE, 224),
                                   ((cx0 + 4) * TILE, (door_row + 1) * TILE, HALL_HEIGHT),
                                   {"*": Face(grid_shader)}))
                lx = (cx0 + 3) * TILE - LABEL_W // 2
                brushes.append(label_brush(cell, lx, 240, (door_row + 1) * TILE + 8, 1))
                brushes.append(label_brush(cell, lx, 240, door_row * TILE - 8, -1))
                centre = ((cx0 + 3) * TILE, (cy0 + 3) * TILE)
                ents.append(entity({"classname": "light", "origin": "%d %d 300" % centre,
                                    "light": "600"}))
                ents.append(entity({"classname": "light", "origin": "%d %d 330" % (
                    (cx0 + 3) * TILE, (y0 + 9.5) * TILE), "light": "500"}))
                goto = "setviewpos %d %d 40 %d" % (centre[0], centre[1], 90)
                index["ids"][ident] = {"shader": name, "file": info["file"], "kind": "sky",
                                       "also": info["also"], "map": "sjk_shaders_s", "goto": goto}
    index["sections"].append({"map": "sjk_shaders_s", "section": "skies", "first": "S001",
                              "last": "%s%03d" % (letter, n), "goto": "setviewpos 128 576 40 90"})
    walls = [[True if solid[y][x] else None for x in range(w)] for y in range(h)]
    for x0, y0, x1, y1, _ in merge_rects(walls, w, h):
        brushes.append(box((x0 * TILE, y0 * TILE, 0), (x1 * TILE, y1 * TILE, HALL_HEIGHT),
                           {"*": Face(grid_shader)}))
    for x0, y0, x1, y1, shader in merge_rects(ceil, w, h):
        brushes.append(box((x0 * TILE, y0 * TILE, HALL_HEIGHT), (x1 * TILE, y1 * TILE, HALL_HEIGHT + 16),
                           {"*": Face(shader)}))
    brushes.append(box((0, 0, -16), (w * TILE, h * TILE, 0), {"*": Face(grid_shader)}))
    # A roof over the ceilings: some sky shaders are non-solid and do not seal.
    brushes.append(box((0, 0, HALL_HEIGHT + 16), (w * TILE, h * TILE, HALL_HEIGHT + 32),
                       {"*": Face(grid_shader)}))
    for cls in ("info_player_deathmatch", "info_player_start"):
        ents.append(entity({"classname": cls, "origin": "128 576 40", "angle": "90"}))
    world = entity({"classname": "worldspawn", "message": "SJK shader gallery S (skies)",
                    "_ambient": "40", "_blocksize": "1024 1024 1024"}, brushes)
    return "\n".join([world] + ents) + "\n", atlas


# ---------------------------------------------------------------- own assets

def write_assets(stage, atlases, shader_files):
    textures = stage / "textures" / PREFIX
    textures.mkdir(parents=True, exist_ok=True)
    img = Image.new("RGB", (256, 256), (118, 118, 118))
    d = ImageDraw.Draw(img)
    for i in range(0, 256, 32):
        d.line([(i, 0), (i, 255)], fill=(100, 100, 100))
        d.line([(0, i), (255, i)], fill=(100, 100, 100))
    d.rectangle([0, 0, 255, 255], outline=(70, 70, 70), width=2)
    img.save(textures / "grid.tga")
    Image.new("RGB", (64, 64), (40, 40, 44)).save(textures / "dark.tga")
    text = []
    for name in ("grid", "dark"):
        full = "textures/%s/%s" % (PREFIX, name)
        text.append("%s\n{\n\tqer_editorimage %s.tga\n\t{\n\t\tmap $lightmap\n\t\trgbGen identity\n"
                    "\t}\n\t{\n\t\tmap %s\n\t\tblendFunc GL_DST_COLOR GL_ZERO\n\t\trgbGen identity\n"
                    "\t}\n}\n" % (full, full, full))
    for atlas in atlases:
        atlas.save(textures)
        text.append(atlas.shader_text())
    (stage / "shaders").mkdir(exist_ok=True)
    (stage / "shaders" / (PREFIX + ".shader")).write_text("\n".join(text), newline="\n")
    # q3map2 reads only the shader files a shaderlist.txt names, and the
    # retail list misses some, whose fog, sky and water shaders would then
    # compile as plain walls. For the compile only; not in the pk3.
    names = sorted({f[:-len(".shader")] for f in shader_files} | {PREFIX})
    (stage / "shaders" / "shaderlist.txt").write_text("\n".join(names) + "\n", newline="\n")


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


def lightmap_mean(bsp):
    data = bsp.read_bytes()
    off, length = struct.unpack_from("<ii", data, 8 + 14 * 8)
    lm = data[off:off + length]
    return (sum(lm) / len(lm) if lm else 0.0), length // (128 * 128 * 3)


# ---------------------------------------------------------------- main

def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--gamedata", default=DEFAULT_GAMEDATA)
    ap.add_argument("--q3map2", default=DEFAULT_Q3MAP2)
    ap.add_argument("--out", default=DEFAULT_OUT)
    ap.add_argument("--install", action="store_true")
    ap.add_argument("--no-compile", action="store_true")
    args = ap.parse_args()
    gamedata, out = Path(args.gamedata), Path(args.out)
    home = out / "home"
    stage = home / "base"
    if out.exists():
        shutil.rmtree(out)
    (stage / "maps").mkdir(parents=True)

    shaders = read_shaders(gamedata / "base")
    for name, info in shaders.items():
        info["kind"] = classify(info["body"])
        info["name"] = name
    world = {n: i for n, i in shaders.items() if n.startswith("textures/")}

    index = {"ids": {}, "sections": [], "skipped": {}}
    for name, info in world.items():
        if info["kind"].startswith("skip"):
            index["skipped"][name] = info["kind"][5:]
    atlases, maps = [], []
    for i, sections in enumerate(plan_galleries(world)):
        letter = "ABCDEFGHIJKLMNOPQR"[i]
        text, atlas = build_gallery(letter, sections, world, index)
        maps.append(("sjk_shaders_" + letter.lower(), text))
        atlases.append(atlas)
    skies = [n for n, i in world.items() if i["kind"] == "sky"]
    text, atlas = build_sky(skies, world, index)
    maps.append(("sjk_shaders_s", text))
    atlases.append(atlas)
    files = {i["file"] for i in shaders.values()}
    files |= {f for i in shaders.values() for f in i["also"]}
    write_assets(stage, atlases, files)
    for name, text in maps:
        (stage / "maps" / (name + ".map")).write_text(text, newline="\n")

    write_index(out, index)
    if args.no_compile:
        print("wrote %d maps in %s (not compiled)" % (len(maps), stage / "maps"))
        return
    for name, _ in maps:
        mapfile = stage / "maps" / (name + ".map")
        print("compiling", name, flush=True)
        compile_map(args.q3map2, gamedata, home, mapfile)
        mean, count = lightmap_mean(mapfile.with_suffix(".bsp"))
        print("  %d lightmaps, mean %.0f" % (count, mean), flush=True)
    pk3 = out / (PREFIX + ".pk3")
    with zipfile.ZipFile(pk3, "w", zipfile.ZIP_DEFLATED) as z:
        for name, _ in maps:
            z.write(stage / "maps" / (name + ".bsp"), "maps/%s.bsp" % name)
        z.write(stage / "shaders" / (PREFIX + ".shader"), "shaders/%s.shader" % PREFIX)
        for tga in sorted((stage / "textures" / PREFIX).glob("*.tga")):
            z.write(tga, "textures/%s/%s" % (PREFIX, tga.name))
    print("wrote", pk3)
    if args.install:
        shutil.copy2(pk3, gamedata / "base" / pk3.name)
        print("installed", gamedata / "base" / pk3.name)


def write_index(out, index):
    (out / "index.json").write_text(json.dumps(index, indent=1), encoding="utf-8")
    lines = ["SJK shader test maps. `devmap <map>`, then `noclip`; IDs are on the labels.", ""]
    for s in index["sections"]:
        lines.append("%-14s %-24s %s-%s  %s" % (s["map"], s["section"], s["first"], s["last"], s["goto"]))
    lines.append("")
    for ident, e in index["ids"].items():
        extra = ("  dup of " + ",".join(e["also"])) if e["also"] else ""
        lines.append("%s  %-50s %-22s %s%s" % (ident, e["shader"], e["file"], e["kind"], extra))
    lines += ["", "Skipped (invisible tool shaders):"]
    lines += ["  %-50s %s" % kv for kv in index["skipped"].items()]
    (out / "index.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
