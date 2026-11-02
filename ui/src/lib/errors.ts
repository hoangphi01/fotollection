// SPDX-License-Identifier: GPL-3.0-only
import i18n from '@/i18n'

/**
 * An IPC error in the interface language, looked up by its stable code (SECURITY.md §8),
 * followed by its permanent `ERR-` code (docs/ERROR_CONTROL.md): the clue a user gives in
 * a report, as the app sends nothing. Codes without a translation fall back to the English
 * message from Rust.
 */
export function errorText(error: { code: string; message: string; err?: string }): string {
  const key = `errors.${error.code}`
  const text = i18n.exists(key) ? i18n.t(key as never) : error.message
  return error.err ? `${text} (${error.err})` : text
}
