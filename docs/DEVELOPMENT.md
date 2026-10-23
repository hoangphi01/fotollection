# Development Environment — Fotollection

> **Status:** Draft v0.1 · **Last reviewed:** 2026-10-23
>
> Toolchain setup, everyday build commands, and **build disk hygiene**. Rust build artifacts grow quickly, so this file defines how we keep them small.
> Code conventions live in [STYLE_RULE.md](STYLE_RULE.md). Release builds are covered in [RELEASE.md](RELEASE.md).

## 1. Prerequisites

| Tool | Version | Notes |
|------|---------|-------|
| Rust | stable, pinned in `rust-toolchain.toml` | Installed via `rustup` |
| Node.js | LTS | Used only for UI tooling (Vite, shadcn CLI). It is not shipped |
| Tauri prerequisites | — | macOS: Xcode Command Line Tools. Windows: MSVC Build Tools + WebView2 |
| vcpkg (Windows) | latest | Static libheif ([ADR-0005](adr/0005-input-formats-jpeg-heic-png.md)) |
| `cargo-sweep` | latest | `cargo install cargo-sweep`, used by [§4](#4-build-disk-hygiene) |
| `cargo-cache` (optional) | latest | `cargo install cargo-cache`, trims `~/.cargo` |

Commands will be finalized when the project is scaffolded in Phase 0.

## 2. Everyday commands

```sh
cargo tauri dev              # run the app with hot-reloading UI
cargo test --workspace       # all Rust tests
cargo bench -p fotollection-core
scripts/check-docs.sh        # docs consistency
scripts/clean.sh             # reclaim disk space (see §4)
```

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
| `scripts/clean.sh deep` | `full` + `ui/node_modules` + trim `~/.cargo` cache | Full rebuild + reinstall |

`scripts/clean.sh --dry-run <level>` shows what would be removed and how much space it would free.

### 4.4 Optional: shared build cache

If you work on several Rust projects, [`sccache`](https://github.com/mozilla/sccache) can share compiled dependencies between them, with its own size cap (`SCCACHE_CACHE_SIZE=10G`). It is not required.

## 5. CI build hygiene

- CI uses `Swatinem/rust-cache`, which already prunes stale artifacts before saving its cache.
- Caches are keyed per OS and per `Cargo.lock` hash. Old caches expire automatically after 7 days without use (GitHub's policy).
