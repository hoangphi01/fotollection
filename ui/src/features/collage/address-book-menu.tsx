// SPDX-License-Identifier: GPL-3.0-only
import { BookUser, History, Star } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { CommitInput } from '@/components/forms/commit-input'
import { RemoveButton } from '@/components/forms/danger-button'
import { Field, FieldLabel, Hint } from '@/components/forms/field'
import { ListRow } from '@/components/forms/list-row'
import { MenuContent } from '@/components/forms/menu-content'
import { PickerTrigger } from '@/components/forms/picker-trigger'
import { Popover, PopoverTrigger } from '@/components/ui/popover'
import { Button } from '@/components/ui/button'
import { commands, type Address, type AddressBook, type Country, type Recipient } from '@/ipc/bindings'
import { errorText } from '@/lib/errors'
import { useLiveQuery } from '@/features/settings/use-ipc-query'

const loadBook = async () => {
  const result = await commands.addressBook()
  return result.status === 'ok' ? result.data : null
}

/** How a recipient reads on one row: its name first, then where it goes. */
function rowText(recipient: Recipient): string {
  if (recipient.kind === 'free') return recipient.value.replaceAll('\n', ', ')
  const a = recipient.value
  return [a.name ?? '', (a.region ?? '') + (a.city ?? '') + (a.town ?? '') + (a.road ?? ''), a.line ?? ''].filter((part) => part.trim() !== '').join(', ')
}

/** A name to offer when saving: the recipient's own, or the address's first line. */
function suggestedLabel(recipient: Recipient | null): string {
  if (!recipient) return ''
  if (recipient.kind === 'free') return recipient.value.split('\n')[0]?.trim() ?? ''
  return (recipient.value.name ?? '').trim()
}

type AddressBookMenuProps = {
  /** The card's country: `null` for a European card, which takes a typed address. */
  country: Country | null
  /** This card's recipient, to save and to mark. */
  current: Recipient | null
  /** Puts a saved or recent recipient on this card. */
  onPick: (recipient: Recipient) => void
}

/**
 * The postcard address book (FR-133, AddAssist 1.1), at the top of the Recipient block: a
 * picker with the recipients saved for this kind of card and, on a Japanese or Korean card,
 * those last used; each a click to put it on the card and × to remove it. Beside it, ☆ *Save*
 * keeps this card's recipient under a name. Every check is Rust's.
 */
export function AddressBookMenu({ country, current, onPick }: AddressBookMenuProps) {
  const { t } = useTranslation()
  const [bump, setBump] = useState(0)
  const [book, setBook] = useState<AddressBook | null>(null)
  const loaded = useLiveQuery('book', bump, loadBook).data
  const shown = book ?? loaded
  const [error, setError] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)
  const fits = (r: Recipient) => (country === null ? r.kind === 'free' : r.kind === 'assisted' && r.value.country === country)
  const saved = (shown?.saved ?? []).filter((s) => fits(s.recipient))
  const recent: Address[] = country === null ? [] : (shown?.recent ?? []).filter((a) => a.country === country)
  const same = (r: Recipient) => current !== null && rowText(r) === rowText(current)
  const answer = (result: Awaited<ReturnType<typeof commands.addressBook>>) => {
    if (result.status === 'ok') {
      setBook(result.data)
      setError(null)
      setBump((b) => b + 1)
      return true
    }
    setError(errorText(result.error))
    return false
  }
  const save = async (label: string) => {
    if (!current || label.trim() === '') return
    if (answer(await commands.saveAddress(label, current))) setSaving(false)
  }
  return (
    <Field>
      <div className="flex min-w-0 items-center gap-2">
        <Popover>
          <PopoverTrigger asChild>
            <PickerTrigger icon={<BookUser />} label={t('postcard.book.open', { count: saved.length + recent.length })} className="min-w-0 flex-1" />
          </PopoverTrigger>
          <MenuContent align="start">
            {saved.length === 0 && recent.length === 0 && <Hint>{t('postcard.book.empty')}</Hint>}
            {saved.length > 0 && <FieldLabel>{t('postcard.book.saved')}</FieldLabel>}
            {saved.map((s) => (
              <div key={s.id} className="flex items-center gap-2">
                <div className="min-w-0 flex-1" data-hint={rowText(s.recipient)}>
                  <ListRow icon={<Star />} chosen={same(s.recipient)} onClick={() => onPick(s.recipient)}>
                    {s.label}
                  </ListRow>
                </div>
                <RemoveButton iconOnly label={t('postcard.book.forget')} onClick={() => void commands.forgetSavedAddress(s.id).then(answer)} />
              </div>
            ))}
            {recent.length > 0 && <FieldLabel>{t('postcard.book.recent')}</FieldLabel>}
            {recent.map((a) => {
              const r: Recipient = { kind: 'assisted', value: a }
              return (
                <div key={rowText(r)} className="flex items-center gap-2">
                  <div className="min-w-0 flex-1" data-hint={rowText(r)}>
                    <ListRow icon={<History />} chosen={same(r)} onClick={() => onPick(r)}>
                      {rowText(r)}
                    </ListRow>
                  </div>
                  <RemoveButton iconOnly label={t('postcard.reuse.forget')} onClick={() => void commands.forgetRecentRecipient(a).then(answer)} />
                </div>
              )
            })}
          </MenuContent>
        </Popover>
        {/* Icon only, as every button in a row (STYLE_RULE §4), so the book's name is never cut
            in an 18.75 rem panel; its words are the hint. A disabled button's hint is on a wrapper. */}
        <span data-hint={t('postcard.book.saveHint')}>
          <Button variant="outline" size="icon-sm" disabled={!current} onClick={() => setSaving((s) => !s)} aria-label={t('postcard.book.save')}>
            <Star />
          </Button>
        </span>
      </div>
      {saving && current && (
        <CommitInput
          value={suggestedLabel(current)}
          placeholder={t('postcard.book.namePlaceholder')}
          ariaLabel={t('postcard.book.name')}
          maxLength={60}
          onCommit={(label) => void save(label)}
        />
      )}
      {error && <Hint className="text-destructive">{error}</Hint>}
    </Field>
  )
}
