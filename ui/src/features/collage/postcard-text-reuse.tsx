// SPDX-License-Identifier: GPL-3.0-only
import { CopyCheck, History, MapPin } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Field, Hint } from '@/components/forms/field'
import { ListRow, ListRows } from '@/components/forms/list-row'
import { PanelNote } from '@/components/layout/panel-note'
import { Button } from '@/components/ui/button'
import { commands, type CullFilter, type PhotoId, type PostcardPart } from '@/ipc/bindings'
import { errorText } from '@/lib/errors'
import { useLiveQuery } from '@/features/settings/use-ipc-query'
import { RemoveButton } from '@/components/forms/danger-button'

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
  const [done, setDone] = useState<{ photo: PhotoId; count: number } | null>(null)
  const [error, setError] = useState<string | null>(null)
  const recent = remembered.filter((a) => !forgotten.includes(a))
  const scope = ticked.length > 0 ? 'Ticked' : 'All'

  const apply = async (overwrite: boolean) => {
    setError(null)
    const result = await commands.applyPostcardText(filter, photo, part, overwrite, [...ticked])
    if (result.status === 'ok') setDone({ photo, count: result.data })
    else setError(errorText(result.error))
  }
  const forget = async (address: string) => {
    setForgotten((f) => [...f, address])
    await commands.forgetAddress(address)
  }

  const blank = text ? (gaps?.blank ?? 0) : 0
  const differ = text ? (gaps?.differ ?? 0) : 0
  // The other cards, as a note with its two choices as buttons (STYLE_RULE §4: a note that
  // offers an action), so the words wrap in an 18.75 rem panel instead of being cut.
  const reuse = (blank > 0 || differ > 0) && (
    <PanelNote
      icon={<CopyCheck />}
      action={
        <>
          {blank > 0 && (
            <Button size="sm" onClick={() => void apply(false)}>
              {t('postcard.reuse.fill')}
            </Button>
          )}
          {differ > 0 && (
            <Button variant="outline" size="sm" onClick={() => void apply(true)}>
              {t('postcard.reuse.replace', { count: blank + differ })}
            </Button>
          )}
        </>
      }
    >
      {blank > 0 ? t(`postcard.reuse.blank${scope}`, { count: blank }) : t(`postcard.reuse.differ${scope}`, { count: differ })}
    </PanelNote>
  )
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
          <div key={address} className="flex items-center gap-2">
            {/* The row shows the address on one line; its hint, the whole of it. */}
            <div className="min-w-0 flex-1" data-hint={address}>
              <ListRow icon={<MapPin />} chosen={address === text.trim()} onClick={() => onPick(address)}>
                {address.replaceAll('\n', ', ')}
              </ListRow>
            </div>
            <RemoveButton iconOnly label={t('postcard.reuse.forget')} onClick={() => void forget(address)} />
          </div>
        ))}
      </ListRows>
    </Field>
  )
  return (
    <>
      {reuse}
      {done?.photo === photo && done.count > 0 && <Hint>{t('postcard.reuse.done', { count: done.count })}</Hint>}
      {error && <Hint className="text-destructive">{error}</Hint>}
      {list}
    </>
  )
}
