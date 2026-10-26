// SPDX-License-Identifier: GPL-3.0-only
import { useTranslation } from 'react-i18next'
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
    <div className="flex h-28 shrink-0 items-center gap-3 border-t border-white/10 bg-neutral-950 px-4">
      <span className="w-16 shrink-0 text-xs text-white/60 tabular-nums">{t('capture.shots', { count: shots.length })}</span>
      <div className="flex min-w-0 flex-1 gap-2 overflow-x-auto py-2">
        {shots.length === 0 ? (
          <p className="text-xs text-white/40">{t('capture.noShots')}</p>
        ) : (
          shots.map((shot) => (
            <button
              key={shot.id}
              type="button"
              aria-label={t('capture.openShot')}
              data-hint={t('capture.openShot')}
              onClick={() => onOpen(shot)}
              className="shrink-0 overflow-hidden rounded-sm ring-offset-neutral-950 transition hover:ring-2 hover:ring-primary focus-visible:ring-2 focus-visible:ring-primary focus-visible:outline-none"
            >
              <img
                src={thumbUrl(shot.id, thumbVersions[shot.id] ?? 0, generation)}
                alt=""
                draggable={false}
                className="aspect-[3/2] h-20 bg-neutral-800 object-cover"
              />
            </button>
          ))
        )}
      </div>
    </div>
  )
}
