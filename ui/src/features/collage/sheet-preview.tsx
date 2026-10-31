// SPDX-License-Identifier: GPL-3.0-only
import { LoaderCircle } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import type { SheetPair } from '@/ipc/bindings'
import { useShownSrc } from '@/features/viewer/use-shown-src'
import { sheetUrl } from '@/lib/photo-url'

type SheetPreviewProps = {
  sheet: SheetPair
  index: number
  count: number
  revision: number
  onClose: () => void
}

/**
 * A collage sheet as it will print, while Space is held (FR-035, FR-059): the same view as
 * Quick-preview, for the whole sheet.
 */
export function SheetPreview({ sheet, index, count, revision, onClose }: SheetPreviewProps) {
  const { t } = useTranslation()
  const shown = useShownSrc(`${sheet.first}:${sheet.second}`, sheetUrl(sheet.first, sheet.second, revision))
  return (
    <div
      className="fixed inset-0 z-50 flex animate-in flex-col items-center justify-center gap-3 bg-black/85 p-10 fade-in-0"
      onClick={onClose}
      role="presentation"
    >
      <div className="relative flex min-h-0 flex-1 items-center justify-center">
        {shown ? (
          <img src={shown} alt={t('collage.sheetLabel')} className="max-h-full max-w-full rounded-sm object-contain shadow-2xl" />
        ) : (
          <LoaderCircle className="size-7 animate-spin text-white/60" />
        )}
      </div>
      <p className="flex items-center gap-2 text-sm text-white/80">
        <span className="tabular-nums">{t('collage.sheetOf', { index: index + 1, count })}</span>
        <span className="text-xs text-white/50">{t('collage.quickHint')}</span>
      </p>
    </div>
  )
}
