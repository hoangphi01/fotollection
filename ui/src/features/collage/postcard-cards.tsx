// SPDX-License-Identifier: GPL-3.0-only
// @shown both: in the gallery panels and in the preview (STYLE_RULE §4, where each card shows, checked by check-lego).
import type { CullFilter, PhotoId, RenderSettings } from '@/ipc/bindings'
import { PostcardCard } from './postcard-card'
import { PostcardRecipientCard } from './postcard-recipient-card'
import { PostcardSenderCard } from './postcard-sender-card'
import { PostcardTextCard } from './postcard-text-card'

type PostcardCardsProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
  photo: PhotoId | null | undefined
  revision: number
  filter?: CullFilter
  ticked?: readonly PhotoId[]
}

/**
 * The postcard's cards in their one order (FR-136, owner 2026-11-15: "three separate lego
 * blocks"), the same in the Collage panel and in a postcard's preview: its shape, then the
 * sender (folded), the recipient and the message.
 */
export function PostcardCards({ settings, onChange, photo, revision, filter, ticked }: PostcardCardsProps) {
  return (
    <>
      <PostcardCard settings={settings} onChange={onChange} />
      <PostcardSenderCard settings={settings} onChange={onChange} />
      <PostcardRecipientCard settings={settings} onChange={onChange} photo={photo} revision={revision} filter={filter} ticked={ticked} />
      <PostcardTextCard settings={settings} onChange={onChange} photo={photo} revision={revision} filter={filter} ticked={ticked} />
    </>
  )
}
