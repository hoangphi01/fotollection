// SPDX-License-Identifier: GPL-3.0-only
import { commands, type CardParty, type Country, type CullFilter, type PhotoId } from '@/ipc/bindings'
import { errorText } from '@/lib/errors'

/** Puts a card's message or recipient on the other cards (FR-077, FR-133): with `overwrite`,
 *  theirs too. Resolves to how many changed, or to an error's words. */
export type ReuseApply = (overwrite: boolean) => Promise<number | string>

/** A card's typed text (message, address, recipients, senders) for `useLiveQuery`, the photo's
 *  id as its subject; null while it cannot be read. One loader for every postcard card. */
export const loadPostcardText = async (photo: string) => {
  const result = await commands.getPostcardText(Number(photo))
  return result.status === 'ok' ? result.data : null
}

/** A Japanese or Korean card's sender or recipient, and the cards it may go to (FR-133). */
export type PartyCards = { filter: CullFilter; photo: PhotoId; country: Country; party: CardParty; ticked: readonly PhotoId[] }

/** How many other cards lack this card's sender or recipient, or have another, for
 *  `useLiveQuery` with a `PartyCards` as its JSON subject. */
export const loadPartyGaps = async (subject: string) => {
  const { filter, photo, country, party, ticked } = JSON.parse(subject) as PartyCards
  const result = await commands.postcardRecipientGaps(filter, photo, country, party, [...ticked])
  return result.status === 'ok' ? result.data : null
}

/** Puts this card's sender or recipient on the other cards, as `PostcardReuseNote` asks. */
export const applyParty =
  ({ filter, photo, country, party, ticked }: PartyCards): ReuseApply =>
  async (overwrite) => {
    const result = await commands.applyPostcardRecipient(filter, photo, country, party, overwrite, [...ticked])
    return result.status === 'ok' ? result.data : errorText(result.error)
  }
