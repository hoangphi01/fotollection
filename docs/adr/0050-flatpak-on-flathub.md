# ADR-0050: Fotollection on Flathub, as `org.fotollection.Fotollection`

- **Status:** Proposed (the app side is ready; the owner writes the manifest and submits)
- **Date:** 2026-11-11
- **Related:** NFR-007, FR-061, FR-103, ADR-0034, ADR-0035

## Context

Linux users find software in GNOME Software, KDE Discover and on the Steam Deck, which all read Flathub. Fotollection reaches Linux today through its AppImage, its `.deb` and the APT repository on fotollection.org ([ADR-0035](0035-linux-install-from-fotollection-org.md)), which keeps Flathub "for later". The owner asked for it on 2026-11-11, as the first step of making the project better known (ROADMAP, *Growth*).

[Flathub's requirements](https://docs.flathub.org/docs/for-app-authors/requirements), read that day: an open-source app is built entirely from source, with no network during the build (every Cargo and npm dependency listed as a source); its ID sits on a domain the project controls and is verified by a token at `/.well-known/org.flathub.VerifiedApps.txt`; static permissions are minimal and portals are used where they exist; an app does not update itself. Its generative-AI rule: AI-written code in the app is disclosed with its extent and reviewers may reject on it; the Flathub manifest contains no AI-written or AI-assisted content; the pull request, its messages and the replies to reviewers are the person's own.

## Options considered

1. **The `.deb` repackaged in the manifest.** The least work, but Flathub builds open-source apps from source, and a `.deb` built on another system breaks on the runtime's libraries (Tauri's Flatpak guide).
2. **Built from source on the GNOME runtime.** Tauri 2 uses WebKitGTK 4.1, which the GNOME runtime carries; Rust and Node come from SDK extensions; the dependencies are vendored by flatpak-builder-tools. Every native library the app links (GTK 3, WebKitGTK, libsoup 3, D-Bus) is in the runtime.
3. **Our own Flatpak repository on fotollection.org.** Flathub rejects a third-party submission of an app whose upstream ships its own Flatpak elsewhere, and a private repository reaches nobody's software centre.

## Decision

Option 2, under the ID `org.fotollection.Fotollection` (the project's domain). The app side lives in this repository and is made here; the manifest and the submission are the owner's own work, as Flathub's AI rule asks.

- **The Flatpak's files** (`src-tauri/linux/flatpak/`): its AppStream metadata and its launcher, made by `node scripts/flatpak-files.mjs` from the `.deb`'s metainfo and `tauri.linux.conf.json`'s file types, so the two never drift (`check-docs.sh` fails when they do; `release.sh` remakes them with each release; CI checks them with Flathub's linter and `desktop-file-validate`). It adds what Flathub's newer AppStream reads and Ubuntu 22.04's does not (the developer, brand colours, source, contribute, donation and help links, `provides` the `.deb`'s ID).
- **No self-update:** in a Flatpak (`FLATPAK_ID`, `/.flatpak-info`), Rust never asks GitHub, at start or on a click (`updates.rs` `UpdateSource`, `ERR-NET-0004`), and *Settings → Updates* says the software centre brings each version.
- **Files:** the file dialog is GTK's (rfd), which works in the sandbox; a photo library reads whole folders and writes beside them, so the manifest grants the home folder and removable media, as Flathub's other photo managers do.
- **Verification:** `site/.well-known/` is copied into the site (`eleventy.config.mjs`) and served (`upload-pages-artifact@v3` keeps dot-folders); the owner adds Flathub's token there.

## Consequences

- The AppImage, the `.deb` and the APT repository stay as they are, under their ID `com.hoangphi01.fotollection`, so the VNLF App Explorer and CaramOS keep their listing; the Flatpak `provides` that ID.
- Each release: `release.sh` remakes the Flatpak's metainfo; the owner updates the Flathub manifest's source to the new tag.
- Data in a Flatpak lives in its sandbox (`~/.var/app/org.fotollection.Fotollection/`); moving from the `.deb` is *Settings → Data* (FR-082).
