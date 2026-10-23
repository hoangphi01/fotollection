# Architecture — Fotollection

> **Status:** Draft v0.1 · **Last reviewed:** 2026-10-23
>
> This document describes **how** the system is built. Requirements live in [PRD.md](PRD.md).
> Performance design rationale lives in [OPTIMIZATION.md](OPTIMIZATION.md).
> Significant decisions are recorded as ADRs in [adr/](adr/README.md).
> When code and this document disagree, the disagreement is a bug: fix one of them in the same PR.

## 1. Overview

Fotollection is a **Tauri 2** desktop application ([ADR-0001](adr/0001-desktop-tauri-over-web.md)):

- A **Rust core** does all image work: decoding, metadata, geocoding, clustering, cropping, stamping, encoding and file I/O.
- A **thin web UI** (Vite + React + TypeScript, styled with Tailwind CSS, shadcn/ui components and Lucide icons; see [ADR-0007](adr/0007-ui-react-shadcn-lucide.md)) renders inside the operating system's own webview: WKWebView on macOS, WebView2 on Windows. No Chromium and no Node.js runtime are bundled.
- The core is a plain Rust library with **no dependency on Tauri** ([ADR-0002](adr/0002-rust-core-crate-separation.md)), so it can be unit-tested, benchmarked and fuzzed on its own.

```mermaid
flowchart LR
  subgraph UI["UI — webview (React + shadcn/ui)"]
    Grid[Library grid]
    Settings[Stamp / crop settings]
    Progress[Export progress]
  end
  subgraph Shell["src-tauri (Rust, thin)"]
    Cmd[IPC commands]
    Proto["fotothumb:// protocol"]
    Ev[Event emitter]
  end
  subgraph Core["crates/core (Rust, pure)"]
    Import --> Meta[Metadata] --> Geo[Geocode] --> Cluster
    Cluster --> Render[Crop + stamp] --> Encode --> Write
    Thumbs[Thumbnail cache]
  end
  Grid -- "img src" --> Proto --> Thumbs
  Settings -- invoke --> Cmd --> Core
  Core -- progress --> Ev -- events --> Progress
```

## 2. Repository layout

Files marked † exist today (Phase 0 scaffold). Everything else is the target layout.

