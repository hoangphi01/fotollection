// SPDX-License-Identifier: GPL-3.0-only
import type { ComponentProps, ReactNode } from 'react'
import { PopoverContent } from '@/components/ui/popover'
import { ListRows } from './list-row'

/**
 * A menu (a ⋯, a right click, a toolbar's arrow; owner, 2026-11-08: "too long, too narrow, the
 * buttons too low"): as wide as its longest row, at least 192 px, at most 320; its rows are
 * `ListRow`s and a red `RemoveButton wide`, every one 32 px as every control (STYLE_RULE §4).
 * One block for every menu, so none is sized by hand.
 */
/** A hairline between a menu's groups of actions, as Lightroom's (FR-117). */
export function MenuSeparator() {
  return <div role="separator" className="my-1 h-px bg-border" />
}

export function MenuContent({ align = 'start', children }: { align?: ComponentProps<typeof PopoverContent>['align']; children: ReactNode }) {
  return (
    <PopoverContent align={align} className="w-max max-w-80 min-w-48 p-1">
      <ListRows>{children}</ListRows>
    </PopoverContent>
  )
}
