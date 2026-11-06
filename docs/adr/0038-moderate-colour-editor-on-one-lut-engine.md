# ADR-0038: A moderate colour editor on one LUT engine, before the develop core

- **Status:** Accepted (planned for v0.10.0); amended by FR-101 and FR-102 (2026-11-06): the moderate editor grows to healing and masks, as local layers beside the one global table, still no layers to stack or compositing; amended by FR-088 (2026-11-05): on a camera RAW, brightness, highlights, shadows, warmth and tint act in linear light before the finish, the rest of the look stays in the table
- **Date:** 2026-11-05
- **Related:** FR-079, FR-070, FR-039, FR-047, FR-044, FR-040, NFR-002, NFR-003, ADR-0011, ADR-0016, ADR-0036, ADR-0037
- **Amends:** [ADR-0011](0011-photo-style-looks.md) (global curves, split toning and vibrance are now in scope), [ADR-0036](0036-colour-looks-cube-luts-first.md) (looks ship in v0.10.0, on encoded values), [ADR-0037](0037-our-own-develop-core.md) (the develop core moves to v0.11.0)

## Context

The owner wants colour editing at a **moderate** level (2026-11-05): more than one-tap filters, far less than Lightroom, which is a professional editor. The models are Snapseed (Tune Image: brightness, contrast, saturation, ambiance, highlights, shadows, warmth; saved *Looks*), LUMIX Lab (a LUT library of 33-point `.cube` files with an opacity; a `#LUMIXPHOTOSTYLE` comment tells whether a LUT expects V-Log or a Photo Style) and Lightroom (profiles are LUTs with an Amount; presets are slider values, also with an Amount).

The plan before this ADR was narrower: `.cube` looks in v0.11.0 (FR-070), after the develop core of v0.10.0 (FR-072), then Lumix Photo Style sliders (FR-039); PRD §Out of scope ruled out curve editors. The owner chose, 2026-11-05:

1. colour first, the develop core after: v0.10.0 "Looks", v0.11.0 "Core";
2. four groups of global tools: **Tone** (brightness, contrast, highlights, shadows), **Colour** (warmth, tint, saturation, vibrance), **Style** (fade, shadow and highlight tint, black and white with a colour filter), **Effects** (grain, vignette);
3. three lego cards, as Snapseed 3 splits its tabs: **Looks** (built-in, the user's own and imported LUTs, with an amount), **Faves** (tone and colour, a fixed set) and **Tools** (style and effects); Snapseed's *Selective* points come after v0.10.0;
4. built-in looks of the project's own making only, under its own names.

Research (2026-11-05): tetrahedral interpolation is more accurate than trilinear at the same cost and keeps the grey axis; creative LUTs expect display-referred, gamma-encoded input, not linear light; HaldCLUT PNG is the other common format; no permissive Rust crate does parsing and tetrahedral application well enough to depend on.

## Options considered

1. **Each slider its own pass:** simple, but the cost grows with every slider, and the preview budget (≤ 5 ms for the look) is lost after a few.
2. **Everything per pixel baked into one 3D table**, spatial effects in the same pass after it: the cost per pixel is fixed whatever is set.
3. **GPU shaders in the webview:** fast on screen, but the export renders in Rust, so preview and print would be two implementations to keep equal.

## Decision

Option 2, in `crates/core/src/looks.rs`:

- **One table.** The imported LUT mixed by its strength, then warmth and tint (channel gains in linear light), the tone curve (brightness, contrast, highlights, shadows), saturation and vibrance (in OKLab, vibrance sparing skin and colours already strong), fade, shadow and highlight tint (by lightness, with a balance), and black and white through a colour filter, are baked into a **33³ u16 table** and applied with **tetrahedral interpolation** and rayon. The table is built in under a millisecond and cached by the settings' hash.
- **Spatial effects** (grain, vignette) run in the same pass, after the table. Grain is sized in **millimetres on the print**, turned into pixels by the print DPI, seeded by the photo, so a 10×15 and a 20×30 show the same grain on paper and the preview matches the export.
- **Encoded values.** The LUT and the sliders work on the photo's encoded values, as LUMIX Lab and Lightroom profiles do; the linear working space of ADR-0037 is not needed, so JPEG and HEIC do not wait for the develop core.
- **Order.** Crop and resize, then the look, then the date stamp, the info frame, the border and the fill, which are never tinted. With nothing set the stage is skipped, bit for bit. Thumbnails stay plain.
- **A look** is `{ name, adjust, lut?, lutStrength, amount }`; the amount (0–100 %) scales every slider and the LUT's strength, as Lightroom's preset Amount. Built-in looks are slider recipes, not LUT files, so they carry no third-party licence: *C200 Natural* (the owner's GX9 recipe, FR-039) and film-inspired looks under descriptive names (FR-047).
- **Per photo**, as the date stamp (FR-044): `render.look` for every photo, an override per photo, *Apply to all*; a collage sheet uses each photo's own.
- **Imported LUTs** stay as ADR-0036 decided (Rust-only parser, caps, fuzzing, `looks/<hash>`), plus HaldCLUT PNG and a warning for a `#LUMIXPHOTOSTYLE VLOG` LUT. `.vlt` and stacked LUTs wait.
- **Still out of scope:** masks and local adjustments (Snapseed's *Selective* points are a later FR of their own, measured on their own, as they cannot bake into the table), a full HSL matrix, a free curve editor, retouching.

## Consequences

- The budgets of ADR-0036 hold for any combination: ≤ 5 ms on the 1600 px preview, ≤ 60 ms on a 24 MP export, measured by `measure_look` before any optimisation.
- Settings v14 (`render.look`, the user's looks) and project v11 (each photo's look, the LUTs it uses embedded, ADR-0016) ([DATABASE.md](../DATABASE.md)).
- FR-039 is merged into FR-079; FR-047's film looks become built-in recipes on this engine; FR-070 keeps the LUT import and moves to v0.10.0.
- A new lego block, `SliderField`, joins STYLE_RULE §4 and the panel measures.
- ROADMAP and STATUS swap v0.10.0 and v0.11.0; the develop core keeps its plan and budgets.
