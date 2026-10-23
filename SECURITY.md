# PhotoOrder - Security, Error Handling & Resilience

> Comprehensive plan for error handling, security, rate limiting, and resilience.
> Researched from Canva's patterns (50M uploads/day), adapted for our client-side-only architecture.

**Key insight**: Because PhotoOrder does ALL processing client-side (no server-side image handling), our threat model is fundamentally different from server-heavy apps. Many server-side concerns (CSRF, SQL injection, server overload) don't apply. But new client-side-specific problems emerge — especially around EXIF security and browser memory limits.

---

## 1. Threat Model (Client-Side Only App)

| Threat | Applies? | Why / Why not |
|--------|----------|---------------|
| **XSS** | YES | Injected script could steal EXIF data (GPS = home address) |
| **CSRF** | MINIMAL | No server-side state changes, no auth, no forms that POST |
| **SQL Injection** | NO | No database |
| **SSRF** | NO | No server-side requests |
| **Supply chain** | YES | Compromised npm package could exfiltrate photos |
| **Man-in-the-middle** | YES | Without HTTPS, attacker could modify served JS |
| **EXIF data leakage** | YES | GPS coordinates reveal home/work locations |
| **Malicious image files** | YES | Crafted images could exploit Canvas API or inject XSS via EXIF text fields |
| **Browser memory exhaustion** | YES | Large batches can crash the tab |

---

## 2. EXIF Malware & Injection Defense (Primary Attack Surface)

EXIF is both our **core feature** (we read it to stamp dates/locations) and our **#1 attack surface**.

### Known EXIF Attack Vectors

| Attack | How It Works | Severity |
|--------|-------------|----------|
| **XSS via EXIF text fields** | Attacker puts `<script>` in ImageDescription, Artist, UserComment, Copyright. If rendered with innerHTML, script executes. | **HIGH** |
| **Polyglot files** | File is simultaneously valid JPEG and valid HTML/JS. Passes image validation but contains executable code. | **MEDIUM** |
| **Steganography payload** | Malware hidden after JPEG EOI marker. Image renders normally but carries hidden data. | **LOW** |
| **Buffer overflow via malformed EXIF** | Crafted IFD with invalid offsets/lengths. exifr is JS (no raw memory), but could throw. | **LOW** |
| **EXIF thumbnail injection** | Malicious code in the EXIF thumbnail JPEG (a JPEG inside a JPEG). | **MEDIUM** |
| **MakerNote exploitation** | Camera-specific binary blob — can contain anything. | **LOW** |

### 4-Layer EXIF Sanitization Pipeline

Every photo goes through this pipeline BEFORE any EXIF data touches the DOM:

```
Layer 1: FILE VALIDATION (before EXIF parsing)
  - Check magic bytes (JPEG: FF D8 FF, PNG: 89 50 4E 47)
  - Reject if file size > 50MB
  - Scan first 4KB for polyglot signatures: <html, <script, %PDF, PK (ZIP), MZ (EXE)
  - For JPEG: check if significant data exists after EOI marker (FF D9)

Layer 2: PARSER SANDBOXING (during EXIF parsing)
  - Parse EXIF in Web Worker (isolate from main thread)
  - Use exifr with explicit field ALLOWLIST (only read what we need):
      ALLOWED: DateTimeOriginal, CreateDate, GPSLatitude, GPSLongitude,
               GPSAltitude, Make, Model, ImageWidth, ImageHeight,
               Orientation, ExposureTime, FNumber, ISO
      BLOCKED: UserComment, ImageDescription, Artist, Copyright,
               MakerNote, XPComment, XPAuthor, XPTitle
               (all text fields that commonly carry XSS payloads)
  - Wrap in try/catch — if exifr throws, mark photo as "no EXIF" and continue
  - Timeout: if parsing takes > 5 seconds, abort (anti-DoS for crafted files)

Layer 3: VALUE VALIDATION (after parsing)
  - Dates: validate format matches EXIF date regex, reject years < 1900 or > 2100
  - GPS: validate ranges (lat: -90..90, lon: -180..180), reject NaN/Infinity
  - GPS (0,0) "null island": treat as missing data, skip geocoding
  - Numeric fields (ISO, FNumber): validate finite number in sane range
  - String fields (Make, Model): max 100 chars, strip all HTML tags
  - If ANY field contains <, >, javascript:, or on\w+= patterns -> strip it

Layer 4: SAFE RENDERING (when displaying to user)
  - NEVER use innerHTML, outerHTML, or insertAdjacentHTML with EXIF values
  - ALWAYS use textContent or createTextNode()
  - Date stamps on photos: render via Canvas fillText() (inherently safe)
  - CSP script-src 'self' is the last safety net
```

