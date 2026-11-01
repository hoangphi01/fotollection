// SPDX-License-Identifier: GPL-3.0-only
import { Columns2, Maximize, Scissors } from 'lucide-react'
import { useId } from 'react'
import { useTranslation } from 'react-i18next'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { Collapse } from '@/components/layout/collapse'
import { SectionCard } from '@/components/layout/section-card'
import type { CellInfo, CellRatio, RenderSettings } from '@/ipc/bindings'
import { formatNumber } from '@/lib/format'
import { dimsLabel, displayUnit } from '@/lib/print-size'
import { CollageSizeFields } from './collage-size-fields'

const RATIOS: CellRatio[] = ['photo', 'sheet', 'fourThree', 'square']

type CollageCellsCardProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
  /** The photo chosen on the sheet, for the size fields and the note. */
  cell: CellInfo | undefined
}

/**
 * How two photos share a sheet (FR-035): ratio, size, cut line and the size and quality
 * note. The same card in the Collage panel and the sheet preview (STYLE_RULE §4).
 */
export function CollageCellsCard({ settings, onChange, cell }: CollageCellsCardProps) {
  const { t } = useTranslation()
  const collage = settings?.collage
  const setCollage = (patch: Partial<RenderSettings['collage']>) =>
    onChange((s) => ({ ...s, collage: { ...s.collage, ...patch } }))
  const unit = displayUnit(settings)
  // Mounted twice (panel and preview), so the switches' ids must be unique.
  const cutLineId = useId()
  const fillId = useId()
  const fill = collage?.fill ?? false

  return (
    <SectionCard icon={<Columns2 />} title={<span className="text-sm font-semibold">{t('collage.cells')}</span>}>
      {/* No white: each photo fills its half, and its crop chooses what it keeps (FR-035). */}
      <div className="space-y-1.5">
        <div className="flex items-center justify-between">
          <Label htmlFor={fillId} className="flex items-center gap-2 text-sm font-normal">
            <Maximize className="size-4 text-muted-foreground" />
            {t('collage.fill')}
          </Label>
          <Switch id={fillId} checked={fill} onCheckedChange={(on) => setCollage({ fill: on })} />
        </div>
        <p className="text-xs text-muted-foreground">{t(fill ? 'collage.fillHint' : 'collage.marginHint')}</p>
      </div>
      <Collapse open={!fill} className="space-y-3">
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
      </Collapse>
      <div className="space-y-1.5">
        <div className="flex items-center justify-between">
          <Label htmlFor={cutLineId} className="flex items-center gap-2 text-sm font-normal">
            <Scissors className="size-4 text-muted-foreground" />
            {t('collage.cutLine')}
          </Label>
          <Switch
            id={cutLineId}
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
            area: dimsLabel(cell.areaWidthMm, cell.areaHeightMm, unit),
            photo: dimsLabel(cell.photoWidthMm ?? cell.areaWidthMm, cell.photoHeightMm ?? cell.areaHeightMm, unit),
            dpi: formatNumber(cell.dpi),
            quality: t(`dpi.${cell.quality}`),
          })}
        </p>
      )}
    </SectionCard>
  )
}
