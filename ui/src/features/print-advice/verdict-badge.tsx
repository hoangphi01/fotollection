// SPDX-License-Identifier: GPL-3.0-only
import { CircleAlert, CircleCheck, TriangleAlert } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Badge } from '@/components/ui/badge'
import type { PrintVerdict } from '@/ipc/bindings'

/** Green prints sharp, amber is still acceptable, red will look pixelated (FR-026): solid,
 * raised badges of the shared kind (FR-099), each with an icon of its own shape, so the verdict
 * reads without its colour (STYLE_RULE §4: never colour alone). */
const VERDICT = {
  sharp: { variant: 'success', Icon: CircleCheck },
  acceptable: { variant: 'warning', Icon: CircleAlert },
  pixelated: { variant: 'destructive', Icon: TriangleAlert },
} as const

export function VerdictBadge({ verdict, dpi, className }: { verdict: PrintVerdict; dpi: number; className?: string }) {
  const { t } = useTranslation()
  const { variant, Icon } = VERDICT[verdict]
  return (
    <Badge variant={variant} data-verdict data-hint={t('advice.dpi', { dpi })} className={className}>
      <Icon data-icon="inline-start" aria-hidden />
      {t(`advice.verdict.${verdict}`)}
    </Badge>
  )
}
