#!/usr/bin/env python3
"""Derive every shipped SJK logo image from the transparent cut-out.

Usage:
    python scripts/logo_alpha.py SJK_Logo.jpg sjk_logo.png
    python scripts/sjk_branding.py sjk_logo.png [repository root, default .]

The cut-out (from scripts/logo_alpha.py) is the square, transparent emblem at
its original resolution. This script writes, relative to the repository root:

- assets/branding/sjk-logo.png: the 1024 master, also the client's menu emblem;
- assets/branding/sjk-logo-512.png: the README and release-notes logo;
- assets/branding/emblem-core.png, emblem-lights.png: the menu emblem's glow
  layers (512, additive: black adds nothing), the orange core and ring and the
  cyan blade lights, which the client pulses and shimmers;
- assets/branding/icon-32.png, icon-64.png: the client's window icons;
- assets/branding/sjk.ico: the Windows icon of sjk.exe and sjk-server.exe;
- site/assets/: the site logo, favicons, Apple touch icon and social preview.

Icons of 40 pixels and less show the medallion (the dark ring, its glowing core
and the blade) instead of the whole starburst, whose spikes blur into a brown
blot at those sizes. assets/branding/README.md explains the choices.

Needs Python 3 with Pillow, numpy and scipy. The output is deterministic for a
given input and library versions.
"""
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFilter, ImageFont
from scipy import ndimage

# Emblem geometry, as fractions of the cut-out's side (measured on it).
RING_RADIUS = 0.164  # outer edge of the dark ring around the glowing core
BLADE_HALF_WIDTH = 0.036  # the blade column, prongs included
# The medallion crop of small icons: the ring a little enlarged so a sliver of
# the gold behind it frames it, on a square that keeps the blade's ends.
MEDALLION_RING = 1.06
MEDALLION_SIDE = 0.42
# Icon sizes in the .ico, and the largest one that uses the medallion.
ICO_SIZES = (16, 20, 24, 32, 40, 48, 64, 96, 128, 256)
MEDALLION_MAX = 40
GLOW_SIZE = 512
# Site colours (site/assets/style.css): --space and --space-2.
SPACE = (6, 10, 20)
SPACE_2 = (11, 20, 48)


def smoothstep(edge0, edge1, x):
    t = np.clip((x - edge0) / (edge1 - edge0), 0.0, 1.0)
    return t * t * (3.0 - 2.0 * t)


def resize(image, size):
    """Lanczos resize with premultiplied alpha, so edges keep their colour."""
    if image.size == (size, size):
        return image.copy()
    return image.convert("RGBa").resize((size, size), Image.LANCZOS).convert("RGBA")


def sharpen(image, size):
    """A light unsharp mask for small icons, which Lanczos leaves soft."""
    if size > 32:
        return image
    rgb = image.convert("RGB").filter(ImageFilter.UnsharpMask(radius=0.6, percent=70, threshold=0))
    rgb.putalpha(image.getchannel("A"))
    return rgb


def hsv(rgb):
    peak = rgb.max(axis=2)
    low = rgb.min(axis=2)
    span = np.maximum(peak - low, 1e-6)
    r, g, b = rgb[..., 0], rgb[..., 1], rgb[..., 2]
    hue = np.where(
        peak == r,
        ((g - b) / span) % 6.0,
        np.where(peak == g, (b - r) / span + 2.0, (r - g) / span + 4.0),
    ) * 60.0
    saturation = np.where(peak > 0, (peak - low) / np.maximum(peak, 1e-6), 0.0)
    return hue, saturation, peak


def centre_distance(side):
    yy, xx = np.mgrid[0:side, 0:side].astype(np.float32)
    centre = (side - 1) / 2.0
    return np.hypot(xx - centre, yy - centre) / side, np.abs(xx - centre) / side


