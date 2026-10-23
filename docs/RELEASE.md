# Release & Distribution — Fotollection

> **Status:** Draft v0.1 · **Last reviewed:** 2026-10-23
>
> How Fotollection is built, signed and shipped. No servers are involved: releases are hosted on GitHub Releases.

## 1. Versioning

- [Semantic Versioning](https://semver.org/). Releases stay on `0.x` until the MVP feature set ([ROADMAP.md](ROADMAP.md#phase-1--mvp)) is complete and stable.
- The version lives in the workspace `Cargo.toml`. `tauri.conf.json` and `ui/package.json` are kept in sync by a release script.
- Every release has a section in [CHANGELOG.md](../CHANGELOG.md).

## 2. Artifacts

| Platform | Target | Artifact |
|----------|--------|----------|
| macOS 11+ | `universal-apple-darwin` (arm64 + x86_64) | `.dmg` + `.app.tar.gz` (updater) |
| Windows 10/11 | `x86_64-pc-windows-msvc` | `.msi` and NSIS `-setup.exe` |

The WebView2 runtime is bootstrapped by the Windows installer when it is missing. Installer size is budgeted by [NFR-008](PRD.md#nfr-008--install-size).

## 3. Code signing

| Platform | Requirement | Cost | Without it |
|----------|-------------|------|------------|
| macOS | Apple Developer ID certificate + notarization (`notarytool`) | 99 USD/year | Gatekeeper blocks first launch. The user must right-click → Open, or allow the app in System Settings |
| Windows | Authenticode certificate (OV/EV) or Azure Trusted Signing | ~10 USD/month (Azure) and up | SmartScreen shows "Unknown publisher" until the app builds reputation |
| Updater | Tauri updater key pair (minisign) | Free | **Mandatory**: updates are rejected without a valid signature |

Signing secrets are stored only in GitHub Actions encrypted secrets, never in the repository. Whether to buy certificates is an open decision ([STATUS.md](STATUS.md#open-decisions)). Early `0.x` builds may ship unsigned, with instructions.

## 4. Build pipeline

`.github/workflows/release.yml` will be created in Phase 0. It is triggered by pushing a `v*` tag:

1. Run the full CI (fmt, clippy, tests, docs checks, `cargo deny`).
2. Build on `macos-latest` (universal) and `windows-latest`, using `tauri-action`.
3. Sign and notarize (macOS) and sign (Windows) when secrets are present.
4. Generate the updater manifest `latest.json`, signed with the updater key.
5. Create a **draft** GitHub Release with the artifacts and the CHANGELOG section.
6. A human smoke-tests the draft ([TESTING.md §7](TESTING.md#7-manual-release-checklist)), then publishes it.

## 5. Auto-update

The updater ([FR-021](PRD.md#fr-021--auto-update)) uses `tauri-plugin-updater` with the `latest.json` endpoint on GitHub Releases. It is **opt-in** ([NFR-006](PRD.md#nfr-006--offline-and-zero-telemetry)): the first-run dialog asks the user, and the choice can be changed in Settings.

## 6. Release checklist

- [ ] All P0 FRs for the milestone are **Done** in [STATUS.md](STATUS.md)
- [ ] Benchmarks are within budget, and the results are logged in [OPTIMIZATION.md §5](OPTIMIZATION.md#5-benchmark-log)
- [ ] CHANGELOG `Unreleased` section moved under the new version
- [ ] Version bumped consistently
- [ ] Tag pushed, draft release smoke-tested on macOS and Windows
- [ ] Release published
