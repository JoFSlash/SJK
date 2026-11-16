"""Make the Order 66 toy's pictures (`toy_order66`, SJK's own).

Order 66 is a meme toy: every player who has it on dies at once, blown apart, when one
of them executes the order. Nothing here comes from another game or a film: the
pictures are drawn from shapes and SJK's bundled Exo 2 font. This script writes, into
the output folder:

- `toy_order66.png`: the toy's icon, 128x128 like `gfx/mp/f_icon_*`: a round red badge
  with "66" on it, on transparency;
- `order66_execute.png`: the order's icon (the quick wheel's Execute Order 66, its key
  binding), 128x128: the badge's "66" on a red and orange blast;
- `confetti_<colour>.png`: one confetti scrap of each colour, 16x16, for the confetti
  that takes the place of the pieces where dismemberment is off.

    python scripts/order66_assets.py crates/sjk-viewer/assets/order66

The output is deterministic. Needs Pillow (pip install pillow). The effect
(`order66_confetti.efx`) and the shader (`order66.shader`) are written by hand.
"""

import math
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont

ICON_SIZE = 128
SCALE = 4
FONT = Path(__file__).resolve().parent.parent / "crates/sjk-viewer/assets/fonts/Exo2-SemiBold.ttf"
RED = (196, 28, 28, 255)
DARK_RED = (92, 8, 8, 255)
ORANGE = (255, 150, 40, 255)
CREAM = (255, 238, 214, 255)
CONFETTI = {
    "red": (235, 50, 60),
    "yellow": (255, 214, 40),
    "green": (60, 200, 90),
    "cyan": (40, 200, 230),
    "blue": (60, 100, 240),
    "pink": (240, 90, 200),
}
CONFETTI_SIZE = 16


def sixty_six(draw: ImageDraw.ImageDraw, centre: tuple[float, float], height: float) -> None:
    """ "66" in Exo 2 SemiBold, `height` pixels tall, centred, cream with a dark rim."""
    font = ImageFont.truetype(str(FONT), int(height * 1.32))
    draw.text(
        centre,
        "66",
        font=font,
        fill=CREAM,
        anchor="mm",
        stroke_width=int(height * 0.06),
        stroke_fill=DARK_RED,
    )


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


def badge(output: Path) -> None:
    """The toy: a round red badge, a darker ring and "66"."""
    size = ICON_SIZE * SCALE
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(canvas)
    margin = size * 0.1
    draw.ellipse([margin, margin, size - margin, size - margin], fill=DARK_RED)
    ring = size * 0.045
    draw.ellipse(
        [margin + ring, margin + ring, size - margin - ring, size - margin - ring], fill=RED
    )
    # A highlight across the top, as a lit enamel badge.
    shine = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    ImageDraw.Draw(shine).ellipse(
        [size * 0.22, size * 0.16, size * 0.78, size * 0.5], fill=(255, 255, 255, 46)
    )
    canvas.alpha_composite(shine.filter(ImageFilter.GaussianBlur(size * 0.03)))
    sixty_six(ImageDraw.Draw(canvas), (size / 2, size * 0.53), size * 0.34)
    halo(canvas).save(output / "toy_order66.png")


def star(centre: tuple[float, float], outer: float, inner: float, points: int, turn: float):
    """The corners of a star of `points` points."""
    corners = []
    for index in range(points * 2):
        radius = outer if index % 2 == 0 else inner
        angle = turn + math.pi * index / points
        corners.append((centre[0] + radius * math.cos(angle), centre[1] + radius * math.sin(angle)))
    return corners


def blast(output: Path) -> None:
    """The order: "66" on a blast, an orange star over a red one."""
    size = ICON_SIZE * SCALE
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(canvas)
    centre = (size / 2, size / 2)
    draw.polygon(star(centre, size * 0.47, size * 0.3, 12, 0.0), fill=RED)
    draw.polygon(star(centre, size * 0.38, size * 0.24, 12, math.pi / 12), fill=ORANGE)
    draw.ellipse([size * 0.27, size * 0.27, size * 0.73, size * 0.73], fill=(255, 214, 120, 255))
    sixty_six(draw, (size / 2, size * 0.53), size * 0.3)
    halo(canvas).save(output / "order66_execute.png")


def confetti(output: Path) -> None:
    """One scrap of each colour: the colour with a lighter middle, opaque to the edge."""
    for name, (r, g, b) in CONFETTI.items():
        scrap = Image.new("RGBA", (CONFETTI_SIZE, CONFETTI_SIZE), (r, g, b, 255))
        light = tuple(min(255, int(c * 0.6 + 255 * 0.4)) for c in (r, g, b))
        ImageDraw.Draw(scrap).rectangle([4, 4, CONFETTI_SIZE - 5, CONFETTI_SIZE - 5], fill=light)
        scrap.save(output / f"confetti_{name}.png")


def main() -> None:
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    output = Path(sys.argv[1])
    output.mkdir(parents=True, exist_ok=True)
    badge(output)
    blast(output)
    confetti(output)


if __name__ == "__main__":
    main()
