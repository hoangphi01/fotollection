# ADR-0025: Check for updates only when the user clicks, never install

- **Status:** Accepted. Supersedes [ADR-0021](0021-versioning-and-automated-releases.md) in part (its "no update check" rule); its releases, versioning and "no in-app updater" stay
- **Date:** 2026-10-25
- **Related:** FR-021, FR-061, NFR-006, ADR-0009, ADR-0021

## Context

Without an updater, people learn about a new version only from the website or GitHub. The owner asked for a way to get the newer version from inside the app, and for installing it to replace the old one cleanly.

Checked first, the installers already replace the old version:

- **macOS:** dragging the new `Fotollection.app` into Applications asks to replace the old one.
- **Windows NSIS (`-setup.exe`):** on an upgrade its first page defaults to uninstalling the old version before installing; app data is deleted only if the user ticks that box.
- **Windows MSI:** WiX `MajorUpgrade` removes the older version and blocks downgrades, as long as the `UpgradeCode` stays the same, so it is now pinned in `tauri.conf.json`.

What remained was telling the user that a newer version exists.

## Options considered

1. **`tauri-plugin-updater`:** downloads, verifies and installs by itself. It needs a private signing key kept forever (losing it ends updates), a manifest, and the app calling the network; [ADR-0021](0021-versioning-and-automated-releases.md) declined that.
2. **An automatic check at start-up**, even as an opt-in: the app would go online without a click.
3. **A button:** one request to GitHub when the user clicks *Check for updates*; a newer version opens its installer in the browser.

## Decision

**Option 3.** `src-tauri/src/updates.rs` (never the core) sends one HTTPS GET to `api.github.com/repos/hoangphi01/fotollection/releases/latest` with `ureq` (rustls), a 10 s timeout and a 1 MB cap, only from the button. The answer is parsed into a fixed type; versions are compared with `semver`; the installer for the system (`_universal.dmg`, `_x64-setup.exe`) is chosen, and every link must start with the project's releases URL. The link stays in Rust; *Download* opens it through `tauri-plugin-opener`'s Rust API, and the webview gets no opener permission. Nothing is downloaded or run by the app.

## Consequences

- The app still never goes online on its own ([NFR-006](../PRD.md#nfr-006--offline-and-zero-telemetry)); a click sends GitHub the user's IP address and the app's version in the User-Agent, which the button's note says.
- No secret to keep; integrity still rests on GitHub's HTTPS and the published checksums.
- Users install the new version themselves, which replaces the old one and keeps settings and projects. Mixing the NSIS and MSI installers installs two copies, so the site and guides recommend `-setup.exe`.
- A debug build can pretend to be older with `FOTOLLECTION_FAKE_VERSION`, to test the "newer version" path.
