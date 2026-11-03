// SPDX-License-Identifier: GPL-3.0-only
import { useTranslation } from 'react-i18next'
import type { StampSettings } from '@/ipc/bindings'
import { SizeField } from './size-field'

type StampSizeFieldProps = {
  stamp: StampSettings | undefined
  onChange: (change: Partial<StampSettings>) => void
}

/** Small, medium or large date stamp (FR-004): the same control in every mode. */
export function StampSizeField({ stamp, onChange }: StampSizeFieldProps) {
  const { t } = useTranslation()
  return <SizeField value={stamp?.size} onChange={(size) => onChange({ size })} label={t('settings.stampSize.label')} />
}
