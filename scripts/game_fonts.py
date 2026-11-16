#!/usr/bin/env python3
"""Build SJK's vector replacements for Jedi Academy's bitmap game fonts.

    python scripts/game_fonts.py --base <GameData/base> --hd <HD fonts pk3> --ocr-a <OCRA.ttf>

Writes three TrueType fonts to crates/sjk-viewer/assets/fonts (docs/sjk.md "Fonts"):

- SJKMenu.ttf replaces `ergoec` (menus, centre prints, scoreboard names) and
  SJKHud.ttf `arialnb` (the classic status HUD): each glyph of an HD replacement
  atlas (JoF_HDFonts&Icons.pk3) is upscaled, lightly blurred, thresholded at half
  coverage and traced with potrace.
- SJKChat.ttf replaces `ocr_a` (chat, selection names, scoreboard numbers): OCR-A
  (John Sauter's public-domain digitisation, Debian's fonts-ocr-a) set to the retail
  font's character widths. OCR-A lacks most of Latin-1, so accented letters and
  symbols are composed from its own parts in its own manner (an accented capital
  is squashed to make room, as its Ñ is); the few with no parts to borrow (TRACED)
  come from the traced HD `ocr_a`.

The retail `.fontdat` metrics come from assets1.pk3 and every font keeps them: a
glyph has the retail advance and sits where the retail glyph sat, at UNITS font
units per retail pixel, so text lays out exactly as it did with the bitmaps. Byte
0xAC is the "WSI fonts" logo in the menu and chat fonts and a not sign in the HUD font,
as in retail.

Nothing from the game data is committed: the inputs are read from the player's
install. Needs Python 3 with Pillow, numpy, fonttools, potracer and skia-pathops.
"""
import argparse
import io
import math
import struct
import zipfile
from pathlib import Path

import numpy as np
import potrace
from fontTools.fontBuilder import FontBuilder
from fontTools.pens.boundsPen import BoundsPen
from fontTools.pens.cu2quPen import Cu2QuPen
from fontTools.pens.recordingPen import RecordingPen
from fontTools.pens.reverseContourPen import ReverseContourPen
from fontTools.pens.transformPen import TransformPen
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.ttLib import TTFont
from fontTools.ttLib.removeOverlaps import removeOverlaps
from PIL import Image, ImageFilter

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "crates" / "sjk-viewer" / "assets" / "fonts"
UNITS = 64  # font units per retail pixel
TRACE_PX = 16  # trace pixels per retail pixel
PAD = 2  # retail pixels of empty border around a traced glyph
# Smoothing before the trace: Gaussian blur in retail pixels, then potrace's
# corner threshold and curve-merging tolerance ("Smooth 1", Sol's pick).
BLUR, ALPHAMAX, OPTTOLERANCE = 0.12, 1.15, 0.5
LOGO = 0xAC
TRACED = "ßþÞð§¶¤"
VERSION = "Version 1.000"


# --- retail metrics ----------------------------------------------------------


def read_fontdat(data):
    """Glyph records by byte and the header, in retail pixels."""
    glyphs = {}
    for byte in range(256):
        w, h, adv, off, base, s, t, s2, t2 = struct.unpack_from("<hhhhiffff", data, byte * 28)
        glyphs[byte] = dict(w=w, h=h, adv=adv, off=off, base=base, uv=(s, t, s2, t2))
    point, height, asc, desc = struct.unpack_from("<hhhh", data, 7168)
    return glyphs, dict(point=point, height=height, asc=asc, desc=desc)


def line_metrics(glyphs, header):
    """Ascent and descent in retail pixels; arialnb's header is empty."""
    if header["asc"] or header["desc"]:
        return header["asc"], header["desc"]
    inked = [g for g in glyphs.values() if g["w"] and g["h"]]
    return max(g["base"] for g in inked), max(g["h"] - g["base"] for g in inked)


def drawn(byte, glyph):
    """Whether the retail font draws a glyph of its own for `byte`."""
    # No-break space draws as a space.
    return glyph["w"] > 0 and glyph["h"] > 0 and byte > 0x20 and not 0x7F <= byte <= 0xA0


