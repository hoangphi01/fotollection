// SPDX-License-Identifier: GPL-3.0-only
import { CopyCheck, History, X } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Field, Hint, RowList } from '@/components/forms/field'
import { Button } from '@/components/ui/button'
import { commands, type CullFilter, type PhotoId, type PostcardPart } from '@/ipc/bindings'
import { errorText } from '@/lib/errors'
import { useLiveQuery } from '@/features/settings/use-ipc-query'

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
 * Typing a card's address once for a whole set (FR-077): under the field, "Use for N
 * postcards without an address" once it is typed, one undo step; and while a card has none,
 * the addresses typed on this machine's last cards, to pick from as the cards are flipped.
 */
export function PostcardTextReuse({ filter, photo, part, text, ticked, revision, onPick }: PostcardTextReuseProps) {
  const { t } = useTranslation()
  const subject = JSON.stringify({ filter, photo, part, ticked } satisfies Subject)
  const gaps = useLiveQuery(text ? subject : null, revision, loadGaps).current
  const remembered = useLiveQuery(text ? null : 'recent', revision, loadRecent).data ?? []
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

  if (text) {
    const blank = gaps?.blank ?? 0
    const differ = gaps?.differ ?? 0
    return (
      <>
        {(blank > 0 || differ > 0) && (
          <div className="flex flex-wrap items-center gap-x-2 gap-y-1 text-xs">
            {blank > 0 && (
              <Button variant="link" size="xs" className="px-0" onClick={() => void apply(false)}>
                <CopyCheck />
                {t(`postcard.reuse.blank${scope}`, { count: blank })}
              </Button>
            )}
            {differ > 0 && (
              <Button
                variant="link"
                size="xs"
                className="px-0 text-muted-foreground"
                onClick={() => void apply(true)}
              >
                {t(`postcard.reuse.overwrite${scope}`, { count: blank + differ })}
              </Button>
            )}
          </div>
        )}
        {done?.photo === photo && done.count > 0 && (
          <Hint>{t('postcard.reuse.done', { count: done.count })}</Hint>
        )}
        {error && <Hint className="text-destructive">{error}</Hint>}
      </>
    )
  }
  if (part !== 'address' || recent.length === 0) return null
  return (
    <Field label={<span className="flex items-center gap-1.5"><History className="size-3.5" />{t('postcard.reuse.recent')}</span>}>
      <RowList>
        {recent.map((address) => (
          <li key={address} className="flex items-center gap-1">
            <Button
              variant="outline"
              size="sm"
              className="min-w-0 flex-1 justify-start font-normal"
              data-hint={address}
              onClick={() => onPick(address)}
            >
              <span className="truncate">{address.replaceAll('\n', ', ')}</span>
            </Button>
            <Button
              variant="ghost-destructive"
              size="icon-sm"
              aria-label={t('postcard.reuse.forget')}
              data-hint={t('postcard.reuse.forget')}
              onClick={() => void forget(address)}
            >
              <X />
            </Button>
          </li>
        ))}
      </RowList>
    </Field>
  )
}