### Why Allowlist > Blocklist

We ONLY read the ~12 fields we actually need. Everything else is ignored at parse time via exifr's `pick` option. Even if someone invents a new EXIF XSS vector in a field we don't read, we're safe.

---

## 3. Content Security Policy & HTTP Headers

### CSP (deployed via Cloudflare `_headers` file)

```
Content-Security-Policy:
  default-src 'none';
  script-src 'self';
  style-src 'self' 'unsafe-inline';
  img-src 'self' blob: data:;
  connect-src 'self' https://nominatim.openstreetmap.org;
  worker-src 'self' blob:;
  font-src 'self';
  base-uri 'self';
  form-action 'none';
  frame-ancestors 'none';
  object-src 'none'
```

Key decisions:
- `script-src 'self'` — NO external scripts, NO inline scripts (prevents XSS)
- `connect-src` — ONLY allow Nominatim (our sole external API)
- `worker-src 'self' blob:` — Allow Web Workers for image processing
- `form-action 'none'` — No forms can POST anywhere (anti-CSRF by design)
- `frame-ancestors 'none'` — Cannot be embedded in iframes (anti-clickjacking)

### Additional Security Headers

```
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
Referrer-Policy: no-referrer
Permissions-Policy: camera=(), microphone=(), geolocation=(), payment=()
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
X-XSS-Protection: 0  (deprecated, CSP is the real protection)
```

---

## 4. EXIF Privacy Protection

Our unique concern — EXIF contains sensitive location data (GPS = home address).

- NEVER send raw EXIF to any server (architecture enforces this, but code must too)
- Strip GPS data from output images by default (opt-in to keep)
- Show privacy warning: "These photos contain GPS coordinates (your location)"
- User options: "Strip all EXIF from output" vs "Keep date only" vs "Keep all"
- GPS coordinates stay in browser memory only, never persisted to disk
- Clear all photo data from memory on page unload

---

## 5. Supply Chain Security

- Lock dependencies with package-lock.json (exact versions)
- Minimal dependencies: exifr, jszip (both well-audited, stable)
- Run `npm audit` in CI pipeline
- Subresource Integrity (SRI) hashes on any CDN-loaded resources
- Review: no dependency should make network requests except our Nominatim client

---

## 6. Input Validation (Malicious Files)

- Validate file type by magic bytes, not just extension
- Cap file size at 50MB (reject larger immediately)
- Max 500 files per batch
- Wrap Canvas operations in try/catch (malformed images can throw)
- Use OffscreenCanvas in Worker (crash worker, not main thread)
- Never use innerHTML with EXIF text — always textContent

---

## 7. Network Resilience

### Offline Impact Analysis

| Scenario | Impact | Solution |
|----------|--------|----------|
| Offline during image processing | **None** — Canvas API is local | No action needed |
| Offline during Nominatim geocoding | Location stamps fail | Queue requests, retry with backoff when online |
| Offline during ZIP download | **None** — Blob is local | Download triggers from memory |
| User closes tab mid-processing | Work lost | Warn via `beforeunload` event |

### NetworkManager Strategy

