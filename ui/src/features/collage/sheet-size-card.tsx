// SPDX-License-Identifier: GPL-3.0-only
// @shown both: in the gallery panels and in the preview (STYLE_RULE §4, where each card shows, checked by check-lego).
import { Crop, Grid2x2, TriangleAlert } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Label } from '@/components/ui/label'
import { PanelNote } from '@/components/layout/panel-note'
import { SectionCard } from '@/components/layout/section-card'
import { Field, ValueRow } from '@/components/forms/field'
import { commands, type RenderSettings, type SheetInfo } from '@/ipc/bindings'
import { formatNumber } from '@/lib/format'
import { BorderSwitch } from '@/features/settings/border-switch'
import { PrintSizePicker } from '@/features/settings/print-size-picker'
import { useIpcQuery } from '@/features/settings/use-ipc-query'

type SheetSizeCardProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
  /** The chosen sheet, for its pixels and how sharp it prints (FR-087). */
  info?: SheetInfo
}

/**
 * The size of a collage sheet (FR-035): a fixed print size (Original has none), the 3 mm
 * border and the paper's colour, and a reminder that sheets are always prints. The same card in the Collage panel
 * and the sheet preview.
 */
export function SheetSizeCard({ settings, onChange, info }: SheetSizeCardProps) {
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
      {/* The two photos make one picture: its pixels, at the less sharp photo's density (FR-087). */}
      {info && info.widthPx > 0 && (
        <Field hint={t('collage.sheetPixelsHint')}>
          <ValueRow label={t('collage.sheetPixels')} icon={<Grid2x2 />}>
            {t('collage.sheetPixelsValue', {
              w: formatNumber(info.widthPx),
              h: formatNumber(info.heightPx),
              mp: formatNumber(Math.round((info.widthPx * info.heightPx) / 1e6)),
            })}
          </ValueRow>
          <ValueRow label={t('collage.sheetSharpness')}>
            {t('collage.sheetSharpnessValue', { dpi: formatNumber(info.dpi), quality: t(`dpi.${info.quality}`) })}
          </ValueRow>
        </Field>
      )}
      <BorderSwitch settings={settings} onChange={onChange} paper={!settings?.collage?.fill} />
    </SectionCard>
  )
}
