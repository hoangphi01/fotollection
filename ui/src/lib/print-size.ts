// SPDX-License-Identifier: GPL-3.0-only
import type { LengthUnit } from '@/ipc/bindings'

export const MM_PER_IN = 25.4

/** `10 × 15 cm`, or `4 × 6 in` for sizes entered in inches (FR-043). */
export function sizeLabel(shortMm: number, longMm: number, unit: LengthUnit = 'cm'): string {
  const f = (mm: number) =>
    unit === 'in' ? `${Math.round((mm / MM_PER_IN) * 10) / 10}` : `${Math.round(mm) / 10}`
  return `${f(shortMm)} × ${f(longMm)} ${unit}`
}
