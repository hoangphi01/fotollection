# ADR-0043: Lossless export as a 32-bit float path to 16-bit or float TIFF, learned from RawTherapee and darktable

- **Status:** Accepted (owner, 2026-11-07)
- **Date:** 2026-11-07
- **Related:** FR-111, FR-112, FR-072, FR-088, NFR-009, ADR-0008, ADR-0037, ADR-0039, ADR-0040

## Context

The owner wants Fotollection to export lossless images beside its print JPEGs, which needs a strong colour and image pipeline, and asked how RawTherapee and darktable do it, where Fotollection stands, and whether its Rust core is stable enough.

**RawTherapee** ([RawPedia, toolchain pipeline](https://rawpedia.pixls.us/toolchain_pipeline/), [colour management](https://rawpedia.pixls.us/color_management/)) runs a fixed order (raw fixes, demosaic, highlights, white balance, working space, tools, resize, output profile) in 32-bit float in a linear ProPhoto working space, with many tools in CIE Lab; it writes TIFF 8/16-bit, 16-bit half and 32-bit float, Deflate with a predictor, the ICC profile embedded, and no dithering.

**darktable** ([pixelpipe](https://docs.darktable.org/usermanual/development/en/darkroom/pixelpipe/the-pixelpipe-and-module-order/), [colour spaces](https://docs.darktable.org/usermanual/development/en/special-topics/color-management/color-spaces/), [export](https://docs.darktable.org/usermanual/development/en/module-reference/utility-modules/shared/export/)) works scene-referred: 4 × 32-bit float, linear Rec.2020, unbounded until a single tone mapper at the end; it exports TIFF 8/16/32, PNG 16, EXR, JPEG XL lossless and more, at full resolution, resampled last. Its cost is complexity: overlapping modules, a movable module order, binary parameters that need migrating.

**Fotollection** (audit 2026-11-07): the RAW develop is linear f32 (FR-088) and the look table is u16, but every stage after decode works on one 8-bit type (`RgbImage`), RAW is cut to 8 bits in its finish, 16-bit PNG is dithered to 8, qcms converts RGB8 only, JPEG is the only encoder, and our RAW develop clips to sRGB primaries and embeds no profile. The core is stable for its job (420 tests, mutation testing of the RAW chain, no `unsafe`, print ΔE2000 0.19); a lossless path is a deep but bounded change on the export path only.

## Decision

- **Lossless is a Pro-mode option, never the default** (FR-111): JPEG stays the default; Simple and Collage do not offer it.
- **One 32-bit float path from decode to encode** for that export, quantised only when the file is written, so a RAW keeps what lies above white (FR-112) and colours outside sRGB survive to the output profile. Previews, thumbnails and the GPU look stay 8-bit (ADR-0039).
- **TIFF 16-bit by default, TIFF 32-bit float as a choice**, Deflate with a predictor, the ICC profile and the EXIF allowlist embedded, never GPS. Writer: the `tiff` crate (MIT); 16-bit and float colour transforms: `moxcms` (BSD-3/Apache-2.0), since qcms is 8-bit only.
- **A fixed order with one good default per step**, as both teach, and none of darktable's movable order or parallel modules; edits stay readable, versioned serde data.
- **A linear Rec.2020 working space for the RAW develop** (v0.15.0) with FR-111 (v0.15.0, owner 2026-11-09), with the gamut clip moved to the output conversion.
- **Learn, never copy.** RawTherapee and darktable are GPL, and Fotollection keeps the option to relicense (ADR-0008): their manuals, release notes and the papers they cite are read; their code is not read for implementation, ported or copied.
- **Image methods are chosen by measurement** (owner, 2026-11-07): a harness with known truth first, candidates compared with the current method, parameters swept, results in RESULT, before any method lands (FR-112 first).

## Consequences

- A new pixel path through orientation, crop, straighten, resize, look, stamp, frame and the writers; budgets for time, file size and memory per photo are set in OPTIMIZATION §3 and measured before it is built.
- The RAW tone curve's 4,096 bins are interpolated and the encode tables stop leaving gaps near black (v0.13.3), which helps today's JPEGs too.
- More dependencies (`tiff`, a direct `moxcms`), both permissive; `cargo deny` checks them.
- Export settings gain a format (settings migration) and the export dialog a Pro-only choice.
