// SPDX-License-Identifier: GPL-3.0-only
import type { ReactNode } from 'react'
import { cn } from '@/lib/utils'

type CollapseProps = {
  open: boolean
  children: ReactNode
  /** Classes for the content (spacing, layout); the wrapper only animates the height. */
  className?: string
}

/**
 * Options that only matter while a feature is on: they fold away when it is switched off,
 * so the panel stays short. Folded content is `inert`, so Tab never lands in it.
 */
export function Collapse({ open, children, className }: CollapseProps) {
  return (
    <div
      className={cn(
        'grid transition-[grid-template-rows,opacity] duration-200 ease-out motion-reduce:transition-none',
        open ? 'grid-rows-[1fr] opacity-100' : 'grid-rows-[0fr] opacity-0',
      )}
      inert={!open}
      aria-hidden={!open}
    >
      {/* The 4 px inset keeps focus rings from being clipped by the overflow. */}
      <div className={cn('-m-1 min-h-0 overflow-hidden p-1', className)}>{children}</div>
    </div>
  )
}
