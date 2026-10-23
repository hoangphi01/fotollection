# ADR-0004: libjpeg-turbo as the default JPEG codec

- **Status:** Accepted (to be confirmed by the Phase 0 benchmark)
- **Date:** 2026-10-23
- **Related:** NFR-001, NFR-003, FR-008

## Context

JPEG is the dominant input and output format, and the codec decides throughput and quality. Our requirements include:

- DCT-domain downscaling for fast thumbnails.
- Lossless DCT-domain crops.
- Full control over quality and chroma subsampling.
- SIMD on both ARM and x86.

## Options considered

1. **`zune-jpeg` (pure Rust):** memory-safe and fast to decode. But it has no encoder, no lossless transforms, and limited DCT scaling.
2. **`image` crate codecs:** convenient, but slower and with few controls.
3. **libjpeg-turbo via `turbojpeg`:** the industry standard, with SIMD, DCT scaling, `tjTransform` lossless crop, and fast encoding. It is a C dependency.
4. **mozjpeg:** produces the smallest files at a given quality, but encodes slowly.

## Decision

Use **libjpeg-turbo (`turbojpeg` crate)** for JPEG decoding and encoding. Offer **mozjpeg** as an optional "smallest files" export mode. Keep a `zune-jpeg` decode spike in Phase 0 for comparison. If it is faster for full-size decode, use it on that path only.

## Consequences

- Meets the needs for thumbnails (DCT scaling) and lossless crops.
- Adds a C dependency to the attack surface. Mitigated by fuzzing ([TESTING.md §4](../TESTING.md#4-fuzzing)) and version pinning ([SECURITY.md §6](../SECURITY.md#6-supply-chain)).
- The result is recorded in [OPTIMIZATION.md §5](../OPTIMIZATION.md#5-benchmark-log), and this ADR is updated if the benchmark overturns the choice.
