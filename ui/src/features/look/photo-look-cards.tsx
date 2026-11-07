// SPDX-License-Identifier: GPL-3.0-only
// @shown preview: in the preview only (STYLE_RULE §4, where each card shows, checked by check-lego).
import { CopyCheck, Images } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { PanelNote } from '@/components/layout/panel-note'
import { Button } from '@/components/ui/button'
import { commands, type LookSettings, type PhotoId, type PhotoLook, type RawLook } from '@/ipc/bindings'
import { FavesCard } from './faves-card'
import { LooksCard } from './looks-card'
import { RawLookCard } from './raw-look-card'
import { ToolsCard } from './tools-card'
import { fullLook, type LookChange } from './look-fields'

type PhotoLookCardsProps = {
  photoId: PhotoId
  /**
   * A collage sheet's photos when none is picked (owner, 2026-11-07): the cards show the
   * first photo's look and every change gives the whole sheet the same look, one undo step.
   */
  sheet?: PhotoId[]
  /** Bumps on every edit and undo, so the cards follow the photo's look. */
  revision: number
  /** After "Apply to all" changed the shared settings. */
  onAppliedToAll(): void
}

/**
 * The open photo's own look, in the photo and sheet previews only (FR-079): the Faves and
 * Tools cards change this photo only, as its date stamp does (FR-044); *Apply to all*, offered
 * once the photo has its own look, makes it the setting for every photo. A camera RAW first
 * gets how it is finished (FR-072, `RawLookCard`).
 */
export function PhotoLookCards({ photoId, sheet, revision, onAppliedToAll }: PhotoLookCardsProps) {
  const { t } = useTranslation()
  const [current, setCurrent] = useState<PhotoLook | null>(null)
  // On a whole sheet: whether its photos' looks differ now, so the note can say one is coming.
  const [differ, setDiffer] = useState(false)
  const sheetKey = sheet?.join(',') ?? ''
  // Local changes so far: an older answer must not put an older look back (STYLE_RULE §4).
  const changes = useRef(0)

  useEffect(() => {
    let live = true
    const asked = changes.current
    const ids = sheetKey ? sheetKey.split(',').map(Number) : [photoId]
    void Promise.all(ids.map((id) => commands.getPhotoLook(id))).then((results) => {
      if (!live || asked !== changes.current) return
      const looks = results.flatMap((r) => (r.status === 'ok' ? [r.data] : []))
      const [first] = looks
      if (first === undefined) return
      setCurrent(first)
      const key = (l: PhotoLook) => JSON.stringify(fullLook(l.look))
      setDiffer(looks.some((l) => key(l) !== key(first)))
    })
    return () => {
      live = false
    }
  }, [photoId, sheetKey, revision])

  if (!current) return null
  const change: LookChange = (edit, first) => {
    const next: LookSettings = edit(fullLook(current.look))
    changes.current += 1
    setCurrent({ ...current, look: next, custom: true })
    setDiffer(false)
    // The first value of a gesture starts an undo step, the rest amend it (FR-023).
    if (sheet) void commands.setSheetLook(sheet, next, !first)
    else void commands.setPhotoLook(photoId, next, !first)
  }
  // Always there, off until this photo has its own look, so the first slider step never
  // grows the card (STYLE_RULE §4: nothing jumps while the user works).
  const applyToAll = (
    <Button
      variant="outline"
      size="sm"
      className="w-full"
      disabled={!current.custom}
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
      {sheet && (
        <PanelNote icon={<Images />}>{t(differ ? 'look.sheetDiffers' : 'look.sheetWhole')}</PanelNote>
      )}
      {current.raw != null && !sheet && (
        <RawLookCard
          value={current.raw}
          onChange={setRaw}
          onApplyToAll={() =>
            current.raw != null &&
            void commands.applyRawLookToAll(current.raw).then((r) => r.status === 'ok' && onAppliedToAll())
          }
        />
      )}
      <LooksCard photoId={photoId} look={current.look} onChange={change} />
      <FavesCard look={current.look} onChange={change} footer={applyToAll} raw={current.raw != null} />
      <ToolsCard look={current.look} onChange={change} />
    </>
  )
}
