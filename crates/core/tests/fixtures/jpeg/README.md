# JPEG layout fixtures

One picture saved in each JPEG layout, for `tests/jpeg_layouts.rs` and `src/jpeg_layout.rs` ([ADR-0033](../../../../../docs/adr/0033-reference-jpeg-decoder.md)). Every decode is compared with `source.png`.

`source.png` is a 250 × 186 picture drawn by a script for this project: gradients, a sine pattern and solid patches, so colour errors show. The size is not a multiple of 16 on purpose. It holds no photo and no personal data.

The JPEGs were made with libjpeg-turbo 3.1 from `source.png` converted to PPM, at quality 90, unless the table says otherwise:

| File | `cjpeg` options | Layout |
|---|---|---|
| `baseline-444.jpg` | `-sample 1x1 -baseline` | baseline, no colour subsampling |
| `baseline-422.jpg` | `-sample 2x1 -baseline` | baseline 4:2:2 |
| `baseline-420.jpg` | `-sample 2x2 -baseline` | baseline 4:2:0 |
| `baseline-440.jpg` | `-sample 1x2` | baseline 4:4:0 (a rotated 4:2:2) |
| `baseline-411.jpg` | `-sample 4x1` | baseline 4:1:1 |
| `progressive-444.jpg`, `-422`, `-420` | `-progressive` with the samplings above | progressive |
| `restart-420.jpg` | `-sample 2x2 -restart 2` | restart markers |
| `optimized-420.jpg` | `-sample 2x2 -optimize` | optimised Huffman tables |
| `noninterleaved-420.jpg`, `-444` | `-scans noninterleaved.scans` | baseline, one scan per component: zune-jpeg 0.5.15 decodes these as green stripes |
| `grey.jpg` | `-grayscale` | one component |
| `restart-444.jpg`, `restart-422.jpg` | `-sample 1x1` / `-sample 2x1` `-restart 2` | restart markers on full and half colour |
| `progressive-noninterleaved-dc-420.jpg` | `-sample 2x2 -scans progressive-noninterleaved-dc.scans` | progressive, its DC scan one component at a time |
| `lossless-16-grey.jpg` | `-lossless 1 -precision 16` from a 16-bit PGM | lossless 16-bit grey (SOF3). The high byte is the picture's luma, the low byte unrelated to it (`(l*97+31) & 255`), so a decoder that keeps the wrong byte shows noise. A low byte equal to the high one (`l × 257`) hid exactly that bug |
| `lossless-rgb.jpg` | `-lossless 1` | lossless 8-bit colour (SOF3) |
| `cmyk.jpg` | Pillow 11 `convert('CMYK').save(quality=90)` | Adobe CMYK (APP14, inverted values), as Photoshop writes it |
| `arithmetic-420.jpg` | `-sample 2x2 -arithmetic` | arithmetic coding (SOF9): neither decoder reads it, so it must be refused |
| `truncated-420.jpg` | the first 60 % of `baseline-420.jpg` | a file cut short: must be refused (`ERR-IMG-0006`), never decoded as half a photo |

Not yet: YCCK (Adobe transform 2) and 8-bit extended (SOF1), which neither `cjpeg` nor Pillow writes; SOF1 stays off zune until a fixture proves it.

To add a layout, save `source.png` in it, add the file to the table and to the test's list with the route it must take (zune, the reference decoder, or refused), and allow it in `Layout::zune_proven` only once the test shows zune decodes it right. The test compares the mean and the 99th-percentile error per channel, and checks the route.
