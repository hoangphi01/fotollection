// SPDX-License-Identifier: GPL-3.0-only
import type { ReactNode } from 'react'
import { cn } from '@/lib/utils'

type CollapseProps = {
  open: boolean
  children: ReactNode
  /** Classes for the content (layout); the wrapper only animates the height. */
  className?: string
  /**
   * How its blocks are spaced: 6 px as the parts of one field, or 12 px as a card's blocks
   * (STYLE_RULE §4, the lego rule). One block needs neither.
   */
  stack?: 'field' | 'card'
}

/**
 * Options that only matter while a feature is on: they fold away when it is switched off,
 * so the panel stays short. Folded content is `inert`, so Tab never lands in it.
 */
export function Collapse({ open, children, className, stack }: CollapseProps) {
  return (
    <div
      className={cn(
        'grid transition-[grid-template-rows,opacity,margin] duration-200 ease-out motion-reduce:transition-none',
        open ? 'grid-rows-[1fr] opacity-100' : 'grid-rows-[0fr] opacity-0',
      )}
      // Folded, it would still take its stack's gap on both sides: it gives one back, so a
      // switched-off option leaves no gap (STYLE_RULE §4). Each stack sets `--stack-gap`.
      style={open ? undefined : { marginTop: 'calc(-1 * var(--stack-gap, 0px))' }}
      inert={!open}
      aria-hidden={!open}
    >
      {/* The 4 px inset keeps focus rings from being clipped by the overflow. */}
      <div
        className={cn(
          '-m-1 min-h-0 overflow-hidden p-1 [--stack-gap:0px]',
          stack === 'field' && 'space-y-1.5 [--stack-gap:--spacing(1.5)]',
          stack === 'card' && 'space-y-3 [--stack-gap:--spacing(3)]',
          className,
        )}
      >{children}</div>
    </div>
  )
}
