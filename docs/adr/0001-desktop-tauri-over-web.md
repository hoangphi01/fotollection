# ADR-0001: Desktop app with Tauri 2 instead of a web app

- **Status:** Accepted
- **Date:** 2026-10-23
- **Related:** NFR-001…NFR-008, ADR-0002

## Context

The original plan (2026-09) was a browser web app built with Vite, TypeScript, Canvas and Web Workers. It would have been hosted on a Netcup VPS behind Cloudflare. Fotollection targets enthusiast and professional photographers, so **performance and output quality come first**. Browser constraints get in the way: per-tab memory limits of about 2–4 GB, `canvas.toBlob` encoding with limited control, ICC profiles dropped, no HEIC support in Chrome or Firefox, and no reliable direct-to-folder writes. Hosting also costs money every month, even for static files.

## Options considered

1. **Web app / PWA:** zero install and works everywhere. But it is bound by the browser's limits on memory, codecs and colour management.
2. **Electron:** full Node.js access. But it bundles Chromium (installers over 150 MB, high RAM use), and image work would still be in JS unless native addons are written.
3. **Pure-Rust GUI (egui / Iced / Slint):** the lightest option. But a polished UI, a virtualized grid and interactive cropping take much more effort.
4. **Tauri 2:** Rust core for the heavy work, with the UI in the OS's own webview. Small binaries, a capability-based security model, macOS and Windows targets.

## Decision

Build Fotollection as a **Tauri 2 desktop application**. All image processing happens in Rust. The UI is a thin web frontend.

## Consequences

- No server and no hosting cost. Distribution is through GitHub Releases ([RELEASE.md](../RELEASE.md)).
- Fully offline by design ([NFR-006](../PRD.md#nfr-006--offline-and-zero-telemetry)).
- New concerns: code signing and notarization, per-platform native dependencies (libheif), and two webview engines to test (WebKit and WebView2).
- The browser-specific security sections (CSP headers, Nominatim rate limits) were replaced in [SECURITY.md](../SECURITY.md) v0.2.
