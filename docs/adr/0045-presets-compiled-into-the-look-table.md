# ADR-0045: Lightroom presets compiled into the look table

- **Status:** Accepted (owner, 2026-11-07 and 2026-11-08); amended 2026-11-09 by measurement
- **Date:** 2026-11-08
- **Related:** FR-089, FR-070, FR-079, FR-115, ADR-0036, ADR-0038, ADR-0044

## Context

FR-089 reads a user's own Lightroom® presets (`.xmp`, legacy `.lrtemplate`, `.zip` packs). A preset holds far more than our sliders: point curves, whites and blacks, an eight-hue mixer, three-way colour grading, split toning, a black-and-white mixer, and spatial tools (clarity, texture, dehaze, sharpening, noise reduction, lens corrections). The PRD had planned only the sliders that match ours first, the rest listed as not applied. The owner asked on 2026-11-07 for curves, HSL and colour grading at once.

## Options considered

1. **Sliders only.** Pros: small. Cons: most presets look wrong; their character is in curves and colour grading.
2. **A second pipeline for preset tools.** Pros: faithful in principle. Cons: a new per-pixel path beside the table, its own cost and GPU twin (ADR-0039).
3. **Compile the per-pixel part into a 33³ LUT** kept as an imported LUT is (FR-070), the sliders that match ours to ours.

## Decision

Option 3 (`preset.rs` reads, `preset_look.rs` compiles):

- **Every per-pixel setting is baked into one table**: point curves (master and per channel, a monotone cubic with harmonic-mean tangents), whites and blacks, the eight-hue mixer (triangular hue weights, weighted by saturation so greys stay grey), colour grading and split toning by lightness, the black-and-white mixer. The table is written as a `.cube` and kept by `lut::import`, so a preset is a look with a `lut` and sliders: one table, one cost, the GPU path unchanged.
- **Amended 2026-11-08 (owner):** the eight-hue mixer is no longer baked: it goes to the look's own Color Mixer (`Adjust.mixer`, `colour_ops`), the same function as the user's sliders, so a preset's mixer stays adjustable and a preset of sliders and a mixer needs no table. Curves, whites and blacks, colour grading and the black-and-white mixer are still baked.
- **What no table can hold is named**, never dropped: spatial tools (clarity, texture, sharpening, noise reduction, lens corrections), the parametric curve (not read yet), Adobe's profiles; an absolute white balance and Highlights (amended 2026-11-09).
- **Amended 2026-11-09 (measured, RESULT §39):** a preset's **Dehaze** goes to ours (FR-115), one for one. An **absolute white balance** is no longer taken against daylight: on both measured photos it moved the colour the wrong way (they were shot below 5,500 K, so Lightroom warmed them), and how far to move depends on each photo's own as shot, so it is named as not applied until the app knows it (v0.14.1). A preset's **Highlights** is named as not applied too: Lightroom's is local, ours the same on every pixel, and at every strength it moved both photos away from Lightroom's render.
- **Mappings are first guesses until measured** (ADR-0044, the owner's rule on image methods): each strength is tuned by `preset_bench` against renders the owner exports from Lightroom (presets written by us, one group of sliders each), and the numbers go to RESULT §39.
- **No Adobe code or file**: the mappings follow what each setting is documented to do; fixtures and bench presets are written by us.

## Consequences

- A preset import writes a LUT file of about 1 MB per preset that needs a table; presets with sliders only write none.
- The look's amount scales both the sliders and the table, as Lightroom's preset Amount.
- Grain and vignette are kept in the look and drawn once session 13 builds them.
