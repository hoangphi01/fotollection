# ADR-0005: MVP input formats — JPEG, HEIC, PNG

- **Status:** Accepted. The HEIC decoder choice is superseded by [ADR-0017](0017-heic-through-os-codecs.md) (OS codecs instead of libheif); the format list stands
- **Date:** 2026-10-23
- **Related:** FR-001, FR-016

## Context

The target users shoot with cameras that produce JPEG (for example the Lumix GX9) and with iPhones that produce HEIC. Screenshots and exports come as PNG. RAW support is valuable but large in scope.

## Decision

The MVP accepts **JPEG, HEIC/HEIF and PNG**. HEIC is decoded with **libheif + libde265 via `libheif-rs`**, statically linked: vendored on macOS and built through a vcpkg static triplet on Windows. **RAW is deferred to Phase 3** ([FR-016](../PRD.md#fr-016--raw-input)).

## Consequences

- iPhone libraries work out of the box. HEIC output is not planned, because print labs expect JPEG.
- Building libheif statically on Windows is the riskiest build task. It is a Phase 0 spike in [STATUS.md](../STATUS.md).
- libheif and libde265 are LGPL, which affects the project license decision ([SECURITY.md §6](../SECURITY.md#licensing-note)).
- HEVC patents: decoding through libde265 is common practice in open-source software, but must be reviewed before any commercial distribution.
