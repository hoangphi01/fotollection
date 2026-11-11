# ADR-0050: Fotollection on Flathub, as `org.fotollection.Fotollection`

- **Status:** Proposed, deferred (owner, 2026-11-11: submit in two or three months, once Fotollection has more users and is steadier, nearer 1.0; free channels first). The app side is ready; the owner writes the manifest and submits
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

## The steps (owner, 2026-11-11: once an Ubuntu virtual machine is set up)

[Flathub's submission guide](https://docs.flathub.org/docs/for-app-authors/submission); the manifest, the pull request, its messages and every reply are the owner's own (the AI rule above), and this repository only prepares the app.

0. **Before:** a Linux machine or VM with `flatpak` and `org.flatpak.Builder` (`flatpak install -y flathub org.flatpak.Builder`); GitHub two-factor authentication on (the write access to the app's Flathub repository is offered once, for a week); a release that carries `src-tauri/linux/flatpak/` (v0.14.1 or later), since the manifest takes the source from a tag and its commit. Optional first: the single-instance plugin's D-Bus name taken from `FLATPAK_ID` (today `com.hoangphi01.fotollection.SingleInstance`, which a Flatpak may own only with `--own-name`).
1. **Dependency sources, generated:** `flatpak-builder-tools`' `cargo/flatpak-cargo-generator.py` from `Cargo.lock` and `flatpak-node-generator` from `package-lock.json`, again for every release.
2. **The manifest** `org.fotollection.Fotollection.yml`, written by the owner from Tauri's Flatpak guide: the latest GNOME runtime (`flatpak remote-info flathub org.gnome.Platform`), the `rust-stable` and matching `node` SDK extensions, an offline npm install and vendored Cargo, `cargo build --release -p fotollection --features tauri/custom-protocol`, then the binary, `src-tauri/linux/flatpak/*` into `share/metainfo/` and `share/applications/`, the icons (512 and 256 px) as `org.fotollection.Fotollection.png`, `src-tauri/linux/fotollection-mime.xml`, `LICENSE` into `share/licenses/org.fotollection.Fotollection/`; Wayland with X11 as fallback, IPC, DRI, the home folder and removable media, no network.
3. **Build, run, lint:** `flatpak run --command=flathub-build org.flatpak.Builder --install <manifest>`, `flatpak run org.fotollection.Fotollection` (a folder in the home and on a card, a RAW, an export, Open with, a second launch, *Settings → Updates*), then `flatpak-builder-lint` on the manifest and on `repo`.
4. **Submit:** fork `flathub/flathub` with its `new-pr` branch, a branch of one's own, the manifest and the two generated source files, a pull request against `new-pr` titled `Add org.fotollection.Fotollection`, the template kept, the permissions explained (a photo library reads whole folders and writes beside them) and the use of AI disclosed truthfully: which parts, how much, and what the owner does himself.
5. **Review:** answer every question in person, fix on the same branch, then comment `bot, build`.
6. **After acceptance:** accept the invitation to `flathub/org.fotollection.Fotollection`; verify the domain on flathub.org and put its token in `site/.well-known/org.flathub.VerifiedApps.txt`; each release afterwards, the tag, the commit and the generated sources updated there by a pull request.

## Before submitting: Flathub's quality guidelines

[The quality guidelines](https://docs.flathub.org/docs/for-app-authors/metainfo-guidelines/quality-guidelines) are optional for publishing but decide whether Flathub features an app. Checked on 2026-11-11 against `org.fotollection.Fotollection.metainfo.xml`:

- **Summary:** 58 characters ("Look through a trip's photos and prepare them for printing"); at most 35, ideally 10 to 25, an imperative verb, no name, no licence, no "app".
- **Release notes:** none of the ten releases has a description; each needs two or three user-facing sentences (from the CHANGELOG's highlights, written into the metainfo by `release.sh`).
- **Screenshots:** made in the browser from the demo, not on Linux; Flathub asks for the app's own window taken on Linux with its title bar and shadow, at most 1000 × 700 (2000 × 1400 HiDPI), in the default settings, with captions of one sentence.
- **Brand colours:** the dark one (`#c98a52`, wood) is lighter than the light one (`#2c836d`, teal); the dark variant must be darker and more muted.
- **Icon:** the 512 px icon fills the whole canvas, carries the word *fotollection* in small type and a baked shadow; Flathub's icon is square with room around it, no small text, no baked shadow, readable on light and dark (an icon of its own for Linux, the owner's design).
- Passing already: the name (12 characters), the description's length and list, the captions, the OARS rating, a runtime that is not end of life (to choose at submission).
