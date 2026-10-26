// SPDX-License-Identifier: GPL-3.0-only
import i18n from '@/i18n'

/** A number in the interface language: `18.6` in English, `18,6` in Vietnamese (FR-020). */
export function formatNumber(value: number, maxDigits = 1, minDigits = 0): string {
  return new Intl.NumberFormat(i18n.language, {
    minimumFractionDigits: minDigits,
    maximumFractionDigits: maxDigits,
  }).format(value)
}

/** A file size: `1.4 MB` from one megabyte up, `850 KB` below. */
export function formatBytes(bytes: number): string {
  return bytes >= 1e6 ? `${formatNumber(bytes / 1e6, 1, 1)} MB` : `${Math.round(bytes / 1e3)} KB`
}

/** A capture time from Rust (`YYYY-MM-DDTHH:MM:SS`) as `YYYY-MM-DD HH:MM`, for the grid. */
export function formatCapture(capture: string): string {
  return capture.replace('T', ' ').slice(0, 16)
}
