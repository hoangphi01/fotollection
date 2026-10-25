// SPDX-License-Identifier: GPL-3.0-only
import { CalendarDays, Crop, Languages, MapPin, RotateCcw } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Separator } from '@/components/ui/separator'
import { Switch } from '@/components/ui/switch'
import { Collapse } from '@/components/layout/collapse'
import { SectionCard } from '@/components/layout/section-card'
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
import { StampStyleFields } from '@/features/stamp/stamp-style-fields'
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
}: SettingsPanelProps) {
  const { t } = useTranslation()
  const { language, change: changeLanguage } = useLanguage()
  const printSizes = useIpcQuery(commands.listPrintSizes)
  const appInfo = useIpcQuery(commands.appInfo)
  const sizesKey = (settings?.customSizes ?? []).map((c) => `${c.id}:${c.shortMm}x${c.longMm}`).join()
  const advice = usePrintAdvice(photo?.id, sizesKey)
  const stamp = settings?.stamp
  const setStamp = (patch: Partial<RenderSettings['stamp']>) =>
    onChange((s) => ({ ...s, stamp: { ...s.stamp, ...patch } }))

  return (
    <aside className="flex w-72 shrink-0 flex-col border-l bg-sidebar text-sidebar-foreground">
      <div className="flex h-12 items-center px-4">
        <span className="text-sm font-semibold">{t('settings.title')}</span>
      </div>
      <Separator />

      <div className="flex-1 space-y-3 overflow-y-auto p-3">
        <SectionCard
          icon={<Languages />}
          title={
            <Label htmlFor="ui-language" className="text-sm font-semibold">
              {t('settings.language')}
            </Label>
          }
          action={
          <Select value={language} onValueChange={(v) => void changeLanguage(v as Language)}>
            <SelectTrigger id="ui-language" size="sm" className="w-32">
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
              <p className="text-xs text-muted-foreground">{t('settings.printSizeHint')}</p>
            </>
          )}
        </SectionCard>

        <SectionCard
          icon={<CalendarDays />}
          title={
            <Label htmlFor="date-stamp" className="text-sm font-semibold">
              {t('settings.dateStamp')}
            </Label>
          }
          action={
            <Switch
              id="date-stamp"
              checked={stamp?.enabled ?? false}
              onCheckedChange={(enabled) => setStamp({ enabled })}
            />
          }
          open={stamp?.enabled ?? false}
          footer={
            customStamps > 0 && (
              <Button variant="outline" size="sm" className="w-full" onClick={() => void commands.resetPhotoStamps()}>
                <RotateCcw data-icon="inline-start" />
                {t('settings.resetCustomStamps', { count: customStamps })}
              </Button>
            )
          }
        >


            <StampStyleFields stamp={stamp} onChange={setStamp} />

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
        </SectionCard>

        <FrameSection frame={frame} fonts={fonts} />
      </div>

      <Separator />
      <div className="space-y-3 p-4">
        <ExportSection exporter={exporter} photoCount={photoCount} folderCount={folderCount} filter={filter} />
        <div className="flex items-center justify-between text-xs text-muted-foreground">
          <span>{t('settings.footer')}</span>
          {appInfo && <VersionHistory version={appInfo.appVersion} />}
        </div>
      </div>
    </aside>
  )
}
