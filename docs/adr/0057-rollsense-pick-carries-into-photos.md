# ADR-0057: RollSense's pick carries into Photos

- **Status:** Accepted
- **Date:** 2026-11-18
- **Related:** FR-141, FR-050, FR-115, ADR-0055

## Context

RollSense (FR-141) sorts a roll into groups that tick to narrow the grid. Until now a pick lived on RollSense's tab only: its actions were *Set aside* and *Delete*, and back in Photos every photo came back, so that "colour and print never stay filtered by a sort left behind" (owner, 2026-11-18). The owner then named the main use: "a 3-day trip of 1,000 photos, the food photos edited first: RollSense, tick Food; but after picking it only lets you delete."

Three UX reviews, done apart and then cross-checked (2026-11-18), agreed: the risk is a hidden filter, not a filter. They also found that the preview opened from RollSense already stepped through the pick while its export and colour took the whole folder.

## Options considered

1. **Keep the pick on its tab**, add only *Add to the print basket*: no path to colour a subset.
2. **Edit inside RollSense**, its column switching between the groups and the colour panel: two masters for one column, and a second select mode.
3. **The pick carries into Photos, always named**: Pro's filter holds the pick, and a chip in the toolbar shows it.

## Decision

Option 3, the owner's choice on 2026-11-18, export following the pick:

- RollSense's footer: **Edit n photos in Photos** (first), **Add to the print basket**, **Set aside**, **Delete** (red, last).
- Pro's filter is `{ ...filter, sort: picks }` on every Pro view, so the grid, the preview, colour and export take the same photos. The core already applies `sort` in its one filter (`library.rs`).
- In Photos, a chip beside RollSense's button names the pick (its first choice, `+n` for the others) with RollSense's violet key. Pressing it goes back to RollSense with its ticks; its **✕** clears the pick. The count reads *n of total*, and the export notes that only the filtered photos go.
- The pick is session state: never saved in the project. A tab with no folder in common clears it. Simple and Collage ignore it, as they ignore Pro's stars and flags.
- Picking single photos stays in Photos' select mode, on the narrowed grid: no second select mode.

## Consequences

- A user can forget the chip and export only the pick. The chip, the count and the export's note show it. The reviewers' test: 4 of 5 users clear it unaided when asked to export the whole trip, to be run after the release.
- Delete stays in RollSense only.
