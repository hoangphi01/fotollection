// SPDX-License-Identifier: GPL-3.0-only
import { CalendarDays, Crop, Languages, MapPin, RotateCcw } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Badge } from '@/components/ui/badge'
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
import { CommitInput } from '@/components/forms/commit-input'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { ExportSection } from '@/features/export/export-section'
import { FrameSection } from '@/features/frame/frame-section'
import type { FrameConfigApi } from '@/features/frame/use-frame-config'
import {
  commands,
  type Corner,
  type CullFilter,
  type DateFormat,
  type Language,
  type PlaceFormat,
  type RenderSettings,
  type StampColor,
} from '@/ipc/bindings'
import type { FontsApi } from '@/features/fonts/use-fonts'
import { LANGUAGES } from '@/i18n'
import { PrintSizePicker } from './print-size-picker'
import { useLanguage } from './use-language'
import { useIpcQuery } from './use-ipc-query'

type SettingsPanelProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
  frame: FrameConfigApi
  fonts: FontsApi
  photoCount: number
  /** Photos with their own stamp from the preview (FR-044). */
  customStamps: number
  filter: CullFilter
}

const PLACE_FORMATS: PlaceFormat[] = ['city', 'cityCountryCode', 'cityCountry']

const FORMATS: { value: DateFormat; label: string }[] = [
  { value: 'iso', label: '2026-07-14' },
  { value: 'dmy', label: '14.07.2026' },
  { value: 'retro', label: "'26 07 14" },
]

const CORNERS: Corner[] = ['bottomRight', 'bottomLeft', 'topRight', 'topLeft']

const COLORS: { value: StampColor; swatch: string }[] = [
  { value: 'white', swatch: 'bg-white' },
  { value: 'retroOrange', swatch: 'bg-[#ff8a1f]' },
]

export function SettingsPanel({
  settings,
  onChange,
  frame,
  fonts,
  photoCount,
  customStamps,
  filter,
}: SettingsPanelProps) {
  const { t } = useTranslation()
  const { language, change: changeLanguage } = useLanguage()
  const printSizes = useIpcQuery(commands.listPrintSizes)
  const appInfo = useIpcQuery(commands.appInfo)
  const stamp = settings?.stamp
  const setStamp = (patch: Partial<RenderSettings['stamp']>) =>
    onChange((s) => ({ ...s, stamp: { ...s.stamp, ...patch } }))

  return (
    <aside className="flex w-72 shrink-0 flex-col border-l">
      <div className="flex h-12 items-center px-4">
        <span className="text-sm font-semibold">{t('settings.title')}</span>
      </div>
      <Separator />

      <div className="flex-1 space-y-6 overflow-y-auto p-4">
        <section className="flex items-center justify-between gap-3">
          <Label htmlFor="ui-language" className="flex items-center gap-2">
            <Languages className="size-4 text-muted-foreground" />
            {t('settings.language')}
          </Label>
          <Select value={language} onValueChange={(v) => void changeLanguage(v as Language)}>
            <SelectTrigger id="ui-language" size="sm" className="w-36">
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
        </section>

        <section className="space-y-2">
          <Label htmlFor="print-size" className="flex items-center gap-2">
            <Crop className="size-4 text-muted-foreground" />
            {t('settings.printSize')}
          </Label>
          <PrintSizePicker settings={settings} builtIn={printSizes} onChange={onChange} />
          <p className="text-xs text-muted-foreground">{t('settings.printSizeHint')}</p>
        </section>

        <section className="space-y-3">
          <div className="flex items-center justify-between">
            <Label htmlFor="date-stamp" className="flex items-center gap-2">
              <CalendarDays className="size-4 text-muted-foreground" />
              {t('settings.dateStamp')}
            </Label>
            <Switch
              id="date-stamp"
              checked={stamp?.enabled ?? false}
              onCheckedChange={(enabled) => setStamp({ enabled })}
            />
          </div>

          {customStamps > 0 && (
            <Button
              variant="outline"
              size="sm"
              className="w-full"
              onClick={() => void commands.resetPhotoStamps()}
            >
              <RotateCcw data-icon="inline-start" />
              {t('settings.resetCustomStamps', { count: customStamps })}
            </Button>
          )}

          <Collapse open={stamp?.enabled ?? false} className="space-y-3">
            <ToggleGroup
              type="single"
              variant="outline"
              size="sm"
              className="w-full"
              value={stamp?.format ?? ''}
              onValueChange={(format) => format && setStamp({ format: format as DateFormat })}
            >
              {FORMATS.map((f) => (
                <ToggleGroupItem key={f.value} value={f.value} className="flex-1 font-mono text-xs">
                  {f.label}
                </ToggleGroupItem>
              ))}
            </ToggleGroup>

            <Select
              value={stamp?.corner ?? ''}
              onValueChange={(corner) => setStamp({ corner: corner as Corner })}
            >
              <SelectTrigger className="w-full" aria-label={t('settings.stampPosition')}>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {CORNERS.map((c) => (
                  <SelectItem key={c} value={c}>
                    {t(`settings.corner.${c}`)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            <ToggleGroup
              type="single"
              variant="outline"
              size="sm"
              className="w-full"
              value={stamp?.color ?? ''}
              onValueChange={(color) => color && setStamp({ color: color as StampColor })}
            >
              {COLORS.map((c) => (
                <ToggleGroupItem key={c.value} value={c.value} className="flex-1 gap-1.5 text-xs">
                  <span className={`size-3 rounded-full border ${c.swatch}`} />
                  {t(`settings.color.${c.value}`)}
                </ToggleGroupItem>
              ))}
            </ToggleGroup>

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
                key={stamp?.caption ?? ''}
                value={stamp?.caption ?? ''}
                placeholder={t('settings.captionPlaceholder')}
                maxLength={80}
                ariaLabel={t('settings.caption')}
                onCommit={(caption) => setStamp({ caption: caption.trim() })}
              />
              <p className="text-xs text-muted-foreground">{t('settings.captionHint')}</p>
            </div>
          </Collapse>
        </section>

        <Separator />
        <FrameSection frame={frame} fonts={fonts} />
      </div>

      <Separator />
      <div className="space-y-3 p-4">
        <ExportSection photoCount={photoCount} filter={filter} />
        <div className="flex items-center justify-between text-xs text-muted-foreground">
          <span>{t('settings.footer')}</span>
          {appInfo && <Badge variant="outline">v{appInfo.appVersion}</Badge>}
        </div>
      </div>
    </aside>
  )
}
