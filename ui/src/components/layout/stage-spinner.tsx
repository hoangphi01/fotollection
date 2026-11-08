// SPDX-License-Identifier: GPL-3.0-only
import { OctagonX } from 'lucide-react'
import { Spinner } from '@/components/forms/spinner'
import { useTranslation } from 'react-i18next'
import { Selectable } from '@/components/forms/field'

/**
 * "Rendering print preview…" on the neutral stage, while the first render loads; with
 * `error`, why it failed (its `ERR-` code included), so the stage never spins for good;
 * `label` says what else it waits for (the camera's live view).
 */
export function StageSpinner({ error, label }: { error?: string | null; label?: string }) {
  const { t } = useTranslation()
  if (error) {
    return (
      <div role="alert" className="absolute flex max-w-sm flex-col items-center gap-2 px-6 text-center text-stage-ink">
        <OctagonX className="size-7 text-destructive" />
        <Selectable className="text-sm">{error}</Selectable>
      </div>
    )
  }
  return (
    <div className="absolute flex flex-col items-center gap-2 text-stage-ink-muted">
      <Spinner size="stage" />
      <span className="text-xs">{label ?? t('viewer.rendering')}</span>
    </div>
  )
}
