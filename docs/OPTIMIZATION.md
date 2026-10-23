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

Result on REF-MAC (M3, 8 cores), 95 × 16 MP JPEG, 2026-10-23:

| Metric | Fotollection (Rust) | Chromium 156 | |
|--------|--------------------:|-------------:|---|
| Batch export | 18.6 photos/s (6 in flight) | 18.7 photos/s (8 workers) | Tie |
| Warm preview, 1600 px, p50 | 26.0 ms | 17.4 ms | Browser 33% faster |
| Cold preview (first open) | 122 ms | 105 ms | Browser faster |
| Peak memory | 1.40 GB | 1.76 GB | Rust 20% lower |

The work is not identical, and the differences favour the browser:

- Chromium always encodes 4:2:0 (verified from the SOF0 marker). The API cannot request 4:4:4. Fotollection encodes 4:4:4, so it writes twice the chroma data.
- The browser discards its blobs. Fotollection writes every file to disk atomically, with EXIF rewritten and GPS stripped.
- Fotollection's RSS covers the whole `measure_folder` run, including T1 thumbnails for all 95 photos, not only export and preview.

**What this changes.** The earlier hypothesis, that `canvas.toBlob` is slow, is **wrong**. Chromium's libjpeg-turbo with SIMD matches our pure-Rust codecs on throughput, and it beats our preview path. Native still wins on what the table below lists: encoder control, bounded memory, colour, bit depth, formats and file output. Raw speed is not the argument for native.

Follow-ups, measured in Stage 1 with `measure_folder` (median of 20 warm renders, 3 runs):

| Warm preview step (1600 px, from the cached 2048 px T2) | Time | Share |
|----------------------------------------------------------|-----:|------:|
| Crop + Lanczos resize (`fast_image_resize`) | 9.7 ms | 37 % |
| Date stamp (`ab_glyph`) | ≈ 0.1 ms | ≈ 0 % |
| JPEG encode q85, 4:4:4, optimized Huffman (`jpeg-encoder`) | 16.2 ms | 62 % |

- **Preview latency.** The encoder is the bottleneck, not image processing. In order of cost: skip optimized Huffman tables for previews (one extra pass that only saves bytes on a local transfer); encode previews 4:2:0, since they are never printed; then a `turbojpeg` codec behind `JpegCodec` ([ADR-0012](adr/0012-pure-rust-codecs-first.md)). Each step is benchmarked before it is kept. Target: ≤ 17 ms warm (Stage 2).
- **Export concurrency: keep 6 in flight.** 8 in flight was about 12 % faster (median 19.4 vs 17.3 photos/s, noisy), but peak RSS rose from 1.32–1.45 GB to 1.76–1.91 GB on 16 MP photos. Scaled to 24 MP buffers, that would break the 2 GB budget ([NFR-004](PRD.md#nfr-004--memory-ceiling)), so `default_inflight()` stays at 6.

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

## 6. Benchmark corpus

The corpus is defined in [TESTING.md §3](TESTING.md#3-sample-corpus). Benchmarks use the `bench` subset: 200 × 24 MP JPEG (Lumix GX9), 100 × 12 MP HEIC (iPhone) and 20 × PNG screenshots.
