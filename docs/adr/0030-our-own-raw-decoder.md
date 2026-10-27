# ADR-0030: Our own RAW decoder, format by format, coloured like the camera

- **Status:** Accepted. Carries out the "our own decoder next" part of [ADR-0029](0029-raw-through-rawler-then-our-own-decoder.md)
- **Date:** 2026-10-27
- **Related:** FR-016, NFR-004, NFR-017, ADR-0008, ADR-0029, [RESULT.md §16](../RESULT.md#16-our-own-raw-decoder-dng-v080-measured-first), [docs/raw/](../raw/README.md)

## Context

rawler (LGPL-2.1) makes RAW open everywhere, but it must be gone before any relicensing (ADR-0008). It also matches the camera's look one channel at a time, which can shift hues, and its develop is slower and heavier than it needs to be. The owner asked for our own decoder, with colour of the quality of Adobe's, without installing anything from Adobe, and chose the **camera's own JPEG** as the default look.

"As good as Adobe" is measured without Adobe, three ways:

1. **Against the camera's embedded JPEG** (the look the owner chose): mean ΔE2000.
2. **Against ImageIO**, another professional RAW engine: SSIM for detail and geometry.
3. **Neutral greys** on a chart in a sample: the Panasonic S5 sample shows a Datacolor SpyderCheckr 24, whose reference values are not published reliably, so only its six grey patches are scored (mean chroma C\*, 0 is a perfect grey).

## Options considered

1. **Keep rawler for good.** No work, but no relicensing, and the look stays per channel.
2. **Port an existing decoder.** Every capable one is GPL or LGPL; porting carries its licence.
3. **Write our own, clean room, one format at a time,** behind the `RawDecoder` trait, each format moving only after it passes a quality gate.

## Decision

Option 3, in `crates/rawcore`: pure Rust, `#![forbid(unsafe_code)]` through the workspace lints, GPL-3.0-only like the rest of the project.

- **Sources:** only public documents (DNG 1.7, TIFF 6.0 and TIFF/EP, ITU T.81 for lossless JPEG, the Malvar–He–Cutler demosaic paper, McCamy's CCT formula, the Bradford transform) and, for formats without a specification (ARW, RW2…), our own observation of files, written up in our own words in `docs/raw/`. No code from rawler, rawloader, dcraw, LibRaw or RawTherapee is read into the project.
- **Colour:** the DNG model (ColorMatrix interpolated by temperature, ForwardMatrix, camera calibration, analog balance, Bradford to D50, then sRGB). Then the **camera's look, learned from its own embedded JPEG**: a lens-shading gain `exp(a·r² + b·r⁴)`, one tone curve on luminance shared by the three channels (so hues do not shift), and a 3×3 colour matrix whose rows sum to 1 (so greys stay grey). Without a JPEG, an automatic exposure and S curve.
- **Adobe profiles:** an Adobe Standard `.dcp` is read only if one is already on the user's machine (Lightroom or DNG Converter installed); Fotollection never bundles or copies one. Not built yet.
- **Order:** `raw::decoders()` puts `Own` first; it answers `Unsupported` for every format it has not taken over, so ImageIO (macOS) or rawler (Windows) take those as before.
- **Quality gate** for a format to move to `Own`, on the CC0 samples (RESULT §16):
  - mean ΔE2000 to the embedded JPEG ≤ 3, and no worse than rawler + `raw_finish`;
  - SSIM against ImageIO ≥ 0.97 (after lens correction, for lenses that need it);
  - develop time and peak memory no more than rawler's;
  - grey patches, when the sample has a chart: mean C\* ≤ 1.5;
  - no panic on any sample cut short anywhere, and 10 minutes of mutation testing without one (`examples/mutate.rs`, run on stable Rust because cargo-fuzz needs nightly).
- **DNG moves first** (Ricoh GR III sample): every gate item passed. ARW and RW2 come next, then lens distortion, then NEF, CR2, ORF, CR3, RAF. rawler is removed when every format has moved.

## Consequences

- DNG opens the same on macOS and Windows, faster and with less memory than either other path, and closer to the camera's JPEG (RESULT §16).
- DNG lens corrections stored as opcodes (WarpRectilinear in OpcodeList3) are not applied yet; the Ricoh sample still scores SSIM 0.976 against ImageIO.
- Linear DNG, non-Bayer filters, lossy DNG and DNG floating point fall through to the next decoder.
- Every new format needs its own notes in `docs/raw/`, its row in RESULT §16 and in `docs/raw/README.md`.
