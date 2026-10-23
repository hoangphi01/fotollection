# Glossary — Fotollection

> **Status:** Draft v0.1 · **Last reviewed:** 2026-10-23

| Term | Meaning |
|------|---------|
| **3D LUT** | A lookup table that maps every input RGB colour to an output colour, sampled on a grid (33×33×33) and interpolated. Fotollection bakes Photo Style parameters into one, so a look costs one lookup per pixel. |
| **ADR** | Architecture Decision Record: a short document capturing one significant decision, its context and its consequences. See [adr/](adr/README.md). |
| **Back-pressure** | When a slower downstream stage blocks upstream stages from producing more work, so that memory stays bounded. |
| **Bleed / safe area** | Labs trim a little (about 1–2 mm) from each edge when cutting prints. The *safe area* is the region guaranteed to survive trimming, so stamps and captions go inside it. |
| **CEWE** | Large European photo-printing service (also behind the dm and Rossmann photo services). Its paper sizes define our print ratios. |
| **Chroma subsampling** | JPEG stores colour at lower resolution than brightness (4:2:0). 4:4:4 keeps full colour resolution, which gives sharper coloured text stamps. |
| **Contact sheet** | A single page of thumbnails, with details, for an entire print set. |
| **Culling** | Quickly reviewing many photos to rate, flag and keep only the best ones. |
| **DBSCAN** | Density-based clustering algorithm. It groups photos that are close in space and time into trips without knowing the number of trips in advance. |
| **DCT scaling** | Decoding a JPEG at 1/2, 1/4 or 1/8 size directly from its frequency coefficients. This is much faster than a full decode followed by a resize. |
| **DPI / PPI** | Dots (pixels) per inch: the pixels in the cropped image divided by the print size in inches. 300 is excellent for photo prints. |
| **EOI** | End Of Image marker (`FF D9`) in a JPEG. Data after it is ignored by decoders but can hide payloads. |
| **EXIF** | Metadata embedded in photos: capture time, camera, GPS, orientation and so on. |
| **FR / NFR** | Functional / non-functional requirement, defined in [PRD.md](PRD.md). |
| **GeoNames** | Open geographic database. Our offline geocoder uses its `cities1000` dataset (all cities with a population of 1,000 or more). |
| **Golden image** | A stored reference output that tests compare against to detect rendering regressions. |
| **GPX** | GPS Exchange Format: an XML track recorded by phones, watches and GPS loggers. Used to infer locations for cameras without GPS. |
| **HEIC / HEIF** | High Efficiency Image format, the iPhone default, compressed with HEVC. |
| **ICC profile** | Colour-space description embedded in an image (sRGB, Display P3, Adobe RGB). Dropping it shifts colours in print. |
| **IPC** | Inter-process communication between the web UI and the Rust core, done through Tauri commands and events. |
| **k-d tree** | Spatial index used for fast nearest-city lookup. |
| **Lossless crop** | Cropping a JPEG in the DCT domain along MCU boundaries without re-encoding, so there is zero quality loss. |
| **MCU** | Minimum Coded Unit: an 8×8 or 16×16 pixel block in a JPEG. Lossless crops align to it. |
| **Normalization (3NF)** | Structuring data so that each fact is stored once and referenced by key, which avoids duplication and update anomalies. See [DATABASE.md §2](DATABASE.md#2-normalization-rules). |
| **Orientation** | EXIF tag (values 1–8) that tells viewers how to rotate or flip the stored pixels. |
| **Output sharpening** | Sharpening applied for the final print size and paper, to compensate for ink spread and viewing distance. |
| **Perceptual hash** | Compact image fingerprint that stays similar for visually similar images. Used for duplicate detection. |
| **Photo Style** | Panasonic Lumix's in-camera look system: a base style plus contrast, sharpness, saturation, hue and highlight/shadow controls. Fotollection's looks are modelled on it ([FR-039](PRD.md#fr-039--photo-style-looks)). |
| **Polyglot file** | A file that is valid in two formats at once (for example JPEG and HTML), used to smuggle content past validation. |
| **Preset (partial)** | A saved bundle of settings that owns only some setting groups, so presets can be stacked ([FR-038](PRD.md#fr-038--print-presets)). |
| **Rendering intent** | How a colour conversion handles colours outside the target space: *perceptual* compresses the whole gamut smoothly, *relative colorimetric* clips only out-of-gamut colours. |
| **RenderPlan** | Fotollection's pure, serializable description of how one output photo is produced: crop rectangle, stamp, output path. It is shared by preview and export. |
| **sRGB** | The standard colour space most print labs expect. Wider spaces (Display P3, Adobe RGB) look dull if a lab treats them as sRGB. |
| **Tauri** | Rust framework for desktop apps that use the operating system's native webview for the UI. |
| **Tier (T0–T3)** | Preview resolution levels, from the embedded thumbnail to full resolution. See [OPTIMIZATION.md §4.1](OPTIMIZATION.md#41-multi-tier-preview). |
| **WebView2 / WKWebView** | The system webviews on Windows and macOS. |
