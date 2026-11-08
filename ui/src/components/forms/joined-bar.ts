// SPDX-License-Identifier: GPL-3.0-only

/**
 * One raised bar holding joined parts (owner, 2026-11-08): `ButtonGroup`'s buttons and
 * `SegmentedToggle`'s choices. Its frame is a ring, so it adds no size; the hairline before each
 * part after the first is drawn as a `::before`, never as a border, so it takes no width (a 1 px
 * border put every segment's words 0.5 px off centre, UI-LAYOUT-013) and a rule that removes a
 * button's border never removes it (UI-LAYOUT-008, "39 % | +").
 */
export const JOINED_BAR =
  "inline-flex overflow-hidden rounded-lg bg-card ring-1 ring-border [--face:var(--card)] raised dark:bg-secondary dark:ring-input dark:[--face:var(--secondary)] [&>*+*]:relative [&>*+*]:before:pointer-events-none [&>*+*]:before:absolute [&>*+*]:before:inset-y-0 [&>*+*]:before:left-0 [&>*+*]:before:z-10 [&>*+*]:before:w-px [&>*+*]:before:bg-border [&>*+*]:before:content-[''] dark:[&>*+*]:before:bg-input"
