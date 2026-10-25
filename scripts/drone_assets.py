"""Make the Drone toy's icon (`toy_drone`, SJK's own).

The Drone is the free camera and Peek as a toy: a little training remote other SJK
players see fly. In game it is Jedi Academy's own seeker ball
(`models/items/remote.md3`); the icon is drawn here from shapes, nothing taken from
another game. This script writes, into the output folder:

- `toy_drone.png`: the toy's icon, 128x128 like `gfx/mp/f_icon_*`: a training remote
  hovering, its band of emitters glowing, on transparency.

    python scripts/drone_assets.py crates/sjk-viewer/assets/drone

The output is deterministic. Needs Pillow (pip install pillow).
"""

import math
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter

ICON_SIZE = 128
SCALE = 4
CREAM = (255, 238, 214, 255)
ORANGE = (255, 150, 40, 255)
SHELL_LIGHT = (196, 190, 182)
SHELL_DARK = (62, 58, 56)
BAND = (40, 36, 34, 255)


def halo(canvas: Image.Image) -> Image.Image:
    """A soft dark outline under `canvas` so it reads on any ground, as the stock icons
    do; scaled down to the icon's size."""
    size = canvas.size[0]
    alpha = canvas.split()[3]
    shadow = alpha.filter(ImageFilter.MaxFilter(9)).filter(ImageFilter.GaussianBlur(3))
    ground = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    ground.putalpha(shadow.point(lambda v: int(v * 0.7)))
    ground.alpha_composite(canvas)
    return ground.resize((ICON_SIZE, ICON_SIZE), Image.LANCZOS)


def shell(size: int, centre: tuple[float, float], radius: float) -> Image.Image:
    """The ball, lit from the upper left: each pixel's shade from its sphere normal."""
    ball = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    pixels = ball.load()
    light = (-0.45, -0.6, 0.66)
    norm = math.sqrt(sum(c * c for c in light))
    light = tuple(c / norm for c in light)
    cx, cy = centre
    for y in range(int(cy - radius) - 1, int(cy + radius) + 2):
        for x in range(int(cx - radius) - 1, int(cx + radius) + 2):
            dx, dy = (x + 0.5 - cx) / radius, (y + 0.5 - cy) / radius
            d2 = dx * dx + dy * dy
            if d2 > 1.0:
                continue
            dz = math.sqrt(1.0 - d2)
            lit = max(0.0, dx * light[0] + dy * light[1] + dz * light[2])
            shine = lit**24 * 0.6
            t = min(1.0, 0.18 + 0.82 * lit + shine)
            colour = tuple(int(d + (l - d) * t) for d, l in zip(SHELL_DARK, SHELL_LIGHT))
            pixels[x, y] = (*colour, 255)
    return ball


def icon(output: Path) -> None:
    """The toy: a training remote with a dark band round its middle, emitters on the
    band glowing orange, a sensor dome on top, and a faint glow under it as it hovers."""
    size = ICON_SIZE * SCALE
    unit = size / 32
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    centre = (16 * unit, 14.5 * unit)
    radius = 10.5 * unit
    # The hover glow under it.
    glow = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    ImageDraw.Draw(glow).ellipse(
        [9 * unit, 27 * unit, 23 * unit, 30 * unit], fill=(ORANGE[0], ORANGE[1], ORANGE[2], 150)
    )
    canvas.alpha_composite(glow.filter(ImageFilter.GaussianBlur(2 * unit)))
    canvas.alpha_composite(shell(size, centre, radius))
    draw = ImageDraw.Draw(canvas)
    cx, cy = centre
    # The band round its middle, seen a little from above: a flattened ring.
    band = 2.2 * unit
    draw.chord(
        [cx - radius, cy - radius * 0.32 - band * 0.5, cx + radius, cy + radius * 0.32 + band * 0.5],
        0,
        180,
        fill=BAND,
    )
    draw.chord(
        [cx - radius, cy - radius * 0.32 + band * 0.5, cx + radius, cy + radius * 0.32 - band * 0.5],
        0,
        180,
        fill=(0, 0, 0, 0),
    )
    # Clip the band's clear part back to the shell.
    shell_only = shell(size, centre, radius)
    mask = Image.new("L", (size, size), 0)
    ImageDraw.Draw(mask).chord(
        [cx - radius, cy - radius * 0.32 + band * 0.5, cx + radius, cy + radius * 0.32 - band * 0.5],
        0,
        180,
        fill=255,
    )
    canvas.paste(shell_only, (0, 0), mask)
    # The emitters along the band's front half, glowing.
    for index in range(5):
        angle = math.pi * (index + 0.5) / 5
        x = cx - math.cos(angle) * radius * 0.93
        y = cy + math.sin(angle) * radius * 0.32
        r = (0.55 + 0.45 * math.sin(angle)) * 1.25 * unit
        light = Image.new("RGBA", (size, size), (0, 0, 0, 0))
        ImageDraw.Draw(light).ellipse([x - r * 2, y - r * 2, x + r * 2, y + r * 2], fill=(255, 120, 30, 140))
        canvas.alpha_composite(light.filter(ImageFilter.GaussianBlur(r)))
        draw = ImageDraw.Draw(canvas)
        draw.ellipse([x - r, y - r, x + r, y + r], fill=BAND)
        draw.ellipse([x - r * 0.55, y - r * 0.55, x + r * 0.55, y + r * 0.55], fill=ORANGE)
    # The sensor dome on top.
    top = cy - radius * 0.86
    draw.ellipse([cx - 2.2 * unit, top - 1.2 * unit, cx + 2.2 * unit, top + 1.4 * unit], fill=BAND)
    draw.ellipse([cx - 1.0 * unit, top - 0.6 * unit, cx + 1.0 * unit, top + 0.6 * unit], fill=CREAM)
    halo(canvas).save(output / "toy_drone.png")


def main() -> None:
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    output = Path(sys.argv[1])
    output.mkdir(parents=True, exist_ok=True)
    icon(output)


if __name__ == "__main__":
    main()
