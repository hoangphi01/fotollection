// SPDX-License-Identifier: GPL-3.0-only
import { Mail, PenLine, RectangleHorizontal, RectangleVertical } from 'lucide-react'
import { useId } from 'react'
import { useTranslation } from 'react-i18next'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { PanelNote } from '@/components/layout/panel-note'
import { SectionCard } from '@/components/layout/section-card'
import type { PostcardOrientation, PostcardSettings, RenderSettings } from '@/ipc/bindings'
import { FontSelect } from '@/features/fonts/font-select'
import { useFonts } from '@/features/fonts/use-fonts'

const ORIENTATIONS: { value: PostcardOrientation; icon: typeof RectangleHorizontal }[] = [
  { value: 'landscape', icon: RectangleHorizontal },
  { value: 'portrait', icon: RectangleVertical },
]
const LINES = [3, 4]
/** As `PostcardSettings::default()` in Rust, for a file from before FR-073. */
const DEFAULTS: PostcardSettings = { enabled: false, orientation: 'landscape', lines: 4, font: null }

type PostcardCardProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
}

/**
 * The postcard switch of Collage mode (FR-073), laid out like the date stamp card: on, each
 * sheet holds one photo and a postcard back with a dashed stamp box and writing lines. The
 * way it is held, the number of lines and the typed text's font; the pen note, as photo
 * paper takes only some pens. Each postcard's own text is typed in its preview.
 */
export function PostcardCard({ settings, onChange }: PostcardCardProps) {
  const { t } = useTranslation()
  const id = useId()
  const fonts = useFonts()
  const postcard = settings?.collage?.postcard
  const setPostcard = (patch: Partial<PostcardSettings>) =>
    onChange((s) => ({ ...s, collage: { ...s.collage, postcard: { ...DEFAULTS, ...s.collage?.postcard, ...patch } } }))
  return (
    <SectionCard
      icon={<Mail />}
      title={
        <Label htmlFor={id} className="text-sm font-semibold">
          {t('postcard.title')}
        </Label>
      }
      action={<Switch id={id} checked={postcard?.enabled ?? false} onCheckedChange={(enabled) => setPostcard({ enabled })} />}
      open={postcard?.enabled ?? false}
    >
      <p className="text-xs text-muted-foreground">{t('postcard.hint')}</p>
      <div className="space-y-1.5">
        <Label className="text-xs font-normal text-muted-foreground">{t('postcard.orientation')}</Label>
        <ToggleGroup
          type="single"
          variant="outline"
          size="sm"
          className="w-full"
          value={postcard?.orientation ?? 'landscape'}
          onValueChange={(v) => v && setPostcard({ orientation: v as PostcardOrientation })}
        >
          {ORIENTATIONS.map(({ value, icon: Icon }) => (
            <ToggleGroupItem key={value} value={value} className="flex-1 px-1 text-xs" data-hint={t(`postcard.${value}Hint`)}>
              <Icon /> {t(`postcard.${value}`)}
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
      </div>
      <div className="space-y-1.5">
        <Label className="text-xs font-normal text-muted-foreground">{t('postcard.lines')}</Label>
        <ToggleGroup
          type="single"
          variant="outline"
          size="sm"
          className="w-full"
          value={String(postcard?.lines ?? 4)}
          onValueChange={(v) => v && setPostcard({ lines: Number(v) })}
        >
          {LINES.map((n) => (
            <ToggleGroupItem key={n} value={String(n)} className="flex-1 px-1 text-xs">
              {t('postcard.lineCount', { count: n })}
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
      </div>
      <div className="space-y-1.5">
        <Label className="text-xs font-normal text-muted-foreground">{t('postcard.font')}</Label>
        <FontSelect
          fonts={fonts}
          value={postcard?.font}
          onChange={(font) => setPostcard({ font })}
          label={t('postcard.font')}
          builtIn={t('postcard.typewriter')}
        />
      </div>
      <PanelNote icon={<PenLine />}>{t('postcard.pens')}</PanelNote>
    </SectionCard>
  )
}
