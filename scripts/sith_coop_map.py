#!/usr/bin/env python3
"""Build sjk_sith_coop, a co-op Sith temple: outdoors, a bridge, seven rooms.

Usage:
    python scripts/sith_coop_map.py [--gamedata DIR] [--q3map2 EXE]
        [--out DIR] [--install] [--no-compile] [--preview PNG]

The players start outside under a red volcanic sky beside an X-wing (the
game's own vehicle; an X-wing wreck, the game's debris models, smoulders by the
chasm), cross a long bridge over a lava chasm lined with hooded statues, and
enter the temple. Long, quiet passages lead from room to room. Rooms 1 to 4
are on the ground; room 4 is a tall hall whose lift (`func_plat`, with rails,
glowing posts and its own sound) rises to a balcony and a long narrow
corridor, which leads to rooms 5 to 7. Room 7 is a huge hall with a giant
Sith statue in the middle, lava channels and, past its middle, the last
enemies on a raised dais.

Music and sounds are made by sith_audio.py: an original dark score in D minor
(`music` in worldspawn), the lift's ride, and looping wind and lava speakers.

Enemies are the game's own NPCs, harder room by room, most of them users of
the dark side of the Force: cultists with grip, lightning and drain, saber
cultists that throw, Reborn, shadowtroopers, Alora, Tavion and, last,
Desann. Each room's spawners wait for a `trigger_once` at its door, so a
room fills when the players walk in; two cultists already wait outside.
`waypoint` entities give the NPCs their navigation graph.

Play it in Free For All (all players and NPCs' foes on one side): in team
game types the blue team is the NPCs' own. Players respawn at the start.

Reuses sith_map.py's textures, statue and helpers (installed with this map
as textures/sjk_sith and shaders/sjk_sith.shader), and adds its own
(textures/sjk_sith_coop: ground, cliffs, the sky box, the trigger).

Needs Python 3 with Pillow and NumPy, and q3map2 from NetRadiant-custom
(`-game ja`) unless --no-compile.
"""
import argparse
import math
import shutil
import sys
import zipfile
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

import sith_map as sith
from arena_map import compile_map
import sith_audio
from sith_map import (FLOOR, ROCK, STONE, TRIM, WALLF, Brush, Face, box, entity,
                      octagon, rgb, ridges, fbm, tint)

DEFAULT_GAMEDATA = r"C:\Program Files (x86)\Steam\steamapps\common\Jedi Academy\GameData"
DEFAULT_Q3MAP2 = r"S:\Tools\netradiant-custom\q3map2.exe"
DEFAULT_OUT = r"S:\Tools\sith_coop"

MAP = "sjk_sith_coop"
COOP = "sjk_sith_coop"  # textures/<COOP>/, shaders/<COOP>.shader
T = 32  # wall, floor and ceiling thickness

GROUND = Face(COOP + "/ground", (1.0, 1.0))
CLIFF = Face(COOP + "/cliff", (1.0, 1.0))
SKY = Face(COOP + "/sky")
TRIGGER = COOP + "/trigger"
LAMP = Face("lamp", (1.0, 1.0))
RUNES = Face("runes", (0.25, 0.25))
CEILING = Face("ceiling")
LAVA = Face("lava", (0.5, 0.5))

# Outdoors: the start plateau, the chasm the bridge crosses, the porch.
OUT_W = 1024
START_Y0, CHASM_Y0, CHASM_Y1, FACADE_Y = -3840, -2560, -768, -544
OUT_TOP, CLIFF_TOP, CHASM_FLOOR = 1024, 512, -576
BRIDGE_W = 128

# The rooms, south to north: half width, depth, height, floor, and the door
# out (width, height, its sill above this room's floor). Each is followed by
# a passage of the given length to the next.
ROOMS = [
    dict(w=512, d=512, h=384, z=0, door=(192, 192, 0), style="obsidian"),
    dict(w=640, d=768, h=448, z=0, door=(192, 224, 0), style="sandstone"),
    dict(w=768, d=896, h=512, z=0, door=(224, 256, 0), style="forge"),
    dict(w=640, d=768, h=960, z=0, door=(128, 160, 640), style="crimson"),
    dict(w=768, d=768, h=512, z=640, door=(224, 256, 0), style="archive"),
    dict(w=896, d=896, h=576, z=640, door=(256, 288, 0), style="jade"),
    dict(w=1280, d=1792, h=1280, z=640, door=None, style="throne"),
]

# Each room has its own stone, metal and light: (wall kind and colours: low,
# high, mortar, veins, vein strength), (floor kind, low, high, inlay), trim,
# ceiling, the runes' glow, the lamps' light and the pillars. The first room
# keeps sith_map.py's obsidian.
PALETTES = {
    "sandstone": dict(wall=("blocks", (140, 110, 72), (205, 170, 115), (90, 70, 45), (120, 60, 30), 0.25),
                      floor=("slabs", (150, 122, 82), (200, 168, 120), (170, 120, 40)),
                      trim=((120, 90, 30), (215, 175, 80)), ceiling=((90, 72, 50), (140, 115, 80)),
                      runes=(255, 170, 50), lamp=(255, 215, 160), pillar=((160, 130, 90), (215, 185, 135))),
    "forge": dict(wall=("plates", (42, 42, 48), (84, 80, 84), (18, 18, 20), (140, 70, 30), 0.5),
                  floor=("grate", (36, 36, 40), (74, 72, 76), (255, 110, 20)),
                  trim=((50, 40, 34), (120, 80, 50)), ceiling=((14, 12, 12), (40, 34, 32)),
                  runes=(255, 120, 20), lamp=(255, 175, 100), pillar=((50, 50, 56), (95, 92, 98))),
    "crimson": dict(wall=("marble", (90, 16, 20), (165, 40, 42), (30, 8, 10), (230, 200, 200), 0.35),
                    floor=("checker", (18, 16, 20), (54, 48, 56), (190, 150, 60)),
                    trim=((20, 18, 20), (190, 150, 60)), ceiling=((40, 8, 10), (80, 20, 24)),
                    runes=(255, 40, 30), lamp=(255, 228, 215), pillar=((25, 22, 26), (70, 62, 72))),
    "archive": dict(wall=("blocks", (38, 28, 54), (84, 64, 104), (14, 10, 20), (180, 120, 255), 0.2),
                    floor=("slabs", (30, 28, 38), (66, 60, 80), (170, 90, 255)),
                    trim=((70, 72, 84), (170, 175, 190)), ceiling=((20, 14, 30), (48, 36, 64)),
                    runes=(200, 90, 255), lamp=(235, 215, 255), pillar=((45, 36, 60), (95, 80, 115))),
    "jade": dict(wall=("marble", (22, 48, 38), (62, 104, 84), (8, 20, 14), (170, 240, 200), 0.3),
                 floor=("slabs", (16, 26, 22), (44, 62, 54), (80, 255, 150)),
                 trim=((90, 66, 34), (170, 130, 70)), ceiling=((10, 22, 18), (30, 52, 42)),
                 runes=(80, 255, 150), lamp=(215, 255, 225), pillar=((30, 60, 48), (80, 125, 100))),
    "throne": dict(wall=("marble", (18, 16, 20), (48, 44, 52), (6, 6, 8), (220, 175, 70), 0.45),
                   floor=("checker", (70, 10, 14), (30, 26, 30), (220, 175, 70)),
                   trim=((40, 30, 14), (225, 185, 85)), ceiling=((10, 9, 12), (30, 26, 32)),
                   runes=(255, 40, 30), lamp=(255, 236, 210), pillar=((20, 18, 22), (60, 54, 64))),
}
PARTS = ("wall", "floor", "trim", "ceiling", "runes", "lamp", "pillar")


