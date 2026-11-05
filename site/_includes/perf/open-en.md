### Before: every photo decoded when it was clicked

Up to v0.10.0, a preview was made when you asked for it: the file read, the photo decoded (a RAW developed), turned upright, made 2,048 px and converted to sRGB, then cropped, coloured, stamped and encoded for the screen. On a 20 MP photo from a Lumix GX9 that took 274 to 278 ms for a JPEG and 440 to 531 ms for a RAW, every time a photo was opened that was not among the last eight. Fast enough for one photo; not for skimming a trip.

### Standard previews, as Lightroom keeps them

Since v0.11.0, the work the user waits for is done before they ask. Once a folder's thumbnails are ready, Fotollection makes each photo's 2,048 px copy in the background, one photo at a time per thread, and keeps it on disk next to the thumbnails, as a JPEG of about 0.66 MB (about 280 MB for the whole trip). A click on any photo, near or far, in this session or the next, starts from that copy: 27 ms for a JPEG and for a RAW alike. What is left is the preview's own work (crop, resize, colour, date stamp and encode), the same 27 ms as from memory. A preview made from the disk copy differs from a fresh one by 1.35 levels of 255 on average, too little to see.

### Memory used well, not as little as possible

A preview app that keeps eight photos in memory on a 16 GB laptop wastes the laptop. Fotollection now gives the preview a quarter of the machine's memory (on a 16 GB Mac, about 190 photos at 2,048 px, 20 RAW develops kept for their sliders and 600 MB for zoom), counted in bytes and freed oldest first. When a photo is on screen, the next two in the direction you are moving and the one behind are loaded too, so the arrow keys never wait. The 2 GB ceiling still holds for exports.

### The photo on screen goes first

Only one RAW is developed at a time, so that a batch of RAW photos fits in memory. Before, the order of the queue was luck: a preview being prepared, the export's size estimate or an export could make the photo you clicked wait. Now every develop says who is waiting for it, and work in the background steps aside whenever the photo on screen needs one. The same work also showed two rare hangs, now gone: one where every thread waited for a develop that had no thread left to run on, and one where the screen waited for an image it had never asked for.

| Version | JPEG p50 / p95 | RAW p50 / p95 | How |
|---|---|---|---|
| v0.9.0 | 274 / 286 ms | 531 / 550 ms | Decoded when clicked; RAW through macOS |
| v0.10.0 | 278 / 293 ms | 440 / 471 ms | Decoded when clicked; RAW through our own develop |
| **v0.11.0** | **27 / 32 ms** | **27 / 30 ms** | From the standard preview on disk, or memory |

Measured on the owner's Prague trip (396 JPEG and 26 RAW, 20 MP) on an Apple M3 with `measure_browse`: v0.9.0 built from its own tag, v0.10.0 on its code just before this change.
