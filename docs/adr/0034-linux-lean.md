# ADR-0034: Linux, the lean build

- **Status:** Accepted. Amends [ADR-0021](0021-versioning-and-automated-releases.md) (a new platform in a PATCH release) and [ADR-0009](0009-unsigned-distribution.md) (Linux)
- **Date:** 2026-11-01
- **Related:** NFR-007, FR-068, FR-062, FR-063, ADR-0017, ADR-0023, ADR-0029, ADR-0030

## Context

Linux has few good photo apps, which makes it a place where Fotollection can be the obvious choice. The app is Tauri with a pure-Rust core, and every platform-specific part already has a fallback: `oscodec` answers "unsupported" off macOS and Windows, `oscamera` builds without its OS modules, RAW goes through our own reader and rawler. What is missing is packaging, CI, a few behaviours and the words.

Three things do not carry over:

- **HEIC** is decoded by the OS on macOS (ImageIO) and Windows (WIC). Linux has no such codec in the system, and HEVC is covered by patent pools that still enforce them (2026). Shipping our own HEVC decoder would make us distribute it.
- **Camera capture over USB** uses ImageCaptureCore and WPD; Linux would need libgphoto2 (LGPL) or our own PTP over libusb.
- **The owner has no Linux machine.**

## Decision

- **A lean Linux build:** every photo feature, without camera capture (the button is not shown) and without HEIC for now. HEIC is rejected with its own reason, "not supported on Linux yet". Later, FR-068 loads the system's own `libheif` at run time (installed by the user's distribution, never bundled), so we never ship an HEVC decoder.
- **Packages:** AppImage (any distribution) and `.deb` (Debian, Ubuntu, Mint…), for x86_64 and ARM64, built on `ubuntu-22.04` runners, so **glibc 2.35 is the floor**. No signing, as on the other systems (ADR-0009); `SHA256SUMS.txt` covers every file.
- **Proof without a machine:** every release installs the `.deb` and checks its file types, and runs the AppImage for 20 s under a virtual display, on Ubuntu 22.04 and in a Debian 12 container. CI runs the whole test suite on Ubuntu on every change.
- **Nvidia:** WebKitGTK's DMA-BUF renderer leaves the window blank with Nvidia's driver; when that driver is loaded, the app restarts itself once with `WEBKIT_DISABLE_DMABUF_RENDERER=1` (`exec`, no `unsafe`).
- **Versioning:** a new platform for the same features is packaging, not a feature: it may ship in a PATCH release. Linux first ships in **v0.8.2**, released from the `release/0.8` branch; v0.9.0 ships the three systems together.
- **Check for updates:** an AppImage downloads the AppImage for its architecture; a `.deb` install opens the release page.

## Consequences

- Linux users get the app with two clear limits, stated on the website in a table by system.
- One more CI column and two more release jobs (free runners for a public repository).
- Distributions older than glibc 2.35 (Ubuntu 20.04, Debian 11, RHEL 8) and musl systems (Alpine) cannot run it.
