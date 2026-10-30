"""Make the teddy bear a mystery box stops on when it blows up (`docs/mystery-boxes.md`,
SJK's own).

A plush bear sitting up, made of ellipsoids: a body with a lighter belly, a head with a
muzzle, a black nose and eyes, round ears, arms, legs with pads under the feet, and a red
bow at the neck. Nothing here comes from another game: the fur is drawn from noise. This
script writes, into the output folder:

- `teddy_bear.jpg`: 256x256, five swatches: brown fur, the light muzzle and belly, the
  black eyes and nose, the pads' tan, the bow's red;
- `teddy_bear.md3`: the bear, about 18 units tall, its origin under its middle, facing +Y
  (the roll turns it side on to the box, so +Y faces the box's front).

    python scripts/teddy_bear_model.py crates/sjk-viewer/assets/mystery_box

The output is deterministic (a fixed seed). Needs Pillow and numpy
(pip install pillow numpy). Its shader is in `mystery_box.shader`, written by hand.
"""

import math
import struct
import sys
from pathlib import Path

import numpy as np
from PIL import Image

SHADER = "models/sjk/teddy_bear"
ATLAS = 256
JPEG_QUALITY = 90
# Swatches, pixels (left, top, right, bottom).
FUR = (0, 0, 128, 128)
LIGHT = (128, 0, 256, 128)
DARK = (0, 128, 128, 256)
PAD = (128, 128, 192, 256)
BOW = (192, 128, 256, 256)

# The bear's parts: (centre, radii, swatch); x across, y back to front (+Y the front), z
# up.
PARTS = [
    # Body and belly.
    ((0.0, 0.0, 6.2), (4.3, 3.8, 5.0), FUR),
    ((0.0, 2.35, 5.6), (2.7, 1.7, 3.3), LIGHT),
    # Head, muzzle, nose, eyes.
    ((0.0, 0.0, 13.4), (3.7, 3.3, 3.3), FUR),
    ((0.0, 2.75, 12.5), (1.8, 1.35, 1.3), LIGHT),
    ((0.0, 4.0, 13.0), (0.65, 0.45, 0.5), DARK),
    ((-1.35, 2.85, 14.2), (0.5, 0.35, 0.5), DARK),
    ((1.35, 2.85, 14.2), (0.5, 0.35, 0.5), DARK),
    # Ears and their insides.
    ((-2.75, -0.2, 16.2), (1.35, 0.85, 1.35), FUR),
    ((2.75, -0.2, 16.2), (1.35, 0.85, 1.35), FUR),
    ((-2.75, 0.5, 16.15), (0.8, 0.4, 0.8), PAD),
    ((2.75, 0.5, 16.15), (0.8, 0.4, 0.8), PAD),
    # Arms.
    ((-4.5, 0.8, 7.4), (1.5, 1.5, 2.9), FUR),
    ((4.5, 0.8, 7.4), (1.5, 1.5, 2.9), FUR),
    # Legs, sticking out in front as it sits, and the pads under the feet.
    ((-2.4, 2.0, 1.7), (1.9, 2.6, 1.7), FUR),
    ((2.4, 2.0, 1.7), (1.9, 2.6, 1.7), FUR),
    ((-2.4, 4.35, 1.8), (1.25, 0.35, 1.25), PAD),
    ((2.4, 4.35, 1.8), (1.25, 0.35, 1.25), PAD),
    # The bow: two loops and a knot under the chin.
    ((-1.15, 3.1, 10.2), (1.05, 0.55, 0.7), BOW),
    ((1.15, 3.1, 10.2), (1.05, 0.55, 0.7), BOW),
    ((0.0, 3.4, 10.2), (0.45, 0.45, 0.45), BOW),
]
RINGS = 10
SEGMENTS = 16


def noise(rng, size, scale: float) -> np.ndarray:
    """Smooth value noise of `size` (h, w), about `scale` pixels per feature, in 0..1."""
    h, w = size
    gh, gw = int(h / scale) + 2, int(w / scale) + 2
    grid = Image.fromarray((rng.random((gh, gw)) * 255).astype(np.uint8))
    big = grid.resize((int(gw * scale), int(gh * scale)), Image.BICUBIC)
    return np.asarray(big, dtype=float)[:h, :w] / 255.0


