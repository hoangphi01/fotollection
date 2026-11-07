// SPDX-License-Identifier: GPL-3.0-only
// Under Simple mode's stage only: a strip, not a card of a panel (STYLE_RULE §4).
import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Filmstrip } from '@/components/layout/filmstrip'
import { commands, type PhotoId } from '@/ipc/bindings'
import { lookTileUrl } from '@/lib/photo-url'
import { lookName, useLooks } from './use-looks'

type LookStripProps = {
  photoId: PhotoId
  /** Bumps on every edit and undo, so the strip marks the photo's look as it is. */
  revision: number
}

/**
 * Simple mode's looks (FR-070, FR-079): *look at the photo, choose a colour, keep it*. The
 * looks of the library as a strip under the photo, each tile the photo through it; one tap
 * gives this photo that look, the chosen one outlined, as the filmstrip's photo is.
 */
export function LookStrip({ photoId, revision }: LookStripProps) {
  const { t } = useTranslation()
  const { looks } = useLooks()
  const [base, setBase] = useState<string | null>(null)
  const picks = useRef(0)
  useEffect(() => {
    let live = true
    const asked = picks.current
    void commands.getPhotoLook(photoId).then((r) => {
      if (live && r.status === 'ok' && asked === picks.current) setBase(r.data.look.base ?? null)
    })
    return () => {
      live = false
    }
  }, [photoId, revision])
  const current = looks.findIndex((l) => l.id === base)
  return (
    <Filmstrip
      label={t('look.looks')}
      items={looks}
      current={current < 0 ? null : current}
      hint={lookName}
      onPick={(entry) => {
        picks.current += 1
        setBase(entry.id)
        void commands.setPhotoLook(photoId, { ...entry.look, name: lookName(entry) }, false)
      }}
      cell={(entry) => (
        <>
          <img src={lookTileUrl(photoId, entry.id)} alt="" loading="lazy" decoding="async" draggable={false} className="size-full object-cover" />
          <span className="absolute inset-x-0 bottom-0 truncate bg-stage-chip px-1 text-xs text-stage-ink">{lookName(entry)}</span>
        </>
      )}
    />
  )
}
