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
 * on two lines at most (the whole name in the hint), raised as a button, the chosen one in the theme's colour,
 * pressed in, with the tick of select mode. The Looks card's grid and Simple mode's strip both use it.
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
          // Raised as an outline button (FR-099, STYLE_RULE §4 "Depth"); the chosen one in the
          // theme's colour and pressed in, as every chosen choice (owner, 2026-11-07).
          'block w-full min-w-0 overflow-hidden rounded-lg border p-1 text-left transition-all outline-none focus-visible:ring-3 focus-visible:ring-ring/50',
          chosen
            ? 'border-transparent bg-primary text-primary-foreground [--face:var(--primary)] pressed-in'
            : 'border-border bg-card [--face:var(--card)] raised hover:bg-muted active:raised-pressed dark:border-input dark:bg-secondary dark:[--face:var(--secondary)] dark:hover:bg-muted',
        )}
      >
        <img src={src} alt="" loading="lazy" className="aspect-square w-full rounded-md bg-muted object-cover" />
        {/* Two lines at most, always two high, so every row of tiles lines up. */}
        <span className="line-clamp-2 h-9 px-1 pt-1 text-xs leading-normal break-words">{name}</span>
      </button>
      {chosen && <TickCircle checked className="pointer-events-none absolute top-2 right-2" />}
      {corner && <span className="absolute top-2 left-2 opacity-0 group-focus-within:opacity-100 group-hover:opacity-100">{corner}</span>}
    </div>
  )
}

/** Tiles of looks, three to a row in an 18 rem card (FR-079). */
export function LookGrid({ children }: { children: ReactNode }) {
  return <div className="grid grid-cols-3 gap-2">{children}</div>
}
