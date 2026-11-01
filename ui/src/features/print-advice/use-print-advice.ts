// SPDX-License-Identifier: GPL-3.0-only
import { commands, type PrintAdvice, type RenderSettings } from '@/ipc/bindings'
import { useLiveQuery } from '@/features/settings/use-ipc-query'

const load = async (subject: string) => {
  const result = await commands.printAdvice(Number(subject))
  return result.status === 'ok' ? result.data : null
}

/**
 * How large the photo prints well, and a verdict per print size (FR-026). Asked again when
 * the photo, the custom sizes or the 3 mm border (FR-067) change; the last answer stays
 * while the next loads, so the card never shrinks under the pointer.
 */
export function usePrintAdvice(photoId: number | undefined, settings: RenderSettings | null): PrintAdvice | null {
  const sizes = (settings?.customSizes ?? []).map((c) => `${c.id}:${c.shortMm}x${c.longMm}`).join()
  const sizesKey = `${sizes}:${settings?.border ?? false}`
  return useLiveQuery(photoId === undefined ? null : String(photoId), sizesKey, load).data
}
