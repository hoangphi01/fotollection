# Optimization — Fotollection

> **Status:** Draft v0.1 · **Last reviewed:** 2026-10-23
>
> Performance is the #1 product quality for Fotollection, which targets enthusiast and professional photographers.
> This document holds the **budgets**, the **methodology** used to measure them, the **design techniques** that meet them, and a **log** of every measurement.
> The budgets themselves are defined as NFRs in [PRD.md](PRD.md#7-non-functional-requirements). This file explains how we hit and verify them.

## 1. Rules

1. **Measure first.** No optimization is merged without a before/after benchmark logged in [§5](#5-benchmark-log).
2. **Budgets are tests.** Once code exists, CI compares benchmark results against `bench/budgets.toml` and fails if a regression exceeds 10%.
3. **Optimize the pipeline, not the micro-op.** Throughput comes from parallelism, avoiding redundant decodes, and bounded memory, rather than from clever loops.
4. **Never trade correctness for speed.** Orientation, ICC profiles and stamp placement must be identical between the fast path and the slow path.

## 2. Why native Rust instead of a web app

*Vite is only a build tool. The real comparison is between the browser runtime (JS + Canvas/WASM) and native Rust code.*

### 2.1 Measured: the same job in Chromium

`bench/run.sh <photo-folder>` runs both sides on the same folder. The browser side (`bench/web/`) does the export job in headless Chromium:

1. `fetch`, then `createImageBitmap` with EXIF orientation.
2. Centre crop to 2:3.
3. Date stamp with `fillText`.
4. `OffscreenCanvas.convertToBlob` as JPEG q0.95.

The browser runs one Web Worker per logical core. For previews, it decodes once, caches a 2048 px bitmap (like T2), then re-renders 1600 px 20 times. The Rust side is `measure_folder --json`. Peak RSS comes from `/usr/bin/time -l` for Rust, and from the sampled RSS of the browser's process tree for Chromium. Each metric is the median of 3 runs. Raw numbers are in `bench/results/`, and the chart is `docs/images/bench.svg`.

Latest result on REF-MAC (M3, 8 cores), 95 × 16 MP JPEG, 2026-10-23, after a cool-down (charts and the full table: [RESULT.md](RESULT.md)):

| Metric | Fotollection (Rust) | Chromium 156 | |
|--------|--------------------:|-------------:|---|
| Batch export | 18.0 photos/s (6 in flight) | 17.9 photos/s (8 workers) | Tie |
| Warm preview, 1600 px, p50 | 17.4 ms (26.0 ms before the screen encoder) | 18.5 ms | Rust faster |
| Cold preview (first open) | 120 ms | 119 ms | Tie |
| Peak memory | 1.35 GB | 1.78 GB | Rust 24% lower |

The work is not identical, and the differences favour the browser:

- Chromium always encodes 4:2:0 (verified from the SOF0 marker). The API cannot request 4:4:4. Fotollection encodes 4:4:4, so it writes twice the chroma data.
- The browser discards its blobs. Fotollection writes every file to disk atomically, with no EXIF, so GPS is always stripped.
- Fotollection's RSS covers the whole `measure_folder` run, including T1 thumbnails for all 95 photos, not only export and preview.

**What this changes.** The earlier hypothesis, that `canvas.toBlob` is slow, is **wrong**. Chromium's libjpeg-turbo with SIMD matches our pure-Rust codecs on throughput, and it beats our preview path. Native still wins on what the table below lists: encoder control, bounded memory, colour, bit depth, formats and file output. Raw speed is not the argument for native.

Follow-ups, measured in Stage 1 with `measure_folder` (median of 20 warm renders, 3 runs):

| Warm preview step (1600 px, from the cached 2048 px T2) | Time | Share |
|----------------------------------------------------------|-----:|------:|
| Crop + Lanczos resize (`fast_image_resize`) | 9.7 ms | 37 % |
| Date stamp (`ab_glyph`) | ≈ 0.1 ms | ≈ 0 % |
| JPEG encode q85, 4:4:4, optimized Huffman (`jpeg-encoder`) | 16.2 ms | 62 % |

- **Preview latency (done in Stage 2).** The encoder was the bottleneck. Previews and thumbnails now use `EncodeOptions::screen` (plain Huffman tables, 4:2:0), exports keep `EncodeOptions::print` (4:4:4, optimized tables). Encode went from 16.2 to 11.7 ms without the Huffman pass and to 7.0 ms with 4:2:0; the warm preview from 26.0 to 17.4 ms, below Chromium (18.5 ms). A CatmullRom resize filter was slower than Lanczos3 (12.5 vs 9.7 ms) and was not kept. `turbojpeg` is no longer needed for previews.
- **Export concurrency: keep 6 in flight.** 8 in flight is 8–13 % faster (latest medians 19.5 vs 18.0 photos/s), but peak RSS rises from 1.30–1.41 GB to 1.70–1.77 GB on 16 MP photos. Scaled to 24 MP buffers, that would break the 2 GB budget ([NFR-004](PRD.md#nfr-004--memory-ceiling)), so `default_inflight()` stays at 6.

### 2.2 Qualitative comparison

| Concern | Web (browser) | Native Rust (Tauri) |
|---------|---------------|---------------------|
| JPEG decode | Fast: browsers use libjpeg-turbo internally | Same or faster, plus **DCT-domain downscaling** (1/2, 1/4, 1/8) for thumbnails |
| Parallelism | Web Workers. Buffers are copied or transferred | `rayon` on all cores, shared memory with zero copies |
| JPEG encode | `convertToBlob`: fast (§2.1), but quality is the only control; always 4:2:0 | turbojpeg/mozjpeg with full control (quality, 4:4:4, progressive) |
| Memory ceiling | About 2–4 GB per tab, so large batches crash | System RAM, explicitly bounded by back-pressure |
| Colour management | Canvas usually converts to sRGB and drops the ICC profile | ICC preserved or converted exactly (`lcms2`) |
| Bit depth | 8-bit | 16-bit capable, needed for RAW in Phase 3 |
| HEIC | Not supported in Chrome or Firefox | libheif |
| File output | ZIP download, or the File System Access API (Chromium only) | Direct folder writes with atomic renames |
| **Measured: 95 × 16 MP export (§2.1)** | 18.7 photos/s at 4:2:0, blobs not written | 18.6 photos/s at 4:4:4, files written |

**Conclusion:** measurement confirms that the advantage is not raw speed. It is control over memory, encoder quality, colour correctness and file handling. For a professional photo tool, these matter most. See [ADR-0001](adr/0001-desktop-tauri-over-web.md).

## 3. Budgets

| NFR | Metric | Budget | How it is measured |
|-----|--------|--------|--------------------|
| [NFR-001](PRD.md#nfr-001--library-load-time) | First thumbnails for 1,000 JPEGs | < 1 s; all ready in < 10 s | `bench_library_load`, cold cache |
| [NFR-002](PRD.md#nfr-002--preview-latency) | Preview re-render, p95 | < 50 ms | `bench_preview`, 2 MP preview buffer |
| [NFR-003](PRD.md#nfr-003--export-throughput) | Export throughput, 24 MP JPEG | ≥ 8 photos/s | `bench_export_batch`, 200 files |
| [NFR-004](PRD.md#nfr-004--memory-ceiling) | Peak RSS during export | ≤ 2 GB | export of 2,000 files, RSS sampled every 50 ms |
| [NFR-005](PRD.md#nfr-005--ui-responsiveness) | Input latency during export | < 100 ms | Manual + E2E probe |
| [NFR-008](PRD.md#nfr-008--install-size) | Installer size | ≤ 25 MB | CI artifact size check |
| [NFR-010](PRD.md#nfr-010--cold-start) | Launch to interactive | < 1.5 s | E2E timing |
| [NFR-012](PRD.md#nfr-012--culling-navigation) | Loupe next/previous to a sharp preview, p95 | < 100 ms | `bench_loupe_navigation` + E2E probe |
| [NFR-013](PRD.md#nfr-013--project-safety) | Edit-to-disk latency; project save time for 10,000 photos | ≤ 5 s; < 50 ms | `bench_project_save`, kill-during-write test |
| [NFR-014](PRD.md#nfr-014--best-shot-analysis-speed) | Best-shot analysis per photo, 8 cores in parallel | ≤ 20 ms; ≤ 40 ms with faces | `bench_culling` on the `bench` corpus |
| [NFR-015](PRD.md#nfr-015--normal-mode-speed) | Normal-mode export to 10×15 @ 300 DPI, q90; first screen of 1,000 photos | ≥ 3× Professional (≥ 50 photos/s); < 0.5 s | `measure_folder` with a Normal-mode flag, same folders as `bench/run.sh` |

### Reference machines

| ID | Machine | Notes |
|----|---------|-------|
| `REF-MAC` | MacBook, Apple M3, 8 cores, 16 GB | Primary development machine |
| `REF-WIN` | Mid-range Windows laptop, 8 cores, 16 GB, NVMe | To be defined in Phase 0 |
| `CI` | GitHub-hosted runners | Used for regression trends only, never for absolute budgets |

## 4. Design techniques

### 4.1 Multi-tier preview

| Tier | Source | Size | Used for |
|------|--------|------|----------|
| T0 | EXIF-embedded thumbnail | ~160 px | Instant grid on first paint |
| T1 | DCT-scaled decode (1/8) | ~400 px | Sharp grid thumbnails, disk-cached |
| T2 | DCT-scaled decode (1/2–1/4) + resize | ~2 MP | Interactive preview of stamp and crop |
| T3 | Full decode | Original | Export only |

The preview renders the same `RenderPlan` as the export ([ARCHITECTURE.md §4](ARCHITECTURE.md#4-processing-pipeline)), only at T2 scale.

**Photo Style cost ([FR-039](PRD.md#fr-039--photo-style-looks)):** style parameters are baked into a 33³ 3D LUT (about 430 KB, built in about 1 ms when a parameter changes) plus a 1D tone curve. Applying the look is **one trilinear lookup per pixel**, done with SIMD across rayon threads. Estimated cost: about 2 ms on a 2 MP preview (within [NFR-002](PRD.md#nfr-002--preview-latency)) and about 15–25 ms per 24 MP export on 8 cores (to be checked against [NFR-003](PRD.md#nfr-003--export-throughput) in Phase 0/2 benchmarks). Grain uses a precomputed tiled noise texture with a per-photo seed, so it is deterministic and costs no extra RNG.

**Culling prefetch ([NFR-012](PRD.md#nfr-012--culling-navigation)):** the loupe keeps T2 previews of the current photo and of the **next two and previous one** in an LRU cache (about 8 MB each). Moving to the next photo shows the cached T2 immediately and requests the next prefetch. Rating and flagging only change project state, never pixels, so they are instant. The safe-area overlay and the DPI badge are computed from `RenderPlan` geometry, not from pixels.

### 4.2 Avoid redundant work

- Decode each source **once** per export, and never twice to get both a thumbnail and the full image.
- Crop before any per-pixel work, so only the pixels that survive are processed.
- For **crop-only** JPEG output (no stamp), use a lossless DCT-domain crop (`tjTransform`). This is faster and gives zero quality loss.
- Memoize geocoding per ~1 km grid cell. A trip of 800 photos needs only tens of lookups.

### 4.3 Parallelism and back-pressure

- Bounded channels between decode → render → encode → write. The in-flight limit is derived from the memory budget: `max_inflight = min(num_cpus, budget / bytes_per_decoded_image)`.
- Writing to disk is I/O-bound and runs on a small dedicated pool, so it never blocks the CPU stages.
- Thumbnail work runs at low priority and pauses while an export is running.

### 4.3.1 Lesson: keep interactive work off background queues

In the first slice, protocol requests shared one pool with background T1 generation. In unoptimized dev builds, grid tiles stayed blank until all 215 thumbnails were done. The fix splits the work into interactive and background pools (ARCHITECTURE.md §5), and `fotollection-core` is built at `opt-level = 3` even in dev (DEVELOPMENT.md §4.1).

### 4.4 SIMD and codecs

- libjpeg-turbo and `fast_image_resize` use NEON on Apple Silicon and AVX2 on x86-64.
- Release builds use `lto = "fat"`, `codegen-units = 1` and `panic = "abort"` for the app binary.
- Do **not** enable `target-cpu=native` for distributed builds. Use runtime feature detection instead.

### 4.5 Deferred until measured

These ideas are recorded here and **not** implemented until a benchmark shows they are needed:

- GPU compositing or resize via `wgpu`.
- A custom JPEG encoder or decoder.
- Memory-mapped input files.
- A persistent library database.

## 5. Benchmark log

Append one row per measurement that matters. Never edit past rows.

| Date | Commit | Machine | Benchmark | Result | Note |
|------|--------|---------|-----------|--------|------|
| 2026-10-23 | slice 1 | REF-MAC (M3) | `measure_folder`: import + EXIF, 95 × 16 MP JPEG | 28 ms total | NFR-001 on track (~0.3 s per 1,000) |
| 2026-10-23 | slice 1 | REF-MAC (M3) | T0 embedded thumbnails, 95 photos | 7 ms total | Instant grid on first paint |
| 2026-10-23 | slice 1 | REF-MAC (M3) | T1 thumbnails (full decode → 400 px), parallel | 49 photos/s | Background, low-priority pool |
| 2026-10-23 | slice 1 | REF-MAC (M3) | Full decode 4608×3456, `zune-jpeg` | 45 ms | ADR-0012 |
| 2026-10-23 | slice 1 | REF-MAC (M3) | Preview re-render (T2 cached) | cold 121 ms, warm p50 26.0 / p95 26.4 ms | NFR-002 ✓ |
| 2026-10-23 | slice 1 | REF-MAC (M3) | Export 95 × 16 MP, q95 4:4:4, 6 in flight | 19.7 photos/s | NFR-003 ✓ (~13/s projected for 24 MP) |
| 2026-10-23 | slice 1 | REF-MAC (M3) | Peak RSS of the whole `measure_folder` run | 1.41 GB | NFR-004 ✓ |
| 2026-10-23 | cda7a5f | REF-MAC (M3) | `bench/run.sh`: Rust export vs Chromium 156, 95 × 16 MP, median of 3 | 18.6 vs 18.7 photos/s | Tie; Chromium encodes 4:2:0, Rust 4:4:4 (§2.1) |
| 2026-10-23 | cda7a5f | REF-MAC (M3) | `bench/run.sh`: warm preview p50, Rust vs Chromium | 26.0 vs 17.4 ms | Browser faster; follow-up in §2.1 |
| 2026-10-23 | cda7a5f | REF-MAC (M3) | `bench/run.sh`: peak RSS, Rust vs Chromium | 1.40 vs 1.76 GB | Rust 20% lower |
| 2026-10-23 | Stage 1 | REF-MAC (M3) | Full PNG decode 4608×3456 RGB8 (`png` crate, `render_one`) | 122 ms | 2.7× JPEG (45 ms); PNG photos are rare, thumbnails come from T1 |
| 2026-10-23 | Stage 1 | REF-MAC (M3) | Warm preview breakdown, 1600 px from T2 (`measure_folder`) | resize 9.7 ms, stamp ≈ 0.1 ms, encode q85 16.2 ms | Encode is 62 % (§2.1) |
| 2026-10-23 | Stage 1 | REF-MAC (M3) | Export 95 × 16 MP, 6 vs 8 in flight, median of 3 | 17.3 vs 19.4 photos/s; peak RSS 1.32–1.45 vs 1.76–1.91 GB | Keep 6: 8 would exceed 2 GB at 24 MP |
| 2026-10-23 | Stage 1 | REF-MAC (M3) | 5,035 photos (95 hard-linked × 53): import, T0, T1, export with `--discard-output` | import 358 ms, T1 32.6/s, export 13.5/s, peak RSS 1.47 GB | NFR-004 ✓, memory flat vs 1.33 GB for 95 photos; machine throttled ([RESULT.md](RESULT.md)) |
| 2026-10-23 | 9b038c1 | REF-MAC (M3) | Re-run after cool-down, one session: `bench/run.sh` (3 runs) + `bench/batch.sh` (5,035 photos) | Rust/Chromium export 18.4/19.0 photos/s, preview p50 25.9/17.4 ms, RSS 1.41/1.77 GB; 8 in flight 20.8 photos/s at 1.71 GB; 5,035 photos peak 1.45 GB | Numbers behind the README and RESULT charts |
| 2026-10-23 | Stage 2 | REF-MAC (M3) | Preview encoder: 4:4:4 + optimized Huffman → plain Huffman → + 4:2:0 (`measure_folder`) | encode 16.2 → 11.7 → 7.0 ms; warm preview p50 26.0 → 16.5 ms (isolated) / 17.4 ms (`bench/run.sh`) | NFR-002; screen encoding only, exports unchanged |
| 2026-10-23 | Stage 2 | REF-MAC (M3) | Preview resize filter: CatmullRom vs Lanczos3 (`fast_image_resize`) | 12.5 vs 9.7 ms | Lanczos3 kept |
| 2026-10-23 | Stage 2 | REF-MAC (M3) | `bench/run.sh` after the encoder change, 3 runs | Rust/Chromium preview p50 17.4/18.5 ms, export 18.0/17.9 photos/s, RSS 1.35/1.78 GB; 8 in flight 19.5 photos/s at 1.72 GB | Fotollection now previews faster than the browser |
| 2026-10-23 | Stage 2 | REF-MAC (M3) | UI bundle (`vite build`), before/after i18next + both locales | 488.7 → 552.3 kB (gzip 152.6 → 173.6 kB) | ADR-0015; read from disk, NFR-010 to be measured |
| 2026-10-24 | Stage 3 | REF-MAC (M3) | HEIC decode 4608×3456 via ImageIO (`render_one`, photo converted with `sips`) | ≈ 170 ms (165–207) | 3.8× the same photo as JPEG (45 ms); follow-up: ImageIO thumbnails for T1 |
| 2026-10-24 | Stage 3 | REF-MAC (M3) | Best-shot analysis from the 2048 px T2, one thread (`measure_folder --analyze`, 95 photos): first version | no faces p50 52 ms, with faces p50 64 ms | Over NFR-014 (20 / 40 ms) |
| 2026-10-24 | Stage 3 | REF-MAC (M3) | Same, after: histogram percentiles instead of sorts and slice windows in `measure` (26 → 3.8 ms); YuNet at the photo's shape, 512 px, instead of a padded 640² (19 → 8.3 ms; portrait padded to 512², 11.0 ms, because tract runs a tall input slower, 12.8 ms); 2×2 halving instead of a box resize (5.3 → ≈ 1 ms) | no faces p50 17.4 / p95 18.0 ms, with faces p50 27.8 / p95 36 ms; whole pool ≈ 110–125 photos/s | NFR-014 ✓ |
| 2026-10-24 | Stage 3 | REF-MAC (M3) | Best-shot grouping signature (32 × 24 luma + colour histogram, ADR-0019) computed on the 1,024 px image, then on a 256 px halving | no faces p50 19.9 / p95 20.5 ms → 17.4 / 18.1 ms; with faces p50 27.6 ms | NFR-014 ✓ kept; accuracy unchanged (15/22 groups, 6/6 picks) |
| 2026-10-24 | FR-005 | REF-MAC (M3) | Offline places (`measure_geocode`): 161,937 GeoNames towns, 5.4 MB | first load 25 ms (checksum + grid), reverse 4.6 µs per lookup, search 25 ms per query | Lookup is negligible next to import (NFR-001); search is fast enough to run on every keystroke (NFR-005) |
| 2026-10-24 | FR-006 | REF-MAC (M3) | Straighten: `render_one` render step, 16 MP photo to a full-size 3:2 crop, 0° vs 4.5°, 5 runs each | 0°: median 8 ms; 4.5°: median 61 ms (bicubic over the crop only, rayon rows) | +53 ms only for straightened photos; decode stays 46 ms. A level photo takes the unchanged lossless path |
| 2026-10-25 | FR-053 | REF-MAC (M3) | Share export, 20 × 16 MP JPEG + 5 Display P3 (`quality_export --share`, `share_audit.py`) | Original q80: 1.42 MB (46 %), ΔE 0.96, SSIM 0.981, 14 photos/s · Facebook q88 + sharpen 0.2: 0.51 MB, SSIM 0.979, ΔE 1.27, 23 photos/s · after Facebook: SSIM 0.965 vs 0.961, P3 colour 99.6 % vs 86 % for the usual way | Sweep: q80/84/88 × sharpen 0/0.2/0.35; RESULT §10 |
| 2026-10-25 | NFR-013 | REF-MAC (M3) | Session save vs library size, every photo edited (`measure_project`, median of 5) | 5,035: 10.5 ms (snapshot 1.5, JSON 1.9, write 7.1), 1.0 MB · 20,000: 20.8 ms, 4.1 MB · 70,000: 70.5 ms (snapshot 20.8 with the library locked, JSON 14.6, write 35.1), 14.4 MB | Linear; fine at the 1 s autosave. An append-only journal is planned, not urgent |
| 2026-10-25 | FR-057 | REF-MAC (M3) | `measure_preview`: one JPEG at 16 / 26 / 32 MP, median of 5 | full decode 46 / 68 / 84 ms, 48 / 78 / 97 MB decoded · preview cold 105 / 139 / 154 ms, warm 17–18 ms · full-size render 4–9 ms · 2560 × 1600 region encode 17 ms · peak RSS 222 / 314 / 371 MB · `RAYON_NUM_THREADS=2` and `=1`: same times | Preview stays capped at 1600 px from T2; zoom renders only the visible part at full resolution, originals cached ≤ 200 MB. RESULT §11 |
| 2026-10-25 | NFR-010 | REF-MAC (M3) | `scripts/measure-startup.sh`, release build, 5 launches with the session restored | first launch after build: Rust setup 594 ms, page loaded 2689 ms, first thumbnail 2895 ms · next 4: setup 212–281 ms, page loaded 309–395 ms, first thumbnail 503–575 ms | Within 1.5 s after the first launch; the first one is the web view starting (≈ 2.1 s) with 22 GB of swap in use. The start-up screen (FR-060) covers it |
| 2026-10-25 | NFR-017 | REF-MAC (M3) | `measure_raw`: 9 CC0 RAW samples (RW2 ×2, CR2, CR3, NEF, ARW, RAF, ORF, DNG) through ImageIO, median of 5 | full develop 385–2172 ms (RW2 GX9 867 ms) · embedded preview 9–32 ms at ≤ 400 px, 19–80 ms at ≤ 2048 px · peak RSS 1.29 GB (CR2 31 MP), 579 MB (RW2) | Two paths (ADR-0023): preview for thumbnails and analysis, develop for preview/export; type hint required for NEF/ARW. RESULT §12 |
| 2026-10-25 | NFR-016 | REF-MAC (M3) | "Open with" end to end: `open -a Fotollection.app <photo>` on the release bundle, `FOTOLLECTION_TIMING`, session restored, deep swap | JPEG (22-photo folder): new folder 1.25 s, folder already a tab 0.70 / 0.70 s · RAW, warm: NEF 1.41 s, RW2 (24 MP) 2.54 s · cold first launch after a build: RW2 6.9 s (session restore 3.2 s) | Within 1 s once the folder is a tab; a new folder pays its import first (to improve: show the photo before the folder is listed). RAW within NFR-017 (< 2.5 s) |
| 2026-10-25 | NFR-008 | CI (windows-latest) | Installer size after FR-061 (`ureq` with rustls, `semver`, `tauri-plugin-opener`), `installers` workflow vs the v0.7.0 release | `-setup.exe` 13.7 → 14.4 MB · `.msi` 18.8 → 19.6 MB | +0.7 MB for the update check; still well under the 25 MB budget |
| 2026-10-26 | DATABASE §8 | REF-MAC (M3) | `measure_cache`: cache housekeeping on N 1-byte files in 256 folders, median of 5 | 10,000 files: walk 28 ms, prune under the limit 27 ms, prune to half 0.31 s · 70,000 files: walk 223 ms, prune under the limit 211 ms, prune to half 7.1 s (38,500 deleted) | Cheap enough to run after every start-up and thumbnail batch, on the background pool; deletions dominate a real prune |

## 6. Benchmark corpus

The corpus is defined in [TESTING.md §3](TESTING.md#3-sample-corpus). Benchmarks use the `bench` subset: 200 × 24 MP JPEG (Lumix GX9), 100 × 12 MP HEIC (iPhone) and 20 × PNG screenshots.