# --- outlines: a glyph is a RecordingPen value list -------------------------


def replay(rec, pen):
    for op, args in rec:
        getattr(pen, op)(*args)


def transform(rec, matrix):
    out = RecordingPen()
    replay(rec, TransformPen(out, matrix))
    return list(out.value)


def move(rec, dx=0, dy=0):
    return transform(rec, (1, 0, 0, 1, dx, dy))


def scale(rec, sx, sy=None, cx=0, cy=0):
    sy = sx if sy is None else sy
    return transform(rec, (sx, 0, 0, sy, cx - sx * cx, cy - sy * cy))


def rotate(rec, degrees, cx, cy):
    a = math.radians(degrees)
    c, s = math.cos(a), math.sin(a)
    return transform(rec, (c, s, -s, c, cx - c * cx + s * cy, cy - s * cx - c * cy))


def mirror(rec):
    b = bounds(rec)
    return transform(rec, (-1, 0, 0, 1, b[0] + b[2], 0))


def bounds(rec):
    pen = BoundsPen(None)
    replay(rec, pen)
    return pen.bounds or (0, 0, 0, 0)


def centre_x(rec):
    b = bounds(rec)
    return (b[0] + b[2]) / 2


def contours(rec):
    out, current = [], []
    for item in rec:
        current.append(item)
        if item[0] in ("closePath", "endPath"):
            out.append(current)
            current = []
    return out


def join(*recs):
    return [item for rec in recs for item in rec]


def poly(*points):
    return [("moveTo", (points[0],))] + [("lineTo", (p,)) for p in points[1:]] + [("closePath", ())]


def rect(x0, y0, x1, y1):
    """Clockwise rectangle."""
    return poly((x0, y0), (x0, y1), (x1, y1), (x1, y0))


def clip_above(rec, y):
    import pathops

    shape, window = pathops.Path(), pathops.Path()
    replay(rec, shape.getPen())
    replay(rect(-10_000, y, 10_000, 10_000), window.getPen())
    out = RecordingPen()
    pathops.op(shape, window, pathops.PathOp.INTERSECTION).draw(out)
    return list(out.value)


def signed_area(rec):
    points = [args[-1] for op, args in rec if args]
    return sum(a[0] * b[1] - b[0] * a[1] for a, b in zip(points, points[1:] + points[:1])) / 2


# --- tracing -----------------------------------------------------------------


def trace_glyph(alpha, glyph, k):
    """Outline of one atlas glyph in font units. `alpha` is the atlas coverage
    (0-1) and `k` its pixels per retail pixel."""
    rows, cols = alpha.shape
    s, t, s2, t2 = glyph["uv"]
    crop = np.pad(alpha[round(t * rows):round(t2 * rows), round(s * cols):round(s2 * cols)], PAD * k)
    up = TRACE_PX / k
    image = Image.fromarray((crop * 255).astype(np.uint8))
    image = image.resize((round(image.width * up), round(image.height * up)), Image.BICUBIC)
    image = image.filter(ImageFilter.GaussianBlur(BLUR * TRACE_PX))
    ink = np.array(image) >= 128
    if not ink.any():
        return []
    # potracer traces dark pixels, so it gets the background.
    path = potrace.Bitmap(~ink).trace(
        turdsize=int(up * up * 0.3), alphamax=ALPHAMAX, opticurve=True, opttolerance=OPTTOLERANCE
    )

    def font_point(point):
        x = point.x / TRACE_PX - PAD
        y = point.y / TRACE_PX - PAD
        return ((glyph["off"] + x) * UNITS, (glyph["base"] - y) * UNITS)

    out = RecordingPen()
    for curve in path:
        out.moveTo(font_point(curve.start_point))
        for segment in curve.segments:
            if segment.is_corner:
                out.lineTo(font_point(segment.c))
                out.lineTo(font_point(segment.end_point))
            else:
                out.curveTo(font_point(segment.c1), font_point(segment.c2), font_point(segment.end_point))
        out.closePath()
    rec = list(out.value)
    # TrueType wants outer contours clockwise; potrace alternates for holes, so
    # turn the whole glyph around when its largest contour runs the other way.
    largest = max(contours(rec), key=lambda c: abs(signed_area(c)))
    if signed_area(largest) > 0:
        reversed_ = RecordingPen()
        replay(rec, ReverseContourPen(reversed_))
        rec = list(reversed_.value)
    return rec


