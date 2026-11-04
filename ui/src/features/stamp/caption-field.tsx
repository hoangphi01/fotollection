// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useId, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { CommitInput } from '@/components/forms/commit-input'
import { Field, FieldLabel } from '@/components/forms/field'
import { Switch } from '@/components/ui/switch'
import { commands, type PhotoId } from '@/ipc/bindings'

type CaptionFieldProps = {
  /** The stamp's line, for every photo. */
  caption: string
  onCommit: (caption: string) => void
  /** The photos on screen together (a collage sheet): they may each have their own line. */
  photos?: { id: PhotoId; name: string }[]
  /** Bumps on every edit and undo, so each photo's own line follows. */
  revision?: number
}

/**
 * The line under the date (FR-036). With two photos on screen, a switch keeps one line for
 * both or gives each its own; the rest of their stamp still follows the settings.
 */
export function CaptionField({ caption, onCommit, photos = [], revision = 0 }: CaptionFieldProps) {
  const { t } = useTranslation()
  const id = useId()
  const ids = photos.map((p) => p.id)
  const key = ids.join(':')
  // Each photo's own line, for the photos it was read for.
  const [owned, setOwned] = useState<{ key: string; lines: (string | null)[] }>({ key: '', lines: [] })
  const own = owned.key === key ? owned.lines : []
  // Local changes so far: an answer asked for before the latest one is dropped (STYLE_RULE §4).
  const changes = useRef(0)
  // The switch turned off on these photos before either has its own line.
  const [apartFor, setApartFor] = useState<string | null>(null)

  // Keyed on the ids' text, not the array a caller rebuilds on every render (LESSONS).
  useEffect(() => {
    if (key === '') return
    let live = true
    const list = key.split(':').map(Number)
    const asked = changes.current
    void Promise.all(list.map((photo) => commands.getPhotoStamp(photo))).then((results) => {
      if (live && asked === changes.current)
        setOwned({ key, lines: results.map((r) => (r.status === 'ok' ? r.data.caption : null)) })
    })
    return () => {
      live = false
    }
  }, [key, revision])

  const anyOwn = own.some((c) => c !== null)
  const apart = photos.length > 1 && (apartFor === key || anyOwn)
  const setSame = (same: boolean) => {
    if (!same) return setApartFor(key)
    setApartFor(null)
    if (anyOwn) {
      changes.current += 1
      setOwned({ key, lines: ids.map(() => null) }) // optimistic; the next revision confirms it
      void commands.setPhotoCaption(ids, null)
    }
  }
  const setPhoto = (index: number, photo: PhotoId, line: string) => {
    changes.current += 1
    setOwned({ key, lines: ids.map((_, i) => (i === index ? line : (own[i] ?? null))) })
    void commands.setPhotoCaption([photo], line)
  }

  return (
    <Field hint={t('settings.captionHint')}>
      <div className="flex items-center justify-between gap-2">
        <FieldLabel>{t('settings.caption')}</FieldLabel>
        {photos.length > 1 && (
          <div className="flex items-center gap-2">
            <FieldLabel htmlFor={id}>{t('settings.captionSame')}</FieldLabel>
            <Switch id={id} checked={!apart} onCheckedChange={setSame} />
          </div>
        )}
      </div>
      {apart ? (
        photos.map((p, i) => (
          <Field key={p.id} label={p.name}>
            <CommitInput
              value={own[i] ?? caption}
              placeholder={t('settings.captionPlaceholder')}
              maxLength={80}
              liveMs={300}
              ariaLabel={t('settings.captionFor', { name: p.name })}
              onCommit={(line) => setPhoto(i, p.id, line)}
            />
          </Field>
        ))
      ) : (
        <CommitInput
          value={caption}
          placeholder={t('settings.captionPlaceholder')}
          maxLength={80}
          liveMs={300}
          ariaLabel={t('settings.caption')}
          onCommit={onCommit}
        />
      )}
    </Field>
  )
}
