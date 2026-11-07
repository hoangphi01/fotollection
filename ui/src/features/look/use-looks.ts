// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useRef, useState } from 'react'
import i18n from '@/i18n'
import { commands, type LookEntry, type PhotoId } from '@/ipc/bindings'

/**
 * The library of looks (FR-079): the built-in ones, then this machine's saved ones. The last
 * list stays while a new one loads, so the grid never empties (STYLE_RULE §4).
 */
export function useLooks() {
  const [looks, setLooks] = useState<LookEntry[]>([])
  const reload = useCallback(() => {
    void commands.listLooks().then((r) => r.status === 'ok' && setLooks(r.data))
  }, [])
  useEffect(reload, [reload])
  return { looks, setLooks, reload }
}

/** A look's name as shown: a built-in one in the interface's language, a user's as typed. */
export function lookName(entry: LookEntry): string {
  if (entry.kind !== 'builtIn') return entry.name
  const key = `look.builtin.${entry.id.slice('builtin:'.length)}`
  return i18n.exists(key) ? i18n.t(key as never) : entry.name
}

/** The looks a key picks: 0 the first (Original), 1–9 the next nine. */
export const LOOK_KEYS = 10

export type LookPicker = {
  looks: LookEntry[]
  /** The photo's look in the list, or `null` when it came from none. */
  current: number | null
  /** Gives the photo the look at `index`; nothing when there is none. */
  pick(index: number): void
}

/**
 * The photo's look among the library's (FR-079), for Simple mode's strip and its number keys:
 * which one it has, and a pick that gives it another.
 */
export function useLookPicker(photoId: PhotoId | undefined, revision: number): LookPicker {
  const { looks } = useLooks()
  const [base, setBase] = useState<string | null>(null)
  const picks = useRef(0)
  useEffect(() => {
    if (photoId === undefined) return
    let live = true
    const asked = picks.current
    void commands.getPhotoLook(photoId).then((r) => {
      if (live && r.status === 'ok' && asked === picks.current) setBase(r.data.look.base ?? null)
    })
    return () => {
      live = false
    }
  }, [photoId, revision])
  const at = looks.findIndex((l) => l.id === base)
  return {
    looks,
    current: at < 0 ? null : at,
    pick: (index) => {
      const entry = looks[index]
      if (photoId === undefined || entry === undefined) return
      picks.current += 1
      setBase(entry.id)
      void commands.setPhotoLook(photoId, { ...entry.look, name: lookName(entry) }, false)
    },
  }
}