def trace_font(glyphs, atlas):
    """Traced outlines by byte of every glyph the retail font draws."""
    alpha = np.asarray(atlas.convert("RGBA"))[..., 3].astype(np.float32) / 255
    widest = max(glyphs.values(), key=lambda g: g["w"])
    k = max(1, round((widest["uv"][2] - widest["uv"][0]) * atlas.width / widest["w"]))
    return {byte: trace_glyph(alpha, g, k) for byte, g in glyphs.items() if drawn(byte, g)}


# --- OCR-A -------------------------------------------------------------------


class OcrA:
    """OCR-A outlines in its own units (UPM 1000), with Latin-1 composed."""

    def __init__(self, path, traced):
        self.font = TTFont(path)
        self.cmap = self.font.getBestCmap()
        self.glyph_set = self.font.getGlyphSet()
        self.traced = traced
        self.cap = bounds(self.of("H"))[3]
        self.x_height = bounds(self.of("x"))[3]
        bar = bounds(self.of("|"))
        self.stem = bar[2] - bar[0]
        self.descender = bounds(self.of("p"))[1]
        # Marks sit in the band of Ñ's tilde, over a capital squashed to Ñ's body.
        self.squashed, self.tilde = self.split_marks("Ñ")
        self.band = (bounds(self.tilde)[1], self.cap)
        a_body, dots = self.split_marks("Ä")
        self.dieresis = self.to_band(dots)
        self.ring = self.to_band(clip_above(self.of("Å"), a_body + 12))
        t = self.stem * 1.05
        self.acute = self.slant(t * 1.4, t)
        self.grave = mirror(self.acute)
        half = self.slant(t * 1.25, t)
        self.circumflex = join(half, move(mirror(half), t * 1.25, 0))
        self.dotless_i = join(*sorted(contours(self.of("i")), key=lambda c: bounds(c)[1])[:-1])

    def of(self, ch):
        out = RecordingPen()
        self.glyph_set[self.cmap[ord(ch)]].draw(out)
        return list(out.value)

    def split_marks(self, ch):
        """(body height, mark contours) of one of OCR-A's accented capitals."""
        parts = contours(self.of(ch))
        body = min(parts, key=lambda c: bounds(c)[1])
        top = bounds(body)[3]
        return top, join(*[c for c in parts if c is not body and bounds(c)[3] > top])

    def to_band(self, rec):
        lo, hi = self.band
        b = bounds(rec)
        sy = (hi - lo) / (b[3] - b[1])
        return transform(rec, (1, 0, 0, sy, 0, lo - b[1] * sy))

    def slant(self, run, thickness):
        """A stroke rising to the right across the mark band."""
        lo, hi = self.band
        return poly((0, lo), (run, hi), (run + thickness, hi), (thickness, lo))

    def accented(self, base_ch, mark):
        base = self.dotless_i if base_ch == "i" else self.of(base_ch)
        if base_ch.isupper():
            base = scale(base, 1, self.squashed / self.cap)
        b = bounds(mark)
        return join(base, move(mark, centre_x(base) - (b[0] + b[2]) / 2, 0))

    def cedilla(self, base_ch):
        base = self.of(base_ch)
        hook = self.of("¸")
        b = bounds(hook)
        hook = scale(hook, 0.8, 0.6, (b[0] + b[2]) / 2, b[3])
        b = bounds(hook)
        return join(base, move(hook, centre_x(base) - (b[0] + b[2]) / 2, self.stem * 0.6 - b[3]))

    def raised(self, ch, factor):
        r = scale(self.of(ch), factor)
        b = bounds(r)
        return move(r, -b[0], self.cap - b[3])

    def upside_down(self, ch):
        r = self.of(ch)
        b = bounds(r)
        r = rotate(r, 180, (b[0] + b[2]) / 2, (b[1] + b[3]) / 2)
        return move(r, 0, self.x_height - bounds(r)[3])

    def composed(self):
        g = {}
        marks = {"`": self.grave, "´": self.acute, "^": self.circumflex, "~": self.tilde,
                 "¨": self.dieresis, "°": self.ring}
        for ch, (base, mark) in {
            "À": "A`", "Á": "A´", "Â": "A^", "Ã": "A~", "È": "E`", "É": "E´", "Ê": "E^",
            "Ë": "E¨", "Ì": "I`", "Í": "I´", "Î": "I^", "Ï": "I¨", "Ò": "O`", "Ó": "O´",
            "Ô": "O^", "Õ": "O~", "Ö": "O¨", "Ù": "U`", "Ú": "U´", "Û": "U^", "Ü": "U¨",
            "Ý": "Y´", "à": "a`", "á": "a´", "â": "a^", "ã": "a~", "ä": "a¨", "å": "a°",
            "è": "e`", "é": "e´", "ê": "e^", "ë": "e¨", "ì": "i`", "í": "i´", "î": "i^",
            "ï": "i¨", "ñ": "n~", "ò": "o`", "ó": "o´", "ô": "o^", "õ": "o~", "ù": "u`",
            "ú": "u´", "û": "u^", "ý": "y´", "ÿ": "y¨",
        }.items():
            g[ch] = self.accented(base, marks[mark])
        g["Ç"], g["ç"] = self.cedilla("C"), self.cedilla("c")
        g["¡"] = self.upside_down("!")
        # OCR-A's own ¿ is an upright ?.
        g["¿"] = self.upside_down("?")
        cap, xh, stem = self.cap, self.x_height, self.stem
        c = self.of("c")
        cb = bounds(c)
        mid = centre_x(c)
        g["¢"] = join(c, rect(mid - stem / 2, cb[1] - 0.18 * xh, mid + stem / 2, cb[3] + 0.18 * xh))
        for ch, arrow in (("«", "<"), ("»", ">")):
            r = self.of(arrow)
            b = bounds(r)
            r = scale(r, 0.6, 0.6, (b[0] + b[2]) / 2, (b[1] + b[3]) / 2)
            w = bounds(r)[2] - bounds(r)[0]
            g[ch] = join(move(r, -w * 0.32), move(r, w * 0.32))
        under = self.of("_")
        ub = bounds(under)
        g["¯"] = move(under, 0, cap - ub[3])
        g["°"] = self.raised("o", 0.5)
        short_bar = move(scale(under, 0.55, 1, (ub[0] + ub[2]) / 2, ub[1]), 0, cap * 0.33 - ub[1])
        for ch, letter in (("ª", "a"), ("º", "o")):
            r = self.raised(letter, 0.55)
            g[ch] = join(r, move(short_bar, centre_x(r) - centre_x(short_bar), 0))
        plus = self.of("+")
        pb = bounds(plus)
        g["±"] = join(move(plus, 0, cap * 0.18), rect(pb[0], 0, pb[2], stem))
        for ch, digit in (("¹", "1"), ("²", "2"), ("³", "3")):
            g[ch] = self.raised(digit, 0.58)
        slash = scale(self.of("/"), 0.85, 0.85, centre_x(self.of("/")), cap / 2)
        for ch, (top, bottom) in (("¼", "14"), ("½", "12"), ("¾", "34")):
            numerator, denominator = scale(self.of(top), 0.45), scale(self.of(bottom), 0.45)
            nb, db = bounds(numerator), bounds(denominator)
            g[ch] = join(move(numerator, 20 - nb[0], cap - nb[3]), slash,
                         move(denominator, 640 - db[2], -db[1]))
        g["×"] = rotate(plus, 45, (pb[0] + pb[2]) / 2, (pb[1] + pb[3]) / 2)
        minus = self.of("-")
        mb = bounds(minus)
        dot = self.of("·")
        db = bounds(dot)
        dot = move(dot, centre_x(minus) - centre_x(dot), (mb[1] + mb[3]) / 2 - (db[1] + db[3]) / 2)
        gap = (db[3] - db[1]) * 1.9
        g["÷"] = join(minus, move(dot, 0, gap), move(dot, 0, -gap))
        g["¨"] = self.dieresis
        hb = bounds(self.of("H"))
        frame_w = stem * 0.8
        # Outer square clockwise, inner one counter-clockwise: a frame.
        frame = join(rect(hb[0], 0, hb[2], cap),
                     poly((hb[0] + frame_w, frame_w), (hb[2] - frame_w, frame_w),
                          (hb[2] - frame_w, cap - frame_w), (hb[0] + frame_w, cap - frame_w)))
        for ch, inner in (("©", "C"), ("®", "R")):
            r = scale(self.of(inner), 0.5)
            b = bounds(r)
            g[ch] = join(frame, move(r, (hb[0] + hb[2]) / 2 - (b[0] + b[2]) / 2, cap / 2 - (b[1] + b[3]) / 2))
        u = self.of("u")
        g["µ"] = join(u, rect(bounds(u)[0], self.descender, bounds(u)[0] + stem, xh * 0.5))
        d = self.of("D")
        g["Ð"] = join(d, rect(bounds(d)[0] - stem * 1.2, cap / 2 - stem / 2, bounds(d)[0] + stem * 2.4, cap / 2 + stem / 2))
        sl = self.of("/")
        g["ø"] = join(self.of("o"), scale(sl, 0.75, xh * 1.2 / (bounds(sl)[3] - bounds(sl)[1]), centre_x(sl), 0))
        a, e = self.of("a"), self.of("e")
        ab, eb = bounds(a), bounds(e)
        g["æ"] = join(scale(a, 0.55, 1, ab[0], 0),
                      move(scale(e, 0.55, 1, eb[0], 0), ab[0] - eb[0] + (ab[2] - ab[0]) * 0.55 - stem * 0.8, 0))
        bar = bounds(self.of("|"))
        g["|"] = rect(bar[0], bar[1], bar[2], bar[3])
        return g

    def outlines(self, glyphs, units_per_pixel):
        """OCR-A outlines by byte for the retail chat font's glyphs, in OCR-A
        units, each centred in the retail advance. The retail ink sat about
        there too, but its atlas cells are padded unevenly (C's is 2 pixels
        left of its neighbours'), so a cell would place the even OCR-A ink
        unevenly."""
        composed = self.composed()
        traced_cap = bounds(self.traced[ord("H")])[3]
        out = {}
        for byte, g in glyphs.items():
            if not drawn(byte, g) or byte == LOGO:
                continue
            ch = bytes([byte]).decode("latin-1")
            if ch in TRACED:
                rec = scale(self.traced[byte], self.cap / traced_cap)
            elif ch in composed:
                rec = composed[ch]
            elif ord(ch) in self.cmap:
                rec = self.of(ch)
            else:
                raise SystemExit(f"OCR-A has no {ch!r} and nothing composes it")
            advance = g["adv"] * units_per_pixel
            b = bounds(rec)
            ink = b[2] - b[0]
            centre = advance / 2
            margin = 0.06 * advance
            left = min(max(centre - ink / 2, margin), max(margin, advance - margin - ink))
            out[byte] = move(rec, left - b[0], 0)
        return out


