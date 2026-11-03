// SPDX-License-Identifier: GPL-3.0-only
import { Mail } from 'lucide-react'
import { useId } from 'react'
import { useTranslation } from 'react-i18next'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import type { RenderSettings } from '@/ipc/bindings'
import { POSTCARD_DEFAULTS } from './postcard-defaults'

type SheetKindToggleProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
}

/**
 * Collage mode's Postcard switch, in the bar above the sheets next to Select (FR-035,
 * FR-073): off, two photos to a print; on, one photo and a postcard back, so 50 photos make
 * 25 sheets or 50 postcards. The Postcard card in the panel then holds its own settings.
 * A labelled switch, as the owner found an icon toggle unclear.
 */
export function SheetKindToggle({ settings, onChange }: SheetKindToggleProps) {
  const { t } = useTranslation()
  const id = useId()
  const postcard = settings?.collage?.postcard?.enabled ?? false
  const set = (enabled: boolean) =>
    onChange((s) => ({
      ...s,
      collage: { ...s.collage, postcard: { ...POSTCARD_DEFAULTS, ...s.collage?.postcard, enabled } },
    }))
  return (
    <div className="flex shrink-0 items-center gap-2" data-hint={t('postcard.cardHint')}>
      <Mail className="size-4 text-muted-foreground" />
      <Label htmlFor={id} className="text-sm font-normal whitespace-nowrap">
        {t('postcard.title')}
      </Label>
      <Switch id={id} checked={postcard} onCheckedChange={set} />
    </div>
  )
}
