# Roadmap — Fotollection

> **Status:** v0.2, by quarter ([ADR-0053](adr/0053-roadmap-by-quarter.md)) · **Last reviewed:** 2026-11-19
>
> This file defines **when**: every requirement has one quarter, or *Shipped*. What each requirement means is defined in [PRD.md](PRD.md); its priority (P0–P3) stays there. Progress is tracked in [STATUS.md](STATUS.md), whose *When* column matches the quarter here. `scripts/check-docs.sh` enforces both.

## The goal: two years at least

Fotollection is kept alive as open source for at least two years, to 2028-11 (owner, 2026-11-16). It is built by one person at about 20 hours a week, so the plan is paced for that:

- **Maintenance before features.** The monthly maintenance below comes first; a month without it ships no feature.
- **One release at a time** ([RELEASE.md](RELEASE.md)), one theme per MINOR. After v1.0.0, one MINOR a month and PATCH releases when needed, security fixes from `release/1.x` with `minSafe` (`.github/update.json`).
- **A week of freeze before each MINOR**: fixes, docs, the screenshots and the owner's checks by hand.
- **Tired means fewer releases, never faster ones.** A quarter that slips moves its last release to the next quarter, written here the same day.
- **Every requirement stays** (owner, 2026-11-16): none is dropped, each has its quarter. A requirement added later goes to the end of a quarter, or to the next one.

## Maintenance, on a calendar

