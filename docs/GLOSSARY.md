# Glossary — Fotollection

> **Status:** Draft v0.1 · **Last reviewed:** 2026-11-09

| Term | Meaning |
|------|---------|
| **3 mm border** | A white border 3 mm wide inside the print size, cut into the photo, so the print keeps the kiosk's exact size ([FR-067](PRD.md#fr-067--3-mm-border-on-every-print)). With the info frame it takes the frame's colour. |
| **3D LUT** | A lookup table that maps every input RGB colour to an output colour, sampled on a grid (33×33×33) and interpolated. Fotollection bakes a look's sliders, its LUT and a preset's curves into one, so a look costs one lookup per pixel. |
| **ADR** | Architecture Decision Record: a short document capturing one significant decision, its context and its consequences. See [adr/](adr/README.md). |
| **Amount** | How much of a whole look a photo gets, 0–100 % (FR-079), as Lightroom's preset Amount: it scales the sliders, the table and the Dehaze together. |
| **Back-pressure** | When a slower downstream stage blocks upstream stages from producing more work, so that memory stays bounded. |
| **Bleed / safe area** | Labs trim a little (about 1–2 mm) from each edge when cutting prints. The *safe area* is the region guaranteed to survive trimming, so stamps and captions go inside it. |
| **Cell** | One half of a collage sheet. The photo sits centred in it, with a white margin round its photo area, or fills it whole (*Fill each half*). |
| **CEWE** | Large European photo-printing service (also behind the dm and Rossmann photo services). Its paper sizes define our print ratios. |
| **Chroma subsampling** | How much of a JPEG's colour is kept against its brightness: 4:4:4 keeps all of it, 4:2:2 half across, 4:2:0 half each way (most cameras and phones). Shown in the preview's *This photo* card ([FR-071](PRD.md#fr-071--image-facts)); print exports are 4:4:4. |
| **Collage / sheet** | Two photos on one print, like a half-frame camera ([FR-035](PRD.md#fr-035--multi-photo-layouts)): a 10 × 15 sheet holds two 10 × 7.5 cells. One file per sheet on export. |
| **Color Mixer** | Eight colours (red, orange, yellow, green, aqua, blue, purple, magenta), each turned, saturated and lightened, as Lightroom's HSL panel (FR-079, `colour_ops.rs`). |
| **Contact sheet** | A single page of thumbnails, with details, for an entire print set. |
| **Containment** | A failing part costs only itself, never the app (ADR-0048, NFR-011): contained pools, supervised threads, healing locks, error boundaries in the interface. |
| **Culling** | Quickly reviewing many photos to rate, flag and keep only the best ones. |
| **Cut line** | A thin light line between a sheet's cells, with ticks at its edges, to cut the print in two. Off, the photos keep the white between them. |
| **DBSCAN** | Density-based clustering algorithm. It groups photos that are close in space and time into trips without knowing the number of trips in advance. |
| **DCT scaling** | Decoding a JPEG at 1/2, 1/4 or 1/8 size directly from its frequency coefficients. This is much faster than a full decode followed by a resize. |
| **Dehaze** | Haze taken out of a far view, or laid on, by the dark channel prior (FR-115, `dehaze.rs`), before the look's table. |
| **DPI / PPI** | Dots (pixels) per inch: the pixels in the cropped image divided by the print size in inches. 300 is excellent for photo prints. |
| **EOI** | End Of Image marker (`FF D9`) in a JPEG. Data after it is ignored by decoders but can hide payloads. |
| **EXIF** | Metadata embedded in photos: capture time, camera, GPS, orientation and so on. |
| **.fotosettings** | This machine's settings, fonts, logos and LUTs in one ZIP, "the app's brain", to carry to another computer (FR-082); imported by merging or replacing. |
| **FR / NFR** | Functional / non-functional requirement, defined in [PRD.md](PRD.md). |
| **GeoNames** | Open geographic database. Our offline geocoder uses its `cities1000` dataset (all cities with a population of 1,000 or more). |
| **Golden image** | A stored reference output that tests compare against to detect rendering regressions. |
| **GPX** | GPS Exchange Format: an XML track recorded by phones, watches and GPS loggers. Used to infer locations for cameras without GPS. |
| **Half-frame** | A film camera that puts two frames on each 35 mm negative, 72 photos on a 36-exposure roll: the idea behind Collage mode. |
| **HEIC / HEIF** | High Efficiency Image format, the iPhone default, compressed with HEVC. |
| **ICC profile** | Colour-space description embedded in an image (sRGB, Display P3, Adobe RGB). Dropping it shifts colours in print. |
| **Info frame** | A band under (or around) the photo showing the camera, lens, exposure, style and date, "Shot on …" style. The whole print keeps the selected size ([FR-040](PRD.md#fr-040--exif-info-frame)). |
| **IPC** | Inter-process communication between the web UI and the Rust core, done through Tauri commands and events. |
| **k-d tree** | Spatial index used for fast nearest-city lookup. |
| **Lightroom preset** | A user's own Lightroom® `.xmp` or `.lrtemplate`, read into a look (FR-089, ADR-0045): the sliders that match ours go to them, the per-pixel rest into one table, what no look holds named as not applied. |
| **Look** | A photo's colour (FR-079): sliders, an optional LUT and an amount, kept per photo; one of Fotollection's eight, a saved one, a LUT or a Lightroom preset. Shown as tiles on the strip under the photo. |
| **Lossless crop** | Cropping a JPEG in the DCT domain along MCU boundaries without re-encoding, so there is zero quality loss. |
| **LUT** | Look-up table: a file that gives each colour the colour it becomes, a `.cube` or a HaldCLUT PNG (FR-070), kept in `looks/` by its content hash. |
| **MCU** | Minimum Coded Unit: an 8×8 or 16×16 pixel block in a JPEG. Lossless crops align to it. |
| **Mode** | Simple, Collage or Pro ([FR-048](PRD.md#fr-048--simple-collage-and-professional-modes)): how much of the app is on screen. The photos, basket and settings are the same in every mode. |
| **Normalization (3NF)** | Structuring data so that each fact is stored once and referenced by key, which avoids duplication and update anomalies. See [DATABASE.md §2](DATABASE.md#2-normalization-rules). |
| **Orientation** | EXIF tag (values 1–8) that tells viewers how to rotate or flip the stored pixels. |
| **Output sharpening** | Sharpening applied for the final print size and paper, to compensate for ink spread and viewing distance. |
| **Perceptual hash** | Compact image fingerprint that stays similar for visually similar images. Used for duplicate detection. |
| **Photo Style** | Panasonic Lumix's in-camera look system: a base style plus contrast, sharpness, saturation, hue and highlight/shadow controls. Fotollection's looks are modelled on it ([FR-039](PRD.md#fr-039--photo-style-looks)). |
| **Polyglot file** | A file that is valid in two formats at once (for example JPEG and HTML), used to smuggle content past validation. |
| **Postcard** | A collage sheet with one photo and the back of a postcard on its other half: a dashed stamp box and writing lines, with typed text or room to write by hand ([FR-073](PRD.md#fr-073--postcard-sheets)). |
| **Preset (partial)** | A saved bundle of settings that owns only some setting groups, so presets can be stacked ([FR-038](PRD.md#fr-038--print-presets)). |
| **Presets & LUTs** | The develop view's left-column panel (Pro mode) listing the looks of the library in groups, with *Import presets…* and *Import LUT…*, as Lightroom's Presets panel. |
| **Print basket / set aside** | The two holding areas ([FR-050](PRD.md#fr-050--print-basket)): the photos that will really be printed, and the ones taken out of the gallery for now. Nothing is ever deleted. |
| **Rendering intent** | How a colour conversion handles colours outside the target space: *perceptual* compresses the whole gamut smoothly, *relative colorimetric* clips only out-of-gamut colours. |
| **RenderPlan** | Fotollection's pure, serializable description of how one output photo is produced: crop rectangle, stamp, output path. It is shared by preview and export. |
| **Settings → Data** | The Settings section of what Fotollection keeps on this computer (FR-116): moving it with a `.fotosettings`, each store's size and folder, starting afresh. |
| **sRGB** | The standard colour space most print labs expect. Wider spaces (Display P3, Adobe RGB) look dull if a lab treats them as sRGB. |
| **Standard preview** | A 1600 px preview kept on disk after the thumbnails, so any photo opens at once (FR-091, ADR-0041). |
| **Tauri** | Rust framework for desktop apps that use the operating system's native webview for the UI. |
| **Tier (T0–T3)** | Preview resolution levels, from the embedded thumbnail to full resolution. See [OPTIMIZATION.md §4.1](OPTIMIZATION.md#41-multi-tier-preview). |
| **WebView2 / WKWebView** | The system webviews on Windows and macOS. |
| **X-Trans** | Fujifilm's 6 × 6 colour filter, in place of the 2 × 2 Bayer pattern of other sensors; it needs its own demosaic ([FR-072](PRD.md#fr-072--our-own-develop-core)). |
