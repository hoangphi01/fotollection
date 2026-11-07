// SPDX-License-Identifier: GPL-3.0-only
import type { ReactNode } from 'react'
import { cn } from '@/lib/utils'

type PanelNoteProps = {
  /** A Lucide icon, sized and coloured here. */
  icon?: ReactNode
  children: ReactNode
  /** A warning that something will be lost (Replace existing files, a version no longer
   * safe): solid deep red with white text, as the red badges. */
  danger?: boolean
  /** What acts on what the note says, under the words: one button in the theme's colour
   * (owner, 2026-11-07: "the Show now button inside the note"), or that and one outline
   * button beside it (owner, 2026-11-07: a postcard's *Use for them* and *Use for all*). */
  action?: ReactNode
}

/**
 * A short note inside a panel card: muted text on a tinted block, with an icon in the theme
 * colour (STYLE_RULE §4, shared pieces). It owns no margin; the card spaces it.
 */
export function PanelNote({ icon, children, danger = false, action }: PanelNoteProps) {
  if (action) {
    return (
      <div className="flex flex-col items-start gap-2 rounded-md bg-muted/60 px-2.5 py-2 text-xs text-muted-foreground">
        <p className="flex gap-1.5 [&>svg]:mt-0.5 [&>svg]:size-3.5 [&>svg]:shrink-0 [&>svg]:text-primary">
          {icon}
          <span>{children}</span>
        </p>
        <div className="flex flex-wrap gap-2">{action}</div>
      </div>
    )
  }
  return (
    <p
      className={cn(
        'flex gap-1.5 rounded-md px-2.5 py-2 text-xs [&>svg]:mt-0.5 [&>svg]:size-3.5 [&>svg]:shrink-0',
        // The red badges' look (owner, 2026-11-06: "no faint text"): deep red, white ink, its lip.
        danger
          ? 'bg-danger-solid font-medium text-white [--lip-color:var(--danger-lip)] chip-raised [&>svg]:text-white'
          : 'bg-muted/60 text-muted-foreground [&>svg]:text-primary',
      )}
    >
      {icon}
      <span>{children}</span>
    </p>
  )
}
