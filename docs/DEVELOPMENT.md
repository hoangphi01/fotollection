# Development Environment — Fotollection

> **Status:** Draft v0.1 · **Last reviewed:** 2026-10-23
>
> Toolchain setup, everyday build commands, and **build disk hygiene**. Rust build artifacts grow quickly, so this file defines how we keep them small.
> Code conventions live in [STYLE_RULE.md](STYLE_RULE.md). Release builds are covered in [RELEASE.md](RELEASE.md).

## 1. Setting up a machine

### 1.1 Quick start (any OS)

```sh
git clone https://github.com/hoangphi01/fotollection.git
cd fotollection
scripts/bootstrap.sh --install            # macOS / Linux
# Windows (PowerShell):
#   powershell -ExecutionPolicy Bypass -File scripts\bootstrap.ps1 -Install
npm run dev                               # launches the app
```

The bootstrap script checks every prerequisite and, with `--install` / `-Install`, installs any that are missing:

- macOS: it asks you to install the Xcode CLT yourself.
- Linux: it installs the system packages with `apt`.
- Windows: it installs the tools with `winget`.

It then installs the exact dependency versions from the lockfiles. Add `--tools` / `-Tools` to also install `cargo-sweep` and `cargo-deny`.

### 1.2 What pins each version

This project's equivalent of Python's `requirements.txt` + lockfile is the set of files below. **All of them are committed.** Never install dependencies without the lockfile.

| Layer | Pinned by | Install command |
|-------|-----------|-----------------|
| Rust compiler + rustfmt/clippy | `rust-toolchain.toml` (exact version) | Automatic: rustup reads the file |
| Rust crates | `Cargo.lock` | `cargo fetch --locked` / `cargo build --locked` |
| Node.js | `.node-version` (recommended LTS) + `engines` in `package.json` (minimum) | fnm / nvm / volta read `.node-version` |
| JS packages (including the Tauri CLI) | `package-lock.json` (npm workspaces: root + `ui/`) | `npm ci` (never `npm install` on a fresh clone) |
| System libraries | `scripts/bootstrap.sh` / `bootstrap.ps1` | The bootstrap script |

### 1.3 Prerequisites

