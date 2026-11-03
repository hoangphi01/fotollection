// SPDX-License-Identifier: GPL-3.0-only
import { useTranslation } from 'react-i18next'
import { CommitInput } from '@/components/forms/commit-input'
import { Label } from '@/components/ui/label'
import { commands, type PhotoId } from '@/ipc/bindings'
import { useLiveQuery } from '@/features/settings/use-ipc-query'

/** The limits Rust checks (`settings::MAX_POSTCARD_CHARS`, `MAX_POSTCARD_LINES`). */
const MAX_CHARS = 400
const MAX_LINES = 8

const load = async (subject: string) => {
  const result = await commands.getPostcardText(Number(subject))
  return result.status === 'ok' ? (result.data ?? '') : ''
}

/** At most `MAX_LINES` lines, as it is typed. */
const withinLines = (text: string) => text.split('\n').slice(0, MAX_LINES).join('\n')

/**
 * The text typed on one postcard (FR-073), in its sheet's preview: a message, an address,
 * or nothing for handwriting. It goes onto the card's lines as the user pauses typing; what
 * does not fit the lines is cut on the print, so the card shows it as it will print.
 */
export function PostcardTextField({ photo, revision }: { photo: PhotoId; revision: number }) {
  const { t } = useTranslation()
  // The last answer holds its place while the next loads (LESSONS).
  const text = useLiveQuery(String(photo), revision, load).data ?? ''
  return (
    <div className="space-y-1.5">
      <div className="flex items-baseline justify-between gap-2">
        <Label className="text-xs font-normal text-muted-foreground">{t('postcard.text')}</Label>
        <span className="text-xs text-muted-foreground tabular-nums">
          {t('postcard.chars', { count: text.length, max: MAX_CHARS })}
        </span>
      </div>
      <CommitInput
        multiline
        value={text}
        placeholder={t('postcard.textPlaceholder')}
        ariaLabel={t('postcard.text')}
        maxLength={MAX_CHARS}
        limit={withinLines}
        liveMs={600}
        onCommit={(next) => void commands.setPostcardText(photo, next.trim() === '' ? null : next)}
      />
      <p className="text-xs text-muted-foreground">{t('postcard.textHint')}</p>
    </div>
  )
}
