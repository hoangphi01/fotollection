// SPDX-License-Identifier: GPL-3.0-only
import { LoaderCircle, type LucideProps } from 'lucide-react'
import { cn } from '@/lib/utils'

const SIZE = { icon: '', tile: 'size-5', stage: 'size-7' } as const

/**
 * Work that runs (FR-100, docs/TRANS_ANIMATION.md): one turning ring, linear, the only loop the
 * app has; it keeps turning with motion reduced, as it says something is happening. In a
 * button or a line it takes the icon size its block gives (`icon`); over a tile or on the
 * stage it has its own (`tile`, `stage`). A wait that has a shape shows a skeleton instead.
 */
export function Spinner({ size = 'icon', className, ...props }: LucideProps & { size?: keyof typeof SIZE }) {
  return <LoaderCircle {...props} className={cn('animate-spin', SIZE[size], className)} />
}
