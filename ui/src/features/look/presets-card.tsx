// SPDX-License-Identifier: GPL-3.0-only
// @shown preview: in the preview only (STYLE_RULE §4, where each card shows, checked by check-lego).
import { Layers } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { SectionCard } from '@/components/layout/section-card'
import type { PhotoId } from '@/ipc/bindings'
import { useLookLibrary } from './look-library'
import { useLookPicker } from './use-looks'

type PresetsCardProps = {
  photoId: PhotoId
  revision: number
  /** A sheet's photos, when none is picked: a look goes to both (the sheet's preview, FR-137). */
  sheet?: PhotoId[]
}

/**
 * *Presets & LUTs*, as Lightroom's Presets panel on the left of its Develop view (owner,
 * 2026-11-07, 2026-11-08): every look of one's own, saved looks, LUTs and Lightroom presets,
 * hovered on the photo, a click gives it to this photo; the list is `useLookLibrary`, shared
 * with the gallery's Colour tab (FR-114). The photo's preview in Pro mode, and a sheet's in
 * Collage mode (owner, 2026-11-16: "import looks there as in the gallery").
 */
export function PresetsCard({ photoId, revision, sheet }: PresetsCardProps) {
  const { t } = useTranslation()
  const picker = useLookPicker(photoId, revision, sheet, true)
  const chosen = picker.current === null ? undefined : picker.looks[picker.current]
  const library = useLookLibrary({
    looks: picker.looks,
    reload: picker.reload,
    remove: picker.remove,
    chosenId: chosen?.id,
    onPick: picker.apply,
    photoId,
    hover: true,
  })
  return (
    <SectionCard icon={<Layers />} title={t('look.presets')} action={library.action}>
      {library.content}
    </SectionCard>
  )
}
