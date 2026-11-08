// SPDX-License-Identifier: GPL-3.0-only
import { Aperture, Camera, FileImage, Minus, OctagonX, Plus, SunMedium } from 'lucide-react'
import { Spinner } from '@/components/forms/spinner'
import { useTranslation } from 'react-i18next'
import { SectionCard } from '@/components/layout/section-card'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { Field, Stack } from '@/components/forms/field'
import { SegmentedToggle } from '@/components/forms/segmented-toggle'
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
      <span className="flex items-center gap-2">
        <Button
          variant="outline"
          size="icon-sm"
          aria-label={t('capture.stepDown', { name: label })}
          data-hint={t('capture.stepDown', { name: label })}
          onClick={() => void capture.step(setting, -1)}
        >
          <Minus />
        </Button>
        <span className="w-16 text-center text-sm tabular-nums">{value}</span>
        <Button
          variant="outline"
          size="icon-sm"
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
        {capture.pending > 0 ? <Spinner data-icon="inline-start" /> : <Camera data-icon="inline-start" />}
        {capture.pending > 0 ? t('capture.shooting', { count: capture.pending }) : t('capture.shoot')}
      </Button>

      {capture.error && (
        <Alert variant="destructive">
          <OctagonX />
          <AlertDescription>{capture.error}</AlertDescription>
        </Alert>
      )}

      <SectionCard icon={<SunMedium />} title={t('capture.exposure')}>
        {/* Rows that each hold a control, 12 px apart (STYLE_RULE §4). */}
        <Stack>
          <StepRow label={t('capture.iso')} value={iso} setting="iso" capture={capture} />
          <StepRow label={t('capture.shutter')} value={shutterText(settings)} setting="shutter" capture={capture} />
          <div className="flex items-center justify-between gap-2">
            <span className="flex items-center gap-1.5 text-xs text-muted-foreground">
              <Aperture className="size-4" /> {t('capture.aperture')}
            </span>
            <span className="w-16 text-center text-sm tabular-nums" data-hint={t('capture.apertureHint')}>
              {apertureText(settings)}
            </span>
          </div>
        </Stack>
      </SectionCard>

      <SectionCard icon={<FileImage />} title={t('capture.format')}>
        <Field hint={t('capture.formatHint')}>
          <SegmentedToggle<CaptureFormat>
            value={settings?.format === 'jpeg' || settings?.format === 'raw' ? settings.format : undefined}
            onChange={(v) => void capture.setFormat(v)}
            ariaLabel={t('capture.format')}
            options={[
              { value: 'jpeg', label: 'JPEG' },
              { value: 'raw', label: 'RAW' },
            ]}
          />
        </Field>
      </SectionCard>
    </>
  )
}
