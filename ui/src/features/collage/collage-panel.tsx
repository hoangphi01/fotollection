// SPDX-License-Identifier: GPL-3.0-only
import { CalendarDays, Crop, TriangleAlert } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Label } from '@/components/ui/label'
import { Separator } from '@/components/ui/separator'
import { Switch } from '@/components/ui/switch'
import { SectionCard } from '@/components/layout/section-card'
import { commands, type CellInfo, type CullFilter, type RenderSettings } from '@/ipc/bindings'
import { ExportSection } from '@/features/export/export-section'
import type { ExportApi } from '@/features/export/use-export'
import { BorderSwitch } from '@/features/settings/border-switch'
import { PrintSizePicker } from '@/features/settings/print-size-picker'
import { useIpcQuery } from '@/features/settings/use-ipc-query'
import { StampStyleFields } from '@/features/stamp/stamp-style-fields'
import { CollageCellsCard } from './collage-cells-card'

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
  const stamp = settings?.stamp

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
          <PrintSizePicker settings={settings} builtIn={printSizes} onChange={onChange} allowOriginal={false} />
          {/* Original has no size until a photo gives it one: sheets use 10 × 15 (FR-035). */}
          {settings?.printSize === 'original' && (
            <p className="flex gap-1.5 rounded-md bg-muted/60 px-2.5 py-2 text-xs text-muted-foreground">
              <TriangleAlert className="mt-0.5 size-3.5 shrink-0 text-primary" />
              {t('collage.originalNote')}
            </p>
          )}
          <p className="text-xs text-muted-foreground">{t('collage.sheetSizeHint')}</p>
          <BorderSwitch settings={settings} onChange={onChange} />
        </SectionCard>

        <CollageCellsCard settings={settings} onChange={onChange} cell={cell} />

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
