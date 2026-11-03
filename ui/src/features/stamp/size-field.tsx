// SPDX-License-Identifier: GPL-3.0-only
import { useTranslation } from 'react-i18next'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import type { StampSize } from '@/ipc/bindings'

const SIZES: StampSize[] = ['small', 'medium', 'large']

type SizeFieldProps = {
  value: StampSize | undefined
  onChange: (size: StampSize) => void
  label: string
}

/**
 * Small, medium or large: the date stamp's size (FR-004) and a postcard's typed text
 * (FR-073), the same control wherever text has a size.
 */
export function SizeField({ value, onChange, label }: SizeFieldProps) {
  const { t } = useTranslation()
  return (
    <ToggleGroup
      type="single"
      variant="outline"
      size="sm"
      className="w-full"
      value={value ?? 'medium'}
      onValueChange={(size) => size && onChange(size as StampSize)}
      aria-label={label}
    >
      {SIZES.map((s) => (
        <ToggleGroupItem key={s} value={s} className="flex-1 text-xs">
          {t(`settings.stampSize.${s}`)}
        </ToggleGroupItem>
      ))}
    </ToggleGroup>
  )
}
