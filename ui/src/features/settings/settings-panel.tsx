// SPDX-License-Identifier: GPL-3.0-only
import { Crop } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { PanelFooter } from '@/features/about/panel-footer'
import { Label } from '@/components/ui/label'
import { CameraClocksCard } from '@/features/cameras/camera-clocks-card'
import { PlacesCard } from '@/features/places/places-card'
import { SectionCard } from '@/components/layout/section-card'
import { SidePanel } from '@/components/layout/side-panel'
import { ExportSection } from '@/features/export/export-section'
import { ExportTargetPicker } from '@/features/export/export-target-picker'
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
import { PrintAdviceNote } from '@/features/print-advice/print-advice-note'
import { FitSizeNote } from '@/features/frame/fit-size-note'
import { BorderSwitch } from './border-switch'
import { DateStampCard } from '@/features/stamp/date-stamp-card'
import { usePrintAdvice } from '@/features/print-advice/use-print-advice'
import { PrintSizePicker } from './print-size-picker'
import { useIpcQuery } from './use-ipc-query'

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

  return (
    <SidePanel
      title={t('settings.title')}
      footer={
        <>
          <ExportSection exporter={exporter} photoCount={photoCount} folderCount={folderCount} filter={filter} revision={revision} ticked={ticked} />
          <PanelFooter />
        </>
      }
    >
        <SectionCard
          icon={<Crop />}
          title={
            <Label htmlFor="print-size" className="text-sm font-semibold">
              {t('settings.printSize')}
            </Label>
          }
        >
          {/* Print or share is chosen with the size, so the export card only exports (FR-053). */}
          <ExportTargetPicker settings={settings} onChange={onChange} />
          {settings?.exportTarget?.kind !== 'share' && (
            <>
              <PrintSizePicker settings={settings} builtIn={printSizes} onChange={onChange} advice={advice} />
              {advice && photo && <PrintAdviceNote advice={advice} fileName={photo.fileName} />}
              <FitSizeNote photoId={photo?.id} revision={revision} />
              <p className="text-xs text-muted-foreground">{t('settings.printSizeHint')}</p>
              <BorderSwitch settings={settings} onChange={onChange} />
            </>
          )}
        </SectionCard>

        <DateStampCard settings={settings} onChange={onChange} customStamps={customStamps} />

        <FrameSection frame={frame} fonts={fonts} />
        <CameraClocksCard />
        <PlacesCard />
    </SidePanel>
  )
}
