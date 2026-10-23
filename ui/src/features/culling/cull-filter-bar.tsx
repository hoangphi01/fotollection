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
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import type { CullFilter, FlagFilter } from '@/ipc/bindings'

const FLAGS: FlagFilter[] = ['all', 'picks', 'unflagged', 'rejects']

type CullFilterBarProps = {
  filter: CullFilter
  onChange: (filter: CullFilter) => void
}

/** Library filter (FR-022). Export writes exactly the photos it keeps. */
export function CullFilterBar({ filter, onChange }: CullFilterBarProps) {
  const { t } = useTranslation()
  return (
    <div className="flex items-center gap-2">
      <ToggleGroup
        type="single"
        variant="outline"
        size="sm"
        value={filter.flag}
        onValueChange={(flag) => flag && onChange({ ...filter, flag: flag as FlagFilter })}
        aria-label={t('culling.filterByFlag')}
      >
        {FLAGS.map((value) => (
          <ToggleGroupItem key={value} value={value} className="px-2.5 text-xs">
            {t(`culling.flag.${value}`)}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>
      <Select
        value={String(filter.minRating)}
        onValueChange={(v) => onChange({ ...filter, minRating: Number(v) })}
      >
        <SelectTrigger size="sm" className="w-28 text-xs" aria-label={t('culling.minRating')}>
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
