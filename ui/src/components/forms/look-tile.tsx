// SPDX-License-Identifier: GPL-3.0-only
import type { ReactNode } from 'react'
import { TickCircle } from '@/features/library/tick-circle'
import { cn } from '@/lib/utils'

type LookTileProps = {
  /** The photo through the look, small and square. */
  src: string
  name: string
  chosen: boolean
  onPick(): void
  /** Over the tile's top-left corner while it is hovered: a user look's remove button. */
  corner?: ReactNode
}

/**
 * One look as a tile (FR-079, STYLE_RULE §4): the photo through it, square, its name under it
 * on two lines at most (the whole name in the hint), the chosen one ringed in the theme's colour with
 * the tick of select mode. The Looks card's grid and Simple mode's strip both use it.
 */
export function LookTile({ src, name, chosen, onPick, corner }: LookTileProps) {
  return (
    <div className="group relative min-w-0">
      <button
        type="button"
        onClick={onPick}
        aria-pressed={chosen}
        data-hint={name}
        className={cn(
          'block w-full min-w-0 overflow-hidden rounded-md border text-left outline-none focus-visible:ring-2 focus-visible:ring-primary',
          chosen && 'ring-2 ring-primary',
        )}
      >
        <img src={src} alt="" loading="lazy" className="aspect-square w-full bg-muted object-cover" />
        {/* Two lines at most, always two high, so every row of tiles lines up. */}
        <span className="line-clamp-2 h-9 px-1.5 pt-0.5 text-xs leading-normal break-words">{name}</span>
      </button>
      {chosen && <TickCircle checked className="pointer-events-none absolute top-1 right-1" />}
      {corner && <span className="absolute top-1 left-1 opacity-0 group-focus-within:opacity-100 group-hover:opacity-100">{corner}</span>}
    </div>
  )
}

/** Tiles of looks, three to a row in an 18 rem card (FR-079). */
export function LookGrid({ children }: { children: ReactNode }) {
  return <div className="grid grid-cols-3 gap-2">{children}</div>
}
