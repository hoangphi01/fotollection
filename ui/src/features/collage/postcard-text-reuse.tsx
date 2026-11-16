// SPDX-License-Identifier: GPL-3.0-only
import { History, MapPin } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Field } from '@/components/forms/field'
import { ListRows, RemovableRow } from '@/components/forms/list-row'
import { commands, type CullFilter, type PhotoId, type PostcardPart } from '@/ipc/bindings'
import { errorText } from '@/lib/errors'
import { useLiveQuery } from '@/features/settings/use-ipc-query'
import { PostcardReuseNote } from './postcard-reuse-note'

type Subject = { filter: CullFilter; photo: PhotoId; part: PostcardPart; ticked: readonly PhotoId[] }

const loadGaps = async (subject: string) => {
  const { filter, photo, part, ticked } = JSON.parse(subject) as Subject
  const result = await commands.postcardTextGaps(filter, photo, part, [...ticked])
  return result.status === 'ok' ? result.data : null
}

const loadRecent = async () => {
  const result = await commands.recentAddresses()
  return result.status === 'ok' ? result.data : null
}

type PostcardTextReuseProps = {
  filter: CullFilter
  photo: PhotoId
  part: PostcardPart
  /** This card's text for the part, as the field shows it. */
  text: string
  /** Photos ticked in select mode: when there are some, only they get the text. */
  ticked: readonly PhotoId[]
  revision: number
  /** Puts a remembered address on this card. */
  onPick: (address: string) => void
}

/**
 * Typing a card's address once for a whole set (FR-077): under the field, once it is typed, a
 * note on the other cards with *Use for them* (those without one) and *Use for all N*
 * (replacing theirs), one undo step; and always, the addresses typed on this machine's last
 * cards, the one on this card marked, to pick from as the cards are flipped.
 */
export function PostcardTextReuse({ filter, photo, part, text, ticked, revision, onPick }: PostcardTextReuseProps) {
  const { t } = useTranslation()
  const subject = JSON.stringify({ filter, photo, part, ticked } satisfies Subject)
  // The last answer stays while the next card's comes, so flipping cards never drops the note
  // for a moment (STYLE_RULE §4: nothing blinks).
  const gaps = useLiveQuery(text ? subject : null, revision, loadGaps).data
  // The addresses stay listed whatever this card holds (owner, 2026-11-07: "the list showed
  // once, then went"): the one on this card is marked, another is one click away.
  const remembered = useLiveQuery(part === 'address' ? 'recent' : null, revision, loadRecent).data ?? []
  const [forgotten, setForgotten] = useState<readonly string[]>([])
  const recent = remembered.filter((a) => !forgotten.includes(a))

  const apply = async (overwrite: boolean) => {
    const result = await commands.applyPostcardText(filter, photo, part, overwrite, [...ticked])
    return result.status === 'ok' ? result.data : errorText(result.error)
  }
  const forget = async (address: string) => {
    setForgotten((f) => [...f, address])
    await commands.forgetAddress(address)
  }

  const list = part === 'address' && recent.length > 0 && (
    <Field
      label={
        <span className="flex items-center gap-1.5">
          <History className="size-4" />
          {t('postcard.reuse.recent')}
        </span>
      }
    >
      <ListRows>
        {recent.map((address) => (
          <RemovableRow
            key={address}
            icon={<MapPin />}
            hint={address}
            chosen={address === text.trim()}
            onClick={() => onPick(address)}
            removeLabel={t('postcard.reuse.forget')}
            onRemove={() => void forget(address)}
          >
            {address.replaceAll('\n', ', ')}
          </RemovableRow>
        ))}
      </ListRows>
    </Field>
  )
  return (
    <>
      <PostcardReuseNote photo={photo} gaps={text ? gaps : null} ticked={ticked} apply={apply} />
      {list}
    </>
  )
}
