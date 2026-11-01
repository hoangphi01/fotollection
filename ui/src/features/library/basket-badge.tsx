// SPDX-License-Identifier: GPL-3.0-only
import { ShoppingBasket } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { cn } from '@/lib/utils'

type BasketBadgeProps = {
  className?: string
}

/** A photo in the print basket (FR-050), the same badge on gallery tiles and sheets. */
export function BasketBadge({ className }: BasketBadgeProps) {
  const { t } = useTranslation()
  return (
    <span className={cn('rounded bg-primary p-0.5 text-primary-foreground', className)} aria-label={t('library.inBasket')}>
      <ShoppingBasket className="size-3" />
    </span>
  )
}
