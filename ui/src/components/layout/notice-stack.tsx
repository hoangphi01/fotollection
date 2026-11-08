// SPDX-License-Identifier: GPL-3.0-only
import { Children, isValidElement, useLayoutEffect, useRef, useState, type ReactElement, type ReactNode } from 'react'
import { OctagonX } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Alert, AlertAction, AlertDescription } from '@/components/ui/alert'
import { motionMs } from '@/lib/motion'
import { cn } from '@/lib/utils'
import { RemoveButton } from '@/components/forms/danger-button'
import { Selectable } from '@/components/forms/field'

/**
 * The notices over the main view (an error, a camera session, a task's progress), stacked
 * like blocks: the stack owns every space between and around them, a notice owns none, so a
 * new one only joins the list (STYLE_RULE §4). Absent notices (`null`, `false`) take no room.
 * A notice comes in and goes out (FR-100): it fades and slides 4 px, and the stack grows or
 * folds with it, so what is below moves smoothly instead of jumping; a leaving notice stays
 * drawn for its exit, then goes. With none at all the stack folds away.
 */
export function NoticeStack({ children }: { children: ReactNode }) {
  const now = Children.toArray(children).filter(isValidElement) as ReactElement[]
  // The notices of the last render, read only in the effect, so one that leaves can still be
  // drawn on its way out with what it last showed.
  const last = useRef<ReactElement[]>([])
  const [gone, setGone] = useState<readonly { key: string; at: number; node: ReactElement }[]>([])
  const signature = now.map((c) => String(c.key)).join('|')
  // A notice gone since the last render keeps what it last showed for its way out; one that
  // comes back stops leaving. Runs before the paint, so no frame lacks it.
  useLayoutEffect(() => {
    const here = new Set(signature ? signature.split('|') : [])
    const left = last.current.flatMap((node, at) => (here.has(String(node.key)) ? [] : [{ key: String(node.key), at, node }]))
    setGone((g) => {
      const kept = g.filter((x) => !here.has(x.key))
      return left.length === 0 && kept.length === g.length ? g : [...kept, ...left]
    })
  }, [signature])
  // Then the last render's notices are this one's (no state: nothing to draw again).
  useLayoutEffect(() => {
    last.current = now
  })
  useLayoutEffect(() => {
    if (gone.length === 0) return
    const timer = window.setTimeout(() => setGone([]), motionMs('motion-base-out'))
    return () => window.clearTimeout(timer)
  }, [gone])
  const open = now.length > 0
  if (!open && gone.length === 0) return null
  // Each leaving notice back where it was, so the others do not jump before it folds.
  const items: { key: string; node: ReactElement; leaving: boolean }[] = now.map((node) => ({ key: String(node.key), node, leaving: false }))
  for (const g of [...gone].sort((x, y) => x.at - y.at)) items.splice(Math.min(g.at, items.length), 0, { key: g.key, node: g.node, leaving: true })
  return (
    // The fold: rows from 0 to their height, as Collapse (`motion-fold`).
    <div className={cn('grid motion-fold', open ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]')}>
      <div className="min-h-0 overflow-hidden">
        {/* At least as tall as the side columns' heads with their line (64 + 1 px), one notice
            in its middle, so a task's progress lines up with them (owner, 2026-11-07). */}
        <div className="flex min-h-[calc(var(--spacing-header)+1px)] flex-col justify-center gap-2 border-b p-3">
          {items.map(({ key, node, leaving }) => (
            <div
              key={key}
              className={cn(
                'grid motion-fold',
                leaving
                  ? 'grid-rows-[0fr] animate-out fill-mode-forwards fade-out-0 slide-out-to-top-1 motion-out'
                  : 'grid-rows-[1fr] animate-in fade-in-0 slide-in-from-top-1 motion-in',
              )}
            >
              <div className="min-h-0">{node}</div>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
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
      <AlertDescription>
        <Selectable>{message}</Selectable>
      </AlertDescription>
      <AlertAction>
        <RemoveButton iconOnly label={t('common.dismiss')} onClick={onDismiss} />
      </AlertAction>
    </Alert>
  )
}
