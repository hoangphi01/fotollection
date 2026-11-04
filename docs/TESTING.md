# Testing Strategy — Fotollection

> **Status:** Draft v0.1 · **Last reviewed:** 2026-11-02
>
> How we prove that Fotollection is correct, robust and fast. Performance benchmarks are specified in [OPTIMIZATION.md](OPTIMIZATION.md).

## 1. Test pyramid

**Today:**

| Level | Scope | Tooling | Runs |
|-------|-------|---------|------|
| Unit | Pure functions in `crates/core`, `crates/rawcore`, `crates/oscamera` and `src-tauri` (crop math, stamp layout, date formatting, path sanitizing, geocode lookup, PTP answers), named `fr_NNN_…` | `cargo test` | Every push that changes code (RELEASE.md §4), on macOS, Windows and Ubuntu ([ci.yml](../.github/workflows/ci.yml)) |
| Integration | `crates/core/tests/pipeline.rs` (import → metadata → library → export on synthetic files) and `tests/jpeg_layouts.rs` (every JPEG layout decodes to the picture, [ADR-0033](adr/0033-reference-jpeg-decoder.md)) | `cargo test` | Every push that changes code |
| UI checks | Types, lint (bans `dangerouslySetInnerHTML` / `eval`), the same i18n keys in both languages, the production build | `tsc -b`, `oxlint`, `ui/scripts/check-i18n.mjs`, `vite build` | Every push that changes code |
| Fuzz | RAW decoder: mutation testing on stable Rust (§4) | `crates/rawcore/examples/mutate.rs` | By hand, before a RAW format moves to our own decoder |
| Benchmark | Budgets from [OPTIMIZATION.md §3](OPTIMIZATION.md#3-budgets) | `crates/core/examples/measure_*`, `bench/run.sh`, `bench/batch.sh`, `scripts/measure-startup.sh` | By hand, logged in [OPTIMIZATION.md §5](OPTIMIZATION.md#5-benchmark-log) |
| Release smoke test | The installed Linux packages (`.deb` file types, the AppImage runs 20 s, also on Debian 12) | [release.yml](../.github/workflows/release.yml) | Every release |
| Manual | §7 checklist | By hand | Before each release |

**Planned** (none of these exists yet):

| Level | Scope | Tooling |
|-------|-------|---------|
| Property tests | Crop math, path sanitizing | `proptest` |
| UI component | React components and hooks, with IPC mocked (no `*.test.ts` file yet) | `vitest` + Testing Library |
| Golden image | Full `RenderPlan` → pixels for fixed inputs, compared to reference outputs (§5) | `cargo test` + image diff |
| Integration, full set | The pipeline on the whole sample corpus (§3) | `cargo test --test pipeline`, nightly |
| Fuzz | Decoders, EXIF parser, path sanitizer, `.cube` parser (§4) | `cargo-fuzz`, nightly |
| E2E | Real app window: import, adjust, export, cancel | Tauri WebDriver (`tauri-driver`), on macOS and Windows before each release |
| Benchmark in CI | The §3 budgets checked automatically (`criterion` is a dev-dependency, but no `benches/` exist yet) | `criterion`, nightly |

## 2. Requirement traceability

- Every FR has at least one test named `fr_NNN_<behaviour>` ([STYLE_RULE.md §2](STYLE_RULE.md#naming)).
- An FR can be marked **Done** in [STATUS.md](STATUS.md) only after its test exists and passes. Once code exists, CI enforces this through `scripts/check-docs.sh`.
- Every NFR with a numeric budget has a matching benchmark or E2E probe, listed in [OPTIMIZATION.md §3](OPTIMIZATION.md#3-budgets).

## 3. Sample corpus

The corpus is stored **outside** git, because the photos are personal and large. It is fetched by `scripts/fetch-corpus.sh` from a private location. A tiny, license-clean subset is committed under `crates/core/tests/fixtures/`.

| Subset | Contents | Purpose |
|--------|----------|---------|
| Synthetic (generated) | `crates/core/tests/common/mod.rs` builds JPEGs with chosen EXIF (date, GPS, orientation) through `jpeg-encoder` + `little_exif`, plus broken and spoofed files, at test time. **Nothing personal is committed** | Unit, pipeline and security tests (implemented) |
| `fixtures` (in git) | `crates/core/tests/fixtures/`: one project file per schema version (`project/v1.fotollection`, `v3`…`v9`), one settings file per version (`settings/v1…v9.json`) and a v1 bundle, so every migration is tested. **Planned:** ~20 small photos (each orientation 1–8, no-EXIF, (0,0) GPS, HEIC, PNG, wide-gamut ICC, CMYK, polyglot) | Migration tests today; golden and security tests later |
| JPEG layouts (in git) | `crates/core/tests/fixtures/jpeg/`: one drawn picture saved by `cjpeg`, Pillow or `ycck.c` in 23 layouts (4:4:4, 4:2:2, 4:2:0, 4:4:0, 4:1:1, progressive, restart markers, optimised tables, one scan per component, a progressive file with a split DC scan, grey, lossless 16-bit grey and 8-bit colour, Adobe CMYK and YCCK, extended SOF1), plus two that must be refused (arithmetic coding, a file cut short); `tests/jpeg_layouts.rs` compares each decode with the picture by its mean and 99th-percentile error and checks each file's route (zune, the reference decoder, refused), so a decoder upgrade that breaks a layout, or a part of one, fails ([ADR-0033](adr/0033-reference-jpeg-decoder.md)) | Decoder regression tests |
| ICC profiles (in git) | `crates/core/tests/fixtures/icc/`: Display P3 and Adobe RGB from the CC0 Compact ICC Profiles; `colour::tests` checks the conversion against the published matrices (±2 levels), `tests/colour_screen.rs` that a tagged photo is sRGB in its thumbnail, preview and zoom (FR-072) | Colour regression tests |
| `bench` | 200 × 20 MP JPEG (Lumix GX9), 100 × 12 MP HEIC (iPhone), 20 × PNG | Benchmarks |
| Real trip (`PRAGUE 11.26`) | The owner's Prague folder as it came off the card, stored outside git (© the owner, all rights reserved): 396 × 20 MP JPEG (Lumix GX9, 4:2:2) and 26 RW2 twins | `bench/trip.sh`, the whole pipeline on a real folder ([RESULT.md §20](RESULT.md#20-a-real-trip-prague-gx9)); 25 of its photos are in the website's screenshots (`site/screenshots/demo_set.txt`) |
| `stress` | 2,000+ mixed files | Memory ceiling, long-run stability |
| Print-quality set | 20 photos of the same folder, plus Display P3 versions made with macOS's profile at test time | `bench/quality/audit.py` against the source, pixel by pixel ([RESULT.md §9](RESULT.md#9-print-quality)) |
| Labelled best-shot set | 95 photos of the owner's (`103___09`), stored outside git; hand labels in `bench/bestshot/truth-103_09.json` (file names only) | Accuracy of best-shot grouping and picks with `eval_bestshot` ([RESULT.md §8](RESULT.md#8-best-shot-accuracy), [ADR-0019](adr/0019-grouping-by-visual-signature.md)) |

## 4. Fuzzing

**Today:** `crates/rawcore/examples/mutate.rs` runs on stable Rust: for a given time it cuts the CC0 RAW samples short, flips bytes, writes extreme values into their directories and damages their compressed data, decodes each result, and prints the seed of any panic so it can be replayed. A format moves to our own decoder only after 10 minutes without one ([ADR-0030](adr/0030-our-own-raw-decoder.md)):

```sh
cargo run --release -p fotollection-rawcore --example mutate -- 600 path/to/samples/*.dng
```

`cargo test -p fotollection-rawcore` with `FOTOLLECTION_RAW_SAMPLES` set also cuts every sample short at 256 places.

**Planned** (`cargo-fuzz` needs a nightly toolchain, not installed yet; `crates/core/fuzz/` does not exist yet):

| Target | Input |
|--------|-------|
| `fuzz_metadata` | Arbitrary bytes → metadata parser |
| `fuzz_decode_jpeg` | Arbitrary bytes → JPEG decode at T1 scale |
| `fuzz_decode_heic` | Arbitrary bytes → HEIC decode |
| `fuzz_sanitize_path` | Arbitrary strings → export path builder; asserts the result stays under the root |
| `fuzz_cube` (with FR-070) | Arbitrary bytes → `.cube` LUT parser; rejects without panicking ([ADR-0036](adr/0036-colour-looks-cube-luts-first.md)) |

Any crash found by fuzzing is added to `fixtures` as a regression test.

## 5. Golden images

**Planned:** none exist yet, and `crates/core/tests/golden/` does not exist. The plan:

- Reference outputs live in `crates/core/tests/golden/`.
- Comparison: exact match on stamp geometry. On pixels, a maximum per-channel difference of 1 and a PSNR of at least 50 dB, to tolerate codec version differences.
- To regenerate, run `UPDATE_GOLDEN=1 cargo test`. The diff must be reviewed visually in the PR.
- **Looks (FR-070, planned):** one golden image per built-in look, and an identity LUT must leave a photo unchanged.

## 6. CI matrix

| Job | macOS (arm64) | Windows (x64) | Ubuntu |
|-----|:---:|:---:|:---:|
| fmt / clippy / unit / integration (`cargo test`) | ✓ | ✓ | ✓ (22.04, since v0.8.2) |
| UI typecheck, lint, i18n keys, build | ✓ | ✓ | ✓ |
| Build installer ([release.yml](../.github/workflows/release.yml); Windows MSI and NSIS also in [installers.yml](../.github/workflows/installers.yml) when their configuration changes) | ✓ (release) | ✓ | ✓ (release: x86_64 and ARM64) |
| Installed-package smoke test (`.deb` file types, AppImage runs 20 s, also on Debian 12) | | | ✓ (release) |
| Docs checks ([docs.yml](../.github/workflows/docs.yml)) | | | ✓ |
| Fuzz, golden images, benchmarks (nightly): **planned**, no nightly job exists (the only scheduled workflow is the website's `pages.yml`) | | | |

## 7. Manual release checklist

Before tagging a release, on both `REF-MAC` and `REF-WIN`. Linux has no reference machine yet: the release's smoke test stands in for it ([ADR-0034](adr/0034-linux-lean.md)), and a Linux user's report is welcome.

1. Import the `bench` corpus. Thumbnails appear within budget.
2. Export with each print size. Spot-check 10 outputs for rotation, stamp placement and colour.
3. Cancel an export midway. No partial files remain.
4. Run with the network disabled. Everything works.
5. Install over the previous version. Settings are preserved.
6. Switch between Simple, Collage and Pro, with the menu and Cmd/Ctrl+1/2/3: the photos, basket and settings stay, and nothing of one mode shows in another.
7. Print one collage sheet at a shop, with and without the cut line, and one print with the 3 mm border: measure the photo areas and the border against the app's figures.
8. Colour (FR-072): open an iPhone HEIC and a JPEG from a camera set to Adobe RGB, next to the system's photo viewer. Thumbnail, preview, zoom and crop view show the same colours as the viewer, not paler. On Windows this is the only check of WIC handing over the HEIC's profile (`oscodec` `profile`); the preview's Image line says *Display P3*.
9. Postcard (FR-073): print one landscape and one portrait postcard at a kiosk (dm or Rossmann). The stamp box measures 26 × 32 mm; write in it with a fine permanent marker and with a ballpoint, and note which dries and which smears (RESULT).
