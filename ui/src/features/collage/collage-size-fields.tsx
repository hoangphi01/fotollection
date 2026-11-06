// SPDX-License-Identifier: GPL-3.0-only
import { Lock, LockOpen } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { CommitInput } from '@/components/forms/commit-input'
import { Field, Hint } from '@/components/forms/field'
import { Toggle } from '@/components/ui/toggle'
import type { CellInfo, RenderSettings } from '@/ipc/bindings'
import { dimsLabel, displayUnit, lengthIn, parseLength } from '@/lib/print-size'
import { ResetButton } from '@/components/forms/danger-button'

type CollageSizeFieldsProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
  /** The photo the panel shows: its area now, and the size suggested for it. */
  cell: CellInfo | undefined
}

/**
 * The photo's size in its cell (FR-035): the suggested size until one is typed. With the
 * ratio locked, one edge gives the other; Rust keeps every size inside the cell's margin.
 */
export function CollageSizeFields({ settings, onChange, cell }: CollageSizeFieldsProps) {
  const { t } = useTranslation()
  const collage = settings?.collage
  const locked = collage?.lockRatio ?? true
  const typed = collage?.size != null
  const width = cell?.areaWidthMm ?? 0
  const height = cell?.areaHeightMm ?? 0
  const unit = displayUnit(settings)
  const length = (mm: number) => (mm > 0 ? lengthIn(mm, unit) : '')

  const setSize = (w: number, h: number) =>
    onChange((s) => ({
      ...s,
      collage: {
        ...s.collage,
        size: { shortMm: Math.round(Math.min(w, h)), longMm: Math.round(Math.max(w, h)) },
      },
    }))
  // A typed edge, in cm or inches; with the lock, the other edge keeps the photo's ratio.
  const commit = (edge: 'width' | 'height', text: string) => {
    const mm = parseLength(text, unit)
    if (!Number.isFinite(mm) || mm <= 0 || width <= 0 || height <= 0) return
    if (edge === 'width') setSize(mm, locked ? (mm * height) / width : height)
    else setSize(locked ? (mm * width) / height : width, mm)
  }

  return (
    <Field label={t('collage.size')}>
      <div className="flex items-center gap-1.5">
        <CommitInput
          value={length(width)}
          placeholder={t('collage.width')}
          ariaLabel={t('collage.width')}
          numeric
          onCommit={(v) => commit('width', v)}
        />
        <span className="text-xs text-muted-foreground">{t('collage.by')}</span>
        <CommitInput
          value={length(height)}
          placeholder={t('collage.height')}
          ariaLabel={t('collage.height')}
          numeric
          onCommit={(v) => commit('height', v)}
        />
        <span className="text-xs text-muted-foreground">{unit}</span>
        <Toggle
          variant="outline"
          pressed={locked}
          onPressedChange={(lockRatio) => onChange((s) => ({ ...s, collage: { ...s.collage, lockRatio } }))}
          aria-label={t('collage.lockRatio')}
          data-hint={t('collage.lockRatio')}
        >
          {locked ? <Lock /> : <LockOpen />}
        </Toggle>
      </div>
      <div className="flex items-center justify-between gap-2">
        <Hint>
          {cell &&
            t('collage.suggested', {
              size: dimsLabel(cell.suggestedWidthMm, cell.suggestedHeightMm, unit),
            })}
        </Hint>
        {typed && (
          <ResetButton small label={t('collage.useSuggested')} onClick={() => onChange((s) => ({ ...s, collage: { ...s.collage, size: null } }))} />
        )}
      </div>
    </Field>
  )
}
