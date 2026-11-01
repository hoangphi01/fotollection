// SPDX-License-Identifier: GPL-3.0-only
import { Camera, LoaderCircle } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { CAMERA_AVAILABLE, CAMERA_OFFERED, type CaptureApi } from './use-capture'

type ConnectCameraButtonProps = {
  capture: CaptureApi
}

/**
 * Connect camera (FR-063), next to Open folder; disabled with a hint where it is not ready,
 * and absent on Linux, which has no camera capture (ADR-0034).
 */
export function ConnectCameraButton({ capture }: ConnectCameraButtonProps) {
  const { t } = useTranslation()
  const connecting = capture.phase === 'connecting'
  if (!CAMERA_OFFERED) return null
  return (
    // The hint sits on a wrapper: a disabled button gets no pointer events.
    <span data-hint={t(CAMERA_AVAILABLE ? 'capture.connectHint' : 'capture.windowsOnly')}>
      <Button
        variant="ghost"
        size="icon-sm"
        aria-label={t('capture.connect')}
        disabled={!CAMERA_AVAILABLE || connecting || capture.phase === 'connected'}
        onClick={() => void capture.connect()}
      >
        {connecting ? <LoaderCircle className="animate-spin" /> : <Camera />}
      </Button>
    </span>
  )
}
