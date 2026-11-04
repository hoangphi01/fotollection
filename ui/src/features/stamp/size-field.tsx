// SPDX-License-Identifier: GPL-3.0-only
import { useTranslation } from 'react-i18next'
import { SegmentedToggle } from '@/components/forms/segmented-toggle'
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
    <SegmentedToggle
      value={value ?? 'medium'}
      options={SIZES.map((s) => ({ value: s, label: t(`settings.stampSize.${s}`) }))}
      onChange={onChange}
      ariaLabel={label}
    />
  )
}
