// SPDX-License-Identifier: GPL-3.0-only
// @shown preview
import { Check, History } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Hint } from '@/components/forms/field'
import { ScrollArea } from '@/components/ui/scroll-area'
import { SectionCard } from '@/components/layout/section-card'
import { commands, type HistoryEdit, type HistoryStep, type PhotoId } from '@/ipc/bindings'
import { ListRow, ListRows } from '@/components/forms/list-row'

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
    // A look of the library picked names it (FR-079); the amount and the sliders their value.
    if (edit.name) return t('history.lookPicked', { name: edit.name })
    if (!edit.field || edit.value === null) return t('history.look')
    if (edit.field === 'amount') return `${t('look.amount')} ${edit.value} %`
    const value = edit.value > 0 ? `+${edit.value}` : String(edit.value)
    return `${t(`look.${edit.field}`, { defaultValue: t('history.look') })} ${value}`
  }
  const restore = (serial: number | null) => void commands.restorePhotoState(photoId, serial)

  return (
    <SectionCard icon={<History />} title={t('history.title')}>
      {steps && steps.length === 0 ? (
        <Hint>{t('history.empty')}</Hint>
      ) : (
        // About six rows, the rest scrolled inside, so a long History never pushes the
        // column (owner, 2026-11-07: "not as long as a scroll of paper").
        <ScrollArea className="max-h-52 [&>[data-slot=scroll-area-viewport]>div]:block!">
        <ListRows as="ul">
          {(steps ?? []).map((step, i) => (
            <li key={step.serial}>
              <ListRow
                icon={i === 0 ? <Check /> : undefined}
                chosen={i === 0}
                current="step"
                onClick={() => i > 0 && restore(step.serial)}
              >
                {step.kind ? t(`history.${step.kind}`) : step.edits.map(name).join(' · ')}
              </ListRow>
            </li>
          ))}
          {steps && steps.length > 0 && (
            <li>
              <ListRow muted onClick={() => restore(null)}>
                {t('history.start')}
              </ListRow>
            </li>
          )}
        </ListRows>
        </ScrollArea>
      )}
    </SectionCard>
  )
}
