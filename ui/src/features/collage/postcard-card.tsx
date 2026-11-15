// SPDX-License-Identifier: GPL-3.0-only
// @shown both: in the gallery panels and in the preview (STYLE_RULE §4, where each card shows, checked by check-lego).
import { Mail, Mailbox, PenLine, RectangleHorizontal, RectangleVertical } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Field, Hint } from '@/components/forms/field'
import { SegmentedToggle } from '@/components/forms/segmented-toggle'
import { SwitchField } from '@/components/forms/switch-field'
import { PanelNote } from '@/components/layout/panel-note'
import { SectionCard } from '@/components/layout/section-card'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import type { PostcardDestination, PostcardOrientation, PostcardSettings, RenderSettings } from '@/ipc/bindings'
import { cardFor, choosesCard } from '@/lib/postcard-card'
import { patchPostcard } from './postcard-defaults'

type Shape = { value: PostcardOrientation; icon: typeof RectangleHorizontal }
const LANDSCAPE: Shape = { value: 'landscape', icon: RectangleHorizontal }
const PORTRAIT: Shape = { value: 'portrait', icon: RectangleVertical }
/** The shapes each card takes: landscape or upright (owner, 2026-11-15: Korea's square gone). */
const SHAPES: Record<PostcardDestination, Shape[]> = {
  europe: [LANDSCAPE, PORTRAIT],
  japan: [PORTRAIT, LANDSCAPE],
  korea: [LANDSCAPE, PORTRAIT],
}
const DESTINATIONS: PostcardDestination[] = ['europe', 'japan', 'korea']
const LINES = [3, 4]

type PostcardCardProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
}

/**
 * The postcard's shape in Collage mode (FR-073), shown while the Postcard switch above the
 * sheets is on: the way it is held and its writing lines, with the pen note for writing on
 * them by hand. Its sender, its recipient and its message each have their own card (FR-136,
 * `PostcardCards`).
 */
export function PostcardCard({ settings, onChange }: PostcardCardProps) {
  const { t, i18n } = useTranslation()
  const postcard = settings?.collage?.postcard
  const card = cardFor(i18n.language, postcard?.destination)
  // Turned on in the bar above the sheets; nothing to show otherwise (a block with nothing
  // to say is not rendered, STYLE_RULE §4).
  if (!postcard?.enabled) return null
  const setPostcard = (patch: Partial<PostcardSettings>) => patchPostcard(onChange, patch)
  const shapes = SHAPES[card]
  const orientation = shapes.find((s) => s.value === postcard.orientation)?.value ?? shapes[0]?.value ?? 'landscape'
  return (
    <SectionCard icon={<Mail />} title={t('postcard.title')}>
      <Hint>{t(`postcard.hintCard.${card}`)}</Hint>
      {/* Where the card goes, in English and Vietnamese only (owner, 2026-11-15, board K3 A). */}
      {choosesCard(i18n.language) && (
        <Field label={t('postcard.cardFor')}>
          <Select value={card} onValueChange={(destination) => setPostcard({ destination: destination as PostcardDestination })}>
            <SelectTrigger className="w-full" aria-label={t('postcard.cardFor')}>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {DESTINATIONS.map((d) => (
                <SelectItem key={d} value={d}>
                  {t(`postcard.destination.${d}`)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>
      )}
      {/* An upright Korean card goes at the non-standard rate (Korea Post); the sender's choice. */}
      <Field
        label={t('postcard.orientation')}
        hint={card === 'korea' && orientation === 'portrait' ? t('postcard.portraitKoreaHint') : undefined}
      >
        <SegmentedToggle<PostcardOrientation>
          value={orientation}
          options={shapes.map(({ value, icon: Icon }) => ({
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
      {card === 'europe' && (
        <Field label={t('postcard.lines')}>
          <SegmentedToggle
            value={String(postcard.lines ?? 4)}
            options={LINES.map((n) => ({ value: String(n), label: t('postcard.lineCount', { count: n }) }))}
            onChange={(v) => setPostcard({ lines: Number(v) })}
            ariaLabel={t('postcard.lines')}
          />
        </Field>
      )}
      {/* Made to be posted (FR-073, owner 2026-11-12): the post's strip kept free, on by default. */}
      <SwitchField
        icon={<Mailbox />}
        label={t('postcard.postalUse')}
        checked={postcard.postalUse ?? true}
        onChange={(postalUse) => setPostcard({ postalUse })}
        hint={t(`postcard.postalUseHintCard.${card}`)}
      />
      <PanelNote icon={<PenLine />}>{t('postcard.pens')}</PanelNote>
    </SectionCard>
  )
}
