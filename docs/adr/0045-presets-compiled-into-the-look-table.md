# ADR-0045: Lightroom presets compiled into the look table

- **Status:** Accepted (owner, 2026-11-07 and 2026-11-08)
- **Date:** 2026-11-08
- **Related:** FR-089, FR-070, FR-079, ADR-0036, ADR-0038, ADR-0044

## Context

FR-089 reads a user's own Lightroom® presets (`.xmp`, legacy `.lrtemplate`, `.zip` packs). A preset holds far more than our sliders: point curves, whites and blacks, an eight-hue mixer, three-way colour grading, split toning, a black-and-white mixer, and spatial tools (clarity, texture, dehaze, sharpening, noise reduction, lens corrections). The PRD had planned only the sliders that match ours first, the rest listed as not applied. The owner asked on 2026-11-07 for curves, HSL and colour grading at once.

## Options considered

1. **Sliders only.** Pros: small. Cons: most presets look wrong; their character is in curves and colour grading.
2. **A second pipeline for preset tools.** Pros: faithful in principle. Cons: a new per-pixel path beside the table, its own cost and GPU twin (ADR-0039).
3. **Compile the per-pixel part into a 33³ LUT** kept as an imported LUT is (FR-070), the sliders that match ours to ours.

## Decision

Option 3 (`preset.rs` reads, `preset_look.rs` compiles):

- **Every per-pixel setting is baked into one table**: point curves (master and per channel, a monotone cubic with harmonic-mean tangents), whites and blacks, the eight-hue mixer (triangular hue weights, weighted by saturation so greys stay grey), colour grading and split toning by lightness, the black-and-white mixer. The table is written as a `.cube` and kept by `lut::import`, so a preset is a look with a `lut` and sliders: one table, one cost, the GPU path unchanged.
- **What no table can hold is named**, never dropped: spatial tools (clarity, texture, dehaze, sharpening, noise reduction, lens corrections), the parametric curve (not read yet), Adobe's profiles; an absolute white balance is taken against daylight (5,500 K) and marked approximate.
- **Mappings are first guesses until measured** (ADR-0044, the owner's rule on image methods): each strength is tuned by `preset_bench` against renders the owner exports from Lightroom (presets written by us, one group of sliders each), and the numbers go to RESULT §36.
- **No Adobe code or file**: the mappings follow what each setting is documented to do; fixtures and bench presets are written by us.

## Consequences

- A preset import writes a LUT file of about 1 MB per preset that needs a table; presets with sliders only write none.
- The look's amount scales both the sliders and the table, as Lightroom's preset Amount.
- Grain and vignette are kept in the look and drawn once session 13 builds them.
