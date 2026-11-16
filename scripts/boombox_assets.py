"""Make the boombox toy's game files (`toy_boombox`, SJK's own).

The boombox is a big 80s stereo a player carries on the shoulder. Nothing here comes
from another game: it is drawn from shapes and noise. This script writes, into the
output folder:

- `boombox.jpg`: the stereo's picture, 1024x1024: the top half its front (two speakers,
  the cassette deck, the tuner and the keys), the bottom left the plain plastic of its
  other faces, the bottom right the chrome of its handle;
- `boombox.md3`: the body, BODY_WIDTH x BODY_DEPTH x BODY_HEIGHT units, and its handle,
  its origin in the middle of the body's bottom, its front facing +X (the way a model
  looks at yaw 0), its width along Y;
- `toy_boombox.png`: the toy's icon, 128x128 like `gfx/mp/f_icon_*`: the stereo's front
  and handle on transparency.

    python scripts/boombox_assets.py crates/sjk-viewer/assets/boombox

The output is deterministic (a fixed seed). Needs Pillow and numpy
(pip install pillow numpy). The shader (`boombox.shader`) is written by hand.
"""

import math
import struct
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFilter

# A player is 64 units tall; a shoulder stereo of the 80s is about a third of that wide.
BODY_WIDTH = 20.0
BODY_DEPTH = 5.0
BODY_HEIGHT = 10.0
# The handle: two posts and a bar over the body.
HANDLE_SPAN = 15.0
HANDLE_HEIGHT = 3.0
HANDLE_THICK = 1.0
SHADER = "models/sjk/boombox"
ATLAS = 1024
FRONT = (0, 0, ATLAS, ATLAS // 2)
PLASTIC = (0, ATLAS // 2, ATLAS // 2, ATLAS)
CHROME = (ATLAS // 2, ATLAS // 2, ATLAS, ATLAS)
ICON_SIZE = 128
JPEG_QUALITY = 90


def noise(rng, size, scale: float) -> np.ndarray:
    """Smooth value noise of `size` (h, w), about `scale` pixels per feature, in 0..1."""
    h, w = size
    gh, gw = int(h / scale) + 2, int(w / scale) + 2
    grid = Image.fromarray((rng.random((gh, gw)) * 255).astype(np.uint8))
    big = grid.resize((int(gw * scale), int(gh * scale)), Image.BICUBIC)
    return np.asarray(big, dtype=float)[:h, :w] / 255.0


def plastic(rng, size, base) -> Image.Image:
    """Dark moulded plastic with a faint grain."""
    grain = noise(rng, size, 3) * 0.5 + noise(rng, size, 40) * 0.5
    rgb = np.array(base, float)[None, None, :] * (0.92 + 0.16 * grain)[:, :, None]
    return Image.fromarray(np.clip(rgb, 0, 255).astype(np.uint8))


def chrome(size) -> Image.Image:
    """Brushed chrome: bright bands across, as a bent bar reflects the sky."""
    h, w = size
    y = np.arange(h)[:, None] / h
    band = 150 + 90 * np.sin(y * math.pi * 3.0) ** 2 + 15 * np.sin(np.arange(w)[None, :] * 0.9)
    rgb = np.repeat(band[:, :, None], 3, axis=2) * np.array([0.95, 0.97, 1.0])
    return Image.fromarray(np.clip(rgb, 0, 255).astype(np.uint8))


def speaker(draw: ImageDraw.ImageDraw, cx: float, cy: float, r: float) -> None:
    """A speaker behind its chrome ring: grille, cone rings and the dust cap."""
    draw.ellipse((cx - r, cy - r, cx + r, cy + r), fill=(200, 202, 208))
    draw.ellipse((cx - r + 10, cy - r + 10, cx + r - 10, cy + r - 10), fill=(92, 94, 100))
    draw.ellipse((cx - r + 16, cy - r + 16, cx + r - 16, cy + r - 16), fill=(24, 24, 27))
    for ring in range(1, 6):
        rr = (r - 16) * (1.0 - ring * 0.14)
        shade = 34 + ring * 6
        draw.ellipse(
            (cx - rr, cy - rr, cx + rr, cy + rr),
            outline=(shade, shade, shade + 3),
            width=3,
        )
    cap = r * 0.22
    draw.ellipse((cx - cap, cy - cap, cx + cap, cy + cap), fill=(60, 60, 66))
    draw.ellipse(
        (cx - cap * 0.6, cy - cap * 0.75, cx - cap * 0.05, cy - cap * 0.25),
        fill=(110, 110, 118),
    )


def front(rng) -> Image.Image:
    w, h = FRONT[2] - FRONT[0], FRONT[3] - FRONT[1]
    img = plastic(rng, (h, w), (40, 40, 44))
    draw = ImageDraw.Draw(img)
    # A chrome trim around the face.
    draw.rectangle((0, 0, w - 1, h - 1), outline=(170, 172, 178), width=8)
    # The two speakers.
    for cx in (210, w - 210):
        speaker(draw, cx, 286, 176)
    # The tuner strip across the top: a lit dial, its marks and the red needle.
    draw.rounded_rectangle((390, 28, w - 390, 108), radius=10, fill=(214, 168, 70))
    for i in range(23):
        x = 404 + i * 9.4
        draw.line((x, 46, x, 46 + (22 if i % 5 == 0 else 12)), fill=(60, 40, 10), width=2)
    draw.line((470, 38, 470, 100), fill=(200, 20, 20), width=4)
    # The cassette deck: its door, the window, two reels and the tape between them.
    draw.rounded_rectangle((408, 134, w - 408, 352), radius=12, fill=(150, 152, 158))
    draw.rounded_rectangle((420, 146, w - 420, 340), radius=8, fill=(30, 30, 34))
    draw.rectangle((446, 200, w - 446, 286), fill=(70, 66, 60))
    for cx in (476, w - 476):
        draw.ellipse((cx - 30, 213, cx + 30, 273), fill=(220, 220, 214))
        draw.ellipse((cx - 11, 232, cx + 11, 254), fill=(30, 30, 34))
        for spoke in range(6):
            a = spoke * math.pi / 3
            draw.line(
                (cx, 243, cx + 26 * math.cos(a), 243 + 26 * math.sin(a)),
                fill=(120, 120, 116),
                width=3,
            )
    draw.line((476, 273, w - 476, 273), fill=(60, 36, 20), width=5)
    # The keys under the deck, and the SJK mark between them.
    colours = [(200, 40, 40), (230, 230, 230), (230, 230, 230), (230, 230, 230), (96, 96, 100)]
    for i, colour in enumerate(colours):
        x = 412 + i * 41
        draw.rounded_rectangle((x, 370, x + 34, 410), radius=4, fill=colour)
        draw.line((x + 4, 374, x + 30, 374), fill=(255, 255, 255), width=2)
    draw.rounded_rectangle((416, 432, w - 416, 486), radius=6, outline=(180, 180, 186), width=3)
    for i, letter in enumerate("SJK"):
        x = 470 + i * 30
        draw.text((x, 446), letter, fill=(220, 220, 226))
    # A soft sheen down from the top, as a glossy face catches the light.
    sheen = np.linspace(1.12, 0.92, h)[:, None, None]
    rgb = np.asarray(img, dtype=float) * sheen
    return Image.fromarray(np.clip(rgb, 0, 255).astype(np.uint8))


def texture(output: Path) -> Image.Image:
    rng = np.random.default_rng(1984)
    atlas = Image.new("RGB", (ATLAS, ATLAS))
    face = front(rng)
    atlas.paste(face, FRONT[:2])
    atlas.paste(plastic(rng, (PLASTIC[3] - PLASTIC[1], PLASTIC[2] - PLASTIC[0]), (34, 34, 38)),
                PLASTIC[:2])
    atlas.paste(chrome((CHROME[3] - CHROME[1], CHROME[2] - CHROME[0])), CHROME[:2])
    atlas.save(output / "boombox.jpg", quality=JPEG_QUALITY)
    return face


def encode_normal(n) -> int:
    # The inverse of sjk-model's `decode_normal` (stock's MD3 normal), as in
    # holocron_assets.py.
    around = round(math.atan2(n[1], n[0]) * 255 / (2 * math.pi)) & 0xFF
    from_up = round(math.acos(max(-1.0, min(1.0, n[2]))) * 255 / (2 * math.pi)) & 0xFF
    return (around << 8) | from_up


def region_uv(region) -> tuple[float, float, float, float]:
    x0, y0, x1, y1 = region
    return x0 / ATLAS, y0 / ATLAS, x1 / ATLAS, y1 / ATLAS


def box(low, high, front_region, rest_region):
    """The six faces of the box from `low` to `high`: each four corners (top-left,
    top-right, bottom-right, bottom-left as seen from outside), its normal and its
    picture's region; the +X face shows `front_region`, the others `rest_region`."""
    low, high = np.array(low, float), np.array(high, float)
    centre, half = (low + high) / 2, (high - low) / 2
    faces = []
    for normal, up in (
        ((1, 0, 0), (0, 0, 1)),
        ((-1, 0, 0), (0, 0, 1)),
        ((0, 1, 0), (0, 0, 1)),
        ((0, -1, 0), (0, 0, 1)),
        ((0, 0, 1), (-1, 0, 0)),
        ((0, 0, -1), (1, 0, 0)),
    ):
        n, u = np.array(normal, float), np.array(up, float)
        # Seen from outside, looking along -n with `up` up, the right is up x n.
        r = np.cross(u, n)
        c = centre + n * np.abs(n @ half)
        hr, hu = np.abs(r @ half), np.abs(u @ half)
        corners = [c - r * hr + u * hu, c + r * hr + u * hu, c + r * hr - u * hu,
                   c - r * hr - u * hu]
        faces.append((corners, normal, front_region if normal == (1, 0, 0) else rest_region))
    return faces


def boombox_md3(output: Path) -> None:
    """The body and its handle, the origin in the middle of the body's bottom."""
    hx, hy = BODY_DEPTH / 2, BODY_WIDTH / 2
    faces = box((-hx, -hy, 0.0), (hx, hy, BODY_HEIGHT), FRONT, PLASTIC)
    post = HANDLE_THICK / 2
    for y in (-HANDLE_SPAN / 2, HANDLE_SPAN / 2):
        faces += box(
            (-post, y - post, BODY_HEIGHT),
            (post, y + post, BODY_HEIGHT + HANDLE_HEIGHT),
            CHROME,
            CHROME,
        )
    faces += box(
        (-post, -HANDLE_SPAN / 2 - post, BODY_HEIGHT + HANDLE_HEIGHT),
        (post, HANDLE_SPAN / 2 + post, BODY_HEIGHT + HANDLE_HEIGHT + HANDLE_THICK),
        CHROME,
        CHROME,
    )
    positions, normals, uvs, triangles = [], [], [], []
    for corners, normal, region in faces:
        u0, v0, u1, v1 = region_uv(region)
        base = len(positions)
        for corner, (s, t) in zip(corners, ((0, 0), (1, 0), (1, 1), (0, 1))):
            positions.append(corner)
            normals.append(normal)
            uvs.append((u0 + (u1 - u0) * s, v0 + (v1 - v0) * t))
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
        + name("boombox", 64)
        + struct.pack(
            "<10i", 0, 1, 1, len(positions), len(triangles),
            ofs_tris, surface_header, ofs_st, ofs_verts, surface_end,
        )
        + shaders + tris + sts + verts
    )
    top_z = BODY_HEIGHT + HANDLE_HEIGHT + HANDLE_THICK
    radius = math.sqrt(hx * hx + hy * hy + (top_z / 2) ** 2)
    frame = struct.pack(
        "<10f", -hx, -hy, 0.0, hx, hy, top_z, 0, 0, top_z / 2, radius
    ) + name("boombox", 16)
    header_size = 108
    ofs_frames = header_size
    ofs_surfaces = ofs_frames + len(frame)
    end = ofs_surfaces + len(surface)
    header = (
        b"IDP3"
        + struct.pack("<i", 15)
        + name("models/sjk/boombox.md3", 64)
        + struct.pack("<9i", 0, 1, 0, 1, 0, ofs_frames, ofs_surfaces, ofs_surfaces, end)
    )
    (output / "boombox.md3").write_bytes(header + frame + surface)


def icon(output: Path, face: Image.Image) -> None:
    """The front and the handle over it, drawn large and scaled down for smooth edges."""
    scale = 4
    size = ICON_SIZE * scale
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    width = int(size * 0.92)
    height = width // 2
    left, top = (size - width) // 2, int(size * 0.34)
    canvas.paste(face.resize((width, height), Image.LANCZOS).convert("RGBA"), (left, top))
    draw = ImageDraw.Draw(canvas)
    span = width * HANDLE_SPAN / BODY_WIDTH
    bar = width * HANDLE_THICK / BODY_WIDTH
    rise = width * HANDLE_HEIGHT / BODY_WIDTH
    x0, x1 = size / 2 - span / 2, size / 2 + span / 2
    y = top - rise
    for x in (x0, x1 - bar):
        draw.rectangle((x, y, x + bar, top), fill=(196, 198, 206, 255))
    draw.rounded_rectangle((x0, y - bar, x1, y + bar * 0.4), radius=bar / 2,
                           fill=(214, 216, 224, 255))
    # A soft dark outline so it reads on any ground, as the stock icons do.
    alpha = canvas.split()[3]
    halo = alpha.filter(ImageFilter.MaxFilter(9)).filter(ImageFilter.GaussianBlur(3))
    ground = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    ground.putalpha(halo.point(lambda v: int(v * 0.7)))
    ground.alpha_composite(canvas)
    ground.resize((ICON_SIZE, ICON_SIZE), Image.LANCZOS).save(output / "toy_boombox.png")


def main() -> None:
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    output = Path(sys.argv[1])
    output.mkdir(parents=True, exist_ok=True)
    face = texture(output)
    boombox_md3(output)
    icon(output, face)


if __name__ == "__main__":
    main()
