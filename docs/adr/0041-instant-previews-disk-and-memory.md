# ADR-0041: Instant previews: standard previews on disk, a quarter of the RAM, the photo on screen first

- **Status:** Accepted (owner, 2026-11-05)
- **Date:** 2026-11-05
- **Related:** FR-091, FR-007, FR-057, FR-086, NFR-002, NFR-004, NFR-012, ADR-0039, ADR-0040

## Context

The owner asked for photos to open "like Lightroom: once the gallery has loaded, any photo clicked shows almost at once", and set the principle: **use the RAM well, not as little as possible**.

Until now the preview decoded each photo when it was asked for. Its memory was capped by count: 8 sources of 2048 px, 3 RAW develops and 200 MB of zoom images, the sizes of NFR-004's thinking (stay small). Only the 400 px T1 thumbnail was kept on disk. On the Prague trip (396 JPEG, 26 RW2, M3), `measure_browse` timed a photo opened with nothing kept at 278 ms p50 for a JPEG and 440 ms for a RAW. The export's estimate for the photo on screen developed the same RAW again, and both waited on one develop lock in no particular order.

## Options considered

1. **Memory only:** prefetch the neighbours and keep more in RAM. Next and previous become instant, but a photo far away still decodes, and nothing survives a restart.
2. **Disk and memory** (Lightroom's *standard previews*): after the thumbnails, each photo's 2048 px source is made once in the background and kept in the cache folder. Any photo opens from it, near or far, this session or the next. It costs disk (0.66 MB a photo measured) and a background pass as long as a decode of every photo.
3. **A fixed memory budget** (2 GB) against **a share of the machine's RAM**: fixed is simple, but leaves a 32 GB machine idle and crowds an 8 GB one.

## Decision

Option 2, with memory sized by the machine (the owner's choices of 2026-11-05):

- **Standard previews on disk:** `thumbs/<aa>/<key>.t2-o<orientation>-<raw look>.jpg`, the upright sRGB source before any look, JPEG q90. For a RAW, finished with its RAW colour and no linear slider. A RAW with linear sliders set uses its kept develop instead (FR-088). They are made after the T1 pass, one photo at a time on a plain thread, paused while an export or an analysis runs (the prefetch is not paused: it is small and the user is browsing).
- **The cache limit is 5 GiB** (was 2 GiB): thumbnails, standard previews and analyses together, oldest first.
- **Memory:** the preview keeps up to **a quarter of the machine's RAM**, split 60 / 25 / 15 between sources, RAW develops and zoom images, by bytes, the oldest first. The image on screen always stays. NFR-004's 2 GB stays the export's ceiling.
- **Prefetch:** the next two photos in the direction of travel and the one behind, once the photo on screen has landed.
- **The photo on screen first:** the one-develop-at-a-time lock becomes a gate with two priorities. Work the user does not wait for (standard previews, prefetch, the export's estimate, an export) marks itself `Background` (`raw::with_priority`) and waits while any `Interactive` develop waits.

### Amended in v0.11.1 (audit of 2026-11-06)

- **One owner, one way in:** a `clear` (another project) bumps an epoch, and an image asked for before it is never kept after it.
- **The photo on screen stays:** a prefetch is kept just below the newest and keeps no full image for zoom; it runs on one worker thread whose queue the latest request replaces, and a photo no longer wanted when its develop's turn comes is dropped.
- **Made once:** background work skips a photo another thread is making; the photo on screen never waits for it, but reads the standard preview if it appeared while it waited for the develop. Each writer has its own `.part`.
- **Priority:** a develop is `Background` unless marked; the photo on screen's claim lasts while it is on screen (`raw::show`).
- **The disk:** standard previews carry their own version in the name; over the limit, previews go before thumbnails, and the limit is checked every 100 previews during the pass.

## Consequences

- Any photo opens in about 27 ms p50 from its standard preview or from memory, JPEG and RAW alike (RESULT §29). NFR-012 is met for any photo, not only the neighbours.
- The process may hold more memory while browsing (up to a quarter of RAM plus the app). Docs that promised "under 2 GB" now say that this is the export's ceiling. Since v0.12.0 the previews give their memory back when the machine runs short, and the share, the cache limit and the preparation are the user's to set (FR-086).
- A disk preview differs from a fresh one by about 1.35 levels in 255 on average (JPEG q90). A decoder fix that changes pixels still bumps `thumbs::CACHE_VERSION`, which remakes them too.
- The Settings window (FR-086) shows the cache and the memory budget, so the user can manage them.
- DATABASE §8, OPTIMIZATION §3 and §4.1, ARCHITECTURE, the user guide and the README's requirements row change with this.
