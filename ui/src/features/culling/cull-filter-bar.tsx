// SPDX-License-Identifier: GPL-3.0-only
import { Star } from 'lucide-react'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import type { CullFilter, FlagFilter } from '@/ipc/bindings'

const FLAGS: { value: FlagFilter; label: string }[] = [
  { value: 'all', label: 'All' },
  { value: 'picks', label: 'Picks' },
  { value: 'unflagged', label: 'Unflagged' },
  { value: 'rejects', label: 'Rejects' },
]

type CullFilterBarProps = {
  filter: CullFilter
  onChange: (filter: CullFilter) => void
}

/** Library filter (FR-022). Export writes exactly the photos it keeps. */
export function CullFilterBar({ filter, onChange }: CullFilterBarProps) {
  return (
    <div className="flex items-center gap-2">
      <ToggleGroup
        type="single"
        variant="outline"
        size="sm"
        value={filter.flag}
        onValueChange={(flag) => flag && onChange({ ...filter, flag: flag as FlagFilter })}
        aria-label="Filter by flag"
      >
        {FLAGS.map(({ value, label }) => (
          <ToggleGroupItem key={value} value={value} className="px-2.5 text-xs">
            {label}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>
      <Select
        value={String(filter.minRating)}
        onValueChange={(v) => onChange({ ...filter, minRating: Number(v) })}
      >
        <SelectTrigger size="sm" className="w-28 text-xs" aria-label="Minimum rating">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="0">Any rating</SelectItem>
          {[1, 2, 3, 4, 5].map((n) => (
            <SelectItem key={n} value={String(n)}>
              <Star className="size-3 fill-current" /> ≥ {n}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  )
}
