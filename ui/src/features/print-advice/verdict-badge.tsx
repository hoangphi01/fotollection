// SPDX-License-Identifier: GPL-3.0-only
import { useTranslation } from 'react-i18next'
import type { PrintVerdict } from '@/ipc/bindings'
import { cn } from '@/lib/utils'

/** Green prints sharp, yellow is still acceptable, red will look pixelated (FR-026). */
const STYLE: Record<PrintVerdict, string> = {
  sharp: 'bg-emerald-500/15 text-emerald-700 dark:text-emerald-400',
  acceptable: 'bg-amber-500/15 text-amber-700 dark:text-amber-400',
  pixelated: 'bg-red-500/15 text-red-700 dark:text-red-400',
}

export function VerdictBadge({ verdict, dpi, className }: { verdict: PrintVerdict; dpi: number; className?: string }) {
  const { t } = useTranslation()
  return (
    <span
      title={t('advice.dpi', { dpi })}
      className={cn('rounded px-1.5 py-px text-[10px] font-medium whitespace-nowrap', STYLE[verdict], className)}
    >
      {t(`advice.verdict.${verdict}`)}
    </span>
  )
}
