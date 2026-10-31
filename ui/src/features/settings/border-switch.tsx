// SPDX-License-Identifier: GPL-3.0-only
import { Frame } from 'lucide-react'
import { useId } from 'react'
import { useTranslation } from 'react-i18next'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import type { RenderSettings } from '@/ipc/bindings'

type BorderSwitchProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
}

/** The 3 mm border inside the print size, the same switch in every mode (FR-067). */
export function BorderSwitch({ settings, onChange }: BorderSwitchProps) {
  const { t } = useTranslation()
  const id = useId()
  return (
    <div className="space-y-1.5">
      <div className="flex items-center justify-between">
        <Label htmlFor={id} className="flex items-center gap-2 text-sm font-normal">
          <Frame className="size-4 text-muted-foreground" />
          {t('settings.border')}
        </Label>
        <Switch
          id={id}
          checked={settings?.border ?? false}
          onCheckedChange={(border) => onChange((s) => ({ ...s, border }))}
        />
      </div>
      <p className="text-xs text-muted-foreground">{t('settings.borderHint')}</p>
    </div>
  )
}
