// SPDX-License-Identifier: GPL-3.0-only
import { Lock, LockOpen, RotateCcw } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { CommitInput } from '@/components/forms/commit-input'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Toggle } from '@/components/ui/toggle'
import type { CellInfo, RenderSettings } from '@/ipc/bindings'
import { formatNumber } from '@/lib/format'

type CollageSizeFieldsProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
  /** The photo the panel shows: its area now, and the size suggested for it. */
  cell: CellInfo | undefined
}

const parseCm = (text: string) => Number.parseFloat(text.replace(',', '.'))

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
  const cm = (mm: number) => (mm > 0 ? formatNumber(mm / 10, 1) : '')

  const setSize = (w: number, h: number) =>
    onChange((s) => ({
      ...s,
      collage: {
        ...s.collage,
        size: { shortMm: Math.round(Math.min(w, h)), longMm: Math.round(Math.max(w, h)) },
      },
    }))
  // A typed edge in cm; with the lock, the other edge keeps the photo's ratio.
  const commit = (edge: 'width' | 'height', text: string) => {
    const mm = parseCm(text) * 10
    if (!Number.isFinite(mm) || mm <= 0 || width <= 0 || height <= 0) return
    if (edge === 'width') setSize(mm, locked ? (mm * height) / width : height)
    else setSize(locked ? (mm * width) / height : width, mm)
  }

  return (
    <div className="space-y-1.5">
      <Label className="text-sm font-normal">{t('collage.size')}</Label>
      <div className="flex items-center gap-1.5">
        <CommitInput
          value={cm(width)}
          placeholder={t('collage.width')}
          ariaLabel={t('collage.width')}
          numeric
          onCommit={(v) => commit('width', v)}
        />
        <span className="text-xs text-muted-foreground">×</span>
        <CommitInput
          value={cm(height)}
          placeholder={t('collage.height')}
          ariaLabel={t('collage.height')}
          numeric
          onCommit={(v) => commit('height', v)}
        />
        <span className="text-xs text-muted-foreground">cm</span>
        <Toggle
          variant="outline"
          size="sm"
          pressed={locked}
          onPressedChange={(lockRatio) => onChange((s) => ({ ...s, collage: { ...s.collage, lockRatio } }))}
          aria-label={t('collage.lockRatio')}
          data-hint={t('collage.lockRatio')}
        >
          {locked ? <Lock /> : <LockOpen />}
        </Toggle>
      </div>
      <div className="flex items-center justify-between gap-2">
        <p className="text-xs text-muted-foreground">
          {cell &&
            t('collage.suggested', {
              size: `${cm(cell.suggestedWidthMm)} × ${cm(cell.suggestedHeightMm)} cm`,
            })}
        </p>
        {typed && (
          <Button
            variant="ghost"
            size="xs"
            onClick={() => onChange((s) => ({ ...s, collage: { ...s.collage, size: null } }))}
          >
            <RotateCcw data-icon="inline-start" />
            {t('collage.useSuggested')}
          </Button>
        )}
      </div>
    </div>
  )
}