```text
Cargo.toml               † # Cargo workspace, shared lints, build-size profiles
rust-toolchain.toml      † # pinned Rust version
deny.toml                † # cargo-deny license/advisory policy (ADR-0008)
package.json             † # npm workspace root: Tauri CLI + scripts (npm run dev)
assets/                 † # logo-source.png (master), logo-square.png (app icons ≥ 128 px), logo-small.png (simplified tile, no wordmark: every size ≤ 64 px), logo-mark.png (older crop)
crates/
  core/                  † # fotollection-core: all domain logic, no Tauri
    src/
      lib.rs             †
      print_size.rs      † # CEWE/EU print sizes (FR-006)
      import.rs          † # discovery, magic bytes, size + polyglot checks (FR-001)
      codec.rs           † # JpegCodec trait + pure-Rust implementation (ADR-0012); decode_image/image_dimensions dispatch by magic bytes (PNG via `png`); HEIC later
      image.rs           † # RgbImage + EXIF orientation transforms
      metadata.rs        † # EXIF allowlist + validation, embedded thumbnail (FR-002)
      settings.rs        † # RenderSettings (in memory until projects land)
      plan.rs            † # RenderPlan: centred crop, stamp, optional frame plan (FR-004/006/040)
      frame.rs           † # info frame: settings, camera profiles, text, ratio-safe layout (FR-040)
      logo.rs            † # user PNG logos: validation + decode (ADR-0014)
      fonts.rs           † # user OTF/TTF fonts: validation + FontArc (FR-042)
      settings_store.rs  † # settings.json v1: load/validate/atomic save (DATABASE.md §6)
      preview.rs         † # T2 LRU + preview rendering (FR-007)
      cancel.rs          † # CancellationToken
      timeline.rs          # clock offsets, manual date/place, GPS borrowing + GPX (FR-030/031/032)
      geocode.rs           # offline reverse geocoder + offline place search (GeoNames + k-d tree)
      cluster.rs           # trip DBSCAN (Phase 2)
      library.rs         † # columnar in-memory library (DATABASE.md §3.3) + sparse per-photo edits (crop, rating, flag)
      project.rs         † # .fotollection v1: snapshot/restore by folder+path, FK validation (ADR-0010)
      history.rs         † # undo/redo stack of inverse edits (FR-023)
      atomic.rs          † # tmp + fsync + .bak + rename
      presets.rs           # preset library: partial/stackable, linked, .fotopreset I/O (FR-033/034/038)
      quality.rs           # effective-DPI checks (FR-026)
      render/              # (target) crop.rs, style.rs (Photo Style → 3D LUT + tone curve + grain, FR-039),
                           # border.rs, stamp.rs (incl. retro LED), caption.rs,
                           # color.rs (ICC → sRGB), sharpen.rs (Phase 3)
      layout/              # multi-photo sheets + contact sheet (Phase 3, FR-035/037)
      render.rs          † # crop/resize + stamp drawing today; splits into render/* as features land
      export.rs          † # bounded parallel export, cancel, per-photo errors, no overwrite (FR-008/009/011)
      thumbs.rs          † # T0 (embedded) + T1 disk cache (DATABASE.md §8)
    benches/               # criterion benchmarks (see OPTIMIZATION.md)
    fuzz/                  # cargo-fuzz targets (see TESTING.md)
  geodata/                 # build-time packer for the GeoNames dataset
src-tauri/               † # app shell: commands, protocol, capabilities
  src/lib.rs             † # tauri-specta builder + bindings export
  src/commands.rs        † # IPC commands (§6)
  capabilities/          † # Tauri 2 permission sets (see SECURITY.md)
  icons/                 † # generated: npx tauri icon assets/logo-square.png, then python3 scripts/small-icons.py for the ≤ 64 px layers
  tauri.conf.json        † # window, CSP, bundle config
ui/                      † # Vite + React + TS frontend (npm workspace member)
  components.json        † # shadcn/ui config (style radix-nova, Lucide)
  .oxlintrc.json         † # lint rules (bans dangerouslySetInnerHTML / eval)
  src/
    components/ui/       † # shadcn/ui components (copied in via the shadcn CLI, owned by us)
    features/            † # library/, settings/, export/ — feature components + hooks
    ipc/bindings.ts      † # GENERATED from Rust types — do not edit
    lib/                 † # small UI utilities (theme sync, shadcn's cn re-export)
    index.css            † # Tailwind layers + shadcn theme tokens (CSS variables)
scripts/                 † # bootstrap, clean, check-docs, small-icons.py (≤ 64 px icons from assets/logo-small.png)
bench/                   † # run.sh (Rust vs Chromium, 6 vs 8 in flight), batch.sh (5,000-photo memory check), lib.sh, web/ (Chromium via Playwright), chart.mjs (EN + VI SVG charts), results/ (numbers only, incl. budgets.json mirrored from OPTIMIZATION §3)
```

## 3. Tech stack

