# Glossary — Fotollection

> **Status:** Draft v0.1 · **Last reviewed:** 2026-10-23

| Term | Meaning |
|------|---------|
| **ADR** | Architecture Decision Record: a short document capturing one significant decision, its context and its consequences. See [adr/](adr/README.md). |
| **Back-pressure** | When a slower downstream stage blocks upstream stages from producing more work, so that memory stays bounded. |
| **CEWE** | Large European photo-printing service (also behind the dm and Rossmann photo services). Its paper sizes define our print ratios. |
| **Chroma subsampling** | JPEG stores colour at lower resolution than brightness (4:2:0). 4:4:4 keeps full colour resolution, which gives sharper coloured text stamps. |
| **DBSCAN** | Density-based clustering algorithm. It groups photos that are close in space and time into trips without knowing the number of trips in advance. |
| **DCT scaling** | Decoding a JPEG at 1/2, 1/4 or 1/8 size directly from its frequency coefficients. This is much faster than a full decode followed by a resize. |
| **EOI** | End Of Image marker (`FF D9`) in a JPEG. Data after it is ignored by decoders but can hide payloads. |
| **EXIF** | Metadata embedded in photos: capture time, camera, GPS, orientation and so on. |
| **FR / NFR** | Functional / non-functional requirement, defined in [PRD.md](PRD.md). |
| **GeoNames** | Open geographic database. Our offline geocoder uses its `cities1000` dataset (all cities with a population of 1,000 or more). |
| **Golden image** | A stored reference output that tests compare against to detect rendering regressions. |
| **HEIC / HEIF** | High Efficiency Image format, the iPhone default, compressed with HEVC. |
| **ICC profile** | Colour-space description embedded in an image (sRGB, Display P3, Adobe RGB). Dropping it shifts colours in print. |
| **IPC** | Inter-process communication between the web UI and the Rust core, done through Tauri commands and events. |
| **k-d tree** | Spatial index used for fast nearest-city lookup. |
| **Lossless crop** | Cropping a JPEG in the DCT domain along MCU boundaries without re-encoding, so there is zero quality loss. |
| **MCU** | Minimum Coded Unit: an 8×8 or 16×16 pixel block in a JPEG. Lossless crops align to it. |
| **Orientation** | EXIF tag (values 1–8) that tells viewers how to rotate or flip the stored pixels. |
| **Perceptual hash** | Compact image fingerprint that stays similar for visually similar images. Used for duplicate detection. |
| **Polyglot file** | A file that is valid in two formats at once (for example JPEG and HTML), used to smuggle content past validation. |
| **RenderPlan** | Fotollection's pure, serializable description of how one output photo is produced: crop rectangle, stamp, output path. It is shared by preview and export. |
| **Tauri** | Rust framework for desktop apps that use the operating system's native webview for the UI. |
| **Tier (T0–T3)** | Preview resolution levels, from the embedded thumbnail to full resolution. See [OPTIMIZATION.md §4.1](OPTIMIZATION.md#41-multi-tier-preview). |
| **WebView2 / WKWebView** | The system webviews on Windows and macOS. |
