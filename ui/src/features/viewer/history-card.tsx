// SPDX-License-Identifier: GPL-3.0-only
// @shown preview
import { Check, History } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Hint, RowList } from '@/components/forms/field'
import { SectionCard } from '@/components/layout/section-card'
import { Button } from '@/components/ui/button'
import { commands, type HistoryEdit, type HistoryStep, type PhotoId } from '@/ipc/bindings'
import { cn } from '@/lib/utils'

type HistoryCardProps = {
  photoId: PhotoId
  /** Bumps on every edit, undo and redo, so the list follows. */
  revision: number
}

/**
 * The develop view's History (FR-095), as Lightroom's: this photo's steps newest first, each
 * named by what it changed (*Warm / cool +8*, *Crop*). The first is where the photo is now; a
 * click on another brings the photo back to it as a new step, so ⌘Z undoes that too. Ratings
 * and flags are not steps here, and going back never changes them.
 */
export function HistoryCard({ photoId, revision }: HistoryCardProps) {
  const { t } = useTranslation()
  const [steps, setSteps] = useState<HistoryStep[] | null>(null)

  useEffect(() => {
    let live = true
    void commands.photoHistory(photoId).then((r) => live && r.status === 'ok' && setSteps(r.data))
    return () => {
      live = false
    }
  }, [photoId, revision])

  const name = (edit: HistoryEdit): string => {
    if (edit.kind !== 'look') return t(`history.${edit.kind}`)
    if (!edit.field || edit.value === null) return t('history.look')
    const value = edit.value > 0 ? `+${edit.value}` : String(edit.value)
    return `${t(`look.${edit.field}`, { defaultValue: t('history.look') })} ${value}`
  }
  const restore = (serial: number | null) => void commands.restorePhotoState(photoId, serial)

  return (
    <SectionCard icon={<History />} title={t('history.title')}>
      {steps && steps.length === 0 ? (
        <Hint>{t('history.empty')}</Hint>
      ) : (
        <RowList>
          {(steps ?? []).map((step, i) => (
            <li key={step.serial}>
              <Button
                variant="ghost"
                size="sm"
                aria-current={i === 0 ? 'step' : undefined}
                className={cn('w-full justify-start font-normal', i === 0 && 'bg-accent font-medium')}
                onClick={() => i > 0 && restore(step.serial)}
              >
                {i === 0 ? <Check data-icon="inline-start" /> : <span className="size-4" aria-hidden />}
                <span className="truncate">{step.edits.map(name).join(' · ')}</span>
              </Button>
            </li>
          ))}
          {steps && steps.length > 0 && (
            <li>
              <Button variant="ghost" size="sm" className="w-full justify-start font-normal text-muted-foreground" onClick={() => restore(null)}>
                <span className="size-4" aria-hidden />
                {t('history.start')}
              </Button>
            </li>
          )}
        </RowList>
      )}
    </SectionCard>
  )
}
