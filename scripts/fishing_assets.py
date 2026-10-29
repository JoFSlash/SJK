"""Make the Fishing toy's game files (`toy_fishing`, SJK's own).

A fishing rod propped beside the player, a bucket at their feet, a float on the water
and the line between. Nothing here comes from another game: every part is a few
cylinders and flat colours. This script writes, into the output folder:

- `fishing.png`: the colour atlas, 128x128, a flat colour in each 16x16 cell;
- `fishing_pond.png`: the pond's water, 128x128, ripples that tile (the shader moves
  them);
- `fishing_rod.md3`: the rod along +X, 72 units from its butt (the origin) to its tip;
- `fishing_bucket.md3`: an open bucket with water in it and a handle, its origin in the
  middle of its bottom;
- `fishing_float.md3`: the float, red over white, its origin where it meets the water;
- `fishing_line.md3`: a thin square strand along +X, 64 units long (scaled to length);
- `fishing_fish.md3`: a small fish along +X, its origin in its middle;
- `fishing_pond.md3`: a small round pond with a muddy bank, stones round it, two lily
  pads and reeds, its origin in the middle of its water at ground level, for a cast
  where there is no water (the toy fishes anywhere);
- `toy_fishing.png`: the toy's icon, 128x128 like `gfx/mp/f_icon_*`: a rod over a
  bucket, on transparency.

    python scripts/fishing_assets.py crates/sjk-viewer/assets/fishing

Standard library only; the output is deterministic. The shader (`fishing.shader`) is
written by hand.
"""

import math
import struct
import sys
import zlib
from pathlib import Path

SHADER = "models/sjk/fishing"
POND_SHADER = "models/sjk/fishing_pond_water"
ATLAS = 128
CELL = 16
ICON_SIZE = 128
ROD_LENGTH = 72.0
LINE_LENGTH = 64.0

# The atlas cells, by name: (column, row) and colour.
COLOURS = {
    "cork": ((0, 0), (196, 160, 112)),
    "rod": ((1, 0), (58, 66, 44)),
    "reel": ((2, 0), (178, 184, 190)),
    "grip": ((3, 0), (34, 30, 28)),
    "bucket": ((0, 1), (120, 132, 142)),
    "rim": ((1, 1), (88, 96, 104)),
    "water": ((2, 1), (40, 92, 132)),
    "float_red": ((3, 1), (214, 44, 36)),
    "float_white": ((0, 2), (236, 236, 228)),
    "line": ((1, 2), (220, 224, 226)),
    "fish": ((2, 2), (226, 140, 52)),
    "fish_belly": ((3, 2), (240, 220, 180)),
    "pond": ((0, 3), (52, 110, 150)),
    "pond_edge": ((1, 3), (70, 96, 64)),
    "mud": ((2, 3), (84, 70, 50)),
    "rock": ((3, 3), (128, 126, 120)),
    "rock_dark": ((4, 0), (96, 94, 90)),
    "lily": ((4, 1), (64, 140, 60)),
    "flower": ((4, 2), (236, 150, 190)),
    "reed": ((4, 3), (98, 130, 62)),
    "cattail": ((5, 0), (104, 70, 40)),
}


def uv(cell: str) -> tuple:
    (column, row), _ = COLOURS[cell]
    return ((column + 0.5) * CELL / ATLAS, (row + 0.5) * CELL / ATLAS)


def png(path: Path, width: int, height: int, rgba: bytearray) -> None:
    """Write 8-bit RGBA pixels (rows top first) as a PNG."""
    rows = b"".join(
        b"\0" + bytes(rgba[y * width * 4 : (y + 1) * width * 4]) for y in range(height)
    )

    def chunk(kind: bytes, data: bytes) -> bytes:
        return (
            struct.pack(">I", len(data))
            + kind
            + data
            + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)
        )

    header = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    path.write_bytes(
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", header)
        + chunk(b"IDAT", zlib.compress(rows, 9))
        + chunk(b"IEND", b"")
    )


