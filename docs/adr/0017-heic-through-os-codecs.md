# ADR-0017: Decode HEIC with the operating system's codecs

- **Status:** Accepted. Supersedes the HEIC part of [ADR-0005](0005-input-formats-jpeg-heic-png.md); JPEG and PNG are unchanged
- **Date:** 2026-10-24
- **Related:** FR-001, NFR-004, ADR-0008, ADR-0012

## Context

iPhones save photos as HEIC (HEIF with HEVC), so HEIC input is part of the MVP ([FR-001](../PRD.md#fr-001--import-photos)). [ADR-0005](0005-input-formats-jpeg-heic-png.md) planned **libheif + libde265**, statically linked. Two problems have appeared since:

- Both libraries are **LGPL**. The project allows only permissive dependencies, to keep the option of relicensing ([ADR-0008](0008-license-gpl3-with-relicensing-option.md)). Static linking also makes LGPL obligations heavier.
- Building libheif statically on Windows was the riskiest build task in the plan, and HEVC carries patent questions that the OS vendors already handle.

No mature pure-Rust HEVC decoder exists.

## Options considered

1. **libheif + libde265 (LGPL)**, as planned. Works everywhere, but it breaks the permissive-only rule and needs a hard native build.
2. **The OS codecs.** macOS has ImageIO, which decodes HEIC on every supported version. Windows has WIC, which decodes HEIC when the free *HEIF Image Extensions* and the *HEVC Video Extensions* are installed from the Microsoft Store (often preinstalled by PC makers, otherwise about USD 1). Nothing is linked, so there are no licence or patent obligations for us.
3. **Convert HEIC before import** with an external tool. Unacceptable for a one-step workflow.

## Decision

Use **the OS codecs**, in a small crate **`fotollection-oscodec`**:

- **macOS:** ImageIO and Core Graphics through `objc2-image-io` and `objc2-core-graphics` (MIT/Apache/Zlib).
- **Windows:** WIC through `windows` (MIT/Apache).
- **Linux:** no codec; HEIC is reported as unsupported.

The crate is the only one in the workspace allowed `unsafe` code. Every block carries a `// SAFETY:` comment, and the API is safe. `fotollection-core` keeps `unsafe_code = "forbid"`. The core calls it from `decode_image` / `image_dimensions` when it sees an ISO-BMFF `ftyp` box.

Details:

- **Limits first:** the size from the image header is checked against `MAX_EDGE` and `MAX_PIXELS` before the bitmap is allocated (SECURITY.md §3).
- **Colour:** macOS draws in the image's own RGB colour space and returns its ICC profile, so export copies it. Windows returns pixels as stored, without the ICC profile for now.
- **Orientation:** pixels are returned as stored, and the EXIF orientation is applied by the core, as for JPEG. EXIF is read from the HEIF container by `kamadak-exif`.
- **Windows without the codec:** import rejects HEIC with the code `import.heic_codec_missing`, and the UI says which Microsoft Store extensions to install.

## Consequences

- No LGPL library and no native build step. `cargo deny` keeps passing with permissive licences only.
- HEIC decoding is slower than JPEG: about 170 ms for a 16 MP photo on REF-MAC, against 45 ms ([OPTIMIZATION.md §5](../OPTIMIZATION.md#5-benchmark-log)). A follow-up can make thumbnails with ImageIO's thumbnail API instead of a full decode.
- Behaviour depends on the OS. Tests generate a synthetic HEIC with `sips` on macOS. On Windows, CI accepts either a successful decode or `heic_codec_missing`, never a panic.
- The `unsafe` FFI code should get a fuzz target when fuzzing is set up ([TESTING.md §4](../TESTING.md#4-fuzzing)).
