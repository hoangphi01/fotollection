# ADR-0037: Our own develop core: rawler unpacks, we demosaic and manage colour

- **Status:** Accepted (planned for v0.10.0; v0.11.0 since [ADR-0038](0038-moderate-colour-editor-on-one-lut-engine.md), after the looks); amended by [ADR-0040](0040-neutral-raw-colour-and-gpu-develop.md) (neutral colour by default)
- **Date:** 2026-11-02
- **Related:** FR-016, FR-071, FR-072, FR-070, NFR-002, NFR-009, NFR-017, ADR-0029, ADR-0030, ADR-0033, ADR-0036
- **Amended:** 2026-11-03: the DNG kinds met in the wild and their order ([docs/raw/dng.md](../raw/dng.md)), added to FR-072.
- **Amends:** [ADR-0029](0029-raw-through-rawler-then-our-own-decoder.md) and [ADR-0030](0030-our-own-raw-decoder.md)

## Context

The owner wants a core with its own thinking: simple software, never careless. The audit of 2026-11-02 found the image path assembled from parts we do not control or measure well:

- **Demosaicing:** our Malvar-He-Cutler (DNG only, linear, prone to zipper), rawler's PPG for Bayer and **bilinear for X-Trans**, and the system codecs as black boxes. There is no branch on the sensor kind; an unknown one reaches rawler's `todo!()`.
- **Colour:** fitted to the embedded JPEG with unexplained thresholds, and scored against that same JPEG, so the score is circular.
- **No artifact defence:** no green equilibration (the main cause of maze and worm patterns on Bayer), no false-colour suppression, no highlight reconstruction.
- **No colour management on screen:** previews ignore the ICC profile, so a P3 photo looks dull while its print is right.

After the green-stripe JPEG bug of v0.8.0 (ADR-0033), "a decoder can be silently wrong" applies to RAW as well.

## Options considered

1. **Keep the mix:** cheap, but the quality is whatever each library does, and X-Trans stays bilinear.
2. **Write every maker's decompressor:** full control, but years of work on formats that change with each camera.
3. **rawler (and our DNG reader) only unpack the sensor samples; from there every step is ours:** the formats stay someone else's problem, the image becomes ours.

## Decision

Option 3.

- **Unpacking:** our DNG reader, rawler for the other formats, then the system codecs as a last fallback. We keep only the CFA samples, levels, pattern and matrices from rawler, never its develop.
- **Sensor kind first:** an explicit `Cfa` (Bayer with its pattern, X-Trans 6 × 6, four-colour, linear, monochrome). Each kind has its path; anything else is a clean `ERR-IMG` "not supported" ([ERROR_CONTROL.md](../ERROR_CONTROL.md)).
- **Pre-demosaic:** per-site black levels honouring the ActiveArea origin, white level, Gr/Gb equilibration, hot pixels, white balance.
- **Demosaic:** **RCD** for Bayer, **Markesteijn** for X-Trans (1 pass default, 3 passes for quality), written from the published descriptions; no third-party code copied, so GPL-3.0 and relicensing stay clean (ADR-0008).
- **Post-demosaic:** false-colour median on colour differences; highlight reconstruction.
- **Colour:** camera → XYZ → a linear f32 working space → tone curve last → sRGB or Display P3 with its profile. The looks of FR-070 (ADR-0036) apply here.
- **Colour management on every surface** (preview, thumbnail, zoom, crop source), linear-light resizing, PNG gAMA/cHRM/sRGB and dithering.
- **The embedded-JPEG look match becomes an option** ("camera look"), every constant named and explained.
- **Measured against independent references:** synthetic charts (zone plate, Siemens star) with an artifact score, and a neutral develop from another program kept as a reference; golden images with a PSNR gate; fuzzed parsers; the RAW sample corpus in CI.

## Consequences

- A larger core to own and test; in return the quality of every RAW is ours, X-Trans gets a real demosaic, and FR-070's looks have a correct colour space to work in.
- Budgets (to measure, [OPTIMIZATION.md §3](../OPTIMIZATION.md#3-budgets)): RCD on 24 MP ≤ 400 ms, Markesteijn 1-pass on 26 MP ≤ 700 ms on REF-MAC; previews stay within NFR-002 through the cached develop.
- Every pixel change bumps the thumbnail cache version.
- rawler stays an LGPL dependency for unpacking only (ADR-0029); dropping it later means writing decompressors, a separate decision.