| Concern | Choice | Notes |
|---------|--------|-------|
| App shell | Tauri 2 | OS webview, small binary, capability-based permissions |
| UI framework | Vite + React 19 + TypeScript | Required by shadcn/ui ([ADR-0007](adr/0007-ui-react-shadcn-lucide.md)) |
| UI components | shadcn/ui (Radix primitives) + Tailwind CSS v4 | Components are copied into the repo, so we own and audit them |
| Icons | Lucide (`lucide-react`) | Tree-shaken: import individual icons only |
| Virtualized grid | `@tanstack/react-virtual` | Required for 1,000+ thumbnails ([NFR-001](PRD.md#nfr-001--library-load-time)) |
| UI state | React state + a small store (`zustand`) | Server state lives in Rust. The UI only caches it |
| IPC type bindings | `specta` + `tauri-specta` | TS types generated from Rust and checked in CI |
| JPEG decode/encode | **Today:** `zune-jpeg` + `jpeg-encoder` (pure Rust) behind the `JpegCodec` trait ([ADR-0012](adr/0012-pure-rust-codecs-first.md)). **Candidate:** `turbojpeg` for DCT scaling and lossless crop ([ADR-0004](adr/0004-libjpeg-turbo-default-codec.md)) | 4:4:4, quality 95, ICC copied |
| JPEG max-quality encode | `mozjpeg` (optional) | Opt-in "smallest file" mode |
| HEIC/HEIF | `libheif-rs` | libheif + libde265, statically linked ([ADR-0005](adr/0005-input-formats-jpeg-heic-png.md)) |
| PNG | `png` | |
| Resize | `fast_image_resize` | SIMD (NEON/AVX2) |
| Colour management | `lcms2` | Preserve or convert the ICC profile |
| Text rendering | `ab_glyph` `FontArc`: bundled Inter (OFL, variable weights) or a user OTF/TTF (FR-042) | `cosmic-text` if complex shaping is ever needed |
| EXIF read | `kamadak-exif` | Allowlisted fields only. Also extracts the IFD1 thumbnail for T0 |
| EXIF write | `little_exif` (candidate) | Writes the privacy-filtered metadata |
| Reverse geocoding | GeoNames `cities1000` + `kiddo` k-d tree | Offline, about 5 MB packed ([ADR-0003](adr/0003-offline-geocoding-geonames.md)) |
| Trip clustering | DBSCAN over (lat, lon, time) | Phase 2 |
| Perceptual hash | `image_hasher` | Phase 2 |
| Face detection | ONNX model via `ort` | Phase 2, on-device |
| Parallelism | `rayon` + bounded `crossbeam-channel` | See [§5](#5-concurrency-and-memory-model) |
| Errors | `thiserror` (core), `anyhow` (shell only) | See [STYLE_RULE.md](STYLE_RULE.md) |
| Project file | `serde_json` + versioned schema | [ADR-0010](adr/0010-project-file-format.md) |
| GPX tracks | `gpx` (candidate; license checked by `cargo deny`) | Phase 2, [FR-031](PRD.md#fr-031--borrow-gps-from-a-phone-or-gpx-track) |
| PDF contact sheet | `pdf-writer` or `printpdf` (candidate, permissive license) | Phase 3, [FR-037](PRD.md#fr-037--contact-sheet) |
| Stamp fonts | Bundled OFL/MIT fonts, including a seven-segment LED face | [FR-033](PRD.md#fr-033--retro-film-date-stamp-and-stamp-presets) |
| Photo Style engine | Own code: parameters baked into a 33³ 3D LUT (trilinear, SIMD) + 1D tone curve + seeded grain | [ADR-0011](adr/0011-photo-style-looks.md) |
| Persisted data model | Normalized tables in JSON (3NF, integer keys, sparse rows) | [DATABASE.md](DATABASE.md) |
| PNG decoding (logos) | `png` (pure Rust) | [ADR-0014](adr/0014-user-supplied-brand-logos.md) |

Crates marked *candidate* are confirmed by Phase 0 benchmarks. The result is logged in [OPTIMIZATION.md](OPTIMIZATION.md#5-benchmark-log).

## 4. Processing pipeline

```text
import → metadata → adjust → geocode → (cluster) → cull/select → plan → render (crop → style → border/stamp) → color → (sharpen) → encode → write
```

| Stage | Input → Output | Notes |
|-------|----------------|-------|
| **import** | paths → `PhotoRef { id, path, kind, size, mtime }` | Magic-byte check only, no decoding |
| **metadata** | `PhotoRef` → `PhotoMeta` | Allowlisted EXIF, validated values ([SECURITY.md §3](SECURITY.md#3-input-and-metadata-defense)) |
| **adjust** | `PhotoMeta` + project → `EffectiveMeta` | Clock offset per camera, manual date/place, borrowed GPS. Originals are kept for reference ([FR-030](PRD.md#fr-030--camera-clock-offset)–[FR-032](PRD.md#fr-032--manual-date-and-place)) |
| **geocode** | `(lat, lon)` → `Place` | Nearest-city lookup, memoized per ~1 km cell |
| **cluster** | `[PhotoMeta]` → `[Trip]` | Phase 2 |
| **cull/select** | library + filter → export set | Ratings, flags and copies come from the project ([FR-022](PRD.md#fr-022--culling-rate-flag-filter)) |
| **plan** | meta + settings → `RenderPlan` | Pure function: print size per photo, crop rect, border, safe area, stamp/caption text and position, colour intent, output path. Also yields the effective-DPI check ([FR-026](PRD.md#fr-026--resolution-warning)) |
| **frame** | photo + `FramePlan` → canvas | When enabled: a canvas at the print ratio; the photo is cropped to fill or fitted above the info bar; brand + lines drawn (FR-040) |
| **render** | `RenderPlan` + pixels → pixels | Crop, then the **Photo Style look** (one 3D-LUT pass + tone curve + grain, [FR-039](PRD.md#fr-039--photo-style-looks)), then fit into the border, then stamp/caption inside the safe area. Overlays come after the look, so they are never tinted |
| **color** | pixels + source ICC → sRGB pixels | `lcms2` transform, with a configurable rendering intent ([FR-027](PRD.md#fr-027--srgb-conversion-for-labs)) |
| **sharpen** | pixels → pixels | Output sharpening for size and paper (Phase 3, [FR-029](PRD.md#fr-029--output-sharpening)) |
| **encode** | pixels → bytes | turbojpeg, ICC re-embedded, metadata filtered ([FR-010](PRD.md#fr-010--metadata-privacy-on-export)) |
| **write** | bytes → file | Written to a temp file, then atomically renamed. Removed if the user cancels |

`RenderPlan` is pure and serializable. The **preview** and the **export** both consume the same plan, so what the user sees is what gets printed.

## 5. Concurrency and memory model

- The UI thread only renders the UI. The Tauri command handlers return immediately and run jobs on the core's worker pool.
- **Export** is a staged pipeline connected by **bounded channels** (back-pressure). The number of decoded full-resolution buffers in flight is capped (`max_inflight`, default `num_cpus`, then clamped by the memory budget in [NFR-004](PRD.md#nfr-004--memory-ceiling)).
- Each job carries a `CancellationToken`. Every stage checks it between photos and between expensive steps.
- Progress events are coalesced to at most 30 per second.
- Three pools never compete:
  - the **interactive** pool (2 threads) serves `fotothumb://` requests for what is on screen;
  - the **background** pool (2 threads) reads EXIF during import and pre-generates T1 thumbnails;
  - **export** has its own bounded pool.

  Visible thumbnails therefore never wait behind background work. A request for a photo whose T1 isn't ready gets the embedded T0 at once.

## 6. IPC contract

All types are defined in Rust and exported to `ui/src/ipc/bindings.ts` through `tauri-specta`. CI fails if the bindings are stale.

### Commands (UI → core)

Commands marked † are implemented.

| Command | Purpose | Requirement |
|---------|---------|-------------|
| `app_info() -> AppInfo` † | App and core versions for the About section | — |
| `list_print_sizes() -> [PrintSize]` † | Print sizes for the settings panel | FR-006 |
| `pick_and_import_folder() -> ImportSummary?` † | Opens the folder dialog **in Rust** and imports it. Drag-and-drop on the window uses the same import path | FR-001, FR-002 |
| `list_photos(filter) -> { photos: [PhotoSummary], total }` † | Data for the grid, sorted by capture time and filtered in Rust by flag and minimum rating. Summaries carry `rating` and `flag` | FR-003, FR-022 |
| `get_settings()` / `set_settings(settings) -> SettingsState` † | Validated render settings plus a `revision` used to refresh previews | FR-004, FR-006 |
| `set_photo_override(id, override)` | Per-photo crop frame or place name | FR-005, FR-006 |
| `pick_export_folder() -> String?` † | Opens the destination dialog **in Rust** and remembers the choice | FR-009 |
| `start_export(filter) -> JobId` † | Exports exactly the photos `filter` keeps (the set the grid shows) to the chosen destination | FR-008, FR-009, FR-010, FR-022 |
| `cancel_job(job_id)` † | Cancel a running job | FR-011 |
| `get_frame_config()` / `set_frame_settings(settings)` † | Info frame settings plus camera profiles (with EXIF-derived defaults) | FR-040 |
| `set_camera_profile(profile)` † | Per-camera overrides: name, brand, lens, focal, aperture, photo style, logo kind | FR-040 |
| `pick_camera_logo(camera_id)` / `clear_camera_logo(camera_id)` † | Choose a PNG with a **Rust-side** dialog, validate it, copy it into `config/logos/` | FR-040 |
| `get_crop_geometry(id) -> CropGeometry` † | Upright size, target ratio, printed cm of the crop area, current crop | FR-006 |
| `set_photo_crop(id, crop?)` † | Set or reset a photo's crop (centre + zoom), undoable | FR-006, FR-023 |
| `set_photo_rating(ids, 0–5)` † / `set_photo_flag(ids, pick\|reject\|null)` † | Culling: one undoable step for 1–10,000 photos; ratings above 5 and unknown IDs are rejected | FR-022, FR-023 |
| `undo()` / `redo()` † | History (also Cmd/Ctrl+Z, Shift+Cmd/Ctrl+Z) | FR-023 |
| `list_fonts()` / `import_font()` / `remove_font(id)` † | User fonts; import opens a **Rust-side** dialog and copies a validated file | FR-042 |
| `photo_details(id) -> PhotoDetails` † | Inspector data: pixel size (header only), file size, GPS, effective DPI at the current print size | FR-002, FR-007, FR-026 |
| `set_rating(ids, stars)` / `set_flag(ids, flag)` | Culling | FR-022 |
| `query_library(filter) -> [PhotoSummary]` | Filtered view (rating, flag, date, camera, GPS, DPI) | FR-022, FR-026 |
| `new_project` / `open_project(path)` / `save_project_as(path)` | Project lifecycle (autosave runs internally) | FR-023 |
| `undo()` / `redo()` | History | FR-023 |
| `set_copies(ids, n)` / `order_summary() -> OrderSummary` | Copies and cost estimate | FR-024 |
| `set_print_size(ids, size_id)` | Per-photo print size | FR-025 |
| `set_clock_offset(camera, offset)` / `match_clock(photo_id, reference_id)` | Clock correction | FR-030 |
| `borrow_gps(source, window)` | Infer GPS from another device or a GPX file | FR-031 |
| `set_manual_meta(ids, date?, place?)` / `search_places(query) -> [Place]` | Manual date and place, offline place search | FR-032 |
| `list_presets()` / `save_preset(preset)` / `delete_preset(id)` | Preset library (stamp, lab, style and print groups) | FR-033, FR-034, FR-038 |
| `apply_preset(ids, preset_id)` / `detach_preset(ids)` / `reset_to_preset(ids)` | Link, detach or reset photos | FR-038 |
| `preview_preset(id, preset_id) -> PreviewHandle` | Hover preview without applying | FR-038 |
| `import_preset(path)` / `export_preset(id, path)` | Share `.fotopreset` files | FR-038 |
| `set_style(ids, style)` | Per-photo Photo Style override | FR-039 |

### Events (core → UI)

Events are typed through `tauri-specta` and named in kebab-case. Events marked † are implemented.

| Event | Payload |
|-------|---------|
| `library-changed` † | `{ count }`. Sent after imports and after every edit, undo and redo (ratings and flags are part of the list). The UI refetches `list_photos` |
| `thumb-ready` † | `{ id }`. The T1 thumbnail is cached, so the UI bumps the image URL |
| `export-progress` † | `{ jobId, done, total }`, coalesced to ≤ 30 per second |
| `export-finished` † | `{ jobId, summary: { exported, failed[], cancelled, destination } }`. Per-photo failures are reported here |
| `revision-changed` † | `{ revision, canUndo, canRedo }` after any render-affecting edit; previews refresh |
| `task-progress` † | `{ task: "import" \| "thumbnails", done, total }`, coalesced to ≤ 30 per second. Drives the progress bar |
| `project://saved` | `{ path, at }`, for the autosave indicator |
| `history://changed` | `{ can_undo, can_redo }` |

### Binary data

Pixels **never** cross IPC as JSON or base64. Thumbnails and previews are served by the asynchronous custom URI scheme `fotothumb` on a low-priority worker pool. On Windows it is addressed as `http://fotothumb.localhost/…`:

- `fotothumb://localhost/thumb/{id}?v={n}`: the cached T1, else the embedded T0, else a T1 generated on the spot. `v` bumps on `thumb-ready`.
- `fotothumb://localhost/source/{id}?rev={n}`: the upright, **uncropped**, unstamped T2 image, the crop editor's background.
- `fotothumb://localhost/preview/{id}?rev={n}`: about 1600 px, rendered from the T2 cache with the **same** `RenderPlan` code as the export. `rev` is the settings revision.

The UI just sets `<img src>`. Routes are parsed strictly: anything other than `/thumb/<u32>` or `/preview/<u32>` returns 404.

## 7. Persistence

All persisted data (project files, settings, presets, the thumbnail cache, built-in datasets), with schemas, versioning and write protocol, is specified in **[DATABASE.md](DATABASE.md)**. In short: there is no database server or SQL database. Work lives in a versioned JSON project file ([ADR-0010](adr/0010-project-file-format.md)), writes are atomic, and source files are never modified.

## 8. Platform notes

| | macOS | Windows |
|---|---|---|
| Webview | WKWebView (system) | WebView2 (evergreen, bootstrapped by installer) |
| Native deps | libheif/libde265 static, via vendored build | libheif via vcpkg static triplet |
| Target | `universal-apple-darwin` | `x86_64-pc-windows-msvc` |

Release packaging and signing are described in [RELEASE.md](RELEASE.md).
