// SPDX-License-Identifier: GPL-3.0-only
import { ChevronDown } from 'lucide-react'
import type { ReactNode } from 'react'
import { Collapse } from '@/components/layout/collapse'
import { cn } from '@/lib/utils'
import { ListRow, ListRows } from './list-row'

type GroupRowsProps = {
  title: string
  count: number
  open: boolean
  onToggle(): void
  children: ReactNode
}

/**
 * A group of a list that folds by its name (STYLE_RULE §4), as Lightroom's preset groups: a
 * row with a chevron and how many it holds, its rows under it, 4 px apart as every list.
 */
export function GroupRows({ title, count, open, onToggle, children }: GroupRowsProps) {
  return (
    <div>
      <ListRow
        icon={<ChevronDown className={cn('motion-move', !open && '-rotate-90')} aria-hidden />}
        trailing={<span className="text-xs text-muted-foreground tabular-nums">{count}</span>}
        onClick={onToggle}
      >
        <span className="font-medium">{title}</span>
      </ListRow>
      <Collapse open={open}>
        <div className="pt-1 pl-4">
          <ListRows>{children}</ListRows>
        </div>
      </Collapse>
    </div>
  )
}
