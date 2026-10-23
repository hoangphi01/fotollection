# PhotoOrder - Project Plan

> **Single Source of Truth** for project vision, architecture, features, and decisions.

## Vision

Privacy-first web app that helps people physicalize their digital photos by automating the printing workflow: batch date/location stamping via EXIF, trip organization, batch cropping to CEWE/EU print ratios, and original-quality output. **All processing client-side — no images ever touch the server.**

## Market Gap (Our Moat)

**Nobody combines all three:** EXIF date/location stamping + trip auto-organization + batch crop to EU print sizes.

### Competitive Landscape

| Tool | Type | Date Stamp | Trip Grouping | CEWE Ratios | Privacy |
|------|------|-----------|---------------|-------------|---------|
| BatchPhoto | Desktop (paid) | Yes | No | No | Local |
| Timestamp It | Web | Yes (3 free) | No | No | Server-side |
| BulkPicTools | Web | No | No | No | Client-side |
| RowPix | Web | No | No | No | Client-side |
| CropMaster | Web | No | No | No | Client-side |
| PhotoPrism | Self-hosted | Partial | GPS maps | No | Self-hosted |
| **PhotoOrder** | **Web** | **Yes** | **Yes** | **Yes** | **Client-side** |

### Target Print Services

CEWE, Rossmann, dm, Boots Photo, Snapfish, FreePrints, Photobox — all use similar EU standard sizes.

## CEWE Print Sizes

| Size (cm) | Size (inch) | Aspect Ratio | Use Case |
|-----------|-------------|-------------|----------|
| 9x13 | 3.5x5 | ~1:1.44 | Wallet/small |
| 10x15 | 4x6 | 2:3 | Standard |
| 11x17 | 4.5x6.5 | ~1:1.55 | Slightly larger |
| 13x18 | 5x7 | ~1:1.38 | Medium |
| 15x20 | 6x8 | 3:4 | Large |
| 20x30 | 8x12 | 2:3 | Extra large |
| 15x15 | 6x6 | 1:1 | Square |
| 5x15 | 2x6 | 1:3 | Photo strip |

## Architecture

### Core Principle: Client-Side Only

- All image manipulation in browser via Canvas API + Web Workers
- No images uploaded to server — true privacy
- Server only serves static assets (HTML/JS/CSS)
- Minimal server cost

### Tech Stack

| Layer | Choice | Rationale |
|-------|--------|-----------|
| Build | Vite + vanilla TypeScript | Fast, minimal, no framework overhead |
| Image processing | Canvas API + OffscreenCanvas in Web Workers | Native browser, no dependencies |
| Fast processing | libvips via WebAssembly | 5-10x faster for heavy batches (Phase 3) |
| EXIF parsing | exifr | Modern, fast, client-side |
| Reverse geocoding | OpenStreetMap Nominatim API | Free, no key needed, privacy-respecting |
| Trip clustering | DBSCAN on GPS + timestamps | Simple, effective for spatial-temporal data |
| Face detection | MediaPipe or face-api.js | Client-side ML for smart cropping |
| Output | JSZip | Batch ZIP download, organized by trip folders |
| Hosting | Netcup (DE) + Cloudflare CDN | GDPR compliant, fast globally |
| Offline | Service Worker (PWA) | Works without internet after first load |

### Hosting: Netcup

- 99.6% uptime, German data centers, GDPR compliant
- VPS 1000 G12: 4 vCores, 8GB DDR5, 256GB NVMe — ~EUR 10/mo
- For static-only: webhosting tier ~EUR 3/mo is enough
- Cloudflare Pages/CDN in front for global speed

## Feature Phases

### Phase 1 — MVP

Core workflow: upload -> stamp -> crop -> download.

1. **Batch photo upload** — drag & drop, folder select, file picker
2. **EXIF metadata reading** — date, GPS coordinates, camera info
3. **Auto date stamp overlay** — customizable font, position, format, color
4. **Location stamp** — reverse geocoding via Nominatim (GPS -> "Berlin, Germany")
5. **Batch crop to CEWE print ratios** — all 8 sizes listed above
6. **Original quality output** — no re-compression artifacts
7. **ZIP download** — organized by date

### Phase 2 — Smart Features

Intelligence layer: auto-organize and protect faces.

8. **Trip auto-grouping** — DBSCAN clustering on GPS + time proximity
9. **Face-aware smart cropping** — don't cut faces when cropping to ratio
10. **ZIP export by trip** — organized folders with readable names (e.g., "2024-03_Berlin_Trip")
11. **Duplicate detection** — perceptual hashing to skip near-duplicates

### Phase 3 — Polish & Growth

Premium features and internationalization.

12. **PWA / offline support** — Service Worker for full offline capability
13. **Photo book layout preview** — album page arrangement preview
14. **QR code on photo back** — links printed photo to digital original
15. **Direct print API upload** — CEWE/Rossmann/dm print APIs (premium)
16. **Multi-language** — EN, DE, VI

## Enhancement Ideas

| Idea | Benefit |
|------|---------|
| WebAssembly (libvips) | 5-10x faster than Canvas API for batch processing |
| Perceptual hashing | Skip near-duplicate photos (save printing cost) |
| QR bridge | Print QR on back linking to digital original |
| Print cost estimator | Show estimated CEWE/dm price for the batch |
| Photo strip mode | CEWE kiosks offer 5x15cm strips, fun for events |

## Error Handling, Security & Resilience

Detailed in **[SECURITY.md](SECURITY.md)** — covers:

- **Threat model** for client-side-only architecture
- **EXIF sanitization** — 4-layer defense pipeline (file validation, parser sandboxing, value validation, safe rendering)
- **Content Security Policy** — strict CSP via Cloudflare `_headers`
- **EXIF privacy** — GPS stripping, user controls, memory-only processing
- **Supply chain security** — locked deps, minimal dependencies, npm audit in CI
- **Network resilience** — offline handling, exponential backoff with jitter for Nominatim
- **Memory management** — processing queue with concurrency control, memory watchdog
- **Nominatim rate limiting** — 1 req/sec throttle, GPS clustering, sessionStorage caching
- **Input validation** — magic bytes, polyglot detection, file size limits
- **Error handling** — per-photo recovery, user-friendly messages, global error boundary

## Key Decisions

| Decision | Choice | Date | Rationale |
|----------|--------|------|-----------|
| Framework | Vanilla TS (no React) | 2026-09 | MVP simplicity, can migrate later if needed |
| EXIF library | exifr | 2026-09 | Best modern client-side EXIF parser |
| Geocoding | Nominatim | 2026-09 | Free, no API key, privacy-respecting |
| Hosting | Netcup + Cloudflare | 2026-09 | GDPR compliant, cheap, fast |
| Processing | Client-side only | 2026-09 | Core privacy promise, zero server cost for compute |
| EXIF field strategy | Allowlist (12 fields) | 2026-09 | Safer than blocklist — ignore unknown fields entirely |
| EXIF rendering | textContent + Canvas fillText only | 2026-09 | Never innerHTML — prevents XSS from EXIF data |
| CSP policy | Strict, no inline scripts | 2026-09 | Last line of defense against XSS |
