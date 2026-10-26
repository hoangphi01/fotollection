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
| [0009](0009-unsigned-distribution.md) | Ship unsigned builds (no paid certificates) | Accepted, updater part superseded by 0021 | 2026-10-23 |
| [0010](0010-project-file-format.md) | Project file format for culling and edits | Superseded by 0016 | 2026-10-23 |
| [0011](0011-photo-style-looks.md) | Photo Style looks via baked 3D LUTs, global adjustments only | Accepted | 2026-10-23 |
| [0012](0012-pure-rust-codecs-first.md) | Start with pure-Rust JPEG codecs behind a trait | Accepted | 2026-10-23 |
| [0013](0013-csp-inline-styles.md) | Allow inline styles in the CSP, keep scripts strict | Accepted | 2026-10-23 |
| [0014](0014-user-supplied-brand-logos.md) | Camera brand logos are user-supplied | Accepted | 2026-10-23 |
| [0015](0015-ui-localization-i18next.md) | Interface localization with bundled i18next resources | Accepted | 2026-10-23 |
| [0016](0016-self-contained-project-bundle.md) | Self-contained project bundle (ZIP) | Accepted | 2026-10-24 |
| [0017](0017-heic-through-os-codecs.md) | Decode HEIC with the operating system's codecs | Accepted | 2026-10-24 |
| [0018](0018-face-models-and-runtime.md) | Face and eye models for best-shot suggestions, run with tract | Accepted | 2026-10-24 |
| [0019](0019-grouping-by-visual-signature.md) | Group similar shots by a layout and colour signature, scored on labelled photos | Accepted | 2026-10-24 |
| [0020](0020-bundled-brand-logos.md) | Bundle public-domain camera-brand logos for the info frame | Accepted | 2026-10-24 |
| [0021](0021-versioning-and-automated-releases.md) | One SemVer version, released automatically when it changes on `main`; no in-app updater | Accepted | 2026-10-24 |
| [0022](0022-srgb-and-exif-for-share-exports.md) | Share exports: sRGB through `qcms`, an allowlisted EXIF block with the app's name, never GPS | Accepted | 2026-10-25 |
| [0023](0023-raw-through-os-codecs.md) | Camera RAW through the OS codecs (ImageIO, WIC + Raw Image Extension); embedded preview for thumbnails and analysis, full develop for preview and export | Accepted | 2026-10-25 |
| [0024](0024-optional-photo-viewer.md) | Fotollection as an optional viewer: "Open with" without becoming the default, the photo's folder as a tab, one instance | Accepted | 2026-10-25 |
| [0025](0025-update-check-on-click.md) | Check for updates only when the user clicks; the installer opens in the browser, never installed by the app | Accepted, supersedes 0021 in part | 2026-10-25 |
| [0026](0026-cameras-through-the-os-usb-stack.md) | Cameras over USB through the OS imaging stack (ImageCaptureCore, WPD), with our own clean-room vendor drivers; UVC/HDMI and a watched folder as fallbacks; no libgphoto2, SDKs, WinUSB or Wi-Fi | Proposed | 2026-10-26 |
| [0027](0027-effective-time-and-place.md) | Effective time and place: per-body clock offsets in the project, manual and borrowed places in a fixed order, the camera's own values kept | Accepted | 2026-10-26 |
| [0029](0029-raw-through-rawler-then-our-own-decoder.md) | Camera RAW through rawler (LGPL-2.1, the only LGPL dependency) behind a `RawDecoder` trait, finished to the camera's embedded JPEG; Windows first rawler, macOS first ImageIO; our own decoder replaces it format by format; `panic = "unwind"` | Accepted, supersedes 0023 in part | 2026-10-26 |

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
