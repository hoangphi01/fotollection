// SPDX-License-Identifier: GPL-3.0-only
import { CopyCheck } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { commands, type LookSettings, type PhotoId, type PhotoLook, type RawLook } from '@/ipc/bindings'
import { FavesCard } from './faves-card'
import { RawLookCard } from './raw-look-card'
import { ToolsCard } from './tools-card'
import { fullLook, type LookChange } from './look-fields'

type PhotoLookCardsProps = {
  photoId: PhotoId
  /** Bumps on every edit and undo, so the cards follow the photo's look. */
  revision: number
  /** After "Apply to all" changed the shared settings. */
  onAppliedToAll(): void
}

/**
 * The open photo's own look in the preview (FR-079): the same Faves and Tools cards as the
 * panel, changing this photo only, as its date stamp does (FR-044); *Apply to all* makes
 * its look the setting for every photo. A camera RAW first gets how it is finished
 * (FR-072, `RawLookCard`).
 */
export function PhotoLookCards({ photoId, revision, onAppliedToAll }: PhotoLookCardsProps) {
  const { t } = useTranslation()
  const [current, setCurrent] = useState<PhotoLook | null>(null)
  // Local changes so far: an older answer must not put an older look back (STYLE_RULE §4).
  const changes = useRef(0)

  useEffect(() => {
    let live = true
    const asked = changes.current
    void commands.getPhotoLook(photoId).then((result) => {
      if (live && result.status === 'ok' && asked === changes.current) setCurrent(result.data)
    })
    return () => {
      live = false
    }
  }, [photoId, revision])

  if (!current) return null
  const change: LookChange = (edit, first) => {
    const next: LookSettings = edit(fullLook(current.look))
    changes.current += 1
    setCurrent({ ...current, look: next, custom: true })
    // The first value of a gesture starts an undo step, the rest amend it (FR-023).
    void commands.setPhotoLook(photoId, next, !first)
  }
  const applyToAll = current.custom && (
    <Button
      variant="outline"
      size="sm"
      className="w-full"
      onClick={() => void commands.applyLookToAll(photoId).then((r) => r.status === 'ok' && onAppliedToAll())}
    >
      <CopyCheck data-icon="inline-start" /> {t('look.applyToAll')}
    </Button>
  )
  const setRaw = (look: RawLook) => {
    changes.current += 1
    setCurrent({ ...current, raw: look })
    void commands.setRawLook(photoId, look)
  }
  return (
    <>
      {current.raw != null && (
        <RawLookCard
          value={current.raw}
          onChange={setRaw}
          onApplyToAll={() => current.raw != null && void commands.applyRawLookToAll(current.raw)}
        />
      )}
      <FavesCard look={current.look} onChange={change} footer={applyToAll} />
      <ToolsCard look={current.look} onChange={change} />
    </>
  )
}
