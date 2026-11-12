// SPDX-License-Identifier: GPL-3.0-only
import { Frame } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { SwatchToggle } from '@/components/forms/swatch-toggle'
import { SwitchField } from '@/components/forms/switch-field'
import type { BorderColor, RenderSettings } from '@/ipc/bindings'

const COLORS: { value: BorderColor; swatch: string }[] = [
  { value: 'white', swatch: 'bg-white' },
  { value: 'black', swatch: 'bg-black' },
]

type BorderSwitchProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
  /** The paper shows round the photos even without the border (a collage sheet's margins). */
  paper?: boolean
  /** On a postcard: white only, and its own short hint (owner, 2026-11-12: "a black postcard?"). */
  postcard?: boolean
}

/**
 * The 3 mm border inside the print size and its colour, white or black, the same switch in
 * every mode (FR-067). On a collage sheet the colour is also the paper's round the photos.
 */
export function BorderSwitch({ settings, onChange, paper = false, postcard = false }: BorderSwitchProps) {
  const { t } = useTranslation()
  const border = settings?.border ?? false
  return (
    <SwitchField
      icon={<Frame />}
      label={t('settings.border')}
      checked={border}
      onChange={(next) => onChange((s) => ({ ...s, border: next }))}
      alwaysOpen={paper && !postcard}
      hint={t(postcard ? 'settings.borderHintPostcard' : paper ? 'settings.borderHintPaper' : 'settings.borderHint')}
    >
      {/* A postcard is white paper: Rust prints it so whatever is chosen elsewhere. */}
      {!postcard && <SwatchToggle
        ariaLabel={t('settings.borderColor')}
        value={settings?.borderColor ?? 'white'}
        options={COLORS.map((c) => ({ ...c, label: t(`settings.borderColors.${c.value}`) }))}
        onChange={(borderColor) => onChange((s) => ({ ...s, borderColor }))}
      />}
    </SwitchField>
  )
}
