// SPDX-License-Identifier: GPL-3.0-only
// Under Simple mode's stage only: a strip, not a card of a panel (STYLE_RULE §4).
import { ChevronDown } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Filmstrip } from '@/components/layout/filmstrip'
import { cn } from '@/lib/utils'
import { Kbd } from '@/components/ui/kbd'
import type { PhotoId } from '@/ipc/bindings'
import { lookTileUrl } from '@/lib/photo-url'
import { LOOK_KEYS, lookName, type LookPicker } from './use-looks'

type LookStripProps = {
  photoId: PhotoId
  picker: LookPicker
  /** Each tile with its key, 0–9 (Simple mode); the previews' digits rate or pick photos. */
  numbered?: boolean
  /** A header that folds it (the photo and sheet previews, owner 2026-11-07: a Pro user mostly
   * works from presets); Simple mode's stays open under its photo. */
  folding?: boolean
}

/** Whether the previews' strip of looks is open, kept while the app runs. */
let stripOpen = true

/**
 * Simple mode's looks (FR-070, FR-079): *look at the photo, choose a colour, keep it*. The
 * looks of the library as a strip under the photo, each tile the photo through it and its key
 * (0–9); one tap or one key gives this photo that look, the chosen one outlined, as the
 * filmstrip's photo is.
 */
export function LookStrip({ photoId, picker, numbered = true, folding = false }: LookStripProps) {
  const { t } = useTranslation()
  const [open, setOpen] = useState(stripOpen)
  const chosen = picker.current === null ? undefined : picker.looks[picker.current]
  return (
    <Filmstrip
      collapsed={folding && !open}
      header={
        folding ? (
          // The strip folds by its name, as a card by its title (STYLE_RULE §4).
          <button
            type="button"
            className="flex min-w-0 items-center gap-1 text-stage-ink"
            aria-expanded={open}
            onClick={() => {
              stripOpen = !open
              setOpen(!open)
            }}
          >
            <ChevronDown className={cn('size-3.5 shrink-0 motion-move', !open && '-rotate-90')} aria-hidden />
            <span className="truncate">
              {t('look.looks')}
              {chosen && <span className="text-stage-ink-muted"> · {lookName(chosen)}</span>}
            </span>
          </button>
        ) : undefined
      }
      items={picker.looks}
      current={picker.current}
      hint={(entry) => {
        const i = picker.looks.indexOf(entry)
        return numbered && i < LOOK_KEYS ? `${lookName(entry)} (${i})` : lookName(entry)
      }}
      // Read as the words the tile shows, its key told apart (UI-A11Y-002: the name holds the label).
      name={lookName}
      keys={(entry) => {
        const i = picker.looks.indexOf(entry)
        return numbered && i < LOOK_KEYS ? String(i) : undefined
      }}
      onPick={(_, index) => picker.pick(index)}
      cell={(entry) => {
        const i = picker.looks.indexOf(entry)
        return (
          <>
            <img src={lookTileUrl(photoId, entry.id)} alt="" loading="lazy" decoding="async" draggable={false} className="size-full object-cover" />
            {/* The key drawn by CSS, not a text node: seen on the tile, told to readers by
                `aria-keyshortcuts`, so the tile's name stays the words it shows (UI-A11Y-002). */}
            {numbered && i < LOOK_KEYS && (
              <Kbd aria-hidden data-key={i} className="absolute top-1 left-1 bg-stage-chip text-stage-ink after:content-[attr(data-key)]" />
            )}
            <span className="absolute inset-x-0 bottom-0 truncate bg-stage-chip px-1 text-xs text-stage-ink">{lookName(entry)}</span>
          </>
        )
      }}
    />
  )
}
