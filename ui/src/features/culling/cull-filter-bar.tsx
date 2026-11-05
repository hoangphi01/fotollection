// SPDX-License-Identifier: GPL-3.0-only
import { ListFilter, Star, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Field, Stack } from '@/components/forms/field'
import { SegmentedToggle } from '@/components/forms/segmented-toggle'
import { Button } from '@/components/ui/button'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import type { CullFilter, FlagFilter, KindFilter } from '@/ipc/bindings'
import { cn } from '@/lib/utils'

const FLAGS: FlagFilter[] = ['all', 'picks', 'unflagged', 'rejects', 'unusual']
const KINDS: KindFilter[] = ['all', 'raw', 'notRaw']

type CullFilterBarProps = {
  filter: CullFilter
  onChange: (filter: CullFilter) => void
}

/** How many of the filters narrow the set. */
function activeCount(filter: CullFilter): number {
  return [filter.flag !== 'all', filter.minRating > 0, (filter.kind ?? 'all') !== 'all'].filter(Boolean).length
}

/**
 * Library filter (FR-022): one button, so the toolbar stays on one line at the narrowest
 * window (STYLE_RULE §4); it opens the flag, the stars and RAW or not. Outlined in the accent
 * colour with how many are on, so a filter is never forgotten. Export writes exactly the
 * photos it keeps.
 */
export function CullFilterBar({ filter, onChange }: CullFilterBarProps) {
  const { t } = useTranslation()
  const active = activeCount(filter)
  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          size="sm"
          className={cn(active > 0 && 'border-primary text-primary')}
          data-hint={t('culling.filtersHint')}
        >
          <ListFilter data-icon="inline-start" />
          {active > 0 ? t('culling.filtersOn', { count: active }) : t('culling.filters')}
        </Button>
      </PopoverTrigger>
      <PopoverContent align="end" className="w-64">
        <Stack>
          <Field label={t('culling.filterByFlag')}>
            <Select value={filter.flag} onValueChange={(flag) => onChange({ ...filter, flag: flag as FlagFilter })}>
              <SelectTrigger className="w-full" aria-label={t('culling.filterByFlag')}>
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
          </Field>
          <Field label={t('culling.minRating')}>
            <Select value={String(filter.minRating)} onValueChange={(v) => onChange({ ...filter, minRating: Number(v) })}>
              <SelectTrigger className="w-full" aria-label={t('culling.minRating')}>
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
          </Field>
          <Field label={t('culling.filterByKind')}>
            <SegmentedToggle<KindFilter>
              value={filter.kind ?? 'all'}
              options={KINDS.map((value) => ({ value, label: t(`culling.kind.${value}`) }))}
              onChange={(kind) => onChange({ ...filter, kind })}
              ariaLabel={t('culling.filterByKind')}
            />
          </Field>
          {active > 0 && (
            <Button
              variant="ghost"
              size="sm"
              className="w-full"
              onClick={() => onChange({ ...filter, flag: 'all', minRating: 0, kind: 'all' })}
            >
              <X data-icon="inline-start" /> {t('culling.clearFilters')}
            </Button>
          )}
        </Stack>
      </PopoverContent>
    </Popover>
  )
}
