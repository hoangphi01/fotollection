// SPDX-License-Identifier: GPL-3.0-only
import type { ReactNode } from 'react'
import { cn } from '@/lib/utils'

type PanelNoteProps = {
  /** A Lucide icon, sized and coloured here. */
  icon?: ReactNode
  children: ReactNode
  /** A warning that something will be lost (Replace existing files): red text and icon. */
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
        danger ? 'bg-destructive/10 text-destructive [&>svg]:text-destructive' : 'bg-muted/60 text-muted-foreground [&>svg]:text-primary',
      )}
    >
      {icon}
      <span>{children}</span>
    </p>
  )
}
