// SPDX-License-Identifier: GPL-3.0-only
import { useTranslation } from 'react-i18next'
import { Badge } from '@/components/ui/badge'
import type { PrintVerdict } from '@/ipc/bindings'

/** Green prints sharp, amber is still acceptable, red will look pixelated (FR-026): solid,
 * raised badges of the shared kind (FR-099), small enough to sit in a line of text. */
const VARIANT = { sharp: 'success', acceptable: 'warning', pixelated: 'destructive' } as const

export function VerdictBadge({ verdict, dpi, className }: { verdict: PrintVerdict; dpi: number; className?: string }) {
  const { t } = useTranslation()
  return (
    <Badge variant={VARIANT[verdict]} data-hint={t('advice.dpi', { dpi })} className={className}>
      {t(`advice.verdict.${verdict}`)}
    </Badge>
  )
}
