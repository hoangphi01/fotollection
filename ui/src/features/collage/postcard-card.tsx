// SPDX-License-Identifier: GPL-3.0-only
// @shown both: in the gallery panels and in the preview (STYLE_RULE §4, where each card shows, checked by check-lego).
import { Mail, Mailbox, PenLine, RectangleHorizontal, RectangleVertical, Square, UserRound } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Field, Hint } from '@/components/forms/field'
import { SegmentedToggle } from '@/components/forms/segmented-toggle'
import { AddAssist } from '@/components/forms/addassist'
import { emptyAddress } from '@/lib/address'
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
const SQUARE: Shape = { value: 'square', icon: Square }
/** The shapes each card takes: a Korean card is landscape or square (owner, 2026-11-15). */
const SHAPES: Record<PostcardDestination, Shape[]> = {
  europe: [LANDSCAPE, PORTRAIT],
  japan: [PORTRAIT, LANDSCAPE],
  korea: [LANDSCAPE, SQUARE],
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
 * them by hand. What is typed on it has its own card (`PostcardTextCard`).
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
  // A shape this card does not take reads as its first one (a square left from a Korean card).
  const orientation = shapes.find((s) => s.value === postcard.orientation)?.value ?? shapes[0]?.value ?? 'landscape'
  const country = card === 'japan' ? 'jp' : 'kr'
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
      <Field label={t('postcard.orientation')} hint={orientation === 'square' ? t('postcard.squareHint') : undefined}>
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
      {/* The sender of Japanese and Korean cards, an optional switch (owner, 2026-11-15). */}
      {card !== 'europe' && (
        <SwitchField
          icon={<UserRound />}
          label={t('postcard.sender')}
          checked={postcard.senderOn ?? false}
          onChange={(senderOn) => setPostcard({ senderOn })}
          hint={t('postcard.senderHint')}
        >
          {/* Each country's own sender: switching cards never shows or prints another's. */}
          <AddAssist
            key={country}
            country={country}
            value={postcard.senders?.find((s) => s.country === country) ?? emptyAddress(country)}
            onCommit={(sender) =>
              setPostcard({ senders: [...(postcard.senders ?? []).filter((s) => s.country !== country), sender] })
            }
            assisted={postcard.assisted ?? true}
            onAssisted={(assisted) => setPostcard({ assisted })}
          />
        </SwitchField>
      )}
      <PanelNote icon={<PenLine />}>{t('postcard.pens')}</PanelNote>
    </SectionCard>
  )
}
