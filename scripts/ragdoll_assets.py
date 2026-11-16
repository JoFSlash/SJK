"""Make the Ragdoll toy's icon and sound (`toy_ragdoll`, SJK's own).

The Ragdoll is a meme toy: the player's body goes limp and falls where physics takes it,
as in the sandbox games, though the player is not dead. Nothing here comes from another
game: the icon is drawn from shapes, and the sound is synthesized. This script writes,
into the output folder:

- `toy_ragdoll.png`: the toy's icon, 128x128 like `gfx/mp/f_icon_*`: a limp figure
  slumped against a wall, on transparency;
- `flatline.wav`: what the player hears as the body goes limp, 22050 Hz mono 16-bit: a
  heart monitor's last two beeps, then the long tone of a flatline, fading out.

    python scripts/ragdoll_assets.py crates/sjk-viewer/assets/ragdoll

The output is deterministic. Needs Pillow (pip install pillow).
"""

import math
import struct
import sys
import wave
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter

ICON_SIZE = 128
SCALE = 4
CREAM = (255, 238, 214, 255)
ORANGE = (255, 150, 40, 255)
WALL = (92, 70, 52, 255)
RATE = 22050
TONE = 1000.0


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


def icon(output: Path) -> None:
    """The toy: a limp figure sitting slumped against a wall, head on its shoulder, one
    arm hanging, legs splayed on the floor."""
    size = ICON_SIZE * SCALE
    unit = size / 32
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(canvas)
    # The wall and the floor.
    draw.rectangle([3 * unit, 4 * unit, 6 * unit, 28 * unit], fill=WALL)
    draw.rectangle([3 * unit, 26 * unit, 29 * unit, 28 * unit], fill=WALL)

    def limb(points, width):
        scaled = [(x * unit, y * unit) for x, y in points]
        draw.line(scaled, fill=CREAM, width=int(width * unit), joint="curve")
        for x, y in scaled:
            r = width * unit / 2
            draw.ellipse([x - r, y - r, x + r, y + r], fill=CREAM)

    pelvis = (10.5, 23.5)
    neck = (9.0, 14.5)
    # The trunk, leaning on the wall.
    limb([pelvis, (9.4, 19.0), neck], 3.4)
    # Head lolling on the right shoulder.
    head = (12.2, 11.6)
    r = 2.9 * unit
    draw.ellipse([head[0] * unit - r, head[1] * unit - r, head[0] * unit + r, head[1] * unit + r], fill=CREAM)
    # An arm hanging to the floor, one in the lap.
    limb([(9.2, 15.2), (8.6, 20.0), (9.6, 24.8)], 2.2)
    limb([(10.0, 15.4), (13.4, 19.2), (16.0, 22.0)], 2.2)
    # Legs splayed out along the floor.
    limb([pelvis, (17.0, 21.6), (24.5, 24.6)], 2.6)
    limb([pelvis, (15.5, 24.6), (22.8, 25.0)], 2.6)
    # The dizzy stars above the head.
    for cx, cy in ((15.5, 6.5), (19.0, 8.5)):
        draw.polygon(star((cx * unit, cy * unit), 1.6 * unit, 0.6 * unit, 4, 0.0), fill=ORANGE)
    halo(canvas).save(output / "toy_ragdoll.png")


def star(centre: tuple[float, float], outer: float, inner: float, points: int, turn: float):
    """The corners of a star of `points` points."""
    corners = []
    for index in range(points * 2):
        radius = outer if index % 2 == 0 else inner
        angle = turn + math.pi * index / points - math.pi / 2
        corners.append((centre[0] + radius * math.cos(angle), centre[1] + radius * math.sin(angle)))
    return corners


def flatline(output: Path) -> None:
    """Two monitor beeps a beat apart, then the flatline's long tone, fading out."""
    # (start, length) of each tone in seconds; the last is the flatline.
    tones = [(0.0, 0.14), (0.62, 0.14), (1.24, 2.4)]
    total = 1.24 + 2.4 + 0.1
    samples = []
    for index in range(int(total * RATE)):
        t = index / RATE
        value = 0.0
        for start, length in tones:
            local = t - start
            if 0.0 <= local < length:
                # Soft edges, no click; the long tone fades over its last second.
                edge = min(1.0, local / 0.01, (length - local) / 0.01)
                fade = 1.0 if length < 1.0 else min(1.0, (length - local) / 1.0)
                # A monitor's tone: a sine with a little of its third harmonic.
                wave_value = math.sin(2 * math.pi * TONE * t) + 0.18 * math.sin(6 * math.pi * TONE * t)
                value += 0.42 * edge * fade * wave_value
        samples.append(int(max(-1.0, min(1.0, value)) * 32767))
    with wave.open(str(output / "flatline.wav"), "wb") as out:
        out.setnchannels(1)
        out.setsampwidth(2)
        out.setframerate(RATE)
        out.writeframes(b"".join(struct.pack("<h", s) for s in samples))


def main() -> None:
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    output = Path(sys.argv[1])
    output.mkdir(parents=True, exist_ok=True)
    icon(output)
    flatline(output)


if __name__ == "__main__":
    main()
