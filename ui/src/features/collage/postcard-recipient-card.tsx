// SPDX-License-Identifier: GPL-3.0-only
// @shown both: in the gallery panels and in the preview (STYLE_RULE §4, where each card shows, checked by check-lego).
import { MapPin } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Hint } from '@/components/forms/field'
import { SectionCard } from '@/components/layout/section-card'
import { commands, type Country, type CullFilter, type PhotoId, type Recipient, type RenderSettings } from '@/ipc/bindings'
import { cardFor } from '@/lib/postcard-card'
import { useLiveQuery } from '@/features/settings/use-ipc-query'
import { AddressBookMenu } from './address-book-menu'
import { PostcardAddressField } from './postcard-address-field'
import { patchPostcard } from './postcard-defaults'
import { PostcardReuseNote } from './postcard-reuse-note'
import { applyParty, loadPartyGaps, loadPostcardText, type PartyCards } from './postcard-reuse'
import { PostcardTextField } from './postcard-text-field'
import { usePostcardCuts } from './use-postcard-cuts'

type PostcardRecipientCardProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
  photo: PhotoId | null | undefined
  revision: number
  filter?: CullFilter
  ticked?: readonly PhotoId[]
}

/**
 * Who the chosen postcard goes to (FR-136's second block, FR-133): the address book at the
 * top, then the European address as typed or the Japanese or Korean recipient AddAssist fills,
 * and the note that puts it on the other cards in one step. The same card in the Collage panel
 * and in a postcard's preview; shown while postcards are on.
 */
export function PostcardRecipientCard({ settings, onChange, photo, revision, filter, ticked = [] }: PostcardRecipientCardProps) {
  const { t, i18n } = useTranslation()
  const postcard = settings?.collage?.postcard
  const card = cardFor(i18n.language, postcard?.destination)
  const country: Country | null = card === 'europe' ? null : card === 'japan' ? 'jp' : 'kr'
  const fit = usePostcardCuts(settings, revision).get(photo ?? -1)
  // A pick from the book changes the card: its own revision makes the fields read it again.
  const [picked, setPicked] = useState(0)
  const seen = revision + picked
  const text = useLiveQuery(photo != null ? String(photo) : null, seen, loadPostcardText).data
  const recipientNow = country === null ? null : (text?.recipients?.find((r) => r.country === country) ?? null)
  const current: Recipient | null =
    country === null ? (text?.address?.trim() ? { kind: 'free', value: text.address } : null) : recipientNow ? { kind: 'assisted', value: recipientNow } : null
  const cards: PartyCards | null = photo != null && country !== null && filter ? { filter, photo, country, party: 'recipient', ticked } : null
  const gaps = useLiveQuery(cards && recipientNow ? JSON.stringify(cards) : null, seen, loadPartyGaps).data
  if (!postcard?.enabled) return null
  const pick = async (recipient: Recipient) => {
    if (photo == null) return
    if (recipient.kind === 'free') await commands.setPostcardText(photo, 'address', recipient.value)
    else await commands.setPostcardAddress(photo, recipient.value.country, recipient.value)
    setPicked((p) => p + 1)
  }
  return (
    <SectionCard icon={<MapPin />} title={t('postcard.recipient')}>
      {photo != null ? (
        <>
          <AddressBookMenu country={country} current={current} onPick={(r) => void pick(r)} />
          {country === null ? (
            <PostcardTextField
              photo={photo}
              part="address"
              revision={seen}
              style={postcard.addressStyle}
              cut={fit?.addressCut ?? false}
              onStyle={(addressStyle) => patchPostcard(onChange, { addressStyle })}
              filter={filter}
              ticked={ticked}
            />
          ) : (
            <>
              <PostcardAddressField
                photo={photo}
                country={country}
                revision={seen}
                assisted={postcard.assisted ?? true}
                onAssisted={(assisted) => patchPostcard(onChange, { assisted })}
              />
              {filter && (
                <PostcardReuseNote
                  photo={photo}
                  gaps={recipientNow ? gaps : null}
                  ticked={ticked}
                  apply={applyParty({ filter, photo, country, party: 'recipient', ticked })}
                />
              )}
            </>
          )}
        </>
      ) : (
        <Hint>{t('postcard.pickToType')}</Hint>
      )}
    </SectionCard>
  )
}
