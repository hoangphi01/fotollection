# ADR-0029: Camera RAW through rawler now, our own decoder next

- **Status:** Accepted. Supersedes [ADR-0023](0023-raw-through-os-codecs.md) in part (Windows no longer needs the Raw Image Extension). Its last step is carried out by [ADR-0030](0030-our-own-raw-decoder.md): DNG has moved to our own decoder
- **Date:** 2026-10-26
- **Related:** FR-016, NFR-008, NFR-017, ADR-0008, ADR-0023, [RESULT.md §15](../RESULT.md#15-camera-raw-built-in-rawler-v080-measured-first)

## Context

ADR-0023 decoded RAW only through the operating system, to keep every dependency permissive and the relicensing option of ADR-0008. On macOS that works (ImageIO). On Windows it needs Microsoft's Raw Image Extension, and without it RAW files are skipped. The website and the v0.7.0 notes did not say so ([LESSONS.md](../LESSONS.md), 2026-10-26). The owner found it, and asked for RAW to work with nothing to install.

Every capable RAW decoder in Rust is LGPL-2.1: rawler (about 300 cameras, CR3, X-Trans), rawloader and quickraw. No MIT or Apache one covers ARW, RW2 and CR3. The project is open source under GPL-3.0, which may include LGPL-2.1 code; the only cost is the relicensing option, which needs rawler gone first. The owner chose rawler now, and our own decoder to replace it format by format.

**Measured first** (`examples/measure_rawler.rs`, `bench/quality/raw_compare.py`, nine CC0 samples, REF-MAC):

- **Speed:** rawler develops faster than ImageIO on eight of nine (GX9 0.21 s against 0.61 s, Sony A7 III 0.22 s against 0.43 s); the Canon 5D IV and the Ricoh GR III are about as fast. Sizes match ImageIO's on all nine.
- **Colour:** rawler's own develop has no exposure, tone curve or lens shading, so it looks dark and flat (ΔE2000 10–16 from the camera's JPEG). Finished by matching each channel to the camera's own embedded JPEG (`raw_finish`), it comes to ΔE 1.2–7.6 from that JPEG, closer than ImageIO on six of nine.
- **Geometry:** rawler does not correct lens distortion. On Micro Four Thirds and L-mount bodies (the Olympus, the S5), whose lenses rely on that correction, the frame is slightly wider and a little barrelled compared with ImageIO.
- **Size:** the app binary grows 3.5 MB with rawler (36.0 → 39.5 MB on macOS, before compression).

## Options considered

1. **Keep the OS only** (ADR-0023). Rejected: every Windows user without the extension loses RAW.
2. **rawler everywhere.** One look on both systems, but macOS users would lose lens-distortion correction.
3. **rawler first on Windows, ImageIO first on macOS, each with the other as fallback.** Windows gets RAW with nothing to install; macOS keeps what works today, and rawler reads what ImageIO cannot (a body newer than the OS).
4. **Our own decoder first.** Keeps the relicensing option, but takes months per family of formats.

## Decision

Option 3 now, option 4 next.

- `fotollection_core::raw` has a `RawDecoder` trait with two implementations, `Os` and `Rawler`, tried in the order `decoders()` gives for the platform. Callers (`codec`, thumbnails, the camera session) only use `raw::decode`, `raw::dimensions` and `raw::preview`.
- `crates/rawlgpl` wraps rawler and is the **only** crate allowed an LGPL dependency (`deny.toml` names rawler alone). Its `rawdb` feature, which goes online, stays off; the core still links no networking crate.
- rawler's develop is finished by our own `raw_finish`: three curves matched to the camera's embedded JPEG, or an automatic exposure and an S curve when the file has none (some ORF).
- RAW files are untrusted: the pixel count is checked before developing, and a panic inside rawler is caught (`catch_unwind`). For that, the release profile changes from `panic = "abort"` to `panic = "unwind"`, which adds 5.1 MB to the binary (39.5 → 44.7 MB, macOS); the Windows installer stays well under NFR-008's 25 MB.
- **Our own decoder** (`crates/rawcore`, pure Rust, from public specifications and our own notes) becomes a third `RawDecoder` and takes over one format at a time once it is as good as rawler on the samples: ARW and RW2 first, then DNG, NEF, CR2, ORF, then CR3 and RAF. When every format has moved, rawler is removed.

## Consequences

- RAW opens on Windows with nothing to install. The claim on the website and in the guide changes only after it has been tried on a Windows PC without the extension.
- **Before any relicensing (ADR-0008), rawler must be removed.** LGPL-2.1 is met by shipping Fotollection's complete source, so anyone can rebuild it with a changed rawler; README's third-party notices name it.
- On Windows, lens distortion is not corrected yet; a print of a wide-angle Micro Four Thirds shot can look slightly barrelled. Our own decoder adds the correction from each camera's own metadata.
- The binary is 8.6 MB larger in all (rawler and unwinding).
