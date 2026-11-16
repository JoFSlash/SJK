"""Build SJK Symbols, SJK's own symbol font (docs/sjk-ui.md "Symbols").

    python scripts/sjk_symbols.py            write crates/sjk-viewer/assets/fonts/SJKSymbols.ttf
    python scripts/sjk_symbols.py --check    fail when the committed font is out of date

The symbols mark qualities in SJK's text: the rarities (Common to Unique), the
cards' finishes, the items' icons (what a thing is and where it comes from) and the
kyber shard. They sit in Unicode's Private Use Area from U+E000, one code point
each for good: a symbol is never moved or reused, new ones take the next free point of
their group. The client draws them as it draws any character its fonts lack, from this
font through the wide-glyph atlas (crates/sjk-viewer/src/text/wide.rs), in the colour of
the text around them; crates/sjk-viewer/src/symbols.rs names them.

Every symbol is drawn here from strokes: lines with round ends, polygon rings and filled
diamonds, in font units (1000 to the em, the capitals 700 high). The strokes are turned
into outlines and merged (skia-pathops), so each glyph is clean, clockwise TrueType.
The build is deterministic (fixed timestamps), so --check compares bytes.

Needs fontTools and skia-pathops (pip install fonttools skia-pathops).
"""

import io
import math
import sys
from pathlib import Path

import pathops
from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "crates" / "sjk-viewer" / "assets" / "fonts" / "SJKSymbols.ttf"

EM = 1000
CAP = 700
# A symbol stands on a square of the capitals' height and a little more, centred on
# their middle, with this much room on each side.
BOX = 780
MIDDLE = CAP / 2
SIDE = 70
ADVANCE = BOX + SIDE * 2
# Stroke width, chosen for 16 to 20 pixel text: about two pixels there.
STROKE = 92
# Points on a half circle of a round line end.
ROUND = 10


def capsule(a, b, width=STROKE):
    """A line from `a` to `b` with round ends, as a polygon."""
    (ax, ay), (bx, by) = a, b
    angle = math.atan2(by - ay, bx - ax)
    r = width / 2
    points = []
    for end, (cx, cy), start in ((0, (bx, by), angle - math.pi / 2), (1, (ax, ay), angle + math.pi / 2)):
        for i in range(ROUND + 1):
            t = start + math.pi * i / ROUND
            points.append((cx + r * math.cos(t), cy + r * math.sin(t)))
    return [points]


def ring(points, width=STROKE):
    """A closed polygon's outline `width` thick: its outer and inner contours."""
    n = len(points)
    cx = sum(p[0] for p in points) / n
    cy = sum(p[1] for p in points) / n

    def offset(by):
        out = []
        for i in range(n):
            prev, here, nxt = points[i - 1], points[i], points[(i + 1) % n]
            # The corner moves along its bisector so both edges move by `by`.
            def normal(a, b):
                dx, dy = b[0] - a[0], b[1] - a[1]
                length = math.hypot(dx, dy)
                return (dy / length, -dx / length)
            n1, n2 = normal(prev, here), normal(here, nxt)
            bx, by_ = n1[0] + n2[0], n1[1] + n2[1]
            length = math.hypot(bx, by_)
            cos_half = (n1[0] * n2[0] + n1[1] * n2[1] + 1) / 2
            scale = by / math.sqrt(max(cos_half, 1e-6))
            # Outward is away from the centre.
            sign = 1 if (here[0] - cx) * bx + (here[1] - cy) * by_ > 0 else -1
            out.append((here[0] + sign * bx / length * scale, here[1] + sign * by_ / length * scale))
        return out

    outer = offset(width / 2)
    inner = offset(-width / 2)
    return [outer, list(reversed(inner))]


def polygon(points):
    return [points]


def octagon(r, cx=0.0, cy=MIDDLE):
    """A regular octagon with flat sides up and down, vertex radius `r`."""
    return [
        (cx + r * math.cos(math.pi / 8 + i * math.pi / 4), cy + r * math.sin(math.pi / 8 + i * math.pi / 4))
        for i in range(8)
    ]


def diamond(r, cx=0.0, cy=MIDDLE):
    return [[(cx, cy + r), (cx + r, cy), (cx, cy - r), (cx - r, cy)]]


