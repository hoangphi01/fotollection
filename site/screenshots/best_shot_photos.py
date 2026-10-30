# SPDX-License-Identifier: GPL-3.0-only
"""The cat burst for the website's print-basket and best-shot screenshots (site/screenshots/README.md).

    uv run python -I site/screenshots/best_shot_photos.py <photo.jpg> <out-folder>

Makes a burst from one photo: the photo itself, then a soft, a shaken and a dark copy, one
second apart like a camera's burst, with its EXIF kept. The app then groups them and
suggests the sharp one with its real reasons. The photo is the owner's own (a cat, Lumix
GX9, by @hoangphi01) and is not kept in the repository.
"""

import sys
from datetime import datetime, timedelta
from pathlib import Path

from PIL import Image, ImageChops, ImageEnhance, ImageFilter

EXIF_IFD = 0x8769
DATE_TIME_ORIGINAL = 0x9003
DATE_FORMAT = "%Y:%m:%d %H:%M:%S"


def shaken(image: Image.Image, length: int = 30) -> Image.Image:
    """A sideways camera shake: the photo averaged over `length` pixels of movement."""
    result = image
    for step in range(1, length):
        result = Image.blend(result, ImageChops.offset(image, step, 0), 1 / (step + 1))
    return result


def main() -> None:
    source, out = Path(sys.argv[1]), Path(sys.argv[2])
    out.mkdir(parents=True, exist_ok=True)
    photo = Image.open(source)
    exif = photo.getexif()
    taken = datetime.strptime(exif.get_ifd(EXIF_IFD)[DATE_TIME_ORIGINAL], DATE_FORMAT)
    # P1057694 → P1057695…: the copies carry on the camera's numbering.
    digits = len(source.stem) - len(source.stem.rstrip("0123456789"))
    prefix, number = source.stem[:-digits], int(source.stem[-digits:])

    rgb = photo.convert("RGB")
    burst = [rgb, rgb.filter(ImageFilter.GaussianBlur(4)), shaken(rgb), ImageEnhance.Brightness(rgb).enhance(0.45)]
    for i, image in enumerate(burst):
        exif.get_ifd(EXIF_IFD)[DATE_TIME_ORIGINAL] = (taken + timedelta(seconds=i)).strftime(DATE_FORMAT)
        image.save(out / f"{prefix}{number + i}.JPG", quality=92, exif=exif.tobytes())
    print(f"{len(burst)} photos in {out}")


if __name__ == "__main__":
    main()
