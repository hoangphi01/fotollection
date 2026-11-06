// SPDX-License-Identifier: GPL-3.0-only
import { useTranslation } from 'react-i18next'
import { Filmstrip } from '@/components/layout/filmstrip'
import type { CameraShot } from '@/ipc/bindings'
import { thumbUrl } from '@/lib/photo-url'

type ShotStripProps = {
  shots: CameraShot[]
  thumbVersions: Record<number, number>
  generation: number
  onOpen: (shot: CameraShot) => void
}

/** The shots of this session, newest first, as a filmstrip under the live view. */
export function ShotStrip({ shots, thumbVersions, generation, onOpen }: ShotStripProps) {
  const { t } = useTranslation()
  return (
    <Filmstrip
      items={shots}
      label={t('capture.shots', { count: shots.length })}
      empty={t('capture.noShots')}
      hint={() => t('capture.openShot')}
      onPick={onOpen}
      cell={(shot) => (
        <img
          src={thumbUrl(shot.id, thumbVersions[shot.id] ?? 0, generation)}
          alt=""
          draggable={false}
          className="size-full object-cover"
        />
      )}
    />
  )
}
