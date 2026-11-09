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
- **Exception (owner, 2026-11-03):** v0.9.3 (image facts, colour management), v0.9.4 (postcards) v0.9.6 (export folders, share fill) and v0.9.7 (postcard face and addresses, the photo at a glance) carry new features as PATCH releases, so v0.10.0 stays the name of the "Core" release in [ROADMAP.md](ROADMAP.md). v0.13.1 (copy, paste and reset, the histogram, `J`, `W`, the welcome screen) is a feature PATCH by the owner's naming too, so v0.14.0 stays *Looks* (owner, 2026-11-06).
- **Commits suggest the bump.** With no argument, `scripts/release.sh` proposes the next version from the [Conventional Commits](https://www.conventionalcommits.org/) since the last tag: a `feat` gives MINOR, otherwise PATCH. The owner decides.
- **File formats are versioned separately.** Each persisted format has its own `schema_version` ([DATABASE.md](DATABASE.md), [EXTENSION.md](EXTENSION.md)). Release notes name the schemas that changed.
- Every release has a section in [CHANGELOG.md](../CHANGELOG.md).
- **Each MINOR series has a title (owner, 2026-11-07).** Above its first version, the CHANGELOG has a rule (`---`) and `## vX.Y — <title>` (v0.13 — The interface); `release.sh` writes it for an `X.Y.0` from that version's name in [ROADMAP.md](ROADMAP.md). The website's *What's new* shows it as a larger title under a line in the brand colour; the release notes stop at it.

## 2. Artifacts

| Platform | Target | Artifact |
|----------|--------|----------|
| macOS 11+ | `universal-apple-darwin` (arm64 + x86_64) | `.dmg` |
| Windows 10/11 | `x86_64-pc-windows-msvc` | `.msi` and NSIS `-setup.exe` |
| Linux, glibc 2.35+ (since v0.8.2, [ADR-0034](adr/0034-linux-lean.md)) | `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, built on `ubuntu-22.04` and `ubuntu-22.04-arm` | `_amd64.AppImage`, `_aarch64.AppImage`, `_amd64.deb`, `_arm64.deb` |
| All | — | `SHA256SUMS.txt` |

**AppStream metadata** (since v0.13.0): the `.deb` and the AppImage carry `/usr/share/metainfo/com.hoangphi01.fotollection.metainfo.xml` (`src-tauri/linux/`), so Linux software centres (GNOME Software, KDE Discover, the VNLF App Explorer, CaramOS's App Center, Flathub later) show its name, words in English and Vietnamese, and the screenshots at `fotollection.org/assets/img/catalog/`. `scripts/release.sh` adds each release to its list; CI validates it with `appstreamcli`.

The WebView2 runtime is bootstrapped by the Windows installer when it is missing. Installer size is budgeted by [NFR-008](PRD.md#nfr-008--install-size).

**Not on crates.io** (owner, 2026-11-09). The names `fotollection` and `fotollection-core` were free that day, but crates.io's usage policy removes a crate that "exists only to reserve a name … without any genuine functionality", and the app itself does not install through `cargo install` (a Tauri app with its built interface). The name is to be protected by a registered trademark instead, a decision still open with the owner (with relicensing, ADR-0008). Publishing the real core library later means publishing its local crates first (`fotollection-oscodec`, the RAW crates, `geodata`), each with a version and under crates.io's 10 MB per package, and moving the large data out of the package; the owner runs `cargo login` and approves the last `cargo publish`.

## 3. Code signing

**Current policy: ship unsigned, without paid certificates** ([ADR-0009](adr/0009-unsigned-distribution.md)).

| Platform | What we do now | Paid option (not used) | User experience |
|----------|----------------|------------------------|-----------------|
| macOS | Ad-hoc signature (required on Apple Silicon) | Developer ID + notarization, 99 USD/year | Gatekeeper blocks the first launch. The user goes to **System Settings → Privacy & Security → Open Anyway** (right-click → Open no longer works on macOS 15+) |
| Linux | Unsigned, as is usual for AppImage and a downloaded `.deb`; `SHA256SUMS.txt` covers them | — | Nothing to approve: `chmod +x` the AppImage, or `sudo apt install ./….deb` |
| Windows | Unsigned. SignPath Foundation (free for OSS) declined on 2026-10-26; reapply later | Authenticode / Azure Trusted Signing | SmartScreen warning: **More info → Run anyway** |

- Every release publishes `SHA256SUMS.txt` for all artifacts.
- The release notes, the website and the README include the first-launch steps for each OS.
- No signing secret exists yet. Any future one is stored only in GitHub Actions encrypted secrets, never in the repository.
- **SignPath Foundation (declined 2026-10-26).** The reply: the Foundation looks for external signs of public trust (stars, forks, contributors, independent articles or discussions, sustained activity) and the project did not show enough yet; reapplying later is welcome. The paid SignPath plan is not worth it on this budget. For the next application: its [terms](https://signpath.org/terms) require: an OSI licence with **no commercial dual-licensing** (a relicence under [ADR-0008](adr/0008-license-gpl3-with-relicensing-option.md) ends the subscription), builds only from this repository by GitHub Actions, two-factor authentication on every account, a manual approval of each signing request, product name and version identical in every signed file, and a *Code signing policy* page linked from the home and download pages with the privacy policy (the page exists: [fotollection.org/code-signing](https://fotollection.org/code-signing/), `site/_includes/code-signing.html`; add the SignPath attribution and a link near the download buttons when approved). If approved, `release.yml` submits the unsigned `-setup.exe` and `.msi` to SignPath, waits for the approval and publishes the signed files; a new ADR then supersedes this part of ADR-0009.
- **Plan:** pay for the Apple Developer Program only when the project earns money or an iPhone app is built; there is no free way to notarize for macOS (a free Apple ID signs only for the developer's own devices, for 7 days).

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
3. **build:** `tauri-action` on `macos-latest` (universal, ad-hoc signed), `windows-latest`, `ubuntu-22.04` and `ubuntu-22.04-arm`. The Linux jobs then run a **smoke test**: install the `.deb` and check its desktop entry and MIME types, and run the AppImage for 20 s under `xvfb-run`, on the runner and in a `debian:12` container given GTK 3 and Mesa (GL, EGL, GLES), the libraries every desktop has and an AppImage leaves out; the container first lists any library the AppImage still lacks. The installers are uploaded before the test and every system finishes, so a failed run still leaves them to try by hand. A crash fails the release ([ADR-0034](adr/0034-linux-lean.md)).
4. **publish:** tags the commit, creates the GitHub Release **published** (pre-release when the version has a `-`), titled `Fotollection vX.Y.Z`, followed by the release's name when the CHANGELOG heading has one after the date (`## [0.8.0] - 2026-10-27 — Places and Lightroom`; add it to the heading `release.sh` writes, before pushing), with the version's **Highlights** from the CHANGELOG (a few one-line points such as "New: …", "UI update: …", "Fixes: …"), a link to the full changelog, the first-launch steps and `SHA256SUMS.txt`. The CHANGELOG itself keeps every detail.
5. **website:** triggers the Pages workflow, so the download buttons show the new version.

A failed build publishes nothing. Re-running the workflow retries it. A fix pushed to `main` does not start it again unless it touches `Cargo.toml`; start it by hand (`gh workflow run release --ref main`). Run `cargo deny check advisories` before `scripts/release.sh`: a crate yanked since the last CI stops the `security` check ([LESSONS.md](LESSONS.md)).

**A release from a maintenance branch** (a fix or a package for the last version while `main` holds the next one, as v0.8.1 and v0.8.2 on `release/0.8`): `release.sh` refuses any branch but `main`, so set the version by hand as it does (`Cargo.toml`, `cargo update --workspace`, `npm version X.Y.Z --workspace ui --no-git-tag-version`, the CHANGELOG heading), commit `chore(release): vX.Y.Z`, push the branch and run `gh workflow run release.yml --ref <branch>`; the tag goes on the branch's commit. Then merge the branch into `main`, and **push `main` only once the tag exists**: `main`'s `Cargo.toml` then carries the same version, and a push without the tag would release it from `main`.

**Which workflows run on a push.** Each one runs only when its own files change, so a docs-only or website-only push builds nothing:

| Workflow | Runs when the push changes |
|----------|----------------------------|
| `ci` (fmt, clippy, tests, bindings on macOS, Windows and Ubuntu 22.04; `cargo deny`) | `crates/`, `src-tauri/`, `ui/`, `Cargo.toml`, `Cargo.lock`, the npm manifests, `rust-toolchain.toml`, `deny.toml` |
| `docs` (`check-docs.sh`, markdownlint, internal links) | any `.md` file, `docs/`, `bench/results/`, and the files `check-docs.sh` compares with the docs (versions, `tauri.conf.json`) |
| `pages` (the website) | `site/`, `CHANGELOG.md`; and after every release |
| `release` | `Cargo.toml` (or started by hand from the Actions tab, for example to retry a failed release) |
| `installers` (MSI and NSIS on Windows, nothing published) | `src-tauri/windows/`, `tauri.conf.json`: installer settings are checked before a release, which `ci` cannot do |

Pull requests use the same filters. After pushing, check that every workflow that started is green (`gh run list --limit 5`), not only the one you expected.

## 5. Updates

There is **no in-app updater** ([FR-021](PRD.md#fr-021--manual-updates), [ADR-0021](adr/0021-versioning-and-automated-releases.md)). Settings → Updates has a **Check for updates** button, and from v0.13.0 the app also checks once at start unless that is turned off ([FR-061](PRD.md#fr-061--check-for-updates), [FR-103](PRD.md#fr-103--check-for-updates-at-start-never-download-unasked), [ADR-0025](adr/0025-update-check-on-click.md)); a newer installer opens in the browser. New versions are also announced on the Releases page and the website.

**The minimum safe version.** When a release fixes a fault serious enough that older versions should not be used (a lost project, a crash, wrong prints), raise `minSafe` in `.github/update.json` in that release's commit and write the reason in both languages, one plain sentence each, without `-->`. `release.yml` appends the file to the release notes as a hidden comment; every app from v0.13.0 that starts below it shows a red warning with the reason. Never lower it, and never set it above the version being released (a test, `fr_103_the_repository_note_warns_below_it`, checks it). The reason is short and plain, for people who are not technical: *Security patches. Please update.* (owner, 2026-11-08); the details belong in the CHANGELOG. **Forgotten at release:** set it in `.github/update.json` on `main` and on the release's branch, then replace only the `<!-- fotollection-update … -->` line of the published release's notes (`gh release edit vX.Y.Z --notes-file`); apps read the latest release, so the warning shows at their next start.

Installing a new version over an old one **replaces it** and keeps settings, profiles and projects, which are migrated on first open ([DATABASE.md](DATABASE.md)):

- **macOS:** drag the new app into Applications and choose *Replace*. Running it from the disk image or Downloads replaces nothing.
- **Windows `-setup.exe` (recommended):** the reinstall page defaults to uninstalling the old version first; app data is kept unless its box is ticked.
- **Windows `.msi`:** `MajorUpgrade` removes the old version and refuses downgrades. This needs the same `UpgradeCode` in every release, so it is pinned in `tauri.conf.json` (`bundle.windows.wix.upgradeCode`); never change it.
- Do not mix the two Windows installers: one installs per user, the other per machine, so both copies would stay.
- **Linux AppImage:** replace the old file with the new one (*Check for updates* downloads it). **Linux `.deb`:** `sudo apt install ./Fotollection_X.Y.Z_amd64.deb` over the old one.

## 6. Release checklist

- [ ] CHANGELOG `Unreleased` describes the release in user terms; an `X.Y.0` gets its series' rule and title (`release.sh` writes it from ROADMAP)
- [ ] A security fix or a serious fault (lost work, a crash, wrong prints): `minSafe` raised to this version in `.github/update.json`, with a one-line plain reason in both languages (§5)
- [ ] `scripts/ui-check.sh` passes the whole matrix ([UI_TESTS.md](UI_TESTS.md)); changed pictures approved on purpose
- [ ] Benchmarks are within budget, and the results are logged in [OPTIMIZATION.md §5](OPTIMIZATION.md#5-benchmark-log)
- [ ] An `X.Y.0`: its milestone in the website's Wiki (`site/_data/timeline.yml` and `scripts/timeline-shot.sh`, DOCS_GUIDE "A release")
- [ ] `scripts/release.sh` run, commit pushed
- [ ] Workflow green, Linux smoke tests included; the release has the `.dmg`, `.msi`, `-setup.exe`, the two AppImages, the two `.deb` and `SHA256SUMS.txt`
- [ ] Smoke test of the published installer on macOS ([TESTING.md §7](TESTING.md#7-manual-release-checklist)); a broken release is fixed by a PATCH release, never by replacing assets

The website shows GitHub's download counts as they are (`scripts/download-stats.mjs`): the smoke test and other checks of a release add a download or two to each file.
