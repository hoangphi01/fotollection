// SPDX-License-Identifier: GPL-3.0-only
import type { LengthUnit, RenderSettings } from '@/ipc/bindings'
import { formatNumber } from './format'

export const MM_PER_IN = 25.4

/** `10 × 15 cm`, or `4 × 6 in` for sizes entered in inches (FR-043). */
export function sizeLabel(shortMm: number, longMm: number, unit: LengthUnit = 'cm'): string {
  const f = (mm: number) => formatNumber(unit === 'in' ? mm / MM_PER_IN : mm / 10, 1)
  return `${f(shortMm)} × ${f(longMm)} ${unit}`
}

/** The unit lengths are shown in: inches when the chosen custom size was entered in them. */
export function displayUnit(settings: RenderSettings | null): LengthUnit {
  return settings?.customSizes?.find((c) => c.id === settings.printSize)?.unit ?? 'cm'
}

/** A length in `unit`, without the unit: `6` or `2.4`. */
export function lengthIn(mm: number, unit: LengthUnit): string {
  return formatNumber(unit === 'in' ? mm / MM_PER_IN : mm / 10, 1)
}

/** A length typed in `unit` (comma or dot), in millimetres; NaN when it is not a number. */
export function parseLength(text: string, unit: LengthUnit): number {
  const value = Number.parseFloat(text.replace(',', '.'))
  return unit === 'in' ? value * MM_PER_IN : value * 10
}

/** `6 × 9 cm`: a width and a height in `unit`. */
export function dimsLabel(widthMm: number, heightMm: number, unit: LengthUnit): string {
  return `${lengthIn(widthMm, unit)} × ${lengthIn(heightMm, unit)} ${unit}`
}
