// SPDX-License-Identifier: GPL-3.0-only
// Under Simple mode's stage only: a strip, not a card of a panel (STYLE_RULE §4).
import { Filmstrip } from '@/components/layout/filmstrip'
import { Kbd } from '@/components/ui/kbd'
import type { PhotoId } from '@/ipc/bindings'
import { lookTileUrl } from '@/lib/photo-url'
import { LOOK_KEYS, lookName, type LookPicker } from './use-looks'

type LookStripProps = {
  photoId: PhotoId
  picker: LookPicker
}

/**
 * Simple mode's looks (FR-070, FR-079): *look at the photo, choose a colour, keep it*. The
 * looks of the library as a strip under the photo, each tile the photo through it and its key
 * (0–9); one tap or one key gives this photo that look, the chosen one outlined, as the
 * filmstrip's photo is.
 */
export function LookStrip({ photoId, picker }: LookStripProps) {
  return (
    <Filmstrip
      items={picker.looks}
      current={picker.current}
      hint={(entry) => {
        const i = picker.looks.indexOf(entry)
        return i < LOOK_KEYS ? `${lookName(entry)} (${i})` : lookName(entry)
      }}
      onPick={(_, index) => picker.pick(index)}
      cell={(entry) => {
        const i = picker.looks.indexOf(entry)
        return (
          <>
            <img src={lookTileUrl(photoId, entry.id)} alt="" loading="lazy" decoding="async" draggable={false} className="size-full object-cover" />
            {i < LOOK_KEYS && <Kbd className="absolute top-1 left-1 bg-stage-chip text-stage-ink">{i}</Kbd>}
            <span className="absolute inset-x-0 bottom-0 truncate bg-stage-chip px-1 text-xs text-stage-ink">{lookName(entry)}</span>
          </>
        )
      }}
    />
  )
}
