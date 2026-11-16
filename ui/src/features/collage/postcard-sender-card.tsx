// SPDX-License-Identifier: GPL-3.0-only
// @shown both: in the gallery panels and in the preview (STYLE_RULE §4, where each card shows, checked by check-lego).
import { UserRound } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { AddAssist } from '@/components/forms/addassist'
import { ResetButton } from '@/components/forms/danger-button'
import { Hint } from '@/components/forms/field'
import { SwitchField } from '@/components/forms/switch-field'
import { SectionCard } from '@/components/layout/section-card'
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { commands, type Address, type CullFilter, type PhotoId, type RenderSettings } from '@/ipc/bindings'
import { emptyAddress } from '@/lib/address'
import { errorText } from '@/lib/errors'
import { cardFor } from '@/lib/postcard-card'
import { useLiveQuery } from '@/features/settings/use-ipc-query'
import { patchPostcard } from './postcard-defaults'
import { PostcardReuseNote } from './postcard-reuse-note'
import { applyParty, loadPartyGaps, loadPostcardText, type PartyCards } from './postcard-reuse'

type PostcardSenderCardProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
  photo: PhotoId | null | undefined
  revision: number
  filter?: CullFilter
  ticked?: readonly PhotoId[]
}

/**
 * The sender of Japanese and Korean cards (FR-136's first block; owner, 2026-11-15: a sender
 * per card, applied to many as a look is): optional, folded until its switch is on. The field
 * is the chosen card's own sender, the machine's one shown while the card has none; the note
 * under it puts it on the other cards, and the red ↺ takes the senders off them after a
 * question. A European card has no sender line, so the card is not shown there.
 */
export function PostcardSenderCard({ settings, onChange, photo, revision, filter, ticked = [] }: PostcardSenderCardProps) {
  const { t, i18n } = useTranslation()
  const postcard = settings?.collage?.postcard
  const card = cardFor(i18n.language, postcard?.destination)
  const country = card === 'japan' ? 'jp' : 'kr'
  const [changed, setChanged] = useState(0)
  const [asking, setAsking] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const seen = revision + changed
  const text = useLiveQuery(photo != null ? String(photo) : null, seen, loadPostcardText).data
  const own = text?.senders?.find((s) => s.country === country) ?? null
  const machine = postcard?.senders?.find((s) => s.country === country) ?? null
  const cards: PartyCards | null = photo != null && filter ? { filter, photo, country, party: 'sender', ticked } : null
  const gaps = useLiveQuery(cards && own ? JSON.stringify(cards) : null, seen, loadPartyGaps).data
  if (!postcard?.enabled || card === 'europe') return null
  const scope = ticked.length > 0 ? 'Ticked' : 'All'
  const save = (sender: Address) => {
    if (photo == null) return
    void commands.setPostcardSender(photo, country, sender).then(() => setChanged((c) => c + 1))
  }
  const reset = async () => {
    setAsking(false)
    if (!filter) return
    const result = await commands.clearPostcardParty(filter, country, 'sender', [...ticked])
    if (result.status !== 'ok') return setError(errorText(result.error))
    setError(null)
    // Every card cleared: the machine's sender goes too, so none prints by itself.
    if (ticked.length === 0) patchPostcard(onChange, { senders: (postcard.senders ?? []).filter((s) => s.country !== country) })
    setChanged((c) => c + 1)
  }
  return (
    <SectionCard
      icon={<UserRound />}
      title={t('postcard.sender')}
      action={
        filter && (own || machine) ? <ResetButton iconOnly label={t(`postcard.senderReset${scope}`)} onClick={() => setAsking(true)} /> : undefined
      }
    >
      <SwitchField
        icon={<UserRound />}
        label={t('postcard.senderPrint')}
        checked={postcard.senderOn ?? false}
        onChange={(senderOn) => patchPostcard(onChange, { senderOn })}
        hint={t('postcard.senderHintCard')}
      >
        {photo != null ? (
          <>
            <AddAssist
              key={`${country}-${photo}`}
              country={country}
              value={own ?? machine ?? emptyAddress(country)}
              onCommit={save}
              assisted={postcard.assisted ?? true}
              onAssisted={(assisted) => patchPostcard(onChange, { assisted })}
            />
            {filter && (
              <PostcardReuseNote
                photo={photo}
                gaps={own ? gaps : null}
                ticked={ticked}
                apply={applyParty({ filter, photo, country, party: 'sender', ticked })}
              />
            )}
          </>
        ) : (
          <Hint>{t('postcard.pickToType')}</Hint>
        )}
        {error && <Hint className="text-destructive">{error}</Hint>}
      </SwitchField>
      <AlertDialog open={asking} onOpenChange={setAsking}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t(`postcard.senderReset${scope}`)}</AlertDialogTitle>
            <AlertDialogDescription>{t('postcard.senderResetBody')}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            {/* Its action is the red one, so Cancel stays neutral: one red, the danger. */}
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <ResetButton label={t('postcard.senderResetOk')} onClick={() => void reset()} />
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </SectionCard>
  )
}
