// SPDX-License-Identifier: GPL-3.0-only
import { ImageUp, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { CommitInput } from '@/components/forms/commit-input'
import { Button } from '@/components/ui/button'
import { Field, Hint } from '@/components/forms/field'
import { SegmentedToggle } from '@/components/forms/segmented-toggle'
import type { CameraProfile, LogoKind, PhotoDetails } from '@/ipc/bindings'
import type { FrameConfigApi } from './use-frame-config'

type CameraProfileEditorProps = {
  frame: FrameConfigApi
  details: PhotoDetails | null
}

function formatExif(details: PhotoDetails | null, manualLens: string) {
  const f = details?.fNumber
  const e = details?.exposureS
  return {
    lens: details?.lens ?? manualLens,
    focal: details?.focalMm ? `${Math.round(details.focalMm)}` : 'mm',
    aperture: f ? `${Math.round(f * 10) / 10}` : 'F',
    shutter: e ? (e >= 1 ? `${e}s` : `1/${Math.round(1 / e)}s`) : '—',
    iso: details?.iso ? `ISO${details.iso}` : '—',
  }
}

const text = (s: string) => (s.trim() === '' ? null : s.trim())
const number = (s: string) => {
  const n = Number.parseFloat(s.replace(',', '.').replace(/^f\/?/i, ''))
  return Number.isFinite(n) && n > 0 ? n : null
}

/** Per-camera overrides for the info frame, remembered across sessions (FR-040). */
export function CameraProfileEditor({ frame, details }: CameraProfileEditorProps) {
  const { t } = useTranslation()
  const view = frame.config?.cameras.find((c) => c.profile.id === details?.cameraId)
  if (!details || !view) {
    return <Hint>{t('camera.noInfo')}</Hint>
  }
  const p = view.profile
  const exif = formatExif(details, t('camera.manualLens'))
  const save = (patch: Partial<CameraProfile>) => void frame.setProfile(p.id, patch)
  const k = (v: unknown) => `${p.id}:${String(v)}`

  return (
    <>
      <Field label={t('camera.name')}>
        <CommitInput
          ariaLabel={t('camera.name')}
          key={k(p.displayName)}
          value={p.displayName ?? ''}
          placeholder={view.defaultDisplayName ?? t('camera.namePlaceholder')}
          onCommit={(v) => save({ displayName: text(v) })}
        />
      </Field>
      <Field label={t('camera.lens')}>
        <CommitInput
          ariaLabel={t('camera.lens')}
          key={k(p.lens)}
          value={p.lens ?? ''}
          placeholder={exif.lens}
          onCommit={(v) => save({ lens: text(v) })}
        />
      </Field>
      <div className="grid grid-cols-2 gap-2">
        <Field label={t('camera.focal')}>
          <CommitInput
          ariaLabel={t('camera.focal')}
            key={k(p.focalMm)}
            numeric
            value={p.focalMm ? String(p.focalMm) : ''}
            placeholder={exif.focal}
            onCommit={(v) => save({ focalMm: number(v) })}
          />
        </Field>
        <Field label={t('camera.aperture')}>
          <CommitInput
          ariaLabel={t('camera.aperture')}
            key={k(p.fNumber)}
            numeric
            value={p.fNumber ? String(p.fNumber) : ''}
            placeholder={exif.aperture}
            onCommit={(v) => save({ fNumber: number(v) })}
          />
        </Field>
      </div>
      <Field label={t('camera.photoStyle')}>
        <CommitInput
          ariaLabel={t('camera.photoStyle')}
          key={k(p.photoStyle)}
          value={p.photoStyle ?? ''}
          placeholder={t('camera.stylePlaceholder')}
          onCommit={(v) => save({ photoStyle: text(v) })}
        />
      </Field>
      <Field label={t('camera.logo')}>
        <div className="flex items-center gap-1.5">
          <SegmentedToggle<LogoKind>
            dense
            value={p.logo}
            options={[
              ...(view.hasBrandLogo ? [{ value: 'brand' as const, label: t('camera.logoBrand') }] : []),
              { value: 'text', label: t('camera.logoText') },
              { value: 'image', label: t('camera.logoImage') },
              { value: 'none', label: t('camera.logoNone') },
            ]}
            onChange={(logo) => {
              if (logo === 'image' && !view.hasLogoImage) void frame.pickLogo(p.id)
              else save({ logo })
            }}
            ariaLabel={t('camera.logo')}
          />
          {view.hasLogoImage ? (
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label={t('camera.removeLogo')}
              data-hint={t('camera.removeLogo')}
              onClick={() => void frame.clearLogo(p.id)}
            >
              <X />
            </Button>
          ) : (
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label={t('camera.chooseLogo')}
              data-hint={t('camera.chooseLogo')}
              onClick={() => void frame.pickLogo(p.id)}
            >
              <ImageUp />
            </Button>
          )}
        </div>
      </Field>
      {p.logo === 'text' && (
        <Field label={t('camera.brandText')}>
          <CommitInput
          ariaLabel={t('camera.brandText')}
            key={k(p.brandText)}
            value={p.brandText ?? ''}
            placeholder={view.defaultBrand ?? t('camera.brandPlaceholder')}
            onCommit={(v) => save({ brandText: text(v) })}
          />
        </Field>
      )}
      <Hint>{t('camera.fromExif', { shutter: exif.shutter, iso: exif.iso })}</Hint>
      {frame.error && <Hint className="text-destructive">{frame.error}</Hint>}
    </>
  )
}