def atlas(output: Path) -> None:
    pixels = bytearray(ATLAS * ATLAS * 4)
    for (column, row), colour in COLOURS.values():
        for y in range(row * CELL, (row + 1) * CELL):
            for x in range(column * CELL, (column + 1) * CELL):
                i = (y * ATLAS + x) * 4
                pixels[i : i + 4] = bytes((*colour, 255))
    png(output / "fishing.png", ATLAS, ATLAS, pixels)


def encode_normal(n) -> int:
    # The inverse of sjk-model's `decode_normal` (stock's MD3 normal), as in
    # cardboard_box_assets.py.
    length = math.sqrt(sum(c * c for c in n)) or 1.0
    n = [c / length for c in n]
    around = round(math.atan2(n[1], n[0]) * 255 / (2 * math.pi)) & 0xFF
    from_up = round(math.acos(max(-1.0, min(1.0, n[2]))) * 255 / (2 * math.pi)) & 0xFF
    return (around << 8) | from_up


class Mesh:
    """Triangles with a normal and an atlas cell per vertex."""

    def __init__(self) -> None:
        self.positions, self.normals, self.uvs, self.triangles = [], [], [], []

    def vertex(self, p, n, cell: str) -> int:
        self.positions.append(tuple(p))
        self.normals.append(tuple(n))
        self.uvs.append(uv(cell))
        return len(self.positions) - 1

    def tube(self, axis: int, start: float, end: float, r0: float, r1: float, cell: str,
             sides: int = 8, centre=(0.0, 0.0), caps: bool = True) -> None:
        """A tapered tube along `axis` (0 x, 2 z) from `start` to `end`, radius r0 to r1."""
        others = [a for a in range(3) if a != axis]

        def point(along: float, angle: float, radius: float):
            p = [0.0, 0.0, 0.0]
            p[axis] = along
            p[others[0]] = centre[0] + radius * math.cos(angle)
            p[others[1]] = centre[1] + radius * math.sin(angle)
            return p

        def normal(angle: float):
            n = [0.0, 0.0, 0.0]
            n[others[0]] = math.cos(angle)
            n[others[1]] = math.sin(angle)
            return n

        ring0, ring1 = [], []
        for i in range(sides):
            angle = 2 * math.pi * i / sides
            ring0.append(self.vertex(point(start, angle, r0), normal(angle), cell))
            ring1.append(self.vertex(point(end, angle, r1), normal(angle), cell))
        for i in range(sides):
            j = (i + 1) % sides
            self.triangles += [(ring0[i], ring1[j], ring1[i]), (ring0[i], ring0[j], ring1[j])]
        if caps:
            for along, radius, sign in ((start, r0, -1.0), (end, r1, 1.0)):
                n = [0.0, 0.0, 0.0]
                n[axis] = sign
                middle = [0.0, 0.0, 0.0]
                middle[axis] = along
                middle[others[0]], middle[others[1]] = centre
                hub = self.vertex(middle, n, cell)
                ring = [self.vertex(point(along, 2 * math.pi * i / sides, radius), n, cell)
                        for i in range(sides)]
                for i in range(sides):
                    j = (i + 1) % sides
                    self.triangles.append(
                        (hub, ring[i], ring[j]) if sign < 0 else (hub, ring[j], ring[i])
                    )

    def disc(self, z: float, radius: float, cell: str, sides: int = 12) -> None:
        hub = self.vertex((0.0, 0.0, z), (0, 0, 1), cell)
        ring = [
            self.vertex(
                (radius * math.cos(2 * math.pi * i / sides),
                 radius * math.sin(2 * math.pi * i / sides), z),
                (0, 0, 1),
                cell,
            )
            for i in range(sides)
        ]
        for i in range(sides):
            self.triangles.append((hub, ring[(i + 1) % sides], ring[i]))

    def add(self, other: "Mesh", place) -> None:
        """Append `other`, each point moved by `place` (a function of a point and its
        normal that returns both)."""
        base = len(self.positions)
        for p, n in zip(other.positions, other.normals):
            p, n = place(p, n)
            self.positions.append(tuple(p))
            self.normals.append(tuple(n))
        self.uvs += other.uvs
        self.triangles += [(base + a, base + b, base + c) for a, b, c in other.triangles]

    def write(self, output: Path, name: str) -> None:
        write_md3(output, name, [(self, SHADER)])