def glow_layer(rgba, mask, sharp, blur_sigma, blur_gain):
    """An additive glow: the masked colour itself plus a soft halo around it,
    normalised so its brightest channel is white-level, at GLOW_SIZE."""
    side = rgba.shape[0]
    colour = rgba[..., :3] * (mask * rgba[..., 3])[..., None]
    sigma = blur_sigma * side
    halo = np.dstack([ndimage.gaussian_filter(colour[..., c], sigma) for c in range(3)])
    glow = sharp * colour + blur_gain * halo
    glow /= max(float(glow.max()), 1e-6)
    image = Image.fromarray((np.clip(glow, 0.0, 1.0) * 255 + 0.5).astype(np.uint8), "RGB")
    return image.resize((GLOW_SIZE, GLOW_SIZE), Image.LANCZOS)


def core_mask(rgba):
    """The glowing orange ring and core inside the dark ring: bright, saturated
    orange within the ring's radius (the gold spikes are less saturated and
    lie outside it)."""
    hue, saturation, value = hsv(rgba[..., :3])
    distance, _ = centre_distance(rgba.shape[0])
    orange = smoothstep(8.0, 18.0, hue) * (1.0 - smoothstep(48.0, 60.0, hue))
    return (
        orange
        * smoothstep(0.45, 0.65, saturation)
        * smoothstep(0.70, 0.92, value)
        * (1.0 - smoothstep(RING_RADIUS - 0.012, RING_RADIUS - 0.002, distance))
    )


def lights_mask(rgba):
    """The cyan lights along the blade."""
    hue, saturation, value = hsv(rgba[..., :3])
    cyan = smoothstep(150.0, 168.0, hue) * (1.0 - smoothstep(205.0, 222.0, hue))
    return cyan * smoothstep(0.25, 0.45, saturation) * smoothstep(0.35, 0.60, value)


def medallion(cutout):
    """The small-icon crop: the ring and its core, with the blade."""
    rgba = np.asarray(cutout).astype(np.float32) / 255.0
    side = rgba.shape[0]
    distance, across = centre_distance(side)
    feather = 2.0 / side
    radius = RING_RADIUS * MEDALLION_RING
    disc = np.clip((radius - distance) / feather + 0.5, 0.0, 1.0)
    blade = np.clip((BLADE_HALF_WIDTH - across) / feather + 0.5, 0.0, 1.0)
    rgba[..., 3] *= np.maximum(disc, blade)
    image = Image.fromarray((rgba * 255 + 0.5).astype(np.uint8), "RGBA")
    crop = int(round(side * MEDALLION_SIDE))
    offset = (side - crop) // 2
    return image.crop((offset, offset, offset + crop, offset + crop))


def icon(cutout, small, size):
    source = small if size <= MEDALLION_MAX else cutout
    return sharpen(resize(source, size), size)


def backdrop(width, height):
    """The site's deep-space gradient with a soft gold glow behind the logo."""
    yy, xx = np.mgrid[0:height, 0:width].astype(np.float32)
    t = (yy / max(height - 1, 1))[..., None]
    base = np.array(SPACE_2, np.float32) * (1.0 - t) + np.array(SPACE, np.float32) * t
    return base, xx, yy


def glow_spot(xx, yy, centre, radius, colour, strength):
    falloff = np.exp(-(((xx - centre[0]) ** 2 + (yy - centre[1]) ** 2) / (2.0 * radius**2)))
    return falloff[..., None] * np.array(colour, np.float32) * strength


def to_image(array):
    return Image.fromarray(np.clip(array + 0.5, 0, 255).astype(np.uint8), "RGB")


def touch_icon(cutout, size=180):
    base, xx, yy = backdrop(size, size)
    base += glow_spot(xx, yy, (size / 2, size / 2), size * 0.32, (232, 184, 74), 0.22)
    image = to_image(base).convert("RGBA")
    logo = resize(cutout, int(size * 0.86))
    offset = (size - logo.width) // 2
    image.alpha_composite(logo, (offset, offset))
    return image.convert("RGB")