def style_faces(style):
    """A style's faces by part; obsidian is sith_map.py's own set."""
    if style == "obsidian":
        names = dict(wall="wall", floor="floor", trim="trim", ceiling="ceiling", runes="runes",
                     lamp="lamp", pillar="stone")
    else:
        names = {k: "%s/%s_%s" % (COOP, style, k) for k in PARTS}
    return dict(wall=Face(names["wall"], (1.0, 1.0)), floor=Face(names["floor"]),
                trim=Face(names["trim"], (0.25, 0.25)), ceiling=Face(names["ceiling"]),
                runes=Face(names["runes"], (0.25, 0.25)), lamp=Face(names["lamp"], (1.0, 1.0)),
                pillar=Face(names["pillar"]))
PASSAGES = [384, 448, 512, 1024, 512, 576]
FACADE_DOOR = (256, 256)
BALCONY_D = 256
LIFT = 256  # the lift's side; it rises LIFT_HEIGHT to the balcony at LIFT_SPEED
LIFT_HEIGHT = 632
LIFT_SPEED = 140
LAMP_SPACING = 384
COOP_SOUNDS = "sound/" + COOP  # the lift's ride and the ambient loops
MUSIC = "music/%s/sith_temple.mp3" % COOP
# The X-wing at the start is the game's own vehicle; the wreck by the chasm is
# the debris its model breaks into, also the game's, placed client-side.
XWING = ("X-Wing", (-560, -3260, 48), 90)
FIELD, FIELD_HEIGHT = 320, 288  # the force field's half width and height round it
WRECK = "models/players/x-wing/x_wing_part%s.md3"
WRECK_PARTS = [  # (part, origin, "pitch yaw roll")
    ("_wing_l", (640, -2880, 40), "0 30 8"),
    ("_wing_r", (880, -3160, 36), "0 -60 -12"),
    ("1", (600, -3360, 40), "0 110 15"),
    ("_nose", (840, -2740, 18), "20 200 0"),
    ("3", (720, -3040, 26), "0 45 80"),
    ("2", (920, -2960, 16), "0 10 0"),
]


def layout():
    """Each room's interior y range, filled into ROOMS."""
    y = FACADE_Y + T
    for i, r in enumerate(ROOMS):
        r["y0"], r["y1"] = y, y + r["d"]
        if i < len(PASSAGES):
            y = r["y1"] + T + PASSAGES[i] + T


layout()


def door_of(i):
    """The door into room i: (width, absolute sill, absolute top)."""
    if i == 0:
        return FACADE_DOOR[0], 0, FACADE_DOOR[1]
    p = ROOMS[i - 1]
    w, h, sill = p["door"]
    return w, p["z"] + sill, p["z"] + sill + h


# ---------------------------------------------------------------- pieces

