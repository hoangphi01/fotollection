// SPDX-License-Identifier: GPL-3.0-only

/**
 * RAW at a glance, in the theme colour so it shows while scrolling, as on the preview's
 * This photo card (owner, 2026-11-05): on the gallery's tiles and the filmstrip's cells.
 */
export function RawBadge() {
  return (
    <span className="inline-flex h-4 items-center rounded bg-primary px-1 pt-px text-[10px] leading-none font-semibold tracking-wide text-primary-foreground">
      RAW
    </span>
  )
}
