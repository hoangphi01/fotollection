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
2. **Cull:** step through the photos with the keyboard, rate and flag them, and filter down to the set to print ([FR-022](#fr-022--culling-rate-flag-filter)). Work is saved automatically to a project file ([FR-023](#fr-023--projects-autosave-and-undo)).
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
| `15x15` | 15×15 | 6×6 | 1:1 | Square |
| `5x15` | 5×15 | 2×6 | 1:3 | Photo strip |

Target labs: CEWE, Rossmann, dm, Boots Photo, Snapfish, FreePrints, Photobox.

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
| `15x15` | 1772×1772 | 1181×1181 | 886×886 |
| `5x15` | 591×1772 | 394×1181 | 295×886 |

## 6. Functional requirements

Priority: **P0** = MVP blocker · **P1** = MVP desirable · **P2** = later.
The phase assignment for each requirement lives in [ROADMAP.md](ROADMAP.md), and its progress in [STATUS.md](STATUS.md).

### FR-001 — Import photos

**P0.** Import photos from a folder picker (recursive), drag and drop of files and folders, or a file picker.

- Accepts JPEG, HEIC/HEIF and PNG, detected by magic bytes rather than file extension.
- Unsupported or invalid files are listed with a reason. They never abort the import.
- Importing does not modify source files. The app opens sources read-only.

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

- Configurable format (presets plus a custom pattern), font (bundled fonts), size relative to the short edge, colour, opacity, optional shadow or outline, and position (each of the 4 corners, with a margin).
- Rendered with anti-aliasing at output resolution and placed inside the final crop area, so the stamp is never cropped away.

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
  - **Edge labels:** the top edge shows the printed width and the left edge the printed height (cm, or inches for sizes entered in inches), each with its pixel count. A centre pill shows the **aspect ratio** (`2:3`, `1:1`, or `1:1.44` for uneven sizes), the print size and the **live DPI** with its quality band ([FR-026](#fr-026--resolution-warning)). When a frame is on, the sizes are those of the photo area.
  - The crop is stored as a centre and zoom, so it survives changes of print size ([FR-025](#fr-025--mixed-print-sizes-per-order)).
  - It is undoable ([FR-023](#fr-023--projects-autosave-and-undo)).
- Offers a "fit with border" (letterbox) alternative to cropping.

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

**P2.** Import camera RAW files (starting with Panasonic RW2), developing them with neutral defaults or using the embedded JPEG preview.

### FR-017 — Photo book layout preview

**P2.** Preview how the selection lays out on album pages.

### FR-018 — QR code on photo back

**P2.** Generate a printable QR sheet that links each print to its digital original.

### FR-019 — Print service upload

**P2.** Opt-in upload to print lab APIs. This is the **only** feature that may use the network, and it must be explicitly enabled by the user.

### FR-020 — Localization

**P2.** UI available in English, German and Vietnamese.

### FR-021 — Auto-update

**P2.** Signed in-app updates from GitHub Releases. Checking for updates is opt-in, consistent with [NFR-006](#nfr-006--offline-and-zero-telemetry).

### FR-022 — Culling (rate, flag, filter)

**P0.** Narrow a large import down to the photos worth printing.

- Rate 0–5 stars and flag photos as *pick* or *reject*, using single-key shortcuts (`0`–`5`, `P`, `X`, `U` to unflag).
- A loupe view shows one photo large, with arrow keys moving to the next and previous photo ([NFR-012](#nfr-012--culling-navigation)).
- Filter by rating, flag, date, camera, "has GPS", and "resolution warning". Only the filtered set is exported.
- Ratings and flags live in the project file ([FR-023](#fr-023--projects-autosave-and-undo)). They are never written into source files.

### FR-023 — Projects, autosave and undo

**P0.** All work is kept in a project file (`.fotollection`): source folders, ratings, flags, crop frames, per-photo overrides and settings.

- Autosave is continuous ([NFR-013](#nfr-013--project-safety)). Reopening the app restores the last project.
- Undo and redo cover every user edit (ratings, flags, crop, overrides, settings), with at least 200 steps.
- If source files have moved, the user can relink the folder. Missing photos are marked, not dropped.
- **Session project:** until "Save as/Open" arrive, work goes to an implicit session project in the app data folder. On launch it is reopened, its folders are **re-scanned automatically** and the edits are re-applied by folder and path.

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

### FR-027 — sRGB conversion for labs

**P0.** On export, colours are converted from the embedded ICC profile (Display P3, Adobe RGB…) to **sRGB**, which is what most labs expect. The conversion is on by default and can be turned off per preset.

- The rendering intent is configurable: perceptual (default) or relative colorimetric with black-point compensation.
- The output embeds the sRGB profile.
- Photos without a profile are assumed to be sRGB.

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

**P1.** Add a free-text caption next to, or instead of, the date stamp. The caption can use the variables `{date}`, `{time}`, `{place}`, `{country}`, `{trip}`, `{camera}` and `{filename}`. Captions use the same layout rules as stamps (inside the safe area or on the border).

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

- **The whole print keeps the selected size.** The canvas has the print ratio, and the info bar is subtracted from it. The photo then either **crops to fill** the remaining area or **fits whole** with a thin margin.
- **Bar content:**
  - Left: the brand (typeset name, a user PNG, or nothing; [ADR-0014](adr/0014-user-supplied-brand-logos.md)).
  - Right-aligned lines: `Shot on <camera>`, then `<lens> <focal> <aperture> <shutter> <ISO>`, then `<photo style>`, then `<date> <place>`.
  - The bar is white or black. Text and logo stay inside the safe area ([FR-028](#fr-028--safe-area-and-borders)).
- **Choose what shows:** every item can be switched on or off. Lines with no data are dropped.
- **Edit per camera:** the camera name, brand text, lens, focal length, aperture and photo style can be overridden. This covers **manual lenses** that report nothing in EXIF. Overrides are remembered per camera across sessions ([DATABASE.md §6](DATABASE.md#6-app-settings)). Empty fields fall back to EXIF.
- The date stamp on the photo ([FR-004](#fr-004--date-stamp)) stays independent.

### FR-041 — Best-shot suggestions

**P1.** When several photos are taken of the same scene, the app suggests the best one and explains why. Nothing is rejected or deleted automatically: the user confirms. The method follows Lightroom Assisted Culling, Google Top Shot, Narrative and Aftershoot.

- **Grouping.** Photos are ordered by capture time, including `SubSecTimeOriginal`. A new group starts when:
  - the gap exceeds max(2 s, 4 × the median gap);
  - the camera or lens changes;
  - the focal length changes by more than 20 %;
  - GPS moves by more than 30 m.
- **Group confirmation.** A 64-bit dHash/pHash on T1 confirms each group: Hamming distance ≤ 10–12, compared with the group's representative photo so that groups do not chain. A group holds at most 30 photos.
- **Metrics.** Each metric is computed in one pass over the greyscale T2 preview:
  - Sharpness: variance of Laplacian and Tenengrad on a 16 × 12 tile grid. The median scores the whole photo; the 90–95th percentile, or the face tiles, score the subject.
  - Camera shake: structure-tensor coherence, relative to the group median. EXIF adds a hint when the shutter is slower than 1/(focal × crop factor).
  - Exposure: the clipped-highlight share (≥ 250), the crushed-shadow share, the median luminance against 118 (in EV), and RMS contrast.
  - Noise: the Immerkær estimate, skipping strong-gradient areas.
- **Faces and eyes.** Faces are detected with a permissively licensed model, and eye landmarks give the eye aspect ratio. An eye counts as closed when its ratio is below 0.2, or below 0.75 × the same person's maximum in the group. Face and eye regions also get their own sharpness score.
- **Score.** Each metric is normalized with a robust z-score (median/MAD) **within the group**; groups smaller than 4 use rank instead. The weights are:

  | Metric | Weight |
  |--------|-------:|
  | Subject sharpness | 0.35 |
  | Eyes open | 0.20 |
  | Shake | 0.15 |
  | Exposure | 0.15 |
  | Clipped highlights | 0.10 |
  | Noise | 0.05 |

  Hard faults, such as closed eyes or heavy blur, are shown as reasons. They are never applied silently.
- **Explanation.** The suggestion names the metrics that differ most from the runner-up, for example "Sharpest (+1.8σ)", "Eyes open" or "Fewer clipped highlights: 0.3 % vs 4 %". When the top two are within 0.5, it says **"Too close to call"**.
- **UI.**
  - A **Groups** view shows one strip per group, with a **"Best of N"** badge and its reasons.
  - A compare view shows the group's photos side by side, with a bar for each metric and faces and eyes marked.
  - **"Keep best"** picks the best photo and rejects the rest of the group, using [FR-022](#fr-022--culling-rate-flag-filter) flags. It can be undone.
- **Models.** Models ship inside the app, are checksum-verified, and are never downloaded. Only permissive weights are allowed. YuNet (MIT) and MediaPipe Face Landmarker (Apache-2.0) are to be verified; InsightFace/SCRFD and pyiqa models are excluded because of their non-commercial licenses. The runtime is pure-Rust `tract-onnx`. Both choices are recorded in an ADR before any code.
- **Cache.** Analysis results are derived data, so they live in the cache keyed by fingerprint, never in the project file ([DATABASE.md](DATABASE.md), rule N4).
- **Speed:** [NFR-014](#nfr-014--best-shot-analysis-speed).

### FR-042 — Custom fonts

**P1.** Import **OTF/TTF** fonts and choose them separately for the date stamp ([FR-004](#fr-004--date-stamp)) and for the info-frame text ([FR-040](#fr-040--exif-info-frame)).

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

**P2 (future).** Next to the batch settings, any setting can be overridden for one photo or a selection. For example, the date stamp can be turned off for photos that already have a timestamp burned in, while the rest of the batch keeps it. Overrides are stored sparsely per photo ([DATABASE.md §4](DATABASE.md#4-project-file-schema), `photos[].overrides`). The UI shows which photos deviate from the batch and offers *Reset to batch*. Until then, every setting applies to the whole batch.

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

The app goes from launch to an interactive window in **under 1.5 s**.

### NFR-011 — Robustness

No input file, however malformed, can crash the app or hang the batch. A bad file only fails that one photo.

### NFR-012 — Culling navigation

In the loupe view, moving to the next or previous photo shows a sharp preview in **under 100 ms** (p95). Neighbouring photos are prefetched.

### NFR-013 — Project safety

Edits reach the project file within **5 s**, and writes are atomic, so a crash or power loss never corrupts the project. The file has a `schema_version` and is migrated forward automatically. Older app versions refuse to open newer files rather than damage them.

### NFR-014 — Best-shot analysis speed

Best-shot analysis ([FR-041](#fr-041--best-shot-suggestions)) takes **≤ 20 ms per photo without faces and ≤ 40 ms with faces** on REF-MAC, running in parallel on the background pool. It never blocks the grid or the preview. The budget is benchmarked before the feature ships.

## 8. Out of scope

The following are out of scope (for now):

- Cloud sync, accounts, and any kind of server backend.
- General photo editing beyond the global Photo Style looks of [FR-039](#fr-039--photo-style-looks): local adjustments, masks, HSL/curve editors, retouching and RAW development controls ([ADR-0011](adr/0011-photo-style-looks.md)).
- Writing ratings, flags or edits back into source files or XMP sidecars. Sources stay read-only. Exporting XMP sidecars may be considered later.
- Mobile apps.
- Video.

## 9. Success metrics

- A 500-photo trip goes from folder to print-ready export in **under 5 minutes** of user time.
- Zero reports of photos with incorrect rotation or cut-off stamps in exported sets.
- All NFR budgets are green in CI benchmarks on every release.
