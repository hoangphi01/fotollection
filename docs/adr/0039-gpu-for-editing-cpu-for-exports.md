# ADR-0039: The GPU for editing, the CPU for exports

- **Status:** Accepted (owner, 2026-11-05); stages measured one at a time
- **Date:** 2026-11-05
- **Related:** FR-079, FR-057, FR-072, NFR-002, NFR-003, NFR-008, NFR-017, ADR-0038

## Context

Every pixel Fotollection works on is computed on the CPU, in Rust, on all cores (rayon) with SIMD. The webview's GPU only draws the window. The owner asked (2026-11-05) to use the hardware fully, as Lightroom does with the GPU, since the app is offline.

Research (2026-11-05):

- **Lightroom Classic** has three separate switches: the GPU for display, for image processing (2019) and for export (2022). GPU export is twice as fast for some users and twice as slow for others.
- **darktable** (OpenCL) and **Capture One** keep a CPU path for every step and fall back to it when the GPU fails. Their support pages list driver bugs: glitched previews, artefacts in processed files, crashes on export.
- **On our stack:**
  - **WebGL2** is in all three webviews and adds no bytes to the installer. On Linux, WebKitGTK may run it slowly in software without an error.
  - **WebGPU** is missing on Linux and on macOS before 26.
  - **`wgpu` in the core** (MIT/Apache) would add an estimated 2–5 MB (unmeasured) to installers already at the 25 MB budget. Its float results differ between GPUs and drivers.

Measured in Fotollection, 2026-11-05:

- One step of a colour slider's drag costs 11–16 ms in Rust, of which the look is 2 ms; the rest is the JPEG encode, IPC and the webview's decode.
- A RAW develop costs 0.4–2.2 s.
- Exports run at 10–23 photos/s, mostly JPEG decode and encode, which are serial per photo.

## Options considered

1. **CPU only:** exact and simple, but every interactive step pays the JPEG round trip, and heavy features (denoise, local adjustments) would be slow.
2. **The GPU everywhere, exports included:** fast where drivers behave, but prints would no longer be the same byte for byte on every machine, and the tests that compare bytes would no longer hold.
3. **The GPU for editing, the CPU for exports:** the interactive view runs on the GPU where it helps and can fall back; every export, thumbnail and test keeps the Rust CPU path as the reference.

## Decision

Option 3.

- **Editing on the GPU, in stages, each measured first:**
  1. Profile one slider step (render, encode, IPC, decode and paint) and a RAW develop (unpack, demosaic, the rest).
  2. Try the cheap CPU fixes during a drag: a smaller or lower-quality preview.
  3. **WebGL2 preview of the look:**
     - Rust bakes the 33³ table, and the shader only interpolates it (tetrahedral, with `texelFetch`), so the maths has one source.
     - A parity test keeps it within ΔE2000 ≤ 1 of the CPU render.
     - On release, the CPU render replaces it, so what the user sees settled is what prints.
     - Zoom and pan follow.
  4. **`wgpu` in the core, only when a feature needs it** (denoise, Selective at full resolution):
     - compute only, behind a cargo feature;
     - a CPU twin for every kernel;
     - started lazily off the main thread;
     - a fallback to the CPU on device loss or timeout;
     - CI on software adapters with tolerance tests.

     Before any feature is built on it, measure the size it adds to each installer.
- **Always a way out:**
  - a setting turns the GPU off, as Lightroom's and Capture One's do;
  - WebGL is off by default on Linux unless a frame-time probe shows real hardware;
  - every failure falls back to the CPU path, with its `ERR-` code in the log.
- **Exports stay on the CPU and use every core.** Lightroom keeps GPU export as a separate opt-in, and so would we, if ever. The CPU export is made faster where measurements show the time goes:
  - photos in flight set by memory (`export::inflight_for`);
  - decode, render and encode overlapped across photos;
  - the parallel parts (resize, look, colour conversion) on rayon inside each photo, without oversubscribing the pool.

## Consequences

- Prints stay exact and reproducible; golden and byte tests stay CPU-only.
- Two implementations of the interpolation (Rust and GLSL) must not drift: the parity test guards it, and curves are computed only in Rust.
- New rows when each stage lands: OPTIMIZATION budgets and log, ERROR_CONTROL codes for GPU fallbacks, a setting in DATABASE §6, LESSONS for driver quirks.
- The installer budget (NFR-008) decides about `wgpu`. The `.msi` and amd64 `.deb` are already just over it at v0.9.8.
