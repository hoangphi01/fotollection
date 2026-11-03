// SPDX-License-Identifier: GPL-3.0-only
import { LoaderCircle, OctagonX } from 'lucide-react'
import { useTranslation } from 'react-i18next'

/**
 * "Rendering print preview…" on the neutral stage, while the first render loads; with
 * `error`, why it failed (its `ERR-` code included), so the stage never spins for good.
 */
export function StageSpinner({ error }: { error?: string | null }) {
  const { t } = useTranslation()
  if (error) {
    return (
      <div role="alert" className="absolute flex max-w-sm flex-col items-center gap-2 px-6 text-center text-white/80">
        <OctagonX className="size-7 text-destructive" />
        <span className="text-sm">{error}</span>
      </div>
    )
  }
  return (
    <div className="absolute flex flex-col items-center gap-2 text-white/60">
      <LoaderCircle className="size-7 animate-spin" />
      <span className="text-xs">{t('viewer.rendering')}</span>
    </div>
  )
}
