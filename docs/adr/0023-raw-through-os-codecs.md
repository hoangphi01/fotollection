# ADR-0023: Read camera RAW with the operating system's codecs

- **Status:** Accepted. Extends [ADR-0017](0017-heic-through-os-codecs.md) from HEIC to camera RAW
- **Date:** 2026-10-25
- **Related:** FR-016, NFR-004, NFR-017, ADR-0008, ADR-0017

## Context

Photographers shoot RAW, and the owner's own camera (Panasonic GX9) writes RW2. Until now every RAW was rejected at import ([FR-016](../PRD.md#fr-016--raw-input)). The choices are the same as for HEIC:

- The Rust RAW decoders (`rawloader`, `rawler`) and LibRaw are **LGPL**, which the permissive-only rule rules out ([ADR-0008](0008-license-gpl3-with-relicensing-option.md)).
- Writing a RAW developer is out of scope: demosaicing, colour matrices and lens data per camera are years of work.
- Asking users to convert to DNG first breaks the one-step workflow.

Measured on nine CC0 samples from raw.pixls.us before deciding ([RESULT.md §12](../RESULT.md#12-camera-raw-v070)): macOS develops every one of RW2, CR2, CR3, NEF, ARW, RAF, ORF and DNG.

## Decision

Read RAW through **`fotollection-oscodec`**, as HEIC is:

- **macOS:** ImageIO develops the RAW. NEF, ARW, CR2 and DNG start like any TIFF, so the file's type identifier (for example `com.sony.arw-raw-image`) is passed as a hint, and the largest image in the file is taken; without the hint ImageIO showed a NEF's 160 × 120 thumbnail and could not open an ARW.
- **Windows:** WIC with Microsoft's free **Raw Image Extension**. The RAW decoder is asked for directly (`GUID_ContainerFormatRaw`), so the TIFF decoder never takes a TIFF-based RAW. Without the extension, import rejects RAW with `import.raw_codec_missing` and the UI says what to install.
- **Linux:** no codec; RAW is unsupported.
- **Recognition:** by magic bytes where the format has its own (RW2 `IIU\0`, ORF `IIRO`, RAF `FUJIFILMCCD-RAW`, CR3 `ftyp` + `crx `, which used to pass as HEIC), and by a TIFF header plus the extension for CR2, NEF, ARW and DNG.

**Two paths, by cost.** A full develop takes 0.4–2.2 s and up to 1.3 GB of RAM; the preview every RAW carries takes 9–80 ms. So:

- **The embedded preview** (`oscodec::preview`) feeds thumbnails and best-shot analysis, which see every photo.
- **The full develop** feeds the print preview, zoom and export, which see one photo at a time. Its pixel size is the only one used for geometry: crops are stored as fractions, and the embedded preview can have another size or aspect.
- **Memory:** RAW develops are limited to a small number at once so peak memory stays within [NFR-004](../PRD.md#nfr-004--memory-ceiling).

## Consequences

- No LGPL library; `cargo deny` keeps passing. New cameras arrive with OS updates, not with ours.
- Colour and sharpness of a developed RAW are the OS vendor's defaults, not Lightroom's; Fotollection is not a RAW editor.
- The result depends on the OS version. Tests decode the downloaded samples on macOS when they are present (`scripts/fetch-raw-samples.py`); Windows CI accepts either a decode or `raw_codec_missing`, never a panic.
- EXIF is read by `kamadak-exif` for TIFF-based RAW; RW2, ORF, RAF and CR3 need the container handled first (FR-016).
