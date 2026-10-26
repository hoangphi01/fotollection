// SPDX-License-Identifier: GPL-3.0-only
import type { CameraSettings } from '@/ipc/bindings'

/** 1/250, or 2.5″ for a speed of a second or longer. */
export function shutterText(settings: CameraSettings | null): string {
  const s = settings?.shutter
  if (!s) return '—'
  if (s.numerator === 1) return `1/${s.denominator}`
  const seconds = s.numerator / s.denominator
  return `${Number.isInteger(seconds) ? seconds : seconds.toFixed(1)}″`
}

/** f/2.0 from the camera's f-number × 100. */
export function apertureText(settings: CameraSettings | null): string {
  return settings?.aperture ? `f/${(settings.aperture / 100).toFixed(1)}` : '—'
}

/** The ISO value, `auto` when the camera chooses it. */
export function isoText(settings: CameraSettings | null, auto: string): string {
  if (settings?.isoAuto) return auto
  return settings?.iso ? String(settings.iso) : '—'
}
