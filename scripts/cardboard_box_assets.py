"""Make the cardboard box toy's game files (`toy_box`, SJK's own).

The box is a plain shipping carton a player hides in, Metal Gear style. Nothing here
comes from another game: the cardboard is drawn from noise and shapes. This script
writes, into the output folder:

- `cardboard_box.jpg`: the carton's picture, 1024x512: the left half a side (kraft
  board, corrugation, a hand hole, printed arrows and the SJK carton mark), the right
  half the top (two flaps meeting under a strip of tape);
- `cardboard_box.md3`: an open-bottomed box BOX_WIDTH x BOX_DEPTH x BOX_HEIGHT units,
  its origin in the middle of its open bottom, the four sides showing the side and the
  top the top;
- `toy_box.png`: the toy's icon, 128x128 like `gfx/mp/f_icon_*`: the carton seen from
  above and to one side, on transparency.

    python scripts/cardboard_box_assets.py crates/sjk-viewer/assets/cardboard_box

The output is deterministic (a fixed seed). Needs Pillow and numpy
(pip install pillow numpy). The shader (`cardboard_box.shader`) is written by hand.
"""

import math
import struct
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFilter

# A crouched player's box is 30 x 30 x 40 units (`CROUCH_MAXS_2` 16 over the -24
# floor); the model's limbs and a holstered hilt reach past it, so the carton is wider.
BOX_WIDTH = 46.0
BOX_DEPTH = 46.0
BOX_HEIGHT = 44.0
SHADER = "models/sjk/cardboard_box"
SIDE = 512
ICON_SIZE = 128
JPEG_QUALITY = 90
KRAFT = np.array([176.0, 134.0, 86.0])


def noise(rng, size, scale: float) -> np.ndarray:
    """Smooth value noise of `size` (h, w), about `scale` pixels per feature, in 0..1."""
    h, w = size
    gh, gw = int(h / scale) + 2, int(w / scale) + 2
    grid = Image.fromarray((rng.random((gh, gw)) * 255).astype(np.uint8))
    big = grid.resize((int(gw * scale), int(gh * scale)), Image.BICUBIC)
    return np.asarray(big, dtype=float)[:h, :w] / 255.0


def kraft(rng, size) -> np.ndarray:
    """Plain kraft board: brown with fibres and a little blotching, RGB floats."""
    h, w = size
    blotch = noise(rng, size, 64) * 0.6 + noise(rng, size, 16) * 0.4
    fibre = rng.random(size)
    fibre = np.asarray(
        Image.fromarray((fibre * 255).astype(np.uint8)).filter(ImageFilter.GaussianBlur(0.6)),
        dtype=float,
    ) / 255.0
    shade = 0.9 + 0.14 * blotch + 0.08 * (fibre - 0.5)
    # Corrugation: faint vertical flutes under the liner.
    flutes = 1.0 + 0.025 * np.sin(np.arange(w) * 2 * math.pi / 9.0)
    rgb = KRAFT[None, None, :] * shade[:, :, None] * flutes[None, :, None]
    return rgb


def darken_edges(rgb: np.ndarray, width: int, amount: float) -> np.ndarray:
    """Scuffed, darker edges where the board folds."""
    h, w, _ = rgb.shape
    y = np.minimum(np.arange(h), np.arange(h)[::-1])[:, None]
    x = np.minimum(np.arange(w), np.arange(w)[::-1])[None, :]
    edge = np.clip(1.0 - np.minimum(x, y) / width, 0.0, 1.0) ** 2
    return rgb * (1.0 - amount * edge[:, :, None])


def ink(rgb: np.ndarray, mask: Image.Image, colour, strength: float) -> np.ndarray:
    """Print `mask` (L) onto the board in `colour`, soaked in a little unevenly."""
    m = np.asarray(mask.filter(ImageFilter.GaussianBlur(0.8)), dtype=float)[:, :, None] / 255.0
    m = m * strength
    return rgb * (1.0 - m) + np.array(colour, float)[None, None, :] * m


