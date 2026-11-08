// SPDX-License-Identifier: GPL-3.0-only
// @shown preview: in the preview only (STYLE_RULE §4, where each card shows, checked by check-lego).
import { Palette, TriangleAlert } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { ResetButton } from '@/components/forms/danger-button'
import { ValueRow } from '@/components/forms/field'
import { SliderField } from '@/components/forms/slider-field'
import { Collapse } from '@/components/layout/collapse'
import { PanelNote } from '@/components/layout/panel-note'
import { SectionCard } from '@/components/layout/section-card'
import type { LookSettings } from '@/ipc/bindings'
import { NO_LOOK, fullLook, type LookChange } from './look-fields'
import { lookName, useLooks } from './use-looks'

type LooksCardProps = {
  /** Simple mode: with the original, the card is its title alone (owner, 2026-11-07). */
  simple?: boolean
  /** Pro mode: the LUT's strength and the V-Log note show here only (owner, 2026-11-07). */
  pro?: boolean
  look: LookSettings | undefined
  onChange: LookChange
}

const percent = (v: number) => `${v} %`

/**
 * The first level of the colour cards (FR-079, "drive first"), beside the strip of looks under
 * the photo and *Presets & LUTs* on the left: once the photo has a colour, the amount of the
 * whole look and of its LUT, the V-Log note, and ↺ for the original. A pick copies the look's
 * values into the photo, so the sliders below show them and a later change of the library
 * changes no photo.
 */
export function LooksCard({ look, onChange, simple = false, pro = false }: LooksCardProps) {
  const { t } = useTranslation()
  const { looks } = useLooks()
  const current = fullLook(look)
  // Coloured: a look or a slider changes the photo (Original changes nothing). Only then is
  // there an amount to set and a reset to show, in every mode (owner, 2026-11-07); the reset
  // gives the photo (or the whole sheet, none picked) the original back, one undo step.
  const { name: _name, base: _base, amount: _amount, ...colours } = current
  const { name: _n, base: _b, amount: _a, ...none } = NO_LOOK
  const coloured = JSON.stringify(colours) !== JSON.stringify(none)
  const chosenEntry = looks.find((e) => e.id === current.base)
  // The look in use, always the card's first line (owner, 2026-11-08: "say the original is in
  // use", "the look's name above, the amount below"): the library's name, else the look's own,
  // else *Custom* for sliders alone, *Original* when nothing is set.
  const original = looks.find((e) => e.id === 'builtin:original')
  const inUse = coloured
    ? chosenEntry
      ? lookName(chosenEntry)
      : (current.name ?? t('look.custom'))
    : original
      ? lookName(original)
      : t('look.original')
  return (
    <SectionCard
      icon={<Palette />}
      title={t('look.looks')}
      // Simple mode has nothing but the amount: with the original, the card is its title alone.
      open={coloured || !simple}
      action={coloured && <ResetButton iconOnly label={t('look.resetLook')} onClick={() => onChange(() => NO_LOOK, true)} />}
    >
      <ValueRow label={t('look.inUse')}>
        <span className="truncate font-medium" data-hint={inUse}>
          {inUse}
        </span>
      </ValueRow>
      <Collapse open={coloured} stack="card">
      <SliderField
        label={t('look.amount')}
        hint={t('look.hints.amount')}
        value={current.amount}
        min={0}
        max={100}
        reset={100}
        format={percent}
        onChange={(v, first) => onChange((l) => ({ ...l, amount: v }), first)}
      />
      </Collapse>
      {/* A look with a LUT (FR-070): how much of the LUT, under the sliders' amount. */}
      <Collapse open={pro && current.lut !== null} stack="card">
      <SliderField
        label={t('look.lutStrength')}
        hint={t('look.hints.lutStrength')}
        value={current.lutStrength}
        min={0}
        max={100}
        reset={100}
        format={percent}
        onChange={(v, first) => onChange((l) => ({ ...l, lutStrength: v }), first)}
      />
      </Collapse>
      {pro && chosenEntry?.vlog && <PanelNote icon={<TriangleAlert />}>{t('look.vlogWarning')}</PanelNote>}
    </SectionCard>
  )
}
