# Architecture Decision Records

Each ADR captures one significant decision. ADRs are **immutable once accepted**: to change a decision, write a new ADR that supersedes the old one, and update the old ADR's status line.

| ADR | Title | Status | Date |
|-----|-------|--------|------|
| [0001](0001-desktop-tauri-over-web.md) | Desktop app with Tauri 2 instead of a web app | Accepted | 2026-10-23 |
| [0002](0002-rust-core-crate-separation.md) | Tauri-independent Rust core crate | Accepted | 2026-10-23 |
| [0003](0003-offline-geocoding-geonames.md) | Offline reverse geocoding with GeoNames | Accepted | 2026-10-23 |
| [0004](0004-libjpeg-turbo-default-codec.md) | libjpeg-turbo as the default JPEG codec | Accepted | 2026-10-23 |
| [0005](0005-input-formats-jpeg-heic-png.md) | MVP input formats: JPEG, HEIC, PNG | Accepted | 2026-10-23 |
| [0006](0006-docs-as-code.md) | Docs-as-code with requirement traceability | Accepted | 2026-10-23 |
| [0007](0007-ui-react-shadcn-lucide.md) | UI stack: React + shadcn/ui + Lucide | Accepted | 2026-10-23 |
| [0008](0008-license-gpl3-with-relicensing-option.md) | License: GPL-3.0, keeping the option to relicense | Accepted | 2026-10-23 |
| [0009](0009-unsigned-distribution.md) | Ship unsigned builds (no paid certificates) | Accepted | 2026-10-23 |
| [0010](0010-project-file-format.md) | Project file format for culling and edits | Accepted | 2026-10-23 |

New ADR: copy [0000-template.md](0000-template.md) to `NNNN-short-title.md` using the next number, then add a row to this table.

## History before ADRs

The following decisions were made in 2026-09 for the original browser-based design ("PhotoOrder"). They were **superseded** by ADR-0001 and later ADRs:

| Original decision | Superseded by |
|-------------------|---------------|
| Vite + vanilla TS web app, Canvas API + Web Workers | ADR-0001, ADR-0007 |
| exifr for EXIF parsing, JSZip for output | ADR-0002 (Rust core) |
| Nominatim online reverse geocoding | ADR-0003 |
| Netcup hosting + Cloudflare CDN | ADR-0001 (no server needed) |
| PWA for offline support | ADR-0001 (offline by design) |

These decisions remain valid: **all processing is local**, an **EXIF field allowlist** is used, and **metadata is never rendered as HTML**.
