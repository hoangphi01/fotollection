# Roadmap — Fotollection

> **Status:** Draft v0.1 · **Last reviewed:** 2026-11-09
>
> This file defines **when**: it assigns requirements to phases. What each requirement means is defined in [PRD.md](PRD.md). Progress is tracked in [STATUS.md](STATUS.md).
> Every FR in the PRD appears in exactly one phase below. `scripts/check-docs.sh` enforces this.

## Phase 0 — Foundation

**Goal:** a running empty app on macOS and Windows, a benchmark harness, and crate choices confirmed by data.

- Cargo workspace (`crates/core`, `src-tauri`, `ui/`) and Tauri 2 scaffold
- UI scaffold: Vite + React + TS, Tailwind, shadcn/ui init (theme tokens, base components), `lucide-react`
- CI: fmt, clippy, tests and docs checks on the macOS and Windows matrix
- `tauri-specta` binding generation, with a staleness check in CI
- Benchmark harness (`criterion`) and the `bench` corpus ([TESTING.md §3](TESTING.md#3-sample-corpus))
- Spikes, each logged in [OPTIMIZATION.md §5](OPTIMIZATION.md#5-benchmark-log): turbojpeg vs `zune-jpeg` decode, `nom-exif` vs `kamadak-exif`, libheif build on Windows
- Release workflow producing unsigned installers ([RELEASE.md](RELEASE.md))

**Exit criteria:** the empty app builds and launches on both platforms from CI artifacts, and baseline benchmarks are recorded.

## Phase 1 — MVP

**Goal:** go from a folder to a culled, correctly dated, colour-correct, print-ready export, fast and private.

| ID | Requirement |
|----|-------------|
| FR-001 | Import photos |
| FR-002 | Read photo metadata |
| FR-003 | Library grid |
| FR-004 | Date stamp |
| FR-005 | Location stamp (offline) |
| FR-006 | Crop to print ratio |
| FR-007 | Live preview |
| FR-008 | Original-quality export |
| FR-009 | Export by date |
| FR-010 | Metadata privacy on export |
| FR-011 | Batch progress, cancel and recovery |
| FR-022 | Culling (rate, flag, filter) |
| FR-023 | Projects, autosave and undo |
| FR-025 | Mixed print sizes per order |
| FR-026 | Resolution warning |
| FR-027 | sRGB conversion for labs |
| FR-028 | Safe area and borders |
| FR-030 | Camera clock offset |
| FR-032 | Manual date and place |
| FR-033 | Retro film date stamp and stamp presets |
| FR-040 | EXIF info frame |
| FR-041 | Best-shot suggestions |
| FR-042 | Custom fonts |
| FR-043 | Custom print sizes |
| FR-045 | Original aspect ratio |
| FR-048 | Simple, Collage and Professional modes |
| FR-050 | Print basket: the photos that will really be printed, exported to an extra folder |
| FR-020 | Localization (English and Vietnamese; German later) |
| FR-052 | Appearance: System / Light / Dark theme, Windows title bar in the app's colours, app-drawn hover hints |
| FR-053 | Export for social media: Original / Facebook / Instagram / custom sizes, sRGB, EXIF with the app's name and never GPS |
| FR-055 | Export the photo open in the preview, with the same export controls as the panel |
| FR-056 | Folders as tabs: All photos, one folder or several ticked, exported together or separately |
| FR-057 | Zoom in the preview: two-finger scroll, pinch, Ctrl/⌘ + − 0, drag to move, double-click for 200 % |
| FR-059 | Quick-preview: hold Space in select mode to look at a photo as it will print |
| FR-060 | Start-up screen: the logo and a loading bar until the last session is back |
| FR-062 | Live view from a camera plugged in by cable (own PTP driver, UVC webcam or HDMI capture card), with the print ratio drawn over it |
| FR-065 | Border in Crop to fill: 3 mm in the frame colour inside the print size, instant-photo look at the exact kiosk size |
| FR-067 | 3 mm border on every print: every mode and frame style, inside the print size (extends FR-065) |
| FR-069 | Report a problem: a red button in the app (*Settings → Report a problem* since v0.12.0) and the website's footer, to the GitHub issue forms (v0.8.3) |
| FR-071 | Image facts at import and in the inspector: chroma subsampling, colour space, sensor kind; shipped in v0.9.3, [ADR-0037](adr/0037-our-own-develop-core.md) |
| FR-072 | Our own develop core: sensor kind first, RCD and Markesteijn, Gr/Gb, false colour, highlights, linear working space, colour management everywhere; Bayer develop, neutral colour and the RAW sliders shipped in v0.10.0, X-Trans, GPU and DNG kinds in v0.15.0, [ADR-0037](adr/0037-our-own-develop-core.md) |
| FR-073 | Postcard sheets in Collage mode: one photo and a postcard back (stamp box, writing lines, typed text); v0.9.4 |

**Exit criteria:** all P0 FRs are Done; NFR-001 through NFR-013 are within budget on `REF-MAC` and `REF-WIN`; v1.0.0 is released. Until then, each work stage ships as a 0.y.0 preview release, starting with v0.1.0 ([RELEASE.md](RELEASE.md#1-versioning)).

### Phase 1 work order (2026-10-23)

The open Phase 1 work is split into three stages, from easiest to hardest. Each stage ships on its own: tests and docs in the same commits, and CI green on macOS and Windows. The stages order work *inside* Phase 1; they are not new phases.

**Stage 1 — Small, low-risk**

| Work | Requirement | Why it is easy |
|------|-------------|----------------|
| `docs/EXTENSION.md`: registry of every Fotollection file format (`.fotollection`, `.fotopreset`, `settings.json`, fonts, logos, cache), with MIME/UTI, versioning rules and limits | FR-023, FR-038 | Documents formats that already exist |
| PNG input: decode with the `png` crate (already a dependency for logos), 8- and 16-bit to RGB8, alpha flattened on white | FR-001 | Pure Rust, no new dependency |
| Basic culling: stars `0`–`5`, flags `P`/`X`/`U`, filter Picks / Rejects / Unflagged; stored in the existing `PhotoEdits` | FR-022 | Data model and project file already hold rating and flag |
| Profile the preview path (resize and encode), and try 8 exports in flight instead of 6 | NFR-002, NFR-003 | Measurement only ([OPTIMIZATION.md §2.1](OPTIMIZATION.md#21-measured-the-same-job-in-chromium)) |

**Stage 2 — Medium, a new file format**

| Work | Requirement | Main risk |
|------|-------------|-----------|
| ✓ `.fotollection` as a self-contained ZIP bundle, like `.docx`: `manifest.json` with BLAKE3 sums, `project.json` (v2), `settings.json`, the fonts and logos in use. Photos are not embedded; presets join when FR-038 lands ([ADR-0016](adr/0016-self-contained-project-bundle.md)) | FR-023 | Done |
| ✓ Save / Save As / Open / New, installing embedded fonts and logos after validation, the **Relink folder** dialog, native File menu and file association | FR-023 | Done; file association checked by hand in a release build |
| ✓ App localization: English (default) and Vietnamese through `i18next` + `react-i18next` (MIT), bundled locale files and typed keys; language stored in `settings.json` v3; Rust errors translated by `code` ([ADR-0015](adr/0015-ui-localization-i18next.md)) | FR-020 | Done |
| ✓ Faster previews: plain Huffman tables and 4:2:0 for screen-only JPEGs took the warm preview from 26 to 17 ms, below Chromium; `turbojpeg` was not needed ([OPTIMIZATION.md §2.1](OPTIMIZATION.md#21-measured-the-same-job-in-chromium)) | NFR-002 | Done |

**Stage 3 — Hard, research and native code**

| Work | Requirement | Main risk |
|------|-------------|-----------|
| ✓ Best-shot suggestions: grouping, sharpness/shake/exposure/noise metrics, face and eye models, explained "Best of N", Keep best | FR-041, NFR-014 | Done: YuNet + FaceMesh on `tract-onnx` ([ADR-0018](adr/0018-face-models-and-runtime.md)); grouping on a layout and colour signature, scored on hand-labelled photos ([ADR-0019](adr/0019-grouping-by-visual-signature.md)) |
| ✓ HEIC/HEIF input (iPhone) | FR-001 | Done through the OS codecs, no LGPL library ([ADR-0017](adr/0017-heic-through-os-codecs.md)) |

### Release plan (2026-10-25)

After v0.3.1, each release has one theme. On 2026-10-25 v0.6.0 shipped the viewer work alone, so "Open with" and RAW moved to v0.7.0 and the later releases moved by one. The order puts the biggest everyday win first: opening photos in Fotollection makes it a daily tool, not only a print tool.

| Release | Theme | Requirements | First, measure |
|---------|-------|--------------|----------------|
| v0.1.0 – v0.3.1 | First previews (2026-10-24 to 10-25): import, culling, crop, stamp, export, info frame, places, best shot | Phase 1 core | NFR-001 to NFR-004 (RESULT) |
| v0.4.0 | Sharing and folders | FR-053, FR-055, FR-056, FR-057 (zoom on the preview), FR-040 (instax-style Fit, drag to reframe) | Done: RESULT §10 |
| v0.5.0 | Print basket | FR-050 (basket and set aside, select mode), FR-059 (Quick-preview) | — |
| v0.6.0 | Photo viewer | FR-057 (full-resolution zoom, zoom buttons), FR-060 (start-up screen) | Done: cold start (NFR-010), open to first photo (NFR-016), preview and zoom at 16/26/32 MP (RESULT §11) |
| v0.7.0 | Open with and RAW | FR-054 ("Open with"), FR-016 (RAW through the OS codecs) | Done: RAW on nine CC0 samples (RESULT §12, NFR-017), open to first photo (NFR-016) |
| v0.8.0 | Places and Lightroom (shipped 2026-10-27) | FR-032, FR-005, FR-030, FR-031, FR-058 (XMP), FR-061 (check for updates), FR-065 (border in Crop to fill), FR-062 + FR-063 (capture screen with live view, Windows, Sony a6400; [ADR-0028](adr/0028-camera-sessions-saved-on-demand.md)) | Lookup speed with manual places, a mixed camera + phone folder; XMP read on a Lightroom-culled folder; NFR-018 and NFR-019 measured in the app with the a6400 |
| v0.8.1 – v0.8.3 | Patches from `release/0.8` (shipped 2026-11-01): green-stripe JPEGs fixed (ADR-0033), Linux (ADR-0034, ADR-0035), Export what you ticked and Report a problem (FR-050, FR-069) | FR-016, NFR-007, FR-050, FR-069 | Linux smoke tests on Ubuntu 22.04 and Debian 12 |
| v0.9.0 | Three modes (shipped 2026-11-02) | FR-048 (the three modes), FR-035 (2-up collage, half-frame style), FR-067 (3 mm border on every print), FR-046 (full-screen ↑ / ↓ in Simple), NFR-021 | Single-photo preview and export, then a sheet against its two photos (NFR-021). The faster Simple export (NFR-015) comes later |
| To place | Camera, continued | The camera on macOS, other Sony bodies (read-only probe first), FR-064 (watch a folder), webcam (UVC) live view | [cameras/README.md](cameras/README.md), [cameras/sony.md](cameras/sony.md) |
| v0.9.1 | Correctness and error codes (shipped 2026-11-02) | Decoding fixes from the 2026-11-02 audit (16-bit grey and CMYK JPEG, the reference decoder for unparsed headers, no grey or CMYK profile on RGB output, DNG black-level origin); `ERR-` codes on screen and an offline error log ([ERROR_CONTROL.md](ERROR_CONTROL.md)) | A fixture for each fix first |
| v0.9.2 | Zoom into RAW fixed (shipped 2026-11-03) | A RAW develop never deadlocks the preview pool; the first zoom reuses the preview's full image; a failed preview shows its `ERR-` code; the language in the version popover (FR-057, FR-016, FR-020) | A test that hung on the old code first |
| v0.9.3 | Image facts and true colours (shipped 2026-11-03) | What each file is in the inspector, unusual files badged and filtered (FR-071); thumbnails, previews and zoom converted to sRGB, PNG gamma and dithering, share exports resized in linear light, straightened reductions without bands, the HEIC profile kept on Windows (the colour part of FR-072) | A PATCH number by the owner's choice (RELEASE.md §1) |
| v0.9.4 | Postcards (shipped 2026-11-04) | FR-073: a Collage switch turns each sheet into one photo and a postcard back, landscape or portrait, with a stamp box, writing lines and typed text | A PATCH number by the owner's choice (RELEASE.md §1) |
| v0.9.5 | Postcard fixes (shipped 2026-11-04) | FR-073: a smaller stamp box, bold as one stroke, a fourth address line when needed, a red ! when text is cut | Fixes and small changes |
| v0.9.6 | Export folders and share fill (shipped 2026-11-04) | FR-009: one folder or a folder per day, one folder by default; FR-075: shared photos whole in a frame of any ratio, filled with white, black or a colour of one's own | A PATCH number by the owner's choice (RELEASE.md §1) |
| v0.9.7 | Postcards to send, faster export (shipped 2026-11-04) | FR-076: a handwriting face; FR-077: one address for many postcards; FR-078: the photo at a glance in the preview; NFR-003 and NFR-004 met on 20 MP photos | A PATCH number by the owner's choice (RELEASE.md §1) |
| v0.9.8 | Two-finger zoom, panels of one style (shipped 2026-11-04) | FR-057: two-finger scroll zooms the preview; every panel card built from the lego blocks, checked by `check-lego` (STYLE_RULE §4); hints kept inside the window (FR-052) | A PATCH number by the owner's choice (RELEASE.md §1) |
| v0.9.9 | Colour, first cards (shipped 2026-11-05) | FR-079: Faves and Tools, a photo's own look, Before / After, a drag that follows the pointer; RAW badges and the RAW filter (FR-022, FR-078) | The Looks card and LUTs followed in v0.14.0, grain and vignette in v0.14.x |
| v0.10.0 | Core (shipped 2026-11-05) | FR-072 (our own develop for every Bayer camera: rawler only unpacks; stuck pixels, green balance, a directional demosaic, a false-colour median; neutral RAW colour by default and the camera's look in one click), FR-088 (brightness, highlights, shadows and white balance on a RAW's own data, with headroom), damaged RAW files refused, [ADR-0037](adr/0037-our-own-develop-core.md), [ADR-0040](adr/0040-neutral-raw-colour-and-gpu-develop.md) | Re-ordered by the owner (2026-11-05): the develop core shipped before the Looks; RESULT §23–§28 |
| v0.11.0 | Instant previews (shipped 2026-11-05) | FR-091 (standard previews on disk after the thumbnails, a quarter of the RAM for the preview, prefetch, the photo on screen develops first: any photo opens in 27 ms, was 274–531 ms); colour cards in the previews only, every card flagged where it shows (FR-079); sliders take a near press and the arrow keys; RAW hangs fixed | The owner's request after v0.10.0 ("like Lightroom") |
| v0.11.1 | Memory, made safe (2026-11-06) | FR-091's audit: nothing kept across a project switch, the photo on screen never pushed out by a prefetch, one prefetch worker, no photo made twice, the disk limit during the previews' pass, a version for standard previews, background unless marked | The owner's choice: fix before v0.12.0 |
| v0.12.0 | Settings & Develop (shipped 2026-11-06) | FR-086 (one Settings window, two columns as on an iPad, the memory and the cache to manage, Default and Auto, memory given back when short), FR-087 (a collage sheet's pixels against its print size), FR-090 (a filmstrip in the photo's preview), FR-095 (a develop view a Lightroom user can read: one toolbar above the photo, Lightroom's keys, zoom in real pixels, a full-window layout with Navigator and History), FR-096 (before and after, Lightroom's four views); zoom of a turned photo fixed | The owner's requests of 2026-11-05 and 2026-11-06 |
| v0.13.0 | The new look (shipped 2026-11-06) | FR-099 (controls with depth, as Sentry's, bold; the light theme's shadows in dark teal), FR-092 and FR-093 (the export dialog as Lightroom's, the quality slider with measured badges), FR-104 and FR-105 (file names as Lightroom's templates, a file-size limit), FR-107 (folder tabs reordered by dragging), FR-103 (checking for updates at start), the Be Vietnam Pro font, one panel width, a toolbar that fits, `MEASURE` checks for text and rows that overflow | The owner's choice on the style board, 2026-11-06 |
| v0.13.1 | Develop, part 2 (shipped 2026-11-07) | FR-097 (copy and paste settings, Reset), FR-098 (histogram above the Navigator, clipping, white balance), FR-109 (a welcome screen on the first launch) | Moved from v0.12.0 when the owner released it as it stood; FR-092 and FR-093 moved on to v0.13.0 |
| v0.13.2 | Develop, part 3 (shipped 2026-11-07) | FR-098 in Collage mode (`J` and `W` on a sheet's picked photo), FR-110 (the manual inside the app, offline), the Performance page made visual (stat tiles and a chart opening each part, more diagrams, shorter prose) | The owner's choice of 2026-11-07, after v0.13.1 |
| v0.13.3 – v0.13.4 | Patches (shipped 2026-11-07 and 08) | The RAW tone curve without steps (ADR-0043, RESULT §33); security patches (minimum safe version 0.13.3) | — |
| v0.14.0 | Looks (shipped 2026-11-09) | FR-079 (the Looks card, built-in looks of our own, the Color Mixer), FR-070 (looks from `.cube` and HaldCLUT LUTs, the highlight of Simple mode: look, choose a colour, print), FR-089 (Lightroom presets, measured against Lightroom's renders), FR-114 (a look for many photos), FR-115's Dehaze, FR-096's press and hold for the original, FR-023's project settings of its own, NFR-011's containment, FR-116 and FR-082 (data on this computer, `.fotosettings`), *Presets & LUTs* in the develop view's left column, FR-100's motion tokens (M1–M2), [ADR-0038](adr/0038-moderate-colour-editor-on-one-lut-engine.md), [ADR-0048](adr/0048-one-owner-per-service-failures-stay-local.md) | Budgets: ≤ 5 ms preview, ≤ 60 ms 24 MP export whatever is set (`measure_look`) |
| v0.14.x | Looks, part 2 (planned; owner 2026-11-09: "cover and complete what the app reports missing from presets") | FR-118 (every preset setting applied: clarity and texture of FR-115, local highlights, the parametric curve, white balance against each photo's as shot, sharpening and noise reduction, older presets), grain and vignette in millimetres on the print (session 13), FR-112 (highlight recovery for RAW, measured against the current fade), FR-100's M3 (motion between views, 60 fps measured) | Each tool within OPTIMIZATION §3 and measured against Lightroom's exports (RESULT) |
| v0.15.0 | Core 2 (planned) | FR-072 (X-Trans with Markesteijn, the develop on the GPU with `wgpu`, the DNG kinds of `raw/dng.md`, a linear Rec.2020 working space; highlight reconstruction moved to FR-112, now v0.14.x), FR-085 (lens distortion correction), FR-117 (right-click menus in more places), FR-111 (lossless export through a 32-bit float path, moved from v0.16.0 by the owner, 2026-11-09, [ADR-0043](adr/0043-lossless-export-f32-path-tiff.md)) | Markesteijn ≤ 700 ms on 26 MP; GPU develop ≤ 80 ms on 24 MP, ΔE ≤ 1 to the CPU |
| After v0.15.0 | Film looks as built-in recipes on the looks engine | FR-047 (FR-039 merged into FR-079) | — |
| Later | Presets, trips, journal | FR-038, FR-012, FR-014, FR-049, FR-015, FR-051 | — |

## Phase 2 — Smart features

**Goal:** the app organizes photos and avoids bad crops automatically.

| ID | Requirement |
|----|-------------|
| FR-012 | Trip auto-grouping |
| FR-013 | Face-aware smart crop |
| FR-014 | Export by trip |
| FR-015 | Duplicate detection |
| FR-024 | Copies and order summary |
| FR-031 | Borrow GPS from a phone or GPX track |
| FR-034 | Lab presets |
| FR-036 | Custom captions |
| FR-038 | Print presets |
| FR-039 | Photo Style looks (merged into FR-079, 2026-11-05) |
| FR-044 | Per-photo overrides |
| FR-046 | Swipe culling (full-screen ↑ / ↓ print list) |
| FR-047 | Film looks (Fujifilm-style film simulations, own names and LUTs) |
| FR-051 | Landmark recognition: suggest a place from what the photo shows (offline, optional model pack) |
| FR-054 | Open photos with Fotollection from the system's "Open with" menu, as a photo viewer |
| FR-058 | Lightroom compatibility: read XMP sidecars on import, write `.xmp` as an opt-in, photos never touched |
| FR-061 | Check for updates: a button that asks GitHub once, then opens the installer in the browser |
| FR-063 | Tethered capture over USB: Capture in the app, the full-resolution photo opens at once; own PTP drivers per maker, no install ([ADR-0026](adr/0026-cameras-through-the-os-usb-stack.md)) |
| FR-064 | Watch a folder: a photo written into it by a maker's tether app opens at once |
| FR-066 | Borrow places from a GPX track: interpolate each photo's time along the track (split out of FR-031) |
| FR-070 | Looks from LUT files (`.cube`, HaldCLUT, strength, per photo), LUMIX Lab style; v0.14.0, the highlight of Simple mode, [ADR-0036](adr/0036-colour-looks-cube-luts-first.md) |
| FR-074 | Postcard backs: a background colour or a pattern (airmail stripes, a grid, a texture) behind the lines; planned, not scheduled |
| FR-075 | Share exports fitted whole into a ratio (1:1, 4:5, 3:2… or custom), the rest filled with white, black or a colour of one's own; v0.9.6 |
| FR-076 | A bundled handwriting face for postcards (Patrick Hand), strokes as thick as the print test asks; v0.9.7 |
| FR-077 | One address for many postcards: Use for N postcards, and the recent addresses of this machine; v0.9.7 |
| FR-078 | The photo at a glance at the top of the preview: camera, lens, exposure, place, date, size; v0.9.7 |
| FR-079 | Colour adjustments, moderate: Looks, Faves and Tools cards (tone, colour, style, effects), looks with an amount, per photo; Faves and Tools shipped in v0.9.9, the Looks card in v0.14.0, [ADR-0038](adr/0038-moderate-colour-editor-on-one-lut-engine.md) |
| FR-080 | The panel's cards in an order of one's own, the same in the gallery and the previews, with *Reset order*; v0.14.0, with FR-116 (2026-11-09) |
| FR-081 | A welcome screen on the first launch: the welcome shipped as FR-109 (v0.13.1); left, a page to pick the cards one likes, Pinterest style, they come first (with FR-080); *Skip*; later, not scheduled |
| FR-082 | The app's brain in one file: *Export / Import settings* as `.fotosettings` (every setting, the card order, looks and LUTs, fonts, logos, postcard addresses), to move to another computer; later, not scheduled |
| FR-083 | A personal photo calendar: a built-in calendar engine for any year, important days marked, a photo a month, PDF or PNG pages; later, not scheduled |
| FR-084 | Language packs anyone can add: a `.fotolang` file in the app's i18next layout, checked in Rust and by `check-i18n`, loaded from the Settings window or shipped through a pull request; carried by `.fotosettings`; later, not scheduled |
| FR-085 | Lens distortion correction in our own develop (Panasonic and Olympus maker data, DNG `WarpRectilinear`), so ImageIO is no longer first for RW2 and ORF on macOS; after v0.11.0 |
| FR-086 | One Settings window: a gear beside the author's name opens every app setting in two columns as on an iPad (language, theme, `.fotosettings`, language packs, cache, updates, version and changelog), the preview's layout reused |
| FR-087 | How large a collage sheet prints: the sheet's pixels and a sharp / acceptable / soft verdict for the chosen size in Collage's *Print size* card |
| FR-088 | RAW edited in linear light: exposure, highlights, shadows, warmth and tint on the RAW's own data before its tone, with headroom above white; v0.10.0 |
| FR-089 | Lightroom presets: a user's own `.xmp` or `.lrtemplate` imported into a look, parsed safely, unsupported settings listed; v0.14.0, the rest in v0.14.x (FR-118) |
| FR-090 | A filmstrip in the photo's preview (Pro mode), as Lightroom's: the view's thumbnails under the photo, skim and pick with one click |
| FR-091 | Instant previews like Lightroom: standard previews on disk after the thumbnails, a quarter of the machine's RAM for the preview, prefetch around the photo on screen, which always develops first |
| FR-092 | Export quality on a slider, as Lightroom's: the size estimate follows the thumb, and badges say whether the setting prints well; v0.13.0 |
| FR-093 | The export's options in one small dialog: a press on *Export* opens it, with *Don't ask again* after which *Export* exports at once with the saved choices, changed later in Settings; the same dialog in every mode and preview, the Settings window's size with Lightroom's presets; v0.13.0 |
| FR-094 | Project history without nagging: dated copies of each project and of the settings kept automatically (seven days, four weeks), restored from the Settings window as a new document; version to be chosen |
| FR-095 | A develop view a Lightroom user can read: a full-window layout chosen in Settings (`F`), Navigator and History on the left, one toolbar above the photo, Lightroom's keys, zoom in real pixels, the inspector develop first; shipped in v0.12.0 |
| FR-096 | Before and after: `\` toggles, `Y` side by side, `⇧Y` split, zoomed together; shipped in v0.12.0; press and hold for the original in v0.14.0 |
| FR-097 | Copy and paste settings (⇧⌘C / ⇧⌘V) to the photo shown or every ticked photo, Reset (⇧⌘R), undo and redo buttons; v0.13.1 |
| FR-098 | Histogram, clipping warnings (`J`) and a white-balance picker (`W`); v0.13.1 |
| FR-099 | Controls with depth, as Sentry's: raised buttons with a lip, inputs set in, lifted cards; v0.13.0 (owner moved it forward, 2026-11-06) |
| FR-100 | Motion that feels like one app: shared timings, smooth transitions between views and layouts, lists that move instead of jumping, *Reduce motion* respected with a setting, 60 fps; M1–M2 in v0.14.0, M3 in v0.14.x (owner, 2026-11-08 and 09, [ADR-0046](adr/0046-one-motion-system.md)) |
| FR-101 | Healing: spot removal, heal and clone, as Lightroom's Remove tool; a hidden tab of the inspector until built; about v0.16 |
| FR-102 | Masking: linear, radial and brush masks with their own sliders, later on-device subject and sky, after FR-115 and the linear gradient first; needs a local layer beside the look's table (ADR-0038 amended); about v0.16 |
| FR-103 | Check for updates at start, on by default and turned off in Settings; check only, the user downloads; a popup, with a red warning below the minimum safe version carried in the release notes (`.github/update.json`); v0.13.0 |
| FR-104 | File names as Lightroom's templates (original, date, custom text and sequence) with a live example; v0.13.0 |
| FR-105 | Limit the file size, by bisection of the quality, measured first; v0.13.0 |
| FR-106 | Output sharpening for screen, matte or glossy paper, measured first; later |
| FR-107 | Folder tabs reordered by dragging, as Chrome's tabs, kept with the project; v0.13.0 |
| FR-110 | The manual inside the app, offline, from the same USER_GUIDE as the site, in Settings and the Help menu; v0.13.2 |
| FR-111 | Lossless export in Pro mode: TIFF 16-bit by default or 32-bit float, through a 32-bit float path; v0.15.0 (owner, 2026-11-09) |
| FR-112 | Highlight recovery for RAW, chosen and tuned by measurement against the current fade; v0.14.x (moved from v0.14.0, owner, 2026-11-09) |
| FR-113 | ID photos: one photo cut to a document's size (35 × 45, 30 × 40, 2 × 2 in…) and repeated on one print with cut marks, at true size; v0.15.x, after the colour engine (looks, LUTs, Lightroom presets) and v0.15.0's 32-bit export (owner, 2026-11-07; moved with FR-111, 2026-11-09) |
| FR-114 | A look, LUT or preset given to many photos at once from Pro mode's gallery (a Colour tab beside Print): the ticked photos or every photo shown, an amount, photos with their own colour asked about once, one undo step; v0.14.0 (owner, 2026-11-08) |
| FR-115 | Local contrast in Pro mode: clarity, texture and dehaze, the spatial tools a colour table cannot hold, before FR-102's masks (a linear gradient first); Dehaze in v0.14.0 (2026-11-09, before the table); Rust after the table, GPU preview, chosen and tuned against Lightroom's renders; clarity and texture in v0.14.x with FR-118 (owner, 2026-11-09) (owner, 2026-11-08: where Pro mode is heading) |
| FR-116 | Data on this computer, in Settings: export and import settings (FR-082), what each store holds with its size and *Open folder*, clear what can be made again, reset settings, erase data; each row asks its store's owner (ADR-0048); v0.14.0 |
| FR-117 | Right-click menus in more places (folders, photos, the filmstrip, collage slots, export presets), each holding only actions found elsewhere too, groups split by hairlines as Lightroom's; `MenuContent` and the style rules; v0.15.0 (owner, 2026-11-09) |
| FR-118 | Every preset setting applied: clarity and texture (FR-115), local highlights, the parametric curve, white balance in kelvins against each photo's as shot, grain and vignette, sharpening and noise reduction, older presets; each tuned against Lightroom's exports; lens corrections stay with FR-085; v0.14.x (owner, 2026-11-09) |
| FR-119 | Smooth on ARM, RISC-V tried: a Windows ARM64 build first (today emulated), every budget measured on a Raspberry Pi 5, an M-series Mac and a Snapdragon laptop, SIMD only where measured; riscv64 Linux cross-built and smoke-tested, experimental until a real board meets the budgets; v0.15.x, RISC-V later (owner, 2026-11-09) |
| FR-120 | Turn the print size: a button and a key lay a print size flat or upright for a photo, so a portrait photo can be cropped across a 15 × 5 strip; kept per photo, in every place the size shows; v0.14.x (owner, 2026-11-10) |
| FR-121 | A small "fotollection.org" on each print, on by default, off with one switch in Settings; measured in millimetres, in the border, frame or a postcard's back first; never on shared exports; v0.14.x (owner, 2026-11-10) |
| FR-109 | A welcome screen on the first launch: five pages (welcome, modes, choose, make it yours, print), Next, a *Show at every start* tick and Start; v0.13.1 |
| FR-108 | Photobooth with the computer's own camera (the Mac's webcam): countdown, a strip or sheet to print, offline; later |
| FR-068 | HEIC on Linux through the libheif the distribution installs, loaded at run time; no bundled HEVC decoder (after v0.9.0, [ADR-0034](adr/0034-linux-lean.md)) |
| FR-049 | Trip journal: an offline report of the trip drafted from the album (days, places, photo counts, cameras, focal-length range), with a journal entry written by every editing session (like the instax app) |

## Phase 3 — Pro & growth

| ID | Requirement |
|----|-------------|
| FR-016 | RAW input |
| FR-017 | Photo book layout preview |
| FR-018 | QR code on photo back |
| FR-019 | Print service upload |
| FR-021 | Manual updates (no in-app updater, ADR-0021) |
| FR-029 | Output sharpening |
| FR-035 | Multi-photo layouts (2-up collage in v0.9.0; 4-up and strips later) |
| FR-037 | Contact sheet |

## Ideas backlog (not yet requirements)

These ideas have no ID yet. Promote one to a PRD requirement before scheduling it.

- **A store of looks and frames, online (owner, 2026-11-10; later).** Looks and info frames by theme and by occasion (Christmas, Tết, a summer, a city), some free, some exclusive to a season, some sold; imported into Fotollection as a look or a frame is today (FR-079, FR-089, `.fotosettings` parts). Privacy first (NFR-006, CLAUDE.md rule 4): the core never goes online; the store is a page on the website, or a window in Fotollection that connects only when the user opens it, never in the background, nothing sent about the photos. Each item checked as an import is (bounded, in Rust); payment by a processor on the web (as Ko-fi, never in the core), the licence of each look stated (the creator's, no Adobe preset). Needs a decision on what is sold and by whom, and the relicensing question (ADR-0008) settled first.
- **Growth, without telemetry (owner, 2026-11-10, ideas kept from advice; each one the owner's choice before it is built):**
  - *Each print a small advert:* decided as FR-121 (owner, 2026-11-10): on by default, off in Settings.
  - *The digicam and date-stamp trend:* 15-second videos (an iPhone photo → the retro date stamp → the print from dm) for TikTok and Reels.
  - *A small web tool as a door:* a page that crops a photo to dm's and Rossmann's print sizes in the browser, uploading nothing, ending with "500 photos at once? Get Fotollection"; it brings search traffic a landing page does not.
  - *Print shops as partners:* a shop's own sizes and looks in Fotollection, the shop recommending it to its customers (a small studio first).
  - *Seasons:* "print the family's photos for Tết" for the Vietnamese community, "Erasmus memories" with ESN groups at the end of a term, Christmas prints from November.
  - *Where people look for software:* Flathub (the AppStream file is ready), winget, a Homebrew cask, the AUR, AlternativeTo as a Lightroom alternative, "awesome" lists on GitHub; each is also a link back.
  - *Communities by feature:* LUMIX owners (LUMIX Lab LUTs read on the desktop, free), Show HN and r/opensource (offline, GPL, no tracking), Vietnamese travel and printing groups; each release a short post with a before and after, not the changelog.
  - *Support that gives back:* a postcard printed with Fotollection and a handwritten thank-you above an amount, a look named after a supporter (with their consent), supporters voting on the next feature; in Fotollection, one quiet line after a good moment ("Fotollection helped you print 100 photos ☕"), off for good with one click; what the money pays for said plainly (the domain, code signing). Never an advert, never a nag, never counted (no telemetry).
- **The website in German, Czech and Polish (owner, 2026-11-10: "later, how hard?").** Where dm and Rossmann print (Germany, Austria, Czechia, Poland, Hungary), pages for the searches people make ("dm Fotodruck richtig zuschneiden", "Datum auf Fotos drucken"). The effort, from today's site: (1) the site is built for two languages: the layout and the includes pick between `en` and `vi` (`page.lang == 'vi'`, `/vi/` paths) on about 20 lines, so the language switch becomes a list (a small menu) and each page's paths come from one table (`_data/languages.yml`): about a day, measured by the same checks (`check-footnotes`, the mirror checks); (2) each language is a block of about 440 strings in `i18n.yml`, its footnote lists and its pages under `/de/`, `/cs/`, `/pl/`: the translation is the real cost, done by a fluent speaker or checked by one, never machine-translated unread; (3) the manual (`USER_GUIDE`) stays English for a new language until it is translated, as `site-manual.sh` allows; (4) the app's own interface is FR-084's language packs, separate. Hungarian is a candidate too (the owner lives there).

- **Look back one trip at a time from the Photos library (macOS, PhotoKit, read-only).** Pick a trip by date or place; the app groups it, suggests the best shots, and puts the rejects in a "To delete" album the user empties in Photos. With iCloud Photos this frees space on the iPhone too, without the app ever deleting anything. Prompted by the owner's own library: about 70,000 photos on a 512 GB iPhone. Measure a 70,000-photo library first (NFR-001, NFR-014).
- **iPhone companion app**, only if the Mac flow proves itself: swipe culling on the way home and real deletion behind iOS's own confirmation. Needs the Apple Developer Program (99 USD/year, which would also notarize the Mac app), a native SwiftUI interface over the Rust core (UniFFI), and Core ML for the face models.
- **Support from abroad:** a Ko-fi or GitHub Sponsors link next to the VietQR code on the website.
- Append-only edit journal for the session (NFR-013): not urgent, a full save is 70 ms at 70,000 edited photos (2026-10-25)
- Lossless rotation fix-up tool
- ~~Linux build~~: shipped as the lean build in v0.8.2 ([ADR-0034](adr/0034-linux-lean.md)); HEIC there is FR-068
