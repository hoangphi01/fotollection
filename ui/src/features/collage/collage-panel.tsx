// SPDX-License-Identifier: GPL-3.0-only
import { useTranslation } from 'react-i18next'
import { PanelFooter } from '@/features/about/panel-footer'
import { SidePanel } from '@/components/layout/side-panel'
import type { CellInfo, CullFilter, PhotoId, RenderSettings } from '@/ipc/bindings'
import { ExportSection } from '@/features/export/export-section'
import type { ExportApi } from '@/features/export/use-export'
import { DateStampCard } from '@/features/stamp/date-stamp-card'
import { CollageCellsCard } from './collage-cells-card'
import { PostcardCard } from './postcard-card'
import { PostcardTextCard } from './postcard-text-card'
import { SheetSizeCard } from './sheet-size-card'

type CollagePanelProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
  exporter: ExportApi
  filter: CullFilter
  photoCount: number
  sheetCount: number
  /** Photos of the sheets ticked in select mode, and how many sheets: only they go out (FR-050). */
  ticked?: readonly PhotoId[]
  tickedSheets?: number
  /** The photo chosen on the selected sheet, for the size fields and the advice. */
  cell: CellInfo | undefined
  /** Photos with their own stamp from the preview (FR-044). */
  customStamps: number
  revision: number
}

/**
 * Collage mode's panel (FR-035, FR-048), in the shared order: the sheet's size, the postcard
 * switch (FR-073), how the photos share it, the date stamp, and Export, which writes one
 * file per sheet. Sheets are always prints.
 */
export function CollagePanel({
  settings,
  onChange,
  exporter,
  filter,
  photoCount,
  sheetCount,
  ticked,
  tickedSheets = 0,
  cell,
  customStamps,
  revision,
}: CollagePanelProps) {
  const { t } = useTranslation()
  return (
    <SidePanel
      title={t('collage.title')}
      footer={
        <>
          <ExportSection exporter={exporter} photoCount={photoCount} filter={filter} revision={revision} sheets={tickedSheets > 0 ? tickedSheets : sheetCount} ticked={tickedSheets > 0 ? ticked : undefined} />
          <PanelFooter />
        </>
      }
    >
      <SheetSizeCard settings={settings} onChange={onChange} />
      <PostcardCard settings={settings} onChange={onChange} />
      <PostcardTextCard
        settings={settings}
        onChange={onChange}
        photo={cell?.id}
        revision={revision}
        filter={filter}
        ticked={tickedSheets > 0 ? ticked : undefined}
      />
      <CollageCellsCard settings={settings} onChange={onChange} cell={cell} />
      <DateStampCard settings={settings} onChange={onChange} customStamps={customStamps} />
    </SidePanel>
  )
}
