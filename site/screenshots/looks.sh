#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# The landing page's interactive Looks section (site/_includes/looks.html): one of the owner's
# Prague photos, with his consent, through each of Fotollection's built-in looks, rendered by
# the app's own engine (crates/core/examples/render_looks.rs), never retouched elsewhere.
#
#   site/screenshots/looks.sh <owner-folder>
#
# Writes site/assets/img/looks/<slug>.webp, 1200 × 1600. The original photo stays outside
# the repository; only the renders are committed (© the owner, the `fn_trip` note).
set -euo pipefail
cd "$(dirname "$0")/../.."

[ $# -eq 1 ] || { echo "usage: site/screenshots/looks.sh <owner-folder>" >&2; exit 2; }
photo="$1/P1067865.JPG"
[ -f "$photo" ] || { echo "looks.sh: $photo not found" >&2; exit 2; }
command -v cwebp >/dev/null || { echo "looks.sh: needs cwebp (libwebp)" >&2; exit 2; }

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
# The camera's orientation applied first: the engine renders pixels as they are stored.
uv run python -I -c 'import sys
from PIL import Image, ImageOps
im = ImageOps.exif_transpose(Image.open(sys.argv[1])).convert("RGB")
im.thumbnail((2400, 2400))
im.save(sys.argv[2], quality=98)' "$photo" "$work/upright.jpg"
CARGO_BUILD_JOBS=4 cargo run --release -q -p fotollection-core --example render_looks -- "$work/upright.jpg" "$work/out" 1600
mkdir -p site/assets/img/looks
for jpg in "$work"/out/*.jpg; do
  cwebp -quiet -q 80 "$jpg" -o "site/assets/img/looks/$(basename "$jpg" .jpg).webp"
done
du -ch site/assets/img/looks/*.webp | tail -1
