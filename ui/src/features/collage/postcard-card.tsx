// SPDX-License-Identifier: GPL-3.0-only
// @shown both: in the gallery panels and in the preview (STYLE_RULE §4, where each card shows, checked by check-lego).
import { Mail, Mailbox, PenLine, RectangleHorizontal, RectangleVertical } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Field, Hint } from '@/components/forms/field'
import { SegmentedToggle } from '@/components/forms/segmented-toggle'
import { SwitchField } from '@/components/forms/switch-field'
import { PanelNote } from '@/components/layout/panel-note'
import { SectionCard } from '@/components/layout/section-card'
import type { PostcardOrientation, PostcardSettings, RenderSettings } from '@/ipc/bindings'
import { patchPostcard } from './postcard-defaults'

const ORIENTATIONS: { value: PostcardOrientation; icon: typeof RectangleHorizontal }[] = [
  { value: 'landscape', icon: RectangleHorizontal },
  { value: 'portrait', icon: RectangleVertical },
]
const LINES = [3, 4]

type PostcardCardProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
}

/**
 * The postcard's shape in Collage mode (FR-073), shown while the Postcard switch above the
 * sheets is on: the way it is held and its writing lines, with the pen note for writing on
 * them by hand. What is typed on it has its own card (`PostcardTextCard`).
 */
export function PostcardCard({ settings, onChange }: PostcardCardProps) {
  const { t } = useTranslation()
  const postcard = settings?.collage?.postcard
  // Turned on in the bar above the sheets; nothing to show otherwise (a block with nothing
  // to say is not rendered, STYLE_RULE §4).
  if (!postcard?.enabled) return null
  const setPostcard = (patch: Partial<PostcardSettings>) => patchPostcard(onChange, patch)
  return (
    <SectionCard icon={<Mail />} title={t('postcard.title')}>
      <Hint>{t('postcard.hint')}</Hint>
      <Field label={t('postcard.orientation')}>
        <SegmentedToggle<PostcardOrientation>
          value={postcard.orientation ?? 'landscape'}
          options={ORIENTATIONS.map(({ value, icon: Icon }) => ({
            value,
            label: (
              <>
                <Icon /> {t(`postcard.${value}`)}
              </>
            ),
            hint: t(`postcard.${value}Hint`),
          }))}
          onChange={(orientation) => setPostcard({ orientation })}
          ariaLabel={t('postcard.orientation')}
        />
      </Field>
      <Field label={t('postcard.lines')}>
        <SegmentedToggle
          value={String(postcard.lines ?? 4)}
          options={LINES.map((n) => ({ value: String(n), label: t('postcard.lineCount', { count: n }) }))}
          onChange={(v) => setPostcard({ lines: Number(v) })}
          ariaLabel={t('postcard.lines')}
        />
      </Field>
      {/* Made to be posted (FR-073, owner 2026-11-12): the post's strip kept free, on by default. */}
      <SwitchField
        icon={<Mailbox />}
        label={t('postcard.postalUse')}
        checked={postcard.postalUse ?? true}
        onChange={(postalUse) => setPostcard({ postalUse })}
        hint={t('postcard.postalUseHint')}
      />
      <PanelNote icon={<PenLine />}>{t('postcard.pens')}</PanelNote>
    </SectionCard>
  )
}
