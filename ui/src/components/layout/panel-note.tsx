// SPDX-License-Identifier: GPL-3.0-only
import type { ReactNode } from 'react'

type PanelNoteProps = {
  /** A Lucide icon, sized and coloured here. */
  icon?: ReactNode
  children: ReactNode
}

/**
 * A short note inside a panel card: muted text on a tinted block, with an icon in the theme
 * colour (STYLE_RULE §4, shared pieces). It owns no margin; the card spaces it.
 */
export function PanelNote({ icon, children }: PanelNoteProps) {
  return (
    <p className="flex gap-1.5 rounded-md bg-muted/60 px-2.5 py-2 text-xs text-muted-foreground [&>svg]:mt-0.5 [&>svg]:size-3.5 [&>svg]:shrink-0 [&>svg]:text-primary">
      {icon}
      <span>{children}</span>
    </p>
  )
}
