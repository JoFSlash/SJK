"""Pack Twemoji's 72x72 PNGs into the client's emoji pack.

    python scripts/twemoji_pack.py <twemoji checkout>

The checkout is a clone of https://github.com/jdecked/twemoji at a release tag; only
`assets/72x72` is read, so a sparse clone is enough:

    git clone --depth 1 --branch v17.0.3 --filter=blob:none --sparse \\
        https://github.com/jdecked/twemoji.git twemoji
    git -C twemoji sparse-checkout set assets/72x72

It writes, under crates/sjk-viewer/assets/emoji/:

- twemoji.bin: the PNG files one after another, unchanged;
- twemoji.txt: one line per emoji, "<key> <offset> <length>", where the key is the
  emoji's code points in lower-case hex joined by '-' with every U+FE0F (variation
  selector 16) left out, the form `text/emoji.rs` matches text with;
- twemoji-LICENSE-GRAPHICS.txt: the graphics' CC-BY 4.0 licence.

and crates/sjk-identity/src/emoji.rs from scripts/emoji.rs.template: the code points
of U+2000 and above the pictures are made of (the emoji marks left out), which the bio
and chat rules allow. The hub keeps that file word for word as its src/emoji.rs: copy it
there when it changes.

The graphics are Twemoji's, by Twitter, Inc and other contributors, under CC-BY 4.0;
CREDITS.md and credits.txt credit them.
"""

import json
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "crates" / "sjk-viewer" / "assets" / "emoji"
EMOJI_RS = ROOT / "crates" / "sjk-identity" / "src" / "emoji.rs"
TEMPLATE = ROOT / "scripts" / "emoji.rs.template"


def is_mark(code: int) -> bool:
    return code in (0x200D, 0xFE0F, 0x20E3) or 0x1F3FB <= code <= 0x1F3FF or 0xE0020 <= code <= 0xE007F


def write_emoji_rs(keys: list[str], version: str) -> None:
    codes = sorted(
        {int(part, 16) for key in keys for part in key.split("-")}
        - {code for code in range(0x2000)}
    )
    codes = [code for code in codes if not is_mark(code)]
    ranges: list[list[int]] = []
    for code in codes:
        if ranges and code == ranges[-1][1] + 1:
            ranges[-1][1] = code
        else:
            ranges.append([code, code])
    cells = [f"(0x{first:X}, 0x{last:X})," for first, last in ranges]
    lines = ["    " + " ".join(cells[at:at + 4]) for at in range(0, len(cells), 4)]
    text = TEMPLATE.read_text(encoding="utf-8")
    text = text.replace("@VERSION@", version).replace("@RANGES@", "\n".join(lines))
    EMOJI_RS.write_text(text, encoding="utf-8", newline="\n")


def key_of(stem: str) -> str:
    return "-".join(part for part in stem.lower().split("-") if part != "fe0f")


def main() -> None:
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    checkout = Path(sys.argv[1])
    pngs = sorted((checkout / "assets" / "72x72").glob("*.png"))
    if not pngs:
        sys.exit(f"no PNGs in {checkout / 'assets' / '72x72'}")
    chosen: dict[str, Path] = {}
    for png in pngs:
        key = key_of(png.stem)
        # Two files for one key differ only by U+FE0F; keep the one without it.
        if key not in chosen or "fe0f" not in png.stem.lower():
            chosen[key] = png
    OUT.mkdir(parents=True, exist_ok=True)
    index = []
    offset = 0
    with open(OUT / "twemoji.bin", "wb") as blob:
        for key in sorted(chosen):
            data = chosen[key].read_bytes()
            blob.write(data)
            index.append(f"{key} {offset} {len(data)}\n")
            offset += len(data)
    (OUT / "twemoji.txt").write_text("".join(index), encoding="ascii", newline="\n")
    shutil.copyfile(checkout / "LICENSE-GRAPHICS", OUT / "twemoji-LICENSE-GRAPHICS.txt")
    version = json.loads((checkout / "package.json").read_text(encoding="utf-8"))["version"]
    write_emoji_rs(sorted(chosen), version)
    print(f"{len(chosen)} emoji ({len(pngs)} files), {offset} bytes")


if __name__ == "__main__":
    main()
