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

*Vite is only a build tool. The real comparison is between the browser runtime (JS + Canvas/WASM) and native Rust code.* The estimates below are hypotheses that Phase 0 will validate.

| Concern | Web (browser) | Native Rust (Tauri) |
|---------|---------------|---------------------|
| JPEG decode | Fast: browsers use libjpeg-turbo internally | Same or faster, plus **DCT-domain downscaling** (1/2, 1/4, 1/8) for thumbnails |
| Parallelism | Web Workers. Buffers are copied or transferred | `rayon` on all cores, shared memory with zero copies |
| JPEG encode | `canvas.toBlob`: slow, few controls, fixed chroma subsampling | turbojpeg/mozjpeg with full control (quality, 4:4:4, progressive) |
| Memory ceiling | About 2–4 GB per tab, so large batches crash | System RAM, explicitly bounded by back-pressure |
| Colour management | Canvas usually converts to sRGB and drops the ICC profile | ICC preserved or converted exactly (`lcms2`) |
| Bit depth | 8-bit | 16-bit capable, needed for RAW in Phase 3 |
| HEIC | Not supported in Chrome or Firefox | libheif |
| File output | ZIP download, or the File System Access API (Chromium only) | Direct folder writes with atomic renames |
| **Estimate: 500 × 24 MP export** | Several minutes, with a risk of crashing the tab | **About 1–2 minutes on 8 cores** |

**Conclusion:** the advantage is not raw decode speed. It is throughput under parallelism, control over memory, encoder quality and colour correctness. For a professional photo tool, these matter most. See [ADR-0001](adr/0001-desktop-tauri-over-web.md).

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
| — | — | — | — | — | No code yet. The baseline is recorded in Phase 0 |

## 6. Benchmark corpus

The corpus is defined in [TESTING.md §3](TESTING.md#3-sample-corpus). Benchmarks use the `bench` subset: 200 × 24 MP JPEG (Lumix GX9), 100 × 12 MP HEIC (iPhone) and 20 × PNG screenshots.
