// SPDX-License-Identifier: GPL-3.0-only
import type { ReactNode } from 'react'
import { Separator } from '@/components/ui/separator'

type SidePanelProps = {
  title: ReactNode
  /** The section cards, scrolling. */
  children: ReactNode
  /** Kept in view under the cards: the export controls. */
  footer: ReactNode
}

/**
 * The right-hand panel of every mode (Pro, Simple, Collage): the same width, header and
 * scrolling cards over a fixed footer, so the three never drift apart (STYLE_RULE §4).
 */
export function SidePanel({ title, children, footer }: SidePanelProps) {
  return (
    <aside className="flex w-72 shrink-0 flex-col border-l bg-sidebar text-sidebar-foreground">
      <div className="flex h-12 items-center px-4">
        <span className="text-sm font-semibold">{title}</span>
      </div>
      <Separator />
      <div className="flex-1 space-y-3 overflow-y-auto p-3">{children}</div>
      <Separator />
      <div className="space-y-3 p-4">{footer}</div>
    </aside>
  )
}
