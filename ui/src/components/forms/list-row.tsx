// SPDX-License-Identifier: GPL-3.0-only
import type { MouseEventHandler, ReactNode } from 'react'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

/**
 * The look of a chosen row in any list, menu or dropdown (owner, 2026-11-06: one chosen style
 * everywhere): the accent wash, an accent outline inside and a deep lip (`row-chosen`). Rows
 * that are not a `ListRow` (the folder list's own buttons) take it from here, never by hand.
 */
export const CHOSEN_ROW = 'bg-accent font-medium text-accent-foreground row-chosen'

/**
 * One measure for every row of a list (owner, 2026-11-07: "the room between icon and words
 * is not the same"): 32 px, 8 px in from the side, the icon 16 px and 8 px before the words
 * in `text-sm`; the icon muted, in the accent on the chosen row. `ListRow` and the folder
 * list's own rows both take it from here.
 */
// Only the row's own icon (a direct child): never a checkbox's tick or a button inside it.
export const ROW_LAYOUT = 'gap-2 px-2 text-sm [&>svg]:size-4! [&>svg]:shrink-0'
export const ROW_ICON = (chosen: boolean) => (chosen ? '[&>svg]:text-primary' : '[&>svg]:text-muted-foreground')
/** The row under the pointer: the ghost button's wash, so every list reacts the same. */
export const ROW_HOVER = 'hover:bg-muted dark:hover:bg-muted/50'

type ListRowProps = {
  /** A Lucide icon before the words; rows without one keep its room, so words line up. */
  icon?: ReactNode
  children: ReactNode
  /** What follows the words: keys (`Kbd`), a count. */
  trailing?: ReactNode
  chosen?: boolean
  /** How `aria-current` names the chosen row: a page (a section), a step, or true. */
  current?: 'page' | 'step' | 'true'
  muted?: boolean
  onClick: MouseEventHandler<HTMLButtonElement>
}

/**
 * One row of a list, a menu or a section list: 32 px as every control, an icon, the words,
 * what follows them, the chosen one marked the same way everywhere (Settings' sections, the
 * export presets, the compare views, History).
 */
export function ListRow({ icon, children, trailing, chosen = false, current = 'true', muted = false, onClick }: ListRowProps) {
  return (
    <Button
      variant="ghost"
      size="sm"
      aria-current={chosen ? current : undefined}
      className={cn(
        'w-full justify-start font-normal',
        ROW_LAYOUT,
        ROW_ICON(chosen),
        muted && 'text-muted-foreground',
        chosen && CHOSEN_ROW,
      )}
      onClick={onClick}
    >
      {icon ?? <span className="size-4 shrink-0" aria-hidden />}
      <span className="min-w-0 flex-1 truncate text-left">{children}</span>
      {trailing}
    </Button>
  )
}

/**
 * The rows of one list or menu, 4 px apart (STYLE_RULE §4, one spacing for every list: the
 * folder list, Settings' sections, the export presets, History, the compare menu).
 */
export function ListRows({ children, as: Tag = 'div' }: { children: ReactNode; as?: 'div' | 'ul' | 'nav' }) {
  return <Tag className="flex flex-col gap-1">{children}</Tag>
}
