# PhotoOrder - Status Tracker

> Living document tracking progress, blockers, and next steps.

## Current Phase: Pre-Development

### Overall Progress

| Phase | Status | Progress |
|-------|--------|----------|
| Phase 1 — MVP | Not Started | 0/7 features |
| Phase 2 — Smart Features | Not Started | 0/4 features |
| Phase 3 — Polish & Growth | Not Started | 0/5 features |

## Phase 1 — MVP Feature Tracker

| # | Feature | Status | Notes |
|---|---------|--------|-------|
| 1 | Batch photo upload | Not Started | Drag & drop, folder select |
| 2 | EXIF metadata reading | Not Started | Using exifr library |
| 3 | Auto date stamp overlay | Not Started | Font, position, format, color options |
| 4 | Location stamp (geocoding) | Not Started | Nominatim reverse geocoding |
| 5 | Batch crop to CEWE ratios | Not Started | 8 standard EU print sizes |
| 6 | Original quality output | Not Started | No re-compression |
| 7 | ZIP download by date | Not Started | JSZip organized output |

## Phase 2 — Smart Features Tracker

| # | Feature | Status | Notes |
|---|---------|--------|-------|
| 8 | Trip auto-grouping | Not Started | DBSCAN on GPS + time |
| 9 | Face-aware smart cropping | Not Started | MediaPipe / face-api.js |
| 10 | ZIP export by trip | Not Started | Readable folder names |
| 11 | Duplicate detection | Not Started | Perceptual hashing |

## Phase 3 — Polish & Growth Tracker

| # | Feature | Status | Notes |
|---|---------|--------|-------|
| 12 | PWA / offline support | Not Started | Service Worker |
| 13 | Photo book layout preview | Not Started | Album arrangement |
| 14 | QR code on photo back | Not Started | Digital-physical bridge |
| 15 | Direct print API upload | Not Started | CEWE/Rossmann/dm APIs |
| 16 | Multi-language | Not Started | EN, DE, VI |

## Security & Resilience Tracker

> Full details in [SECURITY.md](SECURITY.md)

| # | Task | Priority | Status | Notes |
|---|------|----------|--------|-------|
| S1 | EXIF sanitizer (4-layer pipeline) | P0 | Not Started | `src/lib/exif-sanitizer.ts` — allowlisted fields, polyglot detection, value validation |
| S2 | File validation (magic bytes, size) | P0 | Not Started | `src/lib/security.ts` — input guards |
| S3 | Cloudflare security headers (CSP) | P0 | Not Started | `public/_headers` — strict CSP, HSTS, X-Frame-Options |
| S4 | Processing queue (concurrency, memory) | P1 | Not Started | `src/lib/processing-queue.ts` — batch control, AbortController |
| S5 | Network manager (offline/online) | P1 | Not Started | `src/lib/network-manager.ts` — reconnect, request queue |
| S6 | Nominatim client (throttle, cache) | P1 | Not Started | `src/lib/nominatim-client.ts` — 1 req/sec, GPS clustering |
| S7 | Error handler (global boundary) | P2 | Not Started | `src/lib/error-handler.ts` — user-friendly messages, per-photo recovery |
| S8 | innerHTML/eval CI audit | P2 | Not Started | grep checks in CI pipeline |

## Project Setup

| Task | Status | Notes |
|------|--------|-------|
| package.json initialized | Done | Vite + TypeScript, exifr, jszip |
| PLAN.md created | Done | SSOT for project vision & architecture |
| STATUS.md created | Done | This file |
| SECURITY.md created | Done | Error handling, security & resilience plan |
| Vite + TS scaffolding | Not Started | |
| Git repository | Not Started | |
| CI/CD pipeline | Not Started | |
| Domain + hosting | Not Started | Netcup + Cloudflare |

## Blockers

None currently.

## Next Steps

1. Scaffold Vite + TypeScript project structure
2. Build basic UI shell (upload area, settings panel, preview grid)
3. Implement EXIF reading (Feature #2) — with S1 sanitizer from the start
4. Implement date stamp overlay (Feature #3)
5. Implement batch crop to CEWE ratios (Feature #5)
6. Deploy `_headers` to Cloudflare (S3) as soon as hosting is set up
