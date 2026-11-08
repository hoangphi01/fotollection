// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useRef, useState, useSyncExternalStore } from 'react'
import i18n from '@/i18n'
import { commands, type LookEntry, type PhotoId } from '@/ipc/bindings'

/**
 * The library of looks (FR-079): the built-in ones, then this machine's saved ones. One list
 * for the whole app, so a look saved, imported, renamed or removed in one card shows at once
 * in every other (the strips, the Looks card, *Presets & LUTs*, the gallery's Colour tab;
 * owner, 2026-11-08: "saved, but not on the strip"). The last list stays while a new one
 * loads, also across mounts, so the grid never empties and the cards never jump.
 */
let lastLooks: LookEntry[] = []
let asked = false
const listeners = new Set<() => void>()

function publish(next: LookEntry[]) {
  lastLooks = next
  listeners.forEach((listener) => listener())
}

function reloadLooks() {
  void commands.listLooks().then((r) => r.status === 'ok' && publish(r.data))
}

export function useLooks() {
  const looks = useSyncExternalStore(
    (listener) => {
      listeners.add(listener)
      return () => listeners.delete(listener)
    },
    () => lastLooks,
  )
  // Asked once per run; every change after that publishes the new list itself.
  useEffect(() => {
    if (asked) return
    asked = true
    reloadLooks()
  }, [])
  return { looks, setLooks: publish, reload: reloadLooks }
}

/** A look's name as shown: a built-in one in the interface's language, a user's as typed. */
export function lookName(entry: LookEntry): string {
  if (entry.kind !== 'builtIn') return entry.name
  const key = `look.builtin.${entry.id.slice('builtin:'.length)}`
  return i18n.exists(key) ? i18n.t(key as never) : entry.name
}

/** The looks a key picks: 0–9 the first ten of the strip, your own looks first. */
export const LOOK_KEYS = 10

export type LookPicker = {
  looks: LookEntry[]
  /** The photo's look in the list, or `null` when it came from none. */
  current: number | null
  /** Gives the photo the look at `index`; nothing when there is none. */
  pick(index: number): void
  /** Gives the photo (or the whole sheet) `entry`'s look: one just imported or saved. */
  apply(entry: LookEntry): void
  /** Lists the library again, after an import or a save. */
  reload(): void
  /** Forgets a look of the user's (`user:<id>`), the photos that use it unchanged. */
  remove(id: string): void
}

/**
 * The photo's look among the library's (FR-079), for the strips of looks (Simple mode, with
 * its number keys; a collage sheet, its photo or the whole sheet `sheet`):
 * which one it has, and a pick that gives it another.
 */
export function useLookPicker(
  photoId: PhotoId | undefined,
  revision: number,
  sheet?: PhotoId[],
  /** LUTs and presets are Pro mode's only (owner, 2026-11-07): elsewhere they are not listed. */
  luts = false,
): LookPicker {
  const library = useLooks()
  const { setLooks, reload } = library
  // Your own looks first, then Fotollection's (owner, 2026-11-08); the strips and Simple
  // mode's keys follow this order.
  const listed = luts ? library.looks : library.looks.filter((e) => !e.look.lut)
  const looks = [...listed.filter((e) => e.kind === 'user'), ...listed.filter((e) => e.kind !== 'user')]
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
  const apply = (entry: LookEntry) => {
    if (photoId === undefined) return
    picks.current += 1
    setBase(entry.id)
    const look = { ...entry.look, name: lookName(entry) }
    // A whole collage sheet when none of its photos is picked (owner, 2026-11-07).
    if (sheet) void commands.setSheetLook(sheet, look, false)
    else void commands.setPhotoLook(photoId, look, false)
  }
  return {
    looks,
    current: at < 0 ? null : at,
    pick: (index) => {
      const entry = looks[index]
      if (entry !== undefined) apply(entry)
    },
    apply,
    reload,
    remove: (id) => void commands.deleteLook(id).then((r) => r.status === 'ok' && setLooks(r.data)),
  }
}
