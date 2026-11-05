// SPDX-License-Identifier: GPL-3.0-only
import { useTranslation } from 'react-i18next'

/** A look card's title, with how many of its sliders are set, so a folded card still says. */
export function LookCardTitle({ title, changed }: { title: string; changed: number }) {
  const { t } = useTranslation()
  return (
    <span className="flex items-center gap-1.5">
      {title}
      {changed > 0 && (
        <span className="text-xs font-normal text-muted-foreground">· {t('look.changed', { count: changed })}</span>
      )}
    </span>
  )
}
