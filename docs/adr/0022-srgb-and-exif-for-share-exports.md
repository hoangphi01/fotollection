# ADR-0022: sRGB and an allowlisted EXIF block for share exports

- **Status:** Accepted
- **Date:** 2026-10-25
- **Related:** FR-053, FR-027, FR-010, ADR-0008, ADR-0012

## Context

Share exports ([FR-053](../PRD.md#fr-053--export-for-social-media)) go to sites that ignore or strip colour profiles and recompress every upload. Two things the print export never needed are now required:

1. **Colour conversion.** A Display P3 or Adobe RGB photo shown as if it were sRGB looks washed out. Prints keep the original's profile because labs read it; social sites do not.
2. **Metadata.** Print exports write no metadata at all, which is how GPS is always removed ([FR-010](../PRD.md#fr-010--metadata-privacy-on-export)). Shared photos are expected to show the camera and the editing app, as Snapseed's do.

## Decision

1. **Colour: `qcms` 0.3** (MIT, pure Rust, Mozilla's colour engine in Firefox).
   - Little CMS (`lcms2`) is MIT too, but needs a C library on both platforms; `qcms` keeps the core pure Rust ([ADR-0012](0012-pure-rust-codecs-first.md)).
   - Perceptual intent, RGB8, run in parallel over 64-row chunks. Untagged and already-sRGB photos are left alone; a profile `qcms` cannot read is embedded unchanged, so viewers still read it.
   - The converted file embeds no profile: untagged JPEGs are read as sRGB everywhere, and it saves bytes.
2. **EXIF: a small writer of our own** (`exif_out.rs`) instead of a general EXIF crate.
   - It writes only an allowlist: IFD0 Make, Model, Software, DateTime and the Exif IFD (ExposureTime, FNumber, ISO, DateTimeOriginal, FocalLength, LensModel). **There is no code path that writes a GPS IFD.**
   - Values come from `ShotInfo`, already validated on import; text is printable ASCII, at most 64 characters.
   - `Software = "Fotollection <version>"`, the field phone galleries show as the editing app.
   - Tests read every field back with the app's EXIF reader and check that no GPS tag exists.
3. **JPEG for sharing:** quality 88, 4:2:0, optimized Huffman, a light unsharp mask (amount 0.35, 3 × 3) only after a downscale. The values are confirmed or tuned by `bench/quality/share_audit.py`.

## Consequences

- One new dependency (`qcms`, MIT, no transitive C code); `cargo deny` passes.
- The writer is about 200 lines we maintain, but it cannot leak a field we never chose to write.
- The FR-027 option for prints can reuse `colour.rs` later.
