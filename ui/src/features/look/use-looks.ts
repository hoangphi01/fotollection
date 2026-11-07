// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useState } from 'react'
import i18n from '@/i18n'
import { commands, type LookEntry } from '@/ipc/bindings'

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
