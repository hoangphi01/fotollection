// SPDX-License-Identifier: GPL-3.0-only
import { Crop, Languages, MapPin } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Switch } from '@/components/ui/switch'
import { Collapse } from '@/components/layout/collapse'
import { CameraClocksCard } from '@/features/cameras/camera-clocks-card'
import { PlacesCard } from '@/features/places/places-card'
import { SectionCard } from '@/components/layout/section-card'
import { SidePanel } from '@/components/layout/side-panel'
import { CommitInput } from '@/components/forms/commit-input'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { ExportSection } from '@/features/export/export-section'
import { ExportTargetPicker } from '@/features/export/export-target-picker'
import type { ExportApi } from '@/features/export/use-export'
import { FrameSection } from '@/features/frame/frame-section'
import type { FrameConfigApi } from '@/features/frame/use-frame-config'
import {
  commands,
  type CullFilter,
  type Language,
  type PhotoSummary,
  type PlaceFormat,
  type RenderSettings,
} from '@/ipc/bindings'
import type { FontsApi } from '@/features/fonts/use-fonts'
import { PrintAdviceNote } from '@/features/print-advice/print-advice-note'
import { FitSizeNote } from '@/features/frame/fit-size-note'
import { BorderSwitch } from './border-switch'
import { DateStampCard } from '@/features/stamp/date-stamp-card'
import { VersionHistory } from '@/features/about/version-history'
import { usePrintAdvice } from '@/features/print-advice/use-print-advice'
import { LANGUAGES } from '@/i18n'
import { PrintSizePicker } from './print-size-picker'
import { useLanguage } from './use-language'
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
}

const PLACE_FORMATS: PlaceFormat[] = ['city', 'cityCountryCode', 'cityCountry']




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
}: SettingsPanelProps) {
  const { t } = useTranslation()
  const { language, change: changeLanguage } = useLanguage()
  const printSizes = useIpcQuery(commands.listPrintSizes)
  const appInfo = useIpcQuery(commands.appInfo)
  const advice = usePrintAdvice(photo?.id, settings)
  const stamp = settings?.stamp
  const setStamp = (patch: Partial<RenderSettings['stamp']>) =>
    onChange((s) => ({ ...s, stamp: { ...s.stamp, ...patch } }))

  return (
    <SidePanel
      title={t('settings.title')}
      footer={
        <>
          <ExportSection exporter={exporter} photoCount={photoCount} folderCount={folderCount} filter={filter} revision={revision} />
          <div className="flex items-center justify-between text-xs text-muted-foreground">
            <span>{t('settings.footer')}</span>
            {appInfo && <VersionHistory version={appInfo.appVersion} />}
          </div>
        </>
      }
    >
        <SectionCard
          icon={<Languages />}
          title={
            <Label htmlFor="ui-language" className="text-sm font-semibold whitespace-nowrap">
              {t('settings.language')}
            </Label>
          }
          action={
          <Select value={language} onValueChange={(v) => void changeLanguage(v as Language)}>
            <SelectTrigger id="ui-language" size="sm" className="w-auto gap-1 px-2 text-xs">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {LANGUAGES.map((l) => (
                <SelectItem key={l.value} value={l.value}>
                  {l.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          }
        />

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

        <DateStampCard settings={settings} onChange={onChange} customStamps={customStamps}>

            {/* The place on the date line (FR-005), from the photo's GPS. */}
            <div className="flex items-center justify-between">
              <Label htmlFor="stamp-place" className="flex items-center gap-2 text-sm font-normal">
                <MapPin className="size-4 text-muted-foreground" />
                {t('settings.location')}
              </Label>
              <Switch
                id="stamp-place"
                checked={stamp?.place ?? false}
                onCheckedChange={(place) => setStamp({ place })}
              />
            </div>
            <Collapse open={stamp?.place ?? false} className="space-y-1.5">
              <ToggleGroup
                type="single"
                variant="outline"
                size="sm"
                className="w-full"
                value={settings?.placeFormat ?? ''}
                onValueChange={(v) => v && onChange((s) => ({ ...s, placeFormat: v as PlaceFormat }))}
              >
                {PLACE_FORMATS.map((f) => (
                  <ToggleGroupItem key={f} value={f} className="flex-1 text-xs">
                    {t(`settings.placeFormat.${f}`)}
                  </ToggleGroupItem>
                ))}
              </ToggleGroup>
              <p className="text-xs text-muted-foreground">{t('settings.locationHint')}</p>
            </Collapse>

            {/* A line of the user's own text under the date (FR-036). */}
            <div className="space-y-1.5">
              <Label className="text-xs font-normal text-muted-foreground">
                {t('settings.caption')}
              </Label>
              <CommitInput
                value={stamp?.caption ?? ''}
                placeholder={t('settings.captionPlaceholder')}
                maxLength={80}
                liveMs={300}
                ariaLabel={t('settings.caption')}
                onCommit={(caption) => setStamp({ caption })}
              />
              <p className="text-xs text-muted-foreground">{t('settings.captionHint')}</p>
            </div>
        </DateStampCard>

        <FrameSection frame={frame} fonts={fonts} />
        <CameraClocksCard />
        <PlacesCard />
    </SidePanel>
  )
}
