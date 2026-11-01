// SPDX-License-Identifier: GPL-3.0-only
import { commands, type SheetInfo, type SheetPair } from '@/ipc/bindings'
import { useLiveQuery } from '@/features/settings/use-ipc-query'

const load = async (subject: string) => {
  const pair = JSON.parse(subject) as SheetPair
  const result = await commands.sheetInfo(pair)
  return result.status === 'ok' ? result.data : null
}

/**
 * A sheet's size, each photo's area, the suggested size and its quality (FR-035, FR-026).
 * The last answer stays while a new one loads, so the notes built on it never blink.
 */
export function useSheetInfo(sheet: SheetPair | undefined, revision: number): SheetInfo | null {
  const subject = sheet ? JSON.stringify({ first: sheet.first, second: sheet.second ?? null }) : null
  return useLiveQuery(subject, revision, load).data
}
