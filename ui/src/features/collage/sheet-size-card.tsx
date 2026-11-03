// SPDX-License-Identifier: GPL-3.0-only
import { Crop, TriangleAlert } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Label } from '@/components/ui/label'
import { PanelNote } from '@/components/layout/panel-note'
import { SectionCard } from '@/components/layout/section-card'
import { commands, type RenderSettings } from '@/ipc/bindings'
import { BorderSwitch } from '@/features/settings/border-switch'
import { PrintSizePicker } from '@/features/settings/print-size-picker'
import { useIpcQuery } from '@/features/settings/use-ipc-query'

type SheetSizeCardProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
}

/**
 * The size of a collage sheet (FR-035): a fixed print size (Original has none), the 3 mm
 * border and the paper's colour, and a reminder that sheets are always prints. The same card in the Collage panel
 * and the sheet preview.
 */
export function SheetSizeCard({ settings, onChange }: SheetSizeCardProps) {
  const { t } = useTranslation()
  const printSizes = useIpcQuery(commands.listPrintSizes)
  return (
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
        <PanelNote icon={<TriangleAlert />}>{t('collage.originalNote')}</PanelNote>
      )}
      <p className="text-xs text-muted-foreground">
        {t(settings?.collage?.postcard?.enabled ? 'postcard.sheetSizeHint' : 'collage.sheetSizeHint')}
      </p>
      <BorderSwitch settings={settings} onChange={onChange} paper={!settings?.collage?.fill} />
    </SectionCard>
  )
}
