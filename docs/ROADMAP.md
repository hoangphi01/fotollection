# Roadmap — Fotollection

> **Status:** Draft v0.1 · **Last reviewed:** 2026-10-23
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
| FR-048 | Normal and Professional modes |
| FR-050 | Print basket: the photos that will really be printed, exported to an extra folder |
| FR-020 | Localization (English and Vietnamese; German later) |

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
| v0.4.0 | Sharing and folders | FR-053, FR-055, FR-056, FR-057 (zoom on the preview), FR-040 (instax-style Fit, drag to reframe) | Done: RESULT §10 |
| v0.5.0 | Print basket | FR-050 (basket and set aside, select mode), FR-059 (Quick-preview) | — |
| v0.6.0 | Photo viewer | FR-057 (full-resolution zoom, zoom buttons), FR-060 (start-up screen) | Done: cold start (NFR-010), open to first photo (NFR-016), preview and zoom at 16/26/32 MP (RESULT §11) |
| v0.7.0 | Open with and RAW | FR-054 ("Open with"), FR-016 (RAW through the OS codecs) | Done: RAW on nine CC0 samples (RESULT §12, NFR-017), open to first photo (NFR-016) |
| v0.8.0 | Places and Lightroom, and the camera (title to confirm at release) | FR-032, FR-005, FR-030, FR-031, FR-058 (XMP), FR-061 (check for updates), FR-065 (border in Crop to fill), FR-062 + FR-063 (capture screen with live view, Windows, Sony a6400; [ADR-0028](adr/0028-camera-sessions-saved-on-demand.md)) | Lookup speed with manual places, a mixed camera + phone folder; XMP read on a Lightroom-culled folder; NFR-018 and NFR-019 measured in the app with the a6400 |
| v0.9.0 | Normal mode | FR-048, NFR-015, FR-046, FR-050 | Normal against Professional export |
| To place | Camera, continued | The camera on macOS, other Sony bodies (read-only probe first), FR-064 (watch a folder), webcam (UVC) live view | [cameras/README.md](cameras/README.md), [cameras/sony.md](cameras/sony.md) |
| Later | Looks, presets, trips, journal | FR-039, FR-047, FR-038, FR-012, FR-014, FR-049, FR-015, FR-051 | — |

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
| FR-039 | Photo Style looks |
| FR-044 | Per-photo overrides |
| FR-046 | Swipe culling (full-screen ↑ / ↓ print list) |
| FR-047 | Film looks (Fujifilm-style film simulations, own names and LUTs) |
| FR-051 | Landmark recognition: suggest a place from what the photo shows (offline, optional model pack) |
| FR-052 | Appearance: System / Light / Dark theme, Windows title bar in the app's colours, app-drawn hover hints |
| FR-053 | Export for social media: Original / Facebook / Instagram / custom sizes, sRGB, EXIF with the app's name and never GPS |
| FR-054 | Open photos with Fotollection from the system's "Open with" menu, as a photo viewer |
| FR-055 | Export the photo open in the preview, with the same export controls as the panel |
| FR-056 | Folders as tabs: All photos, one folder or several ticked, exported together or separately |
| FR-057 | Zoom in the preview: pinch, Ctrl/⌘ + − 0, drag to move, double-click for 200 % |
| FR-058 | Lightroom compatibility: read XMP sidecars on import, write `.xmp` as an opt-in, photos never touched |
| FR-059 | Quick-preview: hold Space in select mode to look at a photo as it will print |
| FR-060 | Start-up screen: the logo and a loading bar until the last session is back |
| FR-061 | Check for updates: a button that asks GitHub once, then opens the installer in the browser |
| FR-062 | Live view from a camera plugged in by cable (own PTP driver, UVC webcam or HDMI capture card), with the print ratio drawn over it |
| FR-063 | Tethered capture over USB: Capture in the app, the full-resolution photo opens at once; own PTP drivers per maker, no install ([ADR-0026](adr/0026-cameras-through-the-os-usb-stack.md)) |
| FR-064 | Watch a folder: a photo written into it by a maker's tether app opens at once |
| FR-065 | Border in Crop to fill: 3 mm in the frame colour inside the print size, instant-photo look at the exact kiosk size |
| FR-066 | Borrow places from a GPX track: interpolate each photo's time along the track (split out of FR-031) |
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
| FR-035 | Multi-photo layouts |
| FR-037 | Contact sheet |

## Ideas backlog (not yet requirements)

These ideas have no ID yet. Promote one to a PRD requirement before scheduling it.

- **Look back one trip at a time from the Photos library (macOS, PhotoKit, read-only).** Pick a trip by date or place; the app groups it, suggests the best shots, and puts the rejects in a "To delete" album the user empties in Photos. With iCloud Photos this frees space on the iPhone too, without the app ever deleting anything. Prompted by the owner's own library: about 70,000 photos on a 512 GB iPhone. Measure a 70,000-photo library first (NFR-001, NFR-014).
- **iPhone companion app**, only if the Mac flow proves itself: swipe culling on the way home and real deletion behind iOS's own confirmation. Needs the Apple Developer Program (99 USD/year, which would also notarize the Mac app), a native SwiftUI interface over the Rust core (UniFFI), and Core ML for the face models.
- **Support from abroad:** a Ko-fi or GitHub Sponsors link next to the VietQR code on the website.
- Append-only edit journal for the session (NFR-013): not urgent, a full save is 70 ms at 70,000 edited photos (2026-10-25)
- Lossless rotation fix-up tool
- Linux build
