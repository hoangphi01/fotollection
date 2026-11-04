// SPDX-License-Identifier: GPL-3.0-only
import { CircleAlert } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { cn } from '@/lib/utils'

type PostcardCutBadgeProps = {
  className?: string
}

/**
 * A postcard whose text does not all fit its lines (FR-073): the same red mark on the card
 * in the grid, on its preview and beside the part that is cut, so it is seen and fixed
 * before printing.
 */
export function PostcardCutBadge({ className }: PostcardCutBadgeProps) {
  const { t } = useTranslation()
  return (
    <span
      role="img"
      className={cn('inline-flex rounded-full bg-background text-destructive', className)}
      aria-label={t('postcard.cut')}
      data-hint={t('postcard.cutHint')}
    >
      <CircleAlert className="size-4" />
    </span>
  )
}
