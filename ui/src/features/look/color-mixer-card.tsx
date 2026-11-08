// SPDX-License-Identifier: GPL-3.0-only
// @shown preview: in the preview only (STYLE_RULE §4, where each card shows, checked by check-lego).
import { Palette } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { ResetButton } from '@/components/forms/danger-button'
import { FieldLabel } from '@/components/forms/field'
import { SegmentedToggle } from '@/components/forms/segmented-toggle'
import { SliderField } from '@/components/forms/slider-field'
import { SectionCard } from '@/components/layout/section-card'
import type { LookSettings } from '@/ipc/bindings'
import { LookCardTitle } from './look-card-title'
import { MIXER_BANDS, fullLook, mixerChanged, mixerTrack, type LookChange, type MixerPart } from './look-fields'

type Shown = MixerPart | 'all'

/** The part shown, kept while the app runs, in every preview alike. */
let lastShown: Shown = 'hue'

const PARTS: readonly MixerPart[] = ['hue', 'sat', 'lum']

/**
 * The colour mixer (FR-079), as Lightroom's HSL panel (owner, 2026-11-08): *Hue*,
 * *Saturation*, *Luminance* or *All* in a row of buttons, and for the part picked a slider for
 * each of the eight colours, its track drawn in the colours it moves between. Rust lays them on
 * with the look's table (`colour_ops`, the function the 32-bit path will call), so the GPU
 * preview, the thumbnails and the export follow. A Lightroom preset's mixer lands here,
 * adjustable. Pro mode and collage sheets.
 */
export function ColorMixerCard({ look, onChange }: { look: LookSettings | undefined; onChange: LookChange }) {
  const { t } = useTranslation()
  const [shown, setShown] = useState<Shown>(lastShown)
  const adjust = fullLook(look).adjust
  const changed = mixerChanged(adjust)
  const set = (part: MixerPart, at: number, value: number, first: boolean) =>
    onChange((l) => {
      const row = [...l.adjust.mixer[part]] as typeof l.adjust.mixer.hue
      row[at] = value
      return { ...l, adjust: { ...l.adjust, mixer: { ...l.adjust.mixer, [part]: row } } }
    }, first)
  const sliders = (part: MixerPart) =>
    MIXER_BANDS.map((band, at) => (
      <SliderField
        key={`${part}-${band}`}
        label={t(`look.mixer.band.${band}`)}
        ariaLabel={`${t(`look.mixer.${part}`)} · ${t(`look.mixer.band.${band}`)}`}
        hint={t(`look.mixer.hints.${part}`)}
        track={mixerTrack(part, at)}
        value={adjust.mixer[part][at] ?? 0}
        onChange={(v, first) => set(part, at, v, first)}
      />
    ))
  return (
    <SectionCard
      icon={<Palette />}
      title={<LookCardTitle title={t('look.mixer.title')} changed={changed} />}
      action={
        changed > 0 && (
          <ResetButton
            iconOnly
            label={t('look.reset')}
            onClick={() => onChange((l) => ({ ...l, adjust: { ...l.adjust, mixer: fullLook(undefined).adjust.mixer } }), true)}
          />
        )
      }
    >
      <SegmentedToggle<Shown>
        dense
        value={shown}
        ariaLabel={t('look.mixer.part')}
        onChange={(s) => {
          lastShown = s
          setShown(s)
        }}
        // Short words in a quarter of the card, the whole name as the hint (STYLE_RULE §4, a fixed
        // box measured in Vietnamese: "Saturation" took 63 px of 49).
        options={[
          ...PARTS.map((p) => ({ value: p, label: t(`look.mixer.short.${p}`), hint: t(`look.mixer.${p}`) })),
          { value: 'all' as const, label: t('look.mixer.all') },
        ]}
      />
      {shown === 'all'
        ? PARTS.flatMap((part) => [<FieldLabel key={part}>{t(`look.mixer.${part}`)}</FieldLabel>, ...sliders(part)])
        : sliders(shown)}
    </SectionCard>
  )
}
