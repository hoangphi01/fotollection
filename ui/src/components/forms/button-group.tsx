// SPDX-License-Identifier: GPL-3.0-only
import type { ReactNode } from 'react'

type ButtonGroupProps = {
  /** Icon buttons (`variant="ghost"`, `size="icon-sm"`), in order; a wrapper for a hint is fine. */
  children: ReactNode
  ariaLabel: string
}

/**
 * A row of related icon buttons joined into one raised bar, as Bootstrap's button group (owner,
 * 2026-11-06): one lip and lift for the bar, a hairline between the buttons, the ends rounded.
 * The buttons inside stay flat; a pressed one is pressed into the bar.
 */
export function ButtonGroup({ children, ariaLabel }: ButtonGroupProps) {
  return (
    <div
      role="group"
      aria-label={ariaLabel}
      data-slot="button-group"
      className="inline-flex shrink-0 divide-x divide-border overflow-hidden rounded-lg border border-border bg-card [--face:var(--card)] raised dark:border-input dark:bg-secondary dark:[--face:var(--secondary)] [&_[data-slot=button]]:rounded-none [&_[data-slot=button]]:focus-visible:ring-inset"
    >
      {children}
    </div>
  )
}
