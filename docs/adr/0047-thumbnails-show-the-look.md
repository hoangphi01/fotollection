# ADR-0047: Thumbnails show the look

- **Status:** Accepted (owner, 2026-11-08); supersedes the "thumbnails stay plain" line of [ADR-0036](0036-colour-looks-cube-luts-first.md) and [ADR-0038](0038-moderate-colour-editor-on-one-lut-engine.md)
- **Date:** 2026-11-08
- **Related:** FR-114, FR-079, FR-070, FR-089, ADR-0041, ADR-0044

## Context

Since looks were planned (ADR-0036) the grid's thumbnails stayed plain, "so a look never invalidates the cache": a look was a photo-by-photo edit seen in the preview. FR-114 gives a look to hundreds of photos at once from the gallery. The owner gave black and white to 96 photos and the grid did not change (2026-11-08): the one place a batch is judged showed nothing.

## Options considered

1. **Keep thumbnails plain.** Pros: nothing to make. Cons: a batch cannot be seen where it is made; Lightroom's grid shows each photo's develop.
2. **A coloured T1 on disk per look.** Pros: survives a restart. Cons: a file for every photo and colour, stale ones left to the cache limit, a key that must follow every look edit.
3. **Colour on serving, kept in memory.** The plain T1 stays on disk as it is; the route decodes it, applies the look's table and encodes it again, and keeps the bytes in memory.

## Decision

Option 3 (`crates/core/src/look_thumbs.rs`, `shown_thumbnail_bytes`):

- **Rust colours it:** the photo's effective look (its own, or the shared one it follows), a RAW's linear sliders left out as in its preview (`split_look`); the cached table, rayon, JPEG q82. A photo without a look is served its T1 unchanged.
- **Measured** (`measure_look_thumbs`, release, 24 Prague JPEGs, RESULT §37): 1.08 ms p50, 1.19 ms p95 a thumbnail; 500 on the two-thread background pool in 373 ms.
- **Kept in memory, bounded:** at most 32 MB (about 1,000), oldest first out, counted in the memory the Settings window shows, emptied on a project switch, when memory runs short and by *Clear cache*. Nothing new on disk.
- **The address follows the colour:** `PhotoSummary.look`, a short key of the effective colour (0 without one), is part of the thumbnail's URL, so a changed look is a new address and the grid fetches it again; an edit's `library-changed` brings the new keys.
- **A batch shows its progress:** after `apply_look_to_photos`, the batch's thumbnails are made on the background pool with the import's progress bar (`task: "looks"`); a newer batch stops an older one's.
- **What stays plain:** the T1 on disk, the look tiles' and the collage sheets' sources, which start from it.

## Consequences

- The grid, the strips, the quick preview and the sheet preview's list show each photo's colour in every mode.
- A grid scrolled with many coloured photos costs about a millisecond a new tile on a pool thread; the first view after a restart colours them again.
- A dialog that shows thumbnails by id only (place borrowing, clock matching) shows them coloured, without a key to refresh them if the look changes while it is open.
