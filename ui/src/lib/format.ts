// SPDX-License-Identifier: GPL-3.0-only
import i18n from '@/i18n'

/** A number in the interface language: `18.6` in English, `18,6` in Vietnamese (FR-020). */
export function formatNumber(value: number, maxDigits = 1, minDigits = 0): string {
  return new Intl.NumberFormat(i18n.language, {
    minimumFractionDigits: minDigits,
    maximumFractionDigits: maxDigits,
  }).format(value)
}
