// SPDX-License-Identifier: GPL-3.0-only
import { ImageUp, X } from 'lucide-react'
import { useState, type ReactNode } from 'react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import type { CameraProfile, LogoKind, PhotoDetails } from '@/ipc/bindings'
import type { FrameConfigApi } from './use-frame-config'

type CameraProfileEditorProps = {
  frame: FrameConfigApi
  details: PhotoDetails | null
}

function formatExif(details: PhotoDetails | null) {
  const f = details?.fNumber
  const e = details?.exposureS
  return {
    lens: details?.lens ?? 'Manual lens (not in EXIF)',
    focal: details?.focalMm ? `${Math.round(details.focalMm)}` : 'mm',
    aperture: f ? `${Math.round(f * 10) / 10}` : 'F',
    shutter: e ? (e >= 1 ? `${e}s` : `1/${Math.round(1 / e)}s`) : '—',
    iso: details?.iso ? `ISO${details.iso}` : '—',
  }
}

/**
 * A text field that commits on blur or Enter, so typing doesn't send IPC per keystroke.
 * Remounted (via `key`) when the stored value changes.
 */
function CommitInput({
  value,
  placeholder,
  numeric,
  onCommit,
}: {
  value: string
  placeholder: string
  numeric?: boolean
  onCommit: (value: string) => void
}) {
  const [draft, setDraft] = useState(value)
  return (
    <Input
      value={draft}
      placeholder={placeholder}
      inputMode={numeric ? 'decimal' : undefined}
      className="h-8 w-full min-w-0 text-sm"
      onChange={(e) => setDraft(e.target.value)}
      onBlur={() => draft !== value && onCommit(draft)}
      onKeyDown={(e) => e.key === 'Enter' && e.currentTarget.blur()}
    />
  )
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
  const view = frame.config?.cameras.find((c) => c.profile.id === details?.cameraId)
  if (!details || !view) {
    return <p className="text-xs text-muted-foreground">No camera information in this photo.</p>
  }
  const p = view.profile
  const exif = formatExif(details)
  const save = (patch: Partial<CameraProfile>) => void frame.setProfile({ ...p, ...patch })
  const k = (v: unknown) => `${p.id}:${String(v)}`

  return (
    <div className="space-y-2.5">
      <Row label="Camera name">
        <CommitInput
          key={k(p.displayName)}
          value={p.displayName ?? ''}
          placeholder={view.defaultDisplayName ?? 'Camera'}
          onCommit={(v) => save({ displayName: text(v) })}
        />
      </Row>
      <Row label="Lens">
        <CommitInput
          key={k(p.lens)}
          value={p.lens ?? ''}
          placeholder={exif.lens}
          onCommit={(v) => save({ lens: text(v) })}
        />
      </Row>
      <div className="grid grid-cols-2 gap-2">
        <Row label="Focal (mm)">
          <CommitInput
            key={k(p.focalMm)}
            numeric
            value={p.focalMm ? String(p.focalMm) : ''}
            placeholder={exif.focal}
            onCommit={(v) => save({ focalMm: number(v) })}
          />
        </Row>
        <Row label="Aperture F">
          <CommitInput
            key={k(p.fNumber)}
            numeric
            value={p.fNumber ? String(p.fNumber) : ''}
            placeholder={exif.aperture}
            onCommit={(v) => save({ fNumber: number(v) })}
          />
        </Row>
      </div>
      <Row label="Photo style">
        <CommitInput
          key={k(p.photoStyle)}
          value={p.photoStyle ?? ''}
          placeholder="e.g. L. Monochrome D"
          onCommit={(v) => save({ photoStyle: text(v) })}
        />
      </Row>
      <Row label="Logo">
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
            <ToggleGroupItem value="text" className="text-xs">
              Text
            </ToggleGroupItem>
            <ToggleGroupItem value="image" className="text-xs">
              Image
            </ToggleGroupItem>
            <ToggleGroupItem value="none" className="text-xs">
              None
            </ToggleGroupItem>
          </ToggleGroup>
          {view.hasLogoImage ? (
            <Button
              variant="ghost"
              size="icon-xs"
              aria-label="Remove logo image"
              onClick={() => void frame.clearLogo(p.id)}
            >
              <X />
            </Button>
          ) : (
            <Button
              variant="ghost"
              size="icon-xs"
              aria-label="Choose logo PNG"
              onClick={() => void frame.pickLogo(p.id)}
            >
              <ImageUp />
            </Button>
          )}
        </div>
      </Row>
      {p.logo === 'text' && (
        <Row label="Brand text">
          <CommitInput
            key={k(p.brandText)}
            value={p.brandText ?? ''}
            placeholder={view.defaultBrand ?? 'BRAND'}
            onCommit={(v) => save({ brandText: text(v) })}
          />
        </Row>
      )}
      <p className="text-xs text-muted-foreground">
        From EXIF: {exif.shutter} · {exif.iso}. Leave a field empty to use EXIF. Saved for every
        photo from this camera.
      </p>
      {frame.error && <p className="text-xs text-destructive">{frame.error}</p>}
    </div>
  )
}
