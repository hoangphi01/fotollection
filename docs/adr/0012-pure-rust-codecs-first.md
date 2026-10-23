# ADR-0012: Start with pure-Rust JPEG codecs behind a trait

- **Status:** Accepted
- **Date:** 2026-10-23
- **Related:** ADR-0004, NFR-002, NFR-003, NFR-007

## Context

[ADR-0004](0004-libjpeg-turbo-default-codec.md) chose libjpeg-turbo (via `turbojpeg`), pending a Phase 0 benchmark. Building libjpeg-turbo from source needs CMake and NASM. Neither is on the owner's Mac, and NASM is not on GitHub's Windows runners. Linking the Homebrew system library works on macOS, but it would make the Windows build and the release packaging more complex before the first useful slice even exists.

## Options considered

1. **turbojpeg, built from source:** the fastest, with DCT scaling and lossless crop. But it adds CMake/NASM to every developer machine and CI runner.
2. **turbojpeg via the system library (pkg-config/vcpkg):** works on macOS. On Windows it needs vcpkg wiring, and installers must ship the DLL.
3. **Pure Rust (`zune-jpeg` to decode, `jpeg-encoder` to encode) behind a `JpegCodec` trait:** no C toolchain, identical builds on macOS, Windows and Linux, memory-safe decoding. But it has no DCT-domain scaling and no lossless transforms.

## Decision

Option 3 for now. `crates/core/src/codec.rs` defines `JpegCodec { decode, encode }`, and `PureRustCodec` is the default. Thumbnails use the EXIF-embedded JPEG (T0) first, which hides the missing DCT scaling for the grid.

First measurements on the reference Mac (M3, 95 × 16 MP JPEG; [OPTIMIZATION.md §5](../OPTIMIZATION.md#5-benchmark-log)):

- Full decode in 45 ms.
- Warm preview in 26 ms at p95.
- Export at 19.7 photos per second.
- Peak RSS of 1.4 GB.

All are within the NFR budgets.

## Consequences

- CI and onboarding stay simple: there are no native image libraries yet. libheif (ADR-0005) will be the first one.
- A turbojpeg implementation can be added behind the same trait when a feature needs it, such as lossless crop-only export (FR-008) or faster T1 thumbnails through DCT scaling. The switch is decided by benchmark, and this ADR is then superseded.
- `jpeg-encoder` carries the IJG license (permissive, attribution required). The attribution is in the README.
