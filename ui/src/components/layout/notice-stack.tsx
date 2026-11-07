// SPDX-License-Identifier: GPL-3.0-only
import { Children, type ReactNode } from 'react'
import { OctagonX } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Alert, AlertAction, AlertDescription } from '@/components/ui/alert'
import { cn } from '@/lib/utils'
import { RemoveButton } from '@/components/forms/danger-button'

/**
 * The notices over the main view (an error, a camera session, a task's progress), stacked
 * like blocks: the stack owns every space between and around them, a notice owns none, so a
 * new one only joins the list (STYLE_RULE §4). Absent notices (`null`, `false`) take no room,
 * and with none at all the stack is not drawn.
 */
export function NoticeStack({ children }: { children: ReactNode }) {
  const notices = Children.toArray(children)
  if (notices.length === 0) return null
  // At least as tall as the side columns' heads with their line (64 + 1 px), one notice in
  // its middle, so a task's progress lines up with them (owner, 2026-11-07).
  return <div className="flex min-h-[calc(var(--spacing-header)+1px)] flex-col justify-center gap-2 border-b p-3">{notices}</div>
}

/** The shell every notice that is not an `Alert` wears, so all of them line up. */
export function NoticeCard({ className, ...props }: React.ComponentProps<'div'>) {
  return <div className={cn('rounded-lg border bg-card px-2.5 py-2 text-sm lifted', className)} {...props} />
}

/** An error in the stack, with its `ERR-` code in the text and a button that hides it. */
export function ErrorNotice({ message, onDismiss }: { message: string; onDismiss: () => void }) {
  const { t } = useTranslation()
  return (
    <Alert variant="destructive">
      <OctagonX />
      <AlertDescription>{message}</AlertDescription>
      <AlertAction>
        <RemoveButton iconOnly label={t('common.dismiss')} onClick={onDismiss} />
      </AlertAction>
    </Alert>
  )
}
