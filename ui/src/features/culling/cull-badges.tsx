// SPDX-License-Identifier: GPL-3.0-only
import { Flag as FlagIcon, FlagOff, Star } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import type { Flag } from '@/ipc/bindings'
import { cn } from '@/lib/utils'

type CullBadgesProps = {
  rating: number
  flag: Flag | null
  className?: string
}

/** Stars and pick/reject flag of one photo (FR-022). Renders nothing for an unrated photo. */
export function CullBadges({ rating, flag, className }: CullBadgesProps) {
  const { t } = useTranslation()
  if (rating === 0 && flag === null) return null
  return (
    <div className={cn('flex items-center gap-1', className)}>
      {flag === 'pick' && (
        <span className="rounded bg-black/60 p-0.5 text-emerald-300" aria-label={t('culling.pick')}>
          <FlagIcon className="size-3 fill-current" />
        </span>
      )}
      {flag === 'reject' && (
        <span className="rounded bg-black/60 p-0.5 text-red-300" aria-label={t('culling.rejected')}>
          <FlagOff className="size-3" />
        </span>
      )}
      {rating > 0 && (
        <span
          className="flex items-center gap-px rounded bg-black/60 px-1 py-0.5 text-amber-300"
          aria-label={t('culling.stars', { count: rating })}
        >
          {Array.from({ length: rating }, (_, i) => (
            <Star key={i} className="size-2.5 fill-current" />
          ))}
        </span>
      )}
    </div>
  )
}
