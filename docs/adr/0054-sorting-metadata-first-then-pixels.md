# ADR-0054: Sorting a camera roll by its metadata first, the pixels for the rest

- **Status:** Accepted
- **Date:** 2026-11-17
- **Related:** FR-141, FR-012, FR-015, FR-022, NFR-022, ADR-0018, ADR-0019, ADR-0044

## Context

The owner's phone roll on a NAS holds 14,639 files: HEIC, JPG, PNG and MOV, Live Photo videos, screenshots, photos taken through other apps, images saved or received from elsewhere, edited copies and HEIC/JPG twins, under names (`IMG_0596.HEIC`, `IMG_0596_1.MOV`, `IMG_0599_1.PNG`) that carry little meaning and repeat once a phone's counter wraps at 9999. Fotollection's grouping today (FR-041, ADR-0019) finds bursts of similar shots; it cannot say where a file came from. The owner wants it sorted on the desktop, where the Rust core reads thousands of files on every core (ADR-0044), and urgently: v0.14.4 is given to it (2026-11-17).

Research of 2026-11-17 (ExifTool's Apple tag table, forum reports, the event-clustering papers) found that most classes have a near-certain metadata signal (Apple's MakerNote for the phone's own camera, `ContentIdentifier` for a Live Photo's two halves, `BurstUUID`, `PhotoIdentifier` for format twins, `UserComment` "Screenshot"), while saved and received images often carry no metadata at all.

## Options considered

1. **Pixels for everything** (an image model over all 14,639 files). Simple to describe; slow over a NAS, probabilistic where the metadata is certain, and it cannot pair a still with its video.
2. **File names and dates.** Fast; names wrap and collide, a NAS copy rewrites file times, and saved images take the save date.
3. **Metadata first, then names, then pixels only for the undecided.** Certain where the metadata speaks, every decision carrying the rule that made it; the model's cost paid only on the remainder (an estimated 10–20 %).

## Decision

Option 3. The classes, stacks, time and events come from bounded parsers in the core (EXIF, Apple's MakerNote, PNG chunks, QuickTime `moov/meta`), the class rules held as data; events by PhotoTOC's adaptive gaps (Platt et al. 2003) with a split where the place jumps; duplicates by perceptual hashes of the thumbnails already made, compared within a day; for the undecided, CLIP ViT-B/32 (OpenAI, MIT licence) run on the device through tract, as YuNet and FaceMesh are (ADR-0018), its text side computed at build time. Apple's MobileCLIP is not used (its licence is not permissive); Immich and PhotoPrism (AGPL) and digiKam (GPL) are read for their ideas only. Files are never changed, renamed or deleted.

## Consequences

- Import reads more of each header (the MakerNote, a video's metadata); NFR-022 sets the budgets from a first measurement on the owner's files (FR-141 S0).
- MOV files are imported for the first time, as companions of a still, never edited.
- CLIP adds tens of megabytes; if the installer would pass NFR-008's 150 MB, the owner chooses between an optional model pack downloaded once and a larger budget (FR-141 S6).
- FR-012 and FR-015 move from 2027-Q1 to v0.14.4; the looks of v0.14.4 move to v0.14.5.
