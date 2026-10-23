// SPDX-License-Identifier: GPL-3.0-only
import { CalendarDays, Crop, MapPin } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
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
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { ExportSection } from '@/features/export/export-section'
import { FrameSection } from '@/features/frame/frame-section'
import type { FrameConfigApi } from '@/features/frame/use-frame-config'
import {
  commands,
  type Corner,
  type CullFilter,
  type DateFormat,
  type RenderSettings,
  type StampColor,
} from '@/ipc/bindings'
import { FontSelect } from '@/features/fonts/font-select'
import type { FontsApi } from '@/features/fonts/use-fonts'
import { PrintSizePicker } from './print-size-picker'
import { useIpcQuery } from './use-ipc-query'

type SettingsPanelProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
  frame: FrameConfigApi
  fonts: FontsApi
  photoCount: number
  filter: CullFilter
}

const FORMATS: { value: DateFormat; label: string }[] = [
  { value: 'iso', label: '2026-07-14' },
  { value: 'dmy', label: '14.07.2026' },
  { value: 'retro', label: "'26 07 14" },
]

const CORNERS: { value: Corner; label: string }[] = [
  { value: 'bottomRight', label: 'Bottom right' },
  { value: 'bottomLeft', label: 'Bottom left' },
  { value: 'topRight', label: 'Top right' },
  { value: 'topLeft', label: 'Top left' },
]

const COLORS: { value: StampColor; label: string; swatch: string }[] = [
  { value: 'white', label: 'White', swatch: 'bg-white' },
  { value: 'retroOrange', label: 'Retro orange', swatch: 'bg-[#ff8a1f]' },
]

export function SettingsPanel({
  settings,
  onChange,
  frame,
  fonts,
  photoCount,
  filter,
}: SettingsPanelProps) {
  const printSizes = useIpcQuery(commands.listPrintSizes)
  const appInfo = useIpcQuery(commands.appInfo)
  const stamp = settings?.stamp
  const setStamp = (patch: Partial<RenderSettings['stamp']>) =>
    onChange((s) => ({ ...s, stamp: { ...s.stamp, ...patch } }))

  return (
    <aside className="flex w-72 shrink-0 flex-col border-l">
      <div className="flex h-12 items-center px-4">
        <span className="text-sm font-semibold">Print settings</span>
      </div>
      <Separator />

      <div className="flex-1 space-y-6 overflow-y-auto p-4">
        <section className="space-y-2">
          <Label htmlFor="print-size" className="flex items-center gap-2">
            <Crop className="size-4 text-muted-foreground" />
            Print size
          </Label>
          <PrintSizePicker settings={settings} builtIn={printSizes} onChange={onChange} />
          <p className="text-xs text-muted-foreground">
            Centred by default. Adjust per photo with Crop in the preview.
          </p>
        </section>

        <section className="space-y-3">
          <div className="flex items-center justify-between">
            <Label htmlFor="date-stamp" className="flex items-center gap-2">
              <CalendarDays className="size-4 text-muted-foreground" />
              Date stamp
            </Label>
            <Switch
              id="date-stamp"
              checked={stamp?.enabled ?? false}
              onCheckedChange={(enabled) => setStamp({ enabled })}
            />
          </div>

          <div className={stamp?.enabled ? 'space-y-3' : 'pointer-events-none space-y-3 opacity-50'}>
            <FontSelect
              fonts={fonts}
              label="Date stamp font"
              value={stamp?.font}
              onChange={(font) => setStamp({ font })}
            />
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
              <SelectTrigger className="w-full" aria-label="Stamp position">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {CORNERS.map((c) => (
                  <SelectItem key={c.value} value={c.value}>
                    {c.label}
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
                  {c.label}
                </ToggleGroupItem>
              ))}
            </ToggleGroup>
          </div>

          <Tooltip>
            <TooltipTrigger asChild>
              <div className="flex items-center justify-between opacity-50">
                <Label className="flex items-center gap-2">
                  <MapPin className="size-4 text-muted-foreground" />
                  Location
                </Label>
                <Badge variant="outline">Soon</Badge>
              </div>
            </TooltipTrigger>
            <TooltipContent>Offline place names arrive in the next slice (FR-005)</TooltipContent>
          </Tooltip>
        </section>

        <Separator />
        <FrameSection frame={frame} fonts={fonts} />
      </div>

      <Separator />
      <div className="space-y-3 p-4">
        <ExportSection photoCount={photoCount} filter={filter} />
        <div className="flex items-center justify-between text-xs text-muted-foreground">
          <span>Offline · GPS stripped</span>
          {appInfo && <Badge variant="outline">v{appInfo.appVersion}</Badge>}
        </div>
      </div>
    </aside>
  )
}