# --- output ------------------------------------------------------------------


def logo_for(logo, glyph):
    """The traced logo fitted to another font's logo record."""
    b = bounds(logo)
    s = glyph["h"] * UNITS / (b[3] - b[1])
    r = scale(logo, s)
    b = bounds(r)
    left = (glyph["off"] + glyph["w"] / 2) * UNITS - (b[2] - b[0]) / 2
    return move(r, left - b[0], glyph["base"] * UNITS - b[3])


def write_font(path, family, notice, outlines, glyphs, ascent, descent):
    """Write `outlines` (font units, by byte) with the retail advances."""
    names = [".notdef", "space"] + [f"uni{byte:04X}" for byte in sorted(outlines)]
    builder = FontBuilder((ascent + descent) * UNITS, isTTF=True)
    builder.setupGlyphOrder(names)
    builder.setupCharacterMap({0x20: "space", 0xA0: "space", **{b: f"uni{b:04X}" for b in outlines}})
    shapes = {".notdef": TTGlyphPen(None).glyph(), "space": TTGlyphPen(None).glyph()}
    advances = {".notdef": glyphs[ord("0")]["adv"] * UNITS, "space": glyphs[0x20]["adv"] * UNITS}
    for byte, rec in outlines.items():
        pen = TTGlyphPen(None)
        replay(rec, Cu2QuPen(pen, max_err=UNITS / 16))
        shapes[f"uni{byte:04X}"] = pen.glyph()
        advances[f"uni{byte:04X}"] = glyphs[byte]["adv"] * UNITS
    builder.setupGlyf(shapes)
    builder.setupHorizontalMetrics({name: (round(advances[name]), 0) for name in names})
    builder.setupHorizontalHeader(ascent=ascent * UNITS, descent=-descent * UNITS)
    builder.setupNameTable({
        "familyName": family, "styleName": "Regular", "fullName": family,
        "psName": family.replace(" ", "") + "-Regular", "version": VERSION,
        "uniqueFontIdentifier": f"SJK: {family}", "copyright": notice,
    })
    builder.setupOS2(sTypoAscender=ascent * UNITS, sTypoDescender=-descent * UNITS,
                     sTypoLineGap=0, usWinAscent=ascent * UNITS, usWinDescent=descent * UNITS)
    builder.setupPost()
    font = builder.font
    glyf, hmtx = font["glyf"], font["hmtx"]

    def set_bearings():
        for name in names:
            glyph = glyf[name]
            glyph.recalcBounds(glyf)
            hmtx[name] = (hmtx[name][0], getattr(glyph, "xMin", 0))

    # removeOverlaps draws through the glyph set, which moves each outline to
    # its hmtx left bearing, so the bearings must match the outlines first or
    # every glyph it rewrites lands flush left.
    set_bearings()
    removeOverlaps(font)
    set_bearings()
    font.save(path)
    print(f"{path.relative_to(ROOT)}: {len(outlines)} glyphs")


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--base", type=Path, required=True, help="Jedi Academy GameData/base")
    parser.add_argument("--hd", type=Path, required=True, help="PK3 with HD fonts/*.tga atlases")
    parser.add_argument("--ocr-a", type=Path, required=True, help="Sauter's OCRA.ttf")
    args = parser.parse_args()
    retail = zipfile.ZipFile(args.base / "assets1.pk3")
    hd = zipfile.ZipFile(args.hd)

    def font(name):
        glyphs, header = read_fontdat(retail.read(f"fonts/{name}.fontdat"))
        atlas = Image.open(io.BytesIO(hd.read(f"fonts/{name}.tga")))
        return glyphs, line_metrics(glyphs, header), trace_font(glyphs, atlas)

    hd_notice = ("Traced from the {} atlas of the JoF HD Fonts & Icons pack (author unknown), "
                 "a redraw of Raven Software's Jedi Academy font; see SJK-fonts.txt.")
    menu, (asc, desc), menu_outlines = font("ergoec")
    write_font(OUT / "SJKMenu.ttf", "SJK Menu", hd_notice.format("ergoec"), menu_outlines, menu, asc, desc)
    hud, (asc, desc), hud_outlines = font("arialnb")
    write_font(OUT / "SJKHud.ttf", "SJK HUD", hd_notice.format("arialnb"), hud_outlines, hud, asc, desc)

    chat, (asc, desc), chat_traced = font("ocr_a")
    ocr = OcrA(args.ocr_a, chat_traced)
    # OCR-A units per retail pixel, from the HD atlas's ink cap height.
    units_per_pixel = ocr.cap / (bounds(chat_traced[ord("H")])[3] / UNITS)
    outlines = {byte: scale(rec, UNITS / units_per_pixel)
                for byte, rec in ocr.outlines(chat, units_per_pixel).items()}
    outlines[LOGO] = logo_for(menu_outlines[LOGO], chat[LOGO])
    write_font(OUT / "SJKChat.ttf", "SJK Chat",
               "OCR-A by John Sauter (public domain), set to the widths of Jedi Academy's ocr_a; "
               "seven glyphs and the logo traced from the JoF HD Fonts & Icons pack; see SJK-fonts.txt.",
               outlines, chat, asc, desc)


if __name__ == "__main__":
    main()
