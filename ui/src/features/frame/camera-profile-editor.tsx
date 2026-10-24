// SPDX-License-Identifier: GPL-3.0-only
import { ImageUp, X } from 'lucide-react'
import { type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { CommitInput } from '@/components/forms/commit-input'
import { Button } from '@/components/ui/button'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
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

function Row({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="flex min-w-0 flex-col gap-1">
      <span className="text-xs text-muted-foreground">{label}</span>
      {children}
    </label>
  )
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
    return <p className="text-xs text-muted-foreground">{t('camera.noInfo')}</p>
  }
  const p = view.profile
  const exif = formatExif(details, t('camera.manualLens'))
  const save = (patch: Partial<CameraProfile>) => void frame.setProfile({ ...p, ...patch })
  const k = (v: unknown) => `${p.id}:${String(v)}`

  return (
    <div className="space-y-2.5">
      <Row label={t('camera.name')}>
        <CommitInput
          key={k(p.displayName)}
          value={p.displayName ?? ''}
          placeholder={view.defaultDisplayName ?? t('camera.namePlaceholder')}
          onCommit={(v) => save({ displayName: text(v) })}
        />
      </Row>
      <Row label={t('camera.lens')}>
        <CommitInput
          key={k(p.lens)}
          value={p.lens ?? ''}
          placeholder={exif.lens}
          onCommit={(v) => save({ lens: text(v) })}
        />
      </Row>
      <div className="grid grid-cols-2 gap-2">
        <Row label={t('camera.focal')}>
          <CommitInput
            key={k(p.focalMm)}
            numeric
            value={p.focalMm ? String(p.focalMm) : ''}
            placeholder={exif.focal}
            onCommit={(v) => save({ focalMm: number(v) })}
          />
        </Row>
        <Row label={t('camera.aperture')}>
          <CommitInput
            key={k(p.fNumber)}
            numeric
            value={p.fNumber ? String(p.fNumber) : ''}
            placeholder={exif.aperture}
            onCommit={(v) => save({ fNumber: number(v) })}
          />
        </Row>
      </div>
      <Row label={t('camera.photoStyle')}>
        <CommitInput
          key={k(p.photoStyle)}
          value={p.photoStyle ?? ''}
          placeholder={t('camera.stylePlaceholder')}
          onCommit={(v) => save({ photoStyle: text(v) })}
        />
      </Row>
      <Row label={t('camera.logo')}>
        <div className="flex items-center gap-1.5">
          <ToggleGroup
            type="single"
            variant="outline"
            size="sm"
            value={p.logo}
            onValueChange={(logo) => {
              if (!logo) return
              if (logo === 'image' && !view.hasLogoImage) void frame.pickLogo(p.id)
              else save({ logo: logo as LogoKind })
            }}
          >
            {view.hasBrandLogo && (
              <ToggleGroupItem value="brand" className="text-xs">
                {t('camera.logoBrand')}
              </ToggleGroupItem>
            )}
            <ToggleGroupItem value="text" className="text-xs">
              {t('camera.logoText')}
            </ToggleGroupItem>
            <ToggleGroupItem value="image" className="text-xs">
              {t('camera.logoImage')}
            </ToggleGroupItem>
            <ToggleGroupItem value="none" className="text-xs">
              {t('camera.logoNone')}
            </ToggleGroupItem>
          </ToggleGroup>
          {view.hasLogoImage ? (
            <Button
              variant="ghost"
              size="icon-xs"
              aria-label={t('camera.removeLogo')}
              onClick={() => void frame.clearLogo(p.id)}
            >
              <X />
            </Button>
          ) : (
            <Button
              variant="ghost"
              size="icon-xs"
              aria-label={t('camera.chooseLogo')}
              onClick={() => void frame.pickLogo(p.id)}
            >
              <ImageUp />
            </Button>
          )}
        </div>
      </Row>
      {p.logo === 'text' && (
        <Row label={t('camera.brandText')}>
          <CommitInput
            key={k(p.brandText)}
            value={p.brandText ?? ''}
            placeholder={view.defaultBrand ?? t('camera.brandPlaceholder')}
            onCommit={(v) => save({ brandText: text(v) })}
          />
        </Row>
      )}
      <p className="text-xs text-muted-foreground">
        {t('camera.fromExif', { shutter: exif.shutter, iso: exif.iso })}
      </p>
      {frame.error && <p className="text-xs text-destructive">{frame.error}</p>}
    </div>
  )
}