| Tool | Version | Notes |
|------|---------|-------|
| Rust | pinned in `rust-toolchain.toml` | Install `rustup`. It fetches the pinned toolchain |
| Node.js | ≥ 22 (24 LTS recommended) | Build tooling only (Vite, Tauri CLI, shadcn CLI). Not shipped |
| uv | ≥ 0.5 | Python tooling only (quality audits, icon and screenshot helpers): `uv sync` installs the versions locked in `pyproject.toml` / `uv.lock` into `.venv`, then `uv run <script>`. Not shipped |
| Tauri CLI | pinned in `package-lock.json` | A root dev dependency, run through `npm run tauri …`. No `cargo install` is needed |
| Platform toolchain | — | macOS: Xcode CLT. Windows: MSVC Build Tools + WebView2. Linux: webkit2gtk-4.1 and friends (see bootstrap) |
| HEIF + HEVC extensions (Windows) | Microsoft Store | Only to test HEIC on Windows; without them HEIC is rejected as `import.heic_codec_missing` ([ADR-0017](adr/0017-heic-through-os-codecs.md)) |
| `cargo-sweep` | latest | Used by [§4](#4-build-disk-hygiene) |
| `cargo-deny` | latest | License and advisory policy (`deny.toml`) |
| `cargo-cache` (optional) | latest | Trims `~/.cargo` |

## 2. Everyday commands

```sh
npm run dev                  # run the app (Vite + Tauri, hot-reloading UI)
npm run build                # release bundle for the host OS
npm run ui:dev               # UI only, in a browser (IPC calls fail gracefully)
npm run ui:typecheck && npm run ui:lint
cargo test --workspace       # Rust tests; also regenerates ui/src/ipc/bindings.ts
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check             # license / advisory policy
scripts/check-docs.sh        # docs consistency
scripts/clean.sh             # reclaim disk space (see §4)
# Dev only: auto-import a folder at startup (no dialog), handy for UI iteration:
FOTOLLECTION_IMPORT=~/Pictures/some-folder npm run dev
# Render one photo exactly as export does, with an info frame (crop|fit):
cargo run --release -p fotollection-core --example render_one -- in.jpg out.jpg crop 10x15
# Measure the pipeline on a real folder (read-only; photos are never copied into the repo):
cargo run --release -p fotollection-core --example measure_folder -- ~/Pictures/some-folder
# Rust vs browser benchmark (macOS; downloads Playwright Chromium once, ~1.5 GB in
# ~/Library/Caches/ms-playwright). Writes bench/results/*.json and docs/images/bench.svg:
bench/run.sh ~/Pictures/some-folder 3
# Large-batch memory check: hard-links the folder 53 times (no extra disk, same volume as
# $TMPDIR), exports with --discard-output, writes bench/results/batch-<n>.json:
bench/batch.sh ~/Pictures/some-folder 53
# Redraw every chart (docs/images/bench*.svg, English and -vi) from bench/results/*.json:
node bench/chart.mjs
```

The IPC bindings (`ui/src/ipc/bindings.ts`) are generated from Rust by `tauri-specta` on every debug run and every `cargo test`. Commit them. CI fails if they are stale.

## 3. Why Rust builds get heavy

- Each build profile (`dev`, `release`, `bench`) and each target (for example `universal-apple-darwin`) gets its own copy of every dependency in `target/`.
- Debug info is the largest part. A full-debug dev build of a Tauri app with native codecs can exceed **5–10 GB**.
- Incremental compilation caches (`target/*/incremental`) grow with every edit, and old copies are never removed.
- After a toolchain update, the artifacts from the old compiler stay on disk.
- `~/.cargo/registry` and `~/.cargo/git` keep every crate version ever downloaded.

## 4. Build disk hygiene

**Principle:** don't wipe `target/` after each build. That throws away the incremental cache, and the next build goes from seconds to minutes. Instead:

1. Make artifacts smaller up front.
2. Sweep stale artifacts regularly.
3. Do a full clean only on demand.

### 4.1 Smaller artifacts (workspace `Cargo.toml`, set at scaffold time)

```toml
[profile.dev]
debug = "line-tables-only"   # usable backtraces at a fraction of full debug info

[profile.dev.package."*"]
debug = false                # no debug info for dependencies (the biggest win)
opt-level = 2                # dependencies optimized: image codecs are usable in dev

[profile.dev.package.fotollection-core]
opt-level = 3                # our pixel loops too: unoptimized they made dev ~10× slower

[profile.release]
debug = false
strip = "symbols"
lto = "fat"
codegen-units = 1
```

- macOS uses `split-debuginfo = "unpacked"` by default, which avoids duplicating debug info.
- Universal macOS builds and release builds normally run **in CI only**. Locally, build only the host target.

### 4.2 Automatic sweep

`scripts/clean.sh` (the default level) runs `cargo sweep`. It removes artifacts that were not used in the last **7 days** and artifacts built by toolchains that are no longer installed. Run it weekly, or after a `rustup update`. To make it automatic, add it to a scheduled job (cron or launchd) or a git `post-merge` hook. It is never run after each build.

### 4.3 Cleaning levels

| Command | Removes | Next build |
|---------|---------|------------|
| `scripts/clean.sh` (`sweep`) | Artifacts unused for 7 days or from old toolchains | Unaffected |
| `scripts/clean.sh release` | `target/release`, `target/*/release`, bundles | Next release build is a full build |
| `scripts/clean.sh full` | Entire `target/`, `ui/dist`, Vite cache | Full rebuild (minutes) |
| `scripts/clean.sh deep` | `full` + `node_modules` + trim `~/.cargo` cache | Full rebuild + reinstall |

`scripts/clean.sh --dry-run <level>` shows what would be removed and how much space it would free.

### 4.4 Optional: shared build cache

If you work on several Rust projects, [`sccache`](https://github.com/mozilla/sccache) can share compiled dependencies between them, with its own size cap (`SCCACHE_CACHE_SIZE=10G`). It is not required.

## 5. CI build hygiene

- CI uses `Swatinem/rust-cache`, which already prunes stale artifacts before saving its cache.
- Caches are keyed per OS and per `Cargo.lock` hash. Old caches expire automatically after 7 days without use (GitHub's policy).
