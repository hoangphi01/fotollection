# ADR-0060: macOS's RAW develop runs in a child process

- **Status:** Accepted
- **Date:** 2026-11-20
- **Related:** NFR-004, NFR-017, FR-085, FR-091, ADR-0017 (only `oscodec` holds `unsafe`), ADR-0029 (RAW decoders in order), ADR-0048 (contained threads)

## Context

On macOS, Panasonic (RW2) and Olympus (ORF) files are developed by the system (`ImageIO`) first, for its lens correction (FR-085). The monthly run of November found the Prague trip's export at 3.62 GB against NFR-004's 2 GB; measured on 2026-11-20 on REF-MAC through `raw::decode`, the app's own path:

- One develop peaks at **2.42 GB** of footprint: 1.6 GB of IOSurface (Core Image on the GPU, given back when the develop ends) and 0.71 GB of malloc that stays.
- Each further develop keeps about **120 MB more**, never given back: 40 develops of one file reach 6.44 GB. Our own develop stays flat at about 0.66 GB.
- Nothing in process frees it: `kCGImageSourceShouldCache = false`, `CGImageSourceRemoveCacheAtIndex` on every index, and an autorelease pool around the develop each left the growth as it was.
- The memory kept after the first develop is not all waste: the first develop of a process takes 590–635 ms, the next ones 370–395 ms, flat to the tenth. The warm pipeline is worth about 220 ms a photo; the 120 MB a photo is dead.
- The system's preview (thumbnail) plateaus near 100 MB, its dimensions and the HEIF decode stay flat: only the full develop grows.

## Options considered

| Option | Memory | Speed | Lens correction |
|--------|--------|-------|-----------------|
| A1. A child process per develop | no growth; one develop's 2.4 GB peak stays | about +60 % a RAW (the 220 ms warm-up every photo) | kept |
| **A2. One child serving develops, replaced after N or when idle** | no growth; summed peak 3.46 GB on the trip | a RAW export 8.5 % slower at N = 12 (the pipe and the warm-ups), the whole trip within its spread | kept |
| B. Our own develop first for RW2 and ORF | ≤ 2 GB | as before | lost for Panasonic and Olympus |
| C. Leave it, restate the budget only | grows 120 MB a RAW until the app quits | as before | kept |

A2 with C's restatement was chosen by the owner (2026-11-20): "use RAM well" means memory that buys speed is kept and memory that buys nothing is given back. A child cannot remove the single develop's IOSurface peak, so the budget for RAW through `ImageIO` is restated (below); every other format keeps 2 GB.

## Decision

1. **The child is the app's own executable started again** with `--os-decode` as its first argument (`raw_child::serve()`, the first statement of `main`, before any window, log or thread): nothing more to bundle or sign. Tools that measure the app's path (`measure_folder`, `quality_export`) do the same; without `raw_child::use_child` (unit tests, other tools) the develop stays in process.
2. **One child serves develops in turn** over its pipes and is replaced after `DEVELOPS_PER_CHILD` = 12 develops or 30 s without one. Each replacement costs the warm-up again: at 4, 26 RW2 alone exported 11.6 % slower than in process (13.0 s against 10.9 s, medians of 5); at 12, 8.5 % (11.84 against 10.91 s), the summed peak the same (3.09 and 3.12 GB). The owner (2026-11-21): "a few dozen MB are not worth making the app tens of percent slower". The child runs inside the develop gate, so there is at most one, and the lock over it is held only for the request and its answer.
3. **The protocol** (version 1): `FOSD`, the version, a sequence number, the RAW kind and the file's bytes in; the same magic and sequence as soon as the request is read, then a status, the size, the profile's length, RGB and the profile out. The answer is untrusted input: every size is checked before anything is allocated.
4. **Failures:**
   - A child that cannot start, or a new one that does not take the request: the develop runs in process as before, logged once as `raw.child_failed` (ERR-RUS-0007); the next develop tries a child again.
   - A child that had served before and does not take the request (it went idle), or answers out of turn (bytes left from an earlier answer): replaced once, the file sent again.
   - A child that dies, hangs past 120 s or answers wrongly after taking a file: that file fails for `ImageIO` and the next decoder develops it (ours, without the system's lens correction); `ImageIO` never runs on it in process, where it could take the app down.
5. **NFR-004 restated** for RAW through `ImageIO` on macOS: the parent and the child together at most **3.5 GB** of footprint at the export's peak, judged on the **median of 3 runs** (owner, 2026-11-21, as every measurement here is a median), and no growth with the number of photos. Measured on the Prague trip at 6 in flight with N = 12: 3.46 GB (median of 3; 3.42–3.55 GB, the worst run above the line), against 6.14 GB before; 26 RW2 alone 3.12 GB against 5.68 GB; JPEG alone 0.85 GB either way, within 2 GB.

## Consequences

- An export of hundreds of RAW files no longer takes more memory than one of ten; the dead memory goes back every twelve photos and when the app is idle. After the last RAW the child can hold up to about 2 GB (0.75 GB and 120 MB a develop) for up to 30 s before it exits; if that ever matters, the lever is the idle time or ending the child when an export finishes, not the count.
- A RAW export is about 8.5 % slower than in process (about 35 ms a photo: the pipe carries about 80 MB a develop, and a warm-up every twelve); shared memory would need `unsafe` outside `oscodec` (ADR-0017), so it stays a pipe.
- The pixels are the same: 26 RW2 developed in process and through the child are byte-identical, the profile included, cold, warm and after a replacement.
- A crash of `ImageIO` on a damaged file costs that file's lens correction, not the app.
- Windows (WIC) and Linux are unchanged; whether WIC keeps memory the same way is measured on the ThinkPad next.
- The summed peak is close to its budget; the parent's share at the peak (about 2.1 GB) is profiled before more work is added to the export.
