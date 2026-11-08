# ADR-0036: Colour looks: `.cube` LUTs first, then Photo Style on the same engine

- **Status:** Accepted (planned for v0.10.0, before the develop core, since [ADR-0038](0038-moderate-colour-editor-on-one-lut-engine.md))
- **Amended by:** [ADR-0038](0038-moderate-colour-editor-on-one-lut-engine.md) (2026-11-05): looks apply on the photo's encoded values, and the sliders of FR-079 bake into the same table
- **Date:** 2026-11-02
- **Related:** FR-070, FR-039, FR-047, FR-040, FR-044, NFR-002, NFR-003, ADR-0011, ADR-0016
- **Amends:** [ADR-0011](0011-photo-style-looks.md) (its "no `.cube` import")

## Context

The owner edits with **LUMIX Lab**, where a look is a LUT: a file that maps every colour to another. They want the same in Fotollection: pick a look, set how strong it is, print. [ADR-0011](0011-photo-style-looks.md) planned colour looks as Lumix-style Photo Style sliders baked into a 3D LUT, and set aside importing `.cube` files: opaque to users, and a file from outside is an input surface. The owner's own LUTs, and the many shared online, are what they actually use.

## Options considered

1. **Photo Style sliders only** (ADR-0011 as written): familiar to Lumix users, but the owner's existing LUTs cannot be used.
2. **`.cube` import only:** the LUMIX Lab workflow, but nothing to adjust inside the app.
3. **`.cube` import first, then the Photo Style sliders on the same engine:** the workflow the owner has today ships first; the sliders later reuse the engine.

## Decision

Option 3: FR-070 in v0.11.0 (on the linear working space of ADR-0037; since ADR-0038, v0.10.0 on encoded values), then FR-039, then the film looks of FR-047, all on one engine.

- **One engine:** every look, imported or built in, is resampled once to a **33³ table** (u16), applied with tetrahedral interpolation and rayon, and blended with the original by a strength of 0–100 %. Sliders (FR-039) later bake into the same table.
- **Order:** crop, then the look, then the date stamp, frame and border, so those are never tinted. Thumbnails stayed plain until [ADR-0047](0047-thumbnails-show-the-look.md) (2026-11-08), so a look never invalidated the cache.
- **The input surface:** a `.cube` file is text from outside. It is read only in Rust, from a file the user chose in a Rust-side dialog, with a size cap of 8 MB, caps on lines and points (2–65 a side, 1D or 3D), `DOMAIN_MIN`/`DOMAIN_MAX` checked, and NaN or out-of-range values rejected. A fuzz target covers the parser. The file is copied by content hash into the app's `looks/` folder.
- **Licences:** Fotollection ships only looks made by its own code, under its own names (no trademarked film or camera names, as FR-047). Users' LUTs stay theirs: they never leave the machine, except inside the user's own project, which embeds the LUTs it uses so it renders the same elsewhere ([ADR-0016](0016-self-contained-project-bundle.md)).
- **Panasonic's `.vlt`** (LUMIX Real Time LUT) is a candidate, added only after its format is checked.

## Consequences

- The cost per pixel is fixed whatever the look: budgets of ≤ 5 ms on the 1600 px preview and ≤ 60 ms on a 24 MP export, measured by `measure_look` when it is built ([OPTIMIZATION.md §3](../OPTIMIZATION.md#3-budgets)).
- A LUT works on the photo's encoded values, whatever its colour space, so looks are "inspired by" their source, as in ADR-0011.
- Settings gain `render.look` and photos a `look` override; projects embed `looks/<hash>.cube` ([DATABASE.md](../DATABASE.md)).
- Golden images cover each built-in look, an identity LUT must leave a photo unchanged, and the parser is fuzzed ([TESTING.md](../TESTING.md)).
