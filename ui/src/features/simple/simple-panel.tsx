// SPDX-License-Identifier: GPL-3.0-only
import { CalendarDays, Crop, ShoppingBasket } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Label } from '@/components/ui/label'
import { Separator } from '@/components/ui/separator'
import { Switch } from '@/components/ui/switch'
import { SectionCard } from '@/components/layout/section-card'
import { commands, type CullFilter, type RenderSettings } from '@/ipc/bindings'
import { ALL_PHOTOS } from '@/features/culling/filter'
import { ExportSection } from '@/features/export/export-section'
import type { ExportApi } from '@/features/export/use-export'
import { BorderSwitch } from '@/features/settings/border-switch'
import { PrintSizePicker } from '@/features/settings/print-size-picker'
import { useIpcQuery } from '@/features/settings/use-ipc-query'
import { StampSizeField } from '@/features/stamp/stamp-size-field'

type SimplePanelProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
  exporter: ExportApi
  /** Photos in the print basket: what Export sends out. */
  basketCount: number
  revision: number
}

/** The print basket is what Simple mode exports (FR-048, FR-050). */
const BASKET: CullFilter = { ...ALL_PHOTOS, queue: 'print' }

/**
 * Simple mode's few choices (FR-048): print size, the 3 mm border, the date stamp and its size,
 * and Export, with the same controls as Professional so both stay in step.
 */
export function SimplePanel({ settings, onChange, exporter, basketCount, revision }: SimplePanelProps) {
  const { t } = useTranslation()
  const printSizes = useIpcQuery(commands.listPrintSizes)
  return (
    <aside className="flex w-72 shrink-0 flex-col border-l bg-sidebar text-sidebar-foreground">
      <div className="flex h-12 items-center px-4">
        <span className="text-sm font-semibold">{t('simple.title')}</span>
      </div>
      <Separator />
      <div className="flex-1 space-y-3 overflow-y-auto p-3">
        <SectionCard
          icon={<Crop />}
          title={
            <Label htmlFor="print-size" className="text-sm font-semibold">
              {t('settings.printSize')}
            </Label>
          }
        >
          <PrintSizePicker settings={settings} builtIn={printSizes} onChange={onChange} />
          <BorderSwitch settings={settings} onChange={onChange} />
        </SectionCard>
        <SectionCard
          icon={<CalendarDays />}
          title={
            <Label htmlFor="simple-date-stamp" className="text-sm font-semibold">
              {t('settings.dateStamp')}
            </Label>
          }
          action={
            <Switch
              id="simple-date-stamp"
              checked={settings?.stamp.enabled ?? false}
              onCheckedChange={(enabled) => onChange((s) => ({ ...s, stamp: { ...s.stamp, enabled } }))}
            />
          }
          open={settings?.stamp.enabled ?? false}
        >
          <StampSizeField
            stamp={settings?.stamp}
            onChange={(patch) => onChange((s) => ({ ...s, stamp: { ...s.stamp, ...patch } }))}
          />
        </SectionCard>
        <p className="flex gap-1.5 px-1 text-xs text-muted-foreground">
          <ShoppingBasket className="mt-0.5 size-3.5 shrink-0 text-primary" />
          {t('simple.exportHint', { count: basketCount })}
        </p>
      </div>
      <Separator />
      <div className="p-4">
        <ExportSection exporter={exporter} photoCount={basketCount} filter={BASKET} revision={revision} />
      </div>
    </aside>
  )
}
