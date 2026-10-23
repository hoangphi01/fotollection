# ADR-0011: Photo Style looks via baked 3D LUTs, global adjustments only

- **Status:** Accepted
- **Date:** 2026-10-23
- **Related:** FR-038, FR-039, NFR-002, NFR-003, ADR-0010

## Context

The owner shoots a Lumix GX9 with in-camera **Photo Style** recipes (for example "C200 Natural") and wants the same kind of looks in Fotollection, saved as presets. Until now the PRD excluded all photo editing, to keep the app focused on fast print preparation. A full editor (masks, HSL, curves, RAW development) would turn Fotollection into a Lightroom or darktable competitor, and would endanger the performance budgets.

## Options considered

1. **No looks:** the app stays focused, but it misses a feature the owner and retro-minded users clearly want.
2. **Full editor:** maximum flexibility, but huge scope, a UI that competes with mature tools, and it moves the app away from "print fast".
3. **Imported `.cube` LUTs only:** cheap, but opaque to users, with no camera-like controls, and arbitrary third-party files become an input surface.
4. **Camera-style parametric looks (Lumix Photo Style model), baked into a 3D LUT:** a small, familiar set of controls (base style, contrast, sharpness, saturation, hue, highlight/shadow, WB shift, grain), with a fixed cost per pixel.

## Decision

Implement option 4.

- **Parameters only:** a look is a base style plus bounded parameters, all stored in presets ([DATABASE.md §7](../DATABASE.md#7-presets)).
- **Engine:** when parameters change, bake the colour part (base style, saturation, hue, WB shift, contrast) into a **33³ 3D LUT** plus a **1D tone curve** (highlight/shadow). At render time, apply one trilinear lookup per pixel with SIMD and rayon. Grain is a deterministic, seeded, tiled noise. Sharpness feeds the existing sharpen stage ([FR-029](../PRD.md#fr-029--output-sharpening)).
- **Order:** crop → look → border/stamp/caption → sRGB conversion → sharpen → encode. Overlays are never tinted.
- **Boundary:** global adjustments only. No local masks, HSL panel, curve editor, retouching or RAW development controls. Custom `.cube` import may be reconsidered later in a new ADR.

## Consequences

- The cost is predictable and does not depend on how many parameters are set: about 2 ms per 2 MP preview, and about 20 ms per 24 MP export (to be benchmarked).
- Looks are applied to already-rendered JPEG/HEIC, so they approximate camera styles rather than replicate them exactly. The docs and UI say "inspired by".
- Thumbnails stay unstyled, so preset changes never invalidate the cache ([DATABASE.md §8](../DATABASE.md#8-thumbnail-cache)).
- Golden-image tests cover each built-in look ([TESTING.md §5](../TESTING.md#5-golden-images)).
