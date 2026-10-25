# Security, Privacy & Resilience — Fotollection

> **Status:** Draft v0.2 · **Last reviewed:** 2026-10-23
>
> Threat model and defenses for an **offline desktop app** (Tauri 2 + Rust core).
> v0.1 targeted a browser-based web app. Concerns that only applied to that architecture (Cloudflare headers, Nominatim rate limits, browser memory limits) were removed in v0.2. See [ADR-0001](adr/0001-desktop-tauri-over-web.md).
> To report a vulnerability, open a private GitHub security advisory on the repository. Do not open a public issue.

## 1. Threat model

The app processes **untrusted image files** and displays **untrusted metadata** inside a webview. It holds **sensitive data**: GPS coordinates can reveal a home address.

| Threat | Applies? | Notes |
|--------|----------|-------|
| Malicious image exploiting a decoder | **YES** | JPEG and PNG are decoded in pure Rust. HEIC goes to the OS codec (ImageIO, WIC), which gets the OS vendor's security updates; size limits are checked before decoding ([ADR-0017](adr/0017-heic-through-os-codecs.md)) |
| XSS via EXIF text in the webview | **YES** | XSS in a Tauri webview can call IPC commands, so the impact is higher than in a browser tab |
| Path traversal / arbitrary file write via IPC | **YES** | Export destination and file names are derived from metadata |
| Supply chain (crates, npm, native libs) | **YES** | Compromised dependency could exfiltrate photos |
| GPS / metadata leakage in exported files | **YES** | Mitigated by [FR-010](PRD.md#fr-010--metadata-privacy-on-export) |
| Resource exhaustion (decompression bombs, huge dimensions) | **YES** | Must fail one photo, not the app ([NFR-011](PRD.md#nfr-011--robustness)) |
| Tampered update / installer | **YES** | No updater (ADR-0021); published SHA-256 checksums over HTTPS. Installers are unsigned for now ([ADR-0009](adr/0009-unsigned-distribution.md)) |
| Network attacks (MITM, SSRF, CSRF) | **Minimal** | The app makes no network requests by default ([NFR-006](PRD.md#nfr-006--offline-and-zero-telemetry)) |
| SQL injection, server compromise | **NO** | No database, no server |

## 2. Tauri hardening

- **Capabilities (least privilege):** the main window gets only the permissions it needs:
  - Dialog open/save.
  - Filesystem read limited to user-selected paths and the app cache and config directories.
  - Filesystem write limited to the user-selected export destination and the app cache and config directories.
  - **No** shell, **no** `http`, **no** global filesystem scope.
- **CSP** in `tauri.conf.json`:

  ```text
  default-src 'self';
  script-src 'self';
  style-src 'self' 'unsafe-inline';
  img-src 'self' fotothumb: http://fotothumb.localhost data:;
  connect-src ipc: http://ipc.localhost;
  font-src 'self';
  object-src 'none';
  base-uri 'none';
  form-action 'none';
  frame-ancestors 'none'
  ```

  There are no inline scripts, no `unsafe-eval` and no remote origins. Tailwind is compiled to a static CSS file at build time. `'unsafe-inline'` applies to **styles only** ([ADR-0013](adr/0013-csp-inline-styles.md)).
- **The webview never supplies file system paths.** Import and export folders are chosen through dialogs opened **from Rust** (`pick_and_import_folder`, `pick_export_folder`), or dropped onto the window (an OS event handled in Rust). The export destination is kept in Rust state. The main window's capability is `core:default` plus the window actions the Windows title bar needs (minimize, toggle maximize, close, start dragging; FR-052): no dialog, filesystem or shell permissions.
- **Inline styles are allowed, scripts are not** ([ADR-0013](adr/0013-csp-inline-styles.md)). Radix primitives (Select, Dialog, Popover) inject runtime `<style>` tags for scroll locking whose content is computed, so hashes cannot work. `script-src` stays `'self'` with no `unsafe-inline` or `unsafe-eval`, which is what stops XSS. With `img-src`, `connect-src` and `font-src` locked to the app, injected CSS has no channel to exfiltrate data.
- **Commands validate every argument in Rust.** Never trust the UI: IDs must exist, settings are range-checked, and paths are canonicalized ([§4](#4-file-system-safety)).
- **No remote content** is ever loaded into the webview. Navigation to any non-app URL is blocked.
- Devtools are disabled in release builds.

## 3. Input and metadata defense

Every file passes through these four layers before any pixel or metadata value reaches the UI.

```text
Layer 1: FILE VALIDATION (before parsing)
  - Detect type by magic bytes (JPEG FF D8 FF, PNG 89 50 4E 47, HEIF 'ftyp' brands)
  - Reject files > 200 MB; reject declared dimensions > 200 MP (decompression-bomb guard)
  - Polyglot scan of the first 4 KB: <html, <script, %PDF, PK, MZ → reject
  - JPEG: warn if significant data follows the EOI marker (FF D9)
  - HEIC: decoded by the OS (ImageIO / WIC) in `fotollection-oscodec`, the only crate
    with `unsafe` FFI; the size from the header is checked against the same limits
    before the bitmap is allocated
  - PNG: the IHDR size is checked against 30,000 px per edge and 200 MP before the
    pixel buffer is allocated; decoded by the pure-Rust `png` crate with its default
    chunk-allocation limits

Layer 2: CONTAINED PARSING
  - EXIF parsed with an explicit field ALLOWLIST:
      ALLOWED: DateTimeOriginal, OffsetTimeOriginal, CreateDate, GPSLatitude(+Ref),
               GPSLongitude(+Ref), GPSAltitude, Make, Model, Orientation,
               PixelXDimension, PixelYDimension, ExposureTime, FNumber, ISO,
               LensMake, LensModel, FocalLength (range-checked; strings ≤ 100 chars)
      IGNORED: everything else (UserComment, ImageDescription, Artist, MakerNote,
               XP* fields, …) — never read, never displayed
  - Each decode runs inside catch_unwind on a worker thread with a time limit (5 s for metadata, 30 s for full decode)
  - Failures mark the photo as failed and the batch continues

Layer 3: VALUE VALIDATION (typed, in Rust)
  - Dates: valid calendar date, year 1900–2100
  - GPS: finite, lat ∈ [-90, 90], lon ∈ [-180, 180]; (0, 0) treated as missing
  - Numbers: finite and in sane ranges
  - Strings (Make/Model): ≤ 100 chars, control characters stripped, Unicode-normalized

Layer 4: SAFE RENDERING (UI)
  - Values are rendered only as React JSX text (auto-escaped) — never
    dangerouslySetInnerHTML or innerHTML (STYLE_RULE §3)
  - Stamps are rasterized in Rust, so no metadata reaches HTML in order to be printed
  - CSP is the last safety net
```

**Why an allowlist:** a new EXIF injection vector in a field we never read cannot affect us.

**Written metadata is an allowlist too.** Share exports ([FR-053](PRD.md#fr-053--export-for-social-media)) write EXIF with `exif_out.rs`: only make, model, lens, exposure, aperture, ISO, focal length, capture time and `Software`. Text is printable ASCII, at most 64 characters, taken from values already validated on import. There is no GPS IFD, ever; a test reads every export back to check. Print exports write no metadata.

### Other untrusted inputs

| Input | Defenses |
|-------|----------|
| Project file `.fotollection` ([ADR-0016](adr/0016-self-contained-project-bundle.md)) | Reached only through a Rust-side dialog or the OS (file association: macOS `Opened` event, Windows command line); a requested path waits in Rust until the UI confirms, so the webview never supplies it. Recognised by content: a ZIP bundle or a legacy JSON file. **Bundle:** at most 160 MB on disk and uncompressed, 128 entries, per-part caps (project 64 MB, settings 4 MB, font 20 MB, logo 2 MB, manifest 1 MB) and a 100:1 compression-ratio limit, all checked on the central directory **before** decompression; part names from a strict allowlist (`manifest.json`, `project.json`, `settings.json`, `fonts/<id>.ttf\|otf`, `logos/<id>.png`), so `..`, absolute paths, drive letters and stray files are rejected; encrypted entries rejected; reads bounded by the real byte count; every part's BLAKE3 must match the manifest. Each part then goes through its own typed reader: project validation, settings validation, and the same font and PNG validators as a manual import. Nothing is applied until the whole bundle has passed. **JSON:** size cap of 64 MB, typed `serde` schema with unknown fields rejected, `schema_version` checked before the body. In both, folder paths are data: shown to the user, opened read-only, never executed or used as export destinations without the save dialog |
| Camera logo PNG ([FR-040](PRD.md#fr-040--exif-info-frame), [ADR-0014](adr/0014-user-supplied-brand-logos.md)) | Chosen through a Rust-side dialog only. PNG magic bytes, ≤ 2 MB, ≤ 4096 px per edge and a decoder memory limit, checked **before** decoding with the pure-Rust `png` crate. Copied into the config dir, so the original path is never stored |
| User font OTF/TTF ([FR-042](PRD.md#fr-042--custom-fonts)) | Chosen through a Rust-side dialog only. Magic bytes (`00010000`/`true`/`OTTO`; collections and WOFF rejected), ≤ 20 MB, full parse by the pure-Rust `ab_glyph`/`ttf-parser` before use, digits required. Copied into the config dir (the original path is never stored). Up to 50 fonts |
| `settings.json` ([DATABASE.md §6](DATABASE.md#6-app-settings)) | Size cap of 4 MB, format and version checked first, strict typed parse (`deny_unknown_fields`), duplicate keys rejected, user text sanitized (control characters stripped, ≤ 60 chars). An invalid file is quarantined, never executed or partly trusted |
| GPX track ([FR-031](PRD.md#fr-031--borrow-gps-from-a-phone-or-gpx-track)) | Size cap of 50 MB. XML parsed without external entities or DTDs (no XXE). Coordinates and timestamps validated like EXIF ([Layer 3](#3-input-and-metadata-defense)) |
| Captions and stamp text ([FR-036](PRD.md#fr-036--custom-captions)) | Rasterized in Rust. Only a fixed set of `{variables}` is expanded, so there is no format-string or template execution. Shown in the UI as plain text only |
| Presets ([FR-033](PRD.md#fr-033--retro-film-date-stamp-and-stamp-presets), [FR-034](PRD.md#fr-034--lab-presets), [FR-038](PRD.md#fr-038--print-presets)) | Built-in presets are static data compiled into the binary and are never fetched from the network. Imported `.fotopreset` files are capped at 1 MB and parsed with a typed schema (`deny_unknown_fields`). Every parameter is range-checked (for example Photo Style −5…+5, WB −9…+9). Presets contain **values only**, never paths, scripts or LUT files from outside the app |

## 4. File system safety

- Source files are opened **read-only** and never modified.
- Export paths are built only from `dest_root` (chosen by the user through the OS dialog) plus **sanitized** segments: date, trip name and original stem. Segments are restricted to a safe character set, reserved Windows names (`CON`, `NUL`…) are rejected, and segments are length-capped.
- After the path is joined, it is canonicalized and the app verifies that it is still under `dest_root`. Otherwise the photo is rejected.
- Files are written to a temporary file and then atomically renamed. Files are never overwritten unless the user chose that rule ([FR-009](PRD.md#fr-009--export-by-date)).

## 5. Privacy

- **Zero network by default** ([NFR-006](PRD.md#nfr-006--offline-and-zero-telemetry)). No telemetry, no crash upload, no update checks unless the user opts in. CI asserts that no networking crate is linked into the core.
- Reverse geocoding uses a bundled dataset, so coordinates never leave the device ([ADR-0003](adr/0003-offline-geocoding-geonames.md)).
- GPS is stripped from exported files by default ([FR-010](PRD.md#fr-010--metadata-privacy-on-export)).
- The thumbnail cache contains image data. It lives in the OS cache directory, and the user can clear it from Settings.
- Logs never contain GPS coordinates or full file paths beyond the file name.

## 6. Supply chain

- `Cargo.lock` and `package-lock.json` are committed. CI builds with `--locked`.
- `cargo deny check` covers advisories, licenses (allowlist), banned crates and duplicate versions.
- `cargo audit` and `npm audit --omit=dev` run in CI, and a weekly scheduled job catches new advisories.
- No native image library is linked today: JPEG/PNG are pure Rust, HEIC uses the OS codecs. A future native library (for example lcms2) is **pinned** and tracked in [STATUS.md](STATUS.md#dependencies-to-watch), with security releases applied within 7 days.
- **Bundled ML models** (FR-041, [ADR-0018](adr/0018-face-models-and-runtime.md)) are static assets compiled into the binary: YuNet (MIT) and FaceMesh V2 (Apache-2.0). They are checked against pinned BLAKE3 sums when loaded, are never downloaded at runtime, and their origin, licences and checksums are in `crates/core/assets/models/MODELS.md`. The licences are re-checked whenever a model is replaced.
- **Bundled brand logos** ([ADR-0020](adr/0020-bundled-brand-logos.md)) are PNGs rendered from public-domain SVGs on Wikimedia Commons, pinned by SHA-256 and checked for scripts and external references before rendering. At run time they go through the same validating PNG decoder as user logos.
- **Bundled place data** (FR-005, [ADR-0003](adr/0003-offline-geocoding-geonames.md)) is a static asset too: `places.bin`, packed from GeoNames (CC BY 4.0), checked against a pinned BLAKE3 sum and parsed with bounds checks on every length. Place names are third-party text, so they are shown as plain text only, never as HTML.
- Dependencies are kept minimal. Adding a new dependency requires a short justification in the PR.
- `cargo deny` checks what ships (`exclude-dev = true`). Dev-only crates, such as `little_exif` for synthetic test fixtures, never touch user data. They are still updated when advisories appear.

### Licensing note

Fotollection is licensed under **GPL-3.0-only** ([ADR-0008](adr/0008-license-gpl3-with-relicensing-option.md)). To keep the option of relicensing later, `cargo deny` allows **only permissive licenses** for dependencies and bans GPL and AGPL. That is why HEIC uses the OS codecs instead of the LGPL libheif and libde265 ([ADR-0017](adr/0017-heic-through-os-codecs.md)).

## 7. CI security gates

| Gate | Tool |
|------|------|
| Vulnerable dependencies | `cargo audit`, `npm audit` |
| License / banned crates | `cargo deny` |
| Dangerous DOM APIs | `grep -rE "dangerouslySetInnerHTML\|innerHTML\|outerHTML\|insertAdjacentHTML\|document\.write\|eval\(\|new Function" ui/src` must be empty |
| Network-free core | `cargo tree -p fotollection-core` contains no `reqwest`/`hyper`/`ureq` |
| Fuzzing | `cargo-fuzz` targets run nightly ([TESTING.md §4](TESTING.md#4-fuzzing)) |

## 8. Resilience and error handling

- **Per-photo isolation:** every stage returns a per-photo `Result`. A failure is reported through the `job://photo-error` event and the batch continues.
- **Cancellation:** jobs check a token between stages, and temporary files are removed on cancel.
- **Memory:** the bounded pipeline keeps peak memory under [NFR-004](PRD.md#nfr-004--memory-ceiling) for any batch size ([OPTIMIZATION.md §4.3](OPTIMIZATION.md#43-parallelism-and-back-pressure)).
- **Panics:** the app shell installs a panic hook that logs locally and shows a friendly dialog. Worker panics are caught and turned into per-photo errors.

### User-facing error messages

| Code | Message | Recovery |
|------|---------|----------|
| `import.unsupported` | "This file isn't a supported photo (JPEG, HEIC, PNG)." | Skipped |
| `import.too_large` | "This photo is too large to process safely." | Skipped |
| `meta.missing_date` | "No capture date found — using file date." | Continue with fallback |
| `meta.unreadable` | "Couldn't read photo information." | Process without stamp |
| `geo.no_gps` | "No location in this photo." | Date-only stamp |
| `decode.failed` | "This photo appears to be damaged." | Skipped |
| `export.write_failed` | "Couldn't save to the destination folder." | Retry or choose another folder |

## 9. Verification checklist

| Test | Expected result |
|------|-----------------|
| `<script>` in ImageDescription / UserComment | Field never read and never shown |
| JPEG+HTML polyglot | Rejected: `import.unsupported` |
| `.html` renamed to `.jpg` | Rejected by magic bytes |
| 65535×65535 declared JPEG | Rejected before allocation |
| Truncated/corrupt JPEG, HEIC, PNG | Photo fails; app and batch continue |
| Trip name `../../etc` or `CON` | Sanitized; file stays under `dest_root` |
| Export with GPS present, default settings | Output has no GPS tags |
| App run with network disabled | All MVP features work; zero connection attempts |
| CSP violations in release build | None |
