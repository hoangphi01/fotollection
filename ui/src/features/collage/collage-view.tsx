// SPDX-License-Identifier: GPL-3.0-only
import { ListChecks } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { commands, type CullFilter, type PhotoId, type PhotoSummary, type Queue, type RenderSettings } from '@/ipc/bindings'
import type { ExportApi } from '@/features/export/use-export'
import { SelectionBar } from '@/features/library/selection-bar'
import { DatePlaceDialog } from '@/features/places/date-place-dialog'
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
 * Collage mode's main area (FR-035): the sheets as they will print, select mode and the
 * print basket as in the gallery (FR-050), and a sheet's preview, opened like a photo's.
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
  const [selectMode, setSelectMode] = useState(false)
  const [checked, setChecked] = useState<ReadonlySet<PhotoId>>(new Set())
  const [datePlaceIds, setDatePlaceIds] = useState<PhotoId[] | null>(null)
  const { sheets, index } = collage
  const view = filter.queue ?? 'gallery'
  const queueOf = new Map(photos.map((p) => [p.id, p.queue]))
  const toggle = (photo: PhotoId) =>
    setChecked((prev) => {
      const next = new Set(prev)
      if (next.has(photo)) next.delete(photo)
      else next.add(photo)
      return next
    })
  const leaveSelect = () => {
    setSelectMode(false)
    setChecked(new Set())
  }
  // Moves the ticked photos to the basket, aside, or back, as one undoable step (FR-050).
  const moveChecked = (queue: Queue | null) => {
    const ids = [...checked]
    if (ids.length > 0) void commands.setPhotoQueue(ids, queue)
    setChecked(new Set())
  }
  // Esc leaves select mode, unless a dialog has it first.
  useEffect(() => {
    if (!selectMode) return
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== 'Escape' || document.querySelector('[role="dialog"]')) return
      setSelectMode(false)
      setChecked(new Set())
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [selectMode])

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
        {/* Select mode (FR-050): tick photos for the basket or to set aside. */}
        <Button
          variant={selectMode ? 'default' : 'outline'}
          size="sm"
          aria-pressed={selectMode}
          data-hint={t('select.hint')}
          onClick={() => (selectMode ? leaveSelect() : setSelectMode(true))}
        >
          <ListChecks data-icon="inline-start" />
          {t(selectMode ? 'select.done' : 'select.start')}
        </Button>
      </div>
      {selectMode && (
        <SelectionBar
          count={checked.size}
          elsewhere={checked.size - photos.filter((p) => checked.has(p.id)).length}
          view={view}
          onMove={moveChecked}
          onDatePlace={() => setDatePlaceIds([...checked])}
          onSelectAll={() => setChecked(new Set(photos.map((p) => p.id)))}
          onClear={() => setChecked(new Set())}
        />
      )}
      {datePlaceIds && <DatePlaceDialog ids={datePlaceIds} onClose={() => setDatePlaceIds(null)} />}
      <SheetGrid
        sheets={sheets}
        revision={revision}
        generation={generation}
        selected={index}
        selectedCell={collage.cellIndex}
        selectMode={selectMode}
        checked={checked}
        inBasket={(photo) => queueOf.get(photo) === 'print'}
        onToggle={toggle}
        onToggleAll={() => setChecked(new Set(photos.map((p) => p.id)))}
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