def social_preview(cutout, font_dir):
    """1200x630 Open Graph card: the logo left, the name and line right."""
    width, height = 1200, 630
    base, xx, yy = backdrop(width, height)
    logo_size = 520
    centre = (64 + logo_size / 2, height / 2)
    base += glow_spot(xx, yy, centre, 210.0, (232, 184, 74), 0.20)
    base += glow_spot(xx, yy, (width * 0.5, -120.0), 520.0, (23, 40, 79), 0.9)
    image = to_image(base).convert("RGBA")
    logo = resize(cutout, logo_size)
    image.alpha_composite(logo, (int(centre[0] - logo_size / 2), int(centre[1] - logo_size / 2)))
    draw = ImageDraw.Draw(image)
    semibold = ImageFont.truetype(str(font_dir / "Inter-SemiBold.ttf"), 112)
    regular = ImageFont.truetype(str(font_dir / "Inter-Regular.ttf"), 34)
    small = ImageFont.truetype(str(font_dir / "Inter-SemiBold.ttf"), 24)
    left = 64 + logo_size + 48
    draw.text((left, 186), "STAR WARS JEDI KNIGHT: JEDI ACADEMY", font=small, fill=(168, 207, 255))
    draw.text((left - 4, 222), "Sol JK", font=semibold, fill=(255, 217, 122))
    for row, line in enumerate(("Native Rust client and server", "for Jedi Academy multiplayer,", "classic at heart.")):
        draw.text((left, 372 + row * 46), line, font=regular, fill=(226, 233, 246))
    return image.convert("RGB")


def save_png(image, path):
    path.parent.mkdir(parents=True, exist_ok=True)
    image.save(path, optimize=True)
    print(f"{path}: {image.size[0]}x{image.size[1]}, {path.stat().st_size} bytes")


def main():
    if len(sys.argv) not in (2, 3):
        sys.exit(__doc__)
    cutout = Image.open(sys.argv[1]).convert("RGBA")
    root = Path(sys.argv[2] if len(sys.argv) == 3 else ".")
    branding = root / "assets" / "branding"
    site = root / "site" / "assets"
    fonts = root / "crates" / "sjk-viewer" / "assets" / "fonts"
    if cutout.width != cutout.height:
        sys.exit("the cut-out must be square (scripts/logo_alpha.py makes it so)")

    save_png(resize(cutout, 1024), branding / "sjk-logo.png")
    save_png(resize(cutout, 512), branding / "sjk-logo-512.png")

    rgba = np.asarray(cutout).astype(np.float32) / 255.0
    save_png(glow_layer(rgba, core_mask(rgba), 0.55, 0.006, 1.6), branding / "emblem-core.png")
    save_png(glow_layer(rgba, lights_mask(rgba), 0.8, 0.004, 2.4), branding / "emblem-lights.png")

    small = medallion(cutout)
    save_png(icon(cutout, small, 32), branding / "icon-32.png")
    save_png(icon(cutout, small, 64), branding / "icon-64.png")
    frames = [icon(cutout, small, size) for size in ICO_SIZES]
    ico = branding / "sjk.ico"
    frames[-1].save(ico, format="ICO", sizes=[f.size for f in frames], append_images=frames[:-1])
    print(f"{ico}: {', '.join(str(size) for size in ICO_SIZES)}, {ico.stat().st_size} bytes")

    save_png(resize(cutout, 512), site / "sjk-logo-512.png")
    save_png(icon(cutout, small, 32), site / "favicon-32.png")
    favicon = site / "favicon.ico"
    favicon_frames = [icon(cutout, small, size) for size in (16, 32, 48)]
    favicon_frames[-1].save(
        favicon, format="ICO", sizes=[f.size for f in favicon_frames], append_images=favicon_frames[:-1]
    )
    print(f"{favicon}: 16, 32, 48, {favicon.stat().st_size} bytes")
    save_png(touch_icon(cutout), site / "apple-touch-icon.png")
    preview = social_preview(cutout, fonts)
    preview_path = site / "social-preview.jpg"
    preview.save(preview_path, quality=88, optimize=True, progressive=True)
    print(f"{preview_path}: 1200x630, {preview_path.stat().st_size} bytes")


if __name__ == "__main__":
    main()
