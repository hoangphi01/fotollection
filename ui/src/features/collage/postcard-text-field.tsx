// SPDX-License-Identifier: GPL-3.0-only
import { Bold, Italic, Underline } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { CommitInput } from '@/components/forms/commit-input'
import { Field, FieldLabel, Hint } from '@/components/forms/field'
import { Toggle } from '@/components/ui/toggle'
import { commands, type CullFilter, type PhotoId, type PostcardPart, type TextStyle } from '@/ipc/bindings'
import { useLiveQuery } from '@/features/settings/use-ipc-query'
import { PostcardCutBadge } from './postcard-cut-badge'
import { PostcardTextReuse } from './postcard-text-reuse'

/** The limits Rust checks for each part (`PostcardPart::limits`). */
const LIMITS: Record<PostcardPart, { chars: number; lines: number }> = {
  message: { chars: 400, lines: 8 },
  address: { chars: 200, lines: 4 },
}

const STYLES = [
  { key: 'bold', icon: Bold },
  { key: 'italic', icon: Italic },
  { key: 'underline', icon: Underline },
] as const

const load = async (subject: string) => {
  const result = await commands.getPostcardText(Number(subject))
  return result.status === 'ok' ? result.data : null
}

type PostcardTextFieldProps = {
  photo: PhotoId
  part: PostcardPart
  revision: number
  /** How this part is set on every card, and the change of it. */
  style: TextStyle | undefined
  onStyle: (style: TextStyle) => void
  /** This part does not all fit its lines and is cut on the print. */
  cut: boolean
  /** The postcards shown, and those ticked: an address goes to them in one move (FR-077). */
  filter?: CullFilter
  ticked?: readonly PhotoId[]
}

const NO_TICKS: readonly PhotoId[] = []

/**
 * One part of a postcard's text (FR-073), the message or the address, typed for the chosen
 * card, with its bold, italic and underline for every card. It goes onto its lines as the
 * user pauses typing; what does not fit them is cut on the print, as the card shows.
 */
export function PostcardTextField({
  photo,
  part,
  revision,
  style,
  onStyle,
  cut,
  filter,
  ticked = NO_TICKS,
}: PostcardTextFieldProps) {
  const { t } = useTranslation()
  const { chars, lines } = LIMITS[part]
  // The last answer holds its place while the next loads (LESSONS).
  const text = useLiveQuery(String(photo), revision, load).data?.[part] ?? ''
  return (
    <Field>
      <div className="flex items-center justify-between gap-2">
        <FieldLabel>{t(`postcard.${part}`)}</FieldLabel>
        {cut && <PostcardCutBadge />}
        <span className="flex-1 text-right text-xs text-muted-foreground tabular-nums">
          {t('postcard.chars', { count: text.length, max: chars })}
        </span>
        <div className="flex gap-1" role="group" aria-label={t('postcard.style')}>
          {STYLES.map(({ key, icon: Icon }) => (
            <Toggle
              key={key}
              variant="outline"
              size="sm"
              pressed={style?.[key] ?? false}
              onPressedChange={(pressed) =>
                onStyle({ bold: false, italic: false, underline: false, ...style, [key]: pressed })
              }
              aria-label={t(`postcard.styles.${key}`)}
              data-hint={t(`postcard.styles.${key}`)}
            >
              <Icon />
            </Toggle>
          ))}
        </div>
      </div>
      <CommitInput
        multiline
        value={text}
        placeholder={t(`postcard.${part}Placeholder`)}
        ariaLabel={t(`postcard.${part}`)}
        maxLength={chars}
        limit={(v) => v.split('\n').slice(0, lines).join('\n')}
        liveMs={600}
        onCommit={(next) => void commands.setPostcardText(photo, part, next.trim() === '' ? null : next)}
        // The address is offered on the next cards once it is typed, never its drafts (FR-077).
        onDone={part === 'address' ? (done) => done.trim() !== '' && void commands.rememberPostcardAddress(done) : undefined}
      />
      {cut && <Hint className="text-destructive">{t('postcard.cutHint')}</Hint>}
      {part === 'address' && filter && (
        <PostcardTextReuse
          filter={filter}
          photo={photo}
          part={part}
          text={text}
          ticked={ticked}
          revision={revision}
          onPick={(address) => void commands.setPostcardText(photo, part, address)}
        />
      )}
    </Field>
  )
}
