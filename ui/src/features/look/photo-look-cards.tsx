// SPDX-License-Identifier: GPL-3.0-only
// @shown both: in the previews, and Simple mode's panel for the photo shown (owner, 2026-11-07).
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
  /** Simple mode's panel (owner, 2026-11-07): the amount and Faves only; no saved looks, no
   * Tools, no RAW finish. */
  simple?: boolean
  /** Pro mode's photo preview: LUTs belong to it alone (owner, 2026-11-07). */
  pro?: boolean
  /** After "Apply to all" changed the shared settings. */
  onAppliedToAll(): void
}

/**
 * The open photo's own look, in the photo and sheet previews only (FR-079): the Faves and
 * Tools cards change this photo only, as its date stamp does (FR-044); *Apply to all*, offered
 * once the photo has its own look, makes it the setting for every photo. A camera RAW first
 * gets how it is finished (FR-072, `RawLookCard`).
 */
/**
 * The last look read for each photo (or sheet), so cards mounted again draw at once instead of
 * vanishing until Rust answers, which moved every card below them (owner, 2026-11-07). Owned by
 * this module, at most `KEPT` entries (a few KB), the oldest dropped first; only a first frame
 * is drawn from it, Rust's answer always replaces it, so an id another project reuses is right
 * a moment later. Emptied with the webview.
 */
const lastLook = new Map<string, { look: PhotoLook; differ: boolean }>()
const KEPT = 256
const keep = (key: string, value: { look: PhotoLook; differ: boolean }) => {
  lastLook.delete(key)
  lastLook.set(key, value)
  if (lastLook.size > KEPT) {
    const oldest = lastLook.keys().next().value
    if (oldest !== undefined) lastLook.delete(oldest)
  }
}

export function PhotoLookCards({ photoId, sheet, simple = false, pro = false, revision, onAppliedToAll }: PhotoLookCardsProps) {
  const { t } = useTranslation()
  const sheetKey = sheet?.join(',') ?? ''
  const cacheKey = sheetKey || String(photoId)
  const [current, setCurrent] = useState<PhotoLook | null>(lastLook.get(cacheKey)?.look ?? null)
  // On a whole sheet: whether its photos' looks differ now, so the note can say one is coming.
  const [differ, setDiffer] = useState(lastLook.get(cacheKey)?.differ ?? false)
  // Another photo or sheet: its kept look at once, while Rust's answer is on its way.
  const [shownKey, setShownKey] = useState(cacheKey)
  if (shownKey !== cacheKey) {
    setShownKey(cacheKey)
    const kept = lastLook.get(cacheKey)
    if (kept) {
      setCurrent(kept.look)
      setDiffer(kept.differ)
    }
  }
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
      const apart = looks.some((l) => key(l) !== key(first))
      setDiffer(apart)
      keep(sheetKey || String(photoId), { look: first, differ: apart })
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
      {current.raw != null && !sheet && !simple && (
        <RawLookCard
          value={current.raw}
          onChange={setRaw}
          onApplyToAll={() =>
            current.raw != null &&
            void commands.applyRawLookToAll(current.raw).then((r) => r.status === 'ok' && onAppliedToAll())
          }
        />
      )}
      <LooksCard look={current.look} onChange={change} simple={simple} pro={pro} />
      <FavesCard look={current.look} onChange={change} footer={applyToAll} raw={current.raw != null} />
{!simple &&       <ToolsCard look={current.look} onChange={change} />}
    </>
  )
}
