# Fotollection

**Fast, private, offline photo-print preparation for macOS and Windows.**

Fotollection turns a folder of photos into a print-ready set. It stamps the capture date and place from EXIF, crops to standard EU print sizes (CEWE, dm, Rossmann…), groups photos by trip, and keeps the original quality. Built with Rust and Tauri. Your photos never leave your computer.

> **Status:** pre-alpha. The documentation framework is complete; Phase 0 (scaffold and benchmarks) is next. See [docs/STATUS.md](docs/STATUS.md).

## Features (planned)

- **Date and location stamps** read from EXIF, with location resolved offline
- **Batch crop** to 8 standard print sizes (9×13 to 20×30, square, strip), with an adjustable crop frame per photo
- **Original quality:** full resolution, ICC profile preserved, lossless crop when possible
- **JPEG, HEIC (iPhone) and PNG** input
- **Trip grouping, face-aware crop and duplicate detection** (Phase 2)
- **Zero network by default:** no account, no cloud, no telemetry

The full requirements are in [docs/PRD.md](docs/PRD.md), and the phases in [docs/ROADMAP.md](docs/ROADMAP.md).

## Tech

| Rust core | Shell | UI |
|-----------|-------|----|
| libjpeg-turbo, libheif, lcms2, fast_image_resize, rayon | Tauri 2 | React, TypeScript, Tailwind, shadcn/ui, Lucide |

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the design, and [docs/OPTIMIZATION.md](docs/OPTIMIZATION.md) for the performance budgets and why we chose native Rust over a web app.

## Building

Not available yet. Build instructions will be added in Phase 0.

## Documentation

Start at [docs/README.md](docs/README.md). Contributors should read [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[GPL-3.0-only](LICENSE) © hoangphi01. Outside contributions require a CLA ([ADR-0008](docs/adr/0008-license-gpl3-with-relicensing-option.md)).

Builds are currently **unsigned** ([ADR-0009](docs/adr/0009-unsigned-distribution.md)). On first launch:

- **macOS:** System Settings → Privacy & Security → *Open Anyway*.
- **Windows:** SmartScreen → *More info* → *Run anyway*.

Verify downloads with the `SHA256SUMS.txt` file on each release page.

Geographic data from [GeoNames](https://www.geonames.org/) (CC BY 4.0) will be bundled for offline geocoding.
