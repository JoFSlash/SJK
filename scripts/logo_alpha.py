#!/usr/bin/env python3
"""Cut the SJK logo out of its black background.

Usage: python scripts/logo_alpha.py SJK_Logo.jpg sjk_logo.png [--previews]

Only black connected to the image border becomes transparent, so the dark blade and
the dark lines inside the emblem stay opaque. Edge pixels get an alpha from their
brightness and their colour is un-multiplied from black, so no dark fringe remains
over light backgrounds. The result is cropped to the emblem and centred on a square
canvas; scripts/sjk_branding.py derives every shipped image from it
(assets/branding/README.md). With --previews it also writes the cut-out over grey and
over white beside the output, for a visual check.

Needs Python 3 with Pillow, numpy and scipy.
"""
import sys

import numpy as np
from PIL import Image
from scipy import ndimage

args = [arg for arg in sys.argv[1:] if not arg.startswith("--")]
if len(args) != 2:
    sys.exit(__doc__)
src, out = args
rgb = np.asarray(Image.open(src).convert("RGB")).astype(np.float32) / 255.0
peak = rgb.max(axis=2)

# Background: dark pixels connected to the border.
dark = peak < 10 / 255
labels, _ = ndimage.label(dark)
border = set(np.unique(np.concatenate([labels[0], labels[-1], labels[:, 0], labels[:, -1]])))
border.discard(0)
background = np.isin(labels, list(border))

# Edge band: a few pixels around the background where alpha ramps with brightness.
band = ndimage.binary_dilation(background, iterations=6) & ~background
ramp = np.clip(peak / (60 / 255), 0.0, 1.0)

alpha = np.ones_like(peak)
alpha[background] = 0.0
alpha[band] = ramp[band]

# Un-multiply: the edge colour was composited over black, so divide by alpha.
safe = np.maximum(alpha, 1e-3)[..., None]
colour = np.where(alpha[..., None] > 0, np.clip(rgb / safe, 0.0, 1.0), 0.0)

rgba = np.dstack([colour, alpha])
img = Image.fromarray((rgba * 255 + 0.5).astype(np.uint8), "RGBA")
bbox = img.getbbox()
img = img.crop(bbox)
# Square canvas, centred, so icons and the menu can scale it uniformly.
side = max(img.size)
canvas = Image.new("RGBA", (side, side), (0, 0, 0, 0))
canvas.paste(img, ((side - img.width) // 2, (side - img.height) // 2))
canvas.save(out, optimize=True)
print("saved", out, canvas.size, "bbox", bbox)

if "--previews" in sys.argv:
    # Previews over light grey and over white, for a visual check.
    for name, bg in (("grey", (200, 200, 205, 255)), ("white", (255, 255, 255, 255))):
        base = Image.new("RGBA", canvas.size, bg)
        base.alpha_composite(canvas)
        preview = out.replace(".png", f"_on_{name}.jpg")
        base.convert("RGB").resize((768, 768), Image.LANCZOS).save(preview, quality=90)
