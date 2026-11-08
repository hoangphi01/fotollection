// SPDX-License-Identifier: GPL-3.0-only
import type { ReactNode } from 'react'
import { cn } from '@/lib/utils'
import { JOINED_BAR } from './joined-bar'

type ButtonGroupProps = {
  /** Icon buttons (`variant="ghost"`, `size="icon-sm"`) or `ToolbarButton`s, in order; a wrapper
   * for a hint is fine. */
  children: ReactNode
  ariaLabel: string
}

/**
 * A row of related buttons joined into one raised bar, as Bootstrap's button group (owner,
 * 2026-11-06): one lip and lift for the bar, a hairline between the buttons, the ends rounded.
 * The buttons inside are flat, raised ones (a toolbar's outline, a filled main action) made
 * flat here; a pressed one is pressed into the bar. Toolbars join related buttons with it
 * (owner, 2026-11-08: undo and redo, copy and paste, the zoom…); a red button stays outside.
 * The frame is a ring, so the bar is 32 px as every control.
 */
export function ButtonGroup({ children, ariaLabel }: ButtonGroupProps) {
  return (
    <div
      role="group"
      aria-label={ariaLabel}
      data-slot="button-group"
className={cn(
        JOINED_BAR,
        'h-8 shrink-0 [&_[data-slot=button]]:rounded-none! [&_[data-slot=button]]:border-0 [&_[data-slot=button]]:focus-visible:ring-inset [&_[data-slot=button][data-variant=outline]:not(:active):not(:hover)]:bg-transparent [&_[data-slot=button]:not([aria-pressed=true]):not(:active)]:shadow-none',
      )}
    >
      {children}
    </div>
  )
}
