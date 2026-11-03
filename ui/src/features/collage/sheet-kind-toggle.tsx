// SPDX-License-Identifier: GPL-3.0-only
import { Columns2, Mail } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import type { RenderSettings } from '@/ipc/bindings'
import { POSTCARD_DEFAULTS } from './postcard-defaults'

type SheetKindToggleProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
}

/**
 * Collage mode's sheet kind, in the bar above the sheets next to Select (FR-035, FR-073):
 * two photos to a print, or one photo and a postcard back, so 50 photos make 25 sheets or 50
 * postcards. The Postcard card in the panel then holds the postcard's own settings.
 */
export function SheetKindToggle({ settings, onChange }: SheetKindToggleProps) {
  const { t } = useTranslation()
  const postcard = settings?.collage?.postcard?.enabled ?? false
  const set = (enabled: boolean) =>
    onChange((s) => ({
      ...s,
      collage: { ...s.collage, postcard: { ...POSTCARD_DEFAULTS, ...s.collage?.postcard, enabled } },
    }))
  return (
    <ToggleGroup
      type="single"
      variant="outline"
      size="sm"
      value={postcard ? 'postcard' : 'pair'}
      onValueChange={(v) => v && set(v === 'postcard')}
      aria-label={t('postcard.kind')}
    >
      <ToggleGroupItem value="pair" className="px-2 text-xs" data-hint={t('postcard.pairHint')}>
        <Columns2 /> <span className="hidden @3xl:inline">{t('postcard.pair')}</span>
      </ToggleGroupItem>
      <ToggleGroupItem value="postcard" className="px-2 text-xs" data-hint={t('postcard.cardHint')}>
        <Mail /> <span className="hidden @3xl:inline">{t('postcard.title')}</span>
      </ToggleGroupItem>
    </ToggleGroup>
  )
}