def write_md3(output: Path, name: str, surfaces) -> None:
    """One frame, a surface for each `(mesh, shader)`."""

    def text(value: str, size: int) -> bytes:
        return value.encode("ascii").ljust(size, b"\0")

    blocks = []
    for index, (mesh, shader) in enumerate(surfaces):
        shaders = text(shader, 64) + struct.pack("<i", 0)
        tris = b"".join(struct.pack("<3i", *t) for t in mesh.triangles)
        sts = b"".join(struct.pack("<2f", *st) for st in mesh.uvs)
        verts = b"".join(
            struct.pack("<3hH", *(round(c * 64) for c in p), encode_normal(n))
            for p, n in zip(mesh.positions, mesh.normals)
        )
        surface_header = 108
        ofs_tris = surface_header + len(shaders)
        ofs_st = ofs_tris + len(tris)
        ofs_verts = ofs_st + len(sts)
        surface_end = ofs_verts + len(verts)
        blocks.append(
            b"IDP3"
            + text(f"{name}_{index}" if index else name, 64)
            + struct.pack(
                "<10i", 0, 1, 1, len(mesh.positions), len(mesh.triangles),
                ofs_tris, surface_header, ofs_st, ofs_verts, surface_end,
            )
            + shaders + tris + sts + verts
        )
    points = [p for mesh, _ in surfaces for p in mesh.positions]
    lo = [min(p[a] for p in points) for a in range(3)]
    hi = [max(p[a] for p in points) for a in range(3)]
    middle = [(lo[a] + hi[a]) / 2 for a in range(3)]
    radius = math.sqrt(sum((hi[a] - middle[a]) ** 2 for a in range(3)))
    frame = struct.pack("<10f", *lo, *hi, *middle, radius) + text(name, 16)
    body = b"".join(blocks)
    header_size = 108
    ofs_frames = header_size
    ofs_surfaces = ofs_frames + len(frame)
    end = ofs_surfaces + len(body)
    header = (
        b"IDP3"
        + struct.pack("<i", 15)
        + text(f"models/sjk/{name}.md3", 64)
        + struct.pack(
            "<9i", 0, 1, 0, len(surfaces), 0, ofs_frames, ofs_surfaces, ofs_surfaces, end
        )
    )
    (output / f"{name}.md3").write_bytes(header + frame + body)


def rod(output: Path) -> None:
    mesh = Mesh()
    mesh.tube(0, 0.0, 2.0, 0.9, 0.9, "grip")
    mesh.tube(0, 2.0, 13.0, 0.85, 0.7, "cork")
    # The reel under the rod, its spool across it.
    mesh.tube(1, -1.6, 1.6, 1.6, 1.6, "reel", sides=10, centre=(-1.9, 15.0))
    mesh.tube(0, 13.0, 17.0, 0.55, 0.5, "grip")
    mesh.tube(0, 17.0, ROD_LENGTH, 0.45, 0.12, "rod")
    mesh.write(output, "fishing_rod")


