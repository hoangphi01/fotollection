# Testing Strategy — Fotollection

> **Status:** Draft v0.1 · **Last reviewed:** 2026-10-23
>
> How we prove that Fotollection is correct, robust and fast. Performance benchmarks are specified in [OPTIMIZATION.md](OPTIMIZATION.md).

## 1. Test pyramid

| Level | Scope | Tooling | Runs |
|-------|-------|---------|------|
| Unit | Pure functions in `crates/core` (crop math, stamp layout, date formatting, path sanitizing, geocode lookup) | `cargo test`, `proptest` | Every push |
| UI component | React components and hooks, with IPC mocked | `vitest` + Testing Library | Every push |
| Golden image | Full `RenderPlan` → pixels for fixed inputs, compared to reference outputs | `cargo test` + image diff | Every push |
| Integration | Pipeline end-to-end on the sample corpus (import → export to a temp dir) | `cargo test --test pipeline` | Every push (small set), nightly (full set) |
| Fuzz | Decoders, EXIF parser, path sanitizer | `cargo-fuzz` | Nightly |
| E2E | Real app window: import, adjust, export, cancel | Tauri WebDriver (`tauri-driver`) | Before each release, on macOS and Windows |
| Benchmark | Budgets from [OPTIMIZATION.md §3](OPTIMIZATION.md#3-budgets) | `criterion` | Nightly + before each release |

## 2. Requirement traceability

- Every FR has at least one test named `fr_NNN_<behaviour>` ([STYLE_RULE.md §2](STYLE_RULE.md#naming)).
- An FR can be marked **Done** in [STATUS.md](STATUS.md) only after its test exists and passes. Once code exists, CI enforces this through `scripts/check-docs.sh`.
- Every NFR with a numeric budget has a matching benchmark or E2E probe, listed in [OPTIMIZATION.md §3](OPTIMIZATION.md#3-budgets).

## 3. Sample corpus

The corpus is stored **outside** git, because the photos are personal and large. It is fetched by `scripts/fetch-corpus.sh` from a private location. A tiny, license-clean subset is committed under `crates/core/tests/fixtures/`.

| Subset | Contents | Purpose |
|--------|----------|---------|
| `fixtures` (in git) | ~20 small files: each orientation (1–8), no-EXIF, GPS/no-GPS, (0,0) GPS, HEIC, PNG, wide-gamut ICC, CMYK JPEG, corrupt/truncated files, polyglot | Unit, golden and security tests |
| `bench` | 200 × 24 MP JPEG (Lumix GX9), 100 × 12 MP HEIC (iPhone), 20 × PNG | Benchmarks |
| `stress` | 2,000+ mixed files | Memory ceiling, long-run stability |

## 4. Fuzzing

Fuzz targets under `crates/core/fuzz/`:

| Target | Input |
|--------|-------|
| `fuzz_metadata` | Arbitrary bytes → metadata parser |
| `fuzz_decode_jpeg` | Arbitrary bytes → JPEG decode at T1 scale |
| `fuzz_decode_heic` | Arbitrary bytes → HEIC decode |
| `fuzz_sanitize_path` | Arbitrary strings → export path builder; asserts the result stays under the root |

Any crash found by fuzzing is added to `fixtures` as a regression test.

## 5. Golden images

- Reference outputs live in `crates/core/tests/golden/`.
- Comparison: exact match on stamp geometry. On pixels, a maximum per-channel difference of 1 and a PSNR of at least 50 dB, to tolerate codec version differences.
- To regenerate, run `UPDATE_GOLDEN=1 cargo test`. The diff must be reviewed visually in the PR.

## 6. CI matrix

| Job | macOS (arm64) | Windows (x64) | Ubuntu |
|-----|:---:|:---:|:---:|
| fmt / clippy / unit / golden | ✓ | ✓ | ✓ |
| Integration (small) | ✓ | ✓ | |
| Build installer | ✓ | ✓ | |
| Docs checks ([docs.yml](../.github/workflows/docs.yml)) | | | ✓ |
| Fuzz, benchmarks (nightly) | ✓ | | ✓ |

## 7. Manual release checklist

Before tagging a release, on both `REF-MAC` and `REF-WIN`:

1. Import the `bench` corpus. Thumbnails appear within budget.
2. Export with each print size. Spot-check 10 outputs for rotation, stamp placement and colour.
3. Cancel an export midway. No partial files remain.
4. Run with the network disabled. Everything works.
5. Install over the previous version. Settings are preserved.
