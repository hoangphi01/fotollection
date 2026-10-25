// SPDX-License-Identifier: GPL-3.0-only
import { Star } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import type { CullFilter, FlagFilter } from '@/ipc/bindings'
import { cn } from '@/lib/utils'

const FLAGS: FlagFilter[] = ['all', 'picks', 'unflagged', 'rejects']

type CullFilterBarProps = {
  filter: CullFilter
  onChange: (filter: CullFilter) => void
}

/** A filter that narrows the set is outlined in the accent colour, so it is never forgotten. */
const ACTIVE = 'border-primary text-primary'

/**
 * Library filter (FR-022) as two dropdowns, so the toolbar stays on one line. Export writes
 * exactly the photos it keeps.
 */
export function CullFilterBar({ filter, onChange }: CullFilterBarProps) {
  const { t } = useTranslation()
  return (
    <div className="flex items-center gap-2">
      <Select value={filter.flag} onValueChange={(flag) => onChange({ ...filter, flag: flag as FlagFilter })}>
        <SelectTrigger
          size="sm"
          className={cn('w-32 text-xs', filter.flag !== 'all' && ACTIVE)}
          aria-label={t('culling.filterByFlag')}
        >
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {FLAGS.map((value) => (
            <SelectItem key={value} value={value}>
              {t(`culling.flag.${value}`)}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <Select
        value={String(filter.minRating)}
        onValueChange={(v) => onChange({ ...filter, minRating: Number(v) })}
      >
        <SelectTrigger
          size="sm"
          className={cn('w-28 text-xs', filter.minRating > 0 && ACTIVE)}
          aria-label={t('culling.minRating')}
        >
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="0">{t('culling.anyRating')}</SelectItem>
          {[1, 2, 3, 4, 5].map((n) => (
            <SelectItem key={n} value={String(n)}>
              <Star className="size-3 fill-current" /> {t('culling.atLeast', { count: n })}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  )
}
