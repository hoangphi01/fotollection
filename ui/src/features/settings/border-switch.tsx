// SPDX-License-Identifier: GPL-3.0-only
import { Frame } from 'lucide-react'
import { useId } from 'react'
import { useTranslation } from 'react-i18next'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { SwatchToggle } from '@/components/forms/swatch-toggle'
import type { BorderColor, RenderSettings } from '@/ipc/bindings'

const COLORS: { value: BorderColor; swatch: string }[] = [
  { value: 'white', swatch: 'bg-white' },
  { value: 'black', swatch: 'bg-neutral-950' },
]

type BorderSwitchProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
  /** The paper shows round the photos even without the border (a collage sheet's margins). */
  paper?: boolean
}

/**
 * The 3 mm border inside the print size and its colour, white or black, the same switch in
 * every mode (FR-067). On a collage sheet the colour is also the paper's round the photos.
 */
export function BorderSwitch({ settings, onChange, paper = false }: BorderSwitchProps) {
  const { t } = useTranslation()
  const id = useId()
  const border = settings?.border ?? false
  return (
    <div className="space-y-1.5">
      <div className="flex items-center justify-between">
        <Label htmlFor={id} className="flex items-center gap-2 text-sm font-normal">
          <Frame className="size-4 text-muted-foreground" />
          {t('settings.border')}
        </Label>
        <Switch id={id} checked={border} onCheckedChange={(next) => onChange((s) => ({ ...s, border: next }))} />
      </div>
      {(border || paper) && (
        <SwatchToggle
          ariaLabel={t('settings.borderColor')}
          value={settings?.borderColor ?? 'white'}
          options={COLORS.map((c) => ({ ...c, label: t(`settings.borderColors.${c.value}`) }))}
          onChange={(borderColor) => onChange((s) => ({ ...s, borderColor }))}
        />
      )}
      <p className="text-xs text-muted-foreground">{t(paper ? 'settings.borderHintPaper' : 'settings.borderHint')}</p>
    </div>
  )
}