def wall_across(y0, y1, x0, x1, z0, z1, door, face):
    """A wall across the y axis, with a door centred on x = 0 if given."""
    if not door:
        return [box((x0, y0, z0), (x1, y1, z1), {"*": face})]
    w, d0, d1 = door
    out = [box((x0, y0, z0), (-w // 2, y1, z1), {"*": face}),
           box((w // 2, y0, z0), (x1, y1, z1), {"*": face})]
    if d0 > z0:
        out.append(box((-w // 2, y0, z0), (w // 2, y1, d0), {"*": face}))
    if d1 < z1:
        out.append(box((-w // 2, y0, d1), (w // 2, y1, z1), {"*": face}))
    return out


def floor_with_holes(x0, x1, y0, y1, top, holes, faces):
    """A floor slab with rectangular holes (x0, x1, y0, y1), as few boxes as rows allow."""
    xs = sorted({x0, x1, *[h[0] for h in holes], *[h[1] for h in holes]})
    ys = sorted({y0, y1, *[h[2] for h in holes], *[h[3] for h in holes]})
    out = []
    for j in range(len(ys) - 1):
        run = None
        for i in range(len(xs) - 1):
            cx0, cx1, cy0, cy1 = xs[i], xs[i + 1], ys[j], ys[j + 1]
            hole = any(h[0] <= cx0 and cx1 <= h[1] and h[2] <= cy0 and cy1 <= h[3] for h in holes)
            if hole:
                if run:
                    out.append(box((run[0], cy0, top - T), (run[1], cy1, top), faces))
                run = None
            else:
                run = (run[0] if run else cx0, cx1)
        if run:
            out.append(box((run[0], ys[j], top - T), (run[1], ys[j + 1], top), faces))
    return out


def lava_pit(x0, x1, y0, y1, top, depth=160):
    """Walls, bottom and lava under a hole in a floor whose surface is at top."""
    bottom = top - depth
    lava = Brush([(x0, y0), (x1, y0), (x1, y1), (x0, y1)], bottom, top - 48,
                 {"*": LAVA}, lava=True)
    return [
        box((x0 - T, y0 - T, bottom - T), (x1 + T, y1 + T, bottom), {"*": ROCK}),
        box((x0 - T, y0 - T, bottom), (x0, y1 + T, top - T), {"*": ROCK}),
        box((x1, y0 - T, bottom), (x1 + T, y1 + T, top - T), {"*": ROCK}),
        box((x0, y0 - T, bottom), (x1, y0, top - T), {"*": ROCK}),
        box((x0, y1, bottom), (x1, y1 + T, top - T), {"*": ROCK}),
        lava,
    ]


OBSIDIAN = style_faces("obsidian")


def octagon_brush(x, y, apothem, z0, z1, faces, detail=False):
    return Brush(octagon(x, y, apothem), z0, z1, faces, detail=detail)


# Pillars, braziers, lamps, sconces and rune bands are decorations: detail
# brushes, which q3map2 leaves out of the BSP tree and the visibility.
def pillar(x, y, z, h, f=None):
    f = f or OBSIDIAN
    return [octagon_brush(x, y, 40, z, z + h, {"*": f["pillar"]}, True),
            octagon_brush(x, y, 56, z, z + 32, {"*": f["trim"]}, True),
            octagon_brush(x, y, 56, z + h - 64, z + h - 32, {"*": f["trim"]}, True)]


def brazier(x, y, z, f=None):
    f = f or OBSIDIAN
    return [octagon_brush(x, y, 16, z, z + 48, {"*": f["trim"]}, True),
            octagon_brush(x, y, 32, z + 48, z + 64, {"*": f["trim"]}, True),
            octagon_brush(x, y, 28, z + 64, z + 68, {"*": f["trim"], "top": Face("embers")}, True)]


def lamp(x, y, ceiling, half=40, f=None):
    f = f or OBSIDIAN
    return box((x - half, y - half, ceiling - 16), (x + half, y + half, ceiling),
               {"*": f["trim"], "bottom": f["lamp"]}, detail=True)


def sconce(x_wall, y, z, f=None):
    """A glowing panel on a side wall at x = x_wall, facing into the room."""
    f = f or OBSIDIAN
    xs = sorted((x_wall, x_wall + (6 if x_wall < 0 else -6)))
    return box((xs[0], y - 32, z), (xs[1], y + 32, z + 96), {"*": f["trim"], "side": f["lamp"]},
               detail=True)


# ---------------------------------------------------------------- outdoors

def outdoors():
    o = OUT_W
    out = [
        # the plateau, the chasm's bed and the porch
        box((-o, START_Y0, CHASM_FLOOR), (o, CHASM_Y0, 0), {"*": CLIFF, "top": GROUND}),
        box((-o, CHASM_Y0, CHASM_FLOOR - T), (o, CHASM_Y1, CHASM_FLOOR), {"*": ROCK}),
        Brush([(-o, CHASM_Y0), (o, CHASM_Y0), (o, CHASM_Y1), (-o, CHASM_Y1)],
              CHASM_FLOOR, CHASM_FLOOR + 64, {"*": LAVA}, lava=True),
        box((-o, CHASM_Y1, CHASM_FLOOR), (o, FACADE_Y, 0), {"*": CLIFF, "top": GROUND}),
        # cliffs, and the sky above them
        box((-o - 64, START_Y0 - 64, CHASM_FLOOR - T), (-o, FACADE_Y, CLIFF_TOP), {"*": CLIFF}),
        box((o, START_Y0 - 64, CHASM_FLOOR - T), (o + 64, FACADE_Y, CLIFF_TOP), {"*": CLIFF}),
        box((-o, START_Y0 - 64, CHASM_FLOOR - T), (o, START_Y0, CLIFF_TOP), {"*": CLIFF}),
        box((-o - 64, START_Y0 - 64, CLIFF_TOP), (-o, FACADE_Y, OUT_TOP), {"*": SKY}),
        box((o, START_Y0 - 64, CLIFF_TOP), (o + 64, FACADE_Y, OUT_TOP), {"*": SKY}),
        box((-o, START_Y0 - 64, CLIFF_TOP), (o, START_Y0, OUT_TOP), {"*": SKY}),
        box((-o - 64, START_Y0 - 64, OUT_TOP), (o + 64, FACADE_Y + T, OUT_TOP + T), {"*": SKY}),
        box((-o - 64, FACADE_Y, 768), (o + 64, FACADE_Y + T, OUT_TOP), {"*": SKY}),
    ]
    # the facade, the temple's front and room 1's south wall
    w, d0, d1 = door_of(0)
    out += wall_across(FACADE_Y, FACADE_Y + T, -o - 64, o + 64, CHASM_FLOOR - T, 768,
                       (w, d0, d1), WALLF)
    # the bridge and its curbs
    b = BRIDGE_W
    out.append(box((-b, CHASM_Y0, -T), (b, CHASM_Y1, 0), {"*": TRIM, "top": FLOOR}))
    for s in (-1, 1):
        x0, x1 = sorted((s * b, s * (b + 16)))
        out.append(box((x0, CHASM_Y0, -T), (x1, CHASM_Y1, 16), {"*": TRIM}))
    # columns rising from the lava, each under a statue; braziers by the gate
    for x, y, _ in bridge_statues():
        out.append(octagon_brush(x, y, 64, CHASM_FLOOR, 0, {"*": STONE, "top": TRIM}))
    for x in (-224, 224):
        out += brazier(x, FACADE_Y - 112, 0)
    for x in (-512, 512):
        out += pillar(x, FACADE_Y - 160, 0, 448)
    # a red force field round the X-wing: solid, so nobody reaches the ship to board it
    field = Face(COOP + "/forcefield", (0.5, 0.5))
    _, (fx, fy, _), _ = XWING
    x0, x1, y0, y1, top = fx - FIELD, fx + FIELD, fy - FIELD, fy + FIELD, FIELD_HEIGHT
    for lo, hi in (((x0, y0, 0), (x1, y0 + 8, top)), ((x0, y1 - 8, 0), (x1, y1, top)),
                   ((x0, y0 + 8, 0), (x0 + 8, y1 - 8, top)), ((x1 - 8, y0 + 8, 0), (x1, y1 - 8, top)),
                   ((x0, y0, top), (x1, y1, top + 8))):
        out.append(box(lo, hi, {"*": field}, detail=True))
    for x, y in ((x0, y0), (x0, y1), (x1, y0), (x1, y1)):  # emitters at its corners
        out.append(octagon_brush(x, y, 20, 0, top + 24, {"*": TRIM, "top": Face("embers")}, True))
    # the wreck: an invisible clip round each piece, embers smouldering among them
    for _, (x, y, _), _ in WRECK_PARTS:
        out.append(box((x - 72, y - 72, 0), (x + 72, y + 72, 64), {"*": "clip"}))
    for x, y in ((760, -2980), (700, -3200), (860, -2860)):
        out.append(octagon_brush(x, y, 24, 0, 4, {"*": ROCK, "top": Face("embers")}))
    return out


def bridge_statues():
    """(x, y, angle) of the statues on both sides of the bridge, facing it."""
    return [(s * 240, y, 0 if s < 0 else 180)
            for y in range(CHASM_Y0 + 96, CHASM_Y1 - 95, 224) for s in (-1, 1)]


# ---------------------------------------------------------------- rooms

def room(i):
    r = ROOMS[i]
    f = style_faces(r["style"])
    w, y0, y1, zf, h = r["w"], r["y0"], r["y1"], r["z"], r["h"]
    zc = zf + h
    holes = HOLES.get(i, [])
    out = floor_with_holes(-w, w, y0, y1, zf, [hl[:4] for hl in holes],
                           {"*": f["trim"], "top": f["floor"]})
    for hl in holes:
        out += lava_pit(*hl[:4], zf)
    out.append(box((-w, y0, zc), (w, y1, zc + T), {"*": f["trim"], "bottom": f["ceiling"]}))
    out.append(box((-w - T, y0 - T, zf - T), (-w, y1 + T, zc + T), {"*": f["wall"]}))
    out.append(box((w, y0 - T, zf - T), (w + T, y1 + T, zc + T), {"*": f["wall"]}))
    if i > 0:
        out += wall_across(y0 - T, y0, -w, w, zf - T, zc + T, door_of(i), f["wall"])
    exit_door = door_of(i + 1) if i + 1 < len(ROOMS) else None
    out += wall_across(y1, y1 + T, -w, w, zf - T, zc + T, exit_door, f["wall"])
    # rune bands, cut where a door crosses them
    z0, z1 = zf + 192, zf + 208
    for y_face, inward in ((y0, 1), (y1, -1)):
        door = door_of(i) if inward > 0 else exit_door
        ys = sorted((y_face, y_face + inward * 6))
        cut = door and door[1] < z1 and door[2] > z0
        spans = [(-w, -door[0] // 2), (door[0] // 2, w)] if cut else [(-w, w)]
        for a, b in spans:
            out.append(box((a, ys[0], z0), (b, ys[1], z1), {"*": f["trim"], "side": f["runes"]},
                           detail=True))
    for x_face, inward in ((-w, 1), (w, -1)):
        xs = sorted((x_face, x_face + inward * 6))
        out.append(box((xs[0], y0 + 6, z0), (xs[1], y1 - 6, z1), {"*": f["trim"], "side": f["runes"]},
                       detail=True))
    # lamps on a grid under the ceiling, sconces on the side walls
    nx, ny = max(1, round(2 * w / LAMP_SPACING)), max(1, round(r["d"] / LAMP_SPACING))
    for a in range(nx):
        for b in range(ny):
            out.append(lamp(-w + (a + 0.5) * 2 * w / nx, y0 + (b + 0.5) * r["d"] / ny, zc, f=f))
    # banners on the side walls, braziers in the corners
    if i > 0:
        top = zf + min(h - 64, 640)
        for y in range(y0 + 256, y1 - 191, 384):
            out.append(sith.banner("x", -w, y, 96, zf + 256, top))
            out.append(sith.banner("x", w, y, 96, zf + 256, top))
    for y in range(y0 + 64, y1 - 63, 384):
        out.append(sconce(-w, y, zf + 256, f))
        out.append(sconce(w, y, zf + 256, f))
    for sx in (-1, 1):
        for y in (y0 + 128, y1 - 128):
            out += brazier(sx * (w - 128), y, zf, f)
    out += EXTRAS[i](r)
    return out


def passage(i):
    """From room i's door out to room i + 1."""
    a = ROOMS[i]
    w, sill, top = door_of(i + 1)
    ya, yb = a["y1"] + T, ROOMS[i + 1]["y0"] - T
    out = [box((-w // 2 - T, ya, sill - T), (w // 2 + T, yb, sill), {"*": TRIM, "top": FLOOR}),
           box((-w // 2 - T, ya, top), (w // 2 + T, yb, top + T), {"*": TRIM, "bottom": CEILING}),
           box((-w // 2 - T, ya, sill), (-w // 2, yb, top), {"*": WALLF}),
           box((w // 2, ya, sill), (w // 2 + T, yb, top), {"*": WALLF})]
    # a quiet walk between fights: runes along both walls, lamps all the way
    for s in (-1, 1):
        x0, x1 = sorted((s * w // 2, s * (w // 2 - 6)))
        out.append(box((x0, ya, sill + 96), (x1, yb, sill + 112), {"*": TRIM, "side": RUNES},
                       detail=True))
    for y in range(ya + 96, yb - 31, 192):
        out.append(lamp(0, y, top, 24))
    return out


def extras_r1(r):
    return []


def extras_r2(r):
    """A tomb: a colonnade, and sarcophagi along the walls."""
    f = style_faces(r["style"])
    out = []
    for x in (-384, 384):
        for k in range(3):
            out += pillar(x, r["y0"] + 192 + k * 192, r["z"], r["h"], f)
    for s in (-1, 1):
        x = s * (r["w"] - 112)
        for k in range(3):
            y = r["y0"] + 192 + k * 192
            out.append(box((x - 44, y - 76, r["z"]), (x + 44, y + 76, r["z"] + 40),
                           {"*": f["wall"], "top": f["trim"]}, detail=True))
            out.append(box((x - 52, y - 84, r["z"] + 40), (x + 52, y + 84, r["z"] + 54),
                           {"*": f["trim"]}, detail=True))
    return out


def extras_r3(r):
    """A forge: a grated catwalk over the lava, machines along the walls."""
    f = style_faces(r["style"])
    c = (r["y0"] + r["y1"]) // 2
    out = [box((-48, c - 224, r["z"] - 16), (48, c + 224, r["z"]), {"*": f["trim"], "top": f["floor"]})]
    for s in (-1, 1):
        x = s * (r["w"] - 80)
        for y in (r["y0"] + 224, r["y1"] - 224):
            out.append(box((x - 48, y - 80, r["z"]), (x + 48, y + 80, r["z"] + 160),
                           {"*": f["wall"], "top": f["trim"]}, detail=True))
            face_x = x - s * 48
            xs = sorted((face_x, face_x - s * 4))
            out.append(box((xs[0], y - 56, r["z"] + 96), (xs[1], y + 56, r["z"] + 112),
                           {"*": f["trim"], "side": f["lamp"]}, detail=True))
    return out


def extras_r4(r):
    """The balcony at the lift's top, with its rail."""
    w, y1, top = r["w"], r["y1"], r["z"] + LIFT_HEIGHT + 8
    front = y1 - BALCONY_D
    out = [box((-w, front, top - 16), (w, y1, top), {"*": TRIM, "top": FLOOR})]
    for a, b in ((-w, -LIFT // 2), (LIFT // 2, w)):
        out.append(box((a, front, top), (b, front + 8, top + 40), {"*": TRIM}))
    # glowing posts at the shaft's corners, floor to balcony, light the ride
    for x in (-LIFT // 2 - 32, LIFT // 2 + 16):
        for y in (front - LIFT - 32, front - 16):
            out.append(box((x, y, r["z"]), (x + 16, y + 16, top - 16), {"*": TRIM, "side": LAMP}))
    return out


def extras_r5(r):
    """An archive: shelves of books along the walls, holocrons glowing on pedestals."""
    f = style_faces(r["style"])
    out = []
    for x in (-384, 384):
        for k in (0, 1):
            out += pillar(x, r["y0"] + 256 + k * 256, r["z"], r["h"], f)
    shelf = Face(COOP + "/archive_shelf", (0.5, 0.5))
    for s in (-1, 1):
        xs = sorted((s * r["w"], s * (r["w"] - 24)))
        for y0, y1 in ((r["y0"] + 32, r["y0"] + 224), (r["y0"] + 416, r["y0"] + 544),
                       (r["y1"] - 224, r["y1"] - 32)):
            out.append(box((xs[0], y0, r["z"]), (xs[1], y1, r["z"] + 224),
                           {"*": f["trim"], "side": shelf}, detail=True))
    for x, y in ((-160, r["y0"] + 192), (160, r["y0"] + 192), (0, r["y0"] + 256)):
        out.append(octagon_brush(x, y, 20, r["z"], r["z"] + 48, {"*": f["trim"]}, True))
        out.append(box((x - 7, y - 7, r["z"] + 56), (x + 7, y + 7, r["z"] + 70), {"*": f["lamp"]},
                       detail=True))
    return out


def extras_r6(r):
    """A hall of guardians: pillars, and two gates of posts and a lintel across it."""
    f = style_faces(r["style"])
    out = []
    for x in (-576, 576):
        for k in range(3):
            out += pillar(x, r["y0"] + 192 + k * 256, r["z"], r["h"], f)
    for y in (r["y0"] + 300, r["y1"] - 96):
        for x in (-320, 320):
            out.append(octagon_brush(x, y, 32, r["z"], r["z"] + 352, {"*": f["pillar"]}, True))
        out.append(box((-360, y - 28, r["z"] + 352), (360, y + 28, r["z"] + 400),
                       {"*": f["trim"], "bottom": f["runes"]}, detail=True))
    return out


def extras_r7(r):
    """The giant statue's pedestal, the boss dais and its steps, pillars."""
    f = style_faces(r["style"])
    zf, y1 = r["z"], r["y1"]
    cy = giant()[1]
    out = [octagon_brush(0, cy, 288, zf, zf + 64, {"*": f["trim"], "top": f["pillar"]})]
    dais = y1 - 448
    out.append(box((-512, dais, zf), (512, y1, zf + 96), {"*": f["trim"], "top": f["floor"]}))
    for k in range(1, 6):
        out.append(box((-512, dais - 32 * k, zf), (512, dais - 32 * (k - 1), zf + 96 - 16 * k),
                       {"*": f["trim"], "top": f["floor"]}))
    for s in (-1, 1):
        for y in range(r["y0"] + 384, y1 - 383, 384):
            out += pillar(s * 560, y, zf, r["h"], f)
    return out


EXTRAS = [extras_r1, extras_r2, extras_r3, extras_r4, extras_r5, extras_r6, extras_r7]


def holes():
    r3, r7 = ROOMS[2], ROOMS[6]
    c3 = (r3["y0"] + r3["y1"]) // 2
    return {
        2: [(-256, -48, c3 - 224, c3 + 224), (48, 256, c3 - 224, c3 + 224)],
        6: [(640, 896, r7["y0"] + 256, r7["y1"] - 256),
            (-896, -640, r7["y0"] + 256, r7["y1"] - 256)],
    }


HOLES = holes()


def giant():
    r = ROOMS[6]
    return 0, (r["y0"] + r["y1"]) // 2 - 64, r["z"] + 64


# ---------------------------------------------------------------- statues

BRIDGE_FIGURES = ["acolyte", "warrior", "caster", "lord"]


def statues():
    """(x, y, z, angle, scale, pedestal apothem or None, figure)."""
    out = [(x, y, 0, a, 2.2, None, BRIDGE_FIGURES[k // 2 % 4])
           for k, (x, y, a) in enumerate(bridge_statues())]
    r1 = ROOMS[0]
    out += [(s * 256, r1["y1"] - 96, 48, 270, 2.6, 80, "acolyte") for s in (-1, 1)]
    r5, r6, r7 = ROOMS[4], ROOMS[5], ROOMS[6]
    for r, inset, scale, figures in ((r5, 128, 3.0, ("caster", "lord")), (r6, 128, 3.2, ("warrior", "lord"))):
        for s in (-1, 1):
            for k, y in enumerate((r["y0"] + r["d"] // 3, r["y0"] + 2 * r["d"] // 3)):
                out.append((s * (r["w"] - inset), y, r["z"] + 48, 0 if s < 0 else 180, scale, 96,
                            figures[k]))
    gx, gy, gz = giant()
    out.append((gx, gy, gz, 270, 7.0, None, "sith_lord"))
    for s in (-1, 1):
        for k, y in enumerate((r7["y0"] + 448, r7["y0"] + 896, r7["y0"] + 1344)):
            out.append((s * 1088, y, r7["z"] + 48, 0 if s < 0 else 180, 3.6, 112,
                        ("warrior", "lord", "caster")[k]))
    return out


def statue_brushes():
    out = []
    for x, y, z, _, scale, pedestal, _ in statues():
        if pedestal:
            out.append(octagon_brush(x, y, pedestal, z - 48, z, {"*": STONE, "side": TRIM}))
        out.append(octagon_brush(x, y, round(20 * scale), z, z + round(104 * scale), {"*": "clip"}))
    return out


# ---------------------------------------------------------------- enemies

# per room: (classname, spawnflags, x, fraction of the depth). A gentle climb:
# plain cultists, then Force users, Reborn, the stronger Reborn and saber
# cultists, Alora; Desann, Tavion and Alora wait at room 7's dais as its second
# wave, which the middle of the hall sets off.
WAVES = [
    [("npc_cultist", 0, -192, 0.6), ("npc_cultist", 0, 192, 0.6), ("npc_cultist", 2, 0, 0.8)],
    [("npc_cultist", 0, -224, 0.7), ("npc_cultist", 4, 224, 0.7), ("npc_reborn_new", 0, -128, 0.5),
     ("npc_reborn_new", 0, 128, 0.5)],
    [("npc_reborn_new", 1, -512, 0.5), ("npc_reborn_new", 2, 512, 0.5), ("npc_cultist", 8, -448, 0.8),
     ("npc_shadowtrooper", 0, 256, 0.85), ("npc_cultist_saber", 0, 0, 0.88)],
    [("npc_reborn_new", 0, -320, 0.35), ("npc_reborn_new", 0, 320, 0.35),
     ("npc_cultist_saber_powers", 1, 0, 0.3)],
    [("npc_reborn_new", 4, -256, 0.5), ("npc_reborn_new", 4, 256, 0.5),
     ("npc_cultist_saber_powers", 10, 0, 0.8), ("npc_cultist", 4, 384, 0.8)],
    [("npc_alora", 0, 0, 0.6), ("npc_reborn_new", 5, 320, 0.75), ("npc_reborn_new", 6, -320, 0.75),
     ("npc_cultist", 4, 448, 0.45), ("npc_cultist", 2, -448, 0.45)],
    [("npc_reborn_new", 4, -384, 0.3), ("npc_reborn_new", 4, 384, 0.3), ("npc_cultist", 4, -1088, 0.15),
     ("npc_cultist", 4, 1088, 0.15)],
]
BOSS_WAVE = [("npc_desann", 0, 0, 0.94), ("npc_tavion_new", 1, -256, 0.92), ("npc_alora", 1, 256, 0.92)]
BALCONY_WAVE = [("npc_cultist", 2, -384)]
CORRIDOR_WAVE = [("npc_shadowtrooper", 0, 0)]
OUTSIDE_WAVE = [("npc_cultist", 0, -160, FACADE_Y - 128), ("npc_cultist", 0, 160, FACADE_Y - 128)]


def floor_z(x, y, r):
    """The floor under (x, y) in room r: its dais or the room's floor."""
    if r is ROOMS[6] and y >= r["y1"] - 448 and abs(x) <= 512:
        return r["z"] + 96
    return r["z"]


def enemies():
    """(classname, spawnflags, origin, angle, targetname or None)."""
    out = [(c, f, (x, y, 40), 270, None) for c, f, x, y in OUTSIDE_WAVE]
    for i, wave in enumerate(WAVES):
        r = ROOMS[i]
        for c, f, x, frac in wave:
            y = round(r["y0"] + frac * r["d"])
            out.append((c, f, (x, y, floor_z(x, y, r) + 40), 270, "room%d" % (i + 1)))
    r7 = ROOMS[6]
    for c, f, x, frac in BOSS_WAVE:
        y = round(r7["y0"] + frac * r7["d"])
        out.append((c, f, (x, y, floor_z(x, y, r7) + 40), 270, "room7"))
    r4 = ROOMS[3]
    for c, f, x in BALCONY_WAVE:
        out.append((c, f, (x, r4["y1"] - 96, r4["z"] + LIFT_HEIGHT + 8 + 40), 270, "room4"))
    corridor_end = ROOMS[4]["y0"] - T - 96
    for c, f, x in CORRIDOR_WAVE:
        out.append((c, f, (x, corridor_end, ROOMS[4]["z"] + 40), 270, "room4"))
    return out


def triggers():
    """(brush, target): each room's enemies wake while the players are still on the
    way, so they are there when it comes into view: room 1's on the bridge's last
    stretch, every other room's at the start of the passage into it."""
    out = [(box((-BRIDGE_W, CHASM_Y1 - 448, 0), (BRIDGE_W, CHASM_Y1 - 384, 128), {"*": TRIGGER}),
            "room1")]
    for i in range(1, len(ROOMS)):
        w, sill, _ = door_of(i)
        ya = ROOMS[i - 1]["y1"] + T
        out.append((box((-w // 2, ya, sill), (w // 2, ya + 64, sill + 128), {"*": TRIGGER}),
                    "room%d" % (i + 1)))
    return out


def hurt_brushes(brushes):
    """A killing trigger_hurt filling each lava brush up to its surface."""
    return [Brush(b.poly, b.z0, b.z1, {"*": TRIGGER}) for b in brushes if b.lava]


def lift():
    """The lift's brushes, drawn at the top: its floor, and a rail with a glowing
    strip on each side, open to the south (on) and the north (off)."""
    r4 = ROOMS[3]
    front = r4["y1"] - BALCONY_D
    top = r4["z"] + LIFT_HEIGHT + 8
    h = LIFT // 2
    out = [box((-h, front - LIFT, top - 16), (h, front, top), {"*": TRIM, "top": FLOOR})]
    for x0, x1 in ((-h, -h + 8), (h - 8, h)):
        out.append(box((x0, front - LIFT, top), (x1, front, top + 40), {"*": TRIM, "top": LAMP}))
    return out


def lift_sound():
    """A trigger over the lift's floor at the bottom and the speaker it plays:
    (trigger brush, speaker origin)."""
    r4 = ROOMS[3]
    front = r4["y1"] - BALCONY_D
    h = LIFT // 2
    floor = r4["z"] + 8
    trigger = box((-h, front - LIFT, floor), (h, front, floor + 64), {"*": TRIGGER})
    return trigger, (0, front - LIFT // 2, r4["z"] + LIFT_HEIGHT // 2)


def ambient_speakers():
    """(sound, origin) of the looping ambient speakers: wind outside, lava below."""
    out = [("wind", (x, -3300, 256)) for x in (-640, 640)]
    out += [("wind", (0, CHASM_Y1 - 160, 256))]
    out += [("lava", (x, y, -320)) for x in (-640, 640) for y in range(CHASM_Y0 + 256, CHASM_Y1, 512)]
    r3, r7 = ROOMS[2], ROOMS[6]
    out.append(("lava", (0, (r3["y0"] + r3["y1"]) // 2, r3["z"] - 24)))
    out += [("lava", (s * 768, y, r7["z"] - 24)) for s in (-1, 1)
            for y in range(r7["y0"] + 384, r7["y1"] - 255, 512)]
    return out


def player_spawns():
    return [(x, y, 24) for x in (96, 224, 352, 480) for y in (START_Y0 + 160, START_Y0 + 352)]


def waypoint_candidates():
    """Points on a 192 grid over every floor, the passages and the bridge."""
    pts = []
    for r in ROOMS:
        for x in range(-r["w"] + 96, r["w"] - 95, 192):
            for y in range(r["y0"] + 96, r["y1"] - 95, 192):
                pts.append((x, y, floor_z(x, y, r) + 32))
    for i in range(len(PASSAGES)):
        _, sill, _ = door_of(i + 1)
        for y in range(ROOMS[i]["y1"] - 32, ROOMS[i + 1]["y0"] + 33, 128):
            pts.append((0, y, sill + 32))
    r4 = ROOMS[3]
    for x in range(-r4["w"] + 96, r4["w"] - 95, 192):
        pts.append((x, r4["y1"] - 128, r4["z"] + LIFT_HEIGHT + 8 + 32))
    for y in range(START_Y0 + 96, FACADE_Y - 95, 128):
        pts.append((0, y, 32))
    for x in range(-OUT_W + 128, OUT_W - 127, 192):
        for y in list(range(START_Y0 + 128, CHASM_Y0 - 63, 192)) + [CHASM_Y1 + 96, FACADE_Y - 96]:
            pts.append((x, y, 32))
    return pts


# ---------------------------------------------------------------- build

def world_brushes():
    out = outdoors()
    for i in range(len(ROOMS)):
        out += room(i)
    for i in range(len(PASSAGES)):
        out += passage(i)
    return out + statue_brushes()


def clear(brushes, p, half=16, below=23, above=40):
    x, y, z = p
    lo, hi = (x - half, y - half, z - below), (x + half, y + half, z + above)
    return not any(b.touches_box(lo, hi) for b in brushes)


def grounded(solid, p, drop=48):
    x, y, z = p
    return any(b.contains((x, y, z - d)) for b in solid for d in range(25, drop, 8))


def build():
    brushes = world_brushes()
    solid = [b for b in brushes if not b.lava]
    plats = lift()
    trig = triggers()
    everything = brushes + plats
    for p in player_spawns():
        if not (clear(everything, p) and grounded(solid, p)):
            sys.exit("player spawn %s is blocked or floating" % (p,))
    for cls, _, p, _, name in enemies():
        if not (clear(everything, p, below=23 + 16) and grounded(solid, p, 72)):
            sys.exit("%s for %s at %s is blocked or floating" % (cls, name, p))
    for x, y, z, *_ in statues():
        if not grounded(solid, (x, y, z + 24)):
            sys.exit("statue at %s stands on nothing" % ((x, y, z),))
    waypoints = [p for p in waypoint_candidates()
                 if clear(everything, p, half=16, below=31) and grounded(solid, p, 64)]

    xwing_name, (x, y, z), angle = XWING
    if not clear(everything, (x, y, z)):
        sys.exit("the X-wing at %s is blocked" % ((x, y, z),))

    world = {"classname": "worldspawn", "message": "SJK Sith Temple (co-op)",
             "_ambient": "16", "gridsize": "64 64 128", "music": MUSIC}
    ents = [entity(world, brushes)]
    ents.append(entity({"classname": "func_plat", "height": str(LIFT_HEIGHT),
                        "speed": str(LIFT_SPEED), "lip": "8"}, plats))
    for b, target in trig:
        ents.append(entity({"classname": "trigger_once", "target": target}, [b]))
    for b in hurt_brushes(brushes):  # lava kills players and NPCs alike, at once
        ents.append(entity({"classname": "trigger_hurt", "dmg": "1000"}, [b]))
    ride, speaker = lift_sound()
    ents.append(entity({"classname": "trigger_multiple", "target": "lift_ride", "wait": "6"}, [ride]))
    ents.append(entity({"classname": "target_speaker", "targetname": "lift_ride",
                        "noise": "%s/lift.wav" % COOP_SOUNDS,
                        "origin": "%d %d %d" % speaker}))
    for sound, origin in ambient_speakers():
        ents.append(entity({"classname": "target_speaker", "spawnflags": "1",
                            "noise": "%s/%s.wav" % (COOP_SOUNDS, sound),
                            "origin": "%d %d %d" % origin}))
    ents.append(entity({"classname": "npc_vehicle", "NPC_type": xwing_name,
                        "origin": "%d %d %d" % (x, y, z), "angle": str(angle)}))
    for part, origin, angles in WRECK_PARTS:
        ents.append(entity({"classname": "misc_model_static", "model": WRECK % part,
                            "origin": "%d %d %d" % origin, "angles": angles}))
    for x, y, z, angle, scale, _, figure in statues():
        ents.append(entity({"classname": "misc_model", "model": "%s_%s.md3" % (sith.STATUE, figure),
                            "origin": "%d %d %d" % (x, y, z), "angle": str(angle),
                            "modelscale": str(scale), "spawnflags": "4"}))
    for cls, flags, (x, y, z), angle, name in enemies():
        fields = {"classname": cls, "origin": "%d %d %d" % (x, y, z), "angle": str(angle)}
        if flags:
            fields["spawnflags"] = str(flags)
        if name:
            fields["targetname"] = name
        ents.append(entity(fields))
    for x, y, z in player_spawns():
        ents.append(entity({"classname": "info_player_deathmatch",
                            "origin": "%d %d %d" % (x, y, z), "angle": "90"}))
    for x, y, z in waypoints:
        ents.append(entity({"classname": "waypoint", "origin": "%d %d %d" % (x, y, z)}))
    return brushes, plats[0], waypoints, "\n".join(ents) + "\n"


# ---------------------------------------------------------------- assets

# SJK's sky_stage.rs (and rd-vanilla's MakeSkyVec): which axis each face's s, t and
# depth run along, in the order rt, lf, bk, ft, up, dn.
SKY_FACES = ["rt", "lf", "bk", "ft", "up", "dn"]
ST_TO_VEC = [[3, -1, 2], [-3, 1, 2], [1, 3, 2], [-1, -3, 2], [-2, -1, 3], [2, -1, -3]]


def sky_box(folder, size=512):
    """A seamless volcanic sky: colour is a function of direction only."""
    rng = np.random.default_rng(21)
    k = 64
    dirs = rng.normal(size=(k, 3))
    dirs /= np.linalg.norm(dirs, axis=1, keepdims=True)
    freq = 2.5 * 1.8 ** rng.integers(0, 5, k)
    phase = rng.random(k) * 2 * np.pi
    c = (np.arange(size) + 0.5) / size
    s, t = np.meshgrid(2 * c - 1, 1 - 2 * c)
    base = [s, t, np.ones_like(s)]
    folder.mkdir(parents=True, exist_ok=True)
    for name, mapping in zip(SKY_FACES, ST_TO_VEC):
        v = np.stack([np.sign(m) * base[abs(m) - 1] for m in mapping], axis=-1)
        v /= np.linalg.norm(v, axis=-1, keepdims=True)
        n = sum(np.sin(freq[j] * (v @ dirs[j]) + phase[j]) / freq[j] for j in range(k))
        n = (n - n.min()) / (n.max() - n.min() + 1e-9) if name != "dn" else n * 0
        elev = v[..., 2]
        glow = np.exp(-np.clip(elev, 0, None) * 5)[..., None]
        sky = (np.array((14, 4, 6)) * (1 - glow) + np.array((150, 40, 16)) * glow)
        clouds = np.clip((n - 0.45) * 2.2, 0, 1)[..., None] * np.clip(elev * 4 + 0.3, 0, 1)[..., None]
        sky = sky * (1 - clouds * 0.7) + np.array((60, 16, 14)) * clouds * 0.7
        sky = np.where(elev[..., None] < 0, np.array((40, 10, 6)), sky)
        rgb(sky).save(folder / ("sky_%s.tga" % name))


def draw_textures(folder):
    folder.mkdir(parents=True, exist_ok=True)
    s = 512
    yy, _ = np.mgrid[0:s, 0:s]
    n = fbm(s, 1.5, 31)
    ground = tint(n, (44, 38, 36), (96, 84, 76)) * (1 - ridges(s, 32, 0.02)[..., None] * 0.5)
    ground += ridges(s, 33, 0.012)[..., None] * np.array((60, 10, 0))
    rgb(ground).save(folder / "ground.tga")
    strata = (0.85 + 0.15 * np.sin(yy / s * 2 * np.pi * 9 + fbm(s, 2.0, 34) * 6))[..., None]
    cliff = tint(fbm(s, 1.3, 35), (40, 34, 34), (98, 86, 80)) * strata
    rgb(cliff * (1 - ridges(s, 36, 0.015)[..., None] * 0.6)).save(folder / "cliff.tga")
    img = Image.new("RGB", (64, 64), (40, 120, 40))
    ImageDraw.Draw(img).text((8, 24), "TRIG", fill=(255, 255, 255))
    img.save(folder / "trigger.tga")
    sky_box(folder / "env")
    for style, palette in PALETTES.items():
        style_textures(folder, style, palette)
    shelf_texture(folder / "archive_shelf.tga")
    field = fbm(256, 1.1, 41)
    lines = (np.sin(np.mgrid[0:256, 0:256][0] / 256 * 2 * np.pi * 24) * 0.5 + 0.5) ** 6
    v = np.clip(field * 0.6 + lines * 0.5, 0, 1)
    rgb(np.stack([v * 255, v * 50, v * 40], axis=-1)).save(folder / "forcefield.tga")


def blocks(s, lo, hi, mortar, veins, strength, seed, bw=128, bh=64):
    yy, xx = np.mgrid[0:s, 0:s]
    out = tint(fbm(s, 1.4, seed), lo, hi)
    rnd = np.random.default_rng(seed)
    for by in range(0, s, bh):
        off = (by // bh % 2) * bw // 2
        for bx in range(-bw, s, bw):
            m = (yy >= by) & (yy < by + bh) & (((xx - bx - off) % s) < bw)
            out[m] *= rnd.uniform(0.85, 1.12)
    joint = ((yy % bh) < 3) | (((xx - (yy // bh % 2) * bw // 2) % bw) < 3)
    out[joint] = mortar
    v = ridges(s, seed + 1, 0.016)[..., None] * strength
    return out * (1 - v) + np.array(veins) * v


def plates(s, lo, hi, seam, rust, strength, seed):
    yy, xx = np.mgrid[0:s, 0:s]
    out = tint(fbm(s, 1.2, seed), lo, hi)
    streak = np.clip(fbm(s, 1.8, seed + 1) * 1.6 - 0.7, 0, 1) * np.clip(1.2 - (yy % 128) / 128, 0, 1)
    out = out * (1 - streak[..., None] * strength) + np.array(rust) * streak[..., None] * strength
    out[((yy % 128) < 3) | ((xx % 128) < 3)] = seam
    for cy in range(10, s, 128):
        for cx in range(10, s, 128):
            for dy, dx in ((0, 0), (0, 108), (108, 0), (108, 108)):
                y, x = cy + dy, cx + dx
                out[y - 3:y + 3, x - 3:x + 3] = np.array(hi) * 1.2
    return out


def marble(s, lo, hi, veins, strength, seed):
    out = tint(fbm(s, 1.9, seed), lo, hi)
    v = (ridges(s, seed + 1, 0.02, 1.4) + 0.5 * ridges(s, seed + 2, 0.01, 1.8))[..., None] * strength
    return out * (1 - np.clip(v, 0, 1)) + np.array(veins) * np.clip(v, 0, 1)


def floor_texture(s, kind, lo, hi, inlay, seed):
    yy, xx = np.mgrid[0:s, 0:s]
    if kind == "checker":
        out = np.where((((yy // 256) + (xx // 256)) % 2 == 0)[..., None],
                       tint(fbm(s, 1.9, seed), lo, np.array(lo) * 1.6),
                       tint(fbm(s, 1.9, seed + 1), hi, np.array(hi) * 1.5))
    elif kind == "grate":
        out = tint(fbm(s, 1.2, seed), lo, hi)
        holes = ((yy % 32) > 6) & ((yy % 32) < 26) & ((xx % 32) > 6) & ((xx % 32) < 26)
        out[holes] *= 0.25
        out[holes] += np.array(inlay) * 0.18
    else:  # slabs
        out = tint(fbm(s, 1.8, seed), lo, hi)
        out += ridges(s, seed + 2, 0.02)[..., None] * 25
    joint = ((yy % 256) < 4) | ((xx % 256) < 4)
    line = ((yy % 256 >= 1) & (yy % 256 < 3)) | ((xx % 256 >= 1) & (xx % 256 < 3))
    out[joint] = np.array(lo) * 0.4
    out[line] = inlay
    return out


def style_textures(folder, style, p):
    """A style's wall, floor, trim, ceiling, pillar, runes (and their glow) and lamp."""
    seed = sum(map(ord, style))
    s = 512
    kind, lo, hi, mortar, veins, strength = p["wall"]
    wall = {"blocks": lambda: blocks(s, lo, hi, mortar, veins, strength, seed),
            "plates": lambda: plates(s, lo, hi, mortar, veins, strength, seed),
            "marble": lambda: marble(s, lo, hi, veins, strength, seed)}[kind]()
    rgb(wall).save(folder / ("%s_wall.tga" % style))
    rgb(floor_texture(s, *p["floor"], seed + 5)).save(folder / ("%s_floor.tga" % style))
    trim = rgb(tint(fbm(256, 1.2, seed + 7), *p["trim"]))
    d = ImageDraw.Draw(trim)
    d.rectangle([0, 0, 255, 255], outline=tuple(min(255, int(c * 1.3)) for c in p["trim"][1]), width=4)
    d.rectangle([6, 6, 249, 249], outline=tuple(int(c * 0.4) for c in p["trim"][0]), width=2)
    trim.save(folder / ("%s_trim.tga" % style))
    rgb(tint(fbm(s, 1.3, seed + 9), *p["ceiling"])).save(folder / ("%s_ceiling.tga" % style))
    rgb(marble(s, *p["pillar"], tuple(min(255, int(c * 1.4)) for c in p["pillar"][1]), 0.25, seed + 11)
        ).save(folder / ("%s_pillar.tga" % style))
    glyphs = sith_glyphs(512, 64, seed + 13)
    base = tint(fbm(512, 1.2, seed + 15)[:64], np.array(p["trim"][0]) * 0.5, p["trim"][0])
    color = np.array(p["runes"])
    rgb(base * (1 - glyphs[..., None]) + color * 0.55 * glyphs[..., None]).save(
        folder / ("%s_runes.tga" % style))
    rgb(color * glyphs[..., None]).save(folder / ("%s_runes_glow.tga" % style))
    lamp = Image.new("RGB", (64, 64), p["lamp"])
    ImageDraw.Draw(lamp).rectangle([0, 0, 63, 63], outline=(20, 18, 20), width=6)
    lamp.save(folder / ("%s_lamp.tga" % style))


def sith_glyphs(w, h, seed):
    """A strip of angular glyphs, 1 on a stroke."""
    rnd = np.random.default_rng(seed)
    mask = Image.new("L", (w, h), 0)
    d = ImageDraw.Draw(mask)
    for cx in range(16, w, 32):
        pts = [(cx + int(rnd.choice((-8, 0, 8))), 12 + int(rnd.choice((0, 20, 40)))) for _ in range(4)]
        d.line(pts, fill=255, width=3)
        if rnd.random() < 0.5:
            d.line([(cx - 8, 32), (cx + 8, 32)], fill=255, width=3)
    d.line([(0, 3), (w, 3)], fill=160, width=2)
    d.line([(0, h - 4), (w, h - 4)], fill=160, width=2)
    return np.asarray(mask, dtype=float) / 255


def shelf_texture(path):
    """Shelves of books: rows of spines in old leather colours."""
    rnd = np.random.default_rng(51)
    img = Image.new("RGB", (256, 256), (26, 18, 14))
    d = ImageDraw.Draw(img)
    for row in range(4):
        y0 = row * 64 + 6
        d.rectangle([0, y0 + 52, 255, y0 + 57], fill=(60, 44, 30))  # the shelf board
        x = 2
        while x < 250:
            w = int(rnd.integers(5, 13))
            h = int(rnd.integers(34, 52))
            c = tuple(int(v) for v in rnd.choice([(90, 20, 24), (40, 30, 70), (30, 60, 40),
                                                  (110, 80, 40), (60, 50, 45), (20, 20, 24)]))
            d.rectangle([x, y0 + 52 - h, x + w - 1, y0 + 51], fill=c)
            d.line([(x, y0 + 52 - h + 6), (x + w - 1, y0 + 52 - h + 6)], fill=(170, 140, 70))
            x += w + 1
    img.save(path)


def shader_text():
    t = "textures/" + COOP
    out = []
    for name in ("ground", "cliff"):
        out.append("%(t)s/%(n)s\n{\n\tqer_editorimage %(t)s/%(n)s.tga\n"
                   "\t{\n\t\tmap $lightmap\n\t\trgbGen identity\n\t}\n"
                   "\t{\n\t\tmap %(t)s/%(n)s.tga\n\t\tblendFunc GL_DST_COLOR GL_ZERO\n"
                   "\t\trgbGen identity\n\t}\n}\n" % {"t": t, "n": name})
    for style in PALETTES:
        for part in ("wall", "floor", "trim", "ceiling", "pillar"):
            out.append("%(t)s/%(s)s_%(n)s\n{\n\tqer_editorimage %(t)s/%(s)s_%(n)s.tga\n"
                       "\t{\n\t\tmap $lightmap\n\t\trgbGen identity\n\t}\n"
                       "\t{\n\t\tmap %(t)s/%(s)s_%(n)s.tga\n\t\tblendFunc GL_DST_COLOR GL_ZERO\n"
                       "\t\trgbGen identity\n\t}\n}\n" % {"t": t, "s": style, "n": part})
        for part, glow, power in (("runes", "runes_glow", 900), ("lamp", "lamp", 4000)):
            out.append("%(t)s/%(s)s_%(n)s\n{\n\tqer_editorimage %(t)s/%(s)s_%(n)s.tga\n"
                       "\tq3map_surfacelight %(p)d\n\tq3map_lightimage %(t)s/%(s)s_%(g)s.tga\n"
                       "\tsurfaceparm nomarks\n"
                       "\t{\n\t\tmap $lightmap\n\t\trgbGen identity\n\t}\n"
                       "\t{\n\t\tmap %(t)s/%(s)s_%(n)s.tga\n\t\tblendFunc GL_DST_COLOR GL_ZERO\n"
                       "\t\trgbGen identity\n\t}\n"
                       "\t{\n\t\tmap %(t)s/%(s)s_%(g)s.tga\n\t\tblendFunc GL_ONE GL_ONE\n\t\tglow\n\t}\n}\n"
                       % {"t": t, "s": style, "n": part, "g": glow, "p": power})
    out.append("%(t)s/archive_shelf\n{\n\tqer_editorimage %(t)s/archive_shelf.tga\n"
               "\t{\n\t\tmap $lightmap\n\t\trgbGen identity\n\t}\n"
               "\t{\n\t\tmap %(t)s/archive_shelf.tga\n\t\tblendFunc GL_DST_COLOR GL_ZERO\n"
               "\t\trgbGen identity\n\t}\n}\n" % {"t": t})
    out.append("""%(t)s/forcefield
{
\tqer_editorimage %(t)s/forcefield.tga
\tq3map_surfacelight 150
\tsurfaceparm trans
\tsurfaceparm nolightmap
\tsurfaceparm nomarks
\tcull none
\t{
\t\tmap %(t)s/forcefield.tga
\t\tblendFunc GL_ONE GL_ONE
\t\trgbGen wave sin 0.5 0.25 0 0.7
\t\ttcMod scroll 0 0.25
\t\tglow
\t}
\t{
\t\tmap %(t)s/forcefield.tga
\t\tblendFunc GL_ONE GL_ONE
\t\ttcMod scale 0.5 0.5
\t\ttcMod scroll 0.07 -0.11
\t}
}
""" % {"t": t})
    out.append("""%(t)s/sky
{
\tqer_editorimage %(t)s/env/sky_ft.tga
\tsurfaceparm sky
\tsurfaceparm noimpact
\tsurfaceparm nolightmap
\tsurfaceparm nomarks
\tq3map_sun 1 0.45 0.3 90 220 35
\tq3map_surfacelight 40
\tskyParms %(t)s/env/sky - -
}
""" % {"t": t})
    out.append("""%(t)s/trigger
{
\tqer_editorimage %(t)s/trigger.tga
\tqer_trans 0.4
\tsurfaceparm nodraw
\tsurfaceparm nolightmap
\tsurfaceparm nonsolid
\tsurfaceparm trans
\tsurfaceparm nomarks
}
""" % {"t": t})
    return "\n".join(out)


def write_assets(stage):
    sith.write_assets(stage)  # the temple's textures, statue and sjk_sith.shader
    sounds = stage / COOP_SOUNDS
    sith_audio.lift(sounds / "lift.wav", LIFT_HEIGHT / LIFT_SPEED)
    sith_audio.lava_loop(sounds / "lava.wav")
    sith_audio.wind_loop(sounds / "wind.wav")
    (stage / MUSIC).parent.mkdir(parents=True, exist_ok=True)
    sith_audio.music(stage / MUSIC)
    draw_textures(stage / "textures" / COOP)
    shaders = stage / "shaders"
    (shaders / (COOP + ".shader")).write_text(shader_text(), newline="\n")
    (shaders / "shaderlist.txt").write_text("%s\n%s\n" % (sith.PREFIX, COOP), newline="\n")


# ---------------------------------------------------------------- preview

def preview(brushes, plat, waypoints, path):
    xs = [x for b in brushes for x, _ in b.poly]
    ys = [y for b in brushes for _, y in b.poly]
    x0, x1, y0, y1 = min(xs), max(xs), min(ys), max(ys)
    scale = 0.12
    W, H = int((x1 - x0) * scale) + 1, int((y1 - y0) * scale) + 1

    def px(x, y):
        return ((x - x0) * scale, (y1 - y) * scale)

    img = Image.new("RGB", (W, H), (14, 12, 16))
    d = ImageDraw.Draw(img)
    for b in sorted(brushes, key=lambda b: b.z1):
        name = sith.as_face(b.faces.get("top", b.faces.get("*"))).tex
        if name in ("clip", TRIGGER) or COOP + "/sky" == name or "bottom" in b.faces:
            continue  # tools, the sky, ceilings and lamps
        if b.lava:
            color = (230, 90, 20)
        else:
            t = max(0.0, min(1.0, (b.z1 + 64) / 1100))
            color = (int(40 + 120 * t), int(40 + 100 * t), int(60 + 90 * t))
        d.polygon([px(*v) for v in b.poly], fill=color)
    d.polygon([px(*v) for v in plat.poly], fill=(80, 160, 230))
    for x, y, _ in waypoints:
        cx, cy = px(x, y)
        d.point((cx, cy), fill=(120, 120, 255))
    for x, y, *_ in statues():
        cx, cy = px(x, y)
        d.ellipse([cx - 4, cy - 4, cx + 4, cy + 4], fill=(200, 195, 205))
    for _, _, (x, y, _), _, _ in enemies():
        cx, cy = px(x, y)
        d.ellipse([cx - 3, cy - 3, cx + 3, cy + 3], fill=(240, 40, 40))
    for x, y, _ in player_spawns():
        cx, cy = px(x, y)
        d.ellipse([cx - 3, cy - 3, cx + 3, cy + 3], fill=(60, 220, 90))
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

    brushes, plat, waypoints, text = build()
    mapfile = stage / "maps" / (MAP + ".map")
    mapfile.write_text(text, newline="\n")
    write_assets(stage)
    print("wrote %s: %d brushes, %d statues, %d enemies, %d waypoints"
          % (mapfile, len(brushes), len(statues()), len(enemies()), len(waypoints)))
    if args.preview:
        preview(brushes, plat, waypoints, args.preview)
        print("wrote", args.preview)
    if args.no_compile:
        return
    print("compiling", MAP, flush=True)
    compile_map(args.q3map2, gamedata, home, mapfile)
    pk3 = out / (MAP + ".pk3")
    with zipfile.ZipFile(pk3, "w", zipfile.ZIP_DEFLATED) as z:
        z.write(mapfile.with_suffix(".bsp"), "maps/%s.bsp" % MAP)
        for prefix in (sith.PREFIX, COOP):
            z.write(stage / "shaders" / (prefix + ".shader"), "shaders/%s.shader" % prefix)
        for tga in sorted((stage / "textures").rglob("*.tga")):
            z.write(tga, tga.relative_to(stage).as_posix())
        for md3 in sorted((stage / sith.STATUE).parent.glob("*.md3")):
            z.write(md3, md3.relative_to(stage).as_posix())
        for f in sorted((stage / "sound").rglob("*.wav")) + [stage / MUSIC]:
            z.write(f, f.relative_to(stage).as_posix())
    print("wrote", pk3)
    if args.install:
        shutil.copy2(pk3, gamedata / "base" / pk3.name)
        print("installed", gamedata / "base" / pk3.name)


if __name__ == "__main__":
    main()