def side(rng) -> np.ndarray:
    rgb = kraft(rng, (SIDE, SIDE))
    # Hand hole near the top: a dark slot with a lighter torn lip.
    hole = Image.new("L", (SIDE, SIDE), 0)
    draw = ImageDraw.Draw(hole)
    draw.rounded_rectangle((176, 70, 336, 112), radius=21, fill=255)
    lip = Image.new("L", (SIDE, SIDE), 0)
    ImageDraw.Draw(lip).rounded_rectangle((170, 64, 342, 118), radius=26, fill=255)
    rgb = ink(rgb, lip, (205, 170, 120), 0.55)
    rgb = ink(rgb, hole, (22, 16, 10), 1.0)
    # "This side up" arrows.
    arrows = Image.new("L", (SIDE, SIDE), 0)
    draw = ImageDraw.Draw(arrows)
    for cx in (96, 416):
        draw.polygon([(cx, 150), (cx - 34, 196), (cx + 34, 196)], fill=255)
        draw.rectangle((cx - 13, 196, cx + 13, 250), fill=255)
    draw.line((60, 268, 452, 268), fill=255, width=5)
    rgb = ink(rgb, arrows, (40, 30, 22), 0.85)
    # The carton mark: SJK's initials in a stencil-like box, slightly worn.
    mark = Image.new("L", (SIDE, SIDE), 0)
    draw = ImageDraw.Draw(mark)
    draw.rectangle((150, 300, 362, 430), outline=255, width=7)
    stroke = 14
    # S
    draw.line((214, 330, 178, 330), fill=255, width=stroke)
    draw.line((178, 330, 178, 364), fill=255, width=stroke)
    draw.line((178, 364, 214, 364), fill=255, width=stroke)
    draw.line((214, 364, 214, 398), fill=255, width=stroke)
    draw.line((214, 398, 178, 398), fill=255, width=stroke)
    # J
    draw.line((262, 330, 262, 398), fill=255, width=stroke)
    draw.line((262, 398, 234, 398), fill=255, width=stroke)
    draw.line((234, 398, 234, 380), fill=255, width=stroke)
    # K
    draw.line((290, 328, 290, 400), fill=255, width=stroke)
    draw.line((290, 366, 330, 328), fill=255, width=stroke)
    draw.line((294, 362, 332, 400), fill=255, width=stroke)
    wear = (noise(rng, (SIDE, SIDE), 6) > 0.2).astype(np.uint8) * 255
    mark = Image.fromarray(np.minimum(np.asarray(mark), wear))
    rgb = ink(rgb, mark, (40, 30, 22), 0.9)
    # Small print under the mark.
    lines = Image.new("L", (SIDE, SIDE), 0)
    draw = ImageDraw.Draw(lines)
    for i, length in enumerate((180, 132, 156)):
        y = 452 + i * 14
        draw.line((256 - length // 2, y, 256 + length // 2, y), fill=255, width=4)
    rgb = ink(rgb, lines, (40, 30, 22), 0.6)
    return darken_edges(rgb, 26, 0.32)


def top(rng) -> np.ndarray:
    rgb = kraft(rng, (SIDE, SIDE))
    # Two flaps meet across the middle: a dark seam with a shadow on one side.
    seam = Image.new("L", (SIDE, SIDE), 0)
    ImageDraw.Draw(seam).line((0, 256, SIDE, 256), fill=255, width=4)
    shadow = Image.new("L", (SIDE, SIDE), 0)
    ImageDraw.Draw(shadow).rectangle((0, 258, SIDE, 272), fill=255)
    rgb = ink(rgb, shadow, (90, 64, 38), 0.25)
    rgb = ink(rgb, seam, (40, 28, 18), 0.9)
    # Packing tape along the seam: paler, glossy, wrinkled, ends torn short of the edge.
    tape = Image.new("L", (SIDE, SIDE), 0)
    ImageDraw.Draw(tape).rectangle((0, 214, SIDE, 298), fill=255)
    tape_np = np.asarray(tape, dtype=float) / 255.0
    wrinkles = 0.95 + 0.1 * noise(rng, (SIDE, SIDE), 28)
    gloss = np.clip(1.0 - np.abs(np.arange(SIDE) - 236)[:, None] / 22.0, 0, 1) * 0.25
    tape_rgb = (
        np.array([222.0, 190.0, 140.0])[None, None, :] * wrinkles[:, :, None]
        + 255 * gloss[:, :, None] * np.ones((1, SIDE, 1))
    )
    a = (tape_np * 0.78)[:, :, None]
    rgb = rgb * (1 - a) + tape_rgb * a
    return darken_edges(rgb, 26, 0.32)


def texture(output: Path) -> tuple[np.ndarray, np.ndarray]:
    rng = np.random.default_rng(1998)
    s, t = side(rng), top(rng)
    atlas = np.concatenate([s, t], axis=1)
    Image.fromarray(np.clip(atlas, 0, 255).astype(np.uint8)).save(
        output / "cardboard_box.jpg", quality=JPEG_QUALITY
    )
    return s, t


def encode_normal(n) -> int:
    # The inverse of sjk-model's `decode_normal` (stock's MD3 normal), as in
    # holocron_assets.py.
    around = round(math.atan2(n[1], n[0]) * 255 / (2 * math.pi)) & 0xFF
    from_up = round(math.acos(max(-1.0, min(1.0, n[2]))) * 255 / (2 * math.pi)) & 0xFF
    return (around << 8) | from_up


def box_md3(output: Path) -> None:
    """The open-bottomed carton, its origin in the middle of the open bottom."""
    hx, hy, top_z = BOX_WIDTH / 2, BOX_DEPTH / 2, BOX_HEIGHT
    # Each face: its four corners (top-left, top-right, bottom-right, bottom-left as seen
    # from outside), its outward normal and its u range in the atlas.
    sides = [
        ((hx, -hy), (hx, hy), (1, 0, 0)),
        ((-hx, hy), (-hx, -hy), (-1, 0, 0)),
        ((hx, hy), (-hx, hy), (0, 1, 0)),
        ((-hx, -hy), (hx, -hy), (0, -1, 0)),
    ]
    quads = []
    for (ax, ay), (bx, by), normal in sides:
        quads.append(
            (
                [(ax, ay, top_z), (bx, by, top_z), (bx, by, 0.0), (ax, ay, 0.0)],
                normal,
                (0.0, 0.5),
            )
        )
    quads.append(
        (
            [(-hx, hy, top_z), (hx, hy, top_z), (hx, -hy, top_z), (-hx, -hy, top_z)],
            (0, 0, 1),
            (0.5, 1.0),
        )
    )
    positions, normals, uvs, triangles = [], [], [], []
    for corners, normal, (u0, u1) in quads:
        base = len(positions)
        for corner, (s, t) in zip(corners, ((0, 0), (1, 0), (1, 1), (0, 1))):
            positions.append(corner)
            normals.append(normal)
            uvs.append((u0 + (u1 - u0) * s, t))
        # Clockwise seen from outside: stock's front side.
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
        + name("cardboard_box", 64)
        + struct.pack(
            "<10i", 0, 1, 1, len(positions), len(triangles),
            ofs_tris, surface_header, ofs_st, ofs_verts, surface_end,
        )
        + shaders + tris + sts + verts
    )
    radius = math.sqrt(hx * hx + hy * hy + (top_z / 2) ** 2)
    frame = struct.pack(
        "<10f", -hx, -hy, 0.0, hx, hy, top_z, 0, 0, top_z / 2, radius
    ) + name("cardboard_box", 16)
    header_size = 108
    ofs_frames = header_size
    ofs_surfaces = ofs_frames + len(frame)
    end = ofs_surfaces + len(surface)
    header = (
        b"IDP3"
        + struct.pack("<i", 15)
        + name("models/sjk/cardboard_box.md3", 64)
        + struct.pack("<9i", 0, 1, 0, 1, 0, ofs_frames, ofs_surfaces, ofs_surfaces, end)
    )
    (output / "cardboard_box.md3").write_bytes(header + frame + surface)


def icon(output: Path, side_rgb: np.ndarray, top_rgb: np.ndarray) -> None:
    """The carton from above and to one side: three faces mapped by affine warps,
    drawn large and scaled down for smooth edges."""
    scale = 4
    size = ICON_SIZE * scale
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    cx = size / 2
    w, d, h = 0.36 * size, 0.36 * size, 0.40 * size
    # Isometric-ish axes: x to the lower right, y to the lower left, z up.
    ex = np.array([math.cos(math.radians(28)), math.sin(math.radians(28))]) * w * 0.82
    ey = np.array([-math.cos(math.radians(28)), math.sin(math.radians(28))]) * d * 0.82
    ez = np.array([0.0, -1.0]) * h
    top_centre = np.array([cx, size * 0.40])
    o = top_centre - (ex + ey) / 2  # the top face's back corner

    def face(picture: np.ndarray, corner, along_u, along_v, light: float) -> None:
        img = Image.fromarray(np.clip(picture * light, 0, 255).astype(np.uint8)).convert("RGBA")
        pw, ph = img.size
        # Map output (x, y) -> picture (u, v): invert [along_u along_v].
        m = np.array([[along_u[0], along_v[0]], [along_u[1], along_v[1]]]) / np.array([pw, ph])
        inv = np.linalg.inv(m)
        off = -inv @ np.asarray(corner)
        coeffs = (inv[0, 0], inv[0, 1], off[0], inv[1, 0], inv[1, 1], off[1])
        warped = img.transform((size, size), Image.AFFINE, coeffs, Image.BICUBIC)
        mask = Image.new("L", (size, size), 0)
        quad = [tuple(corner), tuple(corner + along_u), tuple(corner + along_u + along_v),
                tuple(corner + along_v)]
        ImageDraw.Draw(mask).polygon(quad, fill=255)
        canvas.paste(warped, (0, 0), mask)

    face(top_rgb, o, ex, ey, 1.08)
    # The two near sides hang from the top's near edges.
    face(side_rgb, o + ey, ex, -ez, 0.82)
    face(side_rgb, o + ex + ey, -ey, -ez, 0.62)
    # A soft dark outline so it reads on any ground, as the stock icons do.
    alpha = canvas.split()[3]
    halo = alpha.filter(ImageFilter.MaxFilter(9)).filter(ImageFilter.GaussianBlur(3))
    ground = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    ground.putalpha(halo.point(lambda v: int(v * 0.7)))
    ground.alpha_composite(canvas)
    ground.resize((ICON_SIZE, ICON_SIZE), Image.LANCZOS).save(output / "toy_box.png")


def main() -> None:
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    output = Path(sys.argv[1])
    output.mkdir(parents=True, exist_ok=True)
    side_rgb, top_rgb = texture(output)
    box_md3(output)
    icon(output, side_rgb, top_rgb)


if __name__ == "__main__":
    main()
