// SPDX-License-Identifier: GPL-3.0-only
import i18n from '@/i18n'

/**
 * An IPC error in the interface language, looked up by its stable code (SECURITY.md §8).
 * Codes without a translation fall back to the English message from Rust.
 */
export function errorText(error: { code: string; message: string }): string {
  const key = `errors.${error.code}`
  return i18n.exists(key) ? i18n.t(key as never) : error.message
}
