// SPDX-License-Identifier: GPL-3.0-only
import { useTranslation } from 'react-i18next'
import { ApplyLookCard } from '@/features/look/apply-look-card'
import { ColourPrintTabs } from '@/features/look/colour-print-tabs'
import { useColourPrintTab } from '@/features/look/use-colour-print-tab'
import { PanelFooter } from '@/features/about/panel-footer'
import { SidePanel } from '@/components/layout/side-panel'
import type { CellInfo, CullFilter, PhotoId, PhotoSummary, RenderSettings, SheetInfo } from '@/ipc/bindings'
import { ExportSection } from '@/features/export/export-section'
import type { ExportApi } from '@/features/export/use-export'
import { DateStampCard } from '@/features/stamp/date-stamp-card'
import { CollageCellsCard } from './collage-cells-card'
import { PostcardCards } from './postcard-cards'
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
  /** That photo's summary, for the Colour tab's look (FR-137). */
  cellPhoto?: PhotoSummary
  /** The chosen sheet's pixels and how sharp it prints (FR-087). */
  sheetInfo?: SheetInfo
  /** Photos with their own stamp from the preview (FR-044). */
  customStamps: number
  revision: number
}


/**
 * Collage mode's panel (FR-035, FR-048) in the Pro panel's two tabs (FR-137, owner
 * 2026-11-15): **Colour**, the same look card as Pro, on the chosen sheet's photo or every
 * ticked one; **Print**, in the shared order: the sheet's size, the postcard (FR-073, FR-136),
 * how the photos share it, the date stamp. Export, which writes one file per sheet, stays
 * under both.
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
  cellPhoto,
  sheetInfo,
  customStamps,
  revision,
}: CollagePanelProps) {
  const { t } = useTranslation()
  const [tab, openTab] = useColourPrintTab('collage')
  return (
    <SidePanel
      title={tab === 'colour' ? t('inspector.colour') : t('collage.title')}
      tabs={
        <ColourPrintTabs value={tab} onChange={openTab} />
      }
      footer={
        <>
          <ExportSection exporter={exporter} photoCount={photoCount} filter={filter} revision={revision} sheets={tickedSheets > 0 ? tickedSheets : sheetCount} ticked={tickedSheets > 0 ? ticked : undefined} />
          <PanelFooter />
        </>
      }
    >
      {tab === 'colour' ? (
        <ApplyLookCard
          filter={filter}
          ticked={tickedSheets > 0 && ticked ? [...ticked] : undefined}
          photoCount={photoCount}
          photo={cellPhoto}
          revision={revision}
        />
      ) : (
        <>
          <SheetSizeCard settings={settings} onChange={onChange} info={sheetInfo} />
          <PostcardCards
            settings={settings}
            onChange={onChange}
            photo={cell?.id}
            revision={revision}
            filter={filter}
            ticked={tickedSheets > 0 ? ticked : undefined}
          />
          <CollageCellsCard settings={settings} onChange={onChange} cell={cell} />
          <DateStampCard settings={settings} onChange={onChange} customStamps={customStamps} folders={filter.folders ?? []} />
        </>
      )}
    </SidePanel>
  )
}
