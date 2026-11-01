// SPDX-License-Identifier: GPL-3.0-only
import { Crop, ShoppingBasket } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { PanelFooter } from '@/features/about/panel-footer'
import { Label } from '@/components/ui/label'
import { SectionCard } from '@/components/layout/section-card'
import { SidePanel } from '@/components/layout/side-panel'
import { commands, type CullFilter, type RenderSettings } from '@/ipc/bindings'
import { ALL_PHOTOS } from '@/features/culling/filter'
import { ExportSection } from '@/features/export/export-section'
import { ExportTargetPicker } from '@/features/export/export-target-picker'
import type { ExportApi } from '@/features/export/use-export'
import { BorderSwitch } from '@/features/settings/border-switch'
import { PrintSizePicker } from '@/features/settings/print-size-picker'
import { useIpcQuery } from '@/features/settings/use-ipc-query'
import { DateStampCard } from '@/features/stamp/date-stamp-card'

type SimplePanelProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
  exporter: ExportApi
  /** Photos in the print basket: what Export sends out. */
  basketCount: number
  /** Photos with their own stamp from the preview (FR-044). */
  customStamps: number
  revision: number
}

/** The print basket is what Simple mode exports (FR-048, FR-050). */
const BASKET: CullFilter = { ...ALL_PHOTOS, queue: 'print' }

/**
 * Simple mode's few choices (FR-048): prints or a share size, the print size and the 3 mm
 * border, the date stamp, and Export, with the same controls as Pro so both
 * stay in step.
 */
export function SimplePanel({ settings, onChange, exporter, basketCount, customStamps, revision }: SimplePanelProps) {
  const { t } = useTranslation()
  const printSizes = useIpcQuery(commands.listPrintSizes)
  const share = settings?.exportTarget?.kind === 'share'
  return (
    <SidePanel
      title={t('simple.title')}
      footer={
        <>
          <ExportSection exporter={exporter} photoCount={basketCount} filter={BASKET} revision={revision} />
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
        {/* Print or share, as in Pro (FR-053). */}
        <ExportTargetPicker settings={settings} onChange={onChange} />
        {!share && (
          <>
            <PrintSizePicker settings={settings} builtIn={printSizes} onChange={onChange} />
            <BorderSwitch settings={settings} onChange={onChange} />
          </>
        )}
      </SectionCard>
      <DateStampCard settings={settings} onChange={onChange} customStamps={customStamps} />
      <p className="flex gap-1.5 px-1 text-xs text-muted-foreground">
        <ShoppingBasket className="mt-0.5 size-3.5 shrink-0 text-primary" />
        {t('simple.exportHint', { count: basketCount })}
      </p>
    </SidePanel>
  )
}
