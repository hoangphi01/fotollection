# ADR-0044: Rust first: the guiding principle for every design and every doc

- **Status:** Accepted (owner, 2026-11-07)
- **Date:** 2026-11-07
- **Related:** NFR-002, NFR-003, NFR-004, NFR-008, ADR-0004, ADR-0039, ADR-0041, ADR-0043

## Context

The owner, 2026-11-07: "always remember our strongest advantage: we use Rust, not Electron; make the most of Rust's power; note it as the compass for every doc." Fotollection sits between two kinds of neighbours:

- **Apps built on a web runtime (Electron and the like):** a browser engine shipped with every copy, hundreds of MB of installer and RAM, image work in JavaScript or behind it.
- **RawTherapee and darktable:** native C and C++ ([ADR-0043](0043-lossless-export-f32-path-tiff.md)). As fast as native code gets, but with manual memory, threads synchronised by hand, decades of code, and parsers of untrusted files written in a language that does not check bounds.

Our core is Rust behind a thin Tauri shell: the system's own webview draws the interface, and every pixel is Rust's.

## Options considered

1. **Treat Rust as an implementation detail.** Pros: nothing to change. Cons: work drifts into the webview (it is easy to write there), the advantage is spent without anyone deciding to.
2. **Make Rust's strengths a rule every design and doc is checked against.** Pros: the edge is kept on purpose and measured. Cons: some UI-side shortcuts are refused.

## Decision

Option 2. Every design, plan and doc asks first: *does this use what Rust gives us?*

- **Every pixel and every file in Rust.** Decoding, developing, looks, resizing, encoding, parsing (RAW, LUT, preset, project, EXIF) run in the core, never in the webview; the webview draws what Rust answered, and the GPU only shows a look while a slider moves, checked against Rust's render (ADR-0039).
- **Parallel by default, safely.** Work on pixels and on batches uses rayon on pools that no waiting caller can hold (LESSONS 2026-11-05); the compiler rules out data races, so parallelism is the first design, not an optimisation added later.
- **Safe parsers for untrusted files.** No `unsafe` in our crates, bounds always checked, every input capped and fuzzed or mutation-tested: what C and C++ editors must police by hand.
- **Small and lean.** The system webview, no bundled browser: installers within NFR-008, memory used for what the user sees (instant previews, ADR-0041), never for a runtime.
- **Types across the boundary.** Every IPC command is typed in Rust and generated for TypeScript (specta), validated in Rust; the webview cannot send what Rust does not accept.
- **Measured, never assumed.** A claim that Rust makes something fast is a number in RESULT, against a budget in OPTIMIZATION §3, as every other claim.

## Consequences

- Plans and ADRs say how a feature uses these strengths (where its work runs, how it parallelises, how its input is bounded); a design that puts pixel or file work in the webview needs a reason a reviewer accepts.
- The pre-work skill and CLAUDE.md name this ADR, so every session starts from it.
- What we say in public stays accurate: against web-runtime apps, size and memory; against RawTherapee and darktable, memory safety, safe parallelism and a modern core, not "native versus web" (they are native too).
