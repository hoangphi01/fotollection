// SPDX-License-Identifier: GPL-3.0-only
import type { ReactNode } from 'react'
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'

type SettingsFrameProps = {
  open: boolean
  onClose: () => void
  /** The window's name, at the top of the list. */
  title: string
  /** The list of sections (FR-086). */
  nav: ReactNode
  /** Under the list: the version. */
  navFooter?: ReactNode
  /** The open section's name and one line on it. */
  heading: string
  description?: string
  /** The open section's cards. */
  children: ReactNode
  /** Under the cards: one line for the whole window. */
  footer?: ReactNode
  /** Under the cards, in place of the line: the window's own buttons (the export dialog's
   * *Don't ask again*, *Cancel* and *Export*, FR-093). */
  actions?: ReactNode
}

/**
 * The Settings window (FR-086): the preview's dialog, its size, header and footer measures,
 * with the columns as on an iPad: a narrow list of sections on the left, the open section's
 * cards on the right, spaced 12 px as in every panel (STYLE_RULE §4, the lego rule).
 */
export function SettingsFrame({ open, onClose, title, nav, navFooter, heading, description, children, footer, actions }: SettingsFrameProps) {
  return (
    <Dialog open={open} onOpenChange={(next) => !next && onClose()}>
      <DialogContent className="flex h-[90vh] w-[95vw] max-w-none gap-0 overflow-hidden p-0 sm:max-w-[min(1100px,95vw)]">
        <aside className="flex w-64 shrink-0 flex-col border-r bg-muted/40">
          <div className="px-4 pt-4 pb-3">
            <p className="truncate text-base font-semibold">{title}</p>
          </div>
          <Separator />
          <ScrollArea className="min-h-0 flex-1">
            <nav className="flex flex-col gap-1 p-3">{nav}</nav>
          </ScrollArea>
          {navFooter && (
            <>
              <Separator />
              <div className="px-4 py-3 text-xs text-muted-foreground">{navFooter}</div>
            </>
          )}
        </aside>

        <div data-slot="settings-body" className="flex min-w-0 flex-1 flex-col bg-background">
          <div className="space-y-0.5 px-4 pt-4 pb-3 pr-12">
            <DialogTitle className="truncate text-base">{heading}</DialogTitle>
            <DialogDescription className="truncate text-xs">{description}</DialogDescription>
          </div>
          <Separator />
          <ScrollArea className="min-h-0 flex-1 [&>[data-slot=scroll-area-viewport]>div]:block!">
            <div className="mx-auto max-w-xl space-y-3 p-3">{children}</div>
          </ScrollArea>
          {actions ? (
            <>
              <Separator />
              <div className="flex items-center gap-2 px-4 py-3">{actions}</div>
            </>
          ) : footer ? (
            <>
              <Separator />
              <p className="px-4 py-3 text-xs text-muted-foreground">{footer}</p>
            </>
          ) : null}
        </div>
      </DialogContent>
    </Dialog>
  )
}
