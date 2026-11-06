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
}

/**
 * A short note inside a panel card: muted text on a tinted block, with an icon in the theme
 * colour (STYLE_RULE §4, shared pieces). It owns no margin; the card spaces it.
 */
export function PanelNote({ icon, children, danger = false }: PanelNoteProps) {
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
