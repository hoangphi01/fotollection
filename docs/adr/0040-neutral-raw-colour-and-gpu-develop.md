# ADR-0040: Neutral RAW colour by default, and the RAW develop on the GPU

- **Status:** Accepted (owner, 2026-11-05)
- **Date:** 2026-11-05
- **Related:** FR-072, FR-079, FR-016, NFR-008, NFR-017, ADR-0037, ADR-0039

## Context

ADR-0037 plans our own develop core for v0.11.0. Two questions were left open, and the owner settled both on 2026-11-05:

- **Colour.** Today a RAW photo is fitted to the camera's embedded JPEG (`match_look`), so it looks like what the camera showed. That look is the camera maker's, changes between models, and is not what the sensor saw. Lightroom and darktable open RAW photos with a neutral rendering and offer the camera's look as a profile.
- **Speed.** A 24 MP DNG takes about 390 ms to open, of which demosaic to finish is about 280 ms ([RESULT §22](../RESULT.md#22-the-gpu-while-editing-and-every-core-for-exports-2026-11-05)). RCD and Markesteijn cost more than Malvar. ADR-0039 left `wgpu` until a feature needs it and the installer budget allows; the `.msi` and amd64 `.deb` are already just over NFR-008's 25 MB.

## Options considered

1. **Colour: the camera's look by default** (today). Familiar, but a guess fitted to a JPEG, and not the sensor's colour.
2. **Colour: neutral by default, the camera's look as a built-in look.** The sensor's colour through its matrix and a gentle tone curve; "Like the camera's JPEG" one click away in the Looks card, for RAW photos only.
3. **Speed: the new develop on the CPU only.** No new dependency, but opening a RAW photo would get slower than today.
4. **Speed: the develop on the GPU with `wgpu`, a CPU twin for every kernel.** Faster previews and zoom; a larger installer.

## Decision

Options 2 and 4.

- **Neutral colour is the default for RAW photos.** The camera's look becomes the built-in look "Like the camera's JPEG" (from `match_look`, its constants named and explained), shown in the Looks card only for RAW photos.
- **The develop runs on the GPU with `wgpu` in v0.11.0**, compute only, for the preview, zoom and thumbnails. **Exports stay on the CPU** (ADR-0039), so prints are the same on every machine. Every kernel has a CPU twin; the parity of the two is ΔE2000 mean and p99 ≤ 1 on the nine samples.
- **No switch.** As in ADR-0039, the app decides: no hardware adapter, or a test run slower than the CPU, means the CPU; a lost device or a timeout falls back to the CPU and writes an `ERR-GPU-*` code to the log.
- **The installer budget grows.** The owner accepts a larger installer for this; NFR-008's new number (proposed 30 MB for `.msi`, `.deb` and `-setup.exe`) is set in session 19 once `wgpu`'s size is measured, keeping its ID.

## Consequences

- Session 15 sets the score card first: synthetic charts mosaicked and scored (`crates/rawcore/examples/charts.rs`, `bench/results/raw-quality.json`), and the budgets in [OPTIMIZATION §3](../OPTIMIZATION.md#3-budgets).
- RAW thumbnails do not change: they are the camera's embedded preview, as in Lightroom (owner, 2026-11-05), so `thumbs::CACHE_VERSION` stays; the preview cache is keyed by the choice.
- Two implementations of each develop step (Rust and WGSL) must not drift: the parity test guards it, the CPU path stays the reference, and golden tests stay CPU-only. CI runs the GPU path on software adapters with a tolerance.
- New docs when each step lands: NFR-008 in the PRD and STATUS, `ERR-GPU-*` and the unsupported sensor kind in ERROR_CONTROL, ARCHITECTURE (`rawcore`, `rawlgpl::unpack`), TESTING, the RAW chapter of the manual.
- Amends ADR-0037 (the camera's look is no longer the default) and ADR-0039 (`wgpu` for the RAW develop now, not only for later features).

## Stage results

- **Neutral and the camera's look (session 17b, 2026-11-05, [RESULT §27](../RESULT.md#27-raw-colour-neutral-or-like-the-cameras-jpeg-v0110-session-17b)):** the neutral rendering keeps the lens shading and the exposure fitted from the camera's JPEG (a property of the lens and of the shot, not of the look), with a gentle curve and a soft toe; the S5's greys stay neutral (C\* 1.39). "Like the camera's JPEG" fits both ways measured in session 16 on a small copy and keeps the closer one per file (ΔE2000 per pixel; a block-average score picked wrong for two of seven cameras): better or equal on every Bayer sample (D750 2.26 → 1.16, GX9 3.85 → 2.76). The owner chose a card among the colour cards, *Apply to all RAW photos*, no setting, and the camera's look for projects from before v0.11.0.
