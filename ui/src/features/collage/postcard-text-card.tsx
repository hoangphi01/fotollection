// SPDX-License-Identifier: GPL-3.0-only
import { Type } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { CommitInput } from '@/components/forms/commit-input'
import { Field, Hint } from '@/components/forms/field'
import { SwatchToggle } from '@/components/forms/swatch-toggle'
import { SectionCard } from '@/components/layout/section-card'
import type { CullFilter, PhotoId, PostcardFace, PostcardInk, RenderSettings } from '@/ipc/bindings'
import { FontSelect } from '@/features/fonts/font-select'
import { useFonts } from '@/features/fonts/use-fonts'
import { SizeField } from '@/features/stamp/size-field'
import { patchPostcard } from './postcard-defaults'
import { PostcardTextField } from './postcard-text-field'
import { usePostcardCuts } from './use-postcard-cuts'

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
  /** The postcards shown, and those ticked, for "Use for N postcards" (FR-077). */
  filter?: CullFilter
  ticked?: readonly PhotoId[]
}

/**
 * Everything typed on a postcard (FR-073), in one card: the chosen card's own text, then how
 * every card sets it (font, size, ink) and the credit line under all of them. The same card
 * in the Collage panel and in a postcard's preview; shown while postcards are on.
 */
export function PostcardTextCard({ settings, onChange, photo, revision, filter, ticked }: PostcardTextCardProps) {
  const { t } = useTranslation()
  const fonts = useFonts()
  const postcard = settings?.collage?.postcard
  const fit = usePostcardCuts(settings, revision).get(photo ?? -1)
  if (!postcard?.enabled) return null
  return (
    <SectionCard icon={<Type />} title={t('postcard.text')}>
      {photo != null ? (
        <>
          <PostcardTextField
            photo={photo}
            part="message"
            revision={revision}
            style={postcard.messageStyle}
            cut={fit?.messageCut ?? false}
            onStyle={(messageStyle) => patchPostcard(onChange, { messageStyle })}
          />
          <PostcardTextField
            photo={photo}
            part="address"
            revision={revision}
            style={postcard.addressStyle}
            cut={fit?.addressCut ?? false}
            onStyle={(addressStyle) => patchPostcard(onChange, { addressStyle })}
            filter={filter}
            ticked={ticked}
          />
        </>
      ) : (
        <Hint>{t('postcard.pickToType')}</Hint>
      )}
      <Field label={t('postcard.font')}>
        <FontSelect
          fonts={fonts}
          value={postcard.font}
          onChange={(font, face) =>
            patchPostcard(onChange, face ? { font, face: face as PostcardFace } : { font })
          }
          label={t('postcard.font')}
          builtIns={[
            { key: 'typewriter', label: t('postcard.typewriter') },
            { key: 'handwriting', label: t('postcard.handwriting') },
          ]}
          builtIn={postcard.face ?? 'typewriter'}
        />
      </Field>
      <Field label={t('postcard.size')}>
        <SizeField
          value={postcard.textSize}
          onChange={(textSize) => patchPostcard(onChange, { textSize })}
          label={t('postcard.size')}
        />
      </Field>
      <Field label={t('postcard.ink')}>
        <SwatchToggle
          dense
          value={postcard.ink ?? 'auto'}
          options={INKS.map((ink) => ({ ...ink, label: t(`postcard.inks.${ink.value}`) }))}
          onChange={(ink) => patchPostcard(onChange, { ink })}
        />
      </Field>
      <Field label={t('postcard.credit')}>
        <CommitInput
          value={postcard.credit ?? ''}
          placeholder={t('postcard.creditPlaceholder')}
          ariaLabel={t('postcard.credit')}
          maxLength={80}
          liveMs={600}
          onCommit={(credit) => patchPostcard(onChange, { credit })}
        />
      </Field>
    </SectionCard>
  )
}
