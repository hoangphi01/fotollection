// SPDX-License-Identifier: GPL-3.0-only
import type { ReactNode } from 'react'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'

type SidePanelProps = {
  title: ReactNode
  /** The section cards, scrolling. */
  children: ReactNode
  /** Kept in view under the cards: the export controls. */
  footer: ReactNode
  /** Tabs under the title, as a preview's inspector has (Pro mode's gallery: Colour and Print,
   * FR-114); the cards are the open tab's. */
  tabs?: ReactNode
}

/**
 * The right-hand panel of every mode (Pro, Simple, Collage): the same width, header and
 * scrolling cards over a fixed footer, so the three never drift apart (STYLE_RULE §4).
 */
export function SidePanel({ title, children, footer, tabs }: SidePanelProps) {
  return (
    <aside className="flex w-panel shrink-0 flex-col border-l bg-sidebar text-sidebar-foreground">
      <div className="flex h-header shrink-0 items-center px-4">
        <span className="text-sm font-semibold">{title}</span>
      </div>
      <Separator />
      {tabs && (
        <>
          {tabs}
          <Separator />
        </>
      )}
      {/* An overlay scrollbar, as in the folder list, so the cards keep the panel's width. */}
      <ScrollArea className="min-h-0 flex-1 [&>[data-slot=scroll-area-viewport]>div]:block!">
        <div className="space-y-3 p-3">{children}</div>
      </ScrollArea>
      <Separator />
      <div className="space-y-3 p-3">{footer}</div>
    </aside>
  )
}
