"""Make the mystery box's model and pictures (`docs/mystery-boxes.md`, SJK's own).

The box is a long wooden chest bound in iron with a glowing question mark painted on each
half of its long sides, and a lid hinged at the back that the client swings open while
the box rolls. Nothing here comes from another game: the wood, the iron and the marks are
drawn from noise and shapes. This script writes, into the output folder:

- `mystery_box.jpg`: the chest's picture, 1024x1024: a long side top (planks, iron
  bands, two question marks), the lid's top below it, then an end, the dark inside and
  the iron edge strip along the bottom;
- `mystery_box_glow.jpg`: the same layout, black but for the question marks' glow (the
  shader adds it, pulsing);
- `mystery_box.md3`: the chest, LENGTH along y, DEPTH along x (its front, a long side,
  facing +X), HEIGHT tall, its origin in the middle of its bottom, open-looking at the
  top (the dark inside);
- `mystery_box_lid.md3`: the lid, its origin on the hinge (the back top edge), reaching
  +X over the chest; the client sets it at (-DEPTH/2 - LID_OVERHANG, 0, HEIGHT) and
  pitches it open;
- `toy_boxhunt.png`: the Box hunt toy's icon, 128x128: the chest's front, on
  transparency.

    python scripts/mystery_box_model.py crates/sjk-viewer/assets/mystery_box

The output is deterministic (a fixed seed). Needs Pillow and numpy
(pip install pillow numpy). The shader (`mystery_box.shader`) is written by hand.
"""

import math
import struct
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFilter

LENGTH = 60.0
DEPTH = 26.0
HEIGHT = 24.0
LID_HEIGHT = 5.0
LID_OVERHANG = 1.0
SHADER = "models/sjk/mystery_box"
ATLAS = 1024
JPEG_QUALITY = 90
ICON_SIZE = 128
# Atlas regions, pixels (left, top, right, bottom).
SIDE = (0, 0, 1024, 410)
LID = (0, 410, 1024, 860)
END = (0, 860, 180, 1024)
INSIDE = (190, 860, 370, 1024)
EDGE = (380, 860, 1024, 940)
WOOD = np.array([92.0, 58.0, 34.0])
IRON = np.array([58.0, 58.0, 62.0])
PAINT = (236, 214, 150)
GLOW = (150, 225, 255)


def noise(rng, size, scale: float) -> np.ndarray:
    """Smooth value noise of `size` (h, w), about `scale` pixels per feature, in 0..1."""
    h, w = size
    gh, gw = int(h / scale) + 2, int(w / scale) + 2
    grid = Image.fromarray((rng.random((gh, gw)) * 255).astype(np.uint8))
    big = grid.resize((int(gw * scale), int(gh * scale)), Image.BICUBIC)
    return np.asarray(big, dtype=float)[:h, :w] / 255.0


