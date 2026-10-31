// SPDX-License-Identifier: GPL-3.0-only
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { PhotoId, PhotoSummary, SheetPair } from '@/ipc/bindings'
import { useQuickPreview } from '@/features/viewer/use-quick-preview'
import { CellCropDialog } from './cell-crop-dialog'
import { SheetGrid } from './sheet-grid'
import { SheetPreview } from './sheet-preview'
import type { CollageApi } from './use-collage'

type CollageViewProps = {
  collage: CollageApi
  photos: PhotoSummary[]
  revision: number
  generation: number
}

/**
 * Collage mode's main area (FR-035): the sheets as they will print, Space held to look at
 * one, and the crop editor for a photo of a sheet.
 */
export function CollageView({ collage, photos, revision, generation }: CollageViewProps) {
  const { t } = useTranslation()
  const [editing, setEditing] = useState<{ photo: PhotoId; sheet: SheetPair } | null>(null)
  const { sheets, index } = collage
  const quick = useQuickPreview(editing === null, sheets.length, null, index, () => {})
  const quickSheet = quick.index === null ? undefined : sheets[quick.index]
  const fileName = (id: PhotoId) => photos.find((p) => p.id === id)?.fileName ?? ''

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
        onEdit={(photo, sheet) => setEditing({ photo, sheet })}
        onMove={collage.move}
        onTurn={collage.turn}
      />
      {quickSheet && quick.index !== null && (
        <SheetPreview
          sheet={quickSheet}
          index={quick.index}
          count={sheets.length}
          revision={revision}
          onClose={quick.close}
        />
      )}
      {editing && (
        <CellCropDialog
          photo={editing.photo}
          sheet={editing.sheet}
          fileName={fileName(editing.photo)}
          revision={revision}
          onClose={() => setEditing(null)}
        />
      )}
    </div>
  )
}