def bucket(output: Path) -> None:
    mesh = Mesh()
    mesh.tube(2, 0.0, 11.0, 6.0, 7.2, "bucket", sides=14, caps=False)
    # Its inside, facing in, and its bottom.
    inner = Mesh()
    inner.tube(2, 0.2, 10.8, 5.8, 7.0, "rim", sides=14, caps=False)
    base = len(mesh.positions)
    mesh.positions += inner.positions
    mesh.normals += [(-n[0], -n[1], -n[2]) for n in inner.normals]
    mesh.uvs += inner.uvs
    mesh.triangles += [(base + a, base + c, base + b) for a, b, c in inner.triangles]
    mesh.disc(0.3, 6.0, "rim", sides=14)
    mesh.tube(2, 10.6, 11.4, 7.5, 7.5, "rim", sides=14, caps=False)
    mesh.disc(8.0, 6.7, "water", sides=14)
    # The handle: a half hoop over the top, from one side to the other.
    steps = 8
    for i in range(steps):
        a0 = math.pi * i / steps
        a1 = math.pi * (i + 1) / steps
        p0 = (7.3 * math.cos(a0), 0.0, 11.0 + 6.0 * math.sin(a0))
        p1 = (7.3 * math.cos(a1), 0.0, 11.0 + 6.0 * math.sin(a1))
        segment = Mesh()
        length = math.dist(p0, p1)
        segment.tube(0, 0.0, length, 0.3, 0.3, "rim", sides=4, caps=False)
        angle = math.atan2(p1[2] - p0[2], p1[0] - p0[0])
        c, s = math.cos(angle), math.sin(angle)
        base = len(mesh.positions)
        for p, n in zip(segment.positions, segment.normals):
            mesh.positions.append((p0[0] + p[0] * c - p[2] * s, p[1], p0[2] + p[0] * s + p[2] * c))
            mesh.normals.append((n[0] * c - n[2] * s, n[1], n[0] * s + n[2] * c))
        mesh.uvs += segment.uvs
        mesh.triangles += [(base + a, base + b, base + c) for a, b, c in segment.triangles]
    mesh.write(output, "fishing_bucket")


def float_model(output: Path) -> None:
    mesh = Mesh()
    mesh.tube(2, -1.6, 0.0, 0.3, 1.2, "float_white", sides=10)
    mesh.tube(2, 0.0, 1.8, 1.2, 0.3, "float_red", sides=10)
    mesh.tube(2, 1.8, 3.2, 0.15, 0.15, "float_red", sides=4)
    mesh.write(output, "fishing_float")


def line(output: Path) -> None:
    mesh = Mesh()
    mesh.tube(0, 0.0, LINE_LENGTH, 0.1, 0.1, "line", sides=4, caps=False)
    mesh.write(output, "fishing_line")


def fish(output: Path) -> None:
    mesh = Mesh()
    mesh.tube(0, -5.0, 0.0, 0.4, 1.6, "fish", sides=8)
    mesh.tube(0, 0.0, 3.5, 1.6, 0.6, "fish_belly", sides=8)
    # The tail, two flat fins.
    mesh.tube(0, -7.5, -5.0, 1.8, 0.3, "fish", sides=4)
    mesh.write(output, "fishing_fish")


POND_RADIUS = 20.0
POND_WATER = 0.8


def pond_water_texture(output: Path) -> None:
    """Water that tiles: a blue ground under soft ripple bands, whole periods across
    the picture so the shader can scroll it."""
    size = 128
    pixels = bytearray(size * size * 4)
    for y in range(size):
        for x in range(size):
            u, v = 2 * math.pi * x / size, 2 * math.pi * y / size
            wave = (
                math.sin(3 * u + 2 * math.sin(2 * v))
                + 0.6 * math.sin(5 * v + 1.5 * math.sin(3 * u))
                + 0.35 * math.sin(7 * (u + v))
            ) / 1.95
            light = max(0.0, wave) ** 3
            shade = 0.86 + 0.1 * wave
            r = 36 * shade + 150 * light
            g = 96 * shade + 150 * light
            b = 136 * shade + 120 * light
            i = (y * size + x) * 4
            pixels[i : i + 4] = bytes((min(255, round(r)), min(255, round(g)), min(255, round(b)), 255))
    png(output / "fishing_pond.png", size, size, pixels)


