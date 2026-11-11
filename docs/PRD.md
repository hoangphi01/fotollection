# Product Requirements Document — Fotollection

> **Owner:** product · **Status:** Draft v0.1 · **Last reviewed:** 2026-11-11
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

**Fotollection is a fast, private, offline desktop app (macOS, Windows and Linux) that turns a folder of photos into a print-ready set.** It stamps date and location from EXIF, crops to standard EU print sizes, groups photos by trip, and keeps the original quality. Photos never leave the user's machine.

It has three modes ([FR-048](#fr-048--simple-collage-and-professional-modes)): **Simple**, for "keep, stamp, print" in a few key presses, **Collage**, two photos on one print to save cost, and **Professional**, with every control described in this document.

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
2. **Cull:** step through the photos with the keyboard, rate and flag them, and filter down to the set to print ([FR-022](#fr-022--culling-rate-flag-filter)). In Simple mode, one photo at a time, ↑ keeps it and ↓ drops it ([FR-048](#fr-048--simple-collage-and-professional-modes), [FR-046](#fr-046--swipe-culling)). Work is saved automatically to a project file ([FR-023](#fr-023--projects-autosave-and-undo)).
3. **Fix time and place:** correct a camera's clock offset ([FR-030](#fr-030--camera-clock-offset)), and fill in a missing date or place by hand ([FR-032](#fr-032--manual-date-and-place)) or from a phone or GPX track (Phase 2).
4. **Choose sizes and style:** print one photo per sheet, or two to a sheet in Collage mode ([FR-035](#fr-035--multi-photo-layouts)); pick a print size per photo or per group ([FR-025](#fr-025--mixed-print-sizes-per-order)) and a stamp style ([FR-033](#fr-033--retro-film-date-stamp-and-stamp-presets)). Adjust crop frames. A live preview updates as settings change.
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
- HEIC/HEIF (iPhone) is decoded by the operating system ([ADR-0017](adr/0017-heic-through-os-codecs.md)): ImageIO on macOS, and WIC on Windows when the free *HEIF Image Extensions* and the *HEVC Video Extensions* from the Microsoft Store are installed. Without them, HEIC files are listed as skipped with a message that says what to install. On Linux HEIC is not supported yet and is skipped with its own reason ([ADR-0034](adr/0034-linux-lean.md), [FR-068](#fr-068--heic-on-linux-through-the-systems-libheif)). EXIF (date, GPS, orientation) is read from the HEIC itself.
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
- Tiles show a skeleton until their thumbnail arrives, then fade in (FR-100: a skeleton says what comes, a spinner on it said it twice). Long tasks (reading photos, preparing thumbnails) show a progress bar with a count and percentage.

### FR-004 — Date stamp

**P0.** Render the capture date onto the photo.

- Configurable format (presets plus a custom pattern), colour, position (each of the 4 corners, with a margin) and **size: Small, Medium or Large** (2.8, 3.6 or 4.8 % of the print's short edge, so 2.8, 3.6 or 4.8 mm on a 10 cm print and more on a larger one; on a collage sheet each photo's stamp is half the sheet's, as two share it; Medium by default; v0.9.0), the same card, place and caption included, in every mode ([FR-048](#fr-048--simple-collage-and-professional-modes)) and per photo ([FR-044](#fr-044--per-photo-overrides)).
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
- Offers a "fit with border" (letterbox) alternative to cropping. For shared photos this is [FR-075](#fr-075--fit-a-shared-photo-whole-into-a-ratio); prints keep their print size.

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

- JPEG quality is configurable in the export dialog (60–100, kept per machine as `ui.exportQuality`, v0.13.0, FR-092), with a default of 90 (95 until 2026-11-06: a third larger for a difference under a visible step, RESULT §30) and 4:4:4 chroma subsampling for prints (share exports, [FR-053](#fr-053--export-for-social-media), are 4:2:0 and start at their preset's quality, 88 or 80).
- The embedded ICC profile is preserved.
- Photos that are crop-only with no stamp use a lossless DCT-domain crop when possible (planned; today every export is re-encoded).

### FR-009 — Export by date

**P0.** Write the export to a user-chosen folder using `YYYY-MM-DD/` subfolders and stable, sortable file names.

- **One folder or by day** (a user's feedback, owner 2026-11-04): a switch in the export card, **One folder** (the default, every file straight into the destination, as a kiosk USB stick wants) or **By day** (the `YYYY-MM-DD/` subfolders, `undated/` without a date). Remembered per machine (`ui.exportFolders`, settings v12). It applies to prints, share exports (inside `Share/`), collage sheets and postcards, and inside each subfolder of a separate export ([FR-056](#fr-056--folders-as-tabs)).

- Existing files are never overwritten silently. The user picks a rule in the export dialog (v0.13.0, kept per machine as `ui.exportExisting`): **add a number** (`_1`, `_2`, the default), **skip** the photo (the result counts them), or **replace** the file, with a red warning under the choice as its confirmation; a replaced file is swapped in whole, and two photos of one export never write over each other.
- **Put in a subfolder** (v0.13.0, Lightroom's *Export location*): a switch and a name, kept per machine (`ui.exportSubfolder`); the subfolder sits in the destination (and in a source folder's own subfolder, FR-056), before `Share/` and the day folders. The name is cleaned in Rust: no separators, no reserved Windows names, at most 100 characters.

### FR-010 — Metadata privacy on export

**P0.** Control which metadata is written into exported files.

- GPS is never written, whatever is chosen.
- **Options** (v0.13.0, the export dialog's *Metadata* card, kept per machine as `ui.exportMetadata`): **camera** (the allowlist below, the default), **date only** (the capture time and `Software`), or **nothing** (no EXIF block; the colour profile stays). Prints, share exports and collage sheets alike.
- No banner for GPS kept is needed: GPS is never kept.
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

- **Three decoders, tried in order** ([ADR-0029](adr/0029-raw-through-rawler-then-our-own-decoder.md), [ADR-0030](adr/0030-our-own-raw-decoder.md), [ADR-0040](adr/0040-neutral-raw-colour-and-gpu-develop.md)): Fotollection's own develop first for every Bayer sensor (a DNG read by our own reader, any other format unpacked by rawler and developed by us since v0.10.0, session 16), finished to the camera's embedded JPEG; then on macOS ImageIO (with the file's type identifier as a hint, as for HEIC, [ADR-0017](adr/0017-heic-through-os-codecs.md)) and rawler's own develop (LGPL-2.1, built in) for what it cannot read, X-Trans for now; on Windows rawler, with WIC and Microsoft's *Raw Image Extension* as the fallback. On macOS, Panasonic and Olympus RAW keep ImageIO first, because it corrects their lenses' distortion ([FR-085](#fr-085--lens-distortion-correction); owner, 2026-11-05). rawler is removed once our own reader covers every format, before any relicensing ([ADR-0008](adr/0008-license-gpl3-with-relicensing-option.md)).
- **Recognised by content**, never by extension: each format's own magic bytes, or for TIFF-based RAW the header plus IFD0 (`DNGVersion`, the maker in `Make`). CR3 is no longer mistaken for HEIC.
- **Two paths** ([NFR-017](#nfr-017--camera-raw-speed)): thumbnails and best-shot analysis use the **preview embedded in every RAW** (9–80 ms); the print preview, zoom and export use the **full develop** (0.4–2.2 s), one at a time, and its size is the only one used for crops.
- **EXIF** comes from the first 1 MB: RW2 and ORF are read as TIFF, RAF from its embedded JPEG, CR3 from its `CMT1`/`CMT2` boxes, Panasonic's ISO from its own tag. CR3's GPS is not read yet.
- **Nothing to install:** with rawler built in, RAW opens on Windows without the extension (to be confirmed on a Windows PC before the release); a file no decoder reads is reported as unsupported.

### FR-017 — Photo book layout preview

**P2.** Preview how the selection lays out on album pages.

### FR-018 — QR code on photo back

**P2.** Generate a printable QR sheet that links each print to its digital original.

### FR-019 — Print service upload

**P2.** Opt-in upload to print lab APIs. This is the **only** feature that may use the network, and it must be explicitly enabled by the user.

### FR-020 — Localization

**P1.** The UI is available in **English (the default) and Vietnamese**. German follows later.

- Every user-visible string in the UI goes through the i18n layer. CI fails on a hard-coded string in a component and on a key that one language lacks.
- The language is chosen in *Settings → General* (FR-086; the version popover of earlier versions is gone), takes effect immediately without a restart, and is remembered ([DATABASE.md §6](DATABASE.md#6-app-settings)).
- Numbers, dates and sizes are formatted for the language, for example `18.6` and `18,6`.
- Errors from the Rust core are shown in the UI language, translated from their stable `code` ([SECURITY.md §8](SECURITY.md#8-resilience-and-error-handling)). The English `message` is the fallback.
- **Out of scope:** text printed on photos (the info frame's "Shot on …" line and the date stamp) does not follow the UI language. It is print content and stays as configured.

### FR-021 — Manual updates

**P2.** New versions are published on GitHub Releases and the website; people download and install them when they choose. There is **no in-app updater**, and the app never checks on its own: a newer version is looked for only when the user clicks *Check for updates* ([FR-061](#fr-061--check-for-updates), [NFR-006](#nfr-006--offline-and-zero-telemetry), [ADR-0025](adr/0025-update-check-on-click.md)).

- **Installing over the old version replaces it** and keeps settings, profiles and projects: on macOS by dragging into Applications and choosing *Replace*; on Windows the `-setup.exe` uninstalls the old version first by default, and the MSI removes it through its pinned `UpgradeCode`. Use one Windows installer, preferably `-setup.exe`: mixing it with the MSI installs two copies. On Linux, a new AppImage replaces the old file, and a new `.deb` installs over the old one with `apt` ([ADR-0034](adr/0034-linux-lean.md)).

- The app shows its version (for example `v0.1.0`) at the foot of the settings panel. Clicking it lists every release with two lines from its Highlights, read from the CHANGELOG bundled at build time (no network).
- Installing over an older version keeps settings, camera profiles and projects; their files are migrated on first open ([DATABASE.md](DATABASE.md)).
- Release notes list user-visible changes and any file format that changed ([RELEASE.md](RELEASE.md)).

### FR-022 — Culling (rate, flag, filter)

**P0.** Narrow a large import down to the photos worth printing.

- Rate 0–5 stars and flag photos as *pick* or *reject*, using single-key shortcuts (`0`–`5`, `P`, `X`, `U` to unflag).
- A loupe view shows one photo large, with arrow keys moving to the next and previous photo ([NFR-012](#nfr-012--culling-navigation)).
- Filter by rating, flag, date, camera, "has GPS", and "resolution warning". Only the filtered set is exported.
  - The flag filter is *All*, *Picks*, *Unflagged* or *Rejects*, combined with a minimum star rating. *All* includes rejected photos, because the grid and the export always show the same set.
  - The file kind (owner, 2026-11-05): *All files*, *RAW only* or *No RAW*, as the grid's RAW badge says (`CullFilter.kind`).
  - The three sit behind one **Filters** button, outlined with how many are on, so the toolbar keeps one line at the narrowest window.
  - The shortcuts work in the grid (on the selected photo) and in the preview. The preview also has clickable stars and pick/reject buttons. Every change can be undone with Cmd/Ctrl+Z.
  - Rejected photos stay visible but dimmed in the grid. Nothing is ever deleted.
- Ratings and flags live in the project file ([FR-023](#fr-023--projects-autosave-and-undo)). They are never written into source files.

### FR-023 — Projects, autosave and undo

**P0.** All work is kept in a project file (`.fotollection`): source folders, ratings, flags, crop frames, per-photo overrides and settings.

- Autosave is continuous ([NFR-013](#nfr-013--project-safety)). Reopening the app restores the last project.
- Undo and redo cover every user edit (ratings, flags, crop, overrides, settings), with at least 200 steps.
- If source files have moved, the user can relink the folder. Missing photos are marked, not dropped; the session keeps a missing folder's edits too (v0.14.0).
- **A project keeps its own settings** (owner, 2026-11-09): its print size, stamp, look, postcards and frame are the document's, kept in the bundle and in the session; opening a project never changes this machine's settings (only fonts, logos and camera profiles it lacks are added, custom sizes joined). A new project starts from the machine's defaults, set by *Settings → General → New projects → Use this project's settings*.
- **A document, like `.docx`:** the `.fotollection` file is saved with **Save** (Cmd/Ctrl+S) and **Save As** (Shift+Cmd/Ctrl+S) and opened with **Open** (Cmd/Ctrl+O). It is self-contained ([ADR-0016](adr/0016-self-contained-project-bundle.md)): the edits, the print settings, the info frame, the camera profiles and copies of the fonts and logos in use. Opening it on another machine installs those fonts and logos and gives the same prints. Photos are referenced, not embedded.
- **Unsaved changes:** the window title and the sidebar show them (`Trip — Edited`). New and Open ask *Save changes?* first (Save / Don't save / Cancel). An untitled project with photos counts as unsaved, after a restart too, since the session is its only copy.
- **New asks for a name** (owner, 2026-11-06): a small dialog with the name, *Create* and *Cancel*, before the unsaved-changes question; an empty name keeps *Untitled*.
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

**P0.** On export, colours are converted from the embedded ICC profile (Display P3, Adobe RGB…) to **sRGB**, which is what most labs expect. **Target:** on by default, turned off per preset. **Today (v0.9.0):** only share exports convert; prints keep the original's profile (see the last point).

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

**P0 (v0.8.0).** Correct capture times per camera (make, model and serial number when available), resolved as in [ADR-0027](adr/0027-effective-time-and-place.md). Step-by-step use: [USER_GUIDE.md §11](USER_GUIDE.md#11-camera-clocks).

- Shift by an offset (±days/hours/minutes/seconds) or set a time zone.
- A helper sets the offset by matching one camera photo to a reference photo, such as a phone photo of the same moment.
- Corrected times drive sorting, date stamps, grouping and GPS matching. The original EXIF time is kept for reference.

### FR-031 — Borrow GPS from a phone or GPX track

**P1 (v0.8.0).** Photos without GPS (for example from a Lumix GX9) get a place from **another imported device's photo of the same scene** taken within a time window (default ±10 min; 5, 10 or 30 min in the dialog), following [ADR-0027](adr/0027-effective-time-and-place.md):

- **Same scene:** the visual signatures of the best-shot analysis, colour first, then layout over the zooms a phone and a camera lens differ by, against the 4 nearest photos in time from another camera body with GPS.
- **High confidence** (layout ≥ 0.75, colour ≥ 0.8, at most 5 min apart) is applied by itself, as one undoable step, after each best-shot analysis or a search. **Medium** (layout ≥ 0.6, colour ≥ 0.65) is shown side by side to accept or skip. The thresholds are provisional until real camera and phone pairs are scored ([RESULT.md §14](RESULT.md#14-places-and-time-v080-measured-first)).
- Borrowed places are marked *borrowed* in the preview and on the thumbnail, never replace a place set by hand or the photo's own GPS, and are never written as GPS into an exported file.
- This depends on correct clocks ([FR-030](#fr-030--camera-clock-offset)). Borrowing from a **GPX track** moved to [FR-066](#fr-066--borrow-places-from-a-gpx-track).

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

Put several photos on one print to save cost. **P1 (v0.9.0) for 2-up**, the Collage mode of [FR-048](#fr-048--simple-collage-and-professional-modes); P2 for 4-up and a photo-booth strip (5×15, 3–4 frames).

**2-up, inspired by half-frame film cameras** (one 36-exposure roll gave 72 frames):

- **Sheet and cells.** The sheet is the chosen print size; it is split in half across its long edge into two cells. A 10×15 sheet standing up holds two 10×7.5 cells side by side on the short edge; turned, it holds two 7.5×10 cells, the half-frame look. The sheet turns to suit the first photo of the pair, and the user can turn it back.
- **Photo area in a cell.** A fixed white margin stays round the photo, 5 mm a side by default. The app suggests the largest size at the chosen ratio that fits inside it, rounded down to the millimetre: a 2:3 photo in a 7.5×10 cell gets **6×9 cm**. The ratio is the photo's own by default, or the sheet's (2:3 on 10×15), 4:3 or 1:1. The user may type another size; with the ratio locked, typing one edge computes the other. Sizes are clamped to the cell, in Rust. The area is centred, and the rest of the cell is white.
- **Fill each half.** A switch: no white at all, each photo fills its whole cell (7.5 × 10 on a 10 × 15) and the two meet edge to edge, however the photo is shaped; the user drags and zooms its crop to choose what stays. Ratio, size, margin and the cut line do not apply (the halves meet at an edge, so the switch hides); the 3 mm border still can.
- **Crop.** Each photo is cropped to its area with a crop and a straightening of its own for sheets, kept apart from its print's ([FR-006](#fr-006--crop-to-print-ratio)) so cropping a cell never changes the photo's print; until they are set, the print's are used, and Reset centres the crop.
- **Rotate a photo in its cell (2026-11-01, owner's request).** **Rotate photo** (⇧R) turns the chosen photo a quarter clockwise in its cell, on the sheet only (`photos[].collage_rotation`, project v9): a landscape photo paired with a portrait one then fills its standing cell instead of sitting small in its middle. The sheet keeps its orientation, even when the first photo turns (the sheet is laid out from the photo before its turns in the cell, `sheet::layout_turned`); *Turn the sheet* is what turns the sheet. Its print and the other modes keep their orientation; the cell's crop starts again from the centre at its zoom. One undo step.
- **Auto turn and Revert (2026-11-02, owner's request).** Two buttons above the sheets, for prints cut in two: **Auto turn** lays every sheet shown landscape and turns each photo to stand in its half (`auto_turn_collage`, `sheet::landscape_turned`, `sheet::fitting_turns`), so no standing photo sits next to a lying one; **Revert** brings every sheet and photo back to its own orientation. A photo that turns gets its sheet crop centred again. One undo step each. The keys of the toolbar sit behind an **ⓘ** button (`KeysInfo`), in the gallery too.
- **Border.** The 3 mm border ([FR-067](#fr-067--3-mm-border-on-every-print)) cuts into the photo area, 3 mm a side: 6×9 shows 5.4×8.4 cm of photo. The sheet and cells keep their size.
- **Cut line.** On by default: a thin light-grey line between the cells, with short ticks at the sheet's edges, to cut the print in two after printing. Off, the line goes and the photos keep their places and the white between them: the print stays one half-frame sheet. A sheet with one photo has no line.
- **Pairing.** Photos are paired in order (the print basket's, or the view's): 1 + 2, 3 + 4…; an odd last photo leaves its other cell white and gets no cut line. The user reorders by dragging a photo onto another cell; the order is one for the whole library, so reordering in one folder never mixes with another's; photos placed by the user come first, and a photo added later joins at the end. A new order starts each sheet turned to suit its first photo.
- **Choosing the photos.** Select mode and the print basket work as in the gallery ([FR-050](#fr-050--print-basket)), by sheet (owner, 2026-11-02): a click ticks a whole sheet, both photos, with one tick circle and one frame round the sheet (no frame per photo, no dragging, no turn button); the bar counts sheets, and while sheets are ticked *Export* writes only them. Then the bar puts them in the basket, sets them aside or brings them back; in the sheet preview ↑ and ↓ do it for the chosen photo. The order and the turned sheets are saved in the project (schema v9); each change is one undo step.
- **Preview.** A sheet opens like a photo from the gallery (double-click, Enter or Space) in the photo preview's frame, with no photo chosen, so it shows exactly as it prints: the sheet large with zoom, ← → through the sheets, a click (or 1, 2) to choose a photo and again (or Esc) to let go, each photo cropped there for its place ([FR-006](#fr-006--crop-to-print-ratio)), the sheet turned, and **Export this sheet**, as [FR-055](#fr-055--export-from-the-preview) does for a photo.
- **Sheet size.** Any built-in or custom size; Original is not offered, since a sheet needs a fixed size (a sheet kept on Original prints at 10 × 15, and the panel says so).
- **Advice.** The panel shows the suggested size, the photo left inside the border and the print quality ([FR-026](#fr-026--resolution-warning)), for example "6 × 9 cm · 2:3 · with border 5.4 × 8.4 cm".
- **Export.** One file per sheet, named `<first>+<second>.jpg` in the first photo's day folder when exported by day, never overwriting ([FR-009](#fr-009--export-by-date)); EXIF from the first photo, without GPS ([FR-010](#fr-010--metadata-privacy-on-export)). Two photos in one colour space keep its profile; two in different spaces are both converted to sRGB ([FR-027](#fr-027--srgb-conversion-for-labs)). Half as many sheets are exported at once as photos, as each holds two decoded photos ([NFR-004](#nfr-004--memory-ceiling)). The sheet's pixel density is the lower of the two cells' own, so no photo is enlarged ([ADR-0032](adr/0032-collage-sheets-from-two-cell-plans.md)).
- **Not in 2-up yet:** the info frame (its bar does not fit a 7.5 cm cell). The date stamp works in each cell.

### FR-036 — Custom captions

**P1.** Add a free-text caption under the date stamp: a line of up to 80 characters, in the stamp's colour and LCD face, set in capitals without accents and running across the photo (done, settings v5; variables below are still to come). The caption can use the variables `{date}`, `{time}`, `{place}`, `{country}`, `{trip}`, `{camera}` and `{filename}`. Captions use the same layout rules as stamps (inside the safe area or on the border).

**A line per photo (v0.9.0, 2026-11-01, owner's request).** In a collage sheet's preview, a switch **Same on both photos** (on by default) keeps one line; off, each photo gets its own field, and that line is kept per photo (`photos[].caption`, project v9) while the rest of its stamp still follows the settings. A photo's own stamp from the preview ([FR-044](#fr-044--per-photo-overrides)) carries its own line instead, and *Reset n photos with their own stamp* clears both.

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
- **Merged into [FR-079](#fr-079--colour-adjustments) (2026-11-05):** the moderate editor takes these parameters, *C200 Natural* becomes one of its built-in looks, and its scope boundary is the one written there ([ADR-0038](adr/0038-moderate-colour-editor-on-one-lut-engine.md)). Before that: `.cube` looks ship first ([FR-070](#fr-070--looks-from-lut-files)); these sliders follow on the same engine ([ADR-0036](adr/0036-colour-looks-cube-luts-first.md)).

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
  - **"Keep best"** picks the best photo and rejects the rest of the group, using [FR-022](#fr-022--culling-rate-flag-filter) flags, and sets the rest aside ([FR-050](#fr-050--print-basket); owner, 2026-11-07: "the shots not kept leave for Set aside, so the workspace stays tidy"); a kept shot set aside before comes back, one in the print basket stays there. It can be undone.
- **Models.** Models ship inside the app, are checksum-verified, and are never downloaded. Only permissive weights are allowed: YuNet (MIT) and MediaPipe FaceMesh V2 (Apache-2.0), both verified from their sources. InsightFace/SCRFD and pyiqa models are excluded because of their non-commercial licences. The runtime is pure-Rust `tract-onnx` ([ADR-0018](adr/0018-face-models-and-runtime.md)).
- **Cache.** Analysis results are derived data, so they live in the cache keyed by fingerprint, never in the project file ([DATABASE.md](DATABASE.md), rule N4).
- **Speed:** [NFR-014](#nfr-014--best-shot-analysis-speed).

### FR-042 — Custom fonts

**P1.** Import **OTF/TTF** fonts and choose them for the info-frame text ([FR-040](#fr-040--exif-info-frame)) and a postcard's typed text ([FR-073](#fr-073--postcard-sheets)). The text inside the photo always uses the bundled LCD face ([FR-004](#fr-004--date-stamp)); its font choice was removed to keep the stamp simple (settings v5).

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

**P2 (future); the full-screen ↑ / ↓ part is P1 in v0.9.0 as the Simple mode of [FR-048](#fr-048--simple-collage-and-professional-modes).** A full-screen, keyboard-first culling mode, inspired by swipe apps (right = like, left = pass) and Lightroom's loupe. It builds the **print list** at the speed of a key press.

- One photo fills the screen. **↑ adds it to the print list, ↓ leaves it out** (already in the preview since v0.5.0, as the print basket and set aside of [FR-050](#fr-050--print-basket)), and both move on to the next photo at once. **← / →** step back and forward without deciding. **Space** shows the photo without overlays (later; not in v0.9.0's Simple mode), and **Cmd/Ctrl+Z** undoes the last decision.
- Leaving a photo out only removes it from the list. The file is never deleted or moved.
- The print list is the set that gets exported. It builds on pick/reject ([FR-022](#fr-022--culling-rate-flag-filter)): ↑ is *pick*, ↓ is *reject*, and an undecided photo stays *unflagged*. Stars keep working inside the mode (`0`–`5`).
- A progress bar shows decided / total and the size of the print list. The mode can resume where it stopped, and best-shot suggestions ([FR-041](#fr-041--best-shot-suggestions)) can pre-sort a burst so the best frame comes first.
- The next photos are prefetched so every decision shows a sharp image in under 100 ms ([NFR-012](#nfr-012--culling-navigation)).
- **Workflow change:** the main flow becomes *import → swipe → refine (crop, stamp, frame) on the print list only → export*. The grid filter, the preview and the export default to the print list once it is not empty. The vision (§2) and the UI flow are updated when this lands.

### FR-047 — Film looks

**P2 (future; on the FR-079 engine as built-in slider recipes, 2026-11-05, [ADR-0038](adr/0038-moderate-colour-editor-on-one-lut-engine.md)).** A set of **film-simulation looks** comparable to Fujifilm's in-camera film simulations, applied with one click to a photo or the whole batch. They use the same engine as Photo Style ([FR-039](#fr-039--photo-style-looks), [ADR-0011](adr/0011-photo-style-looks.md)): a 3D LUT, a tone curve, a white-balance shift and grain.

- **The looks, by intent:** a neutral slide (like Provia), a vivid landscape slide (like Velvia), a soft portrait slide (like Astia), a muted documentary chrome (like Classic Chrome), a hard-shadow negative with shifted colours (like Classic Negative), a warm amber negative (like Nostalgic Neg), a flat cinema look (like Eterna), a natural modern negative (like Reala Ace), a soft studio negative (like Pro Neg Std), and fine-grain black-and-white with filter options (like Acros). Each has *strength* 0–100 % and the usual Photo Style parameters on top.
- **Names and data are ours.** Fujifilm's film-simulation names are trademarks, so the UI uses descriptive names of its own (for example *Chrome Muted*, *Negative Classic*). Fujifilm appears in the docs only to describe the inspiration. The LUTs and curves are built from the project's own reference shots and published recipes, and never copied from Fujifilm software or from third-party LUT packs without a compatible licence.
- **Recipes:** a look plus Photo Style parameters saves as a preset ([FR-038](#fr-038--print-presets)), so recipes like the owner's *C200 Natural* can be shared. Looks are stored by id in the project, so a bundle opens with the same look on another machine ([ADR-0016](adr/0016-self-contained-project-bundle.md)).
- **Order and speed:** the look is applied before the date stamp and the info frame, so they are never tinted. Applying a LUT to the 1600 px preview must keep the preview within budget ([NFR-002](#nfr-002--preview-latency)), and this is benchmarked before release.

### FR-048 — Simple, Collage and Professional modes

**P1 (v0.9.0).** Three modes, switchable at any time from the top of the sidebar and remembered per machine (`ui.mode`, settings v10). They share one library, one project file and one rendering engine, so switching never loses work: a photo kept, cropped or stamped in one mode is the same in the others. (Planned as "Normal and Professional" until 2026-10-31; the owner split Normal into Simple and added Collage.)

- **Professional** is everything built so far: the inspector, the crop editor, the info frame and camera profiles, fonts, custom sizes, filters, the `.fotollection` document, and full-quality export (q95, 4:4:4, full resolution).
- **Collage** puts two photos on one print, like a half-frame film camera put two frames on one negative ([FR-035](#fr-035--multi-photo-layouts)).
- **Simple** is the "instant" mode: the shortest path from a folder to prints.
  - **One screen, three steps:** open a folder → one photo at a time, **↑ keep** (print basket) or **↓ drop** (set aside), and the next photo comes at once; two large buttons do the same with the mouse ([FR-050](#fr-050--print-basket), the full-screen part of [FR-046](#fr-046--swipe-culling)) → **Export**.
  - **Few choices, good defaults:** prints or a size for social media ([FR-053](#fr-053--export-for-social-media)), print size (10×15 by default), the 3 mm border ([FR-067](#fr-067--3-mm-border-on-every-print)), the date stamp with the same card as Pro (format, corner, colour, size, place and caption; changed 2026-11-01 at the owner's request, it had the size only), and the destination. Each photo keeps the crop, frame and stamp it has from Pro (centred when it has none); no inspector, frame editor, fonts or crop editor on screen. Pro's star and flag filters do not apply: the sidebar's folders and basket do.
  - The culling keys work as in the preview: 0–5 stars, P / X / U flags, [ ] quarter turns ([FR-022](#fr-022--culling-rate-flag-filter)).
  - It exports the print basket at Professional quality.
- **Later:** a faster Simple export sized for the print, at 300 DPI on the chosen size (about 1800 × 1200 px for 10×15) and q90, with decoding at reduced scale ([NFR-015](#nfr-015--simple-mode-speed)), measured before it is built; the grid on embedded EXIF thumbnails only.

### FR-049 — Trip journal

**P2 (future, not started).** An offline **journal of a trip**, written from the album itself: a one-page snapshot and report of what the photos say, like a travel diary the app drafts for you. For example: *"6 days, 14–19 September 2025 · Debrecen, then Budapest · 412 photos, 87 kept · Canon PowerShot and iPhone 15 · focal lengths 5–25 mm (28–140 mm equivalent), mostly wide · busiest day: 17 Sep, 128 photos · golden-hour sunsets on 3 evenings".*

- **Everything is derived from data the app already has:** capture times (days, first and last photo, busiest day and hour), places from GPS, borrowed or manual places ([FR-005](#fr-005--location-stamp-offline), [FR-031](#fr-031--borrow-gps-from-a-phone-or-gpx-track), [FR-032](#fr-032--manual-date-and-place)), cameras and lenses, focal-length, aperture, shutter and ISO ranges from EXIF, counts of picks, rejects and stars ([FR-022](#fr-022--culling-rate-flag-filter)), and best-shot groups ([FR-041](#fr-041--best-shot-suggestions)).
- **Offline and private:** the text is generated from templates in the interface language (English, Vietnamese), with no language model in the cloud and no network. The places come from the bundled GeoNames data.
- **Structure:** a summary block, then one section per day: date, places in visiting order, photo count, a strip of that day's best shots, and notable facts (the longest burst, the widest and longest focal length used). It builds on trip grouping ([FR-012](#fr-012--trip-auto-grouping)) when that exists; until then a trip is the whole album or a date range.
- **Output:** shown in the app, and exportable as a self-contained HTML or PDF page, and optionally as one extra print (a journal card) in the chosen print size. Every sentence can be edited before export.
- **Statistics shown are measured, never guessed.** Where the app is unsure (for example a place inferred only from time), it says so ("probably Debrecen").
- **Journal entries from editing (idea from the instax app).** Every editing session is also a journal entry: when the user works on a photo (crop, straighten, date stamp, the caption line, stars, pick), the app records a short entry with the time, the photo and what was done, and the caption the user typed becomes that entry's text. The trip journal gathers these entries per day next to the facts derived from the album, so the diary is written while editing, not afterwards. This needs an edit history saved in the project (a later project schema).

### FR-050 — Print basket

**P1 (in progress, v0.5.0).** Two **holding areas** for the photos of a trip, separate from culling: the **print basket**, the photos that will really be printed, and **set aside**, the photos taken out of the gallery for now. A typical batch: 100 photos are edited together, filters keep 60, only 5–10 go to the lab. Step-by-step use: [USER_GUIDE.md §4](USER_GUIDE.md#4-choosing-the-photos-to-print).

- **One place per photo:** the gallery, the print basket or set aside, never two. Independent of stars and pick/reject ([FR-022](#fr-022--culling-rate-flag-filter)): a pick does not have to be printed.
- **Set aside is never deletion:** the photo leaves *Workspace* and the folders, its file is never touched, and it can be brought back at any time.
- **The sidebar lists both areas** under *Workspace*, with their counts. Opening one shows only its photos; *Export* then follows the view, so the basket exports exactly the basket and photos set aside are never exported from the gallery.
- **Ticked photos export first (v0.8.3, user feedback 2026-11-01):** while photos are ticked in select mode, *Export* writes only them and says so (*Export 3 selected photos*), with the size estimate for them; with no tick it follows the view again. Ticks in other folder tabs count too (FR-056). Collage sheets keep following the view.
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
- **Size and speed:** the model may not fit the install budget ([NFR-008](#nfr-008--install-size), 150 MB since 2026-11-05). It ships as an **optional model pack** that the user installs on purpose; analysis stays within [NFR-014](#nfr-014--best-shot-analysis-speed)-like budgets, measured first.
- **Shown as a suggestion only,** with its confidence, and never applied silently. It is scored on labelled photos like the best-shot suggestions before it ships.

### FR-052 — Appearance: theme, title bar and hints

**P1.** The app looks like one piece on both platforms.

- **Theme:** System, Light or Dark, chosen with a three-way switch at the foot of the library column and stored per machine (`settings.json` `ui.theme`, schema v6). The native window (title bar, scroll bars) follows the choice. The colours come from the logo (STYLE_RULE §4).
- **Windows title bar:** on Windows the system's white title strip is replaced by one drawn in the app's colours, like VS Code's: the app mark and the project name, a drag area (double-click maximizes) and minimize, maximize/restore and close buttons. It stays usable above open dialogs. **macOS** (owner, 2026-11-06, "seamless like VS Code"): the window's title bar is transparent (`titleBarStyle: Overlay`), and a 28 px strip in the app's colours lies under the system's traffic lights, with the project name in the middle and a drag area, so the bar and the app are one surface. **Linux** keeps the system's title bar, which follows the theme: a window without decorations loses its resize edges there and cannot be checked without a Linux machine.
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
  - **Whole photo in a frame** ([FR-075](#fr-075--fit-a-shared-photo-whole-into-a-ratio)): the photo whole in a frame of a chosen ratio, the rest filled.
- **Pipeline:** the photo's edits (crop, straightening, date stamp, info frame) as for prints, then conversion to **sRGB** ([FR-027](#fr-027--srgb-conversion-for-labs)), a light unsharp mask (0.2) after a downscale, JPEG with 4:2:0 chroma and optimized Huffman tables, at quality 88, or 80 for *Original*.
- **Metadata:** camera, lens, exposure and capture time, and `Software = "Fotollection <version>"`, which phone galleries show as the editing app (as they do for Snapseed). **GPS is never written** ([FR-010](#fr-010--metadata-privacy-on-export)).
- **Where:** `<destination>/Share/`, apart from prints, in `<YYYY-MM-DD>/` subfolders when exported by day ([FR-009](#fr-009--export-by-date)).
- **How much lighter, before and after:** the export card estimates the size before exporting ("Estimated: 17.0 MB → ≈ 1.4 MB, 92 % lighter"), from up to four photos encoded in memory with the current settings and updated as they change, exact when every photo was sampled; when the export finishes, it reports the real figure. Print files, which keep every detail, can be larger, and the card says so.
- **Measured, not guessed** ([RESULT.md §10](RESULT.md#10-photos-for-social-media)): against the original resized with Lanczos, SSIM ≈ 0.98 for every preset and mean ΔE2000 0.96 for *Original* (at about half the file); after a simulated Facebook recompression, closer to the photo than the usual way and 99.6 % of the colour of Display P3 photos kept (86 % the usual way). Resized photos stay at ΔE2000 1.3–1.6 because of the 4:2:0 chroma every platform recompresses to.

### FR-054 — Open photos with Fotollection

**P2 (v0.7.0, in progress).** Fotollection can replace a classic photo viewer (IrfanView, Photos): choosing it in the system's "Open with" menu for a JPEG, PNG, HEIC or camera RAW shows that photo straight away in the preview, and ← → browse the other photos of its folder ([ADR-0024](adr/0024-optional-photo-viewer.md)).

- **Never the default viewer.** macOS lists Fotollection under "Open With" (rank `Alternate`); Windows lists it under each type's `OpenWithProgids`, written by the installer and removed on uninstall; on Linux the `.deb` lists it for JPEG, PNG and the RAW types in its desktop entry (no HEIC there yet). Making it the default is the user's choice in the system's settings.
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

- **Workspace** (named *Workspace* until v0.13.1; owner, 2026-11-07) sits above the folders, with its count: the grid shows every photo of every folder still in play, all but those set aside. After *Keep best* it holds the best shot of each group of every folder, which is where they are edited; the folder tabs are views of it, never copies.
- **Clicking a folder shows only that folder**: the grid, the best-shot groups and the export all work on it, like switching tabs. The folder counts never depend on the star or flag filter.
- **Several folders:** the select button above the list (or Ctrl/⌘-click) turns the folders into tick boxes; the grid and the export then cover the ticked folders.
- **Together or separately:** when the export covers two folders or more, the export card asks whether they go **together** (the usual folder, or date folders) or **separately** (one subfolder per source folder, named after it; two folders with the same name get " (2)").
- **Ticks follow the tabs:** in select mode ([FR-050](#fr-050--print-basket)), photos ticked in one folder stay ticked when another folder or *Workspace* is shown, so the best of several folders goes to the basket in one move. The selection bar counts every ticked photo and says how many are in other folders. Opening the basket or the set-aside view, or leaving select mode, clears the ticks.
- The choice of folders is view state, not saved in the project. Removing a folder shows all photos again, since folder ids are renumbered.

### FR-057 — Zoom in the preview

**P1.** The preview zooms like a photo viewer, to check focus and detail.

- A **two-finger scroll up or down** (fingers up zoom in, whatever the system's scroll direction; the mouse wheel's forward notch too), a **trackpad pinch** (sent as Ctrl + wheel by WebKit and WebView2) and **Ctrl/⌘ + / − / 0** zoom about the pointer, from fit up to 800 %; a **double-click** toggles fit and 200 %.
- Zoomed in, a **drag** or a sideways two-finger scroll moves the photo.
- **Zoom buttons** at the bottom right of the photo (− · level · + · fit), so zoom can be found without knowing the shortcuts; each tooltip names its shortcut and the pinch.
- Another photo, or the crop editor, starts again at fit; ← → still browse.
- **A safe preview, full detail on demand.** The preview is never larger than 1,600 px, rendered from a 2,048 px copy, whatever the camera: a weak machine never decodes a 32 MP photo just to show it. Zoomed past the preview's own pixels, once the view has been still for 150 ms, the UI asks Rust for **only the visible part** (plus a 10 % margin) at the screen's pixel count, at most 4,096 px wide. Rust decodes the original then, keeps at most about 200 MB of originals (two 32 MP photos; the one on screen always stays), renders it exactly as export does and sends that part. The zoomed preview stays on screen meanwhile, with a spinner beside the zoom level. Measured cost, REF-MAC: decode 46 ms (16 MP) to 84 ms (32 MP) the first time, then 17 ms per region ([RESULT.md §11](RESULT.md#11-preview-and-zoom-cost-v060)).

### FR-058 — Lightroom compatibility through XMP

**P2 (v0.8.0: reading stars and rejects).** Work culled in Lightroom, Bridge, Capture One or darktable comes into Fotollection, and Fotollection's work can go back out, through **XMP** sidecar files: XMP is XML with standard fields those apps share, so it is used instead of a format of our own.

- **Read on import (always, read-only):** an `IMG_0416.xmp` or `IMG_0416.RW2.xmp` next to a photo gives its stars and the reject flag (`xmp:Rating`, −1 for a reject; v0.8.0), for photos not yet edited in Fotollection: a project restored afterwards wins. The crop and the straightening (`crs:Crop*`, `crs:CropAngle`) follow once their coordinates and the angle's sign are checked against a real Lightroom sidecar; the `crs:` fields have no public specification.
- **Write (a setting, off by default):** the same standard fields, plus Fotollection's own work (stamp, frame, print size, pick flag) in a `fotollection:` namespace that other apps ignore. Only `.xmp` files are created or updated, atomically, never a photo; an existing sidecar keeps every field Fotollection does not own (edited at the XML event level with `quick-xml`, MIT).
- **Known limit, to confirm with Lightroom itself:** Lightroom Classic reads sidecars only for RAW files; for JPEG and PNG it reads XMP embedded in the file, which Fotollection does not modify. Lightroom's pick flag lives only in its catalog.
- Writing next to photos changes the "nothing is written into source folders" rule, so it needs an ADR and updates to [SECURITY.md](SECURITY.md) and §9.

### FR-059 — Quick-preview

**P1 (in progress, v0.5.0).** In the gallery's select mode ([FR-050](#fr-050--print-basket)), **holding Space** over a photo shows it large, as it will be exported, and releasing Space closes it, like Quick Look on macOS. Step-by-step use: [USER_GUIDE.md §4.3](USER_GUIDE.md#43-quick-preview-look-without-opening-select-mode-only).

- It shows the photo under the pointer, or the last one ticked; the thumbnail appears at once and the Rust render with the gallery's print size, stamp and frame replaces it (about 17 ms warm, 120 ms the first time).
- **Nothing can be edited**: it is a look, not the preview with its inspector ([FR-007](#fr-007--live-preview)). While Space is held, ← → show the neighbours and Enter ticks or unticks the photo shown.
- Outside select mode, Space keeps opening the normal preview.

### FR-060 — Start-up screen

**P1 (v0.6.0).** Opening the app shows the **logo and a loading bar** on the theme's background, never a half-built window: the screen is painted by the page itself before any script runs, and it fades out onto the finished workspace once the last session is back ([FR-023](#fr-023--projects-autosave-and-undo)). When the session reopens folders, the bar shows the import's progress, like the import bar; otherwise it slides. It never stays more than 15 s. Later, it may ask which mode to start in the first time ([FR-048](#fr-048--simple-collage-and-professional-modes)); not planned for v0.9.0.

### FR-061 — Check for updates

**P2 (v0.8.0).** The version footer's popover has a **Check for updates** button ([ADR-0025](adr/0025-update-check-on-click.md)).

- **Only on click:** one HTTPS request to GitHub's latest-release API; the app never checks by itself, and a note under the button says so.
- **Up to date:** "You have the latest version". **Newer:** "Fotollection 0.8.0 is available" and *Download the installer*, which opens the installer for this system in the browser; installing it replaces the app and keeps settings and projects ([FR-021](#fr-021--manual-updates)).
- **Offline or an odd answer:** a plain message, never a crash. Pre-releases are never offered. Links outside the project's releases are refused, and the link is kept in Rust, so the webview never opens a URL itself.

### FR-062 — Live view from a camera

**v0.8.0, Windows first.** The capture screen of [FR-063](#fr-063--tethered-capture-over-usb) shows what a camera plugged in by cable sees, live, with the print ratio drawn over it (the crop's ratios), to frame the shot ([ADR-0026](adr/0026-cameras-through-the-os-usb-stack.md), [ADR-0028](adr/0028-camera-sessions-saved-on-demand.md), [cameras/README.md](cameras/README.md)).

- **Sources:**
  - Now: a camera through Fotollection's own PTP driver for its maker (Sony first).
  - Later: a standard webcam (UVC) or an HDMI capture card through the webview.
  - Wi-Fi is not a source.
- Frames are only shown: never stored, never sent ([NFR-006](#nfr-006--offline-and-zero-telemetry)).
- Unplugging stops the view with a plain message, never a hang.

### FR-063 — Tethered capture over USB

**v0.8.0, Windows first.** **Connect camera**, next to *Open folder*, looks for a camera plugged in by cable. When it finds one, a **capture screen** opens.

**The capture screen.**

- It has the preview's frame:
  - the live view on the stage, with the print ratio drawn over it ([FR-062](#fr-062--live-view-from-a-camera));
  - a column of the shots just taken, newest first;
  - a panel where the preview has its inspector.
- The panel shows the camera, ISO and shutter speed (− / +), the aperture (read only), the file format (JPEG or RAW), and a **Capture** button.
- Space captures and Esc closes. Clicking a shot in the column opens it in the preview.

**Where shots go.**

- Each full-resolution file (JPEG or RAW) goes to a temporary session folder that shows as a new tab ([ADR-0028](adr/0028-camera-sessions-saved-on-demand.md)).
- Closing the capture screen shows that tab's photos. A notice offers **Save all**, which asks for the destination only then, moves the files without overwriting any, and keeps every edit.
- Closing a session tab that still has unsaved shots asks first: *Save all*, *Delete photos* or *Cancel*. **Quitting the app** with unsaved shots asks the same. A session left over from a previous run (a crash, a forced quit) comes back on the next start.

**Requirements.**

- No driver to install: Fotollection talks PTP through the system's own USB stack (ImageCaptureCore on macOS, WPD on Windows), with one module per camera maker written from our own protocol notes ([ADR-0026](adr/0026-cameras-through-the-os-usb-stack.md)).
- First maker: Sony (the a6400 at hand), then Canon and Nikon once a body can be borrowed. Standard PTP for the rest. The Panasonic GX9 cannot be tethered over USB at all (Step 0). Which body does what is in [cameras/README.md](cameras/README.md).
- A burst of shots loses none and stays within [NFR-004](#nfr-004--memory-ceiling): presses are queued, and each shot is written to disk before the next one is taken.
- A press that does not shoot (no focus, camera busy) says so plainly.
- v0.8.0 supports the Sony a6400 on Windows. On macOS the button is shown disabled with a hint, until the camera has a thread of its own there.

### FR-064 — Watch a folder for new photos

**P2 (planned).** *Watch this folder* on a folder tab: a photo written into it (by a maker's wired tether app, for a body without a Fotollection driver yet) appears and opens at once, once the file is completely written. Also used by FR-063 for its own session folder.

### FR-065 — Border in Crop to fill

**P1 (v0.8.0).** In the info frame's **Crop to fill** mode, an **Add border** switch leaves a 3 mm border in the frame colour (white or black) on the top and both sides, inside the print size; the info bar stays below. The print keeps the exact size of the shop's kiosk (for example CEWE at dm and Rossmann) and still looks like an instant photo, unlike Fit whole photo, which makes the print larger. Extended to every print and every frame style by [FR-067](#fr-067--3-mm-border-on-every-print) (v0.9.0).

- The photo is cropped to the smaller area, so the crop editor, the print-quality advice and the export follow it.
- Off by default; stored in `settings.json` (v9). Fit whole photo already has its own border, so the switch shows only in Crop to fill.

### FR-066 — Borrow places from a GPX track

**P2 (planned).** Photos without GPS get a place by interpolating their effective time along a GPX track the user opens (from a phone app or a watch), in the time zone set for the camera ([FR-030](#fr-030--camera-clock-offset)). Marked *borrowed* like [FR-031](#fr-031--borrow-gps-from-a-phone-or-gpx-track), reviewable and undoable. Split out of FR-031 for v0.8.0.

### FR-067 — 3 mm border on every print

**P1 (v0.9.0).** A **3 mm border** switch in the print-size card, for every mode and every frame style. It extends [FR-065](#fr-065--border-in-crop-to-fill), which offered the border only in the info frame's Crop to fill. The border cuts into the photo, 3 mm a side; the print keeps the exact size of the shop's kiosk.

- **Colour (2026-11-01, owner's request):** **White** or **Black** under the switch (`render.borderColor`). On a collage sheet it is also the colour of the paper round the photos, so the choice shows there whenever the sheet has paper (not with *Fill each half* and no border); the cut line turns dark grey on black. An info frame keeps its own colour.
- **No info frame:** a white or black border round the photo; the photo is cropped to the print size less 6 mm on each edge. **Original** keeps every pixel: the border goes round the whole photo and the print keeps its 10 cm short edge, so its long edge grows by the border.
- **Not on shared photos:** an export for social media ([FR-053](#fr-053--export-for-social-media)) is not a print and has no border.
- **Info frame, Crop to fill:** as FR-065, in the frame colour on the top and sides, with the bar below.
- **Info frame, Fit whole photo:** the print no longer grows. The whole photo is fitted inside the print size, less the border and the bar, centred, and the rest is the frame colour. With the switch off, Fit whole photo keeps its own 5 mm border and the larger print.
- **Collage:** 3 mm a side inside each photo area ([FR-035](#fr-035--multi-photo-layouts)).
- The crop editor, the print-quality advice (the sizes' badges and the inspector's DPI) and the export follow the smaller photo area; the date stamp stays inside the photo.
- Off by default; `render.border` in `settings.json` v10, which takes over FR-065's `frame.border` (a v9 file with the border on keeps it on).

### FR-068 — HEIC on Linux through the system's libheif

**P2 (planned, after v0.9.0).** On Linux, HEIC photos open through the `libheif` that the user's distribution installs (`libheif1` on Ubuntu and Debian, `libheif` on Fedora and Arch), loaded at run time; Fotollection never bundles an HEVC decoder, because HEVC is covered by patent pools ([ADR-0034](adr/0034-linux-lean.md)). Without it, HEIC is skipped with the command to install it. Until then, HEIC on Linux is skipped as "not supported on Linux yet".

### FR-069 — Report a problem

**P1 (v0.8.3, owner's request 2026-11-01).** A red **Report a problem or an idea** button, in *Settings → Report a problem* since v0.12.0 (FR-086; first in the versions popover, now gone), and a **Report a problem** button in the website's footer. Both open the project's GitHub issue forms (bug or idea) in the browser. The app sends nothing itself: the address is fixed in Rust and opened with the system browser, as for *Check for updates* ([FR-061](#fr-061--check-for-updates)), so the webview never supplies a URL ([SECURITY.md](SECURITY.md)).

**Copy diagnostics (v0.9.1),** under the red button, copies the version, the system and the last error codes from the app's local error log, for the user to paste into the issue ([ERROR_CONTROL.md §3](ERROR_CONTROL.md#3-status-of-the-plan-v091)). The log never holds photos, GPS, paths or file names.

### FR-070 — Looks from LUT files

**P1 (import built for v0.14.0, 2026-11-07: `.cube` and HaldCLUT into a look of one's own, its strength, the V-Log warning; projects embed their LUTs (bundle v2, 2026-11-09); shipped in v0.14.0; the measurements in v0.14.1; v0.14.0 since the owner's re-ordering of 2026-11-05; owner's request 2026-11-02; moved before the develop core on 2026-11-05, as a LUT works on the photo's encoded values, [ADR-0038](adr/0038-moderate-colour-editor-on-one-lut-engine.md)).** Colour looks as in LUMIX Lab: choose a look, set its strength, print ([ADR-0036](adr/0036-colour-looks-cube-luts-first.md)). The sliders of [FR-079](#fr-079--colour-adjustments) bake into the same table, so an imported LUT and the sliders make one look.

- **Import** a `.cube` file (1D or 3D, 2–65 points a side, `DOMAIN_MIN`/`DOMAIN_MAX`) or a HaldCLUT PNG: from LUMIX Lab, the user's own, or downloaded. A LUMIX LUT tagged `#LUMIXPHOTOSTYLE VLOG` is written for V-Log footage and gets a warning: on a JPEG it comes out far too strong. It is copied into the app's `looks/` folder by content hash, so the original can move. Panasonic's `.vlt` is a candidate once its format is checked.
- **Built-in looks** made by our own code, under our own names (no copied LUT files, no trademarked names, as [FR-047](#fr-047--film-looks)), including one for *C200 Natural*.
- **Strength** 0–100 %, default 100.
- **Pro mode only** (owner, 2026-11-07, replacing the line below): importing a LUT or a preset, *Presets & LUTs*, *LUT strength* and the V-Log note are Pro mode's; Simple and Collage list the built-in and saved looks, and a photo given a LUT in Pro keeps it everywhere.
- ~~**The highlight of Simple mode** (owner, 2026-11-05): for the simple user, the workflow is *look at the photo, choose a colour, put it in the print basket*: the looks are a strip in Simple mode's stage, numbered, one tap or one number key (**0**–**9**, 0 the original; owner, 2026-11-07: "a number, then up or down, done") applies a look, whose **Amount** (100 % by default) and the **Faves** sliders sit in Simple mode's panel for the photo on the stage, and the photo goes to the basket with it ([FR-048](#fr-048--simple-collage-and-professional-modes)). Professional and Collage use the same looks, in their colour cards.~~
- **For every photo, or one photo as an override**, like the date stamp ([FR-044](#fr-044--per-photo-overrides)); a collage sheet uses each photo's own look.
- **Preview and export, in every mode**, and the thumbnails (since 2026-11-08, [ADR-0047](adr/0047-thumbnails-show-the-look.md)). The look comes after the crop and before the date stamp, frame and border, which are never tinted. `\` compares before and after.
- **Info frame:** the look's name can fill its `<photo style>` line ([FR-040](#fr-040--exif-info-frame)).
- **Projects** embed the LUTs they use, so they render the same on another machine ([ADR-0016](adr/0016-self-contained-project-bundle.md)).
- **Speed:** ≤ 5 ms on the 1600 px preview, ≤ 60 ms on a 24 MP export ([OPTIMIZATION.md §3](OPTIMIZATION.md#3-budgets)).

### FR-071 — Image facts

**P1 (done; owner's request 2026-11-02).** What each file is, shown at import and in the preview's inspector, so a user (and a bug report) knows what the app is decoding ([ADR-0037](adr/0037-our-own-develop-core.md)).

- **JPEG:** chroma subsampling (4:4:4, 4:2:2, 4:2:0, 4:4:0, 4:1:1, or grey), baseline or progressive, bit depth, the colour space named by its ICC profile (sRGB, Display P3, Adobe RGB, grey, CMYK) and the decoder that read it (ADR-0033).
- **RAW:** the sensor's colour filter, Bayer with its pattern (RGGB, BGGR…), X-Trans 6 × 6, four-colour, linear or monochrome, the bit depth, the maker and the path that developed it.
- **Where:** the file line of the preview's *This photo* card, for example "6000 × 4000 · 4:2:0 · Display P3" or "RAF · X-Trans · 14-bit"; a small badge in the grid, and a filter, for unusual files.
- **Read at import**, from the head of the file the EXIF pass reads anyway, and kept in RAM only: it is derived data, so it is never saved and the project schema does not change ([DATABASE.md](DATABASE.md) N4). A RAW other than DNG gets its sensor when the inspector first reads the whole file (only rawler knows it). HEIC gives its colour space and bit depth from its `colr` and `pixi` boxes. A photo that cannot be shown logs its facts beside the error ([ERROR_CONTROL.md](ERROR_CONTROL.md) §3).

### FR-072 — Our own develop core

**P1 (the Bayer develop, neutral colour and FR-088 shipped in v0.10.0, 2026-11-05; X-Trans, the GPU develop and the DNG kinds planned for v0.15.0; owner's request 2026-11-02).** From the sensor's raw samples to the screen and the print, every step is ours, written from the published methods and measured ([ADR-0037](adr/0037-our-own-develop-core.md)). Maker formats are still unpacked by our DNG reader or by rawler; the system codecs become a last fallback.

- **The sensor kind decides the path:** Bayer, X-Trans, four-colour, linear and monochrome each have their own branch; an unknown kind is a clean "not supported" error, never a panic or a guess.
- **Before demosaicing:** per-site black levels from the ActiveArea origin, white level, **green equilibration (Gr/Gb)** against maze and worm patterns, hot-pixel removal, the as-shot white balance.
- **Demosaicing:** for Bayer, a directional interpolation (RCD's direction decision, Hamilton–Adams estimates weighted by gradient; RCD's own ratio-corrected green scored worse on the charts, RESULT §25); Markesteijn for X-Trans (one pass by default, three passes as the quality option). Malvar-He-Cutler stays only as a test reference.
- **After demosaicing:** false-colour suppression (a median on colour differences) and highlight reconstruction.
- **Colour:** camera → XYZ → a **linear f32 working space**, the tone curve last, then sRGB or Display P3 with its matching profile. FR-070's looks apply in this space. **Neutral by default** (owner, 2026-11-05): the sensor's colour through its matrix, the lens shading and the exposure of the camera's JPEG, and a gentle curve with a soft toe; the camera's look, fitted to its embedded JPEG with whichever of two fits comes closer for that file (one curve and a matrix, or three curves), is the choice "Like the camera's JPEG", in a RAW photo's *RAW colour* card with *Apply to all RAW photos* ([ADR-0040](adr/0040-neutral-raw-colour-and-gpu-develop.md), RESULT §27). A project from before v0.10.0 opens its RAW photos with the camera's look, so its prints keep their colours.
- **On the GPU while editing** ([ADR-0040](adr/0040-neutral-raw-colour-and-gpu-develop.md)): the develop for the preview, zoom and thumbnails runs on `wgpu` where the machine has a capable GPU, with a CPU twin for every step; exports stay on the CPU. The machine decides, no switch.
- **Colour management everywhere:** previews, thumbnails, zoom and the crop source convert the photo's profile to sRGB for the screen (done 2026-11-03; the owner chose sRGB over a tagged Display P3, so every system shows the same, Linux included); PNG honours gAMA, cHRM and sRGB, dithers 16 → 8 bits and flattens alpha in linear light (done 2026-11-03); share exports resize in linear light (2026-11-03: measured about 50 ms more per 16 MP photo on screen, so the owner kept the screen's faster resize and chose linear light for share exports only, OPTIMIZATION §5).
- **DNG as it is met in the wild** ([docs/raw/dng.md](raw/dng.md)), in this order: phone gain maps (`OpcodeList2`), lossy LinearRaw (34892), LinearRaw with lossless JPEG or JPEG XL (ProRAW), the raw image digest as a damaged-file check, floating-point HDR merges, X-Trans DNGs with Markesteijn.
- **Measured, not guessed:** synthetic test charts (zone plate, Siemens star) score zipper, maze and false colour against the old paths; colour is checked against an independent reference develop, never against the embedded JPEG the look is fitted to. Budgets in [OPTIMIZATION.md §3](OPTIMIZATION.md#3-budgets).

### FR-073 — Postcard sheets

**P1 (v0.9.4; owner's request 2026-11-03).** In Collage mode, a **Postcard** switch in the bar above the sheets, next to Select, turns each sheet into a postcard; every photo then has its own, so 50 photos make 25 sheets or 50 postcards: one photo on one half of the 10 × 15 print, the back of a postcard on the other, so a kiosk print can be posted (Deutsche Post takes 14–23.5 × 9–12.5 cm cards of 150–500 g/m²; Česká pošta's limit of 14.8 cm makes it a letter there).

- **Layout,** after the owner's sketch: **landscape** puts the photo on the left half, **portrait** on the bottom half. The other half has a **dashed stamp box** of 20 × 25 mm (smaller than most stamps, so the stamp stuck on it hides the dashed line; 26 × 32 mm in v0.9.4, made smaller at the owner's request 2026-11-04) at the sheet's top right with the word STAMP; the band across the top stays free for more stamps or a long label printed at the post office (owner, 2026-11-03: two or three stamps, La Poste's machine labels). The writing lines sit together at the foot, no more than 8 mm apart: on a landscape card **3 or 4 for the message**, half a line's gap, then **3 for the address** (a long address gets **4** in the room of those 3, closer and in smaller type; the message keeps its spacing, owner 2026-11-04); on a portrait card, too low to stack them, the message in the left column and the address in the right, a rule between them. No box round the lines, as on a real card. The post's 15 mm coding strip is not kept clear (owner's choice).
- **The photo half** is the collage cell it was: ratio, margin, fill, crop, the photo's quarter turns, the date stamp and the 3 mm border all apply. Auto turn and turning the sheet do nothing, as the card is held the way the user chose.
- **Typed text:** each card's own **message** (at most 400 characters, 8 lines) and **address** (200 characters, 4 lines), checked in Rust; each part on its own lines, and what does not fit is cut, with a **red warning** (Lucide `CircleAlert`) on the card in the grid, on its preview and beside the part that is cut. Set in the bundled typewriter face **Xanh Mono** (OFL; every Latin script of Europe and Vietnamese, with its own italic), the bundled handwriting face **Patrick Hand** ([FR-076](#fr-076--a-handwriting-face-for-postcards)) or the user's font ([FR-042](#fr-042--custom-fonts)), any letter the face lacks (Greek, Cyrillic) taken from the bundled Inter, so an address copied from anywhere in Europe prints as typed; small, medium or large; in black, blue, sepia or red ink; **bold, italic and underline per part**. An optional **credit line** ("Photo © …", at most 80 characters) prints small at the foot of every card. Left empty, the lines stay blank for handwriting.
- **Panel:** the *Postcard* card holds the shape (held, lines) and the pen note; the *Text on the postcard* card holds the chosen card's message and address, each with B I U, then the font, size, ink and credit. The same cards in a postcard's preview.
- **Writing by hand:** the panel says to use a fine permanent marker (Sharpie Ultra Fine, edding 400, Stabilo OHPen S) on photo paper, to try it in the stamp box first and let it dry, and that ballpoint and gel pens smear or skip on the coating.
- Settings v11 (`render.collage.postcard`), project v10 (`photos[].postcard`), [DATABASE.md](DATABASE.md).

### FR-074 — Postcard backs: colour and patterns

**P2 (planned; owner's idea 2026-11-03).** The back of a postcard ([FR-073](#fr-073--postcard-sheets)) gets its own look: a **background colour** (cream, kraft, pale blue…) and the colour of the lines and stamp box with it, or a **pattern** (stripes along the edge like an airmail envelope, a light grid, a texture), chosen in the *Postcard* card and the same on every card. Text and lines stay readable on it: the ink follows the background as it follows black paper today. To be designed and measured before it is scheduled (a pattern drawn at 300 DPI must keep NFR-021).

### FR-075 — Fit a shared photo whole into a ratio

**P1 (owner's request 2026-11-04).** A portrait posted to a square feed such as Instagram's is cut at the top and bottom; to keep it whole, people make a white square in Photoshop and drag the photo onto it. In a **Share online** export ([FR-053](#fr-053--export-for-social-media)), a **Whole photo in a frame** switch under the share size does it for every photo:

- **Ratio:** Square 1:1, Portrait 4:5, 3:4 or 2:3, Landscape 3:2 or 16:9, Story 9:16, or a custom `W : H` (each side 1–100, at most 4:1 either way, clamped in Rust). The frame keeps the ratio as written, whatever the photo's orientation, so a feed stays even.
- **Fill:** White, Black or a colour of the user's (a colour picker and a `#rrggbb` code), pure sRGB, not the prints' paper white.
- **The photo is whole:** centred, nothing cut, never enlarged. The photo's own crop, straightening, date stamp and info frame are kept; the fill goes round all of it.
- **Size:** Instagram makes the frame 1080 px wide (1080 × 1080 at 1:1, 1080 × 1350 at 4:5); Facebook 2048 px and Custom the chosen edge on the frame's long side; Original the smallest frame that holds the photo at full size.
- **Prints never get it:** with *Print*, the print size sets the shape ([FR-006](#fr-006--crop-to-print-ratio)); collage sheets and postcards keep their own paper.
- The same switch in the Pro panel, Simple mode and the preview (under the share size); the preview shows the frame as it is exported. Stored in `settings.json` (`render.shareFill`, v12).

### FR-076 — A handwriting face for postcards

**P1 (owner's request 2026-11-04).** Many people want a postcard that looks written by hand without writing on glossy photo paper. Next to Xanh Mono, the **Font** list of the *Postcard text* card offers a bundled handwriting face, **Patrick Hand** (OFL 1.1, chosen by the owner from four OFL faces with Vietnamese, 2026-11-04):

- **Letters:** every Latin script of Europe and Vietnamese (ầ ẫ ặ ữ ỵ ơ ư đ), checked by a test; Greek and Cyrillic still come from Inter.
- **Strokes thick enough to print:** the owner's print test (2026-11-04) found the thinnest readable stroke is Xanh Mono's regular at the medium size, about 0.25 mm. A bundled postcard face must reach it at the **smallest** size (a test measures the stem). Patrick Hand's capitals stand lower than Xanh Mono's at the same size, so it is drawn larger, its capitals as tall as Xanh Mono's: a size reads the same in both faces, and its stroke at the small size is about 0.25 mm.
- **Styles:** bold thickens each stroke as in the other faces; italic is an oblique, as Patrick Hand has no italic.
- Stored as `render.collage.postcard.face` (`typewriter` | `handwriting`, settings v13), used when no user font is chosen.

### FR-077 — One address for many postcards

**P1 (owner's request 2026-11-04).** A set of postcards often goes to one person, and typing the same address on every card is the slow part. In the *Postcard text* card, in the panel and in a postcard's preview:

- **Use for them** (v0.13.2: a note with two buttons, so the words wrap in the panel; before, two link-styled lines that were cut), under the address once it is typed: it goes onto every other postcard shown that has no address, one undoable step. **Or for all N other postcards** replaces theirs too. With postcards ticked in select mode, only the ticked ones get it. The counts come from Rust (`postcard_text_gaps`) and follow every edit.
- **Recent addresses** on every card (since v0.13.2, owner 2026-11-07: before, they showed only while the card had none, so the list went after one pick), the one on the card marked; an address is remembered once its field is left, never the drafts the field saves while it is typed (owner, 2026-11-07: the list held one address five times), and the start of an address already kept gives way to the whole one: the last 8 addresses typed on this machine, newest first, each a click to put it on the card and × to forget it. Kept in `settings.json` (`recent_addresses`, settings v13), checked like a typed address when saved and loaded, never in a project file or a bundle, never in the error log.
- An address put on a card keeps the card's message.

### FR-078 — The photo at a glance in the preview

**P1 (owner's request 2026-11-04).** The preview's first card, **This photo**, sums the photo up before the tools: the camera and lens with the camera profile's names (as the info frame, [FR-040](#fr-040--exif-info-frame)), the exposure as the frame prints it (`20mm · F1.7 · 1/60s · ISO800`, `frame::shot_summary`), the place and the capture date (after a clock offset or a date set by hand), and the size in pixels, megapixels and megabytes. Lines with nothing to say are left out. A badge at its right says what kind of file it is, **RAW** (filled) or JPEG, HEIC, PNG (outlined), so a RAW is told apart at a glance (owner, 2026-11-05); the grid's tiles show the same RAW badge (`PhotoSummary.raw`: a camera RAW name, DNG, RW2, CR2, CR3, NEF, ARW, RAF, ORF, or a known sensor kind; a TIFF is not one). It also holds what the file is ([FR-071](#fr-071--image-facts)), so the preview has no separate Camera and Image cards (owner, 2026-11-04), and its title folds it away, kept while the app runs. The cards below it still change the date, the place and the camera's details.

### FR-079 — Colour adjustments

**P1 (Faves and Tools shipped in v0.9.9; the Looks card, its eight built-in looks, *Save as look* and Simple mode's strip built for v0.14.0 "Looks", 2026-11-07; owner's request 2026-11-05, [ADR-0038](adr/0038-moderate-colour-editor-on-one-lut-engine.md)).** A **moderate** colour editor, in the spirit of Snapseed's *Tune Image* and LUMIX Lab's looks, far short of Lightroom: global adjustments only, one card, the same result on screen and on paper.

- **Three cards, as Snapseed 3 splits Looks, Faves and Tools** (owner, 2026-11-05), each a lego card of its own, in the photo and sheet previews only (owner, 2026-11-05: colour is chosen while the photo is seen large, not in the gallery's panel), after *Print / export size*; the look shared by every photo is set with *Apply this look to all photos*; Simple mode shows only **Looks**:
  - **Looks:** the list of looks, the built-in ones, the user's own and the LUTs imported from LUMIX Lab or elsewhere ([FR-070](#fr-070--looks-from-lut-files)), each shown as the chosen photo through it, small; the **amount**; *Save as look*. *Import LUT…* and *Import presets…* sit in *Presets & LUTs*, Pro mode only (as built, 2026-11-07).
  - **Faves**, the everyday sliders, a fixed set: **tone** (brightness, contrast, highlights, shadows) and **colour** (warmth, blue–amber; tint, green–magenta; saturation; vibrance, which spares skin and colours already strong).
  - **Color mixer** (owner, 2026-11-08, as Lightroom's Color Mixer, Pro mode and collage sheets, not Simple): **Hue**, **Saturation**, **Luminance** or **All** as a row of buttons (owner, 2026-11-08, as Lightroom's HSL panel), and for the part picked a slider for each of the eight hues (red, orange, yellow, green, aqua, blue, purple, magenta), −100…100, its track drawn in the colours it moves between; a Lightroom preset's mixer lands here, adjustable (FR-089). One function of a pixel in `f32` (`colour_ops`), which the table calls today and the 32-bit path of ADR-0043 will call on every pixel, so it needs no rewrite.
  - **Tools**, the rest: **style** (fade, lifted blacks; a tint for the shadows and one for the highlights with their balance; black and white through a yellow, orange, red or green filter) and **effects** (v0.14.1 with FR-118: grain, its amount and size in millimetres on the print, so it looks the same at every print size; vignette around the cropped photo).
- **Three levels, as a car: drive first, learn the rest later** (owner, 2026-11-05). A newcomer needs only the first:
  1. **Looks**, always open, in every mode: tap a look, set the amount. Built-in looks are there from the first launch, named by feel ("Warm old film", "Clear", "Black and white, red filter").
  2. **Faves**, open with four sliders: brightness, contrast, warmth, saturation; *More* shows highlights, shadows, tint and vibrance.
  3. **Tools**, folded at first; its title opens it, and the app remembers which cards are open.
  - A folded card still says what it holds ("Tools · 2 changed"); each card's title has a reset, and a double-click resets one slider; a visible *Before / After* button above the photo, besides the `\` key, both turning it on and off (FR-096). Labels are plain words; what a slider does shows on hover, so the cards stay short.
- **Later, not in v0.14.0:** *Selective*, Snapseed's local control points (brightness, contrast, saturation around a point), as an FR of its own: it cannot bake into the table and needs its own measurement.
- **Sliders:** each from −100 to +100 (amounts and grain 0–100), 0 by default; a double-click resets one. The preview follows while dragging; one undo step per release. Press `\` to compare with the original, and again to come back (FR-096).
- **Looks:** a look is the sliders' values, an optional imported LUT ([FR-070](#fr-070--looks-from-lut-files)) with its strength, and an **amount** of 0–100 % that scales all of them together. Built-in looks are slider recipes made by the project under its own names (no third-party LUTs, no trademarked film or camera names), among them *C200 Natural* ([FR-039](#fr-039--photo-style-looks)) and film-inspired looks ([FR-047](#fr-047--film-looks)). *Save as look* keeps the current values under a name of the user's.
- **For every photo, or one photo as an override,** as the date stamp ([FR-044](#fr-044--per-photo-overrides)), with *Apply to all*; a collage sheet uses each photo's own look.
- **In Collage mode** (owner, 2026-11-05) the cards edit the photo picked on a sheet, in the sheet's preview: a click on a photo picks it, a click beside the sheets (or Esc) lets go. Since v0.14.0 (owner, 2026-11-07) the sheet's preview has two tabs as the photo preview: **Edit** (the photos on the sheet to pick from, then the colour cards) and **Print** (the cards of the Collage panel); with no photo picked (*Whole sheet*, key **0**) the looks are a strip under the sheet (every strip of looks sits under the photo, owner 2026-11-07: in the photo preview a second row over the filmstrip, folding, as a Pro user works mostly from presets), and the colour cards edit both photos, the first photo's look shown, and any change gives both the same look in one undo step, a note saying so when their looks differ. A sheet's two photos never share a slider by surprise: sharing is what *Whole sheet* says. Projects keep each photo's look and embed the LUTs they use ([ADR-0016](adr/0016-self-contained-project-bundle.md)).
- **Never tinted:** the date stamp, the info frame, the border and the fill are drawn after the look. Thumbnails show the photo's colour ([ADR-0047](adr/0047-thumbnails-show-the-look.md)). The info frame's `<photo style>` line can show the look's name ([FR-040](#fr-040--exif-info-frame)).
- **Exact when unused:** with every slider at 0 and no LUT, a print is the same, bit for bit, as without this feature.
- **Speed:** whatever is set, ≤ 5 ms on the 1600 px preview and ≤ 60 ms on a 24 MP export ([OPTIMIZATION.md §3](OPTIMIZATION.md#3-budgets)): every per-pixel adjustment is baked into one 33³ table, grain and vignette run in the same pass.
- **Out of scope:** masks and local adjustments other than the later *Selective* points, a full HSL matrix beyond the eight-hue Color Mixer (which is in scope, 2026-11-08), a free curve editor, retouching ([§8](#8-out-of-scope)).

### FR-080 — Card order of one's own

**P2 (later, not scheduled; owner's request 2026-11-05).** The user puts the right-hand panel's cards in an order of their own, by dragging a card's title (or with the keyboard), in the gallery's panel and in the previews alike: one order, so the same tools stay in the same order everywhere ([STYLE_RULE §4](STYLE_RULE.md#4-uiux)), only it is the user's. *Reset order* brings back the app's order. On the first launch the welcome screen ([FR-081](#fr-081--a-welcome-screen-on-the-first-launch)) asks which cards the user likes, and puts them first. The order is stored per machine with the UI settings (`ui.cardOrder`, [DATABASE.md §6](DATABASE.md#6-app-settings)); a card added by a later version takes its default place.

### FR-081 — A welcome screen on the first launch

**P2 (partly done; owner's request 2026-11-05).** The very first launch opens a short welcome before the library, made together with [FR-080](#fr-080--card-order-of-ones-own), and never shown again unless asked for. *The welcome itself shipped in v0.13.1 as [FR-109](#fr-109--a-welcome-screen-on-the-first-launch) (five pages, Show at every start); picking the cards below is still to do, with FR-080, as a page of that welcome.*

- **Pick what you use**, as Pinterest asks for one's tastes when an account is made: the panel's cards shown as tiles (Looks, Faves, Tools, Print / export size, Date stamp, Info frame, Places…), each with a small picture of what it does; the ones the user taps come first in the panel and the previews, the rest follow in the app's order (`ui.cardOrder`).
- **Never in the way:** *Skip* keeps the app's order; every choice can be changed later, by dragging the cards or with *Reset order* (FR-080). It does not delay the first window ([NFR-010](#nfr-010--cold-start)): the library loads behind it.
- What else the welcome holds (the language, the three modes) is decided when it is built.

### FR-082 — The app's brain in one file (`.fotosettings`)

**P2 (built for v0.14.0 with FR-116's Data section, 2026-11-09; owner's request 2026-11-05).** Recent postcard addresses travel only when *Include recent addresses* is ticked at export, off by default, with a line on privacy (owner, 2026-11-09). Everything Fotollection has learnt on one computer goes to another in one file, "the app's brain": *Export settings…* writes a `.fotosettings` file, *Import settings…* reads one ([EXTENSION.md §2.3](EXTENSION.md#23-settings-bundle-fotosettings)).

- **What it holds:** all of `settings.json`: the interface (language, theme, mode, export folders, the card order and the welcome's choices of [FR-080](#fr-080--card-order-of-ones-own) and [FR-081](#fr-081--a-welcome-screen-on-the-first-launch)), every print and stamp setting, custom print sizes, camera profiles and clocks, the postcard settings and the **recent postcard addresses**, the user's looks with the LUT files they use ([FR-079](#fr-079--colour-adjustments)), fonts and camera logos.
- **What it never holds:** photos, folder paths, projects or the caches. Albums travel as `.fotollection` projects ([FR-023](#fr-023--projects-autosave-and-undo)), which keep leaving the interface group and the addresses out, so opening someone's album never changes one's own app.
- **Import** shows what the file holds and asks before replacing the settings in place; a newer file than the app reads is refused with its `ERR-` code; every part goes through the validators of its own kind (fonts, logos, LUTs, sizes), as untrusted input.
- **Privacy:** the file holds the addresses typed on postcards; the dialog says so, and it is written only where the user chooses. Nothing leaves the machine otherwise.

### FR-083 — A personal photo calendar

**P2 (later, not scheduled; owner's idea 2026-11-05).** Beside postcards, a calendar made from one's own photos, to print or to share.

- **A calendar engine** built in, offline: it lays out any year (or a run of months from any month), weeks starting on Monday or Sunday, week numbers optional, in the interface language.
- **Important days:** the user marks dates (birthdays, anniversaries, trips) with a short label; public holidays of a chosen country could come from a bundled table, never from the network.
- **Photos:** one per month (or a cover and twelve), chosen from the library with the same crop, look ([FR-079](#fr-079--colour-adjustments)) and print quality as a print.
- **Export:** a multi-page PDF, or one PNG per page, at the print size's resolution.
- **Learnt from 7-Eleven Japan's multicopy** (research 2026-11-05, Fujifilm's pages): every product is a short line of steps (product, size, photos, adjust, check, output) with a small set of fixed templates that place and crop the photos themselves; the user changes only a few things (frame, colours, copies, date). Its calendar print (Feb 2026) is one L or 2L sheet in 8 designs, with a theme and a text colour and marked days; a third-party desk calendar adds Sunday or Monday start and the photo left or right of the grid. Neither offers twelve months, a year on one page or a PDF, so those are ours. **Seen on their prints** (the owner's photo of real 7-Eleven calendars, 2026-11-05): one L print holds **two months side by side with a cut line between them**, each half a small calendar of its own; a half is either the photo above the month's grid on a seasonal pattern (cherry blossom for March, mint waves for April), or the photo filling the half with the grid printed over it in a colour that suits it; the heading reads "3 / March" with the year, weeks start on Sunday. **So the calendar is built on the collage sheet** ([FR-035](#fr-035--multi-photo-layouts), [ADR-0032](adr/0032-collage-sheets-from-two-cell-plans.md)): two month cells to a print, with its cut line, border and looks; what is new is the month grid and a few designs. Worth borrowing too: a summary before output, and saying plainly that a print's edge is trimmed a little.

### FR-084 — Language packs anyone can add

**P2 (later, not scheduled; owner's request 2026-11-05).** A language beyond English and Vietnamese can be added without a new build, the way CrystalDiskInfo takes language files: a contributor writes one, others load it.

- **One standard:** a `.fotolang` file, UTF-8 JSON in the app's own i18next layout (the keys of `ui/src/locales/en.json`), with a header `{ "format": "fotollection-language", "schema_version", "code" (BCP 47), "name" (in the language itself), "app_version", "authors" }`. Keys missing from it fall back to English.
- **Checked like any input from outside:** read only in Rust, size-capped; only strings, no markup (the UI shows text, never HTML, [SECURITY.md](SECURITY.md)); every `{{name}}` placeholder must match English's; plural forms follow the language's rules. `ui/scripts/check-i18n.mjs` checks a pack the same way, so a contributor sees what is missing before sending it.
- **Two ways in:** a pull request that adds it to the repository (it then ships with the app), or **Load a language…** in the version popover, which copies the file into the app's `languages/` folder; the language list shows it with its authors.
- **Travels with the app's brain:** `.fotosettings` ([FR-082](#fr-082--the-apps-brain-in-one-file-fotosettings)) carries the packs a user loaded, as it carries fonts and LUTs; it is not the pack's own format, since a pack is made to be shared and the brain is personal.
- A pack translates the app. The website is in its own six languages (`site/_data/languages.yml`, ADR-0049), the full manual (USER_GUIDE) in English and Vietnamese and a short guide in the others; once the app speaks a language, that language's website shows the app's screenshots in it (`assets` in the registry).

### FR-085 — Lens distortion correction

**P1 (planned for v0.15.0 unless the owner moves it; found 2026-11-05).** Our own develop corrects the lens's distortion, as the camera's JPEG and macOS's ImageIO do, so Fotollection's develop can take over every format on every system.

- **From the file, never from a guessed lens profile:** the correction the camera itself records (Panasonic and Olympus maker data, which Micro Four Thirds lenses rely on), DNG's `WarpRectilinear` opcode.
- **Until then:** on macOS, Panasonic and Olympus RAW go to ImageIO first ([FR-016](#fr-016--raw-input)); elsewhere they are developed without the correction, as rawler did.
- **Measured:** against the camera's embedded JPEG by structure (SSIM) and by the shift of straight edges; budgets in [OPTIMIZATION.md §3](OPTIMIZATION.md#3-budgets) before it is built.

### FR-086 — One Settings window

**P1 (done, v0.12.0; owner's request 2026-11-05).** Every setting of the app in one window instead of a corner of each panel: a gear button beside "by @hoangphi01" opens it, laid out as the preview window is, in two columns as on an iPad, the narrow one a list of tabs and the wide one that tab's settings.

- **The cache, to manage** (owner, 2026-11-05): how much the thumbnails, the standard previews ([FR-091](#fr-091--instant-previews-standard-previews-on-disk-and-memory-used-well)) and the analyses take on disk against the limit, and how much memory the previews may use, with *Clear cache*.
- **In it:** the language, the theme, `.fotosettings` export and import ([FR-082](#fr-082--the-apps-brain-in-one-file-fotosettings)), language packs ([FR-084](#fr-084--language-packs-anyone-can-add)), the cache, updates, *Report a problem*, the version and the changelog, and every other app-wide setting now spread over the panels and the version popover.
- **RAM & cache, each control saying what it does to speed** (owner, 2026-11-05): the share of the computer's memory for previews (15 / 25 / 40 %, default 25), the disk cache's limit (2 / 5 / 10 / 20 GB, default 5) with its content by kind, *Prepare previews in the background* (on), *Clear cache*; **Default** and **Auto** (chosen from the machine's RAM and free disk: 40 % from 32 GB, 25 % from 12 GB, else 15 %; the largest cache within a tenth of the free disk; previews off under 5 GB free). Stored per machine in `settings.json` v15, clamped in Rust.
- **This computer, shown** (owner, 2026-11-05, as Photoshop's Performance preferences): *For this computer* lists the processor and its cores, the graphics card and whether it draws the colour sliders (ADR-0039), the memory and the cache disk's free space, so *Auto*'s choice can be read. A share of the graphics card's memory to set, as Photoshop's VRAM, comes with the develop on the GPU (`wgpu`, v0.15.0 "Core 2"); until then the GPU only draws a slider's step and needs no budget.
- **Memory given back** when the machine runs short (under a tenth available, checked every 5 s): the previews keep a tenth of their budget until memory comes back, then their whole share again ([ADR-0041](adr/0041-instant-previews-disk-and-memory.md)).
- **Built from the lego blocks** (STYLE_RULE §4), measured with `MEASURE=1`, in both themes and languages. Sections as on an iPad, as built: *General* (language, theme, motion, photo preview, the welcome, *New projects*), *Export*, *RAM & cache*, *Data* (FR-116, with `.fotosettings`, v0.14.0), *Updates*, *Manual* (FR-110), *Report a problem*, *What's new*; opened by the gear, the version badge (*What's new*) and ⌘, / Ctrl+,. The version popover is gone; language packs (FR-084) follow in their own version.

### FR-088 — RAW edited in linear light

**P1 (v0.10.0; owner's question 2026-11-05: "can a RAW be pushed as far as in Lightroom?").** On a camera RAW, the sliders a RAW editor applies before its tone curve act in scene-linear light, on the develop's own data, before the finish: **brightness as exposure** (±2 stops), **highlights** and **shadows** (brightness-weighted gains up to two stops, from middle grey up into the headroom above white, and down six stops), **warmth** and **tint** as channel gains (the same gains as on a JPEG, so each slider means the same). The rest of the look stays in the 33³ table ([ADR-0038](adr/0038-moderate-colour-editor-on-one-lut-engine.md)). JPEG, HEIC and PNG are unchanged.

- **Headroom:** each channel clips at its own saturation after white balance, and what nears a clip goes to a neutral at its brightest channel, never magenta or cyan; so highlights −100 brings back gradation an 8-bit picture has lost.
- **Speed:** the preview keeps each RAW's develop at 2048 px with its fits (`raw_finish::Prepared`); a step of one of the five sliders re-finishes that copy, 4–11 ms, and is drawn by Rust, not the GPU (whose shader only applies the table). The export develops at full size with the same fits, so what prints is what was shown.
- **Not yet:** Panasonic and Olympus RAW go to macOS's ImageIO first on a Mac (for its lens correction, [FR-085](#fr-085--lens-distortion-correction)), which gives an 8-bit picture: there the sliders still act in linear light, without the headroom.
- Measured old against new on the samples (RESULT §28).

### FR-089 — Lightroom presets

**P2 (built for v0.14.0, 2026-11-08 and 09; what it does not apply yet in v0.14.1 with FR-118; owner's request 2026-11-05, researched the same day).** *Import preset…* reads a user's own Lightroom® preset, an `.xmp` (`crs:PresetType="Normal"`) or a legacy Develop `.lrtemplate`, into a look of their own.

- **Parsed safely:** XML without DOCTYPE or entities, size- and depth-capped; an `.lrtemplate` read by a small parser of Lua's table constructors only, never run as Lua; every number clamped, NaN refused; each failure with its `ERR-` code. Fuzzing (`preset_mutate`) in v0.14.1.
- **Mapped honestly** ([ADR-0045](adr/0045-presets-compiled-into-the-look-table.md), owner 2026-11-07: curves, HSL and colour grading at once; measured 2026-11-09, [RESULT §39](RESULT.md#39-lightroom-presets-against-lightrooms-renders-and-dehaze-v0140-fr-089-fr-115)): the 2012-process sliders that match ours (contrast, shadows, vibrance, saturation, incremental warmth and tint) go to ours, exposure approximated, Dehaze to ours (FR-115); point curves, whites and blacks, colour grading, split toning and the black-and-white mixer are baked into one 33³ table kept as an imported LUT, the eight-hue mixer goes to ours; grain and vignette kept for v0.14.1 (FR-118). Listed after the import as *not applied*, with their reason: the spatial tools a look does not hold yet (clarity, texture, sharpening, noise reduction, lens corrections), the parametric curve, Adobe's profiles, an **absolute white balance** (it depends on each photo's own as shot; taken against daylight it moved the measured photos the wrong way) and **Highlights** (Lightroom's is local; ours moved both measured photos away from Lightroom's render at every strength). Measured with `preset_bench` against the owner's Lightroom renders.
- **Profiles** (`.xmp` with `crs:RGBTable`) become a LUT on the [FR-070](#fr-070--looks-from-lut-files) path, after the presets.
- **Never shipped:** no Adobe or third-party preset or profile is bundled or used as a test fixture; fixtures are written by us. Wording names Lightroom® only to say what is read.

### FR-087 — How large a collage sheet prints

**P1 (done, v0.12.0; owner's request 2026-11-05).** In Collage mode, the *Print size* card says how many pixels a sheet of two photos side by side or one above the other has (two 16 MP photos make a sheet of about W × H px), and whether that is sharp, acceptable or soft for the chosen size, as the resolution warning does for one photo ([FR-026](#fr-026--resolution-warning)). The pixels are the export's: the sheet at the less sharp photo's density, at most 600 DPI ([ADR-0032](adr/0032-collage-sheets-from-two-cell-plans.md)), shown as *Sheet* and *Prints at* in the card, in the panel and the sheet's preview.

### FR-090 — A filmstrip in the photo's preview

**P1 (done, v0.12.0; owner's request 2026-11-05).** In Pro mode, the photo's preview shows a strip of thumbnails under the photo, as Lightroom's filmstrip: the photos of the current view in their order, the one shown marked and kept in sight, so a user skims the set and picks a photo with one click instead of ← → one at a time.

- **Fast:** the strip uses the gallery's T1 thumbnails (no new decode), virtualized so a 500-photo set scrolls smoothly; moving to a far photo starts its preview at once and never waits for the last one's (LESSONS 2026-11-05).
- **The same order and filter as the gallery**, with the RAW, rating and print-basket badges of the grid tiles; select mode and the cull keys keep working.
- **Under the photo, not beside it (owner's choice, 2026-11-06):** most camera photos are landscape, and the stage is wider than 3:2 on most screens, so a 112 px strip below costs a landscape photo nothing from 900 px of screen height (−9 % at 768 px), where a strip on the left would cost 12–13 % everywhere. It fills `ViewerFrame`'s `below` slot, the one the capture screen's shots already use; Simple mode's quick preview and Collage's sheet preview do not show it.
- **Hidden with one button** beside the photo's counter (*3 / 48*), for low screens and portrait sets; the choice is kept (`ui.filmstrip`). It also steps aside while a crop is being edited.
- **Lego:** one block, `Filmstrip` (`ui/src/components/layout/filmstrip.tsx`), for the capture screen's shots and this strip, at a fixed height (STYLE_RULE §4).

### FR-091 — Instant previews: standard previews on disk and memory used well

**P1 (done, v0.11.0; owner's request 2026-11-05: "like Lightroom, once the gallery has loaded any photo clicked shows almost at once; use the RAM well, not as little as possible").** [ADR-0041](adr/0041-instant-previews-disk-and-memory.md).

- **Standard previews on disk:** after a folder's thumbnails, every photo's 2048 px upright sRGB source (for a RAW, finished with its RAW colour and no linear slider) is made in the background and kept in the cache folder; a click on any photo, far or near, opens from it instead of decoding or developing (measured 27 ms against 278 ms for a JPEG and 440 ms for a RAW, [RESULT §29](RESULT.md#29-instant-previews-v0110-fr-091)). Made one photo at a time behind the photo on screen, paused during an export or an analysis, with its progress in the gallery's bar.
- **Memory used well:** the preview keeps up to a quarter of the machine's RAM (on a 16 GB Mac about 190 sources, 20 RAW develops and 600 MB for zoom), the oldest let go first, the photo on screen always kept.
- **Prefetch:** once a photo is on screen, the next two in the direction of travel and the one behind are loaded into memory, so ← and → show at once ([NFR-012](#nfr-012--culling-navigation)). A prefetch never pushes the photo on screen out of memory, and a newer one replaces what is left of an older one (v0.11.1).
- **The photo on screen first:** every RAW develop for work the user does not wait for (standard previews, prefetch, the export's estimate, an export, an analysis) waits while the photo on screen needs one; a photo the user has browsed past no longer goes first (v0.11.1).
- **The cache:** its limit is 5 GB (was 2 GB), the oldest files go first, standard previews before thumbnails, and it holds during the previews' pass too (v0.11.1); *Clear cache* empties the standard previews too. The Settings window shows it ([FR-086](#fr-086--one-settings-window)).

### FR-092 — Export quality on a slider, with a live estimate and a print verdict

**P1 (v0.13.0; owner's request 2026-11-06: "a slider like Lightroom's, but better").** The export dialog ([FR-093](#fr-093--the-exports-options-in-one-small-dialog)) and the Settings window's *Export* section get a *Quality* slider (JPEG quality, 60–100, default 95 as [FR-008](#fr-008--original-quality-export) says), as Lightroom's export dialog has, and goes further:

- **The estimate follows the slider:** while it moves, the export's size estimate ([FR-053](#fr-053--export-for-social-media)) updates for the value under the thumb, not only after release; the last answer stays while the next is computed, and a newer value replaces an older one in flight (never a queue of encodes).
- **Whether it prints well, as badges:** beside the value, a badge in the style of the resolution warning ([FR-026](#fr-026--resolution-warning)) says what the setting is good for, for example *Print, excellent*, *Print, good*, *Screen only*, with a hint explaining it. Measured on the owner's photos at print size ([RESULT §30](RESULT.md#30-export-quality-on-a-print-v0130-fr-092)): 90 and above *Prints beautifully*, 75 to 89 *Prints well*, below 75 *Screens only*; 4:4:4 chroma stays for prints.
- **Lego:** the shared slider and badge blocks, measured in both themes and languages; Collage and postcard exports use the same card.

### FR-093 — The export's options in one small dialog

**P1 (v0.13.0; owner's request 2026-11-06: "gather every export option in one small pop-up instead of spilling down the right-hand panel; a press on Export shows it, with a save button and *don't ask again*, after which Export just exports; to change, go to Settings").** The panels keep only the *Export* button with its count (ticked photos, sheets) and the size estimate. A press on *Export* opens a small dialog that holds every option that today stacks in the panel's footer:

- for print or to share (preset, long edge) and the share fill (ratio, width and height, colour);
- folders together or apart, one folder or by day;
- the quality slider with its live estimate and print badges ([FR-092](#fr-092--export-quality-on-a-slider-with-a-live-estimate-and-a-print-verdict));
- the destination folder.

- **Its buttons:** *Export* (saves the choices and exports) and *Cancel*, with a *Don't ask again* tick. Without the tick, every press on *Export* opens the dialog again, filled with the last choices. Once ticked, a press on *Export* in the panel exports at once with the saved choices; the panel says which ones in one line (for example *Print · by day · quality 90*).
- **Changing them later:** in the Settings window ([FR-086](#fr-086--one-settings-window)), which holds the same options (the same block) and the tick to ask again.
- **One dialog everywhere:** the gallery, Simple mode, Collage, the photo's and the sheet's previews open the same dialog, built once ([STYLE_RULE](STYLE_RULE.md) §4); progress, the result and errors stay in the panel where the export started.
- **As Lightroom's, the Settings window's size** (owner, 2026-11-06: "the size of Settings is fine, look at Lightroom's export"): presets on the left (*Prints, original size* at quality 90, *Facebook*, *Instagram* at 88, *Share, original size* at 80, *Custom*), the options in cards on the right (*Where*, *File*, *Size*, *Names & details*), *Don't ask again*, *Cancel* and *Export N photos* at the foot; `Esc` cancels. A preset sets the target and its quality; the user's own presets come with the Looks.
- **Kept per machine** (settings v16): the quality, *ask every time*, together or apart, and the destination, which is checked when an export starts ([ERR-EXP-0004](ERROR_CONTROL.md) if the folder is gone). The target and the fill stay the project's print settings, so the Settings window's *Export* section holds the machine's part only.

### FR-094 — Project history, without nagging

**P2 (planned, version to be chosen; owner's question 2026-11-06: "Lightroom kept asking to back up its catalog; does ours need it?").** Fotollection already never touches the photos, writes every project and `settings.json` atomically (a temporary file, fsync, then a rename) and keeps one `.bak` of the last save ([DATABASE §4.4](DATABASE.md#44-write-protocol)), so a crash or a full disk cannot leave a broken project, which is what Lightroom's backups guard its one large catalog against. What it lacks is **history**: the single `.bak` is replaced at every save, so a mistake saved a few times over (a hundred photos rejected by accident) cannot be undone after the app is closed.

- **Kept automatically, never asked:** a dated copy of each project and of `settings.json` on the first save of each day, the last seven days and the last four weeks kept, in the app's data folder; a project is small (kilobytes of JSON in its bundle), so this costs little.
- **Restored from the Settings window** ([FR-086](#fr-086--one-settings-window)): *Project history* lists the copies by date with their photo counts, and opens one as a new document, never over the current one.
- **Measured and tested:** the write stays off the interactive path; a test makes a week of saves and checks which copies are kept.

### FR-095 — A develop view a Lightroom user can read

**P1 (done, v0.12.0; owner's request 2026-11-06: "complete and professional, done well, not rushed; a Lightroom user should feel at home fast, with our own character").** Pro mode's photo preview becomes a develop view.

- **Two layouts, chosen in Settings → General** (`ui.previewLayout`): *In a window*, the dialog as before, and *Fill the app window*, edge to edge under the Windows title bar (FR-052). `F` switches between them and the choice is kept.
- **A left column in the full layout** (18 rem, as the side panels): *Navigator*, the photo small with a frame for the zoomed area (click or drag to move it; Fit, Fill, 1:1, 2:1), and *History*, this photo's steps newest first, named by what changed (*Warmth +8*, *Crop*, *Paste settings*); a click brings the photo back to that state as a new step, so it can be undone. v0.14.0 adds *Presets & LUTs* at its top (FR-079, FR-070). `Tab`, pressed with the photo itself focused (so it still moves between the inspector's controls), hides both side columns.
- **One toolbar row above the photo**, never over it: Crop, rotate, basket, Before/After and compare, undo and redo, copy and paste settings, Reset, Export; every button the same shape, each hint naming its key.
- **Lightroom's keys:** `R` crop (`C` kept), `\` before/after, `Y` compare, `Z` fit ↔ 1:1, `F` layout, `Tab` panels, `J` clipping, `W` white-balance picker, ⌘Z / ⇧⌘Z, ⇧⌘C / ⇧⌘V copy and paste settings, ⇧⌘R reset, ⇧⌘E export, ⌘[ ⌘] rotate; the ratings, flags and arrows as before. The keys popover lists them all.
- **Zoom in real pixels:** *Fit*, then percentages of the photo's own pixels, 100 % being 1:1.
- **The inspector in tabs** (owner, 2026-11-06, after a survey of Lightroom's tool strip and Capture One's tool tabs): a row of icon tabs at its head, the inspector staying 18 rem: *Edit* (this photo, RAW, Basic, Tools; `E`), *Crop* (the crop and straighten mode; `R`), then *Print* (print size, stamp, frame, export) and *Info* (rating, captured, place; `I`). *Heal* (FR-101) and *Mask* (FR-102) take their places beside *Crop* once built, hidden until then. One card open at a time in a tab (Lightroom's solo mode); the tab is kept for the session.

### FR-096 — Before and after, side by side

**P1 (done, v0.12.0; owner's request 2026-11-06).** `\` and the *Before* button switch between the photo with and without its colour (keyboard and mouse alike); `Y` shows them side by side, `⇧Y` split by a line one drags across the photo, `⌥Y` one above the other, and the menu beside the button also splits them top and bottom, as Lightroom's views (owner, 2026-11-06). Both halves zoom and pan together and stay sharp at 1:1. *Before* keeps the crop and the frame: what changes is the colour. Press and hold the photo (owner, 2026-11-08, as the iPhone's editor): a still press of 250 ms shows it without its colour until released; a click still zooms and a press that moves still drags. In a collage sheet's preview, where a click picks a photo, *Before* in the toolbar is held instead: the chosen photo, or the whole sheet, without its look while held.

### FR-097 — Settings from one photo to others, and Reset

**P1 (built for v0.13.1, 2026-11-07; owner's request 2026-11-06).** *Copy settings* (⇧⌘C) asks what to copy, in a short list (look, RAW look, crop position, stamp), as Lightroom's dialog; *Paste* (⇧⌘V) gives them to the photo shown or to every ticked photo (Lightroom's *Sync*), as one step to undo. *Reset* (⇧⌘R) gives the photo back to the shared settings: its look, RAW look, crop and own stamp, as one step. Undo and redo are buttons as well as keys. As built (owner's choices, 2026-11-07): Copy, Paste and Reset sit in the toolbar right after undo and redo; with photos ticked, Paste asks *This photo* or *N ticked photos* (⌥⇧⌘V pastes to the ticked ones), and the selection bar of the gallery and Collage has *Paste settings*. A paste never carries the rating, flag or turns; Reset keeps them. On a collage sheet the crop copied and pasted is the photo's collage crop.

### FR-098 — Reading the photo: histogram, clipping, white balance

**P1 (v0.13.1; owner's request 2026-11-06; the histogram built 2026-11-07, `J` and `W` planned).** A histogram (red, green, blue and light) sits at the top of the develop view's left column, above the Navigator (owner, 2026-11-06; it heads the inspector in the dialog layout, which has no left column), made in Rust from the preview just drawn (≤ 3 ms, OPTIMIZATION §3) and following a slider while it is dragged. `J` shows blown highlights in red and blocked shadows in blue over the photo. `W` picks white balance: a click on what should be neutral grey sets warmth and tint, as one step. As built: the histogram counts every second pixel of every second row of the preview's resized photo through its look (1.0 ms p95, RESULT §32), never the stamp, frame or border; it says how much is blown and blocked when any is; in Collage mode it heads the picked photo's cards in the sheet preview. In the window layout it stays in the left column also when a narrow window folds that column (owner, 2026-11-07). `J` (or the ⚠ button on the histogram's title) marks the clipped pixels from Rust (`clip=1`), so the marks follow a slider as the preview does. On a collage sheet (v0.13.2) `J` marks both photos' clipped pixels, never the paper, card or cut line (`sheet/{a}/{b}?clip=1`); the histogram there stays the picked photo's own preview's (a histogram of the photo as it sits in its cell comes later). `W` (or the pipette on the histogram's title) samples the photo before its look around the click, 5 × 5 of its own pixels (never the border, frame or stamp: `ERR-IPC-0026`), and sets warmth and tint by the exact inverse of their gains, one step; a RAW keeps the warmth and tint it was developed with and adds the correction. On a collage sheet (v0.13.2) `W` works on the picked photo: the sheet is drawn twice, that photo plain and then inverted, and only pixels that are the exact inverse are its own, so a click on the other photo, the card or the paper is `ERR-IPC-0026`.

### FR-099 — Controls with depth

**P1 (planned, v0.13.0, the new look; owner's request 2026-11-06, pointing at Sentry's interface, then moved forward from v0.14.0 the same day: "to transform the app").** Buttons and controls stand out from the surface: a raised button has a darker lip under it in a shade of its own colour and a faint highlight on top, and goes down by its lip when pressed; inputs and segmented groups sit slightly into the surface; cards carry a faint border and a lifted shadow; a selected row has an accent outline. Done once in the shared primitives and theme tokens (`button`, `toggle`, inputs, `SectionCard`), so every panel follows; measured with `MEASURE=1` in both themes; the photo stage stays neutral.

### FR-100 — Motion that feels like one app

**P2 (M1–M2 in v0.14.0, M3 in v0.14.1, owner 2026-11-09; owner's request 2026-11-06: "the app's animation is local, not in step and not smooth yet; transitions should be smoother"; moved into v0.14.0 on 2026-11-08 with all of it, a setting beside the system's *Reduce motion* and the website on the same tokens; [ADR-0046](adr/0046-one-motion-system.md), the details in [TRANS_ANIMATION.md](TRANS_ANIMATION.md)).** Today each piece animates on its own: a dialog zooms in, a card folds, a photo fades, with different timings and some changes that simply jump. One motion system, set once:

- **Shared timings and curves** as theme tokens (a short one for controls, a longer one for panels and views), used by every block, so the whole app moves at one pace.
- **Transitions between views:** the gallery to the preview and back (the photo growing from its tile), the dialog and the full-window layout (FR-095), the modes, the Settings window's sections; panels and the filmstrip slide rather than appear.
- **No jump left:** a list that changes (the filmstrip, History, notices, the gallery after a filter) moves its rows to their new place; a card that folds or a badge that comes keeps the rest still until it moves smoothly.
- **Never in the way:** nothing waits for an animation; a photo shows as soon as it is ready; *Reduce motion* in the system, or *Settings → General → Motion: Reduce*, turns it all down to quick fades (spinners keep turning). Measured: 60 fps on the reference machines (OPTIMIZATION §3), checked in both themes.

### FR-101 — Healing: spot removal

**P2 (planned, about v0.16, after v0.15.0 "Core 2"; owner's request 2026-11-06, as Lightroom's Remove tool).** A tool tab in the develop view's inspector (FR-095), shown only once built: a click or a drag on the photo marks a spot (dust, a blemish, a wire), and the app fills it from a nearby source it picks, which the user can move; *Heal* blends the tone, *Clone* copies. `Q` as in Lightroom Classic. Kept per photo, undoable, in the project; done on the full-resolution photo for the export, on the preview's pixels while working. Later, a content-aware fill made on the machine, never online.

### FR-102 — Masking: local adjustments

**P2 (planned, about v0.16, with FR-101; owner's request 2026-11-06, as Lightroom's Masking).** A tool tab in the develop view's inspector (FR-095), shown only once built: linear and radial gradients and a brush, each a mask with its own Basic sliders (brightness, contrast, warmth, tint, saturation…), several per photo, listed, hidden or deleted one by one. `⇧W` as in Lightroom Classic. Later, subject and sky found on the machine by a small model of a permissive licence (never a network call). Needs a local layer in the render pipeline beside the look's global table (ADR-0038, amended), on the GPU of v0.15.0 for a live preview. After FR-115's local contrast; the linear gradient first (a darker sky), the radial and the brush next, subject and sky last (owner, 2026-11-08).

### FR-103 — Check for updates at start, never download unasked

**P2 (v0.13.0; owner's request 2026-11-06: "check only, the user decides to download; on from the install, off only when the user turns it off in Settings, since the app still has bugs"; "a popup, and a red warning below 0.10").** A switch in Settings → Updates, *Check for updates when the app starts*, **on by default**; turned off, it stays off.

- **The check:** once at start, in the background, from the app's shell, never the core. It reads the latest release and a small `update.json` published with it: the version, and the *minimum safe version* with its reason. Nothing about the user or the machine goes in the request beyond what any HTTPS request carries. Offline, nothing is shown.
- **A newer version** opens a small window with *What's new* (the release page), *Download* (the installer for this system, in the browser) and *Later* (until the next launch). Nothing is downloaded or installed by itself.
- **How the minimum travels:** `.github/update.json` (`minSafe`, and a reason in English and Vietnamese) is appended by `release.yml` to the release notes as a hidden comment, so the one request carries it (RELEASE.md §5). The first one says 0.10.0. A note that does not parse, or a minimum above the latest release, is ignored.
- **Tried without a release:** a debug build reads a release from a file (`FOTOLLECTION_UPDATE_FEED`, the fixtures in `src-tauri/tests/fixtures/update/`) and can pretend to be older (`FOTOLLECTION_FAKE_VERSION`); a release build never reads either.
- **Below the minimum safe version** (a release that fixes a serious fault, for example a version that showed zoomed portraits sideways), the window carries a red warning that names the fault and asks to update. Only versions that carry this feature can show it, so it ships as soon as it can.
- SECURITY.md and README's privacy lines say so: one request at start, which can be turned off. The manual *Check for updates* stays as it is (FR-021).

### FR-104 — File names as Lightroom's templates

**P1 (v0.13.0; owner's request 2026-11-06: "custom names, date and file name, like Lightroom").** The export dialog's *File name* card ([FR-093](#fr-093--the-exports-options-in-one-small-dialog)) offers templates, kept per machine: the original name (`P1057694.jpg`, the default), date and original name (`2026-07-14_P1057694.jpg`), custom text and a sequence (`Prague-001.jpg`), custom text, date and sequence, and date and sequence. The sequence starts where the user says and is padded to the width of the last number; it follows the export's order (capture time) whatever thread finishes first. A live example shows the first name the export will write, computed by the same code. Custom text is cleaned in Rust (no path separators or reserved names). Collage sheets keep `<first>+<second>` under the original-name template and take the template otherwise.

### FR-105 — Limit the file size

**P2 (v0.13.0; owner's choice 2026-11-06, Lightroom's *Limit file size*).** A switch and a size in the *File* card: each file is written at the chosen quality when it fits, else at the highest quality from 60 up that does, by bisection; a photo too large even at 60 is written at 60 and counted in the result. Measured first ([RESULT §31](RESULT.md#31-a-file-size-limit-on-a-print-v0130-fr-105)): free until it binds, then about twice an export's time. Off by default.

### FR-106 — Output sharpening

**P3 (later; owner, 2026-11-06: "note it in the docs, do it later").** Lightroom's *Output sharpening* for screen, matte or glossy paper, at low, standard or high: measured first on prints at their size before any default is chosen. Share exports already sharpen lightly after a downscale (FR-053).

### FR-107 — Folder tabs in the order the user drags them

**P2 (v0.13.0; owner's request 2026-11-06: "press and hold, or press and drag a folder, to sort them, as Chrome orders its tabs").** The folders of the library column ([FR-056](#fr-056--folders-as-tabs)) can be reordered: a press that moves or is held lifts the row, the others slide aside, and it drops where it is let go; *Escape* puts it back; **Alt+↑ / Alt+↓** moves the focused folder one place. A click still opens the folder. The order is the project's, saved with it and with the session (the `folders` rows are written in that order), so it survives a restart; the folders' ids do not change, so the chosen tabs stay chosen. Not an undo step, as a tab's move is not.

### FR-108 — Photobooth with the computer's own camera

**P3 (later; owner, 2026-11-07: "a photobooth feature later: shoot with the computer's webcam, as in a shop's booth; the Mac's webcam is quite pretty").** A booth screen that shoots with the camera built into or plugged into the computer, as a photo booth in a shop does: a live mirror view with the print's ratio drawn over it, a countdown, a flash of the screen, then a few shots in a row laid out as a strip or a sheet ready to print (the strip and square sizes of the self-service kiosks, or a size of its own, [FR-043](#fr-043--custom-print-sizes)) or kept as single photos in a session's folder tab, as tethered shots are ([FR-063](#fr-063--tethered-capture-over-usb)). Fully offline: the frames never leave the computer, nothing is uploaded, and the camera is asked for only while the booth is open, through the system's own permission (macOS camera access, Windows' camera privacy setting, Linux's V4L2 device). To be designed when its turn comes, starting from the capture screen of FR-063.

### FR-109 — A welcome screen on the first launch

**P1 (v0.13.1, built 2026-11-07; owner's request 2026-11-07).** The first time the app opens after it is installed, a short welcome shows the basics, at most five pages, each a screenshot of the real app and two lines, with **Next** (and Back) to move on: (1) **Welcome**: open a folder of photos, or drop one on the window; (2) **Three modes**: Simple, Collage and Pro, and what each is for; (3) **Choose**: Find best shots, the Workspace, Set aside and the print basket; (4) **Make it yours**: colour, the date stamp and the info frame; (5) **Print**: export to the shop's sizes. The last page has a **Show at every start** tick (off by default) and a **Start** button that closes it. Only *Start* closes it: no ✕, and neither Esc nor a click outside (owner, 2026-11-07: "whoever installs it should read it through"). It can be opened again from Settings or the Help menu. Whether it was seen and the tick are kept per machine in the settings (a new schema version, since v16 shipped in v0.13.0). Offline, in English and Vietnamese, the screenshots from the site's pipeline. As built (owner, 2026-11-07): in the Settings window's frame and size (`SettingsFrame full`), the pages listed on the left as its sections; each page's picture fills the right side, and its words sit on it in frosted glass; shown once after an update too, since v16 had no welcome; reopened from Settings → General.

### FR-110 — The manual inside the app, offline

**P1 (v0.13.2; owner's request 2026-11-07: "bring the whole manual into the app, in Settings, so people without internet can still read it").** The manual the website shows ([ADR-0031](adr/0031-manual-on-the-website-from-the-user-guide.md)) is in the app too, offline, in the app's language: `docs/USER_GUIDE.md` and `USER_GUIDE.vi.md` bundled at build time, one source for the site and the app. Opened from Settings (a *Manual* section) and the Help menu, in the Settings window's frame: the chapters listed on the left, the open chapter on the right with its screenshots (bundled at about 1,100 px, the theme's own, a few MB), a search over the chapters' titles. Rendered from Markdown to React elements, never `innerHTML` (SECURITY.md); links to the project's technical docs show as plain words, links to the web open in the browser only when clicked.

As built (2026-11-07): the manual is a section of the Settings window itself, *Manual*, never a window of its own (owner, 2026-11-07); macOS has a *Help* menu with *Fotollection Manual* (⌘?), which opens it there. A chapter picker heads the section, with a search over the chapters' titles and words (case and accents aside, so *cat anh* finds *cắt ảnh*); *Back* and *Next* at the window's foot. `ui/src/lib/manual.ts` splits the guide as `scripts/site-manual.sh` does; the reasons in brackets (*(FR-001)*) are left out for the reader. Rendered by react-markdown with remark-gfm (MIT), raw HTML skipped. The guide has no web link besides the site's own manual, so every link shows as its words and the app opens nothing. The layout is the app's, not the page's (owner, 2026-11-07, after the first look: "take only the words and the pictures"): each chapter is cut into cards (its opening, then each section, each paragraph led by a bold name and each bold-led step), in the Settings window's column, keys shown as keys (STYLE_RULE §4). The 27 screenshots are the site's at 1,100 px (`scripts/app-pictures.sh`, 7.6 MB in `ui/public/manual/`, inside the 150 MB install budget, NFR-008), the theme's own; `check-docs.sh` fails when one is missing.

### FR-111 — Lossless export, 16-bit or 32-bit float TIFF (Pro)

**P2 (v0.15.0, owner 2026-11-09, moved from v0.16.0 "Lossless"; owner's decisions 2026-11-07).** Beside the print JPEG, Pro mode can export a photo **losslessly**: TIFF **16-bit** by default, or TIFF **32-bit float** as a choice for editing elsewhere, HDR or archiving. It is an option, never the default: exports stay compressed JPEG, and Simple and Collage modes do not offer it (on a 10 × 15 print no one can tell). The photo is processed in **32-bit float from decode to encode**, so a RAW keeps what lies above white and is quantised only when the file is written; JPEG and 8-bit sources are widened, never compressed again. The file carries the photo's ICC profile and the EXIF allowlist, and never GPS. Designed from a study of RawTherapee and darktable ([ADR-0043](adr/0043-lossless-export-f32-path-tiff.md)); no code of theirs is used.

### FR-112 — Highlight recovery for RAW

**P2 (v0.14.1, moved from v0.14.0 by the owner, 2026-11-09; owner's request 2026-11-07: "learn how to rescue blown highlights instead of fading to white").** Where a RAW's channels clipped, the develop rebuilds their colour from the unclipped channels and the edge of the clipped area, and keeps brightness above white, so lowering exposure or *Highlights* brings back the sky, a lamp or a cloud instead of a flat white patch. It replaces today's fade to neutral (`highlights_to_white`), which stays as the fallback where nothing can be rebuilt. **Chosen and tuned only by measurement** (owner, 2026-11-07: "always through experiments and numbers, never assumed good"): a harness with known truth (synthetic clipped skies, coloured lights, two colours meeting, and the owner's bracketed 0 / −2 EV pairs) runs the current fade and several candidate methods (an average of the edge's colour, edge colour ratios filled smoothly, and variants), each parameter swept, the winner and its numbers kept in RESULT. Methods learned from published papers and manuals only (ADR-0043).

### FR-113 — ID photos on one print

**P2 (planned for v0.15.x, after the colour engine and v0.15.0's 32-bit export (moved with FR-111, 2026-11-09); owner's request 2026-11-07: "for a visa, a CV or a passport in the EU, quick and cheap").** One photo, cut to a document's size, repeated on one print with cut lines, so a 10 × 15 from any kiosk gives a sheet of ID photos for a few cents.

- **Sizes:** presets in millimetres, each with its name and where it is used: 35 × 45 (passports and Schengen visas in the EU and many other countries), 30 × 40 and 40 × 60 (Vietnamese documents), 20 × 30, 51 × 51 (2 × 2 in, the United States), and a size of the user's own.
- **The crop:** the photo's crop editor at that ratio, with guide lines for the head from the size's own rules where it has them (for 35 × 45, chin to crown 32–36 mm, eyes level, centred), drawn on screen and never printed. Placing the face automatically, from the faces the best-shot analysis already finds (FR-041), is a later step, measured before it is offered.
- **The sheet:** as many copies as fit the print size at their true size in millimetres (eight of 35 × 45 on 10 × 15, a 2 mm gap and thin grey cut marks between them), the count shown before export; or a number of copies the user asks for. Printed at exact scale, as every print (FR-007): the photo is never stretched to fill the paper.
- **Honest about the rules:** a short note says that offices decide what they accept (background, light, expression, glasses), that some countries now take photos only from approved studios or at the office, and that the user should check before printing. No claim that a photo is compliant.
- **Never touched:** no retouching, no background replacement; the date stamp, the info frame and the border are off on an ID sheet.

### FR-114 — A look, LUT or preset given to many photos at once

**P2 (v0.14.0; owner's request 2026-11-08: "Lightroom is poor at applying a preset to the photos already in a folder, it only applies one at import").** Pro mode's gallery panel has two tabs, as the photo preview's inspector (FR-095): **Colour** and **Print** (the cards it always had; it opens on Print). The Colour tab gives one look of the library (a built-in look, one's own, an imported LUT or a Lightroom preset, FR-079, FR-070, FR-089) to many photos:

- **Which photos:** the ticked ones (select mode, FR-050), else every photo the gallery shows (its folder and filter), as an export; at most 10,000. The button says how many: *Apply to 12 ticked photos*, *Apply to 422 photos*.
- **Seen first:** the chosen look on the photo picked in the grid, Rust's small tile; an **Amount** (100 by default) as a preset's.
- **Photos with a colour of their own** (a look that does something and is not the one given) are asked about once: **Apply to all** or **Keep theirs**. Crops, date stamps and RAW settings stay either way.
- **Your looks first:** on the strips, Original comes first, then the looks you saved, then Fotollection's, and Simple mode's keys 0–9 follow that order (0 is always Original); a right click on one of yours renames or removes it. The Looks card names the look in use (Original, its name, or *Custom* once a slider moves it) and, for a custom colour, offers **+** to keep it as a look (owner, 2026-11-08).
- **Back to the original colour:** a red ↺ on the card's title takes the same photos' looks away (the built-in *Original* given back), after one question in the same dialog; crops, stamps and RAW settings stay.
- **One undo step** for the whole batch; each photo's History names the look. The library is the preview's *Presets & LUTs* list, one block (`useLookLibrary`) with the built-in looks added, its **+** importing presets and LUTs here too.
- **Rust first:** Rust picks the photos, counts the edited ones and builds the look from its library (the interface sends the filter, the ticked ids, the look's id, the amount and its shown name), under one write lock: 7.4 ms for 10,000 photos, its undo 3.4 ms (RESULT §36). The grid shows the new colour: Rust colours the batch's thumbnails in the background, 1.1 ms each, the import's progress bar showing it ([ADR-0047](adr/0047-thumbnails-show-the-look.md)).
- **Later:** giving a look to the photos imported into a folder from then on, as Lightroom's import presets (FR-038), after v0.14.0.

### FR-115 — Local contrast: clarity, texture and dehaze

**P2 (Dehaze built for v0.14.0, 2026-11-09, owner: "do dehaze now"; clarity and texture in v0.14.1 with FR-118 (owner, 2026-11-09), before FR-102's masks; owner's request 2026-11-08, from street photos graded in Lightroom: "this is where our Pro mode is heading").** The three spatial tools a colour table cannot hold (FR-089, ADR-0045), in the Tools card of Pro mode: **Clarity** (contrast of mid-sized detail), **Texture** (fine detail, skin and surfaces) and **Dehaze** (haze taken out by an estimate of the scene's veil), each −100 to +100 as Lightroom's.

- **Why first:** the graded city photos the owner showed (Tokyo, Shinjuku, Brooklyn, Radio City, Park Avenue) owe most of their look to global colour, which looks and presets already hold, and to local contrast, which nothing in the app does yet; without it a preset that uses them lands flat, and they are listed today as *not applied*. Masks (FR-102) come after, a linear gradient first (a darker sky, the commonest local edit), the radial and the brush next, subject and sky found on the machine last.
- **Rust first (ADR-0044):** clarity and texture in the render pipeline after the look's table, Dehaze before it (below), on the luminance only, so colours do not shift: clarity and texture as unsharp masks at two radii scaled to the photo (a guided or edge-aware filter, so halos stay off strong edges, chosen by measurement), dehaze from a dark-channel estimate; tiled and parallel on rayon, on the GPU of v0.15.0 for the live preview, the CPU for exports with the same result.
- **Measured, never assumed:** the filters and their radii chosen against the owner's Lightroom renders of presets that use them (`preset_bench`, RESULT), halos scored at strong edges; budgets of a 1600 px preview and a 24 MP export set in OPTIMIZATION §3 before the code.
- **Dehaze, built (v0.14.0, `dehaze.rs`):** the dark channel prior (He, Sun and Tang, 2009): the veil estimated on a copy at most 512 px across, smoothed by the fast guided filter (He, 2015) and read over the full photo, so a preview and an export agree; + takes haze out, − lays a milky veil on, 0 changes no byte. It runs **before** the look's table, not after: the GPU preview draws the table over a base that already has it (ADR-0039), so both paths keep one order; its slider is drawn by Rust at each step. In the Tools card, first. Measured on a scene behind a known veil (RESULT §39): ΔE 20.3 → 1.7 at +100; 6.2 ms on the preview, 30 ms at 24 MP. Its strength against Lightroom's waits for one-slider exports from the owner.
- **Presets:** a Lightroom preset's `Clarity2012`, `Texture` and `Dehaze` map to them, leaving the *not applied* list.
- **What it does not do:** no motion trails, reflections or composites: those belong to the shot.

### FR-116 — Data on this computer, in Settings

**P2 (built for v0.14.0; owner's request 2026-11-09: "build a data menu in Settings: how does a user move to another computer?").** Owner's choices (2026-11-09): an import **merges** (the default) or **replaces**; two buttons to start afresh, *Reset settings* (the user's data kept) and *Erase data*; only *unused LUTs* and *recent addresses* can be cleared from the list (the cache from *RAM & cache*). Settings gets a **Data** section, built from the lego blocks:

- **Move to another computer:** *Export settings…* and *Import settings…* ([FR-082](#fr-082--the-apps-brain-in-one-file-fotosettings)); the import shows what the file holds (looks, LUTs, fonts, logos, print sizes, addresses) and asks to merge or replace.
- **What Fotollection keeps here:** one row per store (settings, looks and LUTs, fonts, logos, recent addresses, camera sessions, cache, logs) with its size, count and *Open folder*; *Clear* (red, asked first) only where clearing loses nothing that cannot be made again.
- **Reset settings:** back to the defaults, the old file kept as `.bak`, asked first.
- **Moving:** export settings on the old computer and copy the `.fotollection` projects; on the new one, import, open each project and relink its photo folders. Projects carry the LUTs and fonts they use (S3 of the 2026-11-09 plan).
- **Owners ([ADR-0048](adr/0048-one-owner-per-service-failures-stay-local.md)):** each row asks its store's owner for its size and its clearing; the section owns no data.

### FR-117 — Right-click menus in more places

**P2 (planned for v0.15.0; owner's request 2026-11-09, shown with Lightroom's folder menu: "some features can be right-clicked but are laid out, so it costs scrolling").** A right click on what the user works on opens its actions where the pointer is, so a panel need not show every action as a button and the user scrolls less.

- **Where:** a folder in the folder list (open, rename, remove, show in Finder or Explorer, export this folder), a photo in the gallery and the filmstrip (into or out of the basket, rotate, copy and paste its look, reset its colour, show in Finder or Explorer), a slot of a collage sheet (replace, take out), the rows of the export presets and of the looks (already: the looks strip, FR-079). The list of each is chosen with the owner before the code.
- **What a menu holds:** only actions the panel or a shortcut also offers, so nothing is hidden in a menu alone; groups of related actions split by a hairline, as Lightroom's; an action that opens a dialog ends with "…"; a submenu (an arrow) only where a choice has many values (a colour label); the remove last, red.
- **Style ([STYLE_RULE.md §4](STYLE_RULE.md#4-uiux)):** every menu is `MenuContent`, its rows `ListRow`s 32 px high with their Lucide icon, the red row `RemoveButton wide`, both themes and languages, measured; the web view's own menu stays off (`lib/no-web.ts`). Keys: the context-menu key and Shift+F10 open it on the focused item.
- **Then:** with the menus in, the panels may drop buttons used rarely, one at a time, each by the owner's word.

### FR-118 — Every preset setting applied

**P2 (v0.14.1, "Looks, part 2"; owner's request 2026-11-09: "v0.14.1 covers and completes what the app reports missing from presets").** After v0.14.0 a preset's import names what no look applies (FR-089). v0.14.1 applies each of them, so the list empties, every one chosen and tuned against the owner's Lightroom exports (`preset_bench`, one slider at a time, RESULT), never guessed (ADR-0044, the owner's rule on image methods):

- **Clarity and texture** ([FR-115](#fr-115--local-contrast-clarity-texture-and-dehaze)), and Dehaze's strength tuned against Lightroom's.
- **Highlights, local:** Lightroom's Highlights (and Shadows, Whites, Blacks where they differ) as a local tone map, so a preset's −51 means what it means there; today's global slider stays for the look's own sliders unless the measurement says one tool serves both.
- **The parametric curve** (`ParametricShadows`, `Darks`, `Lights`, `Highlights` and their split points), baked into the look's table with the point curves.
- **White balance in kelvins**, against each photo's own as shot (a RAW's from its white balance, a JPEG's from its XMP when it has one; otherwise named, as today).
- **Grain and vignette** (session 13), in millimetres on the print.
- **Sharpening and noise reduction** from a preset, through the export's sharpening (`sharpen.rs`) and a noise reduction measured first.
- **Older presets** (process before 2012): their sliders converted as Lightroom converts them, measured.
- **Not in v0.14.1:** lens corrections stay with [FR-085](#fr-085--lens-distortion-correction) in v0.15.0 (owner, 2026-11-09): Adobe's lens profiles cannot be bundled and lensfun's library and data are not permissively licensed, so they come from the corrections cameras write into their RAW files.
- **Rust first (ADR-0044):** each tool in the core, on rayon, within OPTIMIZATION §3's budgets set before its code; the spatial ones before or after the look's table as Dehaze is, chosen so the GPU preview keeps Rust's order.
- **Learn from RawTherapee and darktable first** (owner, 2026-11-09: "what is too hard, see how RawTherapee handles it; Fotollection is a marathon"): before each hard tool (local highlights and shadows, clarity and texture, noise reduction, grain, older presets' conversion), read how RawTherapee and darktable solve it: their manuals (RawPedia, darktable's user manual), their discussions and the papers they cite. Write what was learnt, the candidates and why, in RESULT before the code, then build our own in Rust and measure it against theirs and Lightroom's. **Their code is never copied or translated:** both are GPL, and a copy would close the relicensing option of [ADR-0008](adr/0008-license-gpl3-with-relicensing-option.md); methods and published papers are free to learn from.

### FR-119 — Smooth on ARM, and RISC-V tried

**P2 (planned, Windows on ARM first in v0.15.x, RISC-V later and experimental; owner's request 2026-11-09: "how does Fotollection run smoothly on ARM? think about RISC-V too").** Every pixel of Fotollection is portable Rust on rayon (ADR-0044), with no code written for one processor, so it already builds for ARM64 (Apple Silicon in the universal `.dmg`, Linux ARM64 AppImage and `.deb`). Smooth means the budgets of OPTIMIZATION §3 met on each, measured, never assumed:

- **Windows on ARM (Snapdragon X), first:** today only an x86_64 build exists, so those machines run it through Windows' emulation. Add `aarch64-pc-windows-msvc` to `release.yml` (WebView2 has an ARM64 runtime; `oscodec`'s WIC calls and the `-setup.exe`/`.msi` checked on ARM64), a smoke test, and the update check's asset names (`updates.rs` knows the architecture already).
- **Measured on each ARM machine:** a Raspberry Pi 5 (Linux ARM64, 4–8 GB) as the low end, an M-series Mac, a Snapdragon laptop. The budgets of opening a photo, a slider step, a 24 MP export and a RAW develop, with `measure_*`; the GPU look (`GpuLook.worthIt`) on each GPU, the CPU path where it is not worth it.
- **What a measurement may lead to, never before:** the hot loops (the look's table, the demosaic, Dehaze, the resize) checked for NEON in the generated code, and given portable SIMD (`std::simd` once stable, or a permissive crate) only where a benchmark shows the gain; rayon pools sized by performance and efficiency cores on big.LITTLE chips if the efficiency cores are measured to slow a batch; memory budgets (ADR-0041) checked on a 4 GB Pi.
- **The dependencies' own processor code** (`fast_image_resize`, `zune-jpeg`, `jpeg-encoder`, `tract-linalg`, `ring`) listed with what each does on ARM64 and RISC-V, from their source, before relying on it.
- **RISC-V (riscv64gc Linux), experimental:** Rust builds it (tier 2), `ring` and the pure-Rust decoders support it, WebKitGTK ships on Debian and Ubuntu riscv64. No hosted runner exists, so a cross build (`cross` or a riscv64 container) and a smoke test under emulation; then one real board (a SpacemiT K1 with RVV 1.0, or a VisionFive 2) to measure. Its vector extension is not in the baseline, so the first build is scalar; vectors only by measured runtime detection. Shipped only as "experimental" until a real board meets the budgets.
- **Docs when built:** NFR-007, RELEASE.md §2 and §6, the website's systems table and download buttons, an ADR for each new platform (DOCS_GUIDE, "A new platform").

### FR-120 — Turn the print size

**P2 (built for v0.14.1, 2026-11-11; owner's request 2026-11-10, a portrait night photo on a 5 × 15 strip).** Today a print size takes the photo's own orientation: a portrait photo prints on an upright 5 × 15, and nothing lays the strip flat. A **turn** button beside the print size (and a key) swaps it to landscape or back, so a portrait photo can be cropped across a 15 × 5 strip, a 15 × 10 or any size, and the crop editor opens on the turned frame.

- **Per photo, as the crop is:** the turn is kept with the photo's print settings (project schema, migration from "follow the photo"), the default stays "follow the photo"; *Apply to all* as the other print settings.
- **Everywhere the print size shows:** the preview's Print tab, the gallery's Print tab for the ticked photos, Simple mode, the size advice (DPI and the verdict read on the turned size) and the export.
- **Collage mode** keeps its own *Auto turn* (FR-035); a sheet's size turns there as it does today.
- **Rust first (ADR-0044):** the plan (`plan.rs`) owns the orientation, the interface only asks; tests `fr_120_*` that a turned 5 × 15 gives a 15 × 5 crop of a portrait photo at the same DPI rules.
- **As built (2026-11-11):** `PhotoEdits.print_turned` (project v13), `PlanInput.turned` read by `plan_content` and, in the frame's Crop mode, by its layout; `can_turn` refuses square sizes, *Original* and the frame's Fit; `effective_dpi_turned` gives the verdict and the size advice of the crop that prints. `set_print_orientation(ids, landscape)` lays each photo upright or flat in one undo step (*Turn print size* in History); a paste of the crop carries it, *Reset* clears it. *Orientation* (*Upright* / *Flat*, `PrintOrientationField`) sits under the size in the preview's Print tab, Simple mode and the gallery's Print tab (the photo picked, or the ticked ones); **O** in the preview and Simple mode. A 3456 × 4608 photo on 5 × 15: upright 1536 × 4608 px, flat 3456 × 1152 px (585 DPI).

### FR-121 — A small "fotollection.org" on the print

**P2 (built for v0.14.1, 2026-11-11; owner's decision 2026-11-10: "from the next version a small Fotollection in a corner of the photo, turned off in Settings").** Each print carries a small, quiet credit, so a photo passed round or on a fridge tells where it was made: **on by default**, turned off with one switch.

- **What and where:** the words `fotollection.org` (or the mark and the name), in a light grey at a small size measured in millimetres on the print so it reads the same at 10 × 15 and 20 × 30, in a corner of the white border or the info frame when there is one, on a postcard's back beside the stamp box; on the photo itself only as a last place, in the corner away from the date stamp, never over a face or the stamp.
- **Off in one place:** *Settings → General* (or *Print*), a switch "Print a small fotollection.org", kept per machine; nothing else changes when it is off. No reminder, note or link elsewhere (owner, 2026-11-10: it would clutter Fotollection).
- **Only on prints:** never on files exported for sharing online; the manual mentions the switch.
- **Rust first (ADR-0044):** drawn in `render.rs` with the stamp's text path (`stamp.rs`, the bundled fonts), at the print's DPI; tests `fr_121_*` that it is drawn when on, not when off, at the same physical size on two print sizes, and never under the date stamp.
- **As built (2026-11-11, `credit.rs`):** the words `fotollection.org` in Inter at an em of 1.6 mm on the print (letters about 1.2 mm tall). The plan places it (`RenderPlan::credit`), so the preview shows it as it prints: on the 3 mm border, centred under the photo and aligned to its right edge, in a grey from the paper's colour; in the info frame's bar under the logo; on a collage sheet once (never once a photo), under a postcard's stamp box or on the paper under the lower photo; otherwise in the photo's bottom corner away from the date stamp (then the other corners), half-clear white with a soft shadow, kept clear of the stamp and of any face the best-shot analysis found (when the photo was analysed and not turned since). Where nothing is free it is left out. The switch is *Settings → General → Prints*, kept per machine (settings v19).

### FR-122 — A warm gradient behind the app's chrome

**P2 (built for v0.14.1, 2026-11-11, the owner's choice C on the style board; owner's request the same day, from the landing page's keynote: "a soft brown gradient looks richer than one flat brown").** The app's own surfaces (the sidebar, the side panels, the title bar), in the dark theme first, take a soft gradient of the logo's wood, as the keynote's dark stage does, instead of one flat brown.

- **Photos stay on neutral ground** (STYLE_RULE §4, owner 2026-11-01): the stage, the grid and every surface right around a photo keep a chroma of 0.01 or less; only the chrome around them is warm.
- **Shown first:** two or three variants on the style board for the owner to choose, both themes, before any code; the depth tokens (FR-099) and the measures unchanged.
- **As built (choice C, one light across the window):** `chrome-lit` (`ui/src/index.css`) on the title bar, the folder list and the side panels: the sidebar's colour under a warm glow from the window's top left and a deeper one at its bottom right, `background-attachment: fixed` so the three surfaces share one light; both themes; `MEASURE=1` unchanged in EN light and VI dark.

## 7. Non-functional requirements

Performance budgets are measured as defined in [OPTIMIZATION.md](OPTIMIZATION.md). The reference machines are listed there.

### NFR-001 — Library load time

Opening a folder of 1,000 JPEGs shows the first screen of thumbnails in **under 1 s**. All thumbnails are ready in **under 10 s**.

### NFR-002 — Preview latency

A change to stamp or crop settings re-renders the visible preview in **under 50 ms** (p95).

### NFR-003 — Export throughput

Export (decode → crop → stamp → encode → write) runs at **8 or more 24 MP JPEGs per second** on an 8-core reference machine.

### NFR-004 — Memory ceiling

Peak resident memory during export stays **at or below 2 GB**, regardless of batch size. The preview's own caches are sized apart, to a quarter of the machine's RAM ([FR-091](#fr-091--instant-previews-standard-previews-on-disk-and-memory-used-well), owner, 2026-11-05).

### NFR-005 — UI responsiveness

Heavy work never runs on the UI thread. Input-to-feedback latency is **under 100 ms** during an export.

### NFR-006 — Offline and zero telemetry

The app works with no network connection. It makes no telemetry or crash-reporting requests, ever. The only request it can make is the update check: when the user clicks *Check for updates* ([FR-061](#fr-061--check-for-updates)), and, from v0.13.0, once at start unless *Check for updates when the app starts* is turned off ([FR-103](#fr-103--check-for-updates-at-start-never-download-unasked); on by default, the owner's decision of 2026-11-06 while the app still has faults). It reads GitHub's latest release and downloads nothing by itself. Offline, nothing is shown and nothing fails.

### NFR-007 — Platforms

The app supports macOS 11 or later (Apple Silicon and Intel, shipped as a universal binary), Windows 10/11 x64 (on ARM through Windows' emulation until [FR-119](#fr-119--smooth-on-arm-and-risc-v-tried) builds ARM64), and, since v0.8.2, 64-bit Linux with glibc 2.35 or later on x86_64 and ARM64, as an AppImage and a `.deb` ([ADR-0034](adr/0034-linux-lean.md)). Linux is the lean build: no camera capture ([FR-063](#fr-063--tethered-capture-over-usb)) and no HEIC yet ([FR-068](#fr-068--heic-on-linux-through-the-systems-libheif)). Every release proves the Linux packages start, on Ubuntu 22.04 and Debian 12, since the owner has no Linux machine. On Linux a Flatpak for Flathub is proposed ([ADR-0050](adr/0050-flatpak-on-flathub.md)), built from source on the GNOME runtime; in it the app never updates itself.

### NFR-008 — Install size

The installer is **at most 150 MB** per platform (owner, 2026-11-05: Fotollection has become a semi-professional app, with its own RAW develop and, next, a GPU develop; the budget was 25 MB until then, 30 MB proposed by ADR-0040). The AppImage, which carries its own libraries, is held to the same 150 MB.

### NFR-009 — Colour fidelity

The output either keeps the source ICC profile or, when sRGB conversion is on ([FR-027](#fr-027--srgb-conversion-for-labs)), converts it exactly. The app converts with `qcms`; the result is within ΔE₂₀₀₀ < 1 of an independent reference transform (`lcms2`, used only to check, never shipped). Without conversion, a pixel outside the stamp area differs from a reference decode by at most 1 per channel, apart from JPEG re-encoding error at the configured quality.

### NFR-010 — Cold start

The app goes from launch to an interactive window in **under 1.5 s**. Measured 2026-10-25 on REF-MAC with the release build (`scripts/measure-startup.sh`, milestones printed only when `FOTOLLECTION_TIMING` is set): Rust ready in 0.21–0.28 s, page loaded in 0.31–0.40 s and the first thumbnail in 0.50–0.58 s on later launches. The very first launch after install or a new build took 2.7 s, 2.1 s of it the system web view starting for the first time, with the machine deep in swap; the start-up screen ([FR-060](#fr-060--start-up-screen)) covers that time. To re-measure on a cleaner machine and on Windows.

### NFR-011 — Robustness

No input file, however malformed, can crash the app or hang the batch. A bad file only fails that one photo. Beyond input files (owner, 2026-11-09, [ADR-0048](adr/0048-one-owner-per-service-failures-stay-local.md)): no failing part, a decoder, a command, a background job, a lock or a card of the interface, takes the app down or leaves it stuck; it degrades alone, tells the user, and is logged with its `ERR-` code.

### NFR-012 — Culling navigation

In the loupe view, moving to the next or previous photo shows a sharp preview in **under 100 ms** (p95). Neighbouring photos are prefetched. Since [FR-091](#fr-091--instant-previews-standard-previews-on-disk-and-memory-used-well), the same holds for any photo once its standard preview is made.

### NFR-013 — Project safety

Edits reach the project file within **5 s**, and writes are atomic, so a crash or power loss never corrupts the project. Measured 2026-10-25 (`examples/measure_project.rs`): a full save takes 10 ms for 5,035 edited photos and 70 ms for 70,000, the library being locked 21 ms of it; the autosave runs at most once a second. **Planned:** an append-only edit journal (each undoable step written at once, the full snapshot every 30 s), so a crash loses at most one step and the save cost no longer grows with the library; a new ADR replaces that part of [ADR-0010](adr/0010-project-file-format.md). The file has a `schema_version` and is migrated forward automatically. Older app versions refuse to open newer files rather than damage them.

### NFR-014 — Best-shot analysis speed

Best-shot analysis ([FR-041](#fr-041--best-shot-suggestions)) takes **≤ 20 ms per photo without faces and ≤ 40 ms with faces** on REF-MAC, running in parallel on the background pool. It never blocks the grid or the preview. The budget is benchmarked before the feature ships.

### NFR-015 — Simple mode speed

In Simple mode ([FR-048](#fr-048--simple-collage-and-professional-modes)), export of 16–24 MP JPEGs to 10×15 at 300 DPI reaches **at least 3× the Professional throughput** on REF-MAC (target ≥ 50 photos/s), with peak memory still within [NFR-004](#nfr-004--memory-ceiling). The first screen of a 1,000-photo folder appears in **under 0.5 s**. Budgets are confirmed by benchmark before the mode ships.

### NFR-016 — Open to first photo

Opening a photo from Finder or Explorer ("Open with", [FR-054](#fr-054--open-photos-with-fotollection)) shows it in the preview in **under 1 s** on REF-MAC, after the first launch. Measured end to end on 2026-10-25 (`open -a` on the release bundle): 0.70 s for a JPEG whose folder is already a tab; 1.25 s when the folder is new, because it is imported before the photo shows (to improve); a RAW takes 1.4–2.5 s, its develop included ([NFR-017](#nfr-017--camera-raw-speed)).

### NFR-017 — Camera RAW speed

For camera RAW ([FR-016](#fr-016--raw-input)), on REF-MAC: a thumbnail or the input of best-shot analysis comes from the file's embedded preview in **under 100 ms**; the first print preview of a 16–31 MP RAW, which develops it, appears in **under 2.5 s**, and later changes re-render from the cached copy within [NFR-002](#nfr-002--preview-latency); exporting RAW keeps peak memory within [NFR-004](#nfr-004--memory-ceiling) by developing only a few at a time. Measured on nine CC0 samples before implementation ([RESULT.md §12](RESULT.md#12-camera-raw-v070)): preview 9–80 ms, develop 0.39–2.17 s, up to 1.29 GB for one 31 MP develop.

### NFR-018 — Shot to screen

For tethered capture ([FR-063](#fr-063--tethered-capture-over-usb)) over USB, measured in the app with the Sony a6400 on Windows (and on REF-MAC once capture runs on macOS): from the shutter (the Capture button or the camera's own) to the photo shown in the preview. Target to be set from the Step 0 measurements on a GX9 and an a6400 ([cameras/README.md](cameras/README.md)); expected about **2 s for JPEG**, RAW within [NFR-017](#nfr-017--camera-raw-speed) after it arrives.

### NFR-019 — Live view

For [FR-062](#fr-062--live-view-from-a-camera) over USB: at least **15 frames per second** with **under 200 ms** from the scene to the screen, where the camera allows it. To be confirmed by the Step 0 measurements.

### NFR-020 — Place inference speed

Borrowing places for camera photos from another device's photos of the same scene ([FR-031](#fr-031--borrow-gps-from-a-phone-or-gpx-track)) runs in the background and takes **at most 2 s per 1,000 camera photos** on REF-MAC, without slowing browsing ([NFR-005](#nfr-005--ui-responsiveness)). Measured before building: 1.24 ms per camera photo on all cores ([RESULT.md §14](RESULT.md#14-places-and-time-v080-measured-first)); as built, 54 ms per 1,000 when the scenes match and 1.11 s in the worst case ([RESULT.md §17](RESULT.md#17-borrowed-places-v080)).

### NFR-021 — Collage sheet speed

A collage sheet ([FR-035](#fr-035--multi-photo-layouts)) previews in **at most twice** the single-photo budget of [NFR-002](#nfr-002--preview-latency), exports in **no more time than its two photos exported alone**, and a screen of sheets in the Collage grid (ten) shows a change of setting in **under 100 ms** on REF-MAC, with peak memory within [NFR-004](#nfr-004--memory-ceiling). The single-photo times are measured first, and the budget is confirmed by benchmark before the mode ships.

## 8. Out of scope

The following are out of scope (for now):

- Cloud sync, accounts, and any kind of server backend.
- General photo editing beyond the global colour adjustments of [FR-079](#fr-079--colour-adjustments): local adjustments, masks, a full HSL matrix, a free curve editor, retouching and RAW development controls ([ADR-0011](adr/0011-photo-style-looks.md), [ADR-0038](adr/0038-moderate-colour-editor-on-one-lut-engine.md)).
- Writing ratings, flags or edits back into source files. Sources stay read-only. XMP sidecar files next to photos are planned as an opt-in ([FR-058](#fr-058--lightroom-compatibility-through-xmp)).
- Mobile apps.
- Video.

## 9. Success metrics

- A 500-photo trip goes from folder to print-ready export in **under 5 minutes** of user time.
- Zero reports of photos with incorrect rotation or cut-off stamps in exported sets.
- All NFR budgets are green in CI benchmarks on every release.
