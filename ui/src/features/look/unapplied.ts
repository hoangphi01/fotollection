// SPDX-License-Identifier: GPL-3.0-only
import type { TFunction } from 'i18next'

/**
 * A preset's settings no look applies (FR-089), as Rust names them: `"<Setting>: <reason>"`,
 * kept in English with the look. The setting before the colon names a short line in the
 * user's language; one the app does not know yet shows as Rust wrote it.
 */
type Kind =
  | 'clarity'
  | 'texture'
  | 'highlights'
  | 'sharpening'
  | 'noise'
  | 'lens'
  | 'parametric'
  | 'whiteBalance'
  | 'vignettePaint'
  | 'oldProcess'

const KINDS: Record<string, Kind> = {
  Clarity2012: 'clarity',
  Texture: 'texture',
  Highlights2012: 'highlights',
  Sharpness: 'sharpening',
  LuminanceSmoothing: 'noise',
  ColorNoiseReduction: 'noise',
  LensProfileEnable: 'lens',
  ParametricShadows: 'parametric',
  ParametricDarks: 'parametric',
  ParametricLights: 'parametric',
  ParametricHighlights: 'parametric',
  'Temperature and Tint': 'whiteBalance',
  PostCropVignetteStyle: 'vignettePaint',
  ProcessVersion: 'oldProcess',
}

const kindOf = (line: string): Kind | null => KINDS[line.split(':')[0] ?? ''] ?? null

/** Each line in the user's words, once each, with how many presets had it (`counts`). */
export function unappliedLines(t: TFunction, lines: { line: string; count: number }[]): string[] {
  const seen = new Map<string, number>()
  for (const { line, count } of lines) {
    const kind = kindOf(line)
    const text: string = kind ? t(`look.unapplied.${kind}`) : line
    seen.set(text, Math.max(seen.get(text) ?? 0, count))
  }
  return [...seen].map(([text, count]) => (count > 1 ? `${text} (${count})` : text))
}