| When | What |
|------|------|
| Every month, code health | `node scripts/code-health.mjs --list`; a row in [CODE_HEALTH.md](CODE_HEALTH.md) §5 (production lines, the share in green files, red, orange, allows); the next stage of FR-138 or FR-139 picked |
| Every month, performance | the benchmarks run again on `REF-MAC` (`bench/run.sh` against the browser, the `measure_*` examples of OPTIMIZATION §3's budgets), each compared with the first measurements of 2026-10-23 on v0.1, kept as the baseline (owner, 2026-11-17); a result more than 10 % slower than the baseline is a bug fixed before the next release; one row in OPTIMIZATION §5 per run |
| Every month | `cargo update` and the npm lockfile, `cargo deny check`, `npm audit --omit=dev`, Tauri's patch releases, CI green on all three systems; the issues triaged (CONTRIBUTING §7) |
| Every Rust release (six weeks) | `rust-toolchain.toml` raised, clippy's new lints fixed |
| Every September | the new macOS (and its Xcode), WebView2 and WebKitGTK on the supported systems, GitHub's runner images (Ubuntu, Windows, macOS) before the old ones are removed |
| Every year | the domain fotollection.org renewed; the APT repository's signing key checked before it expires; the dependencies' licences reviewed (`cargo deny`, ADR-0008); this roadmap reviewed |
| Every release | RELEASE.md §6, the docs of DOCS_GUIDE.md |

## Quarters at a glance

| Quarter | Releases | Requirements |
|---------|----------|--------------|
| 2026-Q4 | v0.14.5 "Looks, part 3" (November) and v1.0.0 (December) | 38 |
| 2027-Q1 | v1.1 "Core 2" and v1.2 "Smart culling" | 11 |
| 2027-Q2 | v1.3 "The print run" and v1.4 "Cards and sheets" | 11 |
| 2027-Q3 | v1.5 "Places and trips" and v1.6 "Looks, part 4" | 8 |
| 2027-Q4 | v1.7 "Pro" | 4 |
| 2028-Q1 | v1.8 "Retouch" | 3 |
| 2028-Q2 | v1.9 "Community and platforms" | 6 |
| 2028-Q3 | v1.10 "Capture" | 3 |
| 2028-Q4 | v1.11 "Sharing and print services", and the two-year review | 3 |

## 2026-Q4 — v0.14.4 "Sorting" (November), v0.14.5 "Looks, part 3" and v1.0.0 (December)

v0.14.4 sorts a phone's camera roll (FR-141, [ADR-0054](adr/0054-sorting-metadata-first-then-pixels.md); owner, 2026-11-17: urgent): sources, stacks, events and trips (FR-012), duplicates (FR-015), suggested deletions after CLIP (FR-143, [ADR-0055](adr/0055-deleting-to-the-trash-on-the-users-word.md)), on the owner's 14,639 files. v0.14.5 finishes the looks: FR-112 closer to RawTherapee, FR-118 every preset setting, the measurements left from v0.14.0, FR-100's M3. **v1.0.0** closes the old Phase 1: every P0 FR Done and NFR-001 to NFR-013 within budget on `REF-MAC` and `REF-WIN` (STATUS, *Road to v1.0*), the owner's checks by hand, the real cards posted in Japan and Korea, fuzzing (S5) and the panic hook (S7).

| ID | Requirement |
|----|-------------|
| FR-112 | Highlight recovery for RAW, chosen and tuned by measurement against the current fade; v0.14.1 (moved from v0.14.0, owner, 2026-11-09), the gap to RawTherapee in v0.14.5 (moved from v0.14.3, 2026-11-15) |
| FR-118 | Every preset setting applied: clarity and texture (FR-115), local highlights, the parametric curve, white balance in kelvins against each photo's as shot, grain and vignette, sharpening and noise reduction, older presets; each tuned against Lightroom's exports; lens corrections stay with FR-085; v0.14.5 (moved from v0.14.3, 2026-11-15) (owner, 2026-11-09 and 13) |
| FR-114 | A look, LUT or preset given to many photos at once from Pro mode's gallery (a Colour tab beside Print): the ticked photos or every photo shown, an amount, photos with their own colour asked about once, one undo step; v0.14.0 (owner, 2026-11-08) |
| FR-115 | Local contrast in Pro mode: clarity, texture and dehaze, the spatial tools a colour table cannot hold, before FR-102's masks (a linear gradient first); Dehaze in v0.14.0 (2026-11-09, before the table); Rust after the table, GPU preview, chosen and tuned against Lightroom's renders; clarity and texture in v0.14.5 with FR-118 (owner, 2026-11-09 and 13) (owner, 2026-11-08: where Pro mode is heading) |
| FR-070 | Looks from LUT files (`.cube`, HaldCLUT, strength, per photo), LUMIX Lab style; v0.14.0, the highlight of Simple mode, [ADR-0036](adr/0036-colour-looks-cube-luts-first.md) |
| FR-089 | Lightroom presets: a user's own `.xmp` or `.lrtemplate` imported into a look, parsed safely, unsupported settings listed; v0.14.0, the rest in v0.14.5 (moved from v0.14.3, 2026-11-15) (FR-118) |
| FR-100 | Motion that feels like one app: shared timings, smooth transitions between views and layouts, lists that move instead of jumping, *Reduce motion* respected with a setting, 60 fps; M1–M2 in v0.14.0, M3 in v0.14.5 (moved from v0.14.3, 2026-11-15) (owner, 2026-11-08, 09 and 13, [ADR-0046](adr/0046-one-motion-system.md)) |
| FR-002 | Read photo metadata |
| FR-003 | Library grid |
| FR-004 | Date stamp |
| FR-005 | Location stamp (offline) |
| FR-006 | Crop to print ratio |
| FR-007 | Live preview |
| FR-008 | Original-quality export |
| FR-022 | Culling (rate, flag, filter) |
| FR-025 | Mixed print sizes per order |
| FR-026 | Resolution warning |
| FR-027 | sRGB conversion for labs |
| FR-028 | Safe area and borders |
| FR-030 | Camera clock offset |
| FR-032 | Manual date and place |
| FR-033 | Retro film date stamp and stamp presets |
| FR-040 | EXIF info frame |
| FR-041 | Best-shot suggestions |
| FR-048 | Simple, Collage and Professional modes |
| FR-050 | Print basket: the photos that will really be printed, exported to an extra folder |
| FR-052 | Appearance: System / Light / Dark theme, Windows title bar in the app's colours, app-drawn hover hints |
| FR-057 | Zoom in the preview: two-finger scroll, pinch, Ctrl/⌘ + − 0, drag to move, double-click for 200 % |
| FR-059 | Quick-preview: hold Space in select mode to look at a photo as it will print |
| FR-060 | Start-up screen: the logo and a loading bar until the last session is back |
| FR-065 | Border in Crop to fill: 3 mm in the frame colour inside the print size, instant-photo look at the exact kiosk size |
| FR-067 | 3 mm border on every print: every mode and frame style, inside the print size (extends FR-065) |
| FR-135 | A short landing page (six sections, results at full size, legible on a phone) and a `/features/` page; the makeover slider to the Wiki; v0.14.3 (owner, 2026-11-15) |
| FR-130 | The Japanese postcard: a はがき (100 × 148 mm, 「郵便はがき」 at the top, Japan Post's address side with its vermilion 3 + 4 code boxes) in Collage mode, the only card in Japanese and a switch in English and Vietnamese for a card sent to Japan, its layout on the style board first; v0.14.2 (owner, 2026-11-14) |
| FR-132 | The Korean postcard's address side (five code boxes, the 17 mm machine strip) on the 10 × 15 card, and Korea's print sizes (6 × 8, 5 × 7, 3 × 5, 8 × 10, 4 × 4 in); v0.14.2 (owner, 2026-11-15) |
| FR-133 | AddAssist, assisted address entry: a switch between typing by hand and the postal code filling what the country's bundled data knows (Japan, Korea, Hungary, the Czech Republic, Germany researched), name, street, number and an optional phone typed; shown on the website once it ships; v0.14.2 with FR-130 or later (owner, 2026-11-15) |
| FR-141 | Sorting a phone's camera roll: camera, app, screenshot, saved or received, edited, by metadata first and TinyCLIP-39M for the rest ([ADR-0056](adr/0056-tinyclip-39m-bundled-for-sorting.md)); Live Photo, burst and HEIC/JPG stacks; time with its source; events ([ADR-0054](adr/0054-sorting-metadata-first-then-pixels.md)) |
| FR-012 | Trip auto-grouping |
| FR-015 | Duplicate detection |
| FR-143 | Suggested deletions after sorting: too soft, enlarged from a small image (its real resolution measured), saved from the web, memes; each with its reasons, to the system's Trash on the user's word ([ADR-0055](adr/0055-deleting-to-the-trash-on-the-users-word.md)) |
| FR-145 | One time base: a UTC instant per photo from anchors in the photo, shown in the place's local time, a home zone chosen on the welcome and in Settings; the calendar and trips on one local day ([ADR-0058](adr/0058-time-model-utc-instant-zone-anchors.md); owner, 2026-11-19) |
| FR-146 | After an update, the welcome shows only the settings the old version lacked (the home time first); a fresh install the whole welcome; v0.14.4 (owner, 2026-11-20) |
| FR-117 | Right-click menus in more places (folders, photos, the filmstrip, collage slots, export presets), each holding only actions found elsewhere too, groups split by hairlines as Lightroom's; `MenuContent` and the style rules; the shared block and the trips' and looks' menus in v0.14.4, the rest one place at a time, each list chosen with the owner (owner, 2026-11-09; moved up 2026-11-18) |
| FR-138 | Code health, part 1: no source file over 1,500 production lines when v1.0.0 ships, in stages (shared shell helpers, `commands.rs` by concern, logic to the core, `preview.rs` to a folder, the tooling scripts) ([CODE_HEALTH.md](CODE_HEALTH.md)) |
| FR-083 | A personal photo calendar: a built-in calendar engine for any year, important days marked, a photo a month, PDF or PNG pages; on the collage sheet first, two months to a print; v0.14.5 (moved from v0.14.3, 2026-11-15) (moved from v0.14.2, 2026-11-15; owner, 2026-11-14) |

## 2027-Q1 — v1.1 "Core 2" and v1.2 "Smart culling"

v1.1 is the release once called v0.15.0: X-Trans, the GPU develop, the DNG kinds, a linear Rec.2020 working space, lens correction, the 32-bit lossless path and right-click menus; with it the distribution steps deferred on 2026-11-11 (notarization, SignPath, Flathub). v1.2 exports by trip, swipes through photos and watches a folder (trips and duplicates moved to v0.14.4).

| ID | Requirement |
|----|-------------|
| FR-072 | Our own develop core: sensor kind first, RCD and Markesteijn, Gr/Gb, false colour, highlights, linear working space, colour management everywhere; Bayer develop, neutral colour and the RAW sliders shipped in v0.10.0, X-Trans, GPU and DNG kinds in v1.1, [ADR-0037](adr/0037-our-own-develop-core.md) |
| FR-016 | RAW input |
| FR-085 | Lens distortion correction in our own develop (Panasonic and Olympus maker data, DNG `WarpRectilinear`), so ImageIO is no longer first for RW2 and ORF on macOS |
| FR-119 | Smooth on ARM, RISC-V tried: a Windows ARM64 build first (today emulated), every budget measured on a Raspberry Pi 5, an M-series Mac and a Snapdragon laptop, SIMD only where measured; riscv64 Linux cross-built and smoke-tested, experimental until a real board meets the budgets; v1.1, RISC-V later (owner, 2026-11-09) |
| FR-111 | Lossless export in Pro mode: TIFF 16-bit by default or 32-bit float, through a 32-bit float path; v1.1 (owner, 2026-11-09) |
| FR-014 | Export by trip |
| FR-046 | Swipe culling (full-screen ↑ / ↓ print list) |
| FR-064 | Watch a folder: a photo written into it by a maker's tether app opens at once |
| FR-139 | Code health, part 2: the orange files split or shared in stages (camera to `oscamera`, one text engine for the cards, `sheet`, `postcard`, `library`, the rest), the `too_many_lines` allows at most 10 ([CODE_HEALTH.md](CODE_HEALTH.md)) |
| FR-142 | Screenshots told apart by what they show (transfer confirmations, orders, chats, posts), CLIP first then text read on the device and never stored |

## 2027-Q2 — v1.3 "The print run" and v1.4 "Cards and sheets"

v1.3: copies, lab and print presets, contact sheets, ID photos, output sharpening. v1.4: the Polaroid frame, the calendar, the occasions' greetings, postcard backs, more layouts.

| ID | Requirement |
|----|-------------|
| FR-024 | Copies and order summary |
| FR-034 | Lab presets |
| FR-038 | Print presets |
| FR-037 | Contact sheet |
| FR-113 | ID photos: one photo cut to a document's size (35 × 45, 30 × 40, 2 × 2 in…) and repeated on one print with cut marks, at true size; v1.3 (2027-Q2), after the colour engine (looks, LUTs, Lightroom presets) and v1.1's 32-bit export (owner, 2026-11-07; moved with FR-111, 2026-11-09) |
| FR-029 | Output sharpening |
| FR-106 | Output sharpening for screen, matte or glossy paper, measured first |
| FR-126 | A Polaroid frame for two-photo collage sheets: a white instant-film frame, its deeper bottom strip for a typed note or one written by hand; never on a postcard; reuses the info frame and the postcard's text; v0.14.5 (moved from v0.14.3, 2026-11-15) (owner, 2026-11-12 and 13) |
| FR-131 | Greetings for the occasion on one postcard frame: 年賀, 寒中, 暑中, 残暑, 喪中 in Japanese, 설날, 어버이날, 스승의 날, the year's end in Korean, each its words and a rule, never a template of its own; v0.14.5 (moved from v0.14.3, 2026-11-15) after FR-130 or later (owner, 2026-11-15) |
| FR-074 | Postcard backs: a background colour or a pattern (airmail stripes, a grid, a texture) behind the lines |
| FR-035 | Multi-photo layouts (2-up collage in v0.9.0; 4-up and strips later) |

## 2027-Q3 — v1.5 "Places and trips" and v1.6 "Looks, part 4"

v1.5: places from a phone or a GPX track, the trip journal, landmarks. v1.6: film looks and what remains of the colour cards.

| ID | Requirement |
|----|-------------|
| FR-031 | Borrow GPS from a phone or GPX track |
| FR-066 | Borrow places from a GPX track: interpolate each photo's time along the track (split out of FR-031) |
| FR-049 | Trip journal: an offline report of the trip drafted from the album (days, places, photo counts, cameras, focal-length range), with a journal entry written by every editing session (like the instax app) |
| FR-140 | Film scans recognised from a bundled list of scanners (Noritsu, Frontier, Coolscan, Flextight…): the date is marked as the scan's and still printed, a field to type the shot's date for the roll (FR-032) |
| FR-051 | Landmark recognition: suggest a place from what the photo shows (offline, optional model pack) |
| FR-047 | Film looks (Fujifilm-style film simulations, own names and LUTs) |
| FR-039 | Photo Style looks (merged into FR-079, 2026-11-05) |
| FR-079 | Colour adjustments, moderate: Looks, Faves and Tools cards (tone, colour, style, effects), looks with an amount, per photo; Faves and Tools shipped in v0.9.9, the Looks card in v0.14.0, [ADR-0038](adr/0038-moderate-colour-editor-on-one-lut-engine.md) |
| FR-036 | Custom captions |

## 2027-Q4 — v1.7 "Pro"

Project history, XMP written back, per-photo overrides, cards in one's own order.

| ID | Requirement |
|----|-------------|
| FR-094 | Project history without nagging: dated copies of each project and of the settings kept automatically (seven days, four weeks), restored from the Settings window as a new document; version to be chosen |
| FR-058 | Lightroom compatibility: read XMP sidecars on import, write `.xmp` as an opt-in, photos never touched |
| FR-044 | Per-photo overrides |
| FR-080 | The panel's cards in an order of one's own, the same in the gallery and the previews, with *Reset order* |
| FR-144 | Vertical straighten (*Cân dọc*): converging verticals corrected by a perspective warp beside the straighten slider, as Lightroom's Vertical (owner, 2026-11-19) |

## 2028-Q1 — v1.8 "Retouch"

Healing, masks, a face-aware crop.

| ID | Requirement |
|----|-------------|
| FR-101 | Healing: spot removal, heal and clone, as Lightroom's Remove tool; a hidden tab of the inspector until built |
| FR-102 | Masking: linear, radial and brush masks with their own sliders, later on-device subject and sky, after FR-115 and the linear gradient first; needs a local layer beside the look's table (ADR-0038 amended) |
| FR-013 | Face-aware smart crop |

## 2028-Q2 — v1.9 "Community and platforms"

Language packs anyone can add, HEIC on Linux, the welcome's card picker, Open with and updates finished.

| ID | Requirement |
|----|-------------|
| FR-084 | Language packs anyone can add: a `.fotolang` file in the app's i18next layout, checked in Rust and by `check-i18n`, loaded from the Settings window or shipped through a pull request; carried by `.fotosettings` |
| FR-068 | HEIC on Linux through the libheif the distribution installs, loaded at run time; no bundled HEVC decoder (after v0.9.0, [ADR-0034](adr/0034-linux-lean.md)) |
| FR-054 | Open photos with Fotollection from the system's "Open with" menu, as a photo viewer |
| FR-061 | Check for updates: a button that asks GitHub once, then opens the installer in the browser |
| FR-021 | Manual updates (no in-app updater, ADR-0021) |
| FR-081 | A welcome screen on the first launch: the welcome shipped as FR-109 (v0.13.1); left, a page to pick the cards one likes, Pinterest style, they come first (with FR-080); *Skip* |

## 2028-Q3 — v1.10 "Capture"

Live view and tethered capture on more cameras and on macOS, the photobooth.

| ID | Requirement |
|----|-------------|
| FR-062 | Live view from a camera plugged in by cable (own PTP driver, UVC webcam or HDMI capture card), with the print ratio drawn over it |
| FR-063 | Tethered capture over USB: Capture in the app, the full-resolution photo opens at once; own PTP drivers per maker, no install ([ADR-0026](adr/0026-cameras-through-the-os-usb-stack.md)) |
| FR-108 | Photobooth with the computer's own camera (the Mac's webcam): countdown, a strip or sheet to print, offline |

## 2028-Q4 — v1.11 "Sharing and print services", and the two-year review

A photo book preview, a QR code on the back, upload to a print service (checked against the privacy rule first: nothing leaves the computer unless the user sends it). Then the roadmap is reviewed for the next two years.

| ID | Requirement |
|----|-------------|
| FR-017 | Photo book layout preview |
| FR-018 | QR code on photo back |
| FR-019 | Print service upload |

## Shipped

Built before 2026-11-16, from v0.1.0 to v0.14.3 ([CHANGELOG.md](../CHANGELOG.md)). Until then the roadmap ran in phases: Phase 0, the foundation (the workspace, CI, the benchmark harness, unsigned releases); Phase 1, the MVP; Phase 2, smart features; Phase 3, pro and growth. The open requirements of every phase are now in the quarters above.

### Release history (2026-10-24 to 2026-11-16)

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
| v0.9.9 | Colour, first cards (shipped 2026-11-05) | FR-079: Faves and Tools, a photo's own look, Before / After, a drag that follows the pointer; RAW badges and the RAW filter (FR-022, FR-078) | The Looks card and LUTs followed in v0.14.0, grain and vignette in v0.14.1 |
| v0.10.0 | Core (shipped 2026-11-05) | FR-072 (our own develop for every Bayer camera: rawler only unpacks; stuck pixels, green balance, a directional demosaic, a false-colour median; neutral RAW colour by default and the camera's look in one click), FR-088 (brightness, highlights, shadows and white balance on a RAW's own data, with headroom), damaged RAW files refused, [ADR-0037](adr/0037-our-own-develop-core.md), [ADR-0040](adr/0040-neutral-raw-colour-and-gpu-develop.md) | Re-ordered by the owner (2026-11-05): the develop core shipped before the Looks; RESULT §23–§28 |
| v0.11.0 | Instant previews (shipped 2026-11-05) | FR-091 (standard previews on disk after the thumbnails, a quarter of the RAM for the preview, prefetch, the photo on screen develops first: any photo opens in 27 ms, was 274–531 ms); colour cards in the previews only, every card flagged where it shows (FR-079); sliders take a near press and the arrow keys; RAW hangs fixed | The owner's request after v0.10.0 ("like Lightroom") |
| v0.11.1 | Memory, made safe (2026-11-06) | FR-091's audit: nothing kept across a project switch, the photo on screen never pushed out by a prefetch, one prefetch worker, no photo made twice, the disk limit during the previews' pass, a version for standard previews, background unless marked | The owner's choice: fix before v0.12.0 |
| v0.12.0 | Settings & Develop (shipped 2026-11-06) | FR-086 (one Settings window, two columns as on an iPad, the memory and the cache to manage, Default and Auto, memory given back when short), FR-087 (a collage sheet's pixels against its print size), FR-090 (a filmstrip in the photo's preview), FR-095 (a develop view a Lightroom user can read: one toolbar above the photo, Lightroom's keys, zoom in real pixels, a full-window layout with Navigator and History), FR-096 (before and after, Lightroom's four views); zoom of a turned photo fixed | The owner's requests of 2026-11-05 and 2026-11-06 |
| v0.13.0 | The new look (shipped 2026-11-06) | FR-099 (controls with depth, as Sentry's, bold; the light theme's shadows in dark teal), FR-092 and FR-093 (the export dialog as Lightroom's, the quality slider with measured badges), FR-104 and FR-105 (file names as Lightroom's templates, a file-size limit), FR-107 (folder tabs reordered by dragging), FR-103 (checking for updates at start), the Be Vietnam Pro font, one panel width, a toolbar that fits, `MEASURE` checks for text and rows that overflow | The owner's choice on the style board, 2026-11-06 |
| v0.13.1 | Develop, part 2 (shipped 2026-11-07) | FR-097 (copy and paste settings, Reset), FR-098 (histogram above the Navigator, clipping, white balance), FR-109 (a welcome screen on the first launch) | Moved from v0.12.0 when the owner released it as it stood; FR-092 and FR-093 moved on to v0.13.0 |
| v0.13.2 | Develop, part 3 (shipped 2026-11-07) | FR-098 in Collage mode (`J` and `W` on a sheet's picked photo), FR-110 (the manual inside the app, offline), the Performance page made visual (stat tiles and a chart opening each part, more diagrams, shorter prose) | The owner's choice of 2026-11-07, after v0.13.1 |
| v0.13.3 – v0.13.4 | Patches (shipped 2026-11-07 and 08) | The RAW tone curve without steps (ADR-0043, RESULT §33); security patches (minimum safe version 0.13.3) | — |
| v0.14.0 | Looks (shipped 2026-11-09) | FR-079 (the Looks card, built-in looks of our own, the Color Mixer), FR-070 (looks from `.cube` and HaldCLUT LUTs, the highlight of Simple mode: look, choose a colour, print), FR-089 (Lightroom presets, measured against Lightroom's renders), FR-114 (a look for many photos), FR-115's Dehaze, FR-096's press and hold for the original, FR-023's project settings of its own, NFR-011's containment, FR-116 and FR-082 (data on this computer, `.fotosettings`), *Presets & LUTs* in the develop view's left column, FR-100's motion tokens (M1–M2), [ADR-0038](adr/0038-moderate-colour-editor-on-one-lut-engine.md), [ADR-0048](adr/0048-one-owner-per-service-failures-stay-local.md) | Budgets: ≤ 5 ms preview, ≤ 60 ms 24 MP export whatever is set (`measure_look`) |
| v0.14.1 | Looks, part 2 (owner 2026-11-09: "cover and complete what the app reports missing from presets"; 2026-11-13: ships what is built, with Deutsch and Magyar, the rest in v0.14.2) | FR-127 and FR-123 (the app in Deutsch and Magyar), FR-124 (the language first on the welcome, the machine's language on the first launch), FR-125 (long words fit), FR-112 (highlight recovery for RAW, measured against the current fade), grain and vignette in millimetres on the print (session 13), FR-120 (turn the print size), FR-121 (a small "fotollection.org" on each print), FR-073's Postal use, FR-122 (the warm chrome), the Flatpak's metainfo to Flathub's quality guidelines (ADR-0050) | Each tool within OPTIMIZATION §3; every word fits in German and Hungarian (MEASURE) |
| v0.14.2 | 日本語 and 한국어 (owner, 2026-11-15: releases what is built) | FR-128 (the app in Japanese and Korean), FR-129 and FR-132's sizes (Japan's and Korea's print sizes), FR-130 (the はがき), FR-132 (the 우편엽서), FR-133 (AddAssist 1.0, Japan and Korea), the Japanese and Korean pages of the website with their own screenshots on their country's photos (PRD FR-130) | Japan Post's and Korea Post's rules each tested (FR-130, FR-132); a real card posted in each country before FR-130 and FR-132 are done |
| v0.14.3 | Postcards, part 2 (planned; owner, 2026-11-15: v0.14.3 releases what is built, the Looks items move to v0.14.5) | FR-133's AddAssist 1.1 (the address book, recent recipients, a sender per card, one recipient or message for many cards), FR-136 (the Postcard tab in three blocks), FR-137 (Looks in Collage mode), FR-124's flags and backdrop on the welcome, FR-135 (the short landing page and `/features/`, legible on a phone), FR-134 (text drawn through `skrifa`, the ttf-parser advisory closed) | The real cards posted in Japan and Korea; `check-lego` and `MEASURE=1`; the phone crops' text at least 11 px |

| ID | Requirement |
|----|-------------|
| FR-001 | Import photos |
| FR-009 | Export by date |
| FR-010 | Metadata privacy on export |
| FR-011 | Batch progress, cancel and recovery |
| FR-020 | Localization (English and Vietnamese; German later) |
| FR-023 | Projects, autosave and undo |
| FR-042 | Custom fonts |
| FR-043 | Custom print sizes |
| FR-045 | Original aspect ratio |
| FR-053 | Export for social media: Original / Facebook / Instagram / custom sizes, sRGB, EXIF with the app's name and never GPS |
| FR-055 | Export the photo open in the preview, with the same export controls as the panel |
| FR-056 | Folders as tabs: All photos, one folder or several ticked, exported together or separately |
| FR-069 | Report a problem: a red button in the app (*Settings → Report a problem* since v0.12.0) and the website's footer, to the GitHub issue forms (v0.8.3) |
| FR-071 | Image facts at import and in the inspector: chroma subsampling, colour space, sensor kind; shipped in v0.9.3, [ADR-0037](adr/0037-our-own-develop-core.md) |
| FR-073 | Postcard sheets in Collage mode: one photo and a postcard back (stamp box, writing lines, typed text); v0.9.4 |
| FR-075 | Share exports fitted whole into a ratio (1:1, 4:5, 3:2… or custom), the rest filled with white, black or a colour of one's own; v0.9.6 |
| FR-076 | A bundled handwriting face for postcards (Patrick Hand), strokes as thick as the print test asks; v0.9.7 |
| FR-077 | One address for many postcards: Use for N postcards, and the recent addresses of this machine; v0.9.7 |
| FR-078 | The photo at a glance at the top of the preview: camera, lens, exposure, place, date, size; v0.9.7 |
| FR-082 | The app's brain in one file: *Export / Import settings* as `.fotosettings` (every setting, the card order, looks and LUTs, fonts, logos, postcard addresses), to move to another computer; v0.14.0, with FR-116 |
| FR-086 | One Settings window: a gear beside the author's name opens every app setting in two columns as on an iPad (language, theme, `.fotosettings`, language packs, cache, updates, version and changelog), the preview's layout reused |
| FR-087 | How large a collage sheet prints: the sheet's pixels and a sharp / acceptable / soft verdict for the chosen size in Collage's *Print size* card |
| FR-088 | RAW edited in linear light: exposure, highlights, shadows, warmth and tint on the RAW's own data before its tone, with headroom above white; v0.10.0 |
| FR-090 | A filmstrip in the photo's preview (Pro mode), as Lightroom's: the view's thumbnails under the photo, skim and pick with one click |
| FR-091 | Instant previews like Lightroom: standard previews on disk after the thumbnails, a quarter of the machine's RAM for the preview, prefetch around the photo on screen, which always develops first |
| FR-092 | Export quality on a slider, as Lightroom's: the size estimate follows the thumb, and badges say whether the setting prints well; v0.13.0 |
| FR-093 | The export's options in one small dialog: a press on *Export* opens it, with *Don't ask again* after which *Export* exports at once with the saved choices, changed later in Settings; the same dialog in every mode and preview, the Settings window's size with Lightroom's presets; v0.13.0 |
| FR-095 | A develop view a Lightroom user can read: a full-window layout chosen in Settings (`F`), Navigator and History on the left, one toolbar above the photo, Lightroom's keys, zoom in real pixels, the inspector develop first; shipped in v0.12.0 |
| FR-096 | Before and after: `\` toggles, `Y` side by side, `⇧Y` split, zoomed together; shipped in v0.12.0; press and hold for the original in v0.14.0 |
| FR-097 | Copy and paste settings (⇧⌘C / ⇧⌘V) to the photo shown or every ticked photo, Reset (⇧⌘R), undo and redo buttons; v0.13.1 |
| FR-098 | Histogram, clipping warnings (`J`) and a white-balance picker (`W`); v0.13.1 |
| FR-099 | Controls with depth, as Sentry's: raised buttons with a lip, inputs set in, lifted cards; v0.13.0 (owner moved it forward, 2026-11-06) |
| FR-103 | Check for updates at start, on by default and turned off in Settings; check only, the user downloads; a popup, with a red warning below the minimum safe version carried in the release notes (`.github/update.json`); v0.13.0 |
| FR-104 | File names as Lightroom's templates (original, date, custom text and sequence) with a live example; v0.13.0 |
| FR-105 | Limit the file size, by bisection of the quality, measured first; v0.13.0 |
| FR-107 | Folder tabs reordered by dragging, as Chrome's tabs, kept with the project; v0.13.0 |
| FR-109 | A welcome screen on the first launch: five pages (welcome, modes, choose, make it yours, print), Next, a *Show at every start* tick and Start; v0.13.1 |
| FR-110 | The manual inside the app, offline, from the same USER_GUIDE as the site, in Settings and the Help menu; v0.13.2 |
| FR-116 | Data on this computer, in Settings: export and import settings (FR-082), what each store holds with its size and *Open folder*, clear what can be made again, reset settings, erase data; each row asks its store's owner (ADR-0048); v0.14.0 |
| FR-120 | Turn the print size: a button and a key lay a print size flat or upright for a photo, so a portrait photo can be cropped across a 15 × 5 strip; kept per photo, in every place the size shows; v0.14.1 (owner, 2026-11-10) |
| FR-121 | A small "fotollection.org" on each print, on by default, off with one switch in Settings; measured in millimetres, in the border, frame or a postcard's back first; never on shared exports; v0.14.1 (owner, 2026-11-10) |
| FR-122 | A warm gradient of the logo's wood behind the app's chrome (sidebar, panels, title bar), dark theme first, photos kept on neutral ground; variants on the style board first; v0.14.1 (owner, 2026-11-11) |
| FR-123 | Hungarian interface: every string, the menu and the errors in Magyar, the website's short manual, AI-translated with the website's words; LOCALIZATION.md the single source of truth; v0.14.1 (owner, 2026-11-12) |
| FR-124 | The welcome opens on a language page (a greeting through every language, the languages by their own names), and the first launch speaks the machine's language; v0.14.1 (owner, 2026-11-12) |
| FR-125 | Long words fit: a step smaller, then cut with the whole in a hint (`FitText` in every button); English always whole; dry checks of translations' widths and a German-length pseudo-locale; v0.14.1 (owner, 2026-11-12) |
| FR-127 | German interface: every string, the menu and the errors in Deutsch, the website's short manual with the app's labels, translated with the German website's words; v0.14.1 (owner, 2026-11-13) |
| FR-128 | Japanese and Korean interface: every string, the menu, the errors and a short manual in 日本語 and 한국어, each with a bundled Noto Sans JP or KR cut to its characters (built 2026-11-15); both in v0.14.2 with FR-130 (owner, 2026-11-14 and 15) |
| FR-129 | The print sizes of Japan's convenience-store kiosks (7-Eleven, Lawson, FamilyMart), L判, 2L判 and each chain's square (89 or 127 mm), as lab presets from each chain's own pages; v0.14.2 (owner, 2026-11-14) |
| FR-134 | Text drawn without an unmaintained font parser: `skrifa` in place of `ttf-parser` and `ab_glyph`, the same pixels, the RUSTSEC-2026-0192 ignore removed; v0.14.3 (owner, 2026-11-15) |
| FR-136 | The Postcard tab in three blocks: Sender (optional, folded), Recipient, Message; v0.14.3 (owner, 2026-11-15) |
| FR-137 | Looks, LUTs and presets in Collage mode: the Pro panel's Colour tab (`ApplyLookCard`) beside Print in the Collage panel, on the chosen or the ticked photos; v0.14.3 (owner, 2026-11-15) |

## Ideas (not yet requirements)

These ideas have no ID yet. Promote one to a PRD requirement before scheduling it.

- **Germany and Austria, the Czech Republic and Slovakia apart (owner, 2026-11-15; near future).** The website's Deutsch page serves DE/AT and its Čeština page CZ/SK (`regions`, `also` in `site/_data/languages.yml`); each country gets its own page, as each has its own AddAssist data, post and shops (FR-133), and Slovak its own words, a little different from Czech.
- **France as a market (owner, 2026-11-15; researched the same day).** No dm or Rossmann there: the website in French would name CEWE's kiosks (about 3,500, at E.Leclerc, Intermarché, U, Cora, Fnac), Kodak's at Carrefour, Fnac's own and Photomaton; the presets 10 × 15 (the everyday size on every chain's page; 11 × 15 not confirmed), 13 × 18, 11 × 17, 15 × 20, 20 × 30 and the squares, most already built in; AddAssist from La Poste's open postal data (FR-133); the postcard within La Poste's rules (FR-073). The website in French is done (2026-11-15); a French interface comes next.

- **The app grew, the speed held (owner, 2026-11-17; later).** The Wiki of the website shows the monthly measurements over time beside the app's size (features, lines of code, installer size): Fotollection grows while opening, previewing and exporting stay as fast, from the October baseline on.
- **Light leaks (owner, 2026-11-12; maybe).** A film camera's light leak as a look's last touch: a warm glow from one edge, of a few shapes, at an amount, never the same twice on a roll unless asked. Drawn in Rust at the print's millimetres, as grain and vignette are (session 13 of v0.14.1), so the preview and the print agree; a candidate for the looks, now that grain and vignette have shipped (v0.14.1).
- **A store of looks and frames, online (owner, 2026-11-10; later).** Looks and info frames by theme and by occasion (Christmas, Tết, a summer, a city), some free, some exclusive to a season, some sold; imported into Fotollection as a look or a frame is today (FR-079, FR-089, `.fotosettings` parts). Privacy first (NFR-006, CLAUDE.md rule 4): the core never goes online; the store is a page on the website, or a window in Fotollection that connects only when the user opens it, never in the background, nothing sent about the photos. Each item checked as an import is (bounded, in Rust); payment by a processor on the web (as Ko-fi, never in the core), the licence of each look stated (the creator's, no Adobe preset). Needs a decision on what is sold and by whom, and the relicensing question (ADR-0008) settled first.
- **Growth, without telemetry (owner, 2026-11-10, ideas kept from advice; each one the owner's choice before it is built):**
  - *Where we stand (2026-11-11, the numbers we have):* 201 visits to the website since counting began; 369 downloads in all (Windows 141, Linux 122, macOS 102); 5 stars; 8 people on the repository's page in 14 days, 3 of them sent by fotollection.org, 1 by Unikorn. Most releases show 7 or 8 downloads, about what the owner's own smoke tests and checks make; the peaks are v0.9.4 and v0.9.5 (about 80, when the VNLF App Explorer listed it) and v0.14.0 (37). So the real users are some tens, and **the problem is being found, not being kept**. Linux at a third of the downloads, far above its share of desktops, says the Vietnamese Linux community (VNLF, CaramOS) is the strongest channel we have. 30 releases in 18 days spread the few visitors thin: each MINOR release from now is also an announcement.
  - *The plan (owner, 2026-11-11, as marketing lead):* (1) be findable: the repository's description, homepage and topics (done 2026-11-11), a social preview image (`site/assets/img/og-en.jpg`, set by hand in the repository's settings), AlternativeTo, awesome lists and the AUR; Flathub (ADR-0050) and SignPath's free code signing in two or three months, once Fotollection has more users and is steadier (owner, 2026-11-11); (2) the two audiences that already answer: the Vietnamese at home and in Europe (VNLF, CaramOS, Unikorn, Facebook groups, Tết prints from early January) and kiosk printers in Germany, Czechia, Hungary and Poland (the cards at dm and Rossmann, pages for their searches, the web crop tool); (3) enthusiasts by the one thing they lack: LUMIX owners (LUMIX Lab's LUTs on a desktop, free), people leaving Lightroom's subscription, the open-source crowd (Show HN once the web tool exists); (4) every MINOR release one short post per channel, a before and after, in the owner's own words. Measured by `utm_source` on every link (the `/hi` short link) and downloads per release; targets for the next 30 days: 1,000 visits, 50 stars, 150 downloads of the next MINOR release.
  - *Each print a small advert:* decided as FR-121 (owner, 2026-11-10): on by default, off in Settings.
  - *The digicam and date-stamp trend:* 15-second videos (an iPhone photo → the retro date stamp → the print from dm) for TikTok and Reels.
  - *A small web tool as a door:* a page that crops a photo to dm's and Rossmann's print sizes in the browser, uploading nothing, ending with "500 photos at once? Get Fotollection"; it brings search traffic a landing page does not. It comes after v0.14.1 (owner, 2026-11-11): Fotollection's own crop and print-size code built to WebAssembly, files up to 25 MB and 60 MP, JPEG and PNG only.
  - *Print shops as partners:* a shop's own sizes and looks in Fotollection, the shop recommending it to its customers (a small studio first).
  - *Seasons:* "print the family's photos for Tết" for the Vietnamese community, "Erasmus memories" with ESN groups at the end of a term, Christmas prints from November.
  - *Where people look for software:* Flathub (the AppStream file is ready), winget, a Homebrew cask, the AUR, AlternativeTo as a Lightroom alternative, "awesome" lists on GitHub; each is also a link back.
    - *Listed (2026-11-14):* AlternativeTo (approved the same day; similar apps still to add on its page: darktable, digiKam, Google Photos…). *Reddit:* the owner's new account was suspended after two posts on its first day; one appeal, no new account; a later channel, after weeks of taking part.
    - *Search (2026-11-16):* a search for "fotollection" showed AlternativeTo, Facebook and GitHub but not fotollection.org: the home page was not yet indexed. The site was already crawlable (robots.txt, a sitemap of every language with hreflang, canonical, `SoftwareApplication`, a 64 px favicon); the home page now carries `WebSite` (`name`, `url`) so Google can show the site's name. ✓ Google Search Console set up by the owner the same day (its first data after a day). Left: the sitemap and the home pages' indexing requested there, then Bing Webmaster Tools (imported from Search Console).
    - *Submitted (2026-11-16, the README badge set to beta first):* winget ([#481571](https://github.com/microsoft/winget-pkgs/pull/481571)); the awesome lists that take an author's own app at 5 stars: awesome-mac in its five languages ([#4675](https://github.com/jaywcjlove/awesome-mac/pull/4675)), awesome-rust-apps ([#20](https://github.com/jaywcjlove/awesome-rust-apps/pull/20)), awesome-tauri-apps ([#64](https://github.com/chrysler-stone/awesome-tauri-apps/pull/64); tauri-apps/awesome-tauri no longer lists apps), awesome-linux-apps ([#106](https://github.com/cuongtk8/awesome-linux-apps/pull/106)), awesome-free-apps ([#526](https://github.com/Axorax/awesome-free-apps/pull/526)), open-source-mac-os-apps ([#2168](https://github.com/serhii-londar/open-source-mac-os-apps/pull/2168)). iCHAIT/awesome-macOS takes no pull requests from outside since 2026-10-21. *Later:* rust-unofficial/awesome-rust and 0PandaDEV/awesome-windows at 50 stars; pluja/awesome-privacy and Lissy93/awesome-privacy after the first stable release has aged 4–6 months (about 2027-05).
    - *Japan and Korea (researched 2026-11-16, free channels only):* Japan: 窓の杜 (an email in Japanese to the editors, Windows only, they choose), デジカメWatch (a press release by email), a technical write-up on Zenn or Qiita (a post about one's own work is allowed when it explains), Vector (author ID, then a hosted file, 2–3 weeks), フリーソフト100 (a form, Windows only). Korea: GeekNews *Show GN* (an account now, since new accounts wait before posting; the title "Fotollection - 핵심 기능"), a velog or tistory write-up, 클리앙 개발한당 after weeks of taking part (a first post that only promotes is deleted), camera cafés only by answering questions. Avoid: Clien's paid 직접홍보 board, Softonic (download wrappers), 価格.com boards for promotion, any first post that is only a link (the Reddit lesson).
    - *The Japanese press, later (owner, 2026-11-16: "once we are trusted and steady"; the /ja/ pages carry search meanwhile):* one email each, in Japanese, with a release as the news, never twice. Before it: a native speaker's review of the Japanese interface, of /ja/ and of the email (LOCALIZATION §1 still says AI); a real L判 print at a convenience-store kiosk and a はがき posted in Japan, or the email describes the layout without claiming it was proven. 窓の杜 (`mado-no-mori-info@impress.co.jp`): the Windows angle, the kiosk sizes, the date stamp, the はがき; デジカメWatch (`dc-watch@impress.co.jp`): the photographer's angle, LUMIX Lab LUTs on the desktop, RAW, L判 at the kiosk. Links only, no file attached; Japanese screenshots (`grid-ja`). The download page's first-launch steps (SmartScreen, Gatekeeper) are already in /ja/.
    - *Local catalogues, to submit later (owner, 2026-11-14; checked the same day), the download link always fotollection.org/download or GitHub Releases, never a file they host:* Germany: heise download (free developer account, form "Programm hochladen", weeks of review; CHIP has no public form and wraps downloads in its own installer, so skipped). Czechia: Slunečnice.cz and Stahuj.cz ("Přidat program", /pridani-programu/). Japan: Vector (author account, then the software; free software accepted, about a week of checks) and Mado no Mori (an email to the editors, `mado-no-mori-info@impress.co.jp`, once a Japanese reader has read /ja/). Hungary: no catalogue with a submission form found (szoftverbazis.hu is gone; letoltokozpont.hu has only an editors' contact); its large tech forums (Prohardver, Mobilarena) instead, rules to read first.
    - *Flathub, researched 2026-11-11 ([requirements](https://docs.flathub.org/docs/for-app-authors/requirements)):* built entirely from source with no network during the build (the Cargo and npm dependencies vendored with flatpak-builder-tools, Tauri on the GNOME runtime, which carries WebKitGTK 4.1); an ID on a domain the project controls, `org.fotollection.Fotollection`, verified by a token at `fotollection.org/.well-known/org.flathub.VerifiedApps.txt` (check that the Pages upload serves a dot-folder); static permissions minimal, portals for files; no self-update (the update check off when `FLATPAK_ID` is set). **Its generative-AI rule:** AI-written code in the app must be disclosed with its extent, reviewers may reject on it, the Flathub manifest must contain no AI-written or AI-assisted content, and the pull request, its messages and replies are written by the person, never by an agent. So the app side (metainfo, update check, portals) can be prepared here; the manifest and the submission are the owner's own work.
    - *Homebrew:* casks that fail Gatekeeper are disabled since September 2026 ([Homebrew 6.0.0](https://brew.sh/2026/06/11/homebrew-6.0.0/)); a cask waits for a notarized build (Apple Developer Program). *winget:* submitted 2026-11-16 (STATUS 6b), its validation passed with the unsigned installer; SmartScreen warns either way; free code signing for open source (SignPath Foundation, the policy page `fotollection.org/code-signing/` exists for it) opens winget and quiets SmartScreen.
  - *Communities by feature:* LUMIX owners (LUMIX Lab LUTs read on the desktop, free), Show HN and r/opensource (offline, GPL, no tracking), Vietnamese travel and printing groups; each release a short post with a before and after, not the changelog.
  - *Support by audience (2026-11-11, from advice):* more support follows more users, not a change of culture, so Flathub and the German pages come first. Each audience sees the channel it already uses: English and German readers Ko-fi (PayPal and cards) or GitHub Sponsors, Vietnamese readers VietQR (a bank app's QR is quicker for them than Ko-fi), Vietnamese in Europe being a group worth speaking to (EU incomes, a Vietnamese maker, Vietnamese bank apps). On Ko-fi, fixed amounts in euros named as coffees ("one coffee = 3 €") rather than a free amount. Fotollection's one quiet support line, if built, picks the channel by the interface's language.
  - *Support that gives back:* a postcard printed with Fotollection and a handwritten thank-you above an amount, a look named after a supporter (with their consent), supporters voting on the next feature; in Fotollection, one quiet line after a good moment ("Fotollection helped you print 100 photos ☕"), off for good with one click; what the money pays for said plainly (the domain, code signing). Never an advert, never a nag, never counted (no telemetry).
- **The website in German, Czech and Polish (owner, 2026-11-10: "later, how hard?").** Where dm and Rossmann print (Germany, Austria, Czechia, Poland, Hungary), pages for the searches people make ("dm Fotodruck richtig zuschneiden", "Datum auf Fotos drucken"). The effort, from today's site: (1) ✓ done 2026-11-11 ([ADR-0049](adr/0049-website-on-eleventy.md)): the site is built by Eleventy from one list of languages (`site/_data/languages.yml`, with Hungarian too, owner 2026-11-11), each page one template, the language button a menu, `scripts/check-site-i18n.mjs` refusing a published language that shows English; ✓ the sitemap with `hreflang`, `robots.txt`, the `latin-ext` fonts and the link previews per language; ✓ the landing and Download pages in Hungarian, German, Polish and Czech (2026-11-11, written by the maintainer's tools, a native reader welcome); ✓ the first-visit redirect by the browser's language or the menu's choice; ✓ the Performance page, Wiki, code signing policy and a short manual in those four languages, the story staying English and Vietnamese (owner, 2026-11-11); ✓ the keynote video under the hero, in every language (2026-11-11) by the browser's language; (2) each language is a block of about 440 strings in `i18n/<code>.yml`, its footnote lists and its pages under `/de/`, `/cs/`, `/pl/`: the translation is the real cost, done by a fluent speaker or checked by one, never machine-translated unread; (3) the manual (`USER_GUIDE`) stays English for a new language until it is translated, as `site-manual.sh` allows; (4) the app's own interface is separate: Magyar and Deutsch are built in since v0.14.1 (FR-123, FR-127), the Hungarian website since 2026-11-11, other languages later as FR-084's language packs.

- **Look back one trip at a time from the Photos library (macOS, PhotoKit, read-only).** Pick a trip by date or place; the app groups it, suggests the best shots, and puts the rejects in a "To delete" album the user empties in Photos. With iCloud Photos this frees space on the iPhone too, without the app ever deleting anything. Prompted by the owner's own library: about 70,000 photos on a 512 GB iPhone. Measure a 70,000-photo library first (NFR-001, NFR-014).
- **iPhone companion app**, only if the Mac flow proves itself: swipe culling on the way home and real deletion behind iOS's own confirmation. Needs the Apple Developer Program (99 USD/year, which would also notarize the Mac app), a native SwiftUI interface over the Rust core (UniFFI), and Core ML for the face models.
- **Support from abroad:** ✓ (2026-11-11) the website shows the VietQR code to Vietnamese readers only; every other language leads with Ko-fi, then GitHub Sponsors (`support` in `site/_data/languages.yml`).
- Append-only edit journal for the session (NFR-013): not urgent, a full save is 70 ms at 70,000 edited photos (2026-10-25)
- Lossless rotation fix-up tool
- ~~Linux build~~: shipped as the lean build in v0.8.2 ([ADR-0034](adr/0034-linux-lean.md)); HEIC there is FR-068
