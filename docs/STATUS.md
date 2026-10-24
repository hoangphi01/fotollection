# Status — Fotollection

> **Last updated:** 2026-10-24
>
> Living progress tracker. Update it **in the same commit** as the work ([ADR-0006](adr/0006-docs-as-code.md)).
> Statuses: `Not Started` · `In Progress` · `Blocked` · `Done` · `Withdrawn`. An FR is `Done` only when its `fr_NNN_*` test passes ([TESTING.md §2](TESTING.md#2-requirement-traceability)).

## Current phase: Phase 1 — MVP, Stages 1–3 done

**Where we are (2026-10-24).** The app works end to end for **JPEG, PNG and HEIC**:

1. Open or drop a folder.
2. Cull with stars, flags and filters, helped by best-shot suggestions: groups of similar shots, the best of each with reasons, and Keep best.
3. Preview with crop, date stamp and the "Shot on" info frame. The frame can be edited in the preview, and the print size can be Original or a custom one.
4. Export exactly the filtered set.
5. Save the work as a self-contained `.fotollection` document that opens the same on another machine.

The interface is in English and Vietnamese. Measured performance meets every budget ([RESULT.md](RESULT.md)): preview 17 ms (faster than Chromium), export 18 photos/s, memory flat at 1.45 GB for 5,035 photos.

### Done, by stage ([ROADMAP.md — Phase 1 work order](ROADMAP.md#phase-1-work-order-2026-10-23))

1. **Stage 1:**
   - `docs/EXTENSION.md`;
   - PNG input (FR-001);
   - culling with stars, flags, filter and export of the filtered set (FR-022);
   - Original print size (FR-045);
   - preview profiling, the 6 vs 8 in-flight decision, and the 5,035-photo memory check.
2. **Stage 2:**
   - faster previews, 26 → 17 ms (NFR-002);
   - frame controls inside the preview (FR-040);
   - English + Vietnamese UI (FR-020, [ADR-0015](adr/0015-ui-localization-i18next.md));
   - the `.fotollection` document with Save / Save As / Open / New / Relink, the native menu and the file association (FR-023, [ADR-0016](adr/0016-self-contained-project-bundle.md)).
3. **Stage 3:**
   - HEIC input through the OS codecs (FR-001, [ADR-0017](adr/0017-heic-through-os-codecs.md));
   - best-shot suggestions with face and eye models, the Groups and compare views, and Keep best (FR-041, NFR-014, [ADR-0018](adr/0018-face-models-and-runtime.md));
   - accuracy scored on 95 hand-labelled photos, with grouping rebuilt on a layout and colour signature ([ADR-0019](adr/0019-grouping-by-visual-signature.md)).
4. **Docs and tooling:**
   - README with four measured charts and a Vietnamese version (`README.vi.md`);
   - `bench/run.sh`, `bench/batch.sh`, `measure_folder --analyze` and `eval_bestshot`;
   - `docs/RESULT.md`;
   - the `check-i18n` and README-parity gates.

### To check by hand first (not covered by automated tests)

- Save As → Open → rename the photo folder → Relink, in the running app, in English and in Vietnamese.
- The unsaved-changes question on New and Open, and the window title showing *Edited*.
- Double-click the project name to rename it: a saved project (file renamed in Finder / Explorer) and an untitled one (the name is offered by Save).
- Best-shot flow: Find best shots → Groups → Compare → Keep best / Keep this one → undo, in English and Vietnamese, and a narrow window.
- Double-clicking a `.fotollection` file. This needs a release build (`npm run build`), because file associations are not active in `npm run dev`.

### Next, in order

1. **Normal mode (FR-048, NFR-015), asked for next.** It is the "instant" counterpart to the current Professional mode: open → keep/drop → export at 300 DPI for the print size and q90, measured against Professional. Start with the export benchmark (print-sized resize + q90, reduced-scale decode), then the simplified UI and the mode switch.
2. **Stage 3, best-shot suggestions (FR-041, NFR-014).** ✓ Models and licences ([ADR-0018](adr/0018-face-models-and-runtime.md)). ✓ `culling/` core module: grouping, metrics, faces and eyes, scoring with reasons, within NFR-014. ✓ Background analysis job with its cache ([DATABASE §8.1](DATABASE.md#81-analysis-cache)) and the IPC commands `analyze_library`, `list_groups`, `keep_best`. ✓ Groups view ("Best of N" with reasons), compare view with a bar per measure and closed-eye marks, Keep best (per group / all clear groups) and Keep this one, in English and Vietnamese. ✓ Accuracy scored on 95 hand-labelled photos: grouping 3 → 15 of 22 groups with no wrong group, picks 6/6 ([ADR-0019](adr/0019-grouping-by-visual-signature.md), [RESULT.md §8](RESULT.md#8-best-shot-accuracy)). Left: check the flow by hand in the app; read `SubSecTimeOriginal` for bursts within one second; an embedding model for the 7 groups still missed (big pose or framing changes); more labelled folders from other cameras.
3. ✓ **Stage 3, HEIC input (FR-001)** via the OS codecs ([ADR-0017](adr/0017-heic-through-os-codecs.md)). Follow-up: HEIC thumbnails through ImageIO's thumbnail API (faster than a full decode).
4. **Loose ends:**
   - FR-022 filters by date, camera, GPS and DPI, and multi-select;
   - the whole-app memory run with 5,035 photos, using a separate app-data folder (NFR-004);
   - cold-start time (NFR-010);
   - German UI later (FR-020).
5. **Phase 2 queue:**
   - swipe culling (FR-046);
   - Photo Style looks (FR-039);
   - film looks like Fujifilm's film simulations (FR-047);
   - print presets (FR-038);
   - per-photo overrides (FR-044);
   - trip journal, an offline report drafted from the album (FR-049).

## Phase 0 — Foundation

| Task | Status | Notes |
|------|--------|-------|
| Git repository | Done | `hoangphi01/fotollection` (public), branch `main` |
| Documentation framework + docs CI | Done | This commit set |
| Cargo workspace + Tauri 2 scaffold | Done | Tauri 2.12, Rust 1.99.1 pinned |
| UI scaffold: React + Tailwind + shadcn/ui + Lucide | Done | [ADR-0007](adr/0007-ui-react-shadcn-lucide.md) |
| Build-size profiles in `Cargo.toml` + `scripts/clean.sh` | Done | First dev build: `target/` ≈ 1.0 GB ([DEVELOPMENT.md §4](DEVELOPMENT.md#4-build-disk-hygiene)) |
| CI build/test matrix (macOS, Windows) | Done | `.github/workflows/ci.yml` |
| Reproducible setup (`scripts/bootstrap.sh` / `.ps1`, lockfiles) | Done | [DEVELOPMENT.md §1](DEVELOPMENT.md#1-setting-up-a-machine) |
| `tauri-specta` bindings + staleness check | Done | `cargo test` regenerates; CI runs `git diff --exit-code` |
| Benchmark harness + `bench` corpus | In Progress | `examples/measure_folder.rs` on a real folder, plus `bench/run.sh`, the Rust vs Chromium comparison (OPTIMIZATION §2.1). `criterion` benches pending |
| Codec / EXIF crate spikes | In Progress | Pure-Rust codecs adopted for now ([ADR-0012](adr/0012-pure-rust-codecs-first.md)). turbojpeg comparison pending |
| libheif static build on Windows | Withdrawn | HEIC uses the OS codecs instead ([ADR-0017](adr/0017-heic-through-os-codecs.md)) |
| Release workflow (unsigned) | Not Started | |
| Benchmark budget gate (`bench/budgets.toml`) | Not Started | Needs harness |
| Enforce "Done FR has test" in `check-docs.sh` | Not Started | Needs code |

## Functional requirements

| ID | Requirement | Phase | Status | Notes |
|----|-------------|-------|--------|-------|
| FR-001 | Import photos | 1 | Done | JPEG, PNG and HEIC (folder dialog + drag-and-drop), magic bytes, size + polyglot checks. PNG 8/16-bit, grey, palette, alpha on white, ICC kept (`fr_001_png_*`). HEIC via ImageIO (macOS) / WIC (Windows, needs the HEIF+HEVC Store extensions, otherwise `import.heic_codec_missing`) ([ADR-0017](adr/0017-heic-through-os-codecs.md), `fr_001_heic_*`) |
| FR-002 | Read photo metadata | 1 | In Progress | Allowlisted EXIF (date, offset, GPS, orientation, camera) + validation. `fr_002_*` tests pass |
| FR-003 | Library grid | 1 | In Progress | Virtualized grid, T0/T1 thumbnails, skeleton + fade-in, task progress bar with %. Day grouping and multi-select pending |
| FR-004 | Date stamp | 1 | In Progress | 3 formats, 4 corners, white/retro-orange, inside the safe area. More styles with FR-033 |
| FR-005 | Location stamp (offline) | 1 | In Progress | ✓ Bundled GeoNames places (162k towns, 5.4 MB, BLAKE3-checked) and `geocode::reverse` / `search`, about 5 µs per lookup (`fr_005_*`, `fr_032_search_*`). ✓ Place on the date line and in the info frame, from GPS, three formats (settings v5). Next: renaming places, manual and borrowed places (FR-032, FR-031) |
| FR-006 | Crop to print ratio | 1 | In Progress | Crop editor with edge labels (cm/in + px), ratio pill and live DPI; centre + zoom storage. ✓ Lossless quarter turns with `[` `]`, crop follows (`fr_006_*`, project v4). ✓ Straighten ±15° in the crop editor, inscribed crop, crop-only bicubic resampling (project v5) |
| FR-007 | Live preview | 1 | In Progress | Preview dialog with inspector (capture, camera, image, GPS, print DPI, stamp controls). Rust-rendered, warm p95 18 ms (p50 17 ms) |
| FR-008 | Original-quality export | 1 | In Progress | Full resolution, q95 4:4:4, ICC copied. Lossless crop-only path pending |
| FR-009 | Export by date | 1 | Done | `YYYY-MM-DD/` folders, never overwrites (`_1`, `_2`…). `fr_009_*` passes |
| FR-010 | Metadata privacy on export | 1 | In Progress | No EXIF written, so GPS is always stripped. Keep-metadata options pending |
| FR-011 | Batch progress, cancel and recovery | 1 | Done | Progress events, cancel removes partials, per-photo failures. `fr_011_*` pass |
| FR-012 | Trip auto-grouping | 2 | Not Started | |
| FR-013 | Face-aware smart crop | 2 | Not Started | |
| FR-014 | Export by trip | 2 | Not Started | |
| FR-015 | Duplicate detection | 2 | Not Started | |
| FR-016 | RAW input | 3 | Not Started | |
| FR-017 | Photo book layout preview | 3 | Not Started | |
| FR-018 | QR code on photo back | 3 | Not Started | |
| FR-019 | Print service upload | 3 | Not Started | |
| FR-020 | Localization | 1 | Done | English (default) + Vietnamese via i18next ([ADR-0015](adr/0015-ui-localization-i18next.md)): every UI string, Rust error codes, native menu, window title and file dialogs; selector in settings, stored in `settings.json` v3; `check-i18n` gate in lint. German later |
| FR-021 | Manual updates | 3 | In Progress | No updater by design (ADR-0021). ✓ Version in the sidebar byline. ✓ `scripts/release.sh` + `release.yml` (build, tag, publish on a version change). Next: the website |
| FR-022 | Culling (rate, flag, filter) | 1 | In Progress | Stars `0`–`5`, `P`/`X`/`U` in grid and preview, undoable; filter by flag + min rating, export = filtered set (`fr_022_*`). Pending: date/camera/GPS/DPI filters, multi-select, loupe prefetch (NFR-012) |
| FR-023 | Projects, autosave and undo | 1 | Done | `.fotollection` document like `.docx`: Save / Save As / Open / New with Rust-side dialogs, unsaved-changes question, title `Trip — Edited`, native File menu (macOS) and Ctrl shortcuts (Windows), Relink dialog for moved folders, file association. Self-contained ZIP bundle ([ADR-0016](adr/0016-self-contained-project-bundle.md)); fonts installed once, ids remapped; session autosave underneath. `fr_023_*` pass (incl. open on another machine, relink). File association to be checked by hand in a release build |
| FR-024 | Copies and order summary | 2 | Not Started | |
| FR-025 | Mixed print sizes per order | 1 | Not Started | Builds on `PrintSize` |
| FR-026 | Resolution warning | 1 | In Progress | Effective DPI + band shown in the inspector. Grid badge and pre-export summary pending |
| FR-027 | sRGB conversion for labs | 1 | Not Started | `lcms2` |
| FR-028 | Safe area and borders | 1 | Not Started | |
| FR-029 | Output sharpening | 3 | Not Started | Needs real print tests |
| FR-030 | Camera clock offset | 1 | Not Started | |
| FR-031 | Borrow GPS from a phone or GPX track | 2 | Not Started | Depends on FR-030 |
| FR-032 | Manual date and place | 1 | Not Started | |
| FR-033 | Retro film date stamp and stamp presets | 1 | In Progress | ✓ DSEG14 Italic (OFL) LCD face with LED glow and faint shadow (`fr_033_*`). Presets to come |
| FR-034 | Lab presets | 2 | Not Started | |
| FR-035 | Multi-photo layouts | 3 | Not Started | Shares layout engine with FR-017 |
| FR-036 | Custom captions | 2 | In Progress | ✓ A caption line under the date, in the stamp's LCD face (`fr_036_*`). Variables (`{trip}` needs FR-012) and per-photo captions to come |
| FR-037 | Contact sheet | 3 | Not Started | |
| FR-038 | Print presets | 2 | Not Started | Storage model: [DATABASE.md §7](DATABASE.md#7-presets) |
| FR-040 | EXIF info frame | 1 | In Progress | Crop/Fit at print ratio, field switches, per-camera overrides (persisted), user PNG logo, bundled logos of 11 brands with Classic / Brand colours / One colour styles ([ADR-0020](adr/0020-bundled-brand-logos.md), `fr_040_*`). Place needs FR-005 |
| FR-041 | Best-shot suggestions | 1 | In Progress | Models: YuNet (MIT) + FaceMesh V2 (Apache-2.0, ONNX) on `tract-onnx` ([ADR-0018](adr/0018-face-models-and-runtime.md)). Core `culling/` done (`fr_041_*`): bursts grouped by time, camera, GPS and a layout + colour signature that allows for zoom ([ADR-0019](adr/0019-grouping-by-visual-signature.md)); sharpness, shake, exposure, noise; eye aspect ratio; robust z-score with reasons. Scored on 95 hand-labelled photos: 15/22 groups, pair precision 100 %, picks 6/6 (`eval_bestshot`). Analysis job, cache, IPC and the Groups / compare UI done. Left: manual check in the app, `SubSecTimeOriginal` |
| FR-042 | Custom fonts | 1 | Done | OTF/TTF import (Rust dialog, validated, copied), per stamp and per frame, remove → Inter. `fr_042_*` pass |
| FR-043 | Custom print sizes | 1 | Done | cm/in entry, stored in mm, used by crop/frame/DPI/export. `fr_043_*` pass |
| FR-044 | Per-photo overrides | 2 | In Progress | ✓ Date stamp per photo from the preview, Apply to all, Reset in the settings panel, undoable, saved in project v3 (`fr_044_*`). Other settings later |
| FR-045 | Original aspect ratio | 1 | Done | `print_size: "original"`: photo's own ratio, 10 cm short edge, exact pixel crop, framed canvas at photo ratio (`fr_045_*`) |
| FR-046 | Swipe culling | 2 | Not Started | Future: full-screen ↑ add to print list / ↓ leave out, Lightroom-loupe style; changes the main workflow, so it is planned separately |
| FR-047 | Film looks | 2 | Not Started | Future: Fujifilm-style film simulations (slide, chrome, negative, cinema, mono) on the FR-039 engine; own names and LUTs, no trademarks or copied LUT files |
| FR-048 | Normal and Professional modes | 1 | Not Started | **Next.** Normal = instant mode: open → keep/drop → export at 300 DPI for the print size, q90; Professional = everything so far. Shared library and project |
| FR-050 | Print basket | 1 | Not Started | Planned: add to basket after detailed editing; normal export plus an optional extra folder with the basket |
| FR-051 | Landmark recognition | 2 | Not Started | Future: offline landmark model as an optional pack; licence research done (Google on-device classifiers, Apache-2.0); needs an ADR and a size/speed spike |
| FR-049 | Trip journal | 2 | Not Started | Future: offline report drafted from the album (days, places, counts, cameras, focal-length range, best shots per day), templates in EN/VI, editable, export HTML/PDF or a journal print; every editing session adds a journal entry (instax-style) |
| FR-039 | Photo Style looks | 2 | Not Started | [ADR-0011](adr/0011-photo-style-looks.md); "C200 Natural" built-in |

## Non-functional requirements

| ID | Requirement | Budget | Last measured | Status |
|----|-------------|--------|---------------|--------|
| NFR-001 | Library load time | < 1 s first screen | 5,035 photos: import+EXIF 383 ms, T0 236 ms = 0.12 s per 1,000 ([RESULT.md](RESULT.md)) | In Progress |
| NFR-002 | Preview latency | < 50 ms p95 | 18 ms p95, 17 ms p50 (warm), faster than Chromium (18.5 ms) after the screen encoder | In Progress |
| NFR-003 | Export throughput | ≥ 8 photos/s | 18.0/s (16 MP, median of 3) | In Progress |
| NFR-004 | Memory ceiling | ≤ 2 GB | Core: 1.35 GB (95 photos), 1.45 GB (5,035 photos), flat. Whole-app run with 5,035 photos pending | In Progress |
| NFR-005 | UI responsiveness | < 100 ms | — | Not Started |
| NFR-006 | Offline and zero telemetry | 0 requests | — | Not Started |
| NFR-007 | Platforms | macOS 11+, Win 10/11 | — | Not Started |
| NFR-008 | Install size | ≤ 25 MB | — | Not Started |
| NFR-009 | Colour fidelity | ICC kept, Δ ≤ 1 | ICC kept 25/25, ΔE2000 mean 0.19 (sRGB) / 0.31 (Display P3), detail and saturation kept; web-editor export for reference 1.39 / 2.15 ([RESULT.md §9](RESULT.md#9-print-quality)) | In Progress |
| NFR-010 | Cold start | < 1.5 s | — | Not Started |
| NFR-011 | Robustness | 0 crashes | — | Not Started |
| NFR-012 | Culling navigation | < 100 ms p95 | — | Not Started |
| NFR-013 | Project safety | ≤ 5 s, atomic | autosave ≤ ~1 s, atomic + .bak | In Progress |
| NFR-014 | Best-shot analysis speed | ≤ 20 ms/photo (≤ 40 ms with faces) | 17.4 ms p50 / 18.0 ms p95 without faces, 27.8 ms p50 / 36 ms p95 with faces, one thread (`measure_folder --analyze`) | In Progress |
| NFR-015 | Normal mode speed | ≥ 3× Professional export (≥ 50 photos/s), first screen < 0.5 s for 1,000 photos | — | Not Started |

## Security tasks

Defined in [SECURITY.md](SECURITY.md).

| # | Task | Priority | Status |
|---|------|----------|--------|
| S1 | 4-layer input/metadata pipeline (`crates/core/src/import.rs`, `metadata.rs`) | P0 | In Progress |
| S2 | Tauri capabilities + CSP (`src-tauri/capabilities/`, `tauri.conf.json`) | P0 | Done |
| S3 | Export path sanitizer + root check (`crates/core/src/export.rs`) | P0 | In Progress |
| S4 | `cargo deny` / `cargo audit` / `npm audit` in CI | P1 | Done |
| S5 | Fuzz targets (`crates/core/fuzz/`) | P1 | Not Started |
| S6 | DOM-API grep gate + network-free core check | P1 | Done |
| S7 | Panic hook + user-facing error mapping | P2 | Not Started |

## Open decisions

| Decision | Options | Owner | Notes |
|----------|---------|-------|-------|
| `REF-WIN` reference machine | — | @hoangphi01 | Needed for NFR measurement |

Decided:

- Project license: **GPL-3.0-only**, with an option to relicense later ([ADR-0008](adr/0008-license-gpl3-with-relicensing-option.md)).
- Code signing: **unsigned** for now ([ADR-0009](adr/0009-unsigned-distribution.md)).

## Licensing tasks

| Task | Status | Notes |
|------|--------|-------|
| `LICENSE` (GPL-3.0) | Done | |
| CLA set up (e.g. CLA Assistant) | Not Started | **Required before merging the first outside PR** ([ADR-0008](adr/0008-license-gpl3-with-relicensing-option.md)) |
| `cargo deny` license allowlist (permissive only, GPL banned) | Done | `deny.toml` |
| SPDX headers in source files | Done | All hand-written `.rs` / `.ts(x)` files |
| Apply to SignPath Foundation (Windows signing) | Not Started | After the first public releases |

## Dependencies to watch

Native libraries pinned for security tracking (versions to be filled in at scaffold time):

| Library | Pinned version | Notes |
|---------|----------------|-------|
| libjpeg-turbo | — | via `turbojpeg` crate |
| objc2-image-io / windows | 0.3.2 / 0.62.2 | HEIC through ImageIO / WIC in `fotollection-oscodec` ([ADR-0017](adr/0017-heic-through-os-codecs.md)) |
| tract-onnx | 0.23.9 | Face models for best-shot suggestions ([ADR-0018](adr/0018-face-models-and-runtime.md)); models are parsed from bundled, checksummed bytes only |
| lcms2 | — | |

## Blockers

None.
