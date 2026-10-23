# Measured Results — Fotollection

> **Status:** Living · **Last reviewed:** 2026-10-23
>
> The **latest measured numbers** for speed and memory, in one readable page. Each number links to the run that produced it.
> This page is a summary. The append-only history of every measurement is the benchmark log in [OPTIMIZATION.md §5](OPTIMIZATION.md#5-benchmark-log), and the budgets are defined in [OPTIMIZATION.md §3](OPTIMIZATION.md#3-budgets). When a new run changes a number, update this page and add the row to §5 in the same commit.

## 1. Test setup

| Item | Value |
|------|-------|
| Machine | REF-MAC: Apple M3 (8 cores), 16 GB RAM, macOS 26.3 |
| Photos | 95 real JPEGs, 4608 × 3456 (16 MP), 287 MB in total, Canon |
| Large batch | The same 95 photos hard-linked 53 times: **5,035 photos** with distinct paths (no extra disk space) |
| Build | `cargo build --release`; Rust 1.99.1 |
| Tools | `measure_folder` (core pipeline), `/usr/bin/time -l` (peak RSS), `bench/run.sh` (Rust vs browser, 6 vs 8 in flight), `bench/batch.sh` (large batch), `bench/chart.mjs` (charts) |
| Date | 2026-10-23, all numbers from one session after the machine had cooled down |

## 2. Against the budgets

![Measured against the performance budgets](images/bench-budgets.svg)

| What | Budget ([NFR](PRD.md#7-non-functional-requirements)) | Measured | |
|------|--------|----------|---|
| Open a folder: import + EXIF + embedded thumbnails | NFR-001: < 1 s per 1,000 photos | 0.12 s per 1,000 (5,035 photos: import 383 ms + thumbnails 236 ms) | ✓ |
| Preview update (warm) | NFR-002: < 50 ms p95 | p95 27 ms, p50 26 ms; cold (first open) 120 ms | ✓ |
| Export throughput | NFR-003: ≥ 8 photos/s at 24 MP | 18.4 photos/s at 16 MP (runs: 18.1–19.7) | ✓ |
| Peak memory | NFR-004: ≤ 2 GB at any batch size | 1.41 GB (95 photos), 1.45 GB (5,035 photos) | ✓ |
| PNG decode, 16 MP | — | 122 ms (JPEG: 45 ms) | — |

## 3. Large batches: 5,035 photos

Memory must not grow with the number of photos ([NFR-004](PRD.md#nfr-004--memory-ceiling)). The library stores a few bytes per photo, thumbnails go to a disk cache, the preview cache is a bounded LRU, and export decodes at most 6 photos at a time. `bench/batch.sh` checks that design on 53 times the usual batch.

![Peak memory stays flat as the batch grows](images/bench-memory.svg)

| Phase | 95 photos (median of 3) | 5,035 photos |
|-------|-----------|--------------|
| Import + EXIF | 5 ms, RSS 7 MB | 383 ms, RSS 13 MB |
| T1 thumbnails (400 px, disk cache) | 47.8 photos/s, RSS 681 MB | 46.4 photos/s, RSS 803 MB |
| Export at 6 in flight | 18.4 photos/s, RSS 1.29 GB | 15.4 photos/s (327 s), 0 failed, RSS 1.33 GB |
| Peak RSS of the whole run | 1.41 GB | **1.45 GB** |

**Result: memory does not grow with the batch.** 53 times more photos cost 0.04 GB more at peak, and the library itself stays at 13 MB for 5,035 photos. The large export is slower per photo because it runs for more than 5 minutes and the M3 throttles under sustained load.

Not yet measured: the whole app with 5,035 photos (shell + WebKit). The grid is virtualized, and each photo summary is roughly 200 bytes of JSON, so the webview is expected to add little. That run needs a separate app-data directory so it does not overwrite the user's session project, and it is tracked in [STATUS.md](STATUS.md).

## 4. Native Rust vs the browser

The same job (decode → crop 2:3 → date stamp → JPEG q95) on the same 95 photos, in Fotollection and in headless Chromium 156 with one Web Worker per core. Median of 3 runs ([OPTIMIZATION.md §2.1](OPTIMIZATION.md#21-measured-the-same-job-in-chromium)):

![Fotollection vs the browser](images/bench.svg)

| | Fotollection (Rust) | Chromium 156 | |
|---|---:|---:|---|
| Batch export | 18.4 photos/s (6 in flight, 4:4:4, files written) | 19.0 photos/s (8 workers, 4:2:0, blobs only) | About the same |
| Warm preview, 1600 px (p50) | 25.9 ms | 17.4 ms | Browser faster |
| Cold preview (first open) | 120 ms | 104 ms | Browser faster |
| Peak memory | 1.41 GB | 1.77 GB | Fotollection 20 % lower |

## 5. Where the preview time goes

![Where the 1600 px preview time goes](images/bench-preview.svg)

| Step (1600 px preview from the cached 2048 px image) | Time | Share |
|------------------------------------------------------|-----:|------:|
| Crop + Lanczos resize | 9.7 ms | 37 % |
| Date stamp | 0.1 ms | ≈ 0 % |
| JPEG encode q85 | 16.2 ms | 62 % |
| **Total** | **26.0 ms** | Budget 50 ms; Chromium 17.4 ms |

The encoder is the bottleneck, so Stage 2 starts there ([ROADMAP.md](ROADMAP.md#phase-1-work-order-2026-10-23)).

## 6. Export concurrency

| Photos in flight | Throughput (median of 3; runs) | Peak RSS (16 MP) | Decision |
|-----------------:|-------------------------:|-----------------:|----------|
| 6 | 18.4 photos/s (18.1–19.7) | 1.41 GB (1.38–1.49) | **Kept** |
| 8 | 20.8 photos/s (19.3–21.0) | 1.71 GB (1.71–1.73) | Rejected: +13 % speed for +0.3 GB, which would pass 2 GB with 24 MP photos |

## 7. Reproduce

```bash
# Rust vs browser and 6 vs 8 in flight, median of 3 runs; writes bench/results/ and all charts
bench/run.sh ~/Pictures/some-folder 3
# Large batch: hard-links the folder 53 times (no extra disk), exports with --discard-output
bench/batch.sh ~/Pictures/some-folder 53
# Redraw the charts (English and Vietnamese) from bench/results/*.json
node bench/chart.mjs
# One-off run of the core pipeline on any folder (read-only)
/usr/bin/time -l target/release/examples/measure_folder ~/Pictures/some-folder
```

The scripts build `measure_folder` themselves. Photos are only read, and never copied into the repository; `bench/results/` holds numbers only. Let the machine cool down before a run: the M3 throttles under sustained load.
