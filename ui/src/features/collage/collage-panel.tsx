// SPDX-License-Identifier: GPL-3.0-only
import { useTranslation } from 'react-i18next'
import { PanelFooter } from '@/features/about/panel-footer'
import { SidePanel } from '@/components/layout/side-panel'
import type { CellInfo, CullFilter, RenderSettings } from '@/ipc/bindings'
import { ExportSection } from '@/features/export/export-section'
import type { ExportApi } from '@/features/export/use-export'
import { DateStampCard } from '@/features/stamp/date-stamp-card'
import { CollageCellsCard } from './collage-cells-card'
import { SheetSizeCard } from './sheet-size-card'

type CollagePanelProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
  exporter: ExportApi
  filter: CullFilter
  photoCount: number
  sheetCount: number
  /** The photo chosen on the selected sheet, for the size fields and the advice. */
  cell: CellInfo | undefined
  /** Photos with their own stamp from the preview (FR-044). */
  customStamps: number
  revision: number
}

/**
 * Collage mode's panel (FR-035, FR-048), in the shared order: the sheet's size, how two
 * photos share it, the date stamp, and Export, which writes one file per sheet. Sheets are
 * always prints.
 */
export function CollagePanel({
  settings,
  onChange,
  exporter,
  filter,
  photoCount,
  sheetCount,
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
          <ExportSection exporter={exporter} photoCount={photoCount} filter={filter} revision={revision} sheets={sheetCount} />
          <PanelFooter />
        </>
      }
    >
      <SheetSizeCard settings={settings} onChange={onChange} />
      <CollageCellsCard settings={settings} onChange={onChange} cell={cell} />
      <DateStampCard settings={settings} onChange={onChange} customStamps={customStamps} />
    </SidePanel>
  )
}
