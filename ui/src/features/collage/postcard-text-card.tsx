// SPDX-License-Identifier: GPL-3.0-only
import { Type } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { CommitInput } from '@/components/forms/commit-input'
import { Label } from '@/components/ui/label'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { SectionCard } from '@/components/layout/section-card'
import type { PhotoId, PostcardInk, RenderSettings } from '@/ipc/bindings'
import { FontSelect } from '@/features/fonts/font-select'
import { useFonts } from '@/features/fonts/use-fonts'
import { SizeField } from '@/features/stamp/size-field'
import { patchPostcard } from './postcard-defaults'
import { PostcardTextField } from './postcard-text-field'

/** The inks, each with its swatch, as the date stamp shows its colours. */
const INKS: { value: PostcardInk; swatch: string }[] = [
  { value: 'auto', swatch: 'bg-neutral-800' },
  { value: 'blue', swatch: 'bg-[#1c3e96]' },
  { value: 'sepia', swatch: 'bg-[#704214]' },
  { value: 'red', swatch: 'bg-[#b02020]' },
]

type PostcardTextCardProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
  /** The postcard being edited: the chosen sheet's photo, whose own text the field types. */
  photo: PhotoId | null | undefined
  /** Bumps on every edit and undo, so the field follows. */
  revision: number
}

/**
 * Everything typed on a postcard (FR-073), in one card: the chosen card's own text, then how
 * every card sets it (font, size, ink) and the credit line under all of them. The same card
 * in the Collage panel and in a postcard's preview; shown while postcards are on.
 */
export function PostcardTextCard({ settings, onChange, photo, revision }: PostcardTextCardProps) {
  const { t } = useTranslation()
  const fonts = useFonts()
  const postcard = settings?.collage?.postcard
  if (!postcard?.enabled) return null
  return (
    <SectionCard icon={<Type />} title={t('postcard.text')}>
      {photo != null ? (
        <PostcardTextField photo={photo} revision={revision} />
      ) : (
        <p className="text-xs text-muted-foreground">{t('postcard.pickToType')}</p>
      )}
      <div className="space-y-1.5">
        <Label className="text-xs font-normal text-muted-foreground">{t('postcard.font')}</Label>
        <FontSelect
          fonts={fonts}
          value={postcard.font}
          onChange={(font) => patchPostcard(onChange, { font })}
          label={t('postcard.font')}
          builtIn={t('postcard.typewriter')}
        />
      </div>
      <div className="space-y-1.5">
        <Label className="text-xs font-normal text-muted-foreground">{t('postcard.size')}</Label>
        <SizeField
          value={postcard.textSize}
          onChange={(textSize) => patchPostcard(onChange, { textSize })}
          label={t('postcard.size')}
        />
      </div>
      <div className="space-y-1.5">
        <Label className="text-xs font-normal text-muted-foreground">{t('postcard.ink')}</Label>
        <ToggleGroup
          type="single"
          variant="outline"
          size="sm"
          className="w-full"
          value={postcard.ink ?? 'auto'}
          onValueChange={(v) => v && patchPostcard(onChange, { ink: v as PostcardInk })}
        >
          {INKS.map((ink) => (
            <ToggleGroupItem key={ink.value} value={ink.value} className="flex-1 gap-1.5 px-1 text-xs">
              <span className={`size-3 rounded-full border ${ink.swatch}`} />
              {t(`postcard.inks.${ink.value}`)}
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
      </div>
      <div className="space-y-1.5">
        <Label className="text-xs font-normal text-muted-foreground">{t('postcard.credit')}</Label>
        <CommitInput
          value={postcard.credit ?? ''}
          placeholder={t('postcard.creditPlaceholder')}
          ariaLabel={t('postcard.credit')}
          maxLength={80}
          liveMs={600}
          onCommit={(credit) => patchPostcard(onChange, { credit })}
        />
      </div>
    </SectionCard>
  )
}
