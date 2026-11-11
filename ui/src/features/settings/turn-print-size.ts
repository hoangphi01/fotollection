// SPDX-License-Identifier: GPL-3.0-only
import { commands, type PhotoId } from '@/ipc/bindings'

/** What a photo prints as (FR-120): its crop geometry, upright or flat and whether it turns. */
export const loadPrintGeometry = async (subject: string) => {
  const result = await commands.getCropGeometry(Number(subject), null)
  return result.status === 'ok' ? result.data : null
}

/** `O` (FR-120): the photo's print size laid the other way, where it can turn; one path for
 * the preview and Simple mode, asking Rust what it prints as now. */
export async function turnPrintSize(photoId: PhotoId) {
  const geometry = await loadPrintGeometry(String(photoId))
  if (geometry?.canTurn) await commands.setPrintOrientation([photoId], !geometry.printLandscape)
}