- Listen to `navigator.onLine` + `online`/`offline` events
- Show non-blocking banner: "Offline — processing continues, location lookup paused"
- Queue failed Nominatim requests in memory
- On reconnect: flush queue with 1 req/sec rate limit (Nominatim policy)
- Exponential backoff: 1s -> 2s -> 4s -> 8s -> max 30s (with jitter)
- Add random jitter (0-500ms) to prevent thundering herd on reconnect

---

## 8. Memory & Batch Processing Resilience

### The Problem

A single 20MP JPEG is ~6MB compressed but ~60MB decoded as RGBA bitmap on Canvas. 100 photos = 6GB. Browser will crash.

### Processing Guards

| Guard | Limit | Rationale |
|-------|-------|-----------|
| Max files per batch | 500 | UX sanity + DOM performance |
| Max file size | 50MB per file | Covers RAW-ish JPEGs from DSLRs |
| Concurrent processing | 2-4 images | Based on `navigator.hardwareConcurrency` |
| Memory watchdog | Monitor `performance.memory` | Pause queue if heap > 80% (Chrome only) |
| Chunk processing | Process in batches of N | Never decode all images simultaneously |

### Memory Management Rules

1. **Never decode all images at once** — queue them, process N at a time
2. **Release bitmaps immediately** — call `imageBitmap.close()` or `canvas.width = 0` after use
3. **Use Transferable objects** — `postMessage(buffer, [buffer])` for zero-copy to workers
4. **Chunk the ZIP** — JSZip supports streaming; don't hold all processed images in memory
5. **Test on mid-tier devices** — target 4GB RAM phones, not just 32GB dev machines

### Processing Queue Behavior

- Accept all dropped files into a lightweight queue (File references only, no decoding)
- Process N images concurrently (N = Math.min(navigator.hardwareConcurrency || 2, 4))
- Each worker: decode -> process -> encode -> release bitmap (explicit cleanup)
- Progress bar: "Processing 47/200 photos..."
- If one image fails: skip it, log error, continue batch
- User can cancel mid-batch (AbortController per worker)
- On memory pressure: reduce concurrency to 1, show warning

---

## 9. Nominatim Rate Limiting

### Why This Is Different From Server-Side Rate Limiting

Because our app is client-side, each user's browser calls Nominatim directly. Each user has their own IP. No coordination between users needed — just per-user client-side throttling.

### Strategy

**Level 1: Client-side throttle**
- Max 1 request per second per browser tab (Nominatim policy)
- Queue geocoding requests, drain at 1/sec
- Set proper User-Agent header: "PhotoOrder/1.0 (contact@photoorder.app)"
- Cache results in sessionStorage (same GPS coords = same city)

**Level 2: Smart batching**
- Group photos by GPS proximity (within ~1km)
- Geocode once per cluster, apply result to all photos in cluster
- 200 photos from same trip = ~5 geocode calls instead of 200

**Level 3: Fallback for heavy use**
- If user processes 1000+ photos regularly, suggest self-hosted Nominatim
- Phase 3 enhancement: offline geocoding via reverse-geocoding WASM module

**Level 4: CDN/Server protection (Cloudflare)**
- Cloudflare free tier: automatic DDoS protection
- Rate limiting rules: 100 req/min per IP to static assets (prevents scraping)
- Since server only serves HTML/JS/CSS, attack surface is minimal

---

## 10. Error Handling Strategy

### Global Error Boundary

- Catch unhandled errors and promise rejections
- Show user-friendly messages (never raw stack traces)
- Per-photo error recovery: skip failed photo, continue batch
- Log errors to console for debugging (no external error reporting — privacy)

### User-Facing Error Messages

