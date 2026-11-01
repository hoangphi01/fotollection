# JPEG layout fixtures

One picture saved in each JPEG layout, for `tests/jpeg_layouts.rs` and `src/jpeg_layout.rs` ([ADR-0033](../../../../../docs/adr/0033-reference-jpeg-decoder.md)). Every decode is compared with `source.png`.

`source.png` is a 250 × 186 picture drawn by a script for this project: gradients, a sine pattern and solid patches, so colour errors show. The size is not a multiple of 16 on purpose. It holds no photo and no personal data.

The JPEGs were made with libjpeg-turbo 3.1 from `source.png` converted to PPM, at quality 90:

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

To add a layout, save `source.png` in it, add the file to the table and to the test's list, and allow it in `Layout::zune_proven` only once the test shows zune decodes it right.
