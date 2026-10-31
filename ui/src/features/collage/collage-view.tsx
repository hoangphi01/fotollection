// SPDX-License-Identifier: GPL-3.0-only
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { CullFilter, PhotoSummary, RenderSettings } from '@/ipc/bindings'
import type { ExportApi } from '@/features/export/use-export'
import { SheetGrid } from './sheet-grid'
import { SheetPreviewDialog } from './sheet-preview-dialog'
import type { CollageApi } from './use-collage'

type CollageViewProps = {
  collage: CollageApi
  photos: PhotoSummary[]
  settings: RenderSettings | null
  onSettingsChange: (change: (current: RenderSettings) => RenderSettings) => void
  exporter: ExportApi
  filter: CullFilter
  revision: number
  generation: number
}

/**
 * Collage mode's main area (FR-035): the sheets as they will print, and a sheet's preview,
 * opened like a photo's in the gallery.
 */
export function CollageView({
  collage,
  photos,
  settings,
  onSettingsChange,
  exporter,
  filter,
  revision,
  generation,
}: CollageViewProps) {
  const { t } = useTranslation()
  const [viewing, setViewing] = useState(false)
  const { sheets, index } = collage

  if (sheets.length === 0) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-2 p-8 text-center">
        <p className="font-medium">{t('collage.empty')}</p>
        <p className="max-w-sm text-sm text-muted-foreground">{t('collage.emptyHint')}</p>
      </div>
    )
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex min-h-[49px] items-center gap-3 border-b px-4 py-1.5 text-sm">
        <span className="shrink-0 font-medium tabular-nums">
          {t('collage.count', { count: sheets.length, photos: photos.length })}
        </span>
        <span className="min-w-0 flex-1 truncate text-xs text-muted-foreground">{t('collage.keysHint')}</span>
      </div>
      <SheetGrid
        sheets={sheets}
        revision={revision}
        generation={generation}
        selected={index}
        selectedCell={collage.cellIndex}
        onSelect={collage.select}
        onOpen={(sheet, cell) => {
          collage.select(sheet, cell)
          setViewing(true)
        }}
        onMove={collage.move}
        onTurn={collage.turn}
      />
      <SheetPreviewDialog
        collage={collage}
        open={viewing}
        photos={photos}
        settings={settings}
        onSettingsChange={onSettingsChange}
        exporter={exporter}
        filter={filter}
        revision={revision}
        generation={generation}
        onClose={() => setViewing(false)}
      />
    </div>
  )
}