| Error Type | Message | Recovery |
|------------|---------|----------|
| Invalid file type | "Not a valid image file" | Skip file, continue batch |
| File too large | "File exceeds 50MB limit" | Skip file |
| EXIF parse failure | "Could not read photo date/location" | Process without stamp |
| Geocoding failure | "Could not look up location" | Stamp date only |
| Canvas crash | "Error processing image" | Skip, continue batch |
| Memory pressure | "Low memory — processing paused" | Auto-resume at lower concurrency |
| Batch too large | "Max 500 photos per batch" | Reject with clear limit |

---

## 11. Canva Patterns — What We Adopt vs Skip

### Adopted

| Canva Pattern | Our Adaptation |
|---------------|----------------|
| Priority queues | Client-side processing queue with priority (user-visible first) |
| Exponential backoff + jitter | Nominatim retry strategy |
| Message idempotency | Dedup geocoding requests by GPS proximity |
| Strict CSP | Strict CSP on entire app via Cloudflare headers |
| File size/count limits | 500 files max, 50MB per file, concurrent processing cap |
| Async queue with backpressure | Processing queue pauses on memory pressure |
| Graceful degradation | Offline banner, skip failed photos, continue batch |

### Not Needed

| Canva Concern | Why It Doesn't Apply |
|---------------|---------------------|
| Server-side rate limiting | Our server serves static files only |
| Database scaling | No database |
| CSRF tokens | No server-side state mutations |
| Session management | No authentication (for now) |
| Multi-region replication | Cloudflare CDN handles this |
| Custom DDoS mitigation | Cloudflare free tier is sufficient |

---

## 12. Files to Create (When Coding Begins)

| File | Purpose | Priority |
|------|---------|----------|
| `src/lib/exif-sanitizer.ts` | 4-layer EXIF validation pipeline, polyglot detection, field allowlist | P0 |
| `src/lib/security.ts` | File validation (magic bytes), size limits, input guards | P0 |
| `public/_headers` | Cloudflare security headers (CSP, HSTS, X-Frame-Options) | P0 |
| `src/lib/processing-queue.ts` | Batch processing with concurrency control, memory watchdog, AbortController | P1 |
| `src/lib/network-manager.ts` | Online/offline detection, reconnect logic, geocoding queue | P1 |
| `src/lib/nominatim-client.ts` | Throttled geocoding client (1 req/sec), caching, GPS clustering | P1 |
| `src/lib/error-handler.ts` | Global error boundary, user-friendly messages, per-photo error recovery | P2 |

---

## 13. Verification Checklist

### Security Tests

| Test | Expected Result |
|------|-----------------|
| EXIF XSS injection (`<script>` in ImageDescription) | Script NOT executed, field stripped/ignored |
| Polyglot file (JPEG+HTML) | File rejected with "Invalid image file" |
| Oversized EXIF (10MB UserComment) | Parser timeout, photo marked "no EXIF" |
| Post-EOI steganography (100KB after JPEG end) | Warning shown, file still processable |
| Magic byte spoofing (.html renamed to .jpg) | Rejected — magic bytes don't match |
| CSP validation (deploy + check Console) | Zero CSP violations |
| innerHTML audit (`grep -r "innerHTML" src/`) | Zero results |
| eval audit (`grep -r "eval\|Function(" src/`) | Zero results |

### Resilience Tests

| Test | Expected Result |
|------|-----------------|
| Offline during batch processing | Processing continues, geocoding queued |
| Drop 500 x 20MP JPEGs | No crash, processes in chunks, progress shown |
| Nominatim throttle (100 GPS-tagged photos) | Max 1 req/sec, clustered to ~5-10 calls |
| Tab close during processing | beforeunload warning fires |
| Malformed EXIF date | Gracefully skipped, no stamp applied |
| GPS at (0,0) null island | Treated as "no GPS", no geocoding call |

### CI Checks

- `npm audit` (dependency vulnerabilities)
- `grep -r "innerHTML\|outerHTML\|insertAdjacentHTML\|document.write" src/` (must be empty)
- `grep -r "eval\|Function(" src/` (must be empty)
- CSP header validation via securityheaders.com after deploy
