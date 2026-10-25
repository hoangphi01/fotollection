// SPDX-License-Identifier: GPL-3.0-only
import type { ReactNode } from 'react'
import { Collapse } from '@/components/layout/collapse'
import { cn } from '@/lib/utils'

type SectionCardProps = {
  icon: ReactNode
  /** Plain text or a `<Label>` tied to the section's switch. */
  title: ReactNode
  /** The section's master switch, or another control, at the right of the header. */
  action?: ReactNode
  /** False folds the body away (a switched-off feature, STYLE_RULE §4). */
  open?: boolean
  /** Always shown under the body, even when it is folded (for example "Apply to all"). */
  footer?: ReactNode
  children?: ReactNode
  className?: string
}

/**
 * One settings section as a card with a tinted header: icon, title and master switch on the
 * same strip, so sections read apart at a glance (STYLE_RULE §4). Used by the settings
 * panel and the preview inspector alike.
 */
export function SectionCard({ icon, title, action, open = true, footer, children, className }: SectionCardProps) {
  const hasBody = children !== undefined && children !== null && children !== false
  return (
    <section className={cn('overflow-hidden rounded-xl border-[1.5px] bg-card text-card-foreground shadow-xs', className)}>
      <header
        className={cn(
          'flex min-h-11 items-center gap-2 bg-muted/70 px-3 py-2 text-sm font-semibold',
          ((hasBody && open) || footer) && 'border-b',
        )}
      >
        <span className="flex text-primary [&_svg]:size-4">{icon}</span>
        <div className="min-w-0 flex-1 truncate">{title}</div>
        {action && <div className="flex shrink-0 items-center gap-2">{action}</div>}
      </header>
      {hasBody && (
        <Collapse open={open}>
          <div className={cn('space-y-3 p-3 text-sm', footer && 'border-b')}>{children}</div>
        </Collapse>
      )}
      {footer && <div className="p-3">{footer}</div>}
    </section>
  )
}
