// SPDX-License-Identifier: GPL-3.0-only
import { Frame } from 'lucide-react'
import { useId } from 'react'
import { useTranslation } from 'react-i18next'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
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
        <ToggleGroup
          type="single"
          variant="outline"
          size="sm"
          className="w-full"
          aria-label={t('settings.borderColor')}
          value={settings?.borderColor ?? 'white'}
          onValueChange={(color) => color && onChange((s) => ({ ...s, borderColor: color as BorderColor }))}
        >
          {COLORS.map((c) => (
            <ToggleGroupItem key={c.value} value={c.value} className="flex-1 gap-1.5 text-xs">
              <span className={`size-3 rounded-full border ${c.swatch}`} />
              {t(`settings.borderColors.${c.value}`)}
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
      )}
      <p className="text-xs text-muted-foreground">{t(paper ? 'settings.borderHintPaper' : 'settings.borderHint')}</p>
    </div>
  )
}
