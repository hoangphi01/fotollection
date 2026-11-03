// SPDX-License-Identifier: GPL-3.0-only
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { CullFilter, PhotoId, PhotoSummary, RenderSettings } from '@/ipc/bindings'
import type { ExportApi } from '@/features/export/use-export'
import { KeysInfo } from '@/components/layout/keys-info'
import { formatNumber } from '@/lib/format'
import { SelectModeButton } from '@/features/library/select-mode-button'
import { SelectionBar } from '@/features/library/selection-bar'
import type { SelectModeApi } from '@/features/library/use-select-mode'
import { DatePlaceDialog } from '@/features/places/date-place-dialog'
import { AutoTurnButtons } from './auto-turn-buttons'
import { tickedSheets } from './sheet-cells'
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
  /** Photos with their own stamp from the preview (FR-044). */
  customStamps: number
  revision: number
  generation: number
  /** Select mode, shared with the panel so Export follows the ticked sheets (FR-050). */
  select: SelectModeApi
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
  customStamps,
  revision,
  generation,
  select,
}: CollageViewProps) {
  const { t } = useTranslation()
  const [viewing, setViewing] = useState(false)
  const [datePlaceIds, setDatePlaceIds] = useState<PhotoId[] | null>(null)
  const { sheets, index } = collage
  const view = filter.queue ?? 'gallery'
  const queueOf = new Map(photos.map((p) => [p.id, p.queue]))

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
      <div className="@container flex min-h-[49px] items-center gap-3 border-b px-4 py-1.5 text-sm">
        <span className="shrink-0 font-medium tabular-nums">
          {t('collage.count', { count: sheets.length, sheets: formatNumber(sheets.length), photos: formatNumber(photos.length) })}
        </span>
        <KeysInfo lines={t('collage.keysHint').split(' · ')} />
        <span className="flex-1" />
        {/* Select mode (FR-050): tick sheets, both photos at once, for the basket, to set aside
            or to export. */}
        {/* A postcard stays the way it is held (FR-073): nothing to turn. */}
        {!select.active && !settings?.collage?.postcard?.enabled && <AutoTurnButtons filter={filter} />}
        <SelectModeButton select={select} />
      </div>
      {select.active && (
        <SelectionBar
          count={tickedSheets(sheets, select.checked)}
          unit="sheets"
          elsewhere={select.checked.size - photos.filter((p) => select.checked.has(p.id)).length}
          view={view}
          onMove={select.move}
          onDatePlace={() => setDatePlaceIds([...select.checked])}
          onSelectAll={select.tickAll}
          onClear={select.clear}
        />
      )}
      {datePlaceIds && <DatePlaceDialog ids={datePlaceIds} onClose={() => setDatePlaceIds(null)} />}
      <SheetGrid
        sheets={sheets}
        revision={revision}
        generation={generation}
        selected={index}
        selectedCell={collage.cellIndex}
        selectMode={select.active}
        checked={select.checked}
        inBasket={(photo) => queueOf.get(photo) === 'print'}
        onToggle={(sheet) => select.toggleGroup(sheet.second === null ? [sheet.first] : [sheet.first, sheet.second])}
        onToggleAll={select.tickAll}
        onSelect={collage.select}
        onOpen={(sheet, cell) => {
          collage.select(sheet, cell)
          setViewing(true)
        }}
        onMove={collage.move}
        onTurn={settings?.collage?.postcard?.enabled ? undefined : collage.turn}
      />
      <SheetPreviewDialog
        collage={collage}
        open={viewing}
        photos={photos}
        settings={settings}
        onSettingsChange={onSettingsChange}
        exporter={exporter}
        filter={filter}
        customStamps={customStamps}
        revision={revision}
        generation={generation}
        onClose={() => setViewing(false)}
      />
    </div>
  )
}
