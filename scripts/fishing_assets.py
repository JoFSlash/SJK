"""Make the Fishing toy's game files (`toy_fishing`, SJK's own).

A fishing rod propped beside the player, a bucket at their feet, a float on the water
and the line between. Nothing here comes from another game: every part is a few
cylinders and flat colours. This script writes, into the output folder:

- `fishing.png`: the colour atlas, 64x64, a flat colour in each 16x16 cell;
- `fishing_rod.md3`: the rod along +X, 72 units from its butt (the origin) to its tip;
- `fishing_bucket.md3`: an open bucket with water in it and a handle, its origin in the
  middle of its bottom;
- `fishing_float.md3`: the float, red over white, its origin where it meets the water;
- `fishing_line.md3`: a thin square strand along +X, 64 units long (scaled to length);
- `fishing_fish.md3`: a small fish along +X, its origin in its middle;
- `fishing_pond.md3`: a small round pond, flat, its origin in its middle, for a cast
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
ATLAS = 64
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

    def write(self, output: Path, name: str) -> None:
        def text(value: str, size: int) -> bytes:
            return value.encode("ascii").ljust(size, b"\0")

        shaders = text(SHADER, 64) + struct.pack("<i", 0)
        tris = b"".join(struct.pack("<3i", *t) for t in self.triangles)
        sts = b"".join(struct.pack("<2f", *st) for st in self.uvs)
        verts = b"".join(
            struct.pack("<3hH", *(round(c * 64) for c in p), encode_normal(n))
            for p, n in zip(self.positions, self.normals)
        )
        surface_header = 108
        ofs_tris = surface_header + len(shaders)
        ofs_st = ofs_tris + len(tris)
        ofs_verts = ofs_st + len(sts)
        surface_end = ofs_verts + len(verts)
        surface = (
            b"IDP3"
            + text(name, 64)
            + struct.pack(
                "<10i", 0, 1, 1, len(self.positions), len(self.triangles),
                ofs_tris, surface_header, ofs_st, ofs_verts, surface_end,
            )
            + shaders + tris + sts + verts
        )
        lo = [min(p[a] for p in self.positions) for a in range(3)]
        hi = [max(p[a] for p in self.positions) for a in range(3)]
        middle = [(lo[a] + hi[a]) / 2 for a in range(3)]
        radius = math.sqrt(sum((hi[a] - middle[a]) ** 2 for a in range(3)))
        frame = struct.pack("<10f", *lo, *hi, *middle, radius) + text(name, 16)
        header_size = 108
        ofs_frames = header_size
        ofs_surfaces = ofs_frames + len(frame)
        end = ofs_surfaces + len(surface)
        header = (
            b"IDP3"
            + struct.pack("<i", 15)
            + text(f"models/sjk/{name}.md3", 64)
            + struct.pack("<9i", 0, 1, 0, 1, 0, ofs_frames, ofs_surfaces, ofs_surfaces, end)
        )
        (output / f"{name}.md3").write_bytes(header + frame + surface)


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


def pond(output: Path) -> None:
    mesh = Mesh()
    mesh.tube(2, 0.0, 0.6, 22.0, 22.0, "pond_edge", sides=20, caps=False)
    mesh.disc(0.6, 22.0, "pond_edge", sides=20)
    mesh.disc(0.8, 19.0, "pond", sides=20)
    mesh.write(output, "fishing_pond")


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
    pond(output)
    icon(output)


if __name__ == "__main__":
    main()
