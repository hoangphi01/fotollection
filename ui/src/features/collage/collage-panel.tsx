// SPDX-License-Identifier: GPL-3.0-only
import { CalendarDays, Columns2, Crop, Scissors } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Label } from '@/components/ui/label'
import { Separator } from '@/components/ui/separator'
import { Switch } from '@/components/ui/switch'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { SectionCard } from '@/components/layout/section-card'
import { commands, type CellInfo, type CellRatio, type CullFilter, type RenderSettings } from '@/ipc/bindings'
import { ExportSection } from '@/features/export/export-section'
import type { ExportApi } from '@/features/export/use-export'
import { BorderSwitch } from '@/features/settings/border-switch'
import { PrintSizePicker } from '@/features/settings/print-size-picker'
import { useIpcQuery } from '@/features/settings/use-ipc-query'
import { StampStyleFields } from '@/features/stamp/stamp-style-fields'
import { formatNumber } from '@/lib/format'
import { CollageSizeFields } from './collage-size-fields'

const RATIOS: CellRatio[] = ['photo', 'sheet', 'fourThree', 'square']

type CollagePanelProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
  exporter: ExportApi
  filter: CullFilter
  photoCount: number
  sheetCount: number
  /** The photo chosen on the selected sheet, for the size fields and the advice. */
  cell: CellInfo | undefined
  revision: number
}

/**
 * Collage mode's panel (FR-035, FR-048), in the shared order: print size, then how two
 * photos share it, the date stamp, and Export, which writes one file per sheet.
 */
export function CollagePanel({
  settings,
  onChange,
  exporter,
  filter,
  photoCount,
  sheetCount,
  cell,
  revision,
}: CollagePanelProps) {
  const { t } = useTranslation()
  const printSizes = useIpcQuery(commands.listPrintSizes)
  const collage = settings?.collage
  const setCollage = (patch: Partial<RenderSettings['collage']>) =>
    onChange((s) => ({ ...s, collage: { ...s.collage, ...patch } }))
  const stamp = settings?.stamp
  const cm = (mm: number) => formatNumber(mm / 10, 1)

  return (
    <aside className="flex w-72 shrink-0 flex-col border-l bg-sidebar text-sidebar-foreground">
      <div className="flex h-12 items-center px-4">
        <span className="text-sm font-semibold">{t('collage.title')}</span>
      </div>
      <Separator />
      <div className="flex-1 space-y-3 overflow-y-auto p-3">
        <SectionCard
          icon={<Crop />}
          title={
            <Label htmlFor="print-size" className="text-sm font-semibold">
              {t('collage.sheetSize')}
            </Label>
          }
        >
          <PrintSizePicker settings={settings} builtIn={printSizes} onChange={onChange} />
          <p className="text-xs text-muted-foreground">{t('collage.sheetSizeHint')}</p>
          <BorderSwitch settings={settings} onChange={onChange} />
        </SectionCard>

        <SectionCard icon={<Columns2 />} title={<span className="text-sm font-semibold">{t('collage.cells')}</span>}>
          <div className="space-y-1.5">
            <Label className="text-sm font-normal">{t('collage.ratio')}</Label>
            <ToggleGroup
              type="single"
              variant="outline"
              size="sm"
              className="w-full"
              value={collage?.ratio ?? 'photo'}
              onValueChange={(v) => v && setCollage({ ratio: v as CellRatio })}
              aria-label={t('collage.ratio')}
            >
              {RATIOS.map((r) => (
                <ToggleGroupItem key={r} value={r} className="flex-1 px-1 text-xs" data-hint={t(`collage.ratioHints.${r}`)}>
                  {t(`collage.ratios.${r}`)}
                </ToggleGroupItem>
              ))}
            </ToggleGroup>
          </div>
          <CollageSizeFields settings={settings} onChange={onChange} cell={cell} />
          <div className="space-y-1.5">
            <div className="flex items-center justify-between">
              <Label htmlFor="cut-line" className="flex items-center gap-2 text-sm font-normal">
                <Scissors className="size-4 text-muted-foreground" />
                {t('collage.cutLine')}
              </Label>
              <Switch
                id="cut-line"
                checked={collage?.cutLine ?? true}
                onCheckedChange={(cutLine) => setCollage({ cutLine })}
              />
            </div>
            <p className="text-xs text-muted-foreground">
              {t(collage?.cutLine === false ? 'collage.togetherHint' : 'collage.cutLineHint')}
            </p>
          </div>
          {cell && (
            <p className="rounded-md bg-muted/60 px-2.5 py-2 text-xs text-muted-foreground">
              {t(settings?.border ? 'collage.adviceBorder' : 'collage.advice', {
                area: `${cm(cell.areaWidthMm)} × ${cm(cell.areaHeightMm)} cm`,
                photo: `${formatNumber((cell.photoWidthMm ?? 0) / 10, 1)} × ${formatNumber((cell.photoHeightMm ?? 0) / 10, 1)} cm`,
                dpi: cell.dpi,
                quality: t(`dpi.${cell.quality}`),
              })}
            </p>
          )}
        </SectionCard>

        <SectionCard
          icon={<CalendarDays />}
          title={
            <Label htmlFor="collage-date-stamp" className="text-sm font-semibold">
              {t('settings.dateStamp')}
            </Label>
          }
          action={
            <Switch
              id="collage-date-stamp"
              checked={stamp?.enabled ?? false}
              onCheckedChange={(enabled) => onChange((s) => ({ ...s, stamp: { ...s.stamp, enabled } }))}
            />
          }
          open={stamp?.enabled ?? false}
        >
          <StampStyleFields
            stamp={stamp}
            onChange={(patch) => onChange((s) => ({ ...s, stamp: { ...s.stamp, ...patch } }))}
          />
        </SectionCard>
      </div>
      <Separator />
      <div className="p-4">
        <ExportSection
          exporter={exporter}
          photoCount={photoCount}
          filter={filter}
          revision={revision}
          sheets={sheetCount}
        />
      </div>
    </aside>
  )
}
