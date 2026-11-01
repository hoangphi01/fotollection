// SPDX-License-Identifier: GPL-3.0-only
import { LoaderCircle } from 'lucide-react'
import { useTranslation } from 'react-i18next'

/** "Rendering print preview…" on the neutral stage, while the first render loads. */
export function StageSpinner() {
  const { t } = useTranslation()
  return (
    <div className="absolute flex flex-col items-center gap-2 text-white/60">
      <LoaderCircle className="size-7 animate-spin" />
      <span className="text-xs">{t('viewer.rendering')}</span>
    </div>
  )
}