def four_point_star(tip, waist, cx=0.0, cy=MIDDLE):
    """A solid four-point star, its points up, down and to the sides: `tip` from its
    centre to a point, `waist` to the corners between them."""
    points = []
    for i in range(8):
        r = tip if i % 2 == 0 else waist
        a = math.pi / 2 + i * math.pi / 4
        points.append((cx + r * math.cos(a), cy + r * math.sin(a)))
    return [points[::-1]]


def rect_round(x, y, w, h, r):
    """A rounded rectangle's points (corners as quarter circles)."""
    points = []
    corners = [(x + w - r, y + r, -math.pi / 2), (x + w - r, y + h - r, 0.0), (x + r, y + h - r, math.pi / 2), (x + r, y + r, math.pi)]
    for cx, cy, start in corners:
        for i in range(ROUND // 2 + 1):
            t = start + (math.pi / 2) * i / (ROUND // 2)
            points.append((cx + r * math.cos(t), cy + r * math.sin(t)))
    return points


# The holocron rarity marks: one octagon for every rank, the marks inside grow.
R = BOX / 2 - STROKE / 2 - 10
INNER = R * 0.56


def rarity(rank):
    shapes = [ring(octagon(R))]
    if rank == 1:
        shapes.append(diamond(R * 0.24))
    if rank == 2:
        shapes.append(capsule((0, MIDDLE - INNER), (0, MIDDLE + INNER)))
    if rank >= 3:
        gap = R * 0.30
        for sx, sy in ((0, -1), (0, 1), (-1, 0), (1, 0)):
            shapes.append(capsule((sx * gap, MIDDLE + sy * gap), (sx * INNER, MIDDLE + sy * INNER)))
        shapes.append(diamond(R * 0.17))
    if rank == 4:
        # A second frame round the first: the outer one thinner, the first drawn
        # smaller to make room.
        shapes = [ring(octagon(R * 0.78), STROKE * 0.85)]
        inner = R * 0.78 * 0.56
        gap = R * 0.78 * 0.30
        for sx, sy in ((0, -1), (0, 1), (-1, 0), (1, 0)):
            shapes.append(capsule((sx * gap, MIDDLE + sy * gap), (sx * inner, MIDDLE + sy * inner), STROKE * 0.85))
        shapes.append(diamond(R * 0.78 * 0.17))
        shapes.append(ring(octagon(R * 1.08), STROKE * 0.7))
    if rank == 5:
        # Unique (25/10/2026): Mythical's two frames round a solid four-point star in
        # place of its cross, so the two read apart at text sizes (a dot against a cross).
        shapes = [
            ring(octagon(R * 0.78), STROKE * 0.85),
            four_point_star(R * 0.78 * 0.60, R * 0.78 * 0.23),
            ring(octagon(R * 1.08), STROKE * 0.7),
        ]
    return shapes


# The cards' finishes: a card's outline; Holo with slanting bands across it; Solar
# with a sun at its heart.
CARD_W, CARD_H = BOX * 0.62, BOX * 0.9


def card_outline():
    x, y = -CARD_W / 2, MIDDLE - CARD_H / 2
    return ring(rect_round(x, y, CARD_W, CARD_H, CARD_W * 0.16)[::-1])


def finish(index):
    shapes = [card_outline()]
    if index == 1:
        for k in (-1, 1):
            off = k * CARD_W * 0.2
            shapes.append(capsule((off - CARD_W * 0.18, MIDDLE - CARD_H * 0.22), (off + CARD_W * 0.18, MIDDLE + CARD_H * 0.22), STROKE * 0.8))
    if index == 2:
        sun = CARD_W * 0.16
        circle = [(sun * math.cos(t * math.pi / 8), MIDDLE + sun * math.sin(t * math.pi / 8)) for t in range(16)]
        shapes.append(polygon(circle))
        for k in range(8):
            a = k * math.pi / 4
            shapes.append(capsule(
                (sun * 1.55 * math.cos(a), MIDDLE + sun * 1.55 * math.sin(a)),
                (sun * 2.15 * math.cos(a), MIDDLE + sun * 2.15 * math.sin(a)),
                STROKE * 0.6,
            ))
    return shapes


# The items' icons (17/10/2026, the Shaders tab's cards): what a thing is, where it comes
# from and what the player does with it, drawn as line pictures in the rarity marks'
# stroke so they sit beside them in a line of text.
ICON = BOX / 2 - STROKE / 2 - 20


def arc(cx, cy, r, start, end, steps=14, width=STROKE):
    """A round-ended line along a circle's arc, from angle `start` to `end` (radians)."""
    points = [(cx + r * math.cos(start + (end - start) * i / steps), cy + r * math.sin(start + (end - start) * i / steps)) for i in range(steps + 1)]
    return [capsule(a, b, width) for a, b in zip(points, points[1:])]


def circle(cx, cy, r, steps=24):
    return [(cx + r * math.cos(2 * math.pi * i / steps), cy + r * math.sin(2 * math.pi * i / steps)) for i in range(steps)]


def holocron_icon():
    """A cube seen from above a corner: a hexagon, its near corner's three edges."""
    r = ICON
    corners = [(r * math.cos(math.radians(a)), MIDDLE + r * math.sin(math.radians(a))) for a in (90, 30, -30, -90, -150, 150)]
    shapes = [ring(corners)]
    for a in (30, 150, -90):
        shapes.append(capsule((0, MIDDLE), (r * math.cos(math.radians(a)), MIDDLE + r * math.sin(math.radians(a)))))
    return shapes


def medal_icon():
    """A medallion hanging from a ribbon's V."""
    r = ICON * 0.5
    cy = MIDDLE - ICON + r
    shapes = [ring(circle(0, cy, r))]
    top = MIDDLE + ICON
    shapes.append(capsule((-ICON * 0.5, top), (-r * 0.35, cy + r)))
    shapes.append(capsule((ICON * 0.5, top), (r * 0.35, cy + r)))
    return shapes


def lock_icon():
    """A padlock: a solid body, its shackle an arch over it."""
    w, h = ICON * 1.25, ICON * 1.0
    x, y = -w / 2, MIDDLE - ICON
    shapes = [polygon(rect_round(x, y, w, h, w * 0.14))]
    r = w * 0.3
    leg = y + h + ICON * 0.18
    shapes.append(capsule((-r, y + h - 10), (-r, leg)))
    shapes.append(capsule((r, y + h - 10), (r, leg)))
    shapes.extend(arc(0, leg, r, math.pi, 0.0))
    return shapes


def tick_icon():
    """A tick: the thing is yours."""
    width = STROKE * 1.25
    low = (-ICON * 0.25, MIDDLE - ICON * 0.55)
    return [capsule((-ICON * 0.8, MIDDLE), low, width), capsule(low, (ICON * 0.85, MIDDLE + ICON * 0.75), width)]


def stack_icon():
    """Two copies: a card in front, the edge of a second behind it."""
    w, h = ICON * 1.15, ICON * 1.5
    x, y = -ICON, MIDDLE - ICON
    shapes = [ring(rect_round(x, y, w, h, w * 0.16)[::-1])]
    off = ICON * 0.42
    shapes.append(capsule((x + off, y + h + off * 0.75), (x + w + off, y + h + off * 0.75)))
    shapes.append(capsule((x + w + off, y + h + off * 0.75), (x + w + off, y + off)))
    return shapes


def hammer_icon():
    """A smith's hammer: a handle slanting up to the right, its head across it."""
    shapes = [capsule((-ICON * 0.8, MIDDLE - ICON * 0.8), (ICON * 0.3, MIDDLE + ICON * 0.3), STROKE * 1.1)]
    c = (ICON * 0.35, MIDDLE + ICON * 0.35)
    u = (-math.sqrt(0.5), math.sqrt(0.5))
    v = (math.sqrt(0.5), math.sqrt(0.5))
    a, b = ICON * 0.68, ICON * 0.26
    head = [(c[0] + su * u[0] * a + sv * v[0] * b, c[1] + su * u[1] * a + sv * v[1] * b) for su, sv in ((1, 1), (-1, 1), (-1, -1), (1, -1))]
    shapes.append(polygon(head))
    return shapes


def turn_icon():
    """A turning arrow: most of a circle, its head at the end."""
    r = ICON * 0.82
    start, end = math.radians(60), math.radians(330)
    shapes = arc(0, MIDDLE, r, start, end, 18)
    tip = (r * math.cos(end), MIDDLE + r * math.sin(end))
    # The head points on round the circle (clockwise in the font's upward y).
    along = (math.sin(end), -math.cos(end))
    out = (math.cos(end), math.sin(end))
    size = ICON * 0.62
    shapes.append(polygon([
        (tip[0] - along[0] * size * 0.2 + out[0] * size * 0.75, tip[1] - along[1] * size * 0.2 + out[1] * size * 0.75),
        (tip[0] + along[0] * size, tip[1] + along[1] * size),
        (tip[0] - along[0] * size * 0.2 - out[0] * size * 0.75, tip[1] - along[1] * size * 0.2 - out[1] * size * 0.75),
    ]))
    return shapes


def saber_icon():
    """A lightsaber held up to the right: a thick hilt, a guard and a long blade."""
    d = math.sqrt(0.5)
    base = (-ICON * 0.85, MIDDLE - ICON * 0.85)
    guard = (base[0] + ICON * 0.55, base[1] + ICON * 0.55)
    shapes = [capsule(base, guard, STROKE * 1.7)]
    shapes.append(capsule((guard[0] - d * ICON * 0.24, guard[1] + d * ICON * 0.24), (guard[0] + d * ICON * 0.24, guard[1] - d * ICON * 0.24), STROKE * 0.9))
    shapes.append(capsule((guard[0] + ICON * 0.12, guard[1] + ICON * 0.12), (ICON * 0.9, MIDDLE + ICON * 0.9), STROKE * 1.05))
    return shapes


def gift_icon():
    """A gift: a box, its lid, the ribbon down the middle and a bow on top."""
    w = ICON * 1.5
    lid_y = MIDDLE + ICON * 0.15
    shapes = [ring(rect_round(-w / 2 + 30, MIDDLE - ICON, w - 60, lid_y - (MIDDLE - ICON), 40)[::-1], STROKE * 0.9)]
    shapes.append(ring(rect_round(-w / 2, lid_y, w, ICON * 0.38, 30)[::-1], STROKE * 0.9))
    shapes.append(capsule((0, MIDDLE - ICON), (0, lid_y + ICON * 0.38), STROKE * 0.9))
    top = lid_y + ICON * 0.38
    shapes.append(capsule((0, top), (-ICON * 0.45, top + ICON * 0.45), STROKE * 0.9))
    shapes.append(capsule((0, top), (ICON * 0.45, top + ICON * 0.45), STROKE * 0.9))
    return shapes


def star_icon():
    """A five-pointed star, solid: an edition's seal."""
    points = []
    for i in range(10):
        r = ICON if i % 2 == 0 else ICON * 0.42
        a = math.pi / 2 + i * math.pi / 5
        points.append((r * math.cos(a), MIDDLE - ICON * 0.05 + r * math.sin(a)))
    return [polygon(points[::-1])]
# The kyber shard (17/10/2026): a long crystal with a point at each end and a ridge down
# its middle. The credits' sign, Lucasfilm's, stays out of this font (the client draws it
# from shapes, crates/sjk-viewer/src/cards/mark.rs).
def kyber_shard():
    half, top, shoulder = BOX * 0.2, BOX / 2 - 10, BOX * 0.2
    crystal = [
        (0, MIDDLE + top), (half, MIDDLE + shoulder), (half, MIDDLE - shoulder),
        (0, MIDDLE - top), (-half, MIDDLE - shoulder), (-half, MIDDLE + shoulder),
    ]
    return [ring(crystal[::-1]), capsule((0, MIDDLE + top * 0.62), (0, MIDDLE - top * 0.62), STROKE * 0.8)]


# Every symbol: (code point, glyph name, shapes). Code points never move.
SYMBOLS = [
    (0xE000, "rarityCommon", rarity(0)),
    (0xE001, "rarityUncommon", rarity(1)),
    (0xE002, "rarityRare", rarity(2)),
    (0xE003, "rarityLegendary", rarity(3)),
    (0xE004, "rarityMythical", rarity(4)),
    (0xE005, "rarityUnique", rarity(5)),
    (0xE010, "finishNormal", finish(0)),
    (0xE011, "finishHolo", finish(1)),
    (0xE012, "finishSolar", finish(2)),
    (0xE020, "iconHolocron", holocron_icon()),
    (0xE021, "iconMedal", medal_icon()),
    (0xE022, "iconLock", lock_icon()),
    (0xE023, "iconTick", tick_icon()),
    (0xE024, "iconStack", stack_icon()),
    (0xE025, "iconSmith", hammer_icon()),
    (0xE026, "iconTurn", turn_icon()),
    (0xE027, "iconSaber", saber_icon()),
    (0xE028, "iconGift", gift_icon()),
    (0xE029, "iconEdition", star_icon()),
    (0xE030, "kyberShard", kyber_shard()),
]


def glyph(shapes):
    """The shapes merged into one clockwise outline, centred on the advance."""
    paths = []
    for shape in shapes:
        path = pathops.Path()
        for contour in shape:
            first, *rest = contour
            path.moveTo(first[0] + ADVANCE / 2, first[1])
            for x, y in rest:
                path.lineTo(x + ADVANCE / 2, y)
            path.close()
        paths.append(path)
    merged = paths[0]
    for path in paths[1:]:
        merged = pathops.op(merged, path, pathops.PathOp.UNION, fix_winding=True, clockwise=True)
    if len(paths) == 1:
        merged = pathops.op(merged, pathops.Path(), pathops.PathOp.UNION, fix_winding=True, clockwise=True)
    pen = TTGlyphPen(None)
    rounded = pathops.Path()
    for verb, points in merged:
        if verb == pathops.PathVerb.MOVE:
            rounded.moveTo(round(points[0][0]), round(points[0][1]))
        elif verb == pathops.PathVerb.LINE:
            rounded.lineTo(round(points[0][0]), round(points[0][1]))
        elif verb == pathops.PathVerb.CLOSE:
            rounded.close()
        else:
            raise ValueError(f"unexpected curve in a symbol: {verb}")
    rounded.draw(pen)
    return pen.glyph()


def build():
    names = [".notdef", "space"] + [name for _, name, _ in SYMBOLS]
    builder = FontBuilder(EM, isTTF=True)
    builder.setupGlyphOrder(names)
    builder.setupCharacterMap({0x20: "space", **{code: name for code, name, _ in SYMBOLS}})
    empty = TTGlyphPen(None).glyph()
    notdef = TTGlyphPen(None)
    for contour in ring([(100, 0), (100, CAP), (500, CAP), (500, 0)], 60):
        first, *rest = contour
        notdef.moveTo(tuple(round(v) for v in first))
        for point in rest:
            notdef.lineTo(tuple(round(v) for v in point))
        notdef.closePath()
    glyphs = {".notdef": notdef.glyph(), "space": empty}
    metrics = {".notdef": (600, 70), "space": (300, 0)}
    for _, name, shapes in SYMBOLS:
        glyphs[name] = glyph(shapes)
    builder.setupGlyf(glyphs)
    glyf = builder.font["glyf"]
    for _, name, _ in SYMBOLS:
        g = glyf[name]
        g.recalcBounds(glyf)
        metrics[name] = (ADVANCE, g.xMin)
    builder.setupHorizontalMetrics(metrics)
    builder.setupHorizontalHeader(ascent=800, descent=-200)
    builder.setupNameTable({
        "familyName": "SJK Symbols",
        "styleName": "Regular",
        "uniqueFontIdentifier": "SJK Symbols Regular",
        "fullName": "SJK Symbols",
        "psName": "SJKSymbols-Regular",
        "version": "Version 1.000",
        "copyright": "SJK's own symbols, distributed with SJK (crates/sjk-viewer/assets/fonts/SJK-fonts.txt)",
    })
    builder.setupOS2(sTypoAscender=800, sTypoDescender=-200, usWinAscent=800, usWinDescent=200,
                     sCapHeight=CAP, sxHeight=500, achVendID="SJK ")
    builder.setupPost()
    builder.setupHead(unitsPerEm=EM, created=0, modified=0)
    builder.font["head"].created = 0
    builder.font["head"].modified = 0
    out = io.BytesIO()
    builder.save(out)
    return out.getvalue()


def main():
    data = build()
    if "--check" in sys.argv[1:]:
        if not OUT.exists() or OUT.read_bytes() != data:
            print(f"sjk_symbols: {OUT} is out of date: run python scripts/sjk_symbols.py", file=sys.stderr)
            sys.exit(1)
        print("sjk_symbols: up to date")
        return
    OUT.write_bytes(data)
    print(f"wrote {OUT} ({len(data)} bytes, {len(SYMBOLS)} symbols)")


if __name__ == "__main__":
    main()
