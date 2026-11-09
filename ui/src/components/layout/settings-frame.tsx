// SPDX-License-Identifier: GPL-3.0-only
import { PartBoundary } from './part-boundary'
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
  /** The body fills the window's right side with no scroll and no column width: one big
   * picture (the welcome screen, FR-109; owner, 2026-11-07: "the picture bigger"). */
  full?: boolean
  /** False: no ✕, and neither Esc nor a click outside closes it; only its own buttons do
   * (the first launch's welcome, read through once; owner, 2026-11-07). */
  closable?: boolean
  /** A new key starts the cards' scroll at the top (the manual's next chapter, FR-110). */
  scrollKey?: string
}

/**
 * The Settings window (FR-086): the preview's dialog, its size, header and footer measures,
 * with the columns as on an iPad: a narrow list of sections on the left, the open section's
 * cards on the right, spaced 12 px as in every panel (STYLE_RULE §4, the lego rule).
 */
export function SettingsFrame({
  open,
  onClose,
  title,
  nav,
  navFooter,
  heading,
  description,
  children,
  footer,
  actions,
  full = false,
  closable = true,
  scrollKey,
}: SettingsFrameProps) {
  return (
    <Dialog open={open} onOpenChange={(next) => !next && closable && onClose()}>
      <DialogContent
        showCloseButton={closable}
        onEscapeKeyDown={(e) => !closable && e.preventDefault()}
        onPointerDownOutside={(e) => !closable && e.preventDefault()}
        className="flex h-[90vh] w-[95vw] max-w-none gap-0 overflow-hidden p-0 sm:max-w-[min(1100px,95vw)]"
      >
        <aside className="flex w-64 shrink-0 flex-col border-r bg-muted/40">
          {/* Both columns' heads and feet share one height, so their lines meet as one line
              across the window (owner, 2026-11-06), as the gallery's do. */}
          <div className="flex h-header shrink-0 items-center px-4">
            <p className="truncate text-base font-semibold">{title}</p>
          </div>
          <Separator />
          {/* Radix's viewport lays its content out as a table, which grows to its widest row:
              block keeps long rows (a manual's chapter) inside the column, truncated. */}
          <ScrollArea className="min-h-0 flex-1 [&>[data-slot=scroll-area-viewport]>div]:block!">
            <nav className="flex flex-col gap-1 p-3">{nav}</nav>
          </ScrollArea>
          {navFooter && (
            <>
              <Separator />
              <div className="flex h-14 shrink-0 items-center px-4 text-xs text-muted-foreground">{navFooter}</div>
            </>
          )}
        </aside>

        <div data-slot="settings-body" className="flex min-w-0 flex-1 flex-col bg-background">
          <div className="flex h-header shrink-0 flex-col justify-center gap-0.5 px-4 pr-12">
            <DialogTitle className="truncate text-base">{heading}</DialogTitle>
            <DialogDescription className="truncate text-xs" data-hint={description}>{description}</DialogDescription>
          </div>
          <Separator />
          {full ? (
            <div className="flex min-h-0 flex-1 flex-col gap-3 p-3">
              <PartBoundary part="settings">{children}</PartBoundary>
            </div>
          ) : (
            <ScrollArea key={scrollKey} className="min-h-0 flex-1 [&>[data-slot=scroll-area-viewport]>div]:block!">
              <div className="mx-auto max-w-xl space-y-3 p-3">
                <PartBoundary part="settings">{children}</PartBoundary>
              </div>
            </ScrollArea>
          )}
          {actions ? (
            <>
              <Separator />
              <div className="flex h-14 shrink-0 items-center gap-2 px-3">{actions}</div>
            </>
          ) : footer ? (
            <>
              <Separator />
              <p className="flex h-14 shrink-0 items-center px-4 text-xs text-muted-foreground">{footer}</p>
            </>
          ) : null}
        </div>
      </DialogContent>
    </Dialog>
  )
}