def rock(radius: float, height: float, cell: str, sides: int, twist: float) -> Mesh:
    """A low stone: a squat, faceted lump, wider at the bottom."""
    mesh = Mesh()
    mesh.tube(2, 0.0, height * 0.6, radius, radius * 0.8, cell, sides=sides, caps=False)
    mesh.tube(2, height * 0.6, height, radius * 0.8, radius * 0.3, cell, sides=sides)
    c, s = math.cos(twist), math.sin(twist)
    out = Mesh()
    out.add(mesh, lambda p, n: ((p[0] * c - p[1] * s, p[0] * s + p[1] * c, p[2]),
                                (n[0] * c - n[1] * s, n[0] * s + n[1] * c, n[2])))
    return out


def pond(output: Path) -> None:
    """The pond: its water (its own surface and shader, mapped flat so the ripples keep
    their size), a muddy bank sloping up to the ground, stones round it, two lily pads,
    one in flower, and two clumps of reeds with cattails."""
    water = Mesh()
    sides = 24
    hub = len(water.positions)
    water.positions.append((0.0, 0.0, POND_WATER))
    water.normals.append((0, 0, 1))
    water.uvs.append((0.5, 0.5))
    for i in range(sides):
        a = 2 * math.pi * i / sides
        x, y = POND_RADIUS * math.cos(a), POND_RADIUS * math.sin(a)
        water.positions.append((x, y, POND_WATER))
        water.normals.append((0, 0, 1))
        water.uvs.append((0.5 + x / 48.0, 0.5 + y / 48.0))
    for i in range(sides):
        water.triangles.append((hub, hub + 1 + (i + 1) % sides, hub + 1 + i))

    land = Mesh()
    # The bank: from under the water's edge up and out to the ground.
    inner, outer = [], []
    for i in range(sides):
        a = 2 * math.pi * i / sides
        wobble = 1.0 + 0.06 * math.sin(3 * a) + 0.04 * math.sin(7 * a + 1)
        n = (math.cos(a) * -0.3, math.sin(a) * -0.3, 1.0)
        inner.append(land.vertex((POND_RADIUS * 0.96 * math.cos(a), POND_RADIUS * 0.96 * math.sin(a), 0.2), n, "mud"))
        r = (POND_RADIUS + 6.0) * wobble
        outer.append(land.vertex((r * math.cos(a), r * math.sin(a), 0.05), (0, 0, 1), "pond_edge"))
    for i in range(sides):
        j = (i + 1) % sides
        land.triangles += [(inner[i], outer[j], inner[j]), (inner[i], outer[i], outer[j])]
    # Stones round the bank.
    stones = 13
    for i in range(stones):
        a = 2 * math.pi * (i + 0.3 * math.sin(i * 2.3)) / stones
        r = POND_RADIUS + 3.5 + 1.5 * math.sin(i * 1.7)
        size = 2.2 + 1.2 * (0.5 + 0.5 * math.sin(i * 3.1))
        stone = rock(size, size * 0.8, "rock" if i % 3 else "rock_dark", 5 + i % 3, i * 0.9)
        x, y = r * math.cos(a), r * math.sin(a)
        land.add(stone, lambda p, n, x=x, y=y: ((p[0] + x, p[1] + y, p[2]), n))
    # Two lily pads on the water, one in flower.
    for x, y, flower in ((-8.0, 6.0, True), (6.0, -9.0, False)):
        pad = Mesh()
        pad.disc(POND_WATER + 0.15, 3.0, "lily", sides=10)
        land.add(pad, lambda p, n, x=x, y=y: ((p[0] + x, p[1] + y, p[2]), n))
        if flower:
            bloom = Mesh()
            bloom.tube(2, POND_WATER + 0.15, POND_WATER + 1.3, 0.6, 1.1, "flower", sides=6)
            land.add(bloom, lambda p, n, x=x, y=y: ((p[0] + x + 0.8, p[1] + y, p[2]), n))
    # Two clumps of reeds on the bank, cattails on the tallest.
    for a0 in (2.3, 5.4):
        for k in range(5):
            a = a0 + 0.12 * (k - 2)
            r = POND_RADIUS + 1.5 + 0.8 * (k % 2)
            height = 13.0 + 3.0 * math.sin(k * 2.1 + a0)
            lean = 0.08 * (k - 2)
            stem = Mesh()
            stem.tube(2, 0.0, height, 0.28, 0.12, "reed", sides=4)
            if k % 2 == 0:
                stem.tube(2, height * 0.72, height * 0.88, 0.55, 0.55, "cattail", sides=6)
            x, y = r * math.cos(a), r * math.sin(a)
            land.add(stem, lambda p, n, x=x, y=y, lean=lean: ((p[0] + x + lean * p[2], p[1] + y, p[2]), n))
    write_md3(output, "fishing_pond", [(water, POND_SHADER), (land, SHADER)])


