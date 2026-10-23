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
| Tools | `measure_folder` (core pipeline), `/usr/bin/time -l` (peak RSS), `bench/run.sh` (Rust vs browser) |
| Date | 2026-10-23 |

## 2. Against the budgets

| What | Budget ([NFR](PRD.md#7-non-functional-requirements)) | Measured | |
|------|--------|----------|---|
| Import + EXIF | NFR-001: first thumbnails for 1,000 photos < 1 s | 5,035 photos: import 358 ms, embedded thumbnails 237 ms | ✓ |
| Preview update (warm) | NFR-002: < 50 ms | p50 26 ms, p95 27 ms | ✓ |
| Export throughput | NFR-003: ≥ 8 photos/s at 24 MP | 17–19 photos/s at 16 MP (≈ 12/s projected at 24 MP) | ✓ |
| Peak memory, 95 photos | NFR-004: ≤ 2 GB at any batch size | 1.32–1.45 GB | ✓ |
| Peak memory, 5,035 photos | NFR-004 | **1.47 GB**: flat, 53 × the photos for +0.1 GB | ✓ |
| PNG decode, 16 MP | — | 122 ms (JPEG: 45 ms) | — |

## 3. Large batches: 5,035 photos

Memory must not grow with the number of photos ([NFR-004](PRD.md#nfr-004--memory-ceiling)). The library stores a few bytes per photo, thumbnails go to a disk cache, the preview cache is a bounded LRU, and export decodes at most 6 photos at a time. This run checks that design on 53 times the usual batch.

| Phase | 95 photos | 5,035 photos |
|-------|-----------|--------------|
| Import + EXIF | 40 ms, RSS 7 MB | 358 ms, RSS 13 MB |
| T1 thumbnails (400 px, disk cache) | 39.9 photos/s, RSS 720 MB | 32.6 photos/s, RSS 708 MB |
| Export at 6 in flight | 12.1 photos/s, RSS 1.20 GB | 13.5 photos/s (374 s), 0 failed, RSS 1.14 GB |
| Peak RSS of the whole run | 1.33 GB | **1.47 GB** |

**Result: memory does not grow with the batch.** 53 times more photos cost about 0.1 GB more at peak, and the library itself stays at 13 MB for 5,035 photos. The 95-photo column is a run made right after the large one, under the same thermal conditions. Both runs are slower than the 17–19 photos/s of a cool machine (§2), because the M3 throttles after minutes of full load; per photo, the large batch is not slower.

Not yet measured: the whole app with 5,035 photos (shell + WebKit). The grid is virtualized and each photo summary is roughly 200 bytes of JSON, so it is expected to add little. That run needs a separate app-data directory so it does not overwrite the user's session project, and is tracked in [STATUS.md](STATUS.md).

## 4. Native Rust vs the browser

The same job (decode → crop 2:3 → date stamp → JPEG q95) on the same 95 photos, in Fotollection and in headless Chromium 156 with one Web Worker per core. Median of 3 runs ([OPTIMIZATION.md §2.1](OPTIMIZATION.md#21-measured-the-same-job-in-chromium)):

![Batch export: Fotollection 18.6 photos/s, browser 18.7. Warm preview: Fotollection 26.0 ms, browser 17.4 ms. Peak memory: Fotollection 1.40 GB, browser 1.76 GB.](images/bench.svg)

- **Export:** a tie, 18.6 vs 18.7 photos/s. Fotollection encodes 4:4:4 and writes every file; Chromium can only encode 4:2:0 and kept its blobs in memory.
- **Preview:** Chromium is faster, 17.4 vs 26.0 ms.
- **Memory:** Fotollection peaks 20 % lower, 1.40 vs 1.76 GB.

## 5. Where the preview time goes

| Step (1600 px preview from the cached 2048 px image) | Time | Share |
|------------------------------------------------------|-----:|------:|
| Crop + Lanczos resize | 9.7 ms | 37 % |
| Date stamp | ≈ 0.1 ms | ≈ 0 % |
| JPEG encode q85 | 16.2 ms | 62 % |

The encoder is the bottleneck, so Stage 2 starts there ([ROADMAP.md](ROADMAP.md#phase-1-work-order-2026-10-23)).

## 6. Export concurrency

| Photos in flight | Throughput (median of 3) | Peak RSS (16 MP) | Decision |
|-----------------:|-------------------------:|-----------------:|----------|
| 6 | 17.3 photos/s | 1.32–1.45 GB | **Kept** |
| 8 | 19.4 photos/s | 1.76–1.91 GB | Rejected: would exceed 2 GB with 24 MP photos |

## 7. Reproduce

```bash
# Core pipeline on any folder (read-only); add --json for machine-readable output
/usr/bin/time -l target/release/examples/measure_folder ~/Pictures/some-folder
# Large batch without the disk space: exported files are deleted as soon as they are written
target/release/examples/measure_folder ~/Pictures/big-folder --discard-output
# Rust vs browser, median of 3 runs, regenerates docs/images/bench.svg
bench/run.sh ~/Pictures/some-folder 3
```

Build the examples first with `cargo build --release -p fotollection-core --examples`. Photos are only read, and never copied into the repository.
