// SPDX-License-Identifier: GPL-3.0-only
import { Palette, Printer } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { InspectorTabs } from '@/components/layout/inspector-tabs'
import { ApplyLookCard } from '@/features/look/apply-look-card'
import { PanelFooter } from '@/features/about/panel-footer'
import { CameraClocksCard } from '@/features/cameras/camera-clocks-card'
import { PlacesCard } from '@/features/places/places-card'
import { SidePanel } from '@/components/layout/side-panel'
import { ExportSection } from '@/features/export/export-section'
import type { ExportApi } from '@/features/export/use-export'
import { FrameSection } from '@/features/frame/frame-section'
import type { FrameConfigApi } from '@/features/frame/use-frame-config'
import {
  commands,
  type CullFilter,
  type PhotoId,
  type PhotoSummary,
  type RenderSettings,
} from '@/ipc/bindings'
import type { FontsApi } from '@/features/fonts/use-fonts'
import { DateStampCard } from '@/features/stamp/date-stamp-card'
import { usePrintAdvice } from '@/features/print-advice/use-print-advice'
import { useIpcQuery } from './use-ipc-query'
import { PrintSizeCard } from './print-size-card'

type SettingsPanelProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
  frame: FrameConfigApi
  fonts: FontsApi
  exporter: ExportApi
  photoCount: number
  /** Source folders in the export (FR-056). */
  folderCount: number
  /** Photos with their own stamp from the preview (FR-044). */
  customStamps: number
  filter: CullFilter
  /** The photo selected in the grid, for the print-size advice (FR-026). */
  photo?: PhotoSummary
  /** Settings revision, so size notes follow every change. */
  revision: number
  /** Photos ticked in select mode, exported instead of the whole view (FR-050). */
  ticked?: readonly PhotoId[]
}


type GalleryTab = 'colour' | 'print'
/** The tab open, kept while the app runs; Print first, the panel as it always opened. */
let lastTab: GalleryTab = 'print'

/**
 * Pro mode's gallery panel in two tabs as the photo preview's inspector (owner, 2026-11-08):
 * **Colour**, a look, LUT or preset given to many photos at once (FR-114), and **Print**, the
 * print size, stamp, frame, clocks and places; the export stays under both.
 */
export function SettingsPanel({
  settings,
  onChange,
  frame,
  fonts,
  exporter,
  photoCount,
  folderCount,
  customStamps,
  filter,
  photo,
  revision,
  ticked,
}: SettingsPanelProps) {
  const { t } = useTranslation()
  const printSizes = useIpcQuery(commands.listPrintSizes)
  const advice = usePrintAdvice(photo?.id, settings)
  const [tab, setTab] = useState<GalleryTab>(lastTab)
  const openTab = (next: GalleryTab) => {
    lastTab = next
    setTab(next)
  }

  return (
    <SidePanel
      title={tab === 'colour' ? t('inspector.colour') : t('settings.title')}
      tabs={
        <InspectorTabs<GalleryTab>
          groups={[
            [
              { value: 'colour', icon: <Palette />, label: t('inspector.colour') },
              { value: 'print', icon: <Printer />, label: t('inspector.print') },
            ],
          ]}
          value={tab}
          onChange={openTab}
          ariaLabel={t('inspector.tabs')}
        />
      }
      footer={
        <>
          <ExportSection share={settings?.exportTarget?.kind === 'share'} exporter={exporter} photoCount={photoCount} folderCount={folderCount} filter={filter} revision={revision} ticked={ticked} />
          <PanelFooter />
        </>
      }
    >
      {tab === 'colour' ? (
        <ApplyLookCard filter={filter} ticked={ticked ? [...ticked] : undefined} photoCount={photoCount} photo={photo} />
      ) : (
        <>
          <PrintSizeCard settings={settings} onChange={onChange} printSizes={printSizes} photo={photo} advice={advice} revision={revision} orientationIds={ticked?.length ? ticked : photo ? [photo.id] : undefined} />
          <DateStampCard settings={settings} onChange={onChange} customStamps={customStamps} />
          <FrameSection frame={frame} fonts={fonts} />
          <CameraClocksCard />
          <PlacesCard />
        </>
      )}
    </SidePanel>
  )
}
