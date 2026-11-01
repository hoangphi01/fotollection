# ADR-0035: Linux install from fotollection.org: a script and a signed APT repository

- **Status:** Accepted
- **Date:** 2026-11-01
- **Related:** NFR-007, ADR-0034, ADR-0009, ADR-0021, ADR-0025

## Context

Linux users expect to install and update from a terminal. Downloading an AppImage by hand and replacing it on every release is what the other systems do, but on Linux the usual way is the package manager. The owner wants the most out of the project's own domain, fotollection.org, which is GitHub Pages behind Cloudflare and costs nothing more.

## Options considered

1. **Only the downloads** (AppImage, `.deb`) on GitHub Releases. Nothing to run, but no command to give and no updates.
2. **A third-party channel** (Flathub, a PPA, the AUR). Wide reach, but each needs its own build and review, and a PPA builds from source on Launchpad.
3. **Our own channel on fotollection.org:** an install script, and an APT repository built from the release's `.deb` files and served as static files with the website.

## Decision

Option 3, with Flathub kept for later.

- **`https://fotollection.org/install.sh`** (`site/install.sh`, POSIX sh, short enough to read before running):
  - It checks the architecture (x86_64 or ARM64) and glibc 2.35.
  - On Ubuntu, Debian, Mint and their family it adds the APT repository with sudo and installs the package.
  - Elsewhere, or with `--appimage`, it downloads the latest AppImage from GitHub, checks it against `SHA256SUMS.txt`, and puts it in `~/.local` with a menu entry, without sudo.
  - `--uninstall` removes either install, keeping the user's settings and projects.
- **`https://fotollection.org/apt`**, a signed repository (`stable`, `main`, `amd64` and `arm64`):
  - `pages.yml` runs `scripts/apt-repo.sh` on every website build. The script downloads the latest release's `.deb` files, writes the indexes with `apt-ftparchive`, signs `InRelease` and `Release.gpg`, and publishes the public key at `/apt/fotollection.gpg`.
  - Only the latest version is kept, about 30 MB.
  - Users add it with a `signed-by` line, so the key is trusted for this repository only.
- **The key:**
  - An Ed25519 signing key made for this repository only: "Fotollection APT repository <apt@fotollection.org>", fingerprint `1F67 E8C1 93CD CA68 BFC8 82DA 5850 9F27 C429 45A3`, no expiry.
  - The private key is stored only in the GitHub Actions secret `APT_GPG_PRIVATE_KEY`, with the owner's backup in `~/.fotollection-apt-key/` (mode 700, never in the repository).
  - **If the key leaks:** make a new key, replace the secret, publish the new public key, and tell users in the release notes to fetch it again.
- **The app still never updates itself** ([ADR-0025](0025-update-check-on-click.md)). `apt upgrade` is the user's own command.

## Consequences

- Linux users get `curl -fsSL https://fotollection.org/install.sh | sh` and `sudo apt install fotollection`, with updates through `sudo apt upgrade`.
- `curl | sh` asks users to trust the script. It is served over HTTPS from the project's domain, is readable at its URL and in the repository, and does nothing the website does not describe.
- The website build needs the key secret. Without it, the site still builds, without the repository.
- Fedora, openSUSE and Arch use the AppImage. An RPM repository, the AUR or Flathub can come later.
