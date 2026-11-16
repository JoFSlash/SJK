"""Shrink a clan's full-colour logo to an emblem for one icon atlas cell.

The logo (a PNG on transparency, any size) is cropped to its ink, centred on a
transparent square and scaled to `size` (128, one cell) with Lanczos on
premultiplied alpha, so its edges keep no dark or light fringe. The client draws
the emblem in its own colours. Prints the share of the square the ink spans
across, which `jof_tag.rs` needs to place it.

Usage: python scripts/clan_emblem.py <logo.png> <out.png> [size]
"""

import sys

from PIL import Image

INK_ALPHA = 20


def emblem(logo: Image.Image, size: int) -> tuple[Image.Image, float]:
    logo = logo.convert("RGBA")
    mask = logo.getchannel("A").point(lambda a: 255 if a > INK_ALPHA else 0)
    box = mask.getbbox()
    if box is None:
        raise SystemExit("the logo has no ink")
    ink = logo.crop(box)
    width, height = ink.size
    side = max(width, height)
    square = Image.new("RGBA", (side, side), (0, 0, 0, 0))
    square.paste(ink, ((side - width) // 2, (side - height) // 2))
    small = square.convert("RGBa").resize((size, size), Image.LANCZOS).convert("RGBA")
    return small, width / side


def main() -> None:
    if len(sys.argv) not in (3, 4):
        raise SystemExit(__doc__)
    size = int(sys.argv[3]) if len(sys.argv) == 4 else 128
    small, ink_width = emblem(Image.open(sys.argv[1]), size)
    small.save(sys.argv[2], optimize=True)
    print(f"{sys.argv[2]}: {size} square, ink width {ink_width:.2f}")


if __name__ == "__main__":
    main()
