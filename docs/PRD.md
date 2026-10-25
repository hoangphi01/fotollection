# Product Requirements Document — Fotollection

> **Owner:** product · **Status:** Draft v0.1 · **Last reviewed:** 2026-10-23
>
> This document is the single source of truth for **what** Fotollection does and **why**.
> *How* it is built lives in [ARCHITECTURE.md](ARCHITECTURE.md); *when* lives in [ROADMAP.md](ROADMAP.md).
> Requirement IDs (`FR-xxx`, `NFR-xxx`) are permanent: never renumber or reuse them. Retire an ID by marking it **Withdrawn**.

## 1. Problem

People shoot thousands of photos but print almost none. Preparing a print order is tedious and lossy:

- Printed photos carry no date or place, so a few years later nobody remembers where or when a photo was taken.
- Print labs (CEWE, dm, Rossmann, Boots…) use fixed paper ratios (2:3, 3:4, 1:1…). Camera images get auto-cropped by the lab, which can cut off heads and horizons.
- Photos from a trip are scattered across days and devices, with no grouping.
- The tools that exist either upload your photos to a server (a privacy risk, since GPS reveals your home) or each solve only one of these problems.

## 2. Product vision

**Fotollection is a fast, private, offline desktop app (macOS and Windows) that turns a folder of photos into a print-ready set.** It stamps date and location from EXIF, crops to standard EU print sizes, groups photos by trip, and keeps the original quality. Photos never leave the user's machine.

