// SPDX-License-Identifier: GPL-3.0-only
import { Columns2, Maximize, Ruler, Scissors } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Field } from '@/components/forms/field'
import { SegmentedToggle } from '@/components/forms/segmented-toggle'
import { SwitchField } from '@/components/forms/switch-field'
import { PanelNote } from '@/components/layout/panel-note'
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
  const fill = collage?.fill ?? false

  return (
    <SectionCard icon={<Columns2 />} title={t('collage.cells')}>
      {/* No white: each photo fills its half, and its crop chooses what it keeps (FR-035). */}
      <SwitchField
        icon={<Maximize />}
        label={t('collage.fill')}
        checked={fill}
        onChange={(on) => setCollage({ fill: on })}
        hint={t(fill ? 'collage.fillHint' : 'collage.marginHint')}
      />
      <Collapse open={!fill} stack="card">
        <Field label={t('collage.ratio')}>
          <SegmentedToggle
            dense
            value={collage?.ratio ?? 'photo'}
            options={RATIOS.map((r) => ({
              value: r,
              label: t(`collage.ratios.${r}`),
              hint: t(`collage.ratioHints.${r}`),
            }))}
            onChange={(ratio) => setCollage({ ratio })}
            ariaLabel={t('collage.ratio')}
          />
        </Field>
        <CollageSizeFields settings={settings} onChange={onChange} cell={cell} />
      </Collapse>
      {/* Filled halves meet at an edge, and a postcard is not cut: no cut line there. */}
      <Collapse open={!fill && !collage?.postcard?.enabled}>
        <SwitchField
          icon={<Scissors />}
          label={t('collage.cutLine')}
          checked={collage?.cutLine ?? true}
          onChange={(cutLine) => setCollage({ cutLine })}
          hint={t(collage?.cutLine === false ? 'collage.togetherHint' : 'collage.cutLineHint')}
        />
      </Collapse>
      {cell && (
        <PanelNote icon={<Ruler />}>
          {t(settings?.border ? 'collage.adviceBorder' : 'collage.advice', {
            area: dimsLabel(cell.areaWidthMm, cell.areaHeightMm, unit),
            photo: dimsLabel(cell.photoWidthMm ?? cell.areaWidthMm, cell.photoHeightMm ?? cell.areaHeightMm, unit),
            dpi: formatNumber(cell.dpi),
            quality: t(`dpi.${cell.quality}`),
          })}
        </PanelNote>
      )}
    </SectionCard>
  )
}
