// SPDX-License-Identifier: GPL-3.0-only
import { commands, type Fit, type RenderSettings } from '@/ipc/bindings'
import { useLiveQuery } from '@/features/settings/use-ipc-query'

const load = async () => {
  const result = await commands.postcardCuts()
  return result.status === 'ok' ? result.data : null
}

/** Postcards whose text does not all fit, by photo, with the parts that are cut. */
export type PostcardCuts = ReadonlyMap<number, Fit>

/**
 * The postcards whose message or address does not all fit its lines (FR-073), by photo,
 * worked out in Rust as the print sets them. Asked again on every edit and whenever how
 * the cards are set changes (font, size, lines, the way they are held).
 */
export function usePostcardCuts(settings: RenderSettings | null, revision: number): PostcardCuts {
  const postcard = settings?.collage?.postcard
  const subject = postcard?.enabled ? JSON.stringify([postcard, settings?.printSize]) : null
  const cuts = useLiveQuery(subject, revision, load).data
  return new Map((cuts ?? []).map((cut) => [cut.photo, cut.fit]))
}