It has two modes ([FR-048](#fr-048--normal-and-professional-modes)): **Normal**, for "pick, stamp, print" in a few clicks, and **Professional**, with every control described in this document.

### Market gap

Nobody combines EXIF date/location stamping, trip auto-organization, and batch cropping to EU print sizes in one app.

| Tool | Type | Date stamp | Trip grouping | CEWE ratios | Privacy |
|------|------|-----------|---------------|-------------|---------|
| BatchPhoto | Desktop (paid) | Yes | No | No | Local |
| Timestamp It | Web | Yes (3 free) | No | No | Server-side |
| BulkPicTools | Web | No | No | No | Client-side |
| RowPix | Web | No | No | No | Client-side |
| CropMaster | Web | No | No | No | Client-side |
| PhotoPrism | Self-hosted | Partial | GPS maps | No | Self-hosted |
| **Fotollection** | **Desktop (Rust)** | **Yes** | **Yes** | **Yes** | **100% local, offline** |

## 3. Users

| Persona | Description | Key need |
|---------|-------------|----------|
| **Enthusiast photographer** | Shoots a mirrorless camera (e.g. Lumix GX9) plus a phone. Folders of 24 MP JPEG and HEIC files. | Speed, no quality loss, colour fidelity |
| **Traveller / family archivist** | Comes home with 800 photos from a two-week trip and wants a printed album. | Automatic grouping, date and place on every print |
| **Small studio / event shooter** | Delivers prints of events in bulk. | Large-batch throughput, consistent crops, predictable output naming |

## 4. Core user journey

1. **Import:** open a folder, or drag and drop photos or folders into the app. The library grid appears instantly, grouped by date (by trip in Phase 2).
2. **Cull:** step through the photos with the keyboard, rate and flag them, and filter down to the set to print ([FR-022](#fr-022--culling-rate-flag-filter)). Later, a full-screen swipe mode builds the print list with ↑ / ↓ ([FR-046](#fr-046--swipe-culling)). Work is saved automatically to a project file ([FR-023](#fr-023--projects-autosave-and-undo)).
3. **Fix time and place:** correct a camera's clock offset ([FR-030](#fr-030--camera-clock-offset)), and fill in a missing date or place by hand ([FR-032](#fr-032--manual-date-and-place)) or from a phone or GPX track (Phase 2).
4. **Choose sizes and style:** pick a print size per photo or per group ([FR-025](#fr-025--mixed-print-sizes-per-order)) and a stamp style ([FR-033](#fr-033--retro-film-date-stamp-and-stamp-presets)). Adjust crop frames. A live preview updates as settings change.
5. **Check:** resolution warnings ([FR-026](#fr-026--resolution-warning)) and the safe-area overlay ([FR-028](#fr-028--safe-area-and-borders)) flag problems before printing.
6. **Export:** export to a folder, or later with a lab preset ([FR-034](#fr-034--lab-presets)), with sRGB conversion ([FR-027](#fr-027--srgb-conversion-for-labs)). A progress view shows the batch, and the user can cancel at any time.
7. **Order:** upload the exported folders to the print lab.

## 5. Print sizes (CEWE / EU standard)

| ID | Size (cm) | Size (inch) | Aspect ratio | Use case |
|----|-----------|-------------|--------------|----------|
| `9x13` | 9×13 | 3.5×5 | ~1:1.44 | Wallet/small |
| `10x15` | 10×15 | 4×6 | 2:3 | Standard |
| `11x17` | 11×17 | 4.5×6.5 | ~1:1.55 | Slightly larger |
| `13x18` | 13×18 | 5×7 | ~1:1.38 | Medium |
| `15x20` | 15×20 | 6×8 | 3:4 | Large |
| `20x30` | 20×30 | 8×12 | 2:3 | Extra large |
| `10x10` | 10×10 | 4×4 | 1:1 | Small square |
| `13x13` | 13×13 | 5×5 | 1:1 | Medium square |
| `15x15` | 15×15 | 6×6 | 1:1 | Square |
| `20x20` | 20×20 | 8×8 | 1:1 | Large square |
| `5x15` | 5×15 | 2×6 | 1:3 | Photo strip |
| `original` | 10 × (photo ratio), e.g. 10×13.3 for 4:3 | 4 × (photo ratio) | The photo's own | No crop ([FR-045](#fr-045--original-aspect-ratio)) |

Target labs: CEWE, Rossmann, dm, Boots Photo, Snapfish, FreePrints, Photobox. The photo services of dm (Fotoparadies) and Rossmann (Fotowelt) are run by CEWE; every size on dm's in-store price list (9×13, 5×15, 10×15, 10×10, 13×13, 13×18, 15×20, 15×15, 20×20, 20×30; checked 2026-10-24) is built in. Other sizes are added as custom sizes ([FR-043](#fr-043--custom-print-sizes)).

### Minimum resolution per size

The table below gives the pixels needed after cropping, used by [FR-026](#fr-026--resolution-warning). 300 DPI is "excellent", 200 DPI is "good", and below 150 DPI is "poor".

| ID | 300 DPI (px) | 200 DPI (px) | 150 DPI (px) |
|----|--------------|--------------|--------------|
| `9x13` | 1063×1535 | 709×1024 | 531×768 |
| `10x15` | 1181×1772 | 787×1181 | 591×886 |
| `11x17` | 1299×2008 | 866×1339 | 650×1004 |
| `13x18` | 1535×2126 | 1024×1417 | 768×1063 |
| `15x20` | 1772×2362 | 1181×1575 | 886×1181 |
| `20x30` | 2362×3543 | 1575×2362 | 1181×1772 |
| `10x10` | 1181×1181 | 787×787 | 591×591 |
| `13x13` | 1535×1535 | 1024×1024 | 768×768 |
| `15x15` | 1772×1772 | 1181×1181 | 886×886 |
| `20x20` | 2362×2362 | 1575×1575 | 1181×1181 |
| `5x15` | 591×1772 | 394×1181 | 295×886 |

## 6. Functional requirements

Priority: **P0** = MVP blocker · **P1** = MVP desirable · **P2** = later.
The phase assignment for each requirement lives in [ROADMAP.md](ROADMAP.md), and its progress in [STATUS.md](STATUS.md).

### FR-001 — Import photos

**P0.** Import photos from a folder picker (recursive), drag and drop of files and folders, or a file picker.

- Accepts JPEG, HEIC/HEIF and PNG, detected by magic bytes rather than file extension.
- HEIC/HEIF (iPhone) is decoded by the operating system ([ADR-0017](adr/0017-heic-through-os-codecs.md)): ImageIO on macOS, and WIC on Windows when the free *HEIF Image Extensions* and the *HEVC Video Extensions* from the Microsoft Store are installed. Without them, HEIC files are listed as skipped with a message that says what to install. EXIF (date, GPS, orientation) is read from the HEIC itself.
- PNG photos and screenshots are converted to 8-bit RGB. 16-bit, greyscale and palette images are supported, and transparency is flattened onto white, as it would print. An embedded ICC profile is kept for export. A PNG without EXIF takes its date from the file's modification time and is marked as such.
- Unsupported or invalid files are listed with a reason. They never abort the import.
- Importing does not modify source files. The app opens sources read-only.

- **Refresh:** a button next to the library's folders scans them again and adds photos copied in since, without importing anything twice; every edit on the photos already there is kept.
- **Remove a folder:** opening another folder adds it to the library. Each folder row has a close button on hover, like a browser tab, that takes the folder and its photos out of the library. The files on disk are never touched, and the other folders keep every edit (the undo history restarts).

### FR-002 — Read photo metadata

**P0.** Read the allowlisted EXIF fields: capture date and time (with offset), GPS position, camera make and model, orientation, dimensions and exposure fields.

- A photo without EXIF is still usable. The app falls back to the file modification time and shows a warning badge.
- Orientation is applied, so photos are never displayed or exported rotated incorrectly.

### FR-003 — Library grid

**P0.** Show a virtualised thumbnail grid of all imported photos, sorted by capture time and grouped by day.

- Shows a badge for missing date, missing GPS and processing errors.
- Supports multi-select (shift-click and cmd/ctrl-click) for per-photo overrides.
- Tiles show a skeleton and spinner until their thumbnail arrives, then fade in. Long tasks (reading photos, preparing thumbnails) show a progress bar with a count and percentage.

### FR-004 — Date stamp

**P0.** Render the capture date onto the photo.

- Configurable format (presets plus a custom pattern), size relative to the short edge, colour, and position (each of the 4 corners, with a margin).
- **One font, like a film camera's imprint:** the text inside the photo is always set in the bundled **DSEG14 Classic Italic** (OFL 1.1), a slanted 14-segment LCD face, in plain capitals without accents ("Budaörs" → "BUDAORS"). It has the soft glow the imprint's LEDs left on film and a faint shadow for bright skies ([FR-033](#fr-033--retro-film-date-stamp-and-stamp-presets)). User fonts ([FR-042](#fr-042--custom-fonts)) apply to the info frame only.
- **Two lines:** the date, with the place on the same line when switched on (`'25 09 21  DEBRECEN`, [FR-005](#fr-005--location-stamp-offline)), and a caption line of the user's own text below it ([FR-036](#fr-036--custom-captions)). At the bottom, the caption row is always kept free, so the date sits a little higher and does not move when a caption is added; a caption longer than the photo is set smaller to fit.
- Rendered with anti-aliasing at output resolution and placed inside the final crop area, so the stamp is never cropped away.
- **Photos without an EXIF date** (scans, screenshots, files from messaging apps) are stamped with the file's date, as the preview already shows ("No EXIF date · file time used"). Only a photo with no date at all gets no date, and it still prints its place and caption.

### FR-005 — Location stamp (offline)

**P0.** Resolve GPS coordinates to a place name, for example "Prague, CZ", fully offline, and render it with the date stamp or on its own.

- Uses a bundled dataset. No network request is ever made.
- The user can edit the resolved name per day or per group, for example to change "Praha 1" to "Prague".

### FR-006 — Crop to print ratio

**P0.** Crop every photo to the selected print size ([§5](#5-print-sizes-cewe--eu-standard)).

- Portrait and landscape orientation is matched automatically.
- The default crop is centred. The user can adjust it per photo in the **crop editor**:
  - The whole photo is shown, with everything outside the crop **dimmed**.
  - **Drag** to move the crop. **Zoom** with the wheel, a trackpad pinch or a slider (1–8×). Arrow keys nudge it, and a double-click resets it.
  - **Or drag the photo in the preview itself**, without opening the crop editor: the print re-renders as it moves, and one drag is one undo step.
  - **Edge labels:** the top edge shows the printed width and the left edge the printed height (cm, or inches for sizes entered in inches), each with its pixel count. A centre pill shows the **aspect ratio** (`2:3`, `1:1`, or `1:1.44` for uneven sizes), the print size and the **live DPI** with its quality band ([FR-026](#fr-026--resolution-warning)). When a frame is on, the sizes are those of the photo area.
  - The crop is stored as a centre and zoom, so it survives changes of print size ([FR-025](#fr-025--mixed-print-sizes-per-order)).
  - It is undoable ([FR-023](#fr-023--projects-autosave-and-undo)).
- Offers a "fit with border" (letterbox) alternative to cropping.

- **Rotate:** turn a photo a quarter turn left or right (buttons next to Crop in the preview, or `[` and `]`), losslessly, on top of its EXIF orientation. This fixes photos taken upside down or sideways. The crop turns with the photo. It is undoable and saved in the project (`photos[].rotation`, project schema v4).
- **Straighten:** a slider in the crop editor levels a tilted horizon by up to ±15° (0.1° steps; double-click or `0` resets). In crop mode, ← and → turn it by 1° (with ⌥ by 0.1°) and ↑ ↓ still nudge the crop; once the crop is applied or cancelled, ← and → move between photos again. The photo turns under the crop box live, with a fine levelling grid. The crop is then as large as it can be without showing an empty corner, at the print ratio, and the export resamples only the cropped pixels (bicubic, supersampled when reducing). A level photo keeps the plain lossless path, so 0° gives exactly the same file as before. Undoable and saved in the project (`photos[].straighten`, project schema v5).

### FR-007 — Live preview

**P0.** Every change to the stamp or crop settings updates the preview immediately (see [NFR-002](#nfr-002--preview-latency)).

- Opening a photo (double-click, Enter or Space) shows a **preview dialog**. The left side is the Rust-rendered print, with the exact crop and stamp of the export. The right side is an **inspector** with:
  - capture date and time, and whether it came from EXIF or the file;
  - camera;
  - pixel size, megapixels and file size;
  - GPS;
  - print size and effective DPI ([FR-026](#fr-026--resolution-warning));
  - crop mode;
  - quick date-stamp controls.
- `←`/`→` browse within the dialog, and `Esc` closes it. A spinner shows while the preview renders.

### FR-008 — Original-quality export

**P0.** Export at full resolution.

- JPEG quality is configurable, with a default of 95 and 4:4:4 chroma subsampling.
- The embedded ICC profile is preserved.
- Photos that are crop-only with no stamp use a lossless DCT-domain crop when possible.

### FR-009 — Export by date

**P0.** Write the export to a user-chosen folder using `YYYY-MM-DD/` subfolders and stable, sortable file names.

- Existing files are never overwritten silently. The user picks a rule: skip, rename, or overwrite after confirmation.

### FR-010 — Metadata privacy on export

**P0.** Control which metadata is written into exported files.

- GPS is stripped by default.
- Options: keep everything (minus GPS), keep the date only, or strip everything.
- Before export, a banner warns when photos with GPS are about to be exported with GPS kept.
- **Every export writes a small EXIF block from a fixed allowlist** (camera, lens, exposure, capture time and `Software = Fotollection <version>`), and **never GPS**: prints and share exports alike (since 2026-10-25; prints carried no metadata before). It weighs about 0.3 KB; the camera's own maker notes and embedded thumbnail (about 18 KB) are never copied.

### FR-011 — Batch progress, cancel and recovery

**P0.** Show export progress with a count and an estimated time remaining.

- The user can cancel instantly, and partially written files are removed.
- If one photo fails, it is skipped, the error is logged, and the batch continues. A summary report is shown at the end.

### FR-012 — Trip auto-grouping

**P1.** Cluster photos into trips by GPS and time proximity, and suggest a name for each trip, for example "2026-07 Lisbon".

- The user can merge, split and rename trips.

### FR-013 — Face-aware smart crop

**P1.** Detect faces on-device and position the default crop so that no face is cut. If faces cannot fit inside the crop, fall back to a centred crop.

### FR-014 — Export by trip

**P1.** Export into trip folders, for example `2026-07_Lisbon/`, as an alternative to [FR-009](#fr-009--export-by-date).

### FR-015 — Duplicate detection

**P1.** Flag near-duplicate photos, such as bursts or re-saves, using perceptual hashing, and let the user exclude them from the print set. Choosing the best photo of a burst is [FR-041](#fr-041--best-shot-suggestions).

### FR-016 — RAW input

**P2 (v0.7.0, in progress).** Import camera RAW files natively, **without asking for DNG**: RW2 (Panasonic), CR2 and CR3 (Canon), NEF (Nikon), ARW (Sony), RAF (Fujifilm), ORF (Olympus / OM System) and DNG, each tested on a CC0 sample ([ADR-0023](adr/0023-raw-through-os-codecs.md)).

- **Through the operating system's codecs**, as HEIC is ([ADR-0017](adr/0017-heic-through-os-codecs.md)): ImageIO on macOS, with the file's type identifier as a hint so TIFF-based RAW is developed rather than read as a TIFF; WIC with Microsoft's free *Raw Image Extension* on Windows. The Rust RAW decoders and LibRaw are LGPL, which [ADR-0008](adr/0008-license-gpl3-with-relicensing-option.md) rules out.
- **Recognised by content**, never by extension: each format's own magic bytes, or for TIFF-based RAW the header plus IFD0 (`DNGVersion`, the maker in `Make`). CR3 is no longer mistaken for HEIC.
- **Two paths** ([NFR-017](#nfr-017--camera-raw-speed)): thumbnails and best-shot analysis use the **preview embedded in every RAW** (9–80 ms); the print preview, zoom and export use the **full develop** (0.4–2.2 s), one at a time, and its size is the only one used for crops.
- **EXIF** comes from the first 1 MB: RW2 and ORF are read as TIFF, RAF from its embedded JPEG, CR3 from its `CMT1`/`CMT2` boxes, Panasonic's ISO from its own tag. CR3's GPS is not read yet.
- **Windows without the extension:** import skips RAW with `import.raw_codec_missing` and says what to install.

### FR-017 — Photo book layout preview

**P2.** Preview how the selection lays out on album pages.

### FR-018 — QR code on photo back

**P2.** Generate a printable QR sheet that links each print to its digital original.

### FR-019 — Print service upload

**P2.** Opt-in upload to print lab APIs. This is the **only** feature that may use the network, and it must be explicitly enabled by the user.

### FR-020 — Localization

**P1.** The UI is available in **English (the default) and Vietnamese**. German follows later.

- Every user-visible string in the UI goes through the i18n layer. CI fails on a hard-coded string in a component and on a key that one language lacks.
- The language is chosen in the app settings, takes effect immediately without a restart, and is remembered ([DATABASE.md §6](DATABASE.md#6-app-settings)).
- Numbers, dates and sizes are formatted for the language, for example `18.6` and `18,6`.
- Errors from the Rust core are shown in the UI language, translated from their stable `code` ([SECURITY.md §8](SECURITY.md#8-resilience-and-error-handling)). The English `message` is the fallback.
- **Out of scope:** text printed on photos (the info frame's "Shot on …" line and the date stamp) does not follow the UI language. It is print content and stays as configured.

### FR-021 — Manual updates

**P2.** New versions are published on GitHub Releases and the website; people download and install them when they choose. There is **no in-app updater and no update check**, so the app never calls the network on its own ([NFR-006](#nfr-006--offline-and-zero-telemetry), [ADR-0021](adr/0021-versioning-and-automated-releases.md)).

- The app shows its version (for example `v0.1.0`) at the foot of the settings panel. Clicking it lists every release with two lines from its Highlights, read from the CHANGELOG bundled at build time (no network).
- Installing over an older version keeps settings, camera profiles and projects; their files are migrated on first open ([DATABASE.md](DATABASE.md)).
- Release notes list user-visible changes and any file format that changed ([RELEASE.md](RELEASE.md)).

### FR-022 — Culling (rate, flag, filter)

**P0.** Narrow a large import down to the photos worth printing.

- Rate 0–5 stars and flag photos as *pick* or *reject*, using single-key shortcuts (`0`–`5`, `P`, `X`, `U` to unflag).
- A loupe view shows one photo large, with arrow keys moving to the next and previous photo ([NFR-012](#nfr-012--culling-navigation)).
- Filter by rating, flag, date, camera, "has GPS", and "resolution warning". Only the filtered set is exported.
  - The flag filter is *All*, *Picks*, *Unflagged* or *Rejects*, combined with a minimum star rating. *All* includes rejected photos, because the grid and the export always show the same set.
  - The shortcuts work in the grid (on the selected photo) and in the preview. The preview also has clickable stars and pick/reject buttons. Every change can be undone with Cmd/Ctrl+Z.
  - Rejected photos stay visible but dimmed in the grid. Nothing is ever deleted.
- Ratings and flags live in the project file ([FR-023](#fr-023--projects-autosave-and-undo)). They are never written into source files.

### FR-023 — Projects, autosave and undo

**P0.** All work is kept in a project file (`.fotollection`): source folders, ratings, flags, crop frames, per-photo overrides and settings.

- Autosave is continuous ([NFR-013](#nfr-013--project-safety)). Reopening the app restores the last project.
- Undo and redo cover every user edit (ratings, flags, crop, overrides, settings), with at least 200 steps.
- If source files have moved, the user can relink the folder. Missing photos are marked, not dropped.
- **A document, like `.docx`:** the `.fotollection` file is saved with **Save** (Cmd/Ctrl+S) and **Save As** (Shift+Cmd/Ctrl+S) and opened with **Open** (Cmd/Ctrl+O). It is self-contained ([ADR-0016](adr/0016-self-contained-project-bundle.md)): the edits, the print settings, the info frame, the camera profiles and copies of the fonts and logos in use. Opening it on another machine installs those fonts and logos and gives the same prints. Photos are referenced, not embedded.
- **Unsaved changes:** the window title and the sidebar show them (`Trip — Edited`). New and Open ask *Save changes?* first (Save / Don't save / Cancel).
- **Folders move with the project:** folders are stored relative to the project file when possible, so a project copied together with its photos opens anywhere. A folder that cannot be found is listed in a **Relink** dialog; locating it restores its edits. Unrelinked folders are kept in the file on the next save.
- **Rename in place:** double-clicking the project name in the sidebar renames it. A saved document's file is renamed in its folder, never over another file. An untitled project keeps the name for its first Save. Names that are not valid file names on macOS and Windows are refused (separators, `: * ? " < > |`, reserved names such as `CON`, over 120 characters).
- **Double-click to open:** `.fotollection` files are associated with the app. Opening one runs the same unsaved-changes flow.
- **Session autosave:** underneath, every change also goes to a session project in the app data folder (crash safety). On launch it is reopened: folders are re-scanned, edits re-applied, and the document name and unsaved state come back.

### FR-024 — Copies and order summary

**P1.** Set the number of copies per photo (default 1).

- An order summary shows the count of prints per size, plus an **estimated cost** from an editable price table per lab.
- The summary can be copied as text and is written as `order-summary.txt` next to the export.

### FR-025 — Mixed print sizes per order

**P0.** Each photo, or selection of photos, can override the default print size ([FR-006](#fr-006--crop-to-print-ratio)).

- The export writes one subfolder per size (for example `10x15/`, `20x30/`), the way labs expect.
- The crop frame keeps its relative position when the size changes, and is clamped to the new ratio.

### FR-026 — Resolution warning

**P0.** For each photo, the app computes the effective DPI of the **cropped** area at the chosen print size, using the thresholds in [§5](#minimum-resolution-per-size).

- A badge in the grid and loupe shows *excellent*, *good* or *poor*.
- Before export, a summary lists photos below 150 DPI.
- **Size advice when choosing a print size.** Under the print-size picker, one sentence reads the photo selected in the grid: *"photo.jpg: the original is 6240 × 4160 px (26 MP). It prints sharp up to about 35 × 53 cm and is still acceptable up to 70 × 106 cm"* (the whole photo at 300 and 150 DPI, in whole centimetres). In the picker's list, and on the picker itself, every built-in and custom size carries a badge for that photo: **green** "Sharp" (≥ 300 DPI), **yellow** "Acceptable" (150–299 DPI), **red** "Pixelated" (< 150 DPI), from the DPI of the size's centred crop; hovering shows the DPI. Later: with several photos selected, the badge follows the weakest one.

### FR-027 — sRGB conversion for labs

**P0.** On export, colours are converted from the embedded ICC profile (Display P3, Adobe RGB…) to **sRGB**, which is what most labs expect. The conversion is on by default and can be turned off per preset.

- The rendering intent is configurable: perceptual (default) or relative colorimetric with black-point compensation.
- The output embeds the sRGB profile.
- Photos without a profile are assumed to be sRGB.
- **Done for share exports ([FR-053](#fr-053--export-for-social-media))**, with `qcms` and the perceptual intent ([ADR-0022](adr/0022-srgb-and-exif-for-share-exports.md)). Prints keep the original's profile until the print option lands.

### FR-028 — Safe area and borders

**P0.** Show a **safe-area overlay**: the trim margin a lab may cut, about 2 mm per edge by default and configurable per preset. Stamps are always placed inside the safe area.

- **Borders:** none, a uniform white border (mm), or a retro instant-photo style with a thicker bottom edge that can hold the stamp or caption.
- The border colour is configurable. The image is fitted inside the border, never stretched.

### FR-029 — Output sharpening

**P2.** Optional sharpening for print, tuned to the print size and paper type (glossy or matte), at three strengths (low, standard, high). It is applied after resizing and before encoding.

### FR-030 — Camera clock offset

**P0.** Correct capture times per camera (make, model and serial number when available).

- Shift by an offset (±days/hours/minutes/seconds) or set a time zone.
- A helper sets the offset by matching one camera photo to a reference photo, such as a phone photo of the same moment.
- Corrected times drive sorting, date stamps, grouping and GPS matching. The original EXIF time is kept for reference.

### FR-031 — Borrow GPS from a phone or GPX track

**P1.** Photos without GPS (for example from a Lumix GX9) get a location from:

- **another imported device's photos** taken within a configurable time window (default ±10 min), or
- **a GPX track**, by interpolating along the track.

Borrowed locations are marked as *inferred*, and the user can review them before stamping. This depends on correct clocks ([FR-030](#fr-030--camera-clock-offset)).

### FR-032 — Manual date and place

**P0.** Set the date and/or place by hand for one photo or a selection, for example scanned film prints or photos with no EXIF.

- Places are picked through an **offline** search of the bundled place dataset ([ADR-0003](adr/0003-offline-geocoding-geonames.md)).
- Manual values override EXIF and are marked as *manual*.

### FR-033 — Retro film date stamp and stamp presets

**P1.** Offer a **retro film-camera date stamp**: an orange seven-segment LED style in the classic `'98 10 23` format, with a soft glow, as on 1990s compact film cameras.

- This is shipped with the regular stamp styles from [FR-004](#fr-004--date-stamp).
- Stamp settings (style, font, size, colour, position, format) can be saved as named **presets**.
- A few presets are built in: *Retro LED*, *Minimal white* and *Classic serif*.
- Bundled fonts must use the OFL or MIT license.

### FR-034 — Lab presets

**P1.** Built-in presets per lab (CEWE, dm, Rossmann, Boots Photo…) bundle:

- the available sizes, colour space, JPEG quality, maximum file size and safe-area margin;
- the file-naming scheme.

One click applies a preset. Users can duplicate and edit presets. The presets are static data shipped with the app and are never fetched from the network.

### FR-035 — Multi-photo layouts

**P2.** Put several photos on one print to save cost: 2-up (two prints of 10×7.5 on a 10×15 sheet), 4-up, and a photo-booth strip (5×15, 3–4 frames). Each cell has its own crop. Cut guides are optional.

### FR-036 — Custom captions

**P1.** Add a free-text caption under the date stamp: a line of up to 80 characters, in the stamp's colour and LCD face, set in capitals without accents and running across the photo (done, settings v5; variables below are still to come). The caption can use the variables `{date}`, `{time}`, `{place}`, `{country}`, `{trip}`, `{camera}` and `{filename}`. Captions use the same layout rules as stamps (inside the safe area or on the border).

### FR-037 — Contact sheet

**P2.** Export a contact sheet (PDF or JPEG) of the print set: thumbnails with file name, date, place, size and copies. It is useful for archiving and for checking the order before sending it.

### FR-038 — Print presets

**P1.** Save, apply and share bundles of settings, in the spirit of Lightroom presets but for **print** settings.

- **Partial and stackable:** a preset owns only the setting groups it includes: size, crop, border, safe area, stamp, caption, style, colour, sharpening, export. Applying it leaves every other group untouched, so presets can be combined (for example *Film look* + *dm 10×15*).
- **Apply** to a selection, a whole project, or **automatically on import**.
- **Hover preview:** hovering a preset in the list previews it on the selected photo without applying it ([NFR-002](#nfr-002--preview-latency)).
- **Groups** organize presets in the sidebar (*Film looks*, *Labs*, *Stamps*, *My presets*). The stamp presets ([FR-033](#fr-033--retro-film-date-stamp-and-stamp-presets)) and lab presets ([FR-034](#fr-034--lab-presets)) are built-in groups of this system.
- **Linked, not copied:** photos reference a preset. Editing a preset updates every photo that uses it, after a confirmation showing the count. *Detach* turns a preset into per-photo overrides. A "modified" marker shows photos whose overrides differ from their preset, with *Reset to preset*.
- **Share:** import and export a `.fotopreset` file. A project embeds the user presets it uses, so it renders the same on another machine. The storage model is in [DATABASE.md §7](DATABASE.md#7-presets).

### FR-039 — Photo Style looks

**P1.** Apply a global colour look modelled on **Panasonic Lumix Photo Style** (as on the GX9). It is saved and shared through presets ([FR-038](#fr-038--print-presets)).

- **Base styles:** Standard, Vivid, Natural, Flat, Scenery, Portrait, Monochrome, L.Monochrome.
- **Parameters**, each in −5…+5 as on the camera: Contrast, Sharpness, Saturation and Hue (colour tone). Monochrome styles replace Saturation and Hue with a filter effect (yellow, orange, red, green) and a toning option.
- **Highlight / Shadow:** a tone curve with −5…+5 for highlights and for shadows. Note that the signs are reversed compared with Fujifilm recipes.
- **White-balance shift:** an A–B (amber–blue) and G–M (green–magenta) grid in −9…+9, like the Lumix WB fine-adjust.
- **Grain:** off, low, standard or high, applied to colour and monochrome alike.
- **Built-in recipes:** *C200 Natural*, taken from the owner's GX9 recipe (Natural base; Contrast −1, Sharpness −3, Saturation +1; highlights slightly darker, shadows slightly brighter; WB A+3 / M+1), plus a few classic film-inspired looks.
- **Scope boundary:** these are **global** adjustments on the already-rendered JPEG/HEIC (or a RAW preview). There are no local masks, HSL panel, curve editor, retouching or RAW development ([ADR-0011](adr/0011-photo-style-looks.md)).
- **Non-destructive:** the look is applied in preview and at export. Stamps, captions and borders are drawn **after** the look, so they are never tinted. A before/after toggle (`\`) compares with the original.

### FR-040 — EXIF info frame

**P1.** Add a "Shot on …" information frame to prints, like camera makers' companion apps (for example LUMIX Lab).

- **Two ways to make room for the bar:**
  - **Crop to fill:** the print keeps the selected size exactly (10 × 15). The bar is subtracted from it and the photo is cropped to the space above; drag the photo in the preview to choose what stays.
  - **Fit whole photo:** nothing is cut or shrunk, like an instax print. The whole photo keeps the print's width (10 cm on a 10 × 15), and a **5 mm border** is added on the top, left and right, with the bar below. The print is therefore larger than the chosen size (a 2:3 photo at 10 × 15 becomes about 11 × 17.9 cm), and the print-size card shows that size for the photo at hand, so the right paper can be asked for at the shop.
  - The bar is the same height in both modes.
- **Bar content:**
  - Left: the brand: its bundled logo for 12 brands, including Noritsu for film scans (default, [ADR-0020](adr/0020-bundled-brand-logos.md)), the typeset name, a user PNG ([ADR-0014](adr/0014-user-supplied-brand-logos.md)), or nothing. Logo colours: *Classic* (black wordmarks with colour accents kept, colour badges as they are), *Brand colours* or *One colour*; on a black bar, dark parts turn white. A **Logo size** slider (50–200 %, default 100 %, double-click to reset) scales the logo for every photo in the project; Rust clamps the value, and a larger logo may use up to half the bar's width but never reaches the text.
  - Right-aligned lines: `Shot on <camera>`, then `<lens> <focal> <aperture> <shutter> <ISO>`, then `<photo style>`, then `<date> <place>`.
  - **Text never covers the logo.** When the exposure line would reach the logo (a long lens name such as `30mm F1.4 DC DN | Contemporary 016`, mostly on portrait prints), the lens goes on a line of its own above the settings. A line still too long on its own is set smaller.
  - The bar is white or black. Text and logo stay inside the safe area ([FR-028](#fr-028--safe-area-and-borders)).
- **Choose what shows:** every item can be switched on or off. Lines with no data are dropped.
- **Edit per camera:** the camera name, brand text, lens, focal length, aperture and photo style can be overridden. This covers **manual lenses** that report nothing in EXIF. Overrides are remembered per camera across sessions ([DATABASE.md §6](DATABASE.md#6-app-settings)). Empty fields fall back to EXIF.
- **Edit where you look:** every frame setting (on/off, crop or fit, colour, date format, font, the shown items) can be changed both in the print settings panel and inside the preview, next to the photo, without closing the preview. Both places edit the same settings.
- The date stamp on the photo ([FR-004](#fr-004--date-stamp)) stays independent.

### FR-041 — Best-shot suggestions

**P1.** When several photos are taken of the same scene, the app suggests the best one and explains why. Nothing is rejected or deleted automatically: the user confirms. The method follows Lightroom Assisted Culling, Google Top Shot, Narrative and Aftershoot.

- **Grouping.** Photos are ordered by capture time, including `SubSecTimeOriginal`. A new group starts when:
  - the gap exceeds max(2 s, 4 × the median gap);
  - the camera or lens changes;
  - GPS moves by more than 30 m.

  A focal-length change alone does not split a group: re-shooting closer is common, and the zoom is allowed for when the shots are compared.
- **Group confirmation.** Each photo has a visual signature: a 32 × 24 luma layout and a hue × saturation histogram ([ADR-0019](adr/0019-grouping-by-visual-signature.md)). A shot joins the group when its layout correlates ≥ 0.6 with the previous shot and its colours overlap ≥ 0.72. The comparison allows shifts of about ±10 % and zooms of ×0.85–1.18, or the EXIF focal-length ratio when the shots were zoomed. Against the group's first shot it must still correlate ≥ 0.4, so that groups do not chain. A group holds at most 30 photos.
- **Accuracy.** Grouping and picks are scored against hand-labelled photos with `eval_bestshot` (README, [RESULT.md §8](RESULT.md#8-best-shot-accuracy)). A change must not lower the scores.
- **Metrics.** Each metric is computed in one pass over a greyscale copy of the T2 preview, reduced to 1,024 px:
  - Sharpness: variance of Laplacian and Tenengrad on a 16 × 12 tile grid. The median scores the whole photo; the 90–95th percentile, or the face tiles, score the subject.
  - Camera shake: structure-tensor coherence, relative to the group median. EXIF adds a hint when the shutter is slower than 1/(focal × crop factor).
  - Exposure: the clipped-highlight share (≥ 250), the crushed-shadow share, the median luminance against 118 (in EV), and RMS contrast.
  - Noise: the Immerkær estimate over the flatter half of the image (gradient below the median), so that the sharper frame of a burst does not look noisier.
- **Faces and eyes.** Faces are detected with YuNet, and the eyelid landmarks of FaceMesh V2 give the eye aspect ratio ([ADR-0018](adr/0018-face-models-and-runtime.md)). An eye counts as closed when its ratio is below 0.2, or below 0.75 × the same person's maximum in the group. Face and eye regions also get their own sharpness score, used when every shot of the group has a face; otherwise all shots are compared on whole-image sharpness. In the eyes term, each open pair of eyes counts +1, each closed pair −2, and each face turned away or hidden (fewer faces than the most in any shot of the group) −0.5.
- **Score.** Each metric is normalized with a robust z-score (median/MAD) **within the group**; groups smaller than 4 use rank instead. Each metric has a smallest spread (for example 0.15 EV or 2 % of clipped highlights), so differences too small to see in a print never count as several sigmas. The weights are:

  | Metric | Weight |
  |--------|-------:|
  | Subject sharpness | 0.35 |
  | Eyes open | 0.20 |
  | Shake | 0.15 |
  | Exposure | 0.15 |
  | Clipped highlights | 0.10 |
  | Noise | 0.05 |

  Hard faults, such as closed eyes or heavy blur, are shown as reasons. They are never applied silently.
- **Explanation.** The suggestion names up to three metrics that differ most from the runner-up, for example "34 % sharper", "Eyes open" or "Fewer clipped highlights: 0.3 % vs 4 %". Sharpness is given as the difference in edge strength, because small groups have no meaningful sigma. When the top two scores are within half a sigma of subject sharpness (0.175), it says **"Too close to call"**.
- **UI.**
  - A **Groups** view shows one strip per group, with a **"Best of N"** badge and its reasons.
  - A compare view shows the group's photos side by side, with a bar for each metric and faces and eyes marked.
  - **"Keep best"** picks the best photo and rejects the rest of the group, using [FR-022](#fr-022--culling-rate-flag-filter) flags. It can be undone.
- **Models.** Models ship inside the app, are checksum-verified, and are never downloaded. Only permissive weights are allowed: YuNet (MIT) and MediaPipe FaceMesh V2 (Apache-2.0), both verified from their sources. InsightFace/SCRFD and pyiqa models are excluded because of their non-commercial licences. The runtime is pure-Rust `tract-onnx` ([ADR-0018](adr/0018-face-models-and-runtime.md)).
- **Cache.** Analysis results are derived data, so they live in the cache keyed by fingerprint, never in the project file ([DATABASE.md](DATABASE.md), rule N4).
- **Speed:** [NFR-014](#nfr-014--best-shot-analysis-speed).

### FR-042 — Custom fonts

**P1.** Import **OTF/TTF** fonts and choose them for the info-frame text ([FR-040](#fr-040--exif-info-frame)). The text inside the photo always uses the bundled LCD face ([FR-004](#fr-004--date-stamp)); its font choice was removed to keep the stamp simple (settings v5).

- Fonts are validated before use: TTF/OTF magic bytes (font collections and WOFF are rejected), a 20 MB cap, a full parse, and digits 0–9 must be present. They are then copied into the app's config folder, like logos ([ADR-0014](adr/0014-user-supplied-brand-logos.md)).
- Variable fonts render at their default instance.
- The bundled Inter remains the default. Removing a font switches anything using it back to Inter.
- Font licensing is the user's responsibility: the app never redistributes imported fonts.

### FR-043 — Custom print sizes

**P1.** Define and save custom print sizes, in **cm or inches** (for example 4 × 6 in or 13 × 13 cm). They appear next to the built-in sizes everywhere: crop, frame, DPI and export.

- A size has a name and short/long edges, stored in whole millimetres. The entry unit is remembered for labels, so a size shows as `4 × 6 in`.
- Every size in the picker, built-in or custom, shows its reduced aspect ratio, for example `10 × 15 cm · 2:3`. Sizes entered in inches snap to the simple ratio despite mm rounding, so 4 × 6 in shows `2:3`.
- Edges must be between 2 and 100 cm. Up to 30 custom sizes are allowed. Custom sizes are stored in `settings.json` ([DATABASE.md §6](DATABASE.md#6-app-settings)).

### FR-044 — Per-photo overrides

**P1 for the date stamp (done), P2 for the rest.** The **date stamp** can already be set per photo: in the preview, every stamp control (on/off, format, corner, colour, place, caption, the same fields in the same order as the settings panel) applies to the open photo only, with a *This photo* badge and an **Apply to all** button that makes it the setting for every photo. The settings panel shows **Reset N photos with their own stamp** whenever some photos deviate. Each change is undoable. Stored per photo in the project (`photos[].stamp`, project schema v3).

Next to the batch settings, any other setting will later be overridable for one photo or a selection the same way. For example, the date stamp can be turned off for photos that already have a timestamp burned in, while the rest of the batch keeps it. Overrides are stored sparsely per photo ([DATABASE.md §4](DATABASE.md#4-project-file-schema), `photos[].overrides`). The UI shows which photos deviate from the batch and offers *Reset to batch*. Until then, every setting applies to the whole batch.

### FR-045 — Original aspect ratio

**P1.** An **Original** print size keeps each photo's own aspect ratio, so nothing is cropped. It sits first in the print-size picker.

- The print is 10 cm on the short edge, and the long edge follows the photo: a 4:3 photo prints at 10 × 13.3 cm, like a lab's "digital" format, and a 3:2 photo at 10 × 15 cm. The long edge is capped at 100 cm. This physical size drives the date stamp size and the DPI check ([FR-026](#fr-026--resolution-warning)).
- The crop is the whole photo, computed from the exact pixel ratio, so millimetre rounding never cuts a pixel. The crop editor can still zoom and move the crop, at the same ratio.
- With the info frame ([FR-040](#fr-040--exif-info-frame)), the whole print keeps the photo's ratio, and *Crop to fill* or *Fit whole photo* applies to the area above the bar.

### FR-046 — Swipe culling

**P2 (future).** A full-screen, keyboard-first culling mode, inspired by swipe apps (right = like, left = pass) and Lightroom's loupe. It builds the **print list** at the speed of a key press.

- One photo fills the screen. **↑ adds it to the print list, ↓ leaves it out** (already in the preview since v0.5.0, as the print basket and set aside of [FR-050](#fr-050--print-basket)), and both move on to the next photo at once. **← / →** step back and forward without deciding. **Space** shows the photo without overlays, and **Cmd/Ctrl+Z** undoes the last decision.
- Leaving a photo out only removes it from the list. The file is never deleted or moved.
- The print list is the set that gets exported. It builds on pick/reject ([FR-022](#fr-022--culling-rate-flag-filter)): ↑ is *pick*, ↓ is *reject*, and an undecided photo stays *unflagged*. Stars keep working inside the mode (`0`–`5`).
- A progress bar shows decided / total and the size of the print list. The mode can resume where it stopped, and best-shot suggestions ([FR-041](#fr-041--best-shot-suggestions)) can pre-sort a burst so the best frame comes first.
- The next photos are prefetched so every decision shows a sharp image in under 100 ms ([NFR-012](#nfr-012--culling-navigation)).
- **Workflow change:** the main flow becomes *import → swipe → refine (crop, stamp, frame) on the print list only → export*. The grid filter, the preview and the export default to the print list once it is not empty. The vision (§2) and the UI flow are updated when this lands.

### FR-047 — Film looks

**P2 (future).** A set of **film-simulation looks** comparable to Fujifilm's in-camera film simulations, applied with one click to a photo or the whole batch. They use the same engine as Photo Style ([FR-039](#fr-039--photo-style-looks), [ADR-0011](adr/0011-photo-style-looks.md)): a 3D LUT, a tone curve, a white-balance shift and grain.

- **The looks, by intent:** a neutral slide (like Provia), a vivid landscape slide (like Velvia), a soft portrait slide (like Astia), a muted documentary chrome (like Classic Chrome), a hard-shadow negative with shifted colours (like Classic Negative), a warm amber negative (like Nostalgic Neg), a flat cinema look (like Eterna), a natural modern negative (like Reala Ace), a soft studio negative (like Pro Neg Std), and fine-grain black-and-white with filter options (like Acros). Each has *strength* 0–100 % and the usual Photo Style parameters on top.
- **Names and data are ours.** Fujifilm's film-simulation names are trademarks, so the UI uses descriptive names of its own (for example *Chrome Muted*, *Negative Classic*). Fujifilm appears in the docs only to describe the inspiration. The LUTs and curves are built from the project's own reference shots and published recipes, and never copied from Fujifilm software or from third-party LUT packs without a compatible licence.
- **Recipes:** a look plus Photo Style parameters saves as a preset ([FR-038](#fr-038--print-presets)), so recipes like the owner's *C200 Natural* can be shared. Looks are stored by id in the project, so a bundle opens with the same look on another machine ([ADR-0016](adr/0016-self-contained-project-bundle.md)).
- **Order and speed:** the look is applied before the date stamp and the info frame, so they are never tinted. Applying a LUT to the 1600 px preview must keep the preview within budget ([NFR-002](#nfr-002--preview-latency)), and this is benchmarked before release.

### FR-048 — Normal and Professional modes

**P1 (next).** Two modes, switchable at any time and remembered per machine. They share one library, one project file and one rendering engine.

- **Professional** is everything built so far: the inspector, the crop editor, the info frame and camera profiles, fonts, custom sizes, filters, the `.fotollection` document, and full-quality export (q95, 4:4:4, full resolution).
- **Normal** is the "instant" mode: the shortest path from a folder to prints.
  - **One screen, three steps:** open a folder → keep or drop photos with one key or one click (the same pick/reject as [FR-022](#fr-022--culling-rate-flag-filter), later the swipe mode of [FR-046](#fr-046--swipe-culling)) → **Export**.
  - **Few choices, good defaults:** print size (10×15 by default), date stamp on or off, and the destination. Crop is centred automatically, and everything else uses the Professional defaults. No inspector, frame editor or fonts on screen.
  - **"Good enough for print, very fast":** the export is sized for the print, at 300 DPI on the chosen print size (about 1800 × 1200 px for 10×15) instead of full resolution, and encoded at q90. That is visually the same on a 10×15 print, the files are much smaller, and export is several times faster ([NFR-015](#nfr-015--normal-mode-speed)). Decoding at reduced scale when the source is much larger than needed is evaluated as part of this.
  - The grid opens on embedded EXIF thumbnails only, so a folder is browsable at once.
- **Switching** keeps all work: photos flagged or cropped in one mode are the same in the other. A Normal export of a project that has Professional settings (frame, fonts) uses them, but at Normal export quality.
- **Not in Normal:** RAW, per-photo overrides, presets editing, culling suggestions UI. They stay in Professional.

### FR-049 — Trip journal

**P2 (future, not started).** An offline **journal of a trip**, written from the album itself: a one-page snapshot and report of what the photos say, like a travel diary the app drafts for you. For example: *"6 days, 14–19 September 2025 · Debrecen, then Budapest · 412 photos, 87 kept · Canon PowerShot and iPhone 15 · focal lengths 5–25 mm (28–140 mm equivalent), mostly wide · busiest day: 17 Sep, 128 photos · golden-hour sunsets on 3 evenings".*

- **Everything is derived from data the app already has:** capture times (days, first and last photo, busiest day and hour), places from GPS, borrowed or manual places ([FR-005](#fr-005--location-stamp-offline), [FR-031](#fr-031--borrow-gps-from-a-phone-or-gpx-track), [FR-032](#fr-032--manual-date-and-place)), cameras and lenses, focal-length, aperture, shutter and ISO ranges from EXIF, counts of picks, rejects and stars ([FR-022](#fr-022--culling-rate-flag-filter)), and best-shot groups ([FR-041](#fr-041--best-shot-suggestions)).
- **Offline and private:** the text is generated from templates in the interface language (English, Vietnamese), with no language model in the cloud and no network. The places come from the bundled GeoNames data.
- **Structure:** a summary block, then one section per day: date, places in visiting order, photo count, a strip of that day's best shots, and notable facts (the longest burst, the widest and longest focal length used). It builds on trip grouping ([FR-012](#fr-012--trip-auto-grouping)) when that exists; until then a trip is the whole album or a date range.
- **Output:** shown in the app, and exportable as a self-contained HTML or PDF page, and optionally as one extra print (a journal card) in the chosen print size. Every sentence can be edited before export.
- **Statistics shown are measured, never guessed.** Where the app is unsure (for example a place inferred only from time), it says so ("probably Debrecen").
- **Journal entries from editing (idea from the instax app).** Every editing session is also a journal entry: when the user works on a photo (crop, straighten, date stamp, the caption line, stars, pick), the app records a short entry with the time, the photo and what was done, and the caption the user typed becomes that entry's text. The trip journal gathers these entries per day next to the facts derived from the album, so the diary is written while editing, not afterwards. This needs an edit history saved in the project (a later project schema).

### FR-050 — Print basket

**P1 (in progress, v0.5.0).** Two **holding areas** for the photos of a trip, separate from culling: the **print basket**, the photos that will really be printed, and **set aside**, the photos taken out of the gallery for now. A typical batch: 100 photos are edited together, filters keep 60, only 5–10 go to the lab. Step-by-step use: [USER_GUIDE.md §1](USER_GUIDE.md#1-choosing-the-photos-to-print).

- **One place per photo:** the gallery, the print basket or set aside, never two. Independent of stars and pick/reject ([FR-022](#fr-022--culling-rate-flag-filter)): a pick does not have to be printed.
- **Set aside is never deletion:** the photo leaves *All photos* and the folders, its file is never touched, and it can be brought back at any time.
- **The sidebar lists both areas** under *All photos*, with their counts. Opening one shows only its photos; *Export* then follows the view, so the basket exports exactly the basket and photos set aside are never exported from the gallery.
- **In the preview:** **↑** puts the photo in the basket and moves to the next photo; **↓** sets it aside and the next photo takes its place. A *Print basket* button on the photo does the same as ↑, and a photo set aside offers *Back to the gallery*.
- **In the gallery, select mode** (like the Select button on an iPhone): *Select* turns clicks into ticks (Shift+click for a range, Cmd/Ctrl+A for all), and a bar moves the ticked photos to the basket, aside, or back to the gallery. Ticks are temporary; the areas are saved.
- **Saved and undoable:** stored per photo in the project (schema v6, [DATABASE.md](DATABASE.md)); every move is one undo step.
- **Later:** "Also export the print basket" into an extra folder (for example `Print/`), and, with copies and the order summary ([FR-024](#fr-024--copies-and-order-summary)), the order itself.

### FR-051 — Landmark recognition

**P2 (future, not started).** Suggest a place for photos **without GPS** from what they show. For example, a photo of the Eiffel Tower is suggested as *Paris*. It complements time and visual matching with phone photos ([FR-031](#fr-031--borrow-gps-from-a-phone-or-gpx-track)).

- **Offline only.** A bundled model recognises famous landmarks, and its labels are joined to coordinates from Wikidata (CC0) and the bundled GeoNames places ([ADR-0003](adr/0003-offline-geocoding-geonames.md)). Nothing is sent anywhere.
- **Licences checked first** (research of 2026-10-24):
  - **Best candidate:** Google's on-device landmark classifiers (weights under Apache-2.0, trained on GLDv2 with CC BY / CC0 data), about 45,000 landmarks for Europe. The model card lists "inferring the user's location" as an unsuitable use, so this needs an ADR.
  - **Fallback:** TinyCLIP (MIT) matched against a bundled list of Wikidata landmarks.
  - **Rejected:** general geolocation models, because of non-commercial licences (StreetCLIP), GPL (PlaNet), missing weights (PIGEON) or more than 1 GB (GeoCLIP). Even the best of them places only about a third of photos within 25 km.
- **Size and speed:** the model probably does not fit the 25 MB installer ([NFR-008](#nfr-008--install-size)). It ships as an **optional model pack** that the user installs on purpose; analysis stays within [NFR-014](#nfr-014--best-shot-analysis-speed)-like budgets, measured first.
- **Shown as a suggestion only,** with its confidence, and never applied silently. It is scored on labelled photos like the best-shot suggestions before it ships.

### FR-052 — Appearance: theme, title bar and hints

**P1.** The app looks like one piece on both platforms.

- **Theme:** System, Light or Dark, chosen with a three-way switch at the foot of the library column and stored per machine (`settings.json` `ui.theme`, schema v6). The native window (title bar, scroll bars) follows the choice. The colours come from the logo (STYLE_RULE §4).
- **Windows title bar:** on Windows the system's white title strip is replaced by one drawn in the app's colours, like VS Code's: the app mark and the project name, a drag area (double-click maximizes) and minimize, maximize/restore and close buttons. It stays usable above open dialogs. macOS keeps its native title bar.
- **Hover hints:** every hint is drawn by the app in its own style (rounded, theme colours), never the operating system's square tooltip.
- **One-line toolbar and a minimum window size:** the toolbar above the photos never wraps (the photo filters are two dropdowns), and the window cannot be made narrower than 1,180 px, the width at which the library, the toolbar and the settings column all fit.

### FR-053 — Export for social media

**P1.** Camera photos are heavy (16–26 MP, 10–20 MB). Uploaded as they are, Facebook shrinks and recompresses them, and photos in Display P3 or Adobe RGB look washed out. A **Share** export makes light files at the size each platform shows best, keeping about 90 % of the original's colour and detail on screen.

- **Export target:** *Print* (the full-quality print export, [FR-008](#fr-008--original-quality-export)) or *Share online*, chosen in the **Print / export size** card together with the size (the print size for prints, the platform for sharing), in the settings panel and in the preview; the export card then only exports. The choice is remembered (`settings.json` `render.exportTarget`, schema v8).
- **Sizes:** never enlarged, the photo's own aspect ratio kept (no print crop):
  - **Original size:** same width and height, only a lighter file (measured: a 3 MB compact-camera JPEG becomes 1.4 MB; large, lightly compressed camera files shrink more);
  - **Facebook:** 2048 px on the long edge, the size Facebook shows without resizing;
  - **Instagram:** 1080 px wide; portraits stop at 3:4 (1080 × 1440), as Instagram crops anything taller;
  - **Custom:** a long edge of 640–6000 px, clamped in Rust.
- **Pipeline:** the photo's edits (crop, straightening, date stamp, info frame) as for prints, then conversion to **sRGB** ([FR-027](#fr-027--srgb-conversion-for-labs)), a light unsharp mask (0.2) after a downscale, JPEG with 4:2:0 chroma and optimized Huffman tables, at quality 88, or 80 for *Original*.
- **Metadata:** camera, lens, exposure and capture time, and `Software = "Fotollection <version>"`, which phone galleries show as the editing app (as they do for Snapseed). **GPS is never written** ([FR-010](#fr-010--metadata-privacy-on-export)).
- **Where:** `<destination>/Share/<YYYY-MM-DD>/`, apart from prints.
- **How much lighter, before and after:** the export card estimates the size before exporting ("Estimated: 17.0 MB → ≈ 1.4 MB, 92 % lighter"), from up to four photos encoded in memory with the current settings and updated as they change, exact when every photo was sampled; when the export finishes, it reports the real figure. Print files, which keep every detail, can be larger, and the card says so.
- **Measured, not guessed** ([RESULT.md §10](RESULT.md#10-photos-for-social-media)): against the original resized with Lanczos, SSIM ≈ 0.98 for every preset and mean ΔE2000 0.96 for *Original* (at about half the file); after a simulated Facebook recompression, closer to the photo than the usual way and 99.6 % of the colour of Display P3 photos kept (86 % the usual way). Resized photos stay at ΔE2000 1.3–1.6 because of the 4:2:0 chroma every platform recompresses to.

### FR-054 — Open photos with Fotollection

**P2 (v0.7.0, in progress).** Fotollection can replace a classic photo viewer (IrfanView, Photos): choosing it in the system's "Open with" menu for a JPEG, PNG, HEIC or camera RAW shows that photo straight away in the preview, and ← → browse the other photos of its folder ([ADR-0024](adr/0024-optional-photo-viewer.md)).

- **Never the default viewer.** macOS lists Fotollection under "Open With" (rank `Alternate`); Windows lists it under each type's `OpenWithProgids`, written by the installer and removed on uninstall. Making it the default is the user's choice in the system's settings.
- **The photo's folder becomes a tab** in the current session, listed without its subfolders so a photo in Downloads opens fast; a photo already in the library opens in its existing tab. Basket, edits and export work as for any folder.
- **One instance:** opening another photo while the app runs brings the running app forward with that photo.
- Paths come from the operating system, not the webview, and are checked like imported files ([SECURITY.md](SECURITY.md)). A request that arrives during start-up waits for the last session to be restored and is never lost.
- The time from opening to the first photo on screen has its budget, [NFR-016](#nfr-016--open-to-first-photo).

### FR-055 — Export from the preview

**P1.** Liking a photo in the preview is enough to export it: the preview's inspector has an **Export** card, the same as the settings panel's (the destination and the button; Print or Share online is chosen in the Print / export size card above it), that exports **only this photo**. An **Export** button on the photo jumps to it.

- One export job and one destination are shared by the settings panel and the preview: a destination chosen in one is used by the other, and the progress shows in both.
- `start_export` takes the photo's id; Rust exports only that photo, and an unknown id exports nothing.

### FR-056 — Folders as tabs

**P1.** With several folders imported, the library list works like browser tabs.

- **All photos** sits above the folders, with the total count: the grid shows every photo of every folder.
- **Clicking a folder shows only that folder**: the grid, the best-shot groups and the export all work on it, like switching tabs. The folder counts never depend on the star or flag filter.
- **Several folders:** the select button above the list (or Ctrl/⌘-click) turns the folders into tick boxes; the grid and the export then cover the ticked folders.
- **Together or separately:** when the export covers two folders or more, the export card asks whether they go **together** (the usual date folders) or **separately** (one subfolder per source folder, named after it; two folders with the same name get " (2)").
- **Ticks follow the tabs:** in select mode ([FR-050](#fr-050--print-basket)), photos ticked in one folder stay ticked when another folder or *All photos* is shown, so the best of several folders goes to the basket in one move. The selection bar counts every ticked photo and says how many are in other folders. Opening the basket or the set-aside view, or leaving select mode, clears the ticks.
- The choice of folders is view state, not saved in the project. Removing a folder shows all photos again, since folder ids are renumbered.

### FR-057 — Zoom in the preview

**P1.** The preview zooms like a photo viewer, to check focus and detail.

- **Trackpad pinch** (sent as Ctrl + wheel by WebKit and WebView2) and **Ctrl/⌘ + / − / 0** zoom about the pointer, from fit up to 800 %; a **double-click** toggles fit and 200 %.
- Zoomed in, a **drag** or a two-finger scroll moves the photo.
- **Zoom buttons** at the bottom right of the photo (− · level · + · fit), so zoom can be found without knowing the shortcuts; each tooltip names its shortcut and the pinch.
- Another photo, or the crop editor, starts again at fit; ← → still browse.
- **A safe preview, full detail on demand.** The preview is never larger than 1,600 px, rendered from a 2,048 px copy, whatever the camera: a weak machine never decodes a 32 MP photo just to show it. Zoomed past the preview's own pixels, once the view has been still for 150 ms, the UI asks Rust for **only the visible part** (plus a 10 % margin) at the screen's pixel count, at most 4,096 px wide. Rust decodes the original then, keeps at most about 200 MB of originals (two 32 MP photos; the one on screen always stays), renders it exactly as export does and sends that part. The zoomed preview stays on screen meanwhile, with a spinner beside the zoom level. Measured cost, REF-MAC: decode 46 ms (16 MP) to 84 ms (32 MP) the first time, then 17 ms per region ([RESULT.md §11](RESULT.md#11-preview-and-zoom-cost-v060)).

### FR-058 — Lightroom compatibility through XMP

**P2 (planned, not started).** Work culled in Lightroom, Bridge, Capture One or darktable comes into Fotollection, and Fotollection's work can go back out, through **XMP** sidecar files: XMP is XML with standard fields those apps share, so it is used instead of a format of our own.

- **Read on import (always, read-only):** an `IMG_0416.xmp` or `IMG_0416.RW2.xmp` next to a photo gives its stars, the reject flag (`xmp:Rating = -1`), the crop and the straightening (`crs:Crop*`, `crs:CropAngle`), for photos not yet edited in Fotollection.
- **Write (a setting, off by default):** the same standard fields, plus Fotollection's own work (stamp, frame, print size, pick flag) in a `fotollection:` namespace that other apps ignore. Only `.xmp` files are created or updated, atomically, never a photo; an existing sidecar keeps every field Fotollection does not own (edited at the XML event level with `quick-xml`, MIT).
- **Known limit, to confirm with Lightroom itself:** Lightroom Classic reads sidecars only for RAW files; for JPEG and PNG it reads XMP embedded in the file, which Fotollection does not modify. Lightroom's pick flag lives only in its catalog.
- Writing next to photos changes the "nothing is written into source folders" rule, so it needs an ADR and updates to [SECURITY.md](SECURITY.md) and §9.

### FR-059 — Quick-preview

**P1 (in progress, v0.5.0).** In the gallery's select mode ([FR-050](#fr-050--print-basket)), **holding Space** over a photo shows it large, as it will be exported, and releasing Space closes it, like Quick Look on macOS. Step-by-step use: [USER_GUIDE.md §1.3](USER_GUIDE.md#13-quick-preview-look-without-opening-select-mode-only).

- It shows the photo under the pointer, or the last one ticked; the thumbnail appears at once and the Rust render with the gallery's print size, stamp and frame replaces it (about 17 ms warm, 120 ms the first time).
- **Nothing can be edited**: it is a look, not the preview with its inspector ([FR-007](#fr-007--live-preview)). While Space is held, ← → show the neighbours and Enter ticks or unticks the photo shown.
- Outside select mode, Space keeps opening the normal preview.

### FR-060 — Start-up screen

**P1 (v0.6.0).** Opening the app shows the **logo and a loading bar** on the theme's background, never a half-built window: the screen is painted by the page itself before any script runs, and it fades out onto the finished workspace once the last session is back ([FR-023](#fr-023--projects-autosave-and-undo)). When the session reopens folders, the bar shows the import's progress, like the import bar; otherwise it slides. It never stays more than 15 s. It is the place where the choice between Normal and Professional mode ([FR-048](#fr-048--normal-and-professional-modes)) will be asked the first time.

## 7. Non-functional requirements

Performance budgets are measured as defined in [OPTIMIZATION.md](OPTIMIZATION.md). The reference machines are listed there.

### NFR-001 — Library load time

Opening a folder of 1,000 JPEGs shows the first screen of thumbnails in **under 1 s**. All thumbnails are ready in **under 10 s**.

### NFR-002 — Preview latency

A change to stamp or crop settings re-renders the visible preview in **under 50 ms** (p95).

### NFR-003 — Export throughput

Export (decode → crop → stamp → encode → write) runs at **8 or more 24 MP JPEGs per second** on an 8-core reference machine.

### NFR-004 — Memory ceiling

Peak resident memory during export stays **at or below 2 GB**, regardless of batch size.

### NFR-005 — UI responsiveness

Heavy work never runs on the UI thread. Input-to-feedback latency is **under 100 ms** during an export.

### NFR-006 — Offline and zero telemetry

The app works with no network connection. By default it makes **zero** network requests: no telemetry, no crash reporting and no update checks unless the user opts in.

### NFR-007 — Platforms

The app supports macOS 11 or later (Apple Silicon and Intel, shipped as a universal binary) and Windows 10/11 x64.

### NFR-008 — Install size

The installer is **at most 25 MB** per platform.

### NFR-009 — Colour fidelity

The output either keeps the source ICC profile or, when sRGB conversion is on ([FR-027](#fr-027--srgb-conversion-for-labs)), converts it exactly. The result is within ΔE₂₀₀₀ < 1 of a reference `lcms2` transform. Without conversion, a pixel outside the stamp area differs from a reference decode by at most 1 per channel, apart from JPEG re-encoding error at the configured quality.

### NFR-010 — Cold start

The app goes from launch to an interactive window in **under 1.5 s**. Measured 2026-10-25 on REF-MAC with the release build (`scripts/measure-startup.sh`, milestones printed only when `FOTOLLECTION_TIMING` is set): Rust ready in 0.21–0.28 s, page loaded in 0.31–0.40 s and the first thumbnail in 0.50–0.58 s on later launches. The very first launch after install or a new build took 2.7 s, 2.1 s of it the system web view starting for the first time, with the machine deep in swap; the start-up screen ([FR-060](#fr-060--start-up-screen)) covers that time. To re-measure on a cleaner machine and on Windows.

### NFR-011 — Robustness

No input file, however malformed, can crash the app or hang the batch. A bad file only fails that one photo.

### NFR-012 — Culling navigation

In the loupe view, moving to the next or previous photo shows a sharp preview in **under 100 ms** (p95). Neighbouring photos are prefetched.

### NFR-013 — Project safety

Edits reach the project file within **5 s**, and writes are atomic, so a crash or power loss never corrupts the project. Measured 2026-10-25 (`examples/measure_project.rs`): a full save takes 10 ms for 5,035 edited photos and 70 ms for 70,000, the library being locked 21 ms of it; the autosave runs at most once a second. **Planned:** an append-only edit journal (each undoable step written at once, the full snapshot every 30 s), so a crash loses at most one step and the save cost no longer grows with the library; a new ADR replaces that part of [ADR-0010](adr/0010-project-file-format.md). The file has a `schema_version` and is migrated forward automatically. Older app versions refuse to open newer files rather than damage them.

### NFR-014 — Best-shot analysis speed

Best-shot analysis ([FR-041](#fr-041--best-shot-suggestions)) takes **≤ 20 ms per photo without faces and ≤ 40 ms with faces** on REF-MAC, running in parallel on the background pool. It never blocks the grid or the preview. The budget is benchmarked before the feature ships.

### NFR-015 — Normal mode speed

In Normal mode ([FR-048](#fr-048--normal-and-professional-modes)), export of 16–24 MP JPEGs to 10×15 at 300 DPI reaches **at least 3× the Professional throughput** on REF-MAC (target ≥ 50 photos/s), with peak memory still within [NFR-004](#nfr-004--memory-ceiling). The first screen of a 1,000-photo folder appears in **under 0.5 s**. Budgets are confirmed by benchmark before the mode ships.

### NFR-016 — Open to first photo

Opening a photo from Finder or Explorer ("Open with", [FR-054](#fr-054--open-photos-with-fotollection)) shows it in the preview in **under 1 s** on REF-MAC, after the first launch. Measured end to end on 2026-10-25 (`open -a` on the release bundle): 0.70 s for a JPEG whose folder is already a tab; 1.25 s when the folder is new, because it is imported before the photo shows (to improve); a RAW takes 1.4–2.5 s, its develop included ([NFR-017](#nfr-017--camera-raw-speed)).

### NFR-017 — Camera RAW speed

For camera RAW ([FR-016](#fr-016--raw-input)), on REF-MAC: a thumbnail or the input of best-shot analysis comes from the file's embedded preview in **under 100 ms**; the first print preview of a 16–31 MP RAW, which develops it, appears in **under 2.5 s**, and later changes re-render from the cached copy within [NFR-002](#nfr-002--preview-latency); exporting RAW keeps peak memory within [NFR-004](#nfr-004--memory-ceiling) by developing only a few at a time. Measured on nine CC0 samples before implementation ([RESULT.md §12](RESULT.md#12-camera-raw-v070)): preview 9–80 ms, develop 0.39–2.17 s, up to 1.29 GB for one 31 MP develop.

## 8. Out of scope

The following are out of scope (for now):

- Cloud sync, accounts, and any kind of server backend.
- General photo editing beyond the global Photo Style looks of [FR-039](#fr-039--photo-style-looks): local adjustments, masks, HSL/curve editors, retouching and RAW development controls ([ADR-0011](adr/0011-photo-style-looks.md)).
- Writing ratings, flags or edits back into source files. Sources stay read-only. XMP sidecar files next to photos are planned as an opt-in ([FR-058](#fr-058--lightroom-compatibility-through-xmp)).
- Mobile apps.
- Video.

## 9. Success metrics

- A 500-photo trip goes from folder to print-ready export in **under 5 minutes** of user time.
- Zero reports of photos with incorrect rotation or cut-off stamps in exported sets.
- All NFR budgets are green in CI benchmarks on every release.
