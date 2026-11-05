### What Facebook does to a photo, and what we do first

Facebook shrinks every photo to at most 2,048 px, recompresses it at about quality 71 with halved colour, and ignores colour profiles. The Share export does that work properly before the upload, so Facebook has nothing left to shrink: the photo is converted to sRGB through its profile, resized with a Lanczos filter in linear light, lightly sharpened, and saved at quality 88. Each setting was chosen by measuring: stronger sharpening (0.35 instead of 0.2) scored lower, and quality 84 or 80 lost more than it saved.

| Median | Original size | Facebook, 2,048 px | Instagram, 1,080 px |
|---|---|---|---|
| File size (camera JPEGs of 3.08 MB) | 1.42 MB | 0.51 MB | 0.24 MB |
| SSIM against the ideal resize (1 is identical) | 0.981 | 0.979 | 0.982 |
| ΔE2000 | 0.96 | 1.27 | 1.58 |

After Facebook's own recompression, the result stays closer to the photo than the usual way (SSIM 0.965 against 0.961), and a Display P3 photo keeps 99.6 % of its colour instead of 86 %, because its profile was turned into real sRGB colours rather than dropped. The resized presets stay above ΔE 1 because of the halved colour every platform applies anyway; *Original size* keeps every pixel at about half the file, under ΔE 1.
