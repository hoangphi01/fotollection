# Status — Fotollection

> **Last updated:** 2026-11-04
>
> Living progress tracker. Update it **in the same commit** as the work ([ADR-0006](adr/0006-docs-as-code.md)).
> Statuses: `Not Started` · `In Progress` · `Blocked` · `Done` · `Withdrawn`. An FR is `Done` only when its `fr_NNN_*` test passes ([TESTING.md §2](TESTING.md#2-requirement-traceability)).

## Current phase: Phase 1 — MVP, v0.9.4 released (2026-11-04); next v0.10.0 "Core"

**Released** (details in [CHANGELOG.md](../CHANGELOG.md), measurements in [RESULT.md](RESULT.md)): v0.1.0 to v0.8.0 (2026-10-24 to 10-27); **v0.8.1** green-stripe JPEGs fixed ([ADR-0033](adr/0033-reference-jpeg-decoder.md)); **v0.8.2** Linux, AppImage and `.deb` for x86_64 and ARM64, installed with `install.sh` or the signed APT repository at fotollection.org/apt ([ADR-0034](adr/0034-linux-lean.md), [ADR-0035](adr/0035-linux-install-from-fotollection-org.md)); **v0.8.3** Export what you ticked (FR-050) and Report a problem (FR-069), all three from `release/0.8` (2026-11-01); **v0.9.0 "Three modes"** from `main` (2026-11-02): Simple, Collage and Pro (FR-048), two photos on one print with Auto turn (FR-035), the 3 mm border on every print (FR-067), three date stamp sizes (FR-004). v0.9.0 was released at the owner's call without the manual checks of TESTING §7 on Windows and a real collage print: those stay open below. **v0.9.1** (2026-11-02): decoding fixes (cut-short, 16-bit grey and CMYK JPEGs, DNG active-area origin), `ERR-` codes on every error, the offline error log and *Copy diagnostics*, dismissible notices. **v0.9.2** (2026-11-03): zooming into a RAW no longer hangs every preview after it (a develop on its own thread, LESSONS 2026-11-03), the first zoom of a RAW at once, a failed preview shows its `ERR-` code, the language in the version popover. **v0.9.3** (2026-11-03): image facts in the preview, unusual files badged and filtered (FR-071); thumbnails, previews and zoom in sRGB, PNG gamma and dithering, share exports in linear light, straightened reductions without bands, the HEIC profile on Windows (FR-072). **v0.9.4** (2026-11-04): postcards in Collage mode, one photo and a postcard back with a stamp box, writing lines and typed text (FR-073).

**What the app does today.** JPEG, PNG, HEIC and camera RAW (RW2, CR2, CR3, NEF, ARW, RAF, ORF, DNG) on macOS, Windows and Linux; open with, folders as tabs, culling with stars, flags and best-shot suggestions, the print basket, crop, quarter turns and straightening, the film-style date stamp with place and caption, the "Shot on" info frame, offline place names, camera clocks, dates and places by hand, XMP from Lightroom, collage sheets, export for print and for social media, the `.fotollection` project, Check for updates and Report a problem. English and Vietnamese. Measured performance meets every budget ([RESULT.md](RESULT.md)).

**Shipping.** A version change on `main` (or a dispatch on a `release/*` branch) builds, tags and publishes the macOS `.dmg`, the Windows `-setup.exe` and `.msi`, the Linux AppImages and `.deb` for x86_64 and ARM64, and `SHA256SUMS.txt`, after a Linux smoke test ([RELEASE.md](RELEASE.md), [ADR-0021](adr/0021-versioning-and-automated-releases.md)). The website at <https://fotollection.org/> updates its downloads, What's new and the APT repository by itself ([site/README.md](../site/README.md)). [DOCS_GUIDE.md](DOCS_GUIDE.md) is the checklist of what to update for each change.

### Next, in order (plan of 2026-11-02, the audit before v1.0.0)

The audit of 2026-11-02 read the RAW path, the JPEG and colour path and the docs; its findings are summarised in [ADR-0037](adr/0037-our-own-develop-core.md) and the rows below. **Every session starts here: take the first open box.** Docs move in the same commit as the code ([DOCS_GUIDE.md](DOCS_GUIDE.md)); a session ends with this list true.

1. **Session 1: docs in sync (docs only).**
   - ✓ The plan written into the PRD (FR-071, FR-072; FR-070 moved to v0.11.0), [ADR-0037](adr/0037-our-own-develop-core.md), the ROADMAP rows v0.9.1, v0.10.0 and v0.11.0, and this list.
   - ✓ [ERROR_CONTROL.md](ERROR_CONTROL.md): the `ERR-<KIND>-<NNNN>` dictionary of all 80 current errors.
   - ✓ The audit's stale and contradicting statements fixed (STATUS, ROADMAP phases, README, USER_GUIDE, ARCHITECTURE, SECURITY, TESTING, OPTIMIZATION, DATABASE, site copy, ADR index).
   - ✓ `check-docs.sh` checks that ROADMAP and STATUS give every FR the same phase, and that a commit changing code also changes docs.
2. **Session 2: v0.9.1, decoding fixes.** Fixtures first in `crates/core/tests/fixtures/jpeg/` (16-bit lossless grey, Adobe CMYK, YCCK, progressive with a non-interleaved DC scan, restart intervals on 4:4:4 and 4:2:2, a cut-off file), each test failing on the old code, then:
   - ✓ 16-bit grey reads the high byte of native-endian samples (`codec.rs`, jpeg-decoder's `to_ne_bytes`);
   - ✓ CMYK: a fixture proves zune's conversion, the jpeg-decoder fallback no longer comes out negative, no grey or CMYK ICC profile on RGB output; ✓ YCCK: `ycck.jpg` (written by `ycck.c` on libjpeg) decodes right on zune (session 3);
   - ✓ a header `jpeg_layout::read` cannot parse goes to the reference decoder; SOF1 leaves `zune_proven`; `extended-sof1-420.jpg` decodes right on zune, but SOF1 stays off until a fixture with four Huffman tables;
   - ✓ the corpus test checks the mean and 99th-percentile error and the route of each file (zune, reference, refused); a JPEG cut short is refused (`ERR-IMG-0006`);
   - ✓ `ErrCode` in `IpcError` and the core's errors, in the bindings and on screen; the check fails on a code without its row.
   - ✓ `import.raw_codec_missing` (ERR-EXT-0003) raised again by `raw.rs` `first()` when every reader passed and Windows has no codec, with a current message (session 3).
3. **Session 3: v0.9.1, then release.**
   - ✓ DNG: `CFAPattern` and `BlackLevel` count from the ActiveArea's corner (they counted from the image's: an odd area swapped red and blue), black levels repeated 1×2 or 2×1 read; synthetic DNG tests;
   - ✓ "4:4:4" said only of print exports (README, site, done in session 1); ✓ `thumbs::CACHE_VERSION` 3, so thumbnails made from 16-bit grey or cut-short JPEGs are made again;
   - ✓ the offline error log (`error_log.rs`, 5 × 1 MB, paths and file names cut to their extension), *Copy diagnostics* under *Report a problem*, the panic hook writing `ERR-RUS-0005`;
   - ✓ release v0.9.1 (RELEASE.md §6), 2026-11-02; the release run was started again by hand after a yanked crate (LESSONS).
4. **Session 4: v0.10.0, FR-071 image facts.** ✓ (2026-11-03, plus the DNG kinds in [raw/dng.md](raw/dng.md)) `jpeg_layout::read` reads APP14, the ICC description and DRI; RAW gives its sensor kind; `PhotoRecord` keeps them (project schema bump); the inspector shows "4:2:0 · Display P3" or "X-Trans · 14-bit", with a grid badge for unusual files.
5. **Sessions 5–6: v0.10.0, colour management everywhere.** ✓ screen images in sRGB (`colour::screen`, transform cache; thumbnails, previews, zoom, the crop source, collage sheets; a DNG's embedded JPEG), the owner chose sRGB over tagged P3 (2026-11-03). Left by choice: the best-shot analysis reads unconverted pixels (a full-size conversion would break NFR-014, and it compares photos of one burst); rawler's embedded previews carry no profile to convert. ✓ PNG: gAMA/cHRM/sRGB, 16 → 8-bit ordered dither, alpha in linear light. ✓ Linear-light resizing for share exports (the owner's choice after measuring). ✓ Straighten aliasing: bands past 2× fixed (LESSONS). ✓ WIC hands over the frame's ICC profile (HEIC, RAW through the Raw Image Extension); built and linted for Windows, to check by hand on a Windows machine (TESTING §7, item 8). ✓ Carried over from session 4: the facts in the error log's context, HEIC facts. Previews, thumbnails, zoom and the crop source converted for the screen and served with their profile; PNG gAMA/cHRM/sRGB and dithering; linear-light resizing within NFR-002; straighten prefiltered; tests with real P3 and Adobe RGB profiles.
6. **Next: export on large photos (NFR-003, NFR-004).** ☐ The real trip of 2026-11-03 ([RESULT.md §20](RESULT.md#20-a-real-trip-prague-gx9)), 422 files of 20 MP JPEG (about 11 MB each) and RW2: export 7.6 photos/s (budget 8 at 24 MP) and peak RSS 2.14 GB (ceiling 2 GB), against 18.6/s and 1.41 GB on the 16 MP Canon set. Measure first where the peak sits (T1 leaves 1.27 GB resident before export starts; six 20 MP images in flight), then fix the cause: in-flight work sized by megapixels rather than a fixed 6, or memory returned after the thumbnails. The website says both numbers miss, until this box is ticked.
7. **Sessions 7–9: v0.10.0, FR-072 develop core.** ☐ Test charts and an artifact score first, with Malvar, PPG and bilinear X-Trans as the baseline (RESULT); the sensor kind enum; black levels, Gr/Gb, hot pixels; RCD and Markesteijn; false colour and highlights; the linear working space; the JPEG look match as an option; an independent reference, golden images, fuzzing, the RAW corpus in CI. Then the DNG kinds of [raw/dng.md](raw/dng.md), in its order: the phone gain map (`OpcodeList2`, a Pixel sample first), lossy LinearRaw, ProRAW, the digest check, float merges, X-Trans DNGs. Samples wanted: a Pixel or Samsung DNG, a lossy DNG, an HDR merge, a ProRAW file, a DNG edited in Lightroom.
8. **Session 10: release v0.10.0 "Core"**, checked by hand on macOS **and Windows**.
9. **Then:** v0.11.0 "Looks" (FR-070, [ADR-0036](adr/0036-colour-looks-cube-luts-first.md)); the Photo Style sliders (FR-039) and film looks (FR-047) on the same engine; towards v1.0.0, the P0 work below.

**Towards v1.0.0** ([ROADMAP.md](ROADMAP.md) Phase 1 exit criteria): P0 with code left, FR-003 (day grouping), FR-008 (lossless crop), FR-010 (metadata options), FR-022 (date, camera, GPS and DPI filters), FR-025 (mixed sizes), FR-026 (grid badge, pre-export summary), FR-027 (conversion for prints), FR-028 (safe area); NFR-005, 006, 011 and 012; NFR-008 by one `.dmg` per architecture; `REF-WIN` defined and measured; fuzzing (S5) and the panic hook (S7); the CLA.

**Still open from earlier releases:** the manual checks of TESTING §7 for v0.9.0 on Windows and a real collage print (FR-035, FR-067); about 20 real GX9 + phone pairs to score FR-031's thresholds; XMP crop and angle wait for a real Lightroom sidecar; more Simple and Collage use on a real folder.

**Alongside, never blocking a release:** the append-only edit journal (NFR-013; a full save stays at 70 ms for 70,000 edited photos); the Windows title bar on a real machine (FR-052); SignPath signing for Windows (declined 2026-10-26, reapply later, RELEASE.md §3); a Ko-fi link once the account exists.

**Camera over USB (FR-062, FR-063, FR-064; [ADR-0026](adr/0026-cameras-through-the-os-usb-stack.md), [ADR-0028](adr/0028-camera-sessions-saved-on-demand.md)).** Capture works on Windows with the Sony a6400 ([cameras/sony.md](cameras/sony.md), [RESULT §13](RESULT.md#13-camera-over-usb-step-0-fr-062-fr-063)). Open concerns:

1. Touch to focus needs a USB recording of Remote setting the focus point first (clean-room rule).
2. The new-photo event `0xC201` does not come through WPD; shots are noticed by polling `0xD215` every 100 ms.
3. A new file format applies one shot late; the delay is not measured.
4. macOS: the transport must run on the main thread, so *Connect camera* is disabled there.
5. Other Sony bodies are untested; a read-only probe is planned (a friend's ZV-E10 first).
6. To check by hand: *Save all* to another drive, closing a session tab, unplugging during live view, the Vietnamese UI, a print ratio that is not 3:2.
7. To measure in the app: NFR-018 (press to preview) and NFR-019 (frames per second).
8. Missing fuzz targets for `ptp` and `sony`.
9. Open decision: a Wi-Fi exception to NFR-006 for bodies like the GX9 (a direct link to the camera's own network, never the internet).

### To check by hand first (not covered by automated tests)

- Save As → Open → rename the photo folder → Relink, in the running app, in English and in Vietnamese.
- The unsaved-changes question on New and Open, and the window title showing *Edited*.
- Double-click the project name to rename it: a saved project (file renamed in Finder / Explorer) and an untitled one (the name is offered by Save).
- Best-shot flow: Find best shots → Groups → Compare → Keep best / Keep this one → undo, in English and Vietnamese, and a narrow window.
- Double-clicking a `.fotollection` file. This needs a release build (`npm run build`), because file associations are not active in `npm run dev`.

## Phase 0 — Foundation

| Task | Status | Notes |
|------|--------|-------|
| Git repository | Done | `hoangphi01/fotollection` (public), branch `main` |
| Documentation framework + docs CI | Done | This commit set |
| Cargo workspace + Tauri 2 scaffold | Done | Tauri 2.12, Rust 1.99.1 pinned |
| UI scaffold: React + Tailwind + shadcn/ui + Lucide | Done | [ADR-0007](adr/0007-ui-react-shadcn-lucide.md) |
| Build-size profiles in `Cargo.toml` + `scripts/clean.sh` | Done | First dev build: `target/` ≈ 1.0 GB ([DEVELOPMENT.md §4](DEVELOPMENT.md#4-build-disk-hygiene)) |
| CI build/test matrix (macOS, Windows, Ubuntu 22.04) | Done | `.github/workflows/ci.yml`; Ubuntu since v0.8.2 (ADR-0034) |
| Reproducible setup (`scripts/bootstrap.sh` / `.ps1`, lockfiles) | Done | [DEVELOPMENT.md §1](DEVELOPMENT.md#1-setting-up-a-machine) |
| `tauri-specta` bindings + staleness check | Done | `cargo test` regenerates; CI runs `git diff --exit-code` |
| Benchmark harness + `bench` corpus | In Progress | `examples/measure_folder.rs` on a real folder, plus `bench/run.sh`, the Rust vs Chromium comparison (OPTIMIZATION §2.1). `criterion` benches pending |
| Codec / EXIF crate spikes | In Progress | Pure-Rust codecs adopted for now ([ADR-0012](adr/0012-pure-rust-codecs-first.md)). turbojpeg comparison pending |
| libheif static build on Windows | Withdrawn | HEIC uses the OS codecs instead ([ADR-0017](adr/0017-heic-through-os-codecs.md)) |
| Release workflow (unsigned) | Done | `scripts/release.sh` + `release.yml` ([ADR-0021](adr/0021-versioning-and-automated-releases.md)) |
| Benchmark budget gate (`bench/budgets.toml`) | Not Started | Needs harness |
| Enforce "Done FR has test" in `check-docs.sh` | Not Started | Needs code |

## Functional requirements

| ID | Requirement | Phase | Status | Notes |
|----|-------------|-------|--------|-------|
| FR-001 | Import photos | 1 | Done | JPEG, PNG and HEIC (folder dialog + drag-and-drop), magic bytes, size + polyglot checks. PNG 8/16-bit, grey, palette, alpha on white, ICC kept (`fr_001_png_*`). HEIC via ImageIO (macOS) / WIC (Windows, needs the HEIF+HEVC Store extensions, otherwise `import.heic_codec_missing`) ([ADR-0017](adr/0017-heic-through-os-codecs.md), `fr_001_heic_*`) |
| FR-002 | Read photo metadata | 1 | In Progress | Allowlisted EXIF (date, offset, GPS, orientation, camera) + validation. `fr_002_*` tests pass |
| FR-003 | Library grid | 1 | In Progress | Virtualized grid, T0/T1 thumbnails, skeleton + fade-in, task progress bar with %. Day grouping and multi-select pending |
| FR-004 | Date stamp | 1 | In Progress | 3 formats, 4 corners, white/retro-orange, small/medium/large (`stamp.size`, v0.9.0, `fr_004_the_stamp_comes_small_medium_or_large`), inside the safe area. More styles with FR-033 |
| FR-005 | Location stamp (offline) | 1 | In Progress | ✓ Bundled GeoNames places (162k towns, 5.4 MB, BLAKE3-checked) and `geocode::reverse` / `search`, about 5 µs per lookup (`fr_005_*`, `fr_032_search_*`). ✓ Place on the date line and in the info frame, from GPS, three formats (settings v5). ✓ Places on the info frame. ✓ Renaming a place everywhere: Places card, `list_places`, `rename_place`, project v8 `places[]` (`fr_005_a_renamed_place_keeps_its_country`). Left: borrowed places (FR-031), check by hand |
| FR-006 | Crop to print ratio | 1 | In Progress | Crop editor with edge labels (cm/in + px), ratio pill and live DPI; centre + zoom storage. ✓ Lossless quarter turns with `[` `]`, crop follows (`fr_006_*`, project v4). ✓ Straighten ±15° in the crop editor, inscribed crop, crop-only bicubic resampling (project v5) |
| FR-007 | Live preview | 1 | In Progress | Preview dialog with inspector (capture, camera, image, GPS, print DPI, stamp controls). Rust-rendered, warm p95 18 ms (p50 17 ms) |
| FR-008 | Original-quality export | 1 | In Progress | Full resolution, q95 4:4:4, ICC copied. Lossless crop-only path pending |
| FR-009 | Export by date | 1 | Done | `YYYY-MM-DD/` folders, never overwrites (`_1`, `_2`…). `fr_009_*` passes |
| FR-010 | Metadata privacy on export | 1 | In Progress | Every export writes an allowlisted EXIF block (camera, lens, exposure, date, `Software`), never GPS, read back in tests (`fr_010_prints_keep_the_camera_details_but_never_gps`, `fr_053_*`). Options to keep more or less for prints pending |
| FR-011 | Batch progress, cancel and recovery | 1 | Done | Progress events, cancel removes partials, per-photo failures. `fr_011_*` pass |
| FR-012 | Trip auto-grouping | 2 | Not Started | |
| FR-013 | Face-aware smart crop | 2 | Not Started | |
| FR-014 | Export by trip | 2 | Not Started | |
| FR-015 | Duplicate detection | 2 | Not Started | |
| FR-016 | RAW input | 3 | In Progress | ✓ OS codecs with type hints and embedded previews (`oscodec`), recognition by content (`raw.rs`), EXIF for all eight formats (`metadata::read_raw`), thumbnails and analysis on the preview, one develop at a time; RW2 imports and exports end to end (`fr_016_rw2_imports_and_exports`); checked by the owner on Windows with the Raw Image Extension (v0.7.0-beta.1). Left: website screenshots of RAW |
| FR-017 | Photo book layout preview | 3 | Not Started | |
| FR-018 | QR code on photo back | 3 | Not Started | |
| FR-019 | Print service upload | 3 | Not Started | |
| FR-020 | Localization | 1 | Done | English (default) + Vietnamese via i18next ([ADR-0015](adr/0015-ui-localization-i18next.md)): every UI string, Rust error codes, native menu, window title and file dialogs; selector in the version popover of every mode, stored in `settings.json` v3; `check-i18n` gate in lint. German later |
| FR-021 | Manual updates | 3 | In Progress | No updater by design (ADR-0021). ✓ Version shown at the foot of the settings panel. ✓ `scripts/release.sh` + `release.yml` (build, tag, publish on a version change). ✓ Website `site/` (EN + VI, Jekyll on Pages, OG cards, star link, screenshots of the real app on CC0 demo photos) |
| FR-022 | Culling (rate, flag, filter) | 1 | In Progress | Stars `0`–`5`, `P`/`X`/`U` in grid and preview, undoable; filter by flag + min rating, export = filtered set (`fr_022_*`). Pending: date/camera/GPS/DPI filters, multi-select, loupe prefetch (NFR-012) |
| FR-023 | Projects, autosave and undo | 1 | Done | `.fotollection` document like `.docx`: Save / Save As / Open / New with Rust-side dialogs, unsaved-changes question, title `Trip — Edited`, native File menu (macOS) and Ctrl shortcuts (Windows), Relink dialog for moved folders, file association. Self-contained ZIP bundle ([ADR-0016](adr/0016-self-contained-project-bundle.md)); fonts installed once, ids remapped; session autosave underneath. `fr_023_*` pass (incl. open on another machine, relink). File association to be checked by hand in a release build |
| FR-024 | Copies and order summary | 2 | Not Started | |
| FR-025 | Mixed print sizes per order | 1 | Not Started | Builds on `PrintSize` |
| FR-026 | Resolution warning | 1 | In Progress | Effective DPI + band shown in the inspector. Grid badge and pre-export summary pending. ✓ Size advice under the print-size picker and green/yellow/red badges per size (`print_advice.rs`, `fr_026_*`) |
| FR-027 | sRGB conversion for labs | 1 | In Progress | ✓ Share exports convert to sRGB with `qcms` (perceptual, [ADR-0022](adr/0022-srgb-and-exif-for-share-exports.md)). Pending: the optional conversion for prints |
| FR-028 | Safe area and borders | 1 | Not Started | |
| FR-029 | Output sharpening | 3 | Not Started | Needs real print tests |
| FR-030 | Camera clock offset | 1 | In Progress | ✓ Bodies by make, model and `BodySerialNumber`; effective time = camera time + offset in `Library::record` (sort, groups, stamp, day folders, export EXIF follow), camera time kept (`fr_030_*`, [ADR-0027](adr/0027-effective-time-and-place.md)). ✓ Project v7 `cameras[]`, checked on load. ✓ `list_cameras`, `set_camera_clock`, `clock_offset_between`, `clock_match_candidates`; Camera clocks card and the helper in the preview's Captured card. Left: time zone in the UI (with GPX, FR-031), automatic suggestion from same-scene pairs, check by hand |
| FR-031 | Borrow GPS from a phone or GPX track | 2 | In Progress | ✓ `Library::borrow_candidates` (other bodies with GPS, within the window, 4 nearest), `places::borrow` (colour then layout over 5 zooms, early stop), high applied as one undoable step after each analysis or search, medium suggested (`fr_031_*`). ✓ *Find places* in the Places card, review dialog, borrowed badge. 54 ms / 1.11 s worst case per 1,000 (RESULT §17). Left: score 20 real GX9 + phone pairs to set the thresholds; GPX is FR-066 |
| FR-032 | Manual date and place | 1 | In Progress | ✓ `PhotoEdits.date` and `.place` (undoable), effective date and place per ADR-0027 (`Library::effective`, `places::effective`), project v8 (`fr_032_*`). ✓ `search_places`, `set_photo_date`, `set_photo_place` checked in Rust; Set the date / Set the place in the preview, Date and place… in select mode, offline place picker (shadcn `command`). Left: check by hand |
| FR-033 | Retro film date stamp and stamp presets | 1 | In Progress | ✓ DSEG14 Italic (OFL) LCD face with LED glow and faint shadow (`fr_033_*`). Presets to come |
| FR-034 | Lab presets | 2 | Not Started | |
| FR-035 | Multi-photo layouts | 3 | In Progress | **v0.9.0: 2-up collage** ([ADR-0032](adr/0032-collage-sheets-from-two-cell-plans.md)). ✓ Core: `sheet.rs` (cells, margin, suggested and typed size, cut line, pairing), `render.collage` (settings v10), `photos[].collage` and `sheet_turned` (project v9), `fotothumb://…/sheet/{a}/{b}`, `list_sheets`, `set_collage_order`, `turn_sheet`, `sheet_info`, export one file per sheet (`fr_035_*`). ✓ The Collage screen: `SheetGrid` (virtualized, drag on pointer events, ← → ⌥ ← → R, Enter / Space / double-click open the preview), `CollagePanel` (sheet size without Original, `CollageCellsCard`, date stamp, export of sheets), `SheetPreviewDialog` in the preview's frame (zoom, ← → 1 2 C R, crop per photo, Export this sheet: `start_export` with `only` + `sheets`; ↑ / ↓ and a basket button per photo); select mode and the selection bar reused from the gallery, basket badges on cells. Audit of 2026-10-31 fixed: one ICC profile per sheet (sRGB when the photos differ), preview area matching export, photos added later at the end, turns reset on reorder, half the exports in flight, quality from the photo's own crop, no cut line on a lone photo, the bordered preview within its long edge, `sheet::border_inset`, drag listeners attached once. Left: check by hand, a real print. 4-up and strips later ✓ 2026-11-02: Rotate photo keeps the sheet (`sheet::layout_turned`); select mode ticks whole sheets, one frame each, and Export writes only the ticked sheets; **Auto turn** / **Revert** (`auto_turn_collage`, `sheet::landscape_turned`, `sheet::fitting_turns`); toolbar keys behind ⓘ (`KeysInfo`). |
| FR-036 | Custom captions | 2 | In Progress | ✓ A caption line under the date, in the stamp's LCD face (`fr_036_*`). ✓ Per photo (v0.9.0, `photos[].caption`, project v9), and per photo on a collage sheet. Variables (`{trip}` needs FR-012) to come |
| FR-037 | Contact sheet | 3 | Not Started | |
| FR-038 | Print presets | 2 | Not Started | Storage model: [DATABASE.md §7](DATABASE.md#7-presets) |
| FR-040 | EXIF info frame | 1 | In Progress | Crop/Fit at print ratio, field switches, per-camera overrides (persisted), user PNG logo, bundled logos of 12 brands (Noritsu film scans added 2026-10-25), a long lens wraps clear of the logo (`fr_040_a_long_lens_*`), Fit frames the full-size photo with a 5 mm border and the bar, instax-style, and shows the resulting print size (`fr_040_crop_keeps_the_print_and_fit_adds_the_bar_below`), APEX shutter fallback, with Classic / Brand colours / One colour styles ([ADR-0020](adr/0020-bundled-brand-logos.md), `fr_040_*`). Place needs FR-005 ✓ Logo size slider, the same for every photo (settings v7, `fr_040_logo_scale_*`) |
| FR-041 | Best-shot suggestions | 1 | In Progress | Models: YuNet (MIT) + FaceMesh V2 (Apache-2.0, ONNX) on `tract-onnx` ([ADR-0018](adr/0018-face-models-and-runtime.md)). Core `culling/` done (`fr_041_*`): bursts grouped by time, camera, GPS and a layout + colour signature that allows for zoom ([ADR-0019](adr/0019-grouping-by-visual-signature.md)); sharpness, shake, exposure, noise; eye aspect ratio; robust z-score with reasons. Scored on 95 hand-labelled photos: 15/22 groups, pair precision 100 %, picks 6/6 (`eval_bestshot`). Analysis job, cache, IPC and the Groups / compare UI done. Left: manual check in the app, `SubSecTimeOriginal` |
| FR-042 | Custom fonts | 1 | Done | OTF/TTF import (Rust dialog, validated, copied), per stamp and per frame, remove → Inter. `fr_042_*` pass |
| FR-043 | Custom print sizes | 1 | Done | cm/in entry, stored in mm, used by crop/frame/DPI/export. `fr_043_*` pass |
| FR-044 | Per-photo overrides | 2 | In Progress | ✓ Date stamp per photo from the preview, Apply to all, Reset in the settings panel, undoable, saved in project v3 (`fr_044_*`). Other settings later |
| FR-045 | Original aspect ratio | 1 | Done | `print_size: "original"`: photo's own ratio, 10 cm short edge, exact pixel crop, framed canvas at photo ratio (`fr_045_*`) |
| FR-046 | Swipe culling | 2 | Not Started | v0.9.0: the full-screen ↑ keep / ↓ drop loop is the Simple mode (FR-048). Later: progress resume, best-shot pre-sort, the workflow change |
| FR-047 | Film looks | 2 | Not Started | Future: Fujifilm-style film simulations (slide, chrome, negative, cinema, mono) on the FR-039 engine; own names and LUTs, no trademarks or copied LUT files |
| FR-048 | Simple, Collage and Professional modes | 1 | In Progress | ✓ `ui.mode` (settings v10), `ModeSwitch` at the top of the sidebar. ✓ Simple: `SimpleView` (one photo as it prints, ↑ keep / ↓ drop, ← →, Keep / Drop buttons, progress) and `SimplePanel` (size, 3 mm border, date stamp, export of the basket); `sendTo` and `useShownSrc` shared with the preview. ✓ Collage (FR-035). ✓ Cmd/Ctrl+1/2/3 and a View menu on macOS. Left: check by hand. The faster Simple export (NFR-015) later |
| FR-050 | Print basket | 1 | In Progress | ✓ Print basket and set aside (`Queue`, project v6, `set_photo_queue`, `fr_050_*`), sidebar rows with counts, ↑ ↓ and the basket button in the preview, select mode with Shift-range and the action bar; export follows the view. Later: "also export the basket" folder, copies |
| FR-052 | Appearance: theme, title bar, hints | 1 | In Progress | ✓ System/Light/Dark switch (settings v6, `fr_052_*`), ✓ Windows title bar drawn by the app (decorations off, window permissions), ✓ `HintLayer` replaces native tooltips, ✓ one accent per mode (teal light, wood dark). Left: check the title bar on a real Windows machine |
| FR-051 | Landmark recognition | 2 | Not Started | Future: offline landmark model as an optional pack; licence research done (Google on-device classifiers, Apache-2.0); needs an ADR and a size/speed spike |
| FR-053 | Export for social media | 1 | Done | Print / Share target (settings v8), Original, Facebook 2048, Instagram 1080 (3:4 cap), custom 640–6000 px (`fr_053_*`), sRGB via `qcms`, unsharp 0.2 after downscale, q88 (Original q80), EXIF allowlist with `Software = Fotollection`, no GPS, `Share/` folder. Measured 2026-10-25 (RESULT §10): SSIM ≈ 0.98, Original ΔE 0.96 at 46 % of the file, after Facebook P3 colour 99.6 % vs 86 % |
| FR-054 | Open photos with Fotollection | 2 | In Progress | ✓ `open_with.rs`: one route for macOS `Opened`, command-line paths and second launches (`tauri-plugin-single-instance`); the folder imported without subfolders as a tab; the request kept until the UI takes it (`take_photo_request`, `OpenPhoto`). ✓ macOS "Open With" as `Alternate` (`tauri.macos.conf.json`); Windows `OpenWithProgids` through an NSIS hook and a WiX fragment (ADR-0024). ✓ Cold and warm launches from `open -a` on the release bundle; checked by the owner on Windows with the NSIS installer (v0.7.0-beta.1): listed under "Open with", not the default, one window, removed on uninstall. Left: the MSI's WiX fragment ships first in v0.7.0 |
| FR-055 | Export from the preview | 1 | In Progress | Export card in the preview inspector (same `ExportSection` as the panel, one shared job and destination), Export button on the photo; `start_export(filter, only, ticked, …)`. Left: an `fr_055_*` test (STATUS rule: Done needs its test) |
| FR-056 | Folders as tabs | 1 | Done | All photos row, click = one folder, select mode / Ctrl-⌘-click = several (`CullFilter.folders`), counts from `list_photos.folders`, groups scoped, export together or separately (`start_export(…, separate)`, `fr_056_*`); ticks follow the tabs in select mode (`setFilter(…, keepTicks)`, `SelectionBar` `elsewhere`) |
| FR-057 | Zoom in the preview | 1 | In Progress | ✓ Pinch / Ctrl-⌘ + − 0 about the pointer, 100–800 %, drag and two-finger pan, double-click 200 % (`use-zoom.ts`). ✓ Zoom buttons − · level · + · fit (`zoom-controls.tsx`). ✓ Past the preview's pixels, the visible part at full resolution, 150 ms after the view settles (`use-sharp-region.ts`, `PreviewCache::render_region`, originals cached ≤ 200 MB); measured first, RESULT §11. Left: check by hand in the app |
| FR-058 | Lightroom compatibility through XMP | 2 | In Progress | ✓ Read on import: `xmp::read_for` finds `name.xmp` / `name.ext.xmp`, `xmp:Rating` → stars or reject for new photos, the project still wins; ≤ 1 MB, no DTD (`fr_058_*`); the sidebar counts them. Left: crop and straightening (`crs:`) once checked against a real Lightroom sidecar; opt-in writing (needs an ADR) |
| FR-060 | Start-up screen | 1 | In Progress | ✓ Logo and loading bar painted by `index.html` before any script; the bar follows the import while the session reopens; fades out on `StartupFinished` (`use-startup-screen.ts`), never later than 15 s. Left: check by hand; asking for a mode (Simple, Collage, Pro) the first time is not planned for v0.9.0 |
| FR-061 | Check for updates | 2 | In Progress | ✓ `updates.rs`: one GitHub request on click (`ureq`, 10 s, 1 MB), `semver`, installer per system, links kept in Rust and limited to the project's releases, opened with `tauri-plugin-opener`; button in the version popover (`update-check.tsx`); tests on fixture JSON. ✓ MSI `UpgradeCode` pinned. Left: try in the app (`FOTOLLECTION_FAKE_VERSION`) |
| FR-062 | Live view from a camera | 1 | In Progress | ✓ Live view in the capture screen (Windows, Sony a6400; ADR-0028): frames from memory over `fotothumb`, the print ratio drawn over it, the settings under it, about 30 fps from the camera. Left: NFR-019 measured in the app, macOS, touch to focus, UVC |
| FR-063 | Tethered capture over USB | 2 | In Progress | ✓ Step 0 on macOS (RESULT §13): `crates/oscamera` with `ptp` (tested) and the ImageCaptureCore transport, `probe_camera`. ✓ a6400 protocol from USB captures on Windows (sony.md), ✓ `vendor/sony.rs`: handshake, shutter, settings block, live-view frame (tested). ✓ WPD transport, first shots and live view from our code on Windows (RESULT §13.1). ✓ In the app (v0.8.0, Windows): Connect camera, the capture screen, shots into a session tab, Save all and Delete photos (`camera.rs`, `fr_063_*`; ADR-0028). GX9: no capture or live view over USB at all. Left: the open concerns in "Next" item 9 (RAW codec on Windows, touch to focus, macOS, other bodies, manual checks) |
| FR-064 | Watch a folder for new photos | 2 | Not Started | Planned: fallback for a maker's wired tether app; also feeds FR-063's session folder |
| FR-065 | Border in Crop to fill | 1 | In Progress | ✓ `frame.border` (settings v9), 3 mm inside the print in `frame::layout`, the crop and export follow the smaller photo area (`fr_065_*`); the switch moved to the print-size card with FR-067 (v0.9.0). Left: check a print at a CEWE kiosk size by hand |
| FR-066 | Borrow places from a GPX track | 2 | Not Started | Split out of FR-031 for v0.8.0 |
| FR-067 | 3 mm border on every print | 1 | In Progress | ✓ `render.border` (settings v10, a v9 `frame.border` moves there), `plan::Mat` without a frame, Crop to fill as FR-065, Fit whole photo inside the print size (`frame::contain`), *3 mm border* switch in the print-size card (`BorderSwitch`), `fr_067_*`. Left: collage cells (FR-035); check a print by hand |
| FR-069 | Report a problem | 1 | In Progress | `ReportIssue` in the versions popover (`PanelFooter` shows the version in every mode), `open_issue_page` (fixed URL, opener from Rust); the website footer's `.button.report`. v0.8.3. Left: an `fr_069_*` test that the address is the fixed one |
| FR-070 | Looks from LUT files | 2 | Not Started | Planned for v0.11.0, after the v0.10.0 core (owner, 2026-11-02): `.cube` import, built-in looks of our own, strength, per-photo override; one 33³ engine shared with FR-039 ([ADR-0036](adr/0036-colour-looks-cube-luts-first.md)) |
| FR-071 | Image facts | 1 | Done | 2026-11-03: `facts.rs` reads chroma subsampling, coding, bits, colour space (ICC name, EXIF R03), sensor filter and DNG kind at import, in RAM only (DATABASE N4); a RAW's sensor from rawler when the inspector reads the file. The inspector's Image card shows the line, the grid an ⓘ for `unusual()`, the flag filter *Unusual files*. Tests: `facts::tests`, `library::tests::fr_071_the_unusual_filter_keeps_unusual_files`, `dng::tests::facts_tell_the_dng_kinds_apart`, `colour::tests::profile_names_from_v2_and_v4_profiles`. HEIC (`colr`, `pixi`) and the facts in the error log's context followed in session 5. Left: a RAW other than DNG is badged only once opened |
| FR-072 | Our own develop core | 1 | Not Started | Planned for v0.10.0 (owner, 2026-11-02): sensor kind first, RCD and Markesteijn, Gr/Gb, false colour, highlights, linear working space, colour management everywhere ([ADR-0037](adr/0037-our-own-develop-core.md)). Sessions 5–9 |
| FR-073 | Postcard sheets | 1 | Done | Released in v0.9.4 (2026-11-04; owner, 2026-11-03). ✓ Core: `postcard.rs` (dashed stamp box, 20 × 25 mm since 2026-11-04 (26 × 32 in v0.9.4), text box, 3–4 lines, typed text in Xanh Mono or the user's font, wrapped), `sheet::layout` puts the one photo on the left or bottom half, one photo per sheet, Auto turn off; `set_postcard_text` / `get_postcard_text` checked in Rust (`ERR-IPC-0022`); settings v11, project v10. Tests: `postcard::tests`, `sheet::tests::fr_073_a_postcard_sheet_draws_one_photo_and_the_card`, `settings_store::tests::fr_073_v10_files_have_no_postcard_and_it_round_trips`, `project::tests::fr_073_v10_fixture_keeps_postcard_texts_within_limits`. ✓ UI: a labelled **Postcard** switch in the bar above the sheets next to Select (`SheetKindToggle`, owner's choice), then in the panel `PostcardCard` (held landscape or portrait, 3 or 4 lines with no box, the pen note) and `PostcardTextCard` (the chosen card's message and address, each with bold, italic, underline; font with Inter for Greek and Cyrillic, size, ink, credit line; the same card in a postcard's preview); the top band free for more stamps or a long post-office label, the lines together at the foot (stacked on a landscape card, two columns with a rule on a portrait one), pen note in the shared `PanelNote`), `PostcardTextField` in a card's preview (the shared `CommitInput`, now multiline), sheet turn and Auto turn hidden, no cut line; exported as `<photo>-postcard.jpg` (`pipeline::fr_073_postcards_export_one_file_per_photo`); manual §19.1, landing page section, screenshots `postcard`, `manual-postcard` and `postcard-night` (demo.rs records a bordered and a filled set). Still to do by hand: a real print and pen test (TESTING §7, item 9), not done before the release (owner's choice) |
| FR-074 | Postcard backs: colour and patterns | 2 | Not Started | The owner's idea (2026-11-03): a background colour or a pattern behind a postcard's lines and stamp box, the ink following it; after v0.9.4, not scheduled yet |
| FR-068 | HEIC on Linux through the system's libheif | 2 | Not Started | v0.8.2 skips HEIC on Linux with its own reason (`HeicNotOnLinux`); after v0.9.0 ([ADR-0034](adr/0034-linux-lean.md)) |
| FR-059 | Quick-preview | 1 | In Progress | ✓ Hold Space in select mode over a photo: thumbnail at once, then the render with the gallery's settings; ← → and Enter while held (`use-quick-preview.ts`). Left: check by hand in the app |
| FR-049 | Trip journal | 2 | Not Started | Future: offline report drafted from the album (days, places, counts, cameras, focal-length range, best shots per day), templates in EN/VI, editable, export HTML/PDF or a journal print; every editing session adds a journal entry (instax-style) |
| FR-039 | Photo Style looks | 2 | Not Started | [ADR-0011](adr/0011-photo-style-looks.md); "C200 Natural" built-in. After FR-070 (v0.10.0), on its engine ([ADR-0036](adr/0036-colour-looks-cube-luts-first.md)) |

## Non-functional requirements

| ID | Requirement | Budget | Last measured | Status |
|----|-------------|--------|---------------|--------|
| NFR-001 | Library load time | < 1 s first screen | 5,035 photos: import+EXIF 383 ms, T0 236 ms = 0.12 s per 1,000 ([RESULT.md](RESULT.md)) | In Progress |
| NFR-002 | Preview latency | < 50 ms p95 | 18 ms p95, 17 ms p50 (warm), faster than Chromium (18.5 ms) after the screen encoder | In Progress |
| NFR-003 | Export throughput | ≥ 8 photos/s | 18.0/s (16 MP, median of 3) | In Progress |
| NFR-004 | Memory ceiling | ≤ 2 GB | Core: 1.35 GB (95 photos), 1.45 GB (5,035 photos), flat. Whole-app run with 5,035 photos pending | In Progress |
| NFR-005 | UI responsiveness | < 100 ms | — | Not Started |
| NFR-006 | Offline and zero telemetry | 0 requests | — | Not Started |
| NFR-007 | Platforms | macOS 11+, Win 10/11, Linux glibc 2.35+ (x86_64, ARM64) | macOS and Windows shipping; Linux from v0.8.2, smoke-tested in CI on Ubuntu 22.04 and Debian 12 | In Progress |
| NFR-008 | Install size | ≤ 25 MB | v0.8.3: `-setup.exe` 17 MB, `.msi` 23.8 MB, `.deb` 22–24 MB ✓; universal `.dmg` 45.4 MB ✗ (two architectures in one file); AppImage 94–97 MB ✗ (it bundles WebKitGTK) | In Progress |
| NFR-009 | Colour fidelity | ICC kept, Δ ≤ 1 | ICC kept 25/25, ΔE2000 mean 0.19 (sRGB) / 0.31 (Display P3), detail and saturation kept; web-editor export for reference 1.39 / 2.15 ([RESULT.md §9](RESULT.md#9-print-quality)) | In Progress |
| NFR-010 | Cold start | < 1.5 s | page loaded 0.31–0.40 s, first thumbnail 0.50–0.58 s (release build, 4 runs); the very first launch after install 2.7 s, 2.1 s of it WebKit starting (`scripts/measure-startup.sh`) | In Progress |
| NFR-011 | Robustness | 0 crashes | — | Not Started |
| NFR-012 | Culling navigation | < 100 ms p95 | — | Not Started |
| NFR-013 | Project safety | ≤ 5 s, atomic | autosave ≤ ~1 s, atomic + .bak; full save 10 ms at 5,035 edited photos, 70 ms at 70,000 (`measure_project`) | In Progress |
| NFR-014 | Best-shot analysis speed | ≤ 20 ms/photo (≤ 40 ms with faces) | 17.4 ms p50 / 18.0 ms p95 without faces, 27.8 ms p50 / 36 ms p95 with faces, one thread (`measure_folder --analyze`) | In Progress |
| NFR-015 | Simple mode speed | ≥ 3× Professional export (≥ 50 photos/s), first screen < 0.5 s for 1,000 photos | — | Not Started |
| NFR-016 | Open to first photo | < 1 s after the first launch | measured end to end (`open -a`, release bundle): JPEG 0.70 s when its folder is already a tab, 1.25 s for a new folder (its import first); RAW 1.4–2.5 s (develop, NFR-017) | In Progress |
| NFR-020 | Place inference speed | ≤ 2 s per 1,000 camera photos | 54 ms per 1,000 when the scenes match, 1.11 s in the worst case (`measure_borrow`, RESULT §17) | Done |
| NFR-021 | Collage sheet speed | Preview ≤ 2× NFR-002; export ≤ its two photos alone; ten grid sheets after a change < 100 ms | Preview 24 ms (17 + 19 alone), export render + encode 214 ms (633 alone), grid 29 ms (`measure_sheet`, `measure_sheet_grid`, RESULT §18) | Done |
| NFR-018 | Shot to screen | ≈ 2 s for JPEG (to be set by Step 0) | — | Not Started |
| NFR-019 | Live view | ≥ 15 fps, < 200 ms (to be confirmed) | — | Not Started |
| NFR-017 | Camera RAW speed | preview < 100 ms, first RAW print preview < 2.5 s, memory within NFR-004 | embedded preview 9–80 ms, develop 0.39–2.17 s, 1.29 GB peak for one 31 MP develop (`measure_raw`, RESULT §12) | In Progress |

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
| S7 | Panic hook + user-facing error mapping | P2 | In Progress (v0.9.1: the hook logs `ERR-RUS-0005`, every error shows its `ERR-` code; the dialog after a crash is left) |

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
| Apply to SignPath Foundation (Windows signing) | Blocked | Declined 2026-10-26: not enough public adoption yet (stars, forks, independent articles or discussions). Reapply later; the policy page is ready |

## Dependencies to watch

Native libraries pinned for security tracking (versions to be filled in at scaffold time):

| Library | Pinned version | Notes |
|---------|----------------|-------|
| objc2-image-io / windows | 0.3.2 / 0.62.2 | HEIC through ImageIO / WIC in `fotollection-oscodec` ([ADR-0017](adr/0017-heic-through-os-codecs.md)) |
| zune-jpeg / jpeg-decoder | 0.5.15 / 0.3.2 | zune decodes only the layouts `tests/jpeg_layouts.rs` proves; retry 0.5.16 **stable** against that test before upgrading, never an RC ([ADR-0033](adr/0033-reference-jpeg-decoder.md)) |
| tract-onnx | 0.23.9 | Face models for best-shot suggestions ([ADR-0018](adr/0018-face-models-and-runtime.md)); models are parsed from bundled, checksummed bytes only |
| qcms | 0.3.0 | ICC conversion (share exports today; every screen surface in FR-072). `lcms2` only as a test reference for NFR-009 |
| rawler | 0.8.0 | LGPL-2.1, unpacking only from v0.10.0 ([ADR-0029](adr/0029-raw-through-rawler-then-our-own-decoder.md), [ADR-0037](adr/0037-our-own-develop-core.md)); its panics are caught |

## Blockers

None.
