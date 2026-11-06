// SPDX-License-Identifier: GPL-3.0-only
import { Children, type ReactNode } from 'react'
import { OctagonX, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Alert, AlertAction, AlertDescription } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

/**
 * The notices over the main view (an error, a camera session, a task's progress), stacked
 * like blocks: the stack owns every space between and around them, a notice owns none, so a
 * new one only joins the list (STYLE_RULE §4). Absent notices (`null`, `false`) take no room,
 * and with none at all the stack is not drawn.
 */
export function NoticeStack({ children }: { children: ReactNode }) {
  const notices = Children.toArray(children)
  if (notices.length === 0) return null
  return <div className="flex flex-col gap-2 border-b p-3">{notices}</div>
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
        <Button variant="ghost" size="icon-sm" aria-label={t('common.dismiss')} onClick={onDismiss}>
          <X />
        </Button>
      </AlertAction>
    </Alert>
  )
}
