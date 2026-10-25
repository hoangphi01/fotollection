# SPDX-License-Identifier: GPL-3.0-only
"""Small-size icons from the simplified logo (assets/logo-small.png).

The detailed artwork (assets/logo-square.png) is unreadable below ~64 px, so the sidebar mark,
the favicon and every icon layer up to 64 px use the simplified tile instead. Run this after
`npx tauri icon assets/logo-square.png`, which regenerates all sizes from the detailed logo.

    uv run scripts/small-icons.py       # Pillow from pyproject.toml (dev tool only, never shipped); macOS for .icns
"""

import pathlib
import subprocess
import tempfile

from PIL import Image

ROOT = pathlib.Path(__file__).resolve().parent.parent
ICONS = ROOT / "src-tauri" / "icons"
# Opaque tile inside logo-small.png; the soft drop shadow is left out, it only muddies tiny sizes.
TILE = (18, 16, 426, 425)


def main() -> None:
    tile = Image.open(ROOT / "assets" / "logo-small.png").convert("RGBA").crop(TILE)
    side = max(tile.size)
    src = Image.new("RGBA", (side, side), (0, 0, 0, 0))
    src.paste(tile, ((side - tile.width) // 2, (side - tile.height) // 2))

    def sized(n: int) -> Image.Image:
        return src.resize((n, n), Image.LANCZOS)

    # UI: the sidebar mark at 1x/2x/3x, and the webview favicon.
    for n in (28, 56, 84):
        sized(n).save(ROOT / "ui" / "public" / "brand" / f"mark-{n}.png", optimize=True)
    sized(64).save(ROOT / "ui" / "public" / "favicon.png", optimize=True)

    # App icons up to 64 px.
    for name, n in [("32x32.png", 32), ("64x64.png", 64), ("Square30x30Logo.png", 30), ("Square44x44Logo.png", 44)]:
        sized(n).save(ICONS / name, optimize=True)

    # Windows .ico: small layers from the simplified tile, 256 px from the detailed icon.
    ico = Image.open(ICONS / "icon.ico")
    ico.size = (256, 256)
    big = ico.convert("RGBA")
    big.save(
        ICONS / "icon.ico",
        sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (256, 256)],
        append_images=[sized(n) for n in (16, 24, 32, 48, 64)],
    )

    # macOS .icns: replace the 16 and 32 pt layers (16, 32 and 64 px).
    with tempfile.TemporaryDirectory() as tmp:
        iconset = pathlib.Path(tmp) / "icon.iconset"
        subprocess.run(["iconutil", "-c", "iconset", ICONS / "icon.icns", "-o", iconset], check=True)
        for name, n in [("icon_16x16.png", 16), ("icon_16x16@2x.png", 32), ("icon_32x32.png", 32), ("icon_32x32@2x.png", 64)]:
            sized(n).save(iconset / name)
        subprocess.run(["iconutil", "-c", "icns", iconset, "-o", ICONS / "icon.icns"], check=True)


if __name__ == "__main__":
    main()