def picture(output: Path) -> None:
    rng = np.random.default_rng(1983)
    atlas = np.zeros((ATLAS, ATLAS, 3))

    def fill(region, colour, rough: float) -> None:
        left, top, right, bottom = region
        h, w = bottom - top, right - left
        # Fur: fine noise for the hairs, broad noise for the tufts.
        shade = 1.0 + rough * (noise(rng, (h, w), 2.0) - 0.5) + 0.5 * rough * (noise(rng, (h, w), 12) - 0.5)
        atlas[top:bottom, left:right] = np.array(colour)[None, None, :] * shade[:, :, None]

    fill(FUR, (128.0, 82.0, 44.0), 0.55)
    fill(LIGHT, (214.0, 178.0, 128.0), 0.35)
    fill(DARK, (18.0, 14.0, 12.0), 0.25)
    fill(PAD, (196.0, 150.0, 104.0), 0.2)
    fill(BOW, (190.0, 26.0, 34.0), 0.18)
    image = Image.fromarray(np.clip(atlas, 0, 255).astype(np.uint8))
    image.save(output / "teddy_bear.jpg", quality=JPEG_QUALITY)


def encode_normal(n) -> int:
    # The inverse of sjk-model's `decode_normal` (stock's MD3 normal), as in
    # mystery_box_model.py.
    around = round(math.atan2(n[1], n[0]) * 255 / (2 * math.pi)) & 0xFF
    from_up = round(math.acos(max(-1.0, min(1.0, n[2]))) * 255 / (2 * math.pi)) & 0xFF
    return (around << 8) | from_up


def ellipsoid(centre, radii, region, positions, normals, uvs, triangles) -> None:
    """Add an ellipsoid's vertices and triangles, its swatch inset a little so no seam
    samples the next one."""
    left, top, right, bottom = region
    inset = 6
    base = len(positions)
    for ring in range(RINGS + 1):
        lat = math.pi * ring / RINGS
        for segment in range(SEGMENTS + 1):
            lon = 2 * math.pi * segment / SEGMENTS
            unit = (math.sin(lat) * math.cos(lon), math.sin(lat) * math.sin(lon), math.cos(lat))
            positions.append(tuple(c + r * u for c, r, u in zip(centre, radii, unit)))
            n = np.array([u / r for u, r in zip(unit, radii)])
            normals.append(tuple(n / np.linalg.norm(n)))
            s = (left + inset + (right - left - 2 * inset) * segment / SEGMENTS) / ATLAS
            t = (top + inset + (bottom - top - 2 * inset) * ring / RINGS) / ATLAS
            uvs.append((s, t))
    row = SEGMENTS + 1
    for ring in range(RINGS):
        for segment in range(SEGMENTS):
            a = base + ring * row + segment
            b, c, d = a + 1, a + row + 1, a + row
            for tri in ((a, b, c), (a, c, d)):
                p = [np.array(positions[i]) for i in tri]
                if np.linalg.norm(np.cross(p[1] - p[0], p[2] - p[0])) < 1e-9:
                    continue  # the degenerate triangles at the poles
                # Clockwise seen from outside, stock's front side: the cross product
                # points inward.
                outward = np.array(normals[tri[0]]) + np.array(normals[tri[1]]) + np.array(normals[tri[2]])
                if np.dot(np.cross(p[1] - p[0], p[2] - p[0]), outward) > 0:
                    tri = (tri[0], tri[2], tri[1])
                triangles.append(tri)


def write_md3(path: Path, name: str) -> None:
    positions, normals, uvs, triangles = [], [], [], []
    for centre, radii, region in PARTS:
        ellipsoid(centre, radii, region, positions, normals, uvs, triangles)

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


def main() -> None:
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    output = Path(sys.argv[1])
    output.mkdir(parents=True, exist_ok=True)
    picture(output)
    write_md3(output / "teddy_bear.md3", "teddy_bear")


if __name__ == "__main__":
    main()
