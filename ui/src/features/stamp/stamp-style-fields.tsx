// SPDX-License-Identifier: GPL-3.0-only
import { useTranslation } from 'react-i18next'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import type { Corner, DateFormat, StampColor, StampSettings } from '@/ipc/bindings'

const FORMATS: { value: DateFormat; label: string }[] = [
  { value: 'iso', label: '2026-07-14' },
  { value: 'dmy', label: '14.07.2026' },
  { value: 'retro', label: "'26 07 14" },
]

const CORNERS: Corner[] = ['bottomRight', 'bottomLeft', 'topRight', 'topLeft']

const COLORS: { value: StampColor; swatch: string }[] = [
  { value: 'white', swatch: 'bg-white' },
  { value: 'retroOrange', swatch: 'bg-[#ff8a1f]' },
]

type StampStyleFieldsProps = {
  stamp: StampSettings | undefined
  onChange: (change: Partial<StampSettings>) => void
}

/**
 * Date format, corner and colour of the date stamp (FR-004, FR-033): the same fields, in the
 * same order, in the settings panel (every photo) and the preview (one photo, FR-044).
 */
export function StampStyleFields({ stamp, onChange }: StampStyleFieldsProps) {
  const { t } = useTranslation()
  return (
    <>
      <ToggleGroup
        type="single"
        variant="outline"
        size="sm"
        className="w-full"
        value={stamp?.format ?? ''}
        onValueChange={(format) => format && onChange({ format: format as DateFormat })}
      >
        {FORMATS.map((f) => (
          <ToggleGroupItem key={f.value} value={f.value} className="min-w-0 flex-1 px-1 font-mono text-[11px] tracking-tight">
            {f.label}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>

      <Select value={stamp?.corner ?? ''} onValueChange={(corner) => onChange({ corner: corner as Corner })}>
        <SelectTrigger className="w-full" aria-label={t('settings.stampPosition')}>
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {CORNERS.map((c) => (
            <SelectItem key={c} value={c}>
              {t(`settings.corner.${c}`)}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>

      <ToggleGroup
        type="single"
        variant="outline"
        size="sm"
        className="w-full"
        value={stamp?.color ?? ''}
        onValueChange={(color) => color && onChange({ color: color as StampColor })}
      >
        {COLORS.map((c) => (
          <ToggleGroupItem key={c.value} value={c.value} className="flex-1 gap-1.5 text-xs">
            <span className={`size-3 rounded-full border ${c.swatch}`} />
            {t(`settings.color.${c.value}`)}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>
    </>
  )
}
