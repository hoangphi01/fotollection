// SPDX-License-Identifier: GPL-3.0-only
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Field, ValueRow } from '@/components/forms/field'
import { Slider } from '@/components/ui/slider'
import { formatNumber } from '@/lib/format'

/** Matches `frame::LOGO_SCALE_RANGE` in Rust, which clamps whatever is sent. */
const MIN = 0.5
const MAX = 2

type LogoScaleSliderProps = {
  value: number
  /** Called once the thumb is released, so dragging does not re-render every preview. */
  onCommit: (scale: number) => void
}

/** Size of the camera logo in the info frame, the same for every photo (FR-040). */
export function LogoScaleSlider({ value, onCommit }: LogoScaleSliderProps) {
  const { t } = useTranslation()
  const [draft, setDraft] = useState<number | null>(null)
  const shown = draft ?? value

  return (
    <Field>
      <ValueRow label={t('frame.logoSize')}>{formatNumber(shown * 100, 0)} %</ValueRow>
      <Slider
        min={MIN}
        max={MAX}
        step={0.05}
        value={[shown]}
        aria-label={t('frame.logoSize')}
        onValueChange={([v]) => v !== undefined && setDraft(v)}
        onValueCommit={([v]) => {
          if (v !== undefined) onCommit(v)
          setDraft(null)
        }}
        onDoubleClick={() => onCommit(1)}
        data-hint={t('frame.logoSizeHint')}
      />
    </Field>
  )
}
