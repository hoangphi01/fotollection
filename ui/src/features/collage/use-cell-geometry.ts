// SPDX-License-Identifier: GPL-3.0-only
import { commands, type PhotoId, type SheetPair } from '@/ipc/bindings'
import { useLiveQuery } from '@/features/settings/use-ipc-query'

const load = async (subject: string) => {
  const { photo, first, second } = JSON.parse(subject) as { photo: PhotoId; first: PhotoId; second: PhotoId | null }
  const result = await commands.getCropGeometry(photo, { first, second })
  return result.status === 'ok' ? result.data : null
}

/**
 * The crop editor's geometry for a photo of a sheet, while it is open (FR-035, FR-006):
 * only this photo's, never another's, and kept while an edit asks for it again.
 */
export function useCellGeometry(photo: PhotoId | null, sheet: SheetPair | undefined, revision: number) {
  const subject =
    photo === null || !sheet ? null : JSON.stringify({ photo, first: sheet.first, second: sheet.second ?? null })
  return useLiveQuery(subject, revision, load).current
}
