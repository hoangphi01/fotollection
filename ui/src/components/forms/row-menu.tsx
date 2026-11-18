// SPDX-License-Identifier: GPL-3.0-only
import { MoreHorizontal } from 'lucide-react'
import { useState, type ReactNode } from 'react'
import { Button } from '@/components/ui/button'
import { Popover, PopoverAnchor, PopoverTrigger } from '@/components/ui/popover'
import { MenuContent } from './menu-content'

type Point = { x: number; y: number }

/** A point to open a menu from: where the right click was (fixed, so it follows no scroll). */
function PointAnchor({ at }: { at: Point }) {
  return (
    <PopoverAnchor asChild>
      <span aria-hidden className="pointer-events-none fixed size-0" style={{ left: at.x, top: at.y }} />
    </PopoverAnchor>
  )
}

type RowMenuProps = {
  /** The ⋯ button's name and hint. */
  label: string
  /** The menu's rows (`ListRow`s, `MenuSeparator`s, the red `RemoveButton wide` last), given
   * `close`; null when the row has no menu: then a right click opens nothing. */
  menu: ((close: () => void) => ReactNode) | null
  /** Told when the menu opens and closes (a hover preview pauses while it is open). */
  onOpenChange?: (open: boolean) => void
  children: ReactNode
}

/**
 * A row's actions (FR-117, STYLE_RULE §4): a raised ⋯ at the row's end, shown on hover or focus,
 * and the same menu on a right click, where the pointer is, or on Shift+F10 and the menu key
 * for the row in focus. One block, so every row's menu opens the same way.
 */
export function RowMenu({ label, menu, onOpenChange, children }: RowMenuProps) {
  const [open, setOpen] = useState(false)
  const [at, setAt] = useState<Point | null>(null)
  const change = (next: boolean, point: Point | null = null) => {
    setOpen(next)
    setAt(next ? point : null)
    onOpenChange?.(next)
  }
  return (
    <div
      className="group/row flex items-center gap-1"
      onContextMenu={(e) => {
        e.preventDefault()
        if (menu) change(true, { x: e.clientX, y: e.clientY })
      }}
      onKeyDown={(e) => {
        if (menu && (e.key === 'ContextMenu' || (e.shiftKey && e.key === 'F10'))) {
          e.preventDefault()
          change(true)
        }
      }}
    >
      <div className="min-w-0 flex-1">{children}</div>
      {menu && (
        <Popover open={open} onOpenChange={(next) => change(next)}>
          <PopoverTrigger asChild>
            <Button
              variant="outline"
              size="icon-sm"
              aria-label={label}
              data-hint={label}
              className="opacity-0 group-hover/row:opacity-100 focus-visible:opacity-100 aria-expanded:opacity-100"
            >
              <MoreHorizontal />
            </Button>
          </PopoverTrigger>
          {at && <PointAnchor at={at} />}
          <MenuContent align={at ? 'start' : 'end'}>{menu(() => change(false))}</MenuContent>
        </Popover>
      )}
    </div>
  )
}

/**
 * A menu opened at a point (FR-117): a right click on a strip's tile, which has no ⋯. `at` null
 * keeps it closed.
 */
export function PointMenu({ at, onClose, children }: { at: Point | null; onClose: () => void; children: ReactNode }) {
  return (
    <Popover open={at !== null} onOpenChange={(next) => !next && onClose()}>
      {at && <PointAnchor at={at} />}
      <MenuContent align="start">{children}</MenuContent>
    </Popover>
  )
}
