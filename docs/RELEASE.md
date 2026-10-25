# Release & Distribution — Fotollection

> **Status:** Accepted v0.2 · **Last reviewed:** 2026-10-24
>
> How Fotollection is versioned, built and shipped. No servers are involved: releases are hosted on GitHub Releases, and the website on GitHub Pages ([ADR-0021](adr/0021-versioning-and-automated-releases.md)).

## 1. Versioning

- **One [Semantic Versioning](https://semver.org/) version** for the whole app. The workspace `Cargo.toml` is the source; `ui/package.json` and `Cargo.lock` follow it, and `tauri.conf.json` has none of its own. `scripts/check-docs.sh` fails when they disagree.
- **0.x until the MVP is complete** ([ROADMAP.md](ROADMAP.md#phase-1--mvp)):
  - **MINOR** (`0.2.0`): new features. Each work stage ships as one minor release.
  - **PATCH** (`0.2.1`): fixes only.
- **From 1.0.0:**
  - **MAJOR:** a change that breaks users, such as a project file that can no longer be migrated, a dropped operating system, or a workflow that changes meaning.
  - **MINOR:** new features, backwards compatible.
  - **PATCH:** fixes only.
- **Pre-releases** are `X.Y.Z-beta.N` and are marked as pre-releases on GitHub. They ship the Windows `-setup.exe` only, because an MSI version cannot carry a pre-release tag.
- **Commits suggest the bump.** With no argument, `scripts/release.sh` proposes the next version from the [Conventional Commits](https://www.conventionalcommits.org/) since the last tag: a `feat` gives MINOR, otherwise PATCH. The owner decides.
- **File formats are versioned separately.** Each persisted format has its own `schema_version` ([DATABASE.md](DATABASE.md), [EXTENSION.md](EXTENSION.md)). Release notes name the schemas that changed.
- Every release has a section in [CHANGELOG.md](../CHANGELOG.md).

## 2. Artifacts

| Platform | Target | Artifact |
|----------|--------|----------|
| macOS 11+ | `universal-apple-darwin` (arm64 + x86_64) | `.dmg` |
| Windows 10/11 | `x86_64-pc-windows-msvc` | `.msi` and NSIS `-setup.exe` |
| All | — | `SHA256SUMS.txt` |

The WebView2 runtime is bootstrapped by the Windows installer when it is missing. Installer size is budgeted by [NFR-008](PRD.md#nfr-008--install-size).

## 3. Code signing

**Current policy: ship unsigned, without paid certificates** ([ADR-0009](adr/0009-unsigned-distribution.md)).

| Platform | What we do now | Paid option (not used) | User experience |
|----------|----------------|------------------------|-----------------|
| macOS | Ad-hoc signature (required on Apple Silicon) | Developer ID + notarization, 99 USD/year | Gatekeeper blocks the first launch. The user goes to **System Settings → Privacy & Security → Open Anyway** (right-click → Open no longer works on macOS 15+) |
| Windows | Unsigned. Apply to SignPath Foundation (free for OSS) later | Authenticode / Azure Trusted Signing | SmartScreen warning: **More info → Run anyway** |

- Every release publishes `SHA256SUMS.txt` for all artifacts.
- The release notes, the website and the README include the first-launch steps for each OS.
- No signing secret exists yet. Any future one is stored only in GitHub Actions encrypted secrets, never in the repository.
- **Plan:** apply to SignPath Foundation for free Windows signing now that public releases exist. Pay for the Apple Developer Program only when the project earns money or an iPhone app is built; there is no free way to notarize for macOS (a free Apple ID signs only for the developer's own devices, for 7 days).

## 4. Release pipeline

A release is **one commit that changes the version**, pushed to `main`:

```sh
scripts/release.sh            # proposes the next version from the commits since the last tag
scripts/release.sh 0.2.0      # or name it
scripts/release.sh 0.2.0 --dry-run   # show the changes, write nothing
git push
```

`release.sh` checks that the tree is clean and on `main`, sets the version in `Cargo.toml`, `Cargo.lock` and `ui/package.json`, turns `## [Unreleased]` in the CHANGELOG into `## [X.Y.Z] - date` under a fresh empty `Unreleased`, and commits `chore(release): vX.Y.Z`. It never pushes.

`.github/workflows/release.yml` runs on every push to `main` that changes `Cargo.toml`, which every release commit does (it can also be started by hand from the Actions tab):

1. **detect:** reads the version from `Cargo.toml`. If the tag `vX.Y.Z` exists, it stops. A dependency change in `Cargo.toml` therefore starts it but releases nothing.
2. **check:** the full CI (`ci.yml`, called as a reusable workflow): fmt, clippy, tests, bindings, `cargo deny`.
3. **build:** `tauri-action` on `macos-latest` (universal, ad-hoc signed) and `windows-latest`.
4. **publish:** tags the commit, creates the GitHub Release **published** (pre-release when the version has a `-`), with the version's **Highlights** from the CHANGELOG (a few one-line points such as "New: …", "UI update: …", "Fixes: …"), a link to the full changelog, the first-launch steps and `SHA256SUMS.txt`. The CHANGELOG itself keeps every detail.
5. **website:** triggers the Pages workflow, so the download buttons show the new version.

A failed build publishes nothing. Re-running the workflow retries it.

**Which workflows run on a push.** Each one runs only when its own files change, so a docs-only or website-only push builds nothing:

| Workflow | Runs when the push changes |
|----------|----------------------------|
| `ci` (fmt, clippy, tests, bindings, `cargo deny` on macOS and Windows) | `crates/`, `src-tauri/`, `ui/`, `Cargo.toml`, `Cargo.lock`, the npm manifests, `rust-toolchain.toml`, `deny.toml` |
| `docs` (`check-docs.sh`, markdownlint, internal links) | any `.md` file, `docs/`, `bench/results/`, and the files `check-docs.sh` compares with the docs (versions, `tauri.conf.json`) |
| `pages` (the website) | `site/`, `CHANGELOG.md`; and after every release |
| `release` | `Cargo.toml` (or started by hand from the Actions tab, for example to retry a failed release) |
| `installers` (MSI and NSIS on Windows, nothing published) | `src-tauri/windows/`, `tauri.conf.json`: installer settings are checked before a release, which `ci` cannot do |

Pull requests use the same filters. After pushing, check that every workflow that started is green (`gh run list --limit 5`), not only the one you expected.

## 5. Updates

There is **no in-app updater** ([FR-021](PRD.md#fr-021--manual-updates), [ADR-0021](adr/0021-versioning-and-automated-releases.md)). The app never checks for new versions, so it makes no network call of its own ([NFR-006](PRD.md#nfr-006--offline-and-zero-telemetry)). New versions are announced on the Releases page and the website, and the app shows its version at the foot of the settings panel. Installing a new version over an old one keeps settings, profiles and projects, which are migrated on first open ([DATABASE.md](DATABASE.md)).

## 6. Release checklist

- [ ] CHANGELOG `Unreleased` describes the release in user terms
- [ ] Benchmarks are within budget, and the results are logged in [OPTIMIZATION.md §5](OPTIMIZATION.md#5-benchmark-log)
- [ ] `scripts/release.sh` run, commit pushed
- [ ] Workflow green; the release has the `.dmg`, `.msi`, `-setup.exe` and `SHA256SUMS.txt`
- [ ] Smoke test of the published installer on macOS ([TESTING.md §7](TESTING.md#7-manual-release-checklist)); a broken release is fixed by a PATCH release, never by replacing assets
