# ADR-0032: Collage sheets are composed from two cell plans

- **Status:** Accepted
- **Date:** 2026-10-31
- **Related:** FR-035, FR-048, FR-067, NFR-021, ADR-0013

## Context

v0.9.0 adds a Collage mode ([FR-048](../PRD.md#fr-048--simple-collage-and-professional-modes)): two photos on one print, like a half-frame film camera ([FR-035](../PRD.md#fr-035--multi-photo-layouts)). Each photo keeps its own crop, date stamp and 3 mm border ([FR-067](../PRD.md#fr-067--3-mm-border-on-every-print)), inside a cell that is half the sheet, with a white margin and an optional cut line.

The single-print pipeline is `plan::plan` → `render::render` (crop and resize, stamp, `compose_frame`), and its output pixels follow the source photo's resolution, not a fixed DPI. `compose_frame` already fills a canvas and blits the photo onto it. Preview is the `fotothumb://` protocol; export runs one file per photo in a rayon pool.

The user reorders a sheet's photos by dragging. In a Tauri webview the native file drop (used to drop folders into the app, FR-001) intercepts HTML5 drag and drop on Windows.

## Options considered

1. **A layout engine for sheets** with its own crop, stamp and border code. Free for 4-up and strips later, but a second copy of every rule the single print already has, which would drift.
2. **Compose a sheet from two ordinary plans.** Each photo is planned as if its photo area were the print size, rendered as usual, then blitted into its cell on a sheet canvas. Crop editor, stamp, border and advice reuse the single-print code unchanged.
3. **Do the composition in the UI** (two previews side by side). Fast to build, but export would still need Rust, and preview and export would differ.

## Decision

Option 2, in a new `crates/core/src/sheet.rs`:

- **Geometry in millimetres:** sheet → two cells (split across the long edge) → photo area (the margin taken off, the suggested or typed size, clamped and centred) → the photo inside the 3 mm border. Pure functions, tested in mm.
- **Pixels:** the sheet's density is the **lower** of the two photos' own densities over their areas, so neither photo is enlarged; capped at 600 DPI, a cap to confirm by benchmark (NFR-021). An empty cell stays white.
- **Preview** is a new route `fotothumb://localhost/sheet/{a}/{b}`, from the same 2048 px preview sources; **export** pairs the records before the rayon pool, one file per sheet.
- **Drag and drop** in the sheet grid uses pointer events, with no new dependency, so it never meets the webview's file drop.

## Consequences

- Single prints are untouched; a sheet costs about two single renders plus one blit and one encode, measured against NFR-021.
- 4-up and strips later are more cells in `sheet.rs`, not a new engine.
- The info frame is left out of cells for now (its bar does not fit a 7.5 cm cell).
- Docs: PRD FR-035, FR-048, FR-067, NFR-021; ARCHITECTURE (route, module); DATABASE (settings v10 `render.collage`, project v9 `collage`).
