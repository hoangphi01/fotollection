# Bundled brand logos

The camera-brand logos of the info frame ([FR-040](../../../../docs/PRD.md#fr-040--exif-info-frame), [ADR-0020](../../../../docs/adr/0020-bundled-brand-logos.md)). Each PNG is a rendering of a file on Wikimedia Commons that Commons marks as **public domain** (`PD-textlogo` or `PD-shape`: too simple for copyright). `scripts/brands/fetch-and-render.sh` rebuilds them from the pinned SVGs in `src/`.

## Trademarks

These logos are **trademarks of their owners**, listed below. Public domain status covers copyright only. Fotollection is an independent, non-commercial open-source project; it is **not affiliated with, sponsored or endorsed by** any of these companies. The logos are used only to show which camera took a photo, on the user's own prints.

**Removal on request:** if a trademark owner asks, its logo is removed in the next release. The frame then typesets the brand name instead, as it does for brands without a bundled logo. Each brand is one PNG, one SVG and one table row, so removal is a one-commit change.

| Brand | Commons file | Owner of the trademark | SVG SHA-256 |
|-------|--------------|------------------------|-------------|
| `apple` | [Apple logo black.svg](https://commons.wikimedia.org/wiki/File:Apple_logo_black.svg) | Apple Inc. | `9d00ea77a3240f29…` |
| `canon` | [Canon wordmark.svg](https://commons.wikimedia.org/wiki/File:Canon_wordmark.svg) | Canon Inc. | `9b6acea0a1a76099…` |
| `fujifilm` | [Fujifilm logo.svg](https://commons.wikimedia.org/wiki/File:Fujifilm_logo.svg) | FUJIFILM Corporation | `d86cf1e0208b9898…` |
| `hasselblad` | [Hasselblad Logo.svg](https://commons.wikimedia.org/wiki/File:Hasselblad_Logo.svg) | Victor Hasselblad AB | `631f89027756186c…` |
| `leica` | [Leica Camera logo.svg](https://commons.wikimedia.org/wiki/File:Leica_Camera_logo.svg) | Leica Camera AG | `5fcb8f72ae1640fa…` |
| `lumix` | [Lumix logo.svg](https://commons.wikimedia.org/wiki/File:Lumix_logo.svg) | Panasonic Holdings Corporation | `affd3d5e35050c3a…` |
| `nikon` | [Nikon Logo.svg](https://commons.wikimedia.org/wiki/File:Nikon_Logo.svg) | Nikon Corporation | `449bab29e259c46b…` |
| `noritsu` | [Noritsu logo.svg](https://commons.wikimedia.org/wiki/File:Noritsu_logo.svg) | Noritsu Koki Co., Ltd. | `19a90f835d85c9b3…` |
| `phaseone` | [Phase one logo.svg](https://commons.wikimedia.org/wiki/File:Phase_one_logo.svg) | Phase One A/S | `133d0b1459fa967a…` |
| `ricoh` | [Ricoh logo 2012.svg](https://commons.wikimedia.org/wiki/File:Ricoh_logo_2012.svg) | Ricoh Company, Ltd. | `be0ae0727d7d94dd…` |
| `samsung` | [Samsung wordmark.svg](https://commons.wikimedia.org/wiki/File:Samsung_wordmark.svg) | Samsung Electronics Co., Ltd. | `7c1a09ef9076d86b…` |
| `sony` | [Sony logo.svg](https://commons.wikimedia.org/wiki/File:Sony_logo.svg) | Sony Group Corporation | `f6191bc69f8df259…` |

All files were checked on 2026-10-24 (Noritsu on 2026-10-25): licence *Public domain*, restriction *trademarked* (Commons API `extmetadata`). The SVGs were scanned for scripts and external references before rendering.

## Rendering

- `src/<brand>.svg`: the file as downloaded from Commons (pinned by SHA-256 in the script).
- `<brand>.png`: rendered with macOS AppKit (`scripts/brands/render.swift`) on a transparent background, 320 px high for wordmarks and 512 px for square marks. That is twice the largest size the frame draws.
- Colours are adjusted at run time, never in these files: black parts turn white on a black frame, and the *Classic* and *One colour* styles recolour wordmarks (`crates/core/src/brands.rs`).