def icon(output: Path) -> None:
    """A rod arching over a bucket, drawn at four times the size and scaled down."""
    scale = 4
    size = ICON_SIZE * scale
    cover = [[0.0] * size for _ in range(size)]
    colour = [[(0, 0, 0)] * size for _ in range(size)]

    def paint(inside, rgb) -> None:
        for y in range(size):
            for x in range(size):
                if inside(x / scale, y / scale):
                    cover[y][x] = 1.0
                    colour[y][x] = rgb

    def segment(ax, ay, bx, by, width):
        def inside(x, y):
            dx, dy = bx - ax, by - ay
            t = max(0.0, min(1.0, ((x - ax) * dx + (y - ay) * dy) / (dx * dx + dy * dy)))
            return math.hypot(x - (ax + t * dx), y - (ay + t * dy)) <= width
        return inside

    # The bucket: a trapezoid with a rim and water.
    paint(lambda x, y: 82 <= y <= 118 and abs(x - 44) <= 18 + (y - 82) * -0.12 + 4, COLOURS["bucket"][1])
    paint(lambda x, y: 78 <= y <= 84 and abs(x - 44) <= 23, COLOURS["rim"][1])
    paint(lambda x, y: 84 < y <= 88 and abs(x - 44) <= 19, COLOURS["water"][1])
    # The rod from the lower right up to the upper left, the line down to the float.
    paint(segment(116, 120, 30, 14, 3.2), COLOURS["rod"][1])
    paint(segment(116, 120, 104, 105, 4.5), COLOURS["cork"][1])
    paint(segment(30, 14, 18, 60, 0.9), COLOURS["line"][1])
    paint(lambda x, y: math.hypot(x - 18, y - 64) <= 5 and y < 64, COLOURS["float_red"][1])
    paint(lambda x, y: math.hypot(x - 18, y - 64) <= 5 and y >= 64, COLOURS["float_white"][1])
    pixels = bytearray(ICON_SIZE * ICON_SIZE * 4)
    for y in range(ICON_SIZE):
        for x in range(ICON_SIZE):
            r = g = b = a = 0.0
            for sy in range(scale):
                for sx in range(scale):
                    c = cover[y * scale + sy][x * scale + sx]
                    rgb = colour[y * scale + sy][x * scale + sx]
                    r, g, b, a = r + rgb[0] * c, g + rgb[1] * c, b + rgb[2] * c, a + c
            i = (y * ICON_SIZE + x) * 4
            if a > 0:
                pixels[i : i + 4] = bytes(
                    (round(r / a), round(g / a), round(b / a), round(255 * a / scale ** 2))
                )
    png(output / "toy_fishing.png", ICON_SIZE, ICON_SIZE, pixels)


def main() -> None:
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    output = Path(sys.argv[1])
    output.mkdir(parents=True, exist_ok=True)
    atlas(output)
    rod(output)
    bucket(output)
    float_model(output)
    line(output)
    fish(output)
    pond_water_texture(output)
    pond(output)
    icon(output)


if __name__ == "__main__":
    main()
