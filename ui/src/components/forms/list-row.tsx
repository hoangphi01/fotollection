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
        'w-full justify-start font-normal [&_svg]:text-primary',
        muted && 'text-muted-foreground',
        chosen && CHOSEN_ROW,
      )}
      onClick={onClick}
    >
      {icon ?? <span className="size-3.5 shrink-0" aria-hidden />}
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
