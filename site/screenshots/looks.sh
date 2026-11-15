#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# The landing page's interactive Looks section (site/_includes/looks.html): one photo through
# each of Fotollection's built-in looks, rendered by the app's own engine
# (crates/core/examples/render_looks.rs), never retouched elsewhere.
#
#   site/screenshots/looks.sh <owner-folder>/P1067865.JPG            # the Prague tram, every page
#   site/screenshots/looks.sh "<JAPAN DEMO>/IMG_5710.heic" ja        # a page's own (languages.yml `looks`)
#   site/screenshots/looks.sh "<9adox Korea 26>/DSCF1990.JPG" ko
#
# Writes site/assets/img/looks/<slug>[-<lang>].webp, 1200 × 1600, the photo cut to 3:4 about
# its centre. The originals stay outside the repository; only the renders are committed, with
# their author named in the page's `fn_photo` note.
set -euo pipefail
cd "$(dirname "$0")/../.."

[ $# -ge 1 ] && [ $# -le 2 ] || { echo "usage: site/screenshots/looks.sh <photo> [lang]" >&2; exit 2; }
photo="$1"
suffix=${2:+-$2}
[ -f "$photo" ] || { echo "looks.sh: $photo not found" >&2; exit 2; }
command -v cwebp >/dev/null || { echo "looks.sh: needs cwebp (libwebp)" >&2; exit 2; }

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
case "$photo" in
  # Pillow reads no HEIC (an iPhone's photos): macOS's own sips turns it into a JPEG first.
  *.heic|*.HEIC) sips -s format jpeg "$photo" --out "$work/source.jpg" >/dev/null; photo="$work/source.jpg" ;;
esac
# The camera's orientation applied first: the engine renders pixels as they are stored.
uv run python -I -c 'import sys
from PIL import Image, ImageOps
im = ImageOps.exif_transpose(Image.open(sys.argv[1])).convert("RGB")
im = ImageOps.fit(im, (1800, 2400))
im.save(sys.argv[2], quality=98)' "$photo" "$work/upright.jpg"
CARGO_BUILD_JOBS=4 cargo run --release -q -p fotollection-core --example render_looks -- "$work/upright.jpg" "$work/out" 1600
mkdir -p site/assets/img/looks
for jpg in "$work"/out/*.jpg; do
  cwebp -quiet -q 80 "$jpg" -o "site/assets/img/looks/$(basename "$jpg" .jpg)$suffix.webp"
done
du -ch site/assets/img/looks/*.webp | tail -1
