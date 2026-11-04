# Security, Privacy & Resilience — Fotollection

> **Status:** Draft v0.2 · **Last reviewed:** 2026-11-02
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
| Tampered update / installer | **YES** | No updater (ADR-0021); published SHA-256 checksums over HTTPS. Installers are unsigned for now ([ADR-0009](adr/0009-unsigned-distribution.md)); the Linux AppImage and `.deb` too, checked against the same `SHA256SUMS.txt` ([ADR-0034](adr/0034-linux-lean.md)) |
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
- **The webview never supplies file system paths.** Import and export folders are chosen through dialogs opened **from Rust** (`pick_and_import_folder`, `pick_export_folder`), or dropped onto the window (an OS event handled in Rust), or handed over by the OS through "Open with" (macOS `Opened` events, command-line paths, a second launch forwarded by the single-instance plugin; FR-054), where each photo is checked by `import::check_file` before anything else. The export destination is kept in Rust state. The main window's capability is `core:default` plus the window actions the Windows title bar needs (minimize, toggle maximize, close, start dragging; FR-052): no dialog, filesystem or shell permissions.
- **Inline styles are allowed, scripts are not** ([ADR-0013](adr/0013-csp-inline-styles.md)). Radix primitives (Select, Dialog, Popover) inject runtime `<style>` tags for scroll locking whose content is computed, so hashes cannot work. `script-src` stays `'self'` with no `unsafe-inline` or `unsafe-eval`, which is what stops XSS. With `img-src`, `connect-src` and `font-src` locked to the app, injected CSS has no channel to exfiltrate data.
- **One network request, on click only.** *Check for updates* (FR-061, ADR-0025) is the only request the app can make: from `src-tauri/src/updates.rs`, never the core, to a fixed GitHub API URL over rustls, with a 10 s timeout and a 1 MB cap, parsed into a fixed type. Every link must start with `https://github.com/hoangphi01/fotollection/releases/`; the chosen one stays in Rust and is opened by `tauri-plugin-opener`'s Rust API, so the webview has no opener permission and never supplies a URL. *Report* (FR-069) opens one more fixed address, the issue forms (`ISSUES` in `updates.rs`), the same way; it makes no request itself.
- **Commands validate every argument in Rust.** Never trust the UI: IDs must exist, settings are range-checked, and paths are canonicalized ([§4](#4-file-system-safety)). The `fotothumb` protocol is checked the same way: strict routes, numeric IDs only, and a zoom region (FR-057) must be four finite fractions inside the photo with a pixel width clamped to 4,096, or the request gets a 404 before any file is read.
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
  - JPEG: `jpeg_layout::read` walks the markers up to the first scan with bounds checks on
    every read, so a cut or forged header returns "unknown", never a panic (tested on every
    truncation of a fixture); the reference decoder checks the same size limits before it
    allocates (ADR-0033)
  - HEIC: decoded by the OS (ImageIO / WIC) in `fotollection-oscodec`, one of the two crates
    with `unsafe` FFI (the other is `fotollection-oscamera`); the size from the header is checked against the same limits
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

**Written metadata is an allowlist too.** Every export (prints and share exports, [FR-010](PRD.md#fr-010--metadata-privacy-on-export), [FR-053](PRD.md#fr-053--export-for-social-media)) writes EXIF with `exif_out.rs`: only make, model, lens, exposure, aperture, ISO, focal length, capture time and `Software`. Text is printable ASCII, at most 64 characters, taken from values already validated on import. There is no GPS IFD, ever; tests read print and share exports back to check (`fr_010_prints_keep_the_camera_details_but_never_gps`, `fr_053_*`).

### Other untrusted inputs

| Input | Defenses |
|-------|----------|
| Project file `.fotollection` ([ADR-0016](adr/0016-self-contained-project-bundle.md)) | Reached only through a Rust-side dialog or the OS (file association: macOS `Opened` event, Windows command line); a requested path waits in Rust until the UI confirms, so the webview never supplies it. Recognised by content: a ZIP bundle or a legacy JSON file. **Bundle:** at most 160 MB on disk and uncompressed, 128 entries, per-part caps (project 64 MB, settings 4 MB, font 20 MB, logo 2 MB, manifest 1 MB) and a 100:1 compression-ratio limit, all checked on the central directory **before** decompression; part names from a strict allowlist (`manifest.json`, `project.json`, `settings.json`, `fonts/<id>.ttf\|otf`, `logos/<id>.png`), so `..`, absolute paths, drive letters and stray files are rejected; encrypted entries rejected; reads bounded by the real byte count; every part's BLAKE3 must match the manifest. Each part then goes through its own typed reader: project validation, settings validation, and the same font and PNG validators as a manual import. Nothing is applied until the whole bundle has passed. **JSON:** size cap of 64 MB, typed `serde` schema with unknown fields rejected, `schema_version` checked before the body. In both, folder paths are data: shown to the user, opened read-only, never executed or used as export destinations without the save dialog |
| Camera logo PNG ([FR-040](PRD.md#fr-040--exif-info-frame), [ADR-0014](adr/0014-user-supplied-brand-logos.md)) | Chosen through a Rust-side dialog only. PNG magic bytes, ≤ 2 MB, ≤ 4096 px per edge and a decoder memory limit, checked **before** decoding with the pure-Rust `png` crate. Copied into the config dir, so the original path is never stored |
| User font OTF/TTF ([FR-042](PRD.md#fr-042--custom-fonts)) | Chosen through a Rust-side dialog only. Magic bytes (`00010000`/`true`/`OTTO`; collections and WOFF rejected), ≤ 20 MB, full parse by the pure-Rust `ab_glyph`/`ttf-parser` before use, digits required. Copied into the config dir (the original path is never stored). Up to 50 fonts |
| `settings.json` ([DATABASE.md §6](DATABASE.md#6-app-settings)) | Size cap of 4 MB, format and version checked first, strict typed parse (`deny_unknown_fields`), duplicate keys rejected, user text sanitized (control characters stripped, ≤ 60 chars). An invalid file is quarantined, never executed or partly trusted |
| XMP sidecar ([FR-058](PRD.md#fr-058--lightroom-compatibility-through-xmp)) | Only `<name>.xmp` and `<name>.<ext>.xmp` next to a photo being imported are opened, read only. At most 1 MB (read through a bounded reader). Parsed by `quick-xml` (MIT) with namespaces resolved; a DTD ends the parse, so no entity is declared or expanded (no XXE, no billion laughs); any error means no sidecar. Only `xmp:Rating` is used, as a number −1…5. Nothing from it is rendered as HTML |
| GPX track, planned ([FR-066](PRD.md#fr-066--borrow-places-from-a-gpx-track)) | Size cap of 50 MB. XML parsed without external entities or DTDs (no XXE). Coordinates and timestamps validated like EXIF ([Layer 3](#3-input-and-metadata-defense)) |
| `.cube` LUT, planned ([FR-070](PRD.md#fr-070--looks-from-lut-files), [ADR-0036](adr/0036-colour-looks-cube-luts-first.md)) | Chosen through a Rust-side dialog only, read only in Rust. Size cap of 8 MB; 1D or 3D, 2–65 points a side, capped line count; `DOMAIN_MIN`/`DOMAIN_MAX` checked; NaN and out-of-range values rejected. Copied by content hash into `looks/`; a fuzz target covers the parser |
| Dates, places and place names from the UI ([FR-032](PRD.md#fr-032--manual-date-and-place), [FR-005](PRD.md#fr-005--location-stamp-offline)) | Checked in Rust (`src-tauri/src/places.rs`): 1–10,000 photo ids per call; dates parsed from a fixed format and kept within 1826–2100; place ids must exist in the bundled data; searches ≤ 100 characters and ≤ 50 results; names trimmed and cut to 80 characters. Names are rasterized by the stamp or shown as plain text, never as HTML. The project file applies the same checks on load |
| Captions and stamp text ([FR-036](PRD.md#fr-036--custom-captions)) | Rasterized in Rust. Only a fixed set of `{variables}` is expanded, so there is no format-string or template execution. Shown in the UI as plain text only |
| Postcard text ([FR-073](PRD.md#fr-073--postcard-sheets)) | Checked in Rust on every `set_postcard_text` (the message 400 characters and 8 lines, the address 200 and 4, no control character but line breaks) and brought within those limits when a project file is read; rasterized in Rust, never rendered as HTML; the interface shows it in a plain `textarea` |
| Presets ([FR-033](PRD.md#fr-033--retro-film-date-stamp-and-stamp-presets), [FR-034](PRD.md#fr-034--lab-presets), [FR-038](PRD.md#fr-038--print-presets)) | Built-in presets are static data compiled into the binary and are never fetched from the network. Imported `.fotopreset` files are capped at 1 MB and parsed with a typed schema (`deny_unknown_fields`). Every parameter is range-checked (for example Photo Style −5…+5, WB −9…+9). Presets contain **values only**, never paths, scripts or LUT files from outside the app |

## 4. File system safety

- Source files are opened **read-only** and never modified.
- Export paths are built only from `dest_root` (chosen by the user through the OS dialog) plus **sanitized** segments: date, trip name and original stem. Segments are restricted to a safe character set, reserved Windows names (`CON`, `NUL`…) are rejected, and segments are length-capped.
- After the path is joined, it is canonicalized and the app verifies that it is still under `dest_root`. Otherwise the photo is rejected.
- Files are written to a temporary file and then atomically renamed. Files are never overwritten unless the user chose that rule ([FR-009](PRD.md#fr-009--export-by-date)).

## 5. Privacy

- **Zero network by default** ([NFR-006](PRD.md#nfr-006--offline-and-zero-telemetry)). No telemetry, no crash upload. The only request is *Check for updates*, on click; *Report a problem* only opens a fixed page in the browser ([§2](#2-tauri-hardening)). CI asserts that no networking crate is linked into the core.
- **The public privacy policy** is on the website's [code signing policy page](https://fotollection.org/code-signing/#privacy) (`site/_includes/code-signing.html`), as SignPath Foundation requires. It must match this section and [FR-061](PRD.md#fr-061--check-for-updates): change both together.
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
| Fuzzing | Mutation testing of the RAW decoder on stable Rust; `cargo-fuzz` targets planned ([TESTING.md §4](TESTING.md#4-fuzzing)) |

**Cameras over USB** (`fotollection-oscamera`, [ADR-0026](adr/0026-cameras-through-the-os-usb-stack.md)): everything a camera sends is untrusted. `ptp` and the maker drivers (`vendor/sony.rs`: the settings block, the live-view frame's offset and length) check every length and count against the bytes received, cap lists at 4,096 entries, reject unknown data types and forms and returns an error on a bad answer, never a panic (tested by cutting a valid answer at every byte). Every call to the camera has a timeout: on Windows, where WPD calls block, each open camera has its own thread and the caller stops waiting after the timeout, after which that camera refuses further calls. An object read from a camera is capped at 256 MB.

**Camera sessions** (`src-tauri/src/camera.rs`, [ADR-0028](adr/0028-camera-sessions-saved-on-demand.md)):

- **Writing shots:** a file name from the camera keeps only its last part, plain characters and at most 64 of them, with the extension its format calls for. A shot is written as `.part` and renamed. A name already taken gets a suffix; nothing is overwritten.
- **Save all:** the destination comes from a Rust-side dialog, never from the webview, and cannot be inside the app's data. Files move without overwriting; across volumes each is copied, its size checked, and only then removed.
- **Delete photos:** it only removes a folder directly under the app's `camera/` directory, and the UI asks first.
- **Live view:** frames stay in memory and are never written or sent (NFR-006).

**Website secrets.** The footer's visit count (`scripts/ga-visits.mjs`, run by `pages.yml`) reads Google Analytics with a service account that has the *Viewer* role on the one GA property and nothing else. Its JSON key and the property ID live only in the repository secrets `GA_SERVICE_ACCOUNT` and `GA_PROPERTY_ID`; the script never prints them, asks only for the `analytics.readonly` scope, accepts a numeric property ID and a whole number back, and writes nothing on any error. Workflows for pull requests from forks get no secrets. If the key leaks, delete it in Google Cloud and add a new one.

**APT signing key** ([ADR-0035](adr/0035-linux-install-from-fotollection-org.md)).

- **What it is:** the Linux repository at fotollection.org/apt is signed with an Ed25519 key made for it alone (fingerprint `1F67 E8C1 93CD CA68 BFC8 82DA 5850 9F27 C429 45A3`).
- **Where it lives:** the private key is only in the secret `APT_GPG_PRIVATE_KEY`, plus the owner's offline backup outside the repository. `scripts/apt-repo.sh` imports it into a temporary GnuPG home that is deleted after signing, and never prints it.
- **How users trust it:** they add the key with `signed-by`, so it vouches for this repository and nothing else on their system.
- **The install script:** `install.sh` is plain POSIX sh served over HTTPS. It verifies the AppImage against `SHA256SUMS.txt` and asks for sudo only to add the repository.
- **If the key leaks:** make a new key, replace the secret, publish the new public key, and say so in the next release notes.

## 8. Resilience and error handling

- **Per-photo isolation:** every stage returns a per-photo `Result`. A failure is listed in the `export-finished` event's `failed[]` (or counted in `analysis-finished`) and the batch continues.
- **Cancellation:** jobs check a token between stages, and temporary files are removed on cancel.
- **Memory:** the bounded pipeline keeps peak memory under [NFR-004](PRD.md#nfr-004--memory-ceiling) for any batch size ([OPTIMIZATION.md §4.3](OPTIMIZATION.md#43-parallelism-and-back-pressure)).
- **Panics:** a panic hook writes each panic to the local error log (`ERR-RUS-0005`, [ERROR_CONTROL.md §3](ERROR_CONTROL.md#3-status-of-the-plan-v091)); a friendly dialog after a crash is still planned (S7 in [STATUS.md](STATUS.md)). The release profile unwinds (`panic = "unwind"`, ADR-0029) so that a panic inside a third-party decoder can be caught: rawler, which reads untrusted RAW files, runs inside `catch_unwind` in `crates/rawlgpl`, and its failure becomes a per-photo error, never a crash.
- **The error log** (`<app data>/logs/`, 5 files of 1 MB, `crates/core/src/error_log.rs`): local only, read by nothing but *Copy diagnostics*, which the user pastes where they choose. A line holds codes, the version, the system and the error's text with every path and photo file name cut to `<path>.ext`: no image data, no GPS, no folder or file names (tested in `error_log.rs`).
- **Our own RAW decoder** ([ADR-0030](adr/0030-our-own-raw-decoder.md)): `crates/rawcore` has no `unsafe`; the TIFF reader bounds every offset and count (1,024 entries per directory), the pixel count is capped before any buffer is allocated, lossless JPEG rejects difference sizes above 16, and a bad file is an error, never a panic. Tested on samples cut short at 256 places and by 10 minutes of mutation testing ([TESTING.md §4](TESTING.md#4-fuzzing)). The image facts (FR-071) read JPEG APP2/APP14 segments, ICC tag tables, PNG chunks and DNG opcode lists with every offset checked (`jpeg_layout::read_header`, `colour::profile_name`, `facts::of_png`, `dng::facts`, at most 256 tags or opcodes); tests cut each input at every byte. DNG opcode lists (`OpcodeList1/2/3`, planned in FR-072) are untrusted too: the opcode count and each gain map's size are capped before allocation, and the list gets its own fuzz target (`dng_opcodes`).
- **Camera RAW through rawler** ([ADR-0029](adr/0029-raw-through-rawler-then-our-own-decoder.md)): the pixel count is checked against the import limits before any develop; rawler's network feature (`rawdb`) is off, and `cargo tree -p fotollection-core` still contains no networking crate.

### User-facing error messages

Every error the app can show, its message, cause and what the user does, is in [ERROR_CONTROL.md](ERROR_CONTROL.md), with its permanent `ERR-<KIND>-<NNNN>` code. Messages never carry image data, GPS or full paths.

## 9. Verification checklist

| Test | Expected result |
|------|-----------------|
| `<script>` in ImageDescription / UserComment | Field never read and never shown |
| JPEG+HTML polyglot | Rejected: `import.unsupported` |
| `.html` renamed to `.jpg` | Rejected by magic bytes |
| 65535×65535 declared JPEG | Rejected before allocation |
| Truncated/corrupt JPEG, HEIC, PNG | Photo fails (a JPEG cut short: `ERR-IMG-0006`, never half a photo); app and batch continue |
| Trip name `../../etc` or `CON` | Sanitized; file stays under `dest_root` |
| Export with GPS present, default settings | Output has no GPS tags |
| App run with network disabled | All MVP features work; zero connection attempts |
| CSP violations in release build | None |
