// SPDX-License-Identifier: GPL-3.0-only
import { CopyCheck } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Hint } from '@/components/forms/field'
import { PanelNote } from '@/components/layout/panel-note'
import { Button } from '@/components/ui/button'
import type { PhotoId, TextGaps } from '@/ipc/bindings'
import type { ReuseApply } from './postcard-reuse'

type PostcardReuseNoteProps = {
  photo: PhotoId
  /** The other cards: how many lack this card's part, how many have another. */
  gaps: TextGaps | null | undefined
  /** Ticked photos narrow the cards it goes to. */
  ticked: readonly PhotoId[]
  apply: ReuseApply
}

/**
 * One card's message or recipient for a whole set (FR-077, FR-133): a note on the other cards
 * with *Use for them* (those without one) and *Use for all N* (replacing theirs), one undo
 * step, then how many changed. The same note under the message, a European address and a
 * Japanese or Korean recipient.
 */
export function PostcardReuseNote({ photo, gaps, ticked, apply }: PostcardReuseNoteProps) {
  const { t } = useTranslation()
  const [done, setDone] = useState<{ photo: PhotoId; count: number } | null>(null)
  const [error, setError] = useState<string | null>(null)
  const blank = gaps?.blank ?? 0
  const differ = gaps?.differ ?? 0
  const scope = ticked.length > 0 ? 'Ticked' : 'All'
  const run = async (overwrite: boolean) => {
    setError(null)
    const result = await apply(overwrite)
    if (typeof result === 'number') setDone({ photo, count: result })
    else setError(result)
  }
  return (
    <>
      {/* A note that offers an action, its two choices as buttons, so the words wrap in an
          18.75 rem panel instead of being cut (STYLE_RULE §4). */}
      {(blank > 0 || differ > 0) && (
        <PanelNote
          icon={<CopyCheck />}
          action={
            <>
              {blank > 0 && (
                <Button size="sm" onClick={() => void run(false)}>
                  {t('postcard.reuse.fill')}
                </Button>
              )}
              {differ > 0 && (
                <Button variant="outline" size="sm" onClick={() => void run(true)}>
                  {t('postcard.reuse.replace', { count: blank + differ })}
                </Button>
              )}
            </>
          }
        >
          {blank > 0 ? t(`postcard.reuse.blank${scope}`, { count: blank }) : t(`postcard.reuse.differ${scope}`, { count: differ })}
        </PanelNote>
      )}
      {done?.photo === photo && done.count > 0 && <Hint>{t('postcard.reuse.done', { count: done.count })}</Hint>}
      {error && <Hint className="text-destructive">{error}</Hint>}
    </>
  )
}
