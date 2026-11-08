// SPDX-License-Identifier: GPL-3.0-only
// @shown preview: in the preview only (STYLE_RULE §4, where each card shows, checked by check-lego).
import { Palette } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { ResetButton } from '@/components/forms/danger-button'
import { SliderField } from '@/components/forms/slider-field'
import { SwatchToggle } from '@/components/forms/swatch-toggle'
import { SectionCard } from '@/components/layout/section-card'
import type { LookSettings } from '@/ipc/bindings'
import { LookCardTitle } from './look-card-title'
import { MIXER_BANDS, fullLook, mixerChanged, type LookChange } from './look-fields'

type Band = (typeof MIXER_BANDS)[number]
type Part = 'hue' | 'sat' | 'lum'

/** The hue picked, kept while the app runs, in every preview alike. */
let lastBand: Band = 'red'

/** Each hue's dot: the colour it changes, a print colour shown as it is (STYLE_RULE §4). */
const DOTS: Record<Band, string> = {
  red: 'bg-red-500', // swatch
  orange: 'bg-orange-500', // swatch
  yellow: 'bg-yellow-400', // swatch
  green: 'bg-green-500', // swatch
  aqua: 'bg-cyan-400', // swatch
  blue: 'bg-blue-500', // swatch
  purple: 'bg-violet-500', // swatch
  magenta: 'bg-fuchsia-500', // swatch
}

const PARTS: readonly Part[] = ['hue', 'sat', 'lum']

/**
 * The colour mixer (FR-079, as Lightroom's Color Mixer, one colour at a time): the eight hues
 * as dots, and for the one picked its hue, saturation and lightness. Rust lays them on with the
 * look's table (`colour_ops`, the function the 32-bit path will call), so the GPU preview, the
 * thumbnails and the export follow. A Lightroom preset's mixer lands here, adjustable. Pro mode
 * and collage sheets.
 */
export function ColorMixerCard({ look, onChange }: { look: LookSettings | undefined; onChange: LookChange }) {
  const { t } = useTranslation()
  const [band, setBand] = useState<Band>(lastBand)
  const adjust = fullLook(look).adjust
  const changed = mixerChanged(adjust)
  const at = MIXER_BANDS.indexOf(band)
  const moved = (b: Band) => PARTS.some((p) => adjust.mixer[p][MIXER_BANDS.indexOf(b)] !== 0)
  const set = (part: Part, value: number, first: boolean) =>
    onChange((l) => {
      const row = [...l.adjust.mixer[part]] as typeof l.adjust.mixer.hue
      row[at] = value
      return { ...l, adjust: { ...l.adjust, mixer: { ...l.adjust.mixer, [part]: row } } }
    }, first)
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
      <SwatchToggle<Band>
        dense
        value={band}
        ariaLabel={t('look.mixer.colour')}
        onChange={(b) => {
          lastBand = b
          setBand(b)
        }}
        options={MIXER_BANDS.map((b) => ({
          value: b,
          // A moved hue says so in its name (its hint and for screen readers).
          label: moved(b) ? `${t(`look.mixer.band.${b}`)} · ${t('look.mixer.moved')}` : t(`look.mixer.band.${b}`),
          swatch: DOTS[b],
        }))}
      />
      {PARTS.map((part) => (
        <SliderField
          key={part}
          label={`${t(`look.mixer.${part}`)} · ${t(`look.mixer.band.${band}`)}`}
          hint={t(`look.mixer.hints.${part}`)}
          value={adjust.mixer[part][at] ?? 0}
          onChange={(v, first) => set(part, v, first)}
        />
      ))}
    </SectionCard>
  )
}
