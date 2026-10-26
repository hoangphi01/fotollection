// SPDX-License-Identifier: GPL-3.0-only
import { Aperture, FileImage, Gauge, LoaderCircle, Timer } from 'lucide-react'
import { useRef, useState, type KeyboardEvent } from 'react'
import { useTranslation } from 'react-i18next'
import { ViewerFrame } from '@/components/layout/viewer-frame'
import { useIpcQuery } from '@/features/settings/use-ipc-query'
import { commands, type CameraShot, type RenderSettings } from '@/ipc/bindings'
import { ownsArrowKeys } from '@/lib/keyboard'
import { liveViewUrl } from '@/lib/photo-url'
import { CameraControls } from './camera-controls'
import { apertureText, isoText, shutterText } from './settings-text'
import { ShotStrip } from './shot-strip'
import type { CaptureApi } from './use-capture'

type CaptureDialogProps = {
  capture: CaptureApi
  settings: RenderSettings | null
  thumbVersions: Record<number, number>
  generation: number
  /** Closed: the camera is let go, and the app shows this session's tab. */
  onClose: () => void
  /** A shot in the column was clicked: close and open it in the preview. */
  onOpenShot: (shot: CameraShot) => void
}

/** Long side over short side of the print size chosen, or null for Original. */
function usePrintRatio(settings: RenderSettings | null): number | null {
  const builtIn = useIpcQuery(commands.listPrintSizes)
  const id = settings?.printSize
  const size = [...(builtIn ?? []), ...(settings?.customSizes ?? [])].find((s) => s.id === id)
  return size && size.shortMm > 0 ? size.longMm / size.shortMm : null
}

/**
 * The capture screen (FR-062, FR-063): the preview's frame, with the camera's live view on
 * the stage and the print ratio drawn over it, the shots of this session in a column, and
 * the camera's controls where the preview has its inspector.
 */
export function CaptureDialog({ capture, settings, thumbVersions, generation, onClose, onOpenShot }: CaptureDialogProps) {
  const { t } = useTranslation()
  const contentRef = useRef<HTMLDivElement>(null)
  const ratio = usePrintRatio(settings)
  // The live view's own shape, from the frames themselves (3:2 on the a6400).
  const [frameRatio, setFrameRatio] = useState<number | null>(null)
  // Drawn only where the print cuts something off: the same shape needs no frame.
  const overlay = ratio !== null && frameRatio !== null && Math.abs(ratio - frameRatio) > 0.01 ? ratio : null

  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key !== ' ' || ownsArrowKeys(event.target)) return
    // Space is the shutter here, also over a focused button (no double press).
    event.preventDefault()
    void capture.capture()
  }

  return (
    <ViewerFrame
      open={capture.phase === 'connected'}
      onClose={onClose}
      contentRef={contentRef}
      onKeyDown={onKeyDown}
      title={capture.name}
      description={t('capture.sessionNote')}
      stage={
        <>
        {capture.frame > 0 ? (
          <div className="relative flex max-h-full max-w-full items-center justify-center overflow-hidden">
            <img
              src={liveViewUrl(capture.frame)}
              alt={t('capture.liveView')}
              draggable={false}
              onLoad={(e) => {
                const { naturalWidth: w, naturalHeight: h } = e.currentTarget
                if (w > 0 && h > 0) setFrameRatio(Math.max(w, h) / Math.min(w, h))
              }}
              className="max-h-full max-w-full rounded-sm object-contain shadow-2xl"
            />
            {/* The print's shape over the live view: what stays after Crop to fill. */}
            {overlay && (
              <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
                <div
                  className="h-full max-w-full shadow-[0_0_0_9999px_rgb(0_0_0/0.45)] ring-1 ring-white/70"
                  style={{ aspectRatio: overlay }}
                />
              </div>
            )}
          </div>
        ) : (
          <div className="flex flex-col items-center gap-2 text-white/60">
            <LoaderCircle className="size-7 animate-spin" />
            <span className="text-xs">{t('capture.waitingLiveView')}</span>
          </div>
        )}
        {/* What the camera is set to, at a glance: the same pill as the preview's counter. */}
        <span className="absolute bottom-4 left-1/2 flex -translate-x-1/2 items-center gap-3 rounded-full bg-black/60 px-4 py-1.5 text-xs whitespace-nowrap text-white/85 tabular-nums">
          <span className="flex items-center gap-1.5" data-hint={t('capture.iso')}>
            <Gauge className="size-3.5" /> {t('capture.iso')} {isoText(capture.settings, t('capture.auto'))}
          </span>
          <span className="flex items-center gap-1.5" data-hint={t('capture.shutter')}>
            <Timer className="size-3.5" /> {shutterText(capture.settings)}
          </span>
          <span className="flex items-center gap-1.5" data-hint={t('capture.aperture')}>
            <Aperture className="size-3.5" /> {apertureText(capture.settings)}
          </span>
          <span className="flex items-center gap-1.5" data-hint={t('capture.format')}>
            <FileImage className="size-3.5" /> {capture.settings?.format === 'raw' ? 'RAW' : capture.settings?.format === 'jpeg' ? 'JPEG' : '—'}
          </span>
        </span>
        </>
      }
      below={
        <ShotStrip shots={capture.shots} thumbVersions={thumbVersions} generation={generation} onOpen={onOpenShot} />
      }
      footer={
        <>
          <kbd className="font-mono">Space</kbd> {t('capture.shootKey')} · <kbd className="font-mono">Esc</kbd>{' '}
          {t('capture.close')}
        </>
      }
    >
      <CameraControls capture={capture} />
    </ViewerFrame>
  )
}
