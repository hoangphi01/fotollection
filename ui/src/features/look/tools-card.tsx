// SPDX-License-Identifier: GPL-3.0-only
// @shown preview: in the preview only (STYLE_RULE §4, where each card shows, checked by check-lego).
import { Wand2 } from 'lucide-react'
import { useState, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { Field } from '@/components/forms/field'
import { SliderField } from '@/components/forms/slider-field'
import { Collapse } from '@/components/layout/collapse'
import { SectionCard } from '@/components/layout/section-card'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import type { LookSettings, Mono } from '@/ipc/bindings'
import { fullLook, hueSwatch, resetTools, toolsChanged, type FullAdjust, type LookChange } from './look-fields'
import { LookCardTitle } from './look-card-title'
import { ResetButton } from '@/components/forms/danger-button'

/** Whether Tools is unfolded, kept while the app runs; folded at first (FR-079). */
let remembered = false

const MONO: Mono[] = ['off', 'neutral', 'yellow', 'orange', 'red', 'green']

type ToolsCardProps = {
  look: LookSettings | undefined
  onChange: LookChange
  footer?: ReactNode
}

/**
 * Everything else (FR-079, Tools): dehaze (FR-115), fade, a tint for the shadows and one for
 * the highlights, black and white through a filter, then the effects: grain sized in
 * millimetres on the print and the vignette round the cropped photo (session 13). Folded at first, so a newcomer meets Faves first;
 * its title opens it, and the app remembers.
 */
export function ToolsCard({ look, onChange, footer }: ToolsCardProps) {
  const { t } = useTranslation()
  const [open, setOpen] = useState(remembered)
  const adjust = fullLook(look).adjust
  const changed = toolsChanged(adjust)
  const set = (patch: Partial<FullAdjust>, first: boolean) =>
    onChange((l) => ({ ...l, adjust: { ...l.adjust, ...patch } }), first)
  const tint = (key: 'shadowTint' | 'highlightTint') => {
    const value = adjust[key]
    return (
      <Field key={key}>
        <SliderField
          label={t(`look.${key}`)}
          hint={t(`look.hints.${key}`)}
          min={0}
          value={value.amount}
          format={(v) => `${v}`}
          onChange={(amount, first) => set({ [key]: { ...value, amount } }, first)}
        />
        <Collapse open={value.amount > 0}>
          <SliderField
            label={t('look.hue')}
            min={0}
            max={359}
            value={value.hue}
            format={(v) => (
              <>
                <span className="size-3 rounded-full border" style={{ background: hueSwatch(v) }} aria-hidden />
                {v}°
              </>
            )}
            onChange={(hue, first) => set({ [key]: { ...value, hue } }, first)}
          />
        </Collapse>
      </Field>
    )
  }
  const toned = adjust.shadowTint.amount > 0 || adjust.highlightTint.amount > 0
  return (
    <SectionCard
      icon={<Wand2 />}
      open={open}
      title={<LookCardTitle title={t('look.tools')} changed={changed} />}
      onToggle={() => {
        remembered = !open
        setOpen(!open)
      }}
      action={
        changed > 0 && (
          <ResetButton iconOnly label={t('look.reset')} onClick={() => onChange((l) => ({ ...l, adjust: resetTools(l.adjust) }), true)} />
        )
      }
      footer={footer}
    >
      {/* Not in the look's table: Rust draws each step of it (FR-115). */}
      <SliderField
        label={t('look.dehaze')}
        hint={t('look.hints.dehaze')}
        value={adjust.dehaze}
        gpu={false}
        onChange={(dehaze, first) => set({ dehaze }, first)}
      />
      <SliderField
        label={t('look.fade')}
        hint={t('look.hints.fade')}
        min={0}
        value={adjust.fade}
        format={(v) => `${v}`}
        onChange={(fade, first) => set({ fade }, first)}
      />
      {tint('shadowTint')}
      {tint('highlightTint')}
      <Collapse open={toned}>
        <SliderField
          label={t('look.balance')}
          hint={t('look.hints.balance')}
          value={adjust.balance}
          onChange={(balance, first) => set({ balance }, first)}
        />
      </Collapse>
      {/* Six filters do not fit one row of buttons: a select, as the stamp's corner. */}
      <Field label={t('look.mono')}>
        <Select value={adjust.mono} onValueChange={(mono) => set({ mono: mono as Mono }, true)}>
          <SelectTrigger className="w-full" aria-label={t('look.mono')} data-hint={t('look.hints.mono')}>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {MONO.map((m) => (
              <SelectItem key={m} value={m}>
                {t(`look.monoOptions.${m}`)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </Field>
      {/* The effects, drawn after the table on the print (FR-079, session 13): Rust draws
          each step, as for Dehaze; a preset's vignette shape and roughness are kept. */}
      <Field>
        <SliderField
          label={t('look.grain')}
          hint={t('look.hints.grain')}
          min={0}
          value={adjust.grain.amount}
          format={(v) => `${v}`}
          gpu={false}
          onChange={(amount, first) => set({ grain: { ...adjust.grain, amount } }, first)}
        />
        <Collapse open={adjust.grain.amount > 0}>
          <SliderField
            label={t('look.grainSize')}
            hint={t('look.hints.grainSize')}
            min={0}
            value={adjust.grain.size}
            format={(v) => `${v}`}
            gpu={false}
            onChange={(size, first) => set({ grain: { ...adjust.grain, size } }, first)}
          />
        </Collapse>
      </Field>
      <SliderField
        label={t('look.vignette')}
        hint={t('look.hints.vignette')}
        value={adjust.vignette}
        gpu={false}
        onChange={(vignette, first) => set({ vignette }, first)}
      />
    </SectionCard>
  )
}
