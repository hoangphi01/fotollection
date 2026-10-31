// SPDX-License-Identifier: GPL-3.0-only
import { useTranslation } from 'react-i18next'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import type { StampSettings, StampSize } from '@/ipc/bindings'

const SIZES: StampSize[] = ['small', 'medium', 'large']

type StampSizeFieldProps = {
  stamp: StampSettings | undefined
  onChange: (change: Partial<StampSettings>) => void
}

/** Small, medium or large date stamp (FR-004): the same control in every mode. */
export function StampSizeField({ stamp, onChange }: StampSizeFieldProps) {
  const { t } = useTranslation()
  return (
    <ToggleGroup
      type="single"
      variant="outline"
      size="sm"
      className="w-full"
      value={stamp?.size ?? 'medium'}
      onValueChange={(size) => size && onChange({ size: size as StampSize })}
      aria-label={t('settings.stampSize.label')}
    >
      {SIZES.map((s) => (
        <ToggleGroupItem key={s} value={s} className="flex-1 text-xs">
          {t(`settings.stampSize.${s}`)}
        </ToggleGroupItem>
      ))}
    </ToggleGroup>
  )
}
