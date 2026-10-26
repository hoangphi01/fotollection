// SPDX-License-Identifier: GPL-3.0-only
import { Aperture, Camera, FileImage, LoaderCircle, Minus, OctagonX, Plus, SunMedium } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { SectionCard } from '@/components/layout/section-card'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import type { CaptureFormat, ExposureSetting } from '@/ipc/bindings'
import { apertureText, isoText, shutterText } from './settings-text'
import type { CaptureApi } from './use-capture'

type CameraControlsProps = {
  capture: CaptureApi
}

type StepRowProps = {
  label: string
  value: string
  setting: ExposureSetting
  capture: CaptureApi
}

/** One exposure value with − and + steps, as the camera's own dial moves it. */
function StepRow({ label, value, setting, capture }: StepRowProps) {
  const { t } = useTranslation()
  return (
    <div className="flex items-center justify-between gap-2">
      <span className="text-xs text-muted-foreground">{label}</span>
      <span className="flex items-center gap-1">
        <Button
          variant="outline"
          size="icon-xs"
          aria-label={t('capture.stepDown', { name: label })}
          data-hint={t('capture.stepDown', { name: label })}
          onClick={() => void capture.step(setting, -1)}
        >
          <Minus />
        </Button>
        <span className="w-16 text-center text-sm tabular-nums">{value}</span>
        <Button
          variant="outline"
          size="icon-xs"
          aria-label={t('capture.stepUp', { name: label })}
          data-hint={t('capture.stepUp', { name: label })}
          onClick={() => void capture.step(setting, 1)}
        >
          <Plus />
        </Button>
      </span>
    </div>
  )
}

/** The capture screen's inspector (FR-063): the camera, exposure, file format and Capture. */
export function CameraControls({ capture }: CameraControlsProps) {
  const { t } = useTranslation()
  const { settings } = capture
  const iso = isoText(settings, t('capture.auto'))
  return (
    <>
      <Button size="lg" className="w-full" onClick={() => void capture.capture()}>
        {capture.pending > 0 ? <LoaderCircle className="animate-spin" data-icon="inline-start" /> : <Camera data-icon="inline-start" />}
        {capture.pending > 0 ? t('capture.shooting', { count: capture.pending }) : t('capture.shoot')}
      </Button>

      {capture.error && (
        <Alert variant="destructive">
          <OctagonX />
          <AlertDescription>{capture.error}</AlertDescription>
        </Alert>
      )}

      <SectionCard icon={<SunMedium />} title={t('capture.exposure')}>
        <div className="space-y-2">
          <StepRow label={t('capture.iso')} value={iso} setting="iso" capture={capture} />
          <StepRow label={t('capture.shutter')} value={shutterText(settings)} setting="shutter" capture={capture} />
          <div className="flex items-center justify-between gap-2">
            <span className="flex items-center gap-1.5 text-xs text-muted-foreground">
              <Aperture className="size-3.5" /> {t('capture.aperture')}
            </span>
            <span className="w-16 text-center text-sm tabular-nums" data-hint={t('capture.apertureHint')}>
              {apertureText(settings)}
            </span>
          </div>
        </div>
      </SectionCard>

      <SectionCard icon={<FileImage />} title={t('capture.format')}>
        <div className="space-y-2">
          <ToggleGroup
            type="single"
            variant="outline"
            size="sm"
            value={settings?.format === 'jpeg' || settings?.format === 'raw' ? settings.format : ''}
            onValueChange={(v) => v && void capture.setFormat(v as CaptureFormat)}
            aria-label={t('capture.format')}
          >
            <ToggleGroupItem value="jpeg" className="px-3 text-xs">
              JPEG
            </ToggleGroupItem>
            <ToggleGroupItem value="raw" className="px-3 text-xs">
              RAW
            </ToggleGroupItem>
          </ToggleGroup>
          <p className="text-xs text-muted-foreground">{t('capture.formatHint')}</p>
        </div>
      </SectionCard>
    </>
  )
}