def planks(rng, size, count: int) -> np.ndarray:
    """Weathered planks running across `size` (h, w), `count` of them, RGB floats."""
    h, w = size
    # Grain: noise stretched along the planks.
    grain = noise(rng, (h, w // 8 + 2), 3.0)
    grain = np.asarray(
        Image.fromarray((grain * 255).astype(np.uint8)).resize((w, h), Image.BICUBIC),
        dtype=float,
    ) / 255.0
    blotch = noise(rng, size, 70)
    shade = 0.72 + 0.28 * grain + 0.16 * (blotch - 0.5)
    rgb = WOOD[None, None, :] * shade[:, :, None]
    # Each plank its own tone, a dark seam between them.
    for i in range(count):
        top, bottom = i * h // count, (i + 1) * h // count
        rgb[top:bottom] *= 0.88 + 0.2 * rng.random()
        rgb[max(top - 3, 0) : top + 3] *= 0.35
    return rgb


def iron_band(rgb: np.ndarray, x0: int, x1: int, rng) -> None:
    """An iron band across the planks from column `x0` to `x1`, riveted."""
    h = rgb.shape[0]
    band = IRON[None, None, :] * (0.8 + 0.3 * noise(rng, (h, x1 - x0), 6))[:, :, None]
    # A lighter top edge and a darker bottom one, as if raised.
    band[:, :3] *= 1.35
    band[:, -3:] *= 0.55
    rgb[:, x0:x1] = band
    for y in range(18, h, 52):
        cx = (x0 + x1) // 2
        yy, xx = np.ogrid[: h, x0:x1]
        dot = (yy - y) ** 2 + (xx - cx) ** 2 <= 36
        rgb[:, x0:x1][dot] = IRON * 1.6


def question_mark(size: int) -> Image.Image:
    """A bold question mark, white on black, `size` pixels tall."""
    img = Image.new("L", (size, size), 0)
    draw = ImageDraw.Draw(img)
    width = size // 9
    r = size * 0.24
    cx, cy = size / 2, size * 0.3
    draw.arc((cx - r, cy - r, cx + r, cy + r), 160, 405, fill=255, width=width)
    # From the arc's lower right end down to the stem.
    inner = r - width / 2
    end = (cx + inner * math.cos(math.radians(45)), cy + inner * math.sin(math.radians(45)))
    stem_top = (cx, cy + r * 1.35)
    stem_bottom = (cx, size * 0.72)
    draw.line([end, stem_top], fill=255, width=width)
    draw.line([stem_top, stem_bottom], fill=255, width=width)
    # Round joints and ends so the strokes meet cleanly.
    for x, y in (end, stem_top, stem_bottom):
        half = width / 2
        draw.ellipse((x - half, y - half, x + half, y + half), fill=255)
    dot = size * 0.86
    draw.ellipse((cx - width * 0.7, dot - width * 0.7, cx + width * 0.7, dot + width * 0.7), fill=255)
    return img


def darken_edges(rgb: np.ndarray, width: int, amount: float) -> np.ndarray:
    h, w, _ = rgb.shape
    y = np.minimum(np.arange(h), np.arange(h)[::-1])[:, None]
    x = np.minimum(np.arange(w), np.arange(w)[::-1])[None, :]
    edge = np.clip(1.0 - np.minimum(x, y) / width, 0.0, 1.0) ** 2
    return rgb * (1.0 - amount * edge[:, :, None])


def region_size(region) -> tuple[int, int]:
    return region[3] - region[1], region[2] - region[0]


def pictures(output: Path) -> Image.Image:
    rng = np.random.default_rng(1935)
    atlas = np.zeros((ATLAS, ATLAS, 3))
    glow = Image.new("RGB", (ATLAS, ATLAS), (0, 0, 0))

    # The long side: planks, bands at both ends and the middle, a mark on each half.
    h, w = region_size(SIDE)
    side = planks(rng, (h, w), 4)
    for x0 in (0, w // 2 - 28, w - 56):
        iron_band(side, x0, x0 + 56, rng)
    mark = question_mark(int(h * 0.8))
    mark_glow = mark.filter(ImageFilter.GaussianBlur(14))
    for centre in (w // 4 + 14, 3 * w // 4 - 14):
        left, top = centre - mark.width // 2, (h - mark.height) // 2
        m = np.asarray(mark.filter(ImageFilter.GaussianBlur(1.2)), dtype=float)[:, :, None] / 255.0
        patch = side[top : top + mark.height, left : left + mark.width]
        side[top : top + mark.height, left : left + mark.width] = patch * (1 - m) + np.array(PAINT) * m
        coloured = Image.merge(
            "RGB",
            [
                Image.eval(Image.blend(mark_glow, mark, 0.5), lambda v, c=c: int(v * c / 255))
                for c in GLOW
            ],
        )
        glow.paste(coloured, (SIDE[0] + left, SIDE[1] + top))
    atlas[SIDE[1] : SIDE[3], SIDE[0] : SIDE[2]] = darken_edges(side, 18, 0.45)

    # The lid's top: planks along it, bands across.
    h, w = region_size(LID)
    lid = planks(rng, (h, w), 5)
    for x0 in (0, w // 2 - 28, w - 56):
        iron_band(lid, x0, x0 + 56, rng)
    atlas[LID[1] : LID[3], LID[0] : LID[2]] = darken_edges(lid, 18, 0.45)

    h, w = region_size(END)
    end = planks(rng, (h, w), 3)
    iron_band(end, 0, 18, rng)
    iron_band(end, w - 18, w, rng)
    atlas[END[1] : END[3], END[0] : END[2]] = darken_edges(end, 10, 0.4)

    h, w = region_size(INSIDE)
    atlas[INSIDE[1] : INSIDE[3], INSIDE[0] : INSIDE[2]] = planks(rng, (h, w), 3) * 0.25

    h, w = region_size(EDGE)
    edge = IRON[None, None, :] * (0.8 + 0.3 * noise(rng, (h, w), 6))[:, :, None]
    atlas[EDGE[1] : EDGE[3], EDGE[0] : EDGE[2]] = edge

    image = Image.fromarray(np.clip(atlas, 0, 255).astype(np.uint8))
    image.save(output / "mystery_box.jpg", quality=JPEG_QUALITY)
    glow.save(output / "mystery_box_glow.jpg", quality=JPEG_QUALITY)
    return image


def encode_normal(n) -> int:
    # The inverse of sjk-model's `decode_normal` (stock's MD3 normal), as in
    # cardboard_box_assets.py.
    around = round(math.atan2(n[1], n[0]) * 255 / (2 * math.pi)) & 0xFF
    from_up = round(math.acos(max(-1.0, min(1.0, n[2]))) * 255 / (2 * math.pi)) & 0xFF
    return (around << 8) | from_up


def uv_of(region, s: float, t: float) -> tuple[float, float]:
    left, top, right, bottom = region
    return ((left + (right - left) * s) / ATLAS, (top + (bottom - top) * t) / ATLAS)


def cuboid(x0, x1, y0, y1, z0, z1, regions) -> list:
    """The faces of a box as quads: (corners top-left, top-right, bottom-right,
    bottom-left seen from outside, normal, region); `regions` maps a face name (+x, -x,
    +y, -y, +z, -z) to its atlas region, or leaves it out."""
    faces = {
        "+x": ([(x1, y0, z1), (x1, y1, z1), (x1, y1, z0), (x1, y0, z0)], (1, 0, 0)),
        "-x": ([(x0, y1, z1), (x0, y0, z1), (x0, y0, z0), (x0, y1, z0)], (-1, 0, 0)),
        "+y": ([(x1, y1, z1), (x0, y1, z1), (x0, y1, z0), (x1, y1, z0)], (0, 1, 0)),
        "-y": ([(x0, y0, z1), (x1, y0, z1), (x1, y0, z0), (x0, y0, z0)], (0, -1, 0)),
        "+z": ([(x0, y0, z1), (x0, y1, z1), (x1, y1, z1), (x1, y0, z1)], (0, 0, 1)),
        "-z": ([(x1, y0, z0), (x1, y1, z0), (x0, y1, z0), (x0, y0, z0)], (0, 0, -1)),
    }
    return [(*faces[name], region) for name, region in regions.items()]


def write_md3(path: Path, name: str, quads: list) -> None:
    """`quads`: (corners, normal, region) as [`cuboid`] gives them, or (corners, four
    normals, None, four (s, t)) for a curved face; a triangle repeats its last corner.
    Each is turned clockwise seen from outside, stock's front side, if it is not."""
    positions, normals, uvs, triangles = [], [], [], []
    for quad in quads:
        corners, normal, region = quad[:3]
        flat = not isinstance(normal[0], (tuple, list))
        corner_normals = [normal] * 4 if flat else list(normal)
        if len(quad) > 3:
            sts = quad[3]
        else:
            sts = [uv_of(region, s, t) for s, t in ((0, 0), (1, 0), (1, 1), (0, 1))]
        p = [np.array(c, dtype=float) for c in corners]
        outward = np.sum(np.array(corner_normals, dtype=float), axis=0)
        cross = np.cross(p[1] - p[0], p[2] - p[0])
        order = [0, 1, 2, 3] if np.dot(cross, outward) <= 0 else [0, 3, 2, 1]
        base = len(positions)
        for i in order:
            positions.append(tuple(corners[i]))
            normals.append(tuple(corner_normals[i]))
            uvs.append(tuple(sts[i]))
        triangles.append((base, base + 1, base + 2))
        if tuple(corners[order[2]]) != tuple(corners[order[3]]):
            triangles.append((base, base + 2, base + 3))

    def text(value: str, size: int) -> bytes:
        return value.encode("ascii").ljust(size, b"\0")

    shaders = text(SHADER, 64) + struct.pack("<i", 0)
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
        + text(name, 64)
        + struct.pack(
            "<10i", 0, 1, 1, len(positions), len(triangles),
            ofs_tris, surface_header, ofs_st, ofs_verts, surface_end,
        )
        + shaders + tris + sts + verts
    )
    points = np.array(positions)
    mins, maxs = points.min(axis=0), points.max(axis=0)
    centre = (mins + maxs) / 2
    radius = float(np.linalg.norm(maxs - centre))
    frame = struct.pack("<10f", *mins, *maxs, *centre, radius) + text(name, 16)
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
    path.write_bytes(header + frame + surface)


# Every face of an iron part.
IRON_ALL = {"+x": EDGE, "-x": EDGE, "+y": EDGE, "-y": EDGE, "+z": EDGE, "-z": EDGE}
# The chest stands on four feet under an iron plinth; its planked body starts above it.
FOOT = 1.5
PLINTH = 3.5
# The iron bands round the chest and over the lid, where the pictures paint them: both
# ends and the middle, 56 of the side's 1024 pixels wide, standing BAND_OUT proud.
BAND = 56 / 1024 * LENGTH
BAND_OUT = 0.5
# How high the lid's planks arch over its straight edge, and in how many strips.
LID_EDGE = 1.5
LID_ARCH = 7.0
ARCH_STEPS = 10


def bands_y(half: float) -> list:
    """Where the three bands run along the length: both ends and the middle."""
    return [(-half, -half + BAND), (-BAND / 2, BAND / 2), (half - BAND, half)]


def chest() -> list:
    hx, hy = DEPTH / 2, LENGTH / 2
    quads = []
    # Feet at the corners, the plinth on them.
    for sx in (-1, 1):
        for sy in (-1, 1):
            x, y = sx * (hx - 1.0), sy * (hy - 1.0)
            quads += cuboid(x - 2, x + 2, y - 2, y + 2, 0.0, FOOT, IRON_ALL)
    quads += cuboid(-hx - 1, hx + 1, -hy - 1, hy + 1, FOOT, PLINTH, IRON_ALL)
    # The planked body, dark inside at the top.
    quads += cuboid(
        -hx, hx, -hy, hy, PLINTH, HEIGHT,
        {"+x": SIDE, "-x": SIDE, "+y": END, "-y": END, "+z": INSIDE},
    )
    # Raised iron bands round it, over the painted ones (the end ones wrap the ends).
    for y0, y1 in bands_y(hy):
        y0 -= BAND_OUT if y0 <= -hy else 0.0
        y1 += BAND_OUT if y1 >= hy else 0.0
        quads += cuboid(
            -hx - BAND_OUT, hx + BAND_OUT, y0, y1, PLINTH, HEIGHT,
            {"+x": EDGE, "-x": EDGE, "+y": EDGE, "-y": EDGE, "+z": EDGE},
        )
    # Iron corner posts, standing out of the bands.
    for sx in (-1, 1):
        for sy in (-1, 1):
            x, y = sx * hx, sy * hy
            quads += cuboid(x - 1.2, x + 1.2, y - 1.2, y + 1.2, PLINTH, HEIGHT + 0.6, IRON_ALL)
    # A rim round the top opening.
    rim = 1.4
    quads += cuboid(hx - 0.2, hx + 0.7, -hy, hy, HEIGHT - rim, HEIGHT, IRON_ALL)
    quads += cuboid(-hx - 0.7, -hx + 0.2, -hy, hy, HEIGHT - rim, HEIGHT, IRON_ALL)
    quads += cuboid(-hx, hx, hy - 0.2, hy + 0.7, HEIGHT - rim, HEIGHT, IRON_ALL)
    quads += cuboid(-hx, hx, -hy - 0.7, -hy + 0.2, HEIGHT - rim, HEIGHT, IRON_ALL)
    # The lock plate on the front, its hasp standing out of it.
    quads += cuboid(hx, hx + 1.0, -3.0, 3.0, HEIGHT - 7.0, HEIGHT - 1.4, IRON_ALL)
    quads += cuboid(hx + 1.0, hx + 1.6, -1.0, 1.0, HEIGHT - 5.5, HEIGHT - 2.0, IRON_ALL)
    # A handle on each end: a bar on two posts.
    for sy in (-1, 1):
        near, far = hy + 0.6, hy + 2.2
        y0, y1 = (near, far) if sy > 0 else (-far, -near)
        bar0, bar1 = (far - 0.8, far) if sy > 0 else (-far, -far + 0.8)
        quads += cuboid(-4.0, 4.0, bar0, bar1, 14.0, 15.4, IRON_ALL)
        for x in (-4.0, 3.0):
            quads += cuboid(x, x + 1.0, y0, y1, 14.0, 15.4, IRON_ALL)
    return quads


def arch(reach: float, rise: float, steps: int) -> list:
    """Points across the lid's arch from the hinge to the front edge, half an ellipse:
    (x, z, normal x, normal z)."""
    points = []
    for i in range(steps + 1):
        a = math.pi * i / steps
        x = reach / 2 * (1 - math.cos(a))
        z = rise * math.sin(a)
        nx, nz = (x - reach / 2) / (reach / 2) ** 2, z / rise**2
        length = math.hypot(nx, nz)
        points.append((x, z, nx / length, nz / length))
    return points


def lid() -> list:
    reach = DEPTH + 2 * LID_OVERHANG
    hy = LENGTH / 2 + LID_OVERHANG
    quads = cuboid(
        0.0, reach, -hy, hy, 0.0, LID_EDGE,
        {"+x": EDGE, "-x": EDGE, "+y": EDGE, "-y": EDGE, "-z": INSIDE},
    )
    points = arch(reach, LID_ARCH, ARCH_STEPS)

    def on_arch(i: int, y: float, grow: float) -> tuple:
        x, z, nx, nz = points[i]
        return (x + nx * grow, y, LID_EDGE + z + nz * grow)

    def strip(y0: float, y1: float, grow: float, region) -> None:
        for i in range(ARCH_STEPS):
            corners = [on_arch(i, y0, grow), on_arch(i, y1, grow), on_arch(i + 1, y1, grow), on_arch(i + 1, y0, grow)]
            na, nb = (points[i][2], 0.0, points[i][3]), (points[i + 1][2], 0.0, points[i + 1][3])
            ta, tb = i / ARCH_STEPS, (i + 1) / ARCH_STEPS
            sts = [uv_of(region, 0, ta), uv_of(region, 1, ta), uv_of(region, 1, tb), uv_of(region, 0, tb)]
            quads.append((corners, [na, na, nb, nb], None, sts))

    # The planks over the arch, the lid's picture along its length.
    strip(-hy, hy, 0.0, LID)
    # The arch's ends, closed: a fan from the middle of the edge's top.
    for y, ny in ((hy, 1.0), (-hy, -1.0)):
        centre = (reach / 2, y, LID_EDGE)
        for i in range(ARCH_STEPS):
            a, b = on_arch(i, y, 0.0), on_arch(i + 1, y, 0.0)
            st = lambda p: uv_of(END, p[0] / reach, 1 - (p[2] - LID_EDGE) / LID_ARCH)
            quads.append(([centre, a, b, b], [(0.0, ny, 0.0)] * 4, None, [st(centre), st(a), st(b), st(b)]))
    # Raised iron bands over the arch, and their sides.
    for y0, y1 in bands_y(hy):
        strip(y0, y1, BAND_OUT, EDGE)
        for y, ny in ((y0, -1.0), (y1, 1.0)):
            for i in range(ARCH_STEPS):
                corners = [on_arch(i, y, 0.0), on_arch(i, y, BAND_OUT), on_arch(i + 1, y, BAND_OUT), on_arch(i + 1, y, 0.0)]
                sts = [uv_of(EDGE, 0, 0), uv_of(EDGE, 0.05, 0), uv_of(EDGE, 0.05, 1), uv_of(EDGE, 0, 1)]
                quads.append((corners, [(0.0, ny, 0.0)] * 4, None, sts))
    return quads


def models(output: Path) -> None:
    write_md3(output / "mystery_box.md3", "mystery_box", chest())
    write_md3(output / "mystery_box_lid.md3", "mystery_box_lid", lid())


def icon(output: Path, atlas: Image.Image) -> None:
    """The chest's front with its lid's edge above, on transparency."""
    scale = 4
    size = ICON_SIZE * scale
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    front = atlas.crop(SIDE)
    width = int(size * 0.92)
    height = int(width * (SIDE[3] - SIDE[1]) / (SIDE[2] - SIDE[0]))
    lid = atlas.crop((LID[0], LID[1], LID[2], LID[1] + 90))
    lid_height = height // 5
    top = (size - height - lid_height) // 2
    left = (size - width) // 2
    canvas.paste(lid.resize((width + 16, lid_height)).convert("RGBA"), (left - 8, top))
    canvas.paste(front.resize((width, height)).convert("RGBA"), (left, top + lid_height))
    alpha = canvas.split()[3]
    halo = alpha.filter(ImageFilter.MaxFilter(9)).filter(ImageFilter.GaussianBlur(3))
    ground = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    ground.putalpha(halo.point(lambda v: int(v * 0.7)))
    ground.alpha_composite(canvas)
    ground.resize((ICON_SIZE, ICON_SIZE), Image.LANCZOS).save(output / "toy_boxhunt.png")


def main() -> None:
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    output = Path(sys.argv[1])
    output.mkdir(parents=True, exist_ok=True)
    atlas = pictures(output)
    models(output)
    icon(output, atlas)


if __name__ == "__main__":
    main()
